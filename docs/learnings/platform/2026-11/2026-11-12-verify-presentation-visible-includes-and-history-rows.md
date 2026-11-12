---
date: 2026-11-12
modules: .agents/skills/zeroweb-site-optimizer
---

# verifyPresentation 的 visible 归一化包含校验与历史报告行渲染规则

## 问题描述

T62 交付中 PR 正文从 v6 改到 v8，手写润饰审查表行（措辞、行内备注）导致
verifyPresentation 的包含校验失败；另一坑：历史轮次的报告行（head/base 与当前
版本不匹配的旧 review 记录）如何进正文才不破坏校验，没有现成口径。基础契约
（审查表只能由 reviewSection 生成、静默 return false 诊断法）已在
[2026-10-26 reviewSection 契约](../../bugs/2026-10/2026-10-26-pr-review-section-generated-contract.md)
沉淀，本篇补 T62 实证出的两条细则。

## 根因分析

verifyPresentation 对 PR 正文做的是**归一化后逐字包含**校验：先从正文剥掉注释
与代码块得到 visible 文本，再要求 `visible.includes(reviewSection(state))`。
两处推论常被忽视：其一，凡落在 visible 文本里的表格区域，任何手写增删/改序/
行内修饰必漂移——生成器输出是唯一合法行来源；其二，历史报告行也必须由生成器
按规则渲染，不能人工"整理"成更可读的样子，否则 same 失败。

## 解决方案

- 审查表从 state 经 reviewSection 生成后**逐字嵌入**；叙事放其他小节，永不碰
  表格行。
- 历史报告行交给生成器的渲染规则处理：head/base 与当前版本不匹配的行由生成器
  标注为历史（head/base 不匹配标历史），证据保留但不冒充当前审查。
- 发布前在本地用与校验器相同的 visible 口径（去注释/代码块）做一次 includes
  预检，不等门禁爆才定位。
