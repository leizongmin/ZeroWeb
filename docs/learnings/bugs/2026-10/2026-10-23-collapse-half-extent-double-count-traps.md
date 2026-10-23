---
date: 2026-10-23
modules: layout-engine
---

# 折叠表边框中心距补全的代数等价与双重计入陷阱——半分摊换口径时全部消费点要同步换

## 问题描述

border-conflict-element-001a（4×4 collapse 表、td border 20px 空 cell）：表应 100×100，ZeroWeb 渲染 80×160。行高按 cell 整 border-box 计入（40/行），表高 = 4×40 + 10 + 10 = 180 再扣出 160；而边框中心距模型（CSS2 §17.5.2.1/§17.6.2.1）下 collapse 行高 = 相邻边框中心间距 = cell border-box − ½(bt+bb) = 20，表 border-box = Σ中心距 + 每缘 ½×胜出边框 = 4×20 + 10 + 10 = 100。

## 根因

列宽侧 R4027/R4028 早已做半 border 分摊（cell 显式宽 + ½ 相邻 border），行高侧却仍按整 border-box 累加；且 cell 边框胜出表自身边框的表外缘，胜出边框的外半伸出中心距之外未计入表 border-box。两处口径不一致的深层风险：中心距模型的各量是**代数等价形态**（Σ(整扣, 整 border 行高) ≡ Σ(半扣, 半 border 行高)），只改生产端不改消费端就会双重计入或漏计。

## 解决方案

三件套同旗标 `ZW_TABLE_COLLAPSE_HALF_EXTENT`（default-on）：

1. 行高贡献 = cell border-box − ½(bt+bb)（镜像列宽半分摊）；cell 盒不回写、保持整 border-box 高（相邻行 cell 带 ±½ 重叠，恰使 paint 端内绘边框带落边框中心）——cell 高度须加 `.max(cell_box.height)` 防缩 guard；
2. cell 边框胜出的表外缘按**每缘独立**取 max 计入表 border-box（四缘是四条平行带：同缘各行边框带平行铺开不叠加；但 1×1 表同一 cell 同胜左右两缘，共享一个 max 累加器只加一次、少算一缘——单测抓到，拆 outer_l/outer_r/outer_t/outer_b 各自 max 再各取半相加）；
3. 表 size 扣除（table border 覆盖 cell 边框的 R291/R292 既有 edge 扣除）从整扣改半扣——与半分摊行高配套；显式宽 cell 的 R364b floor 同步去 border 半宽（floor 是 border-box 口径，不去则整 border 形态下列宽与外缘加法双重计入）。

**双重计入陷阱清单**（半分摊换口径时逐一核对消费点）：显式列宽 base、intrinsic floor、行高累加、表 size 扣除、表外缘加法。任何一处留在旧口径都会以 +½ 或 −½ border 的形态在特定构造上显形。

## 避免方式

- 几何模型换口径（整 border ↔ 半 border）时，先列出该几何量的全部生产端和消费端，一次换齐；用「新旧形态代数等价」验证总账，再逐案核对特例。
- 每缘独立累加是折叠边框的通用语义：带平行不叠加（同缘取 max），缘与缘相加（跨缘求和）。共享累加器会在对称构造（1×1 表）上少算。
- kill-switch OFF 反证走 corpus OFF 轮（进程内 set_var 走不到 selected 快照路径，R3858 教训）；cell 不回写的 invariants 要配防缩 guard 单测。
