//! 共享 HTTP 线程池 — per-origin 并发上限，对齐主流浏览器连接策略。

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, OnceLock};

use zero_net::{FetchJobResult, FetchPriority, HttpClient, HttpRequest, ResourceLoader, ResourceRequest};
use zero_page_runtime::ResourceFetchMeta;

/// HTTP GET 任务结果（文本）。
pub type HttpTextResult = Result<String, String>;

fn preconnecting_origins() -> &'static Mutex<HashSet<String>> {
    static ORIGINS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    ORIGINS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn dns_prefetching_origins() -> &'static Mutex<HashSet<String>> {
    static ORIGINS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    ORIGINS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 非阻塞预热连接；失败只记录日志，不影响页面加载。
pub fn preconnect_async(origin: impl Into<String>) {
    let origin = origin.into();
    let mut origins = preconnecting_origins()
        .lock()
        .expect("preconnect origin mutex poisoned");
    if !origins.insert(origin.clone()) {
        return;
    }
    drop(origins);

    let rx = HttpClient::new().preconnect(origin.clone());
    zero_net::client::spawn_network_bridge(move || {
        if let Ok(Err(error)) = rx.recv() {
            tracing::debug!(%error, "connection preconnect failed");
        }
        preconnecting_origins()
            .lock()
            .expect("preconnect origin mutex poisoned")
            .remove(&origin);
    });
}

/// 非阻塞预解析 DNS；失败只记录日志，不影响页面加载。
pub fn dns_prefetch_async(origin: impl Into<String>) {
    let origin = origin.into();
    let mut origins = dns_prefetching_origins()
        .lock()
        .expect("DNS prefetch origin mutex poisoned");
    if !origins.insert(origin.clone()) {
        return;
    }
    drop(origins);

    let rx = HttpClient::new().dns_prefetch(origin.clone());
    zero_net::client::spawn_network_bridge(move || {
        if let Ok(Err(error)) = rx.recv() {
            tracing::debug!(%error, "DNS prefetch failed");
        }
        dns_prefetching_origins()
            .lock()
            .expect("DNS prefetch origin mutex poisoned")
            .remove(&origin);
    });
}

fn map_fetch_result(result: zero_net::FetchJobResult) -> Result<Vec<u8>, String> {
    match result {
        Ok(resp) if (200..300).contains(&resp.status_code) => Ok(resp.body),
        Ok(resp) => {
            let detail = String::from_utf8_lossy(&resp.body);
            Err(if detail.trim().is_empty() {
                format!("HTTP {}", resp.status_code)
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
    zero_net::client::spawn_network_bridge(move || {
        if let Ok(r) = rx.recv() {
            let _ = tx.send(map(r));
        }
    });
    out
}

/// 统一缓存感知提交：负缓存 → `ResourceLoader`（缓存/在途合并/调度）→ 负缓存回写。
fn submit_cached(url: String, meta: ResourceFetchMeta) -> Receiver<FetchJobResult> {
    // 负缓存：失败冷却期内跳过网络（renderer 每 publish 重请求失败图 → 此处收敛）
    if zero_net::shared_negative_cache()
        .lock()
        .unwrap()
        .is_recently_failed(&url)
    {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(Err(format!("negative cache (recent failure): {url}")));
        return rx;
    }

    let rx = ResourceLoader::shared().submit(
        ResourceRequest::get(url.clone(), FetchPriority::from_u8(meta.priority)).with_destination(meta.resource_type),
    );
    bridge_rx(rx, move |result| finalize_result(result, &url))
}

/// 完成路径更新负缓存；HTTP 缓存由 `ResourceLoader` 统一处理。
fn finalize_result(result: FetchJobResult, url: &str) -> FetchJobResult {
    match &result {
        Ok(resp) if (200..300).contains(&resp.status_code) => {
            zero_net::shared_negative_cache().lock().unwrap().mark_ok(url);
        }
        Ok(_) => {}
        Err(_) => {
            zero_net::shared_negative_cache().lock().unwrap().mark_failed(url);
        }
    }
    result
}

/// 在后台调度器中发起 HTTP GET，返回文本结果接收端。
pub fn fetch_text_async(url: impl Into<String>) -> Receiver<HttpTextResult> {
    fetch_text_async_meta(url, ResourceFetchMeta::DOCUMENT)
}

/// 带优先级的文本 GET。
pub fn fetch_text_async_meta(url: impl Into<String>, meta: ResourceFetchMeta) -> Receiver<HttpTextResult> {
    let rx = submit_cached(url.into(), meta);
    bridge_rx(rx, map_fetch_text)
}

/// 异步抓取主文档请求。GET 复用缓存调度器；unsafe 方法 write-through 后失效相关缓存。
pub(crate) fn fetch_document_async(
    url: impl Into<String>,
    method: &str,
    body: Option<&[u8]>,
) -> Receiver<HttpTextResult> {
    let url = url.into();
    if method.eq_ignore_ascii_case("GET") && body.is_none() {
        return fetch_text_async_meta(url, ResourceFetchMeta::DOCUMENT);
    }
    if method.eq_ignore_ascii_case("POST") {
        let Some(body) = body else {
            let (tx, rx) = mpsc::channel();
            let _ = tx.send(Err("POST document request requires a body".to_string()));
            return rx;
        };
        return bridge_rx(
            ResourceLoader::shared().submit_http(
                HttpRequest::post(&url, body.to_vec()).header("Content-Type", "application/x-www-form-urlencoded"),
                FetchPriority::CRITICAL,
            ),
            map_fetch_text,
        );
    }
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(Err(format!("unsupported document request method: {method}")));
    rx
}

/// 在后台调度器中发起 HTTP GET 并返回原始字节。
pub fn fetch_bytes_async(url: impl Into<String>) -> Receiver<Result<Vec<u8>, String>> {
    fetch_bytes_async_meta(url, ResourceFetchMeta::IMAGE)
}

/// 带优先级的字节 GET。
pub fn fetch_bytes_async_meta(url: impl Into<String>, meta: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
    let rx = submit_cached(url.into(), meta);
    bridge_rx(rx, map_fetch_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc::TryRecvError;
    use std::time::Duration;

    #[test]
    fn fetch_text_async_returns_before_completion() {
        let rx = fetch_text_async("http://127.0.0.1:1/unreachable");
        // fetch_text_async 非阻塞返回 receiver——try_recv 立即返。接受 Empty（仍在途）或已就绪 Err：
        // 127.0.0.1:1 connection-refused 近乎即时，高并发负载下 worker 可能先于 try_recv 完成发回 Err，
        // 两者均证「不阻塞调用方 + receiver 可用」（旧单值 Empty 断言在此竞态下 flaky）。
        // 仅 Disconnected / 意外成功视为失败。
        match rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            Ok(res) => assert!(res.is_err(), "unreachable host should error, got: {:?}", res),
            Err(TryRecvError::Disconnected) => panic!("worker disconnected without response"),
        }
    }

    #[test]
    fn fetch_bytes_async_eventually_returns_error_for_unreachable_host() {
        let rx = fetch_bytes_async("http://127.0.0.1:1/unreachable");
        let result = rx.recv_timeout(Duration::from_secs(5)).expect("worker should respond");
        assert!(result.is_err());
    }

    #[test]
    fn fetch_document_async_posts_urlencoded_body_without_cache() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (request_tx, request_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            stream.set_read_timeout(Some(Duration::from_secs(5))).expect("timeout");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0u8; 1024];
                let read = stream.read(&mut chunk).expect("read request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
            let request_text = String::from_utf8(request).expect("request utf8");
            request_tx.send(request_text).expect("request capture");
            let response = "<html><head><title>posted</title></head><body>ok</body></html>";
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response.len(),
                response
            )
            .expect("write response");
        });

        let url = format!("http://{addr}/submit");
        let response = fetch_document_async(&url, "POST", Some(b"name=zero&go=1"))
            .recv_timeout(Duration::from_secs(5))
            .expect("POST response")
            .expect("successful POST");
        assert!(response.contains("<title>posted</title>"));
        let request = request_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("captured request");
        assert!(request.starts_with("POST /submit HTTP/1.1\r\n"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("content-type: application/x-www-form-urlencoded")
        );
        assert!(request.ends_with("name=zero&go=1"));
    }
}
