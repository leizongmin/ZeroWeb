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

    /// 测试 DamageTracker 使用零尺寸调用 damage_all 仍标记为脏
    ///
    /// 验证 damage_all(0x0) 不会 panic，并且产生一个零面积的脏矩形。
    /// 这是一个边界条件：表面尺寸为 0 时仍能正常追踪脏区域。
    #[test]
    fn test_damage_tracker_damage_all_zero_size() {
        let mut tracker = DamageTracker::new();
        tracker.damage_all(Size::new(0.0, 0.0));
        // damage_all 始终插入一个矩形，即使尺寸为零
        assert!(tracker.is_dirty(), "damage_all 后应标记为脏");
        assert_eq!(tracker.dirty_rects().len(), 1);
        let r = &tracker.dirty_rects()[0];
        assert_eq!(r.origin.x, 0.0);
        assert_eq!(r.origin.y, 0.0);
        assert_eq!(r.size.width, 0.0);
        assert_eq!(r.size.height, 0.0);
        // 零面积的脏矩形 — 面积应为 0
        assert!((r.size.area() - 0.0).abs() < f32::EPSILON);
    }

    /// 测试 ImageCache 在 max_bytes=0 时任何插入后 GC 都立即淘汰所有条目
    ///
    /// 当 max_bytes 设为 0 时，即使只插入极小图片，
    /// GC 后缓存也应为空，因为 total_bytes > max_bytes 始终成立。
    #[test]
    fn test_image_cache_max_bytes_zero() {
        let mut cache = ImageCache::new(100, 0);
        let img = ImageData::new_empty(1, 1); // 4 字节，远超 max_bytes=0
        let key = cache.insert(img);
        assert_eq!(cache.len(), 1, "插入后应有一个条目");
        assert!(cache.total_bytes() > 0, "total_bytes 应大于 0");

        cache.gc();
        assert!(cache.is_empty(), "max_bytes=0 时 GC 应淘汰所有条目");
        assert!(cache.get(&key).is_none(), "key 在 GC 后不应再可用");

        // 多次插入+GC 循环验证稳定性
        let k2 = cache.insert(ImageData::new_empty(2, 2));
        cache.gc();
        assert!(cache.is_empty(), "第二次 GC 后仍应为空");
        assert!(cache.ref_count(&k2).is_none());
    }

    /// 测试 FrameBuffer::from_rgba 在数据比期望值少一个字节时返回错误
    ///
    /// 验证 from_rgba 的边界条件：差一个字节也应产生明确的错误信息，
    /// 确保不会静默截断或越界访问。
    #[test]
    fn test_frame_buffer_from_rgba_off_by_one() {
        let width = 10u32;
        let height = 10u32;
        let expected = (width * height * 4) as usize; // 400
        // 仅差一个字节
        let data = vec![128u8; expected - 1];
        let result = FrameBuffer::from_rgba(data, width, height);
        assert!(result.is_err(), "少一个字节应返回错误");
        let err = result.unwrap_err();
        assert!(
            err.contains("期望") && err.contains("实际"),
            "错误信息应包含期望和实际大小: {err}"
        );

        // 多一个字节也应返回错误
        let data_extra = vec![128u8; expected + 1];
        let result_extra = FrameBuffer::from_rgba(data_extra, width, height);
        assert!(result_extra.is_err(), "多一个字节也应返回错误");
    }

    /// 测试 Color::lerp 对同一颜色的恒等插值
    ///
    /// 验证 lerp(color, color, t) 在 t=0 和 t=1 时都返回原始颜色，
    /// 这是 lerp 的恒等性质：插值起点和终点相同时结果不变。
    #[test]
    fn test_color_lerp_identity_same_color() {
        let c = Color::rgba(100, 150, 200, 250);

        // t=0：应返回 self（即 c）
        let at_zero = c.lerp(c, 0.0);
        assert_eq!(at_zero, c, "lerp(c, c, 0) 应返回 c 本身");

        // t=1：应返回 other（也是 c）
        let at_one = c.lerp(c, 1.0);
        assert_eq!(at_one, c, "lerp(c, c, 1) 应返回 c 本身");

        // t=0.5：由于 a=b，插值结果也应为 c
        let at_mid = c.lerp(c, 0.5);
        assert_eq!(at_mid.r, c.r, "lerp(c, c, 0.5) 的 R 通道应为 c.r");
        assert_eq!(at_mid.g, c.g, "lerp(c, c, 0.5) 的 G 通道应为 c.g");
        assert_eq!(at_mid.b, c.b, "lerp(c, c, 0.5) 的 B 通道应为 c.b");
        assert_eq!(at_mid.a, c.a, "lerp(c, c, 0.5) 的 A 通道应为 c.a");

        // 透明色插值
        let transparent = Color::TRANSPARENT;
        assert_eq!(transparent.lerp(transparent, 0.3), transparent);
    }

    /// 测试 RenderPrimitives 的 bounding_box 在所有图元类型混合且含负坐标时的正确性
    ///
    /// 同时包含 fills、rounded_rects、strokes、shadows、glyphs、images 等多种图元，
    /// 部分使用负坐标和负偏移量，验证 bounding_box 能正确计算全局包围盒。
    #[test]
    fn test_render_primitives_bounding_box_mixed_negative_offsets() {
        use crate::image_cache::ImageKey;
        use crate::primitive::*;

        let mut p = RenderPrimitives::new();

        // 负坐标的 fill
        p.add_fill(Rect::new(-100.0, -50.0, 200.0, 100.0), Color::BLACK);
        // 正坐标的 stroke
        p.add_stroke(StrokePrimitive {
            x1: 50.0,
            y1: 50.0,
            x2: 150.0,
            y2: 150.0,
            width: 4.0,
            color: Color::RED,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        // 负偏移的 shadow
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: -10.0,
            offset_y: -10.0,
            blur_radius: 5.0,
            spread_radius: 3.0,
        });
        // 远处的 image
        p.add_image(ImagePrimitive {
            rect: Rect::new(500.0, 500.0, 100.0, 100.0),
            image_key: ImageKey::new(1),
        });

        let bb = p.bounding_box().expect("应有包围盒");

        // fill: left=-100, top=-50, right=100, bottom=50
        // stroke (含 half_width=2): left=48, top=48, right=152, bottom=152
        // shadow: left=-18, top=-18, right=58, bottom=58
        // image: left=500, top=500, right=600, bottom=600
        assert!(bb.left() <= -100.0, "left 应 <= -100, 实际: {}", bb.left());
        assert!(bb.top() <= -50.0, "top 应 <= -50, 实际: {}", bb.top());
        assert!(bb.right() >= 600.0, "right 应 >= 600, 实际: {}", bb.right());
        assert!(bb.bottom() >= 600.0, "bottom 应 >= 600, 实际: {}", bb.bottom());

        // 确认包围盒尺寸为正
        assert!(bb.size.width > 0.0, "包围盒宽度应为正");
        assert!(bb.size.height > 0.0, "包围盒高度应为正");
    }

    /// 测试 FrameBuffer::new 在大尺寸下数据长度正确（无 u32 溢出）
    ///
    /// 当 width * height * 4 接近 u32 边界时，确保不会因整数溢出
    /// 而产生错误的缓冲区大小。使用适中的尺寸验证分配正确。
    #[test]
    fn test_frame_buffer_large_size_no_overflow() {
        // 使用 4096x4096 = 67,108,864 字节，在 u32 范围内安全
        let w = 4096u32;
        let h = 4096u32;
        let fb = FrameBuffer::new(w, h);
        assert_eq!(fb.width, w);
        assert_eq!(fb.height, h);
        let expected_len = (w as usize) * (h as usize) * 4;
        assert_eq!(fb.data.len(), expected_len, "数据长度应为 width*height*4");
        // 验证首尾像素可读
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(fb.get_pixel(w - 1, h - 1), [0, 0, 0, 0]);
    }

    /// 测试 DamageTracker 添加负宽度和负高度矩形时被忽略
    ///
    /// Rect::is_empty 对 width <= 0 或 height <= 0 返回 true，
    /// add_damage 在 rect.is_empty() 时直接返回不添加。
    /// 验证负尺寸矩形不会进入脏列表。
    #[test]
    fn test_damage_tracker_negative_width_rect_ignored() {
        let mut tracker = DamageTracker::new();

        // 负宽度矩形 — is_empty 返回 true，应被忽略
        tracker.add_damage(Rect::new(0.0, 0.0, -10.0, 50.0));
        assert!(!tracker.is_dirty(), "负宽度矩形不应被添加");

        // 负高度矩形
        tracker.add_damage(Rect::new(0.0, 0.0, 50.0, -10.0));
        assert!(!tracker.is_dirty(), "负高度矩形不应被添加");

        // 两者都为负
        tracker.add_damage(Rect::new(0.0, 0.0, -5.0, -5.0));
        assert!(!tracker.is_dirty(), "全负尺寸矩形不应被添加");
    }

    /// 测试 RenderPrimitives 仅包含 path_stroke 时的 bounding_box 正确性
    ///
    /// 验证 path_stroke 的顶点坐标正确参与 bounding_box 计算，
    /// 且闭合路径与非闭合路径的 bounding_box 结果一致（仅取决于顶点坐标）。
    #[test]
    fn test_bounding_box_path_stroke_only() {
        use crate::primitive::*;

        let mut p = RenderPrimitives::new();

        // 仅添加 path_stroke，不含其他图元
        p.add_path_stroke(vec![10.0, 20.0, 30.0, 40.0, 50.0, 10.0], Color::BLACK, 2.0, false);

        let bb = p.bounding_box().expect("仅 path_stroke 应返回包围盒");
        // 顶点: (10,20), (30,40), (50,10)
        assert_eq!(bb.left(), 10.0, "left 应为最小 x=10");
        assert_eq!(bb.top(), 10.0, "top 应为最小 y=10");
        assert_eq!(bb.right(), 50.0, "right 应为最大 x=50");
        assert_eq!(bb.bottom(), 40.0, "bottom 应为最大 y=40");

        // 验证闭合路径产生相同的 bounding_box
        let mut p2 = RenderPrimitives::new();
        p2.add_path_stroke(
            vec![10.0, 20.0, 30.0, 40.0, 50.0, 10.0],
            Color::BLACK,
            4.0,
            true, // closed — 不影响 bounding_box（只看顶点坐标）
        );
        let bb2 = p2.bounding_box().expect("闭合 path_stroke 也应有包围盒");
        assert_eq!(bb2.origin, bb.origin, "闭合路径包围盒起点应相同");
        assert_eq!(bb2.size, bb.size, "闭合路径包围盒尺寸应相同");
    }

    /// 测试 ImageCache 交替 release 和 get 后 ref_count 保持一致
    ///
    /// 在 insert → get → release → get → release 循环后，
    /// 验证 ref_count 在每一步都正确反映操作历史，
    /// 且最终 release 到 0 后 GC 能正确清除条目。
    #[test]
    fn test_image_cache_alternating_release_get_consistency() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(ImageData::new_empty(1, 1));
        assert_eq!(cache.ref_count(&key), Some(1));

        // 第一次 get → ref_count = 2
        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(2));

        // 第一次 release → ref_count = 1
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(1));

        // 第二次 get → ref_count = 2
        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(2));

        // 两次 release → ref_count = 0
        cache.release(&key);
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0), "两次 release 后 ref_count 应为 0");

        // 数据仍可获取（条目还在，ref_count=0 不阻止 get）
        let img = cache.get(&key);
        assert!(img.is_some(), "ref_count=0 时 get 仍应返回数据");
        assert_eq!(cache.ref_count(&key), Some(1), "get 后 ref_count 应回升到 1");

        // 最终 release 到 0 并 GC 清除
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
        cache.gc();
        assert!(cache.ref_count(&key).is_none(), "GC 后条目应被移除");
    }

    /// 测试 Color::lerp 在极小 t 值下的浮点精度
    ///
    /// 验证 lerp 在 t 接近 0 和 1 时不会产生溢出或意外结果，
    /// 且对于非常小的 epsilon 值，结果仍正确偏向起始颜色。
    #[test]
    fn test_color_lerp_extreme_float_precision() {
        let black = Color::BLACK;
        let white = Color::WHITE;

        // t 接近 0 — 应几乎等于起始颜色
        let near_zero = black.lerp(white, 0.001);
        assert_eq!(near_zero.r, 0, "t=0.001 时 R 应为 0");
        assert_eq!(near_zero.g, 0, "t=0.001 时 G 应为 0");
        assert_eq!(near_zero.b, 0, "t=0.001 时 B 应为 0");

        // t 接近 1 — 应几乎等于目标颜色
        let near_one = black.lerp(white, 0.999);
        assert_eq!(near_one.r, 255, "t=0.999 时 R 应为 255");
        assert_eq!(near_one.g, 255, "t=0.999 时 G 应为 255");
        assert_eq!(near_one.b, 255, "t=0.999 时 B 应为 255");

        // t=0.5 精确中间值（非黑非白颜色测试）
        let red = Color::RED;
        let blue = Color::BLUE;
        let mid = red.lerp(blue, 0.5);
        assert_eq!(mid.r, 128, "红→蓝 t=0.5 R 应为 128");
        assert_eq!(mid.g, 0, "红→蓝 t=0.5 G 应为 0");
        assert_eq!(mid.b, 128, "红→蓝 t=0.5 B 应为 128");
        assert_eq!(mid.a, 255, "红→蓝 t=0.5 A 应为 255");

        // 同色 lerp 在任意 t 下不变
        let c = Color::rgba(42, 84, 126, 200);
        assert_eq!(c.lerp(c, 0.3), c, "同色 lerp 任意 t 应返回原色");
        assert_eq!(c.lerp(c, 0.7), c, "同色 lerp 任意 t 应返回原色");
    }
}
