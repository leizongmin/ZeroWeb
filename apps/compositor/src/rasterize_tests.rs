//! compositor 光栅化单元测试（S3 脏区域）。

#[cfg(test)]
mod tests {
    use zero_protocol::paint_snapshot::{IpcRect, PaintSnapshotParams};
    use zero_render_foundation::font::{FontLoader, GlyphCache};
    use zero_render_foundation::image_cache::ImageCache;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
    use zero_render_foundation::surface::FrameBuffer;

    use crate::rasterize;

    fn red_fill_snapshot(w: u32, h: u32, dirty: Vec<IpcRect>) -> PaintSnapshotParams {
        PaintSnapshotParams {
            viewport_width: w,
            viewport_height: h,
            document_height: h as f32,
            fills: vec![zero_protocol::paint_snapshot::IpcFill {
                rect: IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: w as f32,
                    height: h as f32,
                },
                color: zero_protocol::paint_snapshot::IpcColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            }],
            dirty_rects: dirty,
            ..Default::default()
        }
    }

    #[test]
    fn physical_viewport_size_uses_device_scale_factor() {
        let mut paint = red_fill_snapshot(800, 600, Vec::new());
        paint.device_scale_factor = 1.5;

        assert_eq!(rasterize::physical_viewport_size(&paint), (1200, 900));
    }

    #[test]
    fn full_rasterization_uses_device_pixel_dimensions() {
        let mut paint = red_fill_snapshot(4, 3, Vec::new());
        paint.device_scale_factor = 2.0;
        let primitives = zero_paint_convert::to_render_primitives(paint.clone());
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        let mut back = FrameBuffer::new(1, 1);

        rasterize::rasterize_into_back(
            &paint,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            false,
        );

        assert_eq!((back.width, back.height), (8, 6));
        assert_eq!(&back.data[back.data.len() - 4..], &[255, 0, 0, 255]);
    }

    #[test]
    fn partial_dirty_preserves_pixels_outside_region() {
        let w = 20u32;
        let h = 10u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut back = FrameBuffer::new(w, h);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        back.clear(0, 0, 255, 255);

        let paint = red_fill_snapshot(
            w,
            h,
            vec![IpcRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }],
        );
        let primitives = zero_paint_convert::to_render_primitives(paint.clone());

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        rasterize::rasterize_into_back(
            &paint,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            true,
        );

        assert_eq!(back.data[0], 255, "top-left in dirty should be red");
        assert_eq!(back.data[1], 0);
        let outside = ((5 * w + 15) * 4) as usize;
        assert_eq!(back.data[outside], 0, "outside dirty should stay blue");
        assert_eq!(back.data[outside + 2], 255);
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }
    }

    #[test]
    fn full_dirty_overwrites_entire_buffer() {
        let w = 8u32;
        let h = 8u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut back = FrameBuffer::new(w, h);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        back.clear(0, 255, 0, 255);

        let paint = red_fill_snapshot(w, h, vec![]);
        let primitives = RenderPrimitives {
            fills: vec![FillPrimitive {
                rect: zero_render_foundation::geometry::Rect::new(0.0, 0.0, w as f32, h as f32),
                color: zero_render_foundation::color::Color::rgb(255, 0, 0),
            }],
            ..RenderPrimitives::new()
        };

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        rasterize::rasterize_into_back(
            &paint,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            false,
        );

        assert_eq!(back.data[0], 255);
        assert_eq!(back.data[1], 0);
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }
    }

    // ── T60：脏区共享归一（normalize_dirty_rects）─────────────────────────

    #[test]
    fn normalize_dirty_rects_expands_and_aligns_to_integer_pixels() {
        let out = rasterize::normalize_dirty_rects(&[(10.0, 10.0, 20.0, 20.0)], 800.0, 600.0);
        assert_eq!(out, vec![(9.0, 9.0, 22.0, 22.0)], "四边外扩 1px 墨迹余量");
    }

    #[test]
    fn normalize_dirty_rects_floor_ceil_aligns_fractional_rects() {
        // x0=floor(8.4-1)=7, x1=ceil(8.4+20.6+1)=30；y0=floor(7.4)=7, y1=ceil(25.8)=26。
        let out = rasterize::normalize_dirty_rects(&[(8.4, 8.4, 20.6, 16.4)], 800.0, 600.0);
        assert_eq!(out, vec![(7.0, 7.0, 23.0, 19.0)], "小数坐标对齐整数像素");
    }

    #[test]
    fn normalize_dirty_rects_clamps_to_viewport() {
        let out = rasterize::normalize_dirty_rects(&[(-5.0, -5.0, 10.0, 10.0)], 100.0, 100.0);
        assert_eq!(out, vec![(0.0, 0.0, 6.0, 6.0)], "负坐标钳位到视口原点");

        let out = rasterize::normalize_dirty_rects(&[(95.0, 95.0, 20.0, 20.0)], 100.0, 100.0);
        assert_eq!(out, vec![(94.0, 94.0, 6.0, 6.0)], "越界部分钳位到视口边界");
    }

    #[test]
    fn normalize_dirty_rects_filters_degenerate_and_outside() {
        assert!(rasterize::normalize_dirty_rects(&[(0.0, 0.0, 0.0, 10.0)], 100.0, 100.0).is_empty());
        assert!(rasterize::normalize_dirty_rects(&[(0.0, 0.0, 10.0, -1.0)], 100.0, 100.0).is_empty());
        // 完全在视口外的矩形过滤掉（钳位后退化）。
        assert!(rasterize::normalize_dirty_rects(&[(200.0, 200.0, 10.0, 10.0)], 100.0, 100.0).is_empty());
        let out = rasterize::normalize_dirty_rects(&[(10.0, 10.0, 20.0, 20.0), (0.0, 0.0, 5.0, 5.0)], 100.0, 100.0);
        assert_eq!(out.len(), 2, "有效矩形逐个保留");
    }
}
