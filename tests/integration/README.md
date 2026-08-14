# ZeroWeb Integration Tests (`zero-integration-tests`)

> 跨 crate 集成测试 — 验证端到端管线正确性

## 概述

`ZeroWeb Integration Tests` (`zero-integration-tests`) 是 ZeroWeb 的跨 crate 集成测试套件。与各 crate 内部的单元测试不同，本套件组合多个 crate 的真实实现（DOM → CSS → 样式 → 布局 → 渲染 → 脚本 → 多进程 IPC），验证端到端管线的协作正确性，覆盖 webview 完整管线、浏览器 shell、多进程协议、存储、安全、Canvas、WASM 沙箱、真实网站兼容与产品级 smoke 等场景。

## 覆盖范围

- **渲染管线** — `render_pipeline` / `e2e_rendering` / `navigation_paint`：DOM→样式→布局→绘制全链路与导航绘制
- **WebView 完整管线** — `webview_full_pipeline` / `webview_product_smoke` / `b3_load_mechanism`：WebView + `AsyncPageLoad` 分阶段加载（含 `BlockingFetchHost` IPC 适配）
- **多进程协议** — `multi_process` / `protocol_navigation` / `headless_protocol`：browser + renderer + compositor IPC 消息往返与导航
- **CSS / DOM 语义** — `dom_css` / `css_style` / `dom_bridge_polyfill` / `html_compat` / `runtime_conformance` / `web_api_pipeline`：CSS 层叠、DOM 桥接 polyfill、Web API 一致性
- **安全与存储** — `net_security` / `security_pipeline`：网络层与安全管线；`storage`：localStorage / IndexedDB 持久化
- **浏览器 shell** — `browser_shell_integration`：标签页、导航、地址栏与 IPC 序列化协作
- **图形与沙箱** — `canvas_render` / `e2e_canvas_dom`：Canvas 2D；`wasm_sandbox`：WASM 运行时集成
- **产品级验证** — `product_level_smoke` / `real_website_compat` / `viewport_adaptive` / `font_fallback_render`：真实站点兼容、响应式视口与字体回退

## 运行方式

```bash
# 运行全部集成测试（默认 v8 feature）
cargo test -p zero-integration-tests

# QuickJS feature
cargo test -p zero-integration-tests --no-default-features --features quickjs

# 单个测试
cargo test -p zero-integration-tests -- b3_load_mechanism
```
