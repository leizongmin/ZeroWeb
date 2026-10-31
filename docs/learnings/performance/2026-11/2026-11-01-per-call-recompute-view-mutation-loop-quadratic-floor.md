---
date: 2026-11-01
modules: engine/js-dom-shim
---

# per-call 重算视图 × mutation 循环 = 单调用线性化后的 O(n²) 地板

## 问题

sel 域 removeChild×2000 在两层修复后仍 3.8s（editing-compat 簇 A）：第一层
（R263b/R262b 预筛门，见 [ring-registry learning](../2026-10/2026-10-30-ring-registry-length-gate-hidden-quadratic.md)）
杀掉 per-entry 隐藏二次项；第二层（R5340-glue `_r126FuseMemo`）把每 removal 3 次物化
降到 2 次。剩余墙钟仍 ~96% 集中在某处，粗粒度归因（调用计数、物化次数）已失效。

## 根因

`_childNodeList` 融合视图是 **per-call 全量重算**：每次读 = 基底快照 + pending 桶
合并，输出全新数组。mutation 循环里每 mutation 读一次视图 ⟹ 总量 O(n) 次 × 每次
O(n) 重算 = O(n²)，**与单次调用内部已经多优化无关**——这是架构地板，不是某个循环
写坏了。tape v3 内部分段（在 overlay 函数内部再插桩拆 added 合并段/removed 扫描段）
钉死剩余墙钟：added 逐条 `seen` 检查与 `nextSibling` 定位各 O(res) 线性扫，bulk 形态
下 added≈n−k × res≈n−k 单次调用即 O(n²)——即「单调用内部」还有一层自己的二次项，
两层嵌套。同时证伪 host 往返假设：基底缓存恒命中（innerHTML= setter 置空基底后
overlay 供全视图），JSON.parse 零次——粗分段看到的「2 次物化」成本全在 JS 内部循环，
不在 host 串回环。

## 解决

两步修复阶梯，风险等级完全不同：

1. **单调用内算法等价改造（安全）**：进循环前建一次 res identity Set——`seen` 判定
   O(1)；`nextSibling` 先 O(1) 成员门、miss 直接落尾插（与线性扫空等价）；removed
   剔除改 Set 单遍过滤（等价性 = 数组条目 identity 唯一 + Set 与数组由记账侧恒成对
   维护；Set 缺失回落旧循环兜底）。w2 3,832→1,506ms。**单调用内等价论证的三要素**：
   条目 identity 唯一（去重侧保证）、成员/序不变式（插入即同步 Set）、防御回落路径。
2. **跨调用缓存/减少物化次数（高危，独立整轮）**：把 2 次/removal 降到 1 次或缓存
   视图，必须先审 wrapper **identity 稳定副作用**（页面持有先前返回的数组，重算时
   mutation 时点同步维持其语义；惰性化/缓存后失效覆盖不全即静默失真）与失效面完备性。
   已证不等价的具体形态：「上次视图 − 1」推导（惰性 ghost 使 ghost 条目可回 rookies）。

## 方法沉淀：负控制 A/B 防「未兑现机制」注记

微修必须单独 A/B 定机制，不能搭车：尾插 push（splice 尾位特形）A/B 零增量——保留
（等价零险的形状改进），但收益归因为循环体常数而非 memmove；trap 读局部复用 A/B 零
增量——**整体回退**，因为注释已写「走 get-trap」这个未兑现机制，留下即是假注记。
判据：修复注释里写的机制必须与实测分离出的收益源一致，否则改注释或改代码。
