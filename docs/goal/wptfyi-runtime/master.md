# Test262、Encoding 与 Wasm控制面

**2026-11-10 · Active**。DC-1 完成（test262 全量基线 53,166 案 96.39%、CI 三分片、
runner 探针两形态缺陷修复）；DC-2 encoding worker 变体面收口（runner commit
`f57d91099`，同口径对账 +11,728 过）：净增账 ~73,903 / 76,000。

**卡点** 净增 76,000 账：test262 ~51,250 + encoding weekly 自动 +10,925 +
worker 变体面 +11,728 ≈ 73,903，余 ~2,100：wasm（缺口 2,803，未定量）+
encoding 残余 ~1,194（iframe 面 ~330、idlharness 划界、引擎小项）。
engine 挂账 2 项（timer 异常无 window error 报告、unhandledrejection 未实现）
见证据文。

**下一步** ① wasm 缺口 2,803 同口径定量（本地 wasm 通道实跑 vs 分母，
DC-2 收尾）；② test262 归簇收敛（module early-error 133、agent/SAB 108、
createRealm 78、提案面 other ~1,446，
[证据](evidence/dc1-full-baseline-2026-11-10.md)）；③ DC-3 全量对账
（同 revision 归一、验收线 1,393,955）。

[执行契约](../wptfyi-runtime.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [DC-1 全量基线](evidence/dc1-full-baseline-2026-11-10.md)——53,166 案 96.39%，变体/归簇定量，CI 三分片 + filter 多段语义，探针缺陷两形态修复；[通道试跑](evidence/dc1-channel-smoke-2026-11-09.md)；发布快照基线 1,317,955 / 1,398,399 |
| DC-2 按簇收敛 | 进行中 | [encoding 缺口定量](evidence/dc2-encoding-gap-2026-11-10.md)——缺口 12,922 构成；[worker 变体面收口](evidence/dc2-worker-variants-2026-11-10.md)——runner `f57d91099`，+11,728 过、组残余 ~1,194（worker 面 ~72）；test262 归簇定量已落证据；wasm 未定量；已知簇：module 早期错误检出、$262.createRealm（281 案）/agent（112 案）/gc（15 案）、ReferenceError 语义 |
| DC-3 全量对账 | 待执行 | 验收线为 1,393,955 项通过，证据待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
（2026-11-09：`resources/test262/` 为本目标新占用 fetch 面，`wpt-data/third_party/test262/`
vendored 子树为本目标独占，无共享写面冲突。）
