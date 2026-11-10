# M2 切片 388（s388）——select-multiple-resize 勘根收口：交互能力上限面（listbox 行渲染 + 元素拖拽 resize 双特性缺失），不施工挂账（零源码改动，域计数零位移）

日期：2026-11-10（承 s387 下一步候选：select-multiple-resize 勘根——s386 簇 C 三分余下的待勘根 1F）
域：html/semantics/forms/the-select-element（driving 案 select-multiple-resize-does-not-autoscroll.html，1F）
性质：**勘根分诊轮——三面定谳 + 归属判决，零源码改动**

## 勘根（driving 案断言链 + 本轮 solo 实测）

测试结构（issues.chromium.org/504375661 + css-ui-4 #resize）：`<select multiple>` 注入 80 个 option 后断言五连——①scrollTop 归 0（listbox 顶部起始）②**scrollHeight > clientHeight（listbox 可滚动，80 option 行产生内容溢出）**③④拖拽右下 resize 手柄（testdriver pointer Actions 60 步 drag）后 gBCR width/height 增大（resize:both 生效）⑤resize 拖拽不触发 listbox 选择 autoscroll（scrollTop 保持 0）。

本轮 solo 实测红点在**断言 2：`scrollHeight == clientHeight == 106`**——option 全体无内容盒，交互阶段（断言③④）从未到达。

## 三面定谳

| 面 | 断言 | ZW 现状 | 判决 |
|---|---|---|---|
| listbox 行渲染面 | ②（红点所在） | option/optgroup 被 R1679 UA `display:none`（style-system `ua_default_display`）+ s387 盒面抑制（tree.rs 汇聚点臂，**谓词不分 single/multiple 形态**）双通道摘出盒面——option 零内容盒，scrollHeight 恒等 clientHeight。上游 picker-none 子树包裹只针对 closed 下拉形态；`select[multiple]`/`size>1` 在真实浏览器恒以 inline listbox 渲染 option 行（可滚动内容） | **缺失**。施工=UA display 按 select 形态分支（multiple→option 可见行）+ 盒面/IFC 谓词同分支——中等特性，单独施工仅翻绿断言②，测试仍红在③④ |
| 元素拖拽 resize 交互面 | ③④（**测试主体**） | `resize` 属性已解析（registry+UA textarea 臂），`paint_resize_handle` 仅绘制角部指示器（painter/effects.rs）；engine input 层**无元素 resize session**（手柄 hit-test→拖拽→style 尺寸改写闭环不存在——`drag_source.rs` 是 HTML5 dnd draggable 面，scrollbar 仅绘制无拖拽交互，全仓 grep 无 resize 拖拽臂） | **缺失**。browser-chrome 交互特性（非语义兼容面），中等以上规模（hit-test 精度/pointer capture/min 尺寸/与 overflow 联动） |
| 选择 autoscroll 面 | ⑤ | ZW 无 listbox 选择拖拽 autoscroll | 平凡真（一旦①②存在即自动满足，不可独立验证） |

## 归属判决

**交互能力上限面，不施工，立项挂账**：断言②的 listbox 面与断言③④的拖拽 resize 交互面任一单独施工都不能翻绿（先②后③④仍红），测试主体是拖拽 resize 交互（Chromium bug 504375661 的 autoscroll 回归测试，依赖 resize 拖拽先工作）。两特性叠加超出流内轻量修复口径（Mission 约束 3），与 listbox-touch（s64/s386 通道上限不施工）同桶挂账——差异注记：本案上限在 **engine 特性缺席**（resize 拖拽交互 + listbox 行渲染），非 runner 注入通道（pointer Actions 通道在案可用，interactive-elements sweep 既有先例）。挂账位置：簇 A 立项六子面⑥交互面邻位（classic select 形态分支非 base-select 门，独立小项）。

## 口径

- the-select-element 域 corpus **149F+2T 零位移**（分类位移：簇 C 14F 内 multiple-resize 1F 待勘根→交互上限挂账，与 listbox-touch 1F 同桶；簇 C 余=立项 scope 10（input-parsing 2+selectedcontent-mutations 8）+特征邻位随簇 2（picker-popover-open-invalid）+通道/特性上限 2（listbox-touch+multiple-resize））。
- 本轮 solo 实测 `make testharness-html-semantics FILTER=select-multiple-resize` exit 1 断言 2 红（s385/s387 基线同形，1F 维持）。

## 下一步

- **customizable-combobox 域分诊**（`html/semantics/forms/customizable-combobox/`，s387 全树 sweep 观测 12F，**控制文档在册零记录的新目录**——s386 方法论复用：源码四标记扫描→门检测→三分归属）。
- A2 簇主体立项征询维持待用户决策。
