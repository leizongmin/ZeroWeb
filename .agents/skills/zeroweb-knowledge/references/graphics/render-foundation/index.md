# 共享渲染基础设施

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-render-foundation` 提供共享图元、CPU/wgpu 光栅、字体与图片缓存，不负责 DOM/CSS 布局、页面资源获取或 IPC 角色调度。browser、compositor、WebDriver 以及 layout-engine/Canvas 各按实际入口消费，不能从共用 crate 推断各产品资源同步已等价。

## API Surface

| 真实 API | 契约 |
|---|---|
| `primitive::RenderPrimitives` / `DrawOp` | 类型数组加跨类型顺序，含 glyph、图片、滤镜和非绘制文本边界。 |
| `cpu::render_full_scene` / `render_full_scene_region_into` | 前者创建按 DPR 缩放的白底帧，后者向已有 FrameBuffer 写指定区域。 |
| `gpu::renderer::GpuRenderer::render_full_scene_gpu` | 消费完整场景与资源缓存，返回是否成功绘制；false 需调用方处理。 |
| `font::FontLoader` / `TextShaper` / `GlyphCache` | face 注册、CSS resolver、shape/measure/raster 与位图复用。 |
| `image_cache::ImageData` / `ImageCache` | 解码像素、内容摘要、固有尺寸信号与引用/预算管理。 |
| `display_list::DisplayList` | 图元与本帧 CSS 脏区的录制契约。 |
| `backing_store::BackingStoreManager` / `rendering_thread::RenderingThread` | 双缓冲提交与持久 CPU job；线程和资源失效仍由宿主协调。 |

## Usage Examples

compositor 从 IPC 脏区构造 display list，再决定全量或局部光栅化。来源：`apps/compositor/src/rasterize.rs`。

```rust
let dirty = ipc_dirty_rects(&paint.dirty_rects);
let display_list = DisplayList::new(primitives.clone(), dirty);
let vw = paint.viewport_width.max(1) as f32;
let vh = paint.viewport_height.max(1) as f32;
```

browser 同步下载字体后失效 CPU glyph cache 和 GPU atlas，避免新字体沿用旧位图。来源：`apps/browser/src/app_page_fonts.rs`。

```rust
self.glyph_cache.clear();
if let Some(gpu) = self.gpu_renderer.as_mut() {
    gpu.clear_glyph_atlas();
}
```

WebDriver 创建截图路径的字体与 glyph cache，接下来传给 CPU `render_full_scene`。来源：`apps/webdriver/src/session.rs`。

```rust
let font_loader = FontLoader::new();
let mut glyph_cache = GlyphCache::new(1024);
```

## Gotchas
- backend 共用 glyph/image 结构却有不同缓存和资源准备阶段；新 glyph variation、图片 source 或 draw_order 字段必须核对 CPU、GPU 与 IPC 转换，不只修改某个 renderer。 (`crates/render-foundation/src/primitive/mod.rs`, `crates/render-foundation/src/cpu/mod.rs`, `crates/render-foundation/src/gpu/renderer/mod.rs`, `crates/paint-convert/src/lib.rs`)
- GPU 场景判断与真实资源/设备限制分别决定能否绘制，CPU、GPU 同样接受 RenderPrimitives 并不构成像素 parity 证据。 (`crates/render-foundation/src/gpu/scene_support.rs`, `crates/render-foundation/src/gpu/renderer/mod.rs`, `crates/render-foundation/src/cpu/mod.rs`)

## Architecture
- 图元转换属于 paint-convert，物理字体与光栅算法属于本 crate；布局只消费共享 face 和度量协议，不应复制字节解析或 GPU 生命周期。 (`crates/paint-convert/src/lib.rs`, `crates/render-foundation/src/lib.rs`, `crates/layout-engine/src/font_resolution.rs`)
- DisplayList 携脏区但不保留上帧像素，BackingStoreManager 负责 front/back，RenderingThread 负责 worker glyph cache；局部重绘正确性需要三个边界共同配合。 (`crates/render-foundation/src/display_list.rs`, `crates/render-foundation/src/backing_store.rs`, `crates/render-foundation/src/rendering_thread.rs`, `apps/compositor/src/rasterize.rs`)

## Consumer Analysis
- browser 在 `append_webview_primitives` 和 transform helpers 换算共享图元到窗口 UI 坐标，页面 glyph 同时携字体/index/rotation；消费端裁剪和滚动不是重新布局。 (`apps/browser/src/app_render_primitives.rs`)
- compositor 的 `rasterize_into_back` 消费 DisplayList、FrameBuffer、GlyphCache/ImageCache，DPR 与局部区域显式换算；只有无图片时采用不携 ImageCache 的 RenderingThread。 (`apps/compositor/src/rasterize.rs`)
- WebDriver 将最近 PaintSnapshot 转换为 RenderPrimitives 再 CPU 截图，字体 loader 当前为空；截图 API 成功不保证系统/下载字体与 compositor 一致。 (`apps/webdriver/src/session.rs`)
- layout-engine 用共享 `resolve_font_faces` 保持 CSS 字体顺序，engine 文本度量按 generic/complex run 分 hmtx 与 shaping；默认估算回退不代表物理字体缺失可忽略。 (`crates/layout-engine/src/font_resolution.rs`, `crates/engine/src/text_metrics.rs`)
- Canvas 借 FontLoader 画真实文本，并把像素转 ImagePrimitive 供共享图像路径；Canvas 当前路径、bitmap 与页面绘制输出是不同层次。 (`crates/canvas/src/context/context_impl.rs`, `crates/engine/src/paint/painter/mod.rs`)

## Child Knowledge Nodes
- [字体加载、整形与字形缓存](fonts/index.md)：查加载、匹配、OpenType、度量和字形缓存。
- [GPU 管线、atlas、纹理与 surface](gpu.md)：查场景支持、atlas、资源索引与纹理交付。
- [CPU 光栅化、颜色与增量 backing store](cpu-and-backing.md)：查脏区、双缓冲、alpha 和 worker。
- [图像解码、缓存与资源身份](images.md)：查 codec、SVG 固有尺寸、资源键和 GC。
