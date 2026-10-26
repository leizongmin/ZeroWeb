---
date: 2026-10-26
modules: .agents/skills/zeroweb-site-optimizer
---

# PR 审查表只能由 reviewSection 生成——呈现层逐字节契约与静默 return false 诊断

## 问题描述

发布 PR 正文时为可读性给独立审查表加了行内修饰（`（DF2-1 返修面，h3）`、
`（被漂移复核覆盖）`）并漏了 1 行历史汇总——delivery-check ready 长期报
「Delivery not ready」却无具体行号。同轮另一处：review 类操作的 result 挂了
markdown 作废回执而非报告 JSON，使 reviewSection 对**所有** completed review
操作的无容差 JSON 解析永久抛错，presented=true 的任何状态从此校验失败，
任务无法进入 merging，最终只能换代（gen2→gen3）解除。

## 根因分析

verifyPresentation 要求 PR 正文**逐字节包含** reviewSection(state) 的当前输出
（`visible.includes(section)`）：人工润饰行内后缀、增删行、调整顺序都会使
includes 为 false；该检查点失败路径是**静默 return false**，不抛错、不带原因，
CLI 层只打印 Usage——两层叠加导致「就绪门禁假败但所有探针均绿」。reviewSection
则把每个 completed review 操作的 result 无条件当 JSON 解析，非 JSON 回执等于
在呈现层埋了永久地雷。

## 解决方案

- 正文审查表只允许两种来源：reviewSection 的直接输出，或「生成器输出 + 原样
  拼接」；补充叙事写正文其他小节，绝不改表格行。
- review 类操作 result 必须是可解析的报告 JSON；作废/失败记录挂 implement 或
  纯记录类操作，不占 review 位。
- 诊断静默 return false 时不依赖 CLI 报错，直接 `node --input-type=module -e`
  逐个调用 reviewSection / verifyPresentation / verifyDelivery 并自检
  includes 与各子条件布尔值，把假项逐项打印出来。

避免方式：派发汇总操作前先核对聚合的每份报告已有先行 review 操作入账；
发布前在本地跑一次「生成器输出 vs 正文段落」的 includes 预检，不等门禁爆。
