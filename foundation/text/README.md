# zero-text-foundation

共享文本/字体基础层，被 UI SDK 和 WebView 共同使用，浏览器无关。

## 架构位置

```
foundation/text  ←── zero-ui-render  ←── zero-browser-chrome
               ←── zero-webview (通过 render-foundation)
```

与 render-foundation 的关系：foundation/text 是纯文本/字体层（fontdue + rustybuzz），不依赖任何 GPU 图形后端。render-foundation 的字体栈已统一到此 crate 的 `FontdueBackend` 作为共享后端。

## 提供的核心能力

| 能力 | 实现 | 说明 |
|------|------|------|
| 字体发现与查询 | `FontProvider` trait, `FontdueBackend` | 按 `FontRequest` 查询字体；回落 chain 系统字体 |
| OpenType shaping | `TextShaper`, rustybuzz | 将字符串 shape 为 `GlyphRun`（含 GPOS kerning） |
| 文本测量 | `TextMeasurer` | 测量文本行宽；`wrap_width_and_lines` 贪心折行 |
| 字符光栅 | `FontdueBackend::rasterize_glyph` | 按 glyph id 光栅为 `GlyphBitmap`（灰度位图 + 定位偏移） |
| 字体回退 | `FontFallbackProvider` | 多字体 fallback chain，覆盖去重 |
| 双向文本 | `bidi` 模块 | Unicode Bidi 算法支持 |
| 断行 | `line_break` 模块 | 行边界检测（含 CJK 规则） |
| 字素处理 | `grapheme` 模块 | Unicode 字素簇边界 |
| Glyph 缓存 | `GlyphCache` / `GlyphAtlas` | 去重光栅；稳定 `GlyphCacheKey`（font_id:32/glyph_id:16/size:16） |
| 顶层类型 | `TextBlob` | 预 shape 文本单元供渲染管线消费 |

## 依赖

- `fontdue 0.9`：字体度量与光栅化
- `rustybuzz 0.20`：OpenType shaping
- `serde` / `thiserror` / `hashbrown`

零浏览器业务 crate 依赖。许可证：MIT/Apache-2.0。

## 关键设计决策

- **独立目录**（`foundation/` 而非 `crates/`），确保不被 render-foundation 的 GPU 栈污染
- **DC-11 字体栈统一**：生产渲染路径默认经共享 `FontdueBackend`，与 render-foundation 的 `FontLoader` 共享同一字体实例
- **notdef 保护**：shape 时 `.notdef` glyph 回退 0.6×size 防塌陷

## 测试

- `cargo test -p zero-text-foundation` — 36 测（shaping·measure·raster·line break·glyph cache·fallback chain·wrap·clamp）
- 深度审查：2026-07-03 全 crate 审查，修复 GPOS kerning + glyph_id 截断 + wrap 防护

## 文件大小

14 源文件 ~1421 行（单文件均 ≤2000 行）。
