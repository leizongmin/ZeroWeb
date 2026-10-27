---
date: 2026-10-27
modules: layout-engine
---

# R2156 虚拟 inline 包装的盒回填必须放在管线终末，不能在 extract 期插入

## 问题

R2156 inline-box-model coherence 把「含嵌套原子 inline 后代」的 inline 包装（如 github
header 的 logo `<a><svg></a>`）整体跳过 taffy 树，原子后代提升为容器 taffy 直子。包装
因此完全无盒——gBCR/命中面返回 0×0。修复时第一版把合成盒回填挂在 `extract_layout`
的子盒构建尾声（LayoutBox 字面量前），本地单测过、全量 reftest 却翻红 2 例
（`atomic-inline-in-inline-zw-001` 6.00%、`-svg-zw-003` 1.67%）：span 包装内并排的
两个 input 变成纵向堆叠。

## 根因

两个独立的机制在 extract 期合成的包装盒上叠加作用：

1. **同步钩子失联**。R4234 原子行位同步、`sync_inline_child_boxes_from_ifc_with_ancestors`
   等按「原子盒是容器直子」遍历 `box_node.children`。合成把原子盒嵌进包装盒一层后，
   这些钩子再也看不到原子盒，其位置停留在 taffy 提升持位的块堆叠初值。
2. **5.6 shrink 误判 + IFC 零宽 spacer 重写**。`shrink_inline_blocks_to_content` 的
   R1480 has_border 检查只看 `border_*_width`（默认 `medium`→`Px(3)`，不看
   `border_*_style: None`），于是**每个** display:inline 元素都判 has_border 进 shrink；
   其 intrinsic 文本度量看不到原子子盒，测得 0，按 R4033 空内容臂把包装盒宽写 0。
   即使绕过 shrink，IFC 对虚拟包装只发零宽 spacer run，sync 尾部「空 inline 单
   fragment 全几何重写」同样把盒打回 0 宽。

定位手段：给合成盒打特征值（x=12345、w=77），在 layout() 各 pass 前后 walk 全树找
特征盒，二分出击杀 pass（extract 后 w=77 → `shrink_inline_blocks_to_content` 后 w=0）。

## 解决

合成挪到**管线终末**：`layout()` 在构造 `LayoutResult` 前对整棵树自底向上重嵌
（`engine/inline_wrapper_box.rs` 的 `synthesize_inline_wrapper_boxes_final`）。此前所有
同步 pass 都按基线形态（原子盒为容器直子）工作，与 60c4c9613 逐字节一致；唯一
delta 是最终树多出包装盒（无 bg/border 时零绘制面）。extract 期的两处几何守卫
（IFC sync 跳过、5.6 shrink 排除）随之不再需要——终末时点树里没有包装盒可被
改写——一并还原，避免留下永不触发的死守卫。reftest 4/4 恢复 0.00%，活体
github.com logo anchor 0×0→32×32。

## 要点

- 对「被跳过/虚拟化节点」回填盒或改几何时，先盘点哪些 pass 按**直接子关系**建立
  同步钩子；回填时点必须在全部钩子消费之后。
- 修不动中间形态就改时点：让管线全程保持基线形态、终末一次性变形，delta 面最小、
  守卫最少。
- layout 管线排查几何被改写时，特征值 + 全树 walk + pass 间二分比读代码猜快得多。
- `border_*_width` 默认 `medium`（Px(3)）与 `border_*_style: None` 并存；判断
  「有边框」必须同时看 style，否则全体元素都命中（R1480 检查的现状口径，另一账）。
