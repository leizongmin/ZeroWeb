---
date: 2026-11-08
modules: .agents/skills/zeroweb-site-optimizer
---

# 完成态 op 绑定的证据文件字节冻结与 open_findings 非空即不就绪

## 问题描述

T49 交付期连续两次被 verify 系列校验器拦截，均为「绑定后的文件还想改」与「字段语义按直觉理解」：

1. **返修节追加进已完成 op 绑定的证据文件**：fix round 把返修记录追加到 `t49-task-evidence.md`，而该文件已在 rev714 被 completed implement op 以 sha256 引用。下一 rev 校验即 `Evidence hash mismatch`（verify-workflow 会 load(prev) 并重探其全部 op result evidence）。首次误判为可原地恢复：控制器按 `head -n 49` 前缀重建自验 sha，结果 45cfeaa6 ≠ a49637a2——序列化细节（行尾、缩进、字段顺序）无法靠人肉拼回，最后由实施代理以原字节副本精确还原，返修内容拆到新文件 `t49-task-evidence-r2.md`。
2. **合并回执在 op 绑定后补字段**：merge receipt 先绑定（rev733 completed），随后补 `artifacts`/`subject` 字段通过 rev734 的 `Invalid delivery receipt` 校验，但 rev733 的冻结引用随即失配，整链回滚重放。
3. **summary 把落池 informational 项记进 `open_findings`**：RC-1 已被处置文件裁 accepted_deferred（落池、无需本 PR 行动），summary 照字面记 `open_findings: ['RC-1']`，verify-delivery 的 `reviewed` 判定硬性要求 `open_findings.length === 0`，delivery-check ready 拒绝，又是一轮回滚重放。

## 根因分析

两类同源：校验器把「op 一旦 completed，其 result.path 指向的文件就是冻结快照」作为不变式执行，而操作直觉把运行目录当普通工作区；`open_findings` 的语义是「合并前需要行动的未决项」而非「审查发现的所有未闭环清单」，informational 落池项的归宿是处置文件与池，不是 open 位。字段契约不存在于任何历史实例的形状里（每个任务踩的路径不同），只存在于 verify-delivery.mjs/verify-review.mjs 的判定代码里。

## 解决方案

- **绑定前**：新回执/报告落盘时先对照 verify 系列的字段清单（receipt 类必须 `schema_version`/`task_id`/`artifacts` 非空数组；summary 的 `open_findings` 只收需行动项）再交 op 绑定；拿不准就先跑一次 delivery-check，把失败留在绑定前。
- **绑定后**：已完成 op 引用的文件一律不改；续写内容拆新文件由新 op 引用。需要修正绑定内容时只有一条路：存证 `-invalid-<reason>` 快照 → 回滚到 intent 态（引用解冻）→ 修文件 → 同 rev 重放。
- **绝不要手拼冻结文件的原字节**：哈希不匹配时找原副本或 git 历史，不要用前缀重建自验。
