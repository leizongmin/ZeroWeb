//! 发布路径图片批量取回（P14，2026-10-25）。
//!
//! [`crate::paint_export::fetch_image_payloads_with_cache`] 的两阶段宿主实现：
//! `issue` 先把全部缺图以与既有阻塞 fetch（runtime `fetch_get` 线型）完全相同的
//! （GET、无附加头——资源类型/优先级由 browser 侧按 URL 推断，既有负缓存与
//! 安全检查语义不变）非阻塞发出；首个 `resolve` 统一阻塞收齐全部在途
//! `FetchResponse`。同 origin 并发由 zero-net 调度器按每来源上限展开——旧逐发
//! 逐收形态下 renderer 侧恒 max-in-flight=1（cnblogs 107 请求 106 串行的第一根因）。
//!
//! 收包 defer 语义与既有阻塞 fetch 等待循环一致：非本批消息暂存局部队列、
//! 结束回填 `deferred`（不得吞 AsyncPageLoad 等在途 fetch 的响应）；Heartbeat
//! 就地回包；IPC 断开放弃剩余在途（对应 URL `resolve` 返回 `None`，图片按既有
//! 宽松降级跳过）。信任 browser 对每个请求恰回一个 `FetchResponse`（错误路径
//! immediate_err/队列满/负缓存均有回包），与既有阻塞 fetch 相同，不加超时。
//!
//! 信任前提披露（DF-F1，2026-10-25）：上句「恰一回响应」在导航取消路径不成立——
//! browser `cancel_tab` 以 retain 丢弃在途 PendingFetch 且不补发 `FetchResponse`，
//! `drain_all` 的无超时 `recv` 遂存在挂起窗口；旧逐图阻塞收单张同型，属既有
//! 隐患（P21 入池），留作后续修复面，本披露不改任何可执行代码。

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Receiver;

use zero_protocol::IpcChannel;
use zero_protocol::message::{FetchParams, FetchResponseParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;

use crate::paint_export::ImageBatchFetcher;
use crate::runtime::DeferredCommand;

/// 渲染进程 → 浏览器 IPC 发送端（stdout）。与 `runtime::IpcOutbound` 同型别名。
type IpcOutbound = PipeTransport<std::io::Empty, Box<dyn std::io::Write + Send>>;

/// 图片批量取回器：先全部非阻塞 `issue`，首个 `resolve` 统一收齐（契约见 [`ImageBatchFetcher`]）。
pub struct IpcImageBatch<'a> {
    outbound: &'a mut IpcOutbound,
    inbound_rx: &'a Receiver<IpcMessage>,
    next_fetch_id: &'a mut u64,
    deferred: &'a mut VecDeque<DeferredCommand>,
    /// 已发起未收齐：request_id → resolved URL。
    pending: HashMap<u64, String>,
    /// 已收齐的 2xx 响应体：URL → body（发送失败/非 2xx 不入表，`resolve` → `None`）。
    collected: HashMap<String, Vec<u8>>,
    /// 统一收齐是否已完成（首个 `resolve` 触发一次）。
    drained: bool,
}

impl<'a> IpcImageBatch<'a> {
    /// 绑定 renderer IPC 通道构造批量取回器。
    pub fn new(
        outbound: &'a mut IpcOutbound,
        inbound_rx: &'a Receiver<IpcMessage>,
        next_fetch_id: &'a mut u64,
        deferred: &'a mut VecDeque<crate::runtime::DeferredCommand>,
    ) -> Self {
        Self {
            outbound,
            inbound_rx,
            next_fetch_id,
            deferred,
            pending: HashMap::new(),
            collected: HashMap::new(),
            drained: false,
        }
    }

    /// 统一收齐全部在途响应（首个 `resolve` 调用一次）。
    ///
    /// 终止条件：`pending` 清空（每请求恰一响应）或通道断开（放弃剩余）。
    fn drain_all(&mut self) {
        // 当前批次不消费的消息先移到局部队列，结束统一回填 `deferred`。若直接
        // 放回 `deferred`，下一轮会立刻 pop 出同一条消息，与 `ipc_fetch` 同理
        // 造成单核自旋。
        let mut skipped = VecDeque::new();
        while !self.pending.is_empty() {
            let msg = if let Some(cmd) = self.deferred.pop_front() {
                match cmd {
                    // 扣压 POST 导航意图与本批无关：同入 skipped 原序回填（T27 返修3）。
                    DeferredCommand::PostNav(_) => {
                        skipped.push_back(cmd);
                        continue;
                    }
                    DeferredCommand::Message(m) => *m,
                }
            } else {
                match self.inbound_rx.recv() {
                    Ok(m) => m,
                    // IPC 断开：放弃剩余在途，未收 URL 由 `resolve` 返 None。
                    Err(_) => break,
                }
            };
            match msg.kind {
                IpcMessageKind::FetchResponse(FetchResponseParams {
                    request_id: rid,
                    status_code,
                    body,
                    headers,
                }) => {
                    if let Some(url) = self.pending.remove(&rid) {
                        // 非 2xx 与旧路径阻塞 `fetch_get(..).ok()` 同效果：None（跳过该图）。
                        if (200..300).contains(&status_code) {
                            self.collected.insert(url, body);
                        }
                    } else {
                        // 非本批请求的响应（如 AsyncPageLoad inflight）：原样暂存回填。
                        skipped.push_back(DeferredCommand::Message(Box::new(IpcMessage {
                            id: msg.id,
                            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                                request_id: rid,
                                status_code,
                                body,
                                headers,
                            }),
                        })));
                    }
                }
                IpcMessageKind::Heartbeat => {
                    let reply = IpcMessage {
                        id: msg.id,
                        kind: IpcMessageKind::Heartbeat,
                    };
                    if self.outbound.send(reply).is_err() {
                        break;
                    }
                }
                other => {
                    skipped.push_back(DeferredCommand::Message(Box::new(IpcMessage {
                        id: msg.id,
                        kind: other,
                    })));
                }
            }
        }
        // T27 返修3（RC2-F1）：按到达序回填队首——skipped 是从队首扫出的条目（早于
        // 窗收场仍留在队列中的条目），须回到它们之前；append 会把早到者压到迟到者
        // 之后，重排到达序。
        for cmd in skipped.into_iter().rev() {
            self.deferred.push_front(cmd);
        }
    }
}

impl ImageBatchFetcher for IpcImageBatch<'_> {
    fn issue(&mut self, url: &str) {
        // 线型与既有阻塞 `fetch_get` 完全一致：GET、无附加头、无 body。
        let request_id = *self.next_fetch_id;
        *self.next_fetch_id += 1;
        let msg = IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchRequest(FetchParams {
                request_id,
                url: url.to_string(),
                method: "GET".to_string(),
                headers: Vec::new(),
                body: None,
            }),
        };
        // 发送失败静默（不注册 pending）：`resolve` 返 None，图片按既有宽松降级跳过。
        if self.outbound.send(msg).is_err() {
            return;
        }
        self.pending.insert(request_id, url.to_string());
    }

    fn resolve(&mut self, url: &str) -> Option<Vec<u8>> {
        if !self.drained {
            self.drain_all();
            self.drained = true;
        }
        self.collected.remove(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex, mpsc};

    use zero_protocol::message::{FetchResponseParams, SetViewportParams};

    /// outbound 字节捕获（同 `ipc_fetch` 测试先例的 SharedBuf 写法）。
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

    /// 逐帧拆解 outbound 字节（4 字节 LE 长度前缀 + bincode 载荷，transport 帧协议）。
    fn parse_frames(bytes: &[u8]) -> Vec<IpcMessage> {
        let mut messages = Vec::new();
        let mut offset = 0;
        while offset < bytes.len() {
            assert!(offset + 4 <= bytes.len(), "帧头不完整（offset={offset}）");
            let len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节帧头")) as usize;
            let end = offset + 4 + len;
            assert!(end <= bytes.len(), "帧体不完整（offset={offset} len={len}）");
            messages.push(zero_protocol::deserialize(&bytes[offset + 4..end]).expect("帧载荷反序列化"));
            offset = end;
        }
        messages
    }

    fn response(request_id: u64, status_code: u16, body: &[u8]) -> IpcMessage {
        IpcMessage {
            id: 0,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id,
                status_code,
                headers: Vec::new(),
                body: body.to_vec(),
            }),
        }
    }

    fn viewport_message() -> IpcMessage {
        IpcMessage {
            id: 11,
            kind: IpcMessageKind::SetViewport(SetViewportParams {
                width: 1024,
                height: 768,
                device_scale_factor: 1.0,
            }),
        }
    }

    /// P14 批量收包钉：三个 URL 全部 issue 后首个 resolve 统一收齐——200 入表、
    /// 404 → None；穿插的非本批消息（SetViewport/Heartbeat）不干扰按 request_id
    /// 归位，SetViewport 结束回填 `deferred`。
    #[test]
    fn batch_collects_responses_by_request_id_and_defers_unrelated() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred);
        batch.issue("https://pic.test/a.png");
        batch.issue("https://pic.test/b.png");
        batch.issue("https://pic.test/c.png");

        // 穿插序：无关消息 → b(200) → heartbeat → a(404) → c(200)。
        tx.send(viewport_message()).unwrap();
        tx.send(response(1, 200, b"B")).unwrap();
        tx.send(IpcMessage {
            id: 77,
            kind: IpcMessageKind::Heartbeat,
        })
        .unwrap();
        tx.send(response(0, 404, b"nope")).unwrap();
        tx.send(response(2, 200, b"C")).unwrap();
        drop(tx);

        assert_eq!(batch.resolve("https://pic.test/a.png"), None, "404 → None");
        assert_eq!(batch.resolve("https://pic.test/b.png"), Some(b"B".to_vec()));
        assert_eq!(batch.resolve("https://pic.test/c.png"), Some(b"C".to_vec()));
        drop(batch);
        assert_eq!(next_fetch_id, 3, "issue 三次各分配一个 request_id");
        // 非本批消息回填 deferred（不得吞）。
        assert!(matches!(
            deferred.pop_front().map(|cmd| match cmd {
                DeferredCommand::Message(m) => m.kind,
                DeferredCommand::PostNav(_) => panic!("图片批量钉无 PostNav 形态"),
            }),
            Some(IpcMessageKind::SetViewport(_))
        ));
        assert!(deferred.is_empty());
    }

    /// 非本批 FetchResponse（如 AsyncPageLoad inflight）必须**原样**回填——含
    /// headers（X-Zero-Final-Url 等语义在阻塞 fetch 等待循环依赖它）。
    #[test]
    fn batch_preserves_foreign_fetch_response_headers_in_deferred() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred);
        batch.issue("https://pic.test/a.png");
        tx.send(IpcMessage {
            id: 9,
            kind: IpcMessageKind::FetchResponse(FetchResponseParams {
                request_id: 99,
                status_code: 200,
                headers: vec![("X-Zero-Final-Url".to_string(), "https://final.test/".to_string())],
                body: b"foreign".to_vec(),
            }),
        })
        .unwrap();
        tx.send(response(0, 200, b"A")).unwrap();
        drop(tx);

        assert_eq!(batch.resolve("https://pic.test/a.png"), Some(b"A".to_vec()));
        let Some(kind) = deferred.pop_front().map(|cmd| match cmd {
            DeferredCommand::Message(m) => m.kind,
            DeferredCommand::PostNav(_) => panic!("图片批量钉无 PostNav 形态"),
        }) else {
            panic!("外来 FetchResponse 必须回填 deferred");
        };
        let IpcMessageKind::FetchResponse(params) = kind else {
            panic!("回填的应是 FetchResponse");
        };
        assert_eq!(params.request_id, 99);
        assert_eq!(params.body, b"foreign".to_vec());
        assert_eq!(
            params.headers,
            vec![("X-Zero-Final-Url".to_string(), "https://final.test/".to_string())]
        );
    }

    /// 未 issue 的 URL resolve → None（首 resolve 触发 drain 也不得挂起/误收）。
    #[test]
    fn batch_resolve_unissued_url_returns_none_without_hang() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (_tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred);
        assert_eq!(batch.resolve("https://pic.test/never-issued.png"), None);
    }

    /// 通道断开（browser 侧消失）：drain 放弃剩余在途，未收 URL → None，不得挂死。
    #[test]
    fn batch_drain_abandons_pending_on_channel_disconnect() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred);
        batch.issue("https://pic.test/a.png");
        batch.issue("https://pic.test/b.png");
        tx.send(response(0, 200, b"A")).unwrap();
        drop(tx); // b 的响应永不到达

        assert_eq!(batch.resolve("https://pic.test/a.png"), Some(b"A".to_vec()));
        assert_eq!(batch.resolve("https://pic.test/b.png"), None, "IPC 断开放弃剩余");
    }

    /// 线型钉（TV-F2/TV-F5）：`issue` 实际发出的帧经反序列化后与阻塞 `fetch_get`
    /// 同线型——FetchRequest、request_id 顺序对应 pending 登记、GET、无附加头、
    /// 无 body；drain 期间的 Heartbeat 必须就地回包且 id 回显（帧序：请求×N → 回包）。
    #[test]
    fn batch_issue_wire_matches_blocking_fetch_get_and_heartbeat_reply_echoes_id() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 41_u64;
        let mut deferred = VecDeque::new();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred);
        batch.issue("https://pic.test/a.png");
        batch.issue("https://pic.test/b.png");
        tx.send(IpcMessage {
            id: 77,
            kind: IpcMessageKind::Heartbeat,
        })
        .unwrap();
        tx.send(response(41, 200, b"A")).unwrap();
        tx.send(response(42, 200, b"B")).unwrap();
        batch.issue("https://pic.test/c.png");
        tx.send(response(43, 200, b"C")).unwrap();
        drop(tx);

        assert_eq!(batch.resolve("https://pic.test/a.png"), Some(b"A".to_vec()));

        // outbound 实际字节恰 4 帧：3 个 FetchRequest + 1 个 Heartbeat 回包。
        let frames = parse_frames(&buf.0.lock().unwrap());
        assert_eq!(frames.len(), 4, "3 请求 + 1 心跳回包，无其他帧");
        for (index, url) in [
            "https://pic.test/a.png",
            "https://pic.test/b.png",
            "https://pic.test/c.png",
        ]
        .into_iter()
        .enumerate()
        {
            let IpcMessageKind::FetchRequest(params) = &frames[index].kind else {
                panic!("第 {index} 帧应是 FetchRequest，实际 {:?}", frames[index].kind);
            };
            assert_eq!(params.request_id, 41 + index as u64, "request_id 顺序对应 pending 登记");
            assert_eq!(params.url, url);
            assert_eq!(params.method, "GET");
            assert!(params.headers.is_empty(), "无附加头（阻塞 fetch_get 同线型）");
            assert_eq!(params.body, None);
        }
        assert_eq!(frames[3].id, 77, "Heartbeat 回包 id 回显");
        assert!(matches!(frames[3].kind, IpcMessageKind::Heartbeat));
        assert_eq!(next_fetch_id, 44);
    }
}
