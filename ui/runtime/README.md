# zero-ui-runtime

通用 UI SDK 的运行时层，将声明树（WidgetSpec）驱动为 retained widget 实例的完整生命周期。浏览器无关（winit 类型不泄漏至此层）。

## 架构位置

```
ui/runtime ←── ui/adapters/winit（PlatformRuntime impl）
     │
     ├── ui/core（基础类型）
     ├── ui/render（Scene 输出）
     └── ui/i18n（运行时 i18n 提供者）
```

## 核心模块

### `host.rs` — WidgetHost（核心）

三棵树（Widget → Element → Render）的运行时驱动：

| 阶段 | 说明 |
|------|------|
| `set_root(WidgetSpec)` | reconcile 声明树→retained widget 实例树（stable WidgetId 跨重建复用） |
| `layout(Size)` | measure 自下而上 + arrange 自上而下定绝对 rect；Row/Column 支持 flex/cross_axis/main_axis/fill-sizing/child min-max |
| `paint(Scene)` | 每 widget 局部坐标 paint → 按绝对 origin 平移并入全局 Scene |
| `dispatch_event(UiEvent)` | hit-test 命中最深最上层节点 + 冒泡；支持 click-to-focus / keyboard / IME / gesture arena additive |
| `flush_accessibility()` | 自动推 a11y tree + focus_moved 到 `AccessibilityBackend` |

**布局能力矩阵**（6 维完整闭合）：

- `flex` 弹性权重（opt-in，默认 0 向后兼容）
- `cross_axis_align`（Start/Center/End row 垂直/column 水平）
- `main_axis_align`（Start/Center/End 剩余主轴分配）
- `main_axis_distribution`（SpaceBetween/Around/Evenly）
- `fill-sizing`（tight/exact 约束填满容器）
- `child_min/max`（单子节点独立钳制）

### `theme_provider.rs` — ThemeProvider

- 四态主题解析（System/Light/Dark/HighContrast）
- `cycle_preference()` 三元循环（System→Light→Dark→System）
- `set_text_scale` / `set_density` → ThemeChanged needs_layout
- `diff_invalidation`：仅颜色变→needs_paint，字体/间距变→needs_layout

### `accessibility.rs` — AccessibilityBackend

- `AccessibilityBackend` trait（update_tree / focus_moved / announce）
- WidgetHost 自动推送（焦点变化不重建全树，与 Flutter a11y 一致）

### `app.rs` — UiApp trait

- 宿主应用接口：`root_spec()` → `build_spec()` → `dispatch(action)` → reduce
- `WinitDriver` 将事件循环抽为可测试核心（`begin` / `pump_event` / `pump_frame`）

## 依赖

- `zero-ui-core` / `zero-ui-render` / `zero-ui-i18n` / `zero-ui-gestures`（opt-in gesture arena）
- `compact_str` / `hashbrown` / `thiserror`

零浏览器业务 crate 依赖（DC-1 机械验证）。

## 测试

- `cargo test -p zero-ui-runtime` — 83 测（host 58 + driver/integration）
- 覆盖：retained 闭环/布局 6 维/焦点+IME+FocusScope+semantics/失效传播/事件路由/theme_changed/accessibility auto-push/gesture arena

## 文件大小

- `host.rs` 1331 行，`host_tests.rs` 1768 行（`#[path]` 模式，单文件合规）
