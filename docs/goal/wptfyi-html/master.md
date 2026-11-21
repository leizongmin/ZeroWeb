# HTML、XHTML 与 DOM 接口控制面

**2026-11-21 · DC-2 批九完成（html/infrastructure + 五小簇语料接入，Pass +623）**。
批一 html/dom +1,888；批二 rendering 通道 +2,551；批三 window 变体源通道 +11；
批四 webappapis +1,537；批五 browsers 簇 +40；批六 semantics 补簇 +1,955 + 未引
号 variant 解析修复；批七 offscreen 主线程面 +1,062（对账 +1,046）；批八 html5lib
驱动三件套 +118（parsing 61 个 .dat fetch_raw + 投递改写 `data-run-type` 透传，
1,049 键零回归）；批九新通道 `testharness-html-infra` 首次接入 html/infrastructure
+ obsolete/links/meta/scripting/select 五小簇搭车（pin 315976933 全集，26 目录 +
根级散文件）：132 键 / 1,158 子测试真执行，**Pass 623** / Fail 381 / Timeout 152 /
PreconditionFailed 2，22 键全绿，`?include=url` 五编码页全过；193 键分母全量对平
——内容门挂账 42 键 197 子测试（iframe 族 33 + worker/open 族 9）、
service/sharedworker 生成形 lane 缺位 4 键 282 子测试、.xhtml 1 键 20、pin 缺失
6 键 18、0 子测试记账 8；通道不相交无重复计账（[批九证据](evidence/2026-11-21-dc2-batch9-infra-small-clusters.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；批八新增同域刀位：Node.normalize 上移共享 Node 原型
（js_dom_shim 面，363 F）、document open/write 管道挂起（engine 域，110 键 TO）
同待碰头；批九新增刀位：带查询串样式表载入链挂起（engine stylesheet fetch/加载
管域，?include=css 105 TO）、HTMLAllCollection 可调用语义（40 F）、query-encoding
scheme/属性编码序列化差（139 F）、structured clone/transfer 深面（47 F）——
前两刀 engine/JS 域待碰头，序列化差本 goal 域内可修；其余 Fail 面归属：UA 样式
表值差（rendering 流域，独立可修）、iframe 子文档管道（零-web 流域——批四
107 键 + 批五 39 键 + 批六 12 键 + 批九 42 键 197 子测试同一内容门，同根因合并
待碰头）。

**下一步** 本 goal 域内续推批十：余簇择一整批接入（11-20 快照缺口：rendering
≈9,164 / syntax 余量=pin 缺失 26+驱动变体 3+无扫描链 43 键随周跑另报）；批九
余量挂账：service/sharedworker 生成形通道（messagechannel 274 子测试最大单项）、
query-encoding 序列化差刀（139 F 本域可修）；testharness-html-infra 通道需接入
DC-3 周跑（对账面 +623 基数固化）。刀位
储备首选仍为批六战役图——popovers 2,744 F+T/66 键（engine popover 语义域）、
command-and-commandfor 318/23 键、interestfor 112/28 键；批八挂起面两刀
（normalize / document.write）与批七战役图（text 65 / createImageBitmap 门
~33）、批五（origin/api 序列化族 215、open-features 特性串族 135）、批四
（navigator.protocol 244 / event-handler 反射面 ~455 / atob 70）续挂。engine 刀
（同任务可见性）与 UA 样式值差刀按碰头结论排期。DC-3 周跑对账需实体矩阵
（catch-all-8）+ rendering 分片首跑入账；syntax 套件周跑墙钟 +80 min（148 页级
TO 各 ~30s）随两刀落地消解。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；批五 browsers window/origin +40：[批五证据](evidence/2026-11-21-dc2-batch5-browsers-window.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch5-browsers-window.json)；批六 semantics 补簇 +1,955＋variant 解析修复：[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch6-semantics-complement.json)；批七 canvas offscreen 主线程面 +1,062（对账口径 +1,046，覆盖键 +1,252）：[批七证据](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.json)；批八 html5lib 170 变体键 +118（对账 +118，覆盖键 +170，零回归）：[批八证据](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.json)；批九 infrastructure+五小簇 132 键 +623（覆盖键 +132，193 键分母对平）：[批九证据](evidence/2026-11-21-dc2-batch9-infra-small-clusters.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch9-infra-small-clusters.json)；余簇缺口（11-20 快照）：rendering ≈9,164 / syntax 余量=pin 缺失 26 + 驱动变体 3 + 无扫描链 43 键（webappapis 3,672 已批四接入余 126 键挂账见批四证据；browsers window/origin 通道分母 234 键已批五接入余 102 键挂账见批五证据；semantics 补簇 428 键已批六接入余 197 键挂账见批六证据；canvas 4,240 已批七接入余 pin 缺失 135 键 + manual 门 469 键挂账见批七证据；syntax parsing+fragments 262 键已批八接入余 72 键挂账见批八证据；infrastructure+五小簇 193 键已批九接入余 61 键挂账=内容门 42 + lane 缺位 4 + .xhtml 1 + pin 缺失 6 + 记账 8，见批九证据） |
| DC-3 全量对账 | 待执行 | 验收线 152,096 项通过；实体矩阵已接周跑 catch-all-8，rendering 通道待 catch-all 首跑入账 |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。2026-11-21 批八战役图新增两刀同性质待碰头：
Node.normalize 上移共享 Node 原型（js_dom_shim 面，363 F）、document open/write
管道挂起（engine 域，110 键 TO）。2026-11-21 批九：workspace clippy `-D warnings`
门禁被渲染流域提交破坏（f25940399 css-parser doc_lazy_continuation 缩进、
21e0ea83d/22787ab21 style-system computed_style 缺字段文档注释 + apply let-else、
layout-engine too-many-arguments ×2 需签名重构）——渲染流域，待该流或碰头修复；
本批门禁降级为 scoped `cargo clippy -p zero-wpt-runner --all-targets`（本批文件
零警告），workspace 门禁恢复前各流提交均需自行 scoped 核验本流域文件。
