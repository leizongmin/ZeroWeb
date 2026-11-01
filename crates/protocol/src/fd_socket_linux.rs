//! Linux Unix socket 辅助通道：经 SCM_RIGHTS 传递 dma-buf fd（RFC 4.3 真纹理零拷贝）。
//!
//! 对端信任模型（renderer-trust-boundary M3）：socket 落在共享的 `/dev/shm`，
//! 防线为三层——① socket 文件权限收 owner-only（0600）；② accept/connect 双侧
//! `SO_PEERCRED` 校验对端 euid（Linux；macOS 无 SO_PEERCRED，交由 ① 与 ③）；
//! ③ socket 名含每帧发布的 128-bit 随机 token，token 经既有 IPC 描述符送达
//! 消费端（wire 不变），同机其他进程无法在发布窗口内猜中路径。

use std::io::{IoSlice, IoSliceMut};
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::ProtocolError;

const FD_SOCKET_PREFIX: &str = "zeroweb-fd-";

/// fd 辅助 socket 路径。
pub fn fd_socket_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/dev/shm/{FD_SOCKET_PREFIX}{name}"))
}

/// fd socket 名：surface/frame 前缀 + 每次发布重新生成的随机 token。
/// 名字经 `CompositorFrameDelivery.fd_socket_name`（既有 IPC 描述符）送达消费端。
pub fn fd_socket_name(surface_id: u64, frame_id: u64) -> String {
    format!("{surface_id}-{frame_id}-{}", random_token())
}

/// 128-bit 随机 hex token（`/dev/urandom`；std-only 不引 rand 依赖）。
/// urandom 不可读时退化为 pid+纳秒组合——仍远优于可预测的 surface-frame 名。
fn random_token() -> String {
    use std::io::Read;
    let mut bytes = [0u8; 16];
    let ok = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok();
    if !ok {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mixed = (nanos as u64) ^ ((std::process::id() as u64) << 32);
        bytes[..8].copy_from_slice(&mixed.to_le_bytes());
        bytes[8..].copy_from_slice(&(mixed.wrapping_mul(0x9E3779B97F4A7C15)).to_le_bytes());
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 对端是否同 euid（Linux `SO_PEERCRED`；macOS 无对应结构，恒 true——
/// 跨用户面由 socket 文件 0600 与不可猜 token 保障）。
fn peer_same_user(stream: &UnixStream) -> bool {
    #[cfg(target_os = "linux")]
    {
        let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
        let mut len = size_of::<libc::ucred>() as libc::socklen_t;
        let rc = unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut cred as *mut libc::ucred).cast(),
                &mut len,
            )
        };
        rc == 0 && cred.uid == unsafe { libc::geteuid() }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = stream;
        true
    }
}

/// compositor 侧：bind 并等待首个同用户连接，经 SCM_RIGHTS 发送 `fd`。
///
/// **所有权契约（R3340）**：`fd` 的所有权转入本函数。`SCM_RIGHTS` 会在内核中
/// 为接收方复制一份新 fd，但**发送方的本地 fd 不会被内核关闭**——必须由本函数显式关闭。
/// 因此 `fd` 经 `OwnedFd` RAII 包裹，确保**成功路径与所有错误路径**都被关闭（否则
/// 每帧 dma-buf 发布泄漏一个 fd，最终耗尽 compositor 进程 fd 上限）。详见
/// `fd_socket_publish_closes_sender_fd_on_success` 测试。
pub fn publish_fd(name: &str, fd: RawFd, accept_timeout: Duration) -> Result<(), ProtocolError> {
    // SAFETY: 调用方把 fd 所有权转入本函数；OwnedFd 保证所有路径都关闭它。
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    let path = fd_socket_path(name);
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    let listener =
        UnixListener::bind(&path).map_err(|e| ProtocolError::Channel(format!("fd socket bind 失败: {e}")))?;
    // bind 默认 0777&~umask；显式收紧 owner-only，挡共享 /dev/shm 上的跨用户连接
    // （set_permissions 失败不致命：Linux 侧仍有 SO_PEERCRED euid 校验兜底）。
    let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    listener
        .set_nonblocking(true)
        .map_err(|e| ProtocolError::Channel(format!("fd socket nonblocking 失败: {e}")))?;

    let deadline = std::time::Instant::now() + accept_timeout;
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if peer_same_user(&stream) {
                    break stream;
                }
                // 非本用户连接：丢弃并继续等合法消费端，不让陌生连接挤占交付窗口。
                if std::time::Instant::now() >= deadline {
                    let _ = std::fs::remove_file(&path);
                    return Err(ProtocolError::Channel("fd socket accept 超时".into()));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    let _ = std::fs::remove_file(&path);
                    return Err(ProtocolError::Channel("fd socket accept 超时".into()));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(error) => {
                let _ = std::fs::remove_file(&path);
                return Err(ProtocolError::Channel(format!("fd socket accept 失败: {error}")));
            }
        }
    };
    // SCM_RIGHTS 内核复制 fd 给接收方；发送方本地副本由 `owned` 在函数返回时关闭。
    send_fd(&stream, owned.as_raw_fd())?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// Browser 侧：connect 并 recv fd；对端（accept 侧）须为同 euid 进程。
pub fn consume_fd(name: &str, connect_timeout: Duration) -> Result<OwnedFd, ProtocolError> {
    let path = fd_socket_path(name);
    let deadline = std::time::Instant::now() + connect_timeout;
    let stream = loop {
        match UnixStream::connect(&path) {
            Ok(stream) => break stream,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    || error.kind() == std::io::ErrorKind::ConnectionRefused =>
            {
                if std::time::Instant::now() >= deadline {
                    return Err(ProtocolError::Channel(format!("fd socket connect 超时: {path:?}")));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(ProtocolError::Channel(format!("fd socket connect 失败: {error}"))),
        }
    };
    // 连接到的监听者若非本 euid（同用户抢位），拒绝收 fd——失败语义明确优于
    // 静默接受陌生进程交付的 fd。
    if !peer_same_user(&stream) {
        return Err(ProtocolError::Channel("fd socket 对端用户不符，拒绝接收".into()));
    }
    recv_fd(&stream)
}

fn send_fd(stream: &UnixStream, fd: RawFd) -> Result<(), ProtocolError> {
    let data = [1u8];
    let iov = [IoSlice::new(&data)];
    let fds = [fd];
    sendmsg_fds(stream.as_raw_fd(), &iov, &fds)
}

fn recv_fd(stream: &UnixStream) -> Result<OwnedFd, ProtocolError> {
    let mut data = [0u8; 1];
    let mut iov = [IoSliceMut::new(&mut data)];
    let fd = recvmsg_fd(stream.as_raw_fd(), &mut iov)?;
    // SAFETY: SCM_RIGHTS 返回的新 fd 由本模块取得所有权。
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn sendmsg_fds(socket: RawFd, iov: &[IoSlice<'_>], fds: &[RawFd]) -> Result<(), ProtocolError> {
    let fd_size = size_of::<RawFd>();
    let cmsg_len = unsafe { libc::CMSG_SPACE(fd_size as u32) as usize };
    let mut cmsg = vec![0u8; cmsg_len];
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = iov.as_ptr() as *mut libc::iovec;
    msg.msg_iovlen = iov.len() as _;
    msg.msg_control = cmsg.as_mut_ptr() as *mut _;
    msg.msg_controllen = cmsg.len() as _;

    unsafe {
        let cptr = libc::CMSG_FIRSTHDR(&msg);
        if cptr.is_null() {
            return Err(ProtocolError::Channel("CMSG_FIRSTHDR 失败".into()));
        }
        (*cptr).cmsg_level = libc::SOL_SOCKET;
        (*cptr).cmsg_type = libc::SCM_RIGHTS;
        (*cptr).cmsg_len = libc::CMSG_LEN(fd_size as u32) as _;
        std::ptr::copy_nonoverlapping(fds.as_ptr(), libc::CMSG_DATA(cptr) as *mut RawFd, fds.len());
    }

    let sent = unsafe { libc::sendmsg(socket, &msg, 0) };
    if sent < 0 {
        return Err(ProtocolError::Channel(format!(
            "sendmsg 失败: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

fn recvmsg_fd(socket: RawFd, iov: &mut [IoSliceMut<'_>]) -> Result<RawFd, ProtocolError> {
    let fd_size = size_of::<RawFd>();
    let cmsg_len = unsafe { libc::CMSG_SPACE(fd_size as u32) as usize };
    let mut cmsg = vec![0u8; cmsg_len];
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = iov.as_mut_ptr() as *mut libc::iovec;
    msg.msg_iovlen = iov.len() as _;
    msg.msg_control = cmsg.as_mut_ptr() as *mut _;
    msg.msg_controllen = cmsg.len() as _;

    let received = unsafe { libc::recvmsg(socket, &mut msg, 0) };
    if received < 0 {
        return Err(ProtocolError::Channel(format!(
            "recvmsg 失败: {}",
            std::io::Error::last_os_error()
        )));
    }

    unsafe {
        let mut cptr = libc::CMSG_FIRSTHDR(&msg);
        while !cptr.is_null() {
            if (*cptr).cmsg_level == libc::SOL_SOCKET && (*cptr).cmsg_type == libc::SCM_RIGHTS {
                let data = libc::CMSG_DATA(cptr) as *const RawFd;
                return Ok(*data);
            }
            cptr = libc::CMSG_NXTHDR(&msg, cptr);
        }
    }
    Err(ProtocolError::Channel("SCM_RIGHTS 未携带 fd".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::thread;

    const FD_COUNT_TEST_CHILD: &str = "ZERO_PROTOCOL_FD_COUNT_TEST_CHILD";

    fn run_fd_count_test_in_isolated_process(test_name: &str) -> bool {
        if std::env::var(FD_COUNT_TEST_CHILD).as_deref() == Ok(test_name) {
            return false;
        }

        let status = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(FD_COUNT_TEST_CHILD, test_name)
            .status()
            .expect("spawn isolated fd count test");
        assert!(status.success(), "隔离 fd 计数测试失败: {status}");
        true
    }

    /// M3 切片 2：socket 名必须含每次发布重新生成的随机 token——同机其他进程
    /// 无法在发布窗口内猜中路径（跨 tab 像素面收口）。同 surface/frame 两次
    /// 生成必须不同，且 token 为 32 位 hex 后缀。
    #[test]
    fn fd_socket_name_carries_per_publish_token() {
        let first = fd_socket_name(7, 9);
        let second = fd_socket_name(7, 9);
        assert_ne!(first, second, "同 surface/frame 的两次发布 socket 名不应相同");
        let token = first.rsplit('-').next().expect("token 段");
        assert_eq!(token.len(), 32, "token 应为 128-bit hex：{first}");
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()), "token 应为 hex：{token}");
    }

    /// M3 切片 2：发布窗口内 socket 文件必须为 owner-only（0600），挡共享
    /// /dev/shm 上其他用户在 accept 前的连接面。
    #[test]
    fn fd_socket_publish_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;
        let name = format!("mode-{}", fd_socket_name(1, std::process::id() as u64));
        let memfd = unsafe { libc::memfd_create(c"zeroweb-fd-mode".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(memfd >= 0, "memfd_create: {}", std::io::Error::last_os_error());

        let name_clone = name.clone();
        let handle = thread::spawn(move || publish_fd(&name_clone, memfd, Duration::from_secs(5)));
        // 等发布端完成 bind（accept 轮询期最长 5s，窗口内路径存在）。
        let path = fd_socket_path(&name);
        let mut checked = false;
        for _ in 0..100 {
            if let Ok(metadata) = std::fs::metadata(&path) {
                assert_eq!(
                    metadata.permissions().mode() & 0o777,
                    0o600,
                    "fd socket 文件应 owner-only：{:o}",
                    metadata.permissions().mode()
                );
                checked = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(checked, "5s 内未观察到 fd socket 路径");
        let received = consume_fd(&name, Duration::from_secs(2)).expect("consume fd");
        handle.join().expect("publish join").expect("publish fd");
        assert!(received.as_raw_fd() >= 0);
    }

    #[test]
    fn fd_socket_round_trip_memfd() {
        let name = format!("test-{}", std::process::id());
        let memfd = unsafe { libc::memfd_create(c"zeroweb-fd-test".as_ptr(), libc::MFD_CLOEXEC) };
        assert!(memfd >= 0, "memfd_create: {}", std::io::Error::last_os_error());

        let name_clone = name.clone();
        // memfd 所有权转入 publish_fd；R3340 起成功路径会关闭它，不再在此 close。
        let handle = thread::spawn(move || publish_fd(&name_clone, memfd, Duration::from_secs(2)));

        thread::sleep(Duration::from_millis(20));
        let received = consume_fd(&name, Duration::from_secs(2)).expect("consume fd");
        handle.join().expect("publish join").expect("publish fd");
        assert!(received.as_raw_fd() >= 0);
    }

    /// 统计本进程当前打开的 fd 数（`/proc/self/fd` 目录项数）。
    ///
    /// 用于确定性检测 fd 泄漏——比 `fcntl(F_GETFD)` 更稳健：fd 编号关闭后可能被
    /// 回收复用，使 `fcntl` 假阳性（误判泄漏的 fd 仍有效）；而打开 fd 总数对
    /// 「泄漏 vs 正常关闭」是单调可靠的判据。
    fn count_open_fds() -> usize {
        std::fs::read_dir("/proc/self/fd").map(|d| d.count()).unwrap_or(0)
    }

    /// R3340：成功 SCM_RIGHTS 后，发送方的本地 fd 副本必须被关闭（所有权契约）。
    ///
    /// `SCM_RIGHTS` 在内核中为接收方复制一份 fd，但**不**关闭发送方的副本——
    /// 若 publish_fd 不关闭，每帧 dma-buf 发布都泄漏一个 fd，最终耗尽 compositor 的
    /// fd 上限。本测反复往返 N 次，用打开 fd 总数单调性判定泄漏（修复前每轮 +1）。
    /// 该断言在只运行本用例的子进程中执行，隔离 test harness 的并行 fd 活动。
    #[test]
    fn fd_socket_publish_closes_sender_fd_on_success() {
        let test_name = "fd_socket_linux::tests::fd_socket_publish_closes_sender_fd_on_success";
        if run_fd_count_test_in_isolated_process(test_name) {
            return;
        }

        let baseline = count_open_fds();
        let mut max_growth = 0isize;
        for i in 0..12 {
            let name = format!("r3340-{i}-{}", std::process::id());
            let memfd = unsafe { libc::memfd_create(c"zeroweb-fd-r3340".as_ptr(), libc::MFD_CLOEXEC) };
            assert!(memfd >= 0, "memfd_create: {}", std::io::Error::last_os_error());

            let name_clone = name.clone();
            let handle = thread::spawn(move || publish_fd(&name_clone, memfd, Duration::from_secs(2)));

            thread::sleep(Duration::from_millis(15));
            let received = consume_fd(&name, Duration::from_secs(2)).expect("consume fd");
            handle.join().expect("publish join").expect("publish fd");
            drop(received); // 接收方副本也释放 → 往返应净增 0 个 fd。

            let growth = count_open_fds() as isize - baseline as isize;
            max_growth = max_growth.max(growth);
        }
        // 修复前：每轮泄漏 1 个发送方 fd → max_growth 随迭代线性增长到 ~12。
        // 修复后：发送方 fd 关闭，残留仅测试框架瞬态 fd（偶发非确定性 ±几），给宽裕阈值 4。
        assert!(
            max_growth <= 4,
            "成功路径泄漏 fd：迭代后打开 fd 较基线净增 {max_growth}（应稳定）"
        );
    }

    /// R3340：错误路径（accept 超时——无消费者连接）也必须关闭发送方 fd。
    /// 该断言在只运行本用例的子进程中执行，隔离 test harness 的并行 fd 活动。
    #[test]
    fn fd_socket_publish_closes_sender_fd_on_error() {
        let test_name = "fd_socket_linux::tests::fd_socket_publish_closes_sender_fd_on_error";
        if run_fd_count_test_in_isolated_process(test_name) {
            return;
        }

        let baseline = count_open_fds();
        let mut max_growth = 0isize;
        for i in 0..6 {
            let name = format!("r3340-err-{i}-{}", std::process::id());
            let memfd = unsafe { libc::memfd_create(c"zeroweb-fd-r3340-err".as_ptr(), libc::MFD_CLOEXEC) };
            assert!(memfd >= 0, "memfd_create: {}", std::io::Error::last_os_error());

            // 不启动消费者线程 → publish_fd 的 accept 必然超时（150ms）。
            let result = publish_fd(&name, memfd, Duration::from_millis(150));
            assert!(result.is_err(), "无消费者时应 accept 超时");

            let growth = count_open_fds() as isize - baseline as isize;
            max_growth = max_growth.max(growth);
        }
        // 修复前：错误路径泄漏 1 个 fd/轮 → max_growth ~6。修复后：发送方 fd 关闭。
        assert!(
            max_growth <= 3,
            "错误路径泄漏 fd：迭代后打开 fd 较基线净增 {max_growth}（应稳定）"
        );
    }
}
