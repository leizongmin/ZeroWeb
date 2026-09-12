---
date: 2026-09-12
modules: tests/wpt-runner
---
# import-wpt 空 REF 触发下载失败路径删除测试文件

## 问题描述

以 `make import-wpt TEST=<path> REF=`（REF 提取失败为空）调用导入脚本后，上游 WPT
用例文件从本地 `wpt-data/` 消失，reftest corpus 静默缩水（该目录不在 git 追踪内，
`git status` 无感知）。

## 根因分析

`tests/wpt-runner/scripts/import-wpt-reftests.sh` 的 `--add` 路径：

1. 空参守卫只在 TEST 与 REF **同时**为空时报错——仅 REF 为空会继续执行；
2. 跳过条件是 `[[ -f test && -f ref ]]`——REF 为空时 ref_file = `wpt-data/`（目录），
   `-f` 为假 → 不跳过；
3. curl 下载 test **成功**（上游存在，覆盖本地已有一份）；随后 ref 下载失败
   （空 URL 404）→ 清理分支 `rm -f "${test_file}" "${ref_file}"` **把刚下载（即
   原有）的 test 文件一并删除**。

即「下载成功 + ref 失败」的清理分支会把本地已存在的 test 文件连带删除。

## 解决方案

- 从兄弟 clone（同为 wpt-data v1.10 pin）恢复被删文件后重跑验证；
- 正确用法：REF 必须为 wpt-data 根相对路径（如
  `css/filter-effects/reference/green-100x100.html`），调用前先从测试文件
  `<link rel="match" href="...">` 提取并回显核对；
- 防御性建议（未实施）：脚本空参守卫改为 TEST 或 REF 任一为空即报错。

## 如何避免

调用 `make import-wpt` 前先回显 `TEST -> REF` 配对确认 REF 非空；导入后用
`grep -c <轮次> tests/wpt-runner/imported-tests.txt` 核对账本条数与导入数一致。
