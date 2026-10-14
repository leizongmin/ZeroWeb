# 网站优化任务、恢复账本与交付检查

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位已有网站任务的规则、账本与工具实现。入口为 [.agents/skills/zeroweb-site-optimizer](../../../../zeroweb-site-optimizer/SKILL.md)，本文只是工程导航，不授予启动、恢复、发布或合并权限。

## Gotchas
- `recover.mjs` 从原 workflow/contract/checkpoint 生成导航，不创建新状态；revision>1 缺 previous 时转 repair_records，历史 required_reads 摘要不能替代本上下文读取原规则。 (`.agents/skills/zeroweb-site-optimizer/scripts/recover.mjs`, `.agents/skills/zeroweb-site-optimizer/references/recovery.md`)
- 交付校验只在当前 candidate 与 review 身份匹配时调用 verifyReview；创作阶段未绑定 delivery.review 时，其他检查通过不代表审查回执已被完整校验。 (`.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`)
- 首轮必须两个非实施者的新上下文、同一 base/head 串行只读，作者修复只能在两份首轮归档之后；角色名字不同不证明输入隔离。 (`.agents/skills/zeroweb-site-optimizer/references/independent-review.md`)
- `checkBody` 忽略注释和围栏代码中的标题；真实正文需五个非空章节，Draft 可写缺口但拒绝空模板与本地图片引用。 (`.agents/skills/zeroweb-site-optimizer/scripts/pr-body.mjs`, `.agents/skills/zeroweb-site-optimizer/scripts/pr-body.test.mjs`)
- 仅附件探活成功不等于 PR 呈现正确；paired 回执另绑定下载摘要、尺寸、实际呈现证据和前后版本，后图必须是当前 head。 (`.agents/skills/zeroweb-site-optimizer/scripts/pr-body.mjs`)
- 当前工具没有接入宿主拦截、压缩 hook 或 GitHub required check；本地校验可拒绝记录但不能阻止所有绕过，不能宣称它是强制执行系统。 (`.agents/skills/zeroweb-site-optimizer/references/recovery.md`, `.agents/skills/zeroweb-site-optimizer/references/github-delivery.md`)

## Architecture
- PR 身份统一为 repo/pr/base_branch/base_sha/head_sha，subject 绑定 candidate manifest 的 SHA-256；干净候选要求 source_sha=head、base_sha=base、dirty_patch=null，不能只绑定分支名。 (`.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`)
- 当前 review PASS 且无 open_findings、reviewer 非主控也非任何 implement 作者之后，仍须 presentation；auto_merge 的 done 再要求确认 merge 与包含该 merge 的 integration PASS。 (`.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`)
- 首轮、复核、汇总沿现有 review operations，补测试也计 implement；每个补丁须非作者复核，旧报告摘要和身份不可改绑。 (`.agents/skills/zeroweb-site-optimizer/references/independent-review.md`, `.agents/skills/zeroweb-site-optimizer/scripts/verify-review.mjs`)

## Decisions
- 历史交付仅在相邻快照兼容路径 allowLegacy 保留，缺新增字段不能拿来放行新交付；已 done 历史不补造新审查，未交付任务须补当前契约。 (`.agents/skills/zeroweb-site-optimizer/scripts/verify-delivery.mjs`, `.agents/skills/zeroweb-site-optimizer/references/independent-review.md`)
- PR 正文图片必须原生持久附件，当前实现网络错误可尝试 curl IPv4/普通两种传输，但仍要求 2xx/image；传输兜底不是证据或视觉标准放宽。 (`.agents/skills/zeroweb-site-optimizer/scripts/pr-body.mjs`, git:127320669e0ea77b2acba489a0333fd52d67d6c7, git:44d3f4dfa8a8ae39c6d487837034e9a984a1a551)

## Conventions
- pr-body、对应测试和 github-delivery 在三个 commits 共同更新，表明公开说明、机械门禁与负例测试共同承担契约；更改截图规则只改文字或只改校验器都会分叉。 (`.agents/skills/zeroweb-site-optimizer/scripts/pr-body.mjs`, `.agents/skills/zeroweb-site-optimizer/scripts/pr-body.test.mjs`, `.agents/skills/zeroweb-site-optimizer/references/github-delivery.md`, git:17ae00187442157e2bb93e39d842fd6118f0c48b, git:30b248d22d06f9ccb982d7938f08dd58a9b60101, git:921c79215330077314778963db12ee667ed4e8b0)
