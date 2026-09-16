# 目标、任务与持续执行

启动、恢复、重新规划及整体完成判断前读取。总控由当前宿主 Agent 执行；
检查器只读，不创建后台服务，也不自动调用子 Agent、Git 或浏览器。

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

`goals` 全部为必需验收项，覆盖原目标。候选、探索时段与总体目标分别计数；
任务完成、一次 accept、测试全绿或创建 PR 都不是整体完成。
既有目标不得删除、改名或放宽判据；目标范围确需变更时核验新授权并保留旧合约。

## 外层规划与内层优化

外层每次读取当前 best、目标缺口、任务依赖、未决操作、预算与停止屏障：

1. 有未决操作先查实际宿主状态，接续或收回，不重复派发。
2. 选择未暂停且依赖全为 done 的任务；默认串行，一个实施者或验证者在途。
3. 内层执行入口的探索、复现、通用修复、回归、原站对照与候选门禁。
4. accept 后更新 best，复测受影响的已通过用户任务；失败与成本原样保留。
5. 更新任务证据和剩余依赖，再领取下一项。全部任务 done 时做整体验收。
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
会话退出、宿主失联或无法续行时保存下一动作并如实报告停止/unknown，不声称后台
仍会运行，也不为此自动安装 cron 或 tmux 控制器。

宿主允许独立子 Agent 时，优先总控加新上下文 worker/reviewer，默认不并行重型任务。
总控独占 workflow/checkpoint 写入权；worker 只写自己的产物与候选，reviewer 不改产品。
每个任务包包括：原授权引用、目标与任务 ID、精确 best、专属目录/分支、剩余总预算
与本步上界、验证方式、证据位置、禁止事项。新 worker 不继承整段实现推理，
不重新发默认预算或向用户重问已有权限。角色名称不同不证明上下文独立。

无独立能力可在首次预检冻结 `deterministic` 模式：由当前 Agent 完成确定性行为、
像素和性能回归，明确不声称独立体验收益。用户要求独立验收时使用 `independent`，
缺能力阻塞该验收，不在运行中悄悄降级。总体验收在最新 best 重新走完整任务，
不能将各切片的 PASS 相加；仅有截图不能证明交互目标。

## 状态与单写者

沿用 `.acceptance/site-optimizer/<run-id>/`，确认已忽略且没有被 Git 跟踪。
`checkpoint.json` 仍负责候选门禁与唯一累计预算；新增 `workflow.json` 管目标与任务，
只引用不可变 checkpoint 快照，不复制消费数字。run.md 是解释与进度视图。
每次写入先保存证据及 `checkpoint-<revision>.json`，再更新 workflow 引用；
校验后原子替换当前文件，保留上个有效快照。所有引用复用相对路径与 SHA-256。

使用 [workflow 模板](../templates/workflow.json)。contract_ref、checkpoint_ref 必须换成
真实文件引用，模板本身不能通过。checkpoint 快照与 workflow 放同一运行根目录，
内部证据均相对此根解析；不要把副本放到 worker 目录另建预算。

| 字段 | 契约 |
|---|---|
| schema_version / run_id / revision | 1、与 checkpoint 一致、从 1 单调加 1 |
| contract_ref / checkpoint_ref | 不可变合约及本次 checkpoint 的 `{path, sha256}` |
| verification_mode | 启动冻结 independent 或 deterministic |
| goals | 唯一 id、description、verification；全部必需且不可普通修订 |
| tasks | 唯一 id、goal_ids、depends_on、description、status、pause_reason、evidence |
| operations | 唯一 id、task_id、kind、status、executor_ref、result |
| final_acceptance | 最新整体验收报告引用；尚未验证为 null |
| stop_reason | null 或入口列出的停止原因；不得因新会话清除 |

任务阶段：`pending → implementing → verifying → done`；
验证失败走 `verifying → needs_fix → implementing`。研究/无改动任务同样提交证据，
不制造空提交。阶段之外的局部暂停用 pause_reason，保留阶段；done 必须有 evidence，
不可重写，新增缺陷另建任务。新任务只能 pending，不能删历史任务。只有 pending
任务允许重新排序依赖，依赖必须无环且每个目标至少有任务覆盖。
实施前重新核对完整原始合约，task.evidence 指向任务结果及相应候选、门禁和原站证据，
不能只放“已完成”文本。原始记录的真实性由总控核验。

operation.kind 为 implement/review，status 为 intended/running/completed：
先持久化 intended，取得真实宿主 ID 后写 running；结束后写 completed 和不可变
result。失败也是 completed，但结果明确失败，不据此将任务记 done。
当前 Agent 自行执行时 executor_ref 使用可核对的宿主任务身份，同样记录开始/结束。
completed 后不能改 result，重试用新 ID。结果未知时保留未决操作。

首次建立后，每次保存必须带上一个不可变 workflow 快照：

```bash
node .agents/skills/zeroweb-site-optimizer/scripts/verify-workflow.mjs \
  "$RUN_DIR/workflow-next.json" "$RUN_DIR/workflow-previous.json"
```

退出 0 = 整体目标证据就绪；1 = 合法但未完成（正常执行中通常为 1）；2 = 结构、
转换或证据错误，不可推进。`next` 是阶段建议，不是权限或实际调度：
continue、wait_or_recover、estimate_budget、verify_executor、final_acceptance、replan、stop。
必须实际核对宿主、原始报告、授权及预算。检查器不证明 Agent 已持续执行。

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
| worker 已退出但结果缺失 | 核对专属目录、费用和产物；未知消费保留预留，不能填零 |
| trial 未验证 | 保留 trial，补取终态或恢复 best；不能推进完成 |
| 新 best 或集成基线变化 | 旧报告保留身份，重做受影响目标及最终整体验收 |
| PR 创建/合并结果未知 | 查询服务端真实状态，确认前不重复写入、不派发依赖项 |
| 用户停止或预算不足 | 先保存停止屏障，再取消/收回已有任务，保全 best 与证据 |

新调用前检查“下一完整步骤＋预留”，不得真正超支后才停止；运行中以剩余预算设置
超时，时间到限立即收回。停止时允许记录已有操作的终态、清理和保存，不允许新
实施/审查/合并调用。状态询问回复后继续仍活跃的原任务；明确“只查询、不恢复”
则只读，用户停止优先。可恢复阻塞在原预算内处理，无法解除才形成硬停止。

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
