# R4524 证据：oracle 探针定谳 + count==1 sequential 嵌套子剪裁——rule-nested-balancing-004 翻绿 0.00

- 日期：2026-09-19（R4524 C 轮）
- 方法：**chromium oracle 探针**（headless chromium + puppeteer-core 对 004 test/ref 双页真实
  截图，headless 对本例可用）+ ZW REFTEST_DUMP 双页逐带对照。

## oracle 定谳（推翻 R4523 的「块轴下延」解读）

chromium 对 004 **test 与 ref 页均只渲染 [8,308]**（STEEL 复合色带）——outer 的
fragmentainer 端外内容**全部不渲染**（inner 溢出段 [308,508]、inner-block [508,608] 全无）。
R4523 的「ZW-ref [308,508] 照绘 = chromium 语义」误读：该带是 **ZW 自己对 ref 页的渲染**
（ZW-ref 与 ZW-test 同错），非 chromium 目标。修正后的律：

> **count==1 sequential（fill:auto）+ definite height：嵌套 multicol 子落在
> fragmentainer 端外 → 整子不渲染（cap 剪裁）**；平铺子照绘下延（column-height-011
> 实证：单列普通子 200>100 溢出可见——chromium 宽容平铺子、剪裁嵌套 multicol 子）。

## 实现（layout 侧，painter 零改动）

1. sequential 分支：count==1 + 嵌套 multicol 子（column_count/column_width 非 auto）→
   `assign_children_to_columns_multirow` + 逐子 cap。count≥2 维持 R1035 排除（007-ref
   自源配对实证：count:2 sequential cap 改变 ref 页碎片化破坏自配对 → 007 翻红 2.51%）。
2. multirow cap 补**第二分支**（子低于列高但当前列已满）：cap 时发 **零高片段**
   （visual 0）而非静默丢弃——零高 cso 使 painter 零高 clip 全裁该子树图元（否则子经
   steps 3/4/5 以自然位 unclipped 照绘，004-ref inner#2 残带）。
3. **painter 单片段截断剪裁臂回退**：A/B 实证有害（multicol-oof-inline-cb-002
   0.79→1.04% 翻红；对 004 无必要——零高片段 + cap 已足）。假说-证伪-回退全程记录。

## 探错轨迹（3 轮中间态）

| 中间态 | 004 | 007 | oof-002 | 011 | 结论 |
|---|---|---|---|---|---|
| R4523（exclusion） | 3.83% | ✓ | ✓ | ✓ | [308,508]+[508,600] 残带 |
| +count==1 全量 cap | 8.33%→0.00 | ✓→✗ 2.51% | ✓→✗ | ✓→✗ | ref 页（sequential）未盖 |
| +sequential 全量 cap | 0.00 | ✗ 2.51% | ✓ | ✗ | 007-ref 自配对破坏 |
| **+嵌套子逐子 cap（终态）** | **0.00** | ✓ | ✓ | ✓ | 全对齐 |

## A/B（终态）

- css-multicol：137→**136**（fail-list diff 唯一 004 移除）；**315→316 pass**。
- 全 corpus：15109/16594（**91.08%**）；唯一 diff = 004 移除 + box-shadow-003（既有
  ±1 flake，本轮 ✗ 相位）。
- kill-switch `ZW_NESTED_MC_CLIP=0`（count==1 sequential 臂）+ R4522 臂共门。
