# R258 Evidence — selectNode 落位 sc 回退（28,0 + 30,x 残余簇全解 +3P）

**日期**: 2026-08-25
**切片**: M4——R258(b) 残余簇重聚类 + 修复
**改动面**: `part06.js`（R212/R236/R242 三 selectNode 尾部补 sc 回退）+
`part23.rs`（探针转 +1 回归单测）
**commit**: 94c1b59c3

## 一、重聚类（R256/R257 行为面变化后取样）

残余 14F 重跑：16,x 11F 不变（startOffset 异步时序深项）+ **28,0/30,4/30,11
三连共享新签名**——`endOffset expected 1 but got 0` 且 **DOM 断言全过**
（树正确，仅边界缺写）。

## 二、根因（trace 探针实证）

30,4 形态（`[foreignDoc.body,0,foreignTextNode,36]` + foreignPara1）：
**newParent 自身是 covered 子**。步骤链：
1. `extractContents()` 把 covered 子移进 frag2——**newParent 本体也在内**
   （preIns 探针：insertNode 前 `p1.parentNode = #document-fragment`）；
2. `insertNode(newParent)` 经工厂 body 的 `_tree.appendChild` 插回——其
   事后父链修复 `if (c.parentNode === _tree) c.parentNode = body` 对
   **frag 旧链 miss**（parentNode 仍指 fragment），R219 的 setEnd 读
   `node.parentNode`（=frag）的 indexOf 也 miss → 边界零写入；
3. R236 尾部 `newParent.parentNode` = frag → `indexOf` = -1 →
   selectNode 落位整体跳过——trace 全程仅 extract 的 collapse 一对
   （`sS:BODY:0, sE:BODY:0`）。

树视图（fb.childNodes）正确含 newParent（DOM 断言过）——**数组视图与
parentNode 链分裂**，落位读错了源。

## 三、修复

R212/R236/R242 三尾部统一：`newParent.parentNode` 查找 miss（indexOf<0）
时**回退 `this.startContainer.childNodes`**（树视图权威）取父与索引。
既有命中路径零变化（回退仅在 miss 时触发）。

## 四、验证（vs R257 基线）

| 项 | R257 | R258 | Δ |
|---|---|---|---|
| Range-surroundContents | 1826P/14F | 1829P/11F | **+3**（28,0/30,4/30,11） |
| ranges 上游 set-diff | — | — | **+3 F2P / 0 P2F** |
| engine 单测 | 2397 | 2398 | +1 回归单测（30,x 形态 so=0/eo=1） |
| fmt / clippy | — | 干净 | — |

## 五、R259 靶点

- **16,x startOffset 11F**（最后一簇）：`[document.body,4,document.body,5]`
  ——R255 已定位 iframe contentDocument 异步 fetch-rebuild 时序，深项
  （harness onload 链 vs 同步 rebuild 的语义同步化评估）
- 深项：customElements 多 registry / :scope query-root
