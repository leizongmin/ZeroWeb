//! # zero-page-runtime
//!
//! 三条页面路径（WPT / TabWorker / zero-renderer）共享的页面处理逻辑契约。
//!
//! 当前提供 [`PageLoadHost`]——分阶段页面加载的宿主抽象（资源抓取 + 绘制推送），
//! 供 in-process（webview / net_pool）与 IPC（renderer）两种宿主实现统一消费。
//! 详见 `docs/specs/runtime-unification.md`。

#![warn(missing_docs)]

use zero_engine::RenderResult;

/// 分阶段页面加载宿主：网络抓取 + 绘制推送。
///
/// 这是 WPT / TabWorker / zero-renderer 三条路径统一的 spine：加载算法
/// （FirstPaint → FetchingStylesheets → StyledPaint → FetchingImages → Complete）
/// 经此 trait 与具体宿主解耦，差异只留在实现（如 `InProcessHttpHost` vs `IpcHost`）。
pub trait PageLoadHost {
    /// 抓取 URL 对应字节（CSS / 图片 / 脚本等子资源）。
    fn fetch_bytes(&mut self, url: &str) -> Result<Vec<u8>, String>;

    /// 推送中间或最终绘制结果。
    ///
    /// `is_final` 为 `true` 表示加载完成后的最终帧；宿主可据此决定是否附带
    /// hit-test 缓存 / 图片 payload 等。
    fn publish(&mut self, result: &RenderResult, title: Option<String>, is_final: bool) -> Result<(), String>;
}
