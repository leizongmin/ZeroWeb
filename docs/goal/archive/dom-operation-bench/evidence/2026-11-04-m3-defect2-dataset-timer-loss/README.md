# M3 缺陷2修复——timer 回调 DOM 变更丢 data-\* 属性（mutation 历史宿主半边接线）

**日期**：2026-11-04
**前置**：[缺陷1 修复](../2026-11-04-m3-defect1-rect-pending-layout/README.md)（同轮基准把本缺陷定性为独立缺陷：`mut.replace.pool` 语义 FAIL 228.8x）
**结论**：缺陷2已修复——`mut.replace.pool` ZeroWeb 侧 ✗INCONCLUSIVE zw:FAIL→✓ANOMALY 语义 PASS（checks 4/4，正反序配对）；配对回归 engine 3730+1、renderer 372 全绿；行为探针 P7 修复前后对照齐备。

## 根因（复现探针 [probe7](../2026-11-04-m1m2-dom-ops-protocol-first-run/probes/probe7-dataset-timer-loss.mjs)）

- **症状**（runner 形态，CDP evaluate + awaitPromise 泵 timer）：createElement 句柄元素同批 `dataset.seq = '0'` 即时可见；首批 evaluate 结束（宿主 drain+apply）后，后续 timer 回调内 `dataset.seq`/`getAttribute('data-seq')` 读翻转为 `undefined`/`null`——**JS 对象身份保持**（`pool[0] === box.children[0]`），子树结构保持。
- **判别读**（本轮新增 fresh-evaluate 臂）：promise 落定后**另起一次** evaluate 读 `data-seq` 返 `"0"` 正常——renderer `cached_html` 完好，**丢失仅在 await 循环内的 worker 视图**。排除 renderer apply 路径。
- **根因**：handle 元素属性读链 = 当前 mutation 批 latest-wins + **MUTATION_HISTORY 回落**（R100/slice48 设计）。但 R100 契约的**宿主半边（apply 前 `append_mutation_history`）生产未接线**——callbacks.rs 第 2 层注明记「webview/renderer apply 链均无 append 调用方，text/tag/attr 三面同债，残余在册」。renderer automation apply 在 renderer **主线程** drain（`apply_recorded_mutations`），读回调在 **js_worker 线程**——主线程直呼 `append_mutation_history` 写的是错误线程的 thread_local，接线不能是直呼，这是该债拖延的真实结构原因（非遗漏调用一行）。
- **规范依据**：属性是元素状态，无 alive/快照概念（https://dom.spec.whatwg.org/#dom-element-getattribute 、https://dom.spec.whatwg.org/#dom-element-dataset ）。

## 修复（收件箱桥，R100 宿主半边接线）

- **engine**（`js_dom_bridge/callbacks.rs`）：`MUTATION_HISTORY_INBOX` thread_local + `publish_mutation_history_inbox`（worker 启动臂发布，每 worker 一份，多 tab 互不可见）+ `sync_mutation_history_inbox`（读回调回落前整批搬空进本线程历史；锁不嵌套）。接线面：`__zw_get_attr_handle`/`__zw_has_attr_handle` 回落前、`__zw_attr_names_handle`（**新增历史层**：dataset 枚举此前无快照基底亦无历史，正序 latest-wins 合并 `[history…, current]`）、`query_history_text`/`query_history_tag`（text/tag 面三调用点同收）。未发布宿主（webview 路径、引擎测试）恒 None = 零行为变化。
- **renderer**（`js_worker.rs` + `page_scripts.rs`）：`RendererJsWorker.history_inbox`（spawn 期创建）→ `js_worker_main` 启动发布 → 宿主侧 `append_applied_mutation_history(batch)`；生产调用点 `apply_recorded_mutations` **drain → append → apply** 装配序（part46/48 钉①镜像）；worker `ResetDocumentState` 臂同点清历史+收件箱（文档域状态，webview `load_html` 清历史同契约）。
- **已知边界**（申报不扩大）：`__zw_get_style_lw_handle`（handle style 读）同族「无快照基底」面未接历史——无观测失败，残余在册；webview 路径 apply 链仍未接（同线程架构其 sel 面经新基底自愈，handle 面残余 R100 既有申报）；browser tab 路径**无本缺陷**——其 apply 以 clone 快照队列非 drain（`tab_scripts.rs` `apply_recorded_mutations`），队列即天然历史，attr 读持续命中（代价是队列不收缩，既有设计权衡，非本轮范围）。

## 配对回归

| 面 | 结果 |
|---|---|
| 引擎新钉（part96，与修复同提交） | `handle_attr_read_survives_drain_via_history_inbox_m3d2` ✓——dataset/getAttribute/枚举/has 四面跨 drain+apply+换代存活；收件箱整批搬空幂等；二次读走纯历史 |
| 引擎相邻面 | `--lib` 全量 **3730 通过/5 失败＝HEAD 既有红**（stylesheets/adopted 族，缺陷1 轮 stash 往返已定责，与本改动无关） |
| renderer 新端到端（与修复同提交） | `renderer_js_worker_handle_attr_read_survives_drain_via_history_inbox_m3d2` ✓——真 RendererJsWorker 两 execute 跨 drain：**不推新快照**证明回落不依赖快照换代（生产 await 循环内 timer 回调同形态） |
| renderer 相邻面 | `--lib` 全量 **372/372 ✓**（含缺陷1 轮 108 与本轮新增） |

构建/测试均走守卫（release systemd-run MemoryMax 24G/2340s；测试 test-guard，engine 编译峰值超 8G/16G 上限改 `-j 4` 限并行后 14G 内通过——并行流活跃期限并行优于抬限）。

## 行为探针（真实 ZeroWeb，release 重建后）

P7 修复前 → 修复后：

| 断言点 | 修复前 | 修复后 |
|---|---|---|
| `in-timer-0` seq0/child0seq | `undefined`/`undefined` | `"0"`/`"0"` |
| `after-async` attr0（getAttribute） | `null` | `"0"` |
| `ident0`（JS 身份） | `true` | `true`（不变） |
| fresh-evaluate（renderer 侧对照） | `"0"`（始终完好） | `"0"`（不变） |

探针本轮补两处：`fileURLToPath` import（v1 SL-008 改造漏 import，与 v1 探针同型即修）+ fresh-evaluate 判别臂（根因判别关键证据）。

## 基准重测（官方采集 v3，`reports/report.json`，git_sha 90572cc78 dirty）

同协议 v1、同环境（Chrome 156.0.8078.79）、正反序配对：

| 行 | v2（缺陷1 轮） | 本轮 v3 | 变化 |
|---|---|---|---|
| **mut.replace.pool** | INCONCLUSIVE zw:FAIL 228.8x | **ANOMALY 358.8x**（zw 4686.25µs vs ch 13.06µs），**语义 PASS 4/4** | **缺陷2 修复**；比值爆炸为真实热点数据——ZeroWeb replaceChildren×50 句柄往返 vs Chrome 增量 13µs |
| geom.write_read.thrash | OK PASS 9885.5x | 语义 **PASS 2/2**（缺陷1 保持），ratio 10079.2x 本轮 timer-INCONCLUSIVE（performance.now ~100µs 粒度，v1 已记平台事实） | 语义保持 |
| style.write.color | 119.4x ANOMALY | 180.8x ANOMALY | 量级不变 |
| geom.read.same / rotate | 49.9x / 63.1x ANOMALY | 48.5x / 63.4x ANOMALY | 量级不变 |
| style.read.same / rotate | 3.9x notable | 3.0x ok / 3.6x notable | 量级不变 |
| mut.build.append | INCONCLUSIVE 顺序效应 2747.8x | INCONCLUSIVE timer 2993.6x | INCONCLUSIVE 原因从顺序效应变为 timer 分辨率（同一平台事实族） |
| mut.remove.children / mut.move.identity / cv.state.fillStyle / cv.draw.rect | INCONCLUSIVE timer | INCONCLUSIVE timer | 未动 |
| cv.state.saveRestore / cv.read.imageData | 6.1x notable / 31.1x ANOMALY | 8.2x notable / 29.8x ANOMALY | 量级不变 |

## 下一步（M3 剩余）

1. 热点归因（M3 收尾）：`thrash` 10079x（全文档重建 vs 增量布局，缺陷1 轮已定性）与 `mut.replace.pool` 358.8x（replaceChildren 句柄往返）两个数量级异常的首个调用链。
2. INCONCLUSIVE timer 行（fillStyle/draw.rect/remove.children/move.identity/build.append）随 performance.now 粒度修复（平台事实在册）后重测。
3. M4：有价值指标接线（bench-report/perf-gate）与 DC-5 审计。

## 清理

- `tests/benchmarks/dom-ops/results/m3-defect2/` 为 gitignore 本地产物，官方三件套已拷贝入本目录 `reports/`。
