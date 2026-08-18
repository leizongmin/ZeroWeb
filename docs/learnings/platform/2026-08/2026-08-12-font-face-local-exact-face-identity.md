---
date: 2026-08-12
modules: css-parser, render-foundation/font, WPT runner
---

# `@font-face src: local()` 必须保持精确 face 身份

## 问题描述

实现 `@font-face src: local()` 时，曾尝试在 Linux 上将 `local("Arial")` / `local("ArialMT")` 映射到 generic `sans-serif` 最终选中的 Liberation Sans，以模拟 Chromium/fontconfig 的常规 Arial 替代。

## 根因分析

`local()` 不是普通 CSS family fallback。CSS Fonts 要求它匹配已安装字体的本地 face 名或 PostScript 名；命中后得到的是该具体 face。将不存在的 Arial 名称替换为 Liberation Sans 改变了 face 身份。

Chromium Oracle `font-face-local-not-family.html` 的像素差从 `7.38%` 退化到 `7.84%`，证明 generic family 的 metric-compatible 替代不能用于 `local()`。

## 解决方案

该实验已完整回退。后续实现必须：

+ 从字体 `name` 表索引 family/full/PostScript face 名。
+ `local()` 只做大小写规则允许范围内的精确 face 查询。
+ 未命中时继续按 `src` 声明顺序尝试下一个 local 或 URL。
+ 不得复用 generic family/fontconfig 替代结果冒充 local face。
