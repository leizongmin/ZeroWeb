# DOM 操作基准控制面

**日期**：2026-11-04
**状态**：Done（2026-11-04 目标收口：DC-1..DC-5 全满足，组合态 main 终树复验 domops GATE PASS exit 0——cap5 干净窗口 6/6 预算内、zw 行贴基线、并行流改动无哨兵回归，见下方「M4 后续轮记录」与 cap5 产物）——M1-M3 已交付（M3：缺陷1/缺陷2 修复+配对回归，两大性能热点归因到代码路径）；**M4 已收口：dom-ops 门禁接线完成**——`scripts/dom-ops-gate.sh` 编排「转换→比较」，比较器原样复用 `scripts/perf-gate.sh`（零改动），独立基线 `docs/perf/baselines/domops-linux-x64.json`（3 哨兵行双浏览器，protocol v1.1）；干净窗口 GATE PASS exit 0，污染窗口 GATE FAIL 经采样日志归因环境噪声（未动基线）——见 [M4 evidence](evidence/2026-11-04-m4-gate-wiring/README.md)。协议 v1→v1.1：thrash 样本 12→8（12 样本贴 CDP awaitPromise 8s 悬崖，capture 实测 reverse 超时）。Firefox 缺测（未安装，NOT_AVAILABLE，不伪造）。
**当前卡点**：无阻塞。残余转性能工作流：thrash 增量布局 / shim 节点成本（架构级）、probe8 A2 便宜态疑点、dom-ops runner 内建 suspect 检测。
**下一步**：无（目标已收口）。架构级残余（thrash 增量布局 / shim 节点成本、probe8 A2 便宜态疑点、dom-ops runner 内建 suspect 检测）转性能工作流跟踪——3 哨兵行已入 perf-gate 常驻门禁，回归由 `domops-linux-x64.json` 基线拦截。
**入口**：[目标契约](../dom-operation-bench.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 真实 DOM/Canvas 基准 fixture | 区分缓存读取、强制布局与真实绘制 | **DONE**（`tests/benchmarks/dom-ops/fixtures/`，3 fixture） |
| 预热/操作数/样本/正反序协议 | 固定浏览器版本与环境，检测污染样本 | **DONE**（协议 v1，`run-dom-ops.mjs` `PROTOCOL` 表 + 引擎预算标定） |
| 桥接、样式与布局热点归因 | 基线后选择数量级异常的首个调用链 | **DONE**（缺陷1/缺陷2 调用链已归因修复；thrash 与 replace.pool 性能链已归因，最小修复为架构级转性能工作流） |
| 报告 schema/入门禁指标/Firefox 环境 | 复用现有消费者；缺失平台记 NOT_RUN | **DONE**（schema v1.1 + `config_hash`；3 哨兵行双浏览器入独立基线，perf-gate 原样复用）；Firefox NOT_AVAILABLE |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 操作/采样与语义协议（style/geometry 双场景先行） | **DONE** | [2026-11-04 evidence](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md) |
| M2 | ZeroWeb/Chrome 实测与操作报告 | **DONE** | 同上（`reports/report.json`，2026-11-04_official-v1） |
| M3 | 首热点最小修复与配对回归 | **DONE**（缺陷1、缺陷2 修复+配对回归，[缺陷1 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md) / [缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md)；两大性能热点归因完成，[热点归因 evidence](evidence/2026-11-04-m3-hotspot-attribution/README.md)） | 同上「发现」节 |
| M4 | 有价值指标接线与 DC 审计 | **DONE** | [2026-11-04 evidence](evidence/2026-11-04-m4-gate-wiring/README.md) |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | BENCH-01-style-and-geometry-read | **PASS**（M3 缺陷1 修复后重测 2026-11-04，style-geometry 全行双浏览器语义 PASS） | [M3 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md) |
| DC-2 | BENCH-02-replace-and-list-mutation | **PASS**（M3 缺陷2 修复后重测 2026-11-04，mut.replace.pool 语义 PASS 4/4 正反序配对） | [M3 缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md) |
| DC-3 | BENCH-03-canvas-state | PASS（像素断言 8/8 双浏览器） | evidence 2026-11-04 |
| DC-4 | BENCH-04-paired-and-semantic-check | PASS（配对同环境、正反序、注入捕获） | evidence 2026-11-04 |
| DC-5 | 报告与既有门禁 | **PASS**（2026-11-04 M4：domops 门禁接线完成，干净窗口 GATE PASS exit 0、污染窗口 FAIL 归因环境噪声未动基线；本改动树全量 `make bench-gate` GATE PASS exit 0——25 指标全预算内、88 NEW 待 capture；schema/基线/不冒充 PASS/常驻回归四条款证据见 [M4 evidence](evidence/2026-11-04-m4-gate-wiring/README.md)） | 同左 |

## 碰撞与执行记录

候选共享面：engine/dom/script-sandbox、bench-report/perf-gate 和自动化消费者。本轮只新增测试资产（`tests/benchmarks/dom-ops/`）与文档，未改产品代码，与并行渲染流无工作面重叠；ZeroWeb 侧重建 release 走既有构建入口（systemd-run scope 包裹）。未覆盖既有 capture 或基线。

2026-11-04 本轮记录：协议标定暴露 ZeroWeb CDP evaluate 预算（同步 30s / awaitPromise 8s）与 `performance.now` ~100µs 粒度（平台事实，待修）；官方采集无超时、无串扰。INCONCLUSIVE 行（build.append 顺序效应、fillStyle/draw.rect/remove.children 分辨率超阈）修复后需重测。详细命令、比值表、探针输出与清理见 [evidence README](evidence/2026-11-04-m1m2-dom-ops-protocol-first-run/README.md)。

2026-11-04 M3 缺陷1 轮记录：**改产品代码**——`crates/engine/src/rect_bridge.rs`（`make_effective_dom_rect_handler`：pending 未应用尾→worker 侧有效视图布局，drain 后回落 snapshot 快路径；kill-switch `ZW_RECT_PENDING_LAYOUT`）、renderer `js_worker.rs` 接线（`SetViewportHint` 同步视口 cell）、browser `tab_js_worker.rs` FIXME（视口通道缺失，tab 路径缺陷仍在）。共享面提示：engine gBCR 桥与 renderer worker 装配属共享 crate——并行渲染流若改 engine 需对表。零回归面：无 pending 时逐字节旧行为（引擎+renderer 108/15 单测、stash 往返复核 5 既有红非本改动）；定向 bench-gate 3 连跑首轮 first_paint_wall 365.44 FAIL→复跑 323.96/326.72 PASS，归因环境方差非本改动（测量二进制无本代码路径，符号 0 命中）。重测、门禁与比值表见 [M3 evidence](evidence/2026-11-04-m3-defect1-rect-pending-layout/README.md)。

2026-11-04 M3 缺陷2 轮记录：**改产品代码**——engine `js_dom_bridge/callbacks.rs`（R100 宿主半边接线：`MUTATION_HISTORY_INBOX` 收件箱桥 + `sync_mutation_history_inbox`；`__zw_attr_names_handle` 补历史层）、renderer `js_worker.rs`（`history_inbox` 共享槽 + 启动发布 + `append_applied_mutation_history` + Reset 臂清历史）、`page_scripts.rs`（`apply_recorded_mutations` drain→append→apply 装配序）。根因非漏调一行：renderer apply 在主线程、读回调在 js_worker 线程，thread_local 直呼写错线程——收件箱是结构必需。同上共享面提示；webview 路径与 handle style 面残余在册（申报不扩大）。基准 v3：mut.replace.pool 语义 ✗FAIL→✓PASS（ANOMALY 358.8x 真实热点），DC-2 翻绿。详见 [M3 缺陷2 evidence](evidence/2026-11-04-m3-defect2-dataset-timer-loss/README.md)。

2026-11-04 M3 热点归因轮记录：只加测试资产与文档，未改产品代码。探针序列 probe8→8f（bisect/cliff 扫描/因果隔离/机制构成/墙钟终判别，每臂独立 tab，Chrome 配对）：**thrash 10079x = pending 读全量有效视图重建**——每次写使 `RECT_LAYOUT_CACHE` 键（含 mlen）失效，每次读全量 parse+cascade+`LayoutEngine::compute`+HitTest 填表 ~13ms/op，300 op 队列 2→600 条全程平坦（非线性 replay，固定成本），setup/timer 间隙/脏元素数/前序子场景/负载均经 bisect 排除；**replace.pool 358.8x = shim 逐节点结构开销**（裸 3.4ms ≈ 属性 3.6ms，+6%；∝ 子节点数；离屏基线 ~100µs/节点）。两热点最小修复均为架构级（增量布局/shim 节点成本），按目标契约转专用性能工作流。未解决在册：probe8 A2 便宜态（~12.7-29.7µs/op）两轮观测但在 15+ 复现臂（含两个逐步一致复刻）不可复现，负载低窗口仍昂贵——不作归因依据，为性能工作流首要疑点。本轮共享面：无（零产品代码改动）。探针、原始数据与全部数字见 [热点归因 evidence](evidence/2026-11-04-m3-hotspot-attribution/README.md)。

2026-11-04 M4 门禁接线轮记录：**改测试资产与脚本，未改产品代码**——`run-dom-ops.mjs`（v1.1：报告带 `config_hash`；thrash 样本 12→8 修 awaitPromise 8s 悬崖——capture#1 实测 reverse 超时）、新增 `to-bench-report.mjs`（格式映射+入选行筛选，坏样本非零退出）与 `scripts/dom-ops-gate.sh`（录制独立基线/比较，比较器原样 exec perf-gate.sh）、新基线 `docs/perf/baselines/domops-linux-x64.json`（6 指标，预算公式与共享基线同源）。同日四连跑：#1 暴露协议悬崖 → v1.1；#1' 污染弃用（zw 抬高 1.5-3x，chrome 哨兵 +50%）；#2 干净录基线；#3 污染 → GATE FAIL exit 1，EMA 采样+时序归因环境噪声（zw forward 相位 +76-86%，chrome 全稳），未录为回归、未动基线；#4 干净（/proc/stat 瞬时 busy ≤19%）→ GATE PASS exit 0。全量 `make bench-gate` GATE PASS exit 0（25 指标全预算内；88 NEW 为基线后新 bench 用例待 capture）。运维事实：target-disk-guard 分级清理 target/ 88GB（全量清空，含 release 二进制）——下次 domops 采集前需 release 重编。共享面：无（perf-gate.sh/bench-report.sh/既有基线零改动；新基线为独立文件）。详细运行表、哨兵证据与限制在册（runner 内建 suspect 检测待做）见 [M4 evidence](evidence/2026-11-04-m4-gate-wiring/README.md)。

2026-11-04 M4 后续轮记录（组合态复验 + 构建入口断裂修复）：**改产品代码一处（修自己 M3 埋的断裂）**——release 重建时发现 `make browser-build`（`-p zero-browser` 默认特性无 v8）自 90572cc78 起 E0425：`rect_bridge.rs` 未门控引用 script-runtime 门控的 callbacks 模块符号（`MUT_DRAIN_GEN` / `applied_mutations_cursor`）；workspace 构建因特性统一（renderer/page-runtime/webview 默认 v8）恒绿而掩盖一天。修复按仓内 cfg 模式：pending 机制（kill-switch 函数/`RECT_LAYOUT_CACHE`/增强工厂）加 `script-runtime` 门 + 补无 JS 回落工厂（纯 snapshot handler，与快路径逐字节一致；无 JS ⇒ 无样式写源 ⇒ 快路径即真值）。验证：`-p zero-browser` 默认组合 35.6s 过、`cargo test -p zero-engine --lib rect_bridge` 15/15（含两个 M3 常驻回归）。共享面：仅改 `rect_bridge.rs`（本流 M3 面），未触 callbacks.rs 与并行流文件。**组合态 main 门禁复验**：release workspace 重建（含并行流 8549f39ec/b0c56a23a/42c206eca 产品改动）→ 定向 thrash PASS 12.28/12.31ms → 全量采集（idle 窗口 busy≤24%）→ **GATE PASS exit 0**（6/6 预算内，zw 行贴基线：thrash 12.64ms / geom.read.same 22.9µs / replace.pool 6.84ms），并行流改动无哨兵回归。教训沉淀 [cargo 特性统一掩盖 cfg 门控符号断裂](../../../docs/learnings/bugs/2026-11/2026-11-04-cargo-feature-unification-hides-cfg-symbol-breaks.md)。
