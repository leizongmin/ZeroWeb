# zero-ui-collections

通用 UI SDK 的集合/虚拟化组件。提供大量数据记录的虚拟化渲染、回收、选择等能力。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `VirtualCollection` trait | 虚拟化集合接口（`item_count` / `item_key` / `build_item`） |
| `ItemKey(u64)` | 稳定项标识（跨重建复用状态） |
| `VisibleWindow` | 可视窗口范围（`first_visible` + `visible_count`） |
| `MaterializedItem` | 物化后的可视项（key + index + data） |
| `Selection` | 选择集（`select` / `clear` / `toggle`） |
| `Recycler<V>` | 状态回收器（`retain_window` 滚动回收；`get` / `remove`） |
| `DynamicCollection` | 闭包支撑的集合实现（测试/简单场景） |

## 工具函数

| 函数 | 说明 |
|------|------|
| `materialize(c, window)` | 物化可视窗口内元素 |
| `materialize_at(c, list, scroll_y)` | 按 scroll offset 确定窗口并物化 |
| `find_duplicate_key(items)` | 检测重复 key 并产出 diagnostic |

## 使用场景

DC-13 §8.4.1B 矩阵要求：下载列表 / 书签列表 / 历史记录 / Tab Overview 等大型列表虚拟化渲染。

## 依赖

- `zero-ui-core` + `compact_str` + `hashbrown`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-collections` — 8 测
- 覆盖：materialize 越界过滤 / materialize_at scroll 窗口确定 / find_duplicate_key / Recycler retain_window / Selection select+clear
- Coverage 96.74%
