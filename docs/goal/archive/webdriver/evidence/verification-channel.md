# WebDriver 验证通道 — 兄弟 goal 使用指南

**日期**: 2026-09-08（M3 收尾交付物）
**读者**: keyboard-default-actions / editing-contenteditable / js-dom / rendering-compat
等 goal 的执行 agent——需要端到端验证交互行为时使用本通道。

## 是什么

`zero-webdriver` 是 W3C WebDriver 协议的 loopback-only 服务（默认 9515），每个 session
spawn 一个真实 `zero-renderer` 子进程，页面操作在 **live document** 上执行（非重新解析
快照）。适合验证：键盘输入/焦点链、表单控件默认动作、contenteditable 编辑、脚本注入
后的 DOM 状态。

## 26 个可用 endpoint

| 族 | Endpoint |
|---|---|
| 会话 | POST /session、DELETE /session/{id}、GET /session/{id}、GET /status |
| 导航 | POST /session/{id}/url、GET /session/{id}/url、back、forward、refresh、GET title |
| 超时 | GET/POST /session/{id}/timeouts（script/pageLoad/implicit） |
| 定位 | POST /element、POST /elements（复数）、GET /element/active |
| 交互 | click、value（send keys 含  Tab 等特殊键）、clear |
| 状态 | text、rect、enabled、selected、attribute/{name}、property/{name}、css/{name} |
| 执行 | POST /execute/sync、POST /execute/async、GET /source |
| 窗口 | GET /window/handle(s)、GET+POST /window/rect、maximize、fullscreen |

行为细节见 [endpoint-matrix.md](endpoint-matrix.md)（每 endpoint 的偏差注记）。

## 集成测试模式（推荐）

不要手写 HTTP 客户端——照 `apps/webdriver/tests/http_session.rs` 的模式写集成测试
（workspace 内直接依赖测试即可跑，真实 TCP + 真实 renderer 子进程）：

```rust
// 1. spawn driver + 本地测试页服务器
let (_driver, port) = spawn_driver();
let (_page_server, page_port) = spawn_test_page_server_with_button();

// 2. New Session + Navigate
let (status, body) = http_request(port, "POST", "/session", Some("{}"));
let session_id = /* parse value.sessionId */;
let url = format!("http://127.0.0.1:{page_port}/");
http_request(port, "POST", &format!("/session/{session_id}/url"),
    Some(&json!({"url": url}).to_string()));

// 3. Find + 操作 + 断言（如 keyboard 流验证 Tab 焦点链）
let (status, body) = http_request(port, "POST",
    &format!("/session/{session_id}/element/{name_ref}/value"),
    Some(&json!({"text": "Zoé\u{E004}"}).to_string()));  //  = Tab
// GET /element/active 断言焦点移动 → execute/sync 读 live value
```

## 兄弟 goal 对接点

| Goal | 可验证行为 | 关键 endpoint 组合 |
|---|---|---|
| keyboard-default-actions | Tab/Shift-Tab 焦点链、Esc 默认动作、方向键组内移动（select/radio） | element/value（特殊键）→ element/active → execute/sync |
| editing-contenteditable | contenteditable 输入/删除/格式化 | element/value → execute/sync（读 innerHTML/textContent） |
| js-dom | 脚本注入 → DOM mutation → live document | execute/sync → source |
| rendering-compat | 布局几何真值（rect 族） | element/rect、element/css |

## 已知边界（勿依赖）

- **截图**：GET /screenshot 未实现（待用户决策，见 master.md）——像素级对比请用
  `make product-smoke` / headless `browsingContext.captureScreenshot` 通道
- **多 tab / frame 深化**：单 session 单文档
- **innerWidth 固定 1280**：shim 全局不跟随 SetViewport（engine shim 已知差距）；
  几何验证用 element/rect（gBCR RectBridge 真值）而非 window.innerWidth
- **alert/print/actions**：排除项
