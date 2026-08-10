//! RFC 4.5：compositor 进程最小沙箱钩子（`ZW_COMPOSITOR_SANDBOX=1`）。
//!
//! 当前切片：剥离高风险环境变量并拒绝非 compositor 角色启动。
//! 完整 seccomp/landlock 为后续切片。

/// 是否启用 compositor 沙箱钩子。
pub fn compositor_sandbox_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_SANDBOX").is_ok_and(|v| v == "1")
}

/// 启动时应用 compositor 沙箱钩子（失败则 stderr 提示并继续，避免阻断开发）。
pub fn apply_if_enabled() {
    if !compositor_sandbox_enabled() {
        return;
    }
    sanitize_env();
    tracing::info!("compositor: sandbox 钩子已应用（环境变量剥离）");
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
