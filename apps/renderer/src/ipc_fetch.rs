//! Renderer 非阻塞 IPC fetch — 供 [`AsyncPageLoad`] 并发发起子资源请求。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};
use zero_protocol::IpcChannel;
use zero_protocol::message::{FetchParams, FetchResponseParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;

type IpcOutbound = PipeTransport<std::io::Empty, Box<dyn std::io::Write + Send>>;

enum InflightReply {
    Text(Sender<Result<String, String>>),
    Bytes(Sender<Result<Vec<u8>, String>>),
    StreamBytes {
        tx: Sender<Result<Vec<u8>, String>>,
        body: Vec<u8>,
    },
    Ignore,
}

/// 进行中的 IPC fetch（request_id → 完成通道）。
pub struct InflightIpcFetches {
    pending: HashMap<u64, InflightReply>,
    document_url: Option<String>,
}

impl InflightIpcFetches {
    /// 创建空表。
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            document_url: None,
        }
    }

    /// 清空（导航取消时丢弃未完成的响应）。
    pub fn clear(&mut self) {
        self.pending.clear();
        self.document_url = None;
    }

    /// 若 `msg` 为匹配的 [`FetchResponse`]，完成对应接收端并返回 `true`。
    pub fn try_complete(&mut self, msg: &IpcMessage) -> bool {
        let IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id,
            status_code,
            headers,
            body,
        }) = &msg.kind
        else {
            return false;
        };
        let Some(reply) = self.pending.remove(request_id) else {
            return false;
        };
        if (200..300).contains(status_code)
            && headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-resource-type") && value == "document")
            && let Some((_, final_url)) = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("x-zero-final-url"))
        {
            self.document_url = Some(final_url.clone());
        }
        if let InflightReply::StreamBytes {
            tx,
            body: mut collected,
        } = reply
        {
            if is_stream_chunk(headers) {
                collected.extend_from_slice(body);
                self.pending
                    .insert(*request_id, InflightReply::StreamBytes { tx, body: collected });
            } else {
                deliver_reply(InflightReply::StreamBytes { tx, body: collected }, *status_code, body);
            }
            return true;
        }
        deliver_reply(reply, *status_code, body);
        true
    }

    /// 取出最近完成的主文档最终 URL。
    pub fn take_document_url(&mut self) -> Option<String> {
        self.document_url.take()
    }
}

fn deliver_reply(reply: InflightReply, status_code: u16, body: &[u8]) {
    match reply {
        InflightReply::Ignore => {}
        InflightReply::Bytes(tx) => {
            let result = if (200..300).contains(&status_code) {
                Ok(body.to_vec())
            } else {
                Err(fetch_error(status_code, body))
            };
            let _ = tx.send(result);
        }
        InflightReply::StreamBytes {
            tx,
            body: mut collected,
        } => {
            let result = if (200..300).contains(&status_code) {
                collected.extend_from_slice(body);
                Ok(collected)
            } else {
                Err(fetch_error(status_code, body))
            };
            let _ = tx.send(result);
        }
        InflightReply::Text(tx) => {
            let result = if (200..300).contains(&status_code) {
                String::from_utf8(body.to_vec()).map_err(|e| e.to_string())
            } else {
                Err(fetch_error(status_code, body))
            };
            let _ = tx.send(result);
        }
    }
}

fn is_stream_chunk(headers: &[(String, String)]) -> bool {
    headers
        .iter()
        .any(|(name, value)| name.eq_ignore_ascii_case("x-zero-stream-chunk") && value == "1")
}

fn fetch_error(status_code: u16, body: &[u8]) -> String {
    if status_code == 0 {
        let msg = String::from_utf8_lossy(body).trim().to_string();
        if msg.is_empty() {
            "网络请求失败（浏览器未能完成 HTTP 抓取）".to_string()
        } else {
            msg
        }
    } else {
        format!("HTTP {status_code}")
    }
}

/// 非阻塞 IPC [`AsyncFetchHost`]：发 `FetchRequest` 后立即返回 `Receiver`。
pub struct IpcAsyncFetchHost<'a> {
    outbound: &'a mut IpcOutbound,
    next_fetch_id: &'a mut u64,
    inflight: &'a mut InflightIpcFetches,
}

impl<'a> IpcAsyncFetchHost<'a> {
    /// 构造 per-tick 借用的 IPC fetch 宿主。
    pub fn new(
        outbound: &'a mut IpcOutbound,
        next_fetch_id: &'a mut u64,
        inflight: &'a mut InflightIpcFetches,
    ) -> Self {
        Self {
            outbound,
            next_fetch_id,
            inflight,
        }
    }

    fn issue_fetch(&mut self, url: &str, meta: ResourceFetchMeta, reply: InflightReply) -> Result<(), String> {
        let request_id = *self.next_fetch_id;
        *self.next_fetch_id += 1;
        tracing::info!(
            request_id,
            url,
            kind = meta.resource_type,
            priority = meta.priority,
            "renderer fetch start"
        );
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id,
                url: url.to_string(),
                method: "GET".into(),
                headers: {
                    let mut headers = vec![
                        ("X-Zero-Resource-Type".into(), meta.resource_type.into()),
                        ("X-Zero-Priority".into(), meta.priority.to_string()),
                    ];
                    if matches!(&reply, InflightReply::StreamBytes { .. }) {
                        headers.push(("X-Zero-Stream-Image".into(), "1".into()));
                    }
                    headers
                },
                body: None,
            }),
        };
        self.outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))?;
        self.inflight.pending.insert(request_id, reply);
        Ok(())
    }

    fn issue_preconnect(&mut self, origin: &str) -> Result<(), String> {
        let request_id = *self.next_fetch_id;
        *self.next_fetch_id += 1;
        tracing::info!(request_id, url = origin, "renderer IPC preconnect request");
        let msg = IpcMessage {
            id: request_id,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id,
                url: origin.to_string(),
                method: "HEAD".into(),
                // Preconnect is an implementation hint, not a page resource request.
                // Do not forward renderer-only metadata to the origin server.
                headers: Vec::new(),
                body: None,
            }),
        };
        self.outbound
            .send(msg)
            .map_err(|e| format!("IPC fetch send failed: {e}"))?;
        self.inflight.pending.insert(request_id, InflightReply::Ignore);
        Ok(())
    }

    fn issue_dns_prefetch(&mut self, origin: &str) -> Result<(), String> {
        let request_id = *self.next_fetch_id;
        *self.next_fetch_id += 1;
        tracing::info!(request_id, url = origin, "renderer IPC DNS prefetch request");
        let msg = IpcMessage {
            id: request_id,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id,
                url: origin.to_string(),
                // Internal browser-process signal; this is never an HTTP method sent to an origin.
                method: "DNS-PREFETCH".into(),
                headers: Vec::new(),
                body: None,
            }),
        };
        self.outbound
            .send(msg)
            .map_err(|e| format!("IPC DNS prefetch send failed: {e}"))?;
        self.inflight.pending.insert(request_id, InflightReply::Ignore);
        Ok(())
    }
}

impl AsyncFetchHost for IpcAsyncFetchHost<'_> {
    fn preconnect(&mut self, origin: &str) {
        let _ = self.issue_preconnect(origin);
    }

    fn dns_prefetch(&mut self, origin: &str) {
        let _ = self.issue_dns_prefetch(origin);
    }

    fn fetch_text_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        let (tx, rx) = channel();
        if let Err(e) = self.issue_fetch(url, meta, InflightReply::Text(tx)) {
            let (fallback_tx, fallback_rx) = channel();
            let _ = fallback_tx.send(Err(e));
            return fallback_rx;
        }
        rx
    }

    fn fetch_bytes_meta(&mut self, url: &str, meta: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        let (tx, rx) = channel();
        let reply = if meta.resource_type == "image" {
            InflightReply::StreamBytes { tx, body: Vec::new() }
        } else {
            InflightReply::Bytes(tx)
        };
        if let Err(e) = self.issue_fetch(url, meta, reply) {
            let (fallback_tx, fallback_rx) = channel();
            let _ = fallback_tx.send(Err(e));
            return fallback_rx;
        }
        rx
    }
}

/// 无 browser 进程时的测试 stub：fetch 立即返回 Err，避免 AsyncPageLoad 永久 pending。
pub struct StubAsyncFetchHost;

impl AsyncFetchHost for StubAsyncFetchHost {
    fn fetch_text_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        let (tx, rx) = channel();
        let _ = tx.send(Err("stub network".into()));
        rx
    }

    fn fetch_bytes_meta(&mut self, _: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        let (tx, rx) = channel();
        let _ = tx.send(Err("stub network".into()));
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);
    impl Write for SharedBuf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn fetch_response_delivers_bytes() {
        let mut inflight = InflightIpcFetches::new();
        let (tx, rx) = channel();
        inflight.pending.insert(42, InflightReply::Bytes(tx));
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 42,
                status_code: 200,
                headers: Vec::new(),
                body: b"ok".to_vec(),
            }),
        };
        assert!(inflight.try_complete(&msg));
        assert_eq!(rx.try_recv().unwrap().unwrap(), b"ok");
        assert!(inflight.pending.is_empty());
    }

    #[test]
    fn streamed_image_chunks_wait_for_final_response() {
        let mut inflight = InflightIpcFetches::new();
        let (tx, rx) = channel();
        inflight
            .pending
            .insert(42, InflightReply::StreamBytes { tx, body: Vec::new() });
        let chunk = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 42,
                status_code: 200,
                headers: vec![("X-Zero-Stream-Chunk".into(), "1".into())],
                body: b"first".to_vec(),
            }),
        };
        assert!(inflight.try_complete(&chunk));
        assert!(rx.try_recv().is_err());
        assert!(matches!(
            inflight.pending.get(&42),
            Some(InflightReply::StreamBytes { .. })
        ));

        let final_response = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 42,
                status_code: 200,
                headers: Vec::new(),
                body: b"second".to_vec(),
            }),
        };
        assert!(inflight.try_complete(&final_response));
        assert_eq!(rx.try_recv().unwrap().unwrap(), b"firstsecond");
    }

    #[test]
    fn unknown_request_id_is_ignored() {
        let mut inflight = InflightIpcFetches::new();
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 99,
                status_code: 200,
                headers: Vec::new(),
                body: vec![],
            }),
        };
        assert!(!inflight.try_complete(&msg));
    }

    #[test]
    fn fetch_response_delivers_text() {
        let mut inflight = InflightIpcFetches::new();
        let (tx, rx) = channel();
        inflight.pending.insert(7, InflightReply::Text(tx));
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 7,
                status_code: 200,
                headers: vec![
                    ("X-Zero-Resource-Type".into(), "document".into()),
                    ("X-Zero-Final-URL".into(), "https://final.example/page".into()),
                ],
                body: b"hello".to_vec(),
            }),
        };
        assert!(inflight.try_complete(&msg));
        assert_eq!(rx.try_recv().unwrap().unwrap(), "hello");
        assert_eq!(
            inflight.take_document_url().as_deref(),
            Some("https://final.example/page")
        );
    }

    #[test]
    fn fetch_response_http_error_propagates() {
        let mut inflight = InflightIpcFetches::new();
        let (tx, rx) = channel();
        inflight.pending.insert(1, InflightReply::Bytes(tx));
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 1,
                status_code: 404,
                headers: Vec::new(),
                body: b"gone".to_vec(),
            }),
        };
        assert!(inflight.try_complete(&msg));
        assert!(rx.try_recv().unwrap().is_err());
    }

    #[test]
    fn non_fetch_message_is_not_consumed() {
        let mut inflight = InflightIpcFetches::new();
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::LoadComplete,
        };
        assert!(!inflight.try_complete(&msg));
    }

    #[test]
    fn clear_drops_pending_replies() {
        let mut inflight = InflightIpcFetches::new();
        let (tx, _rx) = channel();
        inflight.pending.insert(1, InflightReply::Bytes(tx));
        inflight.clear();
        assert!(inflight.pending.is_empty());
    }

    #[test]
    fn ipc_async_fetch_host_issues_fetch_request() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let mut next_id = 1_u64;
        let mut inflight = InflightIpcFetches::new();
        let mut host = IpcAsyncFetchHost::new(&mut outbound, &mut next_id, &mut inflight);
        let rx = host.fetch_bytes("https://example.com/a.png");
        assert!(rx.try_recv().is_err());
        assert_eq!(next_id, 2);
        assert_eq!(inflight.pending.len(), 1);
        assert!(!buf.0.lock().unwrap().is_empty());
    }

    #[test]
    fn ipc_image_fetch_requests_browser_streaming() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let mut next_id = 1_u64;
        let mut inflight = InflightIpcFetches::new();
        let mut host = IpcAsyncFetchHost::new(&mut outbound, &mut next_id, &mut inflight);
        let _ = host.fetch_bytes_meta("https://example.com/a.png", ResourceFetchMeta::IMAGE);
        let frame = buf.0.lock().unwrap();
        let request = zero_protocol::deserialize(&frame[4..]).unwrap();
        assert!(matches!(
            request.kind,
            IpcMessageKind::FetchRequest(FetchParams { headers, .. })
                if headers.iter().any(|(name, value)| name == "X-Zero-Stream-Image" && value == "1")
        ));
        assert!(matches!(
            inflight.pending.get(&1),
            Some(InflightReply::StreamBytes { .. })
        ));
    }

    #[test]
    fn ipc_preconnect_uses_head_and_discards_response() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let mut next_id = 7_u64;
        let mut inflight = InflightIpcFetches::new();
        {
            let mut host = IpcAsyncFetchHost::new(&mut outbound, &mut next_id, &mut inflight);
            host.preconnect("https://cdn.example.test");
        }

        assert_eq!(next_id, 8);
        assert!(matches!(inflight.pending.get(&7), Some(InflightReply::Ignore)));
        let frame = buf.0.lock().unwrap();
        let payload_len = u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize;
        assert_eq!(payload_len, frame.len() - 4);
        let request = zero_protocol::deserialize(&frame[4..]).unwrap();
        assert!(matches!(
            request.kind,
            IpcMessageKind::FetchRequest(FetchParams {
                request_id: 7,
                method,
                headers,
                body: None,
                ..
            }) if method == "HEAD" && headers.is_empty()
        ));
        drop(frame);
        let response = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 7,
                status_code: 204,
                headers: Vec::new(),
                body: Vec::new(),
            }),
        };
        assert!(inflight.try_complete(&response));
        assert!(inflight.pending.is_empty());
    }

    #[test]
    fn ipc_dns_prefetch_uses_internal_method_and_discards_response() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let mut next_id = 9_u64;
        let mut inflight = InflightIpcFetches::new();
        {
            let mut host = IpcAsyncFetchHost::new(&mut outbound, &mut next_id, &mut inflight);
            host.dns_prefetch("https://cdn.example.test");
        }

        let frame = buf.0.lock().unwrap();
        let request = zero_protocol::deserialize(&frame[4..]).unwrap();
        assert!(matches!(
            request.kind,
            IpcMessageKind::FetchRequest(FetchParams {
                request_id: 9,
                method,
                headers,
                body: None,
                ..
            }) if method == "DNS-PREFETCH" && headers.is_empty()
        ));
        drop(frame);
        assert!(matches!(inflight.pending.get(&9), Some(InflightReply::Ignore)));
    }

    #[test]
    fn concurrent_ipc_fetches_use_distinct_request_ids() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf) as Box<dyn Write + Send>);
        let mut next_id = 10_u64;
        let mut inflight = InflightIpcFetches::new();
        let mut host = IpcAsyncFetchHost::new(&mut outbound, &mut next_id, &mut inflight);
        let _r1 = host.fetch_text("https://example.com/a.css");
        let _r2 = host.fetch_bytes("https://example.com/b.png");
        assert_eq!(inflight.pending.len(), 2);
        assert_eq!(next_id, 12);
    }

    #[test]
    fn stub_async_fetch_host_returns_immediate_error() {
        let mut host = StubAsyncFetchHost;
        let rx = host.fetch_text("https://example.com/x");
        assert!(rx.try_recv().unwrap().is_err());
    }
}
