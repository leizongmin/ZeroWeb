# 压缩后的恢复入口

压缩、交接、新会话及进入发布、审查、合并阶段时，历史“已读”标记失效。
先重新读取 SKILL.md、本文件和当前阶段原文；摘要只提供线索，不能替代规则或证据。
只讨论或编辑 Skill 时不恢复网站运行。

## 定位与恢复

从宿主任务或交接记录取得原 run ID，对应 `.acceptance/site-optimizer/<run-id>/`。
未知时只读列出该目录并核对 run.md 的原任务和宿主身份；多个候选不能直接选最近目录。
交接须留下运行目录、当前 workflow 和前一不可变快照路径。不要新建 run 来绕过旧状态。

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/recover.mjs \
  "$RUN_DIR/workflow.json" "$RUN_DIR/workflow-previous.json"
```

仅 revision=1 可以省略 previous。输出是从原账本生成的只读导航，包含原合约引用、
deadline、停止原因、版本、未完成任务、未决操作及必读文件的当前摘要，不是另一份账本。
可以另存到运行目录便于交接，但每次恢复重新生成。退出 2 表示证据或转换不合法，
先补查原始记录；不能猜填成功或删除历史以通过。
输出中的 legacy_delivery_tasks 表示沿用旧契约的已完成或已合并任务，按 workflow
兼容规则保留并披露；它不证明这些历史任务具备新增双审查或正文证据。

1. 完整读取 required_reads 和原 contract，核对授权、拒绝、预算及工作区/分支。
   规则摘要用于识别版本变化，不证明已经读取，也不自动扩大旧授权。
2. 按 workflow 核对单写者、真实宿主、进程及未决操作；查询 PR 创建/合并的服务端
   终态，只补确认缺失的动作。停止屏障优先于 checkpoint.next_action 中的旧建议。
3. 运行相邻快照检查，处理缺失证据，再按当前阶段加载下表原文。summary 里的
   “已审查”“全部完成”不能覆盖本地缺口；两种记录冲突时保持未就绪。
4. 保存当前检查点和精确下一动作，再开始一项有界任务。沿原 deadline 和累计预算，
   不因压缩重置时间、次数、候选或停止原因。

| 阶段边界 | 本上下文重新读取 |
|---|---|
| 发布或更新 PR | github-delivery.md、github-pr.md、当前附件映射；运行 draft 检查 |
| 派发首轮或复核 | independent-review.md、精确身份、该角色原始输入；核对未决 review |
| 标记就绪或合并 | github-delivery.md、workflow.md、当前汇总与 presentation；运行 ready 回读 |
| 集成或整体完成 | workflow.md、checkpoint.md、当前集成与验收证据 |

文件路径相对此 Skill 的 references/ 或 templates/，详见入口链接。
阶段边界即使没有收到压缩事件也执行；首轮 reviewer 不读取主控恢复包中的其他角色报告。

## 强制能力边界

仓库入口提供导航，检查器拒绝不完整记录；仍需 Agent 实际调用并遵守。
`delivery-check.mjs ready` 会只读查询 GitHub，不能代替分支保护或持有写权限的受限工具。
当前未接入 TRAE 压缩 hook、工具拦截或 GitHub required check，不能声称无法绕过。
未来有已验证的宿主回调时，可在压缩后注入恢复导航、在写操作前调用门禁；不得编造配置。
