# Test262、Encoding 与 Wasm控制面

**2026-11-09 · Active**。DC-1 通道接入完成并抽样验证（commit `79862fbb3`，push `a212e5766`）。
\
**卡点** test262 通道已可跑（53,441 案 vendored @ WPT_REV 31597693，变体名与 wpt.fyi 对齐），
全量基线未跑；encoding / wasm 两域通道复用既有 testharness-encoding / testharness-wasm-core
未动。引擎挂账 2 项（timer 异常无 window error 报告、unhandledrejection 未实现）见证据文。\
**下一步** ① test262 全量基线（分片跑出分类通过率落 evidence，预算 ~2.1h 线性外推，按
目录 `FILTER=` 分片入 CI）；② DC-2 按簇收敛：module 早期错误、$262.createRealm/agent 面、
ReferenceError 语义先归簇定量，encoding/wasm 挂账簇同口径；③ DC-3 全量对账前核对上游
漂移（本地 pin vs universe ~350-600 文件）。

[执行契约](../wptfyi-runtime.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 通道接入完成 | [DC-1 试跑证据](evidence/dc1-channel-smoke-2026-11-09.md)——同步/strict/async/负例/module 正反例抽样全过，language/types 113/113（109P/4F）；发布快照基线 1,317,955 / 1,398,399 |
| DC-2 按簇收敛 | 待执行 | 已知簇：module 早期错误检出、$262.createRealm（281 案）/agent（112 案）/gc（15 案）、ReferenceError 语义；缺口按固定 revision 的路径和变体归类 |
| DC-3 全量对账 | 待执行 | 验收线为 1,393,955 项通过，证据待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
（2026-11-09：`resources/test262/` 为本目标新占用 fetch 面，`wpt-data/third_party/test262/`
vendored 子树为本目标独占，无共享写面冲突。）
