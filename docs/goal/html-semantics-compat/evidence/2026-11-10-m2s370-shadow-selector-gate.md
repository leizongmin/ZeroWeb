# M2 切片 370（s370）——shadow 内容唯一选择器门 + closedby-complex 16F 勘根矩阵

日期：2026-11-10（上一轮 session 429 中断承接，本轮收口）
域：html/semantics/interactive-elements/the-dialog-element（dialog-popover-closedby-complex shadow-slots 段 16F）
性质：engine 选择器门修复（绿）+ 域跑红态勘根矩阵（A–L 探针，断点定谳 runner 执行循环生命周期）——**域口径零位移**（16F 红态非本刀引入，修复面正确但目标红另有断点）

## 红态基线

- dialog-popover-closedby-complex solo（修复树，release runner 含本刀改动）：**24P/16F**。16F 全部 `same structure, but with shadow DOM slots` 段；normal 段 20 case 全绿。
- 16F 断言分布：13×「popoverA should be open expected true got false」（主矛盾）+ 1×「popoverB should be closed got true」+ 1×「popoverA should be closed got true」+ 1×「dialogA should be open got false」。
- modalB=false 组的 popoverAExpected=false 断言全部**假绿**（popoverA 从未 open 期望恰为 closed）——16F 全由「shadow 段 popoverA 未开/被误关」单根驱动。
- 与 s173 记录 LOST=20（4P+16F）的 16F 同数：红态是既有面，非本刀回归。

## 勘根矩阵（探针 zw-probe-s370[a–l]，gitignore 区跑后全删）

主链定谳（探针 a 首轮三相位 dump）：
1. findElements 对 shadow 树枚举正常（keys 齐）；popover 属性在场、connected=true、showPopover 不抛异常、`matches(':popover-open')` showPopover 后 = true——**R3071 状态机全链正常**。
2. **clickOn(popoverA) 后 popoverA 被误关**（afterClick=false）——light-dismiss 收到「外部点击」。
3. **根因：shadow 内容 `getBoundingClientRect()` 全零**（popoverA 与 shadow 段 dialogA 均 0,0,0,0）→ clickOn 中心点 = (0,0) → 落全部 dialog/popover 外 → light-dismiss 全关。normal 段 rect 正常（24P 佐证）。

分离矩阵（每行唯一变量）：

| 探针 | 结构 | 变量 | rect | 结论 |
|---|---|---|---|---|
| b | 单 SR | 基线 | 非零 | engine rect 链单 SR 正常 |
| c | 2 层链 | 嵌套 | dialog 零（未 open display:none 正确行为）、探针自身漏 light 递归 | 作废重设 |
| d | 平级双 SR | SR 数量 | **均非零**（dialog 零=display:none 正确） | 平级 SR 正常 |
| e | 2 层链 + slot 摊平 | 摊平 | 非零 | slot 摊平（空内容）正常 |
| f | e + showPopover | showPopover | 非零 | showPopover 不破坏 rect |
| g | 3 层真语料同构 | 层数 | popoverA **零** + popoverB（light 摊平内容、裸 sel）**零**；#unrelated（body 直接子）非零 | **3 层宿主链红**；且红不限于 shadow——摊平链内 light 元素 rect 同零 |
| i | g − showPopover | mutation | 仍零 | showPopover 排除 |
| j | 3 SR、popoverA 无 slot | 摊平形状 | 仍零 | 「slot 摊平带 SR host」形状排除 |
| k | 2 层链、hostSel 结构串（1 段 nth） | 串形态 | 非零 | 结构串（1 段）排除 |
| l | 2 层链、hostSel 结构串（2 段） | 串段数 | 非零 | 结构串段数排除 |

**最小红态条件：3 层宿主链（host1>host2>host3 各带声明式 SR）+ runner 执行**；≤2 层链、平级 SR、任意串形态全绿。

## 引擎侧三层对照（runner 无关面排除）

- engine 纯管线单测 `test_shadow_nested_flatten_rect_s370`（真语料同构 3 层 SR + 两层摊平链）：**绿**——parse→layout→HitTestCache→fill snapshot→zwsh 解析→查表全链 3 层正常（layout-engine collect_shadow_children 无缺口）。
- webview 单测 `test_shadow_nested_flatten_gbcr_webview_s370`（load_html + execute_script + mutation 注入后二段 + execute_script_with_dom runner 同款入口三段）：**全绿**。
- 断点收敛至 **runner 执行循环生命周期**（prepare_harness_html 装配 + testharness.js 完整执行 + 多轮 promise_test + begin/end_script_batch 的 LIVE_QUERY_DOC/rect snapshot 代际面）——本轮未再深入，挂账独立切片。

## 本刀施工（上一轮 429 中断承接，本轮验证收口）

1. **`unique_selector_for_node` shadow 内容门**（`crates/engine/src/js_dom_bridge.rs`）：parent 链穿 ShadowRoot 的节点返 None。修复前：物化 shadow 树 parent 链达 root（s175 门），唯一性计数走 `query_selector_all(root)`（s168 封装门剪 ShadowRoot 子树只数 light 树）→ light 同身份元素顶替计数达标 → shadow slot 借 light 身份铸跨界结构串（`dialog.dialogB > slot:nth-child(1)`）——该串从 root 解析不回 shadow slot，shim light-dismiss `_zwLdFindSlot` 递归断链。修复后 None → `node_entry_json` 走 s176 compound 回落（`zwsh<len>:<host>|<local>`，`try_shadow_scoped_fastpath` 对称解析）。
2. **消费面审计**（None 回落全部改善方向，无负位移）：
   - `node_entry_json`（child_nodes 条目）→ s176 compound 串（本刀目标面）；
   - handle_selectors（2636）→ None 不入表，handle 直达路径（坏串此前误占 selector_handle_map）；
   - `option_activation_snapshot.select_selector`（4910）→ webview user_actions None → 落 `ActionTargetState::Generic` 派发 click（坏串此前 page_node_ref 失败 → noop NotApplicable 点击哑火）；
   - `next_focus_selector`（5132）→ None → 无导航（与坏串 page_node_ref 失败等价）；
   - hit_test（558/945）既有 `.or_else(shadow_content_entry_selector)` 回落不动。
3. **新钉测**：
   - engine `test_shadow_content_selector_gate_s370`：shadow 内容不借 light 唯一计数/不铸跨界串/light 段不变/compound 串 roundtrip。
   - engine `test_shadow_content_layout_rect_s370`：单 SR shadow 内容 rect 在 snapshot 且非退化。
   - engine `test_shadow_nested_flatten_rect_s370`：3 层真语料同构纯管线 rect 全非退化（layout 面钉）。
   - webview `test_shadow_content_gbcr_webview_s370`：单 SR 全管线 gBCR 非零 + querySelector(zwsh) roundtrip。
   - webview `test_shadow_nested_flatten_gbcr_webview_s370`：3 层链全管线（含 mutation 注入后、with_dom 入口）gBCR 非零。

## 顺带发现（未修，挂账）

- `document.querySelector(zwsh 串)` 在 shim 侧抛 **SyntaxError**（`_zwDsRootHandleForContentSel` 解码只服务焦点面；querySelector/matches 无 zwsh 前缀分派）——shadow 内容经 compound 串走 querySelector 全断。本轮未修（语料无钉面），挂账。

## 验证

- engine lib 全量 **3832P/7F**（fmt 后复跑同数）——7 红与 s369 基线逐名同构（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_nonterminal / websocket_event_flow），P+3 组合态漂移，零新增红。
- webview lib 全量 **846P/2F**（修复树复跑两轮定谳）：2 红 = `image_decoder::proxy_mode_decodes_png_correctly`（image-decoder 进程 spawn 环境红，s168c/s256/s336 三账在案 HEAD 同挂）+ `csp_gate::csp_connect_gate_blocks_fetch_sh1_m2s5`（s348 环境依赖红）——s349 基线 844P/2F 逐名同构，零新增红；新 2 测（二态断言版）全绿。首轮全量曾见第三红 `coverage::test_external_worker_fetch_r3091`（worker fetch 消息往返断言 got "none"）：与改动面无机制交集（无 shadow/选择器路径）、单跑 `--test-threads=1` 绿、次轮全量绿——定谳并行负载 flake 非回归。注：stash A/B 干净树对照因后台启动与 pop 竞态作废（跑的是恢复树），归因改走账本基线对照（s349/s348/s168c 逐名在案）。
- solo 域跑 closedby-complex 24P/16F 零位移（16F=s173 LOST 基线同数，非本刀回归）。
- fmt PASS、clippy `-p zero-engine -p zero-webview --all-targets -D warnings` PASS。

## 挂账（移交下一切片）

1. **runner 3 层宿主链 shadow rect 断**（本刀勘根矩阵已备）：prepare_harness_html 装配 + testharness.js 执行 + 多轮 promise_test/batch 的 LIVE_QUERY_DOC 与 rect snapshot 代际面。解锁键 = closedby-complex shadow 段 16F + 假绿修正（modalB=false 组）。
2. shim `querySelector(zwsh)` SyntaxError 缺口（见上）。
3. 既有在册维持：基建面 T=3（s102/s124）、probe8/9/10 转正决策、customizable-select 126F / dialog 41F 等既有未建面（s366 sweep 口径）。
