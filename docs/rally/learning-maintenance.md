# Learning 资产每日维护指南（rally cronjob）

> 本文档是 `learning-maintenance` cronjob 的唯一操作手册。cronjob 的 prompt 只负责引导 agent
> 阅读本文档并执行，具体规则全部在这里维护（改这里不需要改 jobs.yaml）。

## 资产结构

```
docs/learnings/<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md    # Layer 3：单条踩坑/经验记录（事实层）
docs/learnings/INDEX.md                                    # Layer 2：脚本生成的索引（勿手改）
.agents/skills/zeroweb-guidelines/SKILL.md                 # Layer 1：方法论蒸馏层（cron 直接维护，已获常设授权）
scripts/gen-learnings-index.py                             # Layer 2 生成器（含布局/日期一致性校验）
scripts/normalize-learnings.py                             # 一次性迁移脚本（已执行，留档）
scripts/migrate-learnings-layout.py                        # 一次性布局迁移脚本（已执行，留档）
```

**learning 文件格式契约**（新增时必须遵守，否则索引脚本校验失败）：

- 路径：`docs/learnings/<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md`；**frontmatter date 是事实源**，
  文件名日期前缀与月度目录必须由它派生（`make learnings-index` 不一致即报错）
- topic 用 kebab-case 英文；分类仅四种：`bugs` / `patterns` / `performance` / `platform`
- 文件头部必须是 YAML frontmatter，date 必填、modules 可空：

  ```
  ---
  date: 2026-08-18
  modules: crates/css-parser/src/values
  ---

  # 标题

  ## 问题描述 / 根因分析 / 解决方案
  ```

## 每日维护流程

### 第 0 步：前置检查

1. `git pull --rebase origin main`，确认工作区无未提交变更。
2. 找出自上次维护以来的新增/修改 learning：
   `git log --since="2 days ago" --name-only --pretty=format: -- docs/learnings/ | sort -u | grep '\.md$' | grep -v INDEX.md`
   （配合 INDEX.md 当前条目比对；空则直接输出 DONE 退出——**no-op 是合法且常见的成功结局**。）

### 第 1 步：校验格式（自动可修）

对每个新增文件检查 frontmatter 契约（date 存在、路径无日期后缀）。轻微问题（如标题缺失）直接
修复并计入提交；格式不符的新文件**不得重写内容**，只修 header。

### 第 2 步：重建索引（自动）

```
make learnings-index
```

INDEX.md 是生成物，永远整文件重新生成，不做手工增量编辑。

### 第 3 步：评估并维护 skill（判断性，直接修改）

> 授权说明：用户于 2026-08-18 常设授权本 cronjob **直接修改**
> `.agents/skills/zeroweb-guidelines/SKILL.md`，无需走 evolution-drafts 草案审批。
> 该授权仅覆盖此文件与 INDEX.md，不外溢到其他 SKILL.md 或 AGENTS.md/TOOLS.md/MEMORY.md。

阅读新增 learning（通常每天 0–3 篇），对照 SKILL.md 现有条目判断：

**已有不变式的新实例**：新 learning 只是某条现有不变式在新子系统的印证 → **skill 不动**。
exemplar 不保留在 skill 里（规则↔证据的映射靠 INDEX.md 按子系统检索），这是设计决定。

**新增条目**（严格门槛）：必须同时满足——
- 与现有 22 条不变式不同类（不是任何一条的新实例）；
- 被此次新增的 ≥2 篇独立记录印证，或单篇但规则强度极高（架构级约束、必然复现的错误模式）；
- 一句祈使句能说清规则与失效模式（「为什么」段承载因果，不靠举例）；
- 同步注明让步类别：属于信任边界/防数据丢失（进「让步边界」的不可放宽清单，
  参照 SKILL.md「与『简单至上』的关系」节）还是可在原型阶段放宽的验证类不变式。

**不满足门槛的新坑**：留在 Layer 3 即可，INDEX.md 已收录。skill 不是 learning 的目录，
收录不足 2 篇印证的条目会让它退化成摘要集。

**硬性防膨胀约束**：
- SKILL.md 上限 300 行；新增前先看能否并入现有条目（扩充某条的「为什么」）；
- 一次 cron 运行最多新增 1 条不变式；
- 绝大多数运行应该是零 skill 变更——skill 接近 write-once，只有新错误类别才值得进。

产出方式：直接编辑 SKILL.md，提交信息中注明依据（哪些新 learning 印证了变更）。

### 第 4 步：提交

- 提交范围：INDEX.md 重建 + learning header 修复 + SKILL.md 维护（如有）。
- 全部是 `.md` 文件时按 AGENTS.md 豁免项目代码检查，
  但仍须：`git diff --check` + `lei-pre-commit-guard` PASS。
- 提交信息：`docs(learnings): daily index rebuild + N new entries`（有 skill 变更时追加
  `+ skill update` 并在 body 说明依据）。
- **提交并推送后发飞书通知**（按 docs/rally/run-rules.md 第 7 条的命令，`--as bot`），
  消息说明：本次新增 learning 数、索引是否重建、skill 是否变更及依据（零变更时仅一句「无新增，
  no-op」）。通知仅为告知，发送失败不阻塞流程，在输出中注明即可。
- 无任何变更（含无新增 learning）→ 不提交，但仍发一条飞书简报（no-op）后输出 DONE。

### 异常处理

- 索引脚本报 `BAD frontmatter`：修复对应文件 header 后重跑；无法机械修复的输出报告等待人工处理。
- 与本流程冲突的仓库规则（run-rules.md、AGENTS.md）以更严格者为准。
