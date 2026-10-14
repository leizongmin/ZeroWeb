# OpenType、fallback、度量与光栅化

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `TextShaper`、`TextShapingOptions`、hmtx 度量、FreeType/fontdue 光栅化和 `GlyphCache`。排查文字 advance、连字、CJK 缺字、variation 串缓存或基线偏移时读这里。布局最终 IFC 的几何真值归页面内核，不在本节点复制算法。

## Gotchas
- OpenType glyph index 只在产生它的 face 内有效，indexed raster 路径不做字体回退并检查索引范围；不能把同一个数字当 Unicode 码点或换个 fallback face 光栅化。 (`crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/cache.rs`)
- `GlyphKey` 区分码点与 `font_glyph_index`，还包含取整字号和 variation 的浮点 bits；仅用 `(font_id, glyph_id)` 去缓存会混合索引命名空间或变量字体形状。 (`crates/render-foundation/src/font/cache.rs`)
- fallback 光栅化可能把 raw font ID 解析成另一个 face，cache 单独记 raw→resolved 映射；插入用 resolved 而查找用 raw 又不记映射会导致每帧 miss。 (`crates/render-foundation/src/font/cache.rs`)
- FreeType 失败时无 variation 可回退 fontdue，有 variation 时返回错误；关闭 `freetype-raster` 后 variation-aware raster 明确报错，不能静默丢轴坐标。 (`crates/render-foundation/src/font/loader.rs`)
- `ZW_SHAPED_FALLBACK` 当前须显式为 `1` 才启用；不要把 shaper 的多 face 实现存在等同于所有运行路径默认启用 shaped fallback。 (`crates/render-foundation/src/font/shaper.rs`)
- Ahem 不是所有非空白字符都无条件画满 em 方块：当前默认 `ZW_AHEM_REAL_OUTLINE` 路径尊重真实轮廓，降部字形与合成 disclosure triangle 另有分支。 (`crates/render-foundation/src/font/loader.rs`)

## Architecture
- shaping cache key 当前使用字体字节 hash 加 face index，并包含 loader-local IDs、descriptor scale、unicode ranges、各 face feature/variation、字号 bits、size-adjust、方向、语言和文本；旧字段注释中的 instance-ID 描述不能替代实际 key。 (`crates/render-foundation/src/font/loader/shaping.rs`, `crates/render-foundation/src/font/loader.rs`)
- face 默认 feature 被调用方同 tag 覆盖；`TextShapingOptions` 的语言、方向、variation 与 size-adjust 都影响结果，不能只按文本和字号复用 shape。 (`crates/render-foundation/src/font/loader/shaping.rs`, `crates/render-foundation/src/font/shaper.rs`)
- `measure_text_hmtx` 测量无 shaping 上下文的字符 advance，可逐字符求和；它与 `unshaped_advance_x` 同源，不包含 kerning/GPOS，不能拿它代替完整 shaped run 宽度。 (`crates/render-foundation/src/font/loader/hmtx.rs`, `crates/render-foundation/src/font/shaper.rs`)
- hmtx 字符缓存按完整字体链和字号分组，最多 4096 条；借字节的 TLS face 缓存由 Arc 保活且按 font instance ID 隔离，最多 16 个 face。 (`crates/render-foundation/src/font/loader/hmtx.rs`)
- FreeType face 缓存是线程局部、最多 8 项，variation 更新从各轴默认值重新构造并 clamp 合法范围；不传轴不能沿用上次 variable face 的坐标。 (`crates/render-foundation/src/font/loader/freetype_raster.rs`)
- 位图上屏左上角按 `(x + x_offset, baseline_y - y_offset - height)` 计算，FreeType `bitmap_top` 映射为 `y_offset = bitmap_top - height`；更换光栅器必须保留这一坐标契约。 (`crates/render-foundation/src/font/loader/freetype_raster.rs`)

## Decisions
- FreeType advance 测量使用 `NO_HINTING` 避免整像素取整，光栅轮廓仍保持 hinting；历史故障由测量与 rustybuzz hmtx 不同源引起，不能通过统一开启 hinting 修字距。 (`crates/render-foundation/src/font/loader/freetype_raster.rs`, `crates/render-foundation/src/font/loader/hmtx.rs`, `git:7cddc3c597c066ef87065f6f9e3ed775c9f59419`)

## Patterns
- `GlyphCache::get_shared` 用 Arc 返回位图避免逐字 memcpy；命中不提升 LRU，淘汰约 25% 条目，实际是近似 FIFO 而非严格 LRU。 (`crates/render-foundation/src/font/cache.rs`)
- shaping cache 到 65536 项整体清空；descriptor 注册或字体链变化会清缓存，扩大缓存只能改善复用，不能取消这些语义失效点。 (`crates/render-foundation/src/font/loader/shaping.rs`, `crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/loader/unicode_range.rs`)

## Testing Strategy
- `cjk_fallback` 测试依赖 Linux 系统字体，缺字体直接返回，Emoji 不支持光栅化也可仅打印错误；绿色测试状态不能证明当前机器的 CJK/Emoji 实际覆盖。 (`crates/render-foundation/tests/cjk_fallback.rs`)
- hmtx 常驻测试比较多段文本的测量与 shaping 裸 advance，另测 duplicate、独立 loader、unicode-range 和 fallback 失效；这些静态判据不代表本轮已执行。 (`crates/render-foundation/src/font/loader/hmtx.rs`)
