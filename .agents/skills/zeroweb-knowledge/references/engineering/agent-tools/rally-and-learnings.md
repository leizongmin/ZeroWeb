# Rally、任务文档与经验维护

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- initialization_evidence: docs/engineering/learnings.md、self-evolution.md 在初始化核对时为工作区材料，已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交；不归属于上述旧源码基线。 -->

## Module Structure

覆盖 Rally wrapper/config、任务级授权与 learning 三层资产维护。本文只静态分析，不运行 cron、不通知、不改规则。知识节点与 learning 案例格式不同，不能把案例 frontmatter 搬入普通知识主题。

## Gotchas
- cron defaults 的 concurrency=skip 只避免同一 job 重叠，不证明不同 job 互斥或共享工作区安全；构建身份与单写者须按各任务契约单独核对。 (`docs/rally/jobs.yaml`)
- 外层 job 最长 18000s，内层 ralph.agent_timeout 必须不小于它；仅改 jobs timeout 不会延长更短的 Agent 内层截止。 (`docs/rally/jobs.yaml`, `docs/rally/config.yaml`)
- wrapper 注释仍列三个旧 job 和旧 agent 名，实际 jobs.yaml 已有更多任务且 args 明确指定 claude-glm；解释运行配置应读执行参数，不把注释当现行任务列表。 (`scripts/rally-cron.sh`, `docs/rally/jobs.yaml`)
- learnings-index 发现坏 header/日期布局时仍写 INDEX 后返回 1；脚本非零不代表工作区没有变化，不得拿部分生成索引视为验证通过。 (`scripts/gen-learnings-index.py`)
- normalize-learnings 与 migrate-learnings 是已执行的一次性迁移，不是日常重建入口；normalize 非 dry-run 会 git mv 和改 header，不能当只读格式检查使用。 (`scripts/normalize-learnings.py`, `docs/rally/learning-maintenance.md`)

## Architecture
- learning 的 date frontmatter 是日期真值，路径必须 `<分类>/<YYYY-MM>/<YYYY-MM-DD>-topic.md`，四类为 bugs/patterns/performance/platform；生成器拒绝 flat 布局与日期漂移。 (`scripts/gen-learnings-index.py`)
- 事实层案例、生成 INDEX 和 guidelines 方法论各有独立职责；guidelines 只保留跨案例不变式，不重复 exemplar 细节，按索引回读证据。 (`docs/rally/learning-maintenance.md`, `.agents/skills/zeroweb-guidelines/SKILL.md`)
- 每个 cron job prompt 带自己的范围、session_reuse 和 max_session_uses，goal wrapper 则通过 rally run 转发参数；不能从调度器启动推定每个任务都共享同一会话。 (`scripts/rally-zero-web.sh`, `docs/rally/jobs.yaml`)

## Decisions
- learning-maintenance 的直接维护授权只覆盖指定 guidelines 与手册资产，不外溢到 AGENTS/其他 Skills；普通任务读取此手册也不会获得该 cron 的通知或推送权限。 (`docs/rally/learning-maintenance.md`, `docs/rally/jobs.yaml`)
- guidelines 新增优先合并已有不变式，门槛为至少两篇独立记录或单篇极强规则，每轮最多新增一条且上限 300 行；已有不变式的新实例正常不改 Skill。 (`docs/rally/learning-maintenance.md`)

## Conventions
- INDEX 的七次近期更新主要是资产收录/索引重建，不证明算法实现改变；归档时保持事实记录并把新案例分配到月份目录，不能手工补生成索引掩盖路径错误。 (`scripts/gen-learnings-index.py`, `docs/learnings/INDEX.md`, git:321ecb5870a64e36a54a8527a031e5d95f8b22cb, git:70d840e3739af9ae21af81ffae030deb53230c9c, git:f0250eb9331b0690b4ed4521cf6a0eff88ad66dc)
