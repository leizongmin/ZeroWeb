# M3 DC-2 续：可见性消息切片（browser→renderer 前后台差分 + Page Visibility 宿主驱动）

**运行身份**：HEAD d8772f537（rAF 泵修复）+ 本轮工作区；binary `make browser-build` release 产物（2026-10-23）。
**环境**：headless 腿 CDP 采样（`bg02-anim`/`bg02-static`，`ZW_RAF_FRAME_DRIVEN=1`；`bg01-background` 双 tab 抢占）。采样 250ms 进程树 `/proc`，cores = max(cpu_jiffies)/100/wall。重跑命令见各腿 `run-config.txt`。

## 结论

1. **四层链路钉住（单测）**：protocol `SetVisibility(SetVisibilityParams{hidden})` roundtrip（id 36，末尾追加纪律）；engine shim getter 读 `globalThis.__ZW_HIDDEN` + `__zw_dispatch_visibilitychange` 派发契约（doc/window 触达、不冒泡不可取消、isTrusted、IDL `onvisibilitychange`、原子单串 handler 内读新值、裸 state 无事件）；browser poll 前后台差分出站序 `[false, true, false]`（首 tab 可见→同态不重发→切换沿旧 hidden/新 visible）；renderer `SetVisibility` 臂翻 `page_hidden` 宿主态 + worker 内 `document.hidden` 翻转（JS 关闭仍记录宿主态）。
2. **headless 生产腿零回归 + 初态可采样**：`bg02-anim` `__zw_raf` 332→1573（~52 次/s 持续，与上轮 333→1575 同带——泵门控新增 `!page_hidden` 在恒可见下零影响）；`bg02-static` renderer 0.010 / browser 0.014 cores（上轮 0.015/0.020 同带）。两腿 probe 新增 `visibility_state` 终态采样均 `false,visible`（shim getter 生产读路径实证）。
3. **headless 不激活差分链（负结果归因）**：`bg01-background` 双 tab 抢占腿 timerPage 终态仍 `false,visible`——headless 会话有自己的 renderer 驱动面（`HeadlessSession::navigate_renderer`），不经 `ProcessTabBackend::poll`，CDP `Target.createTarget` 切的是 shell tab 模型、无 poll 差分发送。该链的激活面是 GUI 应用（`app.rs` 主循环 → `TabManager::poll` → `ProcessTabBackend::poll`）。
4. **GUI tab 切换腿 NOT_RUN**（既有约束不变）：GUI 模式无 CDP，切换无自动化驱动——差分链的生产端到端验证归 M4 GUI 恢复腿（与 DC-1/DC-4 GUI 面同批）。

## 修复形态

- **protocol**（`message.rs`）：枚举末尾 append `SetVisibility(SetVisibilityParams { hidden: bool })`（bincode 判别值追加纪律）+ roundtrip 清单 id 36。
- **engine**（`script_gen.rs`）：`script_set_visibility_state(hidden)` 裸赋值（跨导航重注入用，新 Document 无过渡语义）+ `script_set_visibility(hidden)` 赋值+`__zw_dispatch_visibilitychange` 原子单串（typeof 守卫，无宿主注册环境零开销）。
- **shim**（`part06.js`）：`hidden`/`visibilityState`/`webkitHidden`/`webkitVisibilityState` 四 getter 改读 `globalThis.__ZW_HIDDEN === true`（缺省可见，语义同旧硬编码）；`__zw_dispatch_visibilitychange` 挂 globalThis（闭包内普通函数声明宿主不可见——首版单测抓出），照 `_firePointerLockEvent` document 级派发形态；`visibilitychange` 补入 `_defineDocOnHandler` 列表（`document.onvisibilitychange` IDL 可触）。
- **renderer**（`runtime.rs`）：`page_hidden` 字段（构造 false）+ `SetVisibility` 派发臂 + `handle_set_visibility`（dedupe 省略——browser 差分是唯一发送方；`execute_script_priority_bounded` + `USER_ACTION_SCRIPT_TIMEOUT`，同 handle_set_viewport 形态）；readyState loading 注入点跨导航重注入（`page_hidden` 时裸 state 脚本，无事件）；rAF 泵条件加 `!self.page_hidden`（恢复可见自动续泵——挂起数仍在）。
- **browser**（`process_backend.rs`）：`visible_tab: Option<TabId>` 差分基线（构造 None）+ `set_tab_visibility`（`send_to_renderer` 单播，renderer 不 live 自然丢弃）+ poll 头部差分（`visible_tab != active_tab` 时旧前台 hidden=true / 新前台 hidden=false / 更新基线；首 poll 对 active_tab 发一次 hidden=false——headless 单 tab 语义）。
- **观测资产**（`background-resource-probe.mjs`）：bg02 族与 bg01-background 腿新增 `visibility_state`（及 preemptor 版）终态采样——后台节流切片的页面侧真值面。

## 腿结果（30s，cores = max(cpu_jiffies)/100/wall）

| 腿 | 采样 | renderer | browser | 说明 |
|---|---|---|---|---|
| bg02-anim（headless，rAF 自旋） | `__zw_raf` 332→1573（~52/s） | 0.031 | 0.013 | 与上轮（333→1575 / 0.049 / 0.015）同带 |
| bg02-static（headless，无 rAF） | visibility `false,visible` | 0.010 | 0.014 | 上轮 0.015 / 0.020 同带，零回归 |
| bg01-background（双 tab 抢占） | timerPage `false,visible`（见结论 3） | 0.059 | 0.012 | `__zw_iters` 1520→6328（DC-1 钳制带） |

## 已知缺口（后续切片）

1. **GUI 差分链端到端 NOT_RUN**：GUI 模式无 CDP 自动化驱动，tab 切换 → hidden 翻转的生产证据归 M4 GUI 腿（与 DC-1/DC-4 GUI 恢复面同批）。
2. **后台 tab renderer 重生错误 visible**：后台 tab 的 renderer 崩溃重生后 `page_hidden` 回 false 初态，直到下次前台切换差分才纠正（重注入只挂 readyState loading，重生保文档路径不触发）；窗口极小，记档不修。
3. **SW client visibilityState**（`service_worker_manager.rs` Clients.get().visibilityState）不在本切片；窗口级 Minimized/Occluded 跟踪不存在。
4. **后台工作节流本体未做**：本切片只交付「可见性真值 + 通知」，后台 rAF 已由泵门控消费 `page_hidden`；后台 timer/动画节流（DC-1「后台策略明确且恢复正确」的页面策略面）是下一切片。
