# 图像解码、缓存与资源身份

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `decode_image_bytes`、`ImageData`、`ImageCache`、SVG 固有尺寸和 WebView 隔离解码分发。图片解码正常却不上屏、SVG 尺寸错误、同键更新像素或图片页内存增长时读这里。解码角色启动和断连机制归 runtime，本节点只保存图像契约。

## Gotchas
- 解码按字节魔数而非 URL 扩展名或传入 MIME 选择 PNG/JPEG/WebP/SVG/GIF；GIF 只解静态首帧，不能把此入口当动画播放器。 (`crates/render-foundation/src/image_cache.rs`, `apps/image-decoder/src/lib.rs`)
- SVG pixmap 尺寸不一定是布局固有尺寸：ratio-only、no-ratio 和一维绝对尺寸加 viewBox 分别有独立元数据，不能一律把 `width/height` 送进 aspect ratio。 (`crates/render-foundation/src/image_cache.rs`)
- `ImageData` 的内容摘要与纯色标志在构造时计算，GPU texture cache 复用摘要；直接改公开 pixels 而不重建数据会使缓存身份和纯色优化失真。 (`crates/render-foundation/src/image_cache.rs`, `crates/render-foundation/src/gpu/renderer/mod.rs`)
- ImageCache 插入 ref_count=1，get 再增、release 再减；GC 超 entries/bytes 预算仍能淘汰正引用项，引用计数不是永久保留保证。 (`crates/render-foundation/src/image_cache.rs`)
- `get_rasterized` 只有目标宽和高都大于固有尺寸且目标不超过 4,000,000 像素才矢量重栅格；八项缓存逐出 HashMap 任一条目，并非严格 FIFO。 (`crates/render-foundation/src/image_cache.rs`)
- 当前 `insert_with_key`、remove、clear 操作基础 entries，却不清同键 `svg_rasterized` 派生项；同键换 SVG 时不能假定派生尺寸缓存也已更新。 (`crates/render-foundation/src/image_cache.rs`)
- WebView 隔离开关一旦启用，栅格图片 spawn 失败或进程故障返回资源错误，不进程内兜底；`SPAWN_FAILED` 上方仍写“后续进程内解码”的旧注释，实际代码是 fail closed。 (`crates/webview/src/image_decoder.rs`)

## Architecture
- `image_resource_key` 先 trim src 再按 document URL 解析并哈希；异步 fetch、尺寸缓存和 painter 必须按相同解析串注入，保留 src 尾换行会导致像素和尺寸双 miss。 (`crates/engine/src/paint/helpers.rs`, `crates/webview/src/async_load.rs`, `crates/engine/src/paint/painter/text.rs`)
- `object-fit` 使用解码真实尺寸，缺失时才回 HTML width/height 与容器；目标 clip 与源尺寸不是同一概念，cover/contain 必须同时核对 rect 和 UV。 (`crates/engine/src/paint/painter/text.rs`, `crates/engine/src/paint/painter/text_image.rs`, `crates/render-foundation/src/gpu/renderer/mod.rs`)
- 隔离角色与 inline 都调用同一个 `decode_image_bytes`，MIME 只出现在错误上下文；解码错误返回零尺寸、空 RGBA 和 error，不等于进程崩溃。 (`apps/image-decoder/src/lib.rs`, `crates/webview/src/image_decoder.rs`)

## Patterns
- `ImageData::get_pixel` 对零尺寸或损坏缓冲返回透明，合法数据的越界坐标 clamp 到最近像素；这不同于 Canvas getImageData 画布外透明语义。 (`crates/render-foundation/src/image_cache.rs`, `crates/canvas/src/context/context_impl.rs`)
- `duplicate_for_snapshot` 克隆像素到独立 cache 并把引用/访问代数重置，`snapshot_entries` 也克隆 ImageData；这些接口不是 Arc 共享的零拷贝快照。 (`crates/render-foundation/src/image_cache.rs`)

## Testing Strategy
- IPC 解码测试构造四色 2×2 PNG 与 inline 逐像素比较，并测垃圾输入 error；它不覆盖全部 codec、SVG 固有尺寸或生产网络资源。 (`apps/image-decoder/tests/ipc_decode.rs`)
