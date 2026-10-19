---
date: 2026-10-19
modules: browser
---

# quickjs 测试矩阵不含 zero-browser --tests，其中 bookmark 建标签页测预存红

## 问题描述

手动跑 `cargo test --no-default-features --features quickjs -p zero-browser
--tests` 会遇到 `tests::bookmark_context_menu_open_in_new_tab_loads_url`
稳定失败（「应新建一个前台标签页」left: 1 right: 2，0.02s 快败）。该测试
在 v8 构建下通过。

## 根因分析

1. 官方 quickjs 腿用的是 `QUICKJS_TEST_CRATES_WITHOUT_BROWSER_OR_RENDERER`
   （Makefile），zero-browser 的 quickjs 测试从来不在门禁矩阵内——此红
   不会被 make test 暴露。
2. A/B 确证与引擎语义层无关： checkout s11 父提交（9db701232）的 shim
   重跑同测仍失败。测试路径为纯 Rust 浏览器壳行为（右键菜单 →
   start_tab_request → tab_count），断言不执行页面脚本；quickjs feature
   构建下浏览器壳该路径存在预存缺陷或环境差异，与 engine shim 改动无关。

## 解决方案

- 不因该红回溯 engine shim 改动；也不把它记为回归。
- 若要真修：属浏览器壳 × quickjs feature 组合域，需单独立项；
  修复前可考虑把该测纳入 quickjs 排除清单或修好后扩矩阵，由浏览器壳
  goal 决定。
