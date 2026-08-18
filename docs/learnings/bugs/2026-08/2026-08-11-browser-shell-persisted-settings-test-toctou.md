---
date: 2026-08-11
modules: crates/browser-shell
---

# BrowserShell 持久化设置测试的 TOCTOU 竞争

## 问题描述

`make test` 并行执行 workspace 测试时，`test_browser_shell_new_with_persisted_settings`
偶发读到不同的搜索引擎设置。测试先构造 shell，再重新读取默认配置文件比较两次结果。

## 根因分析

测试把真实用户配置 `~/.config/zeroweb/settings.json` 当成 fixture。workspace 中其他测试进程会
更新该文件，因此两次读取之间存在 TOCTOU 竞争；即使两次读取都正确，结果也可能不同。

## 解决方案

为 `BrowserShell` 增加 crate 内可用的 path-injected 持久化构造器。测试在独立临时目录写入
确定性设置，只读取一次该隔离文件；生产构造器仍委托默认配置路径。

持久化测试不得读取或改写真实用户配置。需要验证默认路径时，只验证路径形态，不验证并行期间
文件内容保持不变。
