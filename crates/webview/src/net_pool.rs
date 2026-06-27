//! 共享 HTTP 线程池 — 避免在 UI 线程上阻塞网络 I/O。

use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use zero_net::HttpClient;

/// HTTP GET 任务结果。
pub type HttpTextResult = Result<String, String>;

struct NetPoolInner {
    job_tx: Sender<(String, Sender<HttpTextResult>)>,
}

static NET_POOL: OnceLock<NetPoolInner> = OnceLock::new();

fn pool() -> &'static NetPoolInner {
    NET_POOL.get_or_init(|| {
        let (job_tx, job_rx) = mpsc::channel::<(String, Sender<HttpTextResult>)>();

        thread::Builder::new()
            .name("zero-net-pool".into())
            .spawn(move || {
                let client = HttpClient::new();
                while let Ok((url, reply_tx)) = job_rx.recv() {
                    let result = client.get(&url).and_then(|resp| resp.text()).map_err(|e| e.to_string());
                    let _ = reply_tx.send(result);
                }
            })
            .expect("spawn net worker");

        NetPoolInner { job_tx }
    })
}

/// 在后台线程池中发起 HTTP GET，返回文本结果接收端。
pub fn fetch_text_async(url: impl Into<String>) -> Receiver<HttpTextResult> {
    let (tx, rx) = mpsc::channel();
    let _ = pool().job_tx.send((url.into(), tx));
    rx
}

/// 在后台线程池中发起 HTTP GET 并返回原始字节。
pub fn fetch_bytes_async(url: impl Into<String>) -> Receiver<Result<Vec<u8>, String>> {
    let (tx, rx) = mpsc::channel();
    let url = url.into();
    thread::spawn(move || {
        let client = HttpClient::new();
        let result = client.get(&url).map(|resp| resp.body).map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
    rx
}
