# performance-api-compat — 运行时控制面

> 状态头（每轮先核对再追加，依据日期：2026-10-12）

**当前状态**：M2-S1 完成——longtask-timing 可执行面 **13/13（100%）**（M1 基线
1/17）。实现三层：sandbox `set_long_task_observer`（execute 墙钟 ≥50ms 通知）→
webview 任务边界排空发射（`drain_long_task_entries`，4 调用点）→ shim
`PerformanceLongTaskTiming`/`TaskAttributionTiming` 接口 + `__zwEmitLongtask`
（name 'self'、duration 取整、taskattribution 归 containerType 'window'）。
全量回归零红：make test 全绿（71 targets）、fmt/clippy 零告警、timing 家族
（hr-time/user-timing/performance-timeline/web-animations）与 performance-api 三
corpus 对比基线零回归（web-animations 502/952 为并行渲染流域两周漂移的涨数，
失败面无一涉及 performance 条目）。

**卡点**：bench-gate 三连跑 FAIL 集合互不相交（首跑 page/*、二跑 mb/* sandbox、
三跑 mb/* webview）、load average 4+（并行渲染流 rally 编译并发）——定性为机器
争用噪声：HEAD worktree A/B 同负载直接测量 execute_simple_expression 191-200µs
vs 本分支 183-188µs（本分支更快，基线 169µs 为静机读数），实现理论成本仅每次
execute 一次 Option clone + 空槽 mutex lock。**待静机窗口重跑 bench-gate 确认后
方可关 M2-S1**（下一轮首步）。

**下一步**：① 静机重跑定向 bench-gate（`ZERO_WEB_BENCH_CRATES=zero-script-sandbox,
zero-webview make bench-gate`）确认零回归；② M2-S2 event-timing——shim 事件派发
路径包 startTime/processingStart/processingEnd 计时点产 `PerformanceEventTiming`
条目（0/68 全缺面，M1 甄别报告 §缺面 2）。

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入 + 甄别 + 基线 | **完成**（2026-10-12） | [M1 报告](evidence/2026-10-12-m1-scope-baseline.md) |
| M2-S1 longtask 条目生成 | **实现完成**；bench-gate 静机复跑后收口 | [M2-S1 基线](evidence/2026-10-12-m2s1-longtask-green.json) |
| M2-S2 event-timing 条目生成 | 待启动 | — |
| M3 resource/navigation-timing host 管道 | 挂账（缺面分项见 M1 报告） | — |

## 基线账本

| 日期 | corpus | 子测试 Pass/Total | 备注 |
|---|---|---|---|
| 2026-10-12 | resource-timing | 10/113 | buffer 语义面自洽部分；条目生成缺面 |
| 2026-10-12 | navigation-timing | 66/95 | attributes-exist 自洽面为主；条目真值缺面 |
| 2026-10-12 | event-timing | 0/68 | PerformanceEventTiming 全缺 |
| 2026-10-12 | longtask-timing | 1/17 | PerformanceLongTaskTiming 全缺（supportedEntryTypes 例外过） |
| 2026-10-12 | longtask-timing | **13/13** | M2-S1 后全绿（containerTypes 归多 frame skip 面，可执行分母 9 案） |

## 跨流记账

- **wpt-suites.csv 共享面修复（2026-10-12）**：cookies×2 + webmessaging×1 数据行
  含未转义半角逗号（17/21/17 列），`generate-website-metrics.py` 15 列校验硬失败
  ——本轮以「整字段加引号」机械修复（内容逐字保留，已程序化核对），共享管道恢复。
  属碰头信号记录，内容归各自流。
- navigation-timing 生命周期真值与 navigation-compat 流域共享（契约依赖约束条款）；
  本 goal 只认领 PerformanceNavigationTiming 条目面自洽语义。
- **web-animations 计数漂移（2026-10-12）**：timing-animation 归档账本记 52/963
  （2026-09-28），当前 main 实测 502/952——渲染流域两周工作的组合态涨数，本 goal
  不认领、不改写彼账本（归因纪律 run-rules §10）。
