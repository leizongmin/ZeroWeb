# performance-api-compat — 运行时控制面

> 状态头（每轮先核对再追加，依据日期：2026-10-12）

**当前状态**：M1 完成（语料导入 + runner 通道 + 可执行面甄别 + 基线落账）。
基线 **77/293 子测试（26.3%）**，可执行分母 145 案（248 案面减结构性 skip）。
四 corpus 分列：resource-timing 10/113（8.8%）、navigation-timing 66/95（69.5%）、
event-timing 0/68（0.0%）、longtask-timing 1/17（5.9%）。

**卡点**：无阻塞。四缺口甄别完成（见 evidence M1 报告）——longtask/event-timing
可在 shim 层打通（任务循环计时点 / 事件派发计时点均 JS 可控）；resource-timing
需 host→JS entry buffer 钩子（M3）；navigation-timing 真值挂 navigation-compat 流域。

**下一步**：M2-S1 longtask-timing——shim 任务包裹产 `PerformanceLongTaskTiming`
条目（>50ms 判定 + buffered 回放 + toJSON/TaskAttributionTiming 接口面），目标
longtask 分案转绿；随后 M2-S2 event-timing 同批处理。

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入 + 甄别 + 基线 | **完成**（2026-10-12） | [M1 报告](evidence/2026-10-12-m1-scope-baseline.md) |
| M2-S1 longtask 条目生成 | 待启动 | — |
| M2-S2 event-timing 条目生成 | 待启动 | — |
| M3 resource/navigation-timing host 管道 | 挂账（缺面分项见 M1 报告） | — |

## 基线账本

| 日期 | corpus | 子测试 Pass/Total | 备注 |
|---|---|---|---|
| 2026-10-12 | resource-timing | 10/113 | buffer 语义面自洽部分；条目生成缺面 |
| 2026-10-12 | navigation-timing | 66/95 | attributes-exist 自洽面为主；条目真值缺面 |
| 2026-10-12 | event-timing | 0/68 | PerformanceEventTiming 全缺 |
| 2026-10-12 | longtask-timing | 1/17 | PerformanceLongTaskTiming 全缺（supportedEntryTypes 例外过） |

## 跨流记账

- **wpt-suites.csv 共享面修复（2026-10-12）**：cookies×2 + webmessaging×1 数据行
  含未转义半角逗号（17/21/17 列），`generate-website-metrics.py` 15 列校验硬失败
  ——本轮以「整字段加引号」机械修复（内容逐字保留，已程序化核对），共享管道恢复。
  属碰头信号记录，内容归各自流。
- navigation-timing 生命周期真值与 navigation-compat 流域共享（契约依赖约束条款）；
  本 goal 只认领 PerformanceNavigationTiming 条目面自洽语义。
