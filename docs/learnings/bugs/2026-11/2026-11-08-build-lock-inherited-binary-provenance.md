---
date: 2026-11-08
modules: tests/wpt-runner,build-support
---

# 构建锁等待继承二进制内容：无 Compiling 行的构建不能作归因锚点

## 问题描述

两轮并发各自发起 `cargo build`（经 make 入口），后到者打印 `Blocking waiting for file lock on build directory` 后静默等待，全程零 `Compiling` 行。该构建产出的二进制内容来自**持锁方的树状态**，不是后到者工作树的指纹派生。以该二进制跑判读批（如 bisect 归因、revert 验证），会把持锁方的内容误判为受试内容，得出错误罪判。

实际案例：2026-11-08 F3 inserthtml caret 刀 bisect 中，一条含 `Blocking waiting` + 零 `Compiling` 行的 R5460-revert 构建，其产物实为 full-main 内容，若据此判"R5460 无罪/有罪"均为伪证。

## 根因分析

cargo 的 build-dir flock 是互斥构建入口，不区分"谁的工作树"。锁等待者醒来后 cargo 重新扫描指纹：若持锁方刚构建过**相同输入**则直接复用（零 Compiling 行）；若持锁方构建的是**不同内容**，等待者的产物反映的也是持锁方已写入的 target/ 状态与自身增量的混合，而非受试树从头构建的产物。归因实验要求"二进制内容=受试树内容"，锁等待路径不保证这一点。

## 解决方案

构建锚点判读前断言构建可见性，任一不过即视构建为未证明：

1. 日志含 crate 名 `Compiling` 行（数量与预期一致），且无 `Blocking waiting for file lock`；
2. 结尾有 `Finished` 行；
3. 对 JS shim 内容可加二进制特征串 `grep -ac` 断言（Rust 符号 release strip 后不可用此法）；
4. 见锁等待或 Compiling=0 → 判定"内容未证明"，作废该批判读，重建后再跑。

与 2026-11-07 batch-md5-guard-snapshot-timing 互补：那条管快照时机（构建后落 md5），这条管构建本身是否反映受试树（锁继承使 md5 与内容脱钩）。
