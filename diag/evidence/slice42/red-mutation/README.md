# slice42 变异 RED 归档

- `natural-red-prefix-s42.log` — 修复前自然红：4 根新钉 FAILED（① morph 回收 + 同路径 batch 回收、③ 树序两面）；两处既有钉翻转钉在该 run 中跑的是翻转前旧断言、对修前代码为 ok（「+2 翻转面」指覆盖形态，非红）。
- `mutation-only-delta-s42.diff` — RED 时点修复态→变异态纯 delta；hunk 头行号即 RED 时点工作区版行号（与下表锚 1 一致）。
- `mutation-red-s42.log` — 变异态定向红：21P/6F，6 红全数命中三修复面覆盖测试（尾部 TREE_SHA）。
- 逐字节还原自证：RED 时点修复态工作区文件 sha256 `79a654cc4a71df608c0eccb8264e3940d58be319c344bd3f2270dd7c7e324a7f`，还原后实测同值（RESTORE_BYTE_EXACT_OK）。

## 变异位点行号锚表（2026-10-08 主控勘误订正版）

| 位点 | 锚 1：RED 时点工作区版 | 提交态 `79cd8b98a` | 锚 2：终锚 `402dc9e85`（= aae73e4c9/HEAD 同字节） |
|---|---|---|---|
| M-C1 `if (inDoc)` → `if (false)` | L10466 | L10466 | L10472 |
| M-A `if (elsM.length === 0)` → `if (false)` | L10834 | L10835 | L10841 |
| M-B `__zwNADelete` 双面块（try/if/else/catch 4 行）→ plain delete | L10852-10855 | L10853-10856 | L10859-10862 |
| M-C2 `out32.sort(...)` 3 行 → 移除 | L10879-10881 | L10880-10882 | L10886-10888 |

提交态与终锚行号均主控 2026-10-08 以 `git show` 实测确认。平移口径：提交态→终锚 = +6（二轮 merge 位点前插入）；RED 时点（锚 1）→终锚 = M-C1 +6、其余三位点 +7（差 1 = 提交前那 1 行注释级修订，其位于 M-C1 之后 M-A 之前）。

## 勘误（2026-10-08，首轮评审 defect S-1 + testeff xS-1/xS-2/xS-3/xI-2/xI-4 指认，主控核实后订正）

1. **基线身份**：初版「行号锚：修复态 = slice42 提交 `79cd8b98a`」不成立——RED 时点修复态为工作区文件（sha256 `79a654cc…`），与提交 `79cd8b98a` 的 part05.js blob（sha256 `7736040396b349107569d8784314549fcecc9208053cea6c3f08294d011e4f81`）差恰 1 行（M-C1 hunk 后至 M-A 前的区间内、缺陷轮评审判定为 slice42 注释块内注释级修订；确切行未入归档，无法从证据重构）。sha 自证对只证明 RED 运行内部一致（变异前后同值），不证明与提交态字节同一。
2. **提交态重导**：以 `79cd8b98a:part05.js` 为基线：fixed = `77360403…`；fixed + `mutation-only-delta-s42.diff` = `92f2e44622f370ca137bc7d199cbc4dfaf5de88c58a2d98a5a8894970385d0c4`（patch(1) 应用成功，M-A 起各 hunk 相对提交版 +1 行偏移；`git apply` 严格模式需先复现工作区版）。变异语义结论不受影响——四变异位点 minus 块内容与提交版逐一吻合（缺陷轮 §6 实测）。
3. **锚表订正**：初版锚 1 三处（M-A/M-B/M-C2）实为 RED 时点工作区行号（提交版在前述区间后各行 +1）；锚 1 M-B 原申报 3 行（10852-10854）实为 4 行块；锚 2 M-B 原申报 10861-10862 实为 10859-10862；初版「锚 1→锚 2 平移 +7」为单值口径（对 M-C1 不成立），平移正确口径见锚表注（提交态→终锚恒 +6；RED 时点→终锚 M-C1 +6、其余 +7）。
4. **run 数演化**：natural-red 26 run（22P+4F）→ mutation-red 27 run（21P+6F），差 1 = ② 边界钉（dead_handle，现状正确钉恒绿）在两 run 之间补入。
5. **scoped 复跑口径订正**（testeff xS-2 / 缺陷轮 I-4）：run3 实证 3/3（`--bin`）；此前交付申报「6/6」中 3 次为 `--lib` 无效尝试（error: no library targets，`SCOPED_RERUN_EXIT=1` 如实留档），非测试通过。三连留档口径：run1 3/3、run2 5/5、run3 3/3。
6. **残余申报自然红降格**（testeff xS-3）：verdict「残余面（slice32 集合面 parsed 子树移除维护缺口）自然红实证」中 parsed-box 形态的自然红系判定过程观察、未归档 log——降格为「过程观察，未归档」；slice43 立项该缺口时以常驻钉补证。
7. **WPT 兜底选择说明**（testeff xI-4）：上游 named-objects.html order 用例修前即 Pass（未覆盖中插/attr-join 形态），故未走 import-wpt 而以 part42 两根树序单测兜底。
8. **钉位次实测行号**（testeff xS-5）：`crates/engine/src/js_dom_bridge_tests/part42.rs` fn 行——`named_access_morph_product_zero_hit_recycle_s42` :36、`named_access_batch_zero_hit_recycle_backing_s42` :79、`named_access_live_collection_tree_order_childlist_s42` :115、`named_access_live_collection_tree_order_attr_join_s42` :152、`named_access_dead_handle_proxy_entry_boundary_s42` :194（主控实测确认）。
9. **fmt 留档补齐**（testeff xS-4 / 缺陷轮 I-1）：`gates/fmt-s42.log`（TREE_SHA 头 + FMT_EXIT 尾）随本勘误提交补档。
