---
date: 2026-10-02
modules: infra/cargo, infra/worktree
---

# 共享 CARGO_TARGET_DIR 的 git worktree 会静默覆盖主树产物

## 问题描述

同一 target 目录（显式 `CARGO_TARGET_DIR=<主树>/target`）下，从临时 worktree（如 base SHA
验证）编译过的 crate，回到主树增量编译时报「符号不存在」类错误（E0432/E0599），而源文件
明显包含该符号。实测场景：主树给 `zero-protocol` 新增 IPC variant 后，在 base worktree 编译
renderer 测试，回到主树跑 `cargo test -p zero-renderer --lib` 即报新 variant unresolved。

## 根因分析

同名 package、同 version、同 feature 组合在两个 worktree 里算出的 `-C metadata` 哈希相同，
产物落盘为**同一个文件** `target/debug/deps/libzero_protocol-<hash>.rmeta`。worktree 编译时
直接覆盖该文件，而主树的 cargo fingerprint（按 mtime 记录）仍判定「fresh」——后续主树增量
编译会拿被覆盖的旧 rmeta 参与编译/链接，fingerprint 与文件内容脱节。cargo 不校验内容一致性，
错误推迟到下游 crate 的符号解析才暴露，报错指向下游而非真凶。

## 解决方案

- 回主树后强制失效被污染的 crate：`cargo clean -p <crate>`，再增量编译即可恢复
  （2026-10-02 slice20 实测 `cargo clean -p zero-protocol` 后 179P/2F 符合预期）。
- 预防：跨 worktree 验证要么用独立空 target（代价：v8 等全量重编），要么在返回主树后
  立即 `cargo clean -p` 过在 worktree 里编译过的所有 workspace crate；不要信任增量结果。
- 已链接完成的 bin 可执行文件不受影响（链接时快照），但任何「在 worktree 编译过 → 回主树
  直接跑测试」的序列都必须先重建。
