---
date: 2026-10-23
modules: layout-engine
---

# UBA 重排产物必须剥除 bidi 控制码——visual_runs 保留控制码索引位，逐位 push 会带进视觉流

## 问题描述

bidi-005/006（test 段 = 直系文本 + RLO/LRO/PDF 控制码构造，ref 段 = 13 连续 span 构造，Chrome 中两构造逐像素同构）在 ZeroWeb 中 test 段恒比 ref 段宽，✗1.35%（R5204 实测 166px 平移，当时误归因 span-run ls 计量差挂账）。修复后两案 ✓0.95% 翻绿 + bidi-010a 连带翻绿，REGRESSED=0。

## 根因

三重泄漏同一根：`unicode_bidi::BidiInfo::visual_runs` 返回的 run 范围**保留控制码索引位**（控制码不携带自身 level，随周边文段），`bidi_reorder_paragraphs` 逐索引 push 字符时把 RLO/LRO/PDF（U+202A-202E）原样带进 `visual_text`。控制码随后流入分词/计量/绘制全链：

1. `estimate_char_width` 零宽表不含控制码 → 计一个字符宽；
2. 断行 `letter_spacing × char_count` 把控制码计入字符数 → 每控制码多计一个 ls；
3. fragment 文本携带控制码 → paint 绘出字形（reftest dump 可见 tofu 状符号）。

bidi-005 test 段 12 个控制码 × 32px（ls 1em）级累计，正是 166px 平移的主体；ref 段（span 构造）无控制码，两构造恒差。

## 解决方案

规范口径（UAX #9 X9 / css-writing-modes-3 #bidi-control-codes）：控制码驱动重排后对呈现零贡献。修复 = `BidiReorderedText` 产出出口统一 `strip_bidi_controls`（visual_text/visual_to_logical/visual_is_rtl 三平行数组同步过滤）：① `bidi_reorder_paragraphs` 尾部；② `bidi_override_mapping` 尾部；③ `with_levels_logical` 在 levels 按原文逐位 zip 完**之后**剥（剥前 zip 需原文对齐）；④ `logical()`（plaintext 逻辑序）。另在 `estimate_char_width` 零宽表加控制码分支作防御纵深（覆盖 ruby base_w / intrinsic walk 等不经 cursor 的 estimate 调用点）。

关键约束：控制码的**作用域语义**必须先被消费再剥——段落级 UBA（para_bidi）跨 run 解析需要看到原始控制码（`run.text` 不剥），levels 数组按原文字符位对齐；剥除只发生在「重排完成 → 视觉流出口」，反转语义存于 levels 奇偶不受影响（有锚测试钉住）。

## 避免方式

消费 unicode_bidi 产出的 visual 文本时，牢记其 API 保留控制码索引位是给调用方自己剥的（crate 文档惯例）；任何「重排产物 → 计量/绘制」的边界都要过 X9 剥除。诊断特征：同视觉两构造（span 拆分 vs 直系文本 + 控制码）宽度恒差且差值 ≈ N × letter-spacing，渲染出现来历不明的小字形——先查控制码是否泄漏进视觉流。
