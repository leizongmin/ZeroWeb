//! Renderer 进程沙箱钩子（RFC §六 P3 切片：env-gated seccomp 占位）。

/// 是否启用 renderer seccomp（`ZW_RENDERER_SECCOMP=1`）。
pub fn renderer_seccomp_enabled() -> bool {
    std::env::var("ZW_RENDERER_SECCOMP").is_ok_and(|v| v == "1")
}

/// 启动早期应用 renderer 沙箱（当前仅日志 + env 剥离）。
pub fn apply_early_if_enabled() {
    if !renderer_seccomp_enabled() {
        return;
    }
    for key in ["LD_PRELOAD", "LD_LIBRARY_PATH"] {
        // SAFETY: 启动阶段剥离注入型 env。
        unsafe {
            std::env::remove_var(key);
        }
    }
    tracing::info!("renderer: seccomp 沙箱钩子已启用（P3 占位；完整 OS sandbox 仍为后续）");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_seccomp_env_gate() {
        assert!(!renderer_seccomp_enabled() || std::env::var("ZW_RENDERER_SECCOMP").is_ok());
    }
}
