# HTML、XHTML 与 DOM 接口控制面

**2026-11-22 · DC-3 批十六：对账 run 44985566993 五分片逐键收账完成——infra 82→674／semantics +1,959／browsers +40／speculative 43P／syntax +43／xhtml·dom·rendering·CE·SD 持位全兑现，唯 webappapis 438 距本地 face 1,099＝CI 专有缺档两文件（interfaces/html.idl 运行期 fetch＋common/sab.js META，静态扫描盲区）已补档；批十六码待派 coverage run 承接 +1,067 修正并作 DC-3 最新周跑产物**。
批一 html/dom +1,888；批二 rendering 通道 +2,551；批三 window 变体源通道 +11；
批四 webappapis +1,537；批五 browsers 簇 +40；批六 semantics 补簇 +1,955 + 未引
号 variant 解析修复；批七 offscreen 主线程面 +1,062（对账 +1,046）；批八 html5lib
驱动三件套 +118（1,049 键零回归）；批九 `testharness-html-infra` 新通道
html/infrastructure + 五小簇 +623（193 键分母全量对平）；批十 inline 脚本
`pipe=sub` 臂补 `{{GET[name]}}` 注入（infra 套件 A/B Pass 550→665＝scheme +20/
history +6/eventsource#url +3，对批九基线净 +42，其中 +13 为 messagechannel
.any.worker.js 键级截断恢复抖动带，零回归）；批十一 syntax template 子树定向腿
+42（19 键/329 子测试全执行，19/19 键计数精确对平，22 键宽估修正为 19 键）；
批十二 speculative-parsing 族新通道（127 键分母新入账，chrome 过 118）：
126 执行键（负案 popup skip 1）**50 Pass / 76 TO、墙钟 ~15 min、A/B 两轮逐键
零漂移**——无扫描器下 pass 面为引擎 loader 怪癖（iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效＋template 取数不惰性，探针实证），
leg-2 engine 修后此面重排（[批十二证据](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.md)）。
批十二勘误（批十三重跑实测）：route_speculative_stash take 臂漏设 body →
JS fetch text() 恒空 → 7 键假 Pass，重跑 126 执行键 **43 Pass / 83 TO**
（A/B 两轮零漂移），净记账 **50→43**，7 键全 document-write 形（真实头
序列比较生效后两 put 头序列不等 → TO 0 Fail；嵌套 write 管道头序列差为
engine 碰头输入新增一形）；批十三 query-encoding 族四端点备料腿
（resource.py/blank.py/stash.py/css-tmpl.py 上游 1:1 + green-2x2.png 与
media 5 资产补档）：infra 全量 A/B 132 键 before 665/352/151 → after
**674/376/138 净 +9 P**，18 变更键全 query-encoding 面、其余 114 键
（含 messagechannel 抖动键）逐键零漂移（xhr +3P/eventsource +3P/loading
+3P/css 8 TO→F——端点上线前 404 挂起 TO、上线后 poll 收敛断言面曝光，
css 余 97 TO 待 stylesheet 载入链碰头刀）
（[批十三证据](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.md)）。
批十四备料核查（无码笔，DC-3 对账前）：批九 pin_missing 6 键 gh api 按目录
实证 pin 315976933 内不存在（collections/scripting/shared-array-buffers/
obsolete 四目录列表均无）＝上游漂移 bucket（同批五 13 键处理），pin 内不可
拉取、随周跑另报；TO 面 83 键勘误后构成＝document-write 45 + page-load 38
（7 翻转键全 dw 形，page-load 面持 38 不动；余量按批十二证据 img×text/plain
载入不完成＋头序列不等两形归因）。
批十四 CI 缺档腿（无 Rust 码笔）：11-21 快照 infra zw_passes 82 对本地 674
的差定谳＝分片语料缺 9 支撑文件（sab/utils/get-host-info/dispatcher/
subset-tests-by-key/IndexedDB support/structured-clone battery 三件；本地由
其他 goal 通道带入、CI 分片作业只跑本 goal fetch）——缺档形＝load support
os error 2（messagechannel/SAB 大簇）＋ subsetTestByKey 未定义 0 注册 TO
（query-encoding 5 页 × 16 include ＝ 80 键全灭）；三先期假设（限流洞/
截断/端点缺位）全证伪（fetch 失败 0、墙钟 106s 帽内、检出批九码）。62 脚本
+10 fetch_raw 补档（含本地同缺的 wpt-logo svg）；本地同码复跑 131/132 键
逐键零漂移（唯一漂移键 messagechannel.any.worker.js＝批十已知抖动带）。
select/options-length-too-large.html 的 0 注册 TO 本地/CI 同形（三份全量
产物＋过滤单跑核对均同一 Timeout，初稿「速度敏感嫌疑」表述勘误）＝engine
select 慢路径域稳定 TO，归 page-load TO 面候选形。补档后另派 coverage-only
周跑取新快照做 DC-3 对账
（[批十四证据](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.md)）。
批十五 CI 分片三通道缺档定谳（无 Rust 码笔）：控制点 44972755575 快照
infra 82 逐字复现批十四缺档形＝批十四「待实证」兑现；控制快照组基线
（zw_passes 1,914,204/2,276,004＝84.1%，infra 82、webappapis 3、browsers
217、semantics 8,508、rendering 2,793、syntax 2,747、xhtml 25,606、dom
59,634）。根因＝CI fetch 步只拉分片 `fetch:` 清单、套件直跑不经 Makefile
依赖——① syntax-dom 缺批四/五/六三 fetch 目标（run 实测 webappapis 语料
1 文件、补簇全缺＝三通道快照挂账全此形）；② speculative 缺 /common/utils.js
（catch-all-4 实测 126 F/0 P/0 TO「token is not defined」，本地同码 43 P/
83 TO）；③ 57/58/59 支撑文件闭包 14 文件（16 fetch_raw 位点，计分域实跑页
引用扫描）。方法论勘误：朴素「本地语料−CI 键」差集超计（rendering 556/
dom 259 误判缺档），按枚举规则模拟键域后 rendering 152=152 全对齐＝零行动，
56 误改已回退；py-routes 引用非缺档。验证：bash -n ×4＋GITHUB_TOKEN 幂等
重跑失败 0＋discovery 矩阵 syntax-dom 携全 5 目标、53 分片不变。
44983026495（批十五前码）取消（缺四通道必中间态；infra 通道输入同码＝
批十四验证力全额承接），改派 44985566993（cde99ad26，04:33Z）
（[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md)）。
批十六 DC-3 对账收账（44985566993 五分片产物逐键 A/B，无 Rust 码笔）：
infra 82→674（132 键与批十三本地 after 逐项相同＝批十四兑现）、semantics
8,032→9,991（批六 231 delta 键逐键零漂移＝+1,959 face 兑现）、browsers
44→84（批五 +40 逐键兑现；agent-clusters ~55／open-features 13／
on-new-window 零注册 TO 与本地同形＝行为面挂账不重计）、syntax
2,748→2,791（speculative 勘误后 CI 首跑 43P/83TO 同形＋html5lib/template
基底零漂移）、xhtml 25,606／dom 59,644／rendering 2,795／CE·SD 11,361
持位；唯 webappapis 3→438 距本地 face 1,537 缺 1,099 定谳三形：
event-handler 8 页运行期 `fetch("/interfaces/html.idl")` 失败零注册
（−919 P，批十五静态 `<script src>` 闭包扫描盲区）、structured-clone.any.js
缺 common/sab.js（−148 P，os error 2；62 脚本不在本分片清单）、
base64.any.js 双 scope 合成差异（−32 P，非缺档）。批十六 fix＝57 脚本
两 `fetch_raw` 补档（bash -n＋GITHUB_TOKEN 幂等重跑 FAILED_FETCHES=0），
预期 face 438→~1,505。邻居观察：dom/ranges −13＝2 键子测试注册抖动形，
归 wptfyi-dom 账本
（[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md)）。
DC-3 preflight 三腿收账：syntax 腿与批八 after 字节级全同（1049 键
67,387/10,656/286 四项逐一相同）；infra 腿唯一漂移= messagechannel.any.worker.js
单键负载抖动（Pass 102→29 键级截断，键集不变）；rendering 腿对批二基线共享键
漂移 -62（legend-display -56、fieldset-display -11、video-aspect-ratio -1，
aspect-ratio 族 +6 回冲；+2 新键为批三 fetch 带入非漂移），归因渲染流域
（[批十证据](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.md) +
[批十一证据](evidence/2026-11-22-dc2-batch11-syntax-template-leg.md)）。
DC-3 周跑接入预检完成：discovery 矩阵实证五通道落位（html-infra+rendering 入
catch-all-3、实体矩阵随零头打包漂至 catch-all-9、template 腿随 syntax-dom 手调
分片），修正 syntax-dom 预算截断 3600→13,200s（CI 11-20 快照实证 html/syntax
仅 40/439 文件入账；本地 93 min 实测）；批十二后矩阵重跑实证 speculative
通道零 CI 编辑入 **catch-all-4**（伴 import-maps，本地 ~15 min 对 5400s 预算
充裕）、实体矩阵漂至 catch-all-10（编号不锁）
（[接入预检证据](evidence/2026-11-22-dc3-weekly-integration-preflight.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；批八新增同域刀位：Node.normalize 上移共享 Node 原型
（js_dom_shim 面，363 F）、document open/write 管道挂起（engine 域，110 键 TO；
批十新证 base-href 面「This document is an XML document」doc.write XML 拒绝
同域）；批九新增刀位：带查询串样式表载入链挂起（engine stylesheet fetch/加载
管域，?include=css 105 TO）、HTMLAllCollection 可调用语义（40 F）、
structured clone/transfer 深面（47 F）——engine/JS 域待碰头。批十
query-encoding 刀位分解后余量归属：非特殊 scheme 文档编码越界 ~54 F +
`'href' in elm` 原型面 + search setter（js_dom_shim/bridge 面待碰头）、UA 样式
background 属性 8×3 F（rendering 流域）。批十三已把 resource.py/blank.py/
stash.py/css-tmpl.py 四端点备料落地（infra 面 +9 P；css 面 8 TO→F 后余
97 TO + 8 F 落 CSSOM 门；收益余量待 iframe/doc.write/CSSOM 碰头刀兑现）。rendering 腿 -62 漂移待渲染流域定向定谳与修复（不入本流回归账）。
批十二新增 engine 刀位（speculative-parsing 战役图，待碰头）：预加载扫描器
刀（leg-2 主刀，兑现上限 +118）、iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效（与批八 write 挂起同域不同形）、
template 内容取数不惰性（批十一 223 F owner-document 面的取数侧同源）、
非图片响应 img 载入不完成 → 页面 load 不触发（img 管道域）、嵌套 write
管道两 put 头序列不等（批十三勘误后 7 键 TO 形，勘误前被 take 空体遮蔽）。

**下一步** DC-3 收口两步：① 批十六码（57 脚本两 fetch_raw 补档）提交后
派 coverage-only run，承接 webappapis face 修正（438→~1,505，+1,067）并作
DC-3 最新周跑产物；② 下轮取该 run 的 wptfyi-coverage aggregate 快照做
目标级对账入账——预期形：infra 82→~676、webappapis 3→~441→批十六后
~1,508 face、browsers 217→~257、semantics 8,508→~10,467、syntax
2,747→~2,790、xhtml 25,606／dom 59,634／rendering 2,793 持位（rendering
基线仍待渲染流 -62 漂移定谳后再锁），对照验收线 152,096 记余量并按
缺口主序（UA 样式面 ≈4,400、popovers 战役图 2,744、rendering ≈9,164）
更新 DC-2 余簇账。注意控制点快照 coverage-2026-11-22.json 已被
44985566993 aggregate 在飞覆写（控制值存批十五/批十六证据，不丢）。批十二 leg-2 备料
顺序：engine 扫描器刀待碰头排期，其前可本域先行——TO 面 83 键（勘误后
＝document-write 45 + page-load 38）img/script 载入挂起根因定谳（探针已证
img×text/plain 空体载入不完成一形）与 pass 面逐类归因表（批十二证据）
作为 engine 碰头输入。批九余量挂账续推：service/sharedworker 生成形通道
（messagechannel 274 子测试最大单项＝engine worker lane 域待碰头）。
批十刀位余量：.py fixture 备料已批十三落地 + shim 编码越界刀（待碰头）。
刀位储备
首选仍为批六战役图——popovers 2,744 F+T/66 键（engine popover 语义域）、
command-and-commandfor 318/23 键、interestfor 112/28 键；批八挂起面两刀
（normalize / document.write）与批七战役图（text 65 / createImageBitmap
门 ~33）、批五（origin/api 序列化族 215、open-features 特性串族 135）、
批四（navigator.protocol 244 / event-handler 反射面 ~455 / atob 70）续挂。
engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论排期。syntax 套件周跑
墙钟 +80 min（148 页级 TO 各 ~30s）随两刀落地消解。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；批五 browsers window/origin +40：[批五证据](evidence/2026-11-21-dc2-batch5-browsers-window.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch5-browsers-window.json)；批六 semantics 补簇 +1,955＋variant 解析修复：[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch6-semantics-complement.json)；批七 canvas offscreen 主线程面 +1,062（对账口径 +1,046，覆盖键 +1,252）：[批七证据](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.json)；批八 html5lib 170 变体键 +118（对账 +118，覆盖键 +170，零回归）：[批八证据](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.json)；批九 infrastructure+五小簇 132 键 +623（覆盖键 +132，193 键分母对平）：[批九证据](evidence/2026-11-21-dc2-batch9-infra-small-clusters.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch9-infra-small-clusters.json)；批十 {{GET}} 注入刀 infra 净 +42（对批九基线，含抖动带 +13；A/B changed 键 12 零回归）：[批十证据](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.json)；批十一 template 子树 19 键 +42（计数精确对平）：[批十一证据](evidence/2026-11-22-dc2-batch11-syntax-template-leg.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch11-syntax-template-leg.json)；批十二 speculative-parsing 新通道 127 键分母新入账（chrome 118）126 执行键 +50 Pass / 76 TO（A/B 零漂移，loader 怪癖面归属见证据）：[批十二证据](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.json)；批十三 query-encoding 四端点备料腿 infra 净 +9 P（132 键全量 A/B 665→674，18 变更键全 query-encoding 面、其余零漂移，定向三轮互证）＋批十二 speculative 勘误 50→43（take body 缺设致 7 假 Pass，重跑 43/83 零漂移）：[批十三证据](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.json)；批十四 CI 缺档定谳＋62 脚本支撑文件补档（无 Rust 码笔，本地 131/132 键零漂移）：[批十四证据](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.json)；批十五 CI 分片三通道缺档补档（syntax-dom 三 fetch 目标＋speculative utils.js＋57/58/59 闭包 14 文件；rendering/dom/syntax 全对齐零行动，56 误改回退）：[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.json)；批十六 DC-3 对账收账＋CI 专有缺档两文件补档（webappapis 438/1,537 缺 1,099 定谳＝html.idl 运行期 fetch −919＋sab.js −148＋base64 双 scope −32；infra/semantics/browsers/syntax 四通道 face 逐键兑现）：[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.json)；余簇缺口（11-20 快照）：rendering ≈9,164 / syntax 余量=pin 缺失 26 + 驱动变体 3 + 无扫描链 43-19(template 批十一落地)=24 键（speculative-parsing 127 键此前不在 syntax 余量账内，批十二新入账）（webappapis 3,672 已批四接入余 126 键挂账见批四证据；browsers window/origin 通道分母 234 键已批五接入余 102 键挂账见批五证据；semantics 补簇 428 键已批六接入余 197 键挂账见批六证据；canvas 4,240 已批七接入余 pin 缺失 135 键 + manual 门 469 键挂账见批七证据；syntax parsing+fragments 262 键已批八接入余 72 键挂账见批八证据；infrastructure+五小簇 193 键已批九接入余 61 键挂账=内容门 42 + lane 缺位 4 + .xhtml 1 + pin 缺失 6 + 记账 8，见批九证据） |
| DC-3 全量对账 | 批十六对账收账＋缺档补档落码，批十六 coverage run 待派/在飞 | 验收线 152,096 项通过；run 44985566993（批十五码）五分片逐键收账完成：infra 82→674／semantics +1,959／browsers +40／syntax +43（speculative 43P/83TO 勘误形兑现）／xhtml·dom·rendering·CE·SD 持位全兑现，唯 webappapis 438 距本地 face 1,099＝两文件 CI 专有缺档＋base64 双 scope 差异，57 脚本补档落码待派 run 承接：[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.json)；控制点 44972755575（infra=82 形）与 44985566993 预期形记档见[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md)；接入预检五通道落位＋syntax-dom 预算修正 3600→13,200s：[接入预检证据](evidence/2026-11-22-dc3-weekly-integration-preflight.md)；实体矩阵随打包漂至 catch-all-9/10（编号不锁），rendering 通道入 catch-all-3 已首跑入账（2,795 持位）；批十二后 discovery 重跑（147 套件）：speculative 通道入 catch-all-4 兑现 43 P |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。2026-11-21 批八战役图新增两刀同性质待碰头：
Node.normalize 上移共享 Node 原型（js_dom_shim 面，363 F）、document open/write
管道挂起（engine 域，110 键 TO）。2026-11-21 批九：workspace clippy `-D warnings`
门禁被渲染流域提交破坏（f25940399 css-parser doc_lazy_continuation 缩进、
21e0ea83d/22787ab21 style-system computed_style 缺字段文档注释 + apply let-else、
layout-engine too-many-arguments ×2 需签名重构）——渲染流域，待该流或碰头修复；
本批门禁降级为 scoped `cargo clippy -p zero-wpt-runner --all-targets`（本批文件
零警告；批十复扫同样 25 条既有警告全在渲染流域文件，runner 本域零警告），
workspace 门禁恢复前各流提交均需自行 scoped 核验本流域文件。
2026-11-22 凌晨：wptfyi-css 流 stale 工作树提交 ee2ad7855 连带删除批九全部交付
（模块/接线/Makefile target/fetch 脚本/两份 evidence/master.md 回退批八态，非有意
回滚，同 a8de9c124 自记事故型第二起）——已整批还原（f76250c81，blob 与原提交
逐一相同）+ 批十一准备提交 rebased（a62d221df）；跨流 stale 回写已两起，建议
碰头议工作树钉扎约定。同窗口 DC-3 preflight 三腿（本地 test-guard 链）发现
跨流 engine 漂移形：rendering 套件对批二基线**共享键 -62 Pass**（legend-display
-56、fieldset-display -11、video-aspect-ratio -1、aspect-ratio 族 +6 回冲；
批十证据含逐键 A/B），漂移键集中 computed display/UA 呈现域——批三后渲染流
engine 提交 3f05d6acf/21e0ea83d/22787ab21/f25940399 在途（11-21 19:33-22:00 入
HEAD，23:42 runner 二进制内嵌），归因渲染流域待该流定向定谳，不入本流回归账；
syntax 套件字节级零漂移（批八 after 四项 totals 逐一相同）、infra 套件仅
messagechannel.any.worker.js 单键负载抖动（Pass 102→29 键级超时截断，键集
不变，批十后跑 115 恢复带 +13 抖动带宽）——两套件无漂移面。2026-11-22 批十二
战役图新增 engine 刀位三形（探针实证，待碰头）：iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效（批八 write 挂起同域不同形）、
template 内容取数不惰性（template 资源照常 fetch）、非图片响应 img 载入永不
完成 → 页面 load 不触发（76 TO 主构成）——speculative 50 Pass 面依赖前两形，
engine 修后此面重排；本流不动 engine，记碰头输入。
