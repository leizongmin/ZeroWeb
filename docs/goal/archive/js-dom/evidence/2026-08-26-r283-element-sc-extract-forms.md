# R283 Evidence — element-sc 一层 + sibling 提取形态（extract 178P→183P）

**日期**: 2026-08-26
**切片**: M4——R283(a) element-sc 递归组树簇
**改动面**: `part06.js`（extractContents R283 形态 A/B + sc-is-cac 尾段）+ `part23.rs`（+1 单测）
**commit**: `71caaddb5`

## 一、形态扩展（R280 限流器的精确放宽）

R280 首版把 element-sc 整体让位 R242/R236（防 +9 回归）——本轮识别出
**两种可精确表达的形态**后放宽（更深递归仍 defer）：

- **形态 A（48,x）**：ec 的父是 sc 的直接子（一层）——sc 的
  fully-contained 尾段子本体 move 入 frag；ec 路径子**留树**，其内容经
  既有 ④' lastPartial 路径克隆组树承载（P#c.clone + ec 头段文本 move
  进 clone + 源 deleteData）。sandbox 断言：
  `f48 tree=3[P#a,P#c("456"),cm] frag=2[P#b(full),P#c("C0123")] col=(DIV,1)`。
- **形态 B（53,x）**：ec 的父 === sc 的父（sibling 方向）——sc 尾段
  offset 越过路径子时本体全保（spec：P#a 末后无尾段）+ cac 中段
  （sIdx,eIdx）sibling move + ec comment 头段削。sandbox 断言：
  `f53 tree=4[P#a,P#c,P#d(full),cm("ongpad")] frag=2[P#e,cm-head] col=(DIV,3)`。
- **sc-is-cac 尾段**：`cac === sc` 的 element-sc（sc 自身是容器）补
  ⓪ 对称段（fully-contained 尾段子 move，ec 路径子留树）。

## 二、验证（A/B vs R282 基线，全 ranges sweep）

| 项 | R282 | R283 | Δ |
|---|---|---|---|
| Range-extractContents | 178P/9F | **183P/4F** | +5P（48,x 全解；53,x DOM+cursor） |
| Range-deleteContents / insertNode / surround | 125P / 1840P / 1840P 全 0F | 同 | 持平（100%） |
| Range-cloneContents | 180P/7F | 同 | 持平 |
| ranges 全量 | 37841P | **37846P** | +5，set-diff 0 新 fail |
| engine 单测 | 2416 | **2417** | +1（r283 f48/f53 断言） |
| fmt / clippy | 干净 | 干净 | — |

**残余 4F**：51,x 同节点 doc 提取（sandbox 复现 frag=0/docKids=2——
`_coveredChildren` 同容器 doc 路径的 move 域问题）+ 53,x fragment
（moveIn 对 P#e 的 append 在部分域被 flat 成裸 text——harness 期望
`<p id="e">` 包裹）。

## 三、教训

- **「整体让位」的限流器要随专门分支的能力演进逐步精确放宽**：形态
  A/B 各有一个可判定的拓扑条件（ec 父位置），比「全部 defer」或
  「全部接管」都安全——每放宽一个形态跑一次全量 set-diff。
- **测试基线字符串与 fixture 数据耦合**：改 fixture（comment 加长）
  会连带三个既有测试的期望串——批量更新后须全跑（本轮 4 红 3 个是
  这种假红）。

## 四、R284 靶点

- **(a) 51,x 同节点 doc 提取**（`_coveredChildren` 同容器 doc 的 move
  域修复）+ **53,x fragment 域**（moveIn append flat）——两个已知域簇。
- **(b) clone 残余 7F**：54/55,x collapsed foreign/xml + 53,x clone 对应
  形态 + Range.detach()。
- **(c) deleteContents ShadowRoot 一例**。
