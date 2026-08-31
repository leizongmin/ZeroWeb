# R267 Evidence — deleteContents ancestor 分支（+6P，累计 R266+R267 = +18P）

**日期**: 2026-08-26
**切片**: M4——R267(a) deleteContents ancestor-element 族
**改动面**: part06 deleteContents 新增 ancestor 分支 + part23.rs（+1 回归单测）

## 一、形态与实现

**23,x 族**：sc 是 ec 的**父元素**、ec 是 sc 的**直接 CharData 子**
（`[paras[0], 0, paras[0].firstChild, 7]`）。旧版落 `_coveredChildren` 融合
视图空转——text 头部残留（期望削 [0,eo) 保 remainder）。

**R236 extract ancestor 分支的 delete 侧对称缺口**，实现三段（spec
`dom-range-delete-contents` 的 first-partially + contained children +
collapse）：
1. `ec.deleteData(0, eo)`——削 ec 头部；
2. contained 中段子移除（sc 直接子的 [so, ecIdx) 区间，逆序保索引）——
   **remove() 优先 + 结果校验兜底 removeChild 直调**（泛型
   Node.prototype.remove 对部分域形态静默失败——探针实证 post-parent 不变）；
3. 塌缩 (sc, so)。

handled 标志短路后续 `_coveredChildren` 路径（防二次处理）。

## 二、过程教训

- 首版对 contained 子只调 `c267.remove()`——单测场景二（p2 含 span + text）
  立即抓到 kids 不变（探针 `pre=p2/fn=function/post=p2`）：remove 是函数但
  对该域形态静默无效。修复 = remove 后校验 parentNode，非 null 则 removeChild
  直调兜底（part04 域分发更可靠）。
- 测试 JS 的多行字符串字面量在 V8 CompileError（raw string 内真实换行）——
  用数组 join 形态重写。

## 三、验证（vs R266 基线）

| 项 | R266 | R267 | Δ |
|---|---|---|---|
| Range-deleteContents | 92P/37F | **98P/31F** | **+6**（23,x 族 + 中段移除形态） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| engine 单测 | 2405 | **2406** | +1（r267 单测）全绿 |
| fmt / clippy | 干净 | 干净 | — |

两轮累计：deleteContents 80P/49F → **98P/31F（+18）**。

## 四、R268 靶点（残余 31F 聚类）

- **跨容器 CharData 族**（20/21/22/24/52/53,x）：sc/ec 不同容器（如
  `[paras[0].firstChild,0,paras[1].firstChild,8]`、`[testDiv,1,
  paras[2].firstChild,5]` 深祖先、`[paras[3],1,comment,8]`）——需要
  cac 级 contained-children 泛化（myDeleteContents 的 nodesToRemove
  树序遍历算法）。
- **document/documentElement 容器族**（17/25/26/49/50/51,x）：doc 级
  child 数分歧（doctype/html 摘除）。
- extractContents 32F / cloneContents 29F 独立聚类。
