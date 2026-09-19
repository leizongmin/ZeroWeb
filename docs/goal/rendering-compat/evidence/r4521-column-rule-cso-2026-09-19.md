# R4521 证据：column-rule 跨列 breaking 子内容判定（cso 感知）+ css-multicol top-fail 勘察

- 日期：2026-09-19（R4521 P+C 混合轮——1 painter 修复 + 勘察，无翻案）
- 用例：`multicol-span-all-children-height-007`（2.65%→**2.51%**；nested multicol 谱系）

## 修复面（paint_column_rules，text_multicol.rs）

007 结构 = outer（count:2 + h:110 + rule 6px black）> inner（**嵌套 multicol** count:2 +
border 10 + spanner）。wrapper_is_multicol gate 使 synth 不触（R1341 设计），inner 经
generic multirow 路径产生 cso 3 片段（col0/col1/溢出 col2）。`paint_column_rules` 的
列内容判定只用子盒主 x（= 0，col0）→ col1 侧误判空列跳线 + 满宽/跨列子不识别。

修：列内容判定 **cso 感知**——子盒 cso 片段的 col_x 落在列位即视为该列有内容（+
nested-spanner wrapper 视为每列有内容）。007 的外层黑 rule 逐像素恢复（[205,211]×[8,118]
= ref 同位）；007 2.65→2.51。

## A/B

- css-multicol fail-list diff：**空**（138 持平，007 差值缩小不翻案）。
- 全 corpus：唯一 diff = box-shadow-overlapping-003（既有 ±1 flake，本轮跑 ✓）。
- 007 残余 2.51% 全部 = 溢出片段 [432,596]×[8,78]（inner 底边框 + block2 溢出行被
  outer 的 multirow 内联溢出列渲染；ref 剪裁）——**嵌套 multicol 碎片化模型域挂账**
  （外层 definite-height balance 对嵌套 multicol 子的溢出语义 vs R1075 平铺子模型分叉，
  RFC 级）。

## 勘察（css-multicol 138 fail top 分布 → 域归因）

| 案 | diff | 域 |
|---|---|---|
| span-all-rule-002 | 27.67% | **vertical writing-mode** spanner（4 种 writing-mode，深域挂账） |
| rule-nested-balancing-004/002 | 16.67/4.15% | 嵌套 multicol balance（ref 单列 vs ZW 双列，非 spanner 机制） |
| multicol-inherit-002/003 | 14.92/3.80% | multicol 属性继承链 |
| span-float-004 | 14.39% | **script 驱动**（cssFloat 动态改）+ float→spanner 转换 |
| column-height-009 | 13.97% | column-height 域 |
| nested-005 / nested-margin-002/003 | 13.13/5.53/3.72% | 嵌套 multicol margin 协调 |
| block-sibling-003 | 11.28% | auto 高 no_box wrapper + **inline 流平衡**（7 行未均衡分列，ZW 堆叠 112 vs ref 64） |
| on-broken-image-alt-text | 10.46% | 图片 alt 文本 multicol |
| margin-child-001 / margin-002 | 9.62/4.19% | multicol margin 域 |
| width-small-001 / width-ch-001 | 9.08/4.35% | 窄容器列宽计算 |
| children-height-009/010 | 1.05/2.09% | **columns:10 + 负 margin spanner + orphans/widows**（exotic） |

结论：top 段全为独立深域（vertical/script/nested-balance/margin/exotic），无单主导机制；
spanner 区域模型族（R4519/R4520 已清 004a/b/005/006/008）在本目录内已收敛到深域边界。
