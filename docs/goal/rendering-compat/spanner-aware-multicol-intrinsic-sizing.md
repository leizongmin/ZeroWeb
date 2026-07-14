# L3② R1431 — spanner-aware multicol intrinsic sizing

**日期**: 2026-07-14
**承接**: R1430（R1020 proxy fix net-0 revert → 须 spanner-aware 整体解，非单点）
**目标案**: `multicol-width-005.html`（6 case，7.28%）+ `multicol-width-004.html`（4.09%）
**规范**: CSS Multicol §3.4（column-count/width 求解）+ §6.1（column-span:all spanner 跨全宽）+ CSS Sizing-3 max-content

## 缺口

`width:max-content` / shrink-to-fit 的 multicol 容器 intrinsic 宽度，ZW 当前用 R1020 proxy
（`column-count:Number` + 全 leaf 子 → N × children_inner + (N-1)×gap），两处错：

1. **column-width 不参与 intrinsic**：col-width-only（count auto）案走 `inner+frame` 回退，
   用 max 子宽（含 block 显式宽）非 column-width。multicol-width-005 case 1（col-w:80, block 100）
   ZW=104（用 block 100+frame）应 80。
2. **spanner 被 N× 误放大**：R1020 proxy 的 `children_inner` 含 spanner 子，N× 把 spanner 宽乘 N。
   case 6（col-count:2, spanner 250）ZW=514（2×250+frame）应 250（spanner 跨全宽不乘 N）。

LAYOUT_DUMP 实测 ZW 6 case 宽度 vs 期望：104/104/154/214/214/514 vs 80/120/150/210/230/250。

## 算法（6 case 全验证）

```
N            = column-count:Number(n) ? n : 1            // col-width-only → 1 列（shrink-to-fit）
col_content  = column-width:Length(w) ? w                 // column-width 设定 → 子溢出，列宽=column-width
               : max(non-spanner in-flow 子 intrinsic)    // 否则取最宽非 spanner 子
column_driven = N × col_content + (N-1) × gap
spanner_driven = max(column-span:all 子 intrinsic)        // spanner 跨全宽，驱动宽度
result_inner  = max(column_driven, spanner_driven)
result        = result_inner + frame(border+padding)
```

验证（gap=10, frame=2×1px border）：
| case | column-props | 子（block/spanner px） | N | col_content | column_driven | spanner_driven | max | 期望 |
|------|-------------|------------------------|---|-------------|---------------|----------------|-----|------|
| 1 | w:80 | b100/s50 | 1 | 80 | 80 | 50 | 80 | 80 ✓ |
| 2 | w:120 | b100/s50 | 1 | 120 | 120 | 50 | 120 | 120 ✓ |
| 3 | w:120 | b100/s150 | 1 | 120 | 120 | 150 | 150 | 150 ✓ |
| 4 | c:2 | b100/s-auto | 2 | 100 | 210 | ~narrow | 210 | 210 ✓ |
| 5 | c:2,w:110 | b100/s-auto | 2 | 110 | 230 | ~narrow | 230 | 230 ✓ |
| 6 | c:2 | b100/s250 | 2 | 100 | 210 | 250 | 250 | 250 ✓ |

## 实施

`crates/layout-engine/src/intrinsic_sizing.rs::block_max_content_width`：
1. 子循环额外追踪 `nonspanner_block_max`（非 spanner block 子 intrinsic max）+ `spanner_max`
   （column-span:all 子 intrinsic max）。`block_max`（含 spanner）保留供非 multicol `children_inner`。
2. R1020 proxy 分支（column-count:Number + leaf guard + N×children_inner）**替换**为新 spanner-aware
   分支：检测 multicol（column-count:Number OR column-width:Length），按算法算 `result_inner`，
   早返回 `result_inner + frame`。非 multicol 走原 `inner + frame`。

## 验收

- 单测：6 case 的 (N, col_content, column_driven, spanner_driven, result) 逐 case 断言（helper 构造
  LayoutBox + styles 模拟 column-props + 子宽）。
- A/B：`make reftest-oracle DIR=css-multicol`（ORACLE_DUMP_ALL per-case），目标 multicol-width-005/004
  flip（< 1%），net ≥ 0（无回归）。intrinsic sizing 共享 → 额外 spot-check css-flexbox/css-grid
  （multicol 嵌 flex/grid）无回归。
- product-smoke welcome 不变（无 multicol）。
- fmt + clippy `-D warnings` + make test 全绿。

## 风险 / 边界

- intrinsic sizing 共享（max-content/fit-content/auto-float/shrink-to-fit）。新算法对**非 leaf**
  column-count multicol 也乘 N（当前 R1020 leaf guard 排除它们 → 走 inner+frame 不乘）。行为变化
  须 A/B 守 net ≥ 0；回归则加 gate（如仅 leaf 或仅显式宽子）。
- col-width-only 的 N=1 假设 shrink-to-fit 下 1 列；非 shrink-to-fit（definite 宽容器）不走本函数
  （width 已 definite）。须 A/B 确认 col-width-only 案无回归。
- spanner 子 intrinsic 用 `box_content_max_width`（含 spanner 自身显式宽，如 case 3/6 spanner width）。
