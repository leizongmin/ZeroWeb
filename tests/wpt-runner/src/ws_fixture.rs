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
//!   - `/echo_raw`：收到的消息**裸字节**写回 socket（无帧封装——客户端发
//!     `\x81\x04test\x83\x03LOL` 原始帧字节，回写后其解析器读出 text 帧 + 保留
//!     opcode 3 帧触发协议错误断连；binary `exit` → 服务端关闭）（net-api-compat-v3 M2）
//!   - `/sleep_10_v13`：握手响应延迟 10s（`do_extra_handshake` sleep——
//!     close-connecting 于 CONNECTING 中止时序面），握手后 text echo、
//!     `Goodbye` 服务端关闭（net-api-compat-v3 M2）
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
//!
//! **wss 端点**（runner-net-infra M1，D1 设计）：[`fixture_wss_port()`] 第二条
//! 监听——rustls `ServerConnection` TLS 包裹同一 handler 表（握手/帧/数据相经
//! [`FixtureStream`] 泛化 IO 层复用，明文/双态零行为分叉）；证书进程内 rcgen
//! 自签（SAN = DNS `localhost`/`wpt.test`/`*.wpt.test` + IP `127.0.0.1`），仅驻
//! 内存不落盘不入仓（自签 fixture CA + 叶证书两级——webpki 拒叶位 CA 证书）；
//! 同时把「仅 fixture 自签 CA」信任源经 zero-net
//! `install_wss_test_tls_config` 注入客户端覆盖钩子（D2——产品代码不设钩子，
//! TLS 语义 = runner 测试环境旁路，生产 webpki-roots 路径逐字节不变）。
//! TLS 传输层只改加密，端点行为表（`KNOWN_PATHS`/握手/帧语义）与明文端完全一致。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use rustls::ServerConnection;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
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
                        .spawn(move || handle_conn(FixtureStream::Plain(sock)))
                        .ok();
                }
            })
            .expect("ws fixture thread");
        port
    })
}

/// 全局 wss fixture 端口（进程级单 TLS server；对应上游 wptserve
/// `{{ports[wss][0]}}` 独立监听形态，与明文 [`fixture_ws_port()`] 双端口并存）。
/// 首拉惰性启动：rcgen 自签证书生成 → rustls `ServerConnection` 服务端栈 →
/// 「仅 fixture 自签 CA」信任源注入 zero-net 客户端覆盖钩子（D2）。
pub fn fixture_wss_port() -> u16 {
    static PORT: OnceLock<u16> = OnceLock::new();
    *PORT.get_or_init(|| {
        let cert = generate_fixture_cert().expect("wss fixture cert generate");
        let server_config = Arc::new(fixture_server_tls_config(&cert).expect("wss fixture TLS server config"));
        let client_config = Arc::new(fixture_client_tls_config(&cert).expect("wss fixture TLS client config"));
        zero_net::websocket::install_wss_test_tls_config(client_config);
        let listener = TcpListener::bind("127.0.0.1:0").expect("wss fixture bind");
        let port = listener.local_addr().expect("wss fixture addr").port();
        std::thread::Builder::new()
            .name("wss-fixture".into())
            .spawn(move || {
                for sock in listener.incoming() {
                    let Ok(sock) = sock else { break };
                    let _ = sock.set_read_timeout(Some(CONN_READ_TIMEOUT));
                    let config = Arc::clone(&server_config);
                    std::thread::Builder::new()
                        .name("wss-fixture-conn".into())
                        .spawn(move || {
                            let Ok(conn) = ServerConnection::new(config) else {
                                return;
                            };
                            handle_conn(FixtureStream::Tls(Box::new(rustls::StreamOwned::new(conn, sock))));
                        })
                        .ok();
                }
            })
            .expect("wss fixture thread");
        port
    })
}

/// 进程内生成两级自签证书（rcgen/ring，D1）：自签 fixture CA（信任锚）+ CA 签发
/// 叶证书。**单证方案不可行**——信任锚须 CA 位（rustls `RootCertStore::add` 经
/// webpki 校验），而叶位呈现 CA 证书被链校验拒（`CaUsedAsEndEntity`），两级为
/// rustls 最小可行形态。SAN 覆盖 fixture 客户端全部 server name 形态（`127.0.0.1`
/// 直连 + 改写面 `wpt.test`/`*.wpt.test`/`localhost`）。**仅驻内存**（划界「自签
/// 证书不出仓」：不落盘、不入库、不涉真 CA）。
struct FixtureCert {
    ca_der: CertificateDer<'static>,
    leaf_der: CertificateDer<'static>,
    leaf_key: PrivateKeyDer<'static>,
}

fn generate_fixture_cert() -> Result<FixtureCert, rcgen::Error> {
    // fixture CA：CA 位 + keyCertSign（信任锚要件）。
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new())?;
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign];
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "wpt-runner wss fixture CA");
    let ca_key = rcgen::KeyPair::generate()?;
    let ca_cert = ca_params.self_signed(&ca_key)?;
    // 叶证书：SAN 全覆盖 + ServerAuth（`IsCa` 默认 ExplicitNoCa——叶位非 CA）。
    let mut leaf_params = rcgen::CertificateParams::new(vec![
        "localhost".to_string(),
        "wpt.test".to_string(),
        "*.wpt.test".to_string(),
    ])?;
    leaf_params.subject_alt_names.push(rcgen::SanType::IpAddress(
        "127.0.0.1".parse().expect("fixture cert IP SAN"),
    ));
    leaf_params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
    leaf_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
    let leaf_key = rcgen::KeyPair::generate()?;
    let issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let leaf_cert = leaf_params.signed_by(&leaf_key, &issuer)?;
    Ok(FixtureCert {
        ca_der: ca_cert.der().clone(),
        leaf_der: leaf_cert.der().clone(),
        leaf_key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
    })
}

/// fixture 服务端 TLS 配置：叶证书 + CA 链 + 无客户端认证（runner 测试端点）。
/// 显式 ring provider——不依赖进程级默认（零影响生产 TLS 面）。
fn fixture_server_tls_config(cert: &FixtureCert) -> Result<rustls::ServerConfig, rustls::Error> {
    rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.leaf_der.clone(), cert.ca_der.clone()],
            cert.leaf_key.clone_key(),
        )
}

/// fixture 客户端 TLS 配置（D2 信任旁路）：信任根 = **仅** fixture 自签 CA——
/// runner wss 面只信任 fixture 端点，比生产 webpki-roots 全局信任收紧，旁路
/// 不外溢（防 runner 信任面污染其他 wss 目标）。
fn fixture_client_tls_config(cert: &FixtureCert) -> Result<rustls::ClientConfig, rustls::Error> {
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.ca_der.clone())?;
    Ok(
        rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()?
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
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

/// fixture 连接流双形态（明文 TCP / TLS 包裹）：rustls 流不可 `try_clone`，
/// 读写统一收敛为 `BufReader<FixtureStream>` 单句柄——握手/帧/数据相 handler
/// 经 [`Read`]/[`Write`] 泛化面复用，明文/wss 端点行为零分叉（M1 泛化 IO 层）。
enum FixtureStream {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<ServerConnection, TcpStream>>),
}

impl Read for FixtureStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(s) => s.read(buf),
            Self::Tls(s) => s.read(buf),
        }
    }
}

impl Write for FixtureStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(s) => s.write(buf),
            Self::Tls(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(s) => s.flush(),
            Self::Tls(s) => s.flush(),
        }
    }
}

impl FixtureStream {
    /// 关闭连接（TLS 侧先发 close_notify 再关 TCP——干净 TLS 收尾）。
    fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(s) => s.shutdown(std::net::Shutdown::Both),
            Self::Tls(s) => {
                s.conn.send_close_notify();
                let _ = s.flush();
                s.sock.shutdown(std::net::Shutdown::Both)
            }
        }
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

fn read_handshake(reader: &mut BufReader<FixtureStream>) -> Option<Handshake> {
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

fn write_frame(sock: &mut FixtureStream, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
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

fn read_frame(reader: &mut BufReader<FixtureStream>) -> std::io::Result<Option<WireFrame>> {
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
fn read_message(conn: &mut BufReader<FixtureStream>) -> std::io::Result<Option<Incoming>> {
    let mut fragments: Vec<u8> = Vec::new();
    let mut text_msg = false;
    loop {
        let Some(frame) = read_frame(conn)? else {
            return Ok(None);
        };
        match frame {
            WireFrame::Ping(payload) => write_frame(conn.get_mut(), 0xA, &payload)?,
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
                let _ = write_frame(conn.get_mut(), 0x8, &body);
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

fn serve_data(conn: &mut BufReader<FixtureStream>, hs: &Handshake) {
    let result: std::io::Result<()> = (|| {
        match hs.path.as_str() {
            // 单发型 handler（上游 transfer_data 单发即 return——pywebsocket handler
            // 返回后走服务端关闭握手）：发完发空 close 帧再等客户端回关。
            "/empty-message" => {
                // `empty-message_wsh`：收一条，空串回 pass 否则 fail。
                if let Some(Incoming::Text(text)) = read_message(conn)? {
                    let reply = if text.is_empty() { "pass" } else { "fail" };
                    write_frame(conn.get_mut(), 0x1, reply.as_bytes())?;
                }
                server_close(conn)
            }
            "/echo-cookie" => {
                let cookie = hs.headers.get("cookie").cloned();
                let reply = cookie.unwrap_or_else(|| "(none)".to_string());
                write_frame(conn.get_mut(), 0x1, reply.as_bytes())?;
                server_close(conn)
            }
            "/echo_raw" => {
                // net-api-compat-v3 M2：`echo_raw_wsh`——收到的消息**裸字节**写回
                // socket（`request.connection.write(line)`，无 WS 帧封装）：客户端把
                // `\x81\x04test\x83\x03LOL` 作一条 binary 消息发来，回写后其解析器
                // 逐帧读出 text "test" + 保留 opcode 3 帧（协议错误 → fail 连接，
                // events/015/017/018 的 message+error+close 断言面）；binary
                // `b'exit'` → handler return（服务端关闭）。
                loop {
                    match read_message(conn)? {
                        Some(Incoming::Binary(bytes)) => {
                            if bytes == b"exit" {
                                return server_close(conn);
                            }
                            conn.get_mut().write_all(&bytes)?;
                        }
                        // 上游 write(str)：text 帧同 raw 字节回写。
                        Some(Incoming::Text(text)) => conn.get_mut().write_all(text.as_bytes())?,
                        // 载荷非 UTF-8（上游 receive_message 抛错断连）→ close 1007。
                        Some(Incoming::InvalidUtf8) => {
                            write_frame(conn.get_mut(), 0x8, &1007u16.to_be_bytes())?;
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
            }
            "/sleep_10_v13" => {
                // net-api-compat-v3 M2：`sleep_10_v13_wsh` 的 transfer_data——text
                // echo，`Goodbye` return（不回显）→ 服务端关闭；binary 恒按 text 回发
                //（`send_message(line, binary=False)`）。
                loop {
                    match read_message(conn)? {
                        Some(Incoming::Text(text)) => {
                            if text == "Goodbye" {
                                return server_close(conn);
                            }
                            write_frame(conn.get_mut(), 0x1, text.as_bytes())?;
                        }
                        Some(Incoming::Binary(bytes)) => write_frame(conn.get_mut(), 0x1, &bytes)?,
                        _ => return Ok(()),
                    }
                }
            }
            "/echo-query" | "/echo-query_v13" => {
                // net-api 二期 M2-S3b：websockets constructor/016 用上游 echo-query_v13
                // handler（行为同 echo-query——原样回 query 段；_v13 为 wptserve
                // handler 变体名，非语义分叉）。
                write_frame(conn.get_mut(), 0x1, hs.query.as_bytes())?;
                server_close(conn)
            }
            "/origin" => {
                let origin = hs.headers.get("origin").cloned().unwrap_or_default();
                write_frame(conn.get_mut(), 0x1, origin.as_bytes())?;
                server_close(conn)
            }
            "/referrer" => {
                // FETCH 语义 WS 握手不携带 Referer——缺失即 `MISSING AS PER FETCH`。
                let referer = hs
                    .headers
                    .get("referer")
                    .cloned()
                    .unwrap_or_else(|| "MISSING AS PER FETCH".to_string());
                write_frame(conn.get_mut(), 0x1, referer.as_bytes())?;
                server_close(conn)
            }
            "/protocol_array" | "/protocol" => {
                // 协商值已在握手头选定——回发之（handler 单发即返回）。
                let selected = hs.headers.get("sec-websocket-protocol").cloned().unwrap_or_default();
                let selected = if hs.path == "/protocol_array" {
                    selected.split(',').next().unwrap_or("").trim().to_string()
                } else {
                    selected
                };
                write_frame(conn.get_mut(), 0x1, selected.as_bytes())?;
                server_close(conn)
            }
            "/receive-many-with-backpressure" => {
                // `receive-many-with-backpressure_wsh`：每条延迟 100ms 后回字节长度。
                loop {
                    std::thread::sleep(Duration::from_millis(100));
                    match read_message(conn)? {
                        Some(Incoming::Text(text)) => {
                            write_frame(conn.get_mut(), 0x1, text.len().to_string().as_bytes())?;
                        }
                        Some(Incoming::Binary(bytes)) => {
                            write_frame(conn.get_mut(), 0x1, bytes.len().to_string().as_bytes())?;
                        }
                        _ => return Ok(()),
                    }
                }
            }
            // `/echo_exit`、`/echo_close_data`：Goodbye 不回显 + 干净关闭（exit 分支）。
            "/echo_exit" | "/echo_close_data" => echo_loop(conn, true),
            // `/handshake_sleep_2`：transfer_data return → pywebsocket 服务端关闭
            //（constructor/014 在 open 后等 close 事件凑满 events==4）。
            "/handshake_sleep_2" => server_close(conn),
            // `/echo`、`/`：echo 语义（`echo_wsh`——loop 常驻直到客户端关闭/`Goodbye`）。
            "/echo" | "/" => echo_loop(conn, false),
            "/delayed-passive-close" => {
                // `delayed-passive-close_wsh`：等客户端 close 帧 → sleep 1s → 回显
                // 码/原因（无状态码回空）——Close-delayed 的关闭握手耗时/wasClean 面。
                let code_reason = loop {
                    match read_frame(conn)? {
                        None => return Ok(()),
                        Some(WireFrame::Ping(p)) => write_frame(conn.get_mut(), 0xA, &p)?,
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
                write_frame(conn.get_mut(), 0x8, &body)
            }
            // KNOWN_PATHS 白名单已把未知 path 挡在握手段——此臂仅编译器穷尽性兜底。
            _ => echo_loop(conn, false),
        }
    })();
    let _ = result;
    let _ = conn.get_mut().shutdown();
}

/// 服务端主动关闭握手：发空 close 帧（1005 语义）→ 等客户端 close 回应 → 断开。
fn server_close(conn: &mut BufReader<FixtureStream>) -> std::io::Result<()> {
    write_frame(conn.get_mut(), 0x8, &[])?;
    await_client_close(conn)
}

/// echo 语义循环；`exit_on_goodbye`（`/echo_exit`）在 `Goodbye` 后**不发 close 帧**
/// 直接断 TCP（handler 退出 → 客户端 1006），否则发空 close 后等客户端回关
/// （passive close）。
fn echo_loop(conn: &mut BufReader<FixtureStream>, exit_on_goodbye: bool) -> std::io::Result<()> {
    loop {
        match read_message(conn)? {
            Some(Incoming::Text(text)) => {
                if text == "Goodbye" && exit_on_goodbye {
                    // `echo_exit_wsh`/`echo_close_data_wsh`：Goodbye 不回显（handler
                    // return 前无 send——002/004 案 onmessage=unreached 断言面），走
                    // 服务端干净关闭（pywebsocket handler 返回语义，wasClean=true）。
                    return server_close(conn);
                }
                write_frame(conn.get_mut(), 0x1, text.as_bytes())?;
                if text == "Goodbye" {
                    // `/echo`（`echo_wsh`）：Goodbye **先回显**再 passive close。
                    return server_close(conn);
                }
            }
            Some(Incoming::Binary(bytes)) => {
                write_frame(conn.get_mut(), 0x2, &bytes)?;
            }
            // text 载荷非 UTF-8：close 1007 后断开（RFC 6455 §7.1.5）。
            Some(Incoming::InvalidUtf8) => {
                write_frame(conn.get_mut(), 0x8, &1007u16.to_be_bytes())?;
                return Ok(());
            }
            _ => return Ok(()),
        }
    }
}

/// 等客户端 close（回显码/原因）或对端断开——不主动发数据。
fn await_client_close(conn: &mut BufReader<FixtureStream>) -> std::io::Result<()> {
    while let Some(frame) = read_frame(conn)? {
        if let WireFrame::Close(code_reason) = frame {
            let body = match code_reason {
                Some((code, reason)) => {
                    let mut b = code.to_be_bytes().to_vec();
                    b.extend_from_slice(reason.as_bytes());
                    b
                }
                None => Vec::new(),
            };
            let _ = write_frame(conn.get_mut(), 0x8, &body);
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
    "/echo_raw",
    "/sleep_10_v13",
    "/echo_exit",
    "/echo_close_data",
    "/handshake_sleep_2",
    "/empty-message",
    "/echo-cookie",
    "/echo-query",
    "/echo-query_v13",
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

fn handle_conn(stream: FixtureStream) {
    let mut reader = BufReader::new(stream);
    let Some(hs) = read_handshake(&mut reader) else { return };
    if !KNOWN_PATHS.contains(&hs.path.as_str()) {
        let _ = reader
            .get_mut()
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        let _ = reader.get_mut().shutdown();
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
        let _ = reader.get_mut().write_all(&pkt);
        let _ = reader.get_mut().shutdown();
        return;
    }
    if hs.path == "/sleep_10_v13" {
        // net-api-compat-v3 M2：`sleep_10_v13_wsh` 的 `do_extra_handshake` sleep
        // 10s——101 延迟 10s 发出。close-connecting 于 CONNECTING 1s 时 close()
        // 中止连接（断言 wasClean=false + CLOSED 事件不等服务端）；对端早断后本
        // 线程写握手响应失败即退出，无害。
        std::thread::sleep(Duration::from_secs(10));
    }
    let response = handshake_response(&hs.path, &hs.query, &hs);
    if reader.get_mut().write_all(&response).is_err() {
        return;
    }
    // `/handshake_no_protocol`/`no_extensions`/`protocol`：transfer_data 为 pass——
    // handler 即返回，pywebsocket 走服务端关闭（constructor/012 依赖 close 事件）。
    if matches!(
        hs.path.as_str(),
        "/handshake_no_protocol" | "/handshake_no_extensions" | "/handshake_protocol"
    ) {
        let _ = server_close(&mut reader);
        let _ = reader.get_mut().shutdown();
        return;
    }
    // set-cookie 族与 `/wrong_accept_key`：握手即目的（前者等客户端关闭，后者客户端
    // 须拒连不派发 open）；`/handshake_sleep_2` 落 serve_data 的 echo 家族。
    if hs.path.starts_with("/set-cookie") || hs.path == "/wrong_accept_key" {
        let _ = await_client_close(&mut reader);
        let _ = reader.get_mut().shutdown();
        return;
    }
    serve_data(&mut reader, &hs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_net::websocket::{WebSocket, WebSocketMessage};

    /// runner-net-infra M1（DC-2 前半）wss 端到端连通：`fixture_wss_port()` TLS 端点
    /// ↔ zero-net `WebSocket` 真栈（`default_net_ws_host` 的委派底层；ws_bridge 泵面
    /// 与 TLS 无关，由 engine 既有明文 e2e 覆盖）经 [`FixtureWsHost`] 同款权威改写
    /// 连 `wss://wpt.test:<port>/echo` → TLS 握手 + 101 → text echo 往返 → 带码
    /// 关闭握手。覆盖面：rcgen 自签证书（SAN = 127.0.0.1/wpt.test 域）+ rustls
    /// 服务端握手 + D2 客户端信任旁路（钩子未设的 webpki-roots 生产面零改动）。
    #[test]
    fn wss_end_to_end_echo_roundtrip() {
        let port = fixture_wss_port();
        // 宿主同款改写：wpt.test → 127.0.0.1（证书 IP SAN 即为此形态配）。
        let url = rewrite_wpt_authority(&format!("wss://wpt.test:{port}/echo"));
        let mut ws = WebSocket::new(&url);
        ws.connect().expect("wss TLS 握手 + 101 升级");
        ws.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("wss read timeout");
        ws.send("hello-wss").expect("wss text send");
        match ws.receive() {
            Ok(Some(WebSocketMessage::Text(t))) => assert_eq!(t, "hello-wss", "wss echo 往返"),
            other => panic!("wss text echo 往返失败：{other:?}"),
        }
        ws.close_with(1000, "done").expect("wss 带码关闭握手");
    }

    /// M1 泛化 IO 层（`TcpStream` 双句柄 → `BufReader<FixtureStream>` 单句柄）的
    /// 明文面零行为分叉钉：同真栈连明文 `/echo`（握手/帧/echo/close 语义不变）。
    #[test]
    fn ws_plain_end_to_end_echo_roundtrip() {
        let port = fixture_ws_port();
        let mut ws = WebSocket::new(&format!("ws://127.0.0.1:{port}/echo"));
        ws.connect().expect("ws 101 升级");
        ws.set_read_timeout(Some(Duration::from_secs(5)))
            .expect("ws read timeout");
        ws.send("hello-plain").expect("ws text send");
        match ws.receive() {
            Ok(Some(WebSocketMessage::Text(t))) => assert_eq!(t, "hello-plain", "ws echo 往返"),
            other => panic!("ws text echo 往返失败：{other:?}"),
        }
        ws.close_with(1000, "done").expect("ws 带码关闭握手");
    }
}
