# R4538 证据：根因链收敛到顶点——abspos ::before 背景按零尺寸矩形发射（P 轮，实验未收敛回退）

- 日期：2026-09-20（R4538 P 轮——R4537 下轮方向①续，0 net code 落地）
- 前置：R4536 三缺陷 + R4537 draw_order 重放机制发现。

## 决定性数据（逐 op 覆盖 dump，单 clip 探针）

单 clip 探针（clip-probe：父无 clip + ::before 绿 bg + 内层 clip-path，新代码）的
clip 消费现场 dump：
- `op[0] Fill(0) rect=(0,0,100,100)` = 父红背景，in_range=false（子快照之前）；
- `op[1] Fill(1) rect=(0,-2.4640007,-0,18.624)` = **绿色退化矩形**（宽 −0！in_range=false）；
- `op[2] Fill(2) rect=(0,0,0,0)` = **零尺寸矩形，in_range=true, covered=false** → 改写
  目标为空、条带为 0 → render_draw_order（仅 14-16 ops）无绿色可绘 → 全红渲染。

**结论：::before 的背景 fill 从发射起就是退化/零矩形**——abspos inset:0 伪元素的
layout 盒（或其 bg 发射路径）给出 x=0, y=−2.464, w=−0, h=18.624 的退化几何
（数值特征 = IFC 行内零宽盒：w=−0 即空 content 行内盒宽、h=18.624 ≈ 行高、
y=−2.464 ≈ 基线修正——疑似 ::before 在父 IFC 内以**行内零宽盒**身份发射背景，
而 abspos 盒的背景 fill 缺失/为零）。

## 根因链（更新后全序）

1. **顶因（本轮定位）**：abspos ::before 的背景 fill 以退化矩形发射（行内零宽盒
   疑似）→ 后续一切裁剪无对象可裁。
2. draw_order 重放机制（R4537）：未登记 DrawOp 的图元不绘制。
3. clip-path Polygon 臂 path_fill 改写 z 序倒挂（R4536 缺陷 A）。
4. clip_fill_to_polygon 条带顶采样漏带（R4536 缺陷 B，中点修正已验证）。
5. 嵌套消费快照范围（R4536 缺陷 C）。

## 下轮方向（顶因优先）

1. **::before abspos 盒布局/背景发射排查**：为何 inset:0 abspos 伪元素的 bg fill
   为退化矩形（行内零宽盒 vs abspos 盒双发射路径；layout 侧伪元素盒合成）——
   before-probe（无 clip）绿块可见说明存在一条正确路径，对照两页 layout 树
   （render_with_layout_inner 已暴露 root）即得。
2. 顶因修复后，Fix B（中点采样 ✓ 已验证）+ Fix A（条带化 + draw_order 登记）+
   Fix C 按序收口，corner-shape 凹角簇 ~10 案解锁。
3. 或守成轮。

## 门禁

零代码落地（实验回退），无门禁义务；树态 = R4535 干净基线。
