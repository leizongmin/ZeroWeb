//! # zero-page-runtime
//!
//! 三条页面路径（WPT / TabWorker / zero-renderer）共享的页面处理逻辑契约。
//!
//! 当前提供 [`PageLoadHost`]——分阶段页面加载的宿主抽象（资源抓取 + 绘制推送），
//! 供 in-process（webview / net_pool）与 IPC（renderer）两种宿主实现统一消费。
//! 详见 `docs/specs/runtime-unification.md`。

#![warn(missing_docs)]

use std::sync::mpsc::{Receiver, channel};

use zero_engine::{HitTestCache, RenderResult};
use zero_render_foundation::primitive::RenderPrimitives;

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

/// 阻塞抓取宿主：把一个**同步阻塞**的 fetch（如 renderer 的 IPC 抓取）适配成 [`AsyncFetchHost`]。
///
/// 每次 `fetch_*` 同步调用内部 fetch 取得结果，再包进一次性 `Receiver`（立即可读）。
/// renderer 无头、加载期阻塞可接受；tabworker 侧不用本类型（用 webview 的 `InProcessFetchHost`）。
/// renderer（B3）经 per-tick 构造 `BlockingFetchHost::new(\|url\| ipc_fetch_get(...))` 复用 webview 的 `AsyncPageLoad`。
pub struct BlockingFetchHost<F> {
    /// 同步阻塞 fetch 回调（返回字节）。
    fetch: F,
}

impl<F> BlockingFetchHost<F>
where
    F: FnMut(&str) -> Result<Vec<u8>, String>,
{
    /// 用阻塞 fetch 回调构造。
    pub fn new(fetch: F) -> Self {
        Self { fetch }
    }
}

impl<F> AsyncFetchHost for BlockingFetchHost<F>
where
    F: FnMut(&str) -> Result<Vec<u8>, String>,
{
    fn fetch_text(&mut self, url: &str) -> Receiver<Result<String, String>> {
        let (tx, rx) = channel();
        let result = (self.fetch)(url).and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()));
        let _ = tx.send(result);
        rx
    }

    fn fetch_bytes(&mut self, url: &str) -> Receiver<Result<Vec<u8>, String>> {
        let (tx, rx) = channel();
        let _ = tx.send((self.fetch)(url));
        rx
    }
}

/// 统一绘制帧契约（FrameModel）——三路径 frame 输出的抽象接缝（T5）。
///
/// renderer（IPC）从 WebView 读后由此打包成 `PaintSnapshotParams`；tabworker（in-process）
/// 从 WebView 读后填入 `TabSnapshot`。统一「视口 + 文档高度 + 图元 + hit-test」的产出契约，
/// 让两侧不再各自散列这些字段。
pub struct FrameModel {
    /// 视口（CSS 逻辑像素）。
    pub viewport: (u32, u32),
    /// 文档内容高度。
    pub document_height: f32,
    /// 渲染图元。
    pub primitives: RenderPrimitives,
    /// 主线程 hit-test 缓存（可选）。
    pub hit_test: Option<HitTestCache>,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// BlockingFetchHost 同步抓取后须立即预填 Receiver（try_recv 立即可读），
    /// 这样 webview 的 AsyncPageLoad 轮询模型能无缝消费阻塞 fetch（renderer IPC 场景）。
    #[test]
    fn blocking_fetch_host_prefills_receiver() {
        let mut host = BlockingFetchHost::new(|url: &str| Ok(format!("body:{url}").into_bytes()));

        let rx = host.fetch_bytes("http://x");
        assert_eq!(rx.try_recv().unwrap().unwrap(), b"body:http://x");

        let rx = host.fetch_text("http://y");
        assert_eq!(rx.try_recv().unwrap().unwrap(), "body:http://y");
    }

    /// fetch 回调返回 Err 时，Receiver 须透传错误。
    #[test]
    fn blocking_fetch_host_propagates_error() {
        let mut host = BlockingFetchHost::new(|_: &str| Err("net down".to_string()));
        let rx = host.fetch_bytes("http://x");
        assert!(rx.try_recv().unwrap().is_err());
    }

    /// BlockingFetchHost 可作为 `&mut dyn AsyncFetchHost` 使用（trait object，B3 per-tick 传入 tick 所需）。
    #[test]
    fn blocking_fetch_host_is_object_safe() {
        let mut host = BlockingFetchHost::new(|url: &str| Ok(url.as_bytes().to_vec()));
        let dyn_host: &mut dyn AsyncFetchHost = &mut host;
        let rx = dyn_host.fetch_bytes("http://z");
        assert_eq!(rx.try_recv().unwrap().unwrap(), b"http://z");
    }
}
