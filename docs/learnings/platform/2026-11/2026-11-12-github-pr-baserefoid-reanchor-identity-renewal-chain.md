---
date: 2026-11-12
modules: .agents/skills/zeroweb-site-optimizer
---

# baseRefOid 重锚后的全链身份续期序列——六处绑定缺一即门禁假败

## 问题描述

PR base 锚一旦被 GitHub 重锚（粘滞锚语义见
[2026-11-08 baseRefOid 锚漂移](../2026-11/2026-11-08-github-pr-base-ref-oid-anchor-drift.md)：
仅「main 前进 + PR 侧事件」组合触发），以旧 baseRefOid 为身份键的账本资产**同时**
失效：manifest base_sha、RC/SUM 摘要六键、gate subject 绑定、CP 锚、PR 正文内嵌
sha、presentation 的 base_sha 与 review_sha256。T28/T27/T42/T49/T61/T62 六轮先例
反复证明：只换绑其中一处，其余处保持旧 sha，交付门禁与独立复核会以「身份不匹配」
假败，且失败点分散在各检查器，难一眼归因到重锚。

## 根因分析

身份键是**逐字段落盘**的（manifest、run 状态、gate subject、presentation 回执各自
独立存储），没有任何级联更新机制；重锚是一个外部事件，账本侧必须显式走完全部绑定点。
漏更新不会被立即报错——它在后续某个含 sha 比对的检查点（publish/ready/merge 回执）
才爆，且报错文案不带「这是重锚导致」的上下文。

## 解决方案

沿 T28/T27/T42/T49/T61/T62 走通的全链续期序列，顺序执行、逐处核对：

1. manifest base_sha 换绑到新 baseRefOid；
2. RC/SUM 六键续期（以新锚重算三点 diff sha256 后回写）；
3. gate subject 换绑（审查门禁记录的 subject 五键同步）；
4. CP（检查点）锚更新；
5. PR body 重渲染（正文内嵌 sha 与新锚一致）；
6. presentation 的 base_sha + review_sha256 双换绑；
7. 置 ready；发布后再做一次粘滞复核（ready 后仍可能再次重锚，合并前重探
   baseRefOid 现值）。

身份复核一律用三点 diff 字节恒等（`git diff A...head`），不用两点——锚选择下三点
diff 逐字节恒等，这是判断「续期是否只是换绑而非内容变化」的判据。
