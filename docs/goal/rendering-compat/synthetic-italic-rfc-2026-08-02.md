# RFC: Synthetic Italic（系统字体 font-style:italic 合成斜体渲染）

**版本**: v1.0
**日期**: 2026-08-02
**状态**: Design-ready（待实施；R2495 去风险 + 本 RFC 设计完成）
**作者**: Rally agent（rendering-compat）
**关联**: R2495（去风险）、R2493（@font-face italic face 选用）、R2417（font-weight matching）

## 1. 问题陈述

### 1.1 现状（ground truth，R2495 实证）

- `font-style: italic` / `oblique` 已 parse + computed（`ComputedStyle.font_style: FontStyleValue`）。
- R2493 仅覆盖 **@font-face italic face 选用**（`resolve_font_id` 查 `{family}:italic` 键）。
- **系统字体**（无 italic @font-face，如 `<em>`/`<i>`/`font-style:italic` 在 sans-serif 上）的 italic 文本**渲染为 upright（直立）**——painter 把 font_style 传给 `resolve_font_id`，但系统字体无 `:italic` 键 → fallback 链回落 plain family（regular face）→ **无 skew**，glyph 直立渲染。
- **chromium 行为**：对无 italic face 的字体做 **synthetic italic**（~14° 水平 shear / oblique slant）。

### 1.2 影响

- 产品 fixture（welcome/morning-work/wintertc）有 ~21 处 italic 文本（`<em>`/`<i>`/`font-style:italic`），现渲染直立 → 与 chromium oracle 的 skewed 渲染产生像素差（font-wall 之外的 italic-shape 差）。
- DC-5（css-fonts）`font-variant-*` / standard-family 等深 fail 外，italic-shape 是可见正确性 gap。

### 1.3 目标

系统字体 `font-style: italic/oblique` 文本经 synthetic italic（~14° shear）渲染，对齐 chromium 视觉，**net ≥ 0**（welcome/morning/wintertc diff 不恶化）落地。

## 2. 非目标

- **layout 期 italic advance 宽度调整**：真正 italic 会改 advance widths（italic 字形更宽）→ reflow。本 RFC **仅 paint 期 shear**（不改 layout/advance），与 chromium synthetic italic 行为一致（chromium synthetic italic 也是 paint-only，不 reflow）。
- **OpenType italic shaping**：真 italic 字形（fontdue 不支持 GSUB）属 font-stack C-dep，不在范围。
- **oblique 角度精确**：`oblique <angle>` 的精确角度忽略，按 ~14° 默认（与 `italic` 同处理）。

## 3. 设计

### 3.1 关键决策：如何把 italic-synthesize 信号传到 blit

**问题**：CPU blit（`cpu/mod.rs:blit_glyph_bitmap`）按 glyph 渲染，需知道「该 glyph 须 shear」。但 `GlyphPrimitive` 现仅 `rotation`（vertical 90°），无 skew 字段；且 painter 需知「resolved face 是否 italic」（避免对真 italic face double-shear）。

**方案 A（推荐）：`GlyphPrimitive` 加 `synthetic_italic: bool` 字段 + `resolve_font_id` 返回 `(FontId, resolved_italic)`**

- `resolve_font_id` 返回 `(FontId, bool)`——bool = 是否经 italic 后缀（`:700:italic`/`:italic`）解析（= resolved face 本就是 italic，无须合成）。
- caller（painter text 渲染）据 `want_italic && !resolved_italic` 设 glyph 的 `synthetic_italic`。
- CPU blit 读 `glyph.synthetic_italic`，true 时 shear。

**代价**：`GlyphPrimitive` 加字段触 **59 个构造 site**（grep `GlyphPrimitive {` 全 codebase）。机械（cargo 编译错误逐 site 引导补 `synthetic_italic: false` 默认），但量大。

**方案 B：`RenderPrimitives` 平行 `glyph_synthetic_italic: Vec<bool>`（bitmask）**

- 平行于 `glyphs: Vec<GlyphPrimitive>` 的 bool vec，painter push glyph 时同步 push flag。
- 避免 59-site GlyphPrimitive 改动，但须改所有 glyph-push site 同步 push flag（仍 ~16 site）+ CPU 渲染循环读平行 vec。

**裁决**：方案 A 更直接（字段随 glyph，无平行结构同步风险），59-site 机械改动可接受（cargo 引导）。**采纳方案 A**。

### 3.2 `resolve_font_id` 返回值改造

```rust
// 现签名
pub(crate) fn resolve_font_id(&self, family, weight, style) -> FontId { ... }

// 改为
pub(crate) fn resolve_font_id(&self, family, weight, style) -> (FontId, bool) {
    // ... 候选后缀链 ...
    // 记录命中后缀是否含 "italic"
    // 返回 (FontId, /* resolved_italic = 命中 :700:italic 或 :italic */)
}
```

11 个 caller 更新：多数取 `.0`（FontId），仅 text 渲染 caller（text.rs 主路径）用 `.1`（resolved_italic）算 `synthetic_italic = want_italic && !resolved_italic`。

### 3.3 CPU blit shear 算法（`cpu/mod.rs:blit_glyph_bitmap`）

加 `synthetic_italic: bool` 参（或复用现有参列表加一个）。true 时第三分支：

```rust
// italic shear：~14° → tan(14°) ≈ 0.249
const ITALIC_SKEW: f32 = 0.25; // 每 row 像素水平偏移
let baseline_anchor = bitmap.height as f32 / 2.0; // 锚中点（避整体平移）
for row in 0..bitmap.height {
    let shear_dx = ((row as f32 - baseline_anchor) * ITALIC_SKEW).round() as i32;
    for col in 0..bitmap.width {
        let px = start_x + col as i32 + shear_dx;
        let py = start_y + row as i32;
        // ... blend_pixel（同非旋转分支）...
    }
}
```

锚中点（`height/2`）使 shear 上下对称（chromium 锚近基线，中点是合理近似；A/B 量化后可调）。

### 3.4 GPU 路径

GPU 渲染 glyph 若有独立路径，初版 **passthrough（不 shear，TODO）**——reftest/product-smoke 走 CPU，零影响；GPU TODO 留 follow-up（与既有 rotation GPU 处理同模式）。

## 4. 切片计划

| 切片 | 内容 | 风险 | 验证 |
|------|------|------|------|
| **slice 1** | `GlyphPrimitive.synthetic_italic` 字段（59 site 默认 false）+ `resolve_font_id` 返回 `(FontId, resolved_italic)`（11 caller 取 .0）+ 单测（resolve 返回 resolved_italic 正确） | 低（纯机械 + 签名，零行为变更——synthetic_italic 恒 false） | make test 全绿；fmt/clippy |
| **slice 2** | painter text 渲染设 `synthetic_italic = want_italic && !resolved_italic` + CPU blit shear 分支 + 单测（blit shear 产出倾斜位图） | 中（blit 改动 + 信号接通） | scoped 单测 + product-smoke A/B |
| **slice 3** | A/B 量化 + kill-switch（`ZW_SYNTHETIC_ITALIC=0`）+ 锚点/角度微调 | — | welcome/morning/wintertc diff net≥0 |

每切片独立 landable；slice 2/3 net 负则回退（同 R2393/R2479 谱）。

## 5. A/B 门禁

- **kill-switch**：env `ZW_SYNTHETIC_ITALIC`（默认开；`0` 关 = 现状 upright）。
- **三态 A/B**（stash 开/关两轮）：
  1. `make test`（live_fontface + blit 单测，零回归）。
  2. `make product-smoke`：welcome（17.03% baseline）+ morning-work（自抓 oracle，13.33% baseline）+ wintertc——italic 文本区 diff 须 **net ≤ 0**（shear 后更近 chromium skewed）。
  3. 相关 dir oracle（css-fonts `font-style-*` / css-text-decor italic）— 量 italic-shape 改善。
- **net 负 → 回退**（记 evidence，转下一切片或 defer）；**net 0** landable（spec-correctness）；**net 正** 最佳。

## 6. 风险与缓解

| 风险 | 缓解 |
|------|------|
| shear 角度/锚点 ≠ chromium → diff 不改善或恶化 | A/B 量化 + 锚点（中点 vs 基线）/角度（0.20-0.30）可调；net 负回退 |
| 59-site GlyphPrimitive 改动遗漏致编译失败 | cargo 编译错误逐 site 引导；slice 1 独立验证零行为变更 |
| GPU 路径不一致（GPU 不 shear） | passthrough TODO；reftest/smoke 走 CPU 零影响；GPU follow-up |
| broad smoke（所有 italic 文本）不可控 | kill-switch `ZW_SYNTHETIC_ITALIC=0` 一键关 |

## 7. 验收标准

- 系统字体 `font-style: italic` 文本经 ~14° shear 渲染（非直立）。
- `make test` 全绿（+ blit/resolve 单测）。
- product-smoke welcome/morning/wintertc diff **net ≥ 0**（kill-switch 可回退现状）。
- clippy/fmt 干净。

## 8. 未决 / Follow-up

- oblique `<angle>` 精确角度（现按 italic ~14°）。
- GPU 路径 shear（初版 passthrough）。
- 真 italic advance 宽度（layout 期，font-stack C-dep 后）。

## 9. 实施 Readiness

本 RFC 设计完成（方案 A 钉案 + blit 算法 + 3-slice + kill-switch + A/B）。**等实施**——slice 1（机械字段+签名）可立即开工（零风险），slice 2/3 须 A/B 守 net 负回退。
