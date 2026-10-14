# 跨宿主浏览器数据模型

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-browser-shell` 是桌面与 Android 消费的 UI 无关状态层。按标签身份、导航账本和 profile 恢复读第一叶；按下载元数据、设置、建议排序和菜单结构读第二叶。不包含页面 DOM、浏览器网络存储或 renderer 生命周期。

### User Task Navigation
- 恢复后前进后退不对：从 `BrowserShell::restore_session` / `Tab::history_index` 查 [标签与 profile](profile-and-tabs.md)。
- Android 恢复与桌面表现不同：比较 `facade::load_profile` 与 `BrowserApp::new/persist_user_data`，再读 [Android 状态](../android/chrome-and-state.md)。
- 搜索建议、下载状态、菜单动作不对：从 `Autocomplete::suggest` / `DownloadManager` / `MenuItem` 查 [用户服务](user-services.md)。
- 数据功能的真实验证范围查 [验证地图](../../../../zeroweb-verify/features/index.md)；Shell 测试源码和知识条目不代表本轮执行通过。

## Gotchas
- Shell 默认创建一个空白标签，但 restore_session 空快照会清成零标签；Android close_tab 另行补建空标签，产品“始终有一页”不是 TabManager 的不变式。 (`crates/browser-shell/src/browser.rs`, `apps/android-browser/rust/src/facade.rs`)
- profile JSON 和设置文件是两套加载入口；桌面消费默认设置/书签，Android 消费指定 profile 的会话/书签/历史/下载，不能仅因两端复用 BrowserShell 就推断持久化功能完全一致。 (`crates/browser-shell/src/browser.rs`, `apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`)

## Architecture
- Shell 负责可序列化业务状态，宿主负责把导航、下载和菜单动作转成真实操作；零网络/renderer 接线的模型单测只能证明状态转换，不证明页面或文件已经完成动作。 (`crates/browser-shell/src/lib.rs`, `apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`)

## Child Knowledge Nodes
- [标签、历史、书签、会话与 profile](profile-and-tabs.md) — 导航账本、关闭恢复、输入文件边界与两宿主持久化差异。
- [下载、设置、补全与菜单](user-services.md) — 下载元数据状态机、设置写入、建议排序与菜单动作数据。
