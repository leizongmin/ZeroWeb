---
date: 2026-08-09
modules: tests/wpt-runner
---

# macOS 上 product-smoke 的系统字体加载

## 问题描述

`make product-smoke` 在 macOS 上把页面文字渲染成方块，导致 welcome fixture 与 Chromium oracle 的像素差超过门禁阈值。

## 根因分析

`zero-wpt-runner` 的默认 `FontLoader` 只枚举 Linux 的 DejaVu、Liberation 和 Noto CJK 路径。macOS 上这些路径均不存在，最终只加载了测试专用 Ahem 字体，普通文本因此错误地使用方块字形。

## 解决方案

在保持 Linux 字体优先级不变的前提下，增加 macOS 系统自带的 Times New Roman、Arial、Arial Bold 和 Hiragino Sans GB 路径，并用 macOS 定向单测确认默认 loader 能解析 `Arial` 和 `sans-serif`。

产品 oracle 仍作为已提交的离线门禁输入；本地 Chromium 重生成结果受操作系统字体和浏览器版本影响，不应未经 smoke 校验直接替换 canonical oracle。
