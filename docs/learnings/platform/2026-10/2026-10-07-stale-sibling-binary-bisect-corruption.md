---
date: 2026-10-07
modules: renderer,protocol,diag
---

# 陈旧 sibling 二进制使进程级二分定位全盘失效（zero-renderer 未随 zero-browser 重建）

## 问题描述

slice37 mm-regression 排查中，为定位 CDP 诊断域的真实执行域（renderer js_worker 还是 webview 沙箱），在 js_worker.rs / page_scripts.rs 埋了文件型 TMPDBG（renderer stderr 走管道，tracing 不可见）。诊断跑完日志为空，据此得出「js_worker 不在路径上，域是 webview」的结论，并沿着 webview 侧排查了一轮。最终发现 **`target/release/zero-renderer` 停留在前一天 22:17**（TMPDBG 埋点之前），浏览器主进程的 `zero-browser` 却是当日 08:23 新建的——空日志只证明「旧二进制没埋点」，不证明「worker 不在路径上」。

## 根因分析

- 多进程架构下诊断插桩按进程分布：`zero-browser`（主进程/tab worker）与 `zero-renderer`（页面脚本执行域）是两个二进制。`cargo build --release --bin zero-browser` 不连带重建 renderer。
- 验证二进制含埋点时只 `strings target/release/zero-browser`——查错了对象。JS 侧探针（`__zwR140Dbg` 等内嵌 shim 字符串）在 browser 二进制里命中，造成了「埋点已就位」的错觉；而真正执行 shim 的是 renderer 进程，其内嵌 shim 来自旧 renderer 二进制。
- 由此，两轮「域归因」证据（JS 探针数组恒空、Rust 文件日志恒空）全部作废，排查方向被带偏一轮。

## 解决方案

1. 重建时把真实执行进程的二进制一并建：`cargo build --release --bin zero-renderer --bin zero-browser`。
2. 验证埋点就位时，对**实际被 spawn 的二进制**做 `strings | grep` + `ls -la` mtime 双查（本次修复后 renderer 中 2 处命中、时间戳当次构建）。
3. 重建后立即复跑同一诊断——本次复跑立刻翻转结论（js_worker 就是诊断域，值扫描命中累积条目才是真根因）。

## 如何避免

- 诊断轮开始前的例行预检：`ls -la target/release/<所有相关二进制>`，任何 mtime 早于最近一次源码改动的先重建；「改动面在 crate X」不等于「只有 binary X 受影响」。
- 「探针无输出」有两种解释：分支没跑，或**探针根本不在场**。下「不在路径上」的结论前，先证明探针在场且可触发（打一条无条件探针验证）。
