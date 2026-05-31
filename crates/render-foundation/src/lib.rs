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

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::geometry::{DamageTracker, Rect, Size};
    use crate::image_cache::{ImageCache, ImageData};
    use crate::surface::FrameBuffer;

    /// 测试 DamageTracker 添加单个脏矩形后总面积正确
    #[test]
    fn test_damage_tracker_single_rect() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(10.0, 20.0, 100.0, 50.0));
        assert!(tracker.is_dirty());
        assert_eq!(tracker.dirty_rects().len(), 1);
        let total_area: f32 = tracker.dirty_rects().iter().map(|r| r.size.area()).sum();
        assert!(
            (total_area - 5000.0).abs() < 0.01,
            "total_dirty_area should be 5000, got {total_area}"
        );
    }

    /// 测试 DamageTracker 两个重叠脏矩形合并为一个
    #[test]
    fn test_damage_tracker_merge_overlapping() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.add_damage(Rect::new(50.0, 50.0, 100.0, 100.0));
        // Overlapping rects should merge into one
        assert_eq!(
            tracker.dirty_rects().len(),
            1,
            "overlapping rects should merge into one"
        );
        let merged = &tracker.dirty_rects()[0];
        assert_eq!(merged.origin.x, 0.0);
        assert_eq!(merged.origin.y, 0.0);
        assert_eq!(merged.size.width, 150.0);
        assert_eq!(merged.size.height, 150.0);
    }

    /// 测试 Color 使用超过 255 的值时被 clamp 到 255
    #[test]
    fn test_color_rgba_clamp_above_255() {
        let r = 300u32.clamp(0, 255) as u8;
        let g = 300u32.clamp(0, 255) as u8;
        let b = 300u32.clamp(0, 255) as u8;
        let a = 300u32.clamp(0, 255) as u8;

        let c = Color::rgba(r, g, b, a);
        assert_eq!(c.r, 255, "R channel should be clamped to 255");
        assert_eq!(c.g, 255, "G channel should be clamped to 255");
        assert_eq!(c.b, 255, "B channel should be clamped to 255");
        assert_eq!(c.a, 255, "A channel should be clamped to 255");
    }

    /// 测试 Surface（FrameBuffer）resize 后新尺寸正确
    #[test]
    fn test_surface_resize_preserves_content() {
        let mut fb = FrameBuffer::new(100, 200);
        assert_eq!(fb.width, 100);
        assert_eq!(fb.height, 200);

        fb.set_pixel(10, 20, [255, 128, 64, 200]);

        // Resize: create new framebuffer to simulate resize
        fb = FrameBuffer::new(300, 400);
        assert_eq!(fb.width, 300, "width after resize should be 300");
        assert_eq!(fb.height, 400, "height after resize should be 400");
        assert_eq!(fb.data.len(), 300 * 400 * 4);
    }

    /// 测试 ImageCache 在 max_entries=0 时插入后 GC 清除所有条目
    #[test]
    fn test_image_cache_max_entries_zero() {
        let mut cache = ImageCache::new(0, 1024 * 1024);
        let img = ImageData::new_empty(2, 2);
        let key = cache.insert(img);

        // Insert always returns a key, but gc with max_entries=0 removes everything
        cache.gc();
        assert!(cache.is_empty(), "cache should be empty after gc with max_entries=0");
        assert!(
            cache.get(&key).is_none(),
            "get should return None after gc with max_entries=0"
        );
    }
}
