# M2 切片 29 — 模态焦点门接页面 focus() 与 shadow 包含判定（2026-10-23）

## 范围

s28 挂账：模态 inertness 焦点门只覆盖宿主驱动路径（pre-focus 步、
mousedown 默认动作），页面直调 `el.focus()` 不在门内——
inert-node-is-unfocusable 13F。本轮收口 + 门包含判定补全。

## 修复（part01/part04，共享路径）

1. **part04 `focus()` 接门**：页面直调 focus() 走 proxy trap（R114
   detached 臂、label 前递、R148 解析节点臂共用入口），门插在已聚焦
   no-op 之后、失连臂之前。inert-node-is-unfocusable 的 testFocus 面
   （focus 事件 sentinel）全部经此路被门拒。
2. **门位在 R4003b label 前递之后**（中途回归钉定的次序）：前递源
  （label 本就非 focusable area）不受模态 inert 门——inert 语义作用在
   焦点落点；inert-label-focus 断言面（模态外 inert label 前递进模态
   内非 inert target）要求前递先解目标，目标经递归 focus() 再入 trap
   自行过门。门先落在前递前时该案 PASS→FAIL，移位后归位。
3. **showModal 栈推进移到 focusing steps 之前**：嵌套 showModal 时新
   dialog 已是栈顶模态，其子树 focusing steps 过门不被外层模态拒。
4. **R4600b inert 分支 focus fixup 落 body 走 `__zwModalGateBypass`**：
   focus fixup rule 是 UA 级焦点回落（spec
   interaction.html#focus-fixup-rule），非 focusing steps——模态下落
   body 需绕行门；Chromium 同款 activeElement===body。
5. **失连元素门内跳过**：R114/T55 detached focus 契约保持（spec
   focusable area 要求 being rendered，inert 只作用于渲染内容）。
6. **part01 `_zwDialogShadowInclContains`**：`proxy.contains` 只覆盖
   light DOM，delegatesFocus 影子树内锚（既有契约 R4600 R8）被误拒。
   parentNode 上行遇 shadow root（`__zwHandle` 命中 `_shadowHandles`）
   换 host 继续；parentNode 尽头而节点自身是 shadow/frag 容器经
   `_shadowHandleMeta` 跳 host（路数同 part06 activeElement host
   retarget）；64 步 guard 防反链环。spec 包含语义即 shadow-including
   inclusive descendant。

## 契约

- `test_dialog_modal_focus_gate_s28` 增 G5（页面 focus()：模态外元素
  focus() 后 sentinel 空、activeElement 保持模态内；模态内正常）与
  G6（失连 createElement focus() 归解析节点所有权，不受门）。
- `test_dialog_focusing_steps_r4600` R8（delegatesFocus 影子树内锚）
  在门接入后暴露 light-DOM 包含缺口，`_zwDialogShadowInclContains`
  修复后复绿——影子包含判定的常驻回归面。

## 账本（s28 → 修复树）

dialog 集群 82 files 407 subtest（数据见
[2026-10-23-m2s29-page-focus-gate.json](2026-10-23-m2s29-page-focus-gate.json)）：

- **317P/87F/3T** vs s28 同文件集 303P/101F/3T。
- 逐 subtest 对 s28 账本 diff：同 key 回归 **0**，同 key 增益 **14**
  ——inert-node-is-unfocusable 13F 全清（15/15），连带增益 1
  （remove-dialog-should-unblock-document F→P，焦点回落路径）。
- engine lib 全量 **3426P/0F**。
- 3T 与 87F 均为 s28 基线既有（rendering inert 整体、layout/centering、
  popover 交互等仍挂账）。
