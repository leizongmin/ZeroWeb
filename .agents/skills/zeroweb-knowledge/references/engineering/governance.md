# 文档、许可证、贡献与证据来源边界

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- initialization_evidence: README.md 与 docs/engineering/agents-maintenance.md 在初始化核对时为工作区材料，已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交；不归属于上述旧源码基线。 -->

## Module Structure

覆盖规范文档、目标状态、贡献责任和来源/许可证边界。治理政策在 `docs/governance/contribution-responsibility.md`，代码路由在 CODEOWNERS，验证结果需回读具体证据；知识维护不修改这些源规则。

## Gotchas
- 当前 G0 是单人责任维护，不具备双人批准条件；AI/bot 辅助审阅不能成为 owner、RM 或平台批准票，也不能拿网站任务独立 Agent 双审查替代人类 adoption。 (`docs/governance/contribution-responsibility.md`)
- CODEOWNERS 只是评审路由，作者不自动获得所有权；RM 对最终 commit SHA 接纳，后续 push 使旧 adoption 失效。 (`.github/CODEOWNERS`, `docs/governance/contribution-responsibility.md`)
- 风险等级取路径下限与语义风险的较高者，Oracle、分母、baseline、skip、质量数字至少 C2；文件在 tests/docs 中不意味着只是低风险文字或测试改动。 (`docs/governance/contribution-responsibility.md`)
- goal 入口定义 Mission/DC，master 才是持续状态面；每轮重写入口或只改顶部状态会留下历史验证基线矛盾，归档前必须核对所有章节。 (`docs/goal/README.md`)
- 单项 goal 的 DONE 是其 Support Envelope/DC 终态，不是整个浏览器可生产使用的证明；项目 README 和贡献说明均保留实验用途与自评生产风险边界。 (`README.md`, `CONTRIBUTING.md`)

## Architecture
- 跨两个责任域或命中 C2 先关联 accepted proposal，C2 证据包包括最终 diff 的独立二次审阅、相关门禁与回滚方案；C3 只走私密安全通道。 (`CONTRIBUTING.md`, `docs/governance/contribution-responsibility.md`, `.github/pull_request_template.md`)
- G0 维护者自提交用最终 SHA adoption 记录，不是 GitHub 自批准；G1 需两个人类合并权限、关键域 fallback、阻止单人合入的门禁及已验回滚路径，不能仅通过改成熟度标记激活。 (`docs/governance/contribution-responsibility.md`)
- goal 归档有整树移入总 archive 和仅入口自归档两模式；后者保留 master/evidence 原位置，仍须顶层索引登记及完成状态，不能按根目录残留子目录判断仍 Active。 (`docs/goal/README.md`)

## Decisions
- MIT 许可允许分发但要求保留版权和许可声明，不提供担保；项目自身 MIT 不能替第三方素材或依赖完成许可审查，新增/升级依赖仍至少 C2。 (`LICENSE`, `CONTRIBUTING.md`, `docs/governance/contribution-responsibility.md`)
- owner 任命依赖已证明的不变量、验证/回滚能力与治理 PR，不由提交数或 AI 产出规模自动获得；不可用时 fallback 接管且可冻结新功能。 (`docs/governance/contribution-responsibility.md`)

## Conventions
- PR 模板要求 AI 参与、人工验证、未运行检查及许可证影响分别声明，不需要附完整对话；原始需求、实施事实和审阅结论不能合成未经证实的“全部通过”。 (`.github/pull_request_template.md`, `CONTRIBUTING.md`)
- 文档来源区分现行规范/架构、目标控制面、历史 archive 和研究材料；当前行为需回查源码，研究建议与旧 roadmap 不自动升级为实现事实。 (`docs/architecture.md`, `docs/goal/README.md`, `docs/governance/contribution-responsibility.md`)
