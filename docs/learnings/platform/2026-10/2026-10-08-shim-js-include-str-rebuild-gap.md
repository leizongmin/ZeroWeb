---
date: 2026-10-08
modules: engine
---

# shim js 变更后 cargo 未重编 engine：探针打到旧 renderer 的构建盲区

## 问题描述

修改 `crates/engine/src/js_dom_shim/part04.js`/`part05.js` 后执行
`cargo build --release -p zero-engine -p zero-browser`，构建仅 25s 且只出现
`Compiling zero-page-runtime` / `Compiling zero-browser`——zero-engine（include_str!
消费方）没有重编。由此 `target/release/zero-renderer` 停留在旧 shim（multi-process
架构下 JS 宿主是 renderer，browser 二进制不含 shim 字节），后续页面探针全部打到旧
代码，修复"看起来无效"，浪费一整轮排查。

## 根因分析

1. **架构事实**：shim 经 `js_dom_bridge.rs:4222` 起的 `include_str!` 内嵌进
   zero-engine（lib），链接产物是 `zero-renderer`；`zero-browser` 主进程二进制里
   grep 不到任何 shim 标记。验证 shim 是否生效必须查 renderer 二进制。
2. **追踪失效**：rustc dep-info 正常应把 include_str! 文件纳入 cargo 指纹，但本轮
   js 文件 mtime（09:22）晚于上次构建（09:07）仍被判 fresh——原因未深究（可能与
   workspace 增量指纹或文件系统 mtime 粒度有关），**不可依赖 include_str! 的自动
   追踪**。

## 解决方案

- **强制重编**：`touch crates/engine/src/js_dom_bridge.rs`（.rs 源文件变更必触发）
  + 显式 `-p zero-renderer`。
- **构建后自证**（最重要——把"代码生效"从假设变成验证）：

  ```bash
  LC_ALL=C grep -a -o '<本次改动独有标记>' target/release/zero-renderer | wc -l
  ```

  改 shim 时在代码里留一个本次独有的短标记（变量名/注释词均可），构建后 grep
  计数 >0 才开始跑探针。`strings -8` 在大体量内嵌字符串上会漏报，用
  `LC_ALL=C grep -a -o` 直接二进制检索。
