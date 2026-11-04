# DOM 操作基准控制面

**日期**：2026-11-04
**状态**：Active——M1、M2 已交付；M3 进行中：**缺陷1（gBCR 不反映 pending 样式写）与缺陷2（timer 回调丢 data-\*）均已修复**（缺陷1 worker 侧有效视图布局；缺陷2 mutation 历史宿主半边接线/收件箱桥；均四层证据，DC-1/DC-2 重测 PASS，2026-11-04）。Firefox 缺测（未安装，NOT_AVAILABLE，不伪造）。
**当前卡点**：无阻塞。
**下一步**：M3 收尾——热点归因：thrash 10079x（全文档重建 vs 增量布局）与 mut.replace.pool 358.8x（replaceChildren 句柄往返）首个调用链；其后 M4 门禁接线与 DC-5 审计。
**入口**：[目标契约](../dom-operation-bench.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 真实 DOM/Canvas 基准 fixture | 区分缓存读取、强制布局与真实绘制 | **DONE**（`tests/benchmarks/dom-ops/fixtures/`，3 fixture） |
| 预热/操作数/样本/正反序协议 | 固定浏览器版本与环境，检测污染样本 | **DONE**（协议 v1，`run-dom-ops.mjs` `PROTOCOL` 表 + 引擎预算标定） |
| 桥接、样式与布局热点归因 | 基线后选择数量级异常的首个调用链 | **IN_PROGRESS**（缺陷1 调用链已归因修复；缺陷2 探针已入库） |
| 报告 schema/入门禁指标/Firefox 环境 | 复用现有消费者；缺失平台记 NOT_RUN | schema v1 已定；M4 门禁接线待做；Firefox NOT_AVAILABLE |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 操作/采样与语义协议（style/geometry 双场景先行） | **DONE** | [2026-11-04 evidence](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md) |
| M2 | ZeroWeb/Chrome 实测与操作报告 | **DONE** | 同上（`reports/report.json`，2026-11-04_official-v1） |
| M3 | 首热点最小修复与配对回归 | **IN_PROGRESS**（缺陷1、缺陷2 均已修复+配对回归，[缺陷1 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md) / [缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md)；热点归因待做） | 同上「发现」节 |
| M4 | 有价值指标接线与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BENCH-01-style-and-geometry-read | **PASS**（M3 缺陷1 修复后重测 2026-11-04，style-geometry 全行双浏览器语义 PASS） | [M3 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md) |
| DC-2 | BENCH-02-replace-and-list-mutation | **PASS**（M3 缺陷2 修复后重测 2026-11-04，mut.replace.pool 语义 PASS 4/4 正反序配对） | [M3 缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md) |
| DC-3 | BENCH-03-canvas-state | PASS（像素断言 8/8 双浏览器） | evidence 2026-11-04 |
| DC-4 | BENCH-04-paired-and-semantic-check | PASS（配对同环境、正反序、注入捕获） | evidence 2026-11-04 |
| DC-5 | 报告与既有门禁 | NOT_RUN | 待 M3 后接线 |

## 碰撞与执行记录

候选共享面：engine/dom/script-sandbox、bench-report/perf-gate 和自动化消费者。本轮只新增测试资产（`tests/benchmarks/dom-ops/`）与文档，未改产品代码，与并行渲染流无工作面重叠；ZeroWeb 侧重建 release 走既有构建入口（systemd-run scope 包裹）。未覆盖既有 capture 或基线。

2026-11-04 本轮记录：协议标定暴露 ZeroWeb CDP evaluate 预算（同步 30s / awaitPromise 8s）与 `performance.now` ~100µs 粒度（平台事实，待修）；官方采集无超时、无串扰。INCONCLUSIVE 行（build.append 顺序效应、fillStyle/draw.rect/remove.children 分辨率超阈）修复后需重测。详细命令、比值表、探针输出与清理见 [evidence README](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md)。

2026-11-04 M3 缺陷1 轮记录：**改产品代码**——`crates/engine/src/rect_bridge.rs`（`make_effective_dom_rect_handler`：pending 未应用尾→worker 侧有效视图布局，drain 后回落 snapshot 快路径；kill-switch `ZW_RECT_PENDING_LAYOUT`）、renderer `js_worker.rs` 接线（`SetViewportHint` 同步视口 cell）、browser `tab_js_worker.rs` FIXME（视口通道缺失，tab 路径缺陷仍在）。共享面提示：engine gBCR 桥与 renderer worker 装配属共享 crate——并行渲染流若改 engine 需对表。零回归面：无 pending 时逐字节旧行为（引擎+renderer 108/15 单测、stash 往返复核 5 既有红非本改动）；定向 bench-gate 3 连跑首轮 first_paint_wall 365.44 FAIL→复跑 323.96/326.72 PASS，归因环境方差非本改动（测量二进制无本代码路径，符号 0 命中）。重测、门禁与比值表见 [M3 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md)。

2026-11-04 M3 缺陷2 轮记录：**改产品代码**——engine `js_dom_bridge/callbacks.rs`（R100 宿主半边接线：`MUTATION_HISTORY_INBOX` 收件箱桥 + `sync_mutation_history_inbox`；`__zw_attr_names_handle` 补历史层）、renderer `js_worker.rs`（`history_inbox` 共享槽 + 启动发布 + `append_applied_mutation_history` + Reset 臂清历史）、`page_scripts.rs`（`apply_recorded_mutations` drain→append→apply 装配序）。根因非漏调一行：renderer apply 在主线程、读回调在 js_worker 线程，thread_local 直呼写错线程——收件箱是结构必需。同上共享面提示；webview 路径与 handle style 面残余在册（申报不扩大）。基准 v3：mut.replace.pool 语义 ✗FAIL→✓PASS（ANOMALY 358.8x 真实热点），DC-2 翻绿。详见 [M3 缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md)。
