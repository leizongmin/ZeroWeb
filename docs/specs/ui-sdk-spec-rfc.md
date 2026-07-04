# Spec/RFC: Zero UI SDK 抽取与浏览器迁移方案

**版本**: v1.6.1  
**日期**: 2026-06-30  
**作者**: AI Assistant  
**状态**: 草稿  

---

## 0. 执行摘要

- **一句话目标**: 将当前 ZeroBrowser 自绘界面能力抽取为独立的、浏览器无关的 Rust 自绘 UI SDK，并将浏览器迁移为该 SDK 的首个完整宿主应用。
- **本期范围**: 设计独立 `ui/` 顶层目录、核心 UI 架构、WebView 自定义组件模型、主题系统、国际化资源系统、共享文本/字体基础层、响应式/自适应布局、YAML DSL 输入、完整 DSL 表达式语言、完整应用级 UI 能力骨架、浏览器 chrome 迁移路线和分阶段验收标准。
- **明确排除**: 本文不要求首批实现完整 Qt/GTK 级控件生态，不重写浏览器内核，不改变 DOM/CSS/layout/rendering 页面管线，不在首阶段实现完整移动端后端，不在首批实现所有 design-system 风格包。
- **核心约束**:
  - 通用 UI SDK 必须与浏览器业务无关。
  - `ui/` 目录中的核心 crate 不得依赖 `zero-browser-shell`、`zero-webview`、`zero-engine`、`zero-net`。
  - 浏览器专属 chrome 组件与通用组件分开维护。
  - 浏览器迁移完成后必须保持现有功能、输入、渲染和多平台窗口行为不退化。
  - 主题、国际化、字体/文本、输入法、焦点、无障碍和移动端关键概念必须在第一版架构中预留。
- **推荐方案**: 采用 Flutter/Compose 风格的 retained widget tree + 单向数据流 + 局部失效刷新架构；用 YAML 作为声明式 UI 描述格式，内嵌受控表达式语言并解析为 Rust `WidgetSpec`；WebView 作为高级自定义组件集成。
- **首个落地步骤**: 新建文档和架构边界后，优先建立 `ui/core`、text foundation、`ui/render`、`ui/runtime` 与完整 UI 能力 skeleton，再按组件映射表将浏览器 chrome 渐进迁移为 `browser-ui/chrome` 领域组件。

---

## 1. 背景与目标

### 1.1 背景

ZeroBrowser 当前浏览器界面主要是自绘实现：窗口和事件循环来自 `zero-host-runtime` / `winit`，但标签栏、地址栏、菜单、滚动条、页面容器等界面元素集中在 `apps/browser` 中手写布局、绘制和事件处理。

当前已有一些天然边界：

- `crates/browser-shell`: 浏览器 UI-agnostic 状态模型，包含标签、书签、历史、设置、下载等。
- `crates/webview`: 页面渲染和嵌入式 WebView API。
- `crates/host-runtime`: 窗口、事件循环、输入、IME、平台事件封装。
- `crates/render-foundation`: CPU/GPU 图元渲染基础设施。
- `apps/browser`: 当前把浏览器外壳状态编排、UI 几何、绘制、输入处理混合在一起。

用户希望将当前浏览器 GUI 组件抽离为可复用的独立 UI SDK，未来可以被外部 GUI 应用使用，支持桌面端和移动端，并提供主题、DSL、WebView 自定义组件等能力。

### 1.2 目标

- **业务目标**: 让 ZeroBrowser 的自绘 UI 能力沉淀为可复用资产，而不是只服务一个浏览器应用。
- **工程目标**: 降低 `apps/browser` 的 UI 复杂度，使浏览器应用从“绘制所有东西”变成“组装 SDK 组件和浏览器状态”。
- **产品目标**: 为未来编写非浏览器 GUI 程序提供通用 Rust UI SDK。
- **架构目标**: 明确区分通用 UI SDK、浏览器专属 UI、浏览器内核和应用入口。

### 1.3 范围边界

**在范围内**:

- 新增独立 `ui/` 顶层目录和 workspace crate 组织方案。
- 定义 UI SDK 的核心架构：Widget tree、Element tree、Render tree / Scene tree。
- 定义事件、状态、Action、局部失效、布局、绘制、命中测试、焦点和无障碍边界。
- 定义 WebViewWidget 作为自定义组件的渲染和事件协议。
- 定义滚动语义与滚动条组件的职责边界。
- 定义主题系统：系统探测、亮色、暗色、高对比、自定义配色。
- 定义国际化资源系统、共享文本/字体基础层、响应式/自适应布局。
- 定义 YAML DSL 作为声明式 UI 输入格式，并实现完整表达式语言。
- 定义首版完整应用级 UI 能力 skeleton：动画、手势、导航、overlay、集合虚拟化、命令、表单、资产、平台服务、状态恢复、测试和 devtools。
- 定义浏览器迁移到 SDK 的分阶段路线。

**不在范围内**:

- 首阶段不实现任意脚本运行时、插件 VM 或通用系统自动化能力。
- 首阶段不提供完整设计器、可视化编辑器或热重载工具。
- 首阶段不实现完整移动端后端，只建立抽象和最小预留。
- 不将网页 DOM/CSS 渲染纳入 UI SDK 的通用布局系统。
- 不把浏览器专属语义加入通用控件库。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|---|---|---|
| 业务需求 | 是 | 用户要求将浏览器 GUI 组件抽取为外部可复用 UI SDK |
| 用户需求 | 是 | 外部程序可复用、浏览器迁移后正常、支持桌面和移动 |
| 解决方案需求 | 是 | 独立 `ui/` 目录、组件 SDK、WebView 自定义组件、YAML DSL |
| 功能需求 | 是 | 见第 3 节 |
| 非功能需求 | 是 | 见第 4 节 |
| 接口需求 | 是 | 见第 5 节 |
| 过渡需求 | 是 | 浏览器从现有 `apps/browser` 自绘迁移到 SDK |

---

## 3. 功能需求

### FR-001: UI SDK 必须位于独立顶层目录

- **描述**: 系统必须新增独立 `ui/` 顶层目录承载通用 UI SDK crate，使其与现有浏览器内核 `crates/` 明确区分。
- **优先级**: 必须
- **来源**: 用户明确要求

**验收场景**:

```text
场景: 通用 UI crate 放在独立目录
  假设 仓库根目录存在 Cargo workspace
  当 新增 UI SDK crate
  那么 crate 路径必须位于 ui/ 下
  并且 不得放入现有 crates/ 目录
  验证: 检查 Cargo.toml members 和目录结构

场景: 通用 UI crate 不依赖浏览器业务
  假设 存在 ui/core、ui/render、ui/runtime、ui/widgets、ui/patterns、ui/i18n、ui/animation、ui/gestures、ui/navigation、ui/overlay、ui/collections、ui/commands、ui/forms、ui/assets、ui/platform、ui/restoration、ui/testing、ui/devtools、ui/design-system
  当 检查这些 crate 的 Cargo.toml
  那么 它们不得依赖 zero-browser-shell、zero-webview、zero-engine、zero-net
  验证: cargo metadata 或手工依赖审查
```

### FR-002: UI SDK 必须采用浏览器无关的分层架构

- **描述**: UI SDK 必须拆分为核心类型、渲染/场景、运行时适配、通用组件、可选适配器和示例。
- **优先级**: 必须
- **来源**: 架构分析

**推荐目录**:

```text
ui/
├── README.md
├── core/              # zero-ui-core
├── render/            # zero-ui-render
├── runtime/           # zero-ui-runtime
├── widgets/           # zero-ui-widgets
├── patterns/          # zero-ui-patterns
├── i18n/              # zero-ui-i18n
├── animation/         # zero-ui-animation
├── gestures/          # zero-ui-gestures
├── navigation/        # zero-ui-navigation
├── overlay/           # zero-ui-overlay
├── collections/       # zero-ui-collections
├── commands/          # zero-ui-commands
├── forms/             # zero-ui-forms
├── assets/            # zero-ui-assets
├── platform/          # zero-ui-platform
├── restoration/       # zero-ui-restoration
├── testing/           # zero-ui-testing
├── devtools/          # zero-ui-devtools
├── design-system/     # zero-ui-design-system
├── dsl/               # zero-ui-dsl
├── adapters/
│   ├── winit/         # zero-ui-adapter-winit
│   └── webview/       # zero-ui-adapter-webview
└── examples/
    ├── counter/
    ├── form/
    └── browser-shell-demo/

browser-ui/
└── chrome/            # zero-browser-chrome
```

**验收场景**:

```text
场景: 分层目录清晰
  假设 开始实现 UI SDK 骨架
  当 查看仓库根目录
  那么 ui/ 包含 core、render、runtime、widgets、patterns、i18n、animation、gestures、navigation、overlay、collections、commands、forms、assets、platform、restoration、testing、devtools、design-system、dsl、adapters、examples
  并且 browser-ui/chrome 独立于通用 ui/
  验证: 文件树检查

场景: 浏览器专属 UI 不污染通用 SDK
  假设 需要实现地址栏、标签栏、书签栏
  当 放置这些组件
  那么 它们必须放入 browser-ui/chrome 或浏览器集成层
  并且 不得进入 ui/widgets 的通用组件 API
  验证: 模块依赖审查

场景: 领域组件复用通用绘制管线
  假设 AddressBar 位于 browser-ui/chrome
  当 AddressBar 组合 TextInput、IconButton、Popover、ListView
  那么 最终仍生成 UI SDK 的 Widget/Element/Scene tree
  并且 不得绕过 ui/render 另写浏览器专属绘制管线
  验证: browser-ui/chrome 组件集成测试和 scene snapshot
```

### FR-003: UI SDK 必须采用 retained widget tree + 单向数据流

- **描述**: UI SDK 必须使用 retained UI 模型，事件作为输入触发 Action/Message，应用状态更新后驱动 UI 刷新。
- **优先级**: 必须
- **来源**: 用户询问组件事件驱动模型后的决策

**核心流程**:

```text
WindowEvent / Touch / Key / IME
  -> ui-runtime 转成 UiEvent
  -> hit-test / focus route / capture route
  -> Widget::event
  -> Message / Action
  -> AppState reducer / handler
  -> mark needs_layout / needs_paint / needs_semantics
  -> render
```

**验收场景**:

```text
场景: 按钮点击通过 Action 更新业务状态
  假设 IconButton 绑定 action browser.go_back
  当 用户点击按钮
  那么 按钮只发出 Action
  并且 浏览器状态由应用层更新
  并且 UI 根据新状态刷新 enabled、title、url 等绑定
  验证: 单元测试 Action dispatch + state update

场景: 控件内部只保存临时 UI 状态
  假设 TextInput 需要 hover、focus、cursor、selection
  当 用户输入文本
  那么 TextInput 可保存光标和选区等临时状态
  但 URL、书签、下载、标签列表等业务状态必须由应用状态或 browser-shell 管理
  验证: 代码审查和状态归属测试
```

### FR-004: UI SDK 必须定义三棵树模型

- **描述**: SDK 必须区分用户声明结构、组件实例状态和渲染输出结构。
- **优先级**: 必须
- **来源**: 主流框架对比结论

**三棵树**:

| 树 | 职责 | 生命周期 |
|---|---|---|
| Widget tree | 用户声明式结构，可由 Rust API 或 YAML DSL 生成 | 可频繁重建 |
| Element tree | 组件实例、状态、焦点、生命周期、绑定缓存 | retained |
| Render tree / Scene tree | 布局、绘制、命中测试、裁剪、合成、无障碍 | 按失效标记更新 |

**验收场景**:

```text
场景: WidgetSpec 改变不会丢失稳定组件状态
  假设 TextInput 具有稳定 WidgetId
  当 父组件因状态变化重建 WidgetSpec
  那么 TextInput 的光标、选区、焦点可在 Element tree 中保留
  验证: 组件重建状态保持测试

场景: Paint-only 变化不触发布局
  假设 主题色从 light 切换为 dark
  当 字体和间距不变
  那么 系统只标记 needs_paint，不标记 needs_layout
  验证: invalidation 单元测试
```

### FR-005: UI SDK 必须支持 WebViewWidget 自定义组件

- **描述**: `zero-webview` 必须作为 UI SDK 中的高级自定义组件被集成，而不是被普通控件系统重写。
- **优先级**: 必须
- **来源**: WebView 渲染和滚动归属讨论

**设计原则**:

- UI SDK 给 WebViewWidget 分配 viewport、clip rect、scale factor、theme、输入事件。
- `zero-webview` 自己处理 HTML/CSS/DOM/layout/paint，输出 primitives、texture 或 scene node。
- UI SDK 不理解 DOM，不参与网页内部布局。
- WebViewWidget 将 WebView 输出合成到 UI scene。

**验收场景**:

```text
场景: WebView 作为自定义组件绘制
  假设 UI 树中有 WebViewWidget
  当 layout 阶段给出其矩形区域
  那么 WebViewWidget 将 viewport 信息传给 zero-webview
  并且 zero-webview 输出 RenderPrimitives 或 SceneNode
  并且 UI SDK 将其合成到最终 scene
  验证: WebViewWidget paint 单元测试或集成测试

场景: WebView 不进入通用 layout 算法
  假设 WebView 内部网页包含复杂 DOM/CSS
  当 UI SDK 布局 WebViewWidget
  那么 UI SDK 只计算 WebViewWidget 外部矩形
  并且 不尝试将网页 DOM 节点映射为 UI widgets
  验证: 架构审查
```

### FR-006: UI SDK 必须定义滚动语义与滚动条职责边界

- **描述**: WebView 的页面滚动语义由 WebView 管理；滚动条外观和通用交互组件由 UI SDK 提供。
- **优先级**: 必须
- **来源**: 用户询问滚动条处理位置

**边界**:

| 职责 | Owner |
|---|---|
| 页面内容尺寸 | WebView |
| 页面 viewport 尺寸 | UI SDK 分配，WebView 消费 |
| 页面 scroll offset | WebView / WebViewWidget |
| wheel/touch/key scroll 语义 | WebViewWidget 转发给 WebView |
| 通用 ScrollBar 几何和视觉 | ui/widgets |
| 浏览器 overlay scrollbar 风格 | browser-ui/chrome theme 或 ui/widgets style |

**验收场景**:

```text
场景: WebView 页面滚动更新 metrics
  假设 WebView 内容高度大于 viewport
  当 用户滚轮滚动页面
  那么 WebViewWidget 更新 WebView scroll offset
  并且 暴露新的 ScrollMetrics
  并且 ScrollBar 根据 metrics 重新绘制
  验证: WebViewWidget scroll 集成测试

场景: 滚动条拖动转换为 ScrollCommand
  假设 用户拖动 vertical scrollbar thumb
  当 ScrollBar 计算出新的 scroll_y
  那么 它必须向 WebViewWidget 发出 ScrollCommand
  并且 不直接修改浏览器业务状态
  验证: ScrollBar hit-test + command 测试
```

### FR-007: UI SDK 必须原生支持主题系统

- **描述**: SDK 必须支持自动探测系统主题、亮色、暗色、高对比以及用户自定义配色。
- **优先级**: 必须
- **来源**: 用户明确要求

**主题分层**:

```text
SystemThemeProvider
  -> ThemePreference
  -> ThemeResolver
  -> Theme
  -> Widgets / BrowserChrome
```

**必须支持的模式**:

| 模式 | 行为 |
|---|---|
| System | 跟随系统亮/暗色变化，运行时收到变化事件后自动重算主题 |
| Light | 固定亮色 |
| Dark | 固定暗色 |
| Custom | 用户自定义 palette/token |
| HighContrast | 系统报告高对比度时进入高对比主题，除非用户明确关闭无障碍跟随 |

**验收场景**:

```text
场景: 系统主题变化只触发绘制失效
  假设 当前 ThemePreference 为 System
  当 系统从 light 切换到 dark
  那么 ThemeResolver 生成新的 Theme
  并且 Runtime 发出 ThemeChanged
  并且 若字体/间距不变，仅触发 needs_paint
  验证: theme invalidation 测试

场景: 自定义配色覆盖语义 token
  假设 用户加载 custom theme file
  当 palette 中定义 accent、background、text_primary
  那么 组件消费这些 semantic token
  并且 不直接硬编码浏览器颜色
  验证: theme resolve 测试和 contrast lint
```

### FR-008: UI SDK 必须支持 YAML DSL 与完整表达式语言

- **描述**: SDK 必须支持以 YAML 描述 UI 组件树，并在属性、条件渲染、列表渲染、样式绑定和 action payload 中使用受控表达式语言；表达式语言必须完整覆盖 UI 声明常见计算，但不得具备任意脚本能力。
- **优先级**: 必须
- **来源**: 用户提出可用 YAML 作为 DSL 基础，并追加要求实现完整 DSL 表达式语言

**设计原则**:

- YAML 解析为 `WidgetSpec`。
- `WidgetSpec` 再挂载为 Element tree 和 Render tree。
- 表达式解析为 AST，经类型检查后在受限 `EvalContext` 中求值。
- 事件绑定到宿主注册的 ActionId；表达式只能构造 action payload，不得直接执行宿主函数。
- Rust 负责状态、业务逻辑、复杂组件和安全边界。
- 表达式语言必须是确定性的、无副作用的、可缓存的。

**表达式能力**:

- 字面量: string、number、bool、null、array、object。
- 路径读取: `$state.browser.address`、`$props.value`、`$theme.color.text_primary`、`$env.platform`。
- 运算符: 算术、比较、布尔、空值合并、条件表达式。
- 集合操作: index、field access、map/filter/any/all/count 等纯函数。
- 字符串操作: concat、contains、starts_with、ends_with、format。
- 样式计算: token 引用、单位换算、clamp、min、max。
- 结构控制: `if`、`for_each`、`visible_when`、`enabled_when`。
- Action payload: 由表达式生成结构化 payload。

**禁止能力**:

- 禁止循环中的任意 while/loop 或递归。
- 禁止访问文件系统、网络、进程、时钟随机数和任意系统 API。
- 禁止调用未注册的宿主函数或 Rust 函数。
- 禁止修改状态；状态变化必须通过 Action 由宿主处理。

**示例**:

```yaml
version: 1
component: Window
props:
  title: ZeroBrowser
  theme: system

children:
  - component: Toolbar
    children:
      - component: IconButton
        id: back
        icon: chevron-left
        enabled: "$browser.can_go_back"
        on_click: "browser.go_back"

      - component: TextInput
        id: address_bar
        value: "$browser.address"
        placeholder: "Search or enter address"
        on_submit: "browser.navigate"

  - component: WebView
    id: active_webview
    visible_when: "$browser.active_tab != null"
    props:
      source: "$browser.active_tab.webview"
      scrollbars: overlay

  - component: TabStrip
    for_each: "$browser.tabs"
    item_as: tab
    children:
      - component: Tab
        props:
          title: "$tab.title ?? 'New Tab'"
          active: "$tab.id == $browser.active_tab_id"
          color: "$tab.loading ? $theme.color.accent : $theme.color.text_primary"
        on_click:
          action: "browser.activate_tab"
          payload:
            tab_id: "$tab.id"
```

**验收场景**:

```text
场景: YAML 解析为 WidgetSpec
  假设 存在合法 YAML UI 文件
  当 zero-ui-dsl 解析该文件
  那么 输出 WidgetSpec tree
  并且 包含 component、props、bindings、actions、children
  验证: DSL parser 单元测试

场景: DSL 不执行任意脚本
  假设 YAML 中出现 on_click: "browser.go_back"
  当 用户点击组件
  那么 Runtime 只查找已注册 ActionId
  并且 不执行 YAML 中的任意代码
  验证: action registry 测试和安全测试

场景: 表达式解析和求值
  假设 YAML 中存在 visible_when、enabled、for_each 和 action payload 表达式
  当 zero-ui-dsl 解析并类型检查该文件
  那么 输出包含 Expression AST 的 WidgetSpec tree
  并且 在 EvalContext 中能确定性求值
  验证: expression parser/typecheck/eval 单元测试

场景: 表达式禁止副作用
  假设 YAML 中出现文件读取、网络请求、未注册函数调用或状态写入表达式
  当 zero-ui-dsl 校验该文件
  那么 返回 DslError::ForbiddenCapability 或 DslError::UnknownFunction
  并且 不执行任何外部副作用
  验证: expression sandbox negative tests
```

### FR-009: UI SDK 必须提供浏览器迁移所需的首批组件

- **描述**: 首阶段必须优先抽取能支撑当前浏览器运行的组件，而不是先补齐完整通用组件库。
- **优先级**: 必须
- **来源**: 用户希望先抽象已有组件并确保浏览器正常

**首批通用基础组件（`ui/widgets`）**:

- `Button`
- `IconButton`
- `TextInput`
- `Toolbar`
- `Menu`
- `ContextMenu`
- `Popup`
- `Popover`
- `ListView`
- `Badge`
- `Tooltip`
- `ScrollBar`
- `ProgressIndicator`

**首批通用组合模式（`ui/patterns`）**:

- `SearchField`
- `SuggestionList`
- `CommandPalette`
- `DataList`
- `StatusBubble`
- `TabBar`
- `DialogScaffold`

**浏览器领域组件（`browser-ui/chrome`）**:

- `BrowserTabStrip`
- `AddressBar`
- `NavigationButtons`
- `SecurityBadge`
- `SiteInfoPanel`
- `BookmarksBar`
- `FindBar`
- `PermissionPrompt`
- `DownloadPanel`
- `DownloadItemView`
- `BrowserMenu`
- `PageLoadIndicator`
- `PageViewportFrame`

**专用适配组件（`ui/adapters/webview`）**:

- `WebViewWidget`

**分层原则**:

- `ui/widgets` 只包含与业务无关的基础控件和输入/绘制能力。
- `ui/patterns` 只包含跨应用可复用的组合模式，不引用浏览器状态或 WebView。
- `browser-ui/chrome` 可以理解 URL、安全状态、标签页、下载、权限等浏览器语义，但必须通过 `ui/widgets`、`ui/patterns`、`ui/adapters/webview` 输出 UI SDK scene。
- `apps/browser` 负责把 `zero-browser-shell` 和 `zero-webview` 状态映射为 `browser-ui/chrome` 的 props/actions。

**验收场景**:

```text
场景: 浏览器迁移后功能不退化
  假设 浏览器使用 browser-ui/chrome + ui/widgets
  当 启动 zero-browser
  那么 标签、地址栏、导航按钮、书签栏、菜单、滚动、WebView 渲染均保持可用
  验证: cargo run --bin zero-browser + product smoke

场景: 通用组件可被非浏览器示例复用
  假设 ui/examples/counter 使用 Button、TextInput、Column
  当 构建并运行示例
  那么 示例不依赖 zero-browser-shell 或 zero-webview
  验证: cargo run -p zero-ui-example-counter

场景: 浏览器领域组件由通用组件组合绘制
  假设 AddressBar 需要显示 URL、搜索建议和安全状态
  当 browser-ui/chrome 构建 AddressBar
  那么 它组合 TextInput、IconButton、Popover、SuggestionList、SecurityBadge
  并且 绘制输出进入统一 UI SDK scene
  验证: AddressBar component test + scene snapshot
```

### FR-010: UI SDK 必须为桌面和移动端建立宿主抽象

- **描述**: SDK 必须支持桌面端优先落地，并在架构上预留移动端能力。
- **优先级**: 应该
- **来源**: 用户预期支持桌面端和移动端

**宿主能力**:

- 桌面窗口、surface、resize、scale factor。
- 鼠标、键盘、触摸、滚轮、pan gesture。
- IME preedit/commit、软键盘矩形。
- 系统主题、accent color、高对比度。
- safe area、平台 back gesture、触摸 slop。

**验收场景**:

```text
场景: 桌面 winit adapter 提供统一事件
  假设 ui/adapters/winit 接入 winit
  当 收到 WindowEvent
  那么 它转换为 ui-core::UiEvent
  并且 上层 widgets 不直接依赖 winit 类型
  验证: adapter 转换测试

场景: 移动端概念已在核心 API 预留
  假设 首阶段未实现 Android/iOS 后端
  当 检查 ui-core runtime types
  那么 应包含 safe area、touch、soft keyboard、density、text scale 的数据模型
  验证: API 审查
```

### FR-011: UI SDK 必须支持无障碍、焦点和 IME 基础模型

- **描述**: SDK 第一版设计必须包含 SemanticsNode、焦点遍历、键盘导航和 IME rect 等基础能力。
- **优先级**: 必须
- **来源**: 主流框架对比和长期可维护性要求

**验收场景**:

```text
场景: 可聚焦组件参与焦点遍历
  假设 UI 中有 TextInput、Button、WebView
  当 用户按 Tab
  那么 焦点按声明顺序或显式 traversal policy 移动
  验证: focus traversal 单元测试

场景: TextInput 提供 IME 光标矩形
  假设 TextInput 获得焦点
  当 runtime 请求 IME rect
  那么 TextInput 返回当前光标位置对应屏幕矩形
  验证: IME rect 测试
```

### FR-012: UI SDK 必须支持局部失效刷新

- **描述**: SDK 必须区分 layout、paint、semantics、composite 等失效类型，避免任何变化都全量重算。
- **优先级**: 必须
- **来源**: 性能和主流框架经验

**验收场景**:

```text
场景: hover 变化只触发绘制
  假设 鼠标进入 Button
  当 Button hover 状态变化
  那么 Runtime 标记该节点 needs_paint
  并且 不重新布局整棵树
  验证: invalidation 测试

场景: 文本变化触发布局
  假设 TextInput 内容变长导致宽度测量变化
  当 value 更新
  那么 Runtime 标记相关节点 needs_layout 和 needs_paint
  验证: layout invalidation 测试
```

### FR-013: UI SDK 必须支持独立国际化资源文件与 message id 引用

- **描述**: UI SDK 必须提供移动端应用常见的国际化资源机制：用户可见字符串存放在独立 locale 资源文件中，Rust API 和 YAML DSL 通过稳定 message id 引用字符串，不直接在组件定义中硬编码可见文案。
- **优先级**: 必须
- **来源**: 用户要求参考移动端 app 做法，将国际化字符串存到独立文件中，并由 DSL 通过 id 引用

**资源组织**:

```text
ui/i18n/
└── schema/
    └── messages.schema.json

browser-ui/chrome/i18n/
├── en-US.yaml
├── zh-CN.yaml
└── pseudo.yaml

apps/browser/i18n/
├── en-US.yaml
├── zh-CN.yaml
└── zh-TW.yaml
```

**资源文件示例**:

```yaml
locale: zh-CN
direction: ltr
messages:
  browser.address.placeholder:
    value: "搜索或输入网址"
    description: "地址栏为空时显示的占位文案"
  browser.permission.camera.title:
    value: "允许 {origin} 使用摄像头？"
    params:
      origin: string
  browser.download.count:
    one: "{count} 个下载项"
    other: "{count} 个下载项"
    params:
      count: number
```

**DSL 引用示例**:

```yaml
component: TextInput
props:
  placeholder:
    i18n: browser.address.placeholder

component: Text
props:
  text:
    i18n: browser.permission.camera.title
    params:
      origin: "$browser.active_origin"
```

**设计原则**:

- `ui/i18n` 提供通用机制：locale、catalog、fallback、参数替换、plural、text direction、diagnostic。
- `ui/i18n` 不内置浏览器文案；浏览器文案属于 `browser-ui/chrome/i18n` 或 `apps/browser/i18n`。
- `ui/widgets` 和 `ui/patterns` 只消费 `LocalizedText`、`MessageRef` 或已解析字符串。
- DSL 中用户可见文案默认必须使用 `i18n` 引用；示例、测试或开发工具可允许 literal string，但 production strict mode 应报告 diagnostic。
- locale 切换属于状态输入，必须触发文本解析、布局和无障碍语义刷新。
- RTL locale 必须影响文本方向、基础布局方向、对齐和可镜像图标。

**验收场景**:

```text
场景: DSL 通过 message id 引用国际化字符串
  假设 zh-CN catalog 中存在 browser.address.placeholder
  当 DSL 中 TextInput.placeholder 使用 i18n: browser.address.placeholder
  那么 I18nProvider 返回 zh-CN 文案
  并且 TextInput 不包含硬编码可见字符串
  验证: i18n catalog resolve test + DSL loader test

场景: 缺失 message id 使用 fallback
  假设 当前 locale 为 zh-CN
  并且 zh-CN 缺失 browser.download.blocked
  但 en-US fallback 中存在该 key
  当 组件请求 browser.download.blocked
  那么 I18nProvider 返回 en-US fallback 文案
  并且 产生 missing-translation diagnostic
  验证: i18n fallback test

场景: 带参数和复数规则的字符串被解析
  假设 message browser.download.count 定义 count 参数和 plural forms
  当 参数 count 为 3
  那么 返回匹配 plural category 的本地化字符串
  验证: i18n plural/params test

场景: RTL locale 影响布局方向
  假设 当前 locale direction 为 rtl
  当 渲染 Toolbar、TextInput 和 TabBar
  那么 文本方向、默认对齐和可镜像图标遵循 RTL
  验证: rtl layout snapshot test
```

### FR-014: UI SDK 与 WebView 必须共享文本和字体基础层

- **描述**: 系统必须抽象出浏览器无关、UI SDK 无关的文本/字体基础能力，供 `ui/render` 和 `zero-webview` 共同使用；该层只提供字体发现、字体 fallback、shaping、bidi、line breaking、glyph cache、glyph atlas、文本测量和 glyph 绘制 primitive，不承接网页 DOM/CSS 布局规则。
- **优先级**: 必须
- **来源**: 用户询问字体渲染如何设计，以及 UI SDK 与 WebView 是否存在共同能力

**共享能力**:

- 系统字体发现和字体数据库。
- font family / weight / style / stretch 的通用匹配。
- 字体 fallback，包含 emoji、CJK、Arabic、Indic 等脚本。
- Unicode bidi、grapheme cluster、基础 line breaking。
- text shaping。
- 文本测量和光标边界。
- glyph rasterization、glyph cache、glyph atlas。
- subpixel positioning、device scale factor 适配。
- variable font 基础轴支持。
- locale 与 text direction 的输入参数。

**明确不共享**:

- CSS cascade、inheritance、selector、media query。
- CSS inline formatting context 和 line box construction。
- CSS `white-space`、`text-transform`、`text-decoration`、`letter-spacing` 等完整网页语义。
- DOM selection、range、caret painting。
- 网页内部 hit-test 和 accessibility tree。
- Web platform 字体加载策略的全部规范语义，如 `@font-face` lifecycle、font-display 细节；这些由 WebView/engine 解释后调用基础层。

**建议模块**:

```text
foundation/
└── text/              # zero-text-foundation
    font_database.rs
    font_fallback.rs
    font_request.rs
    shaping.rs
    bidi.rs
    line_break.rs
    grapheme.rs
    glyph_cache.rs
    glyph_atlas.rs
    text_measure.rs
    text_blob.rs
    diagnostics.rs
```

**验收场景**:

```text
场景: UI SDK 与 WebView 使用同一字体 fallback
  假设 UI 文本和网页文本都包含 CJK、emoji 和 Arabic 字符
  当 ui/render 和 zero-webview 请求字体 fallback
  那么 两者通过 zero-text-foundation 得到一致的 fallback chain
  验证: shared font fallback test

场景: UI SDK 与 WebView 共享 glyph cache
  假设 UI 地址栏和网页正文渲染同一字体同一字号的相同 glyph
  当 两者提交 glyph raster 请求
  那么 glyph cache 可以复用已有 atlas entry
  验证: glyph cache reuse test

场景: WebView 不把 CSS inline layout 委托给 UI SDK
  假设 网页包含复杂 inline formatting、white-space 和 DOM selection
  当 WebView 布局网页文本
  那么 CSS line box 和 selection 仍由 WebView/engine 处理
  并且 只在 shaping/measure/raster 阶段调用 zero-text-foundation
  验证: architecture boundary test
```

### FR-015: UI SDK 必须支持响应式/自适应布局与浏览器移动端 chrome

- **描述**: UI SDK 必须提供响应式和自适应布局能力，使同一套浏览器状态、Action、领域组件和通用组件可以在 desktop/tablet/phone 等不同 viewport 与 input modality 下选择不同 chrome shell；移动端不应只是 desktop 界面等比缩放，而应按断点、safe area、触摸输入和软键盘约束重排。
- **优先级**: 必须
- **来源**: 用户询问响应式布局、移动端界面接入方式，以及是否能与 desktop 共用一套 UI SDK

**核心概念**:

- `ViewportClass`: `Compact`、`Medium`、`Expanded`。
- `PlatformClass`: `Desktop`、`Tablet`、`Phone`、`Foldable`。
- `InputClass`: `MouseKeyboard`、`Touch`、`Hybrid`。
- `WindowMetrics`: viewport size、safe area、density、text scale、orientation、soft keyboard rect。
- `AdaptiveShell`: 根据 metrics 选择 desktop chrome、tablet chrome、phone chrome。

**浏览器复用边界**:

- 复用: `BrowserChromeModel`、`BrowserAction`、导航状态、标签状态、WebViewWidget、主题、i18n、文本基础层、通用 widgets/patterns。
- 分化: chrome shell 排版、toolbar 位置、tab 展示方式、菜单形态、权限提示形态、下载入口、地址栏焦点态。
- 禁止: 将 desktop chrome 原样缩小到 phone viewport。

**验收场景**:

```text
场景: 同一浏览器状态在 desktop 和 phone 下选择不同 shell
  假设 BrowserChromeModel 中存在 3 个 tab、active url 和 loading state
  当 WindowMetrics 从 Expanded/MouseKeyboard 切换到 Compact/Touch
  那么 browser-ui/chrome 选择 PhoneBrowserShell
  并且 复用相同 BrowserAction 和 BrowserChromeModel
  验证: adaptive shell selection test

场景: 移动端 safe area 和软键盘影响布局
  假设 PhoneBrowserShell 运行在带 bottom safe area 的设备上
  当 地址栏获得焦点且软键盘弹出
  那么 address/search overlay 不被 keyboard rect 遮挡
  并且 bottom toolbar 避开 safe area
  验证: mobile safe-area/keyboard layout test

场景: 响应式 DSL 使用断点选择布局
  假设 YAML DSL 中定义 Compact 和 Expanded 两套 layout branch
  当 viewport width 跨过断点
  那么 Runtime 重新选择对应 branch
  并且 保留稳定 WidgetId 的输入状态
  验证: DSL responsive branch test
```

### FR-016: UI SDK 首版架构必须覆盖完整应用级 UI 能力

- **描述**: 若首版目标是完整 UI SDK，而不是仅服务浏览器 chrome 迁移，则架构必须一开始纳入动画、手势、导航、overlay、集合虚拟化、命令/快捷键、表单、资产、平台服务、状态恢复、测试工具、devtools 和 design-system 扩展点；首版可只实现 skeleton 和浏览器需要的最小子集，但接口和模块边界必须先确定。
- **优先级**: 必须
- **来源**: 对比 Flutter、Compose、SwiftUI、Qt Quick、GTK 等主流 mobile/desktop UI 框架后的缺口分析

**必须补齐的能力域**:

| 能力域 | 模块 | 首版最小能力 | 浏览器接入 |
|---|---|---|---|
| 动画/过渡 | `ui/animation` | frame clock、tween、spring、implicit/explicit animation、reduced motion | tab loading、toolbar transition、sheet/dialog transition、progress |
| 手势 | `ui/gestures` | tap、long press、drag、pan、pinch、fling、gesture arena、pointer capture | mobile tab swipe、bottom sheet drag、pull to refresh、page gesture forwarding |
| 导航/页面栈 | `ui/navigation` | route stack、modal route、sheet route、dialog route、deep link、route restoration | settings/downloads/tab overview/site info/permission sheet |
| Overlay/Portal | `ui/overlay` | popover、tooltip、menu、dialog、sheet、toast、modal barrier、focus trap | omnibox suggestions、context menu、permission prompt、download panel |
| 集合与虚拟化 | `ui/collections` | LazyList、LazyGrid、TreeView、selection model、item key、recycling | tabs overview、history、bookmarks tree、downloads list、settings list |
| 命令/快捷键 | `ui/commands` | CommandId、Shortcut、MenuModel、enabled/checked state、command palette | browser menu、context menu、keyboard shortcuts、command palette |
| 表单 | `ui/forms` | Form、FieldState、validation、dirty/touched、error text、submit lifecycle | settings forms、permission choices、profile/preferences |
| 资产管线 | `ui/assets` | asset manifest、icons/images/fonts/shaders/locales/themes、density variants | toolbar icons、favicon fallback、theme package、locale catalog |
| 平台服务 | `ui/platform` | clipboard、drag/drop、file picker、notifications、haptics、share/open URL/system menu | downloads open/show, paste URL, drag tab/link, desktop menu bar |
| 状态恢复 | `ui/restoration` | restoration id、route stack、scroll offset、input selection、window metrics | restore chrome state、tab overview route、settings route、search input |
| 测试工具 | `ui/testing` | widget test、scene snapshot、semantics snapshot、gesture test、fake clock | browser chrome regression tests |
| DevTools | `ui/devtools` | tree inspector、layout bounds、semantics inspector、theme/i18n preview、perf timeline | browser UI debugging and responsive preview |
| 设计系统 | `ui/design-system` | density、motion tokens、component variants、state layers、platform style packs | desktop/mobile chrome visual consistency |

**验收场景**:

```text
场景: 浏览器命令同时驱动菜单、快捷键和命令面板
  假设 BrowserCommandModel 注册 reload、new_tab、close_tab
  当 用户点击菜单项、按快捷键或从 command palette 执行 reload
  那么 三者都分发同一个 BrowserAction::Reload
  验证: command dispatch integration test

场景: 移动端权限提示通过统一 overlay/sheet 展示
  假设 WebView 发起 camera permission request
  当 当前 shell 为 PhoneBrowserShell
  那么 browser-ui/chrome 使用 ui/overlay 的 bottom sheet route 展示 PermissionPrompt
  并且 focus trap、modal barrier、safe area 生效
  验证: mobile permission sheet test

场景: 大量下载记录使用虚拟化列表
  假设 downloads model 包含 10000 条记录
  当 DownloadPanel 渲染列表
  那么 ui/collections 只 materialize viewport 附近的 item
  并且 selection/keyboard navigation 保持正确
  验证: virtual list materialization test

场景: UI 状态可恢复
  假设 用户打开 downloads route 并滚动到列表中段
  当 runtime 保存并恢复 restoration snapshot
  那么 route、scroll offset 和 focused item 被恢复
  验证: restoration snapshot test
```

---

## 4. 非功能需求

### NFR-001: 浏览器无关性

- **描述**: 通用 UI SDK 核心必须不依赖浏览器业务、WebView 或网络模块。
- **测量标准**: `ui/core`、`ui/render`、`ui/runtime`、`ui/widgets`、`ui/patterns`、`ui/i18n`、`ui/animation`、`ui/gestures`、`ui/navigation`、`ui/overlay`、`ui/collections`、`ui/commands`、`ui/forms`、`ui/assets`、`ui/platform`、`ui/restoration`、`ui/testing`、`ui/devtools`、`ui/design-system` 的 Cargo 依赖审查通过，不依赖 browser-shell/webview/engine/net。
- **优先级**: 必须

### NFR-002: 渐进迁移能力

- **描述**: 浏览器迁移必须分阶段进行，每一阶段都能构建、运行和测试。
- **测量标准**: 每个阶段至少通过受影响 crate 的 `cargo test`，最终通过 `cargo test --workspace` 和 `cargo clippy --workspace --all-targets -- -D warnings`。
- **优先级**: 必须

### NFR-003: 性能

- **描述**: SDK 必须支持局部 layout/paint/composite 失效，不得要求每个事件全量重绘所有 UI。
- **测量标准**: hover、pressed、theme color 等变化不触发布局；resize、文本测量变化才触发布局。
- **优先级**: 必须

### NFR-004: 可访问性

- **描述**: SDK 必须有 SemanticsNode、焦点模型和本地化语义文本，为屏幕阅读器、键盘导航、高对比模式和多语言环境提供基础。
- **测量标准**: 每个可交互组件能生成基本语义节点；语义 label/description 使用 LocalizedText 解析结果。
- **优先级**: 必须

### NFR-005: 安全

- **描述**: YAML DSL 和表达式语言不得执行任意脚本；事件只能绑定到宿主显式注册的 ActionId；表达式求值必须无副作用且受 `EvalContext` 权限边界限制。
- **测量标准**: DSL parser 拒绝未知 action，expression engine 拒绝未知函数、状态写入、文件/网络/进程访问和递归/无限循环结构。
- **优先级**: 必须

### NFR-006: 可测试性

- **描述**: UI 组件布局、事件分发、主题解析、DSL 解析、动画、手势、导航、overlay、集合虚拟化、命令、资产、平台服务和状态恢复必须可单元测试。
- **测量标准**: `ui/*`、text foundation、`browser-ui/chrome` 均有无窗口单元测试、组件测试或 mock integration tests。
- **优先级**: 必须

### NFR-007: 跨平台

- **描述**: 桌面端优先支持 Windows、macOS、Linux；移动端在类型层预留 Android/iOS 后端接入点。
- **测量标准**: `ui-runtime`、`ui-platform`、`ui-gestures`、`ui-navigation`、`ui-overlay` API 不暴露 winit-specific 类型给 widgets/patterns/browser-ui。
- **优先级**: 应该

---

## 5. 接口需求

### IF-001: Widget 基础接口

- **类型**: API
- **规格**:

```rust
pub trait Widget {
    fn mount(&mut self, ctx: &mut MountCtx);
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props);
    fn event(&mut self, ctx: &mut EventCtx, event: &UiEvent) -> EventResult;
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx);
    fn semantics(&self, ctx: &mut SemanticsCtx);
}
```

- **错误处理**: Widget 不应 panic；无效 props 应在 update 阶段转为诊断信息。
- **默认动作**: 未处理事件返回 `EventResult::Ignored`。

### IF-002: Action 与 Message 接口

- **类型**: API
- **规格**:

```rust
pub struct ActionId(pub String);

pub enum EventResult {
    Ignored,
    Consumed,
    Emit(ActionId),
    EmitWithPayload(ActionId, ActionPayload),
}

pub trait ActionRegistry {
    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult;
}
```

- **错误处理**: 未注册 Action 必须返回错误或诊断，不得静默执行。

### IF-003: Theme 接口

- **类型**: API / 系统集成
- **规格**:

```rust
pub enum ColorSchemePreference {
    System,
    Light,
    Dark,
    Custom(ThemeId),
}

pub enum ResolvedColorScheme {
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

pub struct Theme {
    pub id: ThemeId,
    pub name: String,
    pub scheme: ResolvedColorScheme,
    pub palette: ColorPalette,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub shadow: ShadowTokens,
}
```

- **错误处理**: 自定义主题缺失 token 时从 base scheme 派生；非法颜色值导致主题加载失败。

### IF-004: WebViewWidget 接口

- **类型**: API / 自定义组件
- **规格**:

```rust
pub struct WebViewLayoutInput {
    pub rect: Rect,
    pub scale_factor: f32,
    pub theme: Theme,
}

pub struct WebViewPaintOutput {
    pub primitives: RenderPrimitives,
    pub scroll_metrics: ScrollMetrics,
}

pub struct ScrollMetrics {
    pub content_width: f32,
    pub content_height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub max_scroll_x: f32,
    pub max_scroll_y: f32,
}

pub enum ScrollCommand {
    By { dx: f32, dy: f32 },
    To { x: f32, y: f32 },
    Page { pages_x: f32, pages_y: f32 },
}
```

- **错误处理**: WebView render 失败时组件应输出错误占位 scene，并向宿主发出诊断事件。

### IF-005: YAML DSL 与表达式接口

- **类型**: API / 文件格式
- **规格**:

```rust
pub struct WidgetSpec {
    pub component: ComponentType,
    pub id: Option<WidgetId>,
    pub props: PropsMap,
    pub bindings: Vec<Binding>,
    pub actions: Vec<ActionBinding>,
    pub control: ControlDirectives,
    pub children: Vec<WidgetSpec>,
}

pub struct ControlDirectives {
    pub visible_when: Option<Expression>,
    pub enabled_when: Option<Expression>,
    pub for_each: Option<ForEachSpec>,
}

pub enum Expression {
    Literal(Value),
    Path(StatePath),
    Unary { op: UnaryOp, expr: Box<Expression> },
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Conditional { condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    Call { function: PureFunctionId, args: Vec<Expression> },
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
}

pub trait WidgetSpecLoader {
    fn load_str(&self, source: &str) -> Result<WidgetSpec, DslError>;
}

pub trait ExpressionEngine {
    fn parse(&self, source: &str) -> Result<Expression, DslError>;
    fn typecheck(&self, expr: &Expression, schema: &BindingSchema) -> Result<ValueType, DslError>;
    fn eval(&self, expr: &Expression, ctx: &EvalContext) -> Result<Value, DslError>;
}
```

- **错误处理**:
  - 未知 component: parse error。
  - 未知 prop: warning 或 strict mode error。
  - 未注册 action: validation error。
  - 表达式语法错误: parse error。
  - 表达式类型不匹配: typecheck error。
  - 禁止能力、未知函数、越权路径访问: validation error。
  - 任意脚本字段: parse error。

### IF-006: Runtime Adapter 接口

- **类型**: 系统集成
- **规格**:

```rust
pub trait PlatformRuntime {
    fn run(self, app: impl UiApp) -> UiResult<()>;
    fn request_redraw(&mut self, window: WindowId);
    fn set_ime_area(&mut self, window: WindowId, rect: Option<Rect>);
    fn system_theme(&self) -> SystemThemeSnapshot;
}
```

- **错误处理**: 平台事件转换失败必须产生 diagnostic，不得破坏事件循环。

### IF-007: I18n 资源与文本解析接口

- **类型**: API / 文件格式 / 系统集成
- **规格**:

```rust
pub struct LocaleId(pub String);
pub struct MessageId(pub String);

pub enum TextDirection {
    Ltr,
    Rtl,
    Auto,
}

pub enum LocalizedText {
    Literal(String),
    Message(MessageRef),
}

pub struct MessageRef {
    pub id: MessageId,
    pub params: MessageParams,
}

pub struct MessageCatalog {
    pub locale: LocaleId,
    pub direction: TextDirection,
    pub messages: HashMap<MessageId, MessageEntry>,
}

pub struct I18nContext {
    pub locale: LocaleId,
    pub fallback_chain: Vec<LocaleId>,
    pub direction: TextDirection,
}

pub trait I18nProvider {
    fn resolve(&self, text: &LocalizedText, ctx: &I18nContext) -> Result<ResolvedText, I18nError>;
    fn direction(&self, locale: &LocaleId) -> TextDirection;
}
```

- **文件格式**:
  - 首阶段使用 YAML catalog，字段包含 `locale`、`direction`、`messages`。
  - message id 使用稳定点分命名，如 `browser.address.placeholder`。
  - message entry 可包含 `value`、`description`、`params`、plural forms。
  - `description` 只给翻译人员和审核工具使用，不进入运行时可见文本。
- **错误处理**:
  - 缺失 key: 按 fallback chain 查找，全部缺失时返回 key 占位并产生 diagnostic。
  - 缺失参数: 返回 `I18nError::MissingParam`。
  - 参数类型不匹配: 返回 `I18nError::InvalidParamType`。
  - plural form 缺失: 使用 locale 默认 fallback form，并产生 diagnostic。
  - 非法 locale 或 direction: catalog load error。

### IF-008: Text Foundation 接口

- **类型**: API / 渲染基础设施
- **规格**:

```rust
pub struct FontRequest {
    pub families: Vec<FontFamily>,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub locale: Option<LocaleId>,
}

pub struct ShapeInput {
    pub text: String,
    pub font_request: FontRequest,
    pub size_px: f32,
    pub direction: TextDirection,
    pub script: Option<Script>,
    pub scale_factor: f32,
}

pub struct TextMeasureInput {
    pub text: String,
    pub font_request: FontRequest,
    pub size_px: f32,
    pub max_width: Option<f32>,
    pub direction: TextDirection,
}

pub trait FontProvider {
    fn query(&self, request: &FontRequest) -> Result<FontMatch, TextError>;
    fn fallback_chain(&self, text: &str, request: &FontRequest) -> Result<Vec<FontMatch>, TextError>;
}

pub trait TextShaper {
    fn shape(&self, input: &ShapeInput) -> Result<ShapedText, TextError>;
}

pub trait TextMeasurer {
    fn measure(&self, input: &TextMeasureInput) -> Result<TextMetrics, TextError>;
}

pub trait GlyphCache {
    fn get_or_insert(&mut self, glyph: GlyphKey) -> Result<GlyphAtlasEntry, TextError>;
}
```

- **调用方边界**:
  - `ui/render` 调用本接口处理 UI 文本、控件测量、TextInput 光标边界和 glyph primitive。
  - `zero-webview` / `zero-engine` 调用本接口处理网页文本 shaping、测量和 glyph cache。
  - WebView 负责把 CSS `font-*`、writing mode、inline layout 语义转换为基础层输入。
- **错误处理**:
  - 字体缺失: 使用 fallback chain，全部失败时返回 tofu/missing-glyph diagnostic。
  - shaping 失败: 返回 `TextError::ShapeFailed`，调用方决定占位或降级。
  - glyph atlas 满: 触发 atlas eviction 或返回 `TextError::AtlasFull`。
  - 不支持的 font feature / variation axis: 忽略并产生 diagnostic，不得 panic。

### IF-009: Responsive Layout 与 Adaptive Shell 接口

- **类型**: API / UI / 系统集成
- **规格**:

```rust
pub enum ViewportClass {
    Compact,
    Medium,
    Expanded,
}

pub enum PlatformClass {
    Desktop,
    Tablet,
    Phone,
    Foldable,
}

pub enum InputClass {
    MouseKeyboard,
    Touch,
    Hybrid,
}

pub struct WindowMetrics {
    pub viewport: Size,
    pub density: f32,
    pub text_scale: f32,
    pub safe_area: Insets,
    pub soft_keyboard: Option<Rect>,
    pub orientation: Orientation,
    pub viewport_class: ViewportClass,
    pub platform_class: PlatformClass,
    pub input_class: InputClass,
}

pub trait AdaptiveComponent<Props> {
    fn build(&self, props: Props, metrics: &WindowMetrics) -> WidgetSpec;
}

pub trait BrowserChromeShell {
    fn build(&self, model: BrowserChromeModel, metrics: &WindowMetrics) -> WidgetSpec;
}
```

- **DSL 规格**:

```yaml
# responsive 键在 DSL WidgetSpec 节点中代替 component，按 AdaptiveBranch
# (viewport/platform/input) 选择匹配分支；default 始终必需。
responsive:
  compact:
    component: PhoneBrowserShell
  expanded:
    component: DesktopBrowserShell
  default:
    component: TabletBrowserShell
```

- **匹配优先级**（按维度降序）：viewport（`compact`/`medium`/`expanded`）→ platform（`mobile`/`desktop`/`embedded`）→ input（`touch`/`pointer`/`keyboard`）。同一维度内按列表顺序优先匹配，之后按 `default` 回落。
- **默认动作**:
  - `responsive:` 节点**始终要求 `default` 分支**（无 default → DSL validation error）。
  - soft keyboard rect 为空时按无键盘布局。
  - safe area 为空时按 zero insets 处理。
- **错误处理**:
  - `responsive:` 缺少 `default`: DSL validation error。
  - 分支组件不存在: DSL validation error。
  - `WindowMetrics` 非法，如负尺寸或 NaN density: runtime diagnostic，并使用最后一次有效 metrics。

### IF-010: 完整 UI 能力接口集合

- **类型**: API / UI / 系统集成
- **规格**:

```rust
pub trait AnimationClock {
    fn now(&self) -> Duration;
    fn request_frame(&mut self);
}

pub trait GestureRecognizer {
    fn handle_pointer(&mut self, event: &PointerEvent) -> GestureResult;
}

pub trait Navigator {
    fn push(&mut self, route: RouteSpec) -> RouteId;
    fn pop(&mut self) -> Option<RouteId>;
    fn replace(&mut self, route: RouteSpec) -> RouteId;
}

pub trait OverlayHost {
    fn show(&mut self, entry: OverlayEntry) -> OverlayId;
    fn dismiss(&mut self, id: OverlayId);
}

pub trait VirtualCollection {
    fn item_count(&self) -> usize;
    fn item_key(&self, index: usize) -> ItemKey;
    fn build_item(&self, index: usize) -> WidgetSpec;
}

pub trait CommandRegistry {
    fn register(&mut self, command: CommandSpec);
    fn execute(&mut self, id: CommandId) -> CommandResult;
}

pub trait AssetProvider {
    fn load(&self, id: AssetId, variant: AssetVariant) -> Result<Asset, AssetError>;
}

pub trait PlatformServices {
    fn clipboard(&self) -> &dyn ClipboardService;
    fn drag_drop(&self) -> &dyn DragDropService;
    fn file_picker(&self) -> &dyn FilePickerService;
    fn notifications(&self) -> &dyn NotificationService;
}

pub trait RestorationStore {
    fn save(&mut self, id: RestorationId, value: RestorationValue);
    fn restore(&self, id: &RestorationId) -> Option<RestorationValue>;
}
```

- **浏览器接入规范**:
  - browser command 必须先注册为 `CommandSpec`，再映射到 menu、shortcut、context menu、command palette。
  - browser modal UI 必须通过 `Navigator` / `OverlayHost` 打开，不直接在任意组件中自建顶层浮层。
  - history/bookmarks/downloads/tab overview 必须通过 `VirtualCollection` 或等价 lazy collection 承载。
  - downloads、file picker、clipboard、drag/drop 等必须通过 `PlatformServices`，不得让 `ui/widgets` 直接依赖平台 API。
  - route、scroll offset、TextInput selection、active overlay 等恢复点必须有 stable `RestorationId`。
- **错误处理**:
  - 未注册 command: 返回 diagnostic，不得静默忽略。
  - overlay host 不存在: 返回 validation/runtime error。
  - virtual item key 重复: test/runtime diagnostic。
  - asset 缺失: 使用 fallback asset 或返回 `AssetError::NotFound`。
  - platform service 不可用: 返回 capability error，调用方展示降级 UI。

---

## 6. 约束与假设

### 6.1 必须约束

- 通用 SDK 放在 `ui/` 顶层目录。
- 浏览器专属组件放在 `browser-ui/` 或明确的 browser integration 目录。
- UI SDK 核心层不依赖浏览器业务 crate。
- WebView 作为自定义组件集成，不把网页 DOM 映射为 UI widgets。
- UI SDK 与 WebView 必须共享文本/字体基础层，避免字体发现、fallback、shaping 和 glyph cache 重复实现。
- UI SDK 必须支持响应式/自适应布局，浏览器 desktop/mobile chrome 必须复用同一状态模型和 Action 合约。
- UI SDK 首版架构必须包含动画、手势、导航、overlay、集合虚拟化、命令、表单、资产、平台服务、状态恢复、测试和 devtools 扩展点。
- YAML DSL 不执行任意代码；表达式语言只允许无副作用计算。
- 用户可见字符串必须通过 `LocalizedText` 或 message id 引用，production DSL 不得硬编码可见文案。
- 主题系统使用 semantic token，组件不得硬编码浏览器色值。
- 所有公共 API 必须有 `///` 文档注释。

### 6.2 禁止约束

- 禁止把 `zero-browser-shell` 引入 `ui/core`、`ui/render`、`ui/runtime`、`ui/widgets`。
- 禁止让 DSL 直接访问文件系统、网络、进程或任意 Rust 函数。
- 禁止让表达式修改应用状态、启动异步任务、读取系统时间或生成随机数。
- 禁止在表达式语言中提供递归、无限循环或动态代码加载能力。
- 禁止在 `ui/widgets`、`ui/patterns` 中内置浏览器文案或浏览器 message catalog。
- 禁止将网页 DOM/CSS inline layout、CSS text 规范语义或 DOM selection 迁入 UI SDK 通用布局系统。
- 禁止将 desktop 浏览器 chrome 原样缩小作为移动端界面。
- 禁止首阶段为了“通用”重写浏览器内核页面管线。
- 禁止在 `ui/widgets` 中引入浏览器专属概念，如 URL 安全状态、书签、下载、标签历史。

### 6.3 已定决策

- 架构类型: retained UI，而不是 immediate mode。
- 状态流: 单向数据流，事件触发 Action，应用状态更新后驱动 UI。
- DSL: 以 YAML 为第一版格式，内嵌受控表达式语言，解析为 `WidgetSpec` 与 Expression AST。
- WebView: 高级自定义组件，WebView 负责页面语义，UI SDK 负责外部组件矩形和合成。
- 主题: SDK 一等能力，支持 System/Light/Dark/Custom/HighContrast。
- 国际化: 采用独立资源文件 + message id 引用；SDK 提供 `ui/i18n` 机制，浏览器文案由 `browser-ui/chrome/i18n` 或 `apps/browser/i18n` 提供。
- 文本基础层: 新增共享 `foundation/text` 或等价 `zero-text-foundation`，供 UI SDK 与 WebView 共同使用；网页排版规则仍由 WebView/engine 负责。
- 响应式布局: UI SDK 提供 `WindowMetrics`、`ViewportClass`、`InputClass` 和 adaptive branch；浏览器移动端使用 `PhoneBrowserShell` 等 shell 复用 `BrowserChromeModel` 和 `BrowserAction`。
- 完整 UI 能力: 首版先定义 `ui/animation`、`ui/gestures`、`ui/navigation`、`ui/overlay`、`ui/collections`、`ui/commands`、`ui/forms`、`ui/assets`、`ui/platform`、`ui/restoration`、`ui/testing`、`ui/devtools`、`ui/design-system` 的边界和最小接口。

### 6.4 技术约束

- Rust edition 2024，MSRV 1.85。
- 首阶段继续复用 `zero-render-foundation` 的图元和 CPU/GPU 渲染能力。
- 桌面宿主优先复用 winit，但 winit 类型不得泄漏到 widgets 公共 API。
- WebView adapter 可以依赖 `zero-webview`，但 core/widgets 不可依赖。

### 6.5 假设

- 假设 `zero-render-foundation` 的图元模型足以承载第一阶段 SDK scene 输出。状态: 已验证。
- 假设浏览器 chrome 的大部分可视组件可从 `apps/browser` 迁移为 `browser-ui/chrome`。状态: 待验证。
- 假设移动端短期只需要 API 预留，不要求首阶段端到端运行。状态: 已确认方向。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
---|---|---|---|
 窗口和事件循环 | 复用现有模块 | `crates/host-runtime` / winit | 未来迁移为 `ui/adapters/winit` |
 图元渲染 | 复用现有模块 | `crates/render-foundation` | 首阶段不重写 GPU/CPU 后端 |
 文本/字体基础层 | 仓内自实现 + 迁移现有能力 | `foundation/text` 或 `crates/render-foundation/text` | 共享 font fallback、shaping、glyph cache；最终位置需在 M1 确认 |
 响应式布局与 adaptive shell | 仓内自实现 | `ui/core/layout`、`ui/runtime`、`browser-ui/chrome` | metrics、breakpoints、adaptive branch、desktop/tablet/phone shell |
 完整 UI 能力骨架 | 仓内自实现 | `ui/animation`、`ui/gestures`、`ui/navigation`、`ui/overlay`、`ui/collections`、`ui/commands`、`ui/forms`、`ui/assets`、`ui/platform`、`ui/restoration`、`ui/testing`、`ui/devtools`、`ui/design-system` | 首版定义接口和最小测试，逐步实现浏览器所需子集 |
 浏览器状态 | 复用现有模块 | `crates/browser-shell` | 仅 browser-ui/chrome 或 apps/browser 使用 |
 WebView | 复用现有模块 | `crates/webview` | 通过 `ui/adapters/webview` 包装为 Widget |
 滚动条几何 | 迁移现有实现 | `apps/browser/src/page_scroll.rs` | 通用部分进入 `ui/widgets` |
 主题探测 | 迁移现有实现 | `apps/browser/src/app_platform.rs`、`colors.rs` | 下沉到 `ui/runtime` 与 `ui/core/theme.rs` |
 YAML DSL | 仓内自实现 | `ui/dsl` | 使用 serde_yaml 需另行评估依赖 |
 DSL 表达式语言 | 仓内自实现 | `ui/dsl/src/expression/**` | parser、AST、typecheck、eval、sandbox 均在仓内实现；是否引入 parser combinator crate 待评估 |
 国际化资源系统 | 仓内自实现 | `ui/i18n` | catalog loader、fallback、参数替换、plural、RTL direction；是否引入 ICU/Fluent 类依赖待评估 |

### 6.6 代码变更边界

**允许修改**:

- `docs/specs/**`
- `Cargo.toml`
- `foundation/**`
- `ui/**`
- `browser-ui/**`
- `apps/browser/**`
- `crates/host-runtime/**` 中与 UI runtime 迁移相关部分
- `crates/render-foundation/**` 中为 scene 输出所需的最小接口

**禁止修改**:

- 与本任务无关的 DOM/CSS/layout 兼容性代码。
- 将 DOM/CSS/layout 行为迁入 UI SDK 或 text foundation 的修改。
- 与本任务无关的网络、安全、存储、脚本 sandbox 行为。
- 未经明确阶段计划覆盖的 WPT/reftest 行为。

### 6.7 执行技能提示

| 范围 / 触发条件 | Skill | 模式 | 原因 |
|---|---|---|---|
 需求/RFC 变更 | `lei-spec-rfc` | required | 本文档即 Spec/RFC 输出 |
 大规模阶段拆解 | roadmap planner（如可用） | preferred | 本方案跨多个里程碑，适合拆 task pack |
 UI 验收、截图和产品 smoke | `lei-product-acceptance` | preferred | 浏览器迁移后需可视验收 |
 移动端 HarmonyOS/ArkUI 适配 | `lei-harmonyos6-dev` | preferred | 若未来引入 HarmonyOS 端 |

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
---|---|---|---|---|
 FR-001 | 独立 `ui/` 目录 | 必须 | 目录边界决定长期归属 | M0 |
 FR-002 | SDK 分层 | 必须 | 防止通用 SDK 被浏览器污染 | M0 |
 FR-003 | retained UI + 单向数据流 | 必须 | 决定组件事件模型 | M1 |
 FR-004 | 三棵树模型 | 必须 | 支撑状态保持和局部刷新 | M1 |
 FR-005 | WebViewWidget | 必须 | 浏览器迁移核心组件 | M2 |
 FR-006 | 滚动边界 | 必须 | 当前滚动逻辑需拆分 | M2 |
 FR-007 | 主题系统 | 必须 | 用户明确要求，且影响所有组件 | M1 |
 FR-008 | YAML DSL 与完整表达式语言 | 必须 | 对外复用和声明式配置；用户已明确要求完整表达式能力 | M3 |
 FR-009 | 首批组件 | 必须 | 浏览器迁移前置 | M2 |
 FR-010 | 桌面/移动宿主抽象 | 应该 | 长期跨平台目标 | M3 |
 FR-011 | 无障碍/焦点/IME | 必须 | 后补代价高 | M1 |
 FR-012 | 局部失效刷新 | 必须 | 性能基础 | M1 |
 FR-013 | 国际化资源与 message id 引用 | 必须 | 文案、RTL、无障碍和移动端本地化需提前纳入架构 | M1/M3 |
 FR-014 | 共享文本/字体基础层 | 必须 | UI SDK 与 WebView 都需要一致字体 fallback、shaping 和 glyph cache | M1/M2 |
 FR-015 | 响应式布局与移动端浏览器 chrome | 必须 | 移动端需要 adaptive shell，且应复用 desktop 的状态和组件能力 | M2/M4 |
 FR-016 | 完整应用级 UI 能力 | 必须 | 对齐主流 mobile/desktop UI 框架，避免后续补架构返工 | M1-M4 |

### 建议里程碑

- **M0: 文档与架构边界确认**
  - 输出本 Spec/RFC。
  - 建立目录和依赖边界决策。
  - 不修改运行时代码。

- **M1: UI SDK 核心骨架**
  - 新增 `ui/core`、`ui/render`、`ui/runtime`、`ui/widgets`、`ui/patterns`、`ui/i18n` 最小 crate。
  - 新增 `foundation/text` 或等价 text foundation crate skeleton。
  - 新增 animation、gestures、navigation、overlay、collections、commands、forms、assets、platform、restoration、testing、devtools、design-system 的接口 skeleton。
  - 定义基础类型、Widget/Element/RenderNode、Theme、I18n、Text、Event、Action、Focus、Semantics、Invalidation。
  - 提供无窗口单元测试。

- **M2: 浏览器首批组件迁移**
  - 抽取滚动条、按钮、文本输入、菜单、popup、toolbar。
  - 抽取 SearchField、SuggestionList、TabBar 等跨应用组合模式。
  - 新增 `browser-ui/chrome`。
  - `ui/render` 和 `ui/widgets::TextInput` 接入共享文本基础层。
  - `ui/adapters/webview` 明确 WebView 调用共享文本基础层的边界。
  - 定义 `BrowserChromeModel`、`BrowserAction` 与 desktop/tablet/phone shell 的共享合约。
  - Browser menu/shortcut/context menu 接入 `ui/commands`。
  - PermissionPrompt、DownloadPanel、SiteInfoPanel 接入 `ui/overlay`。
  - Downloads/Bookmarks/History/TabOverview 接入 `ui/collections`。
  - 按 §8.4.1A 将浏览器 chrome 逐步迁移到 SDK，功能保持不变。
  - 引入 `ui/adapters/webview` 和 WebViewWidget，避免 `apps/browser` 直接拼接 WebView 与 chrome。

- **M3: DSL 与示例应用**
  - 新增 `ui/dsl`。
  - 支持 YAML -> WidgetSpec。
  - 实现 DSL 表达式语言的 parser、AST、typecheck、eval 和 sandbox negative tests。
  - 支持条件渲染、列表渲染、属性绑定、样式绑定和 action payload 表达式。
  - 支持 DSL 中通过 `i18n` message id 引用独立 locale 资源文件。
  - 支持 DSL 中声明 command、overlay、route、collection、asset、animation 和 gesture 引用。
  - 提供 counter/form/browser-shell-demo 示例。

- **M4: 跨平台与移动端预留落地**
  - 完善 runtime adapter 接口。
  - 增强触摸、软键盘、safe area、text scale、platform back gesture。
  - 实现或补齐 `PhoneBrowserShell`、`TabletBrowserShell` 的最小布局 skeleton。
  - 增加 responsive/adaptive layout snapshot。
  - 增加 gesture、navigation route、state restoration、platform service 的移动端适配 skeleton。
  - 接入 `ui/testing`/`ui/devtools` 的 responsive preview、layout bounds、semantics snapshot。
  - 为 Android/iOS/HarmonyOS 接入提供设计文档或最小 adapter skeleton。

### 实施交接

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|---|---|---|---|
 `docs/specs/ui-sdk-spec-rfc.md` | 新增 | 固化方案 | 本文档为后续实施依据 |
 `Cargo.toml` | 后续修改 | 加入新 workspace members | 不在 M0 修改也可 |
 `foundation/text` 或 `crates/render-foundation/text` | 新增/拆分 | 共享文本和字体基础层 | 位置需在 M1 确认，避免 UI/WebView 重复实现 |
 `ui/core` | 新增 | 核心类型与协议 | 不得依赖浏览器 |
 `ui/render` | 新增 | scene/render node 抽象 | 与 render-foundation 依赖边界需控制 |
 `ui/runtime` | 新增 | runtime 和 platform abstraction | winit 类型不得泄漏到 widgets |
 `ui/widgets` | 新增 | 通用组件 | 不加入浏览器语义 |
 `ui/patterns` | 新增 | 通用组合模式 | 不依赖 browser-shell/webview |
 `ui/i18n` | 新增 | 国际化资源与文本解析 | 不内置浏览器文案 |
 `ui/animation` | 新增 | 动画与过渡 | 首版可 skeleton，需 fake clock 测试 |
 `ui/gestures` | 新增 | 手势识别与 pointer arbitration | 移动端关键能力 |
 `ui/navigation` | 新增 | route stack、modal/sheet/dialog route | 浏览器设置/下载/权限页接入 |
 `ui/overlay` | 新增 | popover、tooltip、menu、dialog、sheet、toast | 统一浮层和 focus trap |
 `ui/collections` | 新增 | lazy list/grid/tree/table 与 selection | 下载/历史/书签/标签概览 |
 `ui/commands` | 新增 | command、shortcut、menu model | 浏览器菜单和快捷键 |
 `ui/forms` | 新增 | 表单状态与校验 | 设置页和偏好页 |
 `ui/assets` | 新增 | asset manifest 与资源变体 | icon/font/image/theme/locale |
 `ui/platform` | 新增 | clipboard、drag/drop、file picker、notification 等平台服务 | 不让 widgets 直接依赖平台 API |
 `ui/restoration` | 新增 | route、scroll、input、window state 恢复 | 移动端和桌面 session 恢复 |
 `ui/testing` | 新增 | widget、scene、semantics、gesture 测试工具 | 统一 UI 验收 |
 `ui/devtools` | 新增 | tree/layout/semantics/theme/i18n/perf inspector | 开发调试 |
 `ui/design-system` | 新增 | density、motion、variants、state layers | 长期多风格组件库 |
 `ui/dsl` | 新增 | YAML DSL 与表达式语言 | 严格禁脚本，表达式只允许无副作用求值 |
 `ui/adapters/webview` | 新增 | WebViewWidget | 可依赖 zero-webview |
 `browser-ui/chrome` | 新增 | 浏览器专属组件 | 可依赖 browser-shell/webview adapter |
 `apps/browser` | 修改 | 迁移为 SDK 消费方 | 分阶段，避免一次大改 |

#### 职责映射

| 模块/文件 | 职责 | 依赖/被依赖 | 验证方式 |
|---|---|---|---|
 `ui/core` | Widget、Event、Theme、Action、Focus、Semantics、Invalidation | 被所有 UI crate 依赖 | cargo test -p zero-ui-core |
 `ui/core/layout` | Constraints、WindowMetrics、ViewportClass、adaptive branch | 被 runtime/widgets/browser-ui 依赖 | responsive layout tests |
 `zero-text-foundation` | FontProvider、TextShaper、TextMeasurer、GlyphCache | 被 ui-render 和 zero-webview 依赖 | shared text tests |
 `ui/render` | SceneNode、PaintCtx、RenderTree、合成输入 | 依赖 ui-core，可复用 render-foundation | cargo test -p zero-ui-render |
 `ui/runtime` | App lifecycle、runtime loop 抽象、ThemeChanged、IME、input routing | 依赖 ui-core/render | runtime mock tests |
 `ui/widgets` | Button、TextInput、Menu、ScrollBar、Popup 等 | 依赖 ui-core/render | widget tests |
 `ui/patterns` | SearchField、SuggestionList、TabBar、DataList、DialogScaffold 等组合模式 | 依赖 ui-core/render/widgets | pattern tests |
 `ui/i18n` | Locale、MessageCatalog、I18nProvider、fallback、plural、RTL direction | 依赖 ui-core 或被 ui-core/runtime 使用 | i18n catalog/fallback/rtl tests |
 `ui/animation` | FrameClock、AnimationController、Tween/Spring、Transition | 被 widgets/overlay/navigation 使用 | fake clock tests |
 `ui/gestures` | Pointer routing、gesture arena、tap/drag/pinch/fling | 被 runtime/widgets/webview adapter 使用 | gesture recognizer tests |
 `ui/navigation` | Route stack、modal/sheet/dialog route、restorable route | 被 apps/browser 和 browser-ui/chrome 使用 | route stack tests |
 `ui/overlay` | OverlayHost、Portal、Popover/Dialog/Sheet/Toast | 被 browser-ui/chrome 使用 | overlay focus/dismiss tests |
 `ui/collections` | LazyList/Grid/Tree/Table、selection、item key | 被 downloads/bookmarks/history/tab overview 使用 | virtualization tests |
 `ui/commands` | CommandRegistry、ShortcutMap、MenuModel、CommandPalette | 被 browser menu/shortcut/context menu 使用 | command dispatch tests |
 `ui/assets` | AssetProvider、manifest、density/theme/locale variants | 被 widgets/patterns/browser-ui 使用 | asset resolution tests |
 `ui/platform` | Clipboard/DragDrop/FilePicker/Notification/Haptics services | 被 apps/browser/runtime adapter 使用 | platform mock tests |
 `ui/restoration` | RestorationStore、RestorationId、snapshot | 被 runtime/navigation/widgets 使用 | restoration tests |
 `ui/testing` | Test runtime、fake clock、scene/semantics snapshot | 被 all UI crates tests 使用 | self tests |
 `ui/devtools` | Inspector protocol/views | dev-only dependency | devtools smoke |
 `ui/dsl` | YAML -> WidgetSpec；Expression parse/typecheck/eval | 依赖 ui-core | parser/typecheck/eval/sandbox tests |
 `ui/adapters/webview` | zero-webview 包装为 Widget | 依赖 ui-core/render/widgets + zero-webview | WebViewWidget tests |
 `browser-ui/chrome` | 浏览器地址栏、标签栏、书签栏、下载面板 | 依赖 ui + browser-shell | browser chrome tests |
 `apps/browser` | 应用入口、状态编排、多进程、宿主绑定 | 依赖 browser-ui/chrome | product smoke |

#### 新能力来源对照

| 能力/需求 | 实现承载位置 | 来源类型 | 验证方式 |
---|---|---|---|
 Widget tree | `ui/core` | 新增 | 单元测试 |
 Element retained state | `ui/runtime` 或 `ui/core` | 新增 | 状态保持测试 |
 Render tree / SceneNode | `ui/render` | 新增 + 复用 render-foundation | scene tests |
 Text foundation | `foundation/text` 或 `crates/render-foundation/text` | 新增 + 迁移现有字体能力 | font fallback/shaping/glyph cache tests |
 Responsive layout | `ui/core/layout` + `ui/runtime` | 新增 | breakpoint/metrics/adaptive tests |
 Browser adaptive shell | `browser-ui/chrome` | 新增 | desktop/tablet/phone shell tests |
 Animation/gesture/navigation/overlay/collections/commands/assets/restoration | `ui/*` 对应模块 | 新增 | module skeleton tests |
 ThemeResolver | `ui/core/theme.rs` | 迁移 + 新增 | theme tests |
 I18nProvider | `ui/i18n` | 新增 | catalog/fallback/plural/rtl tests |
 SystemThemeProvider | `ui/runtime` / `ui/adapters/winit` | 迁移 + 新增 | platform mock tests |
 ScrollBar | `ui/widgets` | 从 page_scroll 迁移通用部分 | geometry/hit-test tests |
 通用组合模式 | `ui/patterns` | 新增 | pattern composition tests |
 浏览器领域组件 | `browser-ui/chrome` | 从 apps/browser 拆分 + 新增组合层 | browser chrome component tests |
 WebViewWidget | `ui/adapters/webview` | 新增 wrapper | integration tests |
 YAML DSL 与表达式语言 | `ui/dsl` | 新增 | parser/schema/typecheck/eval/sandbox tests |

#### 推荐修改顺序

1. 建立 `ui/` 和 `browser-ui/` 目录决策文档。
2. 新增 `ui/core` 最小 crate，仅包含基础类型和测试。
3. 新增或拆分共享 text foundation，定义 FontProvider/TextShaper/TextMeasurer/GlyphCache。
4. 新增 `ui/render`，定义 scene/render node、paint context，并接入 text foundation。
5. 新增 `ui/widgets`，迁移最小 ScrollBar/Button/TextInput。
6. 新增 `ui/patterns`，实现 SearchField、SuggestionList、TabBar 等通用组合模式。
7. 新增 `ui/i18n`，实现独立资源文件加载、message id 解析、fallback、参数和 RTL direction。
8. 新增 `browser-ui/chrome`，按 §8.4.1A 迁移浏览器 chrome 的第一组领域组件。
9. 修改 `apps/browser` 消费 `browser-ui/chrome`，只保留状态编排和 Action dispatch。
10. 新增 `ui/adapters/webview`，将 WebView 封装为 Widget，并明确 WebView 使用 text foundation 的边界。
11. 新增 responsive/adaptive layout primitives，并在 `browser-ui/chrome` 中定义 desktop/tablet/phone shell。
12. 新增 animation、gestures、navigation、overlay、commands、collections、assets、platform、restoration 的接口 skeleton。
13. 将浏览器菜单/快捷键接入 commands，权限/下载/site info 接入 overlay，下载/历史/书签接入 collections。
14. 新增 `ui/dsl`，支持 YAML -> WidgetSpec、完整表达式语言、i18n message id 引用、responsive branch、command/route/overlay/asset 引用。
15. 补示例、测试和验收脚本。

#### 首批提交建议

| 批次 | 范围 | 预期结果 | 验证 |
---|---|---|---|
 Batch 1 | 文档 + 空目录/README | 边界明确 | 文档 review |
 Batch 2 | `ui/core` | 核心类型可编译 | cargo test -p zero-ui-core |
 Batch 3 | text foundation skeleton | 字体查询、fallback、shape/measure 接口可编译 | cargo test -p zero-text-foundation |
 Batch 4 | `ui/render` + `ui/widgets::ScrollBar` | 第一个可测组件 | cargo test -p zero-ui-widgets |
 Batch 5 | `ui/patterns` skeleton | 通用组合模式可复用 | cargo test -p zero-ui-patterns |
 Batch 6 | `ui/i18n` skeleton + locale catalog 示例 | message id 可解析 | cargo test -p zero-ui-i18n |
 Batch 7 | `browser-ui/chrome` skeleton + §8.4.1A 首批映射组件 | 浏览器专属包存在 | cargo check |
 Batch 8 | responsive/adaptive shell skeleton | desktop/tablet/phone shell 可根据 metrics 选择 | adaptive shell tests |
 Batch 9 | app-level UI modules skeleton | animation/gestures/navigation/overlay/commands/collections/assets/restoration 可编译 | targeted cargo test |
 Batch 10 | browser chrome 接入 commands/overlay/collections | 菜单、权限弹层、下载列表走统一模块 | browser chrome integration tests |
 Batch 11 | apps/browser 局部迁移 | 浏览器仍可运行 | cargo run --bin zero-browser + smoke |

---

## 8. 技术设计 RFC

### 8.1 现状分析

**当前架构**:

```text
apps/browser
  -> 手写 BrowserApp 状态、输入、布局、绘制
  -> zero-browser-shell 管理浏览器业务状态
  -> zero-webview 渲染网页
  -> zero-host-runtime 管理窗口和事件
  -> zero-render-foundation 输出图元
```

**问题/痛点**:

- UI 几何、绘制、输入处理集中在 `apps/browser`，复用困难。
- 滚动条、菜单、文本输入、按钮等通用概念没有独立组件模型。
- 主题能力仍偏浏览器本地实现，不是 SDK 级能力。
- WebView 与外层 UI 的边界目前由浏览器应用代码拼接。
- 无障碍、焦点、IME、移动端预留尚未形成统一 SDK 模型。
- 外部程序无法复用当前 GUI 能力。

**相关代码**:

- `apps/browser/src/app.rs`
- `apps/browser/src/app_render.rs`
- `apps/browser/src/app_input.rs`
- `apps/browser/src/page_scroll.rs`
- `apps/browser/src/colors.rs`
- `apps/browser/src/layout.rs`
- `crates/browser-shell/src/lib.rs`
- `crates/webview/src/lib.rs`
- `crates/host-runtime/src/lib.rs`
- `crates/render-foundation/src/lib.rs`

### 8.2 目标状态

**目标架构**:

```text
apps/browser
  -> browser-ui/chrome
    -> ui/widgets
    -> ui/patterns
    -> ui/i18n
    -> ui/adapters/webview
      -> zero-webview
    -> zero-browser-shell
  -> ui/runtime
    -> ui/adapters/winit
  -> ui/render
    -> zero-text-foundation
    -> zero-render-foundation

External GUI apps
  -> ui/core + ui/runtime + ui/widgets + ui/patterns + ui/i18n
  -> zero-text-foundation
  -> optional ui/dsl
```

**关键变化**:

- `apps/browser` 不再直接拥有所有 UI 绘制细节。
- 通用 UI 能力进入 `ui/` 顶层目录。
- 通用组合模式进入 `ui/patterns`，供浏览器和外部应用复用。
- 国际化资源机制进入 `ui/i18n`，供浏览器和外部应用按 locale 加载独立文案资源。
- 文本/字体基础能力进入 `zero-text-foundation`，供 `ui/render` 和 `zero-webview` 共同使用。
- 浏览器专属 UI 进入 `browser-ui/chrome`，作为基于 UI SDK 的领域组件库。
- WebView 作为高级 Widget 集成。
- 主题、焦点、IME、无障碍、事件和失效模型统一归 SDK。

**浏览器接入思路**:

```text
zero-browser-shell state + zero-webview handles
  -> apps/browser 创建 BrowserChromeModel
  -> apps/browser 注入 Locale/I18nProvider/Theme
  -> browser-ui/chrome 组件接收 props、message id 并发出 BrowserAction
  -> ui/widgets / ui/patterns / ui/adapters/webview 生成 Widget tree
  -> ui-render 和 WebView 均调用 zero-text-foundation
  -> ui-runtime 维护 Element tree、事件路由、焦点、IME、无障碍
  -> ui-render 生成 Scene tree
  -> render-foundation 输出像素
```

`apps/browser` 不再直接计算所有 toolbar/tab/address/menu 几何，也不直接绘制这些 chrome 元素；它负责状态编排、进程/导航接入、Action dispatch 和应用生命周期。

### 8.3 影响范围分析

| 影响项 | 影响程度 | 说明 |
---|---|---|
 `apps/browser` | 高 | 需要逐步迁移 UI 绘制和输入 |
 `crates/browser-shell` | 低 | 保持 UI-agnostic，只作为 browser-ui 数据来源 |
 `crates/webview` | 中 | 需要暴露更清晰的 Widget adapter 接口 |
 `crates/host-runtime` | 中 | 部分能力可能迁移/包装为 ui-runtime adapter |
 `crates/render-foundation` | 中 | 可能作为 ui-render backend |
 `tests/integration` | 中 | 需要新增 UI 和浏览器迁移验证 |
 外部 API | 低到中 | 新 SDK 是新增能力，首阶段不破坏现有 API |

### 8.4 详细设计

#### 8.4.1 模块设计

```text
zero-ui-core
  geometry.rs
  event.rs
  widget.rs
  element.rs
  action.rs
  binding.rs
  theme.rs
  focus.rs
  semantics.rs
  invalidation.rs

zero-text-foundation
  font_database.rs
  font_request.rs
  font_fallback.rs
  shaping.rs
  bidi.rs
  line_break.rs
  grapheme.rs
  glyph_cache.rs
  glyph_atlas.rs
  text_measure.rs
  text_blob.rs
  diagnostics.rs

zero-ui-render
  scene.rs
  render_node.rs
  paint_ctx.rs
  layer.rs
  clip.rs
  hit_test.rs

zero-ui-runtime
  app.rs
  scheduler.rs
  tree.rs
  platform.rs
  theme_provider.rs
  i18n_provider.rs
  ime.rs
  accessibility.rs

zero-ui-widgets
  button.rs
  text_input.rs
  menu.rs
  popup.rs
  popover.rs
  list_view.rs
  badge.rs
  tooltip.rs
  tabs.rs
  toolbar.rs
  scrollbar.rs
  progress.rs

zero-ui-patterns
  search_field.rs
  suggestion_list.rs
  command_palette.rs
  data_list.rs
  status_bubble.rs
  tab_bar.rs
  dialog_scaffold.rs

zero-ui-i18n
  locale.rs
  catalog.rs
  message.rs
  formatter.rs
  fallback.rs
  plural.rs
  direction.rs
  diagnostics.rs

zero-ui-animation
  clock.rs
  controller.rs
  curve.rs
  tween.rs
  spring.rs
  transition.rs
  reduced_motion.rs

zero-ui-gestures
  pointer.rs
  arena.rs
  tap.rs
  drag.rs
  pan.rs
  pinch.rs
  fling.rs
  capture.rs

zero-ui-navigation
  route.rs
  navigator.rs
  modal.rs
  sheet.rs
  dialog.rs
  deep_link.rs
  restoration.rs

zero-ui-overlay
  overlay_host.rs
  portal.rs
  popover.rs
  tooltip.rs
  menu.rs
  dialog.rs
  sheet.rs
  toast.rs
  focus_trap.rs

zero-ui-collections
  lazy_list.rs
  lazy_grid.rs
  tree_view.rs
  table_view.rs
  selection.rs
  item_key.rs
  recycler.rs

zero-ui-commands
  command.rs
  shortcut.rs
  menu_model.rs
  command_palette.rs
  dispatcher.rs

zero-ui-forms
  form.rs
  field_state.rs
  validation.rs
  submit.rs

zero-ui-assets
  manifest.rs
  provider.rs
  icon.rs
  image.rs
  shader.rs
  variants.rs

zero-ui-platform
  clipboard.rs
  drag_drop.rs
  file_picker.rs
  notifications.rs
  haptics.rs
  system_menu.rs

zero-ui-restoration
  restoration_id.rs
  store.rs
  snapshot.rs
  route_state.rs
  widget_state.rs

zero-ui-testing
  test_runtime.rs
  fake_clock.rs
  scene_snapshot.rs
  semantics_snapshot.rs
  gesture_driver.rs

zero-ui-devtools
  inspector.rs
  layout_bounds.rs
  semantics_view.rs
  theme_preview.rs
  i18n_preview.rs
  timeline.rs

zero-ui-design-system
  density.rs
  motion_tokens.rs
  component_variants.rs
  state_layers.rs
  platform_style.rs

zero-ui-dsl
  yaml.rs
  schema.rs
  loader.rs
  validator.rs
  i18n.rs
  expression/
    ast.rs
    parser.rs
    typecheck.rs
    eval.rs
    functions.rs

zero-ui-adapter-webview
  webview_widget.rs
  scroll_bridge.rs

zero-browser-chrome
  tab_strip.rs
  address_bar.rs
  navigation_buttons.rs
  security_badge.rs
  site_info_panel.rs
  bookmarks_bar.rs
  find_bar.rs
  permission_prompt.rs
  download_panel.rs
  download_item_view.rs
  browser_menu.rs
  page_load_indicator.rs
  page_viewport.rs
```

#### 8.4.1A 浏览器组件接入映射

| 当前浏览器能力 / 位置 | 新归属 | 复用的通用组件 | 浏览器语义保留位置 | 接入思路 |
|---|---|---|---|---|
| 导航按钮（back/forward/reload/stop/home） | `browser-ui/chrome::NavigationButtons` | `IconButton`、`Toolbar`、`Tooltip` | `browser-shell` navigation state | `apps/browser` 将 can_go_back/can_go_forward/loading 映射为 props，点击发出 `BrowserAction::GoBack/GoForward/Reload/Stop/Home` |
| 地址栏 / omnibox | `browser-ui/chrome::AddressBar` | `TextInput`、`SearchField`、`Popover`、`SuggestionList`、`IconButton` | URL、搜索建议、安全状态 | 输入由 TextInput/IME 处理；提交发出 navigate/search action；建议列表由浏览器模型提供 |
| 站点安全标识 | `browser-ui/chrome::SecurityBadge` | `Badge`、`IconButton`、`Tooltip` | HTTPS、证书、mixed content、危险站点 | 组件只展示安全摘要；点击打开 `SiteInfoPanel` |
| 站点信息面板 | `browser-ui/chrome::SiteInfoPanel` | `Popover`、`DialogScaffold`、`ListView`、`Toggle`、`Button` | 权限、证书、站点设置 | 使用通用弹层绘制；权限变更发出 browser action |
| 标签栏 | `browser-ui/chrome::BrowserTabStrip` | `TabBar`、`IconButton`、`ProgressIndicator`、`Tooltip` | tab title、favicon、loading、crashed、active | browser-shell 提供 tab list；组件输出 activate/close/new/reorder action |
| 书签栏 | `browser-ui/chrome::BookmarksBar` | `Toolbar`、`Button`、`Menu`、`Popover` | bookmark tree、folder、URL | 通用 toolbar/menu 绘制；点击发出 navigate/open bookmark action |
| 查找栏 | `browser-ui/chrome::FindBar` | `TextInput`、`IconButton`、`StatusBubble` | 当前页面 find session | action 进入 WebView find API；结果计数作为 props 回流 |
| 下载面板 | `browser-ui/chrome::DownloadPanel` / `DownloadItemView` | `Popover`、`ListView`、`ProgressIndicator`、`Button`、`Menu` | download item、风险状态、文件动作 | 下载状态来自 browser-shell/download model；打开/取消/保留等通过 action dispatch |
| 浏览器主菜单 / 上下文菜单 | `browser-ui/chrome::BrowserMenu` | `Menu`、`ContextMenu`、`Separator`、`IconButton` | 浏览器命令、页面上下文 | 菜单项由 command model 生成；通用 Menu 负责布局、键盘导航和绘制 |
| 权限提示 | `browser-ui/chrome::PermissionPrompt` | `DialogScaffold`、`Popover`、`Button`、`Toggle` | geolocation/camera/mic/notification 等 Web 权限 | WebView/permission controller 产生请求；组件展示；用户选择发出 grant/deny action |
| 页面加载进度 | `browser-ui/chrome::PageLoadIndicator` | `ProgressIndicator` | navigation progress、loading state | 作为 toolbar 或 tab 内的领域组件，绘制仍走通用 ProgressIndicator |
| 页面视口框架 | `browser-ui/chrome::PageViewportFrame` | `ScrollBar`、`StatusBubble`、`WebViewWidget` | active WebView、page status、hover link | 布局外部矩形；WebViewWidget 绘制网页；ScrollBar 显示外层滚动反馈 |
| WebView 内容 | `ui/adapters/webview::WebViewWidget` | `ScrollBar`（外观） | DOM/CSS/layout/scroll offset/page lifecycle | WebView 自己渲染页面 scene/texture；UI SDK 只分配 viewport、路由输入、合成输出 |

所有 `browser-ui/chrome` 组件都必须实现为 UI SDK 组件或组件函数：输入是 props，输出是 Widget tree / Action；不得直接调用底层 GPU 绘制，也不得绕过 `ui-runtime` 的事件、焦点、IME 和无障碍管线。

#### 8.4.1B 浏览器接入完整 UI 能力矩阵

| 浏览器场景 | 接入模块 | 接入思路 | 验收 |
|---|---|---|---|
| 主菜单、上下文菜单、快捷键 | `ui/commands` + `ui/overlay` | 所有命令注册为 `BrowserCommandId`，菜单项、快捷键、命令面板只触发 command，再映射到 `BrowserAction` | 同一 reload command 可由菜单/快捷键/command palette 触发 |
| Omnibox 建议 | `ui/overlay` + `ui/collections` + `ui/animation` | 建议列表是 anchored popover；大量历史/搜索建议用 lazy collection；打开/关闭走 transition | 地址栏输入时 popover 定位、键盘选择、动画和虚拟化正常 |
| 权限提示 | `ui/navigation` + `ui/overlay` | desktop 用 anchored popover，phone 用 bottom sheet route；同一 `PermissionPrompt` props/action | grant/deny action 一致，focus trap 和 modal barrier 生效 |
| 下载面板 | `ui/overlay` + `ui/collections` + `ui/platform` | desktop popover，mobile sheet/page；下载 item 用 virtual list；打开文件走 platform service | 10000 条下载记录不卡顿，打开/显示文件走 mockable service |
| 书签/历史 | `ui/collections` + `ui/navigation` | 书签树用 TreeView，历史用 LazyList；设置/管理页是 route | selection、keyboard navigation、route restoration 正常 |
| Tab overview | `ui/collections` + `ui/gestures` + `ui/animation` + `ui/restoration` | desktop 可用 tab strip，phone/tablet 用 overview route；卡片列表 lazy 渲染，支持 swipe/drag | tab overview 旋转/恢复后保持 scroll offset 和 selection |
| 移动底部工具栏和 sheet | `ui/gestures` + `ui/animation` + `ui/overlay` | bottom sheet 可拖拽、fling dismiss，动画遵循 reduced motion | 拖拽阈值、fling、safe area、reduced motion 测试通过 |
| 页面下拉刷新/手势导航 | `ui/gestures` + `ui/adapters/webview` | 手势先进入 gesture arena，未被 chrome 消费时转发 WebView；平台 back gesture 映射 route pop 或 browser back | chrome/WebView 手势冲突可预测 |
| 设置页/偏好页 | `ui/forms` + `ui/navigation` + `ui/i18n` | 设置页是 route，字段用 Form/FieldState，文案来自 catalog | validation、dirty/touched、submit 和 locale 切换正常 |
| 拖拽 tab/link/download | `ui/gestures` + `ui/platform` | 组件产生 drag intent，runtime 通过 platform drag/drop service 执行 | 可 mock drag/drop，不污染 widgets |
| Session restore | `ui/restoration` | route stack、tab overview、scroll offset、地址栏 selection、窗口 metrics 使用 stable restoration id | 重启/恢复后 chrome UI 状态可恢复 |
| UI 调试和回归 | `ui/testing` + `ui/devtools` | browser chrome 使用 scene/semantics snapshot；开发时开启 inspector/layout bounds/perf timeline | CI 可跑 snapshot，开发可检查树和布局 |

#### 8.4.2 数据模型

```rust
pub struct WidgetId(pub String);

pub struct WidgetSpec {
    pub component: ComponentType,
    pub id: Option<WidgetId>,
    pub props: PropsMap,
    pub bindings: Vec<Binding>,
    pub actions: Vec<ActionBinding>,
    pub children: Vec<WidgetSpec>,
}

pub struct ElementState {
    pub id: WidgetId,
    pub focusable: bool,
    pub invalidation: InvalidationFlags,
}

pub struct RenderNode {
    pub id: WidgetId,
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub children: Vec<RenderNode>,
}
```

#### 8.4.3 事件分发

事件路由按以下优先级：

1. Pointer capture / gesture capture。
2. Popup/modal capture。
3. Focus route for keyboard/IME。
4. Hit-test route for pointer。
5. Bubble to parent。
6. App-level shortcut fallback。

WebViewWidget 特殊处理：

- 指针在滚动条上: ScrollBar 处理，发 ScrollCommand。
- 指针在网页内容上: 转换为 WebView 坐标，交给 zero-webview。
- WebView 未消费的快捷键: 冒泡给 browser chrome 或 app。

#### 8.4.4 布局设计

布局采用约束模型：

```text
constraints down
size up
position down
```

基础布局组件：

- `Row`
- `Column`
- `Stack`
- `Flex`
- `Padding`
- `Align`
- `SizedBox`
- `ScrollView`
- `Overlay`
- `Adaptive`
- `SafeArea`
- `KeyboardAvoider`
- `BreakpointSwitch`

响应式布局输入：

```text
WindowMetrics
  viewport size
  density
  text scale
  safe area
  soft keyboard rect
  orientation
  platform class
  input class
  viewport class
```

推荐断点：

| ViewportClass | 典型宽度 | 典型设备 | 布局倾向 |
|---|---|---|---|
| Compact | < 600dp | phone / narrow window | 单列、底部工具栏、全屏弹层 |
| Medium | 600-839dp | tablet / foldable / small desktop | 双栏、侧边弹层、简化标签 |
| Expanded | >= 840dp | desktop / large tablet | 顶部 toolbar、完整 tab strip、popover/menu |

首阶段可以用简单自研布局，不需要马上引入完整 CSS/Flexbox。若后续需要复杂布局，可评估复用 `taffy`，但不在第一阶段强制。

#### 8.4.4A 浏览器移动端 chrome 设计

浏览器接入 UI SDK 后，desktop 与 mobile 应复用同一业务模型，而不是复用同一视觉布局：

```text
BrowserChromeModel
  tabs
  active_tab
  navigation_state
  security_state
  permission_requests
  downloads
  find_state

BrowserAction
  Navigate
  GoBack / GoForward / Reload / Stop
  ActivateTab / CloseTab / NewTab
  GrantPermission / DenyPermission
  OpenDownload / CancelDownload

Adaptive shell
  Expanded + MouseKeyboard -> DesktopBrowserShell
  Medium + Touch/Hybrid     -> TabletBrowserShell
  Compact + Touch           -> PhoneBrowserShell
```

| 能力 | DesktopBrowserShell | TabletBrowserShell | PhoneBrowserShell |
|---|---|---|---|
| 地址栏 | 顶部 toolbar 中常驻 | 顶部常驻或焦点时展开 | 焦点时全屏/半屏搜索界面 |
| 标签页 | 完整 tab strip | 可滚动 tab strip 或 tab overview | tab switcher/overview，不常驻完整 strip |
| 导航按钮 | toolbar icon buttons | 简化 icon buttons | 底部或菜单内显示 |
| 菜单 | popover/context menu | popover 或 side sheet | bottom sheet / full-screen sheet |
| 下载 | popover panel | side/bottom sheet | bottom sheet / downloads page |
| 权限提示 | anchored popover | dialog/sheet | bottom sheet，避开 safe area |
| WebView | toolbar 下方 viewport | adaptive viewport | full height，避开 top/bottom chrome 和 keyboard |

接入原则：

- `BrowserChromeModel` 与 `BrowserAction` 由 desktop/tablet/phone shell 共享。
- `AddressBar`、`SecurityBadge`、`PermissionPrompt` 等领域组件共享逻辑和 props，但可提供不同 composition。
- `PhoneBrowserShell` 可以选择不显示完整 tab strip，但必须通过同一 `BrowserAction` 触发 tab overview。
- `SafeArea` 和 `KeyboardAvoider` 是移动端 shell 的默认外层布局。
- `WebViewWidget` 的 viewport 由 shell 重新计算；WebView 内部网页响应式仍由网页 CSS 自己处理。

DSL 示例：

```yaml
component: Adaptive
branches:
  compact:
    component: PhoneBrowserShell
  medium:
    component: TabletBrowserShell
  expanded:
    component: DesktopBrowserShell
```

#### 8.4.5 绘制与合成

UI 渲染阶段输出统一 scene：

```text
Widget paint
  -> SceneNode / RenderPrimitives
  -> ui-render compositor
  -> render-foundation CPU/GPU backend
```

WebViewWidget 输出可以是：

- `RenderPrimitives`
- `TextureNode`
- `ExternalSurfaceNode`
- `SceneNode`

首阶段优先用现有 `RenderPrimitives`，后续再支持 texture 或外部 surface。

#### 8.4.6 主题设计

组件只消费 semantic token：

```rust
pub struct ColorPalette {
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub danger: Color,
    pub warning: Color,
    pub success: Color,
    pub selection: Color,
    pub focus_ring: Color,
}
```

浏览器可派生专属 token：

```rust
pub struct BrowserChromeTheme {
    pub tab_active: Color,
    pub tab_inactive: Color,
    pub address_bar_background: Color,
    pub bookmark_bar_background: Color,
    pub page_frame_border: Color,
}
```

#### 8.4.7 YAML DSL 设计

YAML 是 UI 结构的声明式格式，表达式语言是 YAML 内部的受控计算层。二者共同生成 `WidgetSpec`：

- 允许: component、id、props、children、bindings、actions。
- 允许: `$state.path`、`$props.path`、`$theme.path`、`$env.path` 形式的路径读取。
- 允许: `visible_when`、`enabled_when`、`for_each`、属性绑定、样式绑定、action payload 中出现表达式。
- 允许: `Adaptive` / `branches` 按 `ViewportClass`、`PlatformClass`、`InputClass` 选择布局。
- 允许: 文本属性通过 `{ i18n: message.id, params: ... }` 引用独立国际化资源。
- 允许: 由 SDK 注册的纯函数，例如字符串、集合、数值和样式 token 计算。
- 禁止: 任意脚本、状态写入、未注册函数、递归、无限循环、系统 API、网络访问。
- 禁止: production strict mode 中直接硬编码用户可见字符串。

表达式语言采用四阶段管线：

1. Parse: 将 YAML 中的表达式字符串解析为 Expression AST。
2. Validate: 校验可访问路径、可调用纯函数、action payload schema 和禁止能力。
3. Typecheck: 根据 `BindingSchema` 推导表达式结果类型，拒绝类型不匹配。
4. Eval: 在只读 `EvalContext` 中求值，输出 `Value`。

表达式语法边界：

```text
expr        := conditional
conditional := logical_or ("?" expr ":" expr)?
logical_or  := logical_and ("||" logical_and)*
logical_and := equality ("&&" equality)*
equality    := compare (("==" | "!=") compare)*
compare     := term (("<" | "<=" | ">" | ">=") term)*
term        := factor (("+" | "-") factor)*
factor      := unary (("*" | "/" | "%") unary)*
unary       := ("!" | "-") unary | call
call        := primary ("." ident | "[" expr "]" | "(" args? ")")*
primary     := literal | path | array | object | "(" expr ")"
```

其中 function call 只允许调用 SDK 注册的 pure function registry。复杂业务逻辑仍由 Rust 宿主通过 Action 处理，表达式只负责 UI 声明层的选择、组合和展示计算。

#### 8.4.8 国际化资源设计

国际化采用移动端应用常见的独立资源文件模型：

```text
message id in Rust/YAML
  -> I18nProvider
  -> current locale catalog
  -> fallback locale catalog
  -> ResolvedText
  -> text measurement/layout/paint/semantics
```

资源文件是文案的唯一权威来源。Rust 组件和 YAML DSL 只保存 `MessageId` 与参数，不保存最终可见文案。这样可以：

- 让翻译文件独立维护和审核。
- 支持 locale fallback 与缺失 key diagnostic。
- 支持伪本地化测试，提前发现文本溢出。
- 支持 RTL direction 与图标镜像。
- 让浏览器文案与通用 UI SDK 解耦。

首阶段资源格式使用 YAML：

```yaml
locale: en
direction: ltr
messages:
  # 简写格式：无 plural/description 时可省略 value 键。
  browser.new_tab: "New Tab"
  # 完整格式：带 description 与 plural 变体。
  browser.n_bookmarks:
    value: "{count} bookmarks"
    description: "书签计数：{count} 为条数，支持 plural"
    plural:
      one: "{count} bookmark"
```

DSL 中使用对象形式引用：

```yaml
component: TextInput
props:
  placeholder:
    i18n: browser.address.placeholder

component: Text
props:
  text:
    i18n: browser.permission.camera.title
    params:
      origin: "$browser.active_origin"
```

Rust API 使用等价结构：

```rust
TextInput::new()
    .placeholder(LocalizedText::message("browser.address.placeholder"));
```

locale 切换流程：

1. `apps/browser` 接收系统 locale 或用户设置变化。
2. `ui-runtime` 更新 `I18nContext`。
3. 依赖 `LocalizedText` 的节点标记 `needs_layout`、`needs_paint`、`needs_semantics`。
4. `ui-render` 使用新文案重新测量和绘制。

RTL 规则：

- `direction: rtl` 的 locale 默认将 Row/Toolbar/TabBar 的起始方向切换为右侧。
- 文本默认对齐跟随 direction，显式 alignment 可覆盖。
- 图标默认不镜像；只有标记为 `mirror_in_rtl` 的图标参与镜像。
- WebView 内部网页 direction 仍由网页 DOM/CSS 处理，不由 UI SDK i18n 控制。

#### 8.4.9 共享文本与字体基础层设计

文本基础层是 `ui/render` 与 `zero-webview` 的共同依赖，目标是让字体发现、fallback、shaping、glyph cache 和 glyph atlas 只有一套实现。

```text
UI text path:
  ui/widgets::TextInput / Label / Button
    -> ui-render TextLayoutInput
    -> zero-text-foundation shape/measure/cache
    -> glyph primitives
    -> render-foundation backend

Web text path:
  DOM/CSS/style/layout
    -> WebView/engine 决定 CSS inline layout 和 line boxes
    -> zero-text-foundation shape/measure/cache
    -> glyph primitives
    -> render-foundation backend
```

边界原则：

- `zero-text-foundation` 只理解字体请求、Unicode 文本、locale、direction、script、字号、scale factor、font feature/variation 等基础输入。
- `ui/render` 负责 UI 文本布局，如控件 label、TextInput、菜单项、Tab title。
- `zero-webview` / `zero-engine` 负责网页文本布局，如 CSS inline formatting、line box、white-space、selection、caret painting。
- `render-foundation` 负责最终 CPU/GPU backend；glyph primitive 和 atlas entry 可以由文本基础层提供。

建议数据流：

```text
FontRequest
  -> FontProvider.query / fallback_chain
  -> ShapeInput
  -> TextShaper.shape
  -> ShapedText
  -> GlyphCache.get_or_insert
  -> GlyphAtlasEntry
  -> Scene glyph primitives
```

缓存策略：

- font database 按平台初始化，可由 runtime 提供系统字体目录或平台 API。
- glyph cache key 至少包含 font face、glyph id、size、scale factor、subpixel bucket、render mode。
- atlas 可按 backend 分层，CPU/GPU 资源生命周期由 render backend 管理。
- UI 与 WebView 可共享逻辑 cache；跨进程渲染时通过协议传递 glyph ids/atlas refs 或退化为 per-process cache。

与 i18n / RTL 的关系：

- `ui/i18n` 提供 locale 和 `TextDirection`。
- `zero-text-foundation` 消费 locale/direction 进行 fallback、bidi 和 shaping。
- WebView 内部网页仍以 DOM/CSS direction 为准，不使用应用 locale 覆盖网页内容。

迁移策略：

1. 先定义接口和测试，不立即替换所有现有文本渲染。
2. 让 `ui/widgets::TextInput` 和基础 Label/Button 率先接入。
3. 再让 `ui/adapters/webview` 明确 WebView 只在 shape/measure/raster 阶段接入。
4. 最后评估是否从 `crates/render-foundation` 中拆出现有字体能力到 `foundation/text`。

#### 8.4.10 完整 UI 能力协作模型

完整 UI SDK 首版不要求所有控件生态一次性实现，但必须让应用级 UI 能力在架构上协作闭合：

```text
Platform event
  -> ui-runtime input router
  -> ui-gestures pointer/gesture recognition
  -> focused widget / hit-tested widget
  -> ui-commands or widget action
  -> app state update
  -> ui-navigation / ui-overlay / ui-restoration update
  -> layout / animation / paint / semantics invalidation
  -> ui-render scene
```

浏览器接入时的推荐主线：

```text
browser-shell state
  -> BrowserChromeModel
  -> BrowserCommandModel
  -> BrowserChromeShell desktop/tablet/phone
  -> widgets/patterns/overlay/collections/navigation
  -> BrowserAction
  -> apps/browser dispatch
```

关键约束：

- `ui/commands` 是菜单、快捷键、命令面板、上下文菜单的统一入口。
- `ui/navigation` 管理 app UI route，不替代网页 navigation history。
- `ui/overlay` 管理 UI 浮层，不替代 WebView 内网页弹窗语义；网页弹窗请求可由 browser-ui 转为 browser overlay。
- `ui/collections` 管理 app UI 大列表，不参与 DOM 列表渲染。
- `ui/gestures` 先进行 chrome/WebView 手势仲裁，再决定消费或转发到 WebView。
- `ui/platform` 是平台能力唯一入口；widgets 不直接访问 clipboard/file picker/notification。
- `ui/restoration` 只恢复 app UI 状态；网页 session/history 仍由浏览器模型和 WebView 负责。
- `ui/devtools` 和 `ui/testing` 可以作为 dev/test feature gate，避免影响 release footprint。

### 8.5 安全考虑

- DSL 不执行任意脚本，表达式语言也不提供副作用能力，避免配置文件成为代码执行入口。
- ExpressionEngine 必须使用白名单 pure function registry；默认函数集不得包含文件、网络、进程、时间、随机数、线程、反射或 FFI。
- EvalContext 只暴露宿主显式传入的只读状态、props、theme 和 env snapshot。
- 表达式求值必须有最大 AST depth、最大节点数和最大 collection iteration 数，防止恶意 DSL 消耗过多 CPU。
- I18n catalog 只允许静态文案、参数 schema、plural forms 和 direction，不允许通过文案资源引用本地文件、网络 URL、脚本或富文本执行能力。
- 自定义主题文件只允许颜色、数值、字体 token，不允许引用本地路径加载任意资源，除非由宿主显式授权。
- WebViewWidget 与 UI SDK 间通过受控事件和 scene 输出连接，不暴露 DOM 内部可变引用给通用 UI。
- 外部应用注册 Action 时必须显式声明 ActionId。
- 未来插件化 DSL 加载必须考虑 sandbox 和签名校验。

### 8.6 替代方案

| 维度 | 方案 A: 仅抽 browser-ui | 方案 B: 独立 UI SDK + browser-ui | 方案 C: 直接引入现成 UI 框架 |
|---|---|---|---|
 实现复杂度 | 低 | 中 | 中到高 |
 浏览器迁移风险 | 低 | 中 | 高 |
 外部复用价值 | 低 | 高 | 中 |
 长期可维护性 | 中 | 高 | 取决于框架 |
 自绘控制力 | 高 | 高 | 取决于框架 |
 移动端潜力 | 低 | 中到高 | 取决于框架 |
 推荐度 | 中 | 高 | 低到中 |

**最终选择**: 方案 B。

**理由**:

1. 用户明确希望外部程序复用，而不是只整理浏览器代码。
2. 当前项目已有自绘基础和渲染管线，独立 SDK 能最大化复用现有资产。
3. 直接引入外部 UI 框架会削弱浏览器自绘和 WebView 深度集成能力。

### 8.7 实施计划

1. 文档确认和目录边界落地。
2. 最小 `ui/core` 类型和测试。
3. `ui/render` scene 输出与 render-foundation 对接。
4. `ui/widgets` 首批基础组件。
5. `ui/patterns` 通用组合模式。
6. `browser-ui/chrome` 按 §8.4.1A 迁移标签栏、地址栏、菜单、下载、权限等领域组件。
7. `apps/browser` 改为消费 browser-ui/chrome，只保留状态编排和 Action dispatch。
8. `ui/adapters/webview` 封装 WebViewWidget。
9. `ui/dsl` 支持 YAML 输入与完整表达式语言。
10. 补示例和产品验收。

### 8.8 测试策略

**单元测试**:

- WidgetSpec diff/mount/update。
- Invalidation flags。
- ThemeResolver。
- ScrollBar geometry/hit-test。
- Pattern composition tests。
- Browser chrome component props/action tests。
- I18n catalog load/resolve tests。
- I18n fallback/missing-key diagnostics tests。
- I18n params/plural tests。
- RTL direction and icon mirroring tests。
- Font provider query/fallback tests。
- Text shaping golden tests。
- Text measure and grapheme cursor boundary tests。
- Glyph cache key/reuse/eviction tests。
- WindowMetrics classification tests。
- Breakpoint/adaptive branch selection tests。
- SafeArea/KeyboardAvoider layout tests。
- Animation fake clock/tween/spring tests。
- Gesture arena tap/drag/pinch/fling tests。
- Navigation route stack/modal/sheet tests。
- Overlay focus trap/outside click/escape dismiss tests。
- Virtual collection materialization and stable key tests。
- Command registry/shortcut/menu model tests。
- Form validation/dirty/touched/submit lifecycle tests。
- Asset manifest variant resolution tests。
- Platform service mock tests。
- Restoration snapshot save/restore tests。
- Focus traversal。
- DSL parser/schema validation。
- Expression parser golden tests。
- Expression typecheck tests。
- Expression eval tests with deterministic `EvalContext`。
- Expression sandbox negative tests。

**集成测试**:

- WebViewWidget layout/paint/scroll。
- browser-ui/chrome 与 browser-shell 状态绑定。
- §8.4.1A 组件映射集成测试，确认领域组件组合通用 widgets/patterns。
- browser-ui/chrome scene snapshot，确认没有绕过 ui-render 绘制。
- DSL i18n message id 引用解析测试。
- locale 切换触发布局、绘制和 semantics 失效测试。
- ui-render 与 zero-webview 共用 text foundation 的 fallback/shaping 集成测试。
- WebView CSS inline layout 不进入 UI SDK layout 的边界测试。
- BrowserChromeModel 在 desktop/tablet/phone shell 之间复用的集成测试。
- PhoneBrowserShell WebView viewport 与 soft keyboard/safe area 集成测试。
- Browser menu/shortcut/context menu/command palette 同 command 集成测试。
- PermissionPrompt desktop popover 与 mobile sheet route 集成测试。
- Downloads/Bookmarks/History 使用 virtual collection 的集成测试。
- PlatformServices clipboard/file-picker/drag-drop mock integration tests。
- Restoration route/scroll/input selection integration tests。
- winit adapter event conversion。

**端到端测试**:

- 启动 `zero-browser`。
- 标签、地址栏、WebView、菜单、滚动条 smoke。
- light/dark/system/custom theme 切换。
- YAML 示例应用加载。
- YAML 示例应用通过表达式完成条件渲染、列表渲染、属性绑定和 action payload。
- YAML 示例应用通过独立 locale catalog 切换中英文文案。
- YAML 示例应用通过 Adaptive branches 切换 desktop/mobile layout。
- Browser shell demo 展示 command、overlay、navigation、collection、restoration 最小闭环。

**视觉验收**:

- 浏览器迁移前后关键 scene primitive 数量/几何快照。
- 典型窗口尺寸截图对比。
- 高 DPI / scale factor 检查。
- pseudo locale 文本膨胀检查。
- RTL locale 截图对比。
- UI 文本与 WebView 文本在同字体/字号下的 glyph alignment snapshot。
- desktop/tablet/phone browser chrome responsive screenshot。
- Animation reduced-motion visual snapshot。
- Overlay stacking/focus trap visual snapshot。

### 8.9 回滚计划

- 每个迁移阶段保持 `apps/browser` 可构建可运行。
- 首批组件迁移使用 feature flag 或 adapter shim 时，保留旧路径到阶段验收结束。
- 若某组件迁移导致回归，可仅回滚该组件的 browser-ui/chrome 使用，不影响 `ui/core` 基础。
- 不在一个提交中同时迁移所有浏览器 UI。

---

## 9. Spec Lint 报告

### 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
 包含执行摘要 | Pass | 第 0 节 |
 包含背景和目标 | Pass | 第 1 节 |
 包含 FR/NFR/IF | Pass | 第 3/4/5 节 |
 包含约束与假设 | Pass | 第 6 节 |
 包含里程碑和交接 | Pass | 第 7 节 |
 包含 RFC 技术设计 | Pass | 第 8 节 |
 包含测试和回滚 | Pass | 第 8.8/8.9 节 |

### 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
 区分需求和实现决策 | Pass | FR/NFR 与 RFC 分离 |
 区分通用 SDK 和浏览器专属 UI | Pass | `ui/` 与 `browser-ui/` 分离 |
 区分基础组件、组合模式和领域组件 | Pass | FR-009、§8.4.1、§8.4.1A 定义 `ui/widgets`、`ui/patterns`、`browser-ui/chrome` |
 明确 WebView 边界 | Pass | FR-005/IF-004/8.4.3 |
 明确 DSL 安全边界 | Pass | FR-008/IF-005/8.5 |
 表达式语言边界明确 | Pass | FR-008、IF-005、8.4.7、8.8 定义语法、类型检查、求值、安全和测试 |
 国际化资源边界明确 | Pass | FR-013、IF-007、8.4.8 定义独立资源文件、message id、fallback、参数、RTL 和 DSL 引用 |
 文本基础层边界明确 | Pass | FR-014、IF-008、8.4.9 定义共享字体/shaping/glyph cache 与 WebView CSS 排版边界 |
 响应式布局边界明确 | Pass | FR-015、IF-009、8.4.4、8.4.4A 定义 WindowMetrics、Adaptive shell、desktop/tablet/phone chrome |
 完整 UI 能力边界明确 | Pass | FR-016、IF-010、8.4.10 定义 animation、gestures、navigation、overlay、collections、commands、forms、assets、platform、restoration、testing、devtools |

### 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
目录结构与依赖约束一致 | Pass | 第 3、6、7 节一致 |
 早期范围表述已与 v1.6 对齐 | Pass | 第 0、1.3、3、4、7、8、9 节均包含完整 UI 能力设计与首批 skeleton 边界 |
 主题要求贯穿 FR/NFR/RFC | Pass | FR-007、IF-003、8.4.6 |
 浏览器迁移优先级明确 | Pass | FR-009、M2、实施计划 |
 浏览器组件接入路径闭合 | Pass | §8.2 定义 BrowserChromeModel 流程，§8.4.1A 定义组件映射，§8.8 定义集成和 scene snapshot 验证 |
 移动端为预留而非首阶段交付 | Pass | FR-010、范围边界 |
 表达式语言范围一致 | Pass | FR-008、IF-005、8.4.7、8.8 均将完整表达式语言列为 M3 必须交付 |
 国际化设计贯穿 FR/IF/RFC/测试 | Pass | FR-013、IF-007、6.5A、7、8.4.8、8.8 均覆盖 i18n 机制与验证 |
 文本基础层贯穿来源/交接/RFC/测试 | Pass | FR-014、IF-008、6.5A、7、8.4.9、8.8 均覆盖 text foundation 机制与验证 |
 移动端浏览器接入路径闭合 | Pass | FR-015、IF-009、7、8.4.4A、8.8 定义共享 BrowserChromeModel/BrowserAction 与 adaptive shell 验证 |
 完整 UI 能力浏览器接入闭合 | Pass | FR-016、IF-010、8.4.1B、8.4.10、8.8 定义 commands/overlay/collections/gestures/restoration 等浏览器接入和测试 |

**汇总**: 34 Pass / 0 Warning / 0 Fail / 0 Skip  
**门禁判定**: 允许确认并进入实施拆解。

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
 TBD-1 | 是否引入 serde_yaml | 重要 | 依赖策略未定 | M3 前评估 |
 TBD-2 | ui-render 与 render-foundation 的依赖方向 | 重要 | 是否直接依赖或抽 backend trait 未定 | M1/M2 设计时确认 |
 TBD-3 | 移动端首个目标平台 | 可选 | Android/iOS/HarmonyOS 优先级未定 | M4 前确认 |
 TBD-4 | DSL 是否需要 build-time codegen | 可选 | 表达式语言必须实现；codegen 仅影响性能和分发形态 | M3 后评估 |
 TBD-5 | 浏览器迁移是否使用 feature flag | 重要 | 取决于第一批组件迁移风险 | M2 前确认 |
 TBD-6 | 表达式 parser 是否引入 parser combinator crate | 重要 | 能力已定，依赖策略未定 | M3 前评估 |
 TBD-7 | i18n 是否引入 ICU4X/Fluent 类依赖 | 重要 | 机制已定，plural/formatting 依赖策略未定 | M1/M3 前评估 |
 TBD-8 | text foundation 最终落点 | 重要 | 可放 `foundation/text`，也可先从 `crates/render-foundation/text` 过渡 | M1 前确认 |
 TBD-9 | text shaping/font 依赖选择 | 重要 | 需评估 swash/rustybuzz/fontdb/unicode-bidi 等依赖或现有实现 | M1/M2 前评估 |
 TBD-10 | 移动端首个 chrome shell 验收目标 | 重要 | PhoneBrowserShell/TabletBrowserShell 的首批功能范围需结合目标平台确认 | M4 前确认 |
 TBD-11 | 完整 UI 能力首批实现深度 | 重要 | FR-016 已定接口边界，但每个模块首版实现深度需结合浏览器迁移节奏裁剪 | M1 task pack 前确认 |
 TBD-12 | design-system 首个风格包 | 可选 | 是否先做 Zero default，还是同时提供 Fluent/Cupertino/Material 风格包 | M2/M3 前评估 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
 v1.0 | 2026-06-30 | 初始版本，整合 UI SDK 抽取、WebViewWidget、主题、DSL、事件模型和浏览器迁移方案 |
 v1.1 | 2026-06-30 | 将完整 DSL 表达式语言纳入正式范围，补充 FR-008、IF-005、安全模型、M3 计划和测试策略 |
 v1.2 | 2026-06-30 | 增加 `ui/patterns` 分层，补充浏览器领域组件与通用 UI SDK 的接入映射和统一绘制管线 |
 v1.3 | 2026-06-30 | 增加国际化资源系统设计，采用独立 locale 文件和 DSL message id 引用，并补充 I18nProvider、fallback、plural、RTL 与测试策略 |
 v1.4 | 2026-06-30 | 增加共享文本/字体基础层设计，明确 UI SDK 与 WebView 共用 font fallback、shaping、glyph cache，同时保持 WebView CSS 排版边界 |
 v1.5 | 2026-06-30 | 增加响应式/自适应布局与移动端浏览器 chrome 方案，定义 WindowMetrics、Adaptive shell、desktop/tablet/phone shell 复用边界 |
 v1.6 | 2026-06-30 | 补齐首版完整 UI 能力架构，增加 animation、gestures、navigation、overlay、collections、commands、forms、assets、platform、restoration、testing、devtools、design-system，并完善浏览器接入矩阵 |
 v1.6.1 | 2026-06-30 | 全文自审修正：同步执行摘要、范围边界、FR-001/FR-002 验收、NFR、里程碑、代码边界和 lint，使其与完整 UI 能力设计保持一致 |
