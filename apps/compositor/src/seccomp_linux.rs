//! Linux seccomp-bpf：阻断网络 socket 与 exec/fork（RFC 4.5-S2）。
//!
//! compositor 仅需 stdio IPC、字体/shm 文件读写与内存分配；不应发起网络或
//! 启动子进程。GPU 模式（wgpu/Vulkan）需额外 syscall，与 seccomp 不兼容。

use std::io;

const SECCOMP_SET_MODE_FILTER: libc::c_int = 1;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
const EPERM: u32 = 1;

const SECCOMP_DATA_NR: u32 = 0;
const SECCOMP_DATA_ARCH: u32 = 4;

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

fn build_filter(blocked: &[i64]) -> Vec<libc::sock_filter> {
    let ld_w_abs = (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16;
    let jmp_jeq = (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16;
    let ret_k = (libc::BPF_RET | libc::BPF_K) as u16;

    let mut insns = Vec::with_capacity(blocked.len() * 2 + 4);
    insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_ARCH));
    insns.push(bpf_jump(jmp_jeq, AUDIT_ARCH, 1, 0));
    insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
    insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_NR));
    for &nr in blocked {
        insns.push(bpf_jump(jmp_jeq, nr as u32, 0, 1));
        insns.push(bpf_stmt(ret_k, ret_errno(EPERM)));
    }
    insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
    insns
}

/// 安装网络/exec 阻断 seccomp 过滤器；须在任何子线程 spawn 之前调用。
pub fn install_network_exec_filter() -> Result<(), String> {
    let blocked = blocked_syscalls();
    if blocked.is_empty() {
        return Err("当前架构无可用 seccomp syscall 列表".into());
    }

    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        return Err(format!("PR_SET_NO_NEW_PRIVS 失败: {}", io::Error::last_os_error()));
    }

    let mut filter = build_filter(&blocked);
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
        let filter = build_filter(&blocked);
        assert!(filter.len() >= 5);
        assert_eq!(filter.last().unwrap().k, SECCOMP_RET_ALLOW);
    }
}
