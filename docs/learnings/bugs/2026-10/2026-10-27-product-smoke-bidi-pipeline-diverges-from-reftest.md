---
date: 2026-10-27
modules: crates/engine/src/paint/painter/text.rs, tests/wpt-runner（bidi 域取证）
---

# product-smoke 与 reftest 判定管线在 bidi 域分叉——新鲜 PNG 也不可代表 reftest 判定

**触发轮**：R5299（`ZW_INLINE_SINGLE_FRAG_BOX_BORDER` 旗标矩阵仲裁）

## 问题描述

R5299 给 bidi-001 旗标开跑 product-smoke 取 PNG 证据，看到测试页文本顺序**明显错误**
（visual reorder 与 chromium 语义差距一眼可见），据此预判该页旗标开必红；随后 reftest
直跑同一页旗标开却判 ✗1.01%（基线 ✓0.71%）——只超阈 0.3 个百分点，与 PNG 观感的
「大面积错误」完全不成比例。两条管线对同一页面的渲染输出不同。

## 根因分析

product-smoke 的 fixture 渲染路径与 reftest 的 test/ref 成对判定路径是**两条不同管线**：
bidi 域二者的文本重排输出不一致（product-smoke 侧字序错误显著，reftest 侧差异收敛在
~1% 判别带）。对一般 CSS 几何页两条管线输出近似（既往轮次 PNG 取证有效），但 bidi
重排恰好踩中分叉点——用 product-smoke PNG 评估 bidi 页旗标影响会得出完全错误的量级判断。

连带教训（同轮定谳）：reftest 的 ✓ 是「判别 ≤1.00% 阈」不是「正确」。bidi-001/003/005/
006/010a 基线全绿实为**伪通过**——引擎与 chromium 已分叉（RLO 不跨 span 边界传播、无
跨 item L2 重排、字形级原位反转），~0.71% 判别藏在阈值下；旗标开的 border 像素把判别
推过 1.00% 才显红。绿只证明「与 ref 差 ≤ 阈」，几何面改动（border/padding/背景）会抬高
判别基线，让伪通过现形。

## 解决方案 / 如何避免

- **bidi 域（及一切怀疑两管线分叉的域）的像素证据，以 reftest 直跑为权威**：
  `zero-wpt-runner reftest <单页>` 逐旗标隔离跑（test/ref 双页同管线判定），不用
  product-smoke PNG 做量级判断。
- product-smoke PNG 保留两个用途：趋势指标（diff% 历史）与 struct-check 结构信号；
  不作为单页语义正确性证据。
- 评估「翻转 kill-switch」前，先对目标页跑旗标关基线判别值：基线判别已接近阈值
  （>0.5%）的页，任何抬高判别的几何改动都会让它翻红——这可能是伪通过现形而非回归，
  须逐页归因后再计入翻转成本。
