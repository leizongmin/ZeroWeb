# M4 GUI 生命周期腿：DC-3 重复导航 + DC-4 close/reopen，hidden 呈现链缺口第二收口

日期：2026-10-23。承接 [M4 GUI tab 切换腿](../2026-10-23-m4-gui-tab-switch/ANALYSIS.md)（DC-1
端到端 + tab 隐藏维度首轮）。本轮把双形态编排推到 DC-3（重复导航资源释放）与 DC-4
（close/reopen 生命周期）两条剩余验收场景，并在 close/reopen 形态抓到 hidden 呈现链
收口的**启用缺口**：`page_hidden` 声明维度已合入但从未被消费——修复它在两个形态落证。
编排 `scripts/background-resource-gui-lifecycle.sh`（xdotool 形态，判别面是 RSS/连接/
帧恢复，ui-driver 强制成帧不适用）；分段统计 `background-resource-phase-stats.py`
（本轮加 RSS 列：段内首末/高水位 MiB）。

## 腿清单

| 腿 | 目录 | 场景 | 角色 |
|---|---|---|---|
| DC-3 12 周期 | `bg03-nav-gui/` | 同 tab 地址栏导航 source/target 交替 ×12 | 功能 + RSS 趋势 |
| DC-3 40 周期 | `bg03-nav-long/` | 同上 ×40 | RSS 有界性判别 |
| DC-4 修前对照 | `bg04-close-gui-preFix/` | Ctrl+T 开页 + Ctrl+W 关 ×20 | 抓 compositor 会话断链 |
| DC-4 客户端修复后 | `bg04-close-gui/` | 同上 ×20 | 断链修复验证（暴露 keep-alive 缺口） |
| DC-4 双修复后 | `bg04-close-gui-visrefresh/` | 同上 ×8 | 可见性刷新 + 探测重臂验证 |
| tab 切换验证 | `bg02-gui-tab-switch-visrefresh/` | Ctrl+T/Ctrl+Tab 三段 | 同修复在 tab 切换形态落证 |

## DC-3：重复导航 GUI 腿（成立）

40 周期腿（`bg03-nav-long/`）判定真实宿主地址栏导航路径双侧有界：

- renderer RSS 爬坡至 HWM 228 MiB（周期 25 封顶），其后锯齿 188–228 MiB 有界带——
  与 M2-S2 headless 判定的「爬坡 + 有界平台」同构，非无界增长。
- browser RSS 周期 23 起 ≈540 MiB 恒定（present 缓存稳态，M3 DC-2 已记档）。
- stall 连接 21/21 `closed_by_peer`（fixture `/__stats`）——M2-S1 fetch 取消修复在
  真实导航路径成立，无连接滞留。
- 文档残留计数与 M2-S3 同构：`fetch_abandoned` = source→target 次数、
  `timers_pending` 封顶、`commands_retained` 仅启动行非零。

## DC-4 修前对照：compositor 会话级断链（已修）

`bg04-close-gui-preFix/`（Ctrl+T 路径，新 tab surface 注册前的 Get 命中 compositor
对未注册 surface 的 epoch=0 空帧响应）抓到 `apps/browser/src/compositor_client.rs`
致命臂缺陷：Get 期望 Data 时收到 (surface, epoch) 不匹配的 `CompositorFrameData`
被当通道损坏 → 会话级断链 → compositor 退出且无重启 → phase3 恢复段呈现链全无
（compositor 缺席 / browser 0.002 cores）。

修复：该响应是「非该帧」的合法协议响应（新 tab 竞态 / busy 窗前台帧），丢弃本
响应即可——请求/响应 FIFO 仍一一配对，Get 方下轮重拉。配套 2 测试：空帧丢弃 +
非期望 kind 仍致命（负例）。修复后腿 `bg04-close-gui/`：0 断链行、单一 compositor
pid 存活全程、96 次光栅、phase3 恢复 0.573 cores 回前台带。

## DC-4 第二缺口：page_hidden 维度从未被消费（本轮主修复）

`bg04-close-gui/` 暴露：hidden 窗 browser ~4.5–5.0 cores（应如 tab 切换腿收口）。
ZW_LOG_LEVEL=debug 诊断腿判定：

- tab1 keep-alive GetFrame 在每个 3.6s hidden 窗全速率（零门控）；
- surface 1 的空声明集直到 phase4 导航（46s，**hidden 窗之外**）才发出——hidden
  窗内 renderer 根本没有 publish，空集无从发送。

根因（`apps/renderer/src/runtime.rs`）：`visible_declarations` 的 `page_hidden`
维度只在 publish 末尾的发送单点消费；`handle_set_visibility` 只设宿主态 + 派
visibilitychange 脚本，**不触发 publish**。空闲 hidden CSS 动画页翻转后无 rAF
（泵按 hidden 门控停）、无 timer、无输入 → 永远等不来下一次 publish →
`animations_active` 恒真 → keep-alive 空转。tab 切换与 close/reopen 两形态同病。

**重要更正**：上一轮 [tab 切换腿](../2026-10-23-m4-gui-tab-switch/ANALYSIS.md) B5
腿的 phase2 差分（compositor −38% / browser −50%）**不是** page_hidden 修复的效果
——该机制当时根本无法启用。复核 B4/B5 全部分段：B5 在修复无关的段（load、phase3）
browser 也比 B4 低 ~25–30%，差分主体是腿级基线漂移；其 phase2 中段的 compositor
塌缩 + 回升曲线在同代码的今日 debug 腿中不复现（全程全速率），归不可复现的环境
波动。tab 切换腿的「缺口」判定与修复方向正确，B5 的差分数字作废，以本轮
`bg02-gui-tab-switch-visrefresh/` 为准。

### 修复一：renderer 可见性翻转即刷新声明（`refresh_compositor_animation_state`）

`handle_set_visibility` 在设 `page_hidden` 后、JS 门控前调用新方法：用最近一次
渲染的声明缓存（`last_render.animation_declarations/scopes/transforms`）直接走
`send_compositor_animation_state` 发送单点——advance 判变：hidden 发空集清理、
恢复集合差分重现且 generation 递增。不触发帧事务、无重渲染。恢复方向对称必要：
隐藏期间 compositor 声明已清空，无焦点事件的恢复（Ctrl+W 关 tab 无 DOM focus
事件）否则动画冻结到下一次输入。测试
`set_visibility_refreshes_compositor_animation_declarations_without_publish`
（真渲染缓存 + 真 SetVisibility IPC 链 + 真 发送断言）。

### 修复二：browser 非空声明到达重臂 anim 探测（自举死锁第二变体）

修复一单独生效时 `bg04-close-gui` 首轮复测暴露恢复段死亡（phase3 0.000/0.003）：
恢复方向的声明重现不产生新 frame 代（发送单点、无帧事务），browser 探测以
「每新提交 frame 代一次」为前置（M3 DC-2 自举死锁修复）不会自发再开 → flag 停在
false → keep-alive 不恢复。`process_backend.rs` 的 `CompositorAnimationState` 中继
臂：非空声明到达且 snapshot flag 已落 false 且 surface 匹配时清
`compositor_anim_probe_frame` 重臂一次——flag 仍以 Get 响应采样为唯一产生源，只重开
探测机会。测试 `animation_state_rearms_anim_probe_on_nonempty_declarations`
（非空重臂 + 空集不重臂负例）。

### 顺带修复：renderer 测试 env 竞态（假红）

`animation_export_tests` 三个 compositor_state 测试在负载下随机假红（stash 复核
确认先于本轮改动）：`ZW_COMPOSITOR_ANIMATIONS` 是进程级环境值，cargo 并行测试
线程下 set/remove 与其他测试的构造（with_io 读 env 下推 webview）、发送（每次调用
读 env）互踩。加模块级 `env_guard()` 互斥锁，依赖 env 的测试首行持锁覆盖全程；
三连跑全绿且模块耗时 5.05s（超时主导）→ ~0.5–1.1s。

## 修复验证

### DC-4 close/reopen（`bg04-close-gui-visrefresh/`，8 周期）

按 renderer-1 visibility 翻转精确切分 hidden 窗（每周期 4.4s，8/8）：

| 段 | zero-browser | zero-compositor | 修前同形态（`bg04-close-gui/`） |
|---|---|---|---|
| hidden 窗（avg/min，8 窗） | **0.52–0.66 / 0.00** | **0.01–0.12 / 0.00** | ~4.5–5.0 / ~0.5 |
| phase3 恢复（动画回前台） | 4.314 | 0.578 | 0.003（链未停） / — 修前首轮 0.000（链死） |
| phase4 地址栏导航输入 | 0.955 | 0.125 | 正常 |
| renderer-1 RSS（全程） | 116→126 MiB 有界 | — | — |

hidden 窗 browser 降 ~8×、compositor 降 ~20×；min 0.00 = 稳态零空转，窗均值残量
是翻转边界瞬态（~1s 传播 + 恢复重臂）。恢复段回前台同带——停止与恢复双向成立。

### tab 切换形态（`bg02-gui-tab-switch-visrefresh/`，同修复）

| 段 | zero-browser | zero-compositor | B4 判别腿（修前） |
|---|---|---|---|
| phase2 深隐藏 15s | **0.003** | **0.000** | 4.918 / 0.593 |
| phase3 恢复 10s | 4.325 | 0.579 | 4.371 / 0.570 |
| hidden 全窗 17.4s（avg） | 0.14 | 0.00 | — |

深隐藏窗 browser 降 ~1600×，residual 归零；恢复段与 B4 前台带逐位同带。
（腿结构注：本编排 phase1 锚点后 ~19ms 即 Ctrl+T，phase1 窗实为 hidden 头 2.4s，
B4/B5 同构——B4/B5 的 phase1 高读数正是缺口证据：hide 后链仍全转。）

## 已知边界

- 翻转即刷新使隐藏期间 `getComputedStyle` 覆盖缺失、动画时间轴重启（恢复按新锚点
  重开始）——与 M3 offscreen 门控同族已知边界，CSS Animations 规范本身允许
  （display:none 元素动画时间轴同样重启）。
- 探测重臂以 surface_id 匹配 snapshot；stale epoch 的声明被 compositor 丢弃，
  重臂探测多付一次 Get 后回落，无正确性影响。
- hidden 窗均值残量（DC-4 形态 ~0.55）为翻转瞬态 + 窗口 chrome 基线，tab 切换
  形态深窗已归零；DC-4 形态窗更短（4.4s）故瞬态占比更高。
- xdotool 形态方法学边界同前轮（可复现性弱于 ui-driver，仅判别）。
