---
date: 2026-10-26
modules: script-sandbox,webview
---

# 模块记忆化键误用页面 URL——同文档第二枚 module script 零次求值

## 问题描述

同文档存在**两枚及以上** `<script type="module">` 时，第二枚起模块体静默零次求值：
`test()` 注册不发生，testharness 以 `completed without reporting registered tests
(state={tests:0})` 空转，runner 把整页记 Timeout。该缺陷是潜伏态——语料此前没有
多内联 module 页；runner 投递保真把 `type="module"` 外链标签改写为带 type 的内联标签
后，currentScript-null.html 首次踩中（html-semantics-compat s94，2026-10-26）。

## 根因分析

R5182 引入的 `__moduleCache`（globalThis 文档级，跨 module script 元素共享，模拟
HTML module map「恰好一次求值」）以 `compile_module_script` 的 `url` 参数为记忆化键。
但 `url` 同时承担相对 import 解析基准与 `import.meta.url` 发射，webview 宿主脚本循环
把**页面 URL** 一并传给它——同文档所有 module script（内联/外链皆然）共享同一缓存键，
第二枚被第一枚的记忆化命中短路（`__moduleCache[key] === undefined ? (body) : cache[key]`
取 else 臂），模块体零次执行。

排障链：runner 诊断行 `tests:0` 指向测试注册缺失 → 最小探针二分（单内联 module 绿 /
双内联 module 红，src-module 后跟 classic 脚本绿）锁定「任意双 module 页」→
`compile_module_script` 源码读到 `memoized_dep_expr(url, &body)` 定谳。

## 解决方案

键参数分离：`compile_module_script(source, url, cache_key, registry)`——`url` 保留
解析基准/import.meta 语义，`cache_key` 只做记忆化键。webview 侧按 spec 语义取键：
内联 module = 页面 URL 挂 ordinal 片段（`{doc_url}#!zw-inline-module-{n}`，仍是绝对
URL，相对 import 解析不受影响——URL 构造剥旧 fragment）；外链 module = src 绝对化
（同 src 两元素命中同键恰好一次，spec module map 语义）。spec 依据：module map 只按
请求 URL 缓存 fetch 型模块，内联 module script 每元素是独立模块记录
（html.spec.whatwg.org/multipage/webappapis.html#module-map）。

## 避免方式

- 「一个参数身兼语义」的编译函数加新用途（缓存/日志/遥测）时先拆参数，不搭便车。
- 恰好一次（evaluate-once）记忆化的键要覆盖**每个发射路径**单测：多内联入口、共享依赖、
  同 src 双元素三形态（本次补 `test_s94_inline_module_cache_key_separate_from_url`）。
- 调试 testharness 页 Timeout 先看 runner 的 harness 状态诊断行：`tests:0` = 测试注册
  缺失（脚本没跑/跑错时序），区别于断言失败与页面崩溃。
