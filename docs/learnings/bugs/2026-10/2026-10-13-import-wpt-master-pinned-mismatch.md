---
date: 2026-10-13
modules: tests/wpt-runner
---

# import-wpt 从 upstream master 下载与 pinned 快照不一致——ref 404 时连带删除本地既有案

## 问题描述

`make import-wpt TEST=... REF=...` 对一个 **corpus 已在案**的用例
（`css/css-display/run-in/run-in-contains-abspos-001.xht`）执行时，上游 master
已把该案的 ref 归并为 `run-in-basic-ref.xht`（旧 `-ref.xht` 404）。导入脚本的
ref 失败分支执行 `rm -f "${test_file}" "${ref_file}"`——**把本地既有的 test 和
ref 一并删除**。该案不在 git 跟踪内（wpt-data gitignored）、不在
reftest-manifest.json、pinned `zeroweb-wpt-data v1.10` 快照里也没有（系某轮从
master 导入的本地增量案）——三处都不可恢复。

## 根因分析

1. **双数据源漂移**：corpus 的权威源是 pinned 快照（`WPT_DATA_REPO` /
   `WPT_DATA_REF=v1.10`，`make fetch-wpt-data` 幂等跳过），而 `import-wpt`
   硬编码从 `raw.githubusercontent.com/web-platform-tests/wpt/master` 下载。
   上游 master 持续演进（改名/归并 ref），导入时点越晚与 pinned 快照偏差越大。
2. **失败路径误删既有文件**：脚本先 `curl -o "${test_file}"` 覆盖写本地路径，
   ref 失败后 `rm -f` 清理——清理的是「本次下载产物」假设下的路径，但该路径
   同时是 corpus 既有案的唯一副本。
3. **不可恢复态无护栏**：gitignored + 快照外 + manifest 外 = 删除即丢失。

## 解决方案

- 以 master 版本重写测试文件（同一测试语义、ref 链接随上游归并为
  `run-in-basic-ref.xht`），复测渲染等价（0.23% 过阈）后入账 ledger。
- 防复发操作纪律：对 pinned 快照外的本地导入案执行 import-wpt 前，先把既有
  文件备份到 wpt-data 外；导入后跑一次对应 filter 的 reftest-upstream 确认
  分母与逐案结果未漂移。

## 如何避免

- 给 import-wpt 加「目标文件已存在时先备份/拒绝覆盖」守卫（工具侧修复候选）。
- 区分「pinned 快照案」与「本地导入案」两类分母资产，后者考虑集中登记
  （ledger 之外补一份路径清单），使删除可逆。
