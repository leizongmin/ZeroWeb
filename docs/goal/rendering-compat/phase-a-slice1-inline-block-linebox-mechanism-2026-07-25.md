# Phase A 首切片机制方案 — inline-block line-box 贡献

**日期**：2026-07-25
**性质**：Phase A 最高杠杆轨的可执行首切片机制方案（改动区域 + 验证 + 风险，**非代码实施**）。pre-authorized ruling #4。
**关联**：[blockers-resolution-plan-2026-07-25.md](blockers-resolution-plan-2026-07-25.md) §2、[phase-a-IFC-unification-design.md](phase-a-IFC-unification-design.md) v1.4、[inline-box-model-unification-design.md](inline-box-model-unification-design.md)（R1576 谱系）

---

## 目标

inline-block（及含 inline-block 后代的 inline 元素）的 margin-box 高度正确贡献父 line-box，**解 37-form-controls label overlap**（legacy smoke 唯一 struct FAIL，Phase A 阻塞的 success signal）。

## 缺失点（LAYOUT_DUMP + 代码定位）

37-form-controls 第一个 `<p>`：
```
p        h=18.6        ← line-box 只算 label 文本行高（16×1.16）
  label  h=33.6        ← label 含 inline-block input(h=21)，实际 33.6
    input h=21.0
  label  h=33.6  y=64  ← 第二个 label，与前一个(43~76.6)重叠 ~12px
```

**根因**：label 这种"inline 元素含 inline-block 后代"的 box_height（被 input h=21 撑到 33.6）**没贡献父 `<p>` 的 line-box**，p 只算到 label 文本的 line_height（18.6）。

## 改动区域（机制层，不写代码）

### 1. `crates/layout-engine/src/inline/break_lines.rs` — line-box 高度组装
- 当前：`current_line.height = max(run.line_height)`（line 131-132 文本 run 分支）；inline-block box_height 仅在"空 inline 元素"分支贡献（line 56-57 `run.box_height()`）
- **缺失**：非空 inline run（label，含文本 "Name: " + inline-block input 后代）的 box_height 没进 current_line.height——它走文本分支只贡献 label 文本 line_height
- 改方向：inline run 的有效 box_height（含 inline-block 后代 margin-box 撑高量，≡ R1576 inline-box-model recurse）须纳入 `current_line.height` 的 max（line 131-132 区 + inline-block arm ~line 374）

### 2. `crates/layout-engine/src/inline/mod.rs:1125-1192` — line-box metric（apply_vertical_alignment）
- 当前（line 1183）：inline-block（font_size==0 原子盒）`max_ascent = max(..., run.baseline)`（baseline=底部边缘）
- **缺失**：`line.descent = line_height - max_ascent`（line 1192）未含 inline-block 底部（inline-block height > baseline 时，底部 descent 被吞）
- 改方向：inline-block 的 `descent = height - baseline` 须纳入 line.descent 的 max；line_height 须 = max(文本 line-height, inline-block margin-box)（三量协同，非单点）

## 验证（三态门禁，net 负即回退）

1. **目标**：37-form-controls struct **FAIL→PASS**（5 issue 消：3 sibling overlap + 2 text-concat 中 overlap 部分；text-concat 的 R109 部分可能仍存，归 Phase A 后续）
2. **回归**：`welcome` struct PASS + diff <20%（字节一致或噪声内）
3. **oracle**：linebox / css-text / normal-flow 三 dir **bit-identical net-0**（inline-block 裸用在 WPT reftest 罕见，≡ R1659 input UA 谱系）
4. **self-source** 通过率不降

## 风险

- **交叉点**：R1576（inline-box-model 递归测量）× Phase A（line-box metric）—— deadlock 史 R125/R206/R213
- **须三路径协同**：break_lines（组行）+ mod.rs apply_vertical_alignment（stored metric）+ paint Path A（读 stored）三处一致改，单点 net-negative（Phase A 教训）
- **kill-switch** `ZW_INLINE_BLOCK_LINEBOX=0` default-off + 结构签名 gate（37 号 fixture）+ A/B 三态门禁
- 净负即回退，留机制验证记录

## 与 Phase A 整体的关系

本首切片是 Phase A line-box metric unification 的**窄入口**——只解 inline-block 高度贡献（最常用、可独立验证、product-visible 37号）。**不解**：vertical-align baseline 对齐（va-117a 簇）、Path B 消灭、per-font metric 注入（U1b dormant）——那些是后续切片。**本切片成功 = Phase A 首个可执行实证 + 37-form-controls product gap 修**，为 Phase A 多 session 推进建立三态门禁基线。

## 依据

- 代码：`break_lines.rs:52-86/131-132/374` + `mod.rs:1125-1192`（apply_vertical_alignment）
- 实证：LAYOUT_DUMP 37-form-controls（p h=18.6 vs label h=33.6）
- 设计：[inline-box-model-unification-design.md](inline-box-model-unification-design.md)（R1576）+ [phase-a-IFC-unification-design.md](phase-a-IFC-unification-design.md) v1.4
