//! RFC 4.5：compositor 进程最小沙箱（`ZW_COMPOSITOR_SANDBOX=1` / `ZW_COMPOSITOR_SECCOMP=1`）。
//!
//! S1：剥离高风险环境变量。
//! S2（Linux）：seccomp-bpf 阻断网络 socket 与 exec/fork（`ZW_COMPOSITOR_SECCOMP=1`）。

#[cfg(target_os = "linux")]
#[path = "seccomp_linux.rs"]
mod seccomp_linux;

/// 是否启用 compositor 沙箱钩子（环境变量剥离）。
pub fn compositor_sandbox_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SANDBOX").is_ok_and(|v| v == "1")
}

/// 是否启用 compositor seccomp 过滤器（Linux；`ZW_COMPOSITOR_SECCOMP=1`）。
pub fn compositor_seccomp_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SECCOMP").is_ok_and(|v| v == "1")
}

/// 启动时应用 compositor 沙箱（失败则 warn 并继续，避免阻断开发）。
pub fn apply_if_enabled() {
    if compositor_sandbox_enabled() {
        sanitize_env();
        tracing::info!("compositor: sandbox 钩子已应用（环境变量剥离）");
    }
    apply_seccomp_if_enabled();
}

fn apply_seccomp_if_enabled() {
    if !compositor_seccomp_enabled() {
        return;
    }
    if std::env::var("ZW_COMPOSITOR_GPU").is_ok_and(|v| v == "1") {
        tracing::warn!("compositor: seccomp 与 ZW_COMPOSITOR_GPU=1 不兼容，已跳过");
        return;
    }
    #[cfg(target_os = "linux")]
    {
        match seccomp_linux::install_network_exec_filter() {
            Ok(()) => tracing::info!("compositor: seccomp 网络/exec 过滤已启用"),
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
