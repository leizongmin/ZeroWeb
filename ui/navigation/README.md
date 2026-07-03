# zero-ui-navigation

通用 UI SDK 的导航系统。提供路由栈、modal/sheet/dialog 分层导航、恢复快照等能力。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `RouteId(u64)` | 稳定单调递增的路由 id |
| `RouteKind` | 路由类型：`Page` / `Modal`（模态对话框）/ `Sheet`（手机底部 sheet） |
| `Route` | 路由定义（`RouteSpec`，含 id/kind/name/path/data；Serde derive 可序列化恢复） |
| `RouteStack` | 路由栈：`push` / `pop` / `replace` / `route_of(id)` / `top_overlay()`（最深覆盖层路由）/ `route_names()` / `from_routes()`（快照恢复） |
| `Navigator` trait | 导航器接口（`RouteStack` 实现；宿主可持 `&mut dyn Navigator`） |

## 依赖

- `zero-ui-core` + `compact_str` + `serde`
- 零浏览器业务 crate 依赖
- dev-dep：`serde_json`（JSON roundtrip 测试）

## 测试

- `cargo test -p zero-ui-navigation` — 8 测
- 覆盖：push/pop/replace / route_of 跨重建查 / top_overlay / 恢复快照 route_names+from_routes / Navigator trait
- Coverage 96.67%

## 设计要点

- `RouteStack.top_overlay()` 供 runtime 判断事件屏障（弹层打开时底层页面不响应点击）
- 与 `ui/restoration` 配合：`route_names()` + `from_routes()` 实现 session restore
- 配合 `BackNavigationService`（`ui/platform`）：平台 back 键→Navigator.pop（LIFO 仲裁）
