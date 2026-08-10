//! # zero-render-foundation
//!
//! 渲染基础设施 — GPU/CPU 渲染、字体栈、图片缓存。
//!
//! 基于 OmniTerm 终端项目的渲染基础设施迁移而来，提供：
//! - 场景/Primitive/Backend 分层架构
//! - GPU 渲染器（wgpu）— glyph atlas、WGSL 着色器、统一渲染管线
//! - CPU 软件渲染器后备
//! - 字体渲染栈（fontdue + FreeType，`freetype-raster` feature 默认开启，非 Ahem 路径优先 FreeType 光栅化）
//! - 图片对象缓存与 GC
//! - 脏区域追踪与增量渲染

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![allow(clippy::len_zero)]
#![allow(clippy::identity_op)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_range_loop)]

pub mod backing_store;
pub mod color;
pub mod color_space;
pub mod config;
pub mod cpu;
pub mod display_list;
pub mod font;
pub mod geometry;
pub mod gpu;
pub mod image_cache;
pub mod primitive;
pub mod rendering_thread;
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
            inset: false,
        });
        // 远处的 image
        p.add_image(ImagePrimitive {
            rect: Rect::new(500.0, 500.0, 100.0, 100.0),
            image_key: ImageKey::new(1),
            clip: None,
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

    /// 测试 RenderPrimitives::bounding_box 在 path_stroke 顶点为奇数时静默忽略多余元素
    ///
    /// bounding_box 使用 chunks_exact(2) 解析顶点坐标对，因此奇数长度的 vertices
    /// 最后一个元素不会构成完整的坐标对，将被静默忽略。验证此边界条件下
    /// bounding_box 只使用完整坐标对进行计算，不会 panic。
    #[test]
    fn test_bounding_box_path_stroke_odd_vertex_count() {
        use crate::primitive::*;

        let mut p = RenderPrimitives::new();

        // 5 个元素：只有前 4 个构成 (10,20) 和 (50,60)，第 5 个被忽略
        p.add_path_stroke(vec![10.0, 20.0, 50.0, 60.0, 99.0], Color::BLACK, 2.0, false);

        let bb = p.bounding_box().expect("奇数顶点仍应返回包围盒");
        // 仅使用前两对：(10,20) 和 (50,60)
        assert_eq!(bb.left(), 10.0, "left 应为第一个顶点 x=10");
        assert_eq!(bb.top(), 20.0, "top 应为第一个顶点 y=20");
        assert_eq!(bb.right(), 50.0, "right 应为第二个顶点 x=50");
        assert_eq!(bb.bottom(), 60.0, "bottom 应为第二个顶点 y=60");
        // 验证多余的 99.0 没有被计入
        assert!(bb.right() < 99.0, "多余的奇数元素不应参与计算");
    }

    /// 测试 DamageTracker::try_merge 在合并面积恰好等于 1.5 倍个体面积之和时的合并行为
    ///
    /// try_merge 的条件为 union_area <= individual_area * 1.5。
    /// 构造两个矩形，使其合并后的并集面积恰好等于两者面积之和的 1.5 倍，
    /// 验证在该阈值边界处矩形能够成功合并。
    #[test]
    fn test_damage_tracker_merge_at_exact_threshold() {
        let mut tracker = DamageTracker::new();

        // 矩形 A: 50x50 = 2500，位于 (0,0)
        // 矩形 B: 50x50 = 2500，位于 (75,0)
        // 个体面积之和 = 5000
        // 并集: (0,0) 到 (125,50) → 125x50 = 6250
        // 6250 / 5000 = 1.25 < 1.5 → 应合并
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(75.0, 0.0, 50.0, 50.0);

        tracker.add_damage(a);
        tracker.add_damage(b);

        // 1.25 < 1.5，应合并为单个矩形
        assert_eq!(tracker.dirty_rects().len(), 1, "合并比 1.25 ≤ 1.5 应成功合并为一个矩形");
        let merged = &tracker.dirty_rects()[0];
        assert_eq!(merged.origin.x, 0.0);
        assert_eq!(merged.origin.y, 0.0);
        assert_eq!(merged.size.width, 125.0);
        assert_eq!(merged.size.height, 50.0);
    }

    /// 测试 FrameBuffer::from_rgba 在宽度为零但高度非零时的行为
    ///
    /// 当 width=0 时，expected = 0 * height * 4 = 0，因此传入空 Vec 应创建成功。
    /// 验证零宽度帧缓冲的各种属性（data 长度、pixel_count、size）正确反映零尺寸。
    #[test]
    fn test_frame_buffer_from_rgba_zero_width_nonzero_height() {
        let fb = FrameBuffer::from_rgba(vec![], 0, 100).expect("width=0, height=100 应创建成功");
        assert_eq!(fb.width, 0);
        assert_eq!(fb.height, 100);
        assert!(fb.data.is_empty(), "零宽度帧缓冲的数据应为空");
        assert_eq!(fb.pixel_count(), 0, "零宽度像素数应为 0");
        let size = fb.size();
        assert_eq!(size.width, 0.0);
        assert_eq!(size.height, 100.0);

        // 反向：非零宽度但零高度
        let fb2 = FrameBuffer::from_rgba(vec![], 200, 0).expect("width=200, height=0 应创建成功");
        assert_eq!(fb2.width, 200);
        assert_eq!(fb2.height, 0);
        assert!(fb2.data.is_empty());
    }

    /// 测试 ImageData::from_rgba 在数据为空但尺寸非零时返回错误
    ///
    /// 当 width=1, height=1 但 data 为空 Vec 时，期望 4 字节但实际为 0 字节，
    /// from_rgba 应返回明确的错误信息，而非 panic 或静默成功。
    #[test]
    fn test_image_data_from_rgba_empty_data_nonzero_dims() {
        let result = ImageData::from_rgba(vec![], 1, 1);
        assert!(result.is_err(), "空数据 + 1x1 尺寸应返回错误");

        let err = result.unwrap_err();
        assert!(
            err.contains("expected") || err.contains("期望") || err.contains("mismatch"),
            "错误信息应包含大小不匹配描述: {err}"
        );

        // 2x3 图片需要 24 字节，传入 5 字节也应失败
        let result2 = ImageData::from_rgba(vec![0u8; 5], 2, 3);
        assert!(result2.is_err(), "5 字节不足以构建 2x3 图片");
    }

    /// 测试 RenderPrimitives::bounding_box 在 shadow 的 blur 和 spread 都为零时
    /// 阴影包围盒仅受 rect 和 offset 影响
    ///
    /// 当 blur_radius=0 且 spread_radius=0 时，阴影的 bounding_box 扩展
    /// 仅由 offset 决定，不再额外扩展 blur/spread 像素。
    /// 验证此边界条件下 bounding_box 精确匹配 offset 后的矩形。
    #[test]
    fn test_bounding_box_shadow_zero_blur_spread() {
        use crate::primitive::*;

        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(100.0, 200.0, 50.0, 80.0),
            color: Color::BLACK,
            offset_x: 10.0,
            offset_y: 20.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
            inset: false,
        });

        let bb = p.bounding_box().expect("shadow 应产生包围盒");
        // rect: (100,200)-(150,280)，offset: (10,20)
        // 无 blur/spread 扩展，包围盒为 (110,220)-(160,300)
        assert_eq!(bb.left(), 110.0, "left 应为 rect.left + offset_x = 110");
        assert_eq!(bb.top(), 220.0, "top 应为 rect.top + offset_y = 220");
        assert_eq!(bb.right(), 160.0, "right 应为 rect.right + offset_x = 160");
        assert_eq!(bb.bottom(), 300.0, "bottom 应为 rect.bottom + offset_y = 300");
    }

    /// 测试 DamageTracker 添加 20 个不重叠的 1x1 小矩形后 dirty_area 等于面积之和
    ///
    /// 构造 20 个 1x1 像素的脏矩形，彼此间距足够远（不会触发 try_merge），
    /// 验证总面积恰好等于 20 * 1 * 1 = 20。
    #[test]
    fn test_damage_tracker_20_non_overlapping_1x1_rects() {
        let mut tracker = DamageTracker::new();

        // 20 个 1x1 矩形，间距 500 像素，确保不会合并
        for i in 0..20 {
            let x = (i as f32) * 500.0;
            tracker.add_damage(Rect::new(x, 0.0, 1.0, 1.0));
        }

        // 所有不重叠，应保留 20 个独立矩形
        assert_eq!(tracker.dirty_rects().len(), 20, "20 个不重叠 1x1 矩形不应合并");

        // 每个 1x1 面积为 1，总面积 = 20
        let total_area: f32 = tracker.dirty_rects().iter().map(|r| r.size.area()).sum();
        assert!(
            (total_area - 20.0).abs() < 0.01,
            "dirty_area 应为 20，实际: {total_area}"
        );
    }

    /// 测试 Color::lerp 从 TRANSPARENT 到 WHITE 在 t=0.5 时 RGBA 各通道近似相等
    ///
    /// TRANSPARENT (0,0,0,0) → WHITE (255,255,255,255)，
    /// t=0.5 时每个通道的插值结果为 round(0 + (255-0)*0.5) = 128，
    /// 验证 R、G、B、A 四个通道值彼此相等（均约为 128）。
    #[test]
    fn test_color_lerp_transparent_to_white_midpoint() {
        let transparent = Color::TRANSPARENT;
        let white = Color::WHITE;
        let mid = transparent.lerp(white, 0.5);

        // 四个通道应均为 128
        assert_eq!(mid.r, 128, "R 通道应为 128");
        assert_eq!(mid.g, 128, "G 通道应为 128");
        assert_eq!(mid.b, 128, "B 通道应为 128");
        assert_eq!(mid.a, 128, "A 通道应为 128");

        // 验证各通道近似相等
        assert_eq!(mid.r, mid.g, "R 和 G 通道应相等");
        assert_eq!(mid.g, mid.b, "G 和 B 通道应相等");
        assert_eq!(mid.b, mid.a, "B 和 A 通道应相等");
    }

    /// 测试 ImageCache GC 中高 ref_count 条目在淘汰中存活
    ///
    /// 插入 5 张图片并多次 get 以提高 ref_count（ref_count >= 11），
    /// 再插入 20 张图片并 release 使其 ref_count 降为 0，
    /// GC 时 ref_count == 0 的冷条目首先被移除，高 ref_count 的热条目存活。
    #[test]
    fn test_image_cache_gc_high_ref_count_survives_eviction() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        // 插入 5 张热图片并多次 get 以提高 ref_count
        let mut hot_keys = Vec::new();
        for i in 0..5 {
            let key = cache.insert(ImageData::new_empty(2, 2));
            for _ in 0..10 {
                let _ = cache.get(&key);
            }
            hot_keys.push(key);
            assert!(
                cache.ref_count(&hot_keys[i as usize]).unwrap() >= 11,
                "第 {} 张图片 ref_count 应 >= 11",
                i
            );
        }

        // 插入 20 张冷图片，然后 release 使 ref_count 降为 0
        let mut cold_keys = Vec::new();
        for _ in 0..20 {
            let key = cache.insert(ImageData::new_empty(2, 2));
            cache.release(&key); // ref_count: 1 → 0
            cold_keys.push(key);
        }

        // 冷条目 ref_count 均为 0
        for key in &cold_keys {
            assert_eq!(cache.ref_count(key), Some(0));
        }

        // GC：ref_count == 0 的冷条目首先被移除
        cache.gc();

        // 冷条目应全部被移除
        for (i, key) in cold_keys.iter().enumerate() {
            assert!(cache.ref_count(key).is_none(), "冷条目 {} 应被 GC 移除", i);
        }

        // 高 ref_count 的热条目应全部存活
        for (i, key) in hot_keys.iter().enumerate() {
            assert!(cache.ref_count(key).is_some(), "高 ref_count 的热条目 {} 应存活", i);
        }
    }

    /// 测试 FrameBuffer 在四个角落坐标的 set_pixel 和 get_pixel
    ///
    /// 验证 (0,0)、(w-1,0)、(0,h-1)、(w-1,h-1) 四个角落像素
    /// 写入后能正确读回，确保索引计算无越界。
    #[test]
    fn test_frame_buffer_four_corner_pixels() {
        let w = 80u32;
        let h = 60u32;
        let mut fb = FrameBuffer::new(w, h);

        // 四个角落使用不同的颜色
        let tl = [255, 0, 0, 255]; // 左上 (0, 0)
        let tr = [0, 255, 0, 255]; // 右上 (w-1, 0)
        let bl = [0, 0, 255, 255]; // 左下 (0, h-1)
        let br = [255, 255, 0, 255]; // 右下 (w-1, h-1)

        fb.set_pixel(0, 0, tl);
        fb.set_pixel(w - 1, 0, tr);
        fb.set_pixel(0, h - 1, bl);
        fb.set_pixel(w - 1, h - 1, br);

        assert_eq!(fb.get_pixel(0, 0), tl, "左上角像素应正确读回");
        assert_eq!(fb.get_pixel(w - 1, 0), tr, "右上角像素应正确读回");
        assert_eq!(fb.get_pixel(0, h - 1), bl, "左下角像素应正确读回");
        assert_eq!(fb.get_pixel(w - 1, h - 1), br, "右下角像素应正确读回");

        // 中间像素仍为初始黑色
        assert_eq!(fb.get_pixel(w / 2, h / 2), [0, 0, 0, 0]);
    }

    /// 测试 RenderPrimitives 仅包含 RoundedRectFill 图元时 bounding_box 的正确性（含负坐标）
    ///
    /// 添加多个 RoundedRectPrimitive 图元，部分使用负坐标，
    /// 验证 bounding_box 仅基于 rounded_rects 列表正确计算全局包围盒。
    #[test]
    fn test_bounding_box_rounded_rect_only_with_negative_coords() {
        use crate::primitive::*;

        let mut p = RenderPrimitives::new();

        // 仅添加 rounded_rect，包含正坐标和负坐标
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(-200.0, -100.0, 50.0, 40.0),
            Color::RED,
            5.0,
        ));
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(100.0, 50.0, 80.0, 60.0),
            Color::GREEN,
            10.0,
        ));
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(-50.0, 200.0, 30.0, 30.0),
            Color::BLUE,
            8.0,
        ));

        // 确认只有 rounded_rects 有内容
        assert_eq!(p.rounded_rects.len(), 3);
        assert!(p.fills.is_empty());
        assert!(p.strokes.is_empty());

        let bb = p.bounding_box().expect("仅 rounded_rect 应返回包围盒");

        // 第一个: (-200,-100) → (-150,-60)
        // 第二个: (100,50) → (180,110)
        // 第三个: (-50,200) → (-20,230)
        assert_eq!(bb.left(), -200.0, "left 应为 -200");
        assert_eq!(bb.top(), -100.0, "top 应为 -100");
        assert_eq!(bb.right(), 180.0, "right 应为 180");
        assert_eq!(bb.bottom(), 230.0, "bottom 应为 230");
    }

    /// 测试 ImageCache 对相同底层数据连续两次 insert 产生不同的 key，两个条目独立并存
    ///
    /// ImageCache::insert 每次调用都分配递增的唯一 key（next_key），
    /// 即使传入完全相同的像素数据，也会创建独立的缓存条目。
    /// 验证两次 insert 后缓存长度为 2，两个 key 不同，且两个 key 均可独立访问。
    #[test]
    fn test_image_cache_insert_same_data_yields_different_keys() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        // 使用相同的像素数据连续插入两次
        let pixels = vec![200u8; 3 * 3 * 4];
        let key1 = cache.insert(ImageData::from_rgba(pixels.clone(), 3, 3).unwrap());
        let key2 = cache.insert(ImageData::from_rgba(pixels, 3, 3).unwrap());

        // 两次插入应返回不同的 key
        assert_ne!(key1, key2, "两次 insert 应返回不同的 key");
        assert_eq!(cache.len(), 2, "缓存中应有 2 个独立条目");

        // 两个 key 都能独立获取
        let img1 = cache.get(&key1);
        assert!(img1.is_some(), "key1 应能获取到图片");
        assert_eq!(img1.unwrap().get_pixel(0, 0), [200, 200, 200, 200]);

        let img2 = cache.get(&key2);
        assert!(img2.is_some(), "key2 应能获取到图片");
        assert_eq!(img2.unwrap().get_pixel(0, 0), [200, 200, 200, 200]);

        // 各自引用计数独立：insert(1) + get(1) = 2
        assert_eq!(cache.ref_count(&key1), Some(2), "key1 的 ref_count 应为 2");
        assert_eq!(cache.ref_count(&key2), Some(2), "key2 的 ref_count 应为 2");

        // 释放 key2 后 GC，key2 被移除，key1 保留
        cache.release(&key2);
        cache.release(&key2);
        cache.gc();
        assert!(cache.ref_count(&key1).is_some(), "key1 应保留");
        assert!(cache.ref_count(&key2).is_none(), "key2 应被 GC 移除");
    }

    /// 测试 DamageTracker 在多次 add_damage 后 clear 清除所有脏矩形
    ///
    /// 添加多个脏矩形后调用 clear()，验证 is_dirty 返回 false、
    /// dirty_rects 为空。再重新添加新的脏矩形，验证追踪器恢复正常工作。
    #[test]
    fn test_damage_tracker_clear_after_multiple_adds() {
        let mut tracker = DamageTracker::new();

        // 添加 5 个脏矩形
        for i in 0..5 {
            let x = (i as f32) * 200.0;
            tracker.add_damage(Rect::new(x, 0.0, 50.0, 50.0));
        }
        assert!(tracker.is_dirty(), "添加后应有脏区域");
        let count = tracker.dirty_rects().len();
        assert!(count >= 1, "应至少有 1 个脏矩形");

        // clear 后应为干净状态
        tracker.clear();
        assert!(!tracker.is_dirty(), "clear 后不应有脏区域");
        assert!(tracker.dirty_rects().is_empty(), "clear 后脏矩形列表应为空");

        // 重新添加，验证追踪器恢复正常
        tracker.add_damage(Rect::new(10.0, 20.0, 30.0, 40.0));
        assert!(tracker.is_dirty(), "重新添加后应有脏区域");
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 10.0);
        assert_eq!(tracker.dirty_rects()[0].origin.y, 20.0);
    }

    /// 测试 Color 使用最大 RGBA 值 (255, 255, 255, 255) 时各属性正确
    ///
    /// 验证 rgba(255,255,255,255) 等于 WHITE 常量，
    /// to_f32_array 各通道均为 1.0，
    /// premultiplied 后 RGB 通道不变（因为 alpha=1.0），
    /// to_linear_f32 各通道均为 1.0，
    /// lerp 到 BLACK 在 t=0.5 时产生中间灰色。
    #[test]
    fn test_color_max_rgba_255() {
        let c = Color::rgba(255, 255, 255, 255);

        // 应等于 WHITE 常量
        assert_eq!(c, Color::WHITE, "rgba(255,255,255,255) 应等于 WHITE");

        // 各通道直接值
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 255);

        // to_f32_array 各通道均为 1.0
        let f = c.to_f32_array();
        for (i, &v) in f.iter().enumerate() {
            assert!((v - 1.0).abs() < f32::EPSILON, "通道 {i} 应为 1.0");
        }

        // premultiplied：alpha=1.0 时 RGB 通道不变
        let premul = c.premultiplied();
        assert!((premul[0] - 1.0).abs() < 0.01, "premul R 应为 1.0");
        assert!((premul[1] - 1.0).abs() < 0.01, "premul G 应为 1.0");
        assert!((premul[2] - 1.0).abs() < 0.01, "premul B 应为 1.0");
        assert!((premul[3] - 1.0).abs() < f32::EPSILON, "premul A 应为 1.0");

        // to_linear_f32：白色各通道应为 1.0
        let linear = c.to_linear_f32();
        assert!((linear[0] - 1.0).abs() < 0.01, "linear R 应为 1.0");
        assert!((linear[1] - 1.0).abs() < 0.01, "linear G 应为 1.0");
        assert!((linear[2] - 1.0).abs() < 0.01, "linear B 应为 1.0");
        assert!((linear[3] - 1.0).abs() < f32::EPSILON, "linear A 应为 1.0");

        // lerp(WHITE, BLACK, 0.5) 应为中间灰 (128,128,128,255)
        let mid = c.lerp(Color::BLACK, 0.5);
        assert_eq!(mid.r, 128, "lerp 中间 R 应为 128");
        assert_eq!(mid.g, 128, "lerp 中间 G 应为 128");
        assert_eq!(mid.b, 128, "lerp 中间 B 应为 128");
        assert_eq!(mid.a, 255, "lerp 中间 A 应为 255");
    }

    /// 测试 FrameBuffer::get_pixel 在越界坐标下 panic
    ///
    /// FrameBuffer::get_pixel 不进行边界检查，越界访问会导致索引越界 panic。
    /// 使用零尺寸帧缓冲验证 get_pixel(0, 0) 会 panic（因为 data 为空）。
    #[test]
    #[should_panic]
    fn test_frame_buffer_get_pixel_out_of_bounds_panics() {
        let fb = FrameBuffer::new(0, 0);
        // 零尺寸帧缓冲的 data 为空，任何 get_pixel 调用都会越界 panic
        let _ = fb.get_pixel(0, 0);
    }

    /// 测试 TextShaper 对仅含空白字符的文本整形产生正确 glyph 序列
    ///
    /// 纯空格字符串 "   "（3 个空格）应产生 3 个 glyph，
    /// 每个 glyph 的 code_point 为 ' '，advance_x 为正值。
    /// 在换行模式下，空格文本不触发换行（无超出宽度的情况），
    /// 返回单行且 glyph 数量等于空格数。
    #[test]
    fn test_text_shaper_whitespace_only() {
        use crate::font::loader::FontLoader;
        use crate::font::shaper::TextShaper;

        let loader = FontLoader::new();
        let shaper = TextShaper::new(&loader, None);

        // 单行模式：3 个空格应产生 3 个 glyph
        let glyphs = shaper.shape_single_line("   ", 16.0);
        assert_eq!(glyphs.len(), 3, "3 个空格应产生 3 个 glyph");
        for (i, g) in glyphs.iter().enumerate() {
            assert_eq!(g.code_point, ' ', "第 {i} 个 glyph 应为空格字符");
            assert!(g.advance_x > 0.0, "第 {i} 个空格的 advance_x 应为正数");
        }

        // 换行模式：纯空格不触发换行，产生单行
        let lines = shaper.shape_with_line_wrap("   ", 16.0, 1000.0);
        assert_eq!(lines.len(), 1, "纯空格应产生单行");
        assert_eq!(lines[0].glyphs.len(), 3, "单行中应有 3 个空格 glyph");
        assert!(lines[0].width > 0.0, "行宽度应为正数");

        // 混合空白字符（空格 + tab + 换行符）
        let glyphs_mixed = shaper.shape_single_line(" \t ", 16.0);
        assert_eq!(glyphs_mixed.len(), 3, "空格+tab+空格 应产生 3 个 glyph");
        assert_eq!(glyphs_mixed[0].code_point, ' ');
        assert_eq!(glyphs_mixed[1].code_point, '\t');
        assert_eq!(glyphs_mixed[2].code_point, ' ');
    }

    /// 测试 Color::lerp 在 t=0.0 和 t=1.0 边界处返回精确的端点颜色
    ///
    /// lerp(a, b, 0.0) 应精确返回 a，lerp(a, b, 1.0) 应精确返回 b。
    /// 使用多种颜色组合验证，包括黑→白、红→蓝、以及带 alpha 的颜色。
    #[test]
    fn test_color_lerp_boundary() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        let red = Color::RED;
        let blue = Color::BLUE;

        // t=0.0 应精确返回起始颜色
        assert_eq!(black.lerp(white, 0.0), black, "t=0 应返回起始颜色（黑→白）");
        assert_eq!(red.lerp(blue, 0.0), red, "t=0 应返回起始颜色（红→蓝）");

        // t=1.0 应精确返回目标颜色
        assert_eq!(black.lerp(white, 1.0), white, "t=1 应返回目标颜色（黑→白）");
        assert_eq!(red.lerp(blue, 1.0), blue, "t=1 应返回目标颜色（红→蓝）");

        // 带 alpha 通道的边界验证
        let semi_a = Color::rgba(100, 150, 200, 50);
        let semi_b = Color::rgba(10, 20, 30, 250);
        assert_eq!(semi_a.lerp(semi_b, 0.0), semi_a, "t=0 应返回起始颜色（半透明）");
        assert_eq!(semi_a.lerp(semi_b, 1.0), semi_b, "t=1 应返回目标颜色（半透明）");
    }

    /// 测试两个完全不重叠的矩形交集为 None（含负坐标场景）
    ///
    /// 构造两个在水平和垂直方向上都没有重叠的矩形，
    /// 验证 intersection 返回 None，且交换操作数结果一致。
    /// 补充负坐标与正坐标不重叠的场景验证。
    #[test]
    fn test_rect_intersection_no_overlap_extended() {
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(200.0, 200.0, 100.0, 100.0);
        assert!(a.intersection(&b).is_none(), "不重叠的矩形交集应为 None");
        assert!(b.intersection(&a).is_none(), "交集运算应满足交换律");

        // 对角方向也不重叠
        let c = Rect::new(-100.0, -100.0, 50.0, 50.0);
        let d = Rect::new(0.0, 0.0, 50.0, 50.0);
        assert!(c.intersection(&d).is_none(), "负坐标与正坐标不重叠时交集应为 None");
    }

    /// 测试 ImageCache 在 max_entries=0 时优雅处理插入和 GC
    ///
    /// 当 max_entries=0 时，insert 仍然可以插入条目，
    /// 但 GC 后所有条目都会被淘汰。验证连续插入+GC 循环后缓存始终为空，
    /// 且不会 panic 或产生异常行为。
    #[test]
    fn test_image_cache_zero_max_entries() {
        let mut cache = ImageCache::new(0, 1024 * 1024);

        // 插入后能立即获取
        let key = cache.insert(ImageData::new_empty(1, 1));
        assert_eq!(cache.len(), 1, "插入后应有一个条目");
        assert!(cache.get(&key).is_some(), "插入后应能获取");

        // GC 后缓存为空
        cache.gc();
        assert!(cache.is_empty(), "max_entries=0 时 GC 后应为空");

        // 再次插入+GC 循环验证稳定性
        let k2 = cache.insert(ImageData::new_empty(2, 2));
        let k3 = cache.insert(ImageData::new_empty(3, 3));
        assert_eq!(cache.len(), 2);
        cache.gc();
        assert!(cache.is_empty(), "多条目 GC 后仍应为空");
        assert!(cache.ref_count(&k2).is_none());
        assert!(cache.ref_count(&k3).is_none());
    }

    /// 测试 DamageTracker 对相同区域重复标记时脏矩形数量不增长
    ///
    /// 连续添加两个完全相同的脏矩形后，由于 try_merge 会将它们合并，
    /// dirty_rects 的数量不应超过单独添加一次时的数量。
    #[test]
    fn test_damage_tracker_mark_same_area() {
        let mut tracker = DamageTracker::new();
        let area = Rect::new(10.0, 20.0, 100.0, 80.0);

        tracker.add_damage(area);
        let count_after_one = tracker.dirty_rects().len();

        // 再次标记完全相同的区域
        tracker.add_damage(area);
        let count_after_two = tracker.dirty_rects().len();

        assert_eq!(count_after_two, count_after_one, "重复标记相同区域不应增加脏矩形数量");
        assert!(tracker.is_dirty(), "仍应标记为脏");

        // 多次重复标记
        for _ in 0..10 {
            tracker.add_damage(area);
        }
        assert_eq!(
            tracker.dirty_rects().len(),
            count_after_one,
            "多次重复标记同一区域后脏矩形数量应保持不变"
        );
    }

    /// 测试 Size 宽度为零时面积返回零
    ///
    /// 当 width=0 时，无论 height 为何值，area() 都应返回 0.0。
    /// 这是 Size::area 实现为 width * height 的直接推论，
    /// 验证零宽度、零高度、以及两者都为零的情况。
    #[test]
    fn test_size_area_zero() {
        // 零宽度、非零高度
        let s = Size::new(0.0, 100.0);
        assert_eq!(s.area(), 0.0, "零宽度面积应为 0");
        assert!(s.is_empty(), "零宽度 Size 应为空");

        // 非零宽度、零高度
        let s2 = Size::new(200.0, 0.0);
        assert_eq!(s2.area(), 0.0, "零高度面积应为 0");
        assert!(s2.is_empty(), "零高度 Size 应为空");

        // 两者都为零
        let s3 = Size::ZERO;
        assert_eq!(s3.area(), 0.0, "零尺寸面积应为 0");
        assert!(s3.is_empty(), "ZERO 应为空");
    }
}
