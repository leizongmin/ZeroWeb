---
date: 2026-10-09
modules: .agents/skills/zeroweb-site-optimizer
---

# 校验器惰性实跑：角色报告在首次绑定时才被校验

## 问题描述

网站任务验收校验器（`verify-run.mjs` 系列）按依赖图惰性求值：`verifyReview` 只在 `delivery.review` 被绑定时才运行。slice48 运行中，双首轮、双 recheck、汇总共五份角色 JSON 产出后从未被校验器读过——直到把汇总回执绑进 `delivery.review` 的那一刻，此前所有漂移（6 处 stale artifact 哈希、15 处裸文件名路径）一次性爆出，且此时距最早一份产出已过数轮落账。

## 根因分析

1. **校验时机与创作时机脱钩**：校验器只在交付状态机的绑定点检查引用完整性，创作期没有任何强制实跑；产物在「无人检查的窗口」里持续漂移。
2. **CLI 包装器吞错**：`node verify-xxx.mjs` 形式的命令行调用失败时只给一行笼统错误，真实异常（哪个字段、哪条证据）被吞掉，增加诊断成本。

## 解决方案

1. **创作期即实跑**：每份 role JSON / 回执写盘后立刻用对应校验器实跑一遍（`verify-review.mjs`、`verify-delivery.mjs` 等），不等绑定时刻；漂移在最小修订窗口内修，成本最低。
2. **诊断走模块导入**：需要真实报错时绕过 CLI 包装器：
   `node --input-type=module -e "import {verifyReview} from './verify-review.mjs'; …try/catch console.error(e.message, e.stack)"`
3. **落账前把整条依赖链跑一遍校验器**作为四步落账纪律的一部分（改→json.dump→双校验器→snapshot），确保进入快照的状态全部被验证过。
