# 后台资源生命周期控制面

**日期**：2026-10-22（M2-S1 修复轮）
**状态**：Active；M1 基线 + M2-S1 挂起 fetch 取消路径已落地并生产复测通过；DC-3 RSS 增长面维持 PARTIAL。
**当前卡点**：无阻塞。
**下一步**：M2-S2——renderer RSS 随导航 +216MB 增长主因归因（stall fetch 连接面已由 M2-S1 排除；沿 page-runtime/renderer 文档替换路径查，复用 bg03-nav 腿 + RSS 采样定位）。
**入口**：[目标契约](../background-resource-lifecycle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 嵌套 timer、后台与视口外动画策略 | M1 已测明现状：无钳制、无后台策略、动画呈现链未激活 | PARTIAL（修复未开始） |
| 重复导航旧文档/回调/资源释放 | M2-S1 连接滞留已修复（stall_total 封顶 6→16 逐次新建、closed_by_peer 0→15、active 0↔1）；RSS +216MB 增长主因待归因（stall fetch 面已排除） | PARTIAL（S2 归因中） |
| 关闭后的连接/队列/进程生命周期 | renderer 进程复用 +163MB 不回落已测得；GUI 前台恢复腿 NOT_RUN | PARTIAL |
| CPU/RSS/时长/预热门禁 | M1 协议已固定（run-config + sampler 250ms）；阈值待多轮配对 | 待 M4 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 四类场景与生产资源基线 | **DONE**（2026-10-22，12 腿 headless+GUI 双口径） | [M1 基线](evidence/2026-10-22-m1-baseline/ANALYSIS.md) |
| M2 | 首个增长来源、共享 owner 修复和回归 | **PARTIAL**：S1 挂起 fetch 取消路径修复 + 生产复测通过（2026-10-22）；S2 RSS 增长归因待启动 | [M2-S1](evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md) |
| M3 | 空闲/视口外调度及恢复 | NOT_STARTED | 无 |
| M4 | 资源门禁、持续配对复测与清理 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BG-01-nested-and-hidden-timers | PARTIAL：headless 面已测（无钳制 ~20 万 iter/s；前后台无差异 1.46/1.54 cores）；GUI 前后台恢复 NOT_RUN | M1 基线 ANALYSIS §DC-1 |
| DC-2 | BG-02-offscreen-and-idle-present | PARTIAL：静止免呈现成立（2 帧/30s）；动画呈现链未激活 →「合法工作不误停」INCONCLUSIVE；音视频 NOT_RUN | M1 基线 ANALYSIS §DC-2 |
| DC-3 | BG-03-repeat-navigation-stalled-resource | PARTIAL：连接滞留已修复并生产复测（stall_total 16/逐次新建、closed_by_peer 15、active 0↔1，M2-S1）；RSS +216MB 随导航增长未变，主因归因中 | [M2-S1](evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md) + M1 基线 §DC-3 |
| DC-4 | BG-04-close-and-resume | PARTIAL：destroyed 回执 20/20；renderer 进程复用 +163MB 不回落；GUI 恢复 NOT_RUN | M1 基线 ANALYSIS §DC-4 |
| DC-5 | 固定协议长期复测 | PARTIAL：协议已固定可复现；阈值与配对节奏待 M4 | M1 基线 ANALYSIS §DC-5 |

## 碰撞与执行记录

M1 轮（2026-10-22）只新增脚本、fixture 与文档，未触碰共享源码面。M2-S1 轮改动面：`crates/net`（fetch_scheduler/resource_loader）+ `apps/renderer`（js_worker/runtime）+ `apps/browser`（fetch_proxy）+ fixture server 观测增强——与并行流 rendering-compat（ZeroWeb-2，css/layout/render 面）、html-semantics-compat（~/work/ZeroWeb）工作面不重叠；全量 make test 轮次与兄弟流重负载交叠出现 3 例已知负载敏感瞬态红（IndexedDB 事务 / js_dom 墙钟 / SW 激活轮询），同树首轮对应腿全绿 + 隔离复跑绿归因收口（同 R5085/R4971 判例），非本改动面。本轮复测腿自建实例已随腿收尾。

观测资产：`scripts/background-resource-observe.sh`（单腿编排，GUI 腿 `BG_OBSERVE_GUI` 场景名 `*_gui`/`*_compgui`）+ `background-resource-probe.mjs`（CDP 驱动，零依赖）+ `background-resource-sampler.py`（进程树采样）+ `background-resource-fixture-server.py`（stall 子资源；M2-S1 起 hold 循环轮询对端 FIN，`stall_closed_by_peer` 成为取消生效的直接观测信号）+ `tests/fixtures/background-resource/`（7 fixture）。运行守卫：每腿 timeout 墙钟 + 24G systemd scope，采样仅统计本次 browser 后代进程树。

frame dump .bin 已清理留 manifest（M1）；browser.log/http.log 按 `*.log` 忽略约定不入库。无运行残留。
