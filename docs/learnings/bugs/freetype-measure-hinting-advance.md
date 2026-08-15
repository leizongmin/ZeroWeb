# FreeType measure_advance hinting 取整导致英文文本字距与水平位置错乱

日期：2026-08-15
相关模块：`crates/render-foundation/src/font/loader/freetype_raster.rs`、`loader.rs`、`crates/engine/src/paint/painter/text/`

## 问题描述

渲染页面时英文字体异常：与 Chrome 对比字体间距和水平位置系统性错乱（字距忽宽忽窄、词间位置漂移、换行点错误）。Linux 上症状较轻，Windows/macOS 上更明显。回归由「最近几天」的改动引入。

## 根因分析

`measure_advance`（字符 advance 测量）在提交 `1edfac64`（2026-08-09，CJK 性能优化）从 fontdue（精确 f32 hmtx）切到 FreeType 后，`load_glyph` 使用 `LoadFlag::DEFAULT`——**hinting 会把 advance 取整到整像素**（26.6 定点下 64 的倍数），与 rustybuzz shaping 的精确 hmtx advance 不再一致：

- 实测（Liberation Sans，16px）：「Hello」FreeType=38.0px vs rustybuzz=36.46px（**差 4%**）
- 逐字符累计后，词间位置系统性漂移 → 「水平位置错乱」

该不一致被两条后续路径放大：

1. **R3235-F（08-11）**：paint 对 generic 字体走 `generic_contextual` = `paint_base + (shaped - unshaped)`——该公式隐含 `paint_base ≈ unshaped` 假设（fontdue 时代成立）；取整后假设破裂，paint 字距偏离 shaping 精确值。
2. **R3424-F（08-14，默认开启）**：同一公式进入 @font-face（author font）的**布局 advance**——布局宽度与绘制宽度不一致（Lato「AVATAR」布局 61.99 vs 绘制 66.0），换行点与词间位置错乱。

## 解决方案

`measure_advance` 的 `load_glyph` 改用 `LoadFlag::NO_HINTING`：

- 只影响 advance 读取（26.6 定点下限 1/64px，可忽略），**不动字形轮廓 hinting**——光栅路径（`rasterize_inner`）保持 `DEFAULT`（Chromium-matching 的轮廓 hinting）。
- 修复后 paint 的 per-char advance 与 rustybuzz 逐字符差 ≤ 1/64px，字距恢复与 Chrome（精确 shaping advance）一致。

回归测试：`render-foundation` 的 `measure_advance_matches_shaping_hmtx_after_no_hinting`（Lato webfont + 多段文本，断言 measure 与 shaping hmtx 求和一致，容差 = 字符数/64 + ε；修复前必失败）。

## 如何避免

- **度量路径与 shaping 路径必须同源**：advance 测量与 rustybuzz 都应以 hmtx 精确值为准，任何取整（hinting、整数化）都会在逐字符累计后变成可见的布局误差。
- 修改字体度量代码时，用「measure 与 rustybuzz 一致性」断言做回归（`ZW_SHAPED_ADVANCE_TRACE=1` 可对比 layout/fragment/paint 三方宽度）。
- FreeType `load_glyph` 的 `advance()` 受 hinting 影响；只读 advance 时用 `NO_HINTING`，渲染轮廓时才用 `DEFAULT`。

## 后续修复（2026-08-16，同一轮次完成）

### 修复 B：paint 字符 advance 按字形实际字体测量

measure 回调签名扩展为 `fn(font_id, ch, font_size, is_ahem)`（显式传字形实际解析的字体），
`ShapedAdvanceSource` 的 paint_base 与 `generic_contextual` 的 paint_base 均按字形字体测量；
painter 的 measure 缓存键加入 font_id（消除同帧跨字体污染）。多字体页面字距与 Chrome 一致。

### 修复 A：布局 estimate → hmtx 真实宽度

- `FontLoader::measure_text_hmtx`（ttf_parser 批量读 hmtx，face 缓存 + run 级缓存，
  与 rustybuzz unshaped 同源）
- `ShapedAdvanceSource` 增加 generic 判定：generic/系统字体 run 走 hmtx（替换 estimate
  15-20% 偏差）；author run 维持 shaping（R3424-F）；复杂 shaping 文本（阿拉伯/印度系）
  回退 shaping
- 布局 run 的 font_id 解析放宽（generic 也解析）；paint Path B 的 IFC 注入 font resolver
  与默认回退（sans-serif/0），使 paint-ifc 与布局引擎行断同源
- env `ZW_HMTX_LAYOUT`（默认开，`"0"` 回退 estimate）

**perf**：welcome layout 本机基线 5.5ms → 3.6-6.3ms（无回归；首版无缓存 43ms——
run 级缓存解决，教训：批量测量必须有 run 级缓存，固定开销随调用次数线性放大）。

**reftest 影响**：全量 16269 案净 +21 fail（0.13%）——换行临界用例（hanging-punctuation
等）的 ref 是 chromium shaped（含 kerning），hmtx 无 kerning 差 1-2% 导致临界换行点不同；
视觉上基本不可察觉，换取布局换行点从「偏 15-20%」到「差 1-2%」的正确性提升。

**测试防线（browser 层端到端）**：
- `text_glyph_positions_match_shaping_baseline`（T1）：独立 WebView + 双字体 loader，
  断言 glyph 位置与 rustybuzz 基准一致（修复 B 回归）
- `text_wrap_points_match_shaping_baseline`（T2）：固定宽度盒子换行点与 rustybuzz
  基准一致（修复 A 回归）
