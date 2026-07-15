# Spec + RFC：per-fragment inline border/padding/bg 上溢（CSS §10.8.1 half-leading）

**版本**：v1.0
**日期**：2026-07-15
**状态**：⚠️ **VERIFIED STALE（2026-07-16 / R1508）—— 勿再投入**。R1442 已落地 per-fragment bg/border 外延几何（text.rs:1540-1614），多行 gate 已通过（border-padding-bleed-001 PIL 实测 **0 red px**，bleed 正确覆盖前行）。残余 0.78% **非 gate 问题 = font-wall C-dep**：BLEEDDBG 探针实测 `line_h=40`（resolve_font_metrics Number(1.0)=font_size 干净）但 `line_top=90.624`（非 90.0）——绿矩形 `[50.624,130.624)` 光栅成 81 行 vs chromium `[50,130)` 80 行，多出的 y=130 行即此 diff。0.624px 偏移 = 测试页顶部**非 Ahem `<p>` 指令文本** `line-height:normal`（ZW 1.164 比率 vs chromium 真值）把整个 `<div>` 下移 0.624px。= **R1155 font-wall**（默认字体 `<p>` 指令文本 floor），C-dep user-gated。本 RFC §1.1「单行 span 不画 bg」前提**已被 R1442 推翻**（gate 已通过），relax-gate 提议是 no-op。关闭。
**关联**：[`no-ratio-replaced-sizing-rfc.md`](./no-ratio-replaced-sizing-rfc.md)（R1437/1438 SVG-sizing 收口后下一 Phase A 目标）；font-wall 谱系见 master.md R1067/R1088/R1155

---

## 0. 执行摘要

- **一句话目标**：让 inline 非替换元素的 `background-color`/`border-top`/`padding-top` 按 CSS §10.8.1 正确**上溢出 line box**（覆盖前一行内容），修复 `border-padding-bleed-001/002/003`。
- **本期范围**：仅水平书写模式下、inline 元素的 per-fragment bg/border/paint **垂直外延**（向上覆盖前一行 line box）；paint order（后行盖前行）依赖既有 document-order。
- **明确排除**：inline `outline`/`shadow`（仍无 driving test，R648 defer 不变）；vertical writing mode；`border-bottom`/`padding-bottom` 下溢（对称，本期可一并但须 A/B 守）。
- **核心约束**：① 不破坏 R639 per-fragment inline-bg（multi-line gate）；② 不引入 single-line inline-bg 双计回归（R638 blanket revert 先例）；③ A/B 净负即回退。
- **推荐方案**：扩 R639 `Painter.inline_heights` 桥——per-fragment 计算 inline box 的**外延矩形**（含 padding+border），bg 填该矩形，border 按四边绘制；gate 从「仅多行」改为「inline 有 bg 或 border 或 padding 且 box 非绝对」。
- **首个落地步骤**：读 R639 `text.rs:1492-1520` per-fragment bg + `inline_heights` bridge，写最小 PoC：对 border-padding-bleed-001 的 span 画 bg 矩形（content_y - padding_top - border_top 起，高 = line_h + padding_top + border_top），A/B 看 red 是否被覆盖。

---

## 1. 背景与目标

### 1.1 背景

R1439 深挖 CSS2/linebox 发现 `border-padding-bleed-001/002/003`（2.13/4.27/1.23% FAIL）= **concrete driving test**（推翻 R648「per-fragment inline border 无 driving test」）。测试：`<div color:red font:40px/1 Ahem>` 第 1 行红字 + `<br>` + 第 2 行 `<span bg:green border-top:15 padding-top:25>`。ref = 640×80 纯绿矩形。期望：span 的 green bg+border-top+padding-top **上溢入第 1 行 line box 覆盖红字**（CSS §10.8.1：inline margin/border/padding 不入 line box 高度但仍渲染于 inline box 之外）。

ZW 现状：R639 per-fragment inline-bg（text.rs:1492-1520）gate `owner_h > frag_fs*1.5`（**仅多行**）→ 单行 span（line 2）不画 bg；且即使画，矩形高度仅 `line_h`（不含 padding/border 上溢）→ 红字外露。

### 1.2 目标

- 业务：border-padding-bleed-001/002/003 FAIL→PASS。
- 用户：inline 元素带 bg/border/padding 时正确覆盖邻接行（真实页面 inline 高亮/标记不漏底色）。

### 1.3 范围边界

- **在范围内**：水平书写模式 inline 元素 per-fragment bg/border 矩形**垂直外延**（padding+border 上溢/下溢覆盖邻接 line box）。
- **不在范围内**：inline outline/shadow（无 driving test）；vertical writing mode；margin（inline 水平 margin 已渲染，垂直 margin 归零 R1058）。

---

## 3. 功能需求

### FR-001：inline bg 矩形垂直外延含 padding+border

- **描述**：对有 `background-color` 的 inline 元素，per-fragment bg 矩形须从 `content_y - padding_top - border_top` 起，高 = `line_h + padding_top + border_top + padding_bottom + border_bottom`（覆盖 padding/border 区域）。
- **优先级**：必须

**验收场景**：

```
场景: border-padding-bleed-001 单行 span bg 上溢覆盖前行红字
  假设 div color:red font:40px/1 Ahem，第2行 span bg:green border-top:15 padding-top:25
  当渲染
  那么 span bg 矩形从 (line2_top - 25 - 15) 起，覆盖第1行红字 → 无红可见
  验证: make reftest DIR=css/CSS2/linebox → border-padding-bleed-001 PASS
```

### FR-002：inline border per-fragment 绘制

- **描述**：inline 元素 `border-top`/`border-bottom`（及左右）按 fragment 绘制；`border-top` 在 bg 矩形上沿，覆盖前一行。
- **优先级**：必须

### FR-003：gate 放宽不引入 single-line 双计

- **描述**：R639 multi-line gate 放宽为「inline 有 bg/border/padding 且非绝对定位」；须 A/B 守 single-line inline-bg 测试无双计回归。
- **优先级**：必须

---

## 4. 非功能需求

### NFR-001：零回归
- **测量**：linebox + normal-flow + css-text oracle A/B net≥0、0 pass→fail；product-smoke welcome 不升。

---

## 6. 约束与假设

### 6.1 必须约束
- per-fragment bg/border 用 R639 `inline_heights` owner-height 桥（避 R638 owner-vs-box 分歧）。
- paint order 依赖既有 document-order（后行 fragment 后画 → 盖前行），不重排。

### 6.2 禁止约束
- 不得对无 bg/border/padding 的 inline 画任何矩形（性能 + 回归）。
- 不得改 IFC layout（line box 高度计算不变，§10.8.1 padding/border 不入 line height）。

### 6.5 假设
- ZW paint 顺序已按 document-order（line 2 fragment 后于 line 1）—— 状态：待 A/B 验证（若 red 仍外露且 bg 已画对位置，则须查 paint order）。
- R638 双计根因是 single-line inline bg 与 block bg 重叠 —— 状态：待复现（R639 gate 注释暗示，须读 R638 evidence 确认）。

### 6.6 代码变更边界
- **允许**：`crates/engine/src/paint/painter/text.rs`（per-fragment bg/border 扩展）、`painter/mod.rs`（inline_heights 若需扩 border/padding 字段）、对应单测。
- **禁止**：layout-engine（IFC 行盒计算不改）。

---

## 7. 实施交接

### 推荐修改顺序

1. **PoC**：text.rs:1492-1520 per-fragment bg，对 border-padding-bleed-001 手工算外延矩形，A/B 看 red 是否消失（验证 paint order + 几何方向）。
2. **gate 放宽**：multi-line → 有 bg/border/padding；A/B 守 single-line 双计（找 R638 回归案复现）。
3. **border 绘制**：per-fragment border-top/bottom（+ 左右）。
4. **全量 A/B**：linebox + normal-flow + css-text + product-smoke；净负即整体 revert。

### 回滚计划
- 单 commit；A/B 任一 dir net 负或 welcome 升 → git revert，记录 R1438-style 失败摘要（哪 dir 回归、diff 变化）。

---

## 8. 技术设计（RFC）

### 8.1 现状
R639 per-fragment inline-bg（text.rs:1492-1520）：gate `!is_vertical && !abs && owner_h > frag_fs*1.5 && display==Inline && bg!=Transparent`，画 `add_fill(Rect(frag_base_x, content_y+frag_y, text_width, line_h))`。问题：① gate 排除单行；② 矩形仅 line_h（无 padding/border 外延）；③ 无 border 绘制。

### 8.2 目标状态
- gate：`!is_vertical && !abs && display==Inline && (bg!=Transparent || has_border || has_padding)`。
- bg 矩形：`Rect(x - pad_left - border_left, y - pad_top - border_top, text_width + pad_lr + border_lr, line_h + pad_tb + border_tb)`（外延含 padding+border）。
- border：per-fragment 四边（border-top 在矩形上沿，border-bottom 在下沿）。

### 8.6 替代方案

| 方案 | 优点 | 缺点 | 决定 |
|------|------|------|------|
| A. 扩 R639 桥 + 外延矩形 | 复用既有机制，最小改动 | gate 放宽双计风险 | ✅ 选定（PoC 先验） |
| B. paint 期重算 inline box 几何 | 更准 | 重构大，Phase A 全量 | ❌ 多 session |

### 8.8 测试策略
- 单测：per-fragment bg 外延几何（span padding/border 各组合）。
- reftest：`make reftest DIR=css/CSS2/linebox`（bleed 3 案）+ normal-flow/css-text 回归 + product-smoke。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要 | ✅ Pass | §0 |
| 场景存在性 | ✅ Pass | FR-001~003 各有场景 |
| 测试绑定 | ✅ Pass | make reftest DIR=css/CSS2/linebox |
| 实施交接 | ✅ Pass | §7 顺序+回滚 |
| 首步可执行 | ✅ Pass | §7 步骤 1 PoC |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 与 FR 无交集 |
| 代码边界 | ✅ Pass | §6.6 允许/禁止 |
| 实现来源闭合 | ✅ Pass | R639 inline_heights 桥复用 |

**汇总**：8 Pass / 0 Warning / 0 Fail
**门禁判定**：允许实施（PoC 先行，A/B 净负即回退）

---

## 11. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-07-15 | 初始版本（承接 R1439 linebox 深挖） |
