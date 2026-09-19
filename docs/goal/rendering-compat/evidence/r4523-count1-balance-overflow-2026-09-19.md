# R4523 证据：count==1 balance 溢出语义（不下探内联溢出列）——004 16.67→3.83

- 日期：2026-09-19（R4523 P+C 混合轮——1 gate + 1 painter 修复回退，无翻案）
- 用例：`multicol-rule-nested-balancing-004`（outer count:**1** + h:300 + rule black + 蓝罩 >
  inner count:1 + h:500 + 品红罩 > inner-block h:600 绿罩；α 叠加复合色逐带解码）

## 溢出语义第三律（count==1 分叉，RFC 补充）

- **多列（count≥2）平铺子**溢出 → 内联溢出列（R1075，002）；
- **多列嵌套 multicol 子**溢出 → 剪裁（R4522，007）；
- **单列（count==1）**容器溢出 → **不碎片化**：溢出内容沿块轴下延为可见溢出
  （004 ref：inner 500 在 outer 300 下方 [308,508] 照绘复合色；inner-block 绿罩在
  inner 内被 inner 的块端 [508] 截断）。
- ZW 旧路径：count==1 也走 R1075 multirow → 溢出「列」落在右侧 x=216（count:1 的
  stride）= 与 ref 完全不同轴。

## 实现

1. balance `overflow_inline` 臂加 `info.count > 1`——count==1 回退 balanced 分配
   （单子不拆 → 全高渲染 + 可见下溢出）。
2. 中间误判回退记录：首版「cap 单片段截断 → painter 裁到片段高」假说（ref 308 处剪裁）
   被像素证据证伪（ref [308,508] 照绘）→ painter 剪裁臂回退；multirow cap 结构修正
   （cap 子首片占满即 continue，不在溢出列 push 片段——004 的 [300,500) 片段误入
   x=216 溢出列 bug）。

## 量化

- 004：16.67 → **3.83%**（spurious 右列消除 + [308,508] 带对齐）。
- 残余 3.83% = 唯一带 [508,600]：inner（嵌套 multicol，definite 500）对自己的子
  （inner-block 600）做块端剪裁，而 top-level outer（definite 300）对 inner 不剪裁——
  嵌套深度相关的 child-clipping 语义未定谳（嵌套 multicol 碎片化 RFC 域，下轮项）。
- A/B：css-multicol fail-list diff **空**（137 持平）；corpus 唯一 diff = box-shadow-003
  既有 flake（本轮 ✓）。

## 状态

keep（strictly closer to chromium，无翻案风险面）；004 翻案留待嵌套 child-clipping
语义定谳（RFC 域下轮项）。
