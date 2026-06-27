//! 隐私浏览检测 — 无痕模式不写磁盘缓存。

/// 是否启用隐私浏览（不写磁盘 HTTP 缓存）。
///
/// 环境变量 `ZERO_PRIVATE=1` 或 `true` 时启用。
pub fn private_browsing_enabled() -> bool {
    std::env::var("ZERO_PRIVATE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}
