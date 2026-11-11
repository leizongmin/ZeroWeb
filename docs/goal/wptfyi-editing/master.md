# Editing、Selection 与 Input Events控制面

**2026-11-11 · Active**。DC-1 落定：11-11 weekly 快照已回写（上游 rev
77787fe5fe），goal 目录合计 **109,059/154,595**——对 11-07 基线 16,184
净增 **+92,875**；editing/run 92,867/96,745（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)
11/11 片）、editing/other 12,997/16,358、selection 仍旧面 2,728。
DC-2 P1 读面修复落地：六 inline format 命令 queryCommandState/Indeterm
按 spec state 语义重写（part01c `__zw_ce_format_query`：computed style
effective command value + 含域 formattable 全命中），同树前后二进制对跑
**+114 净增零回归**（面 10,420→10,534；innerHTML 族两轮恒 727 证 apply
面零扰动）；残余 696 state/indeterm 失败经诊断探针实测为 apply 缺口下游
（跨段落 wrap / 跨边界 merge 臂缺失）→ 归 P2（证据
[dc2-p1-format-state](evidence/2026-11-11-dc2-p1-format-state.md)，含
triage 归因修正）。失败簇归类（1,023 queryCommand + 2,342 innerHTML + 49
其他）见 [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md)。
selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片——本地 census 51,695/35,631（selection 本体 ~22.6k 过
vs 旧 CI 面 2.8k，
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞；距 146,184 验收线差 **37,125**，大头在 selection deep
面下周跑生效（census 口径 +~33k）与 input-events（201/3,378）等 0 覆盖面
接线。
**下一步** ①DC-2 开修 P2 apply 面：bold 跨段落 wrap + 跨边界 merge
（spec bold action vendored 移植，outdent 臂 b49bfe19e 先例）——同根带动
innerHTML bold 转换 197 与 state 下游 624/indeterm 72；②P2 innerHTML
长尾按命令开修（inserttext 227 先行；outdent 实应用臂已落 main 待下周
对账）；③selection bidi（0/92）、shadow-dom（27/113）、input-events
小桶按产物归类开修；④下周快照落做 DC-3 同口径对账（净增 ≥130,000 +
邻域回退核对）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面落地（+114 零回归）；P2 apply/innerHTML 开修中；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p1-format-state](evidence/2026-11-11-dc2-p1-format-state.md)：残余 state/indeterm 属 apply 面（跨段落 wrap / 跨边界 merge） |
| DC-3 全量对账 | 对账口径就绪（净增 +92,875，距线 37,125） | coverage-2026-11-11 快照 + 验收线 146,184 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
