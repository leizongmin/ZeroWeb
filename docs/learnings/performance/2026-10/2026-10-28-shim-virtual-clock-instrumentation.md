---
date: 2026-10-28
modules: engine,tests/wpt-runner
---

# shim 内 performance.now() 是虚拟时钟——耗时插桩必须 Date.now()

## 问题

encoding-legacy-mb 语料页单 variant 墙钟 12.87s，但 shim 内用 `performance.now()`
插桩各分段只显示 16ms——「时间消失」，归因三次落空（先后错疑 walk、循环、
索引重建）。

## 根因

runner 的 js_dom_shim 把 `performance.now()` 实现为虚拟时钟（推进量与真实
墙钟解耦，V8 看门狗/计时器语义按虚拟时间走）。任何「页面内」插桩拿到的都是
虚拟时间，与外部观察到的墙钟（`date`/`Date.now()`/进程计时）不可比。

## 解决方案

- 分段耗时插桩一律 `Date.now()`（真实墙钟）；shim 内 `performance.now()`
  只用于理解虚拟时序。
- 长尾案分段打点用 `__zw_dbg_log` 诊断桥（`ZW_DBG_LOG=1` 输出 stderr），
  不污染 tracing。
- runner 案级结果是 all-or-nothing：整案超时/崩溃时已产出的分段打点随
  WebView 销毁丢失——桥打点（stderr 直落）是唯一幸存观测窗。
- 验证：同案用 Date.now() 重测即定位死点（doc 级 wrap 环每结果全文档序列化
  999×12.8ms≈12.8s，与墙钟精确吻合）。
