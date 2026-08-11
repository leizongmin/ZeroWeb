//! RFC 4.5：compositor 进程沙箱（S1 env / S2 seccomp / S3 landlock）。

#[cfg(target_os = "linux")]
#[path = "seccomp_linux.rs"]
mod seccomp_linux;

#[cfg(target_os = "linux")]
#[path = "landlock_linux.rs"]
mod landlock_linux;

/// 是否启用 compositor 沙箱钩子（环境变量剥离）。
pub fn compositor_sandbox_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SANDBOX").is_ok_and(|v| v == "1")
}

/// 是否启用 compositor seccomp 过滤器（Linux；`ZW_COMPOSITOR_SECCOMP=1`）。
pub fn compositor_seccomp_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SECCOMP").is_ok_and(|v| v == "1")
}

/// 是否启用 compositor Landlock（Linux；`ZW_COMPOSITOR_LANDLOCK=1`）。
pub fn compositor_landlock_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_LANDLOCK").is_ok_and(|v| v == "1")
}

/// 启动早期沙箱：env 剥离 + seccomp（须在任何子线程之前）。
pub fn apply_early_if_enabled() {
    if compositor_sandbox_enabled() {
        sanitize_env();
        tracing::info!("compositor: sandbox 钩子已应用（环境变量剥离）");
    }
    apply_seccomp_if_enabled();
}

fn compositor_gpu_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_GPU").is_ok_and(|v| v == "1")
}

/// 字体加载完成后应用 Landlock（`ZW_COMPOSITOR_LANDLOCK=1`）。
pub fn apply_landlock_after_init() {
    if !compositor_landlock_enabled() {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        let gpu = compositor_gpu_enabled();
        let result = if gpu {
            landlock_linux::install_compositor_landlock_gpu_aware()
        } else {
            landlock_linux::install_compositor_landlock()
        };
        match result {
            Ok(()) => tracing::info!("compositor: landlock 文件系统沙箱已启用 (gpu={gpu})"),
            Err(error) => tracing::warn!("compositor: landlock 安装失败（继续运行）: {error}"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing::warn!("compositor: landlock 仅 Linux 可用，已跳过");
    }
}

fn apply_seccomp_if_enabled() {
    if !compositor_seccomp_enabled() {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        let gpu = compositor_gpu_enabled();
        let result = if gpu {
            seccomp_linux::install_network_exec_filter_gpu_aware()
        } else {
            seccomp_linux::install_network_exec_filter()
        };
        match result {
            Ok(()) => tracing::info!("compositor: seccomp 网络/exec 过滤已启用 (gpu={gpu})"),
            Err(error) => tracing::warn!("compositor: seccomp 安装失败（继续运行）: {error}"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing::warn!("compositor: seccomp 仅 Linux 可用，已跳过");
    }
}

fn sanitize_env() {
    for key in [
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "RUST_BACKTRACE",
    ] {
        // SAFETY: compositor 沙箱启动阶段剥离注入型 env；仅删除已知键。
        unsafe {
            std::env::remove_var(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_env_does_not_panic() {
        // SAFETY: 测试专用 env 写入。
        unsafe {
            std::env::set_var("LD_PRELOAD", "/tmp/evil.so");
        }
        sanitize_env();
        assert!(std::env::var("LD_PRELOAD").is_err());
    }
}
