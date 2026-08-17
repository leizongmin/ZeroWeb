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

## 2026-08-17：强制 compositor 后的时序更新

浏览器切为强制 compositor 后，renderer 的 `CompositorFrame` 到达 Browser 时会先解码到 `last_render`，再异步转发给 compositor。测试看到 `last_render` 已包含输入文字，只能证明 renderer 提交已到达，不能证明 compositor 已完成光栅化并由 Browser 采用。此时立即截图会稳定读取旧位图。

修复方式是记录包含目标文字的最新 compositor submission frame id，并轮询到 Browser 已采用的 compositor frame id 不小于该值，再比较 CPU/GPU 像素。等待任意 snapshot sequence 或固定 sleep 都不可靠：前者可能被更早的 click/caret 帧满足，后者不提供完成关系。
