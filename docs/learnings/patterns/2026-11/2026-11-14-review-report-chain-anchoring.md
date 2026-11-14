---
date: 2026-11-14
modules: engineering, review
---

# 审查角色报告链形制：recheck 锚定首轮六键，artifacts 只列当前链身份

## 问题

多轮审查（首轮 → 返修 → recheck）里，角色报告（recheck/非作者复核）常犯
两类锚定错误：一是 recheck 报告的 `from` 字段自造身份（只写 head_sha 或
漏 base_sha），校验器无法把它绑回首轮报告，链条断裂；二是 `summary`
artifacts 贪多列旧轮文件——旧文件不绑定当前审查，延迟校验时文件已不存在，
报错点远离开因。

## 根因分析

审查报告的可信度=身份链（哪一轮、哪个树、覆盖哪些操作）+证据绑定（当前
链身份的 reports/artifacts）。身份链不完整时，同一 PR 的多轮报告无法区分
「对哪个 head 的审查」；artifacts 引用旧轮文件时，链条身份与证据错位。

## 解决方案

形制要点（verify-review 校验器契约）：

1. **recheck `from` 锚定首轮完整六键**：`repo` / `pr` / `base_branch` /
   `base_sha` / `head_sha` / `subject` 全键照抄首轮报告实值（不是凭记忆
   重写；从首轮报告 JSON 读出后原样填入），`covers_operations` 覆盖本次
   返修的 op（证明这轮 recheck 对着返修后的树）。
2. **artifacts.path 只列当前链身份报告**：summary 里列的每个文件必须是
   运行目录真实文件且属于本轮链；旧轮文件不进当前链 summary。
3. **PR 正文审查表行须与 pr-body.mjs `reviewSection` 生成输出字节恒一**：
   手写或改写正文时，用生成器重产比对；不一致即 presentation containment
   FAIL。
4. **head-blob 快照**：报告绑定 head_sha 时同时记录该 head 下被审文件的
   blob OID，防止「同名文件、不同内容」的审查对象漂移。

**Why**：审查链是提交后回归追责的凭据。六键锚定让每轮报告可机器校验地
指回它的审查对象；artifacts 收紧让延迟校验（绑定时才查文件）不在数轮
之后才爆。

**How to apply**：写 recheck 报告前先读首轮报告 JSON，逐键复制；提交前跑
校验器并核对 artifacts.path 每条都 `ls` 得到；PR 正文改动后必须重跑
pr-body.mjs 生成段做字节级比对，不手抄。
