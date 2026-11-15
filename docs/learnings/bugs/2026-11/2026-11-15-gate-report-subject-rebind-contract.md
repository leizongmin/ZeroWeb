---
date: 2026-11-15
modules: site_optimizer
---

# 换 CP 候选必须同步重绑 gate 报告——verify-run 的 Gate report subject 硬校验

## 问题描述

base 重锚后生成 candidate-manifest v4（sha 变化），checkpoint v4 首版把
`required_gates` 仍指向 v3 期的 gate 报告，`scripts/verify-run.mjs` 立即
EXIT=2，错误只有一句概括。探针直调 `verifyRun` 取 REAL_ERROR 才定位：
`Gate report subject mismatch`。

## 根因分析

`verify-run.mjs` 对每个 gate 报告校验 `report.subject === state.candidate_manifest.sha256`
（L107/L130 附近）。gate 报告文件内的 `subject` 字段绑定的是**被验候选的
manifest sha**，不是独立元数据——候选换了，旧报告描述的是旧候选，放行就是
验非所载。v3→v4 只是 base_sha 重锚（代码零变化），subject 也必须跟着换，
因为校验器按 sha 恒等判读，不做语义比较。

## 解决方案

- 换任何 CP 候选（哪怕只是 base 重锚、note 修订）时，同步重生成 gate 报告：
  检查项与判定可沿用（重锚轮不重跑语义写进 note 如实记录），但 `subject`
  字段必须等于新 manifest sha。
- 被拒的 CP 首版按存证纪律改名保留（如 `*.rejected-gatesubject`）；未被任何
  rev 引用的草稿重生成时**版本号不前跳**，同一版本号重生成即可。
- 校验器 CLI 只给概括错误时，探针直调导出函数取 REAL_ERROR，不重试洗白。
