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

## 附加发现（未修，留待专项）

- **布局 estimate 启发式与真实宽度的长期错位**（`estimate_char_width`：字母 0.55em、em dash 0.5em 但实际 1em——差 2 倍）是换行点错误的放大因素，属于 font-stack 重建方向。
- **webfont 页面的 fragment 主字体解析**：`@font-face` 字体可能未成为 fragment 的 primary（fallback 到 generic），paint 的 measure 上下文（thread-local primary font_id）与 shaping 的字体分离——多字体页面字距与 Chrome 仍可能差 ~1px/词，需单独排查。
