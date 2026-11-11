---
date: 2026-11-11
modules: platform,testing
---

# delivery-check CLI catch 吞真实错误只打 Usage：直调函数取 REAL_ERROR

## 问题描述

账本/交付校验器（`verify-review.mjs` / `delivery-check.mjs` 的
`deliveryCheck`/`checkRemote`）CLI 入口失败时输出只有一行 Usage 提示，真实
失败原因被入口的 catch 整体吞掉。T58 SUM 校验两度被拒：首版 review_scope
填描述文本非字面 'dual' + artifacts 缺首轮角色报告，CLI 只打 Usage；二跑
candidate_manifest=null，同样只打 Usage——两个案子的 REAL_ERROR（'Missing
review scope' / 'Invalid evidence reference'）都是绕开 CLI 直调库函数才拿到
的（存证 workflow-v3-rev943-rejected.json / -rejected2.json）。T20 集成期
checkRemote 失败同形态，直调才见 baseRefOid 不一致真相。

## 根因分析

CLI 包装层把库函数调用整体包进 try-catch，catch 分支只打印 Usage 文本不透出
异常消息/栈——设计意图是「参数错时给用法提示」，实际把**所有**库内校验失败
（参数错、字段契约违反、远端状态不符）统一降级成 Usage。库函数抛出的
Error.message 本来就是精确失败原因（校验器内部对每个判定式都有定制文案），
只是被入口层截留。

## 解决方案

调试协议固定两步：

1. 用 node 直接 import 校验器模块，带任务实参直调 `deliveryCheck` /
   `checkRemote`，取 REAL_ERROR 字符串（Error.message 即判定式定制文案）；
2. 按 REAL_ERROR 修字段/状态后，再走 CLI 正常入账（直调只用于诊断，不入
   账本证据链——账本存证仍以 CLI/工作流产物为准）。

## 如何避免

- 校验器 CLI 的 catch 分支补 `error.message` 透出（入口层修复属校验器仓库
  侧，未落地前一律按本条直调协议取错）；
- 见到「只打 Usage」的失败一律先假定真实错误被吞，不按 Usage 文面排查
  参数拼写；
- 同形态警惕面：任何「catch → print usage/return code」包装层（本仓已两处
  实证：delivery-check、checkRemote）。
