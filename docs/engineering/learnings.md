# 技术经验沉淀

排查、修复或开发中发现可复用技术经验时必须读取本文，并按以下流程保存到 `docs/learnings/`。

## 触发范围

需要沉淀：深入排查才定位的 bug 根因与修复、平台环境坑点、可复用代码模式、有效性能优化，以及工具链/构建系统技巧。

不需要沉淀：简单拼写/语法修复、只服务一次任务的临时方案。Agent 行为规则、协作方式和偏好走 [自进化流程](self-evolution.md)，不混入技术案例。

## 格式与位置

每条经验一个 Markdown 文件：

- `bugs`：踩坑根因、修复与避免方式。
- `patterns`：代码模式和最佳实践。
- `platform`：平台经验。
- `performance`：性能优化。

路径必须为 `docs/learnings/<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md`。frontmatter 的 `date` 是事实源，文件名日期与月目录由它派生；topic 使用 kebab-case 英文。文件头为 YAML frontmatter，`date: YYYY-MM-DD` 必填，`modules: <逗号分隔>` 可空，随后 `# 标题`。正文包含问题描述、根因分析和解决方案。

## 三层维护

- 单条记录落地后不手工维护索引。每日 `docs/rally/jobs.yaml` 的 `learning-maintenance` 按 [维护手册](../rally/learning-maintenance.md) 校验格式，由 `make learnings-index` 重建 `docs/learnings/INDEX.md`；索引是生成物，勿手改。
- 方法论层 [.agents/skills/zeroweb-guidelines/SKILL.md](../../.agents/skills/zeroweb-guidelines/SKILL.md) 由该 cronjob 直接维护。用户常设授权仅覆盖此 Skill 文件与手册规定的资产，不外溢到其他规则；防膨胀限制以原手册为准。
- 编写、审查 Rust 代码时对照 `zeroweb-guidelines` 不变式，并在索引中按子系统检索相关 learning 原文。

知识库用于提炼影响定位和修改决策的结论，案例保留在这里，不复制全文。是否维护知识库遵循其实际授权；不能仅因库存在自行回写。
