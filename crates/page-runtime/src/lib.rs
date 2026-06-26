//! # zero-page-runtime
//!
//! 三条页面路径（WPT / TabWorker / zero-renderer）共享的页面处理逻辑契约。
//!
//! 当前提供 [`PageLoadHost`]——分阶段页面加载的宿主抽象（资源抓取 + 绘制推送），
//! 供 in-process（webview / net_pool）与 IPC（renderer）两种宿主实现统一消费。
//! 详见 `docs/specs/runtime-unification.md`。

#![warn(missing_docs)]

use std::sync::mpsc::Receiver;

use zero_engine::RenderResult;

/// 异步抓取宿主：发起网络抓取并返回可轮询的接收器。
///
/// in-process 实现（webview）走 net_pool 线程池；IPC 实现（renderer）把阻塞 IPC 抓取
/// 封装到独立线程后返回接收器。供 webview `AsyncPageLoad` 消除 `net_pool` 硬编码，
/// 并为 renderer 复用同一加载器铺路（tick/轮询模型两端一致）。
pub trait AsyncFetchHost {
    /// 抓取文本资源（主文档 / 外链 CSS）。
    fn fetch_text(&mut self, url: &str) -> Receiver<Result<String, String>>;
    /// 抓取二进制资源（图片等）。
    fn fetch_bytes(&mut self, url: &str) -> Receiver<Result<Vec<u8>, String>>;
}

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
