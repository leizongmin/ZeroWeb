# DOM 操作基准控制面

**日期**：2026-11-04
**状态**：Active——**M1、M2 已交付**（协议 v1 + 首轮真实 ZeroWeb/Chrome 配对数据；语义断言发现 2 个真实缺陷，14 行中 4 行 ANOMALY、2 行 notable、7 行 INCONCLUSIVE）；下一步 M3。Firefox 缺测（未安装，NOT_AVAILABLE，不伪造）。
**当前卡点**：无阻塞。M3 修复对象已定位（见下）。
**下一步**：M3 首热点最小修复——先修「几何读不反映样式写」（缺陷 1），再修「timer 回调 DOM 变更丢 data-\* 属性」（缺陷 2）；各配对性能与行为回归后重测 INCONCLUSIVE 行。
**入口**：[目标契约](../dom-operation-bench.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 真实 DOM/Canvas 基准 fixture | 区分缓存读取、强制布局与真实绘制 | **DONE**（`tests/benchmarks/dom-ops/fixtures/`，3 fixture） |
| 预热/操作数/样本/正反序协议 | 固定浏览器版本与环境，检测污染样本 | **DONE**（协议 v1，`run-dom-ops.mjs` `PROTOCOL` 表 + 引擎预算标定） |
| 桥接、样式与布局热点归因 | 基线后选择数量级异常的首个调用链 | **READY**（缺陷 1/2 复现探针已入库 evidence） |
| 报告 schema/入门禁指标/Firefox 环境 | 复用现有消费者；缺失平台记 NOT_RUN | schema v1 已定；M4 门禁接线待做；Firefox NOT_AVAILABLE |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 操作/采样与语义协议（style/geometry 双场景先行） | **DONE** | [2026-11-04 evidence](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md) |
| M2 | ZeroWeb/Chrome 实测与操作报告 | **DONE** | 同上（`reports/report.json`，2026-11-04_official-v1） |
| M3 | 首热点最小修复与配对回归 | NOT_STARTED（缺陷 1/2 复现探针已入库；ANOMALY 行已定位） | 同上「发现」节 |
| M4 | 有价值指标接线与 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BENCH-01-style-and-geometry-read | RAN，语义 FAIL（缺陷 1：rect 不反映 style 写） | evidence 2026-11-04 |
| DC-2 | BENCH-02-replace-and-list-mutation | RAN，语义 FAIL（缺陷 2：timer 回调丢 data-\* 属性） | evidence 2026-11-04 |
| DC-3 | BENCH-03-canvas-state | PASS（像素断言 8/8 双浏览器） | evidence 2026-11-04 |
| DC-4 | BENCH-04-paired-and-semantic-check | PASS（配对同环境、正反序、注入捕获） | evidence 2026-11-04 |
| DC-5 | 报告与既有门禁 | NOT_RUN | 待 M3 后接线 |

## 碰撞与执行记录

候选共享面：engine/dom/script-sandbox、bench-report/perf-gate 和自动化消费者。本轮只新增测试资产（`tests/benchmarks/dom-ops/`）与文档，未改产品代码，与并行渲染流无工作面重叠；ZeroWeb 侧重建 release 走既有构建入口（systemd-run scope 包裹）。未覆盖既有 capture 或基线。

2026-11-04 本轮记录：协议标定暴露 ZeroWeb CDP evaluate 预算（同步 30s / awaitPromise 8s）与 `performance.now` ~100µs 粒度（平台事实，待修）；官方采集无超时、无串扰。INCONCLUSIVE 行（build.append 顺序效应、fillStyle/draw.rect/remove.children 分辨率超阈）修复后需重测。详细命令、比值表、探针输出与清理见 [evidence README](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md)。
