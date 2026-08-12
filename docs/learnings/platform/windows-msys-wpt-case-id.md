# Windows WPT case id 不能使用宿主路径分隔符

- 日期：2026-08-12
- 相关模块：`wpt-runner`、`run-reftest-smoke.sh`

## 问题描述

WPT 文件实际存在且单个短名称过滤能够通过，但 smoke 清单中的 `css/CSS2/...` 完整 case id 在 Windows 上全部匹配为 0，被误报为语料缺失。

## 根因分析

runner 使用 `Path::to_string_lossy()` 生成 case id，Windows 结果包含反斜杠，而跨平台 smoke 清单使用正斜杠。与此同时，Git Bash 启动 Windows `.exe` 时还可能把包含 `/` 的逻辑参数当成本地路径自动转换。

## 解决方案

对外 case id 按 URL 标识符处理，在 loader 边界统一把 `\\` 转成 `/`，并用单测锁定。Git Bash 调用 Windows runner 时显式选择 `.exe`，设置 `MSYS2_ARG_CONV_EXCL=*`，禁止 MSYS 改写 case id 参数。
