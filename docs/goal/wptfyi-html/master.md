# HTML、XHTML 与 DOM 接口控制面

**2026-11-21 · DC-2 批六完成（html/semantics 补簇 10 簇整批接入 + 未引号
variant 解析修复，Pass +1,955）**。批一 html/dom +1,888；批二 rendering 通道
+2,551；批三 window 变体源通道 +11；批四 webappapis +1,537；批五 browsers 簇
+40；批六 semantics 补簇 30 目录语料（links 归 html-semantics 套件不收，防跨
套件重复计账）+ 通道加收 23 计分目录 + `.https` 簇内解锁 + case_variants 未引
号值解析修复，净 **+1,955 Pass**（html-syntax 65,314 → 67,269，801 → 1,049 键），
共享键零回归（801 键与批五账逐字一致，修复前后对照零漂移）。通道分母 428 键
6,444 子测试（chrome 过 6,041），执行 231 键（203 verbatim + 28 不齐挂账）+
溢出 17 键诚实记账（16 更名漂移 pin 仍 tentative、1 快照新增）；197 键未执行
逐键归因（非 testharness 形态 110 零子测试 / pin 缺失 56 键 212 / iframe 门
12 键 155 / idlharness 门 3 键 159 / 更名镜像 16）（[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；其余 Fail 面归属：UA 样式表值差（rendering 流域，独立
可修）、iframe 子文档管道（零-web 流域——批四 107 键 + 批五 39 键 + 批六 12 键
155 子测试同一 iframe 内容门，同根因合并待碰头）。

**下一步** 本 goal 域内续推批七：余簇择一整批接入（11-20 快照缺口：rendering
≈9,164 / syntax 6,298 / canvas 4,240；semantics 补簇已批六接入、余量 pin 缺失
56 键随周跑另报）；批六 Fail 战役图为刀位储备首选——popovers 2,744 F+T/66 键
（engine popover 语义域）、command-and-commandfor 318/23 键、interestfor 112/28
键；批五战役图（origin/api 序列化族 215、open-features 特性串族 135）与批四
战役图（navigator.protocol 244 / event-handler 反射面 ~455 / atob 70）续挂。
engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论排期。DC-3 周跑对账需实体
矩阵（catch-all-8）+ rendering 分片首跑入账。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；批五 browsers window/origin +40：[批五证据](evidence/2026-11-21-dc2-batch5-browsers-window.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch5-browsers-window.json)；批六 semantics 补簇 +1,955＋variant 解析修复：[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch6-semantics-complement.json)；余簇缺口（11-20 快照）：rendering ≈9,164 / syntax 6,298 / canvas 4,240（webappapis 3,672 已批四接入余 126 键挂账见批四证据；browsers window/origin 通道分母 234 键已批五接入余 102 键挂账见批五证据；semantics 补簇 428 键已批六接入余 197 键挂账见批六证据） |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。
