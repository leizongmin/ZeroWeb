# 产品入口、浏览器状态与平台宿主

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

本域覆盖 ZeroBrowser 桌面与 Android 宿主、嵌入式 WebView API、CDP/BiDi/WebDriver、浏览器 profile 和窗口事件。按用户入口定位，再沿运行时、脚本、网络/存储、图形知识深入；不在这里复制渲染内核、APK 构建或验收操作配方。

### User Task Navigation
- 桌面显示或交互：`apps/browser/src/main.rs` → `BrowserApp` → [桌面索引](desktop/index.md)。
- Android 标签、恢复或软键盘：`MainActivity.kt` → `NativeBridge` → `facade.rs/lib.rs` → [Android 索引](android/index.md)。
- 外部宿主加载/绘制 HTML：`WebViewBuilder/WebView` → [嵌入 API](embedded-webview.md)。
- 自动化连接、元素或调试面：`HeadlessServer/Driver/UiDriver` → [自动化索引](automation/index.md)。
- 用户数据重开丢失或状态错误：`BrowserShell/ProfilePaths` → [浏览器数据模型](browser-state/index.md)。
- 产品功能验证从 [zeroweb-verify 地图](../../../zeroweb-verify/features/index.md) 选择明确范围；本域只有源码/测试断言/历史核对，没有产品实跑，验证资产为本轮工作区新增，不归上述 HEAD。

## Gotchas
- desktop、Android、headless 与 WebDriver 都能操作页面，但状态入口、renderer 分配和持久化接线不同；同名方法或复用 BrowserShell 不证明四种产品入口具有相同行为。 (`apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`, `apps/browser/src/headless/session.rs`, `apps/webdriver/src/session.rs`)
- webview-demo 的当前入口集成 HostRuntime 与 render-foundation 展示文本，没有调用 WebView 加载页面；应用名不能作为嵌入 API 的真实消费者证据。 (`apps/webview-demo/src/main.rs`)

## Architecture
- 窗口事件、浏览器业务模型与页面执行分层：HostRuntime 交付 AppEvent/Window，Shell 保存用户状态，页面宿主驱动 renderer 或 WebView，呈现再交给图形底座。 (`crates/host-runtime/src/window.rs`, `crates/browser-shell/src/browser.rs`, `crates/webview/src/webview.rs`, `apps/browser/src/app.rs`)

## Child Knowledge Nodes
- [桌面浏览器交互](desktop/index.md) — 窗口/DPI、壳层、地址栏、页面输入与标签切换。
- [Android 浏览器宿主](android/index.md) — Rust 快照、系统集成、8 槽 Service 与页面预览。
- [嵌入式 WebView API 与宿主集成](embedded-webview.md) — 加载/完成边界、宿主桥、事件与绘制结果。
- [自动化与开发者协议](automation/index.md) — 协议兼容、会话/对象生命周期、真实 UI 与 DevTools。
- [跨宿主浏览器数据模型](browser-state/index.md) — profile、标签历史、收藏、下载与菜单模型。
- [窗口与事件宿主](host-runtime.md) — winit 生命周期、真实 API、两条产品调用与坐标契约。
