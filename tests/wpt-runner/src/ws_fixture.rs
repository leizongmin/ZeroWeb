//! runner 内嵌 WebSocket fixture 端点 + 宿主改写适配器（net-api-compat M5-S1，P6 二期切片）。
//!
//! 语义基准 = 上游 `websockets/handlers/*.py`（WPT_REV `315976933870b34d6ea30e3f6643403edae678ba`
//! 逐文件核读；wptserve/pywebsocket handler 的 Rust 等价实现）。runner 无 wptserve
//! websocket handler 通道，websockets/ corpus 的连接面由本端点承接：
//!
//! - **端点行为**（按上游 handler 一一对应）：
//!   - `/echo`（及未知路径兜底）：text/binary 逐条回显；text `Goodbye` → 服务端发空
//!     close 帧后等客户端回关（`echo_wsh` passive close）；`/echo_exit` 在 `Goodbye` 后
//!     **不发 close 帧直接断 TCP**（handler 退出语义 → 客户端 1006）
//!   - `/empty-message`：收一条消息，空串回 `pass` 否则 `fail`
//!   - `/echo-cookie`：回握手 `Cookie` 头原值，无则 `(none)`
//!   - `/echo-query`：回 URI query 段（`?` 后原样）
//!   - `/origin`：回握手 `Origin` 头原值
//!   - `/referrer`：回 `Referer` 头原值，无则 `MISSING AS PER FETCH`（FETCH 语义不发送）
//!   - `/receive-many-with-backpressure`：每条消息延迟 100ms 后回其**字节长度**文本
//!   - `/protocol_array`：Sec-WebSocket-Protocol 取**逗号分割第一项**协商并回发
//!   - `/protocol`：协议头**原串**协商并回发（多协议不分割——上游 quirk 即被测面）
//!   - `/handshake_protocol`：恒协商 `foobar`；`/handshake_no_protocol`：请求了也不协商
//!   - `/handshake_no_extensions`：101 不带 `Sec-WebSocket-Extensions`（本端点恒不发，
//!     对 `/echo` 亦然——`Create-extensions-empty` 断言依赖）
//!   - `/wrong_accept_key`：`Sec-WebSocket-Accept` 用错误值（客户端须拒连不派发 open）
//!   - `/invalid`：回非 101 的裸 `FOO BAR BAZ\r\n\r\n`（客户端须按握手失败处理）
//!   - `/handshake_sleep_2`：握手前 sleep 2s（CONNECTING readyState 时序面）
//!   - `/set-cookie*`：101 带 `Set-Cookie: ws_test_<query>=test; Path=/`（`_http` 变体
//!     追加 `; HttpOnly` + `Sec-WebSocket-Origin` 回显）
//!   - `/simple_handshake`：手工响应头（`101 Switching Protocols:` 带冒号 + Set-Cookie）
//!     与 **1001 `PASS` close 帧同包**发送（上游同包语义）
//!   - `/delayed-passive-close`：等客户端 close 帧 → sleep 1s → 回显码/原因（无码回空）
//!     ——`Close-delayed` 的关闭握手耗时/wasClean 断言面
//! - **宿主改写**（[`FixtureWsHost`]）：corpus 以 `ws://wpt.test:<port>/...`（含
//!   `www1.`/`alt.` 前缀变体）连接——host 层把 `*.wpt.test` 权威段改写为
//!   `127.0.0.1:<同端口>` 后委派 [`default_net_ws_host`] 走 zero-net 真 TCP/tungstenite
//!   全栈；页面 Origin/Cookie 语义不经改写（`/origin`、`/echo-cookie` 端点即被测面）。
//!
//! 启用面：仅 `case_name` 以 `websockets/` 开头的用例在 WebView 装配处注入 factory
//! （`run_testharness_html_inner`）——CSP/cookies 通道的 WebSocket 用例维持无宿主
//! 降级路径，跨通道零耦合（run-rules §10：各流只写自己的 goal 控制面）。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use zero_engine::ws_bridge::{self, WsData, WsHost};

/// 握手响应静默读上限（防残连接线程积压；`/handshake_sleep_2` 的 2s 与
/// `/receive-many-with-backpressure` 的每条 100ms 间隔均在此余量内）。
const CONN_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// 全局 fixture 端口（进程级单 server；`apply_wpt_substitutions` 替换
/// `{{ports[ws][0]}}` 族 token 时惰性拉起，供整个 runner 进程复用）。
pub fn fixture_ws_port() -> u16 {
    static PORT: OnceLock<u16> = OnceLock::new();
    *PORT.get_or_init(|| {
        let listener = TcpListener::bind("127.0.0.1:0").expect("ws fixture bind");
        let port = listener.local_addr().expect("ws fixture addr").port();
        std::thread::Builder::new()
            .name("ws-fixture".into())
            .spawn(move || {
                for sock in listener.incoming() {
                    let Ok(sock) = sock else { break };
                    let _ = sock.set_read_timeout(Some(CONN_READ_TIMEOUT));
                    std::thread::Builder::new()
                        .name("ws-fixture-conn".into())
                        .spawn(move || handle_conn(sock))
                        .ok();
                }
            })
            .expect("ws fixture thread");
        port
    })
}

/// net-api 通道 WebView 装配用 factory：改写宿主 + 委派 `default_net_ws_host` 真栈。
pub fn fixture_ws_host_factory() -> ws_bridge::WsHostFactory {
    Arc::new(|emitter| -> Arc<dyn WsHost> {
        Arc::new(FixtureWsHost {
            inner: ws_bridge::default_net_ws_host(emitter),
        })
    })
}

/// 宿主改写适配器：`ws(s)://<*.wpt.test|wpt.test>[:port]/...` 权威段 →
/// `ws(s)://127.0.0.1:<同端口>/...`，路径/查询/协议参数原样透传。
pub struct FixtureWsHost {
    inner: Arc<dyn WsHost>,
}

impl WsHost for FixtureWsHost {
    fn connect(&self, id: &str, url: &str, protocols: &[String], origin: &str, cookie: &str) {
        self.inner
            .connect(id, &rewrite_wpt_authority(url), protocols, origin, cookie);
    }
    fn send(&self, id: &str, data: &WsData) {
        self.inner.send(id, data);
    }
    fn close(&self, id: &str, code: u16, reason: &str) {
        self.inner.close(id, code, reason);
    }
}

fn rewrite_wpt_authority(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let (authority, tail) = match rest.split_once('/') {
        Some((a, t)) => (a, Some(t)),
        None => (rest, None),
    };
    // net-api M5-S4：userinfo 感知（`ws://foo:bar@wpt.test:port/…`——basic-auth 面）。
    // 权威段按**最后一个** `@` 切 userinfo 与 host:port（userinfo 本身可含 `@` 的
    // 百分号编码形态不计——裸 `@` 分隔即权威段终结符）。
    let (userinfo, host_part) = match authority.rsplit_once('@') {
        Some((ui, hp)) => (Some(ui), hp),
        None => (None, authority),
    };
    let host = host_part.rsplit_once(':').map_or(host_part, |(h, _)| h);
    let is_wpt = host == "wpt.test" || host.ends_with(".wpt.test");
    if !is_wpt {
        return url.to_string();
    }
    let rewritten_hostport = match host_part.rsplit_once(':') {
        Some((_, port)) => format!("127.0.0.1:{port}"),
        None => "127.0.0.1".to_string(),
    };
    let rewritten_authority = match userinfo {
        Some(ui) => format!("{ui}@{rewritten_hostport}"),
        None => rewritten_hostport,
    };
    match tail {
        Some(t) => format!("{scheme}://{rewritten_authority}/{t}"),
        None => format!("{scheme}://{rewritten_authority}"),
    }
}

// ── 握手 ────────────────────────────────────────────────────────────────────

struct Handshake {
    path: String,
    query: String,
    headers: HashMap<String, String>,
}

fn read_handshake(reader: &mut BufReader<TcpStream>) -> Option<Handshake> {
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    // "GET /echo HTTP/1.1"
    let target = request_line.split_whitespace().nth(1)?.to_string();
    let (raw_path, raw_query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Some(Handshake {
        path: raw_path,
        query: raw_query,
        headers,
    })
}

/// RFC 6455 §4.2.2 accept 推导（`<key>258EAFA5-E914-47DA-95CA-C5AB0DC85B11` 的
/// SHA-1 → base64）。
fn compute_accept(key: &str) -> String {
    use base64::Engine as _;
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

fn handshake_response(path: &str, query: &str, hs: &Handshake) -> Vec<u8> {
    // `/invalid`：非 101 裸响应（`invalid_wsh` 客户端须按握手失败处理）。
    if path == "/invalid" {
        return b"FOO BAR BAZ\r\n\r\n".to_vec();
    }
    if path == "/handshake_sleep_2" {
        std::thread::sleep(Duration::from_secs(2));
    }
    // `/basic_auth`（net-api M5-S4）：401 挑战流——`Authorization: Basic base64(foo:bar)`
    // （上游 basic-auth.any.js 的 URL userinfo 凭据）正确 → 101 走常规握手；缺失/错误
    // → 401 + `WWW-Authenticate: Basic`（客户端按握手失败收场——zero-net 预置凭据
    // 一步直发，无挑战重试语义）。
    if path == "/basic_auth" {
        use base64::Engine as _;
        let expected = format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(b"foo:bar"));
        let got = hs.headers.get("authorization").cloned().unwrap_or_default();
        if got != expected {
            return b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"wpt\"\r\nContent-Length: 0\r\n\r\n"
                .to_vec();
        }
    }
    let key = hs.headers.get("sec-websocket-key").cloned().unwrap_or_default();
    // `/wrong_accept_key`：accept 用错误值（`wrong_accept_key` 端点语义——客户端
    // 须校验失败拒连，绝不派发 open）。
    let accept = if path == "/wrong_accept_key" {
        "INVALIDACCEPTKEYINVALIDACCEPTKEY===".to_string()
    } else {
        compute_accept(&key)
    };

    // 协议协商（各 path 规则见模块注释；无规则 path 不协商）。
    let requested = hs.headers.get("sec-websocket-protocol").cloned();
    let selected: Option<String> = match path {
        "/echo" => requested
            .filter(|r| r.split(',').any(|p| p.trim() == "echo"))
            .map(|_| "echo".to_string()),
        "/protocol_array" => requested
            .map(|r| r.split(',').next().unwrap_or("").trim().to_string())
            .filter(|p| !p.is_empty()),
        // `/protocol`：原串协商（多协议不分割——上游 quirk 被测面）。
        "/protocol" => requested.filter(|p| !p.is_empty()),
        "/handshake_protocol" => Some("foobar".to_string()),
        // `/handshake_no_protocol`：请求了也不协商；其余无规则 path 不协商。
        _ => None,
    };

    let mut head = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n"
    );
    if let Some(protocol) = &selected {
        head.push_str(&format!("Sec-WebSocket-Protocol: {protocol}\r\n"));
    }
    // `set-cookie_wsh` / `set-cookie_http_wsh`：cookie 名带 query（空则尾下划线）。
    if path == "/set-cookie" || path == "/set-cookie_http" {
        let name = if query.is_empty() {
            "ws_test_".to_string()
        } else {
            format!("ws_test_{query}")
        };
        if path == "/set-cookie_http" {
            let origin = hs.headers.get("origin").cloned().unwrap_or_default();
            head.push_str(&format!(
                "Set-Cookie: {name}=test; Path=/; HttpOnly\r\nSec-WebSocket-Origin: {origin}\r\n"
            ));
        } else {
            head.push_str(&format!("Set-Cookie: {name}=test; Path=/\r\n"));
        }
    }
    head.push_str("\r\n");
    head.into_bytes()
}

// ── 帧编解码（RFC 6455 §5；服务端→客户端不掩码） ─────────────────────────────

fn write_frame(sock: &mut TcpStream, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(0x80 | opcode);
    let len = payload.len();
    if len < 126 {
        frame.push(len as u8);
    } else if len <= u16::MAX as usize {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    sock.write_all(&frame)
}

enum WireFrame {
    /// 数据帧分片（text/binary/continuation，fin 终结当前消息）。
    Data {
        opcode: u8,
        fin: bool,
        payload: Vec<u8>,
    },
    Ping(Vec<u8>),
    /// 客户端 pong（载荷无需处理——服务端不发 ping，仅忽略）。
    Pong,
    Close(Option<(u16, String)>),
}

fn read_frame(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<WireFrame>> {
    let mut hdr = [0u8; 2];
    match reader.read_exact(&mut hdr) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let fin = hdr[0] & 0x80 != 0;
    let opcode = hdr[0] & 0x0F;
    let masked = hdr[1] & 0x80 != 0;
    let len7 = hdr[1] & 0x7F;
    let len = match len7 {
        126 => {
            let mut ext = [0u8; 2];
            reader.read_exact(&mut ext)?;
            u16::from_be_bytes(ext) as usize
        }
        127 => {
            let mut ext = [0u8; 8];
            reader.read_exact(&mut ext)?;
            u64::from_be_bytes(ext) as usize
        }
        n => n as usize,
    };
    let mask = if masked {
        let mut m = [0u8; 4];
        reader.read_exact(&mut m)?;
        Some(m)
    } else {
        None
    };
    let mut payload = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut payload)?;
    }
    if let Some(m) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= m[i % 4];
        }
    }
    Ok(Some(match opcode {
        0x8 => {
            if payload.len() >= 2 {
                let code = u16::from_be_bytes([payload[0], payload[1]]);
                let reason = String::from_utf8_lossy(&payload[2..]).into_owned();
                WireFrame::Close(Some((code, reason)))
            } else {
                WireFrame::Close(None)
            }
        }
        0x9 => WireFrame::Ping(payload),
        0xA => WireFrame::Pong,
        _ => WireFrame::Data { opcode, fin, payload },
    }))
}

/// 客户端来向消息（聚合 continuation 分片后）。
enum Incoming {
    Text(String),
    Binary(Vec<u8>),
    /// text 帧载荷非 UTF-8（RFC：须 close 1007）。
    InvalidUtf8,
}

/// 消息聚合读：跨 continuation 分片累积；控制帧即时旁路处理（ping→pong 自动回，
/// 与 pywebsocket 自动响应语义对齐）。返回 `None` = 对端 EOF/读错/收到 close。
fn read_message(sock: &mut TcpStream, reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<Incoming>> {
    let mut fragments: Vec<u8> = Vec::new();
    let mut text_msg = false;
    loop {
        let Some(frame) = read_frame(reader)? else {
            return Ok(None);
        };
        match frame {
            WireFrame::Ping(payload) => write_frame(sock, 0xA, &payload)?,
            WireFrame::Pong => {}
            WireFrame::Close(code_reason) => {
                // RFC：收到 close 即回显 close（码/原因回带）后断开。
                let body = match code_reason {
                    Some((code, reason)) => {
                        let mut b = code.to_be_bytes().to_vec();
                        b.extend_from_slice(reason.as_bytes());
                        b
                    }
                    None => Vec::new(),
                };
                let _ = write_frame(sock, 0x8, &body);
                return Ok(None);
            }
            WireFrame::Data { opcode, fin, payload } => {
                if opcode == 0x1 {
                    text_msg = true;
                }
                if opcode == 0x2 {
                    text_msg = false;
                }
                fragments.extend_from_slice(&payload);
                if fin {
                    let payload = std::mem::take(&mut fragments);
                    return Ok(Some(if text_msg {
                        match String::from_utf8(payload) {
                            Ok(text) => Incoming::Text(text),
                            Err(_) => Incoming::InvalidUtf8,
                        }
                    } else {
                        Incoming::Binary(payload)
                    }));
                }
            }
        }
    }
}

// ── 数据相（各 path 行为见模块注释） ────────────────────────────────────────

fn serve_data(sock: &mut TcpStream, reader: &mut BufReader<TcpStream>, hs: &Handshake) {
    let result: std::io::Result<()> = (|| {
        match hs.path.as_str() {
            // 单发型 handler（上游 transfer_data 单发即 return——pywebsocket handler
            // 返回后走服务端关闭握手）：发完发空 close 帧再等客户端回关。
            "/empty-message" => {
                // `empty-message_wsh`：收一条，空串回 pass 否则 fail。
                if let Some(Incoming::Text(text)) = read_message(sock, reader)? {
                    let reply = if text.is_empty() { "pass" } else { "fail" };
                    write_frame(sock, 0x1, reply.as_bytes())?;
                }
                server_close(sock, reader)
            }
            "/echo-cookie" => {
                let cookie = hs.headers.get("cookie").cloned();
                let reply = cookie.unwrap_or_else(|| "(none)".to_string());
                write_frame(sock, 0x1, reply.as_bytes())?;
                server_close(sock, reader)
            }
            "/echo-query" => {
                write_frame(sock, 0x1, hs.query.as_bytes())?;
                server_close(sock, reader)
            }
            "/origin" => {
                let origin = hs.headers.get("origin").cloned().unwrap_or_default();
                write_frame(sock, 0x1, origin.as_bytes())?;
                server_close(sock, reader)
            }
            "/referrer" => {
                // FETCH 语义 WS 握手不携带 Referer——缺失即 `MISSING AS PER FETCH`。
                let referer = hs
                    .headers
                    .get("referer")
                    .cloned()
                    .unwrap_or_else(|| "MISSING AS PER FETCH".to_string());
                write_frame(sock, 0x1, referer.as_bytes())?;
                server_close(sock, reader)
            }
            "/protocol_array" | "/protocol" => {
                // 协商值已在握手头选定——回发之（handler 单发即返回）。
                let selected = hs.headers.get("sec-websocket-protocol").cloned().unwrap_or_default();
                let selected = if hs.path == "/protocol_array" {
                    selected.split(',').next().unwrap_or("").trim().to_string()
                } else {
                    selected
                };
                write_frame(sock, 0x1, selected.as_bytes())?;
                server_close(sock, reader)
            }
            "/receive-many-with-backpressure" => {
                // `receive-many-with-backpressure_wsh`：每条延迟 100ms 后回字节长度。
                loop {
                    std::thread::sleep(Duration::from_millis(100));
                    match read_message(sock, reader)? {
                        Some(Incoming::Text(text)) => {
                            write_frame(sock, 0x1, text.len().to_string().as_bytes())?;
                        }
                        Some(Incoming::Binary(bytes)) => {
                            write_frame(sock, 0x1, bytes.len().to_string().as_bytes())?;
                        }
                        _ => return Ok(()),
                    }
                }
            }
            // `/echo_exit`、`/echo_close_data`：Goodbye 不回显 + 干净关闭（exit 分支）。
            "/echo_exit" | "/echo_close_data" => echo_loop(sock, reader, true),
            // `/handshake_sleep_2`：transfer_data return → pywebsocket 服务端关闭
            //（constructor/014 在 open 后等 close 事件凑满 events==4）。
            "/handshake_sleep_2" => server_close(sock, reader),
            // `/echo`、`/`：echo 语义（`echo_wsh`——loop 常驻直到客户端关闭/`Goodbye`）。
            "/echo" | "/" => echo_loop(sock, reader, false),
            "/delayed-passive-close" => {
                // `delayed-passive-close_wsh`：等客户端 close 帧 → sleep 1s → 回显
                // 码/原因（无状态码回空）——Close-delayed 的关闭握手耗时/wasClean 面。
                let code_reason = loop {
                    match read_frame(reader)? {
                        None => return Ok(()),
                        Some(WireFrame::Ping(p)) => write_frame(sock, 0xA, &p)?,
                        Some(WireFrame::Pong) => {}
                        Some(WireFrame::Close(cr)) => break cr,
                        Some(WireFrame::Data { .. }) => {}
                    }
                };
                std::thread::sleep(Duration::from_secs(1));
                let body = match code_reason {
                    Some((code, reason)) => {
                        let mut b = code.to_be_bytes().to_vec();
                        b.extend_from_slice(reason.as_bytes());
                        b
                    }
                    None => Vec::new(),
                };
                write_frame(sock, 0x8, &body)
            }
            // KNOWN_PATHS 白名单已把未知 path 挡在握手段——此臂仅编译器穷尽性兜底。
            _ => echo_loop(sock, reader, false),
        }
    })();
    let _ = result;
    let _ = sock.shutdown(std::net::Shutdown::Both);
}

/// 服务端主动关闭握手：发空 close 帧（1005 语义）→ 等客户端 close 回应 → 断开。
fn server_close(sock: &mut TcpStream, reader: &mut BufReader<TcpStream>) -> std::io::Result<()> {
    write_frame(sock, 0x8, &[])?;
    await_client_close(sock, reader)
}

/// echo 语义循环；`exit_on_goodbye`（`/echo_exit`）在 `Goodbye` 后**不发 close 帧**
/// 直接断 TCP（handler 退出 → 客户端 1006），否则发空 close 后等客户端回关
/// （passive close）。
fn echo_loop(sock: &mut TcpStream, reader: &mut BufReader<TcpStream>, exit_on_goodbye: bool) -> std::io::Result<()> {
    loop {
        match read_message(sock, reader)? {
            Some(Incoming::Text(text)) => {
                if text == "Goodbye" && exit_on_goodbye {
                    // `echo_exit_wsh`/`echo_close_data_wsh`：Goodbye 不回显（handler
                    // return 前无 send——002/004 案 onmessage=unreached 断言面），走
                    // 服务端干净关闭（pywebsocket handler 返回语义，wasClean=true）。
                    return server_close(sock, reader);
                }
                write_frame(sock, 0x1, text.as_bytes())?;
                if text == "Goodbye" {
                    // `/echo`（`echo_wsh`）：Goodbye **先回显**再 passive close。
                    return server_close(sock, reader);
                }
            }
            Some(Incoming::Binary(bytes)) => {
                write_frame(sock, 0x2, &bytes)?;
            }
            // text 载荷非 UTF-8：close 1007 后断开（RFC 6455 §7.1.5）。
            Some(Incoming::InvalidUtf8) => {
                write_frame(sock, 0x8, &1007u16.to_be_bytes())?;
                return Ok(());
            }
            _ => return Ok(()),
        }
    }
}

/// 等客户端 close（回显码/原因）或对端断开——不主动发数据。
fn await_client_close(sock: &mut TcpStream, reader: &mut BufReader<TcpStream>) -> std::io::Result<()> {
    while let Some(frame) = read_frame(reader)? {
        if let WireFrame::Close(code_reason) = frame {
            let body = match code_reason {
                Some((code, reason)) => {
                    let mut b = code.to_be_bytes().to_vec();
                    b.extend_from_slice(reason.as_bytes());
                    b
                }
                None => Vec::new(),
            };
            let _ = write_frame(sock, 0x8, &body);
            break;
        }
    }
    Ok(())
}

/// 上游 wptserve 有 handler 的 path 全集（模块注释逐一对应）；其余 path 404 →
/// 握手失败（opening-handshake/006 以 /invalid1-3 断言未升级 path 不干净失败）。
const KNOWN_PATHS: &[&str] = &[
    "/",
    "/echo",
    "/echo_exit",
    "/echo_close_data",
    "/handshake_sleep_2",
    "/empty-message",
    "/echo-cookie",
    "/echo-query",
    "/origin",
    "/referrer",
    "/protocol_array",
    "/protocol",
    "/receive-many-with-backpressure",
    "/handshake_no_protocol",
    "/handshake_no_extensions",
    "/handshake_protocol",
    "/wrong_accept_key",
    "/invalid",
    "/set-cookie",
    "/set-cookie_http",
    "/simple_handshake",
    "/delayed-passive-close",
    "/basic_auth",
];

fn handle_conn(sock: TcpStream) {
    let Ok(clone) = sock.try_clone() else { return };
    let mut reader = BufReader::new(clone);
    let Some(hs) = read_handshake(&mut reader) else { return };
    if !KNOWN_PATHS.contains(&hs.path.as_str()) {
        let mut sock = sock;
        let _ = sock.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        let _ = sock.shutdown(std::net::Shutdown::Both);
        return;
    }
    if hs.path == "/simple_handshake" {
        // `simple_handshake_wsh`：手工响应头（`101 Switching Protocols:` 带冒号为上游
        // 原样）+ Set-Cookie + Sec-WebSocket-Origin，紧随 1001 `PASS` close 帧同包。
        let key = hs.headers.get("sec-websocket-key").cloned().unwrap_or_default();
        let origin = hs.headers.get("origin").cloned().unwrap_or_default();
        let mut pkt = format!(
            "HTTP/1.1 101 Switching Protocols:\x0D\x0AConnection: Upgrade\x0D\x0AUpgrade: WebSocket\x0D\x0ASet-Cookie: ws_test=test\x0D\x0ASec-WebSocket-Origin: {origin}\x0D\x0ASec-WebSocket-Accept: {}\x0D\x0A\x0D\x0A",
            compute_accept(&key)
        )
        .into_bytes();
        // close body：code 1001 + reason "PASS"。
        let mut close_body = 1001u16.to_be_bytes().to_vec();
        close_body.extend_from_slice(b"PASS");
        let mut close_frame = vec![0x88];
        close_frame.push(close_body.len() as u8);
        close_frame.extend_from_slice(&close_body);
        pkt.extend_from_slice(&close_frame);
        let mut sock = sock;
        let _ = sock.write_all(&pkt);
        let _ = sock.shutdown(std::net::Shutdown::Both);
        return;
    }
    let response = handshake_response(&hs.path, &hs.query, &hs);
    let mut sock = sock;
    if sock.write_all(&response).is_err() {
        return;
    }
    // `/handshake_no_protocol`/`no_extensions`/`protocol`：transfer_data 为 pass——
    // handler 即返回，pywebsocket 走服务端关闭（constructor/012 依赖 close 事件）。
    if matches!(
        hs.path.as_str(),
        "/handshake_no_protocol" | "/handshake_no_extensions" | "/handshake_protocol"
    ) {
        let _ = server_close(&mut sock, &mut reader);
        let _ = sock.shutdown(std::net::Shutdown::Both);
        return;
    }
    // set-cookie 族与 `/wrong_accept_key`：握手即目的（前者等客户端关闭，后者客户端
    // 须拒连不派发 open）；`/handshake_sleep_2` 落 serve_data 的 echo 家族。
    if hs.path.starts_with("/set-cookie") || hs.path == "/wrong_accept_key" {
        let _ = await_client_close(&mut sock, &mut reader);
        let _ = sock.shutdown(std::net::Shutdown::Both);
        return;
    }
    serve_data(&mut sock, &mut reader, &hs);
}
