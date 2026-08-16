# Windows 同排标签栏与 Snap Layout

- 日期：2026-08-16
- 相关模块：`apps/browser`、Win32 non-client hit test

## 问题描述

Windows 上希望标签栏和窗口控制按钮同排，并在悬停最大化/还原按钮时显示 Windows 11 Snap Layout。

## 根因分析

把 winit/wgpu 的客户区透明化，再让 DWM 覆盖绘制 caption buttons，在非最大化窗口上不能稳定工作：DWM 按钮可能不绘制，或与系统 frame 叠加，导致标题栏和透明空洞同时出现。

## 解决方案

采用 Chrome 风格的稳定路径：

- 无装饰窗口由浏览器绘制最小化、最大化/还原、关闭按钮；
- 标签栏右侧预留按钮区域；
- 在 `WM_NCHITTEST` 中，仅将自绘最大化/还原按钮矩形返回为 `HTMAXBUTTON`；
- Windows 11 因此把它识别为最大化控件并在悬停时显示系统 Snap Layout；最小化和关闭仍由浏览器正常处理。

该模式保留 Chrome 式单行标签栏和稳定 GPU/CPU present，不依赖不透明 surface 下不可控的 DWM 覆盖绘制。

## 验证方式

- 单测确认 Windows 使用无装饰窗口和浏览器控制按钮；
- 实际窗口确认三个按钮可见；
- 悬停中间最大化/还原按钮，确认 Windows Snap Layout 出现。
