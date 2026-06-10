# ZeroWeb TUI 浏览器：复用核心引擎，替换宿主与终端渲染后端

日期：2026-06-11  
范围：ZeroWeb 本仓库源码深潜 + 终端浏览器/TUI 渲染生态外部参照  
结论等级：可行性评估，不是实现规格

## 任务规划

### 5W1H 分析摘要

| 维度 | 状态 | 当前理解 | 待澄清 |
|---|---:|---|---|
| What | 已明确 | 基于同一个 ZeroWeb 核心引擎，替换 GUI 宿主，增加 TUI 形态的 WebView 和 Browser App | 暂无 |
| Why | 已明确 | 希望 ZeroWeb 不绑定桌面 GUI，能在 SSH/服务器/纯终端环境运行 | 暂无 |
| Where | 已明确 | `zero-webview`、`zero-browser-shell`、`zero-engine`、`zero-render-foundation`、`apps/browser` | 暂无 |
| When | 已明确 | 关注当前仓库结构和当前 Rust TUI/终端图形生态 | 暂无 |
| Who | 已明确 | ZeroWeb 维护者、未来 TUI WebView 嵌入方、终端浏览器用户 | 暂无 |
| How | 已明确 | 判断复用边界、GUI 耦合点、终端渲染路径、里程碑成本和风险 | 暂无 |

### 时间焦点判断

| 用户信号词 | 时间焦点 | 内容重心 | 搜索策略 |
|---|---|---|---|
| "调研"、"可行性"、"替换 GUI" | 当前时间点 | 以 2026-06-11 仓库源码为准，外部资料只作方案参照 | P0 源码深潜，P1 官方/项目文档，P2 相关项目源码/主页 |

### 术语映射

| 用户原文 | 行业术语 | 搜索种子词 |
|---|---|---|
| "TUI 的浏览器" | Text-mode browser / terminal web browser / TUI web browser | `terminal web browser`, `text based browser`, `TUI web browser` |
| "替换 GUI 部分" | Host backend abstraction / terminal backend / rendering backend | `terminal backend`, `ratatui backend`, `crossterm raw mode` |
| "同一个引擎跑 GUI 和 TUI" | Multi-host rendering engine / backend-agnostic WebView | `headless browser terminal rendering`, `primitive renderer terminal` |
| "webview 和 app" | Embeddable WebView API + Browser shell/application | `webview embedding API`, `browser shell terminal UI` |

### 子任务分解

1. 梳理 ZeroWeb 当前引擎、WebView、Shell、HostRuntime、Browser App 的分层边界：目标是确认哪些能复用，哪些绑定 GUI。
2. 调研终端浏览器和 Rust TUI/终端图形生态：目标是识别可借鉴的渲染模型。
3. 形成方案对比：目标是在最小改动原则下推荐一条路径。
4. 输出落地路线图、风险清单和验证标准：目标是可直接进入后续 Spec/RFC。

## 来源分级总表

| 来源等级 | 本文使用方式 | 可信度 | 示例 |
|---|---|---:|---|
| 一手事实 | 本仓库源码、README、架构文档、测试代码 | 高 | `crates/webview/src/webview.rs`、`crates/browser-shell/src/lib.rs`、`apps/browser/src/app_platform.rs` |
| 外部搜索 | 官方文档/项目主页/项目 README | 中 | Ratatui、Crossterm、Browsh、Carbonyl、Chawan、Kitty graphics、SIXEL、Chafa |
| 前期调研/二手来源 | 本仓库已有架构文档和 README 对项目状态的总结 | 中 | `README.md`、`docs/architecture.md` |
| 假设 | 对未来 TUI API、终端能力探测、渲染降级策略的推断 | 待验证 | `zero-tui-webview`/`zero-tui-browser` 模块划分 |
| 推理 | 从源码事实推出的耦合判断和改造成本判断 | 待验证 | "TUI 不是重写引擎，而是新增宿主 + 新增渲染后端" |
| 作者综合 | 本文方案矩阵、里程碑、风险分级 | 仅供参考 | 第 5 章方案和第 6 章路线图 |

## 30 秒速览

ZeroWeb 当前已经具备把 GUI 替换为 TUI 的基本架构条件：页面内核输出的是 `RenderPrimitives`，`zero-webview` 对外暴露 headless-ish 的加载、布局、渲染和命中测试 API，`zero-browser-shell` 明确是 UI-agnostic 的浏览器数据模型；真正绑定桌面 GUI 的主要是 `zero-host-runtime` 和 `apps/browser`。

- 可行性判断：可行，但不是简单把 `apps/browser` 的按钮改成字符画；需要新增终端宿主和终端渲染后端。
- 推荐路线：新增 `zero-tui-webview` + `apps/tui-browser`，复用 `zero-webview` 和 `zero-browser-shell`，不要直接改造现有 `zero-browser`。
- 推荐渲染策略：先做 `RenderPrimitives -> TerminalCellBuffer` 的终端原生后端；后续再补 Kitty/SIXEL/iTerm2 图片协议或 RGBA 帧缓冲转终端图形。
- GUI 耦合点：`host-runtime` 基于 winit，`apps/browser` 持有 `GpuRenderer`/softbuffer/winit window；这些应保留给桌面 GUI。
- 引擎复用点：`RenderPipeline::render_html` 已串起 DOM/CSS/Style/Layout/Paint，并返回抽象图元；`WebView::load_html/render/fetch_url` 返回 `WebViewRenderResult`，不要求窗口 surface。
- 最大风险：文本可访问性不足。当前 `GlyphPrimitive` 主要是 glyph id/font id，不天然携带原始文本、链接范围、可聚焦节点树；TUI 浏览器需要补一个“语义/可交互投影层”。
- 最小成功闭环：能在终端打开一个 HTML/URL，显示文本、背景色、链接，支持滚动、地址栏、Tab、Enter 导航、链接点击/键盘选择。

一句话完整使用循环：`zero-tui-browser` 读取终端事件，驱动 `BrowserShell + WebView`，把 WebView 的 `RenderPrimitives` 投影为终端 cell buffer，再通过 Crossterm/Ratatui 或直接 ANSI 输出到终端。

## 执行摘要

### 核心结论

| 问题 | 判断 | 依据 |
|---|---|---|
| 同一个 ZeroWeb 引擎能否同时跑 GUI 和 TUI？ | 能，但需要明确把“宿主”和“终端渲染后端”作为新平台后端 | `zero-engine` 输出 `RenderPrimitives`，`zero-webview` 持有 `RenderPipeline` 并返回图元结果 [1][2] |
| 是否需要重写 DOM/CSS/Layout/网络/安全？ | 不需要作为第一阶段目标 | `WebView` 已组合 `RenderPipeline`、`HttpClient`、脚本沙箱、storage、security 等能力 [2] |
| 是否能复用 BrowserShell？ | 高度可复用 | `zero-browser-shell` 文档和源码声明其为 UI-agnostic 数据模型，负责标签、书签、历史、下载、设置 [3][4] |
| 是否能复用现有 HostRuntime？ | 不建议 | `zero-host-runtime` 明确基于 winit；TUI 需要 raw mode、alternate screen、终端尺寸和键鼠事件 [5][12][13] |
| 是否复用现有 GPU/CPU renderer？ | 可部分复用 CPU 帧缓冲，但推荐先直接做终端 cell 后端 | `render-foundation` 已有 CPU RGBA 帧缓冲，外部终端图形协议也可显示 raster，但终端浏览器首版更需要可读文本和链接交互 [6][17][18][19][20] |

### 推荐方案

推荐采用“三层新增、两层复用”的路线：

1. 复用 `zero-engine`、`zero-webview`、`zero-browser-shell`。
2. 新增 `crates/tui-renderer`：把 `RenderPrimitives` 投影为 `TerminalFrame`/`CellBuffer`。
3. 新增 `crates/tui-runtime` 或直接在 app 内使用 `crossterm`：处理 raw mode、alternate screen、键盘、鼠标、resize。
4. 新增 `apps/tui-browser`：TUI 浏览器应用入口。
5. 可选新增 `crates/tui-webview`：面向外部终端 app 的嵌入 API。

### 关键判断

基于源码，ZeroWeb 的 GUI 绑定不在核心引擎层，而在应用宿主和呈现层：`RenderPipeline` 只做 HTML→CSS→Layout→Paint，并输出图元；`WebView` 的核心 API 是加载、渲染、resize、命中测试、脚本执行；`BrowserShell` 管理浏览器状态。相反，`apps/browser` 负责 winit/window、GPU renderer、softbuffer、鼠标触摸、平台窗口控制和浏览器 chrome 绘制 [1][2][3][5][7]。

外部生态也支持这个方向。Ratatui 把终端后端抽象为 backend，应用可以用 backend 画 styled text 并处理终端事件 [12]；Crossterm 提供跨平台 raw mode、alternate screen 和事件能力 [13]；Browsh 和 Carbonyl 证明“现代浏览器输出到终端”成立，但它们走的是 Firefox/Chromium 完整浏览器适配路线 [14][15]；Chawan 证明终端原生小型浏览器引擎可以支持 CSS、inline images 和 JavaScript [16]。ZeroWeb 更适合中间路线：保留自研引擎，把 paint 输出降维到终端。

### 建议下一步

先做 RFC，不要直接进入大改造。RFC 应冻结三件事：

1. `zero-tui-webview` 的最小 API：加载、渲染、滚动、链接导航、focus、尺寸变化。
2. `tui-renderer` 的最小终端渲染模型：cell buffer、色彩降级、文本提取、链接 hit map。
3. `apps/tui-browser` 的最小产品形态：地址栏、标签栏、页面区、状态栏、键盘导航。

> **来源说明（任务规划与执行摘要）**
>
> - 一手事实 [1][2][3][4][5][6][7]：来自本仓库源码、README、架构文档和测试。
> - 外部搜索 [12][13][14][15][16][17][18][19][20]：来自 Ratatui、Crossterm、Browsh、Carbonyl、Chawan、Kitty/SIXEL/Chafa 官方或项目资料。
> - 假设：本文假设 TUI 首版目标是“可用的终端浏览器”，不是像 Carbonyl 那样完整复刻 Chromium 图形能力。
> - 推理：基于 `RenderPrimitives` 边界推出“新增终端后端优先于重写引擎”。

## 1. ZeroWeb 当前分层：TUI 可复用的边界已经存在

### 1.1 工作区层次已经把核心引擎与应用入口分开

仓库根 `Cargo.toml` 把 workspace 分成核心引擎层、基础设施层、脚本与沙箱、API 层、应用入口和测试基础设施；其中 `crates/webview`、`crates/browser-shell` 是 API/产品层，`apps/browser`、`apps/webview-demo`、`apps/renderer` 是应用入口 [1]。README 也把项目定位为同时交付可嵌入 `ZeroWebView` 和完整 `ZeroBrowser` 应用，并说明当前项目仍处于实验阶段 [8]。架构文档把请求到像素的链路描述为 `net -> dom -> css-parser -> style-system -> layout-engine -> engine -> render-foundation -> host-runtime -> webview` [9]。

这说明 TUI 方案不应该从“重写浏览器”开始，而应从“新增一个应用入口 + 新增终端宿主/渲染后端”开始。核心问题是：当前源码是否真的把窗口和 GPU surface 限制在宿主/应用层。下面几节逐层验证。

### 1.2 `zero-engine` 输出抽象图元，不直接依赖窗口

`zero-engine` 的 crate 文档定义其负责 HTML/DOM/CSSOM/样式/布局/绘制/脚本协调，核心模块包括 paint、dirty、composite、pipeline、preload、animation、transition [1]。`RenderPipeline` 的 `render_html` 和 `render_html_animated` 明确执行 HTML parse、CSS stylesheet 收集、样式计算、布局计算、Painter 生成绘制命令，最终返回 `RenderResult`，其中包含 `RenderPrimitives`、layout、timings 和 stats [1]。

关键事实：

- `RenderPipeline::new(viewport_width, viewport_height)` 只需要视口尺寸，不需要窗口、surface 或 GPU 设备 [1]。
- `render_html` 的输出是 `zero_render_foundation::primitive::RenderPrimitives` [1]。
- `RenderPipeline` 缓存 DOM 和 layout，用于 `hit_test_link(x, y)` 和 `document_height()` 这类 WebView 查询 [1][2]。

这对 TUI 很关键：终端渲染不必绕过 DOM/CSS/Layout/Paint，只要消费同一个 `RenderPrimitives`，再做终端投影即可。

### 1.3 `RenderPrimitives` 是适合新增后端的中间表示

`zero-render-foundation` 文档说明它提供 GPU 渲染器、CPU 软件渲染器、字体栈、图片缓存、脏区域追踪，并且采用 Scene/Primitive/Backend 分层 [6]。`primitive/mod.rs` 定义的 `RenderPrimitives` 包含 fills、rounded rects、path fills、path strokes、strokes、gradients、shadows、images、glyphs、filters、blend modes、transforms 等一帧渲染指令 [6]。

这些图元天然不是 GUI 独有：

- 文本来自 `GlyphPrimitive`，位置、字号、颜色、glyph id、font id 都在图元里 [6]。
- 背景、边框、圆角、线段、渐变、阴影、图片分别是独立图元 [6]。
- `RenderStats` 记录各类图元数量和剔除数量 [6]。

但 TUI 也会遇到一个结构性缺口：`GlyphPrimitive` 是 glyph 级数据，不是 DOM 文本节点或 Unicode 文本流。它适合图形绘制，却不够适合终端浏览器的选区、链接焦点、复制文本、屏幕阅读器式导航。`apps/browser` 现有页面选区是基于 glyph 图元做 hit test 和字符恢复，这能作为过渡方案，但不是理想语义层 [7]。

### 1.4 `zero-webview` 是最重要的复用边界

`zero-webview` README 声明它是面向外部应用的稳定 WebView 嵌入接口，提供 builder、load HTML/URL、渲染管线、CSS 注入、resize、状态查询、事件回调 [10]。源码层面，`WebView` 持有：

- `RenderPipeline`
- `HttpClient`
- `V8Sandbox`
- `ServiceWorkerRegistry`
- `HttpCache`
- `SecurityContext`
- `WorkerRuntime`
- `WasmInstance`
- 当前 URL/title/loading/last_render/cached_html/cached_css [2]

它的核心输出是 `WebViewRenderResult { primitives, timings }` [2]。`load_html` 调 `pipeline.render_html` 并缓存 `last_render`；`render` 复用 cached HTML/CSS；`resize` 重建 `RenderPipeline`；`fetch_url` 同步获取 HTML 后渲染；`load_url` 可以只更新状态，供外部异步加载驱动 [2]。

这对 TUI 有两个直接含义：

1. **TUI WebView 可以直接包一层 `zero_webview::WebView`。** 终端尺寸变化时调用 `resize(cols_to_px, rows_to_px)` 或直接以 cell 尺寸映射视口；加载后消费 `last_render().primitives`。
2. **TUI App 不必自己拼 DOM/CSS/Layout/网络。** 第一阶段只需要决定如何把终端输入、滚动和导航映射到 WebView API。

有一个小的代码味道需要注意：`crates/webview/Cargo.toml` 依赖 `zero-host-runtime`，但所读 `webview.rs`/`webview_builder.rs` 并未直接使用窗口或 winit API [2]。这意味着 TUI 不是被源码直接阻塞，但后续可以考虑清理该依赖，降低 WebView 对 GUI 宿主的语义耦合。

### 1.5 `zero-browser-shell` 已是 UI-agnostic，可直接服务 TUI app

`zero-browser-shell` README 说它负责多标签页、收藏夹、地址栏、历史记录等浏览器 UI 功能 [11]。更关键的是源码 `lib.rs` 明确写着：它提供 UI-agnostic 的浏览器 shell 数据模型和协调逻辑，可被任何 UI 框架消费，实际渲染由 render-foundation 完成 [3]。

`BrowserShell` 顶层管理：

- `TabManager`
- `Bookmarks`
- `History`
- `DownloadManager`
- `BrowserSettings`
- zoom
- find state
- autocomplete [4]

`Tab` 和 `TabManager` 是纯数据与导航状态：URL、title、loading、history、active tab index [4]。历史、书签、自动补全、设置也都是 UI 无关模型。集成测试已经覆盖 BrowserShell 与 WebView、protocol、storage 的协作，不需要窗口环境 [23]。

因此，TUI App 应该复用 `BrowserShell`，但不要复用 `apps/browser::BrowserApp` 的整体实现。`BrowserApp` 已经混入大量桌面窗口、鼠标、触摸、GPU/CPU present、chrome 几何和平台差异处理。

### 1.6 GUI 耦合主要集中在 `zero-host-runtime` 和 `apps/browser`

`zero-host-runtime` 文档和源码都明确基于 winit，负责窗口、事件循环、surface、输入法 [5]。它定义了 `AppEvent` 作为抽象事件，但事件转换函数仍来自 winit：keyboard、mouse、scroll、touch、IME、window resize/focus/redraw [5]。`HostRuntime::run` 和 `run_with_window` 都创建 winit event loop 和窗口 [5]。

`apps/browser` 更明显是桌面 GUI 入口：

- `Cargo.toml` 依赖 `winit`、`softbuffer`、`arboard`、`resvg`、`png`、`ico`，并依赖 `zero-host-runtime`、`zero-render-foundation` [7]。
- `main.rs` 创建 `HostRuntime`/`WindowConfig`，解析 `--renderer`、`--headless`、`--remote-debugging-port`，处理窗口缩放、主题、窗口动作 [7]。
- `BrowserApp` 持有 `BrowserShell`、每个 tab 的 `WebView`、`GpuRenderer`、`FontLoader`、`GlyphCache`、窗口尺寸、scale factor、鼠标位置、触摸滚动、窗口控制按钮等状态 [7]。
- `app_platform.rs` 初始化 wgpu window surface 或 CPU softbuffer surface，并把 RGBA 提交到 softbuffer [7]。
- `app_render.rs` 构建浏览器 chrome 的 fills/glyphs，并把 WebView 图元合成到桌面 frame [7]。
- `app_input.rs` 使用 `zero_host_runtime::event` 处理鼠标滚轮、触摸、键盘、IME、上下文菜单、地址栏等 [7]。

结论：桌面 GUI 的复杂度主要在 `apps/browser`，它是可参考的产品逻辑实现，但不应作为 TUI 的继承基类。TUI 应该用同样的底层 `BrowserShell + HashMap<TabId, WebView>` 模式，写自己的状态机和渲染器。

> ### 推理分析：为什么不是“替换 GUI crate”这么简单？
>
> **观察**：`BrowserApp` 同时负责浏览器产品状态、窗口平台差异、chrome 绘制、WebView 图元合成、输入事件和 present [7]。
>
> **推理**：如果直接在 `apps/browser` 里加 TUI 分支，会把 winit/softbuffer/wgpu 和 crossterm/ratatui 混在同一个应用状态中，后续维护成本高，也违反精准修改和简单至上的原则。
>
> **结论**：新增 `apps/tui-browser` 更稳妥；只在必要时抽出 `browser app core` 共享逻辑，例如 URL normalize、tab-WebView 映射、加载状态同步。

> **来源说明（第 1 章）**
>
> - 一手事实 [1]：`crates/engine/src/lib.rs`、`crates/engine/src/pipeline.rs`。
> - 一手事实 [2]：`crates/webview/src/webview.rs`、`crates/webview/src/webview_builder.rs`、`crates/webview/Cargo.toml`。
> - 一手事实 [3][4]：`crates/browser-shell/src/lib.rs`、`crates/browser-shell/src/browser.rs`、`crates/browser-shell/src/tab.rs`、`crates/browser-shell/src/history.rs`。
> - 一手事实 [5]：`crates/host-runtime/src/lib.rs`、`crates/host-runtime/src/event.rs`、`crates/host-runtime/src/window.rs`。
> - 一手事实 [6]：`crates/render-foundation/src/lib.rs`、`crates/render-foundation/src/primitive/mod.rs`、`crates/render-foundation/src/cpu/mod.rs`、`crates/render-foundation/src/surface.rs`。
> - 一手事实 [7]：`apps/browser/src/main.rs`、`apps/browser/src/app.rs`、`apps/browser/src/app_render.rs`、`apps/browser/src/app_input.rs`、`apps/browser/src/app_platform.rs`。
> - 一手事实 [8][9][10][11]：`README.md`、`docs/architecture.md`、`crates/webview/README.md`、`crates/browser-shell/README.md`。
> - 假设：TUI app 首版可以接受复用 `BrowserShell + WebView`，而不是要求与 `zero-browser` 共享同一个 `BrowserApp` 类型。
> - 推理：基于模块依赖和职责混合程度判断新增 app 优于在现有 GUI app 内硬塞 TUI 分支。

## 2. 外部参照：终端浏览器有三条主流路线

### 2.1 Rust TUI 宿主：Ratatui + Crossterm 是低风险组合

Ratatui 官方文档说明，Ratatui 通过 backend 与终端模拟器交互，backend 支撑 `Terminal` 绘制 styled text、控制 cursor、查询终端尺寸；应用通常也直接使用 backend 捕获键盘、鼠标、窗口事件，并启用 raw mode 和 alternate screen [12]。Ratatui 的 backend 对比文档也给出实际建议：大多数任务选择 Crossterm [12]。

Crossterm 官方 docs/crates 页面说明它是跨平台 terminal manipulation library，支持 raw mode、alternate screen、窗口尺寸、光标操作和事件读取 [13]。这与 TUI 浏览器的宿主需求匹配：它不需要 winit 窗口，但需要终端 raw mode、alternate screen、resize、键盘、鼠标、滚轮和粘贴/复制策略。

对 ZeroWeb 的直接启发：

- `zero-host-runtime` 不适合直接复用，但其 `AppEvent` 设计可以参考。
- TUI 首版可以不抽 `zero-tui-runtime` crate，先在 `apps/tui-browser` 内用 Crossterm 驱动事件循环；稳定后再下沉为 crate。
- 如果 UI 只需要 widgets/layout，Ratatui 很合适；如果需要精细控制终端 cell、鼠标坐标、图片协议和局部刷新，直接用 Crossterm 写 `TerminalSurface` 也更直接。

### 2.2 现代浏览器塞进终端：Browsh / Carbonyl 证明可行，但不是 ZeroWeb 的最优路线

Browsh 官方文档说它是纯文本浏览器，可运行在多数 TTY 终端环境和浏览器中；其 GitHub/文档说明它由 headless Firefox 支撑，用真实浏览器生成文本化网页/应用 [14]。Carbonyl README 则说明它是基于 Chromium、运行在终端里的浏览器，支持大量 Web API，包括 WebGL/WebGPU/audio/video/animations，并可在没有 window server 的环境甚至 SSH 中运行 [15]。

这类方案证明“完整现代浏览器体验可以映射到终端”，但它们的代价是把完整 Firefox/Chromium 带进来。ZeroWeb 的项目定位恰好相反：自研 DOM、CSS、layout、rendering、navigation、安全边界。因此，Browsh/Carbonyl 对 ZeroWeb 的价值不是“照搬实现”，而是验证两个产品方向：

1. 终端浏览器用户确实会接受“浏览器跑在远端/SSH/纯终端”的模式。
2. 终端输出可以分为文本模式和位图/高保真模式，二者可以共存。

### 2.3 终端原生浏览器：Chawan 更接近 ZeroWeb 的长期形态

Chawan 官方站点说明它是 text-mode web browser 和 pager，面向 Unix-like 系统，目标是在自包含、可理解、可扩展的前提下实现现代 Web 标准；它支持 CSS、终端 inline images，并通过小型独立浏览器引擎支持 JavaScript [16]。Chawan 的 GitHub mirror 也说明它使用自己从零开发的小型浏览器引擎，可以以接近图形浏览器的方式显示许多网站 [16]。

Chawan 与 ZeroWeb 的相似点：

- 都不是简单 HTML-to-text；都在做 HTML/CSS/JS/browser engine。
- 都需要把复杂布局降维到终端 cell。
- 都需要处理 CSS 支持范围、图片降级、链接导航、键盘模型和安全取舍。

差异是：Chawan 为 TUI 从零设计，而 ZeroWeb 当前先服务 GUI/嵌入式 WebView。因此 ZeroWeb 最合理路径是增加一个投影层，把图形中间表示转成终端中间表示，而不是改变 engine 的主表达。

### 2.4 传统文本浏览器：Lynx/w3m 说明首版体验边界

Lynx 官方站点把它定义为 text web browser [21]。w3m manual 把 w3m 定义为 pager/text-based WWW browser，可在 terminal emulator 中浏览本地文档或 Web 文档 [22]。这些传统浏览器提供了可借鉴的基础交互：键盘导航、链接编号、页面滚动、文本优先、低带宽、适合 SSH。

但传统文本浏览器通常不追求完整 CSS/JS/图形兼容，这与 ZeroWeb 的“同一个核心引擎”目标不同。ZeroWeb TUI 首版可以借用它们的交互习惯，但不应退化成只从 HTML 提取文本。否则就绕开了 ZeroWeb 已有引擎价值。

### 2.5 终端图形协议：适合第二阶段，不适合第一阶段作为唯一输出

Kitty graphics protocol 的目标是让终端里的客户端渲染任意 raster graphics 到终端屏幕，并支持按像素位置绘制 [17]。libsixel 提供 DEC SIXEL 的 encoder/decoder，SIXEL 数据是 terminal-friendly escape sequence [18]。Chafa 可以把图片数据输出为 Sixel、Kitty、iTerm2、Unicode mosaics 等多种终端图形格式，并支持 Truecolor、256-color、16-color 等模式 [19]。iTerm2 也有 inline images protocol，支持在终端中显示图片 [20]。

这些协议给 ZeroWeb 两种可能：

1. **像素路线**：复用 `render_full_scene` 先得到 RGBA `FrameBuffer`，再通过 Kitty/SIXEL/iTerm2/Unicode mosaic 输出到终端。
2. **混合路线**：正文和链接用 cell renderer，图片/canvas/video 等区域用终端图形协议嵌入。

基于以上分析，首版不建议把像素路线作为唯一输出。原因是：终端图形协议支持情况不统一，文本复制/链接焦点/屏幕阅读体验也不如原生 cell。更稳妥的是默认 cell renderer，能力探测通过后启用图片协议作为增强。

> ### 推理分析：三类外部路线对 ZeroWeb 的含义
>
> **观察**：Ratatui/Crossterm 解决终端宿主；Browsh/Carbonyl 解决完整浏览器到终端；Chawan 解决终端原生浏览器；Kitty/SIXEL/Chafa 解决 raster graphics 输出 [12][13][14][15][16][17][18][19][20]。
>
> **推理**：ZeroWeb 已经有自研 engine 和 primitive IR，因此不需要选择“完整外部浏览器进程”路线；它缺的是终端宿主和终端 renderer。
>
> **结论**：ZeroWeb 的 TUI 最优切入点是 Chawan 风格的终端原生浏览器体验 + Browsh/Carbonyl 风格的可选高保真图形增强。

> **来源说明（第 2 章）**
>
> - 外部搜索 [12][13]：Ratatui/Crossterm 官方文档与 crates/docs 页面。
> - 外部搜索 [14][15]：Browsh 官方文档和 Carbonyl GitHub README。
> - 外部搜索 [16]：Chawan 官方站点和 GitHub mirror。
> - 外部搜索 [17][18][19][20]：Kitty graphics protocol、libsixel、Chafa、iTerm2 inline images 文档。
> - 外部搜索 [21][22]：Lynx 官方站点、w3m manual。
> - 假设：终端图形协议作为可选增强，而不是 TUI 首版的强依赖。
> - 推理：外部路线仅作为设计参照，不代表 ZeroWeb 已实现这些能力。

## 3. 可行性拆解：需要替换的是宿主和呈现，不是核心引擎

### 3.1 复用/替换矩阵

| 层 | 当前模块 | TUI 处理方式 | 原因 |
|---|---|---|---|
| DOM/CSS/style/layout/paint | `zero-engine` 及其下游 crate | 复用 | 产出 `RenderPrimitives`，不需要窗口 [1] |
| WebView API | `zero-webview` | 复用，必要时包一层 TUI API | `load_html/render/resize/hit_test_link` 已可无窗口运行 [2] |
| 浏览器状态 | `zero-browser-shell` | 复用 | UI-agnostic 数据模型 [3][4] |
| 桌面窗口宿主 | `zero-host-runtime` | 替换 | 基于 winit，语义是窗口事件循环 [5] |
| 桌面浏览器 app | `apps/browser` | 参考，不直接复用 | 状态和桌面渲染/窗口逻辑混合 [7] |
| 图形 GPU/CPU renderer | `render-foundation::gpu/cpu` | 部分复用 | CPU 可输出 RGBA；终端首版更适合新 cell 后端 [6] |
| 测试基础 | `tests/integration` | 扩展 | 已有 WebView/Shell 无窗口测试，可增加 TUI renderer 测试 [23] |

### 3.2 TUI WebView 的最小架构

建议先新增 `crates/tui-renderer` 和 `apps/tui-browser`，`crates/tui-webview` 可以在 API 稳定后再拆。最小结构如下：

```text
apps/tui-browser
  ├─ TuiBrowserApp
  │   ├─ BrowserShell
  │   ├─ HashMap<TabId, WebView>
  │   ├─ address_bar / focus / scroll / status
  │   └─ TerminalRuntime(Crossterm)
  └─ render loop
      ├─ active WebView.last_render().primitives
      ├─ tui_renderer::project_primitives(...)
      └─ terminal surface present

crates/tui-renderer
  ├─ TerminalCell { ch, fg, bg, attrs, link_id }
  ├─ TerminalFrame { cols, rows, cells, link_map }
  ├─ PrimitiveProjector
  └─ ColorQuantizer / TextProjector / LinkHitMap
```

这张图是作者综合，原始源码中没有该结构。

核心设计点：

- `WebView` 仍按逻辑像素布局；`tui-renderer` 负责把像素坐标映射到 terminal cell 坐标。
- 每个 terminal cell 有前景色、背景色、字符、属性和可选 link id。
- 链接导航不要只依赖鼠标 hit test。TUI 需要额外的 `link_map`，支持 Tab/Shift-Tab、数字编号、Enter 打开。
- 滚动单位建议是 terminal row，而不是物理像素；内部再换算成 WebView coordinate。

### 3.3 为什么需要“语义投影层”

如果只把 `RenderPrimitives` 画成字符格，首版会很快遇到这些问题：

- 链接焦点：`hit_test_link(x, y)` 适合鼠标坐标，不适合键盘在所有链接间跳转 [2]。
- 文本复制：`GlyphPrimitive` 不是完整文本流，`apps/browser` 的 glyph 选区可恢复部分字符，但不是 DOM 语义 [6][7]。
- 表单控件：终端用户需要 focus ring、输入模式、Tab 导航，而不仅是画出 input 边框。
- 可访问性：标题、列表、表格、按钮、表单 label 等语义需要 DOM/layout 辅助，而不是只看像素。

因此，推荐把 TUI renderer 分成两个投影：

1. **视觉投影**：`RenderPrimitives -> CellBuffer`。
2. **交互投影**：DOM/layout/link/form/text ranges -> focus map / selectable ranges。

第一阶段可以用视觉投影 + `hit_test_link` 做鼠标/坐标导航；第二阶段必须补交互投影，否则 TUI 会停留在“能看，难用”。

### 3.4 三种渲染方案对比

| 方案 | 做法 | 优点 | 缺点 | 推荐度 |
|---|---|---|---|---:|
| A. 图元到 cell | 直接消费 `RenderPrimitives`，投影到终端 cell | 复用 engine；文本可读；不依赖特定终端图形协议 | 需要处理 glyph/text、颜色、布局降级 | 高 |
| B. RGBA 到终端图形 | 用 CPU renderer 得到 `FrameBuffer`，再输出 Kitty/SIXEL/iTerm2/Unicode mosaic | 实现快；视觉接近 GUI；可复用 CPU renderer | 文本交互差；协议兼容不一致；SSH/tmux 风险 | 中 |
| C. DOM 到文本排版 | 直接从 DOM/layout 生成文本浏览器页面 | 可控、语义强、性能好 | 绕过 paint；CSS 视觉复用少；与 GUI 差异大 | 中低 |

推荐 A 作为主线，B 作为增强，C 只用于 reader mode 或 fallback。

### 3.5 TUI App 的最小用户体验

首版不需要复刻桌面浏览器完整 chrome。合理的第一屏：

```text
[1] ZeroWeb TUI                    URL: https://example.com
────────────────────────────────────────────────────────────
 Example Domain

 This domain is for use in illustrative examples in documents.

 [1] More information...
────────────────────────────────────────────────────────────
 12 lines  |  link 1/1  |  Ctrl+L address  g open  q quit
```

关键交互：

- `Ctrl+L` 聚焦地址栏。
- `Enter` 打开地址或当前链接。
- `Tab`/`Shift+Tab` 切换链接/表单 focus。
- `j/k` 或 `Down/Up` 滚动。
- `[`/`]` 切换标签页。
- `t` 新标签页，`x` 关闭标签页。
- `r` reload，`b` back，`f` forward。
- 鼠标可选，不作为首版唯一交互。

这类交互更接近 Lynx/w3m/Chawan 的终端习惯，而不是桌面浏览器按钮驱动。

> **来源说明（第 3 章）**
>
> - 一手事实 [1][2][3][4][5][6][7]：ZeroWeb 当前源码分层、WebView API、BrowserShell 数据模型、HostRuntime/winit 绑定、RenderPrimitives。
> - 一手事实 [23]：WebView/Shell/product/headless 集成测试说明无窗口路径已被测试覆盖。
> - 外部搜索 [12][13][16][21][22]：Ratatui/Crossterm 终端宿主和 Chawan/Lynx/w3m 终端浏览器交互参照。
> - 假设：TUI 首版以 keyboard-first terminal UX 为目标，不强求桌面浏览器 chrome 等价。
> - 作者综合：复用/替换矩阵、推荐架构图、渲染方案对比表和首版 UI 草图。

## 4. 推荐落地路线：先做 TUI MVP，再沉淀公共抽象

### 4.1 里程碑 M0：终端渲染器原型

目标：证明 `RenderPrimitives -> TerminalFrame` 可行。

范围：

- 新增 `crates/tui-renderer`。
- 定义 `TerminalCell`、`TerminalFrame`、`TerminalStyle`。
- 支持 `FillPrimitive` 到 cell 背景色。
- 支持 `GlyphPrimitive` 到 cell 字符。
- 支持基础裁剪、scroll offset 和 viewport。
- 单元测试用固定 HTML 调 `WebView::load_html`，再投影为 80x24 terminal frame。

验收标准：

- `<h1>Hello</h1><p>World</p>` 能输出包含 `Hello`、`World` 的 cell buffer。
- 背景色和文本色能按至少 16-color/256-color/truecolor 三档降级。
- 宽度变化会触发 `WebView::resize` 和重新投影。

风险：当前 `GlyphPrimitive` 不含原始 char，只含 glyph id/font id；`apps/browser` 在 `append_webview_primitives` 中把 glyph id 转成 char 的方式可借鉴，但这不是完整字体 shaping 语义 [6][7]。如果 glyph id 不能可靠恢复字符，M0 需要临时从 DOM/layout 旁路补文本投影。

### 4.2 里程碑 M1：TUI WebView MVP

目标：形成可被外部终端 app 嵌入的最小 WebView。

建议 API：

```rust
pub struct TuiWebView {
    webview: zero_webview::WebView,
    scroll_rows: u32,
    focus: TuiFocus,
}

impl TuiWebView {
    pub fn load_html(&mut self, html: &str, css: Option<&str>) -> TuiFrame;
    pub fn fetch_url(&mut self, url: &str) -> Result<TuiFrame, TuiError>;
    pub fn resize_cells(&mut self, cols: u16, rows: u16);
    pub fn scroll_lines(&mut self, delta: i32);
    pub fn next_link(&mut self);
    pub fn activate_focused(&mut self) -> Option<String>;
    pub fn render(&mut self) -> TuiFrame;
}
```

这段 API 是作者综合，用于说明方向，不是对现有源码的描述。

验收标准：

- 能加载 HTML 字符串和 HTTP/HTTPS URL。
- 能滚动长页面。
- 能发现并打开链接。
- 能在 80x24、120x40 两种终端尺寸下正常换行/裁剪。

### 4.3 里程碑 M2：`apps/tui-browser` 产品闭环

目标：交付一个真正可运行的 TUI 浏览器应用，而不是只输出一页文本。

范围：

- `BrowserShell` 管理 tabs/history/bookmarks/autocomplete [3][4]。
- `HashMap<TabId, WebView>` 管理每个 tab 的页面状态，复用 `apps/browser` 的基本思路 [7]。
- Crossterm raw mode + alternate screen + resize event + keyboard event [13]。
- 页面区、地址栏、状态栏、tab bar。
- 基础命令：open URL、back/forward/reload/new tab/close tab/switch tab/scroll/find。

验收标准：

- `cargo run --bin zero-tui-browser -- https://example.com` 能打开网页。
- `Ctrl+L` 输入新 URL 后 Enter 导航。
- `Tab` 在链接间移动，Enter 打开当前链接。
- `q` 或 `Ctrl+C` 退出并恢复终端状态。
- 新增集成测试覆盖 `BrowserShell + WebView + tui-renderer`。

### 4.4 里程碑 M3：终端图形增强

目标：在可用文本浏览基础上增强图片和高保真视觉。

范围：

- 探测终端能力：Kitty graphics、SIXEL、iTerm2 inline images、truecolor、256-color。
- 图片区域优先用终端图形协议输出，失败时 fallback 到 alt text / Unicode mosaic。
- 可选：对复杂页面区域使用 CPU `FrameBuffer` 转 Chafa-like mosaic。

验收标准：

- 支持至少一种现代终端图形协议。
- 不支持图形协议时仍可浏览文本和链接。
- tmux/SSH/普通 xterm-like 终端不崩溃。

### 4.5 里程碑 M4：语义导航与表单

目标：从“可看”推进到“可用”。

范围：

- 从 DOM/layout 输出链接列表、表单控件列表、标题列表。
- 支持表单输入、checkbox/select/button。
- 支持复制当前段落/选中文本/页面标题和 URL。
- 支持 reader mode 或 text extraction mode。

这一步可能需要扩展 `zero-engine` 或 `zero-webview` 的查询 API，例如暴露可交互节点的 layout rect、文本内容、role、href、input state。该扩展应保持 GUI 也能受益，而不是只服务 TUI。

> **来源说明（第 4 章）**
>
> - 一手事实 [2][3][4][6][7][23]：WebView API、BrowserShell 模型、RenderPrimitives、桌面 BrowserApp 的 tab-WebView 管理方式、现有测试路径。
> - 外部搜索 [12][13][17][18][19][20]：终端 backend 和终端图形协议能力。
> - 假设：M0/M1 阶段可以接受有限 CSS/图片支持，先验证核心链路。
> - 作者综合：里程碑划分、API 草案、验收标准。

## 5. 主要风险与应对

### 5.1 文本语义风险：glyph 不是文本

风险：终端浏览器比 GUI 更依赖文本语义。`GlyphPrimitive` 存的是 glyph id/font id/position/color，不是 DOM text run；这会影响复制、链接焦点、搜索高亮、表单 label 和宽字符处理 [6]。

应对：

- M0 可以用 `GlyphPrimitive` 快速验证视觉输出。
- M1 开始补 `TextRun` 或 `SemanticRun` 查询 API。
- 最终应从 engine/layout 暴露“文本内容 + layout rect + DOM 节点关系”，不要从 glyph id 反推文本。

### 5.2 终端能力碎片化风险

风险：不同终端对 truecolor、鼠标、Kitty graphics、SIXEL、iTerm2 images、tmux passthrough 的支持不一致 [17][18][19][20]。

应对：

- 默认输出必须是 ANSI/cell 文本。
- 终端图形协议只作为增强。
- 增加 capability detection，并允许 `--graphics=off|auto|kitty|sixel|iterm2|unicode`。

### 5.3 UI 共享过度风险

风险：为了复用现有 `apps/browser`，把 TUI 分支塞进 `BrowserApp`，会扩大耦合，导致桌面 GUI 和 TUI 互相拖累 [7]。

应对：

- 新建 `apps/tui-browser`。
- 只抽取确实重复且稳定的纯逻辑，例如 URL normalization、tab-WebView lifecycle、title extraction。
- 不共享桌面 chrome 几何、winit event、softbuffer/GPU present。

### 5.4 事件模型差异风险

风险：winit 事件和 terminal event 不同。终端键盘可能只给字符/escape sequence，鼠标坐标是 cell，不是物理像素；IME、粘贴、快捷键、Alt/Meta 组合键在不同终端下表现不一致 [12][13]。

应对：

- TUI App 使用自己的 `TuiEvent`，不要强行复用 `zero_host_runtime::AppEvent`。
- 对常见键位提供可配置 keymap。
- 首版 keyboard-first，鼠标作为增强。

### 5.5 性能风险

风险：每次滚动都重新布局/渲染会浪费；每帧全量输出 ANSI 也会闪烁。

应对：

- WebView 内容变化时才重新 `render_html`；普通滚动只移动 viewport 投影。
- `TerminalFrame` 做 diff，只输出变化 cell。
- 大页面使用行级缓存和 dirty rows。

### 5.6 安全/隔离风险

风险：TUI 浏览器运行在终端中，可能将控制字符、网页文本、下载文件名等写入终端。恶意网页文本如果未转义，可能影响终端状态或复制内容。

应对：

- 所有网页文本输出必须走 cell abstraction，不直接写原始字符串到 terminal。
- 禁止网页内容生成未转义 ANSI escape。
- 下载/文件路径显示做控制字符转义。

> **来源说明（第 5 章）**
>
> - 一手事实 [2][5][6][7]：WebView/RenderPrimitives/HostRuntime/BrowserApp 当前边界。
> - 外部搜索 [12][13][17][18][19][20]：终端 backend 与终端图形协议差异。
> - 假设：TUI 浏览器会运行在 SSH/tmux/不同终端模拟器中。
> - 推理：风险来自源码中图形 IR 与终端语义需求的差距，以及终端生态本身的差异。

## 6. 推荐技术决策

### 6.1 决策 1：新增 `apps/tui-browser`，不要改造 `apps/browser`

理由：

- `apps/browser` 已包含大量桌面窗口和平台细节 [7]。
- `BrowserShell` 和 `WebView` 已经是更合适的复用单元 [2][3][4]。
- 新 app 能快速验证 TUI 路径，不影响现有 GUI。

### 6.2 决策 2：新增 `crates/tui-renderer`

理由：

- `render-foundation` 当前是 GPU/CPU 图形后端，不应塞入终端 cell 逻辑导致职责变宽 [6]。
- TUI renderer 的输入可以是 `RenderPrimitives`，输出是 `TerminalFrame`。
- 稳定后可以再考虑把公共 trait 下沉到 `render-foundation`。

### 6.3 决策 3：短期不抽统一 HostRuntime trait

理由：

- GUI runtime 和 TUI runtime 事件语义差异大 [5][12][13]。
- 过早抽 `HostRuntime` trait 容易产生抽象泄漏。
- 等 TUI app 跑通后，再根据重复代码抽 `AppRuntime` 或 `Surface` trait。

### 6.4 决策 4：默认 cell renderer，图片协议作为增强

理由：

- 终端浏览器首要价值是文本可读、键盘可达、远程可用。
- Kitty/SIXEL/iTerm2 支持不统一 [17][18][19][20]。
- `FrameBuffer -> terminal graphics` 可以作为 M3，不阻塞 M0/M1。

### 6.5 决策 5：尽早补语义查询 API

理由：

- TUI 的交互体验需要链接列表、表单控件、文本范围。
- 这些 API 也能反哺 GUI 的可访问性、查找、选择、自动化测试。
- 不应长期依赖 glyph 反推文本。

> **来源说明（第 6 章）**
>
> - 一手事实 [2][3][4][5][6][7]：WebView/Shell/HostRuntime/render-foundation/apps/browser 的职责边界。
> - 外部搜索 [12][13][17][18][19][20]：终端 runtime 和图形协议约束。
> - 作者综合：五个技术决策是本文基于源码和外部参照给出的建议，原始资料中不存在该决策表。

## 7. 结论

基于 2026-06-11 的源码状态，ZeroWeb 的核心引擎已经具备 GUI/TUI 双宿主的前提：核心 engine 输出抽象图元，WebView API 可以无窗口运行，BrowserShell 是 UI-agnostic 数据模型。TUI 路线的主要工作不是重写浏览器核心，而是新增终端宿主、终端渲染器和终端产品入口。

推荐执行顺序：

1. `crates/tui-renderer`：先证明 `RenderPrimitives -> TerminalFrame`。
2. `apps/tui-browser`：直接复用 `BrowserShell + WebView` 做产品闭环。
3. `crates/tui-webview`：当 app 的 API 稳定后再抽嵌入接口。
4. 语义查询 API：补链接/文本/表单/focus map。
5. 终端图形增强：Kitty/SIXEL/iTerm2/Unicode mosaic。

最终判断：**可行，并且是符合 ZeroWeb 架构方向的扩展**。但成功标准应定义为“同一个 engine 的多宿主输出”，而不是“把桌面 GUI app 换成字符 UI”。这个差别决定了实现边界：复用 `zero-webview` 和 `zero-browser-shell`，新增 TUI app/runtime/renderer，避免污染现有桌面路径。

> **来源说明（第 7 章）**
>
> - 一手事实 [1][2][3][4][5][6][7][23]：ZeroWeb 当前源码和测试。
> - 外部搜索 [12][13][14][15][16][17][18][19][20][21][22]：终端宿主、终端浏览器、终端图形生态。
> - 推理：结论来自第 1-6 章事实和方案对比。

## 参考资料

| 编号 | 来源 | 类型 | 引用章节 | 备注 |
|---:|---|---|---|---|
| [1] | `crates/engine/src/lib.rs`、`crates/engine/src/pipeline.rs` | 一手事实 | 1, 3, 7 | `RenderPipeline`、`RenderResult`、`RenderPrimitives` 输出 |
| [2] | `crates/webview/src/webview.rs`、`crates/webview/src/webview_builder.rs`、`crates/webview/Cargo.toml` | 一手事实 | 摘要, 1, 3, 4, 5, 6, 7 | `WebView` API、状态和依赖 |
| [3] | `crates/browser-shell/src/lib.rs` | 一手事实 | 摘要, 1, 3, 4, 6, 7 | UI-agnostic 声明 |
| [4] | `crates/browser-shell/src/browser.rs`、`crates/browser-shell/src/tab.rs`、`crates/browser-shell/src/history.rs` | 一手事实 | 摘要, 1, 3, 4, 6, 7 | BrowserShell/Tab/History 数据模型 |
| [5] | `crates/host-runtime/src/lib.rs`、`crates/host-runtime/src/event.rs`、`crates/host-runtime/src/window.rs` | 一手事实 | 摘要, 1, 3, 5, 6, 7 | winit 宿主和事件抽象 |
| [6] | `crates/render-foundation/src/lib.rs`、`crates/render-foundation/src/primitive/mod.rs`、`crates/render-foundation/src/cpu/mod.rs`、`crates/render-foundation/src/surface.rs` | 一手事实 | 摘要, 1, 3, 4, 5, 6, 7 | Primitive、CPU renderer、FrameBuffer |
| [7] | `apps/browser/src/main.rs`、`apps/browser/src/app.rs`、`apps/browser/src/app_render.rs`、`apps/browser/src/app_input.rs`、`apps/browser/src/app_platform.rs` | 一手事实 | 摘要, 1, 3, 4, 5, 6, 7 | 桌面 BrowserApp 的 GUI 耦合点 |
| [8] | `README.md` | 前期调研/一手事实 | 1 | 项目定位和当前状态 |
| [9] | `docs/architecture.md` | 前期调研/一手事实 | 1 | 架构分层和请求到像素链路 |
| [10] | `crates/webview/README.md` | 一手事实 | 1 | WebView 对外 API 说明 |
| [11] | `crates/browser-shell/README.md` | 一手事实 | 1 | BrowserShell 说明 |
| [12] | [Ratatui Backends](https://ratatui.rs/concepts/backends/)、[Comparison of Backends](https://ratatui.rs/concepts/backends/comparison/) | 外部搜索 | 摘要, 2, 3, 5, 6, 7 | backend、styled text、raw mode/alternate screen、Crossterm 建议 |
| [13] | [Crossterm terminal docs](https://docs.rs/crossterm/latest/crossterm/terminal/index.html)、[crossterm crate](https://lib.rs/crates/crossterm) | 外部搜索 | 摘要, 2, 3, 4, 5, 6, 7 | raw mode、alternate screen、terminal manipulation |
| [14] | [Browsh Introduction](https://www.brow.sh/docs/introduction/)、[Browsh GitHub](https://github.com/browsh-org/browsh) | 外部搜索 | 摘要, 2, 7 | headless Firefox 支撑的 text-based browser |
| [15] | [Carbonyl GitHub](https://github.com/fathyb/carbonyl) | 外部搜索 | 摘要, 2, 7 | Chromium in terminal |
| [16] | [Chawan official site](https://chawan.net/) | 外部搜索 | 摘要, 2, 3, 7 | text-mode browser、CSS、inline images、JavaScript |
| [17] | [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) | 外部搜索 | 摘要, 2, 4, 5, 6, 7 | terminal raster graphics protocol |
| [18] | [libsixel GitHub](https://github.com/saitoha/libsixel) | 外部搜索 | 摘要, 2, 4, 5, 6, 7 | DEC SIXEL encoder/decoder |
| [19] | [Chafa official site](https://hpjansson.org/chafa/)、[Chafa GitHub](https://github.com/hpjansson/chafa) | 外部搜索 | 摘要, 2, 4, 5, 6, 7 | Sixel/Kitty/iTerm2/Unicode mosaic 输出 |
| [20] | [iTerm2 inline images protocol](https://iterm2.com/documentation-images.html) | 外部搜索 | 摘要, 2, 4, 5, 6, 7 | inline image protocol |
| [21] | [Lynx official site](https://lynx.invisible-island.net/) | 外部搜索 | 2, 3, 7 | traditional text web browser |
| [22] | [w3m manual](https://w3m.sourceforge.net/MANUAL) | 外部搜索 | 2, 3, 7 | pager/text-based WWW browser |
| [23] | `tests/integration/src/webview_product_smoke.rs`、`tests/integration/src/product_level_smoke.rs`、`tests/integration/src/browser_shell_integration.rs`、`tests/integration/src/headless_protocol.rs` | 一手事实 | 1, 3, 4, 7 | 无窗口 WebView/Shell/Product 测试路径 |
