# GitHub PR 前后截图交付

启动合约包含 GitHub 交付时，在前置能力预检和最终交付时读取；仅讨论或编辑 skill
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
- PR 正文及附件映射保留本地备份；清理 worktree 前先归档仍需保留的证据并核验。
  图片交付完成不改变原实验停止原因，也不授权自动合并或发布。
