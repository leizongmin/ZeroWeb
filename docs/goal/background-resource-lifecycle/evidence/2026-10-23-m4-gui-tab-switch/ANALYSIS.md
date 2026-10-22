# M4 GUI tab 切换腿：DC-1 端到端 + hidden 呈现链缺口与 page_hidden 收口

日期：2026-10-23。目标：在真实 zero-browser 窗口（无 CDP 自动化）下取得 DC-1（后台
timer 钳制）的 GUI 前后台差分端到端证据，并检验 DC-2 呈现链在「tab 切走」这一
页面级隐藏维度上的行为。编排脚本 `scripts/background-resource-gui-tab-switch.sh`，
分段分析 `scripts/background-resource-phase-stats.py`（段锚点 = UI_DRIVER_STEP log
或 xdotool 形态的 anchors.txt）。

## 腿清单与时序

三段时序：前台 10s → Ctrl+T 开 bg02-static.html 转 hidden 15s → Ctrl+Tab 切回 10s。

| 腿 | 目录 | 形态 | 角色 | 时序 |
|---|---|---|---|---|
| A | `bg01-gui-tab-switch/`（05:09） | ui-driver | DC-1：1ms interval 页前后台差分 | 修复前后同代码 |
| B2 | `bg02-gui-tab-switch/`（05:10） | ui-driver | 方法学对照（被判别为失灵敏，不计证据） | 修复前 |
| B4 | `bg02-gui-tab-switch-xdo/`（05:18） | xdotool | 判别腿：抓 hidden 呈现链缺口 | 修复前 |
| B5 | `bg02-gui-tab-switch-fixed/`（05:25） | xdotool | 修复验证腿 | release 05:22 重建后 |

双形态方法学：ui-driver 每 tick 强制 `needs_redraw`，browser 背景 ~9.5 cores +
compositor 持续空帧，把呈现链差分淹没（B2 实测印证）——判别腿必须用 xdotool 形态
（browser 正常事件循环，`windowactivate` 保焦点，tab 切换不失焦）。

## 腿 A：DC-1 timer 钳制 GUI 端到端（成立）

bg01-interval-timer.html（setInterval 1ms 计数），COMP_ANIM=0（不涉及 compositor
动画）。renderer-1 为 timer 页 tab：

| 段 | renderer-1 cores | 说明 |
|---|---|---|
| phase1 前台 | 0.052 | 1ms 派发全速 |
| phase2 hidden | 0.004 | **13× 塌缩**（钳 1s 生效） |
| phase3 恢复 | 0.046 | 回带前台同带 |

DC-1 的后台 timer 钳制在真实宿主 tab 切换路径（poll 前后台差分 → SetVisibility
单播 → renderer `__ZW_HIDDEN` → timer 派发门控）端到端生效，且切换后恢复。

## 判别腿 B4：hidden tab 呈现链空转缺口（修复前）

bg02-onscreen-anim.html（视口内 CSS 动画，COMP_ANIM=1）前台段 compositor
0.552→0.630 同带；Ctrl+T 切走后（tab1 hidden 15s，前台为 static 页）：

| 段 | zero-compositor | zero-browser |
|---|---|---|
| phase2 hidden | **0.593 cores** | 4.918 |

tab1 已不可见，compositor 仍持续消耗——offscreen 视口门控按本帧 paint 作用域剔除
声明，但 tab 隐藏页的动画声明照发、`animations_active` 保持置位，browser keep-alive
Get 循环以其为前置持续对不可见 surface 发 Get。帧 dump（512 帧配额已被前台段耗尽）
phase2 窗口 0 帧——空转不在帧产出路径，属 Get→prepare 被可见性抑制仍重复进入的成本。

## 修复：visible_declarations 增加 page_hidden 维度

`apps/renderer/src/paint_export.rs` `visible_declarations` 增加 `page_hidden: bool`
参数：hidden 页无采样面，直接返回空声明集 → `animations_active` 不置位 → keep-alive
前置失效自动停 Get。与 offscreen 维度同构（集合差分恢复：切回后 hidden=false 首帧
声明重现，generation 递增）。`runtime.rs` advance 输入侧传 `self.page_hidden`
（`handle_set_visibility` 维护）。测试：纯函数测 hidden 臂 + 集成测
`compositor_state_clears_declarations_while_page_hidden`（可见发声明 → hidden 发空集
→ 恢复重现递增）。

## 修复验证腿 B5（phase2 前台同为 static 页，唯一差异是 hidden tab1 动画链）

| phase2（tab1 hidden 15s） | B4 修复前 | B5 修复后 | Δ |
|---|---|---|---|
| zero-compositor | 0.593 | 0.370 | **-38%** |
| zero-browser | 4.918 | 2.448 | **-50%** |
| renderer（两 tab） | 0.005 / 0.005 | 0.005 / 0.005 | — |

恢复与链路证据：

- B5 phase3（Ctrl+Tab 切回）compositor 0.565 vs phase1 前台稳态 0.605 同带——恢复后
  呈现链正常重建，集合差分路径无回归。
- browser.log visibility 翻转四次对称配对（hidden=true/false 成对出现在切走/切回，
  renderer-1/renderer-2 各自单播），SetVisibility 链路按 tab 精确到达。

残留归因：B5 phase2 的 0.370 / 2.448 与 phase3（前台动画页 0.565 / 3.681）同量级，
且为修复前后两腿共享的窗口级基线（前台 static tab 呈现、tab 栏 chrome、poll 主循环），
不属于 hidden tab 的动画链——本轮收口对象（hidden 动画链空转）已消除，残留为
窗口基线成本，归 M5+ 常态基线课题。

## 已知边界

- ui-driver 形态失灵敏：每 tick 强制成帧抬高背景，只能做功能腿（tab 切换路径、
  fixture 导航），不能做 CPU 判别。
- 帧 dump 512 帧上限在前台段耗尽，phase2 后段无直接帧样本；本结论以同段 CPU 差分
  为主证据（前台 static 页同构对照，差分唯一来源是 hidden tab1 动画链）。
- B5 phase2 dump 窗口内 77 帧 / 2.4s 是 SetVisibility 单播到达前的最后一次前台
  动画窗口（poll 差分周期 ~1s × ~40fps），非 hidden 后成帧。
- xdotool 注入依赖 X 层窗口管理器行为（title 子串匹配窗口、windowactivate 保焦点），
  腿可复现性弱于 ui-driver，仅用于判别。
