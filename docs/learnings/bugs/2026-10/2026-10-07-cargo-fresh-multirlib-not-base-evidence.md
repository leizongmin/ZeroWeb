---
date: 2026-10-07
modules:
---

# 切分支后 cargo build 显示 Fresh 不能作为「base 代码」证据——多个 rlib fingerprint 变体并存

## 问题

rebase 前 base（不含本任务修复）需要构建对账二进制：切到 base 提交后 `cargo build` 秒级完成且显示 `Fresh`，据此认为拿到的就是干净 base 二进制；后续以该二进制做 reftest-upstream 失败集对账时，一度怀疑对账结果被任务自身代码污染——`strings` 在 deps/ 下某个 `libzero_engine-*.rlib` 里 grep 到了只在修复提交中才引入的新符号 `publish_gcs_drain_record`。

## 根因

`target/release/deps/` 下同一个 crate 可以同时存在**多个 fingerprint 变体的 rlib**（不同 feature/代码内容组合各留一份）。切分支后 cargo 按 fingerprint 判断「Fresh」只说明当前请求的变体已有产物，**不说明该产物对应哪个代码版本**。`strings`/`grep` 搜 rlib 文件名再查内容时，命中的很可能是另一个变体（修复版本）的 rlib，而非当前链接进二进制的那个。

## 解决方案

验证「某二进制是否包含/不包含某段代码」时，以**最终链接产物**为对象、以符号表为准：

```bash
nm target/release/zero-wpt-runner | grep <新符号>
```

- base 二进制应为 **0 命中**（新符号不存在 = 干净 base）。
- 不以 `cargo build` 的 Fresh 输出、deps/ 下 rlib 的 strings 结果、构建耗时作为代码版本证据。

## 如何避免

- 任何「对账 base/head 二进制行为差异」的场景（reftest-upstream 对账、A/B 性能对比），构建完成后先跑一次 nm 符号核验再采信后续数据。
- 若需要绝对干净的 base 产物，用独立 CARGO_TARGET_DIR 或记录并核对产物 hash，而不是依赖共享 target 目录的状态推断。
