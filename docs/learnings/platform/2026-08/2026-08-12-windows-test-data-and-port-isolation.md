---
date: 2026-08-12
modules: zero-wpt-runner, zero-webview, Windows 测试门禁
---

# Windows 全量测试的数据与端口隔离

## 问题描述

Windows 上执行全工作区测试时先后出现两类非产品回归：WPT 字体指标测试找不到被忽略的 `wpt-data`，以及多个 WebView 本地 HTTP 用例并行时偶发连接失败。

## 根因分析

`tests/wpt-runner/wpt-data` 按仓库约定不纳入 Git，测试前必须获取与 WPT 用例匹配的精确字体版本；仅凭同名文件从其他项目复制可能拿到不同版本，字体 advance 指标会不同。WebView 覆盖用例各自启动临时服务器，并行执行时存在端口生命周期竞争，串行执行则稳定通过。

## 解决方案

测试前校验所需 WPT 资产存在且版本正确。本次使用的 `noto-sans-v8-latin-regular.woff` SHA-256 为 `5C38AA037B5D6AC9EC623153FE9288F1A8DA306E03C39F4D5F5B7DF549AEE47B`。完整门禁在 Windows 上采用：非浏览器工作区 `--test-threads=1`，随后单独串行执行 `zero-browser`；所有命令仍由 test-guard 限制内存与墙钟。

## 如何避免

长期应让测试资产获取脚本验证摘要，并让本地 HTTP fixture 绑定端口 `0` 后把操作系统实际分配的端口传给客户端，避免预选端口与服务器就绪之间的竞争。在此之前，不应把并行偶发失败当成产品代码失败，也不能只重跑单个失败用例后省略串行全量门禁。
