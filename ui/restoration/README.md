# zero-ui-restoration

通用 UI SDK 的状态恢复系统。为应用提供 session restore / route state / scroll position / selection 等可恢复状态的保存与恢复能力。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `RestorationId` | 恢复标识符（类型化构造器：`route_stack()` / `scroll_y(widget_id)` / `selection(widget_id)` / `namespaced(prefix)` per-tab 隔离） |
| `RestorationStore` | 键值恢复存储：`save(id, json)` / `restore(id)→Option<Value>` / `take(id)` / `remove(id)` / `clear_namespace(prefix)` |
| JSON 持久化 | `to_json()→String` / `from_json(str)→Self`（按 id 排序输出确定性，重启恢复用） |

## 使用场景

DC-13 §8.4.1B 矩阵要求：
- 路由栈恢复（`route_stack()` → `RouteStack::from_routes()`）
- 滚动位置恢复（`scroll_y(tab_id)` → 页面恢复后跳到上次阅读位置）
- 输入选区恢复（`selection(field_id)` → 光标位置保持）
- 标签页状态恢复（`namespaced("tab:3")` → 当前标签页的完整状态）

## 依赖

- `zero-ui-core` / `compact_str` / `hashbrown` / `serde` / `serde_json`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-restoration` — 9 测
- 覆盖：save/restore/take/remove / clear_namespace / JSON roundtrip 确定性 / 类型化 id 构造
- Coverage 98.19%
