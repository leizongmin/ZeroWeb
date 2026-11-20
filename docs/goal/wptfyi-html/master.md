# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-2 批四完成（webappapis 簇整批接入，Pass +1,537）**。批一
html/dom +1,888；批二 rendering 通道落地 +2,551（根因归因修正已定谳：同任务 JS
创建元素不可见才是真根因，`*` 选择器无罪）；批三 window 变体源通道 +11；批四
webappapis 簇语料 46 目录 + html-syntax 通道加收 25 计分目录 + `.https` 命名面
簇内解锁，净 **+1,537 Pass**（html-syntax 63,737 → 65,274，462 → 669 键），共享键
零回归（462 键数与 Pass 63,737 与批三账逐字一致）。执行 207 键全部分母对齐
（首跑实证收口两处溢出：`-manual.https.html` 复合形补入 manual 门、js 源
`META: global=` 收窄——worker-only 源不生成 window 形键）
（[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；其余 Fail 面归属：UA 样式表值差（rendering 流域，独立
可修）、iframe 子文档管道（~1,400，零-web 流域——webappapis 簇 126 未执行键中
107 键同一 iframe 内容门，同根因合并待碰头）。

**下一步** 本 goal 域内续推批五：余簇择一整批接入（11-20 快照缺口：semantics
13,962 / rendering ≈9,164 / syntax 6,298 / canvas 4,240）；批四 Fail 战役图
968（navigator.protocol 244 / event-handler 反射面 ~455 / atob 70）为后续刀位
储备。engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论排期。DC-3 周跑对账需
实体矩阵（catch-all-8）+ rendering 分片首跑入账。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；余簇缺口（11-20 快照）：semantics 13,962 / rendering ≈9,164 / syntax 6,298 / canvas 4,240（webappapis 3,672 已批四接入，余 126 键挂账归因见批四证据） |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。
