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
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_sendmsg,
        libc::SYS_recvmsg,
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
        // fd 交付通道所需 syscall 不得残留在黑名单。
        for needed in [
            libc::SYS_bind,
            libc::SYS_listen,
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
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
        let test_name = "seccomp_linux::tests::gpu_filter_execution_allows_unix_and_blocks_inet";
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
        let status = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .status()
            .expect("spawn isolated seccomp probe child");
        assert!(
            status.success(),
            "GPU seccomp 过滤器执行行为不符预期（exit={:?}）",
            status.code()
        );
    }
}
