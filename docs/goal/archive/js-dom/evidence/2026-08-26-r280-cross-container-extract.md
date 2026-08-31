# R280 Evidence — 跨容器 extractContents 分支（extract 156P→168P +12）

**日期**: 2026-08-26
**切片**: M4——R280(a) extractContents 重聚类 + 跨容器分支
**改动面**: `part06.js`（extractContents R280 分支：路径克隆组树 + move 兜底 + doc-sc 尾段）+ `part23.rs`（+1 单测）
**commit**: `2a3c355a5`

## 一、重聚类（R278/R279 后 expected 侧真实化的验证）

R279 收口 deleteContents 后重跑 extractContents：**31F 全部仍是真失败**
（无 false-pass 翻真——extract 的 oracle 在 R278 前就依赖的
nextNode 遍历同样被救活，但 extract 引擎侧缺口独立存在）。聚类：

| 簇 | 形态 | 归因 |
|---|---|---|
| 20/21,x | `[textA,0,textB,0/8]` CD→CD 跨 P 段 | 引擎空转（R211 要求同父） |
| 22,x | `[textA,3,P#d,1]` sc CD + ec 元素 | 空转（R242 要求 sc 是元素祖先） |
| 48,x | `[testDiv,1,textC,5]` sc 元素深后代 | R242 只接直接子 |
| 52/53,x | `[textC,4,comment,2]` / `[P#d,1,comment,8]` | 空转 |
| 25/26/51,x | `[document,0,document,1/2]` | assert_throws（doctype 规则） |
| 29/31,x | `[foreignDoc,1,fComment,2]` | comment 克隆域 [object Object] |

## 二、实现（R280 分支，R211 块后）

frag 结构对齐 spec `[firstPartial.clone(sc侧子树), contained…,
lastPartial.clone(ec侧子树)]`，**路径克隆组树**双侧（每层路径 clone
一层、内容挂层内）；数据面 sc 尾段 deleteData + ec 头段 deleteData；
中段 contained 本体 move（R241 兜底硬化：先记原父/原列表，append 后
残留则 removeChild——探针实证 proxy fragment 的 append 对 plain 子
**不摘原件**，pd 同时在两棵树）；塌缩 R279 同款（同树位/cac===sc 分叉）。

**形态限流**（首版教训）：只接 sc CharData/Document——首版泛化抢了
R242/R236 已正确的直接子形态使 24/28/30,x +9 回归（element sc 的
firstPartial.clone+subfrag **递归组树**非扁平可表达，48/53,x 留后续）。

**doc-sc 尾段**（nodeType 9 的 [so, ecPathIdx) 子 move）已接——29/31,x
另有 comment 克隆域问题（`[object Object]`）未随本切片解。

## 三、验证（A/B vs R279 基线，全 ranges sweep）

| 项 | R279 | R280 | Δ |
|---|---|---|---|
| Range-extractContents | 156P/31F | **168P/19F** | +12P（20/21/22/52 全解或部分） |
| Range-deleteContents | 125P/0F | 125P/0F | 持平（100% 保持） |
| Range-insertNode / surround | 1840P / 1840P | 同 | 持平（100%） |
| Range-cloneContents | 158P/29F | 同 | 持平（下轮独立切片） |
| ranges 全量 | 37797P | **37809P** | +12，set-diff 0 新 fail / 12 消失 |
| engine 单测 | 2413 | **2414** | +1（r280 52,x 形态单测） |
| fmt / clippy | 干净 | 干净 | — |

## 四、教训

- **frag 结构是「路径克隆组树」不是扁平列表**：两轮首版错误——①flat
  尾段裸 #text（20,x 期望 `<p id="a">` 包裹）；②sc 端点自身多 clone
  一层空壳（P#c=[#text(""),#text("3")]——端点层的内容就是切片本身）。
- **proxy fragment 的 append 是登记不是 move**：plain 子挂进 frag 后
  原树不摘（双份）——moveIn 必须带「append 后残留则 removeChild」兜底
  （R241 家族第三例：surround/insert/extract 全命中同一坑）。
- **泛化分支要让位专门分支**：直接子形态已有 R242/R236 正确处理时，
  泛化跨容器分支须显式排除（形态限流），否则抢跑引入回归。

## 五、R281 靶点

- **(a) cloneContents 29F 重聚类**（与 extract 同构的 frag 组树——
  R280 的路径克隆模式可直接移植，无需 move/删源）。
- **(b) extract 残余 19F**：25/26/51,x doc-doctype throws 域 +
  29/31,x comment 克隆域 + 48/53,x element-sc 递归组树。
- **(c) deleteContents ShadowRoot 一例**。
