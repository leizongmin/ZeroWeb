# zero-ui-overlay

通用 UI SDK 的浮层管理器。提供 overlay/popover/tooltip/modal/dialog/sheet/toast 等浮层模式的统一管理。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `OverlayId`（= `WidgetId`） | 浮层唯一标识 |
| `DismissPolicy` | 关闭策略：`None` / `OutsideClick` / `Escape` / `Any` |
| `OverlayEntry` | 浮层条目（id + 内容 + 策略 + anchor 位置 + trap_focus + 自定义数据） |
| `OverlayHost` | 浮层管理器：`show()→OverlayId` / `dismiss()` / `top()` / `has_modal()` |

## OverlayHost 能力

| 方法 | 说明 |
|------|------|
| `show(entry)→OverlayId` | 显示浮层并返回 id |
| `dismiss(id)` | 关闭指定浮层 |
| `has_modal()` | 是否有模态浮层打开 |
| `top()` | 最上层浮层 |
| `focus_trap_ids()` | trap_focus 浮层的 id 列表（最上层在前，供 runtime FocusScope 绑定） |
| `dismiss_on_outside_click(point)` | 点击浮层外部可关闭（anchor=None 全屏 modal 覆盖整屏不命中） |
| `dismiss_on_escape()` | Escape 键关闭最上层浮层 |

## 依赖

- `zero-ui-core` + `serde`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-overlay` — 10 测
- 覆盖：popover/tooltip/modal/sheet 显示关闭 / outside-click（含全屏 modal 保护）/ escape dismiss / focus_trap_ids
- Coverage 95.52%
