//! compositor 光栅化单元测试（S3 脏区域）。

#[cfg(test)]
mod tests {
    use zero_protocol::paint_snapshot::{IpcRect, PaintSnapshotParams};
    use zero_render_foundation::font::{FontLoader, GlyphCache};
    use zero_render_foundation::image_cache::ImageCache;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
    use zero_render_foundation::surface::FrameBuffer;

    use crate::convert;
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
        let primitives = convert::to_render_primitives(&paint);

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
}
