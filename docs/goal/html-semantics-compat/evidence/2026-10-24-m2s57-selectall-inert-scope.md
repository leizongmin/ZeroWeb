# M2 切片 57（s57）——D 族 shim 面收口：R5264 execCommand('selectAll') 模态 inert 作用域 + 45F 全量复核钉案

日期：2026-10-24。全域 45F 全量复核（对照 s46 分诊分组逐案核对失败断言机制），D 族中唯一非几何门案 **inert-node-is-unselectable 1F → 1P**（R5264），全域 438P/45F/0T → **439P/44F/0T**（GAIN 1 / REG 0）。

## 分诊：45F 逐案断言机制复核

s46 分诊五桶沿用，本轮逐案读测试源钉死断言机制：

- **A 族 16F**（dialog-popover-closedby-complex）：shadow DOM slots 变体，declarative shadow 底座，待用户点名联合切片。
- **C 族几何/UA/绘制门 ~21F**：abspos-dialog-layout 6（UA 居中布局）、default-color 2、dialog-overlay 2、top-layer-position 1、non-modal-dialog-layout 1、multiple-centered-dialogs 1、modal-dialog-scroll-height 1、modal-dialog-in-visibility-hidden 1（UA sheet visibility）、dialog-closedby-bounds-clicking 2（elementFromPoint 对 dialog 自身盒命中返 null）、modal-dialog-selection 1（testdriver 拖选 origin 坐标需 dialog 布局盒）、dialog-inert 1 / modal-dialog-ancestor-is-inert 1（clickOn 用 gBCR 定位 + elementFromPoint——本轮读源钉死：两案点击定位全吃几何）、inert-svg-hittest 1、dialog-showModal 1（top-layer 绘制序）、closed-details-layout-apis 1（content-visibility 布局 API）。
- **D 族 shadow 焦点恢复 4F**：dialog-focus-previous-outside 3（shadowroot outside/inside、slotted 焦点恢复）、form-submit-dialog-shadow 1（method=dialog 跨 shadow 提交）。
- **spec-consistent divergence 1F**：dialog-closedby-corner-cases ESC（s53 四重证据定性，不弯就）。

无剩余「纯 JS 断言 + 本流可收」案——inert-node-is-unselectable 是最后一案。

## R5264 实现

`document.execCommand('SelectAll')` 此前未接（part06:2223 注记「未接 → false」，permissive 落 `return true` no-op）→ `window.getSelection().toString()` 恒 ''。

修复（part06）：execCommand 新增 `selectall` 分支——作用域取**模态 inert 遮罩**（HTML inert：模态 open dialog 外文档内容不可选）：`_zwDialogModalTopSel()` 命中 → `document.querySelector` 取栈顶模态 dialog 元素，否则 body；复用 `_selection.selectAllChildren`（R2804 面，DocumentType 门 / detached no-op 内建），toString 提取文本既有。不派 beforeinput/input（选区命令不经编辑宿主事件序，与 undo/redo 同口径——R3254-M2 表注释该语义本就成立）。`_zwQueryCommandState` 补 selectAll supported/enabled 真实反射。

对照面核验：inert-node-is-not-highlighted（reftest，视觉无高亮本就满足）；inertness-with-modal-dialogs-and-iframes / details-findstring-crash（直接调 selectAllChildren，路径未动）；form-controls-id-removal-crash / auto-expand-ax-slot-recalc-crash（crash 测试，testharness 语料不载，runner 无 crash）。

## 验证

- 目标文件 inert-node-is-unselectable **1F → 1P**
- 全域 interactive-elements 98 files **438P/45F/0T → 439P/44F/0T**（GAIN 1 / REG 0 / Timeout 0）
- zero-engine lib **3448P/0F**
- fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项

- 本流 engine shim 面可收口项**清空**。余 44F 全为跨流/用户门：渲染流几何/UA/绘制门 ~21F（逐案钉名见上——跨流协调交付物，渲染流可按名取案）、shadow 底座 20F（A 族 16 + D 族 4，declarative shadow 联合切片待用户点名）、spec-consistent divergence 1F 不弯就
- 挂账序不变：parser form pointer 4F 用户决策缓行
