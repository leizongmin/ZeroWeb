# 字体加载、整形与字形缓存

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

字体数据从平台/下载资源进入 `FontLoader`，resolver 把 CSS 请求变成有序 face 列表，shaper 产出带真实 face/index 的字形，CPU/GPU 共用位图光栅与缓存。加载、匹配、整形、度量和绘制必须一起定位，不能只改其中一端。

### User Task Navigation
| 用户问题 | 源码入口 | 阅读主题 |
|---|---|---|
| 粗体、字体族、下载字体或 unicode-range 不生效 | `font/system.rs`、`font/face_match.rs`、`FontLoader` | 加载与匹配 |
| 字距、连字、变量字体或缺字缓存错误 | `TextShaper`、`measure_text_hmtx`、`GlyphCache` | 整形与光栅化 |
| 布局与文字绘制宽度不同 | `crates/layout-engine/src/font_resolution.rs`、`crates/engine/src/text_metrics.rs` | 两个主题共同核对输入 |

## Gotchas
- loader 的 CSS resolver、hmtx 测量与 raster bitmap 是不同产物；注册字体成功或字形位图存在不能证明布局已采用相同 face、字号、feature 与 variation。 (`crates/render-foundation/src/font/loader.rs`, `crates/render-foundation/src/font/loader/hmtx.rs`, `crates/engine/src/text_metrics.rs`, `crates/layout-engine/src/font_resolution.rs`)

## Architecture
- layout-engine 与 engine 均调用 render-foundation 的 face 匹配协议，物理字体解析不应在布局或 painter 中复制；每字符 advance 与完整 shaping 宽度各有适用条件。 (`crates/layout-engine/src/font_resolution.rs`, `crates/engine/src/text_metrics.rs`, `crates/render-foundation/src/font/face_match.rs`)
- 下载字体跨进程需携带字节和 collection face 并在消费端重映射 ID；同一数字在不同页面不是全局字体身份。 (`crates/paint-convert/src/fonts.rs`, `apps/compositor/src/font_resources.rs`)

## Child Knowledge Nodes
- [系统字体、下载字体、匹配与 WOFF](loading-and-matching.md)：查平台顺序、别名、collection 和 CSS 字体加载。
- [OpenType、fallback、度量与光栅化](shaping-and-raster.md)：查字距、glyph index、轴坐标、缓存失效和位图基线。
