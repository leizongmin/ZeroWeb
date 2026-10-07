---
date: 2026-10-07
modules: zero-layout-engine
---

# taffy 的 aspect-ratio 传递不与内容取大，且项目内既有传递臂全部 HorizontalTb 门控

## 问题描述

WPT `css/css-sizing/aspect-ratio/block-aspect-ratio-017.html`（vertical-lr + `width:100px` + `aspect-ratio:2/1`，子 `height:100px`）渲染为 100×50，chromium 为 100×100。全量 fail-list 1414 案中的 1 例，长期在册未定位。

## 根因分析

两层叠加：

1. **taffy 0.12 的行为**：对带 `aspect_ratio` 的节点，一轴 definite、另一轴 auto 时按纯比值传递（100/2=50），**不做** css-sizing-4 §4.1 的 content-based minimum 取大——子盒 inline extent（100px）不参与 max。
2. **项目侧臂覆盖缺口**：`crates/layout-engine/src/aspect_ratio_transfer.rs` 的传递后处理共有三臂（float+%height 反推宽、水平双 auto、flex/grid 容器），全部 `matches!(writing_mode, HorizontalTb)` 门控——vertical 写入模式下块轴=物理宽、内联轴=物理高，三臂都接不住。R1743 父高回填本可兜底，但 `shift_active` 对 vertical 容器显式关闭（R4427，防 y 模型误位移），vertical 盒成为唯一无人管的死角。

## 解决方案

在 `transfer_aspect_ratio_height::walk` 新增 vertical 臂（R4986）：`writing_mode.is_vertical_block_flow()` + definite 物理宽 + Auto 物理高 + AR + overflow 双轴 visible → `height = max(传递值, in-flow 子底边, content_height) + frame_v`，仅增长不收缩。滚动容器豁免与 R3765 同语义（无 content-based minimum）。

## 如何避免

- 给既有后处理 pass 加新臂时，若按书写模式门控，先问「vertical 模式下同一 CSS 语义由谁接管」——本项目 vertical 路径与水平路径是两套代码（`vertical_block_flow.rs` vs taffy+后处理），水平臂修的 bug 在 vertical 侧几乎总有对应缺口。
- 定位 reftest 近似通过（1-2% diff）类案例，`REFTEST_DUMP=1` dump PNG + `layout-dump <case>` 看父子盒几何，比读像素快。
- 性能归因可 kill-switch/`git stash` 做 A/B 微基准：本次 stash 对照 2.61ms→2.55ms，证明 bench-gate 红是环境负载（未编译修复的基线树当时已超预算 +20%），非新增臂成本。
