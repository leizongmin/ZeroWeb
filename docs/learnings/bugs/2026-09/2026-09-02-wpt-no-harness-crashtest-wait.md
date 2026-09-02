---
date: 2026-09-02
modules: wpt-runner,script-sandbox,webview
---

# WPT no-harness crashtest wait completion

## 问题描述

导入 `service-workers/cache-storage/crashtests/cache-response-clone.https.html` 时，页面没有
`/resources/testharness.js`，而是使用 `<html class="test-wait">` + module script 顶层
`await`，最后移除 `test-wait` 表示 crashtest 完成。

旧 runner 对无 testharness 页面有两个风险：

- 把注入的 mini harness 直接拼在原 HTML 前面，可能让原 `<script type="module">` 被解析为
  普通脚本文本的一部分，导致顶层 `await` 以 classic script 编译并报语法错。
- 对无 testharness 引用且 0 个注册测试的页面脚本异常按 crash 语义判 pass，掩盖真实编译/运行错误。

## 根因分析

WPT crashtest 的完成协议不是 testharness completion callback，而是页面自身清除
`test-wait`。runner 若只靠内部 mini harness 的 `phase=4`，会在异步 module body 完成前提前结束；
若脚本异常也被当作“没有崩溃”通过，则会产生假绿。

## 解决方案

- no-harness 注入应插入到 `<head>` 或 `<html>` 开始标签之后，保留原页面脚本标签及
  `type="module"` 属性。
- no-harness 页面脚本抛错必须返回 Fail。
- 对无 testharness 且无 subtest 的页面，只有当 `document.documentElement.classList` 不再包含
  `test-wait` 时才判 pass；否则继续轮询直到超时。
- 模块脚本转换遇到顶层 `await` 时，模块体 IIFE 本身必须是 `async function`，外层包装不足以让
  内部同步 IIFE 中的 `await` 合法。
