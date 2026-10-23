# M2 切片 46 — 历史 menu 面 IDL 归属收口——track/option tag 门 + R3069 MENU/MENUITEM 排除（R5249）（2026-10-23）

## 范围

s46 轮为余 60F 归因分诊轮 + 顺手收口：分诊产出六族挂账地图（见下），
族 G（contextmenu-historical 3F）定性为「IDL 名无 tag 门」同模式且可
shim 面收口，本切片一并落地。

## 归因分诊（余 57F/2T 六族地图）

- **A. closedby/light-dismiss×slot 交叉 16F**（dialog-popover-closedby-
  complex）：全部「modeless/modal dialog 双实例 + shadow DOM slots 点击」
  组合面——closedby 特性 + light dismiss + slot 投影三特性交叉。Chrome-
  parity 同红证据 crbug 417571176（s41 已引，2026-04 仍 failing）。
- **B. light dismiss 5F**（closedby-bounds-clicking 2、light-dismiss-
  pointer-capture 2、closedby-corner-cases 1）：light dismiss 特性缺口，
  与 A 同族。
- **C. top-layer/UA 布局 15F**（abspos 居中 6、default-color 2、overlay
  2、top-layer-position 1、non-modal 1、multiple-centered 1、scroll-
  height 1、visibility-hidden 1）：modal 居中/overlay/默认色/top-layer
  position 计算面——渲染流 UI 面，非 shim 单边可收。
- **D. focus/inert 10F**（focus-previous-outside 3 既有挂账、focusability
  1、inert 1、inert-unselectable 1、inert-svg-hittest 1、ancestor-inert
  1、selection 1、form-submit-shadow 1）：inert 语义与 shadow 焦点恢复。
- **E. dialog API 语义 5F**（close-via-attribute 2、active-document 1、
  showModal throw detached 1、movebefore 1）：可 shim 收口候选（下轮靶）。
- **F. details 零散 4F/1T + dialog 零散**（name-attribute 3、auto-expand
  1、closed-details-layout-apis 1、toggleEvent 1T）：name 互斥展开为新
  特性面；toggleEvent/focus-after-close 两 T 既有挂账。

## 根因（三半边，「IDL 名无 tag 门」同模式）

1. has trap（part05）track IDL 列 `label`/`kind`/`srclang`/`default`
   （media-elements M1 切片 3 加）无条件命中——历史 menu/menuitem（现代
   spec 无任何 IDL 属性，menuitem 已删为 HTMLUnknownElement）
   `'label' in el`/`'default' in el` 假阳性；
2. set trap R3069 臂 `_reflectedStringAttr` 的 _REFLECTED_STRING_FLAT 含
   `type` 无 tag 排除——`menu.type =` 误写内容属性（getAttribute
   反射漂移，expected "context" but got "toolbar"）；
3. 同臂 `p === 'label'` 显式列（R3069-fix，OPTION round-trip）无 tag 门
   ——`menu.label =` 同误写。

## 修复（R5249 双刀）

- **part05 has trap tag 门**：`kind`/`srclang` 收 TRACK（HTMLTrackElement
  IDL 归属）；`label`/`default` 收 TRACK+OPTION（HTMLOptionElement.label
  /.default，R3069-fix 读侧对称）。门内 src/textTracks/addTextTrack/
  track/controlsList 保持原样（corpus 无 menu 面观测，不扩面）。
- **part05 R3069 臂 MENU/MENUITEM 负门**：条件首项排除——赋值落 expando
  默认路径、内容属性不变；name 臂自带 tag 门不受影响；FLAT 表其余名
  （input.type 等）零变化（契约钉 2(d)）。

## 验证

- contextmenu-historical 单案 **11/11 Pass**（3F → 0F）。
- interactive-elements 全域 98 files 482 subtest **423P/57F/2T 对 s45
  逐位 REG 0 / GAIN 3**（GAIN 即三案；track/option/menu 排除面零误伤）。
- zero-engine lib **3442P/0F**（3440 + 2 新契约 part78）。
- 契约翻转力实证（临时 patch 断言红后还原）：has trap tag 读恒 TRACK →
  钉 1(a) 红（假阳性复活）；R3069 负门短路 → 钉 1(c) 红（反射漂移复活）。

## 契约（part78）

- `historical_menu_idl_exclusion_r5249`：(a) menuitem 无 track IDL 成员、
  (b) menu 无 label 成员、(c) menu.type= 不写内容属性、(d) menu.label=
  同、(e) menuitem.constructor 为 HTMLUnknownElement（既有面防误伤）。
- `track_option_idl_gate_kept_r5249`：(a) track 四成员 in 保持、(b) option
  label/default in 保持、(c) option.label= round-trip、(d) input.type=
  round-trip（FLAT 面不受负门影响）。

## 已知边界

- `menu.type`/`menu.label` **读**侧（R3037 FLAT 读）仍会读内容属性返值
  （真浏览器 undefined）——测试无此面，本轮不扩面；set 已落 expando，
  set→get round-trip 经 expando 读自洽。
- src/textTracks/addTextTrack/track/controlsList 等 track 接口其余成员
  的 `in` 可见性仍无条件——menu 面无观测，按挂账序不扩面。

## 门禁

zero-engine lib 3442P/0F、clippy --workspace -D warnings（test-guard 包
裹）/fmt clean；零新增依赖；corpus runner release 二进制含 R5249 符号
（`_r5249Tag`/`_t5249`）。数据
[2026-10-23-m2s46-historical-menu-idl-gate.json](2026-10-23-m2s46-historical-menu-idl-gate.json)。
