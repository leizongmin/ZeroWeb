# RFC: UI SDK 组件画廊 — 技术设计

**版本**：v1.0
**日期**：2026-07-04
**对应 Spec**：`ui-sdk-gallery-spec.md`

---

## 0. 执行摘要

本 RFC 在 Spec 确认的基础上，定义组件画廊的具体模块结构、数据模型、布局策略、关键实现细节和分步实施计划。

**核心思路**：在 `ui/examples` 中新增 `gallery/` 模块，用 `WinitDriver` + `GalleryApp`（实现 `UiApp` trait）驱动；每个组件用 `DemoPage` 结构体描述（preview 构建函数指针 + 源码字符串常量）；左侧导航用 `ListView`，右侧内容区用 `Flex` 分上下两区（预览 + 源码展示）。

---

## 1. 模块树与职责

```
ui/examples/src/
├── lib.rs                 # 追加 pub mod gallery;
├── gallery/
│   ├── mod.rs             # 模块声明 + register_gallery_factories()
│   ├── app.rs             # GalleryApp struct + UiApp impl（主逻辑）
│   ├── model.rs           # DemoPage, PageId, GroupId, DemoContext 定义
│   ├── layout.rs          # GalleryFrame widget（两栏布局）
│   ├── highlight.rs       # highlight_yaml() / highlight_rust() 语法高亮
│   ├── source_display.rs  # SourcePanel widget（代码展示区域）
│   └── pages/
│       ├── mod.rs         # ALL_PAGES 常量定义
│       ├── button.rs      # Button demo
│       ├── toggle.rs      # Toggle demo
│       ├── text_input.rs  # TextInput demo
│       ├── ...            # 每个组件一个文件
│       ├── gesture.rs     # Gestures demo
│       ├── animation.rs   # Animation demo
│       ├── theme.rs       # Theme demo
│       ├── i18n.rs        # i18n demo
│       ├── dsl.rs         # DSL demo
│       └── navigation.rs  # Navigation demo
```

所有 widget factory 注册集中在 `gallery/mod.rs` 的 `register_gallery_factories()` 中。

---

## 2. 核心数据模型

### 2.1 GalleryApp 状态

```rust
pub struct GalleryApp {
    pub current_page: PageId,           // 当前选中页
    pub theme: ThemeKind,               // Light | Dark
    pub locale: LocaleId,               // 当前语言 (en | zh)
    pub search_query: String,           // 导航搜索框内容
    pub collapsed_groups: HashSet<GroupId>,
    pub text_input_value: String,       // TextInput demo 状态
    pub toggle_values: HashMap<String, bool>,
    pub list_selection: Option<usize>,
    pub demo_states: HashMap<PageId, Box<dyn Any>>, // 各 demo 页独立状态
}
```

- **AppState 原则**：所有状态扁平存储在 `GalleryApp` 字段上（而非嵌套结构体）。跨 demo 共享的（theme, locale）直接字段；demo 独立的通过 `demo_states` 泛型容器。
- **Action 映射**：`dispatch()` 根据 `action.id` 的前缀路由到对应 handler。例如 `"gallery.nav.select"` → 更新 `current_page`、`"gallery.theme.toggle"` → 切换 `theme`。

### 2.2 DemoPage 实例

```rust
pub struct DemoPage {
    pub id: PageId,
    pub group: GroupId,
    pub title: &'static str,         // 如 "Button"
    pub title_zh: &'static str,      // 如 "按钮"
    pub description: &'static str,
    pub description_zh: &'static str,
    pub build_preview: fn(&DemoContext, &GalleryApp) -> WidgetSpec,
    pub source_dsl: &'static str,    // DSL YAML 源码
    pub source_rust: &'static str,   // Rust API 源码
}
```

### 2.3 布局约束策略

```
┌─────────────────────────────────────────────┐
│  Header (height: 48px)                       │
│  [Gallery Title]               [🌙] [EN|中文]│
├──────────────┬──────────────────────────────┤
│  NavSidebar  │  DemoArea                     │
│  (width: 220)│  ┌─────────────────────┐     │
│  [🔍 search] │  │  Component Preview   │     │
│              │  │  (flex: 1)           │     │
│  ▸ Widgets   │  └─────────────────────┘     │
│    • Button  │  ┌─────────────────────┐     │
│    • Toggle  │  │  Source Code Panel   │     │
│    • ...     │  │  (height: 250px)     │     │
│  ▸ Patterns  │  └─────────────────────┘     │
│    • ...     │                               │
│  ▸ Forms     │                               │
└──────────────┴──────────────────────────────┘
```

布局实现策略：

| 区域 | 实现方式 | 说明 |
|------|----------|------|
| 顶部 Header | 自定义 `GalleryHeader` widget | Flex row, 固定高度 |
| 左侧导航 | 自定义 `NavSidebar` widget | Flex column: SearchField + ListView(分组嵌套) |
| 右侧 Demo 区 | 自定义 `DemoArea` widget | Flex column: preview(弹性) + source(固定高度250) |
| 整体布局 | `GalleryFrame` widget | Flex row: header + body(Flex row: sidebar + demo) |

---

## 3. 关键实现细节

### 3.1 语法高亮（highlight.rs）

```rust
pub enum HighlightToken {
    Text(&'static str, Color),
    Keyword(&'static str),    // → 蓝色
    String(&'static str),     // → 绿色
    Comment(&'static str),    // → 灰色
    Number(&'static str),     // → 橙色
}

pub fn highlight_yaml(src: &'static str) -> Vec<HighlightToken>;
pub fn highlight_rust(src: &'static str) -> Vec<HighlightToken>;
```

- 纯字符串扫描，不做 AST 解析
- YAML 高亮规则：`key:` 标记为 Keyword，`"..."` 标记为 String，`#` 开头标记为 Comment
- Rust 高亮规则：`fn`/`let`/`pub`/`struct`/`impl` 等关键字 + `"..."` 字符串 + `//` 注释
- 语法高亮结果渲染到 `RichText` widget（支持片段着色）

### 3.2 DSL / Rust 源码嵌入

每段源码在对应 `pages/*.rs` 文件中以 `const` 嵌入：

```rust
// pages/button.rs
pub const BUTTON_DSL_SOURCE: &str = r#"
Button:
  id: my_button
  props:
    label: "Click me"
    on_press:
      action: "button.pressed"
"#;

pub const BUTTON_RUST_SOURCE: &str = r#"
let btn = WidgetSpec::new("Button".into(), "my_button".into())
    .with_prop("label", Value::Str("Click me"))
    .with_prop("on_press", Value::Action("button.pressed"));
"#;
```

### 3.3 组件预览构建

每个 `build_preview` 函数使用 WidgetSpec 构建真实 widget：

```rust
fn preview_button(ctx: &DemoContext, state: &GalleryApp) -> WidgetSpec {
    WidgetSpec::new("Button".into(), "gallery_preview_btn".into())
        .with_prop("label", Value::Str(match ctx.locale {
            LocaleId::En => "Click me",
            LocaleId::Zh => "点击我",
        }))
        .with_prop("theme", Value::Theme(ctx.theme))
}
```

### 3.4 主题切换流程

1. 用户点击 Header 的 `🌙` 按钮 → 发出 `Action { id: "gallery.theme.toggle" }`
2. `GalleryApp::dispatch()` 收到 action → 翻转 `self.theme` → `ActionEffect::Rebuild`
3. `WinitDriver` 收到 Rebuild → 调用 `GalleryApp::root_spec()` → 渲染新 spec
4. 所有组件预览通过 `DemoContext.theme` 统一获取主题色板

### 3.5 i18n 切换流程

1. 用户点击 Header 的 `EN|中文` 按钮 → 发出 `Action { id: "gallery.locale.toggle" }`
2. `GalleryApp::dispatch()` 翻转 `self.locale` → `ActionEffect::Rebuild`
3. 所有 Demo 页的 title/description 根据 locale 选择 `title`/`title_zh` 字段
4. 导航分组名称、组件描述也根据 locale 切换

---

## 4. 组件 Demo 页完整清单

每页需实现：`build_preview()` + `source_dsl` + `source_rust`。

| 类别 | Demo 页 | 预览内容 | 关键状态 |
|------|---------|---------|----------|
| Widgets | Button | 2-3 个不同样式的按钮，点击交互 | pressed state |
| Widgets | Toggle | 开/关/禁用三种状态 toggle | toggle_values |
| Widgets | TextInput | 输入框 + 占位符 + 输入交互 | text_input_value |
| Widgets | IconButton | 不同图标的图标按钮 | pressed |
| Widgets | Progress | 不确定进度条 + 定值进度条 | progress_value |
| Widgets | Badge | 带角标的按钮/图标 | count demo |
| Widgets | Tooltip | 悬停提示演示 | tooltip_visible |
| Widgets | Menu | 下拉菜单 | menu_open |
| Widgets | Tabs | 多标签切换 | selected_tab |
| Widgets | Toolbar | 工具栏布局 | - |
| Widgets | Popover | 弹出面板 | popover_open |
| Widgets | Popup | 模态弹窗 | popup_open |
| Widgets | ListView | 滚动列表 + 选中 | list_selection |
| Patterns | SearchField | 搜索 + suggestion 联动 | query, results |
| Patterns | DataList | 数据列表 + 排序 | sort_column |
| Patterns | CommandPalette | 命令面板 | palette_open |
| Patterns | StatusBubble | 状态指示器 | status enum |
| Patterns | TabBar | Tab 导航 | selected |
| Patterns | DialogScaffold | 确认对话框 | dialog_open |
| Forms | FormDemo | 表单字段 + 验证 + 提交 | form_state, errors |
| Gestures | GestureDemo | Tap/Pan/Pinch 区域 | gesture log |
| Animation | AnimationDemo | Tween + Spring 动画 | anim_progress |
| Collections | CollectionDemo | LazyList 虚拟滚动 | scroll_offset |
| Theme | ThemeDemo | light/dark 对比 + token 列表 | theme |
| i18n | I18nDemo | locale 切换 + 命名翻译 | locale |
| DSL | DslDemo | 表达式计算 + responsive | expr_input |
| Navigation | NavDemo | 页面堆栈 + overlay | nav_stack |

---

## 5. Cargo.toml 变更

```toml
# ui/examples/Cargo.toml — 追加依赖
[dependencies]
# ... 现有依赖保持不变 ...
zero-ui-patterns = { workspace = true }
zero-ui-collections = { workspace = true }
zero-ui-forms = { workspace = true }
zero-ui-commands = { workspace = true }
zero-ui-overlay = { workspace = true }
zero-ui-navigation = { workspace = true }
zero-ui-gestures = { workspace = true }
zero-ui-animation = { workspace = true }
zero-ui-i18n = { workspace = true }
zero-ui-dsl = { workspace = true }
zero-ui-text-foundation = { workspace = true }
```

需确认 `zero-ui-i18n` 和 `zero-ui-text-foundation` 是有效的 workspace member——之前的任务已将 `zero-text-foundation` 重命名为 `zero-ui-text-foundation` 并纳入 workspace。

---

## 6. 实施计划

### Phase 1：骨架搭建（1 步 = 1 commit）

```
Step 1: 创建 gallery/ 骨架
  文件：mod.rs + app.rs + model.rs + layout.rs
  输出：GalleryApp 空壳，空窗口可启动
  验证：cargo run -p zero-ui-examples --example gallery 显示空窗口

Step 2: 两栏布局
  文件：layout.rs GalleryFrame widget
  输出：Header + 左侧空导航 + 右侧空 Demo 区
  验证：窗口显示两栏结构

Step 3: 导航列表
  文件：model.rs ALL_PAGES + 导航 ListView
  输出：左侧显示分组列表，点击可切换 current_page
  验证：点击导航项，current_page 变化
```

### Phase 2：核心集成（4-6 步）

```
Step 4: 组件预览框架
  文件：DemoArea widget + pages/ 目录结构
  输出：右侧根据 current_page 渲染对应的 build_preview()
  验证：点击导航 → 右侧显示对应预览

Step 5: 主题切换
  文件：Header 主题按钮 + dispatch handler
  输出：点击 🌙 → light/dark 切换
  验证：整体色板变化

Step 6: i18n 切换
  文件：Header 语言按钮 + dispatch handler
  输出：点击 EN/中文 → 导航/描述切换语言

Step 7-10: 逐个实现 Demo 页 + 源码
  每组 1-2 页，按优先级：Widgets → Patterns → Forms → 剩余
```

### Phase 3：源码展示（2 步）

```
Step 11: highlight.rs 语法高亮
  输出：highlight_yaml() / highlight_rust() 函数

Step 12: SourcePanel widget
  输出：每个 Demo 页下方显示高亮源码
```

---

## 7. 关键风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 某些 widget 缺少 Widget trait impl | 预览无法渲染 | 在 gallery factory 中补充最小实现，不改动原 crate |
| WinitDriver 不支持嵌套 ListView 分组展开 | 导航无法折叠 | 退化为平面列表，用前缀缩进表示层级 |
| 语法高亮对长代码段性能不足 | 帧率下降 | 源码展示区只渲染可见行，用 ScrollBar 滚动 |
| DSL YAML 格式与 widget props 不 1:1 对应 | 源码和实际渲染不一致 | 源码写为"概念示例"，注释标注"简化版" |

---

## 8. 未定项与后续

| 项 | 状态 | 处理方式 |
|----|------|----------|
| 搜索过滤导航列表 | ✅ 已实现 | `GalleryApp::filtered_pages` 按 `search_query` 缩小返回集；dispatch 收到 `gallery.search` 更新 query（见 `search_query_filters_pages` 测试） |
| 导航分组折叠 | ✅ 已实现 | `collapsed_groups: HashSet<GroupId>` + `gallery.group.toggle` action；sidebar 按 `is_collapsed` 折叠组内子项（见 `group_toggle_collapses_pages` 测试） |
| 统一的 dark/light 色板 | ✅ 已有 | `zero-ui-core` 的 `SemanticTokens` 已提供 |

---

## 9. 架构优化记录（2026-07-05）

第一版 gallery 落地后，针对 ui-sdk 设计层面做了一轮优先级排序的架构优化。
全部已实现并通过测试。

### 9.1 P0 级（影响正确性）

#### P0-1 reconcile key 化（`WidgetId` 复用）

**问题**：早期 `reconcile_children` 按下标匹配新旧子节点，列表头插/删导致后续节点全部重建，
内部状态丢失（toggle、input focus 等）。

**修复**：`reconcile::reconcile_children` 改用 `HashMap<WidgetId, usize>` 做键映射，
新 spec 按稳定 `WidgetId` 在旧 children 中查找复用 `HostNode`（保留 `creation_epoch`
和内部状态）；不匹配的旧节点丢弃，新 spec 构造新节点。配套：
- `reconcile_node` 不再无脑 `NEEDS_LAYOUT`，改为聚合 `Widget::update` 上报的 invalidation
  + 永远 `NEEDS_PAINT`，让控件自己决定 layout 是否需要。
- chrome widget 引入 `mark_layout_if_changed` / `mark_paint_if_changed` helper 区分两类失效。

**测试**：`reconcile_reuses_by_widget_id_across_position` /
`reconcile_reuses_by_widget_id_on_removal` 验证头插/头删后尾部 keyed 节点的
`creation_epoch` 不变。

#### P0-2 `host.rs` 拆模块

**问题**：`host.rs` 单文件逼近 2000 行上限，难维护。

**修复**：按职责拆为 5 个 `pub(super)` 子模块（host.rs 只保留公开 API + 编排）：
- `host/reconcile.rs`：声明树 → HostNode 树构建与 reconcile
- `host/layout.rs`：measure / arrange 两遍
- `host/paint.rs`：SceneRecorder 遍历
- `host/event.rs`：命中 / 派发 / 焦点 / 滚动
- `host/semantics.rs`：a11y 树构建

### 9.2 P1 级（架构清晰度）

#### P1-6 主题单源（`PaintCtx.tokens` 注入）

**问题**：主题色此前有两条并行路径——host 持有 tokens（paint 时注入 PaintCtx）+ chrome widget
自己存 `theme: ThemeKind` 字段在 paint 中 `tokens_for(self.theme)`。两条路径靠每个 widget
update 里 `sync_theme(props, &mut self.theme)` 同步，易漏写或漂移。

**修复**：
- `UiApp` trait 加可选方法 `theme_tokens(&self) -> Option<SemanticTokens>`（默认 None，向后兼容）。
- `WinitDriver::begin` / `pump_event`（handled 后）调 `app.theme_tokens()`，Some 则 `host.set_tokens`。
- `GalleryApp` 实现 `theme_tokens`，按 `self.theme` 映射 light/dark。
- 删除 `HeaderTitle` / `HeaderButton` / `NavItem` / `NavSearch` / `GroupHeader` / `DemoTitle` /
  `DemoPreview` / `SourceLabel` / `SourceCode` 的 `theme` 字段；paint 直接用 `ctx.tokens`。
- 删除不再使用的 `theme_from_props` / `tokens_for` / `sync_theme` helper。

#### P1-5 `LayoutCtx::measure_text` 暴露

**问题**：widget layout 阶段无法拿到真实文本宽度，只能用 `chars * 9` 硬编码估算。

**修复**：
- 新增 trait `TextMeasure`（`measure(&self, text, font_size) -> TextSize`）+ `TextSize` struct。
- `LayoutCtx` 加 `text_measure: Option<&dyn TextMeasure>` + `font_metrics` 字段，并加
  `measure_text(text, font_size)` 方法（无注入时回落 `chars * 0.5 * font_size` heuristic）。
- `WidgetHost` 加 `text_measure: Option<Box<dyn TextMeasure>>` + `set_text_measure` 方法；
  layout 时 take 出来放进 LayoutCtx（避免与 `&mut self.root` 借用冲突），结束后放回。
- 试点改造 `HeaderTitle.layout` 用 `ctx.measure_text` 替代 `chars * 9`。其它 chrome widget
  保留硬编码，等 FontdueBackend 适配 TextMeasure 后续一次性切换（接口已就绪）。

#### P1-3 prop_keys 常量化

**问题**：`PropsMap` 是 `HashMap<String, Value>`，key 字符串散落各处易拼错（`"lable"` vs `"label"`）。

**修复**：新增 `ui/core/prop_keys.rs` 集中标准 prop key 常量（TEXT / LABEL / THEME / LAYOUT /
GAP / FLEX / SELECTED / COLLAPSED / ...）。chrome / app / host/layout / host/paint 所有
`props.get("xxx")` 改用 `props.get(prop_keys::XXX)`。业务 crate 自定义 key 由业务层自己定义。

#### P1-4 容器协议显式化

**问题**：`node_container_kind` 此前有 3 个识别路径——`props["layout"]`、
`props["scroll"] = "vertical"`（gallery 旧写法）、组件名。同一种容器多种声明方式增加
认知负担与不一致风险。

**修复**：收敛为**单一来源**——`layout` prop 或内置容器组件名，二者等价。废弃
`props["scroll"] = "vertical"` 写法（删除 `is_scroll_vertical` helper）；测试改用
`layout = "scroll_vertical"` 新写法。

### 9.3 P2 级（性能与简洁性）

#### P2-7 dispatch_pressed_with_focus 合并遍历

**问题**：Pressed 事件中 `deepest_focusable_at` + `dispatch_node` 是两次独立全树遍历。

**修复**：新增 `dispatch_pressed_with_focus(node, event, emitted) -> (handled, Option<WidgetId>)`
一次递归同时完成 hit-test 派发与最深 focusable 收集。`dispatch_node_inner` 加可选形参
`focus_target: Option<&mut Option<WidgetId>>`，None 时等价旧行为。删除 `deepest_focusable_at`
helper，host.rs Pressed 分支改用合并函数。

#### P2-8 `Scene::extend_translated` 避免 clone

**问题**：retained host paint 每帧每个节点都走 `for e in local.translated(off).entries { push(e) }`，
对每个 entry 做 `primitive.clone() + source.clone()`（含 String 等）。

**修复**：新增 `Scene::extend_translated(&mut self, other: Scene, offset: Vec2)` 消费
`other.entries` 并整体平移后并入 self，`RenderPrimitive::translate(self, ..)` 已消费 self，
全程零 clone。paint_node 改用 `extend_translated`（hot path）。保留 `translated(&self)`
用于需要保留原 Scene 的合成场景（overlay / 测试）。

#### P2-9 `Widget::semantics` 默认空实现

**问题**：`fn semantics(&self, ctx: &mut SemanticsCtx);` 必须实现，每个 chrome/装饰性 widget
都被迫写空 stub。

**修复**：trait 默认 `fn semantics(&self, _ctx: &mut SemanticsCtx) {}`。删除全工程 12 处空实现
（chrome.rs / app.rs / counter.rs / form.rs / browser-ui/render.rs / host_tests.rs /
driver.rs / runtime.rs / event_map.rs 等），同步清理 unused 的 `SemanticsCtx` import。

#### P2-10 字体加载失败不再 `eprintln!`

**问题**：winit runtime 在字体加载（默认栈 + 用户注册）和每帧 scene 统计处用 `[DBG]` eprintln，
release 也刷屏，无法按级别过滤；违反 AGENTS.md 「用 tracing 替代 println!」准则。

**修复**：
- `load_default_fonts` / `load_font_asset`：成功 `tracing::debug!`，失败/解码失败 `tracing::warn!`
  （结构化字段 `family` / `id` / `loaded` / `error`）。
- 每帧 scene 统计：`tracing::trace!`（默认不输出，开发期 `RUST_LOG=trace` 看）。

#### P2-11 Gallery 核心控件真控件化（button / toggle / text_input）

**问题**：用户反馈「gallery 演示的组件大多不能交互，点击按钮没反应，鼠标悬停无变化」。
根因：`DemoPreview` 是单个 retained widget，用 `PreviewPainter` 函数把所有 demo 内容**画**出来——
那些「按钮」「输入框」只是像素图，不是 `HostNode` 树中的真 `Widget` 实例，因此收不到 pointer/key 事件，
也没有 hover/pressed/focus 状态。同时 `ui/widgets` crate 里除 `Button` 外，`Toggle` / `TextInput`
等只是数据模型，根本未实现 `Widget` trait。

**修复（分批，本批只做 3 个核心控件）**：

1. **补全 Widget 实现**（`ui/widgets`）：
   - `Toggle`：保留旧数据模型（`permission_prompt` 等仍用），新增 `ToggleSpec`（props）+ `ToggleWidget`
     （完整 Widget：mount/update/event/layout/paint/semantics/focusable）。受控模式从 `props.checked`
     同步状态；点击 emit `spec.action`，应用回写。
   - `TextInput`：保留 `TextInputState`（纯数据），新增 `TextInputWidget`（键盘/点击/聚焦/caret/IME rect）。
     键盘事件改变文本时 emit `ACTION_TEXT_CHANGED` + payload，应用回写 `props.text`。
2. **Gallery demo 子树重构**（`ui/examples/src/gallery/app.rs`）：
   - 新增 `GalleryApp::build_demo_preview(page)` 按 `page.id` 分发：
     `button` / `toggle` / `text_input` 构建真控件子树（Button × 3 / ToggleWidget × 3 / TextInputWidget × 1 + 镜像），
     其余 page 暂时回落到旧 `DemoPreview` painter（后续按同模式逐个迁移）。
   - `GalleryApp` 新增 `demo_button_pressed` / `demo_toggle_state` / `demo_text_input` 字段，
     `dispatch` 新增 6 个 button/toggle action + 1 个 text_changed action。
3. **工厂注册**：`register_gallery_factories` 注册 `"Button"` / `"ToggleWidget"` / `"TextInputWidget"`
   从 `WidgetSpec.props` 抽取 label/action/enabled/checked/text/placeholder 构造实例。
4. **回归测试**（`ui/examples/tests/gallery_retained.rs`）：
   - `demo_preview_renders_non_trivial_content_for_each_page` 按页面适配 source 前缀。
   - 新增 `click_button_in_demo_area_updates_state` / `click_toggle_in_demo_area_updates_bitmask` /
     `disabled_button_does_not_emit_action`，验证 host → widget → action → reducer → props 回流完整闭环。

**影响面**：
- 旧 `DemoPreview` + `PreviewPainter` 架构**保留**（其它 demo 仍依赖），后续按相同模式逐步迁移，
  本批不动其它 demo。
- `zero_ui_widgets` 公共 API 新增 `ToggleSpec` / `ToggleWidget` / `TextInputWidget` / `ACTION_TEXT_CHANGED`；
  旧 `Toggle` 数据模型保持向后兼容（`permission_prompt` 不受影响）。

#### P2-12 所有剩余 demo 真控件化（25 个 page 全部接入）

**问题**：P2-11 只完成 3 个核心控件（button / toggle / text_input），其余 25 个 page 仍走旧
`DemoPreview` painter 架构——所有 demo 内容仍是「画」出来的，无法交互。

**修复策略（实用主义分层）**：

把所有剩余 demo 改用 `Column` / `Row` 容器 + 已有真控件（`Button` / `ToggleWidget` / `TextInputWidget`）
组合，让交互通过 host → widget → action → reducer → props 完整闭环。视觉为主、交互价值低的 demo
（badge / progress / tooltip / status_bubble 等）通过点击 `Button` 切换展示状态（如 progress +/-%、
badge 计数 +1）来体现「可交互」。

**实现要点**：

1. **新文件** `ui/examples/src/gallery/demo_builders.rs`（~580 行）：
   - 把所有 `build_<page>_demo` 方法集中到独立的 `impl GalleryApp` 扩展块，避免 `app.rs` 超 2000 行
     （AGENTS.md 文件大小准则）。
   - `build_demo_preview(page)` 按 `page.id` 分发到 26 个具体 builder（widgets × 15 + patterns × 6 +
     forms/gestures/animation/collection × 4 + theme/i18n/dsl/nav × 4）。
   - 共享辅助 `themed_container(kind, id)`：构造带 theme prop 的容器，减少重复。
2. **`app.rs` 清理**：删除原 `build_demo_preview` / `build_button_demo` / `build_toggle_demo` /
   `build_text_input_demo`（已迁出）。
3. **dispatch 扩展**：`"text_input.changed"` action（来自 `TextInputWidget`）也写入
   `demo_text_input`，让所有 page 的 TextInputWidget 共享同一 reducer 通道。

**每个 demo 的交互模式**：

| page | 交互 |
|------|------|
| icon_button | 4 个 Button（图标用 ASCII 字符），点击高亮当前选中 |
| badge | Inbox Button + 计数 label，点击 +1（capped 99） |
| progress | +/- Button 控制 ASCII 进度条百分比 |
| tabs | 3 个 tab Button，点击切换 selected 内容 |
| tooltip | Button 切换提示 label 显示/隐藏（无真 hover，但可演示交互） |
| list_view | 5 个 row Button，点击标记选中（▶ 标记） |
| menu | 垂直 Button 菜单项，点击高亮 |
| search_field | TextInput + 实时 suggestion 列表 |
| status_bubble | Button 轮换 3 个 status（ok/!/x） |
| toolbar | 4 个 Button，点击高亮 |
| popover | trigger Button 切换浮动内容 |
| popup | trigger + OK / Cancel Buttons |
| data_list | TextInput + Add Button + 8 行 toggle bitmask 显示 |
| command_palette | TextInput + 过滤后的命令列表 |
| dialog_scaffold | trigger + Confirm / Cancel |
| form_demo | TextInput + Toggle + Submit Button |
| gesture_demo | 3 个 Button（Tap / Double tap / Long press） |
| animation_demo | 4 个 Button 切换状态 label |
| collection_demo | 8 个 Toggle + 选中计数 |
| theme_demo | 当前主题 label + Toggle theme Button（接 gallery.theme.toggle） |
| i18n_demo | 本地化文案 + Toggle language Button（接 gallery.locale.toggle） |
| dsl_demo | 静态 YAML 展示 + Apply Button |
| nav_demo | Back / Forward / Home Buttons，点击标记当前 |

**回归测试**（`ui/examples/tests/gallery_retained.rs`）：
- `demo_preview_renders_non_trivial_content_for_each_page` 重写：覆盖全部 26 个 page，统一用
  `demo_` 前缀统计图元数 ≥ 2。
- 新增 `click_tab_updates_selected_index`、`click_popover_trigger_toggles_open_state`、
  `text_input_in_search_field_updates_state`。
- 新增兜底 `all_pages_render_without_panic`：遍历所有 26 个 page 完整 begin + pump_frame，确保
  新增 builder 不会破坏其它 page。

**保留与后续**：
- 旧 `DemoPreview` + `PreviewPainter` 架构保留在 `preview/` 目录（不再被 build_demo_preview 调用，
  但代码未删除——便于回退或作为视觉参考）。后续可清理。
- 视觉精度让位于交互完整性：旧 painter 用 GPU primitives 画出彩色像素，新实现用真控件 + SourceLabel
  （文本 widget）+ ASCII 字符表达状态，外观简化但全部可点。

#### P2-13 Demo state namespace 化（按 page 隔离）

**问题**：P2-12 全部 demo 共用 `demo_button_pressed / demo_toggle_state / demo_text_input` 三个字段，
切换 page 时各 demo 的状态会互相污染（例如 button demo 点到 #2，切到 tabs demo 看到 tab #2 被选中）。
collection_demo 的 8 个 toggle 复用 toggle.0/1/2 action 也是同类问题的表现。

**修复**：

1. `GalleryApp.demo_states: HashMap<page_id, DemoState>` 取代原来的三个扁平字段。
2. `DemoState { pressed: u32, toggles: u8, text: String }` 单结构按 page 隔离。
3. 访问器：`current_demo(&mut self)` / `current_demo_read(&self)` 自动按 `current_page` 取对应 state，
   切 page 时互不干扰；未访问过的 page 返回 `Default::default()`。
4. `dispatch` 改成 namespace 前缀匹配（`s if s.starts_with("gallery.demo.button_click.")` 等），
   上限提升到 button 1..=4 / toggle 0..8，覆盖 collection_demo 的 8 个 toggle。
5. 所有 `build_*_demo` 改用 `self.current_demo_read()` 读取状态。

**影响面**：
- `GalleryApp` 公共字段类型变了（`demo_button_pressed` 等扁平字段移除）——属于内部 state，外部不依赖。
- dispatch action 名字不变，只是 reducer 写入位置按 page 路由。

#### P2-14 Button hover_action（tooltip 真 hover 联动）

**问题**：P2-12 的 tooltip demo 只能"点击切换"，不是真正的 hover 提示，违反 tooltip 语义。
根因：Button 只在 click 时 emit action，没有 hover 进入/离开事件。

**修复**：

1. `ButtonSpec` 新增 `hover_action: Option<ActionId>`（默认 `None` 不 emit）。
2. `Button::event`：
   - `PointerPhase::Moved` 且 `was_hover == false` → emit `hover_action` + payload `"enter"`。
   - `PointerPhase::Exited` 且 `was_hover == true` → emit `hover_action` + payload `"leave"`。
   - 重复 `Moved`（已 hover）不重复 emit。
3. `gallery.demo.hover` action：dispatch 把 `enter`/`leave` 写入 `current_demo().pressed = 1/0`。
4. tooltip demo 的 Button 用 `hover_action = "gallery.demo.hover"`，hover 进入 → 显示提示，
   离开 → 隐藏。回归测试 `hover_tooltip_button_emits_enter_action` 验证 enter 路径。

**影响面**：
- `ButtonSpec` 公共 API 加字段（`hover_action`），向后兼容（默认 `None`）。
- `with_hover_action(&str)` builder 方法。
- `zero_ui_widgets::Button` 单元测试 + `gallery_retained` 集成测试覆盖。

#### P2-15 死代码清理（P2-12 留下的 preview/ 目录）

**问题**：P2-12 把所有 demo 改用真控件子树后，旧的 `DemoPreview` widget + `preview/*.rs` 8 个 painter 文件
（~1000 行）不再被任何调用方引用，但当时为了"便于回退"保留了。P2-13 进一步把 `DemoPreview` widget 定义
也从 `app.rs` 删除了。

**修复**：
- 删除整个 `ui/examples/src/gallery/preview/` 目录（9 个文件）。
- `mod.rs` 不再声明 `pub mod preview`。
- `register_gallery_factories` 不再注册 `DemoPreview` 工厂（P2-13 已删 widget 定义，工厂本就无效）。

**影响面**：纯删除死代码，无行为变化。

### 9.3-bis P3 视觉精度收尾（ButtonVariant 接入 + dead method 清理）

**问题**：另一会话在 `ButtonSpec` 加了 `variant: ButtonVariant` 字段（Primary / Neutral / Selected 三档，paint 时分别用 primary / 中性 surface / primary.darken(0.18) 背景），但
1. `gallery/app.rs` 的 Button 工厂与 `counter.rs` 未传 `variant`，编译失败（`E0063 missing field`）；
2. `ButtonVariant` 没在 `ui/widgets` lib.rs 公开导出，外部无法引用；
3. `Button::foreground` 辅助方法（按 variant 算文字色）定义了却没被 paint 调用（注释"等 M2 接 text foundation 后补"），clippy `dead_code` 直接 fail。

**修复**：
- `ui/widgets/src/lib.rs`：`pub use button::{Button, ButtonSpec, ButtonVariant};`
- `ui/examples/src/gallery/app.rs` Button 工厂：从 `variant` prop 字符串解析（`neutral` / `selected` / 其他→Primary），让 demo 声明侧能直接用 prop 切换视觉档位。
- `ui/examples/src/counter.rs`：补 `variant: Primary`（counter 用主操作语义）。
- `ui/widgets/src/button.rs`：删除未被调用的 `foreground` 方法（保留 `background`，paint 真正在用）。注释里 "等 M2 接 text foundation 后补" 是推测性开发产物——真到了 M2 再按当时的 Color API 写。
- demo 视觉精度升级（在 `demo_builders.rs` 给"选中态"和"次操作"按钮声明 `variant`）：
  - `tabs`：选中 tab `variant=selected`（深色背景），其他 `variant=neutral`；
  - `list_view`：选中行 Selected，其他 Neutral；
  - `menu`：选中项 Selected，其他 Neutral；
  - `popup` Cancel 按钮 Neutral（OK 仍是 Primary，主次操作视觉分离）；
  - `dialog_scaffold` Cancel 按钮 Neutral。

**为何不全量替换 `>` 文本前缀为视觉标记**：
tabs/list_view/menu 用 `> Label` / `[X]` ASCII 前缀标记选中，本身在单色文本渲染下足够清晰；
新增的 `variant=selected` 是**补充**而非替换——选中态现在同时有文本前缀（语义提示）和深色背景（视觉强化），无障碍层面更稳。强行删除前缀会让屏幕阅读器用户丢失选中提示，得不偿失。

**影响面**：
- 编译恢复（gallery + counter）；
- clippy `-D warnings` 通过；
- 12 个 gallery 测试 + 57 个 widget 测试全绿；
- demo 视觉层面，选中态/主次操作有真实颜色区分。

### 9.3-ter P3-3 真视觉回归（B 档 demo 全部用 ColoredBox 替换 ASCII 占位）

**问题**：P3-2 之后，B 档（真控件 + 伪内容/伪视觉）的 demo 仍有"用 `SourceLabel` 拼接列表"或"用 ASCII 字符 `>` / `[X]` 表达图标"的占位表达，视觉上像演示而非真实组件。

**修复范围**（7 个 demo）：

| demo | 改动前 | 改动后 |
|---|---|---|
| `data_list` | 8 行列表用 1 个 `SourceLabel` 拼接 | 8 个独立 `ToggleWidget` 子树（每项可单独交互） |
| `command_palette` | 命令列表用 1 个 `SourceLabel` 拼 `> cmd` | 5 行 Button + ColoredBox marker，选中 variant=selected |
| `icon_button` | 图标用 `<`/`>`/`R`/`X` ASCII + `[X]` 选中标记 | 每个 icon 配 24x24 `ColoredBox` 色块（选中 primary，未选中 muted） |
| `toolbar` | 同 icon_button 用 ASCII | 4 个 ColoredBox icon marker + 选中态 variant=selected |
| `nav_demo` | `< Back`/`> Forward` 文字 | 每项前加 8x24 ColoredBox marker，选中 primary |
| `animation_demo` | 纯 SourceLabel "State: X" | ColoredBox indicator（按 state 切颜色+宽度：Idle→muted/60, Fade→primary/120, Slide→success/180, Spin→warning/240） |
| `gesture_demo` | 纯 Button 列表 | 每项加 ColoredBox 高亮条 + 选中 variant=selected |

**不变的部分（仍属受限边界）**：
- **真图标**（SVG/字体 glyph）：`PaintCtx` 当前只有 `fill_rect`，无法渲染字形。ColoredBox 是当前能做的最高视觉表达。
- **真浮动层**（popover/popup/dialog）：host runtime 无 overlay / z-index 系统，弹层仍是线性排版。
- **真动画**：render-foundation 无时间线/插值 API，indicator 颜色切换是离散的。

**测试**：
- 新增 4 个回归测试（`data_list_renders_per_item_toggles_not_single_label`、
  `command_palette_renders_per_item_buttons_with_markers`、
  `icon_button_has_coloredbox_markers`、`animation_demo_has_coloredbox_indicator`）。
- 总计 16 个 gallery 测试全绿，clippy clean。

**影响面**：demo 视觉表达升级，所有"伪内容"占位都改成真 widget 子树 + ColoredBox 视觉强化。

### 9.4 影响面汇总

- 新增 crate 内模块：`ui/core/prop_keys.rs`、`ui/runtime/src/host/{reconcile,layout,paint,event,semantics}.rs`。
- trait 扩展：`UiApp::theme_tokens`、`TextMeasure`、`LayoutCtx::measure_text`、
  `Widget::semantics` 默认实现、`WidgetHost::set_text_measure`。
- 删除冗余：chrome widget 的 `theme` 字段、`tokens_for` / `sync_theme` / `theme_from_props`
  helper、`is_scroll_vertical` helper、`deepest_focusable_at` helper、12 处空 `semantics` 实现、
  字体加载的 `eprintln!`。
- 行为契约：`props["scroll"] = "vertical"` 写法废弃（改用 `layout = "scroll_vertical"`）。
- 测试覆盖：新增 keyed reconcile × 2、`measure_text` heuristic + 注入 backend × 2、
  `extend_translated` 等价性 × 1；现有测试全绿。

---

### 9.4 P3-4 底层渲染 API 扩展（圆角 + 真文本 + 真图标 + 真浮层 + 真动画）

**触发**：用户反馈 "gallery 好像还不是所有组件都是真实的啊"。深挖后发现"占位感"来自 5 个底层能力缺失：

1. **圆角缺失**：Button paint 只 `fill_rect`，ColoredBox 不支持 radius → 所有按钮/徽标都是直角方块。
2. **真文本缺失**：Button paint **不画 label**（注释说"M2 后补"）→ 按钮上看不到文字，全靠 semantics 传。
3. **真图标缺失**：icon_button / toolbar / nav_demo 用 ASCII 字符（`<` `>` `R` `X`）当图标。
4. **真浮层缺失**：popover / popup / dialog_scaffold 是线性排版的子树，不真正浮在主树之上；outside-click / escape / modal barrier 都没接。
5. **真动画缺失**：animation_demo 的 indicator 是离散状态切换，没有连续时间插值。

**关键发现**：`zero-ui-overlay`（OverlayHost 完整实现）和 `zero-ui-animation`（AnimationClock + Tween + Spring + FakeClock）**早已实现但从未被 WidgetHost 集成**——只是"挂着没用"。所以本批次主要是**接线**而非重写。

#### 9.4.1 视觉基础：Button/ColoredBox paint 接入圆角 + 真文本

- **Button.paint**：从只 `fill_rect` 升级为 `fill_rounded_rect(6px)` + `draw_text(label)`。
  - 新增 `foreground_color(tokens)` 方法（按 variant + enabled 选 `on_primary` / `on_surface`）。
  - label 字号 14px，垂直居中（baseline = `height/2 + 5`），水平居中近似（`(width - text_w)/2`）。
- **ColoredBox**：新增 `radius` prop（`radius > 0` → `fill_rounded_rect`，否则 `fill_rect`）。
- **回归测试**：`paint_emits_rounded_background_and_label_text`（FullRecorder 同时验证 fill_rounded_rect + draw_text）。

#### 9.4.2 视觉基础：新增 Icon widget + 内置 20 个 Unicode 图标

**设计权衡**：选 Unicode 符号而非 SVG path。原因：
- `PaintRecorder` 当前没有 `draw_path` / `draw_svg` API；扩展它需要改 `ui/render` + `ui/core` trait + 后端实现，工程量大。
- Unicode 几何符号（← → ✕ ✓ ☰ ⚙ ⚠ ★ ♥ ⌘ ▶ ⏸ ⏹ ⏭ ⏮ 🔍 🏠 ✉ 🕐 ℹ）已被字体栈支持，**零扩展成本**，gallery 与外部宿主零依赖开箱即用。
- 字符选型基于 Unicode 1.1-7.0 几何符号区段，覆盖主流字体（Segoe UI / SF Pro / Noto Sans）。

**API**：
- `IconKind` 枚举（20 个 variant）+ `glyph()` 返回 Unicode 字符 + `from_name(&str)` 解析 prop。
- `Icon` widget：props `name` / `size`（默认 20）/ `color`（命名预设或 `#rrggbb`，默认 `on_surface`）/ `label`（a11y）。
- paint：`draw_text(glyph, baseline = height*0.5 + size*0.35, size_px, tint)`。
- 不响应事件（纯视觉）；交互由 sibling Button 承担。

**测试**：`glyph_nonempty_for_all_variants` / `from_name_roundtrip` / `from_name_unknown_falls_back_to_info` / `size_prop_drives_layout_square`。

#### 9.4.3 + 9.4.4 浮层：WidgetHost 集成 OverlayHost

**接线**（已实现的 `zero-ui-overlay::OverlayHost` 接入 `WidgetHost`）：
- `WidgetHost` 新增字段：`overlay: OverlayHost` + `overlay_root: Option<HostNode>` + `overlay_dirty: bool`。
- `runtime/Cargo.toml` 加 `zero-ui-overlay` 依赖。
- **paint 分层**：`paint()` 主树完成后 paint `overlay_root` 并 append 到同一 `Scene`（overlay 在后 = 视觉在上层）。
- **layout 同步**：`layout()` 主树 layout 后用同一 viewport 约束 layout overlay 子树。
- **公开 API**：`show_overlay(entry, spec)` / `dismiss_overlay(id)` / `has_modal()` / `overlay()` / `overlay_rect()`。

**事件路由（P3-4-4）**：`dispatch_event` 在主树路由前先处理 overlay：
- **outside-click**：`Pressed` 落在所有 popover 锚定矩形之外 → `overlay.dismiss_on_outside_click(point)` dismiss 最上层候选；返回非空即消费该点击（不冒泡到下层）。
- **Escape**：`overlay.dismiss_on_escape()` dismiss 最上层 escape-able entry。
- **modal barrier**：`has_modal() == true` → 主树完全不接收任何事件（点哪都不命中下层）。

**UiApp trait 扩展**：新增默认方法 `overlay() -> Option<(OverlayEntry, Option<WidgetSpec>)>`。driver 在 `pump_frame` 开头同步：app 声明 overlay 而 host 无 → show；app 无声明而 host 有 → dismiss。

**GalleryApp 接入**：popover / popup / dialog_scaffold 三个 demo 在 `pressed == 1`（打开）时返回真浮动层：
- popover → `OverlayEntry::popover` + OutsideClick dismiss + 内容卡片（ColoredBox bg + SourceLabel + Close button）
- popup → `OverlayEntry::modal` + Escape dismiss + OK/Cancel buttons
- dialog_scaffold → `OverlayEntry::modal` + Escape dismiss + Confirm/Cancel buttons

#### 9.4.5 动画：host 每 frame tick AnimationClock

**接线**（已实现的 `zero-ui-animation::AnimationClock` 接入 `WidgetHost`）：
- `PaintCtx` 新增 `now_ms: Option<i64>` + `frame_requests: &'a Cell<u64>` 字段。
- `PaintCtx::request_frame()` 方法：递增 `frame_requests` Cell（让 widget 在 `&self` 上下文也能调）。
- `WidgetHost` 新增 `animation_now_ms: i64` + `last_frame_requests: u64` 字段。
- `paint_node` 签名扩展：传入 `now_ms` + `frame_requests` Cell。
- `paint()` 每帧重置 Cell 为 0，paint 完读值存 `last_frame_requests`。
- **driver 接入**：`pump_frame` 开头调 `host.advance_clock(16)`（≈60fps）。
- **公开 API**：`advance_clock(delta_ms)` / `animation_now_ms()` / `has_pending_animation()`。

**ColoredBox 接入 pulse**：新增 `pulse: bool` prop。paint 时若 `pulse && ctx.now_ms.is_some()`：
- `phase = sin(now_ms / 600)` → `lighten(0.15 * phase)` 或 `darken(-0.15 * phase)`（颜色明度连续振荡 ±15%）。
- 调 `ctx.request_frame()` 声明需要下一帧（永续动画，直到 `pulse = false`）。

**GalleryApp 接入**：animation_demo 的 indicator ColoredBox 启用 `pulse = true`，让所有状态都有连续色变（验证完整动画环路：driver tick → host now_ms → widget sample → request_frame → driver 续帧）。

#### 9.4.6 Gallery 全 demo 接入清单

| Demo | 改动 |
|---|---|
| button | paint 自动圆角 + label（无需改 demo） |
| badge | ColoredBox `radius=10` → 胶囊形徽标 |
| status_bubble | ColoredBox `radius=8` → 正圆状态点 |
| progress | filled + track 都 `radius=6` |
| icon_button | ASCII `<` `>` `R` `X` → 真 Icon widget（back/forward/play/close） |
| toolbar | ASCII → Icon widget（back/forward/play/home） |
| nav_demo | ColoredBox marker → Icon widget（back/forward/home） |
| popover | 线性排版 → host.show_overlay 真浮动 + outside-click dismiss |
| popup | 线性排版 → host.show_overlay modal + Escape dismiss |
| dialog_scaffold | 线性排版 → host.show_overlay modal + Escape dismiss |
| animation_demo | 离散状态 → ColoredBox `pulse=true` 连续色变动画 |

#### 9.4.7 测试覆盖

新增 5 个回归测试（gallery_retained.rs，从 16 → 21）：
- `toolbar_uses_real_icon_widgets`：验证 `demo_toolbar_glyph_{1..4}` Icon widget 存在。
- `nav_demo_uses_real_icon_widgets`：验证 `demo_nav_marker_{1..3}` Icon widget 存在。
- `popover_demo_creates_overlay_when_open`：验证 popover 打开时 overlay 视觉子树出现。
- `animation_demo_indicator_has_pulse_enabled`：验证 pulse indicator 触发 `has_pending_animation`。
- `modal_popup_blocks_main_tree_pointer_events`：验证 modal barrier 屏蔽主树事件路由。

新增 1 个 widget 测试（button.rs）：
- `paint_emits_rounded_background_and_label_text`：FullRecorder 验证 paint 同时产出 fill_rounded_rect + draw_text。

新增 4 个 icon 测试（icon.rs）：glyph 非空 / from_name 往返 / unknown fallback / size prop 驱动 layout。

**核心 crate 测试全绿**（276 通过，0 失败）：zero-ui-core 17 + zero-ui-render 57 + zero-ui-runtime 87+3 + zero-ui-widgets 62 + zero-ui-overlay 19 + zero-ui-animation 10 + zero-ui-examples 21。

#### 9.4.8 设计决策记录

- **为什么 Icon 用 Unicode 而非 SVG**：PaintRecorder 无 draw_path API；扩展成本高；Unicode 已覆盖 90% 常用图标且零依赖。未来若需 SVG，可在 Icon 加 `glyph_kind: GlyphSource::Unicode | Svg(AssetId)` 字段扩展。
- **为什么 overlay 用独立 overlay_root 而非主树 z-order**：独立子树让 overlay 的 layout/paint/reconcile 与主树解耦，modal barrier 只需"主树跳过"一行判断；z-order 方案需要重排整个 paint 顺序，侵入性更大。
- **为什么动画用 Cell 而非 `&mut`**：paint 借 `&mut recorder`，若 frame_requests 也 `&mut` 会与 recorder 借用冲突。Cell 让 widget 在 `&self` 上下文（paint 借 `&mut recorder`）也能递增计数，符合 paint 方法签名约束。
- **为什么 pulse 是永续动画**：用于验证完整动画环路（driver tick → widget sample → request_frame → driver 续帧）。真实业务动画（如 Tween 250ms 过渡）会在 `is_done` 后停止 request_frame，自动停止续帧。

---

### 9.5 P3-5 视觉精度收尾（tooltip 真浮层 + 选中标记 Icon 化）

**触发**：P3-4 完成后复查发现仍有 4 处"半真"残留：
1. tooltip 还在用线性排版（hover 时把按钮挤下去），没用 overlay。
2. list_view / menu / tabs 的选中标记还在用 `> ` ASCII 前缀。
3. search_field 建议列表用纯文本 SourceLabel，没有真 Button + Icon。

#### 9.5.1 tooltip 真浮层化

- `build_tooltip_demo` 移除内联 bubble（ColoredBox + SourceLabel），只保留触发按钮 + 说明文字。
- `GalleryApp::overlay()` 加 `"tooltip"` 分支，hover 触发时返回 `OverlayEntry::tooltip`（锚定 + OutsideClick dismiss）。
- 新增 `build_tooltip_overlay()`：深色胶囊背景（radius=12）+ info Icon + 一行文字。
- **行为差异**：hover 按钮时 tooltip 真正浮在主树之上（不再挤压按钮），点外部自动 dismiss。

#### 9.5.2 选中标记用真 Icon（check）替代 ASCII `>`

3 个 demo 统一改造：
- **list_view**：选中项前加 `Icon(check, primary, 16px)`；未选中项留 16×16 muted spacer（保持宽度一致，避免选中切换时跳变）。
- **menu**：同 list_view 模式（check Icon + spacer）。
- **tabs**：选中 tab 前加 check Icon（不加 spacer，tabs 横排不需要对齐）。

#### 9.5.3 search_field 真建议列表

- 输入框前加 `Icon(search, muted, 20px)`（更像真实搜索框）。
- 建议从纯文本 `SourceLabel` 改为每行 `Icon(check) + Button(neutral)`：
  - 空查询 → 显示 `(type to filter suggestions)` 提示。
  - 无匹配 → 显示 `No match`。
  - 有匹配 → 每个候选一行，可点击选中。

#### 9.5.4 测试覆盖

新增 5 个回归测试（gallery_retained.rs，从 21 → 26）：
- `list_view_selected_row_has_check_icon`：选中项有 check Icon，未选中无。
- `menu_selected_item_has_check_icon`：选中项有 check Icon。
- `tabs_selected_tab_has_check_icon`：选中 tab 有 check Icon。
- `search_field_has_search_icon_and_suggestions_are_buttons`：search Icon 存在 + 建议是 Button。
- `tooltip_demo_uses_overlay_not_inline_bubble`：hover 后有 overlay 视觉子树。

---

*RFC 结束。上一段落：Spec（需求规格），当前段落：RFC（技术设计），下一段落：实施交接（逐步骤指令）。*
