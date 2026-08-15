# ZeroWeb WebDriver (`zero-webdriver`)

> W3C WebDriver 服务 — WebDriver 协议 M0 切片（wdspec 覆盖的第一步）

## 概述

`ZeroWeb WebDriver` (`zero-webdriver`) 是 ZeroWeb 的 WebDriver 服务（W3C WebDriver 协议），提供浏览器自动化测试的 HTTP 接口。当前为 M0 切片：实现 HTML 交互子集，每个 session 持有独立 `zero-renderer` 子进程，页面操作经 automation IPC 在 live document 上执行。HTTP 服务保持零依赖、单线程、loopback-only，从 `--port` 参数指定的本机端口提供服务。

协议参考：https://w3c.github.io/webdriver/#protocol

## 主要功能

- **W3C 端点（M0 切片）** — New Session、Navigate To、Get Title、Find Element、Element Click / Send Value、Active Element、Execute Sync、Delete Session
- **Session 隔离** — 每个 session 持有独立 `zero-renderer` 子进程（`RendererHandle`），页面状态互不影响
- **自动化 IPC** — 页面操作经 `zero-protocol` 的 Automation 消息族（`AutomationRequest` / `AutomationResult`）在 live document 上执行
- **元素引用管理** — WebDriver 元素引用 ↔ renderer 元素句柄双向映射（上限 4096），支持多定位策略
- **最小 HTTP 服务** — 零依赖、单线程、请求-响应模型，loopback-only 监听；带 CORS 头（`Access-Control-Allow-Origin: *`）便于工具链对接
- **键盘序列解析** — `parse_webdriver_keys` 支持 WebDriver key 序列（含修饰键与特殊键）

## 使用示例

```bash
# 启动 WebDriver 服务（默认端口 9515）
cargo run --bin zero-webdriver -- --port 9515

# 配合 wdspec / Selenium 等工具链使用
curl -X POST http://127.0.0.1:9515/session \
  -d '{"capabilities": {}}'
```

```http
POST /session                          → New Session（返回 session id）
POST /session/{id}/url                 → Navigate To
GET  /session/{id}/title               → Get Title
POST /session/{id}/element             → Find Element
POST /session/{id}/element/{ref}/click → Element Click
POST /session/{id}/element/{ref}/value → Element Send Value
GET  /session/{id}/element/active      → Active Element
POST /session/{id}/execute/sync        → Execute Script
DELETE /session/{id}                   → Delete Session
```
