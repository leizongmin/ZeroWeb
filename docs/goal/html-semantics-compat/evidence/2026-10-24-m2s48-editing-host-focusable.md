# M2 切片 48 — editing host focusable area——Tab 序补 contenteditable 宿主（R5251）（2026-10-24）

## D 族分诊（本切片主产出之一）

控制面分诊地图将「D 族 focus/inert 10F」列为 shim 面候选。逐案单跑取证后重分诊——
10F 中可收口面只有 focusability 1F，其余 9F 全部改判挂账：

- **dialog-focusability 1F → 本切片收口（GAIN 1）**。editing host 不在 Tab 候选集。
- **dialog-focus-previous-outside 3F + form-submit-dialog-shadow 1F → 挂账（H 族
  前置）**。首层根因是 `<template shadowrootmode=open>` 声明式 shadow root 从未
  被提升——`host.shadowRoot` 返 null，测试第一行 `querySelector` 即炸
  （`Cannot read properties of null`）。shim 全 part 无 `shadowrootmode` 处理
  （attachShadow 命令式已有，R2926/R194b/R179）。与控制面 H 族「declarative
  shadow DOM parser+渲染流联合切片待启」同项，4F 记入其范围；焦点恢复逻辑
  （R4600 shadow/slot 边界）是其后续面。
- **dialog-inert 1F + modal-dialog-ancestor-is-inert 1F + inert-svg-hittest 1F →
  挂账（渲染流面）**。dialog-inert 断言 `document.elementFromPoint` 命中 open
  dialog 返 null（top-layer 无 hit-testing 几何）；ancestor-is-inert 点击穿透
  inert 祖先；svg-hittest 关 dialog 后 SVG 内容命中不激活。三者同底座 = 命中
  测试几何链缺失，与 C 族 top-layer/UA 布局 15F 合并记账。
- **inert-node-is-unselectable 1F + modal-dialog-selection 1F → 挂账（渲染流面）**。
  文本选择断言依赖 hit→selection 链（选择窗口为空串），同上合并。

## 根因（focusability）

WPT dialog-focusability：`<dialog contenteditable=true open>` 须入 Tab 序
（期望序 step 9/11）。实测 Tab 从 after3 直接跳 within4。修复前我方两处
Tab 可聚焦判定都无 editing host 分支：

1. **runner `dispatch_tab_focus` 页面候选序脚本**（testharness.rs，S11 接线）——
   候选选择器 `a[href],area[href],button,input,select,textarea,summary,[tabindex]`
   不含 contenteditable 宿主。**这是本案观测路径**（testdriver send_keys TAB →
   dispatch_tab_focus）。
2. **host `FocusManager::scan_node`**（crates/dom/src/focus.rs）——`FOCUSABLE_TAGS`
   + tabindex 判定，无 editing host。MoveFocus 效果面（webview host 层
   `next_focus_selector`）同语义缺口，一并收口保两端一致。

spec：editing host（contenteditable true 状态且父不可编辑）是 focusable area
（HTML §focusable-area / §sequential-focus-navigation-processing-model）。

## 修复（R5251 两刀）

- **runner `dispatch_tab_focus`**：候选选择器尾补 `[contenteditable]`；无
  tabindex 的显式 contenteditable 元素走 host 门——`el.isContentEditable`
  （shim R3187 枚举态：大小写不敏感 + inherit）为真且 `parentElement` 非可编辑
  → 入零序；嵌套 editable 非 host 与 `contenteditable=false` 孤岛不入序。显式
  tabindex 元素优先原 tabIndex 臂（含 `""`），原生可聚焦元素落原 else 臂（首版
  误让 `<button>` 落 CE 臂 continue 掉——回归当场修正）。
- **dom `FocusManager`**：`contenteditable_state`（Some(true)/Some(false)/None
  = inherit）+ `is_editable`（显式状态截断继承）+ `is_editing_host`（自身 true
  状态且父不可编辑）三 helper；`scan_node` 零序分支补 `eligible && is_editing_host`。

## 验证

- dialog-focusability 单案 1 Pass/0 Fail：目标子测试 **Fail → Pass**。
- interactive-elements 全域 98 files 482 subtest **425P/55F/2T，对 s47 逐位
  REG 0 / GAIN 1**（GAIN 即目标子测试）。
- zero-dom lib **866P/0F**（863 + 3 新单测：host 入序复刻、`contenteditable=""`
  短记 + 嵌套非 host、inert 祖先排除）；fail-first 先行（2 新测红复刻
  `after3→within4` 失败形态）后转绿。
- zero-engine lib **3444P/0F**（3443 + 契约 part02
  `test_next_focus_selector_editing_host_dialog_r5251`——bridge 层 Tab 序跨
  dialog4 与反向）。
- fmt clean；clippy --workspace --all-targets -D warnings clean（test-guard 包裹）。
- release runner 重建并核锚点（`hasAttribute('contenteditable')` ×1）。

## 门禁

零新增依赖。数据
[2026-10-24-m2s48-editing-host-focusable.json](2026-10-24-m2s48-editing-host-focusable.json)。
