---
date: 2026-08-18
modules: 工具链 / cargo / git worktree / 性能 A/B 验证
---

# git worktree 共享 CARGO_TARGET_DIR 导致构建指纹污染

- 问题：跨 worktree 的 A/B 性能对照测得伪结果（32µs），并与真实值（45µs）相差 30% 以上；随后主树构建出现 `no NodeIdMap in the root` 的幽灵编译错误。

## 根因分析

cargo 的构建指纹（fingerprint）按 **package ID**（name + version + source）存储于 target 目录。两个 worktree 检出同一仓库的不同提交时，本地 path 依赖（如 `zero-layout-engine v0.1.0`）具有**相同的 package ID**，但源码不同：

1. worktree A 构建后，其指纹（记录 A 源码 hash）写入共享 target。
2. 切到 worktree B 构建时，cargo 可能认为该 package 已构建（指纹恰好匹配 B 的旧状态、或 last-write-wins 的竞态），复用 A 的 rlib。
3. 结果：B 的源码链接到 A 的 rlib——轻则测得"错误版本"的二进制（性能 A/B 失真），重则 `unresolved import`（新符号在旧 rlib 中不存在）或运行时行为不一致。

本次事故链条：为做 `compositing_layer_analysis_200` 的父 SHA vs HEAD 对照，在 `/tmp` worktree 检出 `873c509e7` 并共享主树 `CARGO_TARGET_DIR`。交替测量期间 cargo 复用了过期工件，测得父/HEAD 均为 32-33µs 的伪 PASS 结论；`cargo clean -p` 只清理部分 crate 后仍残留交叉污染，直至 `cargo clean` 全清 + 独立 target 目录才恢复正确测量（真实值：父 35.8µs / HEAD 45.7µs，回归真实存在）。

## 解决方案

- **A/B 对照必须使用独立 target 目录**（`CARGO_TARGET_DIR=/tmp/xxx-target`），严禁跨 worktree 共享。
- 修复已有污染：`cargo clean`（全量）后重建；`cargo clean -p <crate>` 可能不够。
- 判定 A/B 可信度的快速信号：同命令两次运行结果若与历史值完全一致（同值到小数点后多位），大概率在复用旧样本/旧二进制，而非真实测量。

## 如何避免

- 写性能对照脚本时，把 `CARGO_TARGET_DIR` 设为 worktree 私有路径（cargo 1.85 的 `target-dir` 支持逐 worktree 配置）。
- 对照结论要求：每棵树的测量值与其**独立构建**的二进制一一对应；先跑一次能区分版本的 smoke 断言（如符号存在性）再进入测量。
