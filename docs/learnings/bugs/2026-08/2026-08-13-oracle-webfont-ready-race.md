---
date: 2026-08-13
modules: WPT Chromium oracle capture, CSS Font Loading
---

# Oracle Webfont Ready Race

## 问题描述

Chromium oracle 中依赖本地 `@font-face` 的测试偶发显示 fallback 字体。`font-size-adjust-012/013` 的 ZeroWeb self-source 已通过，但旧 oracle 仍显示 X/A，而测试资源 AhemEx250/500 应渲染方块，导致 13% 至 16% 的伪差异。

## 根因分析

截图脚本等待了 `networkidle0` 和图片 decode，但网络空闲不等于字体解码及 face swap 完成。截图可能发生在 fallback 字体仍生效的窗口。

## 解决方案

在截图前 bounded 等待 `document.fonts.ready`，顺序固定为 network idle、图片完成、字体完成、短延时、截图。等待有超时兜底，损坏字体不会卡住批量捕获。

## 如何避免

导入或更新 webfont 资源后必须重抓对应 oracle。若 ZeroWeb self-source 与 Chromium oracle 的 glyph 形状明显不同，先核对字体资源和 `document.fonts.status`，不要直接修改渲染逻辑追旧截图。
