---
date: 2026-11-15
modules: tooling
---

# systemd-run --scope 不支持 -p TimeoutSec——Unknown assignment 立即失败，超时兜底须放脚本内

## 问题描述

无人值守集成 smoke 用
`systemd-run --user --scope -p MemoryMax=14G -p MemorySwapMax=0 -p TimeoutSec=5400`
包裹，scope 立即失败报 `Unknown assignment: TimeoutSec=5400`，三条腿的输出
文件一个都没产生。去掉该属性重启后成功。

## 根因分析

`-p TimeoutSec` 是 **service** 单元属性，scope 单元不支持；systemd-run 对
不认识的属性直接拒绝启动，不会降级或警告后继续。而 `.service`（`--unit`
默认形态）会进入 cgroup 委派问题：run-unit 的 stdio 进 journal，落盘输出
拿不到——所以本项目守卫统一用 `--scope`，代价就是失去 service 层的超时
属性。

## 解决方案

- scope 形态守卫只带内存属性：`systemd-run --user --scope
  -p MemoryMax=14G -p MemorySwapMax=0`，墙钟超时放**脚本内部**
  `timeout --foreground <sec> <cmd>` 逐腿兜底。
- 每腿退出码落盘（`echo "leg_exit=$?" > leg.exit`），成功与否以输出文件
  成功标记判读，链式脚本禁接 `| tail`（吞崩溃栈、洗白退出码）。
- 守卫首启失败时先查属性合法性（`Unknown assignment` 行），再查
  cgroup/委派——首启无任何输出文件就是 scope 没起来，不是测试挂了。
