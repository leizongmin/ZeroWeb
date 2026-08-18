---
date: 2026-08-14
modules: apps/browser, scripts/browser*.ps1
---

# GPU HiDPI 二次缩放与 compositor 启动缺失

## 问题描述

Windows 高 DPI 环境下，GPU 模式的浏览器界面比 CPU 模式放大数倍；同时启动日志提示找不到 `zero-compositor`，随后回退到 legacy frame publishing。

## 根因分析

Browser 的 chrome 与 WebView 图元在场景拼装阶段已经按窗口 DPI 转换为物理像素。GPU 光栅入口再次接收真实 `scale_factor` 后又缩放一次，形成二次缩放。CPU 光栅入口使用 `1.0`，因此没有该问题。

compositor 客户端会从 browser 可执行文件同目录解析 `zero-compositor.exe`，但 Windows 启动脚本只构建了 browser 和 renderer，导致目标文件不存在。

## 解决方案

Browser 合成边界明确采用物理像素场景，GPU 光栅固定使用 `1.0`；底层 GPU renderer 仍保留 scale 参数，供直接传入逻辑坐标的其他调用方使用。Windows GPU/CPU 启动脚本同时构建并校验 browser、renderer、compositor 三个进程。

回归测试覆盖高 DPI 下的 CPU/GPU 合成像素一致性，以及两个 Windows 启动脚本的 compositor 构建与存在性门禁。
