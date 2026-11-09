# M2 s324 — Face B selectedcontent 元素族施工：21 子测 6P/15F→12P/9F，可达面全绿收口；mutations 8 案=解析期记录天花板定性（s117 同类），test1 案=Face A 协付点

日期：2026-11-07。切片号 s324（承 s323 下一步候选①：Face B selectedcontent 元素族施工）。

## 面与基线

Face B（s323 五面分派候选①）：`html/semantics/forms/the-select-element/customizable-select/`
下 selectedcontent* testharness 页，`make testharness-html-semantics
FILTER=customizable-select/selectedcontent` 通道 **21 子测**（selectedcontent.html 6 +
mutations 8 + insertion-removal 4 + movebefore 1 + nested 1 + attr.tentative 1；
restore.optional ×2 与 in-option-crash 无子测产出不涉）。s323 勘根定谳 `<selectedcontent>`
零实现（crates/ 全 grep 零命中），基线全红。

语料页为 wpt-data 本地 gitignored 数据（s323 勘根批带入，无 imported-tests.txt 登记）。

## 轮次进展（日志 evidence/logs/s324-selectedcontent-r{1..5}.log）

| 轮 | 子测 | 变化 |
|----|--------|------|
| r1 | 6P/15F | Rust 解析 pass + 首版 JS 面 |
| r2 | 5P/16F | sc 克隆改 innerHTML 往返（首版 cloneNode 产物垃圾对象反噬一轮） |
| r3 | 10P/11F | innerHTML 往返 + textContent 融合视图缺口修复 |
| r4 | 11P/10F | option.selected pristine 物化 + moveBefore 微任务延后 → **movebefore 全绿** |
| r5 | 12P/9F | selectedcontentelement 关联（内容属性 + IDL）→ **attr.tentative 全绿** |
| r6 | 12P/9F | 门禁期 output 融合回归修复后终版 shim 复验（零位移） |

终态 12P/9F：**可达面全绿**。9F 全部定性不可达/跨流（见下两节）。

## 施工面（依赖升级 ⓪ + 六根因 ①-⑥）

### ⓪ html5ever 0.29→0.38 升级（workspace Cargo.toml + dom/Cargo.toml + node.rs/parser.rs 适配）

升级动机：0.38 实现新 spec select 解析——`InSelect` 插入模式整体移除、select 规则并入
"in body"（button/div/datalist/selectedcontent 于 select 内**保留不剥壳**、嵌套 select
闭合外层），本切片 sc 钉面（button>selectedcontent 结构存活）与 s325 Face E（select
解析定制族）同链受益。适配面：TreeSink/ElemName/ElementFlags trait 移至
`markup5ever::interface`（node.rs QuirksMode 重导出随迁）；`parse_fragment` 增第 5 参
`context_element_allows_scripting`（与文档级 scripting 旗标同值）；markup5ever 0.38 +
tendril 0.5 workspace 对齐。全量行为回归由 make test + make reftest 门禁覆盖。

### ① 解析期初始化（crates/dom/src/parser/selectedcontent.rs，新模块）

spec：HTML #the-selectedcontent-element（whatwg/html #9799 谱系）——sc 反映所属 select
当前选中 option 内容快照。文档解析完成后对每 select 单遍初始化（解析期 selectedness
模型：selected 属性 last-wins，否则 display size 1 回落首个非 disabled），把选中 option
子节点深克隆进每个 enabled sc（replace-all）；无选中清空。单遍后处理绕 html5ever 0.38
pop/maybe_clone 钩对隐式闭合不触达缺口（html5ever#712；时点等价性论证见模块文档）。
**语料偏差钉形**：multiple select 的 sc 走**清空**面（mutations test6 末态 html 判据，
Chromium update 面清除初始内容）——非 spec「null 不施」，与 shim 运行期同判据。enabled
判定 = sc post-connection steps 镜像（祖先链 option/sc 或第二个 select → disabled）。
unit tests ×2（`zero-dom`：初始克隆 + multiple 清空）随模块入工程。

### ② sc 克隆 = innerHTML 字符串往返（part03 `_zwScCloneInto`）

首版 `chosen` 子节点 `cloneNode(true)` 产物为垃圾对象：Sel 域解析期文本子是**父寻址
轻量包装**（`__zwIsText`，无 nodeType/无自有 cloneNode），克隆体入树后污染子视图
（textContent 出现 `[object Object]`、部分克隆丢文本节点）。改 chosen.innerHTML 读串 →
sc.innerHTML= 写串往返——三域统一 replace-all 通道（R304 基缓存重置 + 融合视图读），
身份/plain 双形态自动覆盖。plain sc 侧发单条合并 childList 记录（`_mo_notify(null,null)`
退化通道，mutations test7 运行尾实证记录形逐字节命中）。

### ③ textContent getter 融合视图共享缺口（part05 `_zwFusedTextSel` 新 + part04 尾钩）

同 turn `innerHTML=`/appendChild 后读 `textContent` 恒 stale（host 写异步 apply，
innerHTML getter 有 R380 三臂融合门、textContent getter 没有——探针 div/sc 双案实证
**非 s324 专属**，基线 div 同红）。补 sel 域 textContent 融合臂（三臂同 R380 门：
pending bucket 世代戳 / 子树脏 / 属性脏），门未激活零变化（host 读热路径不动）。

### ④ option.selected getter pristine 物化（part03 getter + `_zwPristineMatDone` memo）

movebefore 面根因：解析期回落选中面（无 selected 属性 + display size 1 首个非
disabled）在 `option.selected` getter 不可见——dirty/clean 双 miss 后直落属性链恒
false；`select.value`/`selectedIndex` getter 有 pristine 回落臂（`_zwSelectSelectedView`
s135 面）、getter 没有。修复 = 双 miss 时在最近 select 上惰性跑一次 selectedness
setting（HTML §4.10.7：属性背书 last-wins 物化 + 回落 clean true），per-select memo
档重复走查。物化后写侧（moveBefore 插入步 last-wins 消解、setter 族）与读取侧共用
同一状态层——movebefore 语料全链（o1/o2 双真态起点、移入 last-wins 降位 o1、移出
re-fallback o3、value 同步更新）全部命中。

### ⑤ moveBefore 移动面 sc 微任务延后（part04 `__zwScMoveMode` 旗标 + part03 `_zwScMaybeUpdateFor` 消解）

PR #12263 movebefore 语料钉：移动选中 option / sc 自身时同步读保持旧内容，微任务后
归一。s111 walk 插入臂（part01 两处）原为同步更新 → 旗标罩 moveBefore 委派窗口
（try/finally 恢复），窗口内 `_zwScMaybeUpdateFor` 转 `_zwScScheduleUpdate` 微任务批。
移动本体语义不变（R5258 委派 insertBefore 机械，MO 记录全继承——两 select 的
selectedness setting 消解经既有 insertion/removal 面自动到达）。

### ⑥ selectedcontentelement 关联（part03 `_zwScAssocResolve` + `_zwSelectedContentUpdate` 关联臂 + IDL getter；part05 IDL setter；part04 setAttribute 尾钩）

attr.tentative 语料面：select 与**树外** sc 的属性关联。内容属性值作 id 经
document.getElementById 解析（非 selectedcontent 标签/悬空 id → null）；IDL
`select.selectedContentElement` getter 同解析、setter element→属性落 id / null→移除；
`_zwSelectedContentUpdate` 增关联臂（最近 select 非本 select 时同面施克隆，树内面由
enabled 清单覆盖不重施）；setAttribute 尾钩挂 SELECT×selectedcontentelement 变更步
（同步克隆）；**移除面不挂钩**（语料钉移除不清内容）。全部既有触发点（value/
selectedIndex/option.selected setter、selected 属性变更步、form.reset、微任务批、
s111 walk）经 `_zwSelectedContentUpdate` 汇流自动覆盖。FIXME 挂账：解析期带该属性的
select 初始关联克隆未做（本引擎语料为脚本置面，出现带初始属性语料时补解析 pass
关联臂）。

## 不可达面定性（9F 全清单）

### mutations.html tests 1-8（8F）——解析期 MO 记录天花板（s117 同类 FIXME）

语料在**解析 HTML 前**注册 MutationObserver，期望记录是 mid-parse 增量流（每 option
pop 触发 sc 克隆的逐节点记录，期望 10-27 条）。本引擎脚本随整篇解析完成后执行
（无 mid-parse 观察面）→ 观察者注册时 DOM 已整树就位，got [] length 0。测试 7 运行
尾（脚本段 append option 触发面）逐字节命中 5 条期望记录（含两条合并形 sc 记录）——
单记录设计实证正确，页仍 F 因不可匹配的解析期头部。定性：重放架构天花板，与 s117
「解析期脚本时点」同类，挂 FIXME 不追。test8 侧发 3 条 sc 记录 vs 期望 2（移动面
per-descendant OPTION 臂各发一条）——不可达页内无害，不修。

### selectedcontent.html test1（1F）——Face A 协付点（testdriver/渲染流）

`await test_driver.click(optionTwo)` 抛 `action was not applicable:
Some(AlreadySelected)`（page-runtime plan_option 经 option_activation_snapshot 读
engine cached_html 的异步 apply 时序）。该断言面之前全部通过；点击通道属 Face A
:open/picker 状态机消费链（s323 五面分派 A 面 44+ 页总闸=渲染流
appearance:base-select），不在 s324 scope，挂协付点账。

## 验证

- 语料轮 r1-r5（上表），`make testharness-html-semantics
  FILTER=customizable-select/selectedcontent`，test-guard 包裹（--per-proc-mem 4
  --total-mem 8 --time-limit 5400），日志 evidence/logs/s324-selectedcontent-r{1..5}.log。
- 探针页（s324-probe-basic.html，gitignored）实证 ②③ 两根因的基线 div 对照与 sc
  身份/视图面，修复后双绿（页面落账前未入库，探针态见 r3 轮日志）。
- `cargo test -p zero-dom test_selectedcontent` ×2 passed（解析 pass unit tests）。
- **门禁期回归一发与修复**：首轮 make test 10F——其中 `r2846`（output value setter
  替换 textContent 后同 turn 读回旧值 '5'）为本切片 textContent 融合臂（③）引入：
  OUTPUT value setter 走裸 `__zw_set_text` host 直写不进 JS pending 视图，融合臂按旧
  overlay 组合出 stale 值；修复 = value setter 写后同步重注册本地文本视图（R49
  `_zwRegisterTextEl`，'' 清注册，与 textContent= 同视图通道）。修复后单测绿。
  stash 判定（clean main + c80e3dd16^ 双点复测）+ s321 make test 日志比对定谳：其余
  9F = **对端 js_dom_bridge 在册红腿**（CSSOM r2809/r2810/r3276/p99s 族 + p56
  adopted + fetch matrix + iframe SW + websocket ×2；rendering-compat R5441 账本
  「5→9 案跨轮漂移、已飞书通知、workspace 红账移交」，引入窗 81b00542d→8a26c5a44
  远早于本切片）——非本流引入，按在册口径以「本流零新增」过闸。
- 全量门禁终值：`make test` **3787P/9F**（9F = 对端 js_dom_bridge 在册红腿零新增，
  rc 非零属在册账）；`make reftest` **725/725 100%** 零回归；`cargo clippy -p
  zero-dom --all-targets -- -D warnings` 绿（doc_lazy_continuation +
  collapsible_match ×2 修）；`cargo fmt --check` 绿；语料终版复验 r6 = 12P/9F 零位移。

## 下一步

1. s325 = Face E select 解析定制族施工（s323 候选②；html5ever 0.38 升级后先复测
   Face E 各页——剥壳/闭合钉形预计大部分被新解析规则直接覆盖，剩余面再施工）。
2. Face A 协付点（渲染流 appearance:base-select 交付后回头收 test1 案）。
3. selectedcontentelement 解析期初始关联 FIXME（语料出现时补）。
