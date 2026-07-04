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

*RFC 结束。上一段落：Spec（需求规格），当前段落：RFC（技术设计），下一段落：实施交接（逐步骤指令）。*
