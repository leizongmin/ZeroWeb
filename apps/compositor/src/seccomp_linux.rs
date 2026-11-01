//! Linux seccomp-bpf：阻断网络 socket 与 exec/fork（RFC 4.5-S2）。
//!
//! compositor 仅需 stdio IPC、字体/shm 文件读写与内存分配；不应发起网络或
//! 启动子进程。GPU 模式（wgpu/Vulkan）需额外 syscall，且 dma-buf fd 经
//! `/dev/shm` Unix socket 以 SCM_RIGHTS 交付（fd_socket_linux.rs），故 GPU
//! 变体放行 Unix 域 socket 的建立/监听/收发——`socket` 本身按 domain 门控：
//! 仅 AF_UNIX 放行，inet/inet6/netlink 等仍 EPERM（无网络外联面）。

use std::io;

const SECCOMP_SET_MODE_FILTER: libc::c_int = 1;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
const EPERM: u32 = 1;

const SECCOMP_DATA_NR: u32 = 0;
const SECCOMP_DATA_ARCH: u32 = 4;
/// `seccomp_data.args[0]` 低 32 位偏移（x86_64/aarch64 均小端：nr@0、arch@4、ip@8、args@16）。
const SECCOMP_DATA_ARGS0_LO: u32 = 16;
/// Unix 域 socket family——GPU 模式 fd-passing 通道唯一放行的 `socket` domain。
const AF_UNIX: u32 = libc::AF_UNIX as u32;

#[cfg(target_arch = "x86_64")]
const AUDIT_ARCH: u32 = 0xc000_003e;
#[cfg(target_arch = "aarch64")]
const AUDIT_ARCH: u32 = 0xc000_00b7;

fn bpf_stmt(code: u16, k: u32) -> libc::sock_filter {
    libc::sock_filter { code, jt: 0, jf: 0, k }
}

fn bpf_jump(code: u16, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

fn ret_errno(errno: u32) -> u32 {
    SECCOMP_RET_ERRNO | (errno & 0xffff)
}

fn blocked_syscalls() -> Vec<i64> {
    [
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_bind,
        libc::SYS_connect,
        libc::SYS_execve,
        libc::SYS_execveat,
        // aarch64 无 SYS_fork/SYS_vfork（fork 由 clone 承担），仅 x86 系列定义
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        libc::SYS_fork,
        libc::SYS_getpeername,
        libc::SYS_getsockname,
        libc::SYS_getsockopt,
        libc::SYS_listen,
        libc::SYS_ptrace,
        libc::SYS_recvfrom,
        libc::SYS_recvmmsg,
        libc::SYS_recvmsg,
        libc::SYS_sendmmsg,
        libc::SYS_sendmsg,
        libc::SYS_sendto,
        libc::SYS_setsockopt,
        libc::SYS_socket,
        libc::SYS_socketpair,
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        libc::SYS_vfork,
    ]
    .into_iter()
    .filter(|&nr| nr >= 0)
    .collect()
}

/// 构建过滤器。`unix_socket_gate` 为真时 `SYS_socket` 按调用参数分流：
/// domain 为 AF_UNIX → 放行（dma-buf fd 交付通道）；其余 domain 仍 EPERM。
fn build_filter(blocked: &[i64], unix_socket_gate: bool) -> Vec<libc::sock_filter> {
    let ld_w_abs = (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16;
    let jmp_jeq = (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16;
    let jmp_a = (libc::BPF_JMP | libc::BPF_JA | libc::BPF_K) as u16;
    let ret_k = (libc::BPF_RET | libc::BPF_K) as u16;

    let mut insns = Vec::with_capacity(blocked.len() * 2 + 10);
    insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_ARCH));
    insns.push(bpf_jump(jmp_jeq, AUDIT_ARCH, 1, 0));
    insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
    insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_NR));
    if unix_socket_gate {
        // 域门控 5 条（经典 BPF 仅前向跳转，故非 socket 路径用无条件 jmp 跳过检查）：
        //   jeq nr==socket, jt=1, jf=0 —— socket → 域检查；其余 → 无条件跳过
        //   jmp +3                        —— 跳到后续黑名单匹配
        //   ld args[0]                    —— socket(2) 首参即 domain
        //   jeq AF_UNIX, jt=1, jf=0       —— AF_UNIX → 落入后续匹配（socket 已不在
        //                                   黑名单，直达放行尾）；否则 →
        //   ret EPERM                     —— 非 AF_UNIX socket（inet/inet6/netlink…）阻断
        insns.push(bpf_jump(jmp_jeq, libc::SYS_socket as u32, 1, 0));
        insns.push(bpf_jump(jmp_a, 3, 0, 0));
        insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_ARGS0_LO));
        insns.push(bpf_jump(jmp_jeq, AF_UNIX, 1, 0));
        insns.push(bpf_stmt(ret_k, ret_errno(EPERM)));
    }
    for &nr in blocked {
        insns.push(bpf_jump(jmp_jeq, nr as u32, 0, 1));
        insns.push(bpf_stmt(ret_k, ret_errno(EPERM)));
    }
    insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
    insns
}

/// 安装网络/exec 阻断 seccomp 过滤器；须在任何子线程 spawn 之前调用。
pub fn install_network_exec_filter() -> Result<(), String> {
    install_filter(blocked_syscalls(), false)
}

/// GPU 模式：在阻断网络/exec 的同时放行 wgpu/Vulkan 与 dma-buf fd 交付
/// （`/dev/shm` Unix socket SCM_RIGHTS）所需 syscall。`socket` 仅放行
/// AF_UNIX 域，inet/inet6/netlink 等仍阻断。
pub fn install_network_exec_filter_gpu_aware() -> Result<(), String> {
    install_filter(gpu_allowed_syscalls(), true)
}

/// GPU 模式在黑名单之外放行的 syscall。`socket` 不在此列——它既非无条件放行
/// 也非无条件阻断，而是由 `build_filter` 的 AF_UNIX 域门控处理。
fn gpu_allowed_syscalls() -> Vec<i64> {
    let mut blocked = blocked_syscalls();
    let allow = [
        libc::SYS_ioctl,
        libc::SYS_madvise,
        libc::SYS_eventfd2,
        libc::SYS_memfd_create,
        libc::SYS_getrandom,
        libc::SYS_sched_yield,
        libc::SYS_prctl,
        libc::SYS_clone,
        libc::SYS_clone3,
        libc::SYS_mprotect,
        // dma-buf fd 交付通道（fd_socket_linux.rs：UnixListener bind/accept4 +
        // SCM_RIGHTS sendmsg/recvmsg）。bind/listen/accept 系列的作用面被域门控
        // 限制为 AF_UNIX socket fd——网络域 socket 无法创建，故无网络外联面。
        // getsockopt 供 accept 侧 SO_PEERCRED 同用户校验（peer_same_user）；漏放行
        // 会使校验恒 false、合法消费端连接被当陌生连接丢弃（dma-buf 交付必败）。
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_sendmsg,
        libc::SYS_recvmsg,
        libc::SYS_getsockopt,
    ];
    blocked.retain(|nr| !allow.contains(nr));
    blocked.retain(|nr| *nr != libc::SYS_socket);
    blocked
}

fn install_filter(blocked: Vec<i64>, unix_socket_gate: bool) -> Result<(), String> {
    if blocked.is_empty() {
        return Err("当前架构无可用 seccomp syscall 列表".into());
    }

    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        return Err(format!("PR_SET_NO_NEW_PRIVS 失败: {}", io::Error::last_os_error()));
    }

    let mut filter = build_filter(&blocked, unix_socket_gate);
    let prog = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_mut_ptr(),
    };

    let ret = unsafe { libc::syscall(libc::SYS_seccomp, SECCOMP_SET_MODE_FILTER, 0, &prog) };
    if ret != 0 {
        return Err(format!("seccomp 安装失败: {}", io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_has_allow_tail_and_blocks_execve() {
        let blocked = blocked_syscalls();
        assert!(blocked.contains(&libc::SYS_execve));
        let filter = build_filter(&blocked, false);
        assert!(filter.len() >= 5);
        assert_eq!(filter.last().unwrap().k, SECCOMP_RET_ALLOW);
    }

    #[test]
    fn gpu_filter_keeps_network_blocked_and_gates_socket_domain() {
        let blocked = gpu_allowed_syscalls();
        // fd 交付通道所需 syscall 不得残留在黑名单（getsockopt：accept 侧
        // SO_PEERCRED 同用户校验，2026-11-01 dma-buf e2e 事故正是漏了它）。
        for needed in [
            libc::SYS_bind,
            libc::SYS_listen,
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
            libc::SYS_getsockopt,
        ] {
            assert!(!blocked.contains(&needed), "GPU 黑名单误含 fd-passing syscall {needed}");
        }
        // socket 由域门控处理（不在黑名单）；网络外联与 exec 仍阻断。
        assert!(!blocked.contains(&libc::SYS_socket));
        assert!(blocked.contains(&libc::SYS_execve));
        assert!(blocked.contains(&libc::SYS_connect));

        let filter = build_filter(&blocked, true);
        let ld_w_abs = (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16;
        let jmp_jeq = (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16;
        assert!(
            filter
                .iter()
                .any(|f| f.code == ld_w_abs && f.k == SECCOMP_DATA_ARGS0_LO),
            "缺少 args[0]（domain）装载指令"
        );
        assert!(
            filter
                .iter()
                .any(|f| f.code == jmp_jeq && f.k == AF_UNIX && f.jt == 1 && f.jf == 0),
            "缺少 AF_UNIX 域门控比较"
        );
        assert_eq!(filter.last().unwrap().k, SECCOMP_RET_ALLOW);
    }

    /// 在隔离子进程中真实安装 GPU 过滤器并验证 syscall 行为——BPF 程序只有
    /// 真实执行才可信（结构断言无法发现「黑名单误拦自家 fd 通道」这类回归，
    /// GPU 链路默认开后 dmabuf 交付正是因此失败）。退出码：0=全过；
    /// 1=安装失败；2=AF_UNIX 被拦；3=AF_INET 未被拦。
    #[test]
    fn gpu_filter_execution_allows_unix_and_blocks_inet() {
        const CHILD: &str = "ZERO_COMPOSITOR_SECCOMP_GPU_CHILD";
        // 全路径（含模块前缀）：--exact 对短名零匹配会让子进程空跑 exit 0
        // （M3 实测确证的假绿根因，父进程断言另有 stdout 校验兜底）。
        let test_name = "sandbox::seccomp_linux::tests::gpu_filter_execution_allows_unix_and_blocks_inet";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            let code = match install_network_exec_filter_gpu_aware() {
                Err(_) => 1,
                Ok(()) => {
                    // SAFETY: 子进程内安装 seccomp 后直接探测 syscall 并退出。
                    let unix_ok = unsafe {
                        let fd = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
                        let ok = fd >= 0;
                        if ok {
                            libc::close(fd);
                        }
                        ok
                    };
                    if !unix_ok {
                        2
                    } else {
                        let inet_blocked = unsafe {
                            let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
                            if fd >= 0 {
                                libc::close(fd);
                                false
                            } else {
                                std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
                            }
                        };
                        if inet_blocked { 0 } else { 3 }
                    }
                }
            };
            std::process::exit(code);
        }
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .expect("spawn isolated seccomp probe child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // --exact 名称失配时 libtest 空跑且 exit 0（假绿根因），显式抓住。
        assert!(
            stdout.contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行：stdout={stdout}"
        );
        assert!(
            output.status.success(),
            "GPU seccomp 过滤器执行行为不符预期（exit={:?}）",
            output.status.code()
        );
    }

    /// 在隔离子进程中安装 GPU 过滤器后走真实 fd 通道发布，父进程消费——
    /// 端到端钉住「过滤器不得误拦 fd 通道自身 syscall」。2026-11-01 dma-buf
    /// e2e 事故：黑名单漏放行 getsockopt，accept 侧 SO_PEERCRED 校验恒失败，
    /// 合法消费端连接被当陌生连接丢弃（浏览器侧报 SCM_RIGHTS 未携带 fd，
    /// 通道判死回退 legacy）。结构断言发现不了，只有真实执行才可信。
    /// 子进程退出码：0=发布成功；1=安装失败；2=memfd 失败；3=发布失败。
    #[test]
    fn gpu_filter_execution_permits_fd_channel_publish_and_consume() {
        use std::time::Duration;

        const CHILD: &str = "ZERO_COMPOSITOR_SECCOMP_FD_CHANNEL_CHILD";
        const NAME_KEY: &str = "ZERO_COMPOSITOR_SECCOMP_FD_CHANNEL_NAME";
        // 全路径（含模块前缀）：--exact 对短名零匹配会让子进程空跑 exit 0
        //（同上，父进程断言另有 stdout 校验兜底）。
        let test_name =
            "sandbox::seccomp_linux::tests::gpu_filter_execution_permits_fd_channel_publish_and_consume";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            let name = std::env::var(NAME_KEY).unwrap_or_default();
            let code = match install_network_exec_filter_gpu_aware() {
                Err(_) => 1,
                Ok(()) => {
                    // SAFETY: 测试子进程内创建匿名 memfd、写入探针字节后发布。
                    let fd = unsafe { libc::memfd_create(c"seccomp-fd-probe".as_ptr(), 0) };
                    if fd < 0 {
                        2
                    } else {
                        let probe = [0xA5_u8];
                        unsafe {
                            libc::write(fd, probe.as_ptr().cast(), probe.len());
                        }
                        match zero_protocol::fd_socket_linux::publish_fd(&name, fd, Duration::from_secs(5)) {
                            Ok(()) => 0,
                            Err(error) => {
                                eprintln!("publish_fd: {error}");
                                3
                            }
                        }
                    }
                }
            };
            std::process::exit(code);
        }
        let name = zero_protocol::fd_socket_linux::fd_socket_name(4100, std::process::id() as u64);
        let child = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .env(NAME_KEY, &name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn isolated fd-channel seccomp child");
        let path = zero_protocol::fd_socket_linux::fd_socket_path(&name);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !path.exists() {
            assert!(std::time::Instant::now() < deadline, "子进程未发布 fd socket: {path:?}");
            std::thread::sleep(Duration::from_millis(2));
        }
        let fd = zero_protocol::fd_socket_linux::consume_fd(&name, Duration::from_secs(5))
            .expect("GPU 过滤器下 fd 通道发布/消费必须可达（getsockopt 放行）");
        use std::io::{Read as _, Seek as _};
        let mut file = std::fs::File::from(fd);
        // SCM_RIGHTS 复制的是同一 open file description（与子进程共享偏移，
        // 子进程写后偏移已到 1），读探针前须回带开头。
        file.seek(std::io::SeekFrom::Start(0)).expect("memfd 回带");
        let mut probe = [0_u8; 1];
        file.read_exact(&mut probe).expect("memfd 探针字节可读");
        assert_eq!(probe[0], 0xA5, "消费到的 fd 必须是子进程发布的那个 memfd");
        let output = child.wait_with_output().expect("wait fd-channel child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行：stdout={stdout}"
        );
        assert!(
            output.status.success(),
            "子进程 publish 失败（exit={:?}）",
            output.status.code()
        );
    }

    /// 在隔离子进程中真实安装非 GPU 过滤器并验收受限操作（DC-4/M3：真实
    /// helper 受限操作验收）。非 GPU 变体黑名单含 `socket`——fd 交付通道只在
    /// GPU 链路，故 AF_UNIX/AF_INET socket 创建与进程 spawn 全部须被拦。
    /// 退出码：0=全过；1=安装失败；2=AF_INET 未被拦；3=spawn 未被拦。
    #[test]
    fn filter_execution_blocks_sockets_and_spawn() {
        const CHILD: &str = "ZERO_COMPOSITOR_SECCOMP_PLAIN_CHILD";
        // 全路径（含模块前缀）：--exact 对短名零匹配会让子进程空跑 exit 0
        // （M3 实测确证的假绿根因，父进程断言另有 stdout 校验兜底）。
        let test_name = "sandbox::seccomp_linux::tests::filter_execution_blocks_sockets_and_spawn";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            let code = match install_network_exec_filter() {
                Err(_) => 1,
                Ok(()) => {
                    let inet_blocked = unsafe {
                        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
                        if fd >= 0 {
                            libc::close(fd);
                            false
                        } else {
                            std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
                        }
                    };
                    if !inet_blocked {
                        2
                    } else {
                        // 黑名单含 fork/vfork（std spawn 走 libc::fork）——filter
                        // 下无法派生新进程即安全契约成立；Err 的具体步序不区分。
                        let spawn_blocked = std::process::Command::new("/bin/true").status().is_err();
                        if spawn_blocked { 0 } else { 3 }
                    }
                }
            };
            std::process::exit(code);
        }
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .expect("spawn isolated seccomp probe child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // --exact 名称失配时 libtest 空跑且 exit 0（假绿根因），显式抓住。
        assert!(
            stdout.contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行：stdout={stdout}"
        );
        assert!(
            output.status.success(),
            "非 GPU seccomp 过滤器执行行为不符预期（exit={:?}）",
            output.status.code()
        );
    }
}
