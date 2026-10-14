# 标签、历史、书签、会话与 profile

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `BrowserShell`、`Tab/TabManager`、`ProfilePaths`、`SessionState`、`History` 和 `Bookmarks`。定位导航账本、关闭恢复、启动数据丢失或 profile 加载时读取。Shell 是 UI 无关数据模型，不创建页面引擎或 renderer。

### Key Entry Points
- `BrowserShell::load_profile/save_profile`：宿主指定目录的四类状态文件。
- `Tab::navigate/go_back/go_forward`：单标签导航账本。
- `BrowserShell::record_initial_navigation/on_page_loaded`：首次内部页记账与访问记录分离。
- Android 接入见 [状态与系统集成](../android/chrome-and-state.md)，桌面接入见 [标签导航](../desktop/navigation-and-input.md)；页面存储不是这里的 profile JSON，详见 [Web Storage](../../web-data/storage/web-storage-and-persistence.md)。
- [验证入口](../../../../zeroweb-verify/SKILL.md) 提供实际功能地图；本次只读源码与测试断言，未执行恢复验收。

## Gotchas
- `restore_session` 在多条历史时 clear_history 后逐条 push_navigation，但没有恢复 history_index；当快照位于非首条记录时，显示 URL 可被恢复为快照项而索引仍为零，前进/后退能力与显示页不一致，这是当前静态风险而非已修复行为。 (`crates/browser-shell/src/browser.rs`, `crates/browser-shell/src/tab.rs`)
- BookmarkId::next 与 sync_counter 各有自己的函数内静态 COUNTER，恢复最大 ID 并不会推进新 ID 生成器；跨进程恢复含小 ID 的文件后可能与新建书签或文件夹撞 ID，不能采信 sync_counter 注释的防冲突保证。 (`crates/browser-shell/src/bookmarks.rs`)
- Shell 的 private 标记使 save_session 排除无痕标签，但 on_page_loaded 记录历史、关闭快照队列和书签操作没有按 private 过滤；仅凭此模型不能宣称无痕不留任何记录或隔离网络缓存。 (`crates/browser-shell/src/browser.rs`, `crates/browser-shell/src/tab.rs`)
- 当前 TabManager::close_tab 关闭任意有效标签都会重新选择被关位置前一项或首项，即使关闭的不是当前活动标签也会切换；源码行为比“关闭活跃标签时切换”的文档描述更宽。 (`crates/browser-shell/src/tab.rs`)
- load_profile 加载 session/bookmarks/history/downloads，但不加载 settings；new_with_persisted_settings 则加载默认设置与书签，不加载会话、历史和下载，两个构造入口不能互换。 (`crates/browser-shell/src/browser.rs`)
- 桌面 BrowserApp 当前使用 new_with_persisted_settings，persist_user_data 只保存设置与书签；Android facade 使用指定 ProfilePaths 并 save_profile，不能把 Android 的四类恢复能力推断为桌面已接入。 (`apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`)
- ProfilePaths 的单文件读取上限是 1MiB，而 save 没有对序列化结果执行同一上限检查；即使条目数量没超各自阈值，保存成功的大文件也可能在下次 load 被回退为空。 (`crates/browser-shell/src/profile.rs`, `crates/browser-shell/src/session.rs`, `crates/browser-shell/src/history.rs`, `crates/browser-shell/src/bookmarks.rs`)
- SessionState::load 校验活动索引、历史索引、至多10000标签/每标签10000历史以及16KiB文本；直接 serde_json::from_str 绕过 is_valid，旧 active_index_beyond_tabs 测试只证明反序列化保留越界值，不代表 load 会接受。 (`crates/browser-shell/src/session.rs`)

## Architecture
- 每标签的导航 history 与浏览器全局 History 是两本账：Tab::navigate 追加并截断前进分支，History::record 按精确 URL 去重并移到最前，后者不是完整访问时间线且没有时间戳字段。 (`crates/browser-shell/src/tab.rs`, `crates/browser-shell/src/history.rs`)
- Shell 的 go_back/go_forward 只恢复 URL、title、索引并把 loading 置 false；页面重新加载由桌面 BrowserApp 调 start_tab_load 或 Android 宿主另行驱动，Shell 不发 IPC。 (`crates/browser-shell/src/tab.rs`, `apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`)
- ProfilePaths 把四种 JSON 固定到宿主根目录；load_profile 各类独立回退，save_profile 按会话、书签、历史、下载顺序使用问号传播，后项失败不回滚已保存前项，因此不是跨文件事务。 (`crates/browser-shell/src/profile.rs`, `crates/browser-shell/src/browser.rs`)
- atomic_write 使用同目录同扩展名 tmp 写入、sync_all 再 rename，不同步父目录且临时名固定；它提供单文件替换顺序，不提供多个宿主并发写同一路径的协调。 (`crates/browser-shell/src/profile.rs`)
- 会话不保存 TabId，恢复重新生成 ID，也不保存 pinned/muted/zoom/private 等字段；活动索引按过滤后的非私密标签重新计算，活动页若为 private 则快照活动索引为 None。 (`crates/browser-shell/src/session.rs`, `crates/browser-shell/src/browser.rs`)

## Decisions
- 欢迎页必须补记单标签首次导航才能让离开后返回箭头可用，但精确 zero://newtab 不进入全局 History，避免污染历史列表和地址栏建议；其他 zero:// 内部页仍照常入史。 (`crates/browser-shell/src/browser.rs`, `crates/browser-shell/src/tests/browser_shell.rs`, `git:c4a86a14d5f688b6e45af5e9f6b7ffffbb68c048`)
- 最近关闭队列只保存 URL/title，最多10项并按后进先出恢复；重开创建新标签并 navigate，不恢复原历史、滚动、缩放或 renderer 状态。 (`crates/browser-shell/src/browser.rs`)
- duplicate 克隆历史索引、private、pinned、muted 与 zoom，生成新 ID 并重置 loading/crashed/needs_attention；这与会话恢复的字段范围不同。 (`crates/browser-shell/src/tab.rs`)

## Patterns
- 导航按钮和 smoke 状态应调用 Tab::can_go_back/can_go_forward，不在消费者重复索引算式；Tab::set_url 只改当前 URL/loading，不追加历史，不能作为用户新导航的替代。 (`crates/browser-shell/src/tab.rs`)
- move_tab 在重排前保存活动 TabId，重排后再按 ID 定位活动索引；对超界或原位移动返回 false，不以向量索引作为稳定标签身份。 (`crates/browser-shell/src/tab.rs`)
- Bookmarks::add 允许重复 URL 和不存在的 folder_id，find_by_url/remove_by_url 只处理首个精确匹配，remove_folder 则级联删除所属书签；UI 的收藏开关并不是模型层唯一性约束。 (`crates/browser-shell/src/bookmarks.rs`, `crates/browser-shell/src/browser.rs`)
- FindState 只保存查询、1-based 当前匹配、总数与环绕提示，实际字形匹配交给渲染侧；切换大小写或全字选项清空计数而不关闭查找。 (`crates/browser-shell/src/browser.rs`, `apps/browser/src/app.rs`)
