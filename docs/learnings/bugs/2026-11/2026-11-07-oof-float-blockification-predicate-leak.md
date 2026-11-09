---
date: 2026-11-07
modules: layout-engine
---

# §9.7 blockification 让 float span 算块级子：inline wrapper 谓词把离流盒计成块子（R5446 嵌套 float 域）

## 问题描述

`<p><span>Hello<span float/></span>Kitty</p>` 形整族渲染全破：float 从宿主 IFC 整体消失
（无 FloatAnchor → 锚 y 空 → 文本不绕排、容器高塌）、float 不落方向极值缘、前置文本不
绕排、后继文本被裁——floats-placement-vertical-001b 四子缺陷同源（✗3.64%）。直挂对照形
`<p>Hello<span float/></p>` 全对，债潜伏在「wrapper span 隔层」这一种 DOM 形态上。

## 根因分析

链式泄漏，一环扣一环：

1. **§9.7 blockification**（style-system）把 float span 的 computed display 算成
   `Block`（CSS2 §9.7 表：float 化非根元素 display→block）。
2. inline wrapper（span）因此在 `has_block_level_child` /
   `has_flow_block_child` 两个谓词里命中「有块级子」→ FLAT_CHILD_WALK 被跳过 →
   `build_flatten_run_for_element` → `has_flow_block_child` 为真 → BLOCK_CHILD_NO_FLATTEN
   把文本清零、float 条目整体不进 IFC。
3. IFC 无 FloatAnchor 条目 → float_anchor_ys 空 → R3784 锚位机制、R4300b flatten 臂、
   排除带、float_bottom 全部失联——下游每一环都「正确地」处理了一个不存在的输入。
4. 第二层坑：**嵌套 float 高度采样时机**。collector 在 remeasure 入口取
   `child.height`，此刻 taffy 对 float 叶子内容高欠计恒 0（R3787 已知问题在直接 float
   臂由 float_h_backfill 补测，嵌套臂照抄了字段采样却没抄补测）→ 排除带被 `w/h≤0`
   过滤丢弃 + float_bottom 恒 0 → 容器高塌 0（floats-149 div.block 绿条消失，首轮 corpus
   唯一真回归）。

本质：**离流盒（float/abs/fixed，CSS2 §9.3.1）不进块流**，「含块级子」判定语义上必须
排除离流盒；本仓在 abs/fixed 处早已排除（position-relative-003/004/005 实证），float 是
漏网的第三种离流盒。collect_items.rs 里本仓自己的 FIXME（「float 与 abspos 同为离流盒…如
现伪影再按 OOF 同款在谓词处排除」）早已预言此修。

## 解决方案

四缝同修（kill-switch `ZW_NESTED_INLINE_FLOAT=0` 一键回退，collect 侧与放置侧必须同一
开关——半开会分裂出有锚无放置/有放置无锚两种行为）：

1. **谓词排除**（collect_items.rs 两处 + inline_metric_storage.rs 两处 Path B 镜像）：
   `!(嵌套 float 开启 && float≠none) && !display-inline 族`——Path B（空 styles 重跑
   IFC）靠 storage 信号逐位镜像 layout 判定，漏镜像即分段分歧。
2. **FloatAnchor 补发两臂**：walk 臂（wrapper 直接展开时对 float 子发条目）+
   flatten 臂（bidi_special 等 flatten 路径扫描直系元素子发条目）。
3. **remeasure 嵌套目标回写**：collect（认领 wrapper 子树跳过双重处置）→ 锚 pass 回填
   `target=(锚行顶 y, rule 6 方向极值缘 x)` → **h≤0 时按 DOM 子树补测**（R3787 同款
   measure_text_content）→ sync 后按父链偏移反解写回盒位。
4. **compute_final 并入**：嵌套排除带并入 own_floats 链（fail-closed 门：带非空强制
   re-break 存储路径，否则 paint Path B 无锚重跑分段错乱）；float_bottom 链并入
   `target.y + height + margin_bottom`。

## 避免方式

- 新增「含 X 子」类谓词时，先问 X 是否离流；float/abs/fixed 三种离流盒必须同款排除，
  且 layout 侧谓词与 inline_metric_storage 侧信号**成对改**。
- 采样子盒几何前先确认 taffy 已终化该字段；float 叶子的 content height 在 remeasure
  入口恒 0，任何消费点都要走 R3787 补测通道。
- 单测断言嵌套盒几何时用父链累计 abs 口径（x 含 padding/border 逐层累计）——
  inline wrapper 的 x 由 IFC sync 收紧到内容缘（非 0），直接断言 `child.x==0` 会把
  正确的反解位误判为回归。
