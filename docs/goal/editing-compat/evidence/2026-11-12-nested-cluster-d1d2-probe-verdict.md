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
