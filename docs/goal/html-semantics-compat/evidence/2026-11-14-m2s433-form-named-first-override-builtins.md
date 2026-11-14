# s433：FORM named-first [OverrideBuiltins] 全 trap 面施工——撤 fix#19c 排除清单，s137 门决议落账

日期：2026-11-14 ｜ 车道：engine/dom（shim JS 三处 + script_gen 桥 + 桥测试）｜ 前序：s432 links target=_blank
s137 遗留案 form-nameditem.html（16P/1F，F 案即 named shadowing 面）

## 勘根定性：s137 排除清单的前提在 spec 上站不住

HTML spec FORM 元素带 **[OverrideBuiltins]**：具名 listed 控件（name/id 内容属性
→ form.elements 列举）在 form 对象上**遮蔽全部接口成员**（named property 先于
接口成员解析）。WPT form-nameditem.html 摆了名为 `namespaceURI/prefix/localName/
nodeName/nodeType/ownerDocument/action/elements/length/reset/submit/checkValidity/
requestSubmit` 的控件逐一断言遮蔽——shim 内部任何经 form proxy 读这些成员的路径
都是投毒面。

s137 时的 fix#19c 走的是**排除清单**：named 遮蔽对接口成员生效，但
`reset/requestSubmit/submit` 三名豁免（保 shim 内部 form.reset() 等直调不断）。
代价：`reset` 等名的遮蔽案恒红（WPT 断言 `typeof form.reset === 'object'`）
且豁免本身违 spec。s433 前提消解：①named-first 门覆盖**全部**接口成员读；
②shim 内部通道改走 Symbol（`Symbol.for('zw-form-reset')` /
`Symbol.for('zw-form-elements')`）不穿 proxy 命名面，script_gen 宿主侧带
`typeof f.reset==='function'` 未遮蔽页回落。排除清单失去存在理由，整体撤除。
**回滚路径**：恢复 fix#19c 排除清单 + 撤 Symbol 通道（git 历史可考）。

## 施工（五处）

1. **part03.js FORM get trap named-first 门**（`_zwFormGateNamedFirst` 区，~33153）：
   - ⓪ 成本门：具名集合缓存 `_fnSet`（`_elKey` → `{g: 代际, a/r: pending
     记账长度, e: attr 纪元, s: 集合, l: 重入哨兵}`），stamp 不齐才重建——
     重建 collection（宿主 `__zw_form_controls` IPC 往返）包在 `l=true`
     try/finally 里（R20 教训：裸收集在嵌套读下分钟级假挂死）。
   - ③ 具名查找臂：命中走 `_zwFormNamedLookup`（多同名 → RadioNodeList 缓存，
     s137 稳定化产物），同样受 `l` 哨兵。
   - ④ past-names 复查臂、⑤ 索引臂（form[0]）：同受 `!_fnSet.l`——③ 写过
     past 条目后，下一次查找的 ⓪ 重建经 `_zwLiveFormAncestor` 读
     `form.localName` → ④ 命中 → 嵌套收集（每层一次宿主 IPC）无界递归。
   - **action 臂后移**：51 行 HTMLFormElement
     action/method/enctype/encoding/target IDL 臂（R2839/R3202/R5009/s207
     沿革）从门**前**移到门**后**——named-first 胜 IDL 反射（R26 探针
     `action === false` 钉死次序错）。
   - 全部 DBG 摘除（R18 教训：累计计数器在长寿页上被 harness 探针误触发）。
2. **part05.js**：
   - `_zwIsHTMLNamespace`（~12684）payload-first ns 读：`_zwSelNsCache.map`
     按 `el.__zwSelector` 取 ns（''=HTML ns），stamp（gen/added/removed）
     新鲜才走——R6 实证 proxy 读 `el.namespaceURI` 被名 namespaceURI 的
     radio 投毒（document.forms=0）。
   - IDL 反射 setter 漏斗（~1285）：name/id 写 → `_zwFormAttrEpoch++`。
   - `_formControls` 摘 `_wd` 深度参数；`_zwFormNamedLookup` 清 DBG。
3. **part04.js**：setAttribute 漏斗（~4442）与 removeAttribute 漏斗（~4832）
   的 name/id 写 → `_zwFormAttrEpoch++`（past-names 面：removeAttribute('id')
   把控件挪进 past 集合）。Symbol 通道确认在场：elements get
   `Symbol.for('zw-form-elements')`（12894）、reset 直调通道
   `Symbol.for('zw-form-reset')`（13830-13838）。
4. **script_gen.rs**：`script_call_form_reset`/`script_reset_form_controls`
   改走 Symbol 内部通道 + `typeof` 未遮蔽回落。
5. **js_dom_bridge_tests part01/part02**：`test_form_named_access_shadowing_t2pb1`
   翻新不变量——①具名控件对 checkValidity/reset/requestSubmit/submit 全遮蔽
   （返回控件非函数）；②Symbol 通道按名不可达、reset/elements 内部照常工作；
   ③普通具名访问。

**`_zwFormAttrEpoch` 为何必要**：attr 写（改名）不动 pendingAdded/Removed
长度——缓存 stamp 只看 pending 三元会漏。纪元在三处漏斗保守全局自增（任意
元素 name/id 写都翻），parse 期由 pending 三元覆盖。R27（gx=undefined 手柄
form）/R28（改名后 twiddled1=false）/R30（事件 trace 实证收集发生在设名
之前）三轮探针钉死此面。

## 排障链（逐层证据）

- **R6**：document.forms=0 —— `_zwIsHTMLNamespace` proxy ns 读投毒 → payload-first。
- **R19**：churn 探针零投递（首个测试前收集挂死）。
- **R20**：逐读流式探针（title=read #N）——read #1 报出后挂死 → ④ 重入根因
  （③ 写 past 条目 → 下轮 ⓪ 重建读 form.localName → ④ 命中 → 嵌套收集假挂）；
  L1/L2/L3 标记级联（probe-R20-l3.log）钉死臂位。
- **0P 回归**：④⑤ 修后 ⓪ 重建自身未持哨兵 → 重建内嵌套读 → 补 `l=true` try/finally。
- **GET-LOOP 假回归**：html|classList×500 —— 栈捕获定位 harness 探针单行
  （`<anonymous>:1:1145`）每 poll 读一次 classList，累计 DBG 计数器被长寿页
  ~500 poll 生命周期误触发 → 摘全部 DBG（R18 同源教训）。
- **R26**：action===false —— action 臂次序错 → 后移。
- **R27/R28/R30**：attr 写致具名集合过期 → attr 纪元 + 三漏斗自增。
- 修后：form-nameditem solo **17/17 PASS exit 0**（s137 期 16P/1F）；
  六 solo 套件全 exit 0；25-part 拼装 `node --check` 过；51 个
  zzz-s433-probe* 语料探针全部删除，语料还原 23 个真实文件。

## 全域双跑拦下潜伏雷——`_fcKind` 声明行被吞（s433 首轮 sweep 的核心战果）

首轮全域双跑（f1/f2，坏构建留存 `sweep-f{1,2}-broken.log`）join 出**一条确定性
P→F 回归**：`form-elements-sameobject.html :: [SameObject] should apply to
'elements' attr on <form>`（s432 两跑全 Pass → 本轮两跑全 Fail）。逐层定位：

1. 分步探针：STEP-A（裸二次读）即挂，appendChild 无关；
2. Symbol 通道二分（`form[Symbol.for('zw-form-elements')]` 绕过 named 门直达
   getter）仍不等 → **getter 自身坏**，与 named-first 门无关；
3. dump 式探针：每次读返回全新 HTMLFormControlsCollection（`e1===e2=false`、
   `e1===sym=false`，len/ctor/item 全正常）→ 集合缓存 `_zwFormCollCache`
   从未命中；
4. `grep -n _fcKind` 全 part 仅两处**使用**、无声明；`git show HEAD` 对照：
   前一 session 加 Symbol 通道的 Edit 把条件行替换成「新条件 + s433 注释」时，
   **`var _fcKind = (!sel && handle) ? 'h' : 's';` 声明行被一并吞掉**。

**静默失效链**（为什么全程 solo/probe 没抓到）：读 1 缓存空 → `&&` 短路
不求值 `_fcKind` → 建新集合 C1 → `set({k: _fcKind})` 处 ReferenceError 被
**空 `catch (_eFcC) {}` 吞掉** → 缓存永不写入 → 读 2 重建 C2 → 每读新集合。
form-nameditem 断言集合**内容**不验**身份**，探针从未二次读
`form.elements`——solo 全绿 17/17 照样带着雷跑完。全域 A/B sweep 正是为此
类雷设的网。修复：补回声明行（`k` 校验恢复，缓存命中）；复验——探针
`e1===sym=true | e1===e2=true`、真实语料 Pass、form-nameditem solo 17/17
exit 0。**教训**：Edit 换条件行时 Old string 恰含声明行就会无声吞掉，
shim 局部变量无编译器兜底，`try{}catch{}` 沼泽里连 ReferenceError 都是
静默的——solo 通过 ≠ 无雷。

## 验证链

| 层 | 结果 |
|---|---|
| R6/R19/R20/R26/R27/R28/R30 探针 | 全绿，用后即删 |
| form-nameditem solo | 17/17 PASS exit 0（前 16P/1F；fix6 复验同）|
| 六 solo 套件（nameditem/churn/perread/gx/twiddle/forms）| 全 exit 0 |
| 25-part 拼装语法 | node --check 过 |
| cargo fmt / clippy | 过（12:10 clippy，fmt 14:2x 复验；此后无新 Rust 改动）|
| 首轮全域双跑 join（坏构建）| 拦下 `_fcKind` P→F 回归 → 修复 → fix6 |
| 终稿全域双跑 join（g1/g2，守卫启动器）| stable-P 8307→8309（+2）、stable-F 201→202（+1）、stable-T 3→3；P gone=0 |

**基线批口径注**（s412 教训落实）：本轮 join 基线 = /tmp/s432/pairs-{5,6}.txt
（s432 移交批，8514 行/跑）。s432 账本组合态整验引用的 stable-P 8308 在
全部留成批（pairs-1..6）中均未复现（join 侧计 8307）——±1 恰为
`execution-timing/149.html 怪标题键（标题含字面制表符）在两代 join 脚本
间 沾染/stable 归类口径差：该键六跑全 Pass，两侧对称 ±1，集合差逐键不受
影响。s432 账本「home-end 4 子测键翻回 P」一说不被任何留成批支持
（pairs-1..4 同批 4×F、5/6 文件级 T，从未 P）——引用批未留成，簇 D 家族
主导态以留成批为准（F）。

终稿 join 集合差逐案（对 /tmp/s432 全域基线 8514 行/跑）：

- P new ①（驱动案）：`form-nameditem :: Named elements should override
  builtins`——F→P，s137 排除清单退役后 named-first 全遮蔽达成。
- P new ②（红利案）：`selectedcontent-listbox :: <select size=4> should
  support <selectedcontent>.`——F→P 双跑一致。页面无 form 读路径，与本轮
  改动的直链未逐案归因，机制留后续轮次；翻绿为真（基线两跑 F / 本轮两跑 P）。
- F new 4 案 = `select-home-end-pagedown-pageup.optional.html 的
  Home/End/PageUp/PageDown 四子测（customizable-select 键盘导航未实现，
  簇 D 在册 flake 家族）。**跨批同态身份实证**：s432 自家留成批
  pairs-1/2/3/4（同构建无 s433 改动）此 4 键**同为 4×Fail**——子测态
  与构建无关，基线批 pairs-5/6 仅因文件级 Timeout 未报子测（「文件完成
  vs 文件级挂死」同族在 s432 批间自翻转：3/4 完成、5/6 挂死）。本轮
  5 跑（f1/f2/g1/g2/solo）文件全完成、子测恒 4F——断言可见化，非语义回归。
- T new/T gone 对换 = 上文件（文件级 T→子测可见）与 sibling
  `…-detailed.optional.html（基线 stable-F 文件级 → 本轮 sweep 中文件级
  Timeout）。solo 实证 detailed 4F 0T——sweep 的 T 是 8 job 并发负载
  把该长页推过文件超时边界的边界态 flake（s432 pairs-2/3 该文件已在
  F↔T 间闪变，同族行为），非回归；四跑矩阵 + solo 单证齐。
- 沾染 my=1 / base=1：同为 `execution-timing/149.html 标题含字面制表符的
  join 键切分伪影（六跑态全 Pass），两侧对冲，非真 flake。

**运行环境注**：本轮跑动期间存在并发 session（editing/selection 车道）
在同树重建 runner 与跑 solo——终稿双跑启动器（sweep-final.sh）每腿前核对
源文件 sha256（srcfix6.sha）+ 二进制 `_fcKind` 标记，漂移即 ABORT 记账；
fix6 二进制快照留存 `/tmp/s433/runner-fix6.bin`。

## 已知近似（挂账，均 R 探针实证良性）

- **同 turn pending proxy 残余投毒窗**：stamp 只含 pending 三元 + 纪元，
  同 turn 内建又读的 proxy 面未覆盖——现语料无案。
- **`_zwParseParentUnmoved` parentNode 读**：parse 期 proxy 读不在门内
  （parse 期尚无具名集合——良性）。
- **`_zwCtxIsHtmlDoc` ownerDocument 读**：良性回落（named-first 命中返回
  控件 → 回落分支按非 HTML 处理，R 探针全绿实证无案）。
- **set 侧反射名写副作用差**：set trap FORM 具名不可写臂仍走 live lookup
  （无缓存），与 get 侧缓存面存在副作用时点差——spec [[Set]] 具名失败语义
  不变，严格模式 throw 一致。

## 产物

- 代码：part03.js / part04.js / part05.js / script_gen.rs / 桥测试 part01+part02
- 工件：/tmp/s433/（sweep.sh、run-solos.sh、join.sh、probe/solo 日志、
  part-order.txt 25-part 拼装序）
- 基线：/tmp/s432/pairs-5.txt + pairs-6.txt（全域 8514 行/跑）

## 收尾门禁与全域发现（429 断点接手轮补记，2026-11-14 16:xx–17:xx）

上一 session 15:30 于 429 断点中断（中断时正在核对 pairs 批）。本轮接手
完成收尾门禁三项，并**发现 main 桥测试套件 12 例确定性红**（均非本轮引入）：

### 门禁结果

| 门 | 结果 |
|---|---|
| fmt --check | 过 |
| clippy -p zero-engine --all-targets -D warnings | 过（本轮终稿代码复跑） |
| 桥测试套件复跑（guard 包裹，scope `js_dom_bridge`） | 1556P/12F——**12F 全部为 main 既有红**（见下） |
| product-smoke-legacy | exit 0（trend-only）；37-form-controls **struct PASS**；42 fixture 中 **01-body-attrs struct FAIL（渲染域疑似新发，非本轮面——fixture 无 form/script）** |

### 桥测试 A/B 归因（换 engine 单 crate 探针，五跑）

| 探针树 | 结果 |
|---|---|
| 终稿树（s433 + g35 css/layout） | 1556P/12F |
| pre-s433 engine（25fdd6e45） | 1556P/12F——**同 12 红，s433 零新增**（t2pb1 旧断言在旧 engine 绿、新断言在新 engine 绿，双方自洽） |
| pre-#302-merge engine（a1caf8509^1） | **7F**——5 例消失 |
| pre-progress() engine（8c13a2856^1） | 7F——progress() 洗清 |
| pre-R97 engine（e76c9c39e^1） | 7F——R97 洗清 |
| Oct31 锚点（2e260d532 仅换 engine） | 编译失败 17 errors——两周 API 漂移，单 crate 换法到不了；7 例归因需全树 bisect |

隔离复跑（filter 12 名）同 12F——**确定性红，非负载/串扰**。

### main 既有 12 红清单与归属

**5 例 = PR #302（t68-g1-selhandle-registry-sync，a1caf8509 今日 09:16 合入）
合入后新发**（pre-#302 engine 全绿）：

- `t68_remove_child_fused_empty_serialization_case_c`（**#302 自带契约钉自身红**）
- `named_access_subtree_descendant_unregister_s40`（子树移除后代动态名失格）
- `show_modal_fully_active_gate_r5250`（detached showModal 须抛未抛）
- `test_media_pause_on_removal_m3b7` / `test_media_play_pending_and_removal_abort_m3x14`
  （移除后 pause/abort 面）

五案同指 t68 移除/序列化注册表 sync 面，单根可能性高。**归属 site-opt 流
（t70 分支在途）**，按并行归因纪律移交，本域不硬解。

**7 例 = 早于今日 08:07（df85ed2a6）已红，引入点需全树 bisect（下轮任务）**：

- `img_runtime_deferred_settle_r5275` / `svg_img_settle_density_r5277`（load 事件后续派发缺）
- `test_fetch_filtered_response_type_matrix`（matrix undefined——页脚本早夭）
- `test_iframe_service_worker_messages_dispatch_to_iframe_container`（URL userinfo Basic auth）
- `test_response_body_readable_stream_r2967`（response undefined）
- `test_t8k_websocket_error_nonterminal`（error 事件缺）/ `test_t8k_websocket_event_flow_open_message_close`（open 事件缺）

签名共性 = 异步事件/任务派发缺（load/open/error/pause），且七案均系 10-13～
10-31 引入时绿的旧钉。全仓最近一次全桥套件绿无账可考（各轮只跑 filtered
solo——套件红窗至少两周无人发现）。**方法论挂账：轮末收口应含一次全桥套件
跑（约 25 分钟，guard 包裹），防跨流静默红窗复现。**

### body-attrs struct FAIL 旗标（移交渲染流）

42 fixture 首例 `01-body-attrs` struct FAIL（历史 R2163 51/51 全 PASS、
R385x 期 0 struct FAIL）。fixture 纯静态 body 属性（bgcolor/text/link/
vlink/alink），无 form/script——与本轮零关联。候选归因：349a891ed（UA 表单
系统色关键词化，今日 06:52，UA 默认色域相邻）或 g35（css-parser
anchor_area 重写）。归属渲染流 legacy smoke 门禁（其规则：UA 样式变更须跑）。

### 后记（push 后组合态复跑，17:5x）

push 时 pull --rebase 带入他流 94883fbbe（editing 流「ip M3-A 门禁批
enginelib 五新红归因——R125/move-in 线章与身份章分离」）。组合态复跑：
**1561P/7F——上表 5 例（#302 相关移除/序列化单根）已被该修复清偿**，
「单根可能性高」预判实证。归属注记修正：五例出现于 #302 合入态
（pre-#302 engine 7F 实证不变），清偿经 editing 流门禁批落地（归因口径
两流各异，以修复面 move-in 线章/身份章分离为准）。余 7 例异步事件派发
面红仍在 main，全树 bisect 待下轮。

### 产物补充

- 门禁日志：/tmp/s433/ut-bridge-final.log、ut-isolated12.log、
  ut-bridge-pres433.log、ut-bridge-pre302.log、ut-bridge-preprogress.log、
  ut-bridge-prer97.log、ut-bridge-oct31.log（API 漂移证伪）、smoke-legacy.log、
  clippy-final.log
