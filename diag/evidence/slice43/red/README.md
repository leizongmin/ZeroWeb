# slice43 自然红 / 变异 RED 归档

- `natural-red-s43.log` — 修复前自然红（钉提交 `6fa7a564d` 树 `9ec6a77cf8ac`，修前 part05.js）：双钉 0P/2F，EXIT=101。
  - `named_access_collection_parsed_subtree_remove_s43` 首红断言 `__r_na_after`（held NA 集合 stale 2 vs 0；tag held/fresh 两断言在其后，修复态绿 run 覆盖）。
  - `named_access_attr_join_detached_gate_s43` 首红断言 `__r_detached`（detached 混入 3 vs 2）。
- `fixed-green-s43.log` — 修复态复绿（修复提交 `8a43654d7` 树 `c2d1cc741059`）：2P/0F，EXIT=0（双腿含 tag held/fresh 查询面与 in-doc join 对照臂全绿）。
- `mutation-only-delta-s43-a.diff` — 变异 A：vs `8a43654d7` 纯 delta，单腿撤 `_zwHCCollectSubtree` parsed 后代展开块（`part05.js` 提交态 hunk 头 L10232，-5 行）；与修前码在钉 1 判定面行为等价（保 R51c 桶回落 + 撤 slice43 展开）。
- `mutation-red-s43-a.log` — 变异 A 定向红（工作区含 delta，TREE_SHA 同上）：pin1 FAILED / pin2 ok，EXIT=101。
- `mutation-only-delta-s43-b.diff` — 变异 B：vs `8a43654d7` 纯 delta，单腿撤 `_zwNAAttrChanged` join 连接性门谓词（`&& _zwDocContains36(el)`，提交态 L10903，1 行替换）。
- `mutation-red-s43-b.log` — 变异 B 定向红（工作区含 delta，TREE_SHA 同上）：pin2 FAILED / pin1 ok，EXIT=101。
- 逐字节还原自证：两次变异还原后 `part05.js` sha256 均 `14143a065f8a944656bbe28fa08253db3d4f8aa4b8802f6c77606d317eccdb53`（= `8a43654d7` 提交 blob，工作区 sha256 与 `git cat-file` 内容一致）。

## 变异单腿充分性

两变异位点均在 `crates/engine/src/js_dom_shim/part05.js`（zero-engine 域谓词）。engine 不在 `QUICKJS_TEST_CRATES`（Makefile :190：`zero-script-sandbox zero-webview zero-browser zero-renderer zero-webview-demo zero-integration-tests zero-wpt-runner`），quickjs 测试腿不执行 engine 测试 → 单腿 RED 结构性充分（slice41/42 硬标准），双腿仅在此仓其他域要求。

## 钉位实测行号（终态 `8a43654d7`，2026-10-08 实测）

- `crates/engine/src/js_dom_bridge_tests/part43.rs` fn 行：`named_access_collection_parsed_subtree_remove_s43` :40、`named_access_attr_join_detached_gate_s43` :115（`s43_sandbox!` :12）。
- 修复位点（`crates/engine/src/js_dom_shim/part05.js` 提交态）：`_zwHCCollectSubtree` parsed 展开块 L10232-10240（`_childNodeList` 调用 L10236）；`_zwNAAttrChanged` join 门 L10903。
