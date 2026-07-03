# zero-ui-widgets

通用 UI SDK 的基础控件集合。浏览器无关，不含浏览器领域语义。

## 架构位置

```
ui/widgets ←── ui/patterns（组合模式）
          ←── browser-ui/chrome（浏览器领域组件）
```

## 控件清单

| 控件 | 模块 | 说明 |
|------|------|------|
| `Button` | `button.rs` | 完整 `impl Widget`；全 token 派生色（primary/hover/lighten/pressed/darken/disabled mix）；WCAG AA |
| `IconButton` | `icon_button.rs` | 图标按钮（数据 props，组合 Button 样式） |
| `Toggle` | `toggle.rs` | 开关控件 |
| `TextInputState` | `text_input.rs` | 文本输入状态模型（非完整 Widget；insert/backspace/move_cursor char_indices UTF-8 安全） |
| `Menu` / `MenuItem` / `ContextMenu` | `menu.rs` | 菜单/上下文菜单（纯数据模型） |
| `Popover` / `Popup` | `popover.rs` / `popup.rs` | 弹出层（纯数据模型） |
| `ListView` | `list_view.rs` | 列表视图（纯数据模型） |
| `Badge` | `badge.rs` | 徽章：全 token 派生色（5 tone × light/dark WCAG AA） |
| `Tooltip` | `tooltip.rs` | 工具提示（纯数据模型） |
| `ScrollBar` (纯函数) | `scrollbar.rs` | `layout_scrollbar` / `hit_test` / `drag_to_command` / `paint_scrollbar`；`ScrollBarStyle::from_tokens` |
| `ProgressIndicator` | `progress.rs` | 进度指示器 |
| `Tabs` / `Toolbar` | `tabs.rs` / `toolbar.rs` | 标签/工具栏（纯数据模型） |

## 语义色合规

所有控件消费 `SemanticTokens`（经 `PaintCtx.tokens`），零硬编码浏览器色值（grep 全域验证无剩余 non-test `Color::rgb` 残留）。

## 依赖

- `zero-ui-core` + `serde` + `compact_str`
- 零浏览器业务 crate 依赖（DC-1）

## 测试

- `cargo test -p zero-ui-widgets` — 40 测
- 覆盖：Button paint/layout/event/disabled/UTF-8/Exited→清除 pressed/icon_button/toggle/scrollbar hit+drag+style+badge token AA

## 深度审查

2026-07-03 全 crate 审查，修 2 bug：Button paint 硬编码 96×32 截断 + layout 字节计宽→char count。Button 是此 crate 中唯一 `impl Widget` 的完整实现。
