# HTML、XHTML 与 DOM 接口控制面

**2026-11-20 · DC-2 批二完成（html/rendering 通道落地，Pass +2,551）＋根因归因修正**。
批一 html/dom +1,888 零回归；批二接入 rendering 语料（pin `31597693` 754 文件齐平）
＋独立执行通道，CI 零编辑。release runner 150 文件 9,691 子测试：Pass 0 → 2,551，
簇缺口 11,717 → ≈9,166，138/150 键分母逐字对齐。晚间探针 6-8 **推翻批二首版
"`*` 选择器漏命名空间"归因**：`*` 匹配无罪（元素级子树 QSA 命中命名空间克隆），
真根因＝**同任务 JS 创建元素对 document 级 QSA/样式求值不可见**（变异管线跨任务
界排空、JS live 视图与 Rust 视图分裂；普通 createElement 同样不可见，非命名空间
特异）→ test-ua-stylesheet helper 同任务 clone→append→compare 全落幽灵空样式
≈4,400 Fail。归因修正已写回批二证据
（[dc2 批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md)）。

**卡点** 解锁 ≈4,400 面的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；其余 Fail 面归属：UA 样式表值差（rendering 流域，独立
可修）、iframe 子文档管道（~1,400，零-web 流域）。

**下一步** 本 goal 域内续推：html/syntax html5lib write-mode 面评估（分母 61
variant 键/浏览器，批一已入 170 variant 键、write-mode 面单 synthetic Fail）；
`.window.js` window 变体通道评估（rendering 2 键 + 全局面）；IDL 成员面战役批二
对账（挂账清单即缺员表）。engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论
排期。DC-3 周跑对账需实体矩阵（catch-all-8）+ rendering 分片首跑入账。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；余簇缺口（11-20 快照）：semantics 13,962 / rendering ≈9,166 / syntax 6,298 / canvas 4,240 / webappapis 3,672 |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。
