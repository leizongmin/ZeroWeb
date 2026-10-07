# slice38 RED → GREEN → mutation 记录

分支 `slice38-budget-done-state-hygiene`（自 `ebd5d8f1d`）。测试经 `./target/test-guard` 包裹
（--per-proc-mem 4 --total-mem 8 --time-limit 600/900，与 `make test` 同一包裹器），定向过滤跑；
全量门禁另行走 `make test` / `make reftest`。

## 钉清单（3 枚）

| # | 测试 | 位置 | 守卫面 |
|---|---|---|---|
| 1 | `budget_done_step_clears_form_control_state_maps_s38` | `crates/engine/src/tests/pipeline.rs` | 预算 Done 步清 `form_control_values`/`form_control_compositions`（SetFormValue+SetFormComposition 置值 → 不同结构文档预算渲染 → 两表 + overrides 全空） |
| 2 | `budget_done_step_clears_form_control_values_cross_document_s38` | `crates/webview/src/webview.rs` tests | 生产序列（load_html → apply SetFormValue → prepare → 不同 HTML 预算渲染 → `form_control_value_overrides()` 空） |
| 3 | `render_html_in_rect_swapping_cached_doc_clears_node_id_keyed_maps_s38` | `crates/engine/src/tests/pipeline.rs` | 死路径对齐钉：handle 表 + form 两表换代同清 |

## RED（修复前，基线 `ebd5d8f1d` + 钉）

- 钉 1/3（engine）：`2 failed; 0 passed`。钉 1 红：「预算换代后 form_control_values 不得残留上一代 NodeId 键」；钉 3 红：「render_html_in_rect 换代后 handle 表不得残留上一代条目」（该函数今日三表全不清）。日志：`red-engine-pins.log`。
- 钉 2（webview）：`1 failed`，红形态即 ABA ghost 实证：`{"#x": "typed"}` ——doc1 `<input id="a">` 的输入值经 slotmap 换代 ABA 落到 doc2 无关节点 `div#x` 的 selector（表单提交/reset 消费方将读到该 ghost）。日志：`red-webview-pin.log`。

## GREEN（修复后）

- engine `s38` 过滤：`2 passed`（`green-engine-pins.log`）。
- webview `s38 persistent_handle_nodes_s37` 过滤：`2 passed`（新钉 + PR #93 s37 钉同绿）。
- 零回归定向组：`deferred_batch_final_state_matches_per_script_rendering`（表单值跨批量边界 restore 存活）绿；webview `user_actions` 组 `8 passed`（含 reset/submit 事务语义）。

## mutation（删修复行 → 钉红）

- 变更：仅删 `pipeline_budget.rs` Done 步两行清零（`form_control_values.clear()` / `form_control_compositions.clear()`），`render_html_in_rect` 清零块不动。diff：`mutation.diff`。
- 结果：钉 1 红（engine）+ 钉 2 红（webview），钉 3 保持绿（`1 passed; 1 failed`）——各钉守各修复面，无交叉误报。日志：`mutation-red-engine.log`、`mutation-red-webview.log`。
- 恢复修复行后复跑：engine `3 passed`（两钉 + deferred_batch）、webview `2 passed`（s38 钉 + s37 钉）。

## ABA 前提

slotmap key version 计数随每个 `Document` 新建复位（`crates/dom/src/node.rs:6-15`），跨文档同 (slot, version) 复现——PR #93 s37 钉已实证同前提；本切片 webview 钉 RED 形态（ghost 落 `#x`）为该机制的直接实证。
