# M2 切片 374（s374）——inert/编辑点击面双施工（inert-node-is-uneditable 翻绿；dialog 域真案 25F→24F）

日期：2026-11-10（承 s373 挂账：inert 编辑屏蔽 + inert 剩余面）
域：html/semantics/interactive-elements/the-dialog-element（inert-node-is-uneditable / modal-dialog-ancestor-is-inert）
性质：**inert-node-is-uneditable 翻绿（-1）；modal-dialog-ancestor-is-inert shim 半边收口（retarget 端到端验证），上游缺口定位 UA dialog 定位 R-s31 挂账**

## 施工①：CE 宿主 mousedown caret 落点（inert-node-is-uneditable 翻绿）

**勘根链**（探针 zz_probe_s374，勘根毕删）：

1. 败态 = 第二断言（dialog 内 editable 宿主键入未生效，`"I'm editable."` 未变）；第一断言（inert span 键入不变）平凡过。
2. 点击/聚焦链全通：clickOn 坐标正确（offset 系近似可用）、`activeElement=editable` 聚焦成功——**缺口在 caret**：`selection[rangeCount=0] caret_range=null`，`__zw_ce_insert` 空选区守卫静默 no-op（part01 M2 语义）。
3. 结构定性：mousedown caret 默认动作只有 drag-select 臂（part06 selection-compat M2），`_zwDocDragSelectAllowed` 对 CE 子树 decline（「编辑域自有 caret 面」）；而编辑域 caret 面只挂 focus() **脚本路径**（part04 `__zw_initial_selection_on_focus`，注释明示 mousedown 路径归点击 caret 管线）——**该管线不存在**，CE 点击 caret 从未落位。InsertText 动作链（runner printable → `HtmlUserAction::InsertText` → user_actions.rs CE probe → `__zw_ce_insert`）由此断在第一环。

**施工**（part06 `__zw_pointer_down_sequence` 拖选臂 else 分支）：CE 宿主 mousedown caret 落点——门 = `_zwIsFocusableSel(downEff)`（与焦点迁移默认动作同门，inert CE 宿主不聚焦亦不落 caret，「inert 不可编辑」断言面成立）；落点 = CE 内命中点 caret（`_zwCaretFromPointDoc` 同源，Chromium 点击位语义）→ 宿主链上行须达 contenteditable=true 且不跨表单控件（input/textarea 内点击属控件 caret 域不入臂）；命中 miss 回退宿主首 caret 位（`_zwFirstCaretPositionInHost` 复用初选区 machinery）。

**验证**：探针钉 `rangeCount=1 sc=text:"I'm editable":11`（点击位锚定）→ 键入 `"I'm editablae."`（offset 11 插入，点击位语义）→ solo **Pass**。

## 施工②：模态 inert 命中 retarget（modal-dialog-ancestor-is-inert shim 半边）

**勘根**（探针 zz_probe_s374b，勘根毕删）：

1. 败态 = click 落 inert 祖先后 body/div/dialog 全收冒泡（`handled={"dialog","div","body","document"}`）；期望仅 document 收到（Chromium hit-test 对 inert 全落空时指针事件 target=文档根）。
2. 该案前提依赖 UA dialog 定位：ZW 无 `dialog { position: absolute; inset: 0; margin: auto; fit-content }` UA 簇（R-s31 挂账面），dialog `pos=static` 渲染在祖先 div 内部 [101,201,50,50]——点击 div 中心几何命中其子 dialog（非 inert），真 inert 命中只在 div 边界环（探针 efp2=ancestor @ (100,226)）。
3. **retarget 端到端验证**：真 inert 命中下 `handled2={"document":true}`——body/div/dialog 全不在传播路径，正是该案要求的 Chromium 观察面。

**施工**（part06）：`_zwPointerInertRetarget(sel)`——命中元素被栈顶模态阻断（`_zwDialogModalFocusGateOk=false`，spec blocked by a modal dialog）时整段指针序改道 html 根 sel（R40 既有 'html' 派发先例；click 冒泡 html→document，body 及 modal 祖先均不在传播路径）。两处注入：
- down 腿：light-dismiss check（自有模态门）与 `_dragSel0` 捕获**之后**——s372 分类吃原始 sel，此后悬停/pointerdown/mousedown/焦点/caret 按 retarget 走。
- up 腿：light-dismiss up 腿（s372 backdrop-through）**之后**、pointerup 派发之前——closedby 组合的 backdrop-through 分类不受扰。
- **top-layer 豁免**：焦点门不含 top-layer 成员判定——栈外 top-layer dialog 及其子树（closedby 组合 clickOn(dialogA-under-modalB)、popover 面）可交互，祖先链查 `_zwTopLayer` 命中即豁免；top-layer 命中走既有 backdrop-through，事件通道不动。

## 验证

- **interactive-elements 域全域 sweep**（FILTER=interactive-elements，463P）：真案 **20F→19F**（-1=inert-node-is-uneditable 翻绿；s373 口径 22F 含 forms/select-dialog-mode 2F 不在本 sweep 范围）。零新增红——新见 `closed-details-layout-apis.tentative` 1F 为 content-visibility 布局面（源码零 dialog/CE 面，本刀双改动门控均不可达，渲染流域既有红出域）；zz_ld_probe_s372 恒红探针 3 子测试单列口径不入真案数。
- **modal-dialog-ancestor-is-inert**：维持 1F（UA dialog 定位上游缺口——R-s31 挂账承继；shim retarget 半边已就位并端到端验证）。
- **editing-run 域**：0F 全绿（CE caret 臂邻接域）。
- **selection 域**：14F 逐名核源码——editing/event（execCommand 面）/document-selectionchange（API 面）/deleteFromDocument（shadow 面）零 Actions/pointerDown/mousedown/showModal 指针链可达性，既有红非本刀波及。
- **engine lib**：3826P/7F——7 红逐名同构 s373 基线集（img settle/fetch/worker/websocket），零新增红。
- 渲染面零变化（interaction 通道 only）→ reftest/product-smoke 按影响选择检查不适用。
- shim 13 文件拼装 node --check PASS（/tmp/s373/shim_check.js 复用）。

## 挂账（移交下一切片）

1. **modal-dialog-ancestor-is-inert 上游缺口**：UA dialog 定位全规则簇（`dialog { position: absolute; inset-inline 0; margin: auto; width/height: fit-content }` + `dialog:modal` block inset）随 R-s31 渲染流（s372 先例：定位注入需布局侧 abspos 收缩/margin-auto 居中配套，单独注入有全 modal corpus 布局回归风险）。shim retarget 半边已就位，上游落地后该案应直接翻绿（探针已证真 inert 命中 → document 观察面正确）。
2. 既有在册承继：default-color 案2（color-scheme 分支）、select-dialog-mode-focus 2F（focus 行为面）、select picker open 态注入通道、overlay+top-layer 强制定位、inert hit-testing 穿透下层近似面、fresh parse 通道 modal 注入缺口、基建面 T=3+probe8/9/10 转正决策。
3. 探针产物已删（zz_probe_s374/b gitignore 区）；验证日志 /tmp/s374/（本地）。
