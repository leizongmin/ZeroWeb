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
| 搜索过滤导航列表 | 待设计 | UI 的 SearchField 未暴露 filter callback；可用 dispatch + 手动过滤实现 |
| 导航分组折叠 | 待验证 | 看 ListView 是否支持分组头点击折叠 |
| 统一的 dark/light 色板 | 已有 | `zero-ui-core` 的 `SemanticTokens` 已提供 |

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

*RFC 结束。上一段落：Spec（需求规格），当前段落：RFC（技术设计），下一段落：实施交接（逐步骤指令）。*
