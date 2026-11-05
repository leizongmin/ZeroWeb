//! Renderer 进程沙箱（goal ③A-renderer + ④ renderer fail 语义，2026-11-01）。
//!
//! 前身是 RFC §六 P3 占位（仅 env 剥离）。本轮按 [决策包](../../../docs/goal/archive/renderer-trust-boundary/2026-11-01-decision-package-1234.md)
//! ③A 落地两半：
//!
//! - **seccomp（默认开，fail-closed）**：阻断 `execveat`（memfd + AT_EMPTY_PATH
//!   无文件执行载荷的经典通路）、`fork`/`vfork`、`ptrace`。**`execve` 保留**——
//!   renderer 是 image-decoder / media-host helper 的合法 spawn 方
//!   （`webview::image_decoder` / `webview::media_proxy` 经 `Command` →
//!   posix_spawn → `clone(CLONE_VM|CLONE_VFORK)` + `execve`，不经 fork/vfork
//!   syscall），黑名单不会破坏 helper spawn（执行型测试钉住）。**网络面不动**
//!   （拍板原文；renderer 直连网络为现状，fetch_proxy 出网收窄另立切片）。
//! - **landlock 允许清单（默认开，`ZW_RENDERER_LANDLOCK=0` kill-switch）**：字体
//!   ro、HTTP 磁盘缓存 rw、`/dev/shm` rw（fd 通道 + 后续 media C2 帧路径）、DNS/NSS
//!   与 JS `Date` 时区所需的 `/etc` 只读、动态库目录 ro+EXECUTE（dlopen/NSS 老
//!   系统）、helper 二进制所在目录 EXECUTE（execve 路径级收窄）。默认值按 ①-a 拍板
//!   翻开（2026-11-03 决策，2026-11-04 交付）：允许清单不含任意路径读，file:// 的
//!   **模块体动态 import** 面由 renderer worker 本地直读（`js_worker` 的
//!   `__zw_compile_module` fetch 闭包 → `zero_net::read_file_url`），默认启用即拒绝
//!   该面（2026-11-02 双腿实证）——收窄即 Chrome parity（Chrome 对 file:// 页模块
//!   加载本按 CORS 拒绝）；主文档/子资源/静态模块依赖已全部经 browser fetch_proxy
//!   （IPC）直读，产品流不受影响。
//! - **④ fail 语义（本切片拍板记录）**：fail-closed——任一安装失败 `Err` 上抛，
//!   `run_desktop_role` 传播 → main exit(1)，浏览器侧呈现为 renderer 启动失败
//!   （页面错误可见），不降级裸奔。操作员出口：`ZW_RENDERER_SANDBOX=0`（全关）、
//!   `ZW_RENDERER_SECCOMP=0`、`ZW_RENDERER_LANDLOCK=0`。
//! - 非 Linux 平台：编译门控跳过 + 显式日志（与 compositor/image-decoder 同口径）。
//!
//! 时序契约：`run_desktop_role` 首语句调用，先于一切子线程（landlock restrict
//! 仅约束调用线程及其后继线程；M3 切片 2 时序审计已钉此入口）。BPF/landlock
//! UAPI 本地定义的原因同 image-decoder 切片（libc 对 android target 缺类型）。

/// 是否启用 renderer 沙箱主开关（默认开；`0` 禁用——操作员显式豁免面）。
pub fn renderer_sandbox_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_RENDERER_SANDBOX")
}

/// 是否启用 renderer seccomp 过滤（默认开；`ZW_RENDERER_SECCOMP=0` 禁用）。
// 非 Linux 平台无调用方（`apply_early_if_enabled` 的 Linux 臂与 Linux-only
// 测试是全部引用面），不门控会在 mac/windows 上 dead_code 撞 `-D warnings`。
#[cfg(target_os = "linux")]
pub fn renderer_seccomp_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_RENDERER_SECCOMP")
}

/// 是否启用 renderer landlock 允许清单（**默认开**；`ZW_RENDERER_LANDLOCK=0`
/// 禁用——操作员 kill-switch；①-a 拍板 2026-11-03 翻默认，见模块文档）。
#[cfg(target_os = "linux")]
pub fn renderer_landlock_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_RENDERER_LANDLOCK")
}

/// 启动早期应用 renderer 沙箱（env 剥离 + seccomp + landlock）。
///
/// 返回 `Err` 时调用方必须 fail-closed（拒绝运行），不得降级为无沙箱继续。
pub fn apply_early_if_enabled() -> Result<(), String> {
    if !renderer_sandbox_enabled() {
        tracing::warn!("renderer: sandbox 主开关关闭（ZW_RENDERER_SANDBOX=0），进程未沙箱运行");
        return Ok(());
    }
    sanitize_env();
    #[cfg(target_os = "linux")]
    {
        if renderer_seccomp_enabled() {
            seccomp::install_exec_filter().map_err(|error| format!("seccomp: {error}"))?;
            tracing::info!("renderer: seccomp fork/execveat/ptrace 过滤已启用（execve 保留供 helper spawn）");
        }
        if renderer_landlock_enabled() {
            landlock::install_renderer_allowlist().map_err(|error| format!("landlock: {error}"))?;
            tracing::info!("renderer: landlock 允许清单已启用");
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing::info!("renderer: seccomp/landlock 仅 Linux 可用，本平台未沙箱（编译门控跳过）");
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
    //! exec/ptrace 收窄 seccomp-bpf（黑名单形态，镜像 image-decoder 档案；
    //! 差异：`execve` 保留——renderer 是 helper 进程的合法 spawn 方）。

    // BPF 指令集常量与 SockFilter/SockFprog 为稳定内核 UAPI（Linux
    // Documentation/userspace-api/seccomp_filter.rst）。libc 未给 android
    // target 定义这组类型，本地定义、全 Linux 目标共用一套。
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

    /// 阻断清单：`execveat`（memfd 无文件执行）+ `fork`/`vfork`（std `Command`
    /// 走 posix_spawn/clone，不触这两个 syscall）+ `ptrace`（横向移动面）。
    /// `execve` 不在列——image-decoder/media-host spawn 的合法通路。
    pub fn blocked_syscalls() -> Vec<i64> {
        [
            libc::SYS_execveat,
            #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
            libc::SYS_fork,
            libc::SYS_ptrace,
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
    pub fn install_exec_filter() -> Result<(), String> {
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
    //! 允许清单 landlock：handled 四访问位 + 白名单规则 + restrict。
    //!
    //! 位定义与 syscall 封装教训（add_rule 第 4 参是 flags；READ≠1）沿用
    //! compositor `landlock_linux.rs` / image-decoder `sandbox.rs`。

    use std::ffi::CString;
    use std::io;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    // Landlock FS 访问位（ABI v1）。
    pub(super) const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
    pub(super) const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    pub(super) const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
    pub(super) const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;

    /// 目录只读位（READ_FILE 作用于树下文件，READ_DIR 列目录）。
    const FS_RO_DIR: u64 = LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;
    /// 目录读写位。
    const FS_RW_DIR: u64 = FS_RO_DIR | LANDLOCK_ACCESS_FS_WRITE_FILE;
    /// 动态库目录位：读 + EXECUTE（dlopen/mmap PROT_EXEC 需要 EXECUTE 位；
    /// execve 由 seccomp 层拦截 fork 系，此处 EXECUTE 只服务库加载与同目录
    /// helper 二进制）。
    const FS_RO_EXEC_DIR: u64 = FS_RO_DIR | LANDLOCK_ACCESS_FS_EXECUTE;
    /// 单文件只读位（landlock 规则对文件 fd 禁止目录位，否则 EINVAL）。
    const FS_RO_FILE: u64 = LANDLOCK_ACCESS_FS_READ_FILE;

    #[repr(C)]
    struct LandlockRulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C)]
    struct LandlockPathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    pub(super) fn create_ruleset(handled_access_fs: u64) -> Result<i32, io::Error> {
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

    fn open_path_beneath(path: &Path) -> Result<i32, io::Error> {
        // OsStr 字节不带 NUL 终止符，须经 CString（否则 open 读越界、路径尾随
        // 堆上垃圾字节——首轮调试实测全规则 ENOENT/ENOTDIR 的根因）。
        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "landlock 路径含 NUL"))?;
        // SAFETY: c_path 为 NUL 终止的合法 C 字符串；fd 所有权归调用方。
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(fd)
        }
    }

    fn add_rule(ruleset_fd: i32, path: &Path, allowed_access: u64) -> Result<(), io::Error> {
        let parent_fd = open_path_beneath(path)?;
        let attr = LandlockPathBeneathAttr {
            allowed_access,
            parent_fd,
        };
        // SAFETY: attr 指向栈上合法属性；landlock_add_rule 第 4 参是保留 flags（0）。
        let ret = unsafe { libc::syscall(libc::SYS_landlock_add_rule, ruleset_fd, 1u32, &attr, 0u32) };
        // SAFETY: fd 所有权归本函数。
        unsafe {
            libc::close(parent_fd);
        }
        if ret != 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// 可选规则：路径不存在/不可打开时静默跳过（/etc 按发行版差异、lib 目录
    /// 多架构差异等非关键面）。
    fn add_optional_rule(ruleset_fd: i32, path: &str, allowed_access: u64) {
        if let Err(error) = add_rule(ruleset_fd, Path::new(path), allowed_access)
            && std::env::var_os("ZW_RENDERER_LANDLOCK_DEBUG").is_some()
        {
            eprintln!("landlock optional rule {path} ({allowed_access:#x}): {error}");
        }
    }

    /// 从 `path` 向上找最近存在祖先（含自身）——为尚不存在的目录/文件建规则，
    /// 使 create/写路径在 allowlist 内成立（compositor XDG_CACHE_HOME 同型）。
    fn nearest_existing(path: &Path) -> Option<std::path::PathBuf> {
        let mut cur = path.to_path_buf();
        loop {
            if cur.exists() {
                return Some(cur);
            }
            cur = cur.parent()?.to_path_buf();
        }
    }

    /// 授予「目标或其最近存在祖先」的目录级访问（新建多级目录也覆盖）。
    fn grant_dir_or_ancestor(ruleset_fd: i32, path: &Path, dir_access: u64) {
        if let Some(existing) = nearest_existing(path)
            && let Err(error) = add_rule(ruleset_fd, &existing, dir_access)
            && std::env::var_os("ZW_RENDERER_LANDLOCK_DEBUG").is_some()
        {
            eprintln!("landlock grant {path:?} -> {existing:?} ({dir_access:#x}): {error}");
        }
    }

    /// renderer 允许清单安装（审计依据见模块文档；每条规则对应一处在环 IO 面）。
    ///
    /// 任何安装错误（含老内核 ENOSYS / 受限容器 EPERM）原样上抛——fail-closed
    /// 输入面。
    pub fn install_renderer_allowlist() -> Result<(), String> {
        let ruleset_fd = create_ruleset(
            LANDLOCK_ACCESS_FS_EXECUTE
                | LANDLOCK_ACCESS_FS_WRITE_FILE
                | LANDLOCK_ACCESS_FS_READ_FILE
                | LANDLOCK_ACCESS_FS_READ_DIR,
        )
        .map_err(|e| format!("create_ruleset: {e}"))?;

        // ① 字体 ro（拍板文本；render-foundation 字体清单全在 /usr/share/fonts 下，
        //    /usr/local/share/fonts、/etc/fonts 与 compositor 档案同列——后两者当前
        //    代码未引用，optional 授予无扩权对象即空规则）。
        for dir in ["/usr/share/fonts", "/usr/local/share/fonts", "/etc/fonts"] {
            add_optional_rule(ruleset_fd, dir, FS_RO_DIR);
        }

        // ② HTTP 磁盘缓存 rw（拍板文本「profile/缓存 rw」的在环实体；本进程唯一
        //    常规写路径——shared_http_cache open_persistent。ZERO_CACHE_DIR 覆盖与
        //    XDG 默认都随 default_cache_dir 解析，最近存在祖先保证建目录成功）。
        let cache_dir = zero_net::default_cache_dir();
        grant_dir_or_ancestor(ruleset_fd, &cache_dir, FS_RW_DIR);

        // ③ /dev/shm rw（拍板文本：fd 通道 socket 与后续 media C2 帧段；与
        //    compositor 档案同款）。
        add_optional_rule(ruleset_fd, "/dev/shm", FS_RW_DIR);

        // ④ 运行时必需 /etc 只读：DNS（tokio lookup_host → getaddrinfo 读
        //    resolv/hosts/nsswitch/gai/host.conf/services）、JS `Date` 本地时区
        //    （quickjs → libc localtime → /etc/localtime）。
        for file in [
            "/etc/localtime",
            "/etc/resolv.conf",
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/gai.conf",
            "/etc/host.conf",
            "/etc/services",
        ] {
            add_optional_rule(ruleset_fd, file, FS_RO_FILE);
        }

        // ⑤ 动态库目录 ro+EXECUTE（dlopen/NSS 插件/gconv——glibc <2.34 的
        //    libnss_dns 未并入 libc 时 DNS 必需；execve 被 seccomp 收窄，此
        //    EXECUTE 只服务库加载）。
        for dir in ["/usr/lib", "/lib", "/lib64"] {
            add_optional_rule(ruleset_fd, dir, FS_RO_EXEC_DIR);
        }

        // ⑥ helper 二进制目录 EXECUTE+读（image-decoder / media-host 与本进程
        //    同目录部署——`webview::image_decoder::resolve_image_decoder_bin`
        //    sibling 约定）；landlock EXECUTE 位把 execve 路径收窄到本目录与
        //    ⑤ 的库目录。
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            grant_dir_or_ancestor(ruleset_fd, dir, FS_RO_EXEC_DIR);
        }

        // ⑦ 进程内存探针（T13 S7：`mem_reclaim::read_process_pss_kb` 读
        //    /proc/self/smaps_rollup）。
        add_optional_rule(ruleset_fd, "/proc/self/smaps_rollup", FS_RO_FILE);

        // ⑧ 开发追踪面（env 显式指定路径才授；未设不授任何写面）。
        for key in [
            "ZW_RESIZE_TRACE",
            "ZW_MAINLOOP_TRACE",
            "ZW_EXEC_TIMING",
            "ZW_AUTOMATION_TRACE",
        ] {
            if let Ok(path) = std::env::var(key)
                && path != "1"
            {
                grant_dir_or_ancestor(ruleset_fd, Path::new(&path), FS_RW_DIR);
            }
        }

        // restrict_self 契约要求 no-new-privs（生产由先行的 seccomp 满足；此处
        // 自带设置，与 seccomp 开关解耦——镜像 image-decoder）。
        // SAFETY: prctl 常量参数，无指针。
        let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if ret != 0 {
            return Err(format!("PR_SET_NO_NEW_PRIVS 失败: {}", io::Error::last_os_error()));
        }
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
    fn sandbox_env_gates() {
        // 主开关与 seccomp：默认启用；显式 0 禁用（kill-switch 语义）。
        for key in ["ZW_RENDERER_SANDBOX", "ZW_RENDERER_SECCOMP"] {
            unsafe {
                std::env::set_var(key, "0");
            }
        }
        assert!(!renderer_sandbox_enabled());
        assert!(!renderer_seccomp_enabled());
        for key in ["ZW_RENDERER_SANDBOX", "ZW_RENDERER_SECCOMP"] {
            unsafe {
                std::env::remove_var(key);
            }
        }
        assert!(renderer_sandbox_enabled());
        assert!(renderer_seccomp_enabled());
        // landlock：默认启用（①-a 拍板翻默认）；显式 0 禁用（kill-switch 语义）。
        assert!(renderer_landlock_enabled());
        unsafe {
            std::env::set_var("ZW_RENDERER_LANDLOCK", "0");
        }
        assert!(!renderer_landlock_enabled());
        unsafe {
            std::env::remove_var("ZW_RENDERER_LANDLOCK");
        }
    }

    #[test]
    fn seccomp_blocklist_targets_exec_family() {
        let blocked = seccomp::blocked_syscalls();
        for nr in [libc::SYS_execveat, libc::SYS_ptrace] {
            assert!(blocked.contains(&nr), "阻断清单缺 {nr}");
        }
        assert!(
            !blocked.contains(&libc::SYS_execve),
            "execve 必须保留（helper spawn 合法通路）"
        );
        assert!(
            !blocked.contains(&libc::SYS_clone),
            "clone 必须保留（线程/posix_spawn）"
        );
        assert!(!blocked.contains(&libc::SYS_socket), "网络面不动（拍板原文）");
    }

    /// seccomp 面隔离子进程执行型验收（landlock 经 env 显式关——默认已开，不隔离
    /// 会连带拒绝 fs 读探针，landlock 自有下述完整栈用例）：
    /// SYS_fork/execveat/ptrace 拒、execve+posix_spawn 存活（image-decoder spawn
    /// 保全的判别式）、stdio 存活、fs 未受限（landlock 不在环）。fork 用原始
    /// syscall 探测——glibc `fork()` 走 `SYS_clone`，测不到过滤器。子进程 env 由
    /// 父进程钉死（`sandbox_env_gates` 与探针并行时共享进程 env，不能依赖环境态）。
    /// 退出码：0=全过；1=安装失败；2=SYS_fork 未拒；3=Command spawn 失败（posix_spawn
    /// 通路破坏）；4=ptrace 未拒；5=execveat 未拒；6=stdio 破坏；7=fs 读取意外被拒。
    #[test]
    fn seccomp_default_face_blocks_fork_keeps_spawn() {
        const CHILD: &str = "ZERO_RENDERER_SECCOMP_CHILD";
        let test_name = "sandbox::tests::seccomp_default_face_blocks_fork_keeps_spawn";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            let code = match super::apply_early_if_enabled() {
                Err(error) => {
                    eprintln!("install failed: {error}");
                    1
                }
                Ok(()) => {
                    // SAFETY: fork 原始 syscall 探针；仅子探针进程执行。fork 成功时
                    // 父侧 waitpid、子侧立即 exit，均不回到 libtest。
                    // aarch64 等 64 位架构无独立 fork syscall（glibc 经 clone 实现，
                    // `SYS_fork` 常量在 libc 该 target 也不存在），阻断清单同样只在
                    // x86 家族编入 fork——非 x86 架构按「已被拒」语义（-1）直接走
                    // execveat/ptrace/stdio/fs 后续探针。
                    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
                    let fork_pid = unsafe { libc::syscall(libc::SYS_fork) };
                    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
                    let fork_pid: libc::c_long = -1;
                    if fork_pid != -1 {
                        if fork_pid > 0 {
                            unsafe { libc::waitpid(fork_pid as i32, std::ptr::null_mut(), 0) };
                        }
                        2
                    } else if std::process::Command::new("/bin/true").status().is_err() {
                        3
                    } else {
                        // SAFETY: ptrace 探针（不应成功）；仅子探针进程执行。
                        let ptrace_ret = unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) };
                        if ptrace_ret == 0 {
                            4
                        } else {
                            // SAFETY: execveat 原始 syscall 探针（无效 fd 必然失败，
                            // 但失败码应为 EPERM——被过滤器拒绝而非 EBADF）。仅子探针执行。
                            let ret = unsafe {
                                libc::syscall(
                                    libc::SYS_execveat,
                                    -1,
                                    std::ptr::null::<()>(),
                                    std::ptr::null::<()>(),
                                    std::ptr::null::<()>(),
                                    0u32,
                                )
                            };
                            if ret == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::EPERM) {
                                5
                            } else {
                                let stdio_ok = std::io::Write::write_all(&mut std::io::stderr(), b"ok").is_ok();
                                if !stdio_ok {
                                    6
                                } else if std::fs::read_to_string("/etc/passwd").is_err() {
                                    7
                                } else {
                                    0
                                }
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
            // 钉死开关态：sandbox_env_gates 等并行用例会改共享进程 env；landlock
            // 显式关（默认已开）以隔离 seccomp 面。
            .env("ZW_RENDERER_SANDBOX", "1")
            .env("ZW_RENDERER_SECCOMP", "1")
            .env("ZW_RENDERER_LANDLOCK", "0")
            .output()
            .expect("spawn isolated seccomp probe child");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行"
        );
        let code = output.status.code();
        assert_eq!(
            code,
            Some(0),
            "seccomp 默认面行为不符（exit={code:?}）stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// landlock 允许清单隔离子进程执行型验收（完整栈：seccomp + landlock 显式钉开）：
    /// /dev/shm 与字体目录放行、/etc/passwd 拒（允许清单判别式）、/bin/true execve
    /// 拒（EXECUTE 路径收窄——/usr/bin 不在授予面）、stdio 存活。
    /// 退出码：0=全过；1=安装失败；2=/dev/shm 写失败；3=字体目录读失败；
    /// 4=/etc/passwd 读未拒；5=/bin/true spawn 未拒（EXECUTE 收窄失效）；
    /// 6=stdio 破坏；7=内核不支持 landlock（环境受限，父进程跳过）。
    #[test]
    fn landlock_allowlist_real_restrict() {
        const CHILD: &str = "ZERO_RENDERER_LANDLOCK_CHILD";
        let test_name = "sandbox::tests::landlock_allowlist_real_restrict";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            if let Err(error) = landlock::create_ruleset(
                landlock::LANDLOCK_ACCESS_FS_EXECUTE
                    | landlock::LANDLOCK_ACCESS_FS_WRITE_FILE
                    | landlock::LANDLOCK_ACCESS_FS_READ_FILE
                    | landlock::LANDLOCK_ACCESS_FS_READ_DIR,
            ) && matches!(error.raw_os_error(), Some(libc::ENOSYS) | Some(libc::EPERM))
            {
                std::process::exit(7);
            }
            let code = match super::apply_early_if_enabled() {
                Err(error) => {
                    eprintln!("install failed: {error}");
                    1
                }
                Ok(()) => {
                    let shm_write = std::fs::write("/dev/shm/zw-ren-probe", b"x");
                    let shm_ok = shm_write.is_ok() && std::fs::read("/dev/shm/zw-ren-probe").is_ok();
                    let _ = std::fs::remove_file("/dev/shm/zw-ren-probe");
                    if !shm_ok {
                        if let Err(error) = shm_write {
                            eprintln!("shm write errno: {error}");
                        }
                        2
                    } else {
                        let fonts_ok = std::path::Path::new("/usr/share/fonts").read_dir().is_ok();
                        if !fonts_ok {
                            3
                        } else if std::fs::read_to_string("/etc/passwd").is_ok() {
                            4
                        } else if std::process::Command::new("/bin/true").status().is_ok() {
                            5
                        } else {
                            let stdio_ok = std::io::Write::write_all(&mut std::io::stderr(), b"ok").is_ok();
                            if stdio_ok { 0 } else { 6 }
                        }
                    }
                }
            };
            std::process::exit(code);
        }
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            // 钉死开关态（完整栈 = seccomp + landlock）。
            .env("ZW_RENDERER_SANDBOX", "1")
            .env("ZW_RENDERER_SECCOMP", "1")
            .env("ZW_RENDERER_LANDLOCK", "1")
            .env("ZW_RENDERER_LANDLOCK_DEBUG", "1")
            .output()
            .expect("spawn isolated landlock probe child");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行"
        );
        let code = output.status.code();
        assert!(
            matches!(code, Some(0) | Some(7)),
            "landlock 允许清单行为不符（exit={code:?}）stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
