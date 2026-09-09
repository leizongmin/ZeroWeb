# 截图验证通道 — 兄弟 goal 视觉回归使用说明

**创建日期**: 2026-09-09（webdriver-screenshot M3 接线收尾）
**适用范围**: rendering-compat 视觉回归、keyboard-*/editing-* 行为验证、android-browser
截图通道、任何需要「页面像素级断言」的 goal
**实现锚点**: `apps/webdriver/src/session.rs`（`session_screenshot` / `record_paint_snapshot`）、
`crates/paint-convert`（公共转换层）、`apps/webdriver/tests/http_session.rs`
（`webdriver_screenshot_error_and_success_paths` 全链路样例）

---

## 通道形态（一句话）

spawn `zero-webdriver --port <p>` → `POST /session` → `POST /session/{id}/url` →
`GET /session/{id}/screenshot` → base64 字符串 → 解码即 PNG（RGBA8，会话视口尺寸）。

## 端到端样例（真实 TCP + 真实 renderer 子进程）

完整可运行样例见 `apps/webdriver/tests/http_session.rs::webdriver_screenshot_error_and_success_paths`
（含 PNG 魔数 / 解码尺寸 / 像素采样断言三段式）。最小 Rust 客户端流程：

```rust
// 1) 建 session
let (status, body) = http_request(port, "POST", "/session", Some("{}"));
let session_id = serde_json::from_str::<Value>(&body)?["value"]["sessionId"]
    .as_str().unwrap().to_string();

// 2) 导航（LoadComplete 等待内含 ViewPainted 帧记录——返回即可截图）
http_request(port, "POST", &format!("/session/{session_id}/url"),
    Some(&serde_json::json!({ "url": url }).to_string()));

// 3) 截图
let (status, body) = http_request(port, "GET", &format!("/session/{session_id}/screenshot"), None);
let png_b64 = serde_json::from_str::<Value>(&body)?["value"].as_str().unwrap();
let png_bytes = base64::engine::general_purpose::STANDARD.decode(png_b64)?;
```

## 语义契约（消费方必读）

| 项 | 行为 | 依据 |
|---|------|------|
| 截取范围 | 会话视口（默认 800×600，`POST /window/rect` 可调） | W3C Take Screenshot window 语义 |
| 触达时机 | 导航返回后即可截图（`await_load_complete` 等待路径已消费 ViewPainted） | session.rs `record_paint_snapshot` |
| 图片完整性 | 跨导航累积 ImageCache；renderer sent_keys 去重下二次导航图片不缺 | master.md 决策记录 |
| 无帧错误 | 初始 about:blank（无 ViewPainted 帧）→ HTTP 500 `unable to capture screen` | W3C unable to capture screen 等价 |
| 无效会话 | `GET /session/{bad}/screenshot` → HTTP 404 `no such session` | W3C no such session |
| 像素格式 | PNG（RGBA8）；DSF 恒 1.0（webdriver 会话默认） | framebuffer_to_png_base64 |
| 渲染路径 | `zero-paint-convert` 转换 → CPU `render_full_scene`（与 compositor CPU 路径/browser headless 同源） | 与 reftest CPU 口径一致 |

## 典型用法映射

- **rendering-compat 视觉回归**：对 fixture 导航 → 截图 → 与 chromium oracle PNG 做
  逐像素 diff（复用 `zero-wpt-runner product-smoke` 的 diff 口径或 ID 级 XOR 脚本）。
  优势 vs headless CDP `captureScreenshot`：走 WebDriver 标准 33-endpoint 通道，
  可与元素交互（click/focus/send_keys）后截「交互后」状态——reftest 难覆盖的
  动态行为（表单焦点环、dialog、keyboard 导航高亮）可像素断言。
- **keyboard-* / editing-* 行为验证**：`POST /element/{ref}/value` 或 execute_script
  驱动输入 → 截图 → 采样 caret/选区/焦点区域像素。
- **android-browser**：同为 HTTP 消费方，Android 侧 `zero-webdriver` 起端口后同协议
  截图，可作 M0+ 的像素回归通道。

## 已知边界

- **element screenshot（元素裁剪）未实现**——页面级已备齐，元素级属后续评估项
  （入口文档「不在范围内」清单）。
- 每次截图为全量重光栅化（无增量编码/脏区缓存）——「先正确后快」约定。
- 渲染走 CPU 路径：与 GPU compositor 主链路在模糊/阴影等像素级细节可能存在
  既有 CPU/GPU 一致性容差内的差异（参见 render-foundation GPU 一致性测试口径）。
