//! GPU 渲染后端 — 基于 wgpu 的渲染管线
//!
//! 迁移自 OmniTerm 的 wgpu 渲染器架构，提供：
//! - GPU 上下文管理（设备、队列、表面）
//! - Glyph Atlas（R8Unorm 纹理图集）
//! - 统一渲染管线（填充矩形 + 文本 Glyph）
//! - WGSL 着色器

pub mod atlas;
pub mod pipeline;
pub mod renderer;

pub use atlas::GlyphAtlas;
pub use renderer::GpuRenderer;
