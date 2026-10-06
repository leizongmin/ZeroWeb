---
date: 2026-10-07
modules: engine,renderer
---

# 多进程二进制分工：DOM shim 在 zero-renderer，`-p zero-browser` 构建不覆盖它

## 问题描述

t7 修复 DOM shim（`crates/engine/src/js_dom_shim/part05.js`）后，用 `cargo build --release -p zero-browser` 重建并跑 CDP 探针验证——修复完全未生效（gtnDiv 依旧 ~200ms）。排查一度怀疑修复方向错误。

## 根因分析

多进程架构下 JS shim 的实际消费者是 `zero-renderer`（js worker 所在进程）：`zero-browser` 主进程只负责窗口/IPC，`ProcessManager::spawn` 按同目录约定定位 `zero-renderer` 二进制（`apps/browser/src/process_backend.rs` 的 `renderer_binary_filename` + `current_exe` 同目录回退）。

`cargo build -p zero-browser` 只构建 zero-browser 及其依赖闭包——zero-renderer 是独立 bin target，**不在该闭包内**（尽管两者都依赖 zero-engine）。结果：zero-engine 新 rlib 编出来了，但 `target/release/zero-renderer` 还是旧二进制，旧 shim 继续在 renderer 进程里跑。

验证手段：shim 经 `include_str!` 以字符串字面量嵌入，`grep -a -c "<新函数名>" target/release/zero-renderer` 直接确认新代码是否在二进制里。注意 grep zero-browser 二进制找不到 shim 符号是正常的（主进程不引用该字符串，链接器 GC 丢弃）。

## 解决方案

改 shim / js worker / renderer 侧代码后一律：

```bash
cargo build --release -p zero-renderer   # 或 --workspace
```

探针验证前先 `grep -a -c <新代码特征串> target/release/zero-renderer` 确认新鲜性，再做浏览器验证。与 cargo Fresh 陷阱（同目录 [[cargo-fresh-multirlib-not-base-evidence]]）同族：**验证行为前，先用二进制内容自证修复已进入实际执行体**。

## 如何避免

- 涉及 shim / renderer 行为的修复，构建命令默认 `-p zero-renderer`（或 workspace 级），不用 `-p zero-browser`。
- 二进制新鲜性核对（mtime + 特征字符串 grep）作为探针验证的前置步骤。
