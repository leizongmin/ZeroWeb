//! Renderer 进程沙箱钩子（RFC §六 P3 切片：默认启用的 env 剥离 + seccomp 占位）。

/// 是否启用 renderer seccomp（默认开；`ZW_RENDERER_SECCOMP=0` 禁用）。
pub fn renderer_seccomp_enabled() -> bool {
    zero_runtime_config::enabled_by_default("ZW_RENDERER_SECCOMP")
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
        // 默认启用；显式 0 禁用（kill-switch 语义）。
        unsafe {
            std::env::set_var("ZW_RENDERER_SECCOMP", "0");
        }
        assert!(!renderer_seccomp_enabled());
        unsafe {
            std::env::remove_var("ZW_RENDERER_SECCOMP");
        }
        assert!(renderer_seccomp_enabled());
    }
}
