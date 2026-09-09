# ZeroWeb WebDriver (`zero-webdriver`)

> W3C WebDriver 服务 — 34 endpoint 的自动化验证基建（webdriver 与 webdriver-screenshot 两 goal 均已收口归档，2026-09-08 / 2026-09-09）

## 概述

`ZeroWeb WebDriver` (`zero-webdriver`) 是 ZeroWeb 的 WebDriver 服务（W3C WebDriver 协议），提供浏览器自动化测试的 HTTP 接口。从 M0 骨架的 9 endpoint 扩齐到 34 endpoint（webdriver goal DC-1~4 ✅，2026-09-08 收口归档，见 `docs/goal/archive/webdriver/`；screenshot 续作 webdriver-screenshot goal，2026-09-09 收口归档，见 `docs/goal/archive/webdriver-screenshot/`），覆盖 session 管理、导航、超时、元素查找与状态、脚本执行、页面源码、窗口族与视口截图。每个 session 持有独立 `zero-renderer` 子进程，页面操作经 automation IPC 在 live document 上执行。HTTP 服务保持零依赖、单线程、loopback-only，从 `--port` 参数指定的本机端口提供服务。CI 以 v8/quickjs 双 feature 矩阵接线；10 个 HTTP 全链路集成测试（真实 TCP + 真实 renderer 子进程）+ 单元测试全绿。

协议参考：https://w3c.github.io/webdriver/#protocol

## 主要功能

- **Session 与状态** — `GET /status`、New Session、Get Session、Delete Session、timeouts get/set
- **导航族** — Navigate To、Get URL、Back、Forward、Refresh、Get Title、Get Source
- **元素族** — Find Element / Find Elements（多定位策略）、Active Element、Element Click / Send Value / Clear、Get Element Text / Attribute / Property / CSS Value / Rect / Enabled / Selected
- **脚本执行** — Execute Script Sync / Async（经 renderer 在 live document 上执行）
- **窗口族** — Get Window Handle(s)、Get/Maximize/Fullscreen Window Rect（语义面，多窗口深化挂账）
- **截图** — `GET /session/{id}/screenshot` 返回视口 PNG base64：消费 renderer Legacy 模式 `ViewPainted` 图元快照 + 跨帧累积 ImageCache（经 `zero-paint-convert` 公共转换层），CPU `render_full_scene` 光栅化 + PNG 输出；无帧时返回 `unable to capture screen`
- **Session 隔离** — 每个 session 持有独立 `zero-renderer` 子进程（`RendererHandle`），页面状态互不影响
- **自动化 IPC** — 页面操作经 `zero-protocol` 的 Automation 消息族（`AutomationRequest` / `AutomationResult`）在 live document 上执行
- **元素引用管理** — WebDriver 元素引用 ↔ renderer 元素句柄双向映射（上限 4096），支持多定位策略
- **最小 HTTP 服务** — 零依赖、单线程、请求-响应模型，loopback-only 监听；带 CORS 头（`Access-Control-Allow-Origin: *`）便于工具链对接；W3C wire format 错误包络（no such element / stale element reference / javascript error 等错误码映射）
- **键盘序列解析** — `parse_webdriver_keys` 支持 WebDriver key 序列（含修饰键与特殊键）
- **余项挂账** — element screenshot（元素裁剪）、定位策略扩展、frame 深化、多窗口语义

## 使用示例

```bash
# 启动 WebDriver 服务（默认端口 9515）
cargo run --bin zero-webdriver -- --port 9515

# 配合 wdspec / Selenium 等工具链使用
curl -X POST http://127.0.0.1:9515/session \
  -d '{"capabilities": {}}'
```

```http
POST /session                                    → New Session（返回 session id）
GET  /status                                     → 状态与就绪信息
POST /session/{id}/url                           → Navigate To
GET  /session/{id}/title                         → Get Title
GET  /session/{id}/source                        → Get Page Source
POST /session/{id}/element                       → Find Element
POST /session/{id}/elements                      → Find Elements
POST /session/{id}/element/{ref}/click           → Element Click
POST /session/{id}/element/{ref}/value           → Element Send Value
POST /session/{id}/element/{ref}/clear           → Element Clear
GET  /session/{id}/element/active                → Active Element
GET  /session/{id}/element/{ref}/css/{name}      → Get CSS Value
POST /session/{id}/execute/sync                  → Execute Script
POST /session/{id}/execute/async                 → Execute Async Script
GET  /session/{id}/window/handles                → Get Window Handles
POST /session/{id}/window/maximize               → Maximize Window
GET  /session/{id}/screenshot                    → Take Screenshot（视口 PNG base64）
DELETE /session/{id}                             → Delete Session
```
