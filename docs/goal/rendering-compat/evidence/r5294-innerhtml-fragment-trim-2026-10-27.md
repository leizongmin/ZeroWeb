# R5294 — innerHTML/insertAdjacentHTML/outerHTML fragment 解析去 trim——dir-style-04 ✓0.00

日期：2026-10-27。改动：`crates/engine/src/js_dom_bridge.rs`（+15/−9）、`crates/engine/src/js_dom_shim/part04.js`（+7/−4）、`crates/engine/src/js_dom_bridge_tests/part02.rs`（+66）、`crates/layout-engine/src/engine/tests/`（新锚测试 +1）。无新 kill-switch（见「取舍」）。

## 案面

`css/selectors/dir-style-04`（:dir() 运行时追加，fuzzy 预算 maxDifference=0-70; totalPixels=0-180）：`onload` 对 ltr/rtl 两个 div 各 `appendChildren`——`x.innerHTML = "This span should inherit dir from the parent. "`（尾随空格）+ `y.innerHTML = "…grandparent."`，`x.appendChild(y)` 后 `div.appendChild(x)`。期望渲染 "…the parent. This span…"（行中空格）；ZW 渲染 "…parent.This"（空格丢失），ltr/rtl 两臂同红 ✗0.67。

## 取证链

1. PNG 像素取证（前轮）：丢格点定位在 **x 的 innerHTML 文本与子 span y 的 junction**（"parent.This"），非 div 既有文本与追加 span 的外层 junction。
2. 探针矩阵（前轮，probe-static/dyn/long/offline + 字号/族/行高矩阵）：静态同形标记保格；JS 动态构建丢格；与字体度量无关。
3. 本轮 in-process 排除布局面：`zero_dom` 变更 API（create_element/create_text_node/append_child）构建同形 DOM 后全量 `LayoutEngine::compute`，静态 vs 动态 span 盒 x/width 逐项一致（该锚固化为 `r5294_dyn_trailing_space_tests`，修前即绿——布局无辜的证据固化）。
4. 桥面审计定位根因：`replace_inner_html`（js_dom_bridge.rs:1770）`let trimmed = html.trim()`——**尾随空格在 DOM 构建前就被 trim 丢弃**。shim 本地视图解析（`_zwMBuildBodyTree` → `__zw_parse_html_child_nodes` 整包解析）不 trim，两端视图此前已失步。

## 根因

HTML spec fragment parsing algorithm 对入参串无 trim 步骤（innerHTML/insertAdjacentHTML/outerHTML 三 setter 共用该语义，https://html.spec.whatwg.org/multipage/dom.html#dom-element-innerhtml 等）；首尾空白是文本节点内容的合法部分（`el.innerHTML = " "` 应产出一个空白文本节点）。`replace_inner_html` / `insert_adjacent_html` / `replace_outer_html_node` 三处均在解析前 `html.trim()`，违反 spec 且与 shim 视图不一致。

## 实现

1. **host 三处去 trim**：三函数改 `if html.is_empty() { return/跳过 }` + 直接解析原串（空串早退语义不变：innerHTML=''/IAH ''=无操作、outerHTML ''=仅移除目标）；空白-only 串现在按 spec 产出空白文本节点。
2. **shim 视图同步**（part04.js:12084）：`_zwFragmentAdded(_ihVal.trim(), handle)` → `_zwFragmentAdded(_ihVal, handle)`——该 trim 是 t7-r2/D1 修复为对齐 host trim 而设（child-index 对齐契约）；两端一起去 trim 后逐子对齐保持。IAH/outerHTML 视图路径本就传原串，无需动。
3. 涉及其他 trim 点核实不动：detached-stash 重解析的 `html.trim()`（stash 存 `outer_html(node)` 恒以 `<tag` 开头，trim 无作用面）；shim cloneNode 深拷贝路径原文往返无 trim。

## 回归测试

- engine 桥测 +3（part02.rs，`apply_mutations_to_html` 全链路）：innerHTML 尾随空格保留（dir-style-04 原串）、前导空白保留（D1 对齐前提）、IAH beforeend + outerHTML 端侧空白片段。**修前 3/3 FAIL**（旧输出 `<div id="d"><b>y</b>x.</div>` 双空格全丢）、修后 3/3 PASS（单文件 checkout 对照法）。
- layout-engine 结构锚 +1（`r5294_dyn_trailing_space_tests`）：动态构建 DOM 与静态同形标记布局逐盒一致（排查证据固化，非 bug 复现）。

## 交付

- 目标案 `dir-style-04` **✗0.67 → ✓0.00**（真通过-可信，strict 容差下非近纯色）。
- 全量 corpus **16590/15731P/859F** vs R5293 交付态 15730P/860F：失败集归一化 diff 恰为 {-dir-style-04}，零新增翻转。静态 reftest corpus 少脚本面，故仅一案受益；动态影响面由 engine 桥测与 make test WPT 腿覆盖。

## 取舍：不设 kill-switch

本轮改动为 setter 入参处理的 spec 一致性修复，且 host/shim 两侧**必须同态**（host 单侧回 trim 会复活 D1 child-index 失步）——kill-switch 只能包双侧，复杂度不成比例；corpus 零翻转 + engine 3533 全绿 + product-smoke 持平构成回退判据（revert 单 commit 即可）。

## 验证

- make reftest 716/716；engine lib 3533P（+3）；layout-engine lib 2163P（+1）。
- fmt 0 diff；clippy `-p zero-engine -p zero-layout-engine --all-targets` 0 warning/error。
- product-smoke 6/6 struct PASS + welcome oracle 15.50% 持平；bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-engine` GATE PASS（19 指标，NEW=7 为该 crate 首次定向建档，全部预算内）。
- make test：workspace/renderer/browser-bin/gpu/quickjs 组合腿绿。**跨流红清账**：rebase 后 PR #234（`site-opt/20261026-p107-probe-repin`）合入 main，cdp `default_async_probe_gives_up_frame_attached_on_busy_iframe_page` standalone 复验 9/9 绿（R5293 对 PR #230 automation 面的归因经对端流返修确认关闭）；image_batch_e2e 本轮首跑 F 为 `zero-browser` 二进制未构建（spawn 缺失报错非行为红，显式 `cargo build -p zero-browser -p zero-renderer -p zero-compositor` 后该腿绿——R5293 陈旧二进制教训复现）。

## 教训

- 「动态页面渲染差异」未必在渲染管线：DOM 桥的**内容处理**（trim/规范化）先于布局改变树形态。像素取证锁定丢格点后，先以 in-process 布局重算排除布局面（一条测试即可定谳），再沿 DOM 构建链审计，比直接读渲染代码收敛快。
- 修 spec 违例时先查「谁为绕过它打了补丁」：shim 视图的 trim 是 D1 对齐补丁，单侧去掉会复活旧 bug——耦合点靠 part04.js 内 t7-r2 注释锚定，grep `.trim()` 全调用面 + 追注释引用缺一不可。
