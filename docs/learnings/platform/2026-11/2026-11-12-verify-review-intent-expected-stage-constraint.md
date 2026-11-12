---
date: 2026-11-12
modules: .agents/skills/zeroweb-site-optimizer
---

# review 类 intent 的 expectedStage 约束——implementing 态直派必被拒

## 问题描述

任务被 needs_fix 打回后处于 `implementing` 态，返修完成即直派 review 类 intent，
verify-workflow 校验拒绝派发，报 `Task is not ready for dispatch`（T62 rev1035
dispatch-fail 存证）。报错不带哪个条件不满足，初看像账本损坏。

## 根因分析

`verify-workflow.mjs` 派发段（L300-302）对每类 intent 定义 expectedStage 映射，
review 类要求 `task.status` 为 `pr_review`（已是 PR 审查态时）否则 `verifying`；
L313-316 统一断言 `task.status === expectedStage[op.kind]`。也就是说 review intent
期望任务处于**验证中或 PR 审查**态，`implementing` 不在合法集内——语义是「先有
实现完成后的验证段，才有资格进审查派发」，返修后直接跳进 review 会绕过验证段。

## 解决方案

正确序列：needs_fix 返修完成后，先做一次**裸过渡**到 `verifying`（只改任务状态、
不挂任何 op），再派 review intent。裸过渡本身无副作用、不消耗预算；跳过它直派
review 是状态机违例，不是格式问题。排查同类派发失败时直接核对 expectedStage
映射与任务当前态的差值，不必怀疑账本损坏。
