# 下载、设置、补全与菜单

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 DownloadManager、BrowserSettings/SearchEngine、Autocomplete 与 ContextMenu 的 UI 无关模型。下载请求、文件写入、菜单几何和动作执行由产品宿主完成，不在本 crate 自动发生。

### Key Entry Points
- `DownloadManager::start_download/update_progress/mark_completed/load`：下载元数据状态链。
- `BrowserSettings::load/save`：默认目录设置文件，独立于 ProfilePaths。
- `Autocomplete::suggest`：历史、书签与搜索建议。
- `ContextMenu::with_url` / `MenuItem::segmented`：菜单来源 URL 与分段动作模型。
- 实际页面请求策略见 [响应策略](../../web-data/network/websocket-and-resource-policy.md)，UI 命中见 [桌面交互](../desktop/chrome-and-window.md)，功能验证见 [验证入口](../../../../zeroweb-verify/SKILL.md)；本次未运行下载或菜单操作。

## Gotchas
- DownloadManager 的 pause/resume/cancel 只改变元数据，不暂停网络或删除文件；load 原样恢复 Pending/Downloading 状态而不重启传输，不能据此宣称断点续传。 (`crates/browser-shell/src/download.rs`)
- 下载终态不是严格吸收态：mark_failed 可覆盖 Completed，mark_completed 可覆盖 Cancelled；update_progress 仅将 Pending 改为 Downloading，其他状态仍可更新字节数，调用方必须按真实传输结果选择操作。 (`crates/browser-shell/src/download.rs`, `crates/browser-shell/src/tests/download.rs`)
- progress 对未知或零总大小返回0，即使已下载字节非零；总大小已知时仅将比例上限钳为1，完成态本身不保证 progress=1，未知大小下载完成仍为0。 (`crates/browser-shell/src/download.rs`)
- 设置持久化用直接 fs::write，没有 profile::atomic_write 或1MiB读取限制；JSON 解析失败整份退回默认，不能把 profile 的原子替换与输入上限推断到 settings。 (`crates/browser-shell/src/settings.rs`, `crates/browser-shell/src/profile.rs`)
- BrowserSettings::search_url 仅把空格替换为加号，保留 &/#/+ 等字符；它不是通用 query percent encoder，现有特殊字符测试甚至断言原串保留，修改此行为需同时核对导航消费者。 (`crates/browser-shell/src/settings.rs`, `crates/browser-shell/src/tests/settings.rs`)
- Autocomplete 总为搜索建议预留一个槽，即使输入像 URL 不插入搜索项也会少返回一项；max_results=1 时仍保留至少一个匹配，非 URL 查询可得到搜索加匹配共两项，不能把该设置当严格结果上限。 (`crates/browser-shell/src/autocomplete.rs`)
- 菜单 find_item 只递归 SubMenu 的 children，不遍历 Segmented 的 segments；Zoom 子动作必须由宿主按分段命中分发，不能依赖通用 find_item 查到分段 ID。 (`crates/browser-shell/src/context_menu.rs`)
- 桌面 save_attachment_download 整包接收后才记进度并直接写 filename 到下载目录，写失败标 Failed；Android record_download 则用 DownloadId 前缀落盘并保存 profile，两条路径不具有相同文件覆盖和持久化语义。 (`apps/browser/src/app.rs`, `apps/android-browser/rust/src/facade.rs`)
- 桌面活动下载面板最多显示三行并折叠其余计数，绘制和按钮命中共用行高及按钮矩形；暂停/取消先于页面输入分发，但只调用元数据 pause/cancel，暂停项因不再 active 而离开面板，不能把按钮存在解释为传输暂停或提供恢复入口。 (`apps/browser/src/app.rs`, `apps/browser/src/app_input.rs`, `apps/browser/src/app_render.rs`, `apps/browser/src/layout.rs`, `crates/browser-shell/src/download.rs`, `apps/browser/src/tests.rs`)

## Architecture
- DownloadId 使用同一个 NEXT_DOWNLOAD_ID 生成及恢复推进，load 取最大 ID 的 saturating_add；下载 JSON 校验至多10000项、URL/filename至多16KiB，保存的是元数据而非文件字节或完整本地路径。 (`crates/browser-shell/src/download.rs`)
- active_count 只计 Pending/Downloading，Paused 不算活跃；remove 与 clear_completed 允许清掉 Completed/Cancelled/Failed，函数名 clear_completed 的实际范围包括失败和取消。 (`crates/browser-shell/src/download.rs`, `crates/browser-shell/src/tests/download.rs`)
- 菜单 source_url、ContextType、动作 ID、enabled 和 shortcut 是模型数据；shortcut 只是显示字符串，不注册快捷键，Action 的出现也不证明宿主实现了其行为。 (`crates/browser-shell/src/context_menu.rs`)

## Decisions
- 补全先收书签并加100分，再排历史；匹配分是 URL 前缀40或包含20，加标题前缀30或包含10，所以最弱书签也压过最强历史，跨来源同 URL 只保留书签。 (`crates/browser-shell/src/autocomplete.rs`, `crates/browser-shell/src/tests/autocomplete.rs`)
- 补全的 URL 外观判定只用于是否插入搜索建议，不等于桌面 normalize_url：含 scheme、localhost 或字母域名后缀的粗判不能承担内网/IP/非法 authority 的导航校验。 (`crates/browser-shell/src/autocomplete.rs`, `apps/browser/src/app.rs`)
- UiLanguage 按 ZERO_BROWSER_UI_LANG、LC_ALL、LC_MESSAGES、LANG 顺序寻找 zh/en 前缀；不识别的值继续查后项，最终回退英文，不是只取第一个非空环境变量。 (`crates/browser-shell/src/context_menu.rs`)
- ColorThemePreference 新字段以 serde default 兼容旧设置文件，取值以 snake_case 持久化；SearchEngine 的 from_name 则要求 Google/Bing/DuckDuckGo/Baidu 原大小写，不能统一按主题的命名规则解析。 (`crates/browser-shell/src/settings.rs`)

## Patterns
- Shell 的缩放保存在每个 Tab，set_zoom 钳在0.25至5.0；新标签固定从1.0开始，settings.default_zoom 是另一个默认设置字段，new_with_persisted_settings 的全局 zoom 回退不等于每个新标签应用该值。 (`crates/browser-shell/src/browser.rs`, `crates/browser-shell/src/tab.rs`, `crates/browser-shell/src/settings.rs`)
- apply_settings 先改内存再尝试 save_default，失败只记录 warn 不回滚；桌面 JS 开关还调用 tabs.set_javascript_enabled，而 Cookie/DNT 开关在该函数仅保存模型值，实际网络生效需另查消费者。 (`crates/browser-shell/src/browser.rs`, `apps/browser/src/app.rs`)
- ContextMenu 的固定动作 ID 与中英文文案分离，Separator 的空 ID、disabled 状态和 Segmented 父行的空 label 不能作为可点击动作处理。 (`crates/browser-shell/src/context_menu.rs`)
