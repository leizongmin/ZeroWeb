---
date: 2026-10-01
modules: apps/renderer,crates/engine
---

# 改了 js_dom_shim 却在实站看不到效果——只重编 zero-browser 不重编 zero-renderer

## 问题描述

调试一个 JS shim（`js_dom_shim/part05.js`）修复时，实站验证（headless 起
`target/release/zero-browser` 访问页面）始终表现为修复前行为；同一份源码在
`cargo test -p zero-engine --lib` 单测里却是修复后行为。疑似"实站 realm 与单测
realm 不一致"，排查方向一度跑偏到脚本执行上下文重建/竞态。

## 根因分析

多进程架构下页面 JS 的真正执行者是 **`zero-renderer` 子进程**，`zero-browser`
主进程只负责窗口/调度。`cargo build --release -p zero-browser` 只重编主进程，
不会重编 `zero-renderer`（不是它的依赖）。于是：

- 单测直接编译当前源码 → 看到修复；
- 实站 spawn 的 `target/release/zero-renderer` mtime 还停留在修复前 → 跑旧 shim。

且 release 二进制里的 shim 是 `include_str!` 内嵌的 JS 文本，`strings | grep`
在 zero-browser 里搜不到 shim 标识字符串（搜索目标二进制就错了），不能用
"二进制里有没有新字符串"对 zero-browser 做修复注入判断。

## 解决方案

改 shim/engine 后按消费者重编：

```bash
systemd-run --user --scope -p MemoryMax=24G cargo build --release -p zero-renderer
grep -ac "<新shim里的独特字符串>" target/release/zero-renderer   # 验证注入
```

判断"页面行为归哪个二进制"的经验：**页面 JS 的问题先查 zero-renderer 的
mtime，再怀疑代码**。实站验证前用 `stat -c '%y' target/release/zero-renderer`
对照源文件修改时间，二进制比源码旧就是陈旧验证。
