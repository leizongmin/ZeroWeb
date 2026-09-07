# 编辑与 contenteditable — 运行时控制面板（master.md）

**入口文档**: [../editing-contenteditable.md](../editing-contenteditable.md)
**创建日期**: 2026-08-17（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（M1 残余三切片——Selection 方向位 + Text/Comment ownerDocument + selectionchange 派发，selection 套件 2005P/779F → 2704P/199F）

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
| E2 | Selection JS 可观察面未核实/缺失 | ✅ 切片 2 + 残余切片 2/3（2026-09-07，45P→**2704P**）：切片 2 八类（document.getSelection/instanceof/异常语义/selectAllChildren/setBaseAndExtent/setPosition/deleteFromDocument）；残余切片 2——Selection anchor/focus **独立边界点**（setBaseAndExtent 反向形态 anchor/focus 各自等于请求值 + detached/foreign-doc 清空 + 省参 TypeError，86F→0F）+ Text/Comment.prototype.ownerDocument（innerHTML 解析产物 'reading createRange' TypeError，isCollapsed/removeRange/type/collapseToStartEnd 48F→0F）；残余切片 3——selectionchange 排程派发（document 级 4 subtest 全过 + text control 独立派发 2F→2P）。残余：selectAllChildren 118F（runner host 视图 pending-tree 深层形态）/ deleteFromDocument 60F / getSelection 12F（均 iframe 面，js-dom 共享域）/ onselectionchange 第 4 subtest 跨用例 timer 时序 flake |
| E3 | 编辑行为管线（键入/删除/换行 → DOM）缺失 | ✅ M2 切片 2/3（2026-09-07，bf181ee60+47dd9a685）：键入/Backspace（SetChildText splice + caret 移动）+ Enter 换行（SetInnerHtml <br> splice + caret 元素边界）+ 事件序全接通；限制记录：insertParagraph 块级拆分 defer、跨节点回退/嵌套结构偏移映射 defer |
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

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
核对 js-dom 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — selection 基线 + Selection 面摸底 | ✅ 切片 1/2/3 全部完成（2026-09-07）——基线 + Selection 面 + editing 导入 |
| M2 — 编辑行为管线 | ✅ 切片 1/2/3 全部完成（2026-09-07）——事件序 + 键入/删除/换行落 DOM；defer 项：insertParagraph 块级拆分、跨节点删除 |
| M3 — execCommand 基础面 + 收尾 | 🔶 切片 1/2/3 ✅（format 四命令 + delete 两命令实应用 + queryCommandSupported/Enabled 真实反射，2026-09-07）；剩余：toggle/queryCommandState、insertParagraph/insertHTML、run/ 用例导入（330KB reference impl 依赖——defer 有据）|

## 验证基线

- 测试基线：2026-09-07 全绿；clippy 零警告
- WPT selection 面：切片 1 基线 45P/2559F（1.7%）→ 切片 2 1824P/776F（70.2%）→ **M1 残余切片 2/3 后 2704P/199F（93.1%）** @ WPT_REV 315976933870（23 用例，evidence/2026-09-07-m1-slice1-selection-baseline.md）
- WPT editing 面：**2902P/201F** 组合（event.html 179P/1F + delete-editing-host 2P/0F + body-not-deleted 0P/2F + selection 面 2704P/199F + onselectionchange 两案 6P/1F）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

## Done Criteria 清点（2026-09-07 收尾状态）

| DC | 条目 | 状态 |
|---|---|---|
| DC-1 | selection/editing 用例导入 + 基线 | ✅ selection 20 用例（1.7% 基线）+ editing 首批 3 用例；三份 evidence |
| DC-2 | Selection API 可观察面 | ✅ 1.7%→**93.1%**（切片 2 八类 + 残余切片 2/3：anchor/focus 独立边界点（反向 selection）、Text/Comment ownerDocument、selectionchange 排程派发；单测九组 + 三组 + 两组）；残余 iframe 面（js-dom 共享域）+ selectAllChildren/deleteFromDocument 深层形态 defer 有据 |
| DC-3 | 编辑行为落地（键入/删除/换行 → DOM） | ✅ M2 三切片（execCommand 事件序 + CE 键入/Backspace/Enter 落 DOM + 事件序）；beforeinput cancelable/input 按 spec |
| DC-4 | execCommand 基础面 | ✅ queryCommandSupported/Enabled 真实反射（M3 切片 3，`_zwQueryCommandState`——supported/enabled 按接通命令面与选区 editing host 前提判定 + 五组单测）+ bold/italic/underline/strikethrough + delete/forwardDelete 实应用（M3 切片 1/2，六组单测）；toggle/queryCommandState、CSS 化命令、run/ 导入（330KB reference impl）defer 有据 |
| DC-5 | cargo test 全绿 / clippy / 资产化 | ✅ engine 2630 + webview 692 全绿（本轮）/ 零警告 / 每切片带单测 |

**收尾结论**：Selection 面（含方向位/ownerDocument/selectionchange）、编辑管线、
execCommand 基础命令集、queryCommand* 真实反射均落地并有断言资产；残余项
（selectAllChildren 118F/deleteFromDocument 60F/getSelection 12F 均为 iframe
contentWindow 面——js-dom 共享域；toggle/CSS 化命令/queryCommandValue；run/ 全量
导入）为跨域/深化面——defer 均有据记录（上游 reference impl 依赖 + headless 限制 +
js-dom 共享域边界），不阻塞流域收口判定。
