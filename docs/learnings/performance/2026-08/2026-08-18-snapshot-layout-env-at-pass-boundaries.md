---
date: 2026-08-18
modules: zero-layout-engine
---

# 在布局 pass 边界快照环境开关

## 问题描述

R3443 后的 medium DWARF profile 中，`getenv` self 稳定占 3% 以上。调用栈显示部分布局 kill-switch 在递归后处理、字体度量和逐 run advance 热路径中反复读取，即使其值在一次布局期间不会变化。

## 根因分析

环境变量适合进程启动或 pass 策略选择，不适合放在逐节点、逐 run 循环中。递归函数每访问一个盒都重新扫描环境，字体度量则每次解析行高都重复读取相同开关。单个读取很小，但 medium 的数千节点和文本 run 会将固定策略读取放大为可见热点。

## 解决方案

递归 postprocess 在公开入口读取一次开关，并把布尔值传入私有递归 helper。IFC 残余字体/trace/fallback 开关复用 `inline/runtime_flags.rs` 的进程快照机制。`ZW_POSTPROCESS_ENV_SNAPSHOT=0` 与 `ZW_LAYOUT_RESIDUAL_ENV_SNAPSHOT=0` 分别恢复旧的 live lookup，原功能开关语义不变。

快照范围只包含一次布局内稳定的策略；可能需要同进程动态变化的页面状态不纳入。

## 验证

固定 CPU31 的 medium DWARF off/on profile 中，新增四个 IFC runtime caller 的 `getenv` 样本从 26 降到 0，column-flex 与 relative-percent 递归 caller 从 4 降到 0；总 `getenv` 栈从 80 降到 39，self 从 3.64% 降到 2.26%。

非 profile 反序可比组中，paint p50/p95 近稳，layout p50/p95 从 `267.18/328.59ms` 降到 `255.43/312.67ms`，改善 4.4%/4.8%。另一组 parse/style/layout/paint 全部同步变慢 10% 以上，判为共享主机漂移并丢弃。关闭/开启快照的 medium PNG SHA-256 完全一致。
