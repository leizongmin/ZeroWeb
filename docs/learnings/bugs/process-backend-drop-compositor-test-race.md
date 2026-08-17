# Process backend drop can race multiprocess compositor tests

**日期**: 2026-08-18

**相关模块**: `apps/browser/src/process_backend.rs`, `apps/browser/src/tests.rs`

## 问题描述

Browser 全量测试中，真实多进程表单用例偶发等待不到首帧；同一用例单独运行稳定通过。失败时 snapshot sequence 保持为 0。

## 根因分析

`ProcessTabBackend::Drop` 调用 `shutdown_all()`，后者会关闭进程级全局 compositor。部分纯单测构造并销毁 `ProcessTabBackend`，但没有获取真实 GUI 多进程测试使用的 `MULTIPROCESS_TEST_LOCK`。两类测试并行时，纯单测可把活跃 GUI 用例的 compositor 状态切换为 `Disconnected`，导致 epoch 正确的首帧仍在 Browser 入口被丢弃。

## 解决方案

所有构造 `ProcessTabBackend` 的单测与真实多进程 GUI 测试共用 `MULTIPROCESS_TEST_LOCK`。该锁覆盖 backend 的完整生命周期，确保 Drop 阶段不会关闭其他测试正在使用的全局 compositor。

## 如何避免

新增持有进程级全局资源且 Drop 会执行全局清理的测试 fixture 时，必须复用该资源现有的跨模块测试锁。仅给“会 spawn 子进程”的测试加锁不够；不 spawn 子进程但会触发全局 teardown 的 fixture 同样需要加锁。
