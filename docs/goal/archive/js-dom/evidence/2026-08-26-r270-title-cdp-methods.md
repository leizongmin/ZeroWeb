# R270 Evidence — foreignDoc title 域 cDP/contains 补齐（+5P，五轮累计 +33P）

**日期**: 2026-08-26
**切片**: M4——R270(a) oracle 遍历原语族（17,x 直修 + 连带）
**改动面**: part03 `_makeDetachedDocument` 的 title 元素/文本子补
compareDocumentPosition + contains + part23.rs（+1 回归单测）

## 一、定位链

1. 17,x 失败消息「nodeB.compareDocumentPosition is not a function」——
   common.js getPosition 的 oracle 调用链。
2. **全树走查探针**（walk foreignDoc 每节点检查方法面）：missing =
   `TITLE(1), #text(3)`——**R130 的 title 元素 + title 文本子**缺方法（其余
   foreignDoc 节点 docEl/headEl/body/paras 都有 R243 的补齐）。
3. common.js getPosition(nodeA, nodeB) 对遍历到的任意 nodeB 调
   cDP——oracle（myDeleteContents 的 isContained/树遍历）碰到 title 子树
   即抛异常 → expected 侧回退未触碰树 → 断言必败。

## 二、修复

title 元素与 title 文本子各补 contains/cDP（委托共享
`_zwNodeContains`/`_zwCompareDocumentPosition`，与 docEl/headEl 的 R243
同款）。

**连带解锁**：oracle 树遍历不再回退——24,x（`[testDiv,2,paras[4],1]` 的
expected 侧此前回退到未触碰 7 子树）等 2 个 range 族连带翻绿。

## 三、验证（vs R269 基线）

| 项 | R269 | R270 | Δ |
|---|---|---|---|
| Range-deleteContents | 108P/21F | **113P/16F** | **+5**（17,x + 24,x 族连带） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| Range-mutations-removeChild | 20P/0F | 20P/0F | 持平（100%） |
| engine 单测 | 2408 | **2409** | +1（r270 单测：全树方法面 + cDP 活调用）全绿 |
| fmt / clippy | 干净 | 干净 | — |

deleteContents 五轮累计：80P/49F → **113P/16F（+33）**。

## 四、R271 靶点（残余 16F）

- **element 端点跨容器**（22/48/52/53,x）：方向分支 contained 递归
  （sc/ec 是元素时 partially-contained 语义）。
- **6,x CDATA 区间族**：`[paras[5].firstChild,2,paras[5].lastChild,4]`。
- **28,x**（`[testDiv,0,comment,5]` 深形态）/ **49/50,x**（cursor-only 差异）。
- extractContents 32F / cloneContents 29F 独立聚类。
