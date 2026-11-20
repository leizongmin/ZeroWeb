# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-2 批三完成（window 变体源通道，Pass +11）**。批一 html/dom +1,888；
批二 rendering 通道落地 +2,551（根因归因修正已定谳：同任务 JS 创建元素不可见才是
真根因，`*` 选择器无罪）；批三把分母生成形键（`.window.html`/`.any.html`）的本地
源窗体接入 html-syntax 与 rendering 两通道（`window_forms` 门 + testharness 引用
检查收窄到 `.html` 形），不加语料不加套件，净 **+11 Pass**（html-syntax 63,728 →
63,737，rendering 2,551 → 2,553），共享键零回归，新键 10 个 den 107/执行 106/
分母逐字对齐 9/10（[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；其余 Fail 面归属：UA 样式表值差（rendering 流域，独立
可修）、iframe 子文档管道（~1,400，零-web 流域）。

**下一步** 本 goal 域内续推批四：webappapis 簇整批接入（五簇之一，缺口 3,672；
语料拉取 + 通道 + 记账，含 `.window.html` 48 键 195 子测试与 `.any.html` 571——
`.window.html` 全局面余 258 键大半在此簇及 browsers/infrastructure）。
engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论排期。DC-3 周跑对账需实体
矩阵（catch-all-8）+ rendering 分片首跑入账。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；余簇缺口（11-20 快照）：semantics 13,962 / rendering ≈9,164 / syntax 6,298 / canvas 4,240 / webappapis 3,672 |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。
