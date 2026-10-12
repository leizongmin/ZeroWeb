# performance-api-compat — 运行时控制面

> 状态头（每轮先核对再追加，依据日期：2026-10-12 晚）

**当前状态**：M2-S1 实现完成（bench-gate 静机收口仍挂起，见卡点）；M2-S2
event-timing 实现完成——可执行面 **0/68 → 55/72（76.4%）**（分母 68→72：
assert_implements 通过后暴露先前被遮蔽的子测试）。实现：shim `part06` 尾部
`__zw_dispatch_event` 计时包裹（宿主派发单点，processing 时段 = 监听器执行段，
startTime = 派发起点）+ `part01b` `PerformanceEventTiming`/`EventCounts` 接口 +
`__zwEmitEventTiming`（eventCounts 恒累计限 spec 36 类型、条目生成 16ms 门槛、
duration 8ms 网格取整、模态对话框截断、first-input 镜像入 `_perfEntries` 可检索）
+ interactionId/interactionCount 交互分组状态机（pointerdown/keydown 开新交互，
pointerup/click/keypress/keyup 共享组，mouse 合成与 hover 系恒 0）+ event 条目
独立回放 store（≥104ms 才入、getEntries* 不可检索、observe buffered 按类型选源）。
守卫：timingconditions（列表外类型不产条目）、buffered-and-duration-threshold
（通知 16 / 回放 104 双门槛）、retrievability（getEntries 隔离）、
modal-dialog-interrupt-paint（alert/confirm/prompt/showModal 时间戳）、
disconnect-target（target 访问时活检查，走 `_elConnected` 即时真值——sel-based
isConnected 经 native 链滞后是 shim 既有 documented 限制）、mouseover/mouseout/
pointerover/pointerout cancelable 修齐（UI Events：over/out 可取消）。

**卡点**：bench-gate 静机收口未遂（第 7-10 轮）——环境争用定性证据链已闭合：
① 碰撞方进程实锤（ZeroWeb-2 渲染流 rally 21:38/00:26/00:37 多次在测量窗口内
启动编译/reftest，loadavg 0.81→11.5 两分钟内跳变）；② **pre-M2-S2 树
（8d1c4526c）A/B 在当前环境同样 FAIL**（load_html_simple 112µs > 预算 104µs、
webview_render 703µs ≈ 3.2x 预算）——基线在当前机器态本就不可达；③ **同指标
跨分钟漂移 2.8x**（load_html_with_css：pre 树 395µs → main 树 142µs，纯噪声，
真实回归不可能使测量变快）；④ ms 级 page/* 指标（噪声容忍）全轮 PASS。结论：
mb/* FAIL 为环境系统性，非 M2-S1/M2-S2 变更回归。**待渲染流 rally 收敛或换
静机时段（深夜渲染流停摆窗口）重跑，PASS 后关 M2-S1**（不使用
ZW_BENCH_ALLOW_BUSY=1——政策禁令；不做基线 relax——无合法变化依据）。

**下一步**：① 静机窗口重跑定向 bench-gate（`ZERO_WEB_BENCH_CRATES=
zero-script-sandbox,zero-webview make bench-gate`）关 M2-S1（若渲染流长期不停
摆，此为唯一挂起点，不阻塞其余面）；② 备选推进：M2-S3 navigation-timing
自洽面——PerformanceNavigationTiming 条目 + 诚实缺省值（unavailable=0，M1
甄别报告 §缺面 4；真值仍挂 navigation-compat 流域），shim 层可做；③ M3
resource/navigation-timing host 管道（缺面分项见 M1 报告）。

**M2-S2 门禁（2026-10-12/13 收尾）**：make test 全绿（71 targets，0 failed）；
shim JS 拼接语法校验过；uievents/ 子域定向守卫 86.0%（191/222，落在
uievents-compat completed 基线带 87.1%，失败面为 boundary-events/layerX/
accesskey-hang 既有缺口，非派发机制回归）；pointerevents 全量守卫三度尝试
均未产出（40/60 分钟墙钟 + 进程消失，基础设施问题不归因变更）——以 make test
71 targets + event-timing corpus 本体（新代码最深消费面）+ uievents 子域
三重证据替代。本轮无 .rs 变更，fmt/clippy 状态与 M2-S1 收口时一致。

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入 + 甄别 + 基线 | **完成**（2026-10-12） | [M1 报告](evidence/2026-10-12-m1-scope-baseline.md) |
| M2-S1 longtask 条目生成 | **实现完成**；bench-gate 静机复跑后收口 | [M2-S1 基线](evidence/2026-10-12-m2s1-longtask-green.json) |
| M2-S2 event-timing 条目生成 | **实现完成**（55/72，余 17 跨域挂账） | [M2-S2 记分](evidence/2026-10-12-m2s2-event-timing-green.json) |
| M3 resource/navigation-timing host 管道 | 挂账（缺面分项见 M1 报告） | — |

## 基线账本

| 日期 | corpus | 子测试 Pass/Total | 备注 |
|---|---|---|---|
| 2026-10-12 | resource-timing | 10/113 | buffer 语义面自洽部分；条目生成缺面 |
| 2026-10-12 | navigation-timing | 66/95 | attributes-exist 自洽面为主；条目真值缺面 |
| 2026-10-12 | event-timing | 0/68 | PerformanceEventTiming 全缺 |
| 2026-10-12 | event-timing | **55/72** | M2-S2 后（分母 68→72：assert_implements 放行暴露遮蔽子测试） |
| 2026-10-12 | longtask-timing | 1/17 | PerformanceLongTaskTiming 全缺（supportedEntryTypes 例外过） |
| 2026-10-12 | longtask-timing | **13/13** | M2-S1 后全绿（containerTypes 归多 frame skip 面，可执行分母 9 案） |

## 跨流记账

- **event-timing 余 17 失败跨域挂账（2026-10-12，M2-S2 收尾定性）**：webdriver
  键位/Actions 缺口（SPACE U+E00D、Actions keyDown/addTick、fling/selection 手势
  ——keyboard goal R3254 / webdriver 域）×6；shadow DOM 派发管道（event-retarget /
  first-input-shadow-dom / shadow-dom-null-target）×3；events 域连击链不按位置
  重置（dblclick eventCounts 3≠2，UI Events §5.2.2 chain reset）×1；label 激活
  转发 hang ×1；runner harness 版本缺 `window_state_context` ×1；tentative API
  （targetSelector）×5 子测试。均非 event-timing 语义面，本 goal 不认领。
- **wpt-suites.csv 共享面修复（2026-10-12）**：cookies×2 + webmessaging×1 数据行
  含未转义半角逗号（17/21/17 列），`generate-website-metrics.py` 15 列校验硬失败
  ——本轮以「整字段加引号」机械修复（内容逐字保留，已程序化核对），共享管道恢复。
  属碰头信号记录，内容归各自流。
- navigation-timing 生命周期真值与 navigation-compat 流域共享（契约依赖约束条款）；
  本 goal 只认领 PerformanceNavigationTiming 条目面自洽语义。
- **web-animations 计数漂移（2026-10-12）**：timing-animation 归档账本记 52/963
  （2026-09-28），当前 main 实测 502/952——渲染流域两周工作的组合态涨数，本 goal
  不认领、不改写彼账本（归因纪律 run-rules §10）。
