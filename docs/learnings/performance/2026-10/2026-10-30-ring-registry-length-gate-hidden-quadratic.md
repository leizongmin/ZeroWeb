---
date: 2026-10-30
modules: engine/js-dom-shim
---

# push-only 注册表 + length-only gate 后的 O(n) 预处理 = 填充序列 O(n²)

## 问题

cloneNode(true)/innerHTML= 填充 2000 节点耗 3-13s（editing-compat 簇 A）。四轮
裸回调拦截探针把 host 三层（解析/队列/wrap）全部免罪后，残差 100% 在 JS shim
闭包内部，静态阅读无法收口（trap 每调用 25-40 个 try/catch helper，成本分布式）。

## 根因

`__zwLiveRanges`（R183 Range 工厂，part06 环形缓冲 8192，**只进不出**）非空时，
`_zwAdjustRangesForInsert`（R263）的 gate 只查 `regs.length`，随后**无条件**执行
`parentNode` 读 + `childNodes` live-NL refresh（R140 每读 O(n) 全量 slice+concat）
+ O(n) 索引扫，之后才逐条目过滤——即注册表里哪怕只有 1 个与本次插入毫不相干的
死 range，每次 appendChild 也付 O(兄弟数)。填充 n 节点 = O(n²)。R262（remove 侧）
同构。探针环境 R=1 即足以触发；真实 editing 语料 Selection/Range 常驻数百条。
这与 t8c「childNodes 循环内重读 O(n²)」（同函数，2026-09 案）同型：**length-only
gate 挡不住「注册表非空但条目全不相关」的常态路径**。

## 解决

pass1 保守超集预筛门：O(R) 廉价谓词（identity/键域快道 + R352 dead 快道，
R262 侧补同根守卫覆盖子孙匹配面）做存在性短路，无命中直接 return；命中才进
**逐字保留**的原路径。SPAN/IH/CLONE 3.6-9.3×、RESET 10.5×（R263b/R262b，
part03.js）。语义等价靠谓词 ⊇ 候选集论证 + 冷路径不动，A/B 靠确定性案
（bold 1138P/1910F 零位移）+ 同类 verdict 零翻转。

## 方法沉淀：闭包内部归因用临时 shim tape 插桩

裸回调拦截只能测 call-time 全局查找；闭包内 helper（`_mo_notify`/CE/details）
不可拦截。有效做法：shim 源码加临时 `__zwTk(i)` 顺序 delta 检查点
（`globalThis.__zwTapeState` 聚合，结果进 test name），重建 runner 单案复跑，
**测完 git checkout 整体回退**（探针页与插桩都不留痕，避免污染全量基线）。
检查点带 `typeof` 守卫保证旧二进制零影响；逐段细分（先 13 点定位窗口，再 5 点
切窗口）比一次撒网定位成本低。row-count 型 A/B 须先区分看门狗截断方差
（同版二进制两次运行自对照）与真 verdict 位移，确定性案（预算内完整跑完的）
才可做零位移对账。
