# 阶段 PR、自审与自动合并

启动合约包含 GitHub 交付时，在前置能力预检及每项任务交付时读取；仅讨论或编辑 skill
不启动网站优化。先按 [启动授权](intake.md) 收齐权限与工具缺口处理策略，交付阶段
复用原授权，不重新询问 commit/push/PR/脱敏截图许可。
参考 ZeroSeed 的原生附件流程，使用 ZeroWeb 既有 GUI/CDP 与生产帧证据，不引入
REPL/tmux、另一套截图采集器或新的视觉阈值。

## 证据选择与授权

PR description 是评审入口：按问题 ID/任务场景说明原行为、新行为、实际收益和仍存
差异，并用 [模板](../templates/github-pr.md) 并排嵌入优化前/后截图。评论、附件链接
清单或本地报告不能代替正文的图片展示；Chrome/Firefox 图可以作为第三列参考，
不能代替 ZeroWeb 自身的前后对照。

- 前图绑定 original 或候选的 parent best，后图绑定已验证 best；写明源码/构建身份、
  URL、捕获时间、任务状态、viewport、DPR、缩放及字体等条件。不得把 rejected 或
  inconclusive 的最后试验当成优化成果；诊断图如需展示须独立标注结论边界。
- 浏览器 UI 使用实际壳层窗口图；页面兼容性使用既有生产帧路径。固定相同检查点与
  区域，可附局部放大图，但同时提供对应全图附件。fixture 与真实网站分开标注，
  不能拿 headless/fixture 图冒充真实产品效果；一张静态图也不能证明交互或性能收益。
- 缺少改前图只能从精确旧版本、同条件且在授权/剩余预算内补采；线上内容变化导致
  不可比时如实注明。禁止重绘、AI 生成、修改页面或借用别的版本凑出“改前”截图。
- 创建/更新 PR 的授权包含经逐张查看、脱敏且仅展示本次差异的前后截图，无需另问
  同一权限。发布前检查账号、私有正文、地址栏参数、其他窗口内容和图片元数据；
  裁剪或遮罩不能掩盖待评价缺陷。无法安全脱敏则保留本地，不公开原图。
- 上述授权不包含日志、HAR、profile、cookie、token 或其他私有证据。用户明确禁止
  截图时使用文字并注明限制；只有本地交付权限时不创建 PR 或上传附件。

本地原证据和上传副本保留在 gitignored 的 `.acceptance/site-optimizer/<run-id>/`。
在 run.md 记录每张图的场景、角色（before/after/reference）、版本、本地相对路径、
尺寸、SHA-256、脱敏范围及最终附件 URL。PR 正文只公开必要的脱敏版本信息。

## 上传与回读核验

1. 在承诺图片交付前查看实际 `gh pr create/edit --help` 是否支持 `--attach`，核对
   目标仓库与 push 权限。支持时使用官方原生附件功能；只传 `--body-file` 不会上传
   本地图片。正文中的本地图片路径须与 `--attach` 参数一致，CLI 才能原位替换链接。
2. 不支持时可使用已授权且已登录的浏览器，通过 GitHub 正文编辑器上传附件，再将
   返回的 Markdown 放入 description。工具升级/登录仅在既有授权内进行；不擅自
   替换全局工具，不复制会话 cookie，不调用未公开上传接口。
3. 正常 push 后按准确 repo/base/head 查重，创建或更新对应 PR。以下为已确认支持
   `--attach` 的 edit 示例；创建时同样传附件并明确 base/head。所有变量均来自本次
   交付记录，BEFORE/AFTER 为已审查的本地图片，命令受宿主墙钟超时约束：

   ```bash
   gh pr edit "$PR_NUMBER" --repo "$REPO" --body-file "$BODY_FILE" \
     --attach "$BEFORE" --attach "$AFTER"
   ```

4. 创建/更新后回读 PR，核对 head SHA、base/head、最终正文与预期附件数量，确认
   每组 before/after 的图片引用已替换为 GitHub 返回的持久附件 URL。不得拼造 URL，
   不使用本地路径、临时签名 URL 或仓库 blob/raw 链接替代。
   正文中的图片引用由检查器机械校验：`delivery-check.mjs draft` 即拒绝非附件
   URL 的 `![…]` 引用（含本地/相对路径，2026-10-09 PR #117 对比表漏替换实测），
   ready 阶段再对每个附件 URL 逐个探活（2xx + 图片魔数/类型），不可达即拒绝。
5. 按目标权限下载附件到私有临时目录，核对内容类型、尺寸和 SHA-256；只访问核对过
   的 GitHub 附件域名，不向任意重定向目标转发认证头。实际查看 PR 页面中图片是否
   加载、前后是否对应。仅下载成功时写“附件内容已核验、PR 呈现未验证”。

`--attach` 与 Markdown 原位替换的官方说明：
[GitHub CLI 附件文档](https://docs.github.com/en/github-cli/github-cli/attaching-files-with-github-cli)。
运行时以本机 help 为准，不把文档能力等同于已安装版本能力。

## 失败、存储与收尾

- 上传部分成功、超时或非零退出时，先读取远端正文核对实际结果，保留成功 URL，
  仅重试确认缺失的附件。不要用旧 body-file 覆盖已有附件链接或其他人的正文修改。
- 没有上传/呈现能力、缺少必需前后图或证据不可比时，在 run.md 和 PR 中明确记录
  “截图交付 blocked/未验证”、原因与下一步，保留 Draft 或未就绪状态；不默默删掉
  截图要求，不重跑已完成的产品优化，也不以 PR URL 或退出码声称交付完成。
- 仅 skill/文档变更或确无适用产品画面时，正文写截图“不适用”及原因；纯性能收益
  以真实测量表为主，画面不变就注明，不用两张相同图声称性能改善。
- 附件交付状态与实验结论分开记录。`verify-run.mjs` 的 ready 仅校验已有门禁记录，
  不证明 GitHub 上传或图片呈现完成，不为此给 checkpoint schema 添加自创字段。
- 提交前检查 staged、推送前检查明确 base 到 head 的完整 diff：评审截图不得进入
  Git，也不能换目录、LFS、资产分支、Release 或第三方图床绕过。真正被测试依赖的
  常驻基准图不属于评审附件，不因此删除；不顺手清理历史资产或重写 Git 历史。
- PR 正文及附件映射保留本地备份；若使用用户明确要求的 worktree，获准清理前先归档
  仍需保留的证据并核验。默认保留当前工作区与构建缓存。
  图片交付完成不改变原实验停止原因，也不授权自动合并或发布。

## 默认的任务 PR 与集成循环

默认按一个可独立验收的切片创建任务 PR，不等全站目标完成才交付，也不每个候选都
创建 PR。沿用已明确的整轮自动合并授权；仅本地或 PR-only 限制优先。
总时间和 PR 迭代次数遵循启动合约及原账本；默认总时间见入口，暂停/等待也计时。
一个 PR 计一次迭代，创建经服务端确认即记入 workflow；同 PR 返修重审不增次，
合并后另开 PR 另计。创建前核对余额，未知创建结果先查询，不重复创建或漏记；
上限到达仍可完成已有 PR，但不能开启需要新 PR 的工作。
单步超时、独立审查和停止屏障始终有效。

1. 启动冻结真实 repo/remote、integration 分支、最终交付位置、合并方式和权限来源；
   主控持有集成权限，worker 只能交付自己的任务分支。默认串行复用当前工作区及
   构建缓存；新分支按入口从最新 main（或明确约定的 integration 分支）创建，
   记录 fetch 得到的精确 SHA。工作区占用、未提交改动或分支冲突时保留现场，
   不自动 stash、覆盖或另建 worktree；只有用户明确要求才创建 worktree。
2. 未发布分支提交后 rebase 最新 integration 并验证；已发布分支用普通 merge
   集成更新，不 force push。以精确 head/base 保存测试、原站前后证据和独立 review。
3. PR 创建后按 [双审查](independent-review.md) 串行派发测试有效性与缺陷两个
   新上下文，首轮只读同一精确 base/head 的完整变更，不继承作者推理、不互看结论。
   两份首轮归档后安排同一 PR 返修；复用仍有效检查，由非作者复核新增补丁和受影响
   行为，生成当前 head 的汇总回执。不得删掉发现，或把任一首轮 PASS 当最终批准。
   首轮 reviewer 不改产品，补丁另作实施任务；不代替 GitHub 批准或自行合并。
   必需双审查证据、独立上下文或真实 CI 缺口阻塞交付就绪；可继续不依赖该交付的
   已授权工作。pr_only 同样完成适用双审查，但不执行合并。
4. 合并前重查远端 head/base、CI、保护规则、可合并状态、停止屏障与本轮授权。
   按 workflow 核对 candidate_manifest 的 source_sha/base_sha/dirty_patch、review.subject
   及 checkpoint 当前门禁身份；新 head 的审查不能代替新 head 的实际验证。
   主控实际读取汇总 artifacts 中两份首轮、发现处置及非作者复核；检查器核对角色、
   completed operation、版本和非作者关系，实际隔离与报告语义仍须核验。
   版本变化使受影响验证/review 失效。必须通过仓库门禁与原生保护，不以管理员绕过、
   review 文本或本地脚本 PASS 代替。审批/评论等对他人的消息不从 merge 权限推导。
5. 主控在运行目录保存不可变 merge 意图（唯一 operation ID、repo/PR、head/base），
   再执行获准的普通合并。未知结果先查询服务端，核验 merge commit、目标分支及包含
   关系后保存结果；没有确认前不重试合并、不派发依赖任务。
   合并确认后删除该 PR 的远端 head 分支（`git push origin --delete <branch>`），
   避免任务分支长期累积；同分支仍有开启 PR（串行 PR 复用运行分支）时保留并在
   merge 回执记 branch_kept_reason，待该分支最后一个 PR 合并时删除。回执记录
   head_branch 与 branch_deleted；删除失败如实记录，不静默跳过。
6. 确认本任务工作已提交、工作区干净且没有在途 worker 后，在同一工作区 fetch，
   切回约定的集成分支（默认 main）并仅快进到最新远端 SHA；本地分支不存在则从该
   SHA 创建。分叉、被其他 worktree 占用或出现他人改动时保留现场并报告阻塞，
   不强制重置或切换。复用构建缓存完成集成 smoke 和受影响用户任务验证后才能将
   对应 workflow task 标 done。以集成产物建立新 best，旧证据保持原身份；集成失败
   保留 integrating，按 workflow 新建 repairs_task_id 返修任务和新 PR，避免依赖死锁。
   验证通过后报告 PR、实际改善和剩余目标；下一任务开始前再次 fetch，基于最新
   集成 SHA 在同一工作区创建新的开发分支，立即派发，不等待用户说“继续”。
   最终仍须从最新 integration 做完整原站整体验收；失败则追加修复任务再循环。

GitHub 与 merge receipts 保存于私有运行目录，字段和机器检查统一见 workflow 的
“PR 证据与合并后返修”；operations 包含 publish/merge/integrate，合并意图先持久化。
检查器只核验回执结构与身份，主控恢复前必须核对未决操作与真实服务端状态。不能把远端能力缺失
变成再次启动已完成网站优化的理由。仅 push/PR 授权时止于对应交付，不进入本合并循环。

## 正文与审查可见性检查

发布/更新、派发审查、标记就绪与合并前，按 [恢复入口](recovery.md) 重新读取当前阶段原文。
首次创建 Draft 可以尚无审查和图片，但正文须如实说明缺口，不以空占位符发布。
先复制模板填写内容，运行检查后按既有 publish operation 发布：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/delivery-check.mjs draft "$BODY_FILE"
```

已创建 PR 后，从 completed review operations 生成状态表；也可以不传 BODY_FILE，
从模板生成新骨架。输出到新的本地文件，人工核对后使用，不直接覆盖远端正文：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/delivery-check.mjs \
  render "$RUN_DIR/workflow.json" "$TASK_ID" "$BODY_FILE"
```

状态表显示首轮、复核、汇总的真实 head/base、结论和报告 SHA-256；无报告显示待执行，
当前汇总缺失或过期时显示待执行，旧汇总标为历史报告，阻断项数量显示“未汇总”。
当前汇总存在时显示阻断项数量。默认只公开摘要和本地证据标识，不公开宿主 ID、路径或私有日志。
可以另附已经获准公开的持久报告链接；本地路径不能冒充远端可访问证据。
审查字段及报告格式见 [双审查](independent-review.md)。首轮输入仍须裁去已有审查结论。
新汇总落盘后，在 pr_review 登记 publish 更新正文；旧 presentation 此时仅表示过期，
不会阻止保存进展。正文回读完成并保存新 presentation 后，再运行 ready 预检。

发布后回读正文，保留原始 Markdown 和服务端查询结果，在 delivery.presentation 中引用
如下 JSON（所有文件均保存在原运行目录、引用均为 `{path,sha256}`）：

```json
{
  "schema_version": 1, "task_id": "原任务 ID",
  "repo": "owner/repository", "pr": 123, "base_branch": "原授权分支",
  "base_sha": "完整 SHA", "head_sha": "完整 SHA",
  "review_sha256": "当前汇总回执 SHA-256",
  "body_ref": {"path": "pr-body.md", "sha256": "远端原始正文 SHA-256"},
  "artifacts": [{"path": "pr-readback.json", "sha256": "查询结果 SHA-256"}],
  "screenshots": {
    "status": "paired",
    "pairs": [{
      "scene": "正文中的场景名",
      "before": {
        "url": "GitHub 实际返回的持久附件 URL", "sha256": "脱敏图片 SHA-256",
        "revision": "original 或 parent best 的完整源码 SHA",
        "content_verified": true, "render_verified": true,
        "evidence_ref": {"path": "before-verification.json", "sha256": "核验证据 SHA-256"}
      },
      "after": {
        "url": "GitHub 实际返回的持久附件 URL", "sha256": "脱敏图片 SHA-256",
        "revision": "当前 PR head",
        "content_verified": true, "render_verified": true,
        "evidence_ref": {"path": "after-verification.json", "sha256": "核验证据 SHA-256"}
      }
    }]
  }
}
```

上例为结构示意，不能直接通过。每个 image.evidence_ref 指向 JSON，含 `url`、`revision`、
`download_ref: {path,sha256}`、`content_type`、`width`、`height`、`render_verified`，
download_ref 引用实际下载的脱敏附件，摘要须等于 image.sha256。两个 verified 字段只能由
实际核验填写。检查器核对下载文件摘要、正文同一表格行的前后图片引用、版本及核验记录，
不能代替查看图片、确认可比性、排版或发现敏感信息。
无适用画面或用户禁止公开时，screenshots 使用 `status: "not_applicable" | "prohibited"`，
给出 `reason`（正文同样展示）及 `evidence_ref`（实际变更或授权依据）。
工具不可用或缺图使用 `status: "blocked"`，保持未就绪，不能改成“不适用”过门禁。

标记交付就绪、pr_only 记 done、合并前，运行相邻快照检查及远端回读：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/delivery-check.mjs \
  ready "$RUN_DIR/workflow.json" "$TASK_ID" "$RUN_DIR/workflow-previous.json"
```

ready 只读调用 `gh pr view`，核对最新 head/base、OPEN 状态和正文摘要；退出 0 表示本次
预检通过，退出 2 表示缺口或查询失败。Draft 可通过预检以便随后获准标记 ready。
输出保存为本次交付证据，操作后回读真实终态；合并意图仍先写入现有 operations。
若修改了正文、报告或 head/base，重新生成并回读，不继续使用旧预检结果。
未知合并结果必须先恢复，不再运行新的合并。停止或预算不足时不启动远端预检。

verify-workflow 已将 presentation 和双审查完整性接入 merging/远端 done 门禁；
离线检查只证明保存的快照。ready 预检与后续远端写入仍存在时间间隔，实际写入前核对
身份并使用工具支持的 head 匹配保护。尚未接入宿主拦截或远端必需检查，不宣称不可绕过。
