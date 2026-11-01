//! image-decoder 进程沙箱（renderer 信任边界 goal ③A-image-decoder + ④C，2026-11-01）。
//!
//! IO 面查实为纯 stdio 循环（[`crate::run_role`] stdin/stdout 收发字节 → 内存 decode →
//! rgba，无 fs/socket 访问）——档案取最紧形态：
//!
//! - **seccomp**：黑名单阻断网络 socket 族 + exec/fork + ptrace（镜像 compositor 基础
//!   档案 `apps/compositor/src/seccomp_linux.rs`，无 AF_UNIX 域门控——解码器无任何
//!   socket 需求，getsockopt 一并阻断）。
//! - **landlock 空 allowed-set**：ruleset handled 四访问位、零规则即 restrict——生效后
//!   全文件系统读/写/列目录/执行拒绝（已打开 fd 与 exec 期已映射库不受影响；解码器
//!   启动后无 fs 依赖，为空 allowed-set 成立前提）。
//! - **fail-closed（④C 拍板）**：安装失败（含老内核 landlock ENOSYS）返回 `Err`，main
//!   拒绝裸奔运行 exit(1)——隔离模式下 renderer 侧走解码错误面（不内联回落，
//!   `image_decoder.rs` IF-006 订正语义），降级显式可见。老内核宿主的操作出口为
//!   `ZW_IMAGE_DECODER_LANDLOCK=0`（seccomp 仍生效）。
//! - 非 Linux 平台：编译门控跳过（与 compositor 同口径，显式日志）。
//!
//! 时序契约：须在任何子线程之前（本进程单线程，main 首段调用即满足）；landlock
//! `restrict_self` 依赖 no-new-privs，安装函数自带 `PR_SET_NO_NEW_PRIVS`（与 seccomp
//! 先后顺序解耦）。

/// 是否启用 image-decoder 沙箱钩子（主开关；默认开，`0` 禁用——操作员显式豁免面）。
pub fn sandbox_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_IMAGE_DECODER_SANDBOX")
}

/// 是否启用 seccomp 过滤器（Linux；默认开，`0` 禁用）。
pub fn seccomp_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_IMAGE_DECODER_SECCOMP")
}

/// 是否启用 landlock（Linux；默认开，`0` 禁用——老内核宿主降级出口）。
pub fn landlock_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_IMAGE_DECODER_LANDLOCK")
}

/// 启动早期沙箱（env 剥离 + seccomp + landlock 空 allowed-set）。
///
/// 返回 `Err` 时调用方必须 fail-closed（拒绝运行），不得降级为无沙箱继续。
pub fn apply_early_if_enabled() -> Result<(), String> {
    if !sandbox_enabled() {
        tracing::warn!("image-decoder: sandbox 主开关关闭（ZW_IMAGE_DECODER_SANDBOX=0），进程未沙箱运行");
        return Ok(());
    }
    sanitize_env();
    #[cfg(target_os = "linux")]
    {
        if seccomp_enabled() {
            seccomp::install_exec_network_filter().map_err(|error| format!("seccomp: {error}"))?;
            tracing::info!("image-decoder: seccomp 网络/exec 过滤已启用");
        }
        if landlock_enabled() {
            landlock::install_empty_fs_allowlist().map_err(|error| format!("landlock: {error}"))?;
            tracing::info!("image-decoder: landlock 空 allowed-set 已启用（全 fs 拒绝）");
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing::info!("image-decoder: seccomp/landlock 仅 Linux 可用，本平台未沙箱（编译门控跳过）");
    }
    Ok(())
}

fn sanitize_env() {
    for key in [
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
    ] {
        // SAFETY: 沙箱启动阶段剥离注入型 env；仅删除已知键。
        unsafe {
            std::env::remove_var(key);
        }
    }
}

#[cfg(target_os = "linux")]
mod seccomp {
    //! 网络/exec 阻断 seccomp-bpf（黑名单形态，镜像 compositor 基础档案）。

    // BPF 指令集常量与 sock_filter/sock_fprog 为稳定内核 UAPI（Linux
    // Documentation/userspace-api/seccomp_filter.rst）。libc 未给 android
    // target 定义这组类型，本 crate 需在 android（同为 target_os="linux"）
    // 编译，故本地定义、全 Linux 目标共用一套。
    const BPF_LD: u16 = 0x00;
    const BPF_W: u16 = 0x00;
    const BPF_ABS: u16 = 0x20;
    const BPF_JMP: u16 = 0x05;
    const BPF_JEQ: u16 = 0x15;
    const BPF_RET: u16 = 0x06;
    const BPF_K: u16 = 0x00;

    #[repr(C)]
    struct SockFilter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }

    #[repr(C)]
    struct SockFprog {
        len: u16,
        filter: *mut SockFilter,
    }

    const SECCOMP_SET_MODE_FILTER: libc::c_int = 1;
    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
    const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
    const EPERM: u32 = 1;
    const SECCOMP_DATA_ARCH: u32 = 4;

    #[cfg(target_arch = "x86_64")]
    const AUDIT_ARCH: u32 = 0xc000_003e;
    #[cfg(target_arch = "aarch64")]
    const AUDIT_ARCH: u32 = 0xc000_00b7;

    fn bpf_stmt(code: u16, k: u32) -> SockFilter {
        SockFilter { code, jt: 0, jf: 0, k }
    }

    fn bpf_jump(code: u16, k: u32, jt: u8, jf: u8) -> SockFilter {
        SockFilter { code, jt, jf, k }
    }

    fn ret_errno(errno: u32) -> u32 {
        SECCOMP_RET_ERRNO | (errno & 0xffff)
    }

    /// 阻断清单：网络 socket 族 + exec/fork + ptrace。解码器零 socket 需求
    /// （stdio 为 exec 期继承 fd，不经 socket 系），故无域门控、全量阻断。
    pub fn blocked_syscalls() -> Vec<i64> {
        [
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_bind,
            libc::SYS_connect,
            libc::SYS_execve,
            libc::SYS_execveat,
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

    fn build_filter(blocked: &[i64]) -> Vec<SockFilter> {
        let ld_w_abs = BPF_LD | BPF_W | BPF_ABS;
        let jmp_jeq = BPF_JMP | BPF_JEQ | BPF_K;
        let ret_k = BPF_RET | BPF_K;
        let mut insns = Vec::with_capacity(blocked.len() * 2 + 5);
        // 非目标架构直接放行（seccomp_data.arch 过滤，避免异架构 syscall 号误拦）。
        insns.push(bpf_stmt(ld_w_abs, SECCOMP_DATA_ARCH));
        insns.push(bpf_jump(jmp_jeq, AUDIT_ARCH, 1, 0));
        insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
        insns.push(bpf_stmt(ld_w_abs, 0)); // seccomp_data.nr
        for &nr in blocked {
            insns.push(bpf_jump(jmp_jeq, nr as u32, 0, 1));
            insns.push(bpf_stmt(ret_k, ret_errno(EPERM)));
        }
        insns.push(bpf_stmt(ret_k, SECCOMP_RET_ALLOW));
        insns
    }

    /// 安装过滤器；须在任何子线程 spawn 之前调用（附带设置 no-new-privs）。
    pub fn install_exec_network_filter() -> Result<(), String> {
        let blocked = blocked_syscalls();
        if blocked.is_empty() {
            return Err("当前架构无可用 syscall 列表".into());
        }
        // SAFETY: prctl 常量参数，无指针。
        let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if ret != 0 {
            return Err(format!("PR_SET_NO_NEW_PRIVS 失败: {}", std::io::Error::last_os_error()));
        }
        let mut filter = build_filter(&blocked);
        let prog = SockFprog {
            len: filter.len() as u16,
            filter: filter.as_mut_ptr(),
        };
        // SAFETY: prog 指向本函数栈上的合法 SockFprog。
        let ret = unsafe { libc::syscall(libc::SYS_seccomp, SECCOMP_SET_MODE_FILTER, 0u32, &prog) };
        if ret != 0 {
            return Err(format!("seccomp 安装失败: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod landlock {
    //! 空 allowed-set landlock：handled 四访问位 + 零规则 + restrict。

    use std::io;

    // Landlock FS 访问位（ABI v1）——位定义教训见 compositor
    // `landlock_linux.rs` 模块头（READ≠1；add_rule 第 4 参是 flags）。
    const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
    const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
    const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;

    #[repr(C)]
    struct LandlockRulesetAttr {
        handled_access_fs: u64,
    }

    fn create_ruleset(handled_access_fs: u64) -> Result<i32, io::Error> {
        let attr = LandlockRulesetAttr { handled_access_fs };
        // SAFETY: attr 指向本函数栈上合法 ruleset 属性，size 精确。
        let fd = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &attr as *const LandlockRulesetAttr,
                std::mem::size_of::<LandlockRulesetAttr>(),
                0u32,
            )
        };
        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(fd as i32)
        }
    }

    /// 空允许清单安装：生效后全部 fs 访问拒绝。任何安装错误（含老内核
    /// ENOSYS / 受限容器 EPERM）原样上抛——fail-closed 输入面。
    pub fn install_empty_fs_allowlist() -> Result<(), String> {
        install_with_handled(default_handled_access_fs())
    }

    fn default_handled_access_fs() -> u64 {
        LANDLOCK_ACCESS_FS_EXECUTE
            | LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_READ_FILE
            | LANDLOCK_ACCESS_FS_READ_DIR
    }

    /// 能力预探测（测试专用）：create_ruleset 默认 handled 面，区分「内核不
    /// 支持」与「行为失败」——前者测试跳过，后者才当作断言对象。
    #[cfg(test)]
    pub(crate) fn create_ruleset_probe() -> Result<i32, io::Error> {
        create_ruleset(default_handled_access_fs())
    }

    /// `handled_access_fs` 可注入（测试用保留位构造 EINVAL 路径）。
    pub fn install_with_handled(handled_access_fs: u64) -> Result<(), String> {
        // landlock_restrict_self 契约要求 no-new-privs（生产由先行的 seccomp
        // 满足；此处自带设置，与 seccomp 开关解耦）。
        // SAFETY: prctl 常量参数，无指针。
        let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if ret != 0 {
            return Err(format!("PR_SET_NO_NEW_PRIVS 失败: {}", io::Error::last_os_error()));
        }
        let ruleset_fd = create_ruleset(handled_access_fs).map_err(|e| format!("create_ruleset: {e}"))?;
        // 零规则：不 add_rule 直接 restrict——handled 面内全部拒绝。
        // SAFETY: ruleset_fd 为 create_ruleset 返回的合法 fd，flags 保留 0。
        let ret = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd, 0u32) };
        let restrict_err = if ret != 0 {
            Some(io::Error::last_os_error())
        } else {
            None
        };
        // SAFETY: fd 所有权归本函数。
        unsafe {
            libc::close(ruleset_fd);
        }
        restrict_err.map_or(Ok(()), |e| Err(format!("restrict_self: {e}")))
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn filter_blocks_network_and_exec() {
        let blocked = seccomp::blocked_syscalls();
        for nr in [
            libc::SYS_execve,
            libc::SYS_execveat,
            libc::SYS_socket,
            libc::SYS_connect,
        ] {
            assert!(blocked.contains(&nr), "阻断清单缺 {nr}");
        }
    }

    /// 隔离子进程真实安装双沙箱并验证受限行为——BPF/landlock 只有真实执行
    /// 才可信（compositor M3 双缺陷教训）。空 allowed-set 判别式：连 /dev/shm
    /// 写也须被拒（与 compositor 白名单档案的关键差异）；execve 拒绝；stdio
    /// （继承 fd）不受影响。退出码：0=全过；1=安装失败；2=白名单外 fs 未拒；
    /// 3=execve 未拒；4=/dev/shm 写未被拒（空 allowed-set 失效）；5=内核不支
    /// 持 landlock（环境受限，父进程跳过）。
    #[test]
    fn sandbox_execution_denies_fs_exec_keeps_stdio() {
        const CHILD: &str = "ZERO_IMAGE_DECODER_SANDBOX_CHILD";
        let test_name = "sandbox::tests::sandbox_execution_denies_fs_exec_keeps_stdio";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            // 能力预探测：内核不支持 landlock → 环境受限跳过（不当作行为失败）。
            if let Err(error) = landlock::create_ruleset_probe()
                && matches!(error.raw_os_error(), Some(libc::ENOSYS) | Some(libc::EPERM))
            {
                std::process::exit(5);
            }
            let code = match super::apply_early_if_enabled() {
                Err(error) => {
                    eprintln!("install failed: {error}");
                    1
                }
                Ok(()) => {
                    let fs_denied = std::fs::read("/etc/passwd").is_err();
                    if !fs_denied {
                        2
                    } else {
                        // SAFETY: 探测 execve 是否被过滤器拒绝；仅在子探针进程执行。
                        let exec_denied = unsafe {
                            let c = b"/bin/true\0";
                            libc::execve(c.as_ptr() as *const _, std::ptr::null(), std::ptr::null()) < 0
                        };
                        if !exec_denied {
                            3
                        } else {
                            let shm_allowed = std::fs::write("/dev/shm/zw-dec-probe", b"x").is_ok();
                            let _ = std::fs::remove_file("/dev/shm/zw-dec-probe");
                            if shm_allowed {
                                4
                            } else {
                                // stdio 存活面：继承 fd 写入不受空 allowed-set 影响。
                                let stdio_ok = std::io::Write::write_all(&mut std::io::stderr(), b"ok").is_ok();
                                if stdio_ok { 0 } else { 6 }
                            }
                        }
                    }
                }
            };
            std::process::exit(code);
        }
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .expect("spawn isolated sandbox probe child");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行"
        );
        let code = output.status.code();
        assert!(
            matches!(code, Some(0) | Some(5)),
            "沙箱执行行为不符预期（exit={code:?}）stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// fail-closed 输入面：非法 handled 位（保留 bit 62）→ create_ruleset
    /// EINVAL → `install_with_handled` 上抛 Err（main 据此 exit(1) 拒绝运行）。
    /// 退出码：0=Err 如期上抛；1=意外 Ok；5=内核缺 landlock（跳过）。
    #[test]
    fn landlock_invalid_ruleset_surfaces_err() {
        const CHILD: &str = "ZERO_IMAGE_DECODER_LANDLOCK_EINVAL_CHILD";
        let test_name = "sandbox::tests::landlock_invalid_ruleset_surfaces_err";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            if let Err(error) = landlock::create_ruleset_probe()
                && matches!(error.raw_os_error(), Some(libc::ENOSYS) | Some(libc::EPERM))
            {
                std::process::exit(5);
            }
            let code = match landlock::install_with_handled(1 << 62) {
                Err(_) => 0,
                Ok(()) => 1,
            };
            std::process::exit(code);
        }
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .expect("spawn isolated landlock einval probe child");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行"
        );
        let code = output.status.code();
        assert!(
            matches!(code, Some(0) | Some(5)),
            "非法 ruleset 应上抛 Err（exit={code:?}）"
        );
    }
}
