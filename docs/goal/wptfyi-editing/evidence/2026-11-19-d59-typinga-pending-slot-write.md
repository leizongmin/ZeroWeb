# d59 · 刀 E 落地——sel 域 parsed 文本 `_write` 全 miss 消费 pending slot：TypingA 替换面陈旧寻址修复

**2026-11-19 · d59 轮**。d58 残余 11 键中的 TypingA 双插簇（×4：non-collapsed?TypingA
＋zz-probe-d46b?TypingA 各含 `abc[<p>]def<br>ghi` 跨块形与 `abc{<br>}def` br 形）。
本轮 DIAG57 wire 批复拍（zz-diag-bs14 姊妹页）定谳病**不是 copy 双插**而是
**陈旧寻址写入**，刀 E（part05 `_write` 全 miss 后消费 `_zwSelPendingParent`）落地。

## 1. 定谳：wire 批＋引擎终态对账（形 1）

bs14 形 1（`initializeTest("<div>abc<p>def<br>ghi</p></div>")`＋
`setBaseAndExtent(abc,3,def,0)`＋sendKeyA）7-op wire 批（d59-knifeE-runner 前置
诊断二进制拍得，插桩已回退）：

```
[SetInnerHtml { div, "<div>abc<p>def<br>ghi</p></div>" }, CreateTextNode { __n0, "abc" },
 CreateTextNode { __n1, "def" }, RemoveChildAt { p, 0 }, Remove { "p > br:nth-child(1)" },
 InsertAdjacentText { p, beforebegin, "def" },                      ← R5450 wire（join 搬 def 入 div）
 SetChildText { parent_selector: "p", child_index: 0, text: "adef" }]  ← 病灶 op
```

- 与 d58 caret 形同构前 6 op，但**无 op7 copy**（刀 D 守卫消费生效）而尾 op 是
  `_sdFinish` 键入拼接的 `SetChildText`——**寻址 `{p, 0}`＝引擎 p 的 child 0
  （ghi）**，engine 终态 `div=abcdef / p=adef`（ghi 被毁、'a' 落错文本）。
- 期望形 `<div>abcadef<p>ghi</p></div>`（corpus accept[0]）。
- **机制链**：键入替换面非塌缩选区走 `_zwCeSelectDelete(…, 'insertText', {data})`
  （part01:11490 刀面，d45）→ 跨块臂 `_zwCeJoinMutate`（与 caret delete 同机制）→
  takeLineFrom 摘 def（tombstone）→ 本地 insertBefore 命中 R5450 臂＝**wire＋簿记
  （`_zwSelPendingParent.parentSel`=新父 div、`_zwForceParentLink`），引擎节点不搬**，
  引擎侧 wire 新建 def 于 div（part04:10091-10141 全文核）→ `_sdFinish` 对 shim def
  节点 `nodeValue=` 拼接 'a' → part05 sel 域 parsed 节点 `_write` 闭包**捕获构造时
  parentSel（"p"，world-sync 重建时的原父）**：identity miss（p 融合视图无此节点）、
  戳槽校验 fail（戳 0 位=ghi，data 不符）、data 回退 miss（p 视图无 "def" 文本）→
  **三重全 miss 后旧码回退 `node.__zwChildIndex||0`＝陈旧戳** → SetChildText 打在
  引擎 p child 0（ghi）上。

## 2. 形 2 分形：跨测 ghost 毒化（非同病，不入刀 E）

bs14 形 2（`<p>abc{<br>}def</p>`＋(p,1)-(p,2)＋'a'）隔离页 dump
`join="<p>abcadef</p>"`＝**期望形**（wire 批 2 op：Remove br＋SetChildText{p,1} 语义
正确）；corpus 上下文 got `<p>abca<br>def</p>`（br 存活＋拼接打 abc）＝页内跨测
ghost 毒化（d56b ③ 族在册形），与刀 E 病不同域。corpus 全文件腿见 §4——形 2 两键
d59 维持红。

## 3. 刀设计（最小一处）

part05 sel 域 parsed 文本 `_write`（:13217 后）：identity/戳槽/data 三重全 miss 且
`node._zwSelPendingParent.parentSel` 存在且 ≠ 构造时 parentSel 时，按新父重查
（identity 优先、data 回退同口径），命中即改写闭包 `parentSel`（后续写同址）；
全 miss 保持旧回退（陈旧戳）零行为变化；`parentSel` 同父（R97 重注册形）不入新径。
无热路径成本（仅在三重全 miss 的失败路径上多一次 pending 槽探测）。

## 4. A/B（全套件电池，base＝d58 终版产物）

- 定向腿（先证）：non-collapsed-selection 与 zz-probe-d46b 两文件各共键 207
  **F2P=3/P2F=0**——目标形 1 键（`TypingA at <div>abc[<p>]def<br>ghi</p></div>`
  innerHTML）＋`abc[<ul><li>]def` list-join 两 bonus 键（同病同修：join 搬文本后
  `_sdFinish` 拼接同样按陈旧 parentSel+戳写）；形 2 键维持红（ghost 毒化域）。
- **input-events** 全套件（/tmp/d59-suite-knife.json vs /tmp/d58-suite-knife.json）：
  **F2P=6/P2F=0**＝恰为 2 目标键＋4 bonus（non-collapsed?TypingA +3、d46b?TypingA
  +3）；零键回归。
- 对 d56r 真 base 口径：d58 净 +806 → d59 **净 +812**（+6 无 P2F）；d58 的 11 键
  P2F 中 2 键翻 PP、9 键残余分形不变（形 2 ghost ×2／bold 壳 bi=0 no-join ×5／
  zz-diag bi=0 ×2，d58/d59 失败消息逐字节相同＝零抑制）。
- **dom**（57858 共键）/ **uievents**（2400）/ **selce**（430）：**三零回归**。

## 5. 哨兵（enginelib）

**4227P/0F**（exit=0，29.21s；d58 为 4221P——+6 测试＝并行流入树）。

