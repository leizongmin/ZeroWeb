//! Linux Unix socket 辅助通道：经 SCM_RIGHTS 传递 dma-buf fd（RFC 4.3 真纹理零拷贝）。

use std::io::{IoSlice, IoSliceMut};
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::ProtocolError;

const FD_SOCKET_PREFIX: &str = "zeroweb-fd-";

/// fd 辅助 socket 路径。
pub fn fd_socket_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/dev/shm/{FD_SOCKET_PREFIX}{name}"))
}

/// compositor 侧：bind 并等待单次连接，经 SCM_RIGHTS 发送 `fd`。
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
    listener
        .set_nonblocking(true)
        .map_err(|e| ProtocolError::Channel(format!("fd socket nonblocking 失败: {e}")))?;

    let deadline = std::time::Instant::now() + accept_timeout;
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
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

/// Browser 侧：connect 并 recv fd。
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
    recv_fd(&stream)
}

/// fd socket 名。
pub fn fd_socket_name(surface_id: u64, frame_id: u64) -> String {
    format!("{surface_id}-{frame_id}-fd")
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
