# WPT testharness 需要显式 timer owner

- 日期：2026-08-13
- 相关模块：`tests/wpt-runner`、`zero-webview` DOM shim

## 问题描述

在 in-process WebView 中注入上游 `testharness.js` 后，所有用例都在页面测试脚本执行前结束，runner 最终只能观察到 completion timeout。

## 根因分析

`testharness.js` 初始化时通过 `setTimeout` 安排 10 秒 watchdog。WebView 没有注入宿主 timer callback 时，DOM shim 会把 `setTimeout` 回退为当前 checkpoint 的微任务，因此 10 秒 watchdog 被立即执行，harness 在尚未注册任何 subtest 时进入 COMPLETE。

此外，in-process `run_page_scripts` 不具备完整导航的 load lifecycle，不能只依赖 window `load` 关闭 harness 注册期。

## 解决方案

HTML testharness runner 在注入上游 harness 前提供 no-op timer host，由 Rust `Instant` 墙钟负责 case timeout；同时在注入副本内部增加私有 loaded hook，复用 harness 自身的 `all_done()`/`complete()` 状态机结束注册期。

不要把 JS timeout fallback 当作真实墙钟，也不要用“无 completion 即通过”掩盖问题。
