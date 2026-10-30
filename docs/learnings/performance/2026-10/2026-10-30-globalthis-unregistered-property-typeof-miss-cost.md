---
date: 2026-10-30
modules: engine
---

# 未注册 globalThis 属性的 typeof 判定走慢路径——惰性注册让页面常驻付 17.6ms/次

## 问题

editing 语料 remove 侧 `AdjustRangesForRemove` 每次调用测得 ~17.6ms，
但函数体内所有分段计时器都是 µs 级——时间消失在段间 gap。2000 次
remove 放大成 ~35s，是 innerHTML 重设（S8 5.8s）与 move 循环
（4.5s/250 次）的主项。

## 根因

函数尾有 selection 联动钩子：

```js
if (typeof globalThis.__zwSelectionFollowRemoval === 'function') {
  globalThis.__zwSelectionFollowRemoval(...);
}
```

回调在 `_getSelection()` 内部惰性注册——**从不调 getSelection 的页面
永远不注册**。`typeof` 对已注册属性命中是 ns 级（`__zwLiveRanges`
对照实测）；对未注册属性走 globalThis 解析 miss 慢路径（命名属性/
宿主扫描遍历累计页面状态），实测 **17.60ms/次，与页面累计状态规模
正相关**。探针直接插桩钉死：`typeof` 一行单独耗时 `"undefined|17.60"`。

这类「存在性探测 + 惰性注册」组合在热路径上是隐形地雷：写代码时
属性注册后走快路径，测试通过；真实页面不触发注册分支，每次探测
都付全量 miss。

## 解决方案

注册提升到模块顶层（IIFE 内文件级），幂等门
`if (!globalThis.__zwProperty)` 保留；回调函数体首行做无依赖
no-op 门（selection 为空直接 return），提升后提前注册语义安全。
效果：17.3ms→0.009ms/call，move×250 4553→112ms（-97%）。

规则：热路径上的 `typeof globalThis.X`/`'X' in globalThis` 存在性
探测，X 必须顶层急注册（或改用 IIFE 局部变量承载，彻底绕开
globalThis 解析）。

## 排查方法论（测量悖论定位）

装饰器报 17-18ms/调用而所有段计时器全 µs——三步收口：

1. 完成标记 + throw 计数器：排除异常控制流吃掉时间。
2. 阶梯 stub：L1（只读 parentNode）/L2（加子扫描）逐层替换真函数，
   隔离「哪个读取项」昂贵——本轮证明读取全免罪。
3. 入口/出口单对计时进 closure 持有累计对象（**不走 globalThis**，
   探针自身不能污染被测路径），确认真实单次耗时；段间 gap 子计时
   g1-g4 定位到具体语句后直接插桩。

注意分层：shim 内部插桩的 `performance.now()` 是虚拟时钟，须
`Date.now()`（见 [2026-10-28 shim 虚拟时钟](../2026-10/2026-10-28-shim-virtual-clock-instrumentation.md)）；
页面级 bench 同钟 A/B 比值不受影响，但墙钟换算须用外部计时。
