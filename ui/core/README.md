# zero-ui-core

通用 UI SDK 的基础类型与协议层。此 crate 位于依赖树最底层，被所有 UI SDK crate 依赖。

## 架构位置

```
ui/core  ←── 所有 ui/* crate 依赖的最底层
     (零 ui/* 上层依赖)
```

浏览器无关，仅依赖 `compact_str` / `hashbrown` / `serde` / `thiserror`。

## 模块

| 模块 | 核心类型 | 职责 |
|------|----------|------|
| `geometry` | `Rect`, `Point`, `Size`, `Edges`, `Constraints` | 2D 几何原语；`Rect::contains`（edge-inclusive）、`Rect::intersect`（边相接→None） |
| `event` | `UiEvent`, `PointerEvent`, `KeyEvent`, `WheelEvent`, `ImeEvent` | 浏览器无关的 UI 事件模型（winit 类型不泄漏至此层） |
| `widget` | `Widget` trait, `WidgetId`, `ComponentType` | Widget 接口定义（生命周期 `build`→`update`→`layout`→`paint`→`event`→`semantics`） |
| `element` | `Element` | Element reconcile（按位置比较决定复用/重建） |
| `action` | `ActionId`, `ActionPayload` | 事件处理产生的 Action（`ActionId` 是 DSL action 的强类型 id） |
| `binding` | `Binding`, `BindingSource` | DSL 属性绑定模型 |
| `theme` | `Color`, `SemanticTokens`, `ColorScheme`, `Theme`, `Typography`, `Spacing`, `ThemeResolver` | 主题系统：WCAG contrast lint、四态 token（Light/Dark/HC-Light/HC-Dark）、`diff_invalidation` |
| `focus` | `FocusDirection`, `TraversalPolicy`, `FocusScope` | 焦点遍历（Tab 序 + scope trap 折返/逃逸） |
| `semantics` | `SemanticsNode`, `SemanticsRole`, `SemanticsFlags` | 无障碍语义树节点 |
| `invalidation` | `InvalidationFlags` | 局部失效标记（NEEDS_LAYOUT / NEEDS_PAINT / NEEDS_SEMANTICS） |
| `layout` | `WindowMetrics`, `ViewportClass`, `PlatformClass`, `InputClass`, `AdaptiveBranch`, `Orientation`, `TypographyScale` | 响应式布局度量（含 phone/tablet/desktop presets） |
| `scroll` | `ScrollMetrics`, `ScrollCommand` | 滚动状态与命令 |

## 设计约束

1. 零浏览器业务 crate 依赖（DC-1 机械验证）
2. winit 类型不泄漏到 ui/core（事件模型用自有 `UiEvent`）
3. `Rect::contains` 含端点（edge-inclusive），`Rect::intersect` 边相接返 None——这是 host clip 链的关键语义

## 测试

- `cargo test -p zero-ui-core` — 53 测
- 覆盖：geometry 边界/主题 WCAG AA all-6-pair/HighContrast AAA diff_invalidation/FocusScope trap+逃逸/WindowMetrics presets+text_scale+orientation+density/semantics/resolve_scheme 四态

## 深度审查

2026-07-03 全 crate 深度审查，0 behavior bug。新增 2 处 Rect 边界语义回归测。
