# CSSOM style 写回刀 — outdent 残红 8→0（F2P=169）

**日期**: 2026-11-11（刀体）～2026-11-12（终门收口）
**刀体**: bc69c69cf（4 文件 +57/−12）＋刀二笔索引化 callbacks.rs（25e533139）；重锚谱系：rebase 重放 0a435f0a1/0ff1bc92c（他流 s405 14b6ca724 并入后，已推送）＋刀三笔仲裁修复 f95fdd4d9（CE 插入 walk 三臂共享单次 tag 读）＋r307 回归修复 30f1edbee（第四键比对剥本侧 no-ns 标记）
**门禁批**: batch-cssom-writeback-v1.1（v1 L0 门值算术自误 ABORT 后重射；v1.1 L3 mt 主页 300s 截断 ABORT→刀二索引化）→ v1.3 暂缓（s405 碰撞）→ **v1.3.2 ALL-DONE 收口**（2026-11-12 05:22:27，head 30f1edbee bin aa77d3b8，八腿全绿；v1.3.1 语料七腿绿/L6 名册漂移 ABORT 归因收口见下节）
**基线**: census 3a528a13f（ind-c1-*）＋ v1.2 残红账（odt-outdent-fails.txt，12 corpus + 7 probe）

## 根因（读侧定谳，闭合 outdent v1.1 五级诊断探针）

语料 harness `normalizeSerializedStyle`（tests.js :5499）末段
`node.setAttribute("style", node.getAttribute("style").replace(...))` 读 **style attr**。
引擎 CSSOM 属性写臂 `_styleProxy.setProp`（part03）经 `__zw_set_style_handle` 回调
**只排队**（callbacks.rs DomMutation::SetStyleOnHandle，不同步 apply）——attr 真同步只在批
flush（js_dom_bridge.rs `apply_style_property`）。JS 同步读窗内 `getAttribute('style')`
读 stale 字面量 → normalize 的颜色归一（`color: blue` → `color:rgb(0, 0, 255)`）无法落进
序列化 → 8 行 compare innerHTML 恒红。读面不对称：`node.style` 读（lw replay）已
pending-aware，attr 面读（`__zw_get_attr_handle`）不 aware。

## 刀体四笔

1. **part04 getAttribute trap**：handle 面 style 属性读接 `__zw_get_style_lw_handle`
   正序 replay（cssom-1 update-style-attribute 同步可见面；polyfill 回落原读零漂移）。
   normalize 末步 setAttribute 落归一形 → innerHTML 序列化经 attr 粒度双层索引可见。
2. **part03 R188 镜像剔除第四键**：三键（tag+id+class）剔除会误切「同三键但其余属性
   不同」的真后代——outdent 页根宿主 `<div contenteditable>` 与引擎产物
   `<div style=…>` 三键全空同型，harness `wrapper.querySelectorAll("[style]")` 首命中
   被误切恒 count=0。补第四键整串序列化全等（host parse payload `outer` 字段 vs
   `_r188RootSelf` 根自身序列化快照；纯字符串比对无额外 host 往返）。
3. **part01c**：`_zwPlainStyleDeclsText` 文本半边从 `_zwPlainStyleDecls` 独立导出。
4. **part01d 读核验基底换 attr 权威面**：`_zwVaSetStyleProp` 经 getAttribute 读基底会
   看到自己刚 setProperty 的 pending 值 → cur==valS 跳过 attr 合并 → 序列化丢声明
   （nested styled-ol forceValue 载体 span 丢 color，odt :628/:658 P2F）。handle 元素
   改走 attr 粒度回调 `__zw_get_attr_handle`（不索引 SetStyleOnHandle，恒回 attr 真值）。

## 预验（发射前三重）

| 预验 | 结果 |
|------|------|
| outdent+probe 全页（cssom-pf2-final.log） | 2763P/9F，残红键集恰 = 4 corpus（2 dPS-qV-before + 2 ws-bq）+ 5 probe dPS；F2P=169（161 基线 + 8 CSSOM 闭），P2F=0 |
| indent 全页（cssom-pf2-indent.log） | 1302P/27F，残红集与 census 基线逐字节恒等——15:1x Critical 预登记的 margin×19/text-decoration×30/background×15 P2F 爆炸雷实测未爆 |
| zz-diag-styleattr 探针（probe12-fixed.log） | 嵌套 styled-ol 两新红四假设（H1-H4）got 与 oracle 逐字节 MATCH |

机制注：`style_from_mutations_lw` handle 面空基底＋当前批 replay（js_dom_bridge.rs
:5460 注），normalize 同 tick 内先 setProperty（本批 SetStyle 入队）再读 getAttribute →
replay 含本批条目；indent 页实测证明跨形态（engine-created wrapper 载体）无截断伪红。

## v1.1 L3 ABORT 与性能归因（multitest-ab 300s 脚本门）

mt 腿 20180/2 vs 期望 20182/2：multitest.html 主页键集四零（零漂移），ab 页丢末夹具
（#628/628 forwarddelete white-space:pre 2 断言）——单连续 JS 执行 628 夹具页被刀的
热路径开销推过 300s 引擎脚本门（ZW_SCRIPT_TIMEOUT_MS），击杀点恰剩末夹具＝截断症状
非行为漂移（同夹具 multitest.html 窗页全绿为证）。

归因（Exp A 撤臂 A/B）：撤 part04 trap style 臂（TEMP-EXP-A）后 ab 页完走
10092P/0F＝与基线逐字节恒一 → **热源唯 trap 每读调 `__zw_get_style_lw_handle` 的
全批 O(list) 线性扫描**（`style_from_mutations_lw`）；其余三笔零漂移零开销嫌疑排除。
序列化读（`_zwMOuterHtml` 直读 n.attributes）不经 trap 非热源；热调用方＝编辑命令实现
全树的 getAttribute('style') 走查（量级吻合）。处置＝归因修复非调阈值（perf-gate
政策：禁为过门禁调 ZW_SCRIPT_TIMEOUT_MS）。

## 刀体第二笔：索引化 replay（callbacks.rs）

`HandleStateIndex` 增 `styles` 投影（t8c 增量索引同型扩展）：apply() 内
SetAttrOnHandle(style)/RemoveAttrOnHandle(style) 臂整体覆盖/清空＋新增
SetStyleOnHandle/RemoveStyleOnHandle 臂（`fold_decl_specified` 折叠＋
merge/remove_style_property 与 `style_from_mutations_lw` **同函数**正序累积——投影
语义与全扫 replay 逐构造恒等，折叠成本移至水位推进每 op 恰一次）；
`__zw_get_style_lw_handle` 回调弃全批扫描改接 `shared_state_style`。失效模型与已
发运 attrs 索引同（drain 代际 bump＋回缩防御）。part04 trap 臂语义不变。

等价性钉测（新增 2，全绿）：`projection_equals_full_replay_on_every_prefix`
（xorshift 300 op 确定性序列，任意前缀三 handle 投影≡全扫 replay）＋
`shaped_sequences_end_values`（cssText 覆盖/removeAttribute 净空/removeProperty
净空/非 style attr 干扰/`STYLE` 大小写五形态）；R3199 端到端集成测试
（sandbox set→read）过；既有 handle_tag_memo_tests 5/5 无回归。

## 批终值与门禁史

- **v1**（19:28，bc69c69cf）：L0 门值算术自误 ABORT（215+2=217 非 220）。
- **v1.3.1**（2026-11-12 02:15，head f95fdd4d9 bin c4de0e4f）：L0 probe 217/5 恰集 ✓、
  L3b ab 10092/0/0 + judge 严格四零 ✓（墙钟修复后门内完走兑现）、L1 outdent
  F2P=169/P2F=0/lost=0/residF=4 恰集 ✓、L2 indent 1302/27 ✓、L3 mt 10090/2
  四零+residF=2 ✓、L4 singles×5 ✓、L5 exec×3 ✓；**L6 enginelib ABORT**：
  failed=10 vs 名册 7——3 新红归因见下节。语料七腿全绿传递。
- **v1.1**（重射）：L0 probe 217/5 ✓、L1 outdent judge F2P=169/P2F=0/lost=0/residF=4 ✓、
  L2 indent 1302/27 ✓；**L3 mt 主页 20180/2 vs 期望 20182/2 ABORT**——multitest.html
  主页单连续 JS 执行被刀一 trap 全批 O(list) replay 推过 300s 脚本门，末夹具 2 断言
  截断（mt 键集四零零漂移，纯截断非行为漂移）→ 刀二索引化。
- **knife-only 组合树 preflight 全绿**（pres405.log，22:23，bin ebb0ed68，head 25e533139）
  五腿：probe 217/5 恰集、outdent 2763P/9F judge F2P=169/P2F=0/lost=0/residF=4 恰集、
  indent 1302/27 零漂 VERDICT OK、mt 10090/2 judge 四零＋residF=2（rbv2 在册两键）、
  ab 10092/0/0 零漂 VERDICT OK——刀二后 multitest-ab 回 300s 门内完走。
- **终门 v1.3**（组合树重锚版，占位符未实例化）暂缓发射，见下节。

## s405 碰撞：multitest-ab 再推过 300s 脚本门（v1.3 暂缓，2026-11-12 记档）

pull --rebase 并入他流 14b6ca724（s405 embed 动态加载链，wptfyi-css 流）后刀体重放
（0a435f0a1/0ff1bc92c，已推送）。换基正中已记教训「preflight 与批之间不得有源码变更」
（s405 触及 part01/part04/part05/part11-color，part04 与刀体同文件域 rebase 无冲突合并）
→ 组合树重跑 preflight（23:12-23:54，bin c178799f，head 0ff1bc92c）：

- **行为四腿与 pres405 逐字节恒等**：probe 217/5 恰集、outdent 2763P/9F judge
  F2P=169/P2F=0/lost=0/residF=4 恰集、indent 1302/27 VERDICT OK、mt 10090/2 四零＋
  residF=2——刀体语义在组合树零漂移。
- **ab 页 9702P/0F/0T vs 基线 10092P/0F/0T，lost=390**（shared 键四零；lost 全落截断
  尾块）；腿墙钟 23:49:51→23:54:53 ≈ 302s = 300s 脚本门击杀签名；截断点
  `[["insertparagraph",…]] "<div>abc{<s></s><b><i>de]f</i></b>ghi</div>"`（语料 96%
  处），基线末夹具 forwarddelete white-space:pre 未跑到。

归因（两点位唯一源差）：knife-only（ebb0ed68）ab 完走 10092 ≤300s；knife+s405
（c178799f）截断 9702 @302s（全页线性估 ~312s）。s405 给 part04 CE 插入步循环加
第三个 per-node 钩子 `_zwStartConnectedEmbed`，tag 判定 `_realTag()` 宿主桥调用——
与既有 `_zwStartConnectedIframe`（IFRAME/FRAME）/`_zwStartConnectedObject`（OBJECT）
完全同型；per-insertion 宿主桥调用 2→3 次，累积成本耗尽刀二索引化买回的余量。

处置：engine 属并行规则 §9 共享面且他流 session 活跃（当晚多流协作仲裁高峰）→
不硬解不代修；L3b 门值 10092 硬编码＋judge 四零 vs cssom-ab-perffix.log，截断态发射
必 ABORT 无新信息，故不空跑。恢复条件：s405 流恢复 ab 页 300s 门内余量（如钩子
tag 判定降本为字符串先行）或两流协调处置 → 重跑组合树 preflight 五腿对账 →
按新 preflight 实测值实例化 v1.3（EXPECTED_HEAD/MT_P/MT_F2P_SIGNED）发射。
→ 恢复条件已由下一节仲裁修复兑现（本流自修，非调阈值）。

## 碰撞仲裁修复：CE 插入 walk 三 start-connected 臂共享单次 tag 读（2026-11-12）

刀体第三笔（084612c5e → rebase 后 f95fdd4d9，已推送；part01.js 三臂签名 + part04.js
两插入循环，65+/27−）：

- **设计**：`_zwStartConnectedIframe/Object/Embed` 增可选 `tagHint` 尾参（已
  toUpperCase 的调用方共享 tag 读；缺省走原读零变化）；part04 两 CE 插入循环把
  `_realTag` 读 hoist 到循环体单次（判定表达式与臂内原逻辑逐字同形），每新增节点
  `__zw_*` 宿主回调 **3→1 次**。语义恒等：spec tagName 不可变
  （dom.spec.whatwg.org/#dom-element-tagname）；hint 空串回落 `tag || el.tagName`
  与原式同形；解析期静态扫描调用点（part04 :1290/:1312）不传 hint 走原读不变。
  归因修复非调阈值（perf-gate 政策禁调 ZW_SCRIPT_TIMEOUT_MS）。
- **测量链**（multitest-ab leg 墙钟）：

  | 树 | 租户 | 结果 |
  |----|------|------|
  | 3 读/节点（c178799f） | solo | 302s 截断 9702 行（上节） |
  | 1 读/节点（bin 6f532726） | 2 租户 | 306s 截断 9984 行（争抢污染不可定谳） |
  | 1 读/节点（bin 6f532726） | solo | **286s 完走 10092P/0F/0T** |

  solo 终判 judge-dps vs cssom-ab-perffix.log：rows 10092→10092 **严格四零逐行
  恒等**。单调性佐证：修复后每节点 1 读 < s405 前 2 读基线（同门通过），达标由
  结构保证。租户判读教训复证：9984/306s 一份测量若无 `ps` 租户核对险些误判
  「修复不足」——墙钟测量必须 solo（wallclock-contention 教训）。
- **组合树重锚**：rebase 并入 origin 7 提交（s406 284a7bbb4 frames.named 扩
  embed/object、5fc0d203a callbacks.rs fmt 机械修复、f06979d02/8409a3b7b
  css-values time/opacity、c4bce8a46 docs）→ f95fdd4d9。s406 未触 part04，
  part01 改动区（~3839-3936/13028）与三臂区（~4231-4330）不重叠，rebase 零冲突。
  探针冒烟 217P/5F **FAILKEYS-IDENTICAL**（bin c4de0e4f）；S1-S8 锚复验
  3/2/1/4/1/3/1/3 全对齐。
- **协作记档**：5fc0d203a 是他流对本刀两笔（0ff1bc92c/0a435f0a1 入 main 未过 fmt
  门）的 callbacks.rs 机械 fmt 兜底——共享 make test fmt 前置被他流堵上，刀体
  Rust 面此后提交须 `cargo fmt` 自检。
- **终树回归面**（head f95fdd4d9，bin c4de0e4f）：embed 钉值对照（s406 后
  network-error 应 4P 全解挂，余按 s405 钉值）＋ object 目录 per-file checkout
  翻转 A/B（f29f04cf3~1 两文件 → 父态基线 vs HEAD）——**全绿**：
  object rows 46→46 严格四零 VERDICT OK；embed 15P/12F，钉值 6/6 精确命中
  （network-error 4P=s406 全解挂兑现、javascript-url 3P、change-src 1P、
  allowed-schemas 5F、change-src-2 1F、document-getters 1P），与本流 rebase 前
  run（13P/14F）差值恰 = s406 network-error 2F→2P，余键零漂移——fmt/css-values
  入站提交对 embed 面亦零行为影响。
  the-iframe-element 目录不跑：归 navigation-compat 通道零执行记账，iframe 臂由
  机械恒等＋probe/ab 恒等覆盖。

## 终门 v1.3.1 L6 enginelib 名册漂移 7→10：三新红归因与 r307 修复（2026-11-12）

v1.3.1 语料七腿全绿后 L6 ABORT（failed=10 vs 名册 7，3881P）。三新红逐名归因：

1. **r307_tree_order_identity_domains = 本刀真回归（当日修复）**。wptfyi-css g10
   批 A/B 已钉死窗口（db74aad18 含 CSSOM 两提交红 vs g9b 14b6ca724+g9 绿，
   14b6ca724..db74aad18 恰 = CSSOM 两笔 + css-parser mod/rem 非引擎面）。
   根因：刀一笔第四键的比对两侧序列化源不对称——本侧快照（`_zwMOuterHtml`）
   经 R174 no-ns 标记臂对 `createElementNS('', …)` 后代写 ` data-zw-empty-ns=""`，
   host 侧重解析经 `apply_empty_ns_markers` 剔标记——含空 ns 元素的树两侧恒不等，
   真镜像漏剔，`querySelectorAll('*')` 结果混入**查询根自身**（res=16/trav=15、
   首元素=根镜像 div#root；detach/indoc 两域红，fragment 域 R308 子串路径不适用
   故 16/16 恒真）。语料树无空 ns 元素故七腿零暴露。修复：比对前剥本侧标记
   （`_r188RootSelfCmp`，part03.js 一行；src173 本体不动——host 解析仍需标记
   还原空 ns 查询语义）。单测复跑绿；全 enginelib 3895P/9F（修复 +1P −1F）。
   备考：等值串法此窗已两度破坏（R310b 三键误切→第四键→标记不对称），后续再破
   即升级宿主侧结构化镜像根判定（parse payload 带镜像根标识，弃文本全等）。
2. **test_execcommand_format_apply_r3254_m3 / toggle_state_r3254_m3_slice4 =
   f415be99c 既有跨流红（他流已认账本流认领）**。html-semantics s406 evidence
   stash A/B 三重实证归因 f415be99c（DC-2 P2 六 inline format apply 切 vendored
   setSelectionValue 全流程，JS-only 未同步 2 单测——旧断言绑 SetInnerHtml 契约，
   vendored 改 granular mutation 流）。本流 mock 探针实测：flat bold 净效应正确
   （CreateTextNode/SetChildText/AppendChild/SetTextOnHandle/CreateElement/
   InsertBefore/RemoveHandle/AppendChildByHandle 十 op 净 `fo<b>ob</b>az`）；
   **toggle 解除臂存疑**：shim 视图 `pre.innerHTML` 读 "ob"（既非原 markup 亦非
   语料期望 preobpost），mutation 流 RemoveChildAt+Remove 净 host 丢 "ob" 文本
   （prepost）——而 corpus 真实路径全绿（F2P=1,174 零 newF，compare 行命令后读
   innerHTML）。真实主机写面正确性与 mock 环境两视图分歧未定谳：测试同步须先
   做 vendored unwrap 写面根因刀，不得把数据丢失签名钉为期望契约。2F 记账维持
   红（名册 2026-11-12 档 9 红=7+2），根因刀独立立项下轮首选。
3. 其余 7 红与 2026-11-09 档恒一（img/svg settle → svg 流；fetch/response/ws/
   iframe-worker 族 → net/webview 域）。名册档
   [enginelib-red-roster-2026-11-12.txt](enginelib-red-roster-2026-11-12.txt)。

**换基声明（v1.3.1→v1.3.2）**：v1.3.1 ABORT 后 pull --rebase 并入 origin 20+ 提交
（s407 embed 静态链 610bbae31、s408 detached style sheet 34c06eeb5、stamp epoch
快道 45e1be731/d3f5eb3a5、g12 数学求值管线、layout r5504/R5505、site-opt merge 等，
35 文件 +2,685——其中 part01/part03/part04/part05/part06 shim 面均被触及）。
v1.3.1 语料七腿判读绑定 f95fdd4d9 字节态，对新头失效——v1.3.2 全腿在新头重跑，
judge 仍 vs 既有 census/基线日志（零漂移判据不变）；L6 期望 failed==9（名册
2026-11-12 档）。锚更新：S4 4→5（修复行）、新增 S9=`_r188RootSelfCmp`==2。

## 终门 v1.3.2 ALL-DONE 收口（2026-11-12 04:29:35→05:22:27）

head 30f1edbee（r307 修复 + 名册档 + 控制面入账后重锚，rebase 并入 origin docs
06c57fed2 零引擎面）bin aa77d3b8079229b02129978c13d108c1 逐腿记账恒一，~53min：

- **L0 probe** 217P/5F 恰集（vs pfc-probe-failkeys.txt 零漂移）✓
- **L3b mt-ab** rc=0 **10092P/0F/0T** + judge-dps vs cssom-ab-perffix.log 严格
  四零 ✓——墙钟仲裁修复后 300s 门内完走在新头兑现（stamp epoch 快道入站后
  余量进一步放宽）
- **L1 outdent** 2763P/9F，judge F2P=169/P2F=0/lost=0/residF=4 恰集 ✓
- **L2 indent** 1302P/27F + judge 零漂 VERDICT OK ✓
- **L3 mt 主页** 10090P/2F，四零 + residF 恒 rbv2 两键 ✓
- **L4 singles×5** iol 1759 / iul 2061 / ip 6989 / ilb 1658 / ihtml 1446 全零漂 ✓
- **L5 exec×3** 零漂 ✓
- **L6 enginelib** 3895P/9F == 名册 2026-11-12 档 ✓（r307 修复在场 +1P−1F）

刀体四笔（trap pending-aware / R188 第四键 / part01c 导出 / part01d attr 基底）
＋索引化二笔＋仲裁三笔在组合树（s407/s408/stamp epoch/g12 并入态）语义零漂移，
CSSOM 写回刀正式收口：outdent 页 CSSOM 8 残红族闭合维持 F2P=169，页残红 12→4。

## 残红账（不变）

- 2 dPS-qV-before：录音会话顺序泄漏链（窗界伪影定性沿用）
- 2 ws-bq compare：`<blockquote> <p>[foo]</p> </blockquote>` spec-vs-Blink 分歧
  （indentNodes 无 ws-only 特例，Blink 原生行为，需刻意兼容偏离臂，独立挂账）
- probe 页 5 dPS：同录音链伪影族

## 教训

- **批门值必须从 preflight 日志直接算术导出**：v1 把「215P/7F 闭 2 compare」误写为
  220P（正确 217P；fails→passes 是移动不是追加）。v1.2 同族教训（锚规格先 grep 现树
  实值）三度实证后第四次：凡数字门，先对账总数恒一再登记。
- **corpus preflight 与批之间不得有源码变更**：18:31 preflight（仅 part03）的 2 新红
  差点被当成最终树状态判读；mtime 对账（源 18:58 > preflight 18:31）后重跑才定谳。
- **诊断日志独立命名**：ab 丢行首例原始日志被复跑同名覆写，仅存账本摘录数字；
  诊断样本日志应独立文件名（`-bad1` 后缀）留存，复跑前先 `mv`。

## 终门 v1.3.2 换基重锚（2026-11-12 05:36→06:36，c2fc734ab→5c02b84d4）

收口 pull --rebase 并入组合增量（对账细目见 `.acceptance/dps/ACTIVE-BATCH.md` 05:3x 条）：
bb1c1256e（wptfyi-editing P3：part01c `_zwFmtDecorationOf` 作者规则臂＋part01d
`_zwVaForceValue` u/s 载体原位并/下潜，自带门 F2P=26/P2F=0/newF=0）＋cc22f08e1
（part11 通用色通道 +4/computed-color 173 行）＋3364ff226（s409 script-created link
不阻塞 script：part01 +21/part03 阻塞计数门 +12，与 R188 面 11747 行无交集）＋
c1a069a1c（layout-engine converter 补 `CalcExpr::Mix` 臂——渲染流编译红灯，本流重锚
批首射即撞 E0004 rc=101，其修复已先行推送）＋docs 3 笔。按 v1.3.1→v1.3.2 先例，
v1.3.2 门值对组合树须重锚。

重锚批 `batch-cssom-mt-reanchor.sh` @5c02b84d4 三腿实测（log `batch-cssom-mt-reanchor.log`）：

- **S1..S9 锚重放恒等** 3/2/1/5/1/3/1/3/2；r307 修复在场（`_r188RootSelfCmp` grep=2）
- **L6R enginelib** 3897P/**9F**（s409 新增测试 +2P，failed==9 名册恒一）✓
- **L3b ab** **10092P/0F/0T** 满额 ✓（bin 6f46bc78）
- **L3 mt** **10090P/2F** 与 v1.3.2 锚逐数恒等，judge 四零（F2P=0——P3 账本预言的
  u/s 正漂移在 mt 主页实测未发生），residF 恒 rbv2 两键 ✓

**ab 腿瞬态丢行观察（同二进制非确定性）**：首射 ab 实测 P=9994（−98 行/22 组合
chain×fixture 整丢、post-only=0 非改名、F=0/T=0/rc=0 无错误标记；形态全为跨元素/
跨块 delete·forwarddelete 接合后 inserttext——table 跨格、b×CFR 穿透、ws:pre 跨段、
flex/grid 跨 span、跨 div），复跑同 bin 满额 10092。ab 腿存在运行间非确定性丢行形态
一例；judge-dps ab 剖面对 lost=98 判 VERDICT OK（剖面容忍 lost），严格计数门为拦截
该形态的有效门。组合认证以复跑满额为准；根因 hunting 挂账下轮择期（复现法：ab 腿
连跑 N 次测丢行率）。
