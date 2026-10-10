# 目标、任务与持续执行

启动、恢复、重新规划及整体完成判断前读取。总控由当前宿主 Agent 执行；
检查器只读，不创建后台服务，也不自动调用子 Agent、Git 或浏览器。
压缩、交接和阶段切换按 [恢复入口](recovery.md) 重新加载规则及原账本。

## 从输入形成完成条件

- 网址：公开只读地探索适用功能，形成主要用户任务、页面状态与成功条件。
- 目标：在目标范围内选择代表站点或自有 fixture，写明选择依据与未覆盖范围。
  缺少网址本身不阻塞；无法判断目标对象或修改范围时只澄清该缺项。
- 网址加目标：以指定目标排序任务，不自动扩到全站或所有浏览器能力。

首次有界探索后，将原始请求、授权来源、网站范围、必需用户任务、允许自主拆分的
边界、预算和验证模式写入不可变 contract 文件。简单任务使用短目标清单；只有必要
跨模块改造才补技术方案。用户已委托范围内的方案与任务是执行记录，不逐项审批。
禁止把“优化得更好”直接作为完成条件；转换为可观察的任务终态、行为/几何判据或
冻结的性能测量条件，复用 parity 和仓库阈值，不另发明体验总分。

`goals` 全部为必需验收项，覆盖原目标。一个 PR 为一次预算迭代；候选、探索时段
另作过程统计，不代替 PR 次数。总时间从 started_at 到现在，包含暂停和离线；
任务完成、一次 accept、测试全绿或创建 PR 都不是整体完成。
既有目标不得删除、改名或放宽判据；目标范围确需变更时核验新授权并保留旧合约。

## 外层规划与内层优化

外层每次读取当前 best、目标缺口、任务依赖、未决操作、预算与停止屏障：

1. 有未决操作先查实际宿主状态，接续或收回，不重复派发。
2. 选择未暂停且依赖全为 done 的任务；默认串行，一个实施者或验证者在途。
3. 内层执行入口的探索、复现、通用修复、回归、原站对照与候选门禁。
4. accept 后更新 best，复测受影响的已通过用户任务；将可验收切片提交为任务 PR。
   按 [双审查](independent-review.md) 完成两个独立首轮，再同一 PR 返修、重验和非作者复核。
5. 主控按 github-delivery 合并并验证集成版本后标 done，报告阶段成果并立即领取
   下一项，不等待用户回复。全部任务 done 时在最新集成版本做整体验收。
6. 整体验收发现缺陷，追加关联原目标的修复任务；保留原 done 历史，继续循环。

同簇连续两次无新证据，暂停该簇并尝试新假设、规范研究、最小复现、前置能力拆分
或其他独立任务。三段没有新发现只触发重新规划，不终止整个目标。
所有任务都暂停时，检查可解除的依赖和工具故障；仍有获准、可负担的调查就继续。
只有确实没有安全可执行路径时记 infrastructure_blocked，写明已尝试路径、缺口和
恢复条件。“难修”“暂未定位”不直接等于基础设施阻塞；不得重复空转耗尽预算。
调研和诊断同样有单步上限，实际消耗进入累计预算。

## 宿主与角色

先核对实际工具 schema 和可用能力，不臆造 spawn/wait/cancel 参数。持续运行需要
宿主在目标未完成时继续当前任务，支持有界等待、停止、进程回收和检查点恢复。
有 Goal/自动续行能力且已获授权时复用；没有时在当前前台会话继续执行。
启动长期目标时，在宿主允许且用户明确要求长期目标的前提下建立该目标；只改本 skill
不创建运行目标。上下文将满时先保存检查点和精确下一动作，再交接到新上下文继续，
不能用“已有阶段成果”“建议下一步”作为目标未完成时的正常结束。
会话退出、宿主失联或无法续行时保存下一动作并如实报告停止/unknown，不声称后台
仍会运行，也不为此自动安装 cron 或 tmux 控制器。

宿主允许独立子 Agent 时，优先总控加新上下文 worker/reviewer，默认不并行重型任务。
每个实施任务完成后关闭其已结束上下文，下一任务使用新 worker；首轮审查使用未参与
实现的新 reviewer。定向复核按补丁非作者安排，最终汇总者不得参与该运行的任何实施；
具体身份要求按双审查契约执行。默认串行复用当前工作区及构建缓存，不为 worker 新建 worktree；
首个开发分支按入口从最新 main（或明确约定的 base）创建，后续已合并任务按
github-delivery 同步集成版本再建下一任务分支。主控负责规划、账本、合并与停止。
local/pr_only 模式的成果尚未集成时，后续工作沿用当前 best 及其分支；需要拆分 PR
则记录依赖和实际 base，不能重新从 main 起步丢掉已有成果。恢复时继续原任务分支。
总控独占 workflow/checkpoint 写入权；worker 只写自己的产物与候选，review operation
只读源码。测试角色需补测试时另领 implement operation，记录作者后按双审查契约安排非作者复核。
每个任务包包括：原授权引用、目标与任务 ID、精确 best、当前工作区/任务分支、剩余总预算
与本步上界、验证方式、证据位置、禁止事项。新 worker 不继承整段实现推理，
不重新发默认预算或向用户重问已有权限。角色名称不同不证明上下文独立。

无独立能力可在首次预检冻结 `deterministic` 模式：由当前 Agent 完成确定性行为、
像素和性能回归，明确不声称独立体验收益。用户要求独立验收时使用 `independent`，
缺能力阻塞该验收，不在运行中悄悄降级。总体验收在最新 best 重新走完整任务，
不能将各切片的 PASS 相加；仅有截图不能证明交互目标。
verification_mode 控制产品验收；任务 PR 的适用双审查始终需要独立上下文，
不能用 deterministic 绕过。能力暂不可用时先做独立可推进工作，再按真实缺口处理。

## 状态与单写者

沿用 `.acceptance/site-optimizer/<run-id>/`，确认已忽略且没有被 Git 跟踪。
`checkpoint.json` 仍负责候选门禁与唯一累计预算；新增 `workflow.json` 管目标与任务，
只引用不可变 checkpoint 快照，不复制消费数字。PR 次数从任务 delivery 按小写 repo
加 PR 编号去重计算，输出 pr_iterations_used/pr_iteration_limit/pr_iteration_available；
本地模式已用 PR 数为 0，不用 PR 上限拦截本地工作。run.md 是解释与进度视图。
每次写入先保存证据及 `checkpoint-<revision>.json`，再更新 workflow 引用；
校验后原子替换当前文件，保留上个有效快照。所有引用复用相对路径与 SHA-256。

使用 [workflow 模板](../templates/workflow.json)。contract_ref、checkpoint_ref 必须换成
真实文件引用，模板本身不能通过。checkpoint 快照与 workflow 放同一运行根目录，
内部证据均相对此根解析；不要把副本放到 worker 目录另建预算。
初始 original/best/trial 都为 null 时，可凭真实 manifest 一次性记录 original=best，
trial 保持 null；这是原始基线，即使原站目标失败也能保存，失败门禁原样保留。
之后 original 不可更改，任何新 best 仍须通过候选门禁，不能重复走初始化路径。

| 字段 | 契约 |
|---|---|
| schema_version / run_id / revision | 1、与 checkpoint 一致、从 1 单调加 1 |
| contract_ref / checkpoint_ref | 不可变合约及本次 checkpoint 的 `{path, sha256}` |
| verification_mode | 启动冻结 independent 或 deterministic |
| delivery_mode | auto_merge（默认方案）、pr_only 或 local；以实际授权冻结，旧记录缺省为 local |
| goals | 唯一 id、description、verification；全部必需且不可普通修订 |
| tasks | 唯一 id、goal_ids、depends_on、description、status、pause_reason、evidence、repairs_task_id、delivery |
| operations | 唯一 id、task_id、kind、status、executor_ref、result |
| final_acceptance | 最新整体验收报告引用；尚未验证为 null |
| recovery_ref | 可选，默认为 null；本次解除基础设施阻塞的证据引用，见下文 |
| stop_reason | null 或入口列出的停止原因；不得因新会话清除 |

默认任务阶段：`pending → implementing → verifying → pr_review → merging → integrating → done`。
验证或审查失败走 `verifying|pr_review → needs_fix → implementing`，仍修同一 PR。
head/base 漂移时回 pr_review；合并结果未知保持 merging，先查服务端。
local 模式 verifying 后可 done；pr_only 在独立审查通过后从 pr_review 到 done，
不调用合并。研究/确认无需改动的任务用 delivery={kind:"no_change",reason:"实际原因"}，
在 verifying 后凭原站/研究证据 done，不制造空 PR，也不能给有代码改动的任务套此豁免。
阶段之外的局部暂停用 pause_reason，保留阶段；done 必须有 evidence，
不可重写，新增缺陷另建任务。新任务只能 pending，不能删历史任务。只有 pending
任务允许重新排序依赖，依赖必须无环且每个目标至少有任务覆盖。
实施前重新核对完整原始合约，task.evidence 指向任务结果及相应候选、门禁和原站证据，
不能只放“已完成”文本。原始记录的真实性由总控核验。

operation.kind 为 implement/review/publish/merge/integrate/final_acceptance，
status 为 intended/running/completed：
先持久化 intended，取得真实宿主 ID 后写 running；结束后写 completed 和不可变
result。失败也是 completed，但结果明确失败，不据此将任务记 done。
当前 Agent 自行执行时 executor_ref 使用可核对的宿主任务身份，同样记录开始/结束。
completed 后不能改 result，重试用新 ID。结果未知时保留未决操作。
首次 publish 在 verifying；已有 PR 的正文、附件和审查状态更新也可在 pr_review 派发
publish，无须重开实施。代码 review 在 pr_review、merge 在 merging、integrate 在
integrating 派发；本地验证 review 可在 verifying。合并操作的 subject 必须保存
delivery 中的 repo/pr/base_branch/base_sha/head_sha，持久化后不得改写。
子任务工具不支持某种角色时用当前主控实际宿主身份执行其获准操作，不虚构 child ID。
final_acceptance 是目标级操作，task_id=null；不重开或篡改 done 任务来派发最终验收。
subject 固定为 `{manifest_sha256,contract_sha256}`，分别绑定派发时的 best 与合约。
全部任务 done、无未验证 trial、best 门禁通过后才能先写 intended；然后记录实际
验收者 ID、running、completed 和结果。结果的 subject、contract_sha256、reviewer ID
须与操作一致，并将结果引用写入 workflow.final_acceptance。失败也保留完整终态；
重试用新操作 ID。中断后先接续该操作，不能重复派发。

### PR 证据与合并后返修

有代码改动的远端任务 delivery 使用：

```json
{
  "kind": "pr", "repo": "owner/repository", "pr": 123,
  "base_branch": "已授权集成分支", "base_sha": "完整 Git SHA", "head_sha": "完整 Git SHA",
  "candidate_manifest": null,
  "review": null, "presentation": null, "merge": null, "integration": null
}
```

candidate_manifest 与各项回执均为运行目录内 `{path,sha256}`。候选 manifest 须明确
source_sha、base_sha、dirty_patch；前两项分别等于 PR head/base，dirty_patch=null
表示验证的是干净提交。沿用现有构建清单的二进制、features 等证据，字段不同则生成
引用原清单摘要的适配清单，禁止猜填。尚未取得候选身份时引用可为 null，但不能合并。
review.subject 绑定此清单摘要；合并派发还要求 checkpoint.candidate_manifest 是
同一清单且门禁通过。仅重做 review、不重验新 head，不能借用旧候选的 PASS。
创建 PR 前检查次数余额，经服务端确认后立即记录 delivery，Draft、关闭或放弃也计一次；
创建结果未知先查服务端，不能重复创建或漏记。任务首次记录的 repo/pr 不得改写或删除，
同一 PR 的 head/base 变化不增次；新 PR 用新任务保留原历史，不以换编号重置计数。
历史任务保留自己的清单，不随全局 best 更新。旧记录缺少这些身份时补采/重验，
不能把旧通过证据直接改绑当前 PR。

回执公共字段为 schema_version=1、task_id、
artifacts（非空原始证据引用）。审查和合并报告还须包含同一精确 PR 的五项身份字段：

- review：verdict=PASS 或 CHANGES_REQUIRED、open_findings 数组、
  subject（候选 manifest 摘要）、reviewer={executor_ref,independent}。
  stage、review_scope、角色报告和完成 operation 按 [双审查字段契约](independent-review.md)。
  交付前须 PASS、零未解决阻断项，汇总 reviewer 不是总控或该运行的任何实施者。
  适用双审查时，artifacts 引用两份独立首轮、发现处置及必要的非作者补丁复核；
  各首轮与复核分别作为串行 review operation 保存，不能把首轮单独填为 delivery.review。
  返修和 base/head 漂移后定向复核并生成当前版本的汇总报告；旧报告保留，不能重绑 SHA。
- presentation：远端正文与附件核验回执，字段见 [GitHub 交付](github-delivery.md)。
  Draft 阶段可为 null；进入 merging、integrating 或远端 done 时须通过正文与截图检查。
  正文修改后保存新的回读和回执；head 未变也不能复用旧 body 摘要。
  更新汇总后，旧 presentation 自动视为过期，pr_review 仍允许登记 publish 补齐；
  过期状态不能进入 merging 或记 done。
- merge：confirmed=true、commit（服务端确认的完整合并 SHA）、
  review_sha256（使用的 review 引用摘要）。未知结果保持引用 null、操作未决；
  明确未合并则完成失败操作并回 pr_review，不能把失败回执放入成功 merge 字段。
  head_branch（合并的远端分支名）与 branch_deleted（合并确认后是否已删除该远端
  分支）；同分支仍有开启 PR（串行 PR 复用运行分支）时为 false 并记
  branch_kept_reason，待该分支最后一个 PR 合并的回执删除。历史回执不追溯。
- integration：status=PASS/FAIL/INCONCLUSIVE、commit（实际验证的集成 SHA）、
  merge_commit、contains_merge（真实 Git 包含关系）、subject（集成 manifest SHA）。
  主控核对实际构建、smoke、受影响任务及原始门禁；进入 done 时 subject 必须是
  当时通过门禁的新 best。历史 done 的证据保留原身份，不要求跟随后续 best 改写。

集成验证失败时，原任务保持 integrating 并记录暂停原因，另建 pending 返修任务，
repairs_task_id 指向原任务；返修任务不得直接或间接依赖未完成的原任务。
在同一工作区同步最新 integration 后创建新分支/PR，复用构建缓存；
不重开或继续提交到已合并分支。修复 PR 合并并
验证后，重新验证原任务，替换其 integration 引用并解除暂停，再记 done。
原任务的 merge 身份与回执不可修改。其下游一直等待原任务 done。
整体验收失败同样追加任务，关联原 goals，保留已 done 的历史。

检查器核对结构、摘要、状态转换和回执身份，不查询 GitHub 或判断报告是否造假。
双角色、非作者复核、报告版本和 completed operation 关系已进入机器检查；
CI、保护规则、PR 当前状态、原始日志语义和上下文独立性仍由主控实查；
不得把本地 review 回执伪装成 GitHub required approval。
合并前另运行 delivery-check.mjs ready 回读当前正文；离线 workflow PASS 不能证明远端未变。

首次建立后，每次保存必须带上一个不可变 workflow 快照：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/verify-workflow.mjs \
  "$RUN_DIR/workflow-next.json" "$RUN_DIR/workflow-previous.json"
```

退出 0 = 整体目标证据就绪；1 = 合法但未完成（正常执行中通常为 1）；2 = 结构、
转换或证据错误，不可推进。`next` 是阶段建议，不是权限或实际调度：
continue、wait_or_recover、estimate_budget、verify_executor、final_acceptance、replan、stop。
必须实际核对宿主、原始报告、授权及预算。检查器不证明 Agent 已持续执行。

升级前的回执没有 stage/review_scope，且 delivery 没有 presentation 字段时，按旧格式
核验 previous 快照。当前快照只保留两类旧交付：与 previous 完全一致的 done 任务，
以及 previous 已确认合并且身份、review/merge 回执不变的任务；后者继续完成集成验证。
输出 `legacy_delivery_tasks` 明确列出这些历史任务，不能宣称其已通过新增双审查或正文检查。
尚未交付的旧任务须回到 pr_review，保留旧证据文件、清空过期 delivery.review 引用后，
通过新的 review/publish operations 补齐再交付。不得改旧 operation、预算或已 done 记录。
新合并、新的 pr_only done 和无 previous 的检查始终要求完整新版回执。

新目标需要初始规划；旧运行已有冻结任务时从原记录迁移，保留 original/best/trial、
已有次数上限、deadline、失败和消费。缺失字段按原始证据补齐，不能猜补为成功。
范围/预算扩展或解除 user/safety stop 必须保存明确授权修订及前后快照，独立核验来源后
建立修订起点；普通恢复必须走相邻校验，不用省略 previous 参数规避检查。

## 恢复与停止

同一运行只允许一个总控。优先宿主排他任务；必要时使用运行目录内原子 mkdir 锁，
记录 controller、host、PID 与启动身份。不能按锁文件年龄抢占；确认旧主控已退出，
并在独立恢复锁内二次核验后才接管。原子 rename 本身不提供互斥。

| 中断点 | 下一动作 |
|---|---|
| intended 已保存但没有 child ID | 按 operation ID 查询；确认未启动才重试，未知则阻塞 |
| worker 仍存活 | attach/有界等待，不开第二个实施者 |
| worker 已退出但结果缺失 | 核对记录中的工作区/分支、费用和产物；未知消费保留预留，不能填零 |
| trial 未验证 | 保留 trial，补取终态或恢复 best；不能推进完成 |
| 新 best 或集成基线变化 | 旧报告保留身份，重做受影响目标及最终整体验收 |
| PR 创建/合并结果未知 | 查询服务端真实状态，确认前不重复写入、不派发依赖项 |
| 用户停止或预算不足 | 先保存停止屏障，再取消/收回已有任务，保全 best 与证据 |

infrastructure_blocked 可在能力恢复后沿原合约续跑：先核对并收回全部未决操作，
确认执行者 running、剩余额度可用，再保存新的 recovery_ref 报告：
schema_version=1、run_id、previous_revision（此次被解除的 workflow 修订号）、
resolved=true、checked_at（带时区时间）及非空 artifacts。核验时间须位于前后
checkpoint.updated_at 之间。先单独保存清除停止原因的快照，下一修订才能派发工作。
旧恢复回执不能复用；预算和权限不变。user_stopped/safety_blocked 不走此路径，
仍需用户明确授权修订；预算耗尽等其他停止原因也不能借恢复回执清除。

新调用前检查“下一完整步骤＋预留”，不得真正超支后才停止；运行中以剩余预算设置
超时，时间到限立即收回。停止时允许记录已有操作的终态、清理和保存，不允许新
实施/审查/合并调用。状态询问回复后继续仍活跃的原任务；明确“只查询、不恢复”
则只读，用户停止优先。可恢复阻塞在原预算内处理，无法解除才形成硬停止。
PR 次数达限后不派发尚无 PR 的新实施或发布；在途操作先接续，已有 PR 可继续返修、
重审、合并和集成，无代码改动的验证及全部 done 后的最终验收仍可执行。
合并后另建修复 PR 需要新的次数，额度不足按 iteration_limit 收尾。
若原合约另有限制候选/探索次数，则这些独立上限仍阻止相应新尝试，不当作 PR 计次。
总时间、明确的额外费用限制和人工停止始终有效，不因限次后的收尾而扩大额度。

## 整体验收报告

final_acceptance 指向如下报告，checks 必须逐项覆盖 goals，subject 绑定当前 best：

```json
{
  "schema_version": 1,
  "subject": "当前 best manifest 的 SHA-256",
  "contract_sha256": "冻结合约的 SHA-256",
  "checks": [{"id": "primary-journey", "status": "PASS"}],
  "reviewer": {"executor_ref": "实际宿主执行者 ID", "independent": true},
  "artifacts": [{"path": "final/actual-results.json", "sha256": "原始证据 SHA-256"}]
}
```

检查状态沿用 PASS/FAIL/INCONCLUSIVE/SKIPPED。缺失、过期、跳过或失败不能完成；
independent 模式还要求实际新上下文且不是任何实施者或当前总控。
脚本只能检查记录 ID 与摘要，不能证明上下文隔离、真实交互或报告语义。
任务全部 done、当前 best 的目标与工程门禁无豁免通过、无未决操作/未验证 trial，
且整体验收逐项 PASS 才能记 completed。带授权豁免可交付阶段成果，但不冒充完整达标。
如果原目标已满足而无需改动，仍采集 original=best 的门禁和整体证据即可完成。
交付成功与目标完成分别报告；停止后不为补齐形式证据启动新付费任务。
