# Agent 工作流与证据工具

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖工程 Agent 的恢复、证据、审查与维护工具；仅按需求导航原 Skill/契约，不在知识入口启动流程。网站任务、Chrome parity、低频巡检、Rally job 各有独立授权范围。

## Gotchas
- 从旧 workflow 生成 required_reads 仅提示当前规则摘要，不证明读过正文或仍有授权；恢复不能改 run ID 重置预算与停止屏障。 (`.agents/skills/zeroweb-site-optimizer/scripts/recover.mjs`, `.agents/skills/zeroweb-site-optimizer/references/recovery.md`)
- 呈现摘要、报告哈希和 JSON 字段健康只证明记录一致；独立输入、真实窗口、图片内容和远端保护仍需实际核验。 (`.agents/skills/zeroweb-site-optimizer/references/independent-review.md`, `.agents/skills/zeroweb-browser-chrome-parity/references/evidence-contract.md`)

## Architecture
- 网站 workflow/candidate/review/presentation 与巡检 success 都绑定各自版本和证据摘要，但两种状态不互相放行；巡检同 SHA 缓存还要求环境、场景和策略一致。 (`.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`, `.agents/skills/zeroweb-regression-guard/references/guard-policy.md`)
- learning 案例与生成索引为事实资产，guidelines 是跨案例规则；job 常设维护只适用于指定目标，不自动授权其他规则或其他任务的外部操作。 (`docs/rally/learning-maintenance.md`, `docs/rally/jobs.yaml`)

## Child Knowledge Nodes
- [网站优化任务、恢复账本与交付检查](site-optimizer.md) — 查旧任务恢复、双首轮、补丁复核、PR 正文/附件身份与旧回执兼容。
- [Chrome 一致性、巡检与证据契约](parity-and-regression.md) — 查 GUI/headless 分支、状态/事件/几何/像素门禁和内存峰值口径。
- [Rally、任务文档与经验维护](rally-and-learnings.md) — 查配置实际参数、内外层时限、同 job 并发及 learning 生成/授权。
