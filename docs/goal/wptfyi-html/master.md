# HTML、XHTML 与 DOM 接口控制面

**2026-11-21 · DC-2 批七完成（canvas offscreen 主线程面整批接入，Pass +1,062）**。
批一 html/dom +1,888；批二 rendering 通道 +2,551；批三 window 变体源通道 +11；
批四 webappapis +1,537；批五 browsers 簇 +40；批六 semantics 补簇 +1,955 + 未引
号 variant 解析修复；批七 offscreen `.html` 主线程面（~1,368 文件此前只执行
worker 变体、`.html` 分母键从未执行）链入 canvas 主套件 + offscreen 顶层死配置
激活 + worker 顶层 2 键 + wide-gamut 语料修复，套件净 **+1,062 Pass**（canvas
1,134 → 2,190、canvas-worker 1,044 → 1,050），共享键零回归（main 1,331 + worker
1,010 键逐键一致）；对账口径净 **+1,046 Pass**（2,163 → 3,209），覆盖分母键
+1,252（2,209 → 3,461）；分母 4,074 键全量对平（manual 门 469 / pin 缺失 135 /
canvas 顶层无扫描链 7 / element 顶层 2）；新增键 Fail 102 战役图：text 65
（TextMetrics getIndexFromOffset/getSelectionRects）、createImageBitmap 选项门
~33（[批七证据](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；其余 Fail 面归属：UA 样式表值差（rendering 流域，独立
可修）、iframe 子文档管道（零-web 流域——批四 107 键 + 批五 39 键 + 批六 12 键
155 子测试同一 iframe 内容门，同根因合并待碰头）。

**下一步** 本 goal 域内续推批八：余簇择一整批接入（11-20 快照缺口：rendering
≈9,164 / syntax 6,298；canvas 已批七接入、余量 pin 缺失 135 键 + manual 门 469
键随周跑另报）；批六 Fail 战役图续为刀位储备首选——popovers 2,744 F+T/66 键
（engine popover 语义域）、command-and-commandfor 318/23 键、interestfor 112/28
键；批七战役图（text 65 / createImageBitmap 门 ~33）与批五（origin/api 序列化族
215、open-features 特性串族 135）、批四（navigator.protocol 244 / event-handler
反射面 ~455 / atob 70）续挂。engine 刀（同任务可见性）与 UA 样式值差刀按碰头
结论排期。DC-3 周跑对账需实体矩阵（catch-all-8）+ rendering 分片首跑入账。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；批五 browsers window/origin +40：[批五证据](evidence/2026-11-21-dc2-batch5-browsers-window.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch5-browsers-window.json)；批六 semantics 补簇 +1,955＋variant 解析修复：[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch6-semantics-complement.json)；批七 canvas offscreen 主线程面 +1,062（对账口径 +1,046，覆盖键 +1,252）：[批七证据](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.json)；余簇缺口（11-20 快照）：rendering ≈9,164 / syntax 6,298（webappapis 3,672 已批四接入余 126 键挂账见批四证据；browsers window/origin 通道分母 234 键已批五接入余 102 键挂账见批五证据；semantics 补簇 428 键已批六接入余 197 键挂账见批六证据；canvas 4,240 已批七接入余 pin 缺失 135 键 + manual 门 469 键挂账见批七证据） |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。
