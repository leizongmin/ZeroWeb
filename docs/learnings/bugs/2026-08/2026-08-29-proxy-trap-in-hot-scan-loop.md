---
date: 2026-08-29
modules: zero-engine
---

# js_dom_shim 热扫描循环内的 proxy trap 读是隐形单点瓶颈（R350）

## 问题描述

`Range-mutations-{dataChange,appendData,...}` 五个 WPT 用例文件级 Timeout（90s 只跑 28%）。单操作微基准全部「便宜」（写文本 0.004ms、range 操作 0.019ms），但整体二次方劣化，放宽超时无解。

## 根因分析

shim 的 live-range 注册表（`__zwLiveRanges`）随用例序列线性累积；R260/R262/R263 三个 adjust 函数对全部历史条目逐条比对。比对代码看似纯 JS，实际每条目的属性读走 **proxy get trap → host 回调**：

- `cont.__zwHandle` / `cont.__zwSelector`：part04 get trap 数百行分支，~0.5µs/读；
- `cont.parentNode`：host `__zw_parent` 回调（apply_pending_query_html + query doc），3 跳上行 13.8µs；
- `sameParent263` 在循环内**每条目重读恒定值**（newParent263 的键）。

三者相乘 = O(条目 × ops × host 往返 µs)，WPT 重建树循环实测 97ms/iter。

**定位陷阱**：逐段微基准（W14 探针）中 root-walk-only 0.39ms、+键读 0.60ms、全循环 0.60ms——单独每段都不慢！慢的是**真实代码里键比对命中后的 parentNode 读**与**每条目重复读恒定键**的组合。只有「等价轻量实现替换整个函数」的 A/B（W13：native 11.46ms vs stub 0.10ms）才暴露总量差；再逐段把 native 逻辑加回 stub（W18 变体 A）才锁定键读位置。

## 解决方案

三层修复（part03.js `_zwAdjustRangesForData/ForRemove/ForInsert`）：

1. **键域快道**：mutation 节点无 handle/sel 键时，跨域键比对必不命中 → identity-only，跳过全部键读；
2. **恒定键读提出循环外**（每函数一次而非每条目一次）；
3. **root walk 后置**：从「每条目前置守卫」改为「键命中后验证」——常态路径 O(1)，仅键字符串相等（罕见）才付 root walk。

效果：60 条目 × 30 append 从 11.93ms → 0.21ms（57x）；真实用例 90s 内完成 subtest 3.1x；全量 sweep Fail 集合恒等零回归。顺带修掉一个真 bug：跨树同 selector 字符串的旧死条目此前可被键匹配误命中并篡改 range offset。

## 如何避免

- shim 热路径循环内的「属性读」不是纯内存访问——凡对象可能是 `_makeProxy`/`_wrapHandle` 产物，字段读即 trap + 可能 host 往返。写扫描循环前先问：每条目读什么、读几次、读的对象属于哪个域。
- 循环不变量（尤其另一对象的键）必须提出循环外——在 trap 域这是 50x 级别差异，不是常规的微优化。
- 「前置守卫」不总是快：守卫本身若含昂贵读（parentNode 上行），改为「便宜键比对命中后才做守卫验证」。
- 探针方法论：单段微基准可能全部「不慢」（缓存/短路掩盖），需用**等价轻量实现整体替换**做 A/B 得到总量差，再逐段加回定位。
