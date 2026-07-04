# Spec: UI SDK 组件画廊（Component Gallery）

**版本**：v1.0
**日期**：2026-07-04
**作者**：AI Assistant
**状态**：草稿

---

## 0. 执行摘要

- **一句话目标**：构建一个独立浏览器的交互式组件画廊应用，展示 UI SDK 全部组件的视觉效果、交互行为及对应源码/DSL。
- **本期范围**：在 `ui/examples` 中新增 `gallery` 模块，实现左侧导航 + 右侧 Demo 区的两栏布局，覆盖所有 widget、pattern、form、gesture 等核心组件，支持主题切换和 i18n 切换，并展示每个组件的 Rust 代码/DSL 源码。
- **明确排除**：不接入真实浏览器；不覆盖 browser-ui/chrome 组件（因依赖 browser-shell）；不实现生产级源码编辑器（只做语法高亮展示）。
- **核心约束**：
  1. 必须保持 `ui/examples` 的浏览器无关性
  2. 所有组件演示必须是可交互的（不仅是静态渲染）
  3. 每个组件页必须展示对应源码（DSL YAML 或 Rust API）
- **推荐方案**：`WinitDriver` + 自定义 `GalleryApp`（UiApp impl）+ 按组件类别分页，左侧 `ListView` 导航，右侧分上下两区（组件预览 + 源码展示）。
- **首个落地步骤**：创建 `gallery/mod.rs` + `gallery/app.rs` 骨架，实现空壳 `GalleryApp` 和两栏布局，注册到 `WinitDriver` 可启动空窗。

---

## 1. 背景与目标

### 1.1 背景
UI SDK 现有 25 个 crate、14 个 widget、7 个 pattern、forms、gestures、animation、i18n 等能力，但缺乏一个统一的演示入口。现有的 `CounterApp`/`FormApp` 示例覆盖 <10% 的能力，浏览器 chrome 示例又耦合了 browser-shell，无法作为 SDK 的独立验收工具。

### 1.2 目标
- 提供一个可执行、可交互的组件画廊，用于 UI SDK 的视觉验收和功能演示
- 每个组件同时展示"运行效果"和"实现源码"，降低 SDK 使用者的学习成本
- 作为回归测试的可视化基线——新增组件或修改组件后，跑一次画廊即可目视检查

### 1.3 范围边界
- **在范围内**：
  - `ui/widgets` 所有组件（Button、IconButton、Toggle、TextInput、ListView、Menu、Toolbar、Tabs、ScrollBar、Badge、Tooltip、Popover、Popup、Progress、Checkbox）
  - `ui/patterns` 所有模式（SearchField、SuggestionList、CommandPalette、DataList、StatusBubble、TabBar、DialogScaffold）
  - `ui/forms` 表单演示（FieldState、验证器、FormState）
  - `ui/collections` 虚拟列表演示（LazyList + VirtualCollection）
  - `ui/gestures` 手势演示（Tap/Pan/Pinch）
  - `ui/animation` 动画演示（Tween/Spring）
  - `ui/navigation` 路由/overlay 演示
  - 主题切换（light/dark）
  - i18n 语言切换（en/zh 演示）
  - 每个组件的 DSL YAML 和 Rust API 源码展示
- **不在范围内**：
  - browser-ui/chrome 组件（AddressBar、NavigationButtons 等）：因依赖 `zero-browser-shell`
  - 真实浏览器网页渲染
  - 在线 IDE/代码编辑器（源码展示只做语法高亮和格式化显示）
  - 组件编辑/属性面板（非本期目标）
  - 自动化测试（后续可补充 snapshot 测试）

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | 组件展示/验收/学习 |
| 用户需求 | 是 | 用户交互操作 |
| 解决方案需求 | 是 | UI SDK 组合展示 |
| 功能需求 | 是 | 本文档第3节 |
| 非功能需求 | 是 | 本文档第4节 |
| 接口需求 | 是 | 本文档第5节 |
| 过渡需求 | 否 | 无迁移需求 |

---

## 3. 功能需求

### FR-001：两栏布局与导航
- **描述**：画廊必须提供左侧导航列表 + 右侧内容区的两栏布局。导航列表按类别分组（Widgets、Patterns、Forms、Gestures、Animation、Theme、i18n），每组可展开/折叠。
- **优先级**：必须
- **来源**：hmos 代码工坊 / 统一应用软件 风格

**验收场景**：

```
场景: 启动画廊
  假设 应用启动
  当 窗口打开
  那么 左侧显示导航列表，右侧显示第一个组件的 Demo 页
  验证: cargo run -p zero-ui-examples --example gallery 观察布局

场景: 导航切换
  假设 左侧导航列表可见
  当 点击导航中的 "Button" 项
  那么 右侧切换到 Button 组件的 Demo 页
  验证: 观察右侧内容变化
```

### FR-002：组件演示页
- **描述**：每个组件在一个 Demo 页中同时展示：
  1. 组件名称和简短描述
  2. 交互式渲染效果（组件可操作）
  3. 实现源码（DSL YAML + Rust API 片段）
- **优先级**：必须
- **来源**：需求要求

**验收场景**：

```
场景: 组件页展示
  假设 已选中 Button 组件
  那么 右侧显示 Button 预览（可点击的 Button widget）
  并且 预览下方显示对应的 YAML DSL 源码和 Rust API 源码片段
  验证: 观察页面内容

场景: 交互操作
  假设 Button 组件预览可见
  当 点击 Button
  那么 Button 应有 pressed/hover 视觉反馈
  验证: 观察按钮视觉变化
```

### FR-003：主题切换
- **描述**：画廊顶部/Header 区域提供 light/dark 主题切换开关。切换后所有组件预览实时刷新。
- **优先级**：必须
- **来源**：验收场景需要

**验收场景**：

```
场景: 主题切换
  假设 画廊在 light 主题下
  当 点击主题切换按钮
  那么 整体 UI 切换为 dark 主题，所有组件预览适配 dark 色板
  验证: 观察背景/文字/按钮颜色变化
```

### FR-004：i18n 语言切换
- **描述**：画廊提供中/英语言切换。切换后导航文案、组件描述文案同步切换。
- **优先级**：应该
- **来源**：验收场景需要

**验收场景**：

```
场景: 语言切换
  假设 画廊当前为中文
  当 点击语言切换开关
  那么 导航列表和组件描述切换为英文
  验证: 观察文案变化
```

### FR-005：组件覆盖清单
- **描述**：画廊必须覆盖以下核心组件（每个一个独立 Demo 页）：

| 类别 | 组件 |
|------|------|
| Widgets | Button、IconButton、Toggle、TextInput、ListView（含 VirtualCollection）、Menu、Tabs、Toolbar、ScrollBar、Progress、Badge、Tooltip、Popover、Popup |
| Patterns | SearchField、SuggestionList、CommandPalette、DataList、StatusBubble、TabBar、DialogScaffold |
| Forms | FieldState + 验证器 + FormState 提交演示 |
| Gestures | Tap/Pan/Pinch 交互演示 |
| Animation | Tween + Spring 动画演示 |
| Collections | LazyList 滚动 + Recycler 虚拟化 |
| Theme | light/dark 切换 + 主题 token 展示 |
| i18n | locale 切换 + catalog 加载演示 |
| DSL | 表达式引擎 + responsive branch 演示 |
| Navigation | Route 推栈/Overlay 演示 |

- **优先级**：必须
- **来源**：需求中"所有组件"

**验收场景**：

```
场景: 完整覆盖
  假设 画廊启动
  当 浏览左侧导航列表
  那么 上述每个组件都有对应的 Demo 页
  验证: 逐一点击导航项确认

场景: 空状态/禁用状态展示
  假设 选中 ListView 或 Menu
  那么 展示空列表状态和禁用项状态
  验证: 观察空状态 UI
```

### FR-006：源码展示
- **描述**：每个 Demo 页的下方展示对应组件的实现源码，包含：
  - DSL YAML 片段（展示如何用 YAML 声明该组件）
  - Rust API 片段（展示如何用 Rust API 构造该组件）
  - 简单的语法高亮（颜色标注关键字/字符串/注释）
- **优先级**：应该
- **来源**：需求要求

**验收场景**：

```
场景: DSL 代码展示
  假设 选中 Button 组件
  那么 组件预览下方展示对应 YAML 源代码
  并且 代码中 component、props、id 等关键字有颜色区分
  验证: 观察代码显示区域

场景: Rust API 代码展示
  假设 选中 TextInput 组件
  那么 预览下方同时展示 Rust API 构造代码
  验证: 观察代码显示区域
```

### FR-007：导航分组与可搜索
- **描述**：导航列表按组件类别分组（Widgets / Patterns / Forms / Gestures / Animation / Theme / i18n / DSL / Navigation），每组可展开折叠。导航列表顶部提供搜索框，过滤组件名。
- **优先级**：应该
- **来源**：类 Storybook 交互

**验收场景**：

```
场景: 分组折叠
  假设 左侧导航列表展开
  当 点击分组标题 "Widgets"
  那么 该组折叠，下属组件隐藏
  验证: 观察导航变化

场景: 搜索过滤
  假设 导航列表显示所有组件
  当 在搜索框输入 "but"
  那么 导航列表只保留匹配项（Button）
  验证: 观察过滤结果
```

---

## 4. 非功能需求

### NFR-001：启动时间
- **描述**：画廊必须在 2 秒内启动到可交互状态。
- **测量标准**：`cargo run -p zero-ui-examples --example gallery` 按下回车到窗口显示的时间
- **优先级**：应该

### NFR-002：独立可执行
- **描述**：画廊必须作为独立 binary example 运行，不依赖任何其他服务或数据文件。
- **测量标准**：`cargo run -p zero-ui-examples --example gallery` 在无外部服务时正常启动
- **优先级**：必须

### NFR-003：浏览器零依赖
- **描述**：`ui/examples/Cargo.toml` 不得依赖 `zero-browser-shell`、`zero-webview`、`zero-engine`、`zero-net` 中的任何一个。
- **测量标准**：`cargo check -p zero-ui-examples` 编译通过，无浏览器 crate 依赖
- **优先级**：必须

### NFR-004：代码组织
- **描述**：画廊代码放在 `ui/examples/src/gallery/` 目录下，每个组件 Demo 页一个独立文件。
- **测量标准**：查看 `ui/examples/src/gallery/` 目录结构
- **优先级**：应该

---

## 5. 接口需求

### IF-001：GalleryApp（UiApp 实现）
- **类型**：API
- **规格**：
  ```rust
  pub struct GalleryApp {
      current_page: PageId,          // 当前选中页
      theme: ThemeKind,              // Light | Dark
      locale: LocaleId,              // 当前语言
      search_query: String,          // 导航搜索
      collapsed_groups: HashSet<GroupId>, // 折叠的分组
      pages: Vec<DemoPage>,          // 所有 Demo 页配置
  }

  impl GalleryApp {
      pub fn new() -> Self;
      fn build_sidebar(&self) -> WidgetSpec;
      fn build_demo_area(&self) -> WidgetSpec;
      fn build_source_panel(&self, page: &DemoPage) -> WidgetSpec;
  }

  impl UiApp for GalleryApp {
      fn root_spec(&self) -> WidgetSpec;
      fn dispatch(&mut self, action: &EmittedAction) -> ActionResult;
  }
  ```
- **错误处理**：未知 PageId → 回落默认页；未知 ActionId → UnknownAction

### IF-002：DemoPage 数据模型
- **类型**：API
- **规格**：
  ```rust
  pub struct DemoPage {
      pub id: PageId,
      pub group: GroupId,
      pub title: &'static str,
      pub description: &'static str,
      pub build_preview: fn(&DemoContext) -> WidgetSpec,  // 生成预览 widget
      pub source_dsl: &'static str,      // DSL YAML 源
      pub source_rust: &'static str,     // Rust API 源
      pub source_lang: SourceLang,       // DSL | Rust
  }

  pub enum GroupId {
      Widgets, Patterns, Forms, Gestures,
      Animation, Collections, Theme, I18n, Dsl, Navigation,
  }

  pub struct DemoContext {
      pub theme: SemanticTokens,
      pub locale: LocaleId,
  }
  ```

### IF-003：源码语法高亮
- **类型**：UI
- **规格**：DSL YAML 和 Rust 代码展示使用简单的基于颜色 token 的语法高亮（关键字蓝色、字符串绿色、注释灰色）。不引入外部语法高亮 crate，仓内自实现一个最小 `highlight_yaml()` / `highlight_rust()` 函数，返回 `Vec<(TextSpan, Color)>`。
- **错误处理**：无法解析的代码 → 纯白色文本显示（不报错）

---

## 6. 约束与假设

### 6.1 必须约束（Must）
1. `ui/examples/Cargo.toml` 不得新增对任何浏览器 crate 的依赖
2. 所有组件 Demo 必须使用真实 widget 实现（非 mock）
3. 画廊必须可脱离浏览器独立运行

### 6.2 禁止约束（Must Not）
1. 不得依赖 `syntect`/`tree-sitter` 等重量级语法高亮库（仓内自实现最小高亮）
2. 不得修改 `ui/widgets`、`ui/patterns`、`ui/core` 等现有 crate 的代码（只新增 `ui/examples` 内代码）

### 6.3 已定决策
1. 使用 `WinitDriver` 驱动画廊（与现有 CounterApp/FormApp 一致的模式）
2. 源码展示为静态字符串常量嵌入（每个 DemoPage 的 `source_dsl`/`source_rust` 字段）
3. 布局用 `Flex`（`zero-ui-core` 的布局约束）+ 自定义 widget 组合实现两栏

### 6.4 技术约束
1. 由于 UI SDK 目前只有 Button 有完整 Widget trait 实现，其他组件为数据模型，Demo 预览区需要为每个组件创建 Widget+factory 注册（在 `register_gallery_factories()` 中统一注册）

### 6.5 假设
- `WinitDriver` 提供的事件循环足够驱动所有交互演示 — 状态：已验证（现有 counter/form example 已证明）
- 所有 widget 的 props/state 可以通过 `WidgetSpec` 属性驱动 — 状态：已验证（button、toggle、etc.）

### 6.6 代码变更边界
- **允许修改**：`ui/examples/Cargo.toml`、`ui/examples/src/` 全部新增
- **禁止修改**：`ui/widgets/`、`ui/patterns/`、`ui/core/`、`ui/collections/`、`ui/i18n/`、`ui/runtime/`、`ui/render/`、`ui/gestures/`、`ui/animation/`、`ui/forms/`、`ui/navigation/`、`ui/overlay/`、`ui/dsl/` 等现有 crate

### 6.7 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| 两栏布局 | 现有 widget 组合 | `Flex` + 自定义 `GalleryFrame` widget | 用约束实现左右分栏 |
| 语法高亮 | 仓内自实现 | `gallery/highlight.rs` 最小高亮函数 | 只处理关键字/字符串/注释颜色标记 |
| Demo 页数据 | 仓内自实现 | `gallery/pages/` 每个组件一个 .rs 文件 | 静态 DSL/Rust 源码字符串常量 |
| 组件预览 | 现有 widget + factory | `register_gallery_factories()` | 为每个组件注册预览 widget |
| 导航搜索 | 现有 pattern | `SearchField` + 过滤逻辑 | 复用 `ui/patterns` |


