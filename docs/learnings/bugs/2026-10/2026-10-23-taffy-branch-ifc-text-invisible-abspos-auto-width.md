---
date: 2026-10-23
modules: layout-engine/engine, taffy-local
---

# taffy branch 节点对 IFC 文本不可见——原子提升后 abspos auto 宽塌最宽原子子

## 问题描述

`<span style="position:absolute">1<img 32×32>"Before gen abs</span>` 的 auto 宽解析为 32px（= img 宽，min-content 侧，窄列逐词换行 h≈104），CSS2 §10.3.8 shrink-to-fit 应为单行 max-content（160px）。悖论：同形态 img 无固有尺寸（0×0）时反而正确（128 单行）。

## 根因分析

R2156 slice19 将 abs 容器的原子 inline 后代提升为容器的 taffy 子（R4941 标 `Position::Absolute`）后，容器有元素子 → taffy **branch** 节点（`build_subtree` 按子空不空分流，branch 无 measure 函数）。taffy block 布局的 content_size = `inflow_content_size.max(absolute_content_size)`（taffy-local/compute/block.rs）只统计 taffy 子贡献，提升的原子计入——**IFC 文本不在其中**（只有 leaf 进 measure）→ auto 宽 = 最宽原子/块子。

img 0×0 时 content_size 塌 0，触发 engine 侧塌 0 补测臂（`block_max_content_width`）反而得到正确 max-content——「正确」是意外落网，非 taffy 语义正确。三条代码路径用三臂探针即可判别：**原子带尺寸 / 原子 0×0 / 纯文本 leaf**（leaf 有 R5104 measure 臂，文本可见）。

## 解决方案

不在 taffy 侧为 abspos branch 补 IFC 测量（需跨层搬 inline 测量依赖）；在 engine `apply_abspos_shrink_to_fit_width` 扩既有 `width_fix` 重测臂，窄门四条件：有 inline 内容（文本是缺口主项）+ 有元素子（排除 leaf）+ 无 aspect-ratio（transferred 宽 taffy 已解，且 R4987 域重叠）+ 无 definite max-width（`block_max_content_width` 测量不含 max-width 钳，扩臂会撑破钳制盒）。写侧注意 `block_max_content_width` 返回**border-box**（`inner + frame` 收口），直取不再叠 frame；单行放下 ceil（R5104 同理，防 paint Path B 整宽复排折行）；只扩不缩（`b.width < target` 才写）。kill-switch `ZW_*`（OnceLock 进程级单读）。

## 避免方式

任何「taffy 把 abspos/容器 auto 宽解析错了」的假设，先查该节点是 branch 还是 leaf、它的 content_size 能看见什么（taffy 子贡献 ≠ IFC 文本）；`has_inline_content && !children.is_empty()` 的盒宽 = 最宽原子子是结构性结果，不是 sizing 公式 bug。同形态疑案用三臂探针（带尺寸原子 / 0×0 原子 / 纯文本）一次性分流到塌 0 补测臂、measure 臂或 branch 盲区。另注意 0×0 塌 0 案的「正确」可能是补测臂意外救场，扩任何相关门时先确认不破坏该通路（本臂对塌 0 路径口径原样保留）。
