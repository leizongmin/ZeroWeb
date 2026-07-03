# zero-ui-patterns

通用 UI SDK 的组合模式（由多个基础控件组合成的常用 UI 组件）。浏览器无关。

## 架构位置

```
ui/widgets → ui/patterns → browser-ui/chrome
```

patterns 层位于 widgets 之上，由基础控件组合成更高层次的、跨应用可复用的 UI 模式。

## 模式清单

| 模式 | 模块 | 说明 |
|------|------|------|
| `SearchField` | `search_field.rs` | 搜索输入域（含搜索图标+清空按钮模式） |
| `SuggestionList` | `suggestion_list.rs` | 建议下拉列表（地址栏自动补全等场景） |
| `CommandPalette` | `command_palette.rs` | 命令面板（VS Code 风格命令搜索） |
| `DataList` | `data_list.rs` | 数据列表（列表+数据显示模式） |
| `StatusBubble` | `status_bubble.rs` | 状态气泡（查找栏匹配数等临时 status） |
| `TabBar` | `tab_bar.rs` | 标签页条（水平 tab 切换模式） |
| `DialogScaffold` | `dialog_scaffold.rs` | 对话框脚手架（标题+内容+操作按钮布局） |

## 依赖

- `zero-ui-core` / `zero-ui-widgets` / `serde` / `compact_str`
- 零浏览器业务 crate 依赖（DC-1）
- 零硬编码色值（全仓 grep 验证无 `Color::rgb` 用量）

## 测试

- `cargo test -p zero-ui-patterns` — 7 测
