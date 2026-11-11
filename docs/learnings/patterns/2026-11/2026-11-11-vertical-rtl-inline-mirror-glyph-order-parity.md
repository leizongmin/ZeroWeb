---
date: 2026-11-11
modules: layout-engine
---

# 竖排 rtl 行内轴镜像与 run 内字形序的 parity 结构

## 问题

竖排（vertical-rl/lr）＋`direction: rtl` 修复行内轴推进时，最容易顺手把「段落 levels 接进竖排 cursor」或「对 run 内字形序做反转」——R5502 归因轮修复设计初稿即包含「段落基方向进竖排 cursor」一点。这类改动会破坏本已正确的 run 内字形序，且**锚点看不见**：Ahem 定宽同形字形、corpus self-source（test/ref 双侧同镜像即 PASS，guidelines #33）、band 位锚（V4 只断言 [432,600]）三层验证全部对 run 内序盲。

## chromium 真值（fresh oracle 像素扫描，R5503 轮）

- vrl+rtl 与 vrl+ltr 的 run 内字形序**逐位相同**（band 整体底锚是唯一差异）：even run（拉丁）逻辑首字贴带顶（"Hx"：H 顶 281..346、x 底 351..398；LTR 对照 H 3..68、x 73..120）。
- odd run（阿文）逻辑首字贴带底（分色 span：س y 332..392 底、لا 289..331 顶）＝沿行内流向上读。

## 根因结构

ZW 竖排 emission 消费 `visual_text()`（**水平**视觉序）做顶向烘焙——这是水平行为的轴转置副本：水平左→右视觉位映射到竖排顶→底。水平世界里 even run 左起＝逻辑首、odd run 左起＝视觉首（＝逻辑末），转置到竖排后恰是 chromium 竖排 rtl 的 run 内真值。**镜像臂只需翻转 band 位与 item 序（`y' = c − y − h`，vec 保持 DOM 序），run 内序天然正确**；行级 L2 重排 pass（`reorder_lines_l2_for_para_bidi`）必须显式 `&& !vertical` 门禁——其 x 镜像在竖排是块轴错轴操作、字符级 L2 对已烘焙视觉序文本是二次重排。

## 验证方法：分色 span 探针

run 内序的判别需要**逐逻辑字符可定位**：把每个字符包进不同颜色的 `<span>`（如 `س`红＋`لا`蓝），headless 截图后按色相像素扫描 y 段（同色段的 y 跨度即该字符的行内位）。glyph 形状/宽度差探针（"Hx" 的 x 跨度差）只适用 even run；分色 span 对任意 script/parity 通用。同类陷阱：凡「修复整级行为、锚断言 band 或盒级」的轮次，run 内序都要单独探针定谳，不能由 band 锚外推。

## 案例锚

- 修复：R5503 `break_items_into_columns` 行内轴镜像（crates/layout-engine/src/inline/mod.rs）。
- 锚：crates/layout-engine/tests/r5503_vertical_rtl_inline_order.rs（V1-V7）＋独立进程 kill-switch 锚。
- 证据：docs/goal/rendering-compat/evidence/r5503-vertical-rtl-inline-order-fix-2026-11-11.md（Q1-Q3 探针表）。
