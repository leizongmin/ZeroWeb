# 编辑与 contenteditable — 运行时控制面板（master.md）

**入口文档**: [../editing-contenteditable.md](../editing-contenteditable.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（E2 切片 6 收口——iframe realm Selection/Range 面；deleteFromDocument 60F→0F + getSelection 12F→0F；selection+editing 组合 **2898P/5F/1T，98.9%**）

---

## 当前状态

**专项定位**：键盘/编辑方向三拆之二。把 contenteditable 从属性反射（R3187）深化为可用
编辑基础（Selection/Range 可观察 + 键入落 DOM + 编辑事件 + execCommand 基础面），WPT
`selection`/`editing` 真实用例驱动。

**与兄弟 goal 的边界**：
- keyboard-default-actions — 非编辑宿主默认动作归其管；编辑宿主内按键归本目标（分发
  顺序：编辑宿主优先消费）
- keyboard-page-scrolling — 滚动键归其管
- js-dom — zero-dom range.rs 是其 deep-review 过的共享模型（不撞 dom_bindings 面）；
  Selection JS 绑定进 js_dom_shim 前先 `git log` 核对（run-rules §9）

## 实测基线（2026-08-17 立项时）

### 现有实现

- ✅ contenteditable 反射：R3187（part01.js:328）枚举状态求值 getter/setter
- ✅ Range 模型：zero-dom range.rs（952 行，R3377 deep-review 确认健壮；insert_node 有
  文本节点字符偏移分裂底座；跨容器分支已知简化记录在案——本目标 Selection 面若逼出
  跨容器需求即到接线时点）
- ✅ 宿主选区底座：page_selection.rs（browser 侧文本选区基础设施）
- ✅ execCommand copy/cut 返 true 语义（part06.js:1432 桩——format 类不真应用）
- ⚠️ 编辑行为为零（键入/删除/换行不落 DOM）
- ⚠️ `window.getSelection()` 可观察面待摸底（M1 首项）
- ⚠️ beforeinput/input 事件缺失
- ⚠️ WPT `editing`/`selection` 未导入，无基线

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| E1 | WPT selection/editing 用例覆盖为零 | ✅ selection 首批 20 用例 2026-09-07（基线 1.7%）；editing 目录未导 |
| E2 | Selection JS 可观察面未核实/缺失 | ✅ 切片 2 + 残余切片 2/3（2026-09-07，45P→**2704P**）：切片 2 八类（document.getSelection/instanceof/异常语义/selectAllChildren/setBaseAndExtent/setPosition/deleteFromDocument）；残余切片 2——Selection anchor/focus **独立边界点**（setBaseAndExtent 反向形态 anchor/focus 各自等于请求值 + detached/foreign-doc 清空 + 省参 TypeError，86F→0F）+ Text/Comment.prototype.ownerDocument（innerHTML 解析产物 'reading createRange' TypeError，isCollapsed/removeRange/type/collapseToStartEnd 48F→0F）；残余切片 3——selectionchange 排程派发（document 级 4 subtest 全过 + text control 独立派发 2F→2P）。残余切片 4（2026-09-07）：**selectAllChildren 118F→0F**（`nodeType === 9` in-doc 短路移除——foreignDoc/xmlDoc 的 parentNode=null 沿链上行自然 false，旧捷径误判外文档节点改写 selection；本文档经 _zwSameNode 命中）+ **Selection.prototype.deleteFromDocument** 挂载（WebIDL 原型成员，.length 断言）。切片 5（2026-09-07）：body-should-not-deleted 2F→2P（① outerHTML getter 加 R380 融合门——pending 桶当代时从融合 childNodes 序列化，此前同 execute 内结构 mutation 后读恒 stale；② harness dispatcher 不再尾部 append `<script>`——被解析器归入 body 成为页面可见 DOM 污染 documentElement.outerHTML 精确断言，改 runner 侧 run_page_scripts_strict 后 execute_script 执行，时序不变）。切片 6 收口：deleteFromDocument 60F→**0F** + getSelection 12F→**0F**（iframe realm 面：style 面板 + per-iframe Selection + createRange 原型链 + defaultView null + 空 selection no-op 勘误）。残余：deleteFromDocument-HTMLDetails 1T（editor-test-utils.js helper 已补 fetch，test 注册面待查）/ anchor-removal 2F + Document-open 1F / event.html Changing-selection 1F（legacy 形态记录不追）/ onselectionchange 第 4 subtest 跨用例 timer 时序 flake |
| E3 | 编辑行为管线（键入/删除/换行 → DOM）缺失 | ✅ M2 切片 2/3 + M3 切片 5（2026-09-07，bf181ee60+47dd9a685+切片 5）：键入/Backspace（SetChildText splice + caret 移动）+ Enter 换行（SetInnerHtml <br> splice + caret 元素边界）+ **insertParagraph 块级拆分**（SetOuterHtml 同型兄弟块重写）+ 事件序全接通；限制记录：跨节点回退/嵌套结构偏移映射 defer |
| E4 | beforeinput/input 事件缺失 | 🔶 M2 切片 1（2026-09-07，commit 74a0c879b）：execCommand 编辑类命令 editing-host 事件序（beforeinput cancelable+trusted → input bubbles+trusted，inputType 映射，全选区在 host 内前提，preventDefault 阻断）——event.html 104P/76F→179P/1F；残余：键入/删除管线（宿主侧 keydown → contenteditable DOM 变更）未接 |
| E5 | execCommand format 桩（不真应用） | 🔶 M3 切片 1/2（2026-09-07）：① bold/italic/underline/strikethrough inline 包裹（SetInnerHtml splice + 标签/实体感知扫描）；② delete/forwardDelete 选区删除（SetChildText splice + 代理对安全 + caret 回落）；③ queryCommandSupported/Enabled 真实反射（切片 3，`_zwQueryCommandState`——copy/cut/paste 恒 true；编辑类按选区 editing host 前提；未接命令 false，替换无条件 true 桩）。限制：toggle/queryCommandState、CSS 化命令（color 族）、queryCommandValue 值面、跨容器删除、嵌套结构偏移映射 defer |

## 下一步计划

1. ~~**M1 切片 1**：`selection` 用例导入 + 基线~~ ✅ 2026-09-07（45P/2559F 1.7%，evidence/2026-09-07-m1-slice1-selection-baseline.md）
2. ~~**M1 切片 2**：Selection 面缺口修复~~ ✅ 2026-09-07（45P→1824P，70.2%；commit 679059d2f + 单测 test_selection_surface_r3254_m1 九组断言）。残余聚类：① selectAllChildren 456F「createRange of undefined」= runner host 视图 pending-tree 变动伪失败（js-dom 共享面已知限制，非 Selection API 缺陷）；② deleteFromDocument 60F iframe 面（test-iframe.html 依赖）；③ selectionchange 派发（onselectionchange-* 6F）；④ setBaseAndExtent 方向位（anchor/focus 反向形态 86F）
3. ~~**M1 切片 3**：`editing` 用例导入 + 失败聚类~~ ✅ 2026-09-07（editing/event.html 104P/76F——76F 全为 execCommand 不派发 input 事件 = E4/E5 缺口，M2/M3 领域；other/delete-editing-host 2P/0F；组合面 1930P/854F，evidence 同日追加段）
4. ~~**M2 切片 1**：execCommand editing-host 事件序~~ ✅ 2026-09-07（commit 74a0c879b；event.html 104P/76F → 179P/1F 99.4%，selection 面零回归；单测 test_execcommand_editing_host_events_r3254_m2 六组断言；残余 1F = Changing-selection target 身份比较，上游用例 legacy 形态记录不追）
5. ~~**M2 切片 2**：contenteditable 键入/删除落 DOM~~ ✅ 2026-09-07（commit bf181ee60；shim __zw_ce_insert/delete + webview CE 分支 + 单测五组；engine 2623/webview 691+18 全绿）。
6. ~~**M2 切片 3**：CE Enter 换行落 DOM~~ ✅ 2026-09-07（commit 47dd9a685；__zw_ce_enter <br> splice → SetInnerHtml mutation；InsertText{"\n"} 与无表单 Submit 双路由接通；单测四组）。残余：insertParagraph 块级拆分、跨节点选区删除（defer 记录）、editing/other 逐批追加（body-should-not-deleted 0P/2F = execCommand delete 实应用 = M3 领域）
7. ~~**M3 切片 3**：queryCommandSupported/Enabled 真实反射（DC-4 首条硬缺口）~~ ✅ 2026-09-07（`_zwQueryCommandState`——copy/cut/paste 恒 supported+enabled（ClipboardEvent 路径）；编辑类（_zwExecCmdInputType 表）supported，enabled 与 execCommand 编辑分支同前提（选区在单一 editing host 内）；未接命令（undo/styleWithCSS/justify*/未知）false；大小写不敏感；queryCommandValue 保持 ''。单测 test_query_command_reflect_r3254_m3_slice3 五组断言；engine 2630 全绿 + webview 692 全绿 + selection/editing 套件 2005P/779F 零回归）。残余 defer：queryCommandState/toggle、queryCommandValue 值面
8. ~~**M1 残余切片 2**：Selection 方向位 + Text/Comment ownerDocument~~ ✅ 2026-09-07（Selection 单例 `_anchorNode/_anchorOffset/_focusNode/_focusOffset` 独立边界点 + `_zwSync` helper——setBaseAndExtent/extend 写反向端点，getter 直读保存值，range 层恒正向化；detached/foreign-doc 清空 + WebIDL 省参 TypeError；Text/Comment.prototype ownerDocument getter。selection 套件 779F→262F，单测 test_selection_direction_r3254_m1_slice2 三组断言）
9. ~~**M1 残余切片 3**：selectionchange 排程派发~~ ✅ 2026-09-07（`_zwScheduleSelectionChange`——setTimeout(0) 记录式 timer 任务边界 + pending target 数组按身份去重（document 与 text control 并存）；Selection mutator `_zwSync` 钩子 + part03 setSelectionRange 钩子（target=控件自身）。onselectionchange-on-document 4 subtest（Timeout+0P→4P）、onselectionchange-on-distinct-text-controls 2F→2P；单测 test_selectionchange_dispatch_r3254_m1_slice3 两组断言）。**selection 套件终态 2704P/199F**（基线 2005P/779F，净 +699P）
10. ~~**M3 切片 4**：execCommand toggle 语义 + queryCommandState 真实反射~~ ✅ 2026-09-07（`_zwQueryFormatState`——选区起点祖先链 tagName 匹配（大小写不敏感）；`_zwExecCmdUnwrapFormat`——祖先链定位宿主直子 tag 元素整体替换为其 innerHTML（caret 落原位置）；execCommand format 臂按 state 走 wrap/unwrap toggle。单测 test_execcommand_toggle_state_r3254_m3_slice4 五组断言；M2 事件序测试同步适配（选区改非包裹文本——toggle 语义下 <b> 内选区会解除包裹）。**defer 记录**：跨部分包裹形态（选区只覆盖包裹的一段）unwrap 不处理；CSS 化命令 state 面仍 defer）。**残余聚类归因补记**：anchor-removal 2F = window named access（bare id 标识符）整体缺失——js-dom/native 绑定域（动态 innerHTML 后 id 无全局注册；探针归档）；script-and-style-elements 1F = Selection.toString 的 CSS 空白渲染语义（display:none 排除 + style/script 内容计入 + 块间换行）——渲染域投影，defer 有据
11. ~~**M3 切片 5**：insertParagraph 块级拆分（M2 切片 3 defer 解除）~~ ✅ 2026-09-07（`__zw_ce_insert_paragraph(sel)`——caret 处宿主 outerHTML 重写拆两同型兄弟块（SetOuterHtml mutation 流转宿主重解析；同型开标签 + contenteditable 属性保留）；execCommand insertParagraph 臂同语义（`_zwExecCmdApplyParagraphSplit`）；collapsed caret tail 段 = caret 起整段。单测 test_insert_paragraph_split_r3254_m3_slice5 三组断言（beforeinput 事件序 + SetOuterHtml 拆分形态 + ce 通道）。残余：嵌套结构内拆分（flat 模型限制，维持 defer）
12. ~~**M3 切片 6**：insertHTML 最小实现~~ ✅ 2026-09-07（`_zwExecCmdApplyInsertHtml`——选区在宿主直子文本节点内 → SetInnerHtml splice 中插 fragment 串（collapsed 纯文本/含标签原样插入/选区非空先删选中段三形态）；fragment 原样不消毒（上游语义——信任边界在页面脚本自身）；caret 落插入内容后。单测 test_insert_html_r3254_m3_slice6 三组断言。M3 剩余仅 run/ 全量导入（330KB reference impl 依赖——defer 有据）
13. **E2 残余切片 5**：outerHTML 融合视图 + dispatcher 移出 DOM → ✅ 2026-09-07（① part04 outerHTML getter 加 R380 同款融合门（pending 桶当代 → 融合 childNodes 序列化 + 属性 latest-wins 包裹）；② runner prepare_harness_html 尾部 dispatcher `<script>` 移除，改 run_page_scripts_strict 后 execute_script（时序不变，全部页面脚本之后）。body-should-not-deleted 2F→2P；selection 2827P/77F；keyboard 18P 零回归；dom 抽样（ParentNode-innerHTML 437P）零回归）
14. **E2 残余切片 6**：iframe realm 面地基 → 🔶 2026-09-07（两件前置落地：① fetch 通道 .html 响应内联相对 src 脚本；② runInlineScripts 顶层 var 导出尾声。selection 套件零回归（2827P/77F）。**未闭合**：bare iframe doc 内 common.js 自动 setupRangeTests 中途异常（错误被 runInlineScripts 吞）——下一切片需错误显面（win.__zwLastIframeScriptError）+ runner 路径探针（srcdoc 通道与 src fetch 通道行为有差异，探针实证 contentWindow var 均 undefined）+ R115 bare doc 的 body/子树完备性核查）
15. **E2 切片 6 收口**：iframe realm Selection/Range 面 → ✅ 2026-09-07（错误显面 + 逐层探针定位四缺口：① 工厂元素无 `.style`——test-iframe.html 顶层 display TypeError 经 R206 per-part catch 落 unexpectedException（此前 runInlineScripts 静默吞 + R115/R206 双通道误判延误定位）；② iframe win 无 getSelection/Selection——per-iframe Selection 实例（spec 每 Window 一个）；③ iframe doc/detached-doc（R204）createRange 缺 Range.prototype 链——rangeFromEndpoints 的 foreign-doc 域 instanceof false；④ detached doc defaultView 应 null 非 undefined。另勘误 deleteFromDocument 空 selection = no-op（selection-api「If this is empty, return」——既有单测同步修正）。**deleteFromDocument.html 60/60 + getSelection.html 18/18 全 Pass**；selection 套件 **2898P/5F/1T**；keyboard 18P 零回归）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — selection 基线 + Selection 面摸底 | ✅ 切片 1/2/3 全部完成（2026-09-07）——基线 + Selection 面 + editing 导入 |
| M2 — 编辑行为管线 | ✅ 切片 1/2/3 全部完成（2026-09-07）——事件序 + 键入/删除/换行落 DOM；defer 项：跨节点删除 |
| M3 — execCommand 基础面 + 收尾 | 🔶 切片 1/2/3/4/5/6 ✅（format 四命令 + delete 两命令实应用 + queryCommandSupported/Enabled + toggle/queryCommandState + insertParagraph 块级拆分 + insertHTML，2026-09-07）；剩余：run/ 用例导入（330KB reference impl 依赖——defer 有据）|

## 验证基线

- 测试基线：2026-09-07 全绿；clippy 零警告
- WPT selection 面：切片 1 基线 45P/2559F（1.7%）→ 切片 2 1824P/776F（70.2%）→ M1 残余切片 2/3 后 2704P/199F（93.1%）→ **切片 6 收口后 2898P/5F/1T（98.9%）** @ WPT_REV 315976933870（23 用例，evidence/2026-09-07-m1-slice1-selection-baseline.md）
- WPT editing 面：**2911P/78F** 组合（event.html 179P/1F + delete-editing-host 2P/0F + **body-not-deleted 2P/0F** + selection 面 2827P/77F + onselectionchange 两案 6P/1F）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## Done Criteria 清点（2026-09-07 收尾状态）

| DC | 条目 | 状态 |
|---|---|---|
| DC-1 | selection/editing 用例导入 + 基线 | ✅ selection 20 用例（1.7% 基线）+ editing 首批 3 用例；三份 evidence |
| DC-2 | Selection API 可观察面 | ✅ 1.7%→**97.3%**（切片 2 八类 + 残余切片 2/3/4：anchor/focus 独立边界点、Text/Comment ownerDocument、selectionchange 排程派发、selectAllChildren 外文档判定、Selection.prototype 成员面）；残余 iframe contentWindow 面（deleteFromDocument 60F + getSelection 12F——realm 隔离后续切片，根因归档）|
| DC-3 | 编辑行为落地（键入/删除/换行 → DOM） | ✅ M2 三切片（execCommand 事件序 + CE 键入/Backspace/Enter 落 DOM + 事件序）；beforeinput cancelable/input 按 spec |
| DC-4 | execCommand 基础面 | ✅ queryCommandSupported/Enabled 真实反射（M3 切片 3 + 五组单测）+ queryCommandState/toggle（切片 4 + 五组单测——format 命令 wrap/unwrap 全语义）+ bold/italic/underline/strikethrough + delete/forwardDelete 实应用（M3 切片 1/2，六组单测）；CSS 化命令、跨部分包裹 unwrap、run/ 导入（330KB reference impl）defer 有据 |
| DC-5 | cargo test 全绿 / clippy / 资产化 | ✅ engine 2637 + webview 692 全绿（本轮）/ 零警告 / 每切片带单测（M3 切片 3~6 共 +4 单测资产）|

**收尾结论**：Selection 面（含方向位/ownerDocument/selectionchange/外文档判定/原型
成员面）、编辑管线、execCommand 基础命令集、queryCommand* 真实反射均落地并有断言资产；
selection 通过率 **97.3%**（2825P/79F）。残余项均为 iframe contentWindow 面
（deleteFromDocument 60F + getSelection 12F——runner iframe 的外部 src 脚本不执行 +
with(window) 全局穿透主 realm，需 iframe realm 隔离切片；test-iframe.html helper 已补
fetch）与深化面（toggle/CSS 化命令/queryCommandValue、run/ 全量导入 330KB reference
impl、execCommand delete 空段落根元素 outerHTML 同步视图——R380 融合序列化扩展）——
defer 均有据记录，不阻塞流域收口判定。
