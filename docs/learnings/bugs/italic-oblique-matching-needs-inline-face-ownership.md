# Italic/oblique matching 依赖 inline face ownership

日期：2026-08-14

相关模块：`zero-render-foundation::font`、`zero-engine::paint`

## 问题描述

`italic-oblique-fallback` 要求 `italic` 与 `oblique` 使用不同 face category，并规定非对称 fallback：italic 优先 oblique，oblique 在存在 normal face 时不能回退 italic。

ZeroWeb 的字体加载链把两者折叠为 `is_italic: bool` 和同一个 `:italic` alias。

## 根因分析

实验将静态 `@font-face` 链升级为 Normal/Italic/Oblique 三态，并让 resolver 使用 `italic -> oblique -> normal`、`oblique -> normal -> italic` 顺序。shared resolver 单测证明选择规则正确，所有宿主也成功编译。

但同一 release runner 的完整 css-fonts Chromium Oracle A/B 为 282 案逐像素全持平，目标页保持 `3.49%`，self-source保持 `3.36%`。目标声明位于 inline span；当前 paint 路径没有让该 owner 的三态 face选择影响最终 glyph。单独修 resolver只是 dormant plumbing。

## 解决方案

不要单独重开三态 alias/resolver。后续必须先让 inline owner 的 resolved face成为 layout、shaping 与 paint 的共同输入，再以该 WPT 验证三态 matching。该工作属于现有 inline ownership/Phase A 边界。

验收必须同时要求目标页像素变化和全 css-fonts net 不退；resolver 单测通过不能替代生产像素证据。
