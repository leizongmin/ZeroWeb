# 后台资源生命周期控制面

**日期**：2026-10-22（DC-1 嵌套钳制修复轮）
**状态**：Active；M2 三步全 DONE；DC-1 页面 timer 嵌套 4ms 钳制修复 + 生产复测 DONE（~237 iter/s，修前 ~20 万，~840× 降；renderer CPU 1.46→0.05 cores）。
**当前卡点**：无阻塞。
**下一步**：worker/service-worker timer 钳制补齐（DC-1 同病不同址收尾小切片：`script-sandbox/worker.rs` 1ms floor + FIXME、SW 域 shim 无 nesting；随后 M3 DC-2 视口外动画/空闲调度修复面）。
**入口**：[目标契约](../background-resource-lifecycle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 嵌套 timer、后台与视口外动画策略 | 页面 timer 嵌套 4ms 钳制修复 + 生产复测 DONE（DC-1 轮，~237 iter/s）；worker/SW 面钳制待补（1ms floor + FIXME）；后台策略与视口外动画归 M3 | PARTIAL（页面面 DONE） |
| 重复导航旧文档/回调/资源释放 | M2-S1 连接滞留修复 + 生产复测（对端 15/15 关闭）；M2-S2 双 120 次长程腿判定 RSS 爬坡+有界平台；M2-S3 残留计数生产验证：fetch 15/15 复位放弃、命令 31/31 行零残留、timer entry 有界滞留（0→15，24.8d expiry 上界）且回调对象已释放 | DONE（headless 面；GUI 腿归 M4） |
| 关闭后的连接/队列/进程生命周期 | renderer 进程复用 +163MB 不回落已测得；GUI 前台恢复腿 NOT_RUN | PARTIAL |
| CPU/RSS/时长/预热门禁 | M1 协议已固定（run-config + sampler 250ms）；阈值待多轮配对 | 待 M4 |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 四类场景与生产资源基线 | **DONE**（2026-10-22，12 腿 headless+GUI 双口径） | [M1 基线](evidence/2026-10-22-m1-baseline/ANALYSIS.md) |
| M2 | 首个增长来源、共享 owner 修复和回归 | **DONE**（2026-10-22）：S1 fetch 取消修复 + 复测；S2 RSS 有界平台判定；S3 残留计数落地 + BG-01 长程判定——三个增长源候选全部收口 | [M2-S1](evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md) / [M2-S2](evidence/2026-10-22-m2-s2-rss-attribution/ANALYSIS.md) / [M2-S3](evidence/2026-10-22-m2-s3-timer-form/ANALYSIS.md) |
| M3 | 空闲/视口外调度及恢复 | NOT_STARTED | 无 |
| M4 | 资源门禁、持续配对复测与清理 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BG-01-nested-and-hidden-timers | PARTIAL：嵌套最小延迟钳制修复 + 单测边界钉住 + 生产复测 DONE（~237 iter/s vs 修前 ~20 万，CPU 1.46→0.05 cores；worker/SW 面与 promise 续体欠钳为已知缺口）；后台策略明确且恢复正确未开始；GUI 前后台恢复 NOT_RUN | [DC-1 钳制](evidence/2026-10-22-dc1-nested-clamp/ANALYSIS.md) / M1 基线 ANALYSIS §DC-1 |
| DC-2 | BG-02-offscreen-and-idle-present | PARTIAL：静止免呈现成立（2 帧/30s）；动画呈现链未激活 →「合法工作不误停」INCONCLUSIVE；音视频 NOT_RUN | M1 基线 ANALYSIS §DC-2 |
| DC-3 | BG-03-repeat-navigation-stalled-resource | DONE（headless 面）：连接面修复+复测（M2-S1）；RSS 面爬坡+有界平台、无界泄漏排除（M2-S2 + BG-01 120s 腿同形态）；对象/回调面残留计数生产验证——fetch 15/15 放弃、命令零残留、timer entry 有界滞留且回调已释放（M2-S3）；GUI 腿归 M4 | [M2-S1](evidence/2026-10-22-m2-s1-fetch-cancel/ANALYSIS.md) / [M2-S2](evidence/2026-10-22-m2-s2-rss-attribution/ANALYSIS.md) / [M2-S3](evidence/2026-10-22-m2-s3-timer-form/ANALYSIS.md) |
| DC-4 | BG-04-close-and-resume | PARTIAL：destroyed 回执 20/20；renderer 进程复用 +163MB 不回落；GUI 恢复 NOT_RUN | M1 基线 ANALYSIS §DC-4 |
| DC-5 | 固定协议长期复测 | PARTIAL：协议已固定可复现；阈值与配对节奏待 M4 | M1 基线 ANALYSIS §DC-5 |

## 碰撞与执行记录

M1 轮（2026-10-22）只新增脚本、fixture 与文档，未触碰共享源码面。M2-S1 轮改动面：`crates/net`（fetch_scheduler/resource_loader）+ `apps/renderer`（js_worker/runtime）+ `apps/browser`（fetch_proxy）+ fixture server 观测增强——与并行流 rendering-compat（ZeroWeb-2，css/layout/render 面）、html-semantics-compat（~/work/ZeroWeb）工作面不重叠；全量 make test 轮次与兄弟流重负载交叠出现 3 例已知负载敏感瞬态红（IndexedDB 事务 / js_dom 墙钟 / SW 激活轮询），同树首轮对应腿全绿 + 隔离复跑绿归因收口（同 R5085/R4971 判例），非本改动面。M2-S2 轮只加观测编排参数（`BG_NAV_SRC/DST/CYCLES` 透传、WALL 参数化）与 fixture，零产品源码改动。M2-S3 轮改动面：`crates/engine`（timer_bridge `pending_stats` + 单测）+ `apps/renderer`（js_worker 复位收口处残留计数日志），诊断只写 tracing 日志不改行为；BG-01 120s 腿与 bg03 残留腿自建实例已随腿收尾。DC-1 轮（2026-10-22）改动面：`crates/engine`（js_dom_shim/part01.js setTimeout/setInterval 嵌套 4ms 钳制 + 2 单测）——与兄弟流工作面不重叠；全量 make test 首轮 zero-browser bin 腿 2 例红（compositor 关停 SIGSEGV + IndexedDB 事务竞争，后者属已知负载敏感族），两例隔离复跑双绿归因收口（同 R5085/R4971 判例，均非本改动面——compositor/IndexedDB 均不执行页面 JS），其后 browser `--tests` 腿按 Makefile 同口径补跑；bg01 修后腿自建实例已随腿收尾。历轮复测腿自建实例已随腿收尾。

观测资产：`scripts/background-resource-observe.sh`（单腿编排，GUI 腿 `BG_OBSERVE_GUI` 场景名 `*_gui`/`*_compgui`；M2-S2 起 bg03-nav 支持 `BG_NAV_SRC/DST/CYCLES` 与墙钟参数）+ `background-resource-probe.mjs`（CDP 驱动，零依赖；`--nav-src/--nav-dst` 覆盖）+ `background-resource-sampler.py`（进程树采样）+ `background-resource-fixture-server.py`（stall 子资源；M2-S1 起 hold 循环轮询对端 FIN，`stall_closed_by_peer` 成为取消生效的直接观测信号）+ `tests/fixtures/background-resource/`（8 fixture，M2-S2 增 `bg03-nav-plain.html`）。运行守卫：每腿 timeout 墙钟 + 24G systemd scope，采样仅统计本次 browser 后代进程树。

frame dump .bin 已清理留 manifest（M1）；browser.log/http.log 按 `*.log` 忽略约定不入库。无运行残留。
