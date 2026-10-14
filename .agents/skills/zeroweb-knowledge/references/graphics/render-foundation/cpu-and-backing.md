# CPU 光栅化、颜色与增量 backing store

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `render_full_scene`、region into、直 alpha、颜色插值、`DisplayList`、双缓冲和持久 raster worker。局部重绘残影、DPR 错位、透明滤镜白盒或渲染线程缓存失效时读这里；呈现/导航事务属于 runtime。

## Gotchas
- `DisplayList::is_full_viewport` 把空脏区视为全量，非空时按面积之和达到视口的 99% 判全量，不算矩形并集；重叠脏区可能更早触发全帧，空列表不是“无需画”。 (`crates/render-foundation/src/display_list.rs`)
- region-into 的 region 实际与乘 scale 后的坐标比较，compositor 先把 CSS 脏矩形乘 DPR；不能据 region 函数旧注释的 CSS 像素措辞再乘一次或漏乘。 (`crates/render-foundation/src/cpu/mod.rs`, `apps/compositor/src/rasterize.rs`)
- 局部重绘前必须保留旧 front 到 back；compositor 的 copy_front 标志只表示调用方已经复制，不在 `rasterize_into_back` 内复制，未复制则白底清除。 (`crates/render-foundation/src/backing_store.rs`, `apps/compositor/src/rasterize.rs`)
- `FrameBuffer::new` 实际全零含 alpha=0，尽管注释称全黑不透明；页面场景入口另用 `new_filled(...,255)` 建不透明背景，不能据构造器注释推断 alpha。 (`crates/render-foundation/src/surface.rs`, `crates/render-foundation/src/cpu/mod.rs`)
- `RenderingThread` job 不携 ImageCache，worker 总传 None；compositor 只有无 images 时采用持久 worker，含图帧回主路径避免丢图。 (`crates/render-foundation/src/rendering_thread.rs`, `apps/compositor/src/rasterize.rs`)
- `render_full_scene_straight_alpha` 用白/黑底双渲染解出直 alpha，因为普通 blend sink 按不透明底写 alpha；简单改透明 clear 会产生暗边或假 alpha。 (`crates/render-foundation/src/cpu/mod.rs`)

## Architecture
- CPU 默认消费非空 draw_order，`ZERO_DRAW_ORDER=0` 或空序列退类型分桶；旧分桶会破坏背景图与子内容关系，不能以其作为正确 painting order。 (`crates/render-foundation/src/cpu/mod.rs`)
- UI glyph 在 overlay 前且近似按 region 过滤；overlay fills/rounded rects/glyphs 始终最后、不过滤 region，用于滚动平移后恢复 chrome 静态层。 (`crates/render-foundation/src/cpu/mod.rs`)
- 双缓冲只有 swap 后 back 才成为 front，resize 重建两个 buffer；该 manager 不自己同步线程，调用方须保证 worker 不在持有旧 back。 (`crates/render-foundation/src/backing_store.rs`)
- `Color::lerp` 是逐 RGBA 通道插值，渐变 `interp_pair` 才按指定空间 premultiply、插值再 unpremultiply，极坐标 hue 不预乘；两种 API 在半透明输入下不等价。 (`crates/render-foundation/src/color.rs`, `crates/render-foundation/src/color_space.rs`)
- CPU `render_full_scene` 结尾调用 image_cache.gc，region-into 不替调用方执行同一回收；资源寿命须结合图像 cache retain/release 协议核对。 (`crates/render-foundation/src/cpu/mod.rs`, `crates/render-foundation/src/image_cache.rs`)

## Patterns
- worker 独占 GlyphCache 并持有固定 Arc loader，字体 namespace 改变时 compositor 重建 worker；提交 `rasterize_sync` 仍同步等待结果，不是异步呈现。 (`crates/render-foundation/src/rendering_thread.rs`, `apps/compositor/src/font_resources.rs`)
- worker Drop 先取走发送端再 join，让 recv 退出；反过来 join 会等待永不关闭的接收循环。 (`crates/render-foundation/src/rendering_thread.rs`)
- 普通生产渲染线程默认开、测试/headless helper 只在显式 true 时开；两套 helper 的默认值不同，不能由生产开关推断 oracle 线程模式。 (`crates/render-foundation/src/rendering_thread.rs`)
