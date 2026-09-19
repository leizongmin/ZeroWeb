# R4520 证据：R1473 step-2 slice ②——count==1 单列 spanner 区域分段（005/008 双翻绿 0.00）

- 日期：2026-09-19（R4520 C 轮——1 新函数 + gate 扩展）
- 用例：`multicol-span-all-children-height-005`（count:1 + fill:auto + container h:250 **无框** pink + 3 blocks + 2 spanners）、`-008`（count:1 + fill:auto + container **border 20 purple** + auto 高 + pink + 3 blocks + 2 spanners）
- 方法：REFTEST_DUMP 双页 PNG 逐带扫描（x=10/30/110/190 纵向 runs）。

## 残差定位（P 轮并入本轮）

- **005**（1.04%）：ZW 容器 = 单个 250px 连续 pink 盒；chromium = 「just enough」区域模型——pink 三段 [8,108]/[158,258]/[308,358]（预算 250 → 区域 100/100/50，spanner 50×2 插入），block3 黄块 [308,408] 溢出区域 3（50）下方，article 高 = 350（ref 358 以下白、无绿）。唯一 diff = 区域 3 pink 带 50×80 缺失。
- **008**（1.06%）：children/bg/高度全对齐；唯一 diff = 左右边框连续贯穿 spanner 带（ZW [8,448] 连续紫），chromium 按区域分段 [8,128]/[178,278]/[328,448]（spanner 相邻边 skip：r0 top+左右、r1 左右、r2 左右+bottom [428,448]）。

## 实现（`apply_count1_region_segments`，count==1 早臂）

- 单列无须碎片化/重定位（children 自然堆叠 ≡ 区域序），不经 synth：只发射装饰段
  （背景恒发、边框按区域归属）+ 「just enough」高度回写（cell = 区域总量，无除列）；
- 段盒宽 = wrapper 全宽（单列无列宽约束/double-subtraction）；
- spanner 归位：x = −(bl+pl)、宽 = article content 宽（column-span:all 跨容器列组；
  008 的 spanner 从 [28,188] 扩到 [8,208]）；
- 005：显式高 250 = 预算 → wrapper 盒 250→350、article 同步 350；008：auto 高 →
  rendered=总量、盒高 440 不变。

## 中间回归与 gate（column-balancing-paged-001-print 18.77% 一现即修）

首版无 fill gate：`column-balancing-paged-001-print`（outer count:1 **balance** > inner
嵌套 multicol + 分页域）被误触翻红 18.77%。加双 gate 收口：①`info.sequential_fill`
（自然堆叠假设只在 fill:auto 成立）②wrapper 非嵌套 multicol（其 spanner 属内层列组）。
复测三案全绿（005/008 0.00%、paged-001 0.00%）。

## A/B

- css-multicol：312→**314**（fail-list diff 唯一 005/008 移除，138 fail）。
- 全 corpus：fail-list diff = 005/008 移除 + box-shadow-overlapping-003（既有 ±1 flake，
  3 连跑 2✓/1✗ 实证非本轮回归）。15105→**15107**/16594（91.05%）。
- kill-switch `ZW_COUNT1_REGION_SEGS=0`。+1 单测（008 全几何断言）。

## 余账

直连 spanner 预算化剩 rule-002（27.67%，vertical writing-mode 域——深域挂账）、
block-sibling-003（11.28%，auto 高 no_box wrapper + inline 流平衡域）、
margin-nested-firstchild-001（7.65%）。
