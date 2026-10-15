//! 图片“另存为”异步抓取（后台线程，主线程 poll 结果后走下载管线落盘）。
//!
//! 与 view_source_fetch / favicon_fetch 同模式：阻塞的 zero_net 请求放在独立线程
//! （客户端自带 30s 超时），事件循环线程不被慢/黑洞目标阻塞；主循环经
//! `take_completed` 非阻塞取结果再复用 attachment 下载管线写盘。

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use zero_browser_shell::TabId;

/// 单条进行中的图片抓取。
pub struct PendingImageSaveFetch {
    /// 发起另存为的标签。
    pub tab_id: TabId,
    /// 图片 URL（命中测试绝对化后的 src）。
    pub url: String,
    rx: Receiver<Result<Vec<u8>, String>>,
}

/// 已完成的抓取：`(标签, 图片 URL, 抓取结果)`；失败也交付，供主循环显式落败。
pub type CompletedImageSaveFetch = (TabId, String, Result<Vec<u8>, String>);

/// 进行中的图片另存为网络请求队列。
#[derive(Default)]
pub struct ImageSaveFetchState {
    pending: Vec<PendingImageSaveFetch>,
}

impl ImageSaveFetchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 为图片 URL 发起后台线程抓取；非 http(s) 来源不发起（另存为需要独立可保存
    /// 的资源），同一 URL 在途时忽略重复请求。
    pub fn request(&mut self, tab_id: TabId, url: &str) {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            tracing::warn!("save_image 仅支持 http/https URL，忽略 {url}");
            return;
        }
        if self.pending.iter().any(|fetch| fetch.url == url) {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let fetch_url = url.to_string();
        thread::spawn(move || {
            // 接收端与队列同生命周期，发送失败仅发生在关闭竞争，静默丢弃
            let _ = tx.send(fetch_image_bytes(&fetch_url));
        });
        self.pending.push(PendingImageSaveFetch {
            tab_id,
            url: url.to_string(),
            rx,
        });
    }

    /// 取出已完成的抓取（按发起顺序）；未完成的留在队列继续等待。
    pub fn take_completed(&mut self) -> Vec<CompletedImageSaveFetch> {
        let mut completed = Vec::new();
        self.pending.retain_mut(|pending| match pending.rx.try_recv() {
            Ok(result) => {
                completed.push((pending.tab_id, std::mem::take(&mut pending.url), result));
                false
            }
            // 发送端中止（抓取线程 panic）无结果可交付，与 view_source_fetch 同策略丢弃
            Err(TryRecvError::Disconnected) => false,
            Err(TryRecvError::Empty) => true,
        });
        completed
    }
}

/// 同步抓取图片原始字节（另存为落盘用；复用 zero_net 阻塞客户端，30s 超时）。
/// 非 2xx 视为失败（错误页不是图片内容，不能落盘）。
fn fetch_image_bytes(target: &str) -> Result<Vec<u8>, String> {
    let response = zero_net::HttpClient::new()
        .get(target)
        .map_err(|error| format!("无法下载 {target}: {error}"))?;
    if !response.is_success() {
        return Err(format!("无法下载 {target}: HTTP {}", response.status_code));
    }
    Ok(response.body)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 非 http(s) 图片 URL 直接拒绝（不发网络请求）。
    #[test]
    fn request_rejects_non_http_urls() {
        let mut state = ImageSaveFetchState::new();
        let tab = TabId(1);
        state.request(tab, "data:image/png;base64,AAAA");
        state.request(tab, "zero://about");
        assert!(state.pending.is_empty());
    }

    /// 同一 URL 在途时不重复发起。
    #[test]
    fn request_dedupes_in_flight_url() {
        let mut state = ImageSaveFetchState::new();
        let tab = TabId(1);
        state.request(tab, "http://127.0.0.1:9/a.png");
        state.request(tab, "http://127.0.0.1:9/a.png");
        assert_eq!(state.pending.len(), 1);
    }

    /// 已完成的抓取按发起顺序取出，未完成的留在队列。
    #[test]
    fn take_completed_drains_finished_and_keeps_pending() {
        let mut state = ImageSaveFetchState::new();
        let (tx1, rx1) = mpsc::channel();
        tx1.send(Ok(b"png".to_vec())).unwrap();
        let (_tx2, rx2) = mpsc::channel();
        state.pending.push(PendingImageSaveFetch {
            tab_id: TabId(1),
            url: "http://a/a.png".to_string(),
            rx: rx1,
        });
        state.pending.push(PendingImageSaveFetch {
            tab_id: TabId(2),
            url: "http://b/b.png".to_string(),
            rx: rx2,
        });

        let done = state.take_completed();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].0, TabId(1));
        assert_eq!(done[0].1, "http://a/a.png");
        assert_eq!(done[0].2.as_deref(), Ok(b"png".as_slice()));
        assert_eq!(state.pending.len(), 1);
    }

    /// 发送端中止（无结果）的条目被移除，不阻塞后续完成项。
    #[test]
    fn take_completed_drops_disconnected_entries() {
        let mut state = ImageSaveFetchState::new();
        let (tx, rx) = mpsc::channel();
        drop(tx);
        state.pending.push(PendingImageSaveFetch {
            tab_id: TabId(1),
            url: "http://a/a.png".to_string(),
            rx,
        });
        assert!(state.take_completed().is_empty());
        assert!(state.pending.is_empty());
    }
}
