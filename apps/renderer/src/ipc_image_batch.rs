//! 发布路径图片批量取回（P14，2026-10-25；K1/T98 持久化非阻塞化，2026-11-23）。
//!
//! [`crate::paint_export::fetch_image_payloads_with_cache`] 的两阶段宿主实现：
//! `issue` 先把全部缺图以与既有阻塞 fetch（runtime `fetch_get` 线型）完全相同的
//! （GET、无附加头——资源类型/优先级由 browser 侧按 URL 推断，既有负缓存与
//! 安全检查语义不变）非阻塞发出；`resolve` **非阻塞**收取已到达的响应（通道
//! Empty 即返回——publish 不再内联等待缺图网络往返）。同 origin 并发由
//! zero-net 调度器按每来源上限展开——旧逐发逐收形态下 renderer 侧恒
//! max-in-flight=1（cnblogs 107 请求 106 串行的第一根因）。
//!
//! K1（T98，2026-11-23）持久化语义：`pending`/`collected` 移入
//! [`ImageBatchState`] 跨 publish 保留（此前每批一次性、首个 `resolve` 经
//! `drain_all` 阻塞收齐全部在途——load 完成拍 publish 内联等待全部缺图往返，
//! T97 定谳的 publish 重事件刀位：cnblogs 重事件 281ms/baidu 92ms）。publish
//! 拍未到达的缺图由 runtime 主循环 tick（`tick_image_batch`）非阻塞收取，新
//! 收齐未发送过的图片即重发布——渐进 pop-in，终帧携带全部图片。
//!
//! 收包 defer 语义与既有阻塞 fetch 等待循环一致：非本批消息暂存局部队列、
//! 结束按到达序回填 `deferred`（不得吞 AsyncPageLoad 等在途 fetch 的响应）；
//! Heartbeat 就地回包；IPC 断开放弃剩余在途（对应 URL `resolve` 返回 `None`，
//! 图片按既有宽松降级跳过）。信任 browser 对每个请求恰回一个 `FetchResponse`
//! （错误路径 immediate_err/队列满/负缓存均有回包），与既有阻塞 fetch 相同，
//! 不加超时。
//!
//! 信任前提披露（DF-F1，2026-10-25）：上句「恰一回响应」在导航取消路径不成立
//! ——browser `cancel_tab` 以 retain 丢弃在途 PendingFetch 且不补发
//! `FetchResponse`。阻塞 `drain_all` 时代该缺口是无超时 `recv` 挂起窗（P21 入
//! 池）；非阻塞化后收窄为残余：被丢弃请求的 pending 条目仅余每 tick 一次空
//! drain（`try_recv` Empty 即返回），无挂起，导航复位 [`ImageBatchState::reset`]
//! 兜底清空。

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{Receiver, TryRecvError};

use zero_protocol::IpcChannel;
use zero_protocol::message::{FetchParams, FetchResponseParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;

use crate::paint_export::ImageBatchFetcher;
use crate::runtime::DeferredCommand;

/// 渲染进程 → 浏览器 IPC 发送端（stdout）。与 `runtime::IpcOutbound` 同型别名。
type IpcOutbound = PipeTransport<std::io::Empty, Box<dyn std::io::Write + Send>>;

/// 持久化图片批量取回状态（K1/T98）：跨 publish 保留在途请求与已收齐响应体；
/// 导航时 [`ImageBatchState::reset`] 清空（runtime `run_staged_load`/`navigate_with`）。
#[derive(Default)]
pub struct ImageBatchState {
    /// 已发起未收齐：request_id → resolved URL。
    pending: HashMap<u64, String>,
    /// 已收齐的 2xx 响应体：URL → body（发送失败/非 2xx 不入表，`resolve` → `None`）。
    collected: HashMap<String, Vec<u8>>,
}

impl ImageBatchState {
    /// 导航复位：清空在途与已收齐。旧文档缺图请求随文档废弃；其迟到响应按未知
    /// request_id 落主循环 dispatch 的 `FetchResponse(_)` 忽略臂（无副作用）。
    pub fn reset(&mut self) {
        self.pending.clear();
        self.collected.clear();
    }

    /// 是否存在在途缺图（runtime 主循环 tick 门）。
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

/// 图片批量取回器：先全部非阻塞 `issue`，`resolve` 非阻塞收取已到达响应
/// （契约见 [`ImageBatchFetcher`]）；状态存 [`ImageBatchState`] 跨 publish 保留。
pub struct IpcImageBatch<'a> {
    outbound: &'a mut IpcOutbound,
    inbound_rx: &'a Receiver<IpcMessage>,
    next_fetch_id: &'a mut u64,
    deferred: &'a mut VecDeque<DeferredCommand>,
    state: &'a mut ImageBatchState,
}

impl<'a> IpcImageBatch<'a> {
    /// 绑定 renderer IPC 通道与持久状态构造批量取回器。
    pub fn new(
        outbound: &'a mut IpcOutbound,
        inbound_rx: &'a Receiver<IpcMessage>,
        next_fetch_id: &'a mut u64,
        deferred: &'a mut VecDeque<crate::runtime::DeferredCommand>,
        state: &'a mut ImageBatchState,
    ) -> Self {
        Self {
            outbound,
            inbound_rx,
            next_fetch_id,
            deferred,
            state,
        }
    }

    /// 是否存在在途缺图（状态透传；测试断言用——批持有 `&mut state`，测试无法
    /// 直达 state）。
    #[cfg(test)]
    pub(crate) fn has_pending(&self) -> bool {
        self.state.has_pending()
    }

    /// 非阻塞收取已到达的在途响应（publish 的 `resolve` 与 runtime tick 共用）。
    /// 返回本轮新收齐的 2xx 响应 URL（到达序）；已 sent 图片的到达由调用方据此
    /// 决定是否重发布。终止条件：在途清空、通道 Empty（在途保留，后续
    /// tick/`resolve` 续收）或断开（放弃全部在途）。
    pub(crate) fn drain_ready(&mut self) -> Vec<String> {
        if self.state.pending.is_empty() {
            return Vec::new();
        }
        // 当前批次不消费的消息先移到局部队列，结束统一回填 `deferred`。若直接
        // 放回 `deferred`，下一轮会立刻 pop 出同一条消息，与 `ipc_fetch` 同理
        // 造成单核自旋。
        let mut skipped = VecDeque::new();
        let mut arrived = Vec::new();
        while !self.state.pending.is_empty() {
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
                match self.inbound_rx.try_recv() {
                    Ok(m) => m,
                    // 在途响应未到：保留 pending，publish 帧/本 tick 先行（K1 渐进 pop-in）。
                    Err(TryRecvError::Empty) => break,
                    // IPC 断开：放弃剩余在途，未收 URL 由 `resolve` 返 None。
                    Err(TryRecvError::Disconnected) => {
                        self.state.pending.clear();
                        break;
                    }
                }
            };
            match msg.kind {
                IpcMessageKind::FetchResponse(FetchResponseParams {
                    request_id: rid,
                    status_code,
                    body,
                    headers,
                }) => {
                    if let Some(url) = self.state.pending.remove(&rid) {
                        // 非 2xx 与旧路径阻塞 `fetch_get(..).ok()` 同效果：None（跳过该图）。
                        if (200..300).contains(&status_code) {
                            self.state.collected.insert(url.clone(), body);
                            arrived.push(url);
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
        arrived
    }
}

impl ImageBatchFetcher for IpcImageBatch<'_> {
    fn issue(&mut self, url: &str) {
        // K1 持久化去重：同 URL 已在途或已收齐不重发——重发布 PASS1 对同文档
        // 同图不产生重复 FetchRequest（旧每批一次性形态天然去重，持久化后须显式）。
        if self.state.pending.values().any(|u| u == url) || self.state.collected.contains_key(url) {
            return;
        }
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
        self.state.pending.insert(request_id, url.to_string());
    }

    fn resolve(&mut self, url: &str) -> Option<Vec<u8>> {
        self.drain_ready();
        self.state.collected.remove(url)
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

    /// 批量收包钉：三个 URL 全部 issue 后 `resolve` 非阻塞收取——预置全队列已就绪
    /// 故一次收齐：200 入表、404 → None；穿插的非本批消息（SetViewport/Heartbeat）
    /// 不干扰按 request_id 归位，SetViewport 结束回填 `deferred`。
    #[test]
    fn batch_collects_responses_by_request_id_and_defers_unrelated() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        {
            let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
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
        }
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
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
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

    /// 未 issue 的 URL resolve → None（pending 空 early-return，不触队列扫描/挂起）。
    #[test]
    fn batch_resolve_unissued_url_returns_none_without_hang() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (_tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        assert_eq!(batch.resolve("https://pic.test/never-issued.png"), None);
    }

    /// 通道断开（browser 侧消失）：drain 放弃剩余在途，未收 URL → None，不得挂死。
    #[test]
    fn batch_drain_abandons_pending_on_channel_disconnect() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        batch.issue("https://pic.test/a.png");
        batch.issue("https://pic.test/b.png");
        tx.send(response(0, 200, b"A")).unwrap();
        drop(tx); // b 的响应永不到达

        assert_eq!(batch.resolve("https://pic.test/a.png"), Some(b"A".to_vec()));
        assert_eq!(batch.resolve("https://pic.test/b.png"), None, "IPC 断开放弃剩余");
        assert!(!state.has_pending(), "断开放弃后 pending 清空（tick 门收口）");
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
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
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

    /// K1 核心钉：publish 拍响应未到 → `resolve` 返 None（帧先行发布，图片渐进
    /// pop-in，不内联等待）；在途 pending 跨 resolve 保留，响应到达后下一次
    /// `resolve` 零等待收取（重发布 pop-in 路径）。
    #[test]
    fn resolve_returns_none_before_response_arrives_and_collects_later() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        batch.issue("https://pic.test/a.png");

        assert_eq!(
            batch.resolve("https://pic.test/a.png"),
            None,
            "响应未到 → None（非阻塞）"
        );
        assert!(batch.has_pending(), "在途保留跨 publish");

        tx.send(response(0, 200, b"A")).unwrap();
        assert_eq!(
            batch.resolve("https://pic.test/a.png"),
            Some(b"A".to_vec()),
            "到达后零等待收取"
        );
        assert!(!batch.has_pending());
    }

    /// K1 去重钉：同 URL 已在途或已收齐时 `issue` 不再发 FetchRequest（重发布
    /// PASS1 对同文档同图不重复请求，线型帧数不增）。
    #[test]
    fn issue_dedupes_url_already_pending_or_collected() {
        let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(buf.clone()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        batch.issue("https://pic.test/a.png");
        tx.send(response(0, 200, b"A")).unwrap();
        assert_eq!(batch.resolve("https://pic.test/a.png"), Some(b"A".to_vec()));

        // 在途去重：b 已 issue 未收齐，重复 issue 不新增帧（collected 取走后的
        // 重新 issue 属既有每 publish 重取语义，不在此钉）。
        batch.issue("https://pic.test/b.png");
        batch.issue("https://pic.test/b.png");
        tx.send(response(1, 200, b"B")).unwrap();
        assert_eq!(batch.resolve("https://pic.test/b.png"), Some(b"B".to_vec()));

        let frames = parse_frames(&buf.0.lock().unwrap());
        assert_eq!(frames.len(), 2, "重复 issue 不产生重复 FetchRequest");
        assert_eq!(next_fetch_id, 2, "重复 issue 不消耗 request_id");
    }

    /// K1 tick 面：`drain_ready` 返回本轮新收齐 URL（到达序，404 不计入）；
    /// 404 是终态（从 pending 移除不重试，与阻塞 `fetch_get` 同语义）；「响应
    /// 永不到达」（DF-F1 残余形态）的条目保留在途。
    #[test]
    fn drain_ready_reports_arrivals_and_preserves_inflight() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        batch.issue("https://pic.test/a.png");
        batch.issue("https://pic.test/b.png");
        batch.issue("https://pic.test/c.png");
        batch.issue("https://pic.test/d.png");

        tx.send(response(0, 200, b"A")).unwrap();
        tx.send(response(1, 404, b"nope")).unwrap();
        assert_eq!(
            batch.drain_ready(),
            vec!["https://pic.test/a.png".to_string()],
            "404 不计到达"
        );
        assert!(batch.has_pending(), "c/d 在途保留");

        tx.send(response(2, 200, b"C")).unwrap();
        assert_eq!(batch.drain_ready(), vec!["https://pic.test/c.png".to_string()]);
        assert_eq!(
            batch.resolve("https://pic.test/b.png"),
            None,
            "404 终态：None 且不入 collected"
        );
        assert!(batch.has_pending(), "d 永不到达（DF-F1 残余形态）：pending 保留");
    }

    /// K1 导航复位钉：`reset` 清空在途与已收齐；废弃请求的迟到响应按未知
    /// request_id 原样回填 `deferred`（落主循环忽略臂，不误收、不丢）。
    #[test]
    fn reset_clears_pending_and_late_responses_stay_foreign() {
        let mut outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()) as Box<dyn Write + Send>);
        let (tx, rx) = mpsc::channel();
        let mut next_fetch_id = 0;
        let mut deferred = VecDeque::new();
        let mut state = ImageBatchState::default();
        {
            let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
            batch.issue("https://pic.test/a.png");
            assert_eq!(batch.resolve("https://pic.test/a.png"), None);
            assert!(batch.has_pending());
        }

        state.reset();
        assert!(!state.has_pending(), "复位清空在途");

        // 复位后的新 publish 批：旧文档请求（rid 0）的迟到响应与新请求（rid 1）
        // 响应混到——只收 rid 1，迟到响应原样回填 deferred（未知 id 落主循环忽略臂）。
        tx.send(response(0, 200, b"LATE")).unwrap();
        tx.send(response(1, 200, b"B")).unwrap();
        drop(tx);
        let mut batch = IpcImageBatch::new(&mut outbound, &rx, &mut next_fetch_id, &mut deferred, &mut state);
        batch.issue("https://pic.test/b.png");
        assert_eq!(
            batch.resolve("https://pic.test/b.png"),
            Some(b"B".to_vec()),
            "复位后不误收旧请求"
        );
        let Some(IpcMessageKind::FetchResponse(params)) = deferred.pop_front().map(|cmd| match cmd {
            DeferredCommand::Message(m) => m.kind,
            DeferredCommand::PostNav(_) => panic!("图片批量钉无 PostNav 形态"),
        }) else {
            panic!("迟到响应必须回填 deferred");
        };
        assert_eq!(params.request_id, 0);
        assert_eq!(params.body, b"LATE".to_vec());
    }
}
