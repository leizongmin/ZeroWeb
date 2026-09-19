# R4522 证据：definite-height balance × 嵌套 multicol 子溢出行剪裁（007 翻绿 2.51→0.00）

- 日期：2026-09-19（R4522 C 轮）
- 用例：`multicol-span-all-children-height-007`（outer count:2 + h:110 + rule 6px black > inner **count:2** + h:270 + border 10 purple + [block1 200][spanner 50][block2 240]）
- 前置：R4519/R4520 区域模型（inner 自身 spanner 区域分段已对齐）+ R4521 外层 rule cso 感知（黑 rule 已逐像素恢复）。

## 溢出语义分叉律（本轮定谳，RFC 修订依据）

definite-height balance 容器内容超 col_count × 列高时：
- **平铺子**（plain block）→ 内联溢出列（向右，R1075 chromium 实证；002 谱系「two extra
  overflow columns are created. Total 4 columns」assert，ZW 已对）；
- **嵌套 multicol 子**（自身为碎片化上下文，column_count/column_width 非 auto）→ **剪裁**
  （007 ref：仅 2 列片段，无溢出列；inner 第 3 行片 [220,290) 不渲染——含其底边框与
  block2 溢出行）。

007 残余 2.51% 的 100% = 该溢出片段（[432,596]×[8,78]：YEL block2 溢出行 + PURP 底边框，
约 11.5k px）。

## 实现

`assign_children_to_columns_multirow` 增 `cap_nested: &[bool]` 逐子 cap：cap 子占满最后
一列（col_count−1）仍有剩余行时 break——不创建溢出列。balance `overflow_inline` 臂按子
样式判定 cap（column_count/column_width 非 auto = 嵌套 multicol）；平铺子 cap=false 零
变化；sequential（fill:auto）分支与 spanner 区域分支传 `&[]` 零触（breaking-004 谱系的
sequential multirow 不受影响——css-multicol A/B 证实）。

## A/B

- css-multicol：314→**315**，fail-list diff **唯一 007 移除**；multicol-nested-*
  （005 等）/ breaking-* / rule-nested-balancing-* 全持平。
- 全 corpus：15108/16594（91.06%），唯一 diff = 007 移除 + box-shadow-overlapping-003
  （既有 ±1 flake，本轮 ✓）。
- kill-switch `ZW_NESTED_MC_CLIP=0`。

## 测试

+2 单测：`multirow_cap_nested_clips_overflow_rows`（290/110/2 列：无 cap 3 片 vs cap 2 片
+ 混排子平铺溢出保留）、`multirow_plain_child_keeps_overflow_columns`（R1075 守卫）；
既有 multirow 测试调用点同步补 `&[]`。

## RFC nested-multicol-fragmentation-rfc.md 关系

007 分叉律 + 本 slice 入档（RFC v0.2 的 breaking-001..006 主线 = font-wall-entangled，
维持 R1512「勿单试结构修复」结论——本 slice 不涉文本宽度，纯几何剪裁，故可达 net-positive）。
