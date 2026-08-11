//! Linux Landlock 文件系统沙箱（RFC 4.5-S3 / P2 GPU 共存）。

use std::ffi::CString;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

const LANDLOCK_ACCESS_FS_READ: u64 = 1;
const LANDLOCK_ACCESS_FS_WRITE: u64 = 2;
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
    let ret = unsafe {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            ruleset_fd,
            LANDLOCK_RULE_PATH_BENEATH,
            rule as *const _,
            std::mem::size_of::<LandlockPathBeneathAttr>(),
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
        let _ = add_path_rule(
            ruleset_fd,
            Path::new(path),
            LANDLOCK_ACCESS_FS_READ | LANDLOCK_ACCESS_FS_WRITE,
        );
    }
}

fn add_optional_ro_rules(ruleset_fd: i32, paths: &[&str]) {
    for path in paths {
        let _ = add_path_rule(ruleset_fd, Path::new(path), LANDLOCK_ACCESS_FS_READ);
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
        handled_access_fs: LANDLOCK_ACCESS_FS_READ | LANDLOCK_ACCESS_FS_WRITE,
    };
    let ruleset_fd =
        syscall_landlock_create_ruleset(&attr).map_err(|e| format!("landlock_create_ruleset 失败: {e}"))?;

    add_path_rule(
        ruleset_fd,
        Path::new("/dev/shm"),
        LANDLOCK_ACCESS_FS_READ | LANDLOCK_ACCESS_FS_WRITE,
    )
    .map_err(|e| format!("landlock /dev/shm 规则失败: {e}"))?;

    for dir in font_read_dirs() {
        let _ = add_path_rule(ruleset_fd, Path::new(dir), LANDLOCK_ACCESS_FS_READ);
    }

    if gpu {
        let _ = add_path_rule(
            ruleset_fd,
            Path::new("/dev/dri"),
            LANDLOCK_ACCESS_FS_READ | LANDLOCK_ACCESS_FS_WRITE,
        );
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
            handled_access_fs: LANDLOCK_ACCESS_FS_READ | LANDLOCK_ACCESS_FS_WRITE,
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
}
