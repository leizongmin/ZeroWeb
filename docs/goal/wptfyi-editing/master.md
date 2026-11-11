# Editing、Selection 与 Input Events控制面

**2026-11-11 · Active**。DC-1 CI 验证落定：weekly
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)
editing/run **11/11 片全部成功**，面 **96,284 子测试 / 92,867 过（96.5%）**
（对面 11-07 基线 0/96,745；差 461 为探针/iframe 口径），失败总量 3,414
Fail + 3 Timeout 按片 er9（850）> er8（779）> er2（334）；slow 簇全部在
case_timeout 3600 内完成，**无需再拆**（证据
[dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md)）。
失败簇全量归类完成：1,023 queryCommand\*（inline format + subscript/
superscript 状态语义共享路径 ~807）+ 2,342 innerHTML 长尾 + 49 其他
（[triage](evidence/2026-11-11-dc2-fail-cluster-triage.md)）。
DC-2 selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片（script 300s/case 300s/limit 7200）——本地全面 census
51,695/35,631（selection 本体 ~22.6k 过 vs 旧 CI 面 2.8k，
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** run 42947347500 aggregate 未落（test262 builtins/language 两条
长腿在跑 + 快照回写），净增数字以快照为准。
**下一步** ①快照落后对账净增与邻域回退（DC-3 口径预演）；②DC-2 开修
P1 共享路径：inline format（bold/underline/strikethrough/italic 608）+
subscript/superscript（199）的 queryCommandState 状态语义（~800 子测试，
规范 state/effective command value 按 computed style 判定）；P2 innerHTML
长尾按命令开修（inserttext 227、outdent 173 先行）；③selection bidi
（0/92）、shadow-dom（19/81）、input-events 小桶（3,177 缺口）按产物归类。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片，96,284/92,867） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | selection 扩面已接线；失败簇已归类 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md)：P1 queryCommand 状态语义 ~800 / P2 innerHTML 长尾 2,342；selection bidi/shadow 两簇近零过 |
| DC-3 全量对账 | 待快照落 | 验收线 146,184 项通过 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
