# zero-ui-commands

通用 UI SDK 的命令系统。提供命令注册/执行、快捷键解析、菜单模型生成和命令面板搜索等能力。同一命令可通过菜单/快捷键/命令面板三种入口触发。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `CommandId`（= `ActionId`） | 命令标识符（与 Action 系统共享类型） |
| `Command` | 命令定义（label / description / shortcut / menu_path / in_palette / enabled / action_id / payload） |
| `CommandResult` | 执行结果：`Executed` / `Unknown` / `Disabled`（§8.4.1B 三态，不静默失败） |
| `CommandRegistry` | 注册/执行/快捷键路由（register 同 id 替换 / resolve_shortcut 跳过禁用 / execute） |
| `MenuModel` / `MenuNode` | 按 `menu_path` 归组建树（嵌套子组 View/Zoom；叶子 label 走 message id） |
| `MenuEntry` | 菜单条目（item / separator / submenu） |
| `PaletteEntry` | 命令面板条目（label + description + action） |
| `Shortcut` | 快捷键（modifiers + key） |

## §8.4.1B 验收

同一 reload command 可由三种入口触发：
1. **菜单**：View → Reload（经 `menu_model` 从 `menu_path = "View/Reload"` 派生）
2. **快捷键**：Ctrl+R → `resolve_shortcut` → 同一 ActionId
3. **命令面板**：输入 "reload" → `palette_search`（大小写不敏感，搜索 label + description + id）→ 同一 ActionId

测试证明三种入口都解析到同一个 `ActionId::Reload`。

## 依赖

- `zero-ui-core` + `compact_str`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-commands` — 13 测
- 覆盖：register/execute/resolve_shortcut/Label+description/id 三字段 palette_search/嵌套子组 menu_model/Disabled 不路由
- Coverage：lib.rs 98.23% / menu.rs 93.48%
