---
date: 2026-08-12
modules: apps/browser, apps/renderer, tests/wpt-runner, Makefile
---

# Windows GUI 自动测试的进程与字体隔离

## 问题描述

表单交互修复在单测中间歇失败，产品页像素门禁也只在 Windows 超过阈值。失败表面上分别表现为合成器断连、渲染快照超时、文字重叠和像素差异增大，容易被误判为鼠标坐标或本轮布局修改回归。

## 根因分析

浏览器测试中存在两类进程级共享状态：合成器客户端和渲染子进程。并行测试若修改合成器环境变量、启动后再杀死全局合成器，会污染同进程的其他用例。另一方面，测试构建下的进程内 `TabWorker` 不启动真实 JS worker，不能代表 `scripts/browser.ps1` 使用的多进程生产路径。

WPT runner 的基础字体列表原先只有 Linux 和 macOS 路径。Windows 因此只能加载 Ahem，导致普通页面文字被画成方块。补齐 Windows 字体后，产品页默认启用的实验性 shaped-text 路径仍会把 Chromium Oracle 差异从 17.34% 放大到 23.68%；使用项目已有的 `ZW_SHAPED_TEXT=0` 诊断开关后，结构检查和 20% 像素门禁同时通过，证明该差异属于测试器整形路径，而不是表单坐标或布局回归。

## 解决方案

- 需要模拟断连时向状态处理函数注入 `CompositorStatus::Disconnected`，不再修改进程环境或终止全局合成器。
- `zero-browser` 测试串行执行；其余 workspace crate 保持并行，兼顾隔离与耗时。
- 表单端到端用例启动真实 `zero-renderer`，断言最终 DOM 值、唯一事件目标和新帧，不能只验证进程内替身或快照序号。
- Windows WPT runner 从 `WINDIR/Fonts` 加载 Times New Roman、Arial regular/bold、Consolas 和 Microsoft YaHei，并用单测约束 regular/bold 映射不同。
- 在 shaped-text 的 Windows 字体兼容问题单独修复前，产品基线诊断显式使用现有 kill switch；不得提高像素阈值或替换 Oracle。

## 如何避免

GUI 测试必须区分线程内替身、真实 renderer 和进程级共享服务。涉及全局合成器的异常测试应使用依赖注入，不应操纵其他用例共享的外部进程。跨平台像素测试必须先证明实际加载了普通、粗体和 CJK 字体；出现平台独有差异时，应做同一二进制的功能开关 A/B，再判断是产品回归还是测试路径偏差。
