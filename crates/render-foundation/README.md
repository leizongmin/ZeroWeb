# ZeroWeb Render Foundation (`zero-render-foundation`)

> GPU/CPU 渲染基础设施，提供字体渲染栈、Glyph Atlas 纹理图集、增量脏区域追踪与统一 WGSL 渲染管线。

## 概述

`ZeroWeb Render Foundation` (`zero-render-foundation`) 是 ZeroWeb 渲染管线的底层基石，源自 OmniTerm 终端项目的渲染架构迁移。它采用 Scene / Primitive / Backend 三层分离设计，基于 wgpu 提供 GPU 加速渲染（含无头模式后备），同时集成 fontdue + FreeType 字体渲染栈（`freetype-raster` feature 默认开启，非 Ahem 字形优先 FreeType 光栅化）和行式打包的 Glyph Atlas，为上层引擎提供高效的像素输出能力。

## 主要功能

- **GPU 渲染器（wgpu）** — 窗口模式直接合成到屏幕，无头模式渲染到纹理后回读像素，适用于测试与 CPU 后备
- **统一 WGSL 渲染管线** — 单管线同时处理填充矩形和 Glyph 文本渲染，通过 UV 标记区分图元类型
- **Glyph Atlas 纹理图集** — 2048x2048 R8Unorm 纹理，行式打包放置策略，满时自动清空重建，带半纹素内缩避免采样泄漏
- **字体渲染栈** — fontdue + FreeType 的字体加载与光栅化（`freetype-raster` feature 默认开启，非 Ahem 字形优先 FreeType，纯 Rust 构建时 `--no-default-features`），支持字体族查找、Glyph 位图缓存与淘汰
- **颜色系统** — RGBA 颜色表示，支持十六进制解析、sRGB 到线性空间转换、预乘 alpha
- **几何与脏区域追踪** — Point / Size / Rect 基础几何类型，DamageTracker 管理增量重绘区域，支持智能合并
- **帧缓冲** — CPU 侧 RGBA 像素数据管理，支持逐像素读写与批量清除

## 使用示例

```rust
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::{Rect, Size, DamageTracker};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_render_foundation::gpu::GpuRenderer;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::surface::SurfaceDescriptor;

// 创建无头 GPU 渲染器
let mut renderer = GpuRenderer::new_headless(800, 600)
    .expect("GPU 渲染器创建失败");

// 构建渲染图元
let mut primitives = RenderPrimitives::new();
primitives.add_fill(
    Rect::new(0.0, 0.0, 800.0, 600.0),
    Color::WHITE,
);

// 跟踪脏区域
let mut damage = DamageTracker::new();
damage.add_damage(Rect::new(0.0, 0.0, 800.0, 600.0));

// 加载字体并渲染场景
let mut font_loader = FontLoader::new();
let mut glyph_cache = GlyphCache::default();
// font_loader.load_font(&font_data)...
// renderer.render_scene(&primitives.fills, &font_loader, &mut glyph_cache, &glyphs);
```
