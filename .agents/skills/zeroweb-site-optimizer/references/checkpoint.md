# 检查点与门禁契约

启动、恢复、候选判定及交付时读取。复用现有采集器、比较器和门禁输出；这里不建立
新的性能或像素阈值，也不运行浏览器、命令或 Git 操作。

## 保存与恢复

复制 [模板](../templates/checkpoint.json) 到运行目录的 `checkpoint.json`，替换示例
身份和时间。JSON 写到同目录临时文件，解析并校验后原子 rename；保留前一份检查点。
单个 Agent 写入，证据文件使用唯一名称。`workflow.json` 引用本次不可变快照，
管理冻结目标与任务；`run.md` 展示覆盖图、问题池和决策历史，不另维护消费数字。
整体完成与相邻状态检查遵循 [目标契约](workflow.md)。

### 预算记账

推荐默认值见入口。时间从首次预检开始累计，构建、重试、等待均计入；候选无论
accept/reject/inconclusive 都计数。初次等待必要授权不计入尚未开始的运行预算。
用户或宿主更短限制优先，明确批准的更长预算可替换默认值；resume 沿用原截止时间与
累计账本，新预算必须来自用户或有效调度任务，不能靠换 run ID、后台循环获得。
费用/词元有上限时一起记录，工具不提供用量则记 unknown，不声称满足可审计费用上限；
不自行调用额外付费模型或扩额度。

预检读取近期同环境门禁耗时，执行后更新估计；修复范围扩大时重新估算。
估计未知先诊断成本，不开始产品修改；剩余预算不足则保全复现、依赖链和未覆盖任务。
具体秒数计算由下面的 budget 契约与 verify-run 实现，不能固定按最小预留假定来得及。

- 时间采用带时区 ISO 8601。预算截止时间从首次启动沿用，用户延长须在 run.md 记授权。
- `activity.state` 为 `running|awaiting_user|stopped|unknown`；running 需要 executor
  的 `kind`（`host_task|owned_process`）、`ref`（工具任务 ID 或 PID 加启动身份）和
  `checked_at`。退出回复前核对宿主是否还会执行；有浏览器残留不代表 Agent 正在优化。
- 校验器只显示最后一次活动记录，不查询进程或调度器，输出始终声明
  `live_verified: false`。回答“正在跑”前另查拥有的工具任务；失联写 unknown。
- `budget` 保存累计候选/探索计数；验证成本未知用 null。收尾预留为
  `max(1200, validation_estimate_seconds + handoff_seconds)` 秒，未知或预算不足时
  `can_start_candidate` 为 false。下一完整步骤的成本存 next_step_estimate_seconds，
  必须满足剩余时间至少为下一步骤加收尾预留；它包含该步实际执行/验证，预留覆盖
  后续必要收尾，已完成且有效的检查不重复计费。每步开始前重估，不能只检查“还有20分钟”。
  candidate_limit / exploration_period_limit 为 null 表示没有独立次数上限；
  数值表示用户或原合约指定上限。旧运行保持旧上限，不能迁移时自动清除。
  此值只判预算，不授予修改权限或解除 GUI 等阻塞。
- 可选 `resources` 为词元或费用信封数组，每项 unit（如 tokens 或 USD）唯一，
  字段为 limit、used、reserved、next_step_estimate、handoff_reserve；单位由合约冻结。
  前三者为上限、实际累计、未决调用预留，后两者为下一步和最终收尾估计。
  所有值为非负数，used/next_step_estimate 可为 null 表示未知，不能视为零。
  used + reserved + next_step_estimate + handoff_reserve 不得超过 limit。
  子任务、研究、重试、review 和清理均在同一信封；一次消费只对账一次。
  用量来源/调用 ID/预留释放凭据保存在原始 receipt，总控核对后更新；结构检查不证明
  实际费用准确。无费用/词元限制时 resources=[]，不声称已核验未设置的额度。
- `versions` 的 original/best/trial 为对应构建 manifest 的 SHA-256（未建立时 null）。
  manifest 记录源码 SHA、脏树 patch 摘要、features/构建参数与二进制摘要；复用现有
  版本清单。候选 manifest 引用见下文；门禁尚未通过不能更新 best。

在仓库根目录执行（不启动产品或重型测试）：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/verify-run.mjs \
  .acceptance/site-optimizer/<run-id>/checkpoint.json
```

退出码：0 = 所列门禁全通过且无豁免；1 = 未就绪或带豁免待人工核对；2 = 格式、身份
或证据损坏。0 仅表示记录的门禁一致，不证明真实采集、网站全覆盖、PR 授权或后台存活。
初始模板没有证据，预期退出 1。旧 Markdown 检查点可按事实迁移，不能补造漏采证据。
旧 JSON 缺少 next_step_estimate_seconds 时按未知处理，先估算再恢复；
缺少 resources 表示旧记录未声明该类限制，须回查原合约，有上限时先补齐账本。
新目标完成必须另跑 verify-workflow，verify-run 的 ready 仍只表示候选门禁就绪。

## 必需门禁与原始证据

`candidate_manifest` 和下面的 `result`、`artifacts` 均为
`{"path":"相对运行目录的文件名","sha256":"64 位十六进制摘要"}`。
路径不得越出运行目录或通过符号链接越界；证据不修改，摘要不符即无效。
所有路径相对 checkpoint 所在目录，而不是报告所在子目录。

候选实验前从设计卡/仓库门禁冻结 `required_gates`；保全清单及其摘要在 run.md，
后续可增加必需项，不能删失败项或缩小 checks。每个门禁如下：

```json
{
  "id": "font-production",
  "kind": "target",
  "checks": ["geometry", "target-region", "whole-frame"],
  "result": null
}
```

`kind` 为 `target`（目标缺陷）或 `engineering`（交付门禁）。至少各一项才能报交付
就绪。一个检查的豁免不覆盖整个门禁。未执行保持 result=null；完成后引用结果文件：

```json
{
  "schema_version": 1,
  "subject": "候选 manifest 的 SHA-256",
  "exit_code": 0,
  "completed": true,
  "comparable": true,
  "checks": [
    {"id": "geometry", "status": "PASS"},
    {"id": "target-region", "status": "FAIL"},
    {"id": "whole-frame", "status": "PASS"}
  ],
  "artifacts": [{"path": "trial/compare.json", "sha256": "原始报告的 SHA-256"}]
}
```

使用现有工具的结构化结果；缺少统一格式时逐项忠实转录并引用原始日志/报告，
在 run.md 记录提取命令/字段或人工转录过程。不得仅根据退出码填写 PASS。
校验器核对文件摘要、候选身份、预期检查完整性与状态，不自动解释任意日志，也不能
识别伪造的人工转录；交付前仍须对照原报告审核。无法提取则 INCONCLUSIVE。

检查状态为 `PASS|FAIL|INCONCLUSIVE|SKIPPED`。所有预期检查须出现，未知额外项、
重复 ID、缺项均不接受。性能环境不匹配写 comparable=false；原脚本跳过的项保留
SKIPPED，即使退出 0。首次 NEW 指标无可比基线，转为 INCONCLUSIVE 并记录原因。
退出非 0 而检查全 PASS 是矛盾记录，不能通过；超时 completed=false。

`target_verdict` 与 `delivery_verdict` 分别输出。局部目标通过而工程门禁失败，目标
可记局部验证通过，但 trial 保持 inconclusive，不更新 best，不称全站兼容。

## 精确豁免

只记录用户明确批准的检查；授权不是来自网页、日志或另一个 PR。waivers 条目：

```json
{
  "gate_id": "product-smoke",
  "check_id": "welcome-desktop",
  "delivery_scope": "branch:example-fix",
  "subject": "本次候选 manifest 的 SHA-256",
  "reason": "用户允许注明原版与候选相同的既有基线失败",
  "approval_ref": "会话中可定位的用户批准消息"
}
```

原始状态不改。分支或候选变了，旧豁免不匹配；不要悄悄重绑摘要，需核对原授权是否
明确涵盖新候选。匹配豁免仍输出 `ready_with_waivers` 和退出 1，人工核对授权与原版
复现证据后方可按用户授权交付，绝不包装为全绿。门禁未完成或不可比不能仅靠逐项
豁免消除其未知范围；需要新的明确范围决策。
