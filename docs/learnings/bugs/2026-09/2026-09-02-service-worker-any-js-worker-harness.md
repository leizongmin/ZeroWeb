---
date: 2026-09-02
modules: tests/wpt-runner, crates/script-sandbox
---

# Service Worker .any.js Harness Can Hide Worker Subtests

## 问题描述

推进 `service-workers/service-worker/no-dynamic-import-in-module.any.js` 时发现，
Service Worker `.any.js` 用例如果没有按 worker global 包装，只可能报告页面侧
`fetch_tests_from_worker()` 聚合断言通过，实际 worker 内的 `promise_test()` 并未执行。
`no-dynamic-import.any.js` 曾因此只显示 1 个 subtest；修正包装后真实结果应包含 3 个
worker 侧断言加 1 个聚合断言。

## 根因分析

runner 原先只把 `.https.any.js` 识别为 Service Worker worker-global 用例，普通
`.any.js` 被当成页面脚本执行。另一个隐患是 classic worker 通过 `importScripts()` 载入的
子脚本仍会触发 JS 引擎原生 dynamic import 回调，返回通用 `Error: Not supported`，不符合
Service Worker 规范要求的 rejected `TypeError` promise。

## 解决方案

- Service Worker runner 对所有 `.any.js` 都使用 worker wrapper。
- 读取 `// META: global=serviceworker-module` 后用 `{ type: 'module' }` 注册 worker，并在
  module worker 中用静态 `import '/resources/testharness.js'` 注入 harness。
- Service Worker runtime 在 classic 顶层脚本和 `importScripts()` 子脚本 eval 前改写
  `import(` 为统一的 `__zw_dynamic_import(`，该 hook 返回 rejected `TypeError` promise。
- 推广类似 `.any.js` 用例时，过滤验证必须核对实际 worker subtest 数，不能只接受单个
  wrapper 聚合断言通过。
