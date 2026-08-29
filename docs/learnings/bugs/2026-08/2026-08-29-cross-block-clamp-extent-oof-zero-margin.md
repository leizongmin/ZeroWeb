---
date: 2026-08-29
modules: zero-layout-engine
---

# 跨块 line-clamp 容器收缩：oof 子污染 extent + 边界零高自塌盒 margin 丢失

## 问题描述

`line-clamp: auto` + `max-height` 页面（line-clamp-auto-032）：clamp 正确裁掉了边界后的内容，但容器高度收缩到 233px 而非约束 138px——边界处的 100px abspos（应可见）把容器"撑高"了 95px。

## 根因分析

收缩公式 = max(可见子盒 y+height)，两处偏差：

1. **extent fold 含 abspos/fixed 子**。CSS §10.6.3：oof 脱流不贡献容器 auto 高。但 clamp 场景特殊——css-overflow-4 规定「containing block 在 clamp point 前/跨 → abspos shown」，即边界上的 abspos 恰恰是**保留**的（R3770b 豁免不隐藏）。保留的盒又参与 extent → 自相矛盾：豁免它绘制，又让它撑高容器。
2. **零高自塌盒 margin 丢失**。auto-032 的 `.collapse-through`（h=0, margin 5px）被 R3775「边界后零高盒隐藏」规则裁掉后，其 bottom margin（css-overflow-4 assert 明确「bottom margins end at the clamp boundary」，属于边界）从 extent 消失 → 容器少 5px。

## 排查弯路（两版否弃方案）

- **min(pre_clamp_height, constraint) 全局下限**：auto-032 对了，但 auto-047 回退（其边界 76.2 < 约束 83.8，下限强行抬到约束 → +1.10% fail）。教训：边界 ≠ 约束，边界是内容相关的。
- **walk 边界推进时顺带加零高盒 margin-box**：零高盒在 032 走的是 R3775 隐藏分支，根本到不了 leaf 分支的推进代码——改了不改行为的路径。

## 最终方案

`zero_margin_extent` 独立通道：R3775 零高隐藏分支记录隐藏盒的 margin-box 底（y+h+mb），嵌套递归坐标上抛；消费侧 `max(extent, min(zero_margin_extent, 约束 px))`。**约束 cap 是关键**——真在边界后的零高盒（031 型，margin-box 143 > 约束 128）被截掉，边界上的（032 型，138 = 约束）保留。两案兼顾。

## 如何避免

- 「容器比预期高」+ 页面含 clamp/隐藏逻辑：先 LAYOUT_DUMP 对比 test/ref 的逐盒高度，确认多出的高度来自哪个盒的 y+height（本次 = abspos 133+100=233）。
- clamp/隐藏 pass 里「豁免绘制」与「参与收缩计算」是两个独立决策，豁免绘制的盒不能同时参与 extent。
- 中间方案先跑**全目录 A/B** 再定：两版否弃方案都是单案对、邻案错，css-overflow 目录一次 A/B 就暴露。
