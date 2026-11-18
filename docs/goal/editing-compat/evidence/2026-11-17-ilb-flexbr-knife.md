# ilb I flex/grid br 元素面刀收口（ILB-FLEXBR，2026-11-17）

刀体 5dc363b35＋v1.1 垫节点三分修复 baaa8f1a3（`fix(editing): ilb I
flex/grid br 元素面刀——声明 pre 系坍缩行断 <br> 占位（ILB-FLEXBR）` ＋
`垫节点三分恢复 br 型 <br> 垫`）＋门禁批 i7f2 ALL-DONE（head=b6e2ddc05，
bin 03886ee67e1cec2b55cd7b6a36cbbb3b，2026-11-17 05:11 完走）。census 刀序
④兑现，F2P 恰 4 键零新增红，ilb 残 13F。

## 基线

刀④基线＝i7e2 门禁双腿（1674P/17F head=7356bf2aa，03:22 完走）——刀③
fresh 即现树面态，协议免重跑基线腿。组合态注意：现树含并行编辑流 d48
join 壳吸收刀（70b18f81e，part01.js join/FD/select-delete 臂），本批即
d48＋flexbr 双刀组合态首验。

## 根因与修复面

I 族 4 键（display:flex / inline-flex / grid / inline-grid ×
white-space:pre `<span>abc[]</span>`）：wsSig（\n 型）臂坍缩行断（caret 在
内容尾、`\n` 无后继）垫旧规则补 `\n` 文本（got `abc\n\n`），Chrome 142
实录补 `<br>` **元素**（exp `abc\n<br>`，语料 ：925-932 flex／:987-996
grid，注记 "based on Chrome 142"）——br 忽略容器（flex/grid item 化）仍须
插 br 元素非文本换行。非坍缩行断（`[]abc`／`a[]bc` 现役绿 8 键，`\n` 有
后继）不触发垫，零扰动；同族非 pre 形（`abc<br><br>` 双 br 垫）本就绿。

修复＝垫节点按 wsSig 来源三分（`_zwCeLineBreakAt` 垫块）：wsSig∧forceLf
（inline div 无声明，cannot-have-div 语料域 `a\nb` 行断形，无垫形语料锚）
维持 `\n` 垫；其余（br 型 wsSig=false＋声明 pre 系 wsSig=true）一律
`document.createElement('br')` 垫。坍缩补断概念与 spec isCollapsedLineBreak
（:795 zwsp 布局探针／:7459 补第二条）同源，shim 无渲染域以「`\n` 无同父
后继」结构判定（wsSig 域维持旧同父兄弟规则）。

## v1.1 归因修复（i7f0 v1 ABORT）

v1 门禁 ilb-a 1635P/56F ABORT：v1 三元 `(wsSig && !forceLf) ? br : '\n'`
把 wsSig=false br 型臂翻成 `\n` 垫——`<div>foo[]</div>` got
`<div>foo<br>\n</div>`（exp `<div>foo<br><br></div>`）P2F 39 键实录（垫
触发的全部 br 型键，`foo[]`／块尾／表表格簇）。原形 `wsSig ? '\n' : br` 的
br 型臂语义被反写。修复：垫节点三分。ilb 钉测全家 4 passed/0 failed
（part126＋127＋128＋129）。

i7f1 judge 前置 ABORT（脚本期望算术错，非引擎债）：ilb-a 1678P/13F 键集
零新增红恰 4 刀键翻绿——期望表漏算 F→P 翻绿同步 +4（1674+4=1678，刀③
i7e0 同型 1669+5=1674 先例），i7f2 修期望重发射。

## 钉测（part129，同提交常驻）

`part129_ilb_flexbr_pins.rs` 6 钉：I 族 4 语料形（idx 925/930/987/992）＋
非坍缩行断控制 2（idx 905 `[]abc`→`\nabc`／idx 915 `a[]bc`→`a\nbc`）。
定向 test-guard 包裹 1 passed/0 failed（库总数 4099→4100）。

## 门禁批链（i7f0 → i7f2）

- **i7f0 v1 ABORT**（04:32 head=5dc363b35）：ilb-a 1635P/56F，v1.1 修复
  baaa8f1a3。
- **i7f1 ABORT**（04:36 head=b6e2ddc05）：ilb-a 1678P/13F——键集已零新增红
  恰 4 刀键翻绿，纯脚本 P 期望算术错（见上），i7f2 修期望重发射。
- **i7f2 ALL-DONE**（04:38:07 pid=1628824 → 05:11:49）：

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=03886ee6… ksig=1 head=b6e2ddc05 |
| L1/L2 ilb 双腿 | 1678P/13F/0T | 1678P/13F/0T 双腿同数（37s/37s） |
| L3 键集 judge | 零新增红＋F2P 恰 4 I 族刀键 | F2P-EXACT-4（白名单同配方提取恒一） |
| L4 enginelib strict | 0F | **4100P/0F**（含 part129 新钉） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** vs ihe6-mt 基线恒一（wall 1925s；d48 join 刀组合态首验通过） |

批工件 `.acceptance/dps/i7f0-*`（v1 ABORT）／`i7f1-*`（脚本期望 ABORT）／
`i7f2-*`（终判）。

## 尾巴

ilb 残 13F＝B 元素端点选区删除旁路 6＋G 块级壳全含 1＋E table 选区/落位 4＋
D rawtext no-op 2。census 刀序 ⑤**B+G+E 选区删除域 8**（+3 E1 等待归域，
跨三子机制，拆子族借 ip M3/ihtml 轴二刀经验）→ ⑥**D rawtext 2**。
