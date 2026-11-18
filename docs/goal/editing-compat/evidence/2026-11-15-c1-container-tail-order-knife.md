# C1 容器尾拆分序反转刀——根因定谳与收口（2026-11-15）

## 族面与目标

ip 残 28F 的 C1 族 8 键：bq-bare×2、bq-br×2、td-single×2、td-cross×2（insertparagraph.js :640-658 bq 两形 + :38-56 td 两形，dPS div/p 双面）。四形 got 同签名：空块居前（`<blockquote><div><br></div><div>foo</div></blockquote>`），exp 全为内容块在前、caret 块空垫底（`<div>foo</div><div>{}<br></div>`）。`<blockquote><p>foo[]<p>bar</blockquote>` 中插形不属 C1（innerHTML 恒等，caret 面败，另族）。

## 根因链（探针全闭环）

1. **入口提升**：caret 文本尾位（`foo[]` offset=len）在 part01f.js:1956-1959 提升 `node` 至父块（`node=bq, offset=1+idx(text)`），1967 `collapse(bq,1)` 重锚活动选区。
2. **wrap 改指**：bq 非 SLC → 爬升到编辑宿主 → 走 1993 非 SLC 支路 wrap 面（2215 `_zwJusWrap`），container 重指新建 div-w（`bq>[div-w>foo]`）。提升位（bq,1）此时已落在 div-w **域外**。
3. **域外 range**：主拆分 2430 `newLineRange.setStart(arMain.startContainer=(bq,1))` + `setEnd(container=(div-w,1))`——起点在 container 域外（spec 该步起点恒在域内：spec 爬升先于 range 构造、无 wrap 改指）。
4. **extract 整克隆**：`_zwIparaExtractRange`（1603）sc=bq≠ec=div-w → cac=bq、lastPartial=div-w → 1719 臂克隆 div-w、子区间递归 `cloneSlice(foo)+DeleteData(foo,0,3)`——foo 以嵌套 div 克隆形态入 frag→newContainer（2503），原 div-w 清空。
5. **下钻展平 + 误垫**：2813 禁段落子下钻 `newContainer=newContainer.lastChild`（div 克隆在 prohibited 表）→ newContainer 重指克隆（序列化展平为 `<div>foo</div>`）；div-w 空置 → 2816 Arm L 裸垫 br、2932 caret 落 div-w。

**探针证据三源互证**（part119_c1_mut_probe_tmp + part119_c1_probe_tmp，游离不提交）：

- 时序快照（CE/collapse 处 innerHTML 快照）：`COLLAPSE<BLOCKQUOTE>1`（=入口提升重锚）→ CE:div#1（div-w）→ CE:div#2 时 `<div>foo</div>`（wrap 完成）→ CE:br（Arm L）时已 `<div></div><div>foo</div>`（挪动窗=2491-2854）→ `COLLAPSE<DIV>0`。
- 引擎侧 MUT 序：`SetChildTextByPath [0,0,0]=""`（div-w 清空同步）+ `[0,1,0]="foo"`（新块得内容）——shim 对宿主树的最终意图与 got 逐字吻合。
- 静态读码：如上 5 步链全部落位；concat→source 偏移 136613（br CE@139466=Arm L 2854）核验一致。

## 刀体（part01f.js 主拆分起点源修正）

2430 setStart 源从 `arMain.startContainer/startOffset` 改为归一前拆分位 `ipM1OrigNode/ipM1OrigOff`（piece-3 同源捕获，1953-1955）：

- 域内判：自拆分位上行找 container；命中且 parentNode 活 → `setStart(ipM1OrigNode, ipM1OrigOff)`——wrap 后拆分位随内容入包裹块，域内性恢复；下方归一两 while 恒能从原位重放出提升位（prohibited 止步），SLC 无 wrap 面恒等不受扰。
- 拆分位被摘（parentNode null）或域外（delete 落块界形）→ 收拢 `(container, len)`（行尾面语义：左段原文保留、新块空垫底）。

bq-br 面连带修复：尾随 br 已被 wrap 面卷入 div-w（nodeList push 臂 2211-2214，br 为 p-allowed），拆分位起点 `(text,3)` 归一提升到 `(div-w,1)` + end `(div-w,2)` → extract 移 br 入新块 → 新块 `<div><br></div>` 无双垫（2905 可见性门生效）。

## 钉测 RED→GREEN（part119_c1_order_pins.rs，8 键全形）

| 钉 | 形 | RED got | GREEN |
|---|---|---|---|
| c1_bq_bare_div | `<blockquote>foo[]</blockquote>` dPS div | `<div><br></div><div>foo</div>` | ✓ exp 唯一形 |
| c1_bq_bare_p | 同 dPS p | 同签名 | ✓ |
| c1_bq_br_div | `<blockquote>foo[]<br></blockquote>` dPS div | 同签名 | ✓（尾 br 入新块无双垫） |
| c1_bq_br_p | 同 dPS p | 同签名 | ✓ |
| c1_td_single_div/p | `<td>fo[o…` 拆分面（删除前言绕行） | `<div><br></div><div>fo</div>` | ✓ |
| c1_td_cross_div/p | `<td>fo[o<td>b]ar…` 拆分面（同上） | 同签名 | ✓ |

钉 JS 须走 JS 侧 `innerHTML` 赋值路径（引擎预设 dom_html 物化会让 shim 视图分叉，探针同教训）；bq 文本导航深度 2 非 3（三跳 setStart(null) 抛 InvalidNodeTypeError 伪红，已修）。

## enginelib 全量

zero-engine `--lib` 4021P / 0F（16.03s；2 败=两枚游离探针按设计 panic，提交时已摘除）。验证域=组合树（并行轮 part01c.js:1110 补偿刀在树时跑定；该刀改共享 wrap 补偿 `=== ni`→`>= ni`，不并入本提交——归因分离，其独立增量由作者轮自证）。钉测 GREEN 于组合树三度复验（含桩摘除+startInTail 复原后）。

## 另档残留线索（非 C1 域，待普查）

跨表 delete 边界偏差：钉 RED 期实录 `<td>fo[o</table>b]ar` 选删后尾字符多吞（`ar`→`r`，dPS div/p 双面同）；corpus 全序列化 judge 若 td 键翻绿不全，先查该面。未入 28F census 键集（census 以全序列化计），独立排查。

## 协调

并行轮（96d0179e）同期同域诊断：C1DBG 桩 17 行（2491+ 域 + _eP1-3 单行桩，trace push 无行为面）由本轮提交前摘除（摘除中误伤 `startInTail` 声明行已复原并复验）；分域、根因交接与 part01c 补偿刀交叉通告见 `.acceptance/dps/COORD-ipm3-c1-order-reversal.md`。桩摘除后钉 8/8 GREEN 复验。
