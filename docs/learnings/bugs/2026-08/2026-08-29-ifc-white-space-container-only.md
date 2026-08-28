date: 2026-08-29
modules: layout-engine,rendering-compat

# IFC white-space 标志只读容器样式——inline 子元素声明的 white-space 被忽略

## 问题描述

`<div class=block><span class=clamp style="white-space:pre">Line 1\n…Line 5</span></div>`
（line-clamp-014 谱系）：span 上的 `white-space:pre` 不生效，5 行文本被折叠成 1 行。
把 `white-space:pre` 直接放到 block 上**仍然折叠**——文本物理上位于 span 内，span 是
inline 元素，其声明的 white-space 应经继承作用于文本，但 IFC 断行标志没有消费它。

## 根因分析

`InlineFormattingContext` 的 `preserve_whitespace` / `break_at_newline` / `no_wrap`
是**容器级**标志：`compute_final_inline_layouts` / measure 回调 / paint Path B 均从
**容器**的 ComputedStyle 读取一次（如 `resolve_no_wrap_for_ifc_measure(styles.get(&dom_id))`），
`split_into_words(&self, …)` 亦读 `self.preserve_whitespace`。CSS Text 3 §4.1 中
white-space 是**继承**属性、按元素生效——同一行内不同文本段可携带不同有效 white-space
（span 段 pre、容器段 normal）。ZW 的全局标志模型无法表达混排段。

文本落在 inline 子元素内时，容器自身声明不继承到该子树（span 的 pre 只作用于 span
子树），而 IFC 从不查 span 的样式 → pre 丢失。文本为容器直接子节点时容器声明恰好
等于文本的有效值 → 一切正常（wk001 等直文本 pre 用例全绿的原因）。

## 解决方案（指向正确修法，本轮未实施）

TextRun 增加逐 run 的有效 white-space（从文本节点最近祖先的声明解析），collect_items
在构造 TextRun 时写入；break_lines 在分段（split_into_words）、空白折叠、`\n` 强制断行
处按 run 自身标志处理；混排行盒（同一段内 pre 段与 normal 段相邻）按 CSS Text 分段
语义处理。涉及 break_lines.rs（956 行）与 collect_items.rs 的分段主逻辑，须独立成轮
+ 全量 corpus A/B（pre 族用例密集，回归面大）。

## 如何避免

- 排查「多行文本变一行」类问题时，先确认**文本节点的有效 white-space**（继承链）
  而非容器声明——inline 包裹层是常见丢失点。
- IFC 级全局标志（preserve/no_wrap/break_at_newline）是容器近似的既有债务；任何
  「行数不对」问题若文本在 inline 元素内，优先怀疑此债务而非 clamp/高度回填等
  下游 pass。
