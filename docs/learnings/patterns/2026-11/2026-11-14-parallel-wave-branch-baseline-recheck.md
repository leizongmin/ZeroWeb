---
date: 2026-11-14
modules: site_optimizer
---

# 切任务分支前先 fetch 定 tip——并行波次会翻动冲突面与门禁基线

## 问题描述

T70 修复切片注册时，任务 description/intent note 按上一任务的认知把切出点写成
`a1caf8509`；实际 `git fetch` 后 origin/main 已被并行波次推进到 `266b45602`
（M3-A fix(engine)＋B1 切片＋多 docs 提交）。两个实质影响：M3-A 恰好改了本任务
要动的 part04 R140 邻区（冲突检查等级失效）；新增五形钉测翻动 roster P 账
（3944P/7F、reftest 725/725、renderer 383P/1F 全部不可再引用）。首放契约失实，
走了一轮存证-回滚-重放（`-invalid-branch-base` 存证 ×2）才修正。

## 根因分析

并行运行下 origin/main 是移动目标，而任务契约（manifest base_sha、冲突检查
范围、门禁基线数字）在**切分支动作之前**就被写死。时序倒置：先落账、后切分支、
再发现 tip 变了——契约里的身份字段从落账那一刻起就是猜测值。

## 解决方案

固化顺序：`git fetch origin && git rev-parse origin/main` 确认 tip → 按**实际
tip** 定稿任务契约（切出点、冲突检查等级、基线数字「以切出点实算为准」的表述）
→ 落账注册 → 再切分支。三件事必须随切出点重算，不沿用上一任务的数字：
①冲突检查等级（邻区有提交就升级）；②门禁基线（新钉测即翻动 P 账/红名册）；
③manifest.base_sha（=实际切出点）。基线数字在契约里写「以切出点基线轮实算，
不引用旧数」，把实算责任留给执行轮而非预设值。
