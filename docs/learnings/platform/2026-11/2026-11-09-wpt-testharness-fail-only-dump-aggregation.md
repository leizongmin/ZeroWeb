---
date: 2026-11-09
modules: wpt-runner, engine
---

# wpt testharness 断言消息只在 Fail 输出——状态转储用聚合单断言

## 问题描述

用 testharness.js 诊断引擎内部行为时，断言里的状态转储字符串（中间 html、carry 状态、埋点序列）在测试 Pass 时完全不可见，只有 Fail 才把 assert 消息打进日志。逐断言转储的多步骤链因此拿不到中间现场；而把断言改成恒假以输出转储后，断言失败会中止 test 函数，后续转储点全部丢失，try/catch 包裹又会把失败吞掉反而变绿。

## 根因分析

testharness 的输出协议是 Fail 行附带 assert 消息；assert_true(false) 抛出后 test 函数即终止，同一 test 内后续代码不执行。单点转储与多步骤链两者矛盾。

## 解决方案

链上每步把状态 `DUMPS.push(...)` 聚合到数组，test 函数结尾一次 `assert_true(false, DUMPS.join(" || "))`——失败恰好中止在末尾，所有转储已入列且只产生一行聚合 Fail。引擎侧配合 `globalThis.__zwXxxTrace` 埋点由页面读取。实践样例 `tests/wpt-runner/wpt-data/editing/run/zz-probe-f6b-diag.html`（诊断 fixture，不入库）。教训：在 testharness 里"打印"只有一条通道——制造一次受控失败；多步转储先聚合再一次性输出。
