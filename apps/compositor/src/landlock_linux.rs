//! Linux Landlock 文件系统沙箱（RFC 4.5-S3 / P2 GPU 共存）。

use std::ffi::CString;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

// Landlock FS 访问位（ABI v1，linux/landlock.h）：
// https://docs.kernel.org/userspace-api/landlock.html
// 历史 bug（M3 实测确证，两处独立缺陷叠加导致 landlock 从未生效）：
// 1) `landlock_add_rule(ruleset_fd, rule_type, rule_attr, flags)` 第 4 参是
//    flags（内核无 size 参），旧实现把 `size_of::<PathBeneathAttr>()` 传入
//    flags 位，非零一律 EINVAL，安装永远失败走 fail-open；
// 2) 旧常量把 READ 定义为 `1`——按 ABI `1` 是 EXECUTE 位，READ_FILE 实为
//    `1<<2`。错位使 handled 从未包含读位，即便装上也只限制 exec/write，
//    读保护静默失效。另：目录 fd 授予 EXECUTE 本身合法（治理树下文件执行），
//    并不触发 EINVAL；本模块目录规则不授 EXECUTE 只是无需执行权。
const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
/// 目录 fd 可授予的只读位（READ_FILE 作用于树下文件，READ_DIR 列目录）。
const FS_RO_DIR: u64 = LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;
/// 目录 fd 可授予的读写位。
const FS_RW_DIR: u64 = FS_RO_DIR | LANDLOCK_ACCESS_FS_WRITE_FILE;
const LANDLOCK_RULE_PATH_BENEATH: i32 = 1;

#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

static FONT_READ_DIRS: OnceLock<Vec<&'static str>> = OnceLock::new();

fn font_read_dirs() -> &'static [&'static str] {
    FONT_READ_DIRS.get_or_init(|| {
        #[cfg(target_os = "macos")]
        {
            vec!["/System/Library/Fonts", "/Library/Fonts"]
        }
        #[cfg(target_os = "windows")]
        {
            vec!["C:\\Windows\\Fonts"]
        }
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            vec!["/usr/share/fonts", "/usr/local/share/fonts", "/etc/fonts"]
        }
    })
}

fn gpu_extra_read_dirs() -> &'static [&'static str] {
    &[
        "/usr/share/vulkan",
        "/usr/share/dri",
        "/usr/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
    ]
}

fn syscall_landlock_create_ruleset(attr: &LandlockRulesetAttr) -> Result<i32, io::Error> {
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            attr as *const _,
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

fn syscall_landlock_add_rule(ruleset_fd: i32, rule: &LandlockPathBeneathAttr) -> Result<(), io::Error> {
    // 第 4 参是 flags（保留 0）；landlock_add_rule 没有 size 参——传 size 会让
    // 内核按非零 flags 拒绝（EINVAL），历史 bug 见模块头注释。
    let ret = unsafe {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            ruleset_fd,
            LANDLOCK_RULE_PATH_BENEATH,
            rule as *const _,
            0u32,
        )
    };
    if ret != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn syscall_landlock_restrict_self(ruleset_fd: i32) -> Result<(), io::Error> {
    let ret = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd, 0u32) };
    if ret != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn open_path_beneath(path: &Path) -> Result<i32, io::Error> {
    let c_path = CString::new(path.to_string_lossy().as_ref())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "landlock 路径含 NUL"))?;
    let fd = unsafe { libc::openat(libc::AT_FDCWD, c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

fn add_path_rule(ruleset_fd: i32, path: &Path, access: u64) -> Result<(), io::Error> {
    let parent_fd = match open_path_beneath(path) {
        Ok(fd) => fd,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let rule = LandlockPathBeneathAttr {
        allowed_access: access,
        parent_fd,
    };
    let result = syscall_landlock_add_rule(ruleset_fd, &rule);
    unsafe {
        libc::close(parent_fd);
    }
    result
}

fn add_optional_rw_rules(ruleset_fd: i32, paths: &[&str]) {
    for path in paths {
        let _ = add_path_rule(ruleset_fd, Path::new(path), FS_RW_DIR);
    }
}

fn add_optional_ro_rules(ruleset_fd: i32, paths: &[&str]) {
    for path in paths {
        let _ = add_path_rule(ruleset_fd, Path::new(path), FS_RO_DIR);
    }
}

/// 安装 Landlock：允许 `/dev/shm` 读写 + 字体目录只读。
pub fn install_compositor_landlock() -> Result<(), String> {
    install_compositor_landlock_inner(false)
}

/// GPU 模式：追加 `/dev/dri`、Vulkan/驱动目录与缓存路径。
pub fn install_compositor_landlock_gpu_aware() -> Result<(), String> {
    install_compositor_landlock_inner(true)
}

fn install_compositor_landlock_inner(gpu: bool) -> Result<(), String> {
    let attr = LandlockRulesetAttr {
        handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE
            | LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_READ_FILE
            | LANDLOCK_ACCESS_FS_READ_DIR,
    };
    let ruleset_fd =
        syscall_landlock_create_ruleset(&attr).map_err(|e| format!("landlock_create_ruleset 失败: {e}"))?;

    add_path_rule(ruleset_fd, Path::new("/dev/shm"), FS_RW_DIR)
        .map_err(|e| format!("landlock /dev/shm 规则失败: {e}"))?;

    for dir in font_read_dirs() {
        let _ = add_path_rule(ruleset_fd, Path::new(dir), FS_RO_DIR);
    }

    if gpu {
        let _ = add_path_rule(ruleset_fd, Path::new("/dev/dri"), FS_RW_DIR);
        add_optional_ro_rules(ruleset_fd, gpu_extra_read_dirs());
        if let Some(cache) = std::env::var_os("XDG_CACHE_HOME") {
            add_optional_rw_rules(ruleset_fd, &[cache.to_string_lossy().as_ref()]);
        } else if Path::new("/tmp").exists() {
            add_optional_rw_rules(ruleset_fd, &["/tmp"]);
        }
    }

    syscall_landlock_restrict_self(ruleset_fd).map_err(|e| format!("landlock_restrict_self 失败: {e}"))?;
    unsafe {
        libc::close(ruleset_fd);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landlock_create_ruleset_or_enosys() {
        let attr = LandlockRulesetAttr {
            handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE
                | LANDLOCK_ACCESS_FS_WRITE_FILE
                | LANDLOCK_ACCESS_FS_READ_FILE
                | LANDLOCK_ACCESS_FS_READ_DIR,
        };
        match syscall_landlock_create_ruleset(&attr) {
            Ok(fd) => unsafe {
                libc::close(fd);
            },
            Err(error) if error.raw_os_error() == Some(libc::ENOSYS) => {}
            Err(error) if error.raw_os_error() == Some(libc::EPERM) => {}
            Err(other) => panic!("unexpected landlock error: {other}"),
        }
    }

    /// 隔离子进程真实 restrict 后的文件访问验收（DC-4/M3：真实 helper 受限
    /// 操作验收）：允许路径（/dev/shm）读写成功，白名单外路径读取被拒。
    /// `restrict_self` 契约依赖调用方已设 `PR_SET_NO_NEW_PRIVS`——生产路径由
    /// 先行的 seccomp 安装（`install_filter` 第一步）满足，本测试镜像该前置。
    /// 退出码：0=全过；1=安装失败；2=允许路径读写失败；3=白名单外读未被拒；
    /// 4=内核不支持 landlock（按既有 ENOSYS/EPERM 容忍口径跳过）；5=目标路径
    /// 缺失（无法验收，跳过）。
    #[test]
    fn landlock_restrict_blocks_reads_outside_allowlist() {
        const CHILD: &str = "ZERO_COMPOSITOR_LANDLOCK_RESTRICT_CHILD";
        // 全路径（含模块前缀）：--exact 对短名零匹配会让子进程空跑 exit 0。
        let test_name = "sandbox::landlock_linux::tests::landlock_restrict_blocks_reads_outside_allowlist";
        if std::env::var(CHILD).as_deref() == Ok("1") {
            if !Path::new("/etc/passwd").exists() {
                std::process::exit(5);
            }
            // 能力预探测：内核不支持 landlock 时按容忍口径跳过（与
            // landlock_create_ruleset_or_enosys 一致），不当作行为失败。
            let attr = LandlockRulesetAttr {
                handled_access_fs: LANDLOCK_ACCESS_FS_EXECUTE
                    | LANDLOCK_ACCESS_FS_WRITE_FILE
                    | LANDLOCK_ACCESS_FS_READ_FILE
                    | LANDLOCK_ACCESS_FS_READ_DIR,
            };
            if let Err(error) = syscall_landlock_create_ruleset(&attr) {
                if matches!(error.raw_os_error(), Some(libc::ENOSYS) | Some(libc::EPERM)) {
                    std::process::exit(4);
                }
                std::process::exit(1);
            }
            // 生产路径中 no_new_privs 由先行的 seccomp 安装设置；此处镜像该前置。
            unsafe {
                libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            }
            let code = match install_compositor_landlock() {
                Err(error) => {
                    eprintln!("landlock install failed: {error}");
                    1
                }
                Ok(()) => {
                    let probe = "/dev/shm/zeroweb-landlock-probe";
                    let write_read_ok =
                        std::fs::write(probe, b"ok").is_ok() && std::fs::read(probe).is_ok_and(|b| b == b"ok");
                    let _ = std::fs::remove_file(probe);
                    if !write_read_ok {
                        2
                    } else {
                        match std::fs::read("/etc/passwd") {
                            Ok(_) => 3,
                            Err(error) if matches!(error.raw_os_error(), Some(libc::EACCES) | Some(libc::EPERM)) => 0,
                            Err(_) => 3,
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
            .expect("spawn isolated landlock probe child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // --exact 名称与真实测试路径（含模块前缀）失配时 libtest 空跑且 exit 0
        // （M3 实测确证的假绿根因），须显式抓住。
        assert!(
            stdout.contains("running 1 test"),
            "探测子进程未匹配到目标测试，探测未执行：stdout={stdout}"
        );
        // 4/5 为环境不支持（无 landlock 内核 / 目标路径缺失），跳过不断言。
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            matches!(output.status.code(), Some(0) | Some(4) | Some(5)),
            "landlock restrict 执行行为不符预期（exit={:?}）stdout={stdout} stderr={stderr}",
            output.status.code()
        );
    }
}
