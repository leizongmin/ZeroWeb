# Placeholder Caret 与 Legacy 合成测试

- 日期：2026-08-13
- 相关模块：`zero-engine` paint、`zero-browser` 多进程合成测试

## 问题描述

为文本输入框补绘 placeholder 后，多进程 CPU/GPU 合成测试出现“输入后画面不变”。同时，renderer 已发布包含输入文字的新 legacy 快照，但浏览器本地合成仍显示旧画面。

## 根因分析

1. Placeholder 是提示文字，不属于控件当前值。若按 placeholder 字符生成 UTF-16 caret 边界，空 value 的点击 selection 会落到不存在的 offset。
2. 测试只要求 renderer 切换为 `ViewPainted`，但浏览器场景装配仍按 Healthy compositor 读取 compositor 位图，导致新的 `last_render` 从未进入本地合成。
3. Progressive paint 会连续发布首帧和完整帧。仅等待“任意 snapshot sequence 增长”不能证明目标输入帧已经到达。

## 解决方案

- Placeholder 只生成空 value 的单个 offset 0 caret anchor，不为提示字符生成可编辑边界。
- Legacy 本地合成测试同时固定 compositor 状态为 `Disconnected`。
- 交互前等待首屏快照 quiet period；输入后等待 legacy glyph 快照明确包含目标文本，再比较 CPU/GPU 像素。
- 区域视觉断言与全屏 parity 分开，避免小控件变化被全屏面积稀释。
