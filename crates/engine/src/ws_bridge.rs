//! WebSocket bridge——共享于 browser `tab_js_worker` / renderer `js_worker` 与 webview 进程内路径。
//!
//! 形态对齐 [`crate::fetch_bridge::FetchBridge`]：本模块承载「回调注册 + 事件路由 + 连接数
//! gate + 生产宿主实现」。生产 socket（`zero_net::WebSocket` + 专用线程读泵）由
//! [`default_net_ws_host`] 提供——`zero-engine` 已依赖 `zero-net`（fetch 同款），app 侧仅经
//! `SetWsHandler` 命令 arm（factory 注入，解 host↔bridge 的 emitter chicken-and-egg）或
//! webview config 接线；自定义宿主可实现 [`WsHost`] 替换（测试合成宿主同此口）。
//!
//! **事件回推通道**：复用 `__zw_pending` Promise 通道（P1b S1）。shim 侧每连接串行泵——
//! `__zw_ws_next(connId, pid)` 创建一条 pending Promise，宿主连接线程经 [`WsEmitter::emit`]
//! 投递事件：有 pending → 立即 resolve；无 → 入每连接事件队列（串行泵保证至多 1 条 pending，
//! 队列兜底服务端先发消息的时序）。
//!
//! **安全/资源边界**：
//! - 并发/长连接预算：open 及连接中（握手进行中）连接数上限（[`MAX_OPEN_WS_CONNECTIONS`]）
//!   由宿主实现持注册表负责（默认 [`NetWsHost`] 内置）。t8k 返修删除了原「connect wrapper
//!   线程 + 16 槽 gate」组合：connect 本身非阻塞（trait 契约），wrapper 只把连接代际捕获
//!   推迟到 wrapper 调度点（跨文档 reset 下错标代际、close-before-insert 仲裁丢失），gate
//!   计数的是 wrapper 生命周期、从未真正钳制握手并发——握手并发现由 [`MAX_OPEN_WS_CONNECTIONS`]
//!   槽位上限统一承载（连接中槽位同样计入）。
//! - wire 字段过滤：`reason`/`protocol`/`error` 消息剥离 `\x1f`/`\x1e` 控制分隔符（网络输入
//!   不可携带 wire 元字符）；text/binary 数据字段恒为末字段（取首个分隔符后全部，与 fetch
//!   body 约定一致，数据可含 `\x1f`）。
//! - CSP：webview 进程内路径在 host 实现处按 `connect-src` 校验（与 webview fetch 同位）；
//!   renderer/tab 生产路径与既有页面 fetch 一致（当前无 CSP 接线——既有设计缺口，如实记录）。

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};

use crate::async_resolver::AsyncResolver;
use zero_script_sandbox::Sandbox;

/// 宿主→JS 事件（[`WsEmitter::emit`] 输入；wire 序列化见 [`serialize_event`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsEvent {
    /// 握手完成（连接打开）。`protocol` = 服务端选中的子协议（未协商为空串）。
    Open {
        /// 服务端选中的子协议（未协商为空串）。
        protocol: String,
    },
    /// 文本消息。
    Text(String),
    /// 二进制消息。
    Binary(Vec<u8>),
    /// 连接层错误（其后必随 Close——err 只驱动 `onerror`，状态终结由 close 承载）。
    Error(String),
    /// 宿主已把 `send()` 数据写入网络（回投 JS 递减 `bufferedAmount`）。
    Sent {
        /// 已写出字节数。
        bytes: usize,
    },
    /// 连接关闭。`clean` = 是否完成关闭握手（本地 close/服务端 Close 帧 = true；IO 错误 = false）。
    Close {
        /// close 状态码（RFC 6455 §7.4；1005/1006 为合成码——wire 上禁止）。
        code: u16,
        /// 关闭原因（服务端 close 帧载荷或合成说明；已过 wire 元字符过滤）。
        reason: String,
        /// 是否完成关闭握手（本地 close/服务端 Close 帧 = true；IO 错误/策略终止 = false）。
        clean: bool,
    },
}

/// JS→宿主数据（`send`）。文本经 text wire；字节经 csv-decimal wire（R3020 同型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsData {
    /// 文本（`ws.send("...")`）。
    Text(String),
    /// 字节（`ws.send(ArrayBuffer/TypedArray)`）。
    Bytes(Vec<u8>),
}

/// 生产 socket 宿主——app 层实现（`zero_net::WebSocket` + 每连接专用线程读泵）。
///
/// 三个方法都**不得阻塞 JS worker 线程**：`connect` 的阻塞 TCP/TLS/握手与读泵放宿主自己的
/// 线程；`send`/`close` 只入宿主内部通道立即返回。事件经 [`WsEmitter`] 回投。
pub trait WsHost: Send + Sync {
    /// 发起连接（异步）：成功后宿主须依次 emit `Open`…；失败 emit `Error` + `Close{clean:false}`。
    ///
    /// **必须非阻塞**（阻塞 TCP/TLS/握手与读泵放宿主自有线程），且实现须假定本方法由
    /// JS 回调线程**同步**调用——连接的导航代际在入口捕获，bridge 侧不得再包 wrapper
    /// 线程推迟调用点（t8k 返修实证：wrapper 调度延迟晚于 `reset_context` 时代际错标、
    /// close-before-insert 仲裁丢失）。
    fn connect(&self, id: &str, url: &str, protocols: &[String], origin: &str, cookie: &str);
    /// 发送数据（连接不存在/未 open 时静默忽略——JS 侧已按 readyState 前置拦截）。
    fn send(&self, id: &str, data: &WsData);
    /// 关闭连接（`code` 0 = 不带状态码；宿主发 Close 帧后 emit `Close`）。
    fn close(&self, id: &str, code: u16, reason: &str);
}

/// 单元分隔符（与 fetch_bridge 同约定）/ 记录分隔符（预留）。
const FIELD_SEP: char = '\x1f';
/// 每连接事件队列上限——溢出时桥主动合成 `Close{1008,"event-queue-overflow"}` 并丢弃后续
/// （病态服务端洪水不耗内存；正常页面远不可达）。宿主收到 overflow close 自行收线。
const EVENT_QUEUE_CAP: usize = 4096;

/// wire 元字符过滤：`\x1f`/`\x1e` 是事件 wire 的字段/记录分隔符，网络输入（服务端 close
/// reason、子协议、错误消息）不得携带——剥除。
fn sanitize_field(s: &str) -> String {
    s.chars().filter(|&c| c != FIELD_SEP && c != '\x1e').collect()
}

/// 字节 → csv-decimal（R3020 同型；空 → 空串）。
fn encode_bytes_csv(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 4);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&b.to_string());
    }
    s
}

/// csv-decimal → 字节；malformed（超 u8/非数字）→ None。
fn decode_bytes_csv(wire: &str) -> Option<Vec<u8>> {
    if wire.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in wire.split(',') {
        out.push(part.parse::<u8>().ok()?);
    }
    Some(out)
}

/// 序列化 [`WsEvent`] 为 host→JS wire（经 `__zw_ws_next` 的 pending Promise resolve 回投）。
///
/// 数据字段（text/binary 内容）恒为末字段：`msg\x1f<text…可含 \x1f>` / `bin\x1f<csv>`；
/// 结构字段（protocol/ reason/ code）经 [`sanitize_field`] 过滤。
pub fn serialize_event(ev: &WsEvent) -> String {
    match ev {
        WsEvent::Open { protocol } => format!("open{FIELD_SEP}{}", sanitize_field(protocol)),
        WsEvent::Text(t) => format!("msg{FIELD_SEP}{t}"),
        WsEvent::Binary(b) => format!("bin{FIELD_SEP}{}", encode_bytes_csv(b)),
        WsEvent::Error(m) => format!("err{FIELD_SEP}{}", sanitize_field(m)),
        WsEvent::Sent { bytes } => format!("sent{FIELD_SEP}{bytes}"),
        WsEvent::Close { code, reason, clean } => {
            format!(
                "close{FIELD_SEP}{code}{FIELD_SEP}{}{FIELD_SEP}{}",
                if *clean { "1" } else { "0" },
                sanitize_field(reason)
            )
        }
    }
}

/// 解码 wire 数据字段为 [`WsData`]（`__zw_ws_send` 回调用）：`t\x1f<text…>` / `b\x1f<csv>`。
/// 未知 kind / malformed csv → None（调用方静默忽略）。
pub fn decode_send_data(kind: &str, wire: &str) -> Option<WsData> {
    match kind {
        "t" => Some(WsData::Text(wire.to_string())),
        "b" => decode_bytes_csv(wire).map(WsData::Bytes),
        _ => None,
    }
}

/// 事件路由核心：每连接事件队列 + pending-next（resolveId）登记 + 导航代际。
///
/// 代际（t8k defect-r1 D1）：renderer 路径跨文档导航 `reset_context` 重建 JS 上下文后
/// shim 连接计数器（`_zwWsConnSeq`）归零，而本核心与宿主连接表以 worker 生命周期存活——
/// 新文档首个连接（同名 `ws1`）会撞上旧文档残留的 pending/队列/泵线程。`generation` 在
/// [`WsBridge::reset_generation`] 推进：旧 pending/队列清空，旧泵线程残余事件凭代际戳
/// 在 [`WsEmitter::emit_gen`] 栅栏处丢弃。
#[derive(Default)]
struct WsCore {
    queues: HashMap<String, VecDeque<WsEvent>>,
    pending: HashMap<String, VecDeque<String>>,
    /// 导航代际（`reset_generation` 递增）；连接事件带建立时代际戳，不匹配即弃。
    generation: u64,
}

/// 事件投递柄——宿主实现持有（`WsBridge::emitter()`），从任意线程回投事件。
#[derive(Clone)]
pub struct WsEmitter {
    core: Arc<Mutex<WsCore>>,
    resolver: AsyncResolver,
    host: Arc<Mutex<Option<Weak<dyn WsHost>>>>,
}

impl WsEmitter {
    /// 绑定宿主——事件队列溢出时调用 [`WsHost::close`] 收线。
    pub fn bind_host(&self, host: Arc<dyn WsHost>) {
        if let Ok(mut cell) = self.host.lock() {
            *cell = Some(Arc::downgrade(&host));
        }
    }

    fn teardown_on_overflow(&self, conn_id: &str) {
        if let Ok(cell) = self.host.lock()
            && let Some(weak) = cell.as_ref()
            && let Some(h) = weak.upgrade()
        {
            h.close(conn_id, 1008, "event-queue-overflow");
        }
    }
}

impl WsEmitter {
    /// 当前导航代际（[`WsCore::generation`]）——宿主在连接建立时捕获作事件戳。
    fn current_generation(&self) -> u64 {
        self.core.lock().expect("ws core lock").generation
    }

    /// 推进导航代际并清空 pending/队列（跨文档导航 `reset_context` 后调用）：旧文档的
    /// 残留 pending（未 resolve 的 pid 指向已销毁上下文）与积压消息随之作废；仍在运行的
    /// 旧泵线程后续 emit 因代际不匹配被 [`WsEmitter::emit_gen`] 丢弃。
    fn reset_generation(&self) {
        let mut core = self.core.lock().expect("ws core lock");
        core.generation += 1;
        core.pending.clear();
        core.queues.clear();
    }

    /// 投递一条连接事件（捕获当前代际——JS 回调线程同步路径用）。
    pub fn emit(&self, conn_id: &str, ev: WsEvent) {
        let nav_gen = self.current_generation();
        self.emit_gen(conn_id, ev, nav_gen);
    }

    /// 投递一条带代际戳的连接事件：代际不匹配（`reset_generation` 后仍在运行的旧泵线程）
    /// → 静默丢弃，不入新文档队列；有 pending-next → 立即 resolve；否则入队（超
    /// [`EVENT_QUEUE_CAP`] 合成 overflow close 并丢弃本事件）。
    fn emit_gen(&self, conn_id: &str, ev: WsEvent, nav_gen: u64) {
        let wire = serialize_event(&ev);
        let (pid, overflow_teardown) = {
            let mut core = self.core.lock().expect("ws core lock");
            if core.generation != nav_gen {
                // 旧代际残余事件（reset 后未退出的泵线程）——丢弃，不入新文档队列。
                return;
            }
            let mut overflow_teardown = false;
            let pid = if let Some(pids) = core.pending.get_mut(conn_id) {
                if let Some(pid) = pids.pop_front() {
                    Some(pid)
                } else {
                    core.pending.remove(conn_id);
                    let q = core.queues.entry(conn_id.to_string()).or_default();
                    let overflow = q.len() >= EVENT_QUEUE_CAP;
                    if overflow {
                        // 队列上限：连接被策略性终止——积压消息随连接作废，清队列并把合成
                        // close 置于队首（泵立刻见到 close，不再吐 4096 条过期消息）。
                        q.clear();
                        q.push_back(WsEvent::Close {
                            code: 1008,
                            reason: "event-queue-overflow".to_string(),
                            clean: false,
                        });
                        overflow_teardown = true;
                    } else {
                        q.push_back(ev);
                    }
                    None
                }
            } else {
                let q = core.queues.entry(conn_id.to_string()).or_default();
                let overflow = q.len() >= EVENT_QUEUE_CAP;
                if overflow {
                    q.clear();
                    q.push_back(WsEvent::Close {
                        code: 1008,
                        reason: "event-queue-overflow".to_string(),
                        clean: false,
                    });
                    overflow_teardown = true;
                } else {
                    q.push_back(ev);
                }
                None
            };
            (pid, overflow_teardown)
        };
        if overflow_teardown {
            self.teardown_on_overflow(conn_id);
        }
        if let Some(pid) = pid {
            self.resolver.resolve(&pid, &wire);
        }
    }

    /// pending-next 到达（`__zw_ws_next`）：队列非空 → 立即 resolve 队首；否则登记 pending。
    fn next(&self, conn_id: &str, pid: &str) {
        let wire = {
            let mut core = self.core.lock().expect("ws core lock");
            match core.queues.get_mut(conn_id) {
                Some(q) if !q.is_empty() => {
                    let ev = q.pop_front().expect("queue non-empty");
                    if q.is_empty() {
                        core.queues.remove(conn_id);
                    }
                    Some(serialize_event(&ev))
                }
                _ => {
                    core.pending
                        .entry(conn_id.to_string())
                        .or_default()
                        .push_back(pid.to_string());
                    None
                }
            }
        };
        if let Some(wire) = wire {
            self.resolver.resolve(pid, &wire);
        }
    }
}

/// WebSocket bridge——注册 `__zw_ws_connect/send/close/next` 回调 + 事件路由。
///
/// 各宿主构造（传入自身 [`AsyncResolver`]）→ `register(sandbox)` 注回调 → `SetWsHandler`
/// 命令 arm（或 webview config）`set_host` 注入生产实现。宿主构造需要 emitter 时经
/// [`WsBridge::emitter`] 取（host 与 bridge 互相持有的 chicken-and-egg 用两步构造解）。
pub struct WsBridge {
    emitter: WsEmitter,
    host_cell: Arc<Mutex<Option<Arc<dyn WsHost>>>>,
}

impl WsBridge {
    /// 构造——`resolver` 用于 pending Promise resolve（复用 P1b S1 通路）。
    pub fn new(resolver: AsyncResolver) -> Self {
        Self {
            emitter: WsEmitter {
                core: Arc::new(Mutex::new(WsCore::default())),
                resolver,
                host: Arc::new(Mutex::new(None)),
            },
            host_cell: Arc::new(Mutex::new(None)),
        }
    }

    /// 事件投递柄（宿主实现构造时持有；可早于 `set_host` 克隆）。
    pub fn emitter(&self) -> WsEmitter {
        self.emitter.clone()
    }

    /// 注入生产宿主（app 的 `SetWsHandler` 命令 arm 调用；chicken-and-egg 解同 fetch）。
    pub fn set_host(&self, host: Arc<dyn WsHost>) {
        if let Ok(mut cell) = self.host_cell.lock() {
            *cell = Some(Arc::clone(&host));
        }
        self.emitter.bind_host(host);
    }

    /// 跨文档导航代际推进（t8k defect-r1 D1）：renderer `reset_context` 重建 JS 上下文后
    /// shim 连接计数器归零（`_zwWsConnSeq` 从 0 重来），而本桥的 WsCore 与宿主连接表以
    /// worker 生命周期存活——不推进代际，新文档首个连接（同名 `ws1`）的事件会被旧代际
    /// 残留 pending 吞掉、旧泵终态分支可误删新槽位、旧连接消息可串入新文档（WHATWG
    /// HTML：文档销毁其 WebSocket 连接随之作废）。reset 后：旧 pending/队列清空、旧泵
    /// 线程 emit 因代际不匹配丢弃、旧槽位由新 connect 覆盖（[`ConnSlot::nav_gen`] 保证
    /// 旧泵只删本代槽位）。tab worker（持久 context）与 webview（shim 幂等单装）路径无
    /// reset_context 调用点，不作废——FIXME(t8k/N1)：该两路径文档销毁仍不关闭 WS 连接。
    pub fn reset_generation(&self) {
        self.emitter.reset_generation();
    }

    /// 注册 4 个回调（均**非阻塞**，JS worker 线程绝不在 WS 路径上等待网络）：
    ///
    /// - `__zw_ws_connect(id, url, protocolsWire, origin, cookie)`：同步转发宿主（[`WsHost::connect`] 契约
    ///   非阻塞，网络 I/O 全在宿主自有线程；并发由宿主 [`MAX_OPEN_WS_CONNECTIONS`] 槽位
    ///   上限钳制）；宿主未注入 → 直接 emit `Error`+`Close`（fetch no-handler 同型，不悬挂）。
    /// - `__zw_ws_send(id, kind, dataWire)` / `__zw_ws_close(id, code, reason)`：转发宿主
    ///   （宿主内部通道，非阻塞）。
    /// - `__zw_ws_next(id, pid)`：pending Promise 登记（见 [`WsEmitter::next`]）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let emitter = self.emitter.clone();
        let host_cell = Arc::clone(&self.host_cell);
        sandbox.register_callback(
            "__zw_ws_connect",
            Box::new(move |args: &[String]| -> String {
                let id = args.first().cloned().unwrap_or_default();
                let url = args.get(1).cloned().unwrap_or_default();
                // protocolsWire = 逗号 join（spec 禁止 protocol 含逗号，逗号分隔无歧义）。
                let protocols: Vec<String> = args
                    .get(2)
                    .cloned()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_string)
                    .collect();
                let origin = args.get(3).map(String::as_str).unwrap_or("");
                let cookie = args.get(4).map(String::as_str).unwrap_or("");
                let host = host_cell.lock().ok().and_then(|c| c.as_ref().cloned());
                match host {
                    Some(h) => {
                        // 同步转发（`WsHost::connect` 契约非阻塞，网络 I/O 全在宿主自有
                        // 线程——JS worker 线程不在 WS 路径上等待网络）。t8k 返修：此处曾
                        // 把 connect 包进一层 wrapper 线程，连接代际捕获被推迟到 wrapper
                        // 调度点——晚于 reset_context 执行时旧文档连接被错标成新代际，且
                        // close 在插槽前查不到槽位丢失仲裁；connect 与 reset_context 同在
                        // JS worker 线程串行，直接调用后两个窗口均不存在。并发由宿主
                        // [`MAX_OPEN_WS_CONNECTIONS`] 槽位上限统一钳制。
                        h.connect(&id, &url, &protocols, origin, cookie);
                    }
                    None => {
                        // no-handler（宿主尚未注入/未配置）：fetch 同型——不悬挂，快速失败。
                        emitter.emit(&id, WsEvent::Error("no-handler".to_string()));
                        emitter.emit(
                            &id,
                            WsEvent::Close {
                                code: 1006,
                                reason: String::new(),
                                clean: false,
                            },
                        );
                    }
                }
                String::new()
            }),
        );

        let host_cell = Arc::clone(&self.host_cell);
        sandbox.register_callback(
            "__zw_ws_send",
            Box::new(move |args: &[String]| -> String {
                if let (Some(id), Some(kind), Some(data)) = (args.first(), args.get(1), args.get(2))
                    && let Some(d) = decode_send_data(kind, data)
                    && let Ok(cell) = host_cell.lock()
                    && let Some(h) = cell.as_ref()
                {
                    h.send(id, &d);
                }
                String::new()
            }),
        );

        let host_cell = Arc::clone(&self.host_cell);
        sandbox.register_callback(
            "__zw_ws_close",
            Box::new(move |args: &[String]| -> String {
                if let Some(id) = args.first() {
                    let code = args.get(1).and_then(|c| c.parse::<u16>().ok()).unwrap_or(0);
                    let reason = args.get(2).cloned().unwrap_or_default();
                    if let Ok(cell) = host_cell.lock()
                        && let Some(h) = cell.as_ref()
                    {
                        h.close(id, code, &reason);
                    }
                }
                String::new()
            }),
        );

        let emitter = self.emitter.clone();
        sandbox.register_callback(
            "__zw_ws_next",
            Box::new(move |args: &[String]| -> String {
                if let (Some(id), Some(pid)) = (args.first(), args.get(1)) {
                    emitter.next(id, pid);
                }
                String::new()
            }),
        );
    }
}

// ── 生产宿主实现（zero_net::WebSocket + 每连接专用线程读泵）──

/// 同宿主 open/连接中（握手进行中）连接数上限（guidelines #21 资源预算：WS 长连接占
/// FD+线程，比 fetch inflight 更贵；恶意页无法以极低成本堆积 socket）。连接中槽位同样
/// 计入——并发握手上限即本值。
const MAX_OPEN_WS_CONNECTIONS: usize = 128;
/// 读泵轮询间隔（阻塞 `receive()` 的读超时——超时返 WouldBlock/TimedOut，泵得以轮询发送通道）。
const WS_PUMP_POLL_MS: u64 = 50;

/// 泵线程发送命令（JS→宿主 send/close 经通道 marshal 到泵线程——泵独占 socket 写权，
/// 免互斥锁网络读写交织）。
enum HostCmd {
    Text(String),
    Bytes(Vec<u8>),
    Close { code: u16, reason: String },
}

/// 连接槽状态（close-during-connect 竞态仲裁： Connecting 中 close → 本地仲裁终结，
/// connect 线程握手完成时看到 Closed 即静默退出——**恰好一条 Close** 契约的保障点）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ConnState {
    Connecting,
    Open,
    Closed,
}

struct ConnSlot {
    state: ConnState,
    cmd_tx: std::sync::mpsc::Sender<HostCmd>,
    /// 建立时的导航代际（[`WsCore::generation`]）——同 id 跨文档重连时新槽位覆盖旧槽，
    /// 旧泵线程终态分支凭本戳只删本代槽位（t8k defect-r1 D1 概率性孤儿化臂）。
    nav_gen: u64,
}

/// 泵线程终态清理：只删除仍属本代际的槽位——同 id 已被新文档重连覆盖时不得误删
/// （t8k defect-r1 D1 概率性孤儿化：旧泵删新槽 → 新连接零事件孤儿化）。
fn remove_conn_if_current(conns: &std::sync::Mutex<HashMap<String, ConnSlot>>, id: &str, nav_gen: u64) {
    let mut conns = conns.lock().expect("ws conn table lock");
    if conns.get(id).is_some_and(|s| s.nav_gen == nav_gen) {
        conns.remove(id);
    }
}

/// 生产 WS 宿主：[`default_net_ws_host`] 产物。每连接一条专用泵线程（阻塞 connect →
/// 50ms 读超时轮询循环）；连接表持 [`ConnSlot`]（状态 + 命令通道）。
struct NetWsHost {
    emitter: WsEmitter,
    conns: Arc<Mutex<HashMap<String, ConnSlot>>>,
    cookie_store: Arc<Mutex<zero_net::CookieStore>>,
}

/// RFC 6265 / WHATWG WebSocket：握手 Cookie 取自 jar 中匹配连接 URL 的条目（含 HttpOnly），
/// 不用 `document.cookie`。`ws`/`wss` 按 `http`/`https` 参与 Secure 与域匹配。
fn ws_handshake_cookie(store: &zero_net::CookieStore, url: &str, document_origin: &str) -> String {
    let mut parsed = match zero_net::parse_url(url) {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    match parsed.scheme.as_str() {
        "ws" => parsed.scheme = "http".to_string(),
        "wss" => parsed.scheme = "https".to_string(),
        _ => {}
    }
    let doc_host = if document_origin.is_empty() || document_origin == "null" {
        None
    } else {
        zero_net::parse_url(document_origin)
            .ok()
            .and_then(|u| u.host)
            .or_else(|| {
                url::Url::parse(document_origin)
                    .ok()
                    .and_then(|u| u.host_str().map(str::to_string))
            })
    };
    let context = match (parsed.host.as_deref(), doc_host.as_deref()) {
        (Some(req), Some(doc)) => zero_net::request_context(req, doc),
        _ => zero_net::RequestContext::CrossSiteSubresource,
    };
    store.cookie_header_with_context(&parsed, context, true)
}

impl WsHost for NetWsHost {
    fn connect(&self, id: &str, url: &str, protocols: &[String], origin: &str, _cookie: &str) {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<HostCmd>();
        // 代际在本函数入口捕获。调用契约（见 [`WsHost`] trait 文档）：connect 非阻塞、
        // 且由 JS 回调线程同步调用——捕获与下方插槽插入之间没有跨线程窗口，
        // `reset_context`（同在 JS worker 线程串行执行）无法插队错标代际（t8k 返修：
        // 曾把 connect 包进一层 wrapper 线程、握手也在 wrapper 阻塞，捕获被推迟到
        // wrapper 调度点，可晚于 reset 而错标代际；且 close 在插槽前查不到槽位丢失
        // 仲裁——该 wrapper 已删除，阻塞握手移回泵线程）。
        let nav_gen = self.emitter.current_generation();
        {
            let mut conns = self.conns.lock().expect("ws conn table lock");
            if conns.len() >= MAX_OPEN_WS_CONNECTIONS {
                drop(conns);
                self.emitter
                    .emit(id, WsEvent::Error("too many open connections".to_string()));
                self.emitter.emit(
                    id,
                    WsEvent::Close {
                        code: 1013,
                        reason: "open-connection-cap".to_string(),
                        clean: false,
                    },
                );
                return;
            }
            conns.insert(
                id.to_string(),
                ConnSlot {
                    state: ConnState::Connecting,
                    cmd_tx,
                    nav_gen,
                },
            );
        }
        let emitter = self.emitter.clone();
        let conns = Arc::clone(&self.conns);
        let id = id.to_string();
        let url = url.to_string();
        let cookie = {
            let store = self.cookie_store.lock().expect("ws cookie store lock");
            ws_handshake_cookie(&store, &url, origin)
        };
        let handshake = zero_net::websocket::WebSocketHandshake {
            origin: origin.to_string(),
            cookie,
        };
        let _protocols = protocols.to_vec(); // FIXME(t8k): Sec-WebSocket-Protocol 协商头与 101 选定子协议未透出（zero_net::WebSocket 不暴露握手响应）——本切片先发空协议头，子协议协商面留待 net 层扩展
        std::thread::spawn(move || {
            // 阻塞 TCP/TLS/握手（OS 级 TCP 超时兜底；服务端 accept 后不回 101 的
            // 无限阻塞缺口见 defect-r1 N2，非本次返修范围）。
            let mut ws = zero_net::websocket::WebSocket::new(&url);
            if let Err(e) = ws.connect_with_handshake(&handshake) {
                // 终结权仲裁：槽位已消失（close-during-connect 的 close() 仲裁已发唯一
                // Close）、已 Closed、或已被跨代际重连覆盖（新代际持有终结权）——本线程
                // 一律静默，且只删除仍属本代际的槽位（t8k defect-r1 D1）。
                let mine = {
                    let mut conns = conns.lock().expect("ws conn table lock");
                    let mine = conns.get(&id).is_some_and(|s| s.nav_gen == nav_gen);
                    if mine {
                        conns.remove(&id);
                    }
                    mine
                };
                if !mine {
                    return;
                }
                // 恰好一条 Close 契约：connect 失败 → err + close(1006)。
                emitter.emit_gen(&id, WsEvent::Error(format!("connect failed: {e}")), nav_gen);
                emitter.emit_gen(
                    &id,
                    WsEvent::Close {
                        code: 1006,
                        reason: String::new(),
                        clean: false,
                    },
                    nav_gen,
                );
                return;
            }
            // 握手成功——close-during-connect 竞态：close() 已仲裁终结（发过 close 事件），
            // 或跨代际重连已覆盖槽位——本线程静默弃连，不动新代际槽位。
            {
                let mut conns = conns.lock().expect("ws conn table lock");
                match conns.get_mut(&id) {
                    Some(slot) if slot.nav_gen == nav_gen => {
                        if slot.state == ConnState::Closed {
                            conns.remove(&id);
                            return;
                        }
                        slot.state = ConnState::Open;
                    }
                    _ => return,
                }
            }
            // 读泵：阻塞 read() 超时返 WouldBlock/TimedOut → 轮询发送通道（tungstenite
            // read 自动回 Pong）。全部 emit 带 connect 时捕获的代际戳——跨文档 reset 后
            // 旧泵残余事件在 emit_gen 栅栏处丢弃。
            let _ = ws.set_read_timeout(Some(std::time::Duration::from_millis(WS_PUMP_POLL_MS)));
            emitter.emit_gen(
                &id,
                WsEvent::Open {
                    protocol: String::new(),
                },
                nav_gen,
            );
            loop {
                loop {
                    match cmd_rx.try_recv() {
                        Ok(HostCmd::Text(t)) => {
                            let n = t.len();
                            if ws.send(&t).is_ok() {
                                emitter.emit_gen(&id, WsEvent::Sent { bytes: n }, nav_gen);
                            }
                        }
                        Ok(HostCmd::Bytes(b)) => {
                            let n = b.len();
                            if ws.send_binary(&b).is_ok() {
                                emitter.emit_gen(&id, WsEvent::Sent { bytes: n }, nav_gen);
                            }
                        }
                        Ok(HostCmd::Close { code, reason }) => {
                            // code 0 = 页面 close() 无参——发不带状态码的 Close 帧（RFC 6455
                            // 禁止 wire 上出现 0）；close 事件代码按 1005（no status）报告。
                            if code == 0 {
                                let _ = ws.close();
                            } else {
                                let _ = ws.close_with(code, &reason);
                            }
                            emitter.emit_gen(
                                &id,
                                WsEvent::Close {
                                    code: if code == 0 { 1005 } else { code },
                                    reason,
                                    clean: true,
                                },
                                nav_gen,
                            );
                            remove_conn_if_current(&conns, &id, nav_gen);
                            return;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        // 通道断（表项被移）——不 emit（close() 路径已仲裁终结）。
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                    }
                }
                match ws.receive() {
                    Ok(Some(zero_net::websocket::WebSocketMessage::Text(t))) => {
                        emitter.emit_gen(&id, WsEvent::Text(t), nav_gen);
                    }
                    Ok(Some(zero_net::websocket::WebSocketMessage::Binary(b))) => {
                        emitter.emit_gen(&id, WsEvent::Binary(b), nav_gen);
                    }
                    Ok(Some(zero_net::websocket::WebSocketMessage::Close(code, reason))) => {
                        // net 层 receive 已对服务端 Close 回送 Close 帧（RFC 6455 §7.1.5）。
                        emitter.emit_gen(
                            &id,
                            WsEvent::Close {
                                code: code.unwrap_or(1005),
                                reason: reason.unwrap_or_default(),
                                clean: true,
                            },
                            nav_gen,
                        );
                        remove_conn_if_current(&conns, &id, nav_gen);
                        return;
                    }
                    // Ping/Pong：tungstenite read 期间自动回 Pong，页面不可见（spec：UA 处理）。
                    Ok(Some(zero_net::websocket::WebSocketMessage::Ping(_)))
                    | Ok(Some(zero_net::websocket::WebSocketMessage::Pong(_))) => {}
                    Ok(None) => {} // 读超时（WouldBlock/TimedOut）或连接已静默关闭——下轮重查
                    Err(e) => {
                        emitter.emit_gen(&id, WsEvent::Error(format!("receive: {e}")), nav_gen);
                        emitter.emit_gen(
                            &id,
                            WsEvent::Close {
                                code: 1006,
                                reason: String::new(),
                                clean: false,
                            },
                            nav_gen,
                        );
                        remove_conn_if_current(&conns, &id, nav_gen);
                        return;
                    }
                }
            }
        });
    }

    fn send(&self, id: &str, data: &WsData) {
        let cmd = match data {
            WsData::Text(t) => HostCmd::Text(t.clone()),
            WsData::Bytes(b) => HostCmd::Bytes(b.clone()),
        };
        if let Ok(conns) = self.conns.lock()
            && let Some(slot) = conns.get(id)
        {
            let _ = slot.cmd_tx.send(cmd);
        }
    }

    fn close(&self, id: &str, code: u16, reason: &str) {
        let arbitration = {
            let mut conns = self.conns.lock().expect("ws conn table lock");
            match conns.get_mut(id).map(|s| (s.state, s.cmd_tx.clone())) {
                Some((ConnState::Connecting, _)) => {
                    // 连接中关闭：本地仲裁终结（本函数发唯一 Close），connect 线程见 Closed 静默弃连。
                    if let Some(slot) = conns.get_mut(id) {
                        slot.state = ConnState::Closed;
                    }
                    conns.remove(id);
                    Some((false, 1006))
                }
                Some((ConnState::Open, cmd_tx)) => {
                    // open 连接：Close 帧交泵线程发送，泵发终态 Close 事件（clean=true）。
                    let _ = cmd_tx.send(HostCmd::Close {
                        code,
                        reason: reason.to_string(),
                    });
                    None
                }
                // 已关/从未存在：幂等 no-op（spec close 幂等语义）。
                _ => None,
            }
        };
        if let Some((_, code)) = arbitration {
            self.emitter.emit(
                id,
                WsEvent::Close {
                    code,
                    reason: String::new(),
                    clean: false,
                },
            );
        }
    }
}

/// WebSocket 宿主工厂——`SetWsHandler` 命令载荷 / webview config 字段类型。factory 形态
/// 解 host↔bridge emitter chicken-and-egg（js_worker spawn 时 emitter 未就绪；宿主又必须
/// 持 emitter 推事件）。
pub type WsHostFactory = Arc<dyn Fn(WsEmitter) -> Arc<dyn WsHost> + Send + Sync>;

/// 生产 WS 宿主工厂——`SetWsHandler` 命令 arm / webview config 调用（factory 形态解
/// host↔bridge emitter chicken-and-egg：host 需要 [`WsEmitter`]，而 emitter 归 bridge）。
pub fn default_net_ws_host(emitter: WsEmitter) -> Arc<dyn WsHost> {
    Arc::new(NetWsHost {
        emitter,
        conns: Arc::new(Mutex::new(HashMap::new())),
        cookie_store: zero_net::shared_cookie_store(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── wire 序列化 ──

    #[test]
    fn wire_open_and_close_sanitize_control_seps() {
        let w = serialize_event(&WsEvent::Open {
            protocol: "chat\x1fv2".to_string(),
        });
        assert_eq!(w, "open\x1fchatv2", "protocol 含 \\x1f 须剥除");
        let w = serialize_event(&WsEvent::Close {
            code: 1000,
            reason: "bye\x1fnow".to_string(),
            clean: true,
        });
        assert_eq!(w, "close\x1f1000\x1f1\x1fbyenow");
        let w = serialize_event(&WsEvent::Close {
            code: 1006,
            reason: String::new(),
            clean: false,
        });
        assert_eq!(w, "close\x1f1006\x1f0\x1f");
    }

    #[test]
    fn wire_text_is_last_field_and_preserves_field_sep() {
        let w = serialize_event(&WsEvent::Text("a\x1fb".to_string()));
        // 数据字段是末字段：首个 \x1f 后全部保留（与 fetch body 约定一致）。
        let idx = w.find(FIELD_SEP).expect("sep");
        assert_eq!(&w[..idx], "msg");
        assert_eq!(&w[idx + 1..], "a\x1fb");
    }

    #[test]
    fn wire_binary_csv_round_trip() {
        let bytes = vec![0x48u8, 0x00, 0xFF, b','];
        let w = serialize_event(&WsEvent::Binary(bytes.clone()));
        assert!(w.starts_with("bin\x1f"));
        let csv = &w["bin\x1f".len()..];
        assert_eq!(decode_bytes_csv(csv), Some(bytes));
        assert_eq!(decode_bytes_csv(""), Some(Vec::new()));
        assert_eq!(decode_bytes_csv("72,999"), None, "超 u8 → None");
    }

    #[test]
    fn send_data_decode_kinds() {
        assert_eq!(decode_send_data("t", "hello"), Some(WsData::Text("hello".to_string())));
        // 文本数据可含 \x1f（末字段语义）。
        assert_eq!(
            decode_send_data("t", "a\x1fb"),
            Some(WsData::Text("a\x1fb".to_string()))
        );
        assert_eq!(decode_send_data("b", "104,105"), Some(WsData::Bytes(vec![104, 105])));
        assert_eq!(decode_send_data("b", "nope"), None);
        assert_eq!(decode_send_data("x", "1"), None);
    }

    // ── 事件路由（emitter + pending）──

    /// 收集 resolver resolve 调用的槽。
    type ResolvedLog = Arc<Mutex<Vec<(String, String)>>>;

    fn logging_resolver() -> (AsyncResolver, ResolvedLog) {
        let log: ResolvedLog = Arc::new(Mutex::new(Vec::new()));
        let l = Arc::clone(&log);
        (
            AsyncResolver::new(move |id, result| l.lock().unwrap().push((id.to_string(), result.to_string()))),
            log,
        )
    }

    #[test]
    fn emit_before_next_queues_then_next_resolves_fifo() {
        let (resolver, log) = logging_resolver();
        let em = WsEmitter {
            core: Arc::new(Mutex::new(WsCore::default())),
            resolver,
            host: Arc::new(Mutex::new(None)),
        };
        // open 先于 next 到达（握手快于 JS 泵注册）→ 入队；next 到达 → 立即 FIFO resolve。
        em.emit(
            "c1",
            WsEvent::Open {
                protocol: String::new(),
            },
        );
        em.emit("c1", WsEvent::Text("hello".to_string()));
        em.next("c1", "p1");
        em.next("c1", "p2");
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[0], ("p1".to_string(), "open\x1f".to_string()));
        assert_eq!(l[1], ("p2".to_string(), "msg\x1fhello".to_string()));
    }

    #[test]
    fn next_before_emit_registers_pending() {
        let (resolver, log) = logging_resolver();
        let em = WsEmitter {
            core: Arc::new(Mutex::new(WsCore::default())),
            resolver,
            host: Arc::new(Mutex::new(None)),
        };
        em.next("c1", "p1"); // JS 泵先注册
        em.emit(
            "c1",
            WsEvent::Open {
                protocol: "json".to_string(),
            },
        ); // 事件后到
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], ("p1".to_string(), "open\x1fjson".to_string()));
    }

    #[test]
    fn emit_from_other_thread_reaches_pending() {
        let (resolver, log) = logging_resolver();
        let em = WsEmitter {
            core: Arc::new(Mutex::new(WsCore::default())),
            resolver,
            host: Arc::new(Mutex::new(None)),
        };
        em.next("c1", "p1");
        let em2 = em.clone();
        std::thread::spawn(move || em2.emit("c1", WsEvent::Error("boom".to_string())))
            .join()
            .unwrap();
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], ("p1".to_string(), "err\x1fboom".to_string()));
    }

    #[test]
    fn queue_overflow_synthesizes_close() {
        let (resolver, log) = logging_resolver();
        let em = WsEmitter {
            core: Arc::new(Mutex::new(WsCore::default())),
            resolver,
            host: Arc::new(Mutex::new(None)),
        };
        for i in 0..(EVENT_QUEUE_CAP + 5) {
            em.emit("c1", WsEvent::Text(format!("m{i}")));
        }
        em.next("c1", "p_only");
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 1, "溢出后 next 只收合成 close");
        assert_eq!(l[0].0, "p_only");
        assert!(
            l[0].1.starts_with("close\x1f1008\x1f0\x1fevent-queue-overflow"),
            "overflow → 合成 close 1008：{}",
            l[0].1
        );
    }

    // ── register：回调捕获 + gate ──

    /// 捕获注册回调的假 sandbox（fetch_bridge 测试同型）。
    struct CaptureCb {
        cbs: Mutex<HashMap<String, Box<dyn Fn(&[String]) -> String + Send + Sync>>>,
    }
    impl CaptureCb {
        fn new() -> Self {
            Self {
                cbs: Mutex::new(HashMap::new()),
            }
        }
        fn take(&self, name: &str) -> Box<dyn Fn(&[String]) -> String + Send + Sync> {
            self.cbs.lock().unwrap().remove(name).expect(name)
        }
    }
    impl zero_script_sandbox::Sandbox for CaptureCb {
        fn execute(&mut self, _: &str) -> Result<zero_script_sandbox::ScriptResult, zero_script_sandbox::ScriptError> {
            unreachable!()
        }
        fn execute_json(
            &mut self,
            _: &str,
        ) -> Result<zero_script_sandbox::ScriptResult, zero_script_sandbox::ScriptError> {
            unreachable!()
        }
        fn register_callback(&mut self, name: &str, callback: Box<dyn Fn(&[String]) -> String + Send + Sync>) {
            self.cbs.lock().unwrap().insert(name.to_string(), callback);
        }
        fn set_timeout_ms(&mut self, _: u64) {}
        fn reset_context(&mut self) {}
        fn config(&self) -> &zero_script_sandbox::SandboxConfig {
            unreachable!()
        }
    }

    /// 记录 WsHost 命令的假宿主。
    #[derive(Default)]
    struct LogHost {
        cmds: Mutex<Vec<String>>,
    }
    impl WsHost for LogHost {
        fn connect(&self, id: &str, url: &str, protocols: &[String], origin: &str, cookie: &str) {
            self.cmds
                .lock()
                .unwrap()
                .push(format!("connect {id} {url} {protocols:?} origin={origin} cookie={cookie}"));
        }
        fn send(&self, id: &str, data: &WsData) {
            self.cmds.lock().unwrap().push(format!("send {id} {data:?}"));
        }
        fn close(&self, id: &str, code: u16, reason: &str) {
            self.cmds.lock().unwrap().push(format!("close {id} {code} {reason}"));
        }
    }

    #[test]
    fn register_routes_all_four_callbacks() {
        let (resolver, _log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        let host = Arc::new(LogHost::default());
        bridge.set_host(Arc::clone(&host) as Arc<dyn WsHost>);
        let sandbox = {
            let mut sb = CaptureCb::new();
            bridge.register(&mut sb);
            sb
        };
        let connect = sandbox.take("__zw_ws_connect");
        connect(&["w1".into(), "ws://127.0.0.1:9/echo".into(), "chat,v2".into()]);
        let send = sandbox.take("__zw_ws_send");
        send(&["w1".into(), "t".into(), "hi there".into()]);
        send(&["w1".into(), "b".into(), "1,2,3".into()]);
        let close = sandbox.take("__zw_ws_close");
        close(&["w1".into(), "1000".into(), "done".into()]);
        // connect 同步转发（非阻塞契约）——命令立即落齐；循环兜底保留。
        for _ in 0..100 {
            let n = host.cmds.lock().unwrap().len();
            if n >= 4 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let cmds = host.cmds.lock().unwrap().clone();
        assert!(
            cmds.iter()
                .any(|c| c.contains("connect w1 ws://127.0.0.1:9/echo") && c.contains("\"chat\", \"v2\"")),
            "{cmds:?}"
        );
        assert!(cmds.iter().any(|c| c == "send w1 Text(\"hi there\")"), "{cmds:?}");
        assert!(cmds.iter().any(|c| c == "send w1 Bytes([1, 2, 3])"), "{cmds:?}");
        assert!(cmds.iter().any(|c| c == "close w1 1000 done"), "{cmds:?}");
    }

    #[test]
    fn register_no_host_fails_fast_with_error_and_close() {
        let (resolver, log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        let mut sb = CaptureCb::new();
        bridge.register(&mut sb);
        let connect = sb.take("__zw_ws_connect");
        let next = sb.take("__zw_ws_next");
        connect(&["w1".into(), "ws://127.0.0.1:9/x".into(), "".into()]);
        // 事件先入队列（无 pending）；走真实泵路径：next 登记 → resolve。
        next(&["w1".into(), "p1".into()]);
        next(&["w1".into(), "p2".into()]);
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 2, "no-handler → err + close 快速失败：{l:?}");
        assert_eq!(l[0], ("p1".to_string(), "err\x1fno-handler".to_string()));
        assert_eq!(l[1], ("p2".to_string(), "close\x1f1006\x1f0\x1f".to_string()));
    }

    // ── NetWsHost 端到端（真实 TCP RFC6455 echo server → default_net_ws_host 全路径）──

    /// 极简 base64（RFC 4648 标准 alphabet）——Sec-WebSocket-Accept 编码用（测试 fixture，
    /// 不引新依赖）。
    fn b64(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            out.push(T[(n >> 18) as usize & 63] as char);
            out.push(T[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                T[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }

    /// RFC6455 最小 echo server：握手（Sec-WebSocket-Accept 校验由 tungstenite 客户端承担）
    /// → 帧 echo（text/binary 原样回、close 回空 close、ping 回 pong）。多连接（每连接
    /// 一线程）——代际钉测需同 id 跨代重连到同一 server。
    fn spawn_echo_server() -> std::net::SocketAddr {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind fixture");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            for sock in listener.incoming() {
                let Ok(sock) = sock else { return };
                std::thread::spawn(move || handle_echo_conn(sock));
            }
        });
        addr
    }

    /// 单连接 echo 处理（[`spawn_echo_server`] 每连接一线程）。
    fn handle_echo_conn(mut sock: std::net::TcpStream) {
        {
            use std::io::{BufRead, BufReader, Read, Write};
            let mut reader = BufReader::new(sock.try_clone().expect("clone"));
            let mut key = String::new();
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                if line == "\r\n" {
                    break;
                }
                if let Some(v) = line.strip_prefix("Sec-WebSocket-Key:") {
                    key = v.trim().to_string();
                }
            }
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            hasher.update(format!("{key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11"));
            let accept = b64(&hasher.finalize());
            let _ = sock.write_all(
                format!(
                    "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
                )
                .as_bytes(),
            );
            loop {
                let mut hdr = [0u8; 2];
                if reader.read_exact(&mut hdr).is_err() {
                    return;
                }
                let opcode = hdr[0] & 0x0f;
                let masked = hdr[1] & 0x80 != 0;
                let mut len = usize::from(hdr[1] & 0x7f);
                if len == 126 {
                    let mut ext = [0u8; 2];
                    reader.read_exact(&mut ext).unwrap();
                    len = usize::from(u16::from_be_bytes(ext));
                } else if len == 127 {
                    let mut ext = [0u8; 8];
                    reader.read_exact(&mut ext).unwrap();
                    len = u64::from_be_bytes(ext) as usize;
                }
                let mut mask = [0u8; 4];
                if masked {
                    reader.read_exact(&mut mask).unwrap();
                }
                let mut payload = vec![0u8; len];
                if len > 0 {
                    reader.read_exact(&mut payload).unwrap();
                }
                if masked {
                    for (i, b) in payload.iter_mut().enumerate() {
                        *b ^= mask[i % 4];
                    }
                }
                match opcode {
                    0x8 => {
                        let _ = sock.write_all(&[0x88, 0x00]); // close → 空 close 帧
                        return;
                    }
                    0x9 => {
                        let _ = sock.write_all(&[0x8A, 0x00]); // ping → pong
                    }
                    0x1 | 0x2 => {
                        let mut frame = vec![0x80 | opcode];
                        if payload.len() < 126 {
                            frame.push(payload.len() as u8);
                        } else {
                            frame.push(126);
                            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
                        }
                        frame.extend_from_slice(&payload);
                        let _ = sock.write_all(&frame);
                    }
                    _ => {}
                }
            }
        }
    }

    /// 轮询等 resolver log 落齐 n 条（泵线程异步 emit）。
    fn wait_log(log: &ResolvedLog, n: usize, timeout: std::time::Duration) -> Vec<(String, String)> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let snapshot = log.lock().unwrap().clone();
            if snapshot.len() >= n || std::time::Instant::now() > deadline {
                return snapshot;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn net_ws_host_echo_roundtrip_end_to_end() {
        let addr = spawn_echo_server();
        let (resolver, log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        bridge.set_host(default_net_ws_host(bridge.emitter()));
        let mut sb = CaptureCb::new();
        bridge.register(&mut sb);
        let connect = sb.take("__zw_ws_connect");
        let next = sb.take("__zw_ws_next");
        let send = sb.take("__zw_ws_send");
        let close = sb.take("__zw_ws_close");
        connect(&["e1".into(), format!("ws://{addr}/echo"), "chat".into()]);
        // 串行泵：逐条取事件（bridge 队列兜底先到事件）。send 成功后宿主回投
        // `sent{sep}{bytes}`（bufferedAmount 递减依据）——事件序 open → sent → echo msg。
        next(&["e1".into(), "q1".into()]);
        let l = wait_log(&log, 1, std::time::Duration::from_secs(5));
        assert_eq!(l.len(), 1, "open 事件超时：{l:?}");
        assert_eq!(l[0].1, "open\x1f", "open wire（子协议面未协商——FIXME 记录）");
        // 文本 echo：sent:9（hello-t8k UTF-8 字节数）→ msg 回显。
        send(&["e1".into(), "t".into(), "hello-t8k".into()]);
        next(&["e1".into(), "q2".into()]);
        let l = wait_log(&log, 2, std::time::Duration::from_secs(5));
        assert_eq!(l[1].1, "sent\x1f9");
        next(&["e1".into(), "q2b".into()]);
        let l = wait_log(&log, 3, std::time::Duration::from_secs(5));
        assert_eq!(l[2].1, "msg\x1fhello-t8k");
        // 二进制 echo（csv 往返）：sent:3（3 字节）→ bin 回显。
        send(&["e1".into(), "b".into(), "104,105,255".into()]);
        next(&["e1".into(), "q3".into()]);
        let l = wait_log(&log, 4, std::time::Duration::from_secs(5));
        assert_eq!(l[3].1, "sent\x1f3");
        next(&["e1".into(), "q3b".into()]);
        let l = wait_log(&log, 5, std::time::Duration::from_secs(5));
        assert_eq!(l[4].1, "bin\x1f104,105,255");
        // 带码 close：宿主发 Close 帧（服务端回空 close）→ 终态 close wire clean。
        close(&["e1".into(), "1000".into(), "done".into()]);
        next(&["e1".into(), "q4".into()]);
        let l = wait_log(&log, 6, std::time::Duration::from_secs(5));
        assert_eq!(l[5].1, "close\x1f1000\x1f1\x1fdone", "本地 close 帧码/原因回投：{l:?}");
    }

    // ── 跨文档代际（t8k defect-r1 D1 返修钉）──

    #[test]
    fn reset_generation_fences_stale_generation_events() {
        let (resolver, log) = logging_resolver();
        let em = WsEmitter {
            core: Arc::new(Mutex::new(WsCore::default())),
            resolver,
            host: Arc::new(Mutex::new(None)),
        };
        // 旧代残余事件（reset 前捕获代际的泵线程）——reset 后到达须被栅栏丢弃。
        let stale_gen = em.current_generation();
        em.reset_generation();
        em.next("ws1", "p_new"); // 新文档泵 pending 登记（reset 后的新上下文）
        let em2 = em.clone();
        std::thread::spawn(move || {
            em2.emit_gen(
                "ws1",
                WsEvent::Open {
                    protocol: String::new(),
                },
                stale_gen,
            );
            em2.emit_gen("ws1", WsEvent::Text("stale".to_string()), stale_gen);
        })
        .join()
        .unwrap();
        // 新代事件正常投递。
        em.emit("ws1", WsEvent::Text("fresh".to_string()));
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 1, "旧代残余事件须被栅栏丢弃：{l:?}");
        assert_eq!(l[0], ("p_new".to_string(), "msg\x1ffresh".to_string()));
    }

    #[test]
    fn net_ws_host_reset_generation_purges_and_reconnects_same_id() {
        let addr = spawn_echo_server();
        let (resolver, log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        bridge.set_host(default_net_ws_host(bridge.emitter()));
        let mut sb = CaptureCb::new();
        bridge.register(&mut sb);
        let connect = sb.take("__zw_ws_connect");
        let next = sb.take("__zw_ws_next");
        // 旧代文档：connect 即导航（open 未消费、泵 pending 随 reset 作废）。
        connect(&["ws1".into(), format!("ws://{addr}/echo"), "".into()]);
        next(&["ws1".into(), "p_old".into()]);
        bridge.reset_generation();
        // 新文档同 id 重连（shim 计数器归零 → 同名 ws1——D1 触发条件）。
        connect(&["ws1".into(), format!("ws://{addr}/echo"), "".into()]);
        next(&["ws1".into(), "p_new".into()]);
        // 等待窗口覆盖两个连接的握手完成：旧代 open（gen0 戳）若漏栅栏会在此到达。
        let l = wait_log(&log, 2, std::time::Duration::from_secs(2));
        assert_eq!(l.len(), 1, "旧代残余 open 须被代际栅栏丢弃、新代 open 须到达：{l:?}");
        assert_eq!(l[0].0, "p_new", "新代 open resolve 新文档泵");
        assert_eq!(l[0].1, "open\x1f");
    }

    #[test]
    fn net_ws_host_stale_connect_failure_does_not_orphan_new_generation() {
        // gen1 握手挂起服务端（accept 后不回 101）——泵阻塞在握手中。
        let stall = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stall fixture");
        let stall_addr = stall.local_addr().expect("stall addr");
        let _stall_thread = std::thread::spawn(move || {
            let Ok((s, _)) = stall.accept() else { return };
            std::thread::sleep(std::time::Duration::from_millis(500));
            drop(s); // EOF → gen1 泵迟到失败
        });
        let addr = spawn_echo_server();
        let (resolver, log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        bridge.set_host(default_net_ws_host(bridge.emitter()));
        let mut sb = CaptureCb::new();
        bridge.register(&mut sb);
        let connect = sb.take("__zw_ws_connect");
        let next = sb.take("__zw_ws_next");
        let send = sb.take("__zw_ws_send");
        // gen1：连挂起服务端（同 id ws1 首连）。
        connect(&["ws1".into(), format!("ws://{stall_addr}/x"), "".into()]);
        // 跨文档 reset + gen2 同 id 重连（D1：新文档首连同名 ws1）。
        bridge.reset_generation();
        connect(&["ws1".into(), format!("ws://{addr}/echo"), "".into()]);
        next(&["ws1".into(), "p1".into()]);
        let l = wait_log(&log, 1, std::time::Duration::from_secs(5));
        assert_eq!(l[0].1, "open\x1f", "gen2 open 到达：{l:?}");
        // gen1 泵迟到失败（500ms EOF）：不得删 gen2 槽位（孤儿化臂）、失败事件被栅栏丢弃。
        std::thread::sleep(std::time::Duration::from_millis(1200));
        send(&["ws1".into(), "t".into(), "alive".into()]);
        next(&["ws1".into(), "p2".into()]); // sent:5（send 成功回投）
        next(&["ws1".into(), "p3".into()]); // msg 回显
        let l = wait_log(&log, 3, std::time::Duration::from_secs(5));
        assert_eq!(l[2].1, "msg\x1falive", "gen1 迟到失败不得孤儿化 gen2 连接：{l:?}");
        assert_eq!(l.len(), 3, "gen1 失败事件（err/close）须被栅栏丢弃：{l:?}");
    }

    #[test]
    fn net_ws_host_close_during_connect_emits_exactly_one_close() {
        // 握手挂起服务端——slot 停留 Connecting（close-during-connect 窗口）。
        let stall = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stall fixture");
        let stall_addr = stall.local_addr().expect("stall addr");
        let _stall_thread = std::thread::spawn(move || {
            let Ok((s, _)) = stall.accept() else { return };
            std::thread::sleep(std::time::Duration::from_millis(800));
            drop(s); // EOF → 泵迟到失败分支
        });
        let (resolver, log) = logging_resolver();
        let bridge = WsBridge::new(resolver);
        bridge.set_host(default_net_ws_host(bridge.emitter()));
        let mut sb = CaptureCb::new();
        bridge.register(&mut sb);
        let connect = sb.take("__zw_ws_connect");
        let next = sb.take("__zw_ws_next");
        let close = sb.take("__zw_ws_close");
        connect(&["ws1".into(), format!("ws://{stall_addr}/x"), "".into()]);
        // 连接中关闭：本地仲裁——恰好一条 close(1006, clean=false)。
        close(&["ws1".into(), "1000".into(), "bye".into()]);
        next(&["ws1".into(), "p1".into()]);
        let l = wait_log(&log, 1, std::time::Duration::from_secs(5));
        assert_eq!(
            l[0].1, "close\x1f1006\x1f0\x1f",
            "Connecting 期 close → 仲裁唯一 close：{l:?}"
        );
        // 泵迟到失败（800ms EOF）不得再发第二条 close（恰好一条 Close 契约）。
        std::thread::sleep(std::time::Duration::from_millis(1200));
        let l = log.lock().unwrap();
        assert_eq!(l.len(), 1, "close-during-connect 后泵须静默：{l:?}");
    }
}
