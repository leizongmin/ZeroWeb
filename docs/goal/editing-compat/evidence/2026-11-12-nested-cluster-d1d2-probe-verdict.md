# 嵌套簇刀 r4/r5 探针定谳——D2 stop=null 空转 + D1 解析宿主物化身份分裂

- 日期：2026-11-12
- 数据源：`.acceptance/dps/nested-probe-r4.log`（writer 棒 12:07）、`nested-probe-r5-verify.log`（本棒 13:2x，含 idA 身份探针 + rngShape trace + bold 对照）、italic 测试 `--ignored --nocapture` 直跑 ×3（r4'/r5 均经 test-guard 包裹）

## D2 定谳：stop=null 走查空转（writer 修复 + 本棒复验通过）

`_zwValGetAllEffectivelyContained` 主循环 `while (node && _zwJusIsBefore(node, stop))`：选区终点落文档链顶（endContainer 为末节点）时 `NextNodeDescendants` 返 null，`IsBefore(x, null)` 经 `BpCompare` 的 `IsAncestorOf(null, x)=true`（根 parentNode=null 链）+ `oB(0)<=i2(0)` 恒返 false → 走查零迭代 → contained 集空 → 六 format 命令落 override-only 臂（qS 翻转、DOM 零变更）。

- writer 落修复：part01c.js:2430 `(!stop || _zwJusIsBefore(node, stop))`（迭代至树尽），本棒 r5 复验：italic 0..3 containedAll 0→1、DOM 应用发生、ctrl 宿主 `fo<i>ob</i>az` 同步正确。
- bold=1 vs italic=0 矛盾闭环：bold exec 跑在 ctrl 宿主 append 之后，flat2 文本的 stop=ce-x div（非 null）正常走查；italic/ctrl exec 时 body 仅 flat2 → stop=null 空转。**stop 与 offset 无关、与文档拓扑有关**——非命令相关分叉。

## D1 定谳：解析宿主 _ensureMutTree 物化身份分裂（头部选区 wrap 序错）

r5 实测 exec#1（0..3 头部选区、解析宿主 ce-flat2）after=`baz<i>foo</i>`（期望 `<i>foo</i>baz`）；ctrl 宿主（JS createElement 创建、节点有 handle）`fo<i>ob</i>az` 全程正确。机制链：

1. 解析宿主首触 mutable API 时 `_zwParseEl._ensureMutTree()`（part02.js:5774）经 `__zw_parse_html_child_nodes(outerHTML)` **重新解析子树**——物化树里的 text 子是新对象 W2，与 JS 侧已持有引用 W1（splitText 链产物）身份分裂（W1 ∉ mutTree.childNodes）。
2. `_zwVaWrap`（part01d.js:663）`insParent.insertBefore(newParent, nodeList[0])` 的 ref=nodeList[0]=W1 → mutTree ref 查找失败抛 → catch fallback `appendChild(newParent)` → 壳落尾部 → **JS 序错** `baz<i>foo</i>`。
3. mutation 发射降级：wrap 三操作（壳插回/成员入壳，handle 型 InsertBeforeByHandle/AppendChildByHandle）对解析宿主静默或降级 index 型（R2-MUT exec#1 面仅 `RemoveChildAt #ce-flat2 idx0`；ctrl 面四操作全发射）。Rust 侧重放（apply_dom_mutations）缺步骤 → 断言必然 fail。

R2-MUT 对照（exec#1 5 条 flat2 mutation vs ctrl 面全 9 条 ByHandle）：`CreateTextNode baz / SetChildText idx0 foo / AppendChild baz / CreateElement i / RemoveChildAt idx0`。

## 判读与处置建议

- D2 修复（part01c.js 7 行）已复验有效，建议 writer 棒按门禁批先入库协议收口（guard → commit → 重锚 release bin → 批腿 judge 基准复用 gate v4 的 ind-c1/cssom-mt-reanchor/rbv2 日志）。
- D1 面两选项：a) 深修=物化身份保持（重解析物化前按身份对账/引用重绑，双 wrapper 家族 apply 面变体，影响面广须独立刀）；b) 浅修=只解 mutation 发射缺失（JS 序错依旧，重放断言仍不成立，不推荐）。倾向 a，与 writer __zwFmtErr 探针（part06.js 5332 脏改，抓 `_eFmt` 异常面）判读合流。
- 两枚 #[ignore]（nested_cross_b / italic_after_prior_cmd）解除前置=D1 修复 + 净效果期望钉（`<i>foo</i>baz` / `fo<b>o<b>b</b>ar</b>az`）。
- 探针还原义务：part06.rs/part01d.js 探针 hunk 已由本棒 checkout 还原（**披露：checkout 抹掉了 writer 的 part06.rs R2 探针块**——writer 判读依赖时可从 r4/r5 log 的 R2-MUT/R2-PROBE 输出与本文机制链重建）；part06.js `__zwFmtErr` 探针（writer 在飞）未触碰。

## 门禁批收口（batch-nested-d2-gate-v1 ALL-DONE，2026-11-12 15:17）

- 八腿全绿零 ABORT @cf1d2942a bin e50a656（域绑定：EXPECTED_HEAD/BIN md5 + D2 卫兵特征串 d2sig=2 + prev/decl 串，批中复读恒一；14:29 发射、15:17 ALL-DONE）。逐腿：L0 probe 217/5 恰集；L3b ab fresh 10092/0/0（wall=235s 门内完走）；L1 outdent 页键 judge F2P=169/P2F=0/lost=0/残红恰 4；L2 indent 1302/27；L3 mt 10090/2 四零（F2P=0 signed=0 unsigned=0、P2F=0、po_fail=0、lost=0、residF=2=X37 双键残）；L4 singles×5（iol 1759/iul 2061/ip 6989/ilb 1658/ihtml 1446）零漂移；L5 exec×3 绿；L6 enginelib 3908P/7F failed==7 名册逐名恒一（P 3904→3908 为刀间新增单测，门判据是 failed 名册面）。
- mt flip 面判读：发射账预期「语料 F2P signed flip」实测为 0——multitest 语料无文档末选区形行（D2 修复域零语料暴露），flip 面为空非判读错位；红线门（unsigned F2P/P2F/po_fail/lost）全零、residF=2 与 gate v4 重锚恒一，修复域覆盖由钉测 `test_execcommand_format_apply_doc_end_selection_r3254_m3`（修复前 fooobaz override-only 空转红 → 修复后 fooob<i>az</i> 绿）承载。
- writer 遗产处置定谳：part06.js `__zwFmtErr` 探针已 checkout 还原，全批日志零命中——`_eFmt` 未走异常路径，D5（mutation 静默丢弃）定谳为非异常静默路径，探针无产出（上文「未触碰」状态就此更新为已还原）。
- D1/D5 处置维持：独立刀，方向 a=物化身份保持（重解析物化前身份对账/引用重绑，双 wrapper 家族 apply 面变体）；两枚 #[ignore]（nested_cross_b / italic_after_prior_cmd）解除前置=该刀落地（净效果期望钉已在测试体）。

## 门禁批 batch-nested-d2-gate-v1 判读（2026-11-12 15:17:25 ALL-DONE 全绿）

绑 head=cf1d2942a bin=e50a656fe4ace01b168dde55c7b9e727（锚复读 d2sig=2/prev=6/decl=28、探针在位），14:29:39 发射真 pid 652539，日志 `.acceptance/dps/batch-nested-d2-gate-v1.log`：

| 腿 | 结果 | vs 基线（gate v4 census） |
|---|---|---|
| L0 probe zz-probe-outdent | 217/5 恰集 | 恒一 |
| L3b ab fresh | 10092P/0F/0T wall=235s | 严格四零恒一 |
| L1 outdent 主门 | P=2763 F=9；judge F2P=169/P2F=0/lost=0/残红恰 4 | CSSOM 闭合面维持 |
| L2 indent | 1302P/27F | 恒一 |
| L3 mt 主页 | P=10090 F=2 T=0 wall=1495s；judge F2P=0（signed=0/unsigned=0）/P2F=0/po_fail=0/lost=0/残红恰 2 | 残红=X37 双键（subscript-qS-after+formatblock-qV-after）恒一 |
| L4 singles×5 | iol 1759/iul 2061/ip 6989/ilb 1658/ihtml 1446 | 恒一 |
| L5 exec×3 | 三页全 OK | 恒一 |
| L6 enginelib | 3908P/7F/2I | failed==7 名册逐名恒一 |

- **mt F2P=0 判读**：语料 multitest 无 doc-end-selection 形条目，D2 面正向证据由钉测 `test_execcommand_format_apply_doc_end_selection_r3254_m3` 承载（修复前净效果 "fooobaz" override-only 空转红 / 修复后 "fooob<i>az</i>" 绿，双向证据随 cf1d2942a 入库）；零 flip 零 P2F 零 lost = 修复对既有语料面纯增量零扰动。
- enginelib passed 3904→3908 归因：+1=本刀钉测、+3=9341f69be 新增 js_dom_bridge_tests/part110_s411_input_maxlength_number 模块（换基并入入站域）；门判据为 failed==7 逐名恒一，passed 增量非漂移。

## writer 遗产处置与 D5 补记

- part06.js `__zwFmtErr` 探针（writer 在飞半成品）已由收口轮 checkout 还原；全日志零命中 = `_eFmt` 路径未抛异常 → D1 机制链第 3 步 mutation 丢失为**静默降级**（handle 型发射对解析宿主 skip / index 型降级）而非异常吞壳，与 r5 R2-MUT 实证（解析宿主仅 5 条 index 型 vs ctrl 宿主 9 条 ByHandle）一致。
- 13:4x 核验棒自伤披露（checkout 抹 writer part06.rs R2 探针块）已在发射账 13:4x 条留档，可重建。

## 嵌套簇残余 = M-wire 章/裸不对称三缺陷根因图（下一刀对象）

章=持 handle/selector 登记节点，裸=物化未登记子。三缺陷共享根因：

- **Form A**（splitText 裸子 trim 欠发射）：handle 世界 R196（part04.js:3955）`if (handle)` 门对裸子 trim 臂静默跳过 `__zw_set_text_handle`，重放树文本未裁——D2 钉测 v1 形 "fooobaz<i>ob</i>az" 实证（钉测改形避开 split 面）；精确臂（vs part05.js:15148 本地登记文本臂）待探针定谳。
- **B1**（selector 世界 ref 身份失配）：`_zwVaWrap`（part01d.js:643）`insParent.insertBefore(newParent, nodeList[0])` 对解析宿主 ref=nodeList[0] 身份分裂查失败抛 → catch fallback `appendChild` → 壳落尾 JS 序错 `baz<i>foo</i>`（D1 机制链第 2 步）。
- **B2**（movePreserving 裸子 append 静默）：`_zwJusMovePreservingRanges`（part01c.js:717）removeChild index 型发射 OK，insert/append 对裸子静默 → 重放缺成员搬移步骤（D1 机制链第 3 步）。

修复方向：a) **身份/发射统一**（推荐）——selector 域视图 per-generation 身份缓存 + 裸子首触 stamp（须 Rust 契约同步，影响面广独立刀）；b) 浅修=发射补全（JS 序错依旧，不推荐单独走）。解除前置：两枚 #[ignore]（nested_cross_b / italic_after_prior_cmd，期望钉 `<i>foo</i>baz` / `fo<b>o<b>b</b>ar</b>az` 已就位）+ Form A 精确臂探针（rngShape 式 trace 记 __zwHandle/__zwSelector/parentNode.__zwHandle 三值，方案现成）。

## 门禁批收口（batch-nested-d2-gate-v1 ALL-DONE，2026-11-12 15:17）

修复 commit **cf1d2942a**（D2 卫兵 + 文档末选区钉测 `test_execcommand_format_apply_doc_end_selection_r3254_m3` 三 createTextNode 持章节点、选末整节点 stop=null 形 + 本定谳文档）。钉测双向证据：修复前 actual `fooobaz`（override-only 空转）红 → 修复后 `fooob<i>az</i>` 绿；钉测改三整节点形避开 split 面（Form A：handle 世界 splitText 裸子 trim 无发射=既有共享面挂账，见下）。

批判读（发射 14:29:39，锚复读过 head=cf1d2942a bin=e50a656fe4ace01b168dde55c7b9e727 d2sig=2/prev=6/decl=28，ALL-DONE 15:17:25 零 ABORT，日志 `.acceptance/dps/batch-nested-d2-gate-v1.log`）：

| 腿 | 终值 | 判据 | 结果 |
|---|---|---|---|
| L0 probe | 217/5 | 恰集 | OK |
| L3b ab fresh | 10092/0/0 wall=235s | 基线全 P 严格 + judge 四零 | OK |
| L1 outdent | rows 2550 恒一 F2P=169 P2F=0 lost=0 residF=4 | vs ind-c1-outdent census | OK |
| L2 indent | 1302/27 | vs census 恒一 | OK |
| L3 mt | rows 10092 恒一 **F2P=0（signed=0/unsigned=0）** P2F=0 po_fail=0 lost=0 residF=2 wall=1495s | vs cssom-mt-reanchor | OK |
| L4 singles×5 | 1759/2061/6989/1658/1446 | 基线恒一 | OK |
| L5 exec×3 | — | 零漂移 | OK |
| L6 enginelib | 3908P/7F | failed==7 名册逐名恒一（grep 修正版） | OK |

- mt 零翻越判读：语料 multitest 无「选区终点落文档链顶」形行——D2 修复对既有语料净零行为变化（红线 unsigned/P2F/po_fail/lost 全零），行为正确性由钉测承载；残红恰 2=X37 双键（subscript-qS-after + formatblock-qV-after）恒一。
- D5 合流定谳：writer `__zwFmtErr` 探针全日志零命中=未捕获任何异常——wrap ref 失配走 catch→appendChild fallback 是静默控制流非异常路径；探针无产出，已 checkout 还原（账本 13:4x 条披露还原范围）。

## 挂账移交：嵌套簇根因刀（M-wire 章/裸不对称三缺陷）

三缺陷共享根因=mutation 发射按「章」（handle/selector wrapper）门控，物化/搬移产物是裸节点（无章）时静默跳过：

- **Form A**：handle 世界 splitText 裸子 trim 欠发射（part04.js:3955 R196 `if (handle)` 门——裸原件 `cur.slice(0,o)` 截断零发射，重放树文本未裁）；精确臂待 rngShape 探针定谳（1293-1309 合取方案现成）。
- **B1**：selector 世界 `_zwVaWrap`（part01d.js:643）`insParent.insertBefore(newParent, nodeList[0])` ref=nodeList[0] 为 JS 侧旧对象、mutTree 重解析物化后身份分裂→查找失败抛→fallback `appendChild` 序错（D1 已定谳，机制链见上节）。
- **B2**：`_zwJusMovePreservingRanges`（part01c.js:717）removeChild index 型发射 OK，insert/append 裸子静默。

修复方向：a) 身份/发射统一（sel 域视图 per-generation 身份缓存 + 裸子首触 stamp，须 Rust 契约同步，影响面广独立刀）> b) 发射补全浅修（JS 序错依旧，不推荐单独走）。两枚 #[ignore]（nested_cross_b 期望 `fo<b>o<b>b</b>ar</b>baz` / italic_after_prior_cmd 期望 `<i>foo</i>baz`）钉已就位，解除前置=D1 修复+净效果期望验证。
