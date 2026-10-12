//! view-source 源码异步抓取（后台线程，主线程 poll 结果后开页）。
//!
//! 与 favicon_fetch 同模式：阻塞的 zero_net 请求放在独立线程（客户端自带 30s 超时），
//! 主循环经 `take_completed` 非阻塞取结果再开源码内页——事件循环线程不再被慢/黑洞
//! 目标阻塞（原实现在 UI 线程同步抓取，最长整窗冻结 30s）。

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

/// 单条进行中的源码抓取。
pub struct PendingViewSourceFetch {
    /// 规范化后的内层 URL（抓取目标，同时用于源码页头）。
    target: String,
    /// `view-source:<target>`（地址栏回填与错误页标题）。
    view_url: String,
    rx: Receiver<Result<String, String>>,
}

/// 一条已完成的抓取（成功或失败都有结果，供主循环开内页）。
pub struct CompletedViewSourceFetch {
    pub target: String,
    pub view_url: String,
    pub result: Result<String, String>,
}

/// 进行中的 view-source 网络请求队列。
#[derive(Default)]
pub struct ViewSourceFetchState {
    pending: Vec<PendingViewSourceFetch>,
}

impl ViewSourceFetchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 为目标 URL 发起后台线程抓取；重复导航各自独立记录，结果不丢。
    pub fn request(&mut self, target: String) {
        let view_url = format!("view-source:{target}");
        let (tx, rx) = mpsc::channel();
        let fetch_target = target.clone();
        thread::spawn(move || {
            // 接收端与队列同生命周期，发送失败仅发生在关闭竞争，静默丢弃
            let _ = tx.send(fetch_document_source(&fetch_target));
        });
        self.pending.push(PendingViewSourceFetch { target, view_url, rx });
    }

    /// 取出已完成的抓取（按发起顺序）；未完成的留在队列继续等待。
    pub fn take_completed(&mut self) -> Vec<CompletedViewSourceFetch> {
        let mut completed = Vec::new();
        self.pending.retain_mut(|pending| match pending.rx.try_recv() {
            Ok(result) => {
                completed.push(CompletedViewSourceFetch {
                    target: std::mem::take(&mut pending.target),
                    view_url: std::mem::take(&mut pending.view_url),
                    result,
                });
                false
            }
            // 发送端中止（抓取线程 panic）无结果可交付，与 favicon_fetch 同策略丢弃
            Err(TryRecvError::Disconnected) => false,
            Err(TryRecvError::Empty) => true,
        });
        completed
    }
}

/// 同步抓取目标 URL 响应体（view-source 源码展示；复用 zero_net 阻塞客户端，
/// 带默认 30s 超时，与 tab_js_worker 的直连读取同一路径家族）。
pub(crate) fn fetch_document_source(target: &str) -> Result<String, String> {
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return Err(format!("view-source 仅支持 http/https URL，实际为 {target}"));
    }
    let response = zero_net::HttpClient::new()
        .get(target)
        .map_err(|error| format!("无法加载 {target}: {error}"))?;
    Ok(String::from_utf8_lossy(&response.body).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 非 http(s) 内层直接拒绝（不发网络请求）。
    #[test]
    fn fetch_document_source_rejects_non_http_targets() {
        assert!(fetch_document_source("file:///etc/hosts").is_err());
        assert!(fetch_document_source("zero://settings").is_err());
        assert!(fetch_document_source("data:text/html,hi").is_err());
    }

    /// 已完成的抓取按发起顺序取出，未完成的留在队列。
    #[test]
    fn take_completed_drains_finished_and_keeps_pending() {
        let mut state = ViewSourceFetchState::new();
        let (tx1, rx1) = mpsc::channel();
        tx1.send(Ok("<html></html>".to_string())).unwrap();
        let (_tx2, rx2) = mpsc::channel();
        state.pending.push(PendingViewSourceFetch {
            target: "http://a/".to_string(),
            view_url: "view-source:http://a/".to_string(),
            rx: rx1,
        });
        state.pending.push(PendingViewSourceFetch {
            target: "http://b/".to_string(),
            view_url: "view-source:http://b/".to_string(),
            rx: rx2,
        });

        let done = state.take_completed();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].target, "http://a/");
        assert_eq!(done[0].view_url, "view-source:http://a/");
        assert_eq!(done[0].result.as_deref(), Ok("<html></html>"));
        assert_eq!(state.pending.len(), 1);
        assert_eq!(state.pending[0].target, "http://b/");
    }

    /// 发送端中止（无结果）的条目被移除，不阻塞后续完成项。
    #[test]
    fn take_completed_drops_disconnected_entries() {
        let mut state = ViewSourceFetchState::new();
        let (tx, rx) = mpsc::channel();
        drop(tx);
        state.pending.push(PendingViewSourceFetch {
            target: "http://a/".to_string(),
            view_url: "view-source:http://a/".to_string(),
            rx,
        });
        assert!(state.take_completed().is_empty());
        assert!(state.pending.is_empty());
    }
}
