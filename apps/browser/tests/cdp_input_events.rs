//! CDP 宿主文本插入输入事件钉（slice23 input events，slice22 移交 2b）。
//!
//! 同桶判别结论（2026-10-03，base e93df1d4a，探针 JSON 存档 diag/evidence/slice23/）：
//! slice22 的「值落事件缺」签名**被证伪**——本地页与线上 baidu 上宿主文本插入（CDP
//! keyDown 带 text / Input.insertText）均已有 trusted 的 `beforeinput`/`input`/`keyup`
//! 且 value 落地。与 Chrome/154 同桶比对，判别集内唯一引擎层缺口是 **`keypress` 缺失**：
//! Chrome 序 keydown → keypress → beforeinput → input → keyup；ZW 跳过 keypress，
//! 其余逐面一致。WPT runner 路径早已合成 keypress（testharness.rs「未取消的字符键
//! keydown 之后派 keypress」），renderer CDP 路径未对齐——本钉把该缺口钉在引擎最小
//! 复现上，修复点 renderer `handle_keyboard_event`（keydown 派发与字符插入默认动作之间）。
//!
//! 钉面（真实进程：zero-browser + zero-renderer + 本地静态 HTTP，CDP ws 逐连接独占）：
//! 1. `host_key_text_insertion_dispatches_full_ui_events_sequence`（宿主插入事件钉，
//!    RED 先行）——textarea 页点击聚焦后 CDP 逐键键入 "we"（keyDown 带 text + keyUp）：
//!    逐字符 keydown → keypress（key 语义面 + trusted）→ beforeinput（cancelable，
//!    inputType=insertText，data）→ input（同面）→ keyup，value === "we"。base 预期
//!    翻红（keypress 缺失）；修复后转绿。
//! 2. `insert_text_command_lands_value_and_dispatches_input_events`（insertText 值落钉，
//!    slice21 挂账同族判别面，负控制）——CDP `Input.insertText` 于单行 input 变体页：
//!    value 与 beforeinput/input 双断言。**base 即绿**（slice21 的「insertText 不落值」
//!    面在 base 不复现，本钉转负控制防回退）。
//! 3. `raw_key_down_without_text_skips_keypress_and_insertion`（无 text 边界负控制钉）——
//!    rawKeyDown（无 text）只派 keydown/keyup 事件轴：无 keypress、无 beforeinput/input、
//!    value 不变。keypress 仅属于产生字符值的键（UI Events character value 判据）。
//! 4. `keypress_synth_kill_switch_reverts_to_base_sequence`（kill-switch 常驻钉）——
//!    `ZW_KEYPRESS_SYNTH=0` 子进程注入回落：无 keypress、插入轴照常，逐字节回到 base
//!    事件序。
//!
//! Harness 同 slice22 `cdp_focus_governance.rs`（TestSite/CdpClient/spawn 前提断言）。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// 测试页服务：线程化静态 HTTP（进程生命周期 = 测试函数）。
struct TestSite {
    port: u16,
    #[allow(dead_code)]
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

/// 输入事件轴监听样板：document 捕获监听全键盘/输入事件族，记录语义面 JSON 到
/// `window.__elog`（type/target/key/data/inputType/isComposing/cancelable/trusted/value）。
const INPUT_LISTENER_SNIPPET: &str = r#"
window.__elog = [];
['keydown','keypress','keyup','beforeinput','input','compositionstart','compositionupdate','compositionend'].forEach(function (t) {
  document.addEventListener(t, function (e) {
    var tg = e.target && e.target.id ? e.target.id : (e.target && e.target.tagName ? e.target.tagName : 'unknown');
    var v = null;
    try { v = (e.target && e.target.value !== undefined) ? String(e.target.value).slice(0, 24) : null; } catch (_) {}
    window.__elog.push({
      type: t, target: tg,
      key: e.key !== undefined ? e.key : null,
      data: e.data !== undefined ? e.data : null,
      inputType: e.inputType !== undefined ? e.inputType : null,
      isComposing: e.isComposing === true,
      cancelable: e.cancelable === true,
      trusted: e.isTrusted === true,
      value: v
    });
  }, true);
});"#;

impl TestSite {
    /// 启动静态服务。路径集固定：/insert（textarea 聚焦键入页）、/insert-input（单行 input 变体）。
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test site");
        let port = listener.local_addr().expect("addr").port();
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let body = match path.split('?').next().unwrap_or("/") {
                    // 宿主插入事件钉页：可见 textarea（点击聚焦），无 boot focus。
                    "/insert" => format!(
                        r#"<!DOCTYPE html><html><head><title>S23 Insert</title><style>
#T{{position:fixed;left:40px;top:40px;width:300px;height:60px}}
</style></head><body>
<textarea id="T"></textarea>
<script>{INPUT_LISTENER_SNIPPET}</script>
</body></html>"#
                    ),
                    // 单行 input 变体（邻近边界：input 与 textarea 通道一致性）。
                    "/insert-input" => format!(
                        r#"<!DOCTYPE html><html><head><title>S23 InsertInput</title><style>
#I{{position:fixed;left:40px;top:40px;width:300px;height:28px}}
</style></head><body>
<input id="I">
<script>{INPUT_LISTENER_SNIPPET}</script>
</body></html>"#
                    ),
                    _ => "<!DOCTYPE html><html><head><title>S23 NotFound</title></head><body><h1>ok</h1></body></html>"
                        .to_string(),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        Self { port, shutdown }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{port}{path}", port = self.port)
    }
}

/// 被测浏览器进程（RAII 关停）。
struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪（默认 env）。
fn spawn_browser() -> (u16, BrowserProcess) {
    spawn_browser_with_env(&[])
}

/// 同上，附加子进程 env 注入（kill-switch 钉用）。
fn spawn_browser_with_env(envs: &[(&str, &str)]) -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-s23-pin-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    // renderer 与 browser 同 target 目录（dev profile），取同级；ZERO_S23_RENDERER_PATH 可覆盖。
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::env::var("ZERO_S23_RENDERER_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::Path::new(browser_bin)
                .parent()
                .expect("browser bin dir")
                .join("zero-renderer")
        });
    assert_renderer_bin_current(&renderer);

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn zero-browser");

    // 等 /json/version 就绪（首次启动含 renderer 子进程拉起）。
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut ready = false;
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.write_all(b"GET /json/version HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = String::new();
            if stream.read_to_string(&mut buf).is_ok() && buf.contains("200 OK") {
                ready = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(ready, "zero-browser CDP endpoint not ready on port {port}");
    (port, BrowserProcess(child))
}

/// renderer bin spawn 前存在性 + 陈旧断言（同 slice22 T-S3——陈旧 peer-bin 假绿面消除）。
fn assert_renderer_bin_current(renderer: &std::path::Path) {
    assert!(
        renderer.exists(),
        "renderer binary missing at {} — build it explicitly (cargo build -p zero-renderer) before running this suite",
        renderer.display()
    );
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../renderer/src");
    let bin_mtime = std::fs::metadata(renderer)
        .and_then(|m| m.modified())
        .expect("renderer bin mtime readable");
    if let Some(newest) = newest_source_mtime(&src_dir) {
        assert!(
            bin_mtime >= newest,
            "renderer binary at {} is stale (bin mtime {bin_mtime:?} < newest renderer source mtime {newest:?}) — rebuild zero-renderer",
            renderer.display()
        );
    }
}

/// `dir` 下递归最新 `.rs` 源码 mtime（无目录/无源码 → None——跳过陈旧判定）。
fn newest_source_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut newest: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            newest_source_mtime(&path)
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            entry.metadata().and_then(|m| m.modified()).ok()
        } else {
            None
        };
        if let Some(t) = candidate {
            newest = Some(match newest {
                Some(prev) => prev.max(t),
                None => t,
            });
        }
    }
    newest
}

/// 最小 CDP 客户端（同步 ws；每连接独占——与 s22 钉形态一致）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
}

impl CdpClient {
    fn connect(port: u16) -> Self {
        let stream = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let (ws, _resp) = tungstenite::client(format!("ws://127.0.0.1:{port}"), stream).expect("ws handshake");
        Self { ws, next_id: 0 }
    }

    /// 发送命令并等待响应（读超时 = 断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            assert!(Instant::now() < deadline, "timeout waiting for {method} response");
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).expect("cdp json");
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return msg;
                    }
                }
                Ok(Message::Ping(data)) => {
                    self.ws.send(Message::Pong(data)).expect("pong");
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => panic!("ws read: {e}"),
            }
        }
    }

    /// evaluate 表达式并取 returnByValue 结果值。
    fn evaluate(&mut self, expression: &str, timeout: Duration) -> Value {
        let response = self.call(
            "Runtime.evaluate",
            serde_json::json!({ "expression": expression, "returnByValue": true }),
            timeout,
        );
        response.pointer("/result/result/value").cloned().unwrap_or(Value::Null)
    }

    /// CDP 点击：mousePressed + mouseReleased(clickCount:1)——mousedown 默认动作聚焦。
    fn click(&mut self, x: f64, y: f64) {
        self.call(
            "Input.dispatchMouseEvent",
            serde_json::json!({ "type": "mousePressed", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            Duration::from_secs(10),
        );
        self.call(
            "Input.dispatchMouseEvent",
            serde_json::json!({ "type": "mouseReleased", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            Duration::from_secs(10),
        );
    }

    /// CDP 逐键键入（keyDown 带 text + keyUp——s19/s22 探针同形态）。
    fn type_key(&mut self, ch: char) {
        let key = ch.to_string();
        let upper = ch.to_ascii_uppercase();
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyDown", "text": key, "key": key,
                "code": format!("Key{upper}"),
                "windowsVirtualKeyCode": upper as i64
            }),
            Duration::from_secs(10),
        );
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyUp", "key": key,
                "code": format!("Key{upper}"),
                "windowsVirtualKeyCode": upper as i64
            }),
            Duration::from_secs(10),
        );
    }

    /// CDP rawKeyDown（无 text——修饰键/导航键宿主形态）+ keyUp。
    fn raw_key(&mut self, key: &str, code: &str) {
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({ "type": "rawKeyDown", "key": key, "code": code }),
            Duration::from_secs(10),
        );
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({ "type": "keyUp", "key": key, "code": code }),
            Duration::from_secs(10),
        );
    }

    /// 读取页面 `window.__elog`（语义面 JSON 数组）。
    fn event_log(&mut self) -> Vec<Value> {
        let raw = self.evaluate("JSON.stringify(window.__elog || [])", Duration::from_secs(10));
        let parsed: Value = serde_json::from_str(raw.as_str().unwrap_or("[]")).expect("parse event log");
        parsed.as_array().cloned().unwrap_or_default()
    }
}

/// log 中首个匹配 `type` + `data`（编辑事件面）+ `key`（键盘事件面）的事件下标。
fn index_of(log: &[Value], event_type: &str, data: Option<&str>, key: Option<&str>) -> Option<usize> {
    log.iter().position(|e| {
        e.get("type").and_then(Value::as_str) == Some(event_type)
            && match data {
                Some(d) => e.get("data").and_then(Value::as_str) == Some(d),
                None => true,
            }
            && match key {
                Some(k) => e.get("key").and_then(Value::as_str) == Some(k),
                None => true,
            }
    })
}

/// 导航 path 并点击目标控件聚焦（返回前 settle 1.2s——注入时机已修面）。
fn navigate_and_focus(cdp: &mut CdpClient, site: &TestSite, path: &str, focus_id: &str) {
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url(path) }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));
    cdp.click(100.0, 60.0); // #T/#I（left:40 top:40，宽 300 高 60/28——中心命中）
    std::thread::sleep(Duration::from_millis(1200));
    // 前提自证：焦点已落目标控件（slice22 已修的状态轴面）。
    let active = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', value: document.activeElement && document.activeElement.value !== undefined ? document.activeElement.value : null})",
        Duration::from_secs(10),
    );
    let parsed: Value = serde_json::from_str(active.as_str().expect("active json")).expect("parse active");
    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some(focus_id),
        "click must focus the target control before typing (got {active}) — slice22 focus face regression"
    );
}

/// 钉 1（宿主插入事件钉，RED 先行）：CDP 逐键键入 "we" → 逐字符
/// keydown → keypress → beforeinput(insertText,data) → input(同面) → keyup 全序 +
/// value === "we"。keypress 面：key 语义 + target + trusted（R312 宿主派发可信戳）。
/// base 预期翻红（keypress 缺失——同桶判别的引擎最小面）。
///
/// 键序背离注记：本钉冻结的序是 keypress 在 beforeinput/input 之前——Chrome/154
/// 实测同桶序（探针 JSON 存档 s23-chrome-local.json），也是两合成点（WPT runner
/// send_keys 与宿主 CDP 路径）的现行实现序；UI Events §8.3.2 的字面处理序与之
/// 相反。若未来统一两合成点键序、对齐规范字面序，须同步修改本钉的序断言。
// https://w3c.github.io/uievents/#keypress-event-order （keypress 相对 beforeinput/
//   input 的位置句在此；#events-keyboard-event-order 的字面序与实现序相反，勿引）
// https://w3c.github.io/uievents/#event-type-keypress
// https://w3c.github.io/input-events/#interface-InputEvent
#[test]
fn host_key_text_insertion_dispatches_full_ui_events_sequence() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    navigate_and_focus(&mut cdp, &site, "/insert", "T");

    for ch in ['w', 'e'] {
        cdp.type_key(ch);
        std::thread::sleep(Duration::from_millis(300));
    }
    std::thread::sleep(Duration::from_millis(600));

    let value = cdp.evaluate("document.getElementById('T').value", Duration::from_secs(10));
    assert_eq!(
        value.as_str(),
        Some("we"),
        "host key text insertion must land the value (got {value:?})"
    );

    let log = cdp.event_log();
    assert!(
        !log.is_empty(),
        "input event axis must reach page listeners (log empty)"
    );
    for ch in ['w', 'e'] {
        let expected = ch.to_string();
        let keydown = index_of(&log, "keydown", None, Some(&expected)).expect("keydown event missing");
        let keypress = index_of(&log, "keypress", None, Some(&expected)).unwrap_or_else(|| {
            panic!("keypress missing after text-bearing keydown in {log:?} — UI Events sequence gap")
        });
        let beforeinput = index_of(&log, "beforeinput", Some(&expected), None)
            .unwrap_or_else(|| panic!("beforeinput(insertText,data={expected}) missing in {log:?}"));
        let input = index_of(&log, "input", Some(&expected), None)
            .unwrap_or_else(|| panic!("input(insertText,data={expected}) missing in {log:?}"));
        let keyup =
            index_of(&log, "keyup", None, Some(&expected)).unwrap_or_else(|| panic!("keyup event missing in {log:?}"));
        assert!(
            keydown < keypress && keypress < beforeinput && beforeinput < input && input < keyup,
            "per-character event order must be keydown < keypress < beforeinput < input < keyup for '{expected}' \
             (keydown={keydown} keypress={keypress} beforeinput={beforeinput} input={input} keyup={keyup}; log: {log:?})"
        );
        // keypress 语义面：key=字符、target=聚焦控件、trusted（宿主合成也必须页面可见可信）。
        let kp = &log[keypress];
        assert_eq!(
            kp.get("key").and_then(Value::as_str),
            Some(expected.as_str()),
            "keypress.key face (got {kp})"
        );
        assert_eq!(
            kp.get("target").and_then(Value::as_str),
            Some("T"),
            "keypress target face (got {kp})"
        );
        assert!(
            kp.get("trusted").and_then(Value::as_bool).unwrap_or(false),
            "keypress must be trusted (got {kp})"
        );
        // beforeinput 语义面：cancelable + inputType=insertText；input 同 inputType。
        let bi = &log[beforeinput];
        assert_eq!(
            bi.get("inputType").and_then(Value::as_str),
            Some("insertText"),
            "beforeinput.inputType must be insertText (got {bi})"
        );
        assert!(
            bi.get("cancelable").and_then(Value::as_bool).unwrap_or(false),
            "beforeinput must be cancelable (got {bi})"
        );
        assert_eq!(bi.get("target").and_then(Value::as_str), Some("T"));
        let ip = &log[input];
        assert_eq!(
            ip.get("inputType").and_then(Value::as_str),
            Some("insertText"),
            "input.inputType must be insertText (got {ip})"
        );
    }
}

/// 钉 2（insertText 值落钉，slice21 挂账同族判别面，负控制）：CDP `Input.insertText`
/// 于单行 input 变体页 → value === "we" 且 beforeinput(insertText,data="we") →
/// input(同面)。base 即绿（slice21 的 insertText 面在 base 不复现——同桶判别在案），
/// 本钉防该轴回退。
// https://w3c.github.io/input-events/#interface-InputEvent
// https://chromedevtools.github.io/devtools-protocol/tot/Input/#method-insertText
#[test]
fn insert_text_command_lands_value_and_dispatches_input_events() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    navigate_and_focus(&mut cdp, &site, "/insert-input", "I");

    cdp.call(
        "Input.insertText",
        serde_json::json!({ "text": "we" }),
        Duration::from_secs(10),
    );
    std::thread::sleep(Duration::from_millis(600));

    let value = cdp.evaluate("document.getElementById('I').value", Duration::from_secs(10));
    assert_eq!(
        value.as_str(),
        Some("we"),
        "Input.insertText must land the value (got {value:?})"
    );

    let log = cdp.event_log();
    let beforeinput = index_of(&log, "beforeinput", Some("we"), None)
        .unwrap_or_else(|| panic!("beforeinput(insertText,data=we) missing in {log:?}"));
    let input = index_of(&log, "input", Some("we"), None)
        .unwrap_or_else(|| panic!("input(insertText,data=we) missing in {log:?}"));
    assert!(
        beforeinput < input,
        "beforeinput must precede input (beforeinput={beforeinput} input={input}; log: {log:?})"
    );
    assert_eq!(
        log[beforeinput].get("inputType").and_then(Value::as_str),
        Some("insertText"),
        "beforeinput.inputType must be insertText (got {})",
        log[beforeinput]
    );
    assert!(
        !log.iter()
            .any(|e| e.get("type").and_then(Value::as_str) == Some("keydown")),
        "Input.insertText is not a key axis — no keydown expected (log: {log:?})"
    );
}

/// 钉 3（无 text 边界负控制钉）：rawKeyDown（无 text）只派 keydown/keyup 事件轴——
/// 无 keypress（keypress 仅属于产生字符值的键）、无 beforeinput/input、value 不变。
/// base 预期翻红（text→key 坍缩使 rawKeyDown 幻插入、值落 "w"，见 s23-pin-red-green.md
/// RED 实录）；修复后绿。守护 keypress 的 text 判据不被放大到全部键。
// https://w3c.github.io/uievents/#event-type-keypress
#[test]
fn raw_key_down_without_text_skips_keypress_and_insertion() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    navigate_and_focus(&mut cdp, &site, "/insert", "T");

    cdp.raw_key("w", "KeyW");
    std::thread::sleep(Duration::from_millis(600));

    let value = cdp.evaluate("document.getElementById('T').value", Duration::from_secs(10));
    assert_eq!(
        value.as_str(),
        Some(""),
        "rawKeyDown without text must not insert (got {value:?})"
    );

    let log = cdp.event_log();
    let keydown = index_of(&log, "keydown", None, Some("w"))
        .unwrap_or_else(|| panic!("keydown must reach page listeners (log: {log:?})"));
    assert_eq!(
        log[keydown].get("key").and_then(Value::as_str),
        Some("w"),
        "keydown.key semantic face must be present (got {})",
        log[keydown]
    );
    let keyup = index_of(&log, "keyup", None, Some("w"))
        .unwrap_or_else(|| panic!("keyup must reach page listeners (log: {log:?})"));
    assert!(
        keydown < keyup,
        "keydown must precede keyup (keydown={keydown} keyup={keyup}; log: {log:?})"
    );
    assert!(
        index_of(&log, "keypress", None, None).is_none(),
        "keypress must not fire without text (log: {log:?})"
    );
    assert!(
        index_of(&log, "beforeinput", None, None).is_none() && index_of(&log, "input", None, None).is_none(),
        "no editing events without text (log: {log:?})"
    );
}

/// 钉 4（kill-switch 常驻钉）：`ZW_KEYPRESS_SYNTH=0` 子进程注入回落——无 keypress、
/// 字符插入轴照常（beforeinput/input/value 全在），逐字节回到 base 事件序。
/// base（无合成路径）与修复后（门关断回落）均须绿。
#[test]
fn keypress_synth_kill_switch_reverts_to_base_sequence() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser_with_env(&[("ZW_KEYPRESS_SYNTH", "0")]);
    let mut cdp = CdpClient::connect(port);
    navigate_and_focus(&mut cdp, &site, "/insert", "T");

    cdp.type_key('w');
    std::thread::sleep(Duration::from_millis(600));

    let value = cdp.evaluate("document.getElementById('T').value", Duration::from_secs(10));
    assert_eq!(
        value.as_str(),
        Some("w"),
        "insertion axis must stay on with gate off (got {value:?})"
    );

    let log = cdp.event_log();
    let keydown = index_of(&log, "keydown", None, Some("w")).expect("keydown missing");
    let beforeinput = index_of(&log, "beforeinput", Some("w"), None)
        .unwrap_or_else(|| panic!("beforeinput missing with gate off in {log:?}"));
    let input =
        index_of(&log, "input", Some("w"), None).unwrap_or_else(|| panic!("input missing with gate off in {log:?}"));
    let keyup = index_of(&log, "keyup", None, Some("w")).expect("keyup missing");
    assert!(
        keydown < beforeinput && beforeinput < input && input < keyup,
        "base insertion sequence must be intact with gate off (log: {log:?})"
    );
    assert!(
        index_of(&log, "keypress", None, None).is_none(),
        "ZW_KEYPRESS_SYNTH=0 must revert keypress synthesis (log: {log:?})"
    );
}
