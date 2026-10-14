# 自动化与开发者协议

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 CDP/BiDi headless、WebDriver HTTP、真实窗口 UiDriver、Chrome parity 采集与 DevTools 静态资源。按连接和输出类型选入口，不把 headless、CPU screenshot 和真实窗口 GPU 证据互换。

### User Task Navigation
- Playwright/CDP 首连、导航后卡住或句柄错误：`headless/mod.rs/session.rs/domains` → [CDP 与 BiDi](cdp-and-bidi.md)。
- W3C 会话、元素 stale、回调超时：`webdriver main.rs/session.rs` → [WebDriver](webdriver.md)。
- GUI 脚本、GPU 截图、调试页面资源：`ui_driver.rs/parity_smoke.rs/devtools_serve.rs` → [UI 与 DevTools](ui-driver-and-devtools.md)。
- 本轮未实跑协议客户端或窗口；验证范围从 [自动化地图](../../../../zeroweb-verify/features/embedding-automation.md) 和 [原 parity 入口](../../../../zeroweb-browser-chrome-parity/SKILL.md) 选择，zeroweb-verify 为工作区新增，不归上述 HEAD。

## Gotchas
- CDP 多连接共享一个 HeadlessSession，WebDriver 每 session 一个 renderer，UiDriver 驱动真实 BrowserApp；跨客户端问题先确认状态/进程归属，而不是只比较 URL 字符串。 (`apps/browser/src/headless/mod.rs`, `apps/webdriver/src/session.rs`, `apps/browser/src/ui_driver.rs`)
- 协议 ack、健康端点、DOM 返回值和截图分别证明不同范围；Debugger/Profiler 等 ack 不能代替真实语义，CPU 页面截图也不能证明生产 GPU 一致。 (`apps/browser/src/headless/domains/mod.rs`, `apps/webdriver/src/session.rs`, `apps/browser/src/parity_smoke.rs`)

## Architecture
- 产品协议层负责外部方法/错误/会话外形，renderer 的 typed AutomationOperation 负责 live 页面动作；网络与加载、句柄代际和 frame publication 要沿运行时知识进一步核对。 (`apps/browser/src/headless/session.rs`, `apps/webdriver/src/session.rs`, `apps/renderer/src/automation.rs`)

## Child Knowledge Nodes
- [CDP、BiDi、会话与远程对象](cdp-and-bidi.md) — 发现/认证、Target/Page/Runtime/DOM/CSS/Input/Storage 与兼容缺口。
- [WebDriver 会话与 renderer 命令桥](webdriver.md) — HTTP 状态、等待、live 元素身份、脚本及 CPU 截图。
- [真实 UI 驱动、DevTools 与自动化兼容边界](ui-driver-and-devtools.md) — JSONL 驱动、生产 GPU 证据、frontend bundle 与测试客户端 pin。
