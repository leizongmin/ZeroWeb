# R3424-F 默认开启后 layout 10x 回归：每 IFC 全文档 collect/clone 的 O(n²)

日期：2026-08-15
相关模块：layout-engine（inline_finalization / font_resolution / engine）

## 问题描述

R3424-F（author face layout advance）默认开启 advance source 后，morning fixture 的
layout_ms 从 6.8ms 涨到 72ms（10x），medium fixture（数千个小 IFC）layout 甚至到
12.3 秒（perf-gate Hard Gate 2000ms 绝对预算被击穿）。CI benchmarks 连续多轮 FAIL。

## 根因分析

`configure_inline_fonts` 在**每个 IFC 构造时**都执行：

1. `collect_font_overrides(doc, styles, root, resolver)` —— 从 IFC root **递归遍历整个
   子树**，对每个文本节点调用 `resolve_font_ids_for_style`（内部 4 次 `format!`
   字符串堆分配 + HashMap lookup）。morning 446 个 IFC × 全文档遍历 ≈ 44ms。
2. 注入时 `overrides.ids.clone()` —— **每 IFC clone 全文档 map**。medium 数千个
   小 IFC × 全文档 map clone ≈ 12 秒。

本质：**原本 opt-in 的路径（advance source 只在 `ZW_SHAPED_LAYOUT=1` 时注入）被
R3424-F 改成默认开启，而 collect/clone 的成本与 IFC 数量 × 文档大小成正比（O(n²)）**。
R3234-F 曾有同源教训（全量 shaping 37x 回归 → 改回 opt-in），R3424-F 想"只对
author face 生效"却未审计 collect 路径的复杂度。

## 解决方案（三层递进，各独立生效）

1. **collect 提升 pass 级一次**（`LayoutEngine::collect_font_overrides_for_pass`，
   Rc 持有全文档结果，所有 IFC 共享）——从"每 IFC 一次全子树遍历"变为"每 pass
   一次全文档遍历"。
2. **collect 内部按 (family, bold, italic, stretch) 组合 memo**——页面内组合数远小于
   文本节点数（morning ~1500 节点 vs ~10 组合），避免每节点重复 format! 分配：
   全文档 collect 8ms → <1ms。
3. **注入改 Rc 共享**（IFC 的 4 个 override 字段 + FontOverrides 都改
   `Rc<HashMap<...>>`，`with_*` 只收 Rc，读取点经 auto-deref 零改动）——每 IFC
   仅 `Rc::clone`（O(1)），消除"每 IFC clone 全文档 map"的 O(n²)。

修复后：medium 12.3s → 326ms，morning ~14ms；`font-size-adjust-009/010/011` WPT
保持 PASS（行为零变化：overrides 内容与注入条件不变，Rc 只是共享方式）。

## 如何避免

- 给"默认开启"的路径做**复杂度审计**：凡在布局/绘制热路径上按节点/IFC 粒度执行
  全文档扫描或全量 clone 的，都是 O(n²) 隐患（本仓教训：R3234-F、R3424-F 两次
  同源回归）。
- 字体 ID 解析（`resolve_font_ids_for_style` 类）是纯函数且组合数有限——**组合
  memo 是低成本高收益的标准模式**。
- 共享只读数据用 `Rc`/`Arc` 而非 clone；Rust 的 auto-deref 让读取点无需改动。
