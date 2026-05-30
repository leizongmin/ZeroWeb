//! # zero-render-foundation
//!
//! 渲染基础设施 — GPU/CPU 渲染、字体栈、图片缓存。
//!
//! 基于 OmniTerm 终端项目的渲染基础设施迁移而来，提供：
//! - 场景/Primitive/Backend 分层架构
//! - GPU 渲染器（wgpu）— glyph atlas、WGSL 着色器、统一渲染管线
//! - CPU 软件渲染器后备
//! - 字体渲染栈（fontdue + swash）
//! - 图片对象缓存与 GC
//! - 脏区域追踪与增量渲染

#![warn(missing_docs)]

pub mod color;
pub mod font;
pub mod geometry;
pub mod gpu;
pub mod image_cache;
pub mod primitive;
pub mod surface;

/// 渲染错误类型
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// GPU 设备不可用
    #[error("GPU 设备不可用: {0}")]
    GpuUnavailable(String),
    /// 表面创建失败
    #[error("表面创建失败: {0}")]
    SurfaceCreationFailed(String),
    /// 字体加载失败
    #[error("字体加载失败: {0}")]
    FontLoadFailed(String),
    /// 渲染失败
    #[error("渲染失败: {0}")]
    RenderFailed(String),
    /// 缓冲区大小不匹配
    #[error("缓冲区大小不匹配: 期望 {expected}, 实际 {actual}")]
    BufferSizeMismatch {
        /// 期望大小
        expected: usize,
        /// 实际大小
        actual: usize,
    },
    /// 图片数据无效
    #[error("图片数据无效: {0}")]
    ImageInvalid(String),
}

/// 渲染结果
pub type RenderResult<T> = Result<T, RenderError>;
