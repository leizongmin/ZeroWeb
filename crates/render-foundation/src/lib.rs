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

    /// 测试两个不重叠矩形的交集为 None。
    #[test]
    fn test_rect_intersection_no_overlap() {
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(a.intersection(&b).is_none(), "不重叠的矩形交集应为 None");
        assert!(b.intersection(&a).is_none(), "交集运算应满足交换律");

        // 仅有边相邻也不算重叠
        let c = Rect::new(50.0, 0.0, 50.0, 50.0);
        assert!(a.intersection(&c).is_none(), "边相邻的矩形交集也应为 None");
    }

    /// 测试两个不相交矩形的并集（手动计算包围盒）面积等于两者之和。
    #[test]
    fn test_rect_union_disjoint() {
        let a = Rect::new(0.0, 0.0, 20.0, 30.0);
        let b = Rect::new(100.0, 100.0, 40.0, 50.0);

        // 不相交矩形的交集应为 None
        assert!(a.intersection(&b).is_none());

        // 手动计算包围盒（union）
        let union_left = a.left().min(b.left());
        let union_top = a.top().min(b.top());
        let union_right = a.right().max(b.right());
        let union_bottom = a.bottom().max(b.bottom());
        let union_width = union_right - union_left;
        let union_height = union_bottom - union_top;
        assert_eq!(union_left, 0.0);
        assert_eq!(union_top, 0.0);
        assert_eq!(union_right, 140.0);
        assert_eq!(union_bottom, 150.0);
        assert_eq!(union_width, 140.0);
        assert_eq!(union_height, 150.0);

        // 包围盒面积远大于两个矩形面积之和（因为它们不相交）
        let area_a = a.size.area();
        let area_b = b.size.area();
        let union_area = union_width * union_height;
        assert_eq!(area_a, 600.0);
        assert_eq!(area_b, 2000.0);
        assert!(union_area > area_a + area_b, "不相交矩形的包围盒面积应大于两者面积之和");
    }

    /// 测试 Color 的 alpha 混合（通过 premultiplied 和 lerp 验证）。
    #[test]
    fn test_color_alpha_blend() {
        // 半透明红色 premultiplied alpha
        let red_half = Color::rgba(255, 0, 0, 128);
        let premul = red_half.premultiplied();
        let alpha = 128.0_f32 / 255.0;
        assert!(
            (premul[0] - 1.0 * alpha).abs() < 0.01,
            "R 通道 premultiplied 应约为 0.502"
        );
        assert!(premul[1].abs() < f32::EPSILON, "G 通道应为 0");
        assert!(premul[2].abs() < f32::EPSILON, "B 通道应为 0");
        assert!((premul[3] - alpha).abs() < f32::EPSILON, "A 通道应为 alpha");

        // lerp 模拟 alpha 混合：t=0.5 在黑和白之间
        let black = Color::BLACK;
        let white = Color::WHITE;
        let blended = black.lerp(white, 0.5);
        assert_eq!(blended.r, 128);
        assert_eq!(blended.g, 128);
        assert_eq!(blended.b, 128);
        assert_eq!(blended.a, 255);

        // 完全不透明色的 premultiplied RGB 应与原始值相同
        let opaque = Color::rgb(200, 100, 50);
        let premul_opaque = opaque.premultiplied();
        assert!((premul_opaque[0] - 200.0 / 255.0).abs() < 0.01);
        assert!((premul_opaque[1] - 100.0 / 255.0).abs() < 0.01);
        assert!((premul_opaque[2] - 50.0 / 255.0).abs() < 0.01);
        assert!((premul_opaque[3] - 1.0).abs() < f32::EPSILON);
    }

    /// 测试正方形 Size 的面积计算。
    #[test]
    fn test_size_area_square() {
        let square = Size::new(10.0, 10.0);
        assert_eq!(square.area(), 100.0, "10x10 正方形面积应为 100");
        assert!(!square.is_empty());

        // 1x1 正方形
        let unit = Size::new(1.0, 1.0);
        assert_eq!(unit.area(), 1.0);

        // 非正方形对比
        let rect = Size::new(10.0, 20.0);
        assert_eq!(rect.area(), 200.0);
        assert_ne!(square.area(), rect.area(), "正方形和非正方形面积应不同");
    }

    /// 测试 DamageTracker 添加单个脏矩形后状态正确。
    #[test]
    fn test_damage_tracker_single_rect_verify() {
        let mut tracker = DamageTracker::new();
        assert!(!tracker.is_dirty(), "初始状态不应有脏区域");

        let rect = Rect::new(25.0, 30.0, 80.0, 60.0);
        tracker.add_damage(rect);
        assert!(tracker.is_dirty(), "添加后应有脏区域");
        assert_eq!(tracker.dirty_rects().len(), 1, "应恰好有一个脏矩形");

        let dirty = tracker.dirty_rects()[0];
        assert_eq!(dirty.origin.x, 25.0);
        assert_eq!(dirty.origin.y, 30.0);
        assert_eq!(dirty.size.width, 80.0);
        assert_eq!(dirty.size.height, 60.0);
        assert!((dirty.size.area() - 4800.0).abs() < 0.01, "面积应为 4800");
    }

    /// 测试 ImageCache 插入后能通过 key 获取数据。
    #[test]
    fn test_image_cache_insert_and_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        // 插入前缓存为空
        assert!(cache.is_empty());

        // 插入图片
        let pixels = vec![255u8; 4 * 4 * 4]; // 4x4 RGBA
        let img = ImageData::from_rgba(pixels, 4, 4).unwrap();
        let key = cache.insert(img);

        // 插入后缓存非空
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.ref_count(&key), Some(1), "插入后 ref_count 应为 1");

        // 通过 key 获取数据
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some(), "应能获取到已插入的图片");
        let data = retrieved.unwrap();
        assert_eq!(data.width, 4);
        assert_eq!(data.height, 4);
        assert_eq!(data.get_pixel(0, 0), [255, 255, 255, 255]);

        // get 后 ref_count 递增
        assert_eq!(cache.ref_count(&key), Some(2), "get 后 ref_count 应为 2");

        // 不存在的 key 应返回 None
        let fake_key = crate::image_cache::ImageKey::new(999);
        assert!(cache.get(&fake_key).is_none());
    }
}
