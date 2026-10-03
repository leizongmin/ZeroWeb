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
//!
//! 收尾钉（PR #64 双首轮审查合并，2026-10-03；仅测试面）：
//! 4. `tab_key_navigation_moves_active_element`（T-S2）——Tab 键驱 focus_via_tab 委托路径，
//!    activeElement 须随键盘焦点迁移（收敛声明的常驻面）。
//! 5. `blur_handler_samples_body_during_null_window`（T-S4/缺陷 S1）——blur handler 执行期
//!    `document.activeElement` 采样：宿主路径先清 `_activeElKey` 再派 blur（null 窗口 → body
//!    回落），与 HTML focusing steps「先更状态再派事件」的规范序存在先存差异（缺陷轮 I5 同族），
//!    本钉固化现语义供后续对齐。
//! 6. `host_migration_after_parsed_node_focus_emits_no_stale_blur`（T-S5/缺陷 S2）——R114/R148
//!    解析节点获焦（宿主不被通知）后宿主迁移：旧节点不得收到 stale focusout/blur（新守卫消除
//!    旧路径的重复派发；基数恰 1 固化）。
//! 常驻基数面（T-S1/测试有效轮 S-TE1）：钉 1 事件 log 恰等 `["focus@A","focusin@A"]`（该序
//! 规范可冻结）；钉 2 升级为多重集基数断言（4 事件各恰 1 次——捕捉幂等守卫回归的重复派发），
//! 不冻结 I5 已知规范序偏差的顺序。
//! Harness 面（T-S3/缺陷 S3）：`spawn_browser_with_env` spawn 前断言 renderer bin 存在且
//! mtime 不早于 apps/renderer/src 最新源码——陈旧 peer-bin 假绿面消除。

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
                    // blur 相位采样钉页（T-S4）：#A blur handler 内采样 document.activeElement——
                    // 固化宿主路径「先清 _activeElKey 再派 blur」的 null 窗口语义（body 回落）。
                    "/focusblursample" => format!(
                        r#"<!DOCTYPE html><html><head><title>S22 BlurSample</title><style>
#A{{position:fixed;left:40px;top:40px;width:200px;height:28px}}
#B{{position:fixed;left:40px;top:120px;width:200px;height:28px}}
</style></head><body>
<input id="A"><input id="B">
<script>
document.getElementById('A').addEventListener('blur', function () {{
  var ae = document.activeElement;
  window.__blurAe = ae ? ae.tagName : 'NULL';
}});
{FOCUS_LISTENER_SNIPPET}
</script>
</body></html>"#
                    ),
                    // 解析节点迁移钉页（T-S5）：boot 先 proxy 聚焦 #A（宿主被通知），再聚焦
                    // createElement 解析节点（R114/R148：_activeElKey 清空 + _zwMElFocused 接管
                    // + 旧 proxy 原位派发 focusout/blur——raw dispatchEvent，仅元素自身监听可见；
                    // 宿主不被通知）——宿主随后迁移时旧节点不得收到 stale focusout/blur。
                    // __aDirect = A 自身监听（捕获 R114 转移面 + 任何 stale 补派），__flog =
                    // document 捕获（捕获宿主路径 document 可见面）。
                    "/parsedfocus" => format!(
                        r#"<!DOCTYPE html><html><head><title>S22 ParsedFocus</title><style>
#A{{position:fixed;left:40px;top:40px;width:200px;height:28px}}
#B{{position:fixed;left:40px;top:120px;width:200px;height:28px}}
</style></head><body>
<input id="A"><input id="B">
<script>
window.__aDirect = [];
['focusout', 'blur', 'focus', 'focusin'].forEach(function (t) {{
  document.getElementById('A').addEventListener(t, function (e) {{
    window.__aDirect.push(t + '@A');
  }});
}});
document.getElementById('A').focus();
var __dyn = document.createElement('input');
__dyn.focus();
window.__dynFocused = (document.activeElement === __dyn);
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
    assert_renderer_bin_current(&renderer);

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

/// T-S3（缺陷轮 S3 收尾钉）：renderer bin spawn 前存在性 + 陈旧断言。`cargo test -p
/// zero-browser` 不重编 renderer——陈旧 peer-bin 会产出稳定假结果。缺失即 panic（不静默
/// 回退）；bin mtime 早于 `apps/renderer/src` 最新源码即 panic（陈旧面，构建顺序保证新鲜
/// bin 恒新于源码）。
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

/// 事件 log 中 `needle`（`type@target` 形态）出现次数——T-S1 基数断言面。
fn count_event(log: &[String], needle: &str) -> usize {
    log.iter().filter(|e| e.as_str() == needle).count()
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

    /// CDP Tab 键：keyDown + keyUp——键盘焦点导航默认动作（focus_via_tab 委托路径）。
    fn press_tab(&mut self) {
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({ "type": "keyDown", "key": "Tab", "code": "Tab", "windowsVirtualKeyCode": 9 }),
            Duration::from_secs(10),
        );
        self.call(
            "Input.dispatchKeyEvent",
            serde_json::json!({ "type": "keyUp", "key": "Tab", "code": "Tab", "windowsVirtualKeyCode": 9 }),
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
    // 事件轴（本就工作的面——负控制）：focus 事件落 #A。T-S1 升级为恰等断言（基数+序）：
    // 该两事件序本身规范（无 I5 偏差面），冻结之捕捉幂等守卫回归的重复派发与事件缺失。
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        log,
        vec!["focus@A", "focusin@A"],
        "focus event axis must be exactly [focus@A, focusin@A] (got {log:?})"
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
    // T-S1 多重集基数断言（每事件恰 1 次——捕捉双层幂等守卫回归的重复派发与事件缺失）；
    // 刻意不断言序：focusout→blur→focus→focusin 为先存规范序偏差（缺陷轮 I5），待对齐后再冻结。
    for expected in ["focusout@kw", "blur@kw", "focus@chat-textarea", "focusin@chat-textarea"] {
        assert_eq!(
            count_event(&log, expected),
            1,
            "event {expected} must fire exactly once (idempotence cardinality; log: {log:?})"
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

/// T-S2（测试有效轮 S-TE2 收尾钉）：Tab 键驱动 `focus_via_tab` 委托路径（收敛到
/// `focus_target` → `dispatch_host_focus`）。键盘焦点导航：无 boot focus 页首 Tab 落首个
/// 可聚焦元素 #A，次 Tab 迁 #B——activeElement 须随行（base 状态轴不迁移，翻红）。
#[test]
fn tab_key_navigation_moves_active_element() {
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

    cdp.press_tab();
    std::thread::sleep(Duration::from_millis(1200));
    let first = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let first_str = first.as_str().expect("tab first json string").to_string();
    let first_parsed: Value = serde_json::from_str(&first_str).expect("parse tab first");
    assert_eq!(
        first_parsed.get("active").and_then(Value::as_str),
        Some("A"),
        "first Tab must move activeElement to the first focusable #A (got {first_str})"
    );

    cdp.press_tab();
    std::thread::sleep(Duration::from_millis(1200));
    let second = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let second_str = second.as_str().expect("tab second json string").to_string();
    let second_parsed: Value = serde_json::from_str(&second_str).expect("parse tab second");

    assert_eq!(
        second_parsed.get("active").and_then(Value::as_str),
        Some("B"),
        "second Tab must move activeElement to the next focusable #B (got {second_str})"
    );
    // 事件轴随行面（负控制）：A 的获焦/失焦事件对在 log。
    let log: Vec<String> = second_parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    for expected in ["focus@A", "focusin@A", "focusout@A", "blur@A", "focus@B"] {
        assert!(
            log.iter().any(|e| e == expected),
            "expected {expected} in tab navigation focus log (log: {log:?})"
        );
    }
}

/// T-S4（缺陷轮 S1 收尾钉）：blur handler 执行期 activeElement 采样——行为锚定断言。
///
/// 宿主路径 `__zw_host_blur` 先清 `_activeElKey` 再派 focusout/blur：handler 重入时 getter
/// 走 body 回落（null 窗口）。HTML focusing steps 规范序为先更状态（新元素）再派事件——真实
/// 浏览器在旧元素 blur handler 内 activeElement 已指向新元素。先存差异（同缺陷轮 I5 族），
/// 本钉固化现语义（body 窗口）供后续对齐。判别面在迁移完成断言：base 旧路径宿主驱动点击
/// 从不更新 shim 状态 → 终态 activeElement 空（body），翻红；BODY 采样断言双侧一致（base
/// 的宿主点击同样不经 shim 状态，采样同为 body——非判别面，仅语义锚点）。
#[test]
fn blur_handler_samples_body_during_null_window() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/focusblursample") }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));

    cdp.click(60.0, 50.0); // #A 获焦
    std::thread::sleep(Duration::from_millis(1200));
    cdp.click(60.0, 130.0); // #B：A 失焦，blur handler 内采样
    std::thread::sleep(Duration::from_millis(1200));

    let state = cdp.evaluate(
        "JSON.stringify({blurAe: window.__blurAe || null, active: (document.activeElement && document.activeElement.id) || '', log: window.__flog})",
        Duration::from_secs(10),
    );
    let state_str = state.as_str().expect("blur sample json string").to_string();
    let parsed: Value = serde_json::from_str(&state_str).expect("parse blur sample");

    // 行为锚点（现语义）：blur handler 执行期 activeElement = body 回落（null 窗口）。
    assert_eq!(
        parsed.get("blurAe").and_then(Value::as_str),
        Some("BODY"),
        "blur handler must sample the null-window body fallback (got {state_str})"
    );
    // 迁移完成面：handler 返回后 activeElement 落新元素 #B。
    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some("B"),
        "activeElement must land on #B after the migration (got {state_str})"
    );
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        log.iter().any(|e| e == "blur@A"),
        "blur event must land on #A (log: {log:?})"
    );
}

/// T-S5（缺陷轮 S2 收尾钉）：解析节点获焦后宿主迁移不得向旧节点派 stale focusout/blur。
///
/// 场景：boot 先 `A.focus()`（shim 聚焦 + `__zw_focus_changed` 通知宿主），再聚焦
/// `createElement` 解析节点（R114/R148：`_activeElKey` 清空 + `_zwMElFocused` 接管 + 旧
/// proxy 原位派发 focusout/blur；宿主不被通知，focus_owner 滞留 A）。CDP 点击 #B 触发
/// 宿主迁移：`__zw_host_blur(A)` 的 `_activeElKey !== elKey(A)` 守卫早退——旧节点不得收到
/// stale 补派。双通道固化：A 自身监听（`__aDirect`——R114 raw 转移面恰 1 轮 + 任何 stale
/// 补派可观测）+ document 捕获 log（宿主路径 document 可见面零 focusout/blur@A）。
/// base 旧路径无守卫 → `__aDirect` 两轮（2 次）+ document log 出现 stale 对（翻红）。
#[test]
fn host_migration_after_parsed_node_focus_emits_no_stale_blur() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/parsedfocus") }),
        Duration::from_secs(15),
    );
    std::thread::sleep(Duration::from_millis(1200));

    // 前提自证：解析节点获焦成立（activeElement getter 返回 _zwMElFocused）。
    let pre = cdp.evaluate(
        "JSON.stringify({dynFocused: window.__dynFocused === true})",
        Duration::from_secs(10),
    );
    let pre_str = pre.as_str().expect("dyn focus pre json string").to_string();
    let pre_parsed: Value = serde_json::from_str(&pre_str).expect("parse dyn focus pre");
    assert_eq!(
        pre_parsed.get("dynFocused").and_then(Value::as_bool),
        Some(true),
        "parsed node focus must take ownership (R148) before the host migration (got {pre_str})"
    );

    cdp.click(60.0, 130.0); // #B：宿主迁移（blur_focused(A) + focus_target(B)）
    std::thread::sleep(Duration::from_millis(1200));

    let state = cdp.evaluate(
        "JSON.stringify({active: (document.activeElement && document.activeElement.id) || '', log: window.__flog, aDirect: window.__aDirect})",
        Duration::from_secs(10),
    );
    let state_str = state.as_str().expect("stale-blur json string").to_string();
    let parsed: Value = serde_json::from_str(&state_str).expect("parse stale-blur");

    assert_eq!(
        parsed.get("active").and_then(Value::as_str),
        Some("B"),
        "host migration must land activeElement on #B (got {state_str})"
    );
    // 通道 1：A 自身监听——R114/R148 raw 转移面恰派一轮 focusout/blur（多轮 = stale 补派）。
    let a_direct: Vec<String> = parsed["aDirect"]
        .as_array()
        .expect("aDirect array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    for event in ["focusout@A", "blur@A"] {
        assert_eq!(
            count_event(&a_direct, event),
            1,
            "{event} on #A's own listener must fire exactly once (R114 transition face); extra = stale dispatch (aDirect: {a_direct:?})"
        );
    }
    // 通道 2：document 捕获 log——宿主迁移面只有 #B 的获焦事件对（零 stale focusout/blur@A）。
    let log: Vec<String> = parsed["log"]
        .as_array()
        .expect("focus log array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect();
    for stale in ["focusout@A", "blur@A"] {
        assert_eq!(
            count_event(&log, stale),
            0,
            "{stale} must NOT appear in the document log (host stale dispatch; log: {log:?})"
        );
    }
    for expected in ["focus@B", "focusin@B"] {
        assert_eq!(
            count_event(&log, expected),
            1,
            "{expected} must fire exactly once (host migration face; log: {log:?})"
        );
    }
}
