# 后台资源生命周期控制面

**日期**：2026-10-22（M1 基线轮）
**状态**：Active；M1 四类场景资源基线已建立，M2 归因未开始。
**当前卡点**：无阻塞。M2 首切片待选（候选按证据强度排序见下）。
**下一步**：M2——从 BG-03 stall fetch 连接滞留切入归因首个增长源（server 侧直接观测、复现最小），随后追 renderer RSS 线性增长根因。
**入口**：[目标契约](../background-resource-lifecycle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 嵌套 timer、后台与视口外动画策略 | M1 已测明现状：无钳制、无后台策略、动画呈现链未激活 | PARTIAL（修复未开始） |
| 重复导航旧文档/回调/资源释放 | stall fetch 连接滞留 + RSS +224MB 已测得；归因真实 owner | 归因 NOT_STARTED |
| 关闭后的连接/队列/进程生命周期 | renderer 进程复用 +163MB 不回落已测得；GUI 前台恢复腿 NOT_RUN | PARTIAL |
| CPU/RSS/时长/预热门禁 | M1 协议已固定（run-config + sampler 250ms）；阈值待多轮配对 | 待 M4 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 四类场景与生产资源基线 | **DONE**（2026-10-22，12 腿 headless+GUI 双口径） | [M1 基线](evidence/2026-10-22-m1-baseline/ANALYSIS.md) |
| M2 | 首个增长来源、共享 owner 修复和回归 | NOT_STARTED | 无 |
| M3 | 空闲/视口外调度及恢复 | NOT_STARTED | 无 |
| M4 | 资源门禁、持续配对复测与清理 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BG-01-nested-and-hidden-timers | PARTIAL：headless 面已测（无钳制 ~20 万 iter/s；前后台无差异 1.46/1.54 cores）；GUI 前后台恢复 NOT_RUN | M1 基线 ANALYSIS §DC-1 |
| DC-2 | BG-02-offscreen-and-idle-present | PARTIAL：静止免呈现成立（2 帧/30s）；动画呈现链未激活 →「合法工作不误停」INCONCLUSIVE；音视频 NOT_RUN | M1 基线 ANALYSIS §DC-2 |
| DC-3 | BG-03-repeat-navigation-stalled-resource | PARTIAL：stall 连接随导航滞留（1→6，peer 关闭 0）+ renderer RSS +224MB 已测得；根因未归因 | M1 基线 ANALYSIS §DC-3 |
| DC-4 | BG-04-close-and-resume | PARTIAL：destroyed 回执 20/20；renderer 进程复用 +163MB 不回落；GUI 恢复 NOT_RUN | M1 基线 ANALYSIS §DC-4 |
| DC-5 | 固定协议长期复测 | PARTIAL：协议已固定可复现；阈值与配对节奏待 M4 | M1 基线 ANALYSIS §DC-5 |

## 碰撞与执行记录

本轮（2026-10-22 M1）只新增脚本、fixture 与文档，未触碰共享源码面（browser/renderer/page-runtime/engine/compositor 无改动）；并行流 rendering-compat（ZeroWeb-2）、html-semantics-compat（~/work/ZeroWeb）工作面不重叠。M2 修 owner 时按 run-rules §9–11 先核对占用。

观测资产：`scripts/background-resource-observe.sh`（单腿编排，GUI 腿 `BG_OBSERVE_GUI` 场景名 `*_gui`/`*_compgui`）+ `background-resource-probe.mjs`（CDP 驱动，零依赖）+ `background-resource-sampler.py`（进程树采样）+ `background-resource-fixture-server.py`（stall 子资源）+ `tests/fixtures/background-resource/`（7 fixture）。运行守卫：每腿 timeout 墙钟 + 24G systemd scope，采样仅统计本次 browser 后代进程树。

本轮已清理自己创建的实例（browser/http/sampler 进程随腿收尾）；frame dump .bin 已清理留 manifest。无运行残留。
