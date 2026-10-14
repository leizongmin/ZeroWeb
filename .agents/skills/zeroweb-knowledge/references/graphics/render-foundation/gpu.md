# GPU 管线、atlas、纹理与 surface

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 wgpu `GpuRenderer` 的生产场景、WGSL/pipeline、glyph atlas、图片纹理缓存和 Linux 纹理导入导出。窗口和 headless 都须沿 `render_full_scene_gpu` 定位；角色回退调度、FD 交付和最终呈现归 runtime。

## Gotchas
- `scene_supported` 的开头旧说明仍列 clips、模糊阴影和滤镜未支持，但当前函数只拒绝“有 blend 且 draw_order 为空”；不能按旧说明删掉已实现的 GPU 分支。 (`crates/render-foundation/src/gpu/scene_support.rs`, `crates/render-foundation/src/gpu/renderer/mod.rs`)
- `render_full_scene_gpu` 在 present suspended、场景不支持或图片超过 adapter 最大纹理尺寸时返回 false；GPU 后端初始化成功不保证每帧能由 GPU 画完。 (`crates/render-foundation/src/gpu/renderer/mod.rs`)
- 图片资源准备结果保留每个输入图元的槽位，缺失缓存或零尺寸填 None；不得压缩结果列表，否则 draw_order 图片索引会指向其他图片。 (`crates/render-foundation/src/gpu/renderer/mod.rs`, `git:25975e2e3da42bb8b3972cd762cd6cc7fffca9ce`)
- glyph atlas key 区分码点和 face 内索引并包含 variation；页面字体 namespace 换代时必须清 atlas，单独清 CPU `GlyphCache` 不够。 (`crates/render-foundation/src/gpu/atlas.rs`, `apps/compositor/src/font_resources.rs`)
- GPU 图片按 `Rgba8Unorm` 上传已经着色的字节，不能换成 sRGB texture 再解码，否则半透明图片变暗。 (`crates/render-foundation/src/gpu/renderer/mod.rs`)
- Linux `try_export_headless` 当前 Vulkan OPAQUE_FD 分支恒返回待实现错误，实际回退 `read_pixels` 加 memfd；“GPU dma-buf 导出”名称不证明零回读或零拷贝。 (`crates/render-foundation/src/gpu/texture_export.rs`)

## Architecture
- 生产 GPU 按非空 draw_order 绘制并合批连续同类图元，空顺序退类型分桶；UI 和 overlay 仍作为独立最后层，不能为合批跨越页面绘制顺序。 (`crates/render-foundation/src/gpu/renderer/mod.rs`)
- blend 段各自清空 GPU source texture 后与 backdrop 合成；当前代码注明 CPU 的累积源缓冲在多段交叠时存在近似差异，不能用 scene_supported=true 当像素一致性证明。 (`crates/render-foundation/src/gpu/renderer/mod.rs`, `crates/render-foundation/src/cpu/mod.rs`)
- 2048×2048 R8 atlas 行式打包，满时清空并增加 generation；UV 半纹素内缩避免采到相邻 glyph，geometry 或缓存必须尊重 atlas 换代。 (`crates/render-foundation/src/gpu/atlas.rs`)
- 图片 GPU cache key 为资源键、尺寸、插入时内容摘要；256 MiB 预算按 last_used 逐出，8192 条硬上限清空，仅核算图片纹理而非整个 GPU 内存。 (`crates/render-foundation/src/gpu/renderer/mod.rs`)
- 图片 source 是归一化源子矩形，clip 在目标 rect 上裁剪并换算 UV；缩小 clip 不能重新把全图缩放进去。 (`crates/render-foundation/src/gpu/renderer/mod.rs`)

## Patterns
- Linux import 只接受 modifier=0 和指定 ABGR8888 fourcc，通过 mmap 后 `queue.write_texture` 上传；虽然省中间 Vec/ImageCache，仍是 CPU 映射到 GPU 上传而非原生共享纹理。 (`crates/render-foundation/src/gpu/texture_import.rs`)
- `RenderMode` 接受 auto/gpu/cpu，software/soft 是 CPU 别名，无效环境配置返回错误而不是静默 auto；实际优先级与产品选择还须查调用方。 (`crates/render-foundation/src/config.rs`)

## Testing Strategy
- headless 纹理导入测试在 GPU 初始化失败时直接返回；读取测试资产只能确认预期尺寸与格式，不能证明本机或窗口 GPU 生产帧已验收。 (`crates/render-foundation/src/gpu/texture_import.rs`)
