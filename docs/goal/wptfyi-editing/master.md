# Editing、Selection 与 Input Events控制面

**2026-11-11 · Active**。DC-1 本地完成：探针陷阱修复 + editing/run 拆 11 分片
（证据 [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md)）。基线口径：11-10
快照 editing/run 0/120 文件（探针陷阱，0 过）、editing/other 12,997 过、selection
2,728 过、input-events 201 过；目标三目录对 11-07 基线净增 130,000。
**卡点** DC-1 的 CI 验证未跑——分片已 push，待 dispatch weekly
（run_wptfyi_coverage）取 CI 证据；本地管道面验证已过（underline+italic 双文件
完整变体入 JSON）。
**下一步** weekly CI 结果回来后：①核 11 片编辑面 CI 入账（重点 slow 簇
multitest/delete/insertparagraph 是否超预算，超了按 form-encode 先例再拆）；
②selection/ 31,681 缺口接语料定分片（DC-2 扩面）；③input-events catch-all 面
201 过已有，缺口 3,177 按失败簇归类。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 本地完成，CI 待验证 | 根因定谳＋修复＋11 片落地，[证据](evidence/2026-11-11-dc1-run-shards.md) |
| DC-2 按簇收敛 | 待执行 | run 面 M1 基线 96,656F/7P（editing-compat 证据），缺口语料按分片归类后开修 |
| DC-3 全量对账 | 待执行 | 验收线 146,184 项通过，证据待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
