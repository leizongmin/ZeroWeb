---
date: 2026-10-15
modules: style-system,engine/paint
---
# outline-width 初始值是 medium(3px) 非 0，不能当「作者已声明」信号

## 问题

实现 UA 默认键盘焦点环（Tab 导航聚焦且作者未声明 outline 时画 2px 环）时，
适用判定写成「outline_style == None && resolve_outline_length(outline_width) <= 0」，
真机验证（ui_driver 驱动真实窗口 + renderer stderr 探针）所有片段
`applicable=false`，焦点环完全不出现。

## 根因

`style-system/src/property/default_impl.rs` 按 CSS UI 规范把 outline-width 初始值
实现为 `Px(3.0)`（medium，与 border-width 初始 medium 一致），而非 0。默认元素是
「style=None + width=3px」，宽度非零条件把全部未声明 outline 的元素误判为
「作者已声明 width」。「computed 值等于初始值」与「作者显式声明」在 computed 层
本就不可区分——但用非零初始值做判据会连默认元素一起误杀。

## 解决

适用判定只看 `outline_style == None`（作者声明任何非 none style 即退出，
由 paint_outline 正常绘制；style=None 时 paint_outline 本就不绘制，width 声明
无可观察效果）。教训：**用 computed 属性探测「作者是否声明过」时，先查该属性的
初始值实现——非零初始值（medium/normal/auto 类）不能当声明信号**。同类隐患：
border-width（medium）、font-weight（normal=400）等。

## 验证

ui_driver Tab 遍历 + 截图：链接整体单环（行内片段带合并）、input 盒路径环；
`.acceptance/site-optimizer/browser-gui-20261012-r1/ui-batch-t27t30-20261015/shots/`。
