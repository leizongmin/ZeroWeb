# R5327 — 扩张轴 FIXME 收口实验取证（负结果轮，无 corpus 变化）

- **日期**: 2026-10-29
- **性质**: 实验轮（负结果）——R5171 `inline_ca_paint_override` 收缩轴门放开为双轴
  （扩张轴 FIXME 收口尝试），同树 corpus A/B + 像素级取证定谳**死路**后**全量回退**。
  corpus 16590 案 **809F/810F 不变**（±1 = content-none-select-1 负载脆弱族 flake；
  FIXED=0，REGRESSED=0 净账），无 trend 行。
- **基线**: R5326 交付态（clean HEAD 本地复跑 809F，与 R5325 记账一致）

## 实验 WIP（已回退）

单 hunk `crates/engine/src/paint/painter/mod.rs` `inline_ca_paint_override`：删除
「只收缩不扩张」门（`reported_h > box_node.height − 0.25 → None`），保留 ≤0.25px
等值回退与 clone 覆写路径。driving: line-height-125（✗1.20%，CSS2/linebox）——
span line-height 10px < 主字体 A+D≈58px 时报告盒高于树盒，box-level 绿底按内容区
锚定绘制（CSS2 §10.6.2）以盖住上一行 FAIL 字形。旧挂账（R5171 FIXME）预言扩张轴
与 highlight overlay 行盒缘语义及 R639/R1442 fragment 站相交、active-selection-027
A/B 实证翻红；实验假设「旧实验面 ≠ 本 deco-override 面」。

## 定谳（A/B + 像素取证）

- **corpus A/B（同树全量）**：基线 fail-list 810F（本会话首跑）vs WIP 810F/810F
  两跑。逐案 diff 恰 **+active-selection-027**（✗1.16%，同二进制复跑同值 = 真回
  归）与 **−content-none-select-1**（WIP 树两跑均 ✓0.00% = 基线跑的负载脆弱族
  flake，R5313/R5317 既有账）。**WIP 净效果 = 0 修 −1 回归。**
- **driving 案零像素**：line-height-125 test 页 PNG base vs WIP **AE=0（逐字节一
  致）**——扩张轴门放开对该页零效果；corpus % 1.20↔1.08 为 run-to-run 漂移
  flake（同二进制复跑双值稳定复现于不同树，见 flake 清单）。
- **可达性插桩（临时 eprintln，已随回退移除）**：`inline_ca_paint_override` 入口
  + reported 记录值双点插桩跑单案——整页渲染（test+ref）**无任何 fs=50 盒进入本
  函数**，全部命中为 runner 预热页（welcome）的 fs=16/25 盒。绿底 span 在
  `inline_reported_rect?` 前更早离开（或 box-level 装饰节整体不达）。
- **真根因（路径重定位）**：line-height-125 的 `.pass` span 绿底不走 R5171
  box-level 面，而是 **R639/R1442 per-line-fragment 臂**（painter/text.rs，
  `owner_h > ml_gate_lh*1.5 || phasea_orphan_fire` 门）——该 span 行盒域高度
  （~10px）不达多行门、亦非 orphan（owner_h 缺无条目），其背景实际由
  R2160 Phase A `ZW_PHASEA_MULTI_INLINE` orphan 臂或 R1441 bleed 扩展的行盒锚
  fragment 背景承担（绿带 ~18px = 字形带 + padding-top 10px，行盒锚非内容区锚）。
  R5171 收缩轴轮（inline-non-replaced-height-002/003）与本案的 span 树形态不同
  （前者 span 有独立 LayoutBox 走 box-level 面），扩张轴实验落错施画面。
- **active-selection-027 回归面**：该页 inline 盒**确实走** box-level 面（R5086
  位点消费），扩张轴放开后选区页两侧分化翻红——R5171 FIXME 的约束
  「扩张轴须与 fragment/selection 站同切片校准」在本 deco-override 面**复验证成
  立**，实验假设被 A/B 否证。

## 回退依据

准则 4/5：驱动案零像素（施画面错位）、无 witness、无 pin、有稳定回归——保留即
无验证面且带已知红的行为变更。FIXME 注释原文随回退恢复（扩张轴继续门控）。

## line-height-125 真修复方向（挂账，深结构/跨切片域）

绿底需按内容区锚定盖住邻行字形，须在 **R639/R1442 per-fragment 臂**引入内容区锚
（span A+D + padding/border，基线锚定）——该臂受三重既有约束：R638 双绘史（门改
动须与 box-level 抑制门成对，R5296 成对不变量）、R1442 fragment 站与选区高亮行盒
缘语义相交（R5052/R5054 口径）、orphan 判定与 R2197/R2198 skip 链耦合。与扩张轴
FIXME 同属「fragment/selection 站同切片校准」域，非单点轻修。R5228 域核时代曾因
paint 域跨流碰头挂账不代修；现 paint 面已归本流（R5313/R5320/R5325 先例），但
切片规模仍属深结构专项轮，不属账本择轻。

## flake 清单增补（同二进制复跑实证）

- `css/CSS2/linebox/line-height-125`：✗1.20% ↔ ✗1.08%（非骑阈，% 值漂移）——
  全量 % 逐值 diff 归因时先复跑再定谳（R5326 教训同款）。

## 下一步（承接 R5326 菜单）

①809F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF 锚域）②multicol
嵌套分片专项轮预研（8×1.04% 聚簇杠杆最大）③css-flexbox 52F 域内挑轻
（writing-mode-008 1.25%、min-width-auto-004 1.45% 等候选）。
