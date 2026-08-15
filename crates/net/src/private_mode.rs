//! 隐私浏览检测 — 无痕模式不写磁盘缓存。

/// 是否启用隐私浏览（不写磁盘 HTTP 缓存）。
///
/// 环境变量 `ZERO_PRIVATE=1` 或 `true` 时启用。
pub fn private_browsing_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZERO_PRIVATE")
}
