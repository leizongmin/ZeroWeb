---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# 账本状态机的字段约束以转换器代码为准——回填/绑定/位置三坑

## 问题描述

网站任务验收账本（workflow.json/checkpoint.json）落账时连续踩中三个同源坑，均为「按直觉补字段」与状态机实际约束不符：

1. **intended op 完成时 `subject` 不得回填**：把 `op-s48-integrate` 从 intended 推进到 completed 时补了 `subject`（PR 串），转换器报 `Operation history changed`。规则（verify-workflow L275-281）：prev 与 current 之间 op 的 `subject` 必须逐字节相等；可变字段只有 `status`（只能前进）、`executor_ref`（只能 null→有值）、`result`、`summary`。slice47 的 completed integrate op 带 subject，是因为它当轮**直接新增**（新增 op 不受相等约束），不是后填的。
2. **task done 必须绑 `evidence`**：任务转 done 后校验器报 `Done task requires evidence`——done 任务需 `task.evidence` 指向 task 级 manifest（diag/manifests/task-sliceNN.json），不能只靠 delivery 回执链。
3. **`checkpoint_ref` 挂 workflow 顶层**：回绑时写 `wf['activity']['checkpoint_ref']` 抛 KeyError——workflow.json 无 activity 键，checkpoint_ref 在顶层。

## 根因分析

与「回执 schema 以校验器代码为单一事实源」（同日另一条 learning）同根：状态机的字段级约束（哪些字段不可变、哪个阶段必须绑什么、字段挂在哪一层）只存在于 verify-workflow.mjs/verify-run.mjs 的判定代码里，运行目录的历史文件各切片形状有差异，照着任一实例抄都可能踩到别的切片没走过的路径。

## 解决方案

落账前把本次 revision 触及的转换路径在 verify-workflow.mjs 里对照一遍：

- op 推进：核对 L275-282 相等字段清单（task_id/kind/subject + 条件字段），只改允许集合内的字段；
- 任务状态转换：grep 对应状态的 requireValue（如 done→evidence）；
- 回绑引用：先 `print(sorted(wf.keys()))` 确认字段层級，不凭记忆写路径。

每次落账后立即跑双校验器（`node verify-run.mjs <checkpoint>` + `node verify-workflow.mjs <workflow> [prev]`），错误在最小修订窗口内定位；CLI 吞真实报错，诊断用 `node --input-type=module -e "import {verifyWorkflow} from …; try/catch console.error(e.message)"`。
