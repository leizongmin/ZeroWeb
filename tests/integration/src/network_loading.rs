//! 网络加载器的协议级集成验收。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use zero_net::{CacheLookup, FetchPriority, HttpCache, ResourceLoader, ResourceRequest};

struct FixtureServer {
    address: std::net::SocketAddr,
    requests: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl FixtureServer {
    fn start(cache_control: &'static str, vary: Option<&'static str>, response_delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        listener.set_nonblocking(true).expect("make fixture server nonblocking");
        let address = listener.local_addr().expect("fixture address");
        let requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let request_count = Arc::clone(&requests);
        let stopping = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        request_count.fetch_add(1, Ordering::Relaxed);
                        thread::spawn(move || respond(stream, cache_control, vary, response_delay));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("fixture accept failed: {error}"),
                }
            }
        });
        Self {
            address,
            requests,
            stop,
            thread: Some(thread),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    fn request_count(&self) -> usize {
        self.requests.load(Ordering::Relaxed)
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.thread
            .take()
            .expect("fixture thread")
            .join()
            .expect("join fixture thread");
    }
}

fn respond(mut stream: TcpStream, cache_control: &str, vary: Option<&str>, response_delay: Duration) {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set fixture read timeout");
    let mut request = [0_u8; 4096];
    let _ = stream.read(&mut request).expect("read fixture request");
    thread::sleep(response_delay);
    let body = b"fixture-response";
    let vary = vary.map(|value| format!("Vary: {value}\r\n")).unwrap_or_default();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nCache-Control: {cache_control}\r\n{vary}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write fixture response headers");
    stream.write_all(body).expect("write fixture response body");
}

fn loader() -> ResourceLoader {
    ResourceLoader::new(Arc::new(Mutex::new(HttpCache::new())), "fixture")
}

/// FR-001：fresh cache 命中必须在加载器内完成，不能再启动网络事务。
#[test]
fn cache_fresh_hit_bypasses_scheduler() {
    let server = FixtureServer::start("max-age=60", None, Duration::ZERO);
    let loader = loader();
    let url = server.url("/a.css");

    loader
        .submit(ResourceRequest::get(url.clone(), FetchPriority::CRITICAL))
        .recv_timeout(Duration::from_secs(2))
        .expect("seed response")
        .expect("successful seed response");
    let first = loader.submit(ResourceRequest::get(url.clone(), FetchPriority::CRITICAL));
    let second = loader.submit(ResourceRequest::get(url, FetchPriority::CRITICAL));

    assert!(
        first
            .recv_timeout(Duration::from_millis(100))
            .expect("first cache response")
            .is_ok()
    );
    assert!(
        second
            .recv_timeout(Duration::from_millis(100))
            .expect("second cache response")
            .is_ok()
    );
    assert_eq!(server.request_count(), 1, "fresh hits must not consume scheduler slots");
    let stats = loader.stats();
    assert_eq!(stats.network_requests, 1);
    assert_eq!(stats.fresh_hits, 2);
    assert_eq!(stats.network_response_bytes, b"fixture-response".len() as u64);
    assert!(
        stats.network_elapsed_ms <= 2_000,
        "fixture request must complete within its timeout"
    );
}

/// FR-003：`no-store` 响应不得进入可复用缓存。
#[test]
fn no_store_never_persists() {
    let server = FixtureServer::start("no-store", None, Duration::ZERO);
    let loader = loader();
    let url = server.url("/token");

    for _ in 0..2 {
        let response = loader
            .submit(ResourceRequest::get(url.clone(), FetchPriority::MEDIUM))
            .recv_timeout(Duration::from_secs(2))
            .expect("network response");
        assert!(response.is_ok());
    }

    assert_eq!(server.request_count(), 2, "no-store responses must not be reused");
    assert_eq!(loader.stats().fresh_hits, 0);
    assert_eq!(loader.stats().network_requests, 2);
}

/// FR-003：过期 ETag 条目的并发消费者必须合并为一次条件 GET，并共同得到 304 合并后的 body。
#[test]
fn stale_etag_revalidation_is_coalesced() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    let (request_tx, request_rx) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        for response_index in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept fixture request");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("set timeout");
            let mut request = [0_u8; 4096];
            let count = stream.read(&mut request).expect("read fixture request");
            request_tx
                .send(String::from_utf8_lossy(&request[..count]).into_owned())
                .expect("record request");
            if response_index == 0 {
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nCache-Control: no-cache\r\nETag: \"v1\"\r\nContent-Length: 9\r\nConnection: close\r\n\r\ncached-v1",
                    )
                    .expect("write seed response");
            } else {
                stream
                    .write_all(
                        b"HTTP/1.1 304 Not Modified\r\nCache-Control: max-age=60\r\nETag: \"v1\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .expect("write revalidation response");
            }
        }
    });
    let cache = Arc::new(Mutex::new(HttpCache::new()));
    let loader = ResourceLoader::new(Arc::clone(&cache), "fixture");
    let url = format!("http://{address}/app.js");

    let seeded = loader
        .submit(ResourceRequest::get(url.clone(), FetchPriority::HIGH))
        .recv_timeout(Duration::from_secs(2))
        .expect("seed response")
        .expect("successful seed");
    assert_eq!(seeded.body, b"cached-v1");
    assert!(
        matches!(cache.lock().unwrap().lookup(&url, &[]), CacheLookup::Revalidate { .. }),
        "seed entry must be retained as a revalidatable cache entry"
    );
    let first = loader.submit(ResourceRequest::get(url.clone(), FetchPriority::HIGH));
    let second = loader.submit(ResourceRequest::get(url, FetchPriority::HIGH));
    for response in [first, second] {
        assert_eq!(
            response
                .recv_timeout(Duration::from_secs(2))
                .expect("revalidated response")
                .expect("successful revalidation")
                .body,
            b"cached-v1"
        );
    }

    let requests: Vec<_> = (0..2)
        .map(|_| {
            request_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("recorded request")
        })
        .collect();
    assert!(requests[1].to_ascii_lowercase().contains("if-none-match: \"v1\""));
    assert_eq!(loader.stats().revalidations, 1);
    assert_eq!(loader.stats().network_requests, 2, "seed plus one shared revalidation");
    server.join().expect("join fixture server");
}

/// FR-002：仅完全相同的请求身份可合并；会影响 Vary 的请求头必须保留隔离。
#[test]
fn coalesce_respects_request_identity_and_vary() {
    let server = FixtureServer::start("no-store", Some("Accept-Language"), Duration::from_millis(100));
    let loader = loader();
    let url = server.url("/greeting");
    let english = || {
        ResourceRequest::get(url.clone(), FetchPriority::MEDIUM)
            .with_headers(vec![("Accept-Language".into(), "en-US".into())])
    };
    let chinese = ResourceRequest::get(url.clone(), FetchPriority::MEDIUM)
        .with_headers(vec![("Accept-Language".into(), "zh-CN".into())]);

    let first_english = loader.submit(english());
    let second_english = loader.submit(english());
    let chinese = loader.submit(chinese);

    for response in [first_english, second_english, chinese] {
        assert!(
            response
                .recv_timeout(Duration::from_secs(2))
                .expect("fixture response")
                .is_ok()
        );
    }
    assert_eq!(server.request_count(), 2, "only the equal en-US requests may collapse");
    assert!(
        loader
            .events()
            .iter()
            .any(|event| event.coalesced_subscriber_count == 2),
        "the owning transaction event must report both coalesced subscribers"
    );
}

/// NFR-001：顶级站点分区是缓存与在途事务的隔离边界，不能从另一站点复用响应。
#[test]
fn cache_partition_prevents_cross_site_reuse() {
    let server = FixtureServer::start("max-age=60", None, Duration::ZERO);
    let loader = loader();
    let url = server.url("/shared.png");
    let site_a = || ResourceRequest::get(url.clone(), FetchPriority::MEDIUM).with_partition("https://a.example");
    let site_b = || ResourceRequest::get(url.clone(), FetchPriority::MEDIUM).with_partition("https://b.example");

    for request in [site_a(), site_b(), site_a(), site_b()] {
        assert!(
            loader
                .submit(request)
                .recv_timeout(Duration::from_secs(2))
                .expect("fixture response")
                .is_ok()
        );
    }
    assert_eq!(server.request_count(), 2, "each partition needs its own initial fetch");
}
