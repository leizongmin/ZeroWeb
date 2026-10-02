//! CDP 焦点治理钉（slice22 focus governance）。
//!
//! 既有主支缺陷（slice19/20/21 判别签名）：宿主驱动焦点迁移（CDP 点击 → renderer mousedown
//! 默认动作）只派发 focus/focusin 事件、从不更新页面可见焦点状态——`document.activeElement`
//! 读的 shim `_activeElKey` 停留在页面 JS 最后 `focus()` 的元素。真实症状：baidu 首页 boot
//! `kw.focus()` 聚焦隐藏 #kw 后，点击可见 `TEXTAREA#chat-textarea`——focus 事件正确落在
//! textarea（s20-zw-focus-timeline t1 log）而 `document.activeElement` 恒报 INPUT#kw
//!（s21-focus-axis-after.txt）——「焦点状态报告」与「事件落点」分离，页面 sugrec 触发器读
//! activeElement 走错分支：逐键 sugrec XHR 0 请求、无建议下拉（Chrome 侧 4 请求 + 下拉）。
//!
//! 钉面（真实进程：zero-browser + zero-renderer + 本地静态 HTTP，CDP ws 逐连接独占）：
//! 1. `click_assigns_active_element_and_focus_event`（焦点指派钉）——两可聚焦元素页点击 #A：
//!    `document.activeElement === #A` 且 focus/focusin 落 #A。base 上状态断言翻红
//!    （activeElement 回落 body——无页面 boot focus 时 `_activeElKey` 恒 null）；事件断言
//!    base 上绿（事件轴本就工作——隔离「状态」单一缺陷面）。
//! 2. `kw_fallback_hidden_input_visible_textarea`（#kw 回退回归钉，baidu 最小形态）——
//!    boot `kw.focus()` 聚焦 0×0 隐藏 #kw（baidu virtual-form 形态：非 type=hidden、非
//!    display:none，可 Tab 聚焦）+ 可见 textarea，点击 textarea：activeElement 须迁移到
//!    textarea 且 focusout@kw/blur@kw/focus@textarea/focusin@textarea 全在。base 上状态
//!    断言翻红（恒 kw）。
//! 3. `kill_switch_off_keeps_legacy_active_element`（kill-switch 常驻钉）——
//!    `ZW_HOST_FOCUS_STATE_SYNC=0` 子进程注入回落旧「只派事件不更状态」路径：钉 2 页面
//!    activeElement 恒 kw（旧行为面）。判别方向：修复代码上若 env=0 未回落旧路径，本钉翻红。
//!
//! 负控制面：钉 1/2 的事件序列断言在 base 即绿（证明缺陷仅在状态轴、事件轴无恙——修复不得
//! 改变事件流）；钉 3 在 base/修复双侧均绿（开关方向自证）。

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

/// 焦点事件监听样板：document 捕获监听 focus/blur/focusin/focusout，记录 `type@target.id`
///（无 id 记 tagName）到 `window.__flog`——与 s20-zw-focus-timeline 探针同构。
const FOCUS_LISTENER_SNIPPET: &str = r#"
window.__flog = [];
['focus', 'blur', 'focusin', 'focusout'].forEach(function (t) {
  document.addEventListener(t, function (e) {
    var tg = e.target && e.target.id ? e.target.id : (e.target && e.target.tagName ? e.target.tagName : 'unknown');
    window.__flog.push(t + '@' + tg);
  }, true);
});"#;

impl TestSite {
    /// 启动静态服务。路径集固定：/focustwo（两可聚焦元素）、/kwfallback（baidu 形态最小复现）。
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
                    // 焦点指派钉页：两个固定坐标可聚焦文本框，无 boot focus（_activeElKey 初始 null）。
                    "/focustwo" => format!(
                        r#"<!DOCTYPE html><html><head><title>S22 FocusTwo</title><style>
#A{{position:fixed;left:40px;top:40px;width:200px;height:28px}}
#B{{position:fixed;left:40px;top:120px;width:200px;height:28px}}
</style></head><body>
<input id="A"><input id="B">
<script>{FOCUS_LISTENER_SNIPPET}</script>
</body></html>"#
                    ),
                    // #kw 回退回归钉页（baidu 最小形态）：0×0 包裹层隐藏 #kw（可 Tab 聚焦——
                    // 非 type=hidden/display:none，同 baidu virtual-form）+ 可见 textarea，
                    // boot JS 聚焦 #kw（页面最后 focus() 的元素）。
                    "/kwfallback" => format!(
                        r#"<!DOCTYPE html><html><head><title>S22 KwFallback</title><style>
#kw-wrap{{position:absolute;left:0;top:0;width:0;height:0;overflow:hidden}}
#chat-textarea{{position:fixed;left:50px;top:50px;width:300px;height:28px}}
</style></head><body>
<form><span id="kw-wrap"><input id="kw"></span></form>
<textarea id="chat-textarea"></textarea>
<script>
document.getElementById('kw').focus();
{FOCUS_LISTENER_SNIPPET}
</script>
</body></html>"#
                    ),
                    _ => "<!DOCTYPE html><html><head><title>S22 NotFound</title></head><body><h1>ok</h1></body></html>"
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

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪（默认环境）。
fn spawn_browser() -> (u16, BrowserProcess) {
    spawn_browser_with_env(&[])
}

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪；`vars` 注入子进程环境
///（kill-switch 等行为开关用子进程注入，不用 `std::env::set_var`——edition 2024
/// unsafe 且与并行测试竞态）。
fn spawn_browser_with_env(vars: &[(&str, &str)]) -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-s22-pin-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    // renderer 与 browser 同 target 目录（dev profile），取同级；ZERO_RENDERER_PATH 可覆盖。
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::env::var("ZERO_S22_RENDERER_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::Path::new(browser_bin)
                .parent()
                .expect("browser bin dir")
                .join("zero-renderer")
        });

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer);
    for (key, value) in vars {
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

/// 最小 CDP 客户端（同步 ws；每连接独占——与 s21 钉形态一致）。
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
}

/// 焦点指派钉：点击 #A 后 `document.activeElement` 须为 #A 且 focus/focusin 落 #A。
///
/// base 行为：事件轴正确（focus@A/focusin@A 在 log——base 即绿，隔离单一缺陷面），但
/// `_activeElKey` 恒 null → activeElement 回落 body（翻红）。
#[test]
fn click_assigns_active_element_and_focus_event() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/focustwo") }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));

    cdp.click(60.0, 50.0); // #A（left:40 top:40 200x28）
    std::thread::sleep(Duration::from_millis(1200));

    let state = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let state_str = state.as_str().expect("focus state json string").to_string();
    let parsed: Value = serde_json::from_str(&state_str).expect("parse focus state");

    // 状态轴（缺陷面）：activeElement 必须是被点元素本身。
    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some("A"),
        "document.activeElement must be the clicked element (got {state_str})"
    );
    // 事件轴（本就工作的面——负控制）：focus 事件落 #A。
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        log.iter().any(|e| e == "focus@A"),
        "focus event must land on #A (log: {log:?})"
    );
    assert!(
        log.iter().any(|e| e == "focusin@A"),
        "focusin event must land on #A (log: {log:?})"
    );
}

/// #kw 回退回归钉（baidu 最小形态）：boot focus 隐藏 #kw → 点击可见 textarea →
/// activeElement 须迁移到 textarea（base 恒 kw——slice19/20/21 判别签名），
/// 且失焦/获焦事件对全在（focusout@kw、blur@kw、focus@chat-textarea、focusin@chat-textarea）。
#[test]
fn kw_fallback_hidden_input_visible_textarea() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/kwfallback") }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));

    // boot focus 已落 #kw：先自证前提（页面最后 focus() 的元素是 #kw）。
    let pre = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || ''})",
        Duration::from_secs(10),
    );
    let pre_parsed: Value = serde_json::from_str(pre.as_str().expect("pre json")).expect("parse pre");
    assert_eq!(
        pre_parsed.get("active").and_then(Value::as_str),
        Some("kw"),
        "boot focus() must land on #kw before the click"
    );

    cdp.click(100.0, 60.0); // #chat-textarea（left:50 top:50 300x28）
    std::thread::sleep(Duration::from_millis(1200));

    let state = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let state_str = state.as_str().expect("focus state json string").to_string();
    let parsed: Value = serde_json::from_str(&state_str).expect("parse focus state");

    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some("chat-textarea"),
        "activeElement must follow the clicked textarea, not stay on hidden #kw (got {state_str})"
    );
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    for expected in ["focusout@kw", "blur@kw", "focus@chat-textarea", "focusin@chat-textarea"] {
        assert!(
            log.iter().any(|e| e == expected),
            "expected {expected} in focus log (log: {log:?})"
        );
    }
}

/// kill-switch 常驻钉：`ZW_HOST_FOCUS_STATE_SYNC=0` 子进程注入回落旧「只派事件不更状态」
/// 路径——钉 2 页面 activeElement 恒 #kw（旧行为面）。
///
/// 判别方向：修复代码上若 env=0 未回落旧路径（状态仍同步），本钉翻红。
#[test]
fn kill_switch_off_keeps_legacy_active_element() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser_with_env(&[("ZW_HOST_FOCUS_STATE_SYNC", "0")]);
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/kwfallback") }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));

    cdp.click(100.0, 60.0);
    std::thread::sleep(Duration::from_millis(1200));

    let state = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let state_str = state.as_str().expect("focus state json string").to_string();
    let parsed: Value = serde_json::from_str(&state_str).expect("parse focus state");

    // 旧行为面：状态不迁移（恒 boot focus 的 #kw）；事件轴照常（focusout@kw 等在）。
    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some("kw"),
        "kill-switch off must keep legacy behavior (activeElement stays #kw, got {state_str})"
    );
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        log.iter().any(|e| e == "focus@chat-textarea"),
        "kill-switch off must keep legacy event dispatch (log: {log:?})"
    );
}
