//! 共享 HTTP 线程池 — per-origin 并发上限，对齐主流浏览器连接策略。

use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::{self, Receiver};

use zero_net::{FetchJobResult, PerOriginFetchScheduler};

/// HTTP GET 任务结果（文本）。
pub type HttpTextResult = Result<String, String>;

static NET_SCHEDULER: OnceLock<Arc<Mutex<PerOriginFetchScheduler>>> = OnceLock::new();

fn scheduler() -> Arc<Mutex<PerOriginFetchScheduler>> {
    NET_SCHEDULER
        .get_or_init(|| Arc::new(Mutex::new(PerOriginFetchScheduler::new())))
        .clone()
}

fn map_fetch_result(result: FetchJobResult) -> Result<Vec<u8>, String> {
    match result {
        Ok((status, body)) if (200..300).contains(&status) => Ok(body),
        Ok((status, body)) => {
            let detail = String::from_utf8_lossy(&body);
            Err(if detail.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                detail.trim().to_string()
            })
        }
        Err(e) => Err(e),
    }
}

fn map_fetch_text(result: FetchJobResult) -> HttpTextResult {
    map_fetch_result(result).and_then(|b| String::from_utf8(b).map_err(|e| e.to_string()))
}

fn bridge_rx<T, F>(rx: Receiver<FetchJobResult>, map: F) -> Receiver<T>
where
    F: FnOnce(FetchJobResult) -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, out) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(r) = rx.recv() {
            let _ = tx.send(map(r));
        }
    });
    out
}

/// 在后台调度器中发起 HTTP GET，返回文本结果接收端。
pub fn fetch_text_async(url: impl Into<String>) -> Receiver<HttpTextResult> {
    let rx = PerOriginFetchScheduler::submit_shared(&scheduler(), url.into());
    bridge_rx(rx, map_fetch_text)
}

/// 在后台调度器中发起 HTTP GET 并返回原始字节。
pub fn fetch_bytes_async(url: impl Into<String>) -> Receiver<Result<Vec<u8>, String>> {
    let rx = PerOriginFetchScheduler::submit_shared(&scheduler(), url.into());
    bridge_rx(rx, map_fetch_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::TryRecvError;
    use std::time::Duration;

    #[test]
    fn fetch_text_async_returns_before_completion() {
        let rx = fetch_text_async("http://127.0.0.1:1/unreachable");
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
    }

    #[test]
    fn fetch_bytes_async_eventually_returns_error_for_unreachable_host() {
        let rx = fetch_bytes_async("http://127.0.0.1:1/unreachable");
        let result = rx.recv_timeout(Duration::from_secs(5)).expect("worker should respond");
        assert!(result.is_err());
    }
}
