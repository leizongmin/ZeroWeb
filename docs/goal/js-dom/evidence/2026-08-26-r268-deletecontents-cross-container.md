# R268 Evidence — deleteContents 跨容器 CharData 分支（+4P，三轮累计 +22P）

**日期**: 2026-08-26
**切片**: M4——R268(a) deleteContents 跨容器族首切片（安全收敛形态）
**改动面**: part06 deleteContents 新增 cross-container 分支（双侧 CharData 端点
+ 子树不相交形态）+ part23.rs（+1 回归单测）

## 一、形态与实现

**20/21,x 族**：sc=textA（paras[0] 内）、ec=textB（paras[1] 内）——跨段
CharData 区间（cac=testDiv，sc/ec 子树不相交）。旧版落 `_coveredChildren`
（读 sc 的融合视图）空转——textA 尾部残留。

**实现**（对齐 common.js myDeleteContents 的树序算法四段）：
1. sc 尾段 `deleteData(so, len−so)`；
2. sc 侧爬升：sc 路径各「中间级」的右侧兄弟移除（**cac 级跳过**——右侧全删
   会误删 ec 路径子）；
3. ec 侧爬升：ec 路径各中间级的左侧兄弟移除（cac 级同样跳过）；
4. cac 级中段 `(sIdx, eIdx)` 开区间统一移除 + ec 头段 `deleteData(0, eo)` +
   塌缩 (cac, sc 路径子 idx+1)。

remove() + parentNode 校验兜底 removeChild 直调（R267 教训复用）。

## 二、首版教训（范围收敛）

首版泛化到 **element 端点**：24,x `[testDiv,2,paras[4],1]` 大幅过度删除
（DIV 7→2）——sc 元素尾部规则把 partially-contained 的 ec 子树整体删掉
（ec 是元素时本体保留、仅内部 [0,eo) 删）；ancestor 方向同理错。**收敛到
已验证正确的形态**：双侧 CharData 端点 + 子树不相交（20/21,x）；element
端点/ancestor 方向走既有回落。方向分支的 contained 递归算法记 R269 靶点。

## 三、验证（vs R267 基线）

| 项 | R267 | R268 | Δ |
|---|---|---|---|
| Range-deleteContents | 98P/31F | **102P/27F** | **+4**（20/21,x 四 subtest） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| engine 单测 | 2406 | **2407** | +1（r268 单测）全绿 |
| fmt / clippy | 干净 | 干净 | — |

deleteContents 三轮累计：80P/49F → **102P/27F（+22）**。

## 四、R269 靶点（残余 27F）

- **element 端点跨容器**（22/24/48/52/53,x）：需方向分支的 contained 递归
  （sc/ec 是元素时的 partially-contained 语义——本体保留仅内部区间删）。
- **document/documentElement 容器族**（17/25/26/49/50/51,x）：doc 级 child
  数分歧（doctype/html 摘除）。
- **6,x** CDATA 区间族（`[paras[5].firstChild,2,paras[5].lastChild,4]`）。
- **28/29,x**（`[testDiv,0,comment,5]`——sc 元素 + ec textel 深形态）。
- extractContents 32F / cloneContents 29F 独立聚类。
