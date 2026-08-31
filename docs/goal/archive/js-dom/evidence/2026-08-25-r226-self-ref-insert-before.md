# R226 Evidence — `_zwMEl` insertBefore 的 ref 位先取后摘（30,4+28,0 成对收口）

**日期**: 2026-08-25
**切片**: M4——R226(a) 30,4 + 28,0 成对（R225 的 refNode-advance 单侧实现净 0 回退件的根因收口）
**改动面**: `part03.js`（`_zwMEl` insertBefore 的 ref index 先取后摘）+ `part23.rs`（回归单测 `r226_self_ref_insert_before_keeps_position`）

## 一、根因（探针复刻 30,4 形态实证）

R225 回退记录的「net 0（30,4 +2 / 28,0 -2）」其实是**同一根因的两种表现**：
`_zwMEl` 的 insertBefore 先 `removeChild(c)` 再 `indexOf(ref)`——当 `c === ref`
（自引用，WPT 30,4 的 `foreignDoc.body[0] === node`）时，detach 后 ref miss →
push 到**尾部**（探针实证 `body:[fp2, ftn, fp1]`，期望 `[fp1, fp2, ftn]`）。

spec `concept-node-pre-insert` 的 referenceNode index（「child 的 index」）在 adopt
摘除**之前**即固定——正确序：先取 ref 位，再摘除，再按固定位 splice。
自引用形态净效果 = 原位重插 = no-op。

R225 的单侧 advance 实现之所以净 0：advance 使 ref 变为 `kids[off+1]`，自引用
detach 后新 ref 仍可寻位（30,4 转绿）；但 28,0 的 sel 容器链上另一处 detach 使
p0/p1 互换（-2）。修根因后 advance 不再需要——`insertBefore(node, node)` 天然
no-op。

## 二、修法

`_zwMEl` insertBefore（part03 5990）：

```js
var _r226RefIdx = (ref == null) ? -2 : node.childNodes.indexOf(ref);  // 先取位
if (c && c.parentNode) { try { c.parentNode.removeChild(c); } catch … }  // 后摘除
… splice(_r226RefIdx, 0, c) …  // 按固定位插入
```

https://dom.spec.whatwg.org/#concept-node-pre-insert

## 三、验证链（vs R225）

| 项 | R225 | R226 | Δ |
|---|---|---|---|
| Range-insertNode | 1838P/2F | **1841P/0F（全量套件，单文件跑 1840P）** | **+2~3，文件 100%** |
| dom/nodes | 12660P | 12661P | +1 |
| dom/events / collections / traversal | 578/49/1602 | 同 | 0 |
| Range-surroundContents | 893P | 893P | 0 |

- **engine 单测**：**2379 全绿**（新增 `r226_self_ref_insert_before_keeps_position`
  ——30,4 形态 fp1 保持首位 + 28,0 形态 p0 不动双断言）。
- **fmt / clippy**：零警告。

## 四、R227 靶点

- **surround ~350F 重聚类**（893P 基线）——insertNode 整文件 100% 后 ranges 族
  最大剩余簇。
- 深项：customElements 多 registry / :scope query-root / lone-surrogate wire /
  MO-document parser 记录 / insertBefore 自旋。

## 五、commit

73610d6ea
