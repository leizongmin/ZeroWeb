---
date: 2026-11-14
modules: engine
---

# 无宿主定时器环境下轮询自链饿死 _deferBudget：img load 红表象离 SMIL 泵根因极远

## 问题描述

桥套件（cargo test -p zero-engine --lib js_dom_bridge）两例 img/svg 红经久
不愈：`img_runtime_deferred_settle_r5275`（load 后续任务派发 0≠1）、
`svg_img_settle_density_r5277`（load 计数 0≠1）。表象在 img load 派发，
与 SMIL 无任何可见交集；同套件其余异步面（ws/xhr/fetch）正常，solo 间歇
复绿的旧记录加剧误导。

## 根因分析

四步因果链，症状与根因隔了整个 shim：

1. SMIL 事件泵 `_scheduleTick`（part09b）以 setTimeout ≤50ms 轮询自
   re-arm；
2. 裸 sandbox/桥测试环境无宿主定时器 → shim setTimeout 回落 = `_defer`
   = queueMicrotask → **轮询自链不可终止**（guidelines 不变式 #30 禁
   microtask 链的退化形态）；
3. `_deferBudget`（单次脚本执行 microtask 派发上限 256）在首个 checkpoint
   被泵链烧尽；桥测试环境 `__zw_begin_script`（预算重置点）只由生产
   webview.rs 路径调用、裸 execute 从不重置 → 预算归零后**同环境一切后续
   `_defer` 回调永久丢弃**——img/svg load 派发（与泵无关的面）全灭；
4. 生产 webview/runner 有宿主定时器（`__zw_setTimeout` webview.rs /
   `__zw_test_setTimeout` testharness stub）→ task 粒度真实任务，不受此
   病——所以 upstream 语料（有宿主定时器）定谳不了这类红，只有桥套件
   （裸 sandbox）暴露。

诊断方法论：红表象离根因极远时，先判据二值化全树 git bisect 钉引入提交
（bad iff 判据测试 FAILED，每轮 good = 上轮 first bad 锚），再对引入 diff
勘根——本例 bisect 一轮即钉到 SMIL 时间线 M1（015172475），把「img 面」
假象直接推翻。

## 解决方案

`_scheduleTick` 单点守卫：双探针
（`globalThis.__zw_test_setTimeout`/`globalThis.__zw_setTimeout`，与
part01 `_zwHostSetTimeout()` 同判据）皆缺即不排轮询——无任务调度的环境
下轮询语义本就不成立；seek 一次性扫描（`_scheduleSeekScan`）单发自终结、
不烧预算，不守卫。生产/runner 双探针命中，行为零变化。

推广判据：任何 setTimeout 轮询/自链结构进 shim 前，先核「无宿主定时器
回落面」——回落是 microtask 时，自链必然烧 `_deferBudget` 并殃及同环境
后续一切 deferred 派发；守卫点选在调度入口（探针判据），不选在回调内。
同构饿死面在生产仍存在（泵 tick 链单 checkpoint 内 ×256 自链挤占同批
deferred 回调），行为零变化确认后留观。
