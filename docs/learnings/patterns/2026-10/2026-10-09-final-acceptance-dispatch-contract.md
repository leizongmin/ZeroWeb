---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# final_acceptance 派发与回执契约速查

## 问题描述

运行整体验收（final acceptance）的状态机契约分散在 verify-workflow.mjs 多处判定里，首次走全流程需要反复试错。本条把可直接照做的契约集中记录（slice 风格运行实测通过）。

## 契约要点

**前置**（全部满足才允许派发）：所有 task done；`versions.trial === null`；delivery_verdict ready；`checkpoint.candidate_manifest.sha256 === versions.best`。

**op 形状**（两段式）：

1. 派发（rev N）：新增 `kind='final_acceptance'` op，`task_id` 必须为 **null**（run 级操作，校验器 L107 对 final op 要求 null，对其他 kind 要求既有 task id）；`status='intended'`、`executor_ref=null`、`result=null`；`subject={manifest_sha256: <best 清单 sha>, contract_sha256: <workflow.contract_ref.sha256>}`（两个 64 位 digest）。
2. 完成（rev N+1）：`status='completed'` + `executor_ref` + `result={path, sha256}`；subject 不动（intended→completed 的相等约束）。

**回执报告**（绑定到 `workflow.final_acceptance` 的 JSON）：

- `schema_version: 1`；`checks[]` 每项 `{id, status}`，id 必须恰好覆盖 workflow goals 全部 id（数量相等），finalPassed 要求全 PASS；
- `subject === checkpoint.versions.best`、`contract_sha256 === contract_ref.sha256`；
- `reviewer: {independent: true, executor_ref: '…'}`——independent 模式下 `executor_ref` 不得等于 checkpoint.activity.executor.ref，也不得出现在任何 implement op 的 executor_ref 里，且**必须等于** final_acceptance op 的 executor_ref（L121/L180）；
- `artifacts[]` 非空、全部过 evidence 哈希校验。

**完成后**：全 task done + finalPassed + 无未决 op + trial null + candidate==best + delivery ready → 校验器自动判 `goal_verdict=PASS`、`next=stop`、`reason=completed`；随后把 `workflow.stop_reason` 显式置 `'completed'`（rev N+2，同 rev 回绑 checkpoint_ref + prev 快照 ref 同步）固化终态，停止屏障生效后不再允许新增操作。
