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

    // ── TV-2：生产入口接线——rasterize_into_back 必须对 paint.dirty_rects 归一 ──

    /// T60/TV-2 接线：`rasterize_into_back`（CPU 区域渲染入口）对小数脏区先归一
    /// 再重绘。脏区 (8.0, 8.0, 20.6, 16.4) 归一为 [7,30)×[7,26)：(29,24) 在
    /// trunc 边界（[8,28)×[8,24)）之外、归一区域之内，若入口漏掉归一则该像素
    /// 保持旧帧蓝色——本测钉住「入口必须消费 normalize_dirty_rects」的接线。
    #[test]
    fn partial_dirty_fractional_rect_entry_wiring_repaints_normalized_region() {
        let w = 40u32;
        let h = 30u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut back = FrameBuffer::new(w, h);
        let mut image_cache = ImageCache::new(8, 1 << 20);
        back.clear(0, 0, 255, 255);

        let paint = red_fill_snapshot(
            w,
            h,
            vec![IpcRect {
                x: 8.0,
                y: 8.0,
                width: 20.6,
                height: 16.4,
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
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }

        let pixel = |x: u32, y: u32| -> (u8, u8, u8) {
            let off = ((y * w + x) * 4) as usize;
            (back.data[off], back.data[off + 1], back.data[off + 2])
        };
        // trunc 区域外、归一区域内的右/底边像素必须被重绘为红。
        assert_eq!(pixel(29, 24), (255, 0, 0), "ceil 边像素 (29,24) 应被归一后重绘");
        assert_eq!(pixel(7, 7), (255, 0, 0), "1px 墨迹外扩角点 (7,7) 应被重绘");
        // 归一区域外保留旧帧蓝。
        assert_eq!(pixel(5, 5), (0, 0, 255), "归一区域外应保留旧帧");
        assert_eq!(pixel(31, 27), (0, 0, 255), "归一区域外应保留旧帧");
    }

    // ── M3-c：半透明 partial 必须与全量逐位一致（src-over 二次混合回归）──

    /// M3-c 回归：partial 脏区重绘在旧帧上叠画半透明图元会二次 src-over 混合
    /// （rgba(255,0,0,0.5) 逐帧 (255,127,127)→(255,63,63)）。修复：copy_front
    /// 分支先按归一化脏区重铺白底再重绘。本测以相同图元模拟 paint-only 变更，
    /// 断言 partial 帧与全量帧逐位一致——修复前脏区内变深，测试失败。
    #[test]
    fn partial_dirty_semi_transparent_matches_full_render() {
        let w = 20u32;
        let h = 10u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);

        // 全视口半透明红 fill；paint-only 变更 = 图元不变、仅脏区收窄到局部。
        let paint_full = PaintSnapshotParams {
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
                    a: 128,
                },
            }],
            dirty_rects: Vec::new(),
            ..Default::default()
        };
        let primitives = zero_paint_convert::to_render_primitives(paint_full.clone());

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        // 帧 1：全量渲染（白底上一次混合）——既是上一帧，也是期望结果。
        let mut expected = FrameBuffer::new(w, h);
        rasterize::rasterize_into_back(
            &paint_full,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut expected,
            false,
        );

        // 帧 2：copy_front_to_back 后按局部脏区 partial 重绘。
        let mut back = FrameBuffer::new(w, h);
        back.data.copy_from_slice(&expected.data);
        let mut paint_partial = paint_full.clone();
        paint_partial.dirty_rects = vec![IpcRect {
            x: 5.0,
            y: 2.0,
            width: 8.0,
            height: 6.0,
        }];
        rasterize::rasterize_into_back(
            &paint_partial,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            true,
        );
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }

        // 半透明背景：白底一次混合应为 (255,127,127)；修复前脏区内二次混合为 (255,63,63)。
        assert_eq!(
            back.data, expected.data,
            "partial 重绘相同内容必须与全量逐位一致（脏区内不得二次混合）"
        );
        let px = |x: u32, y: u32| -> [u8; 3] {
            let off = ((y * w + x) * 4) as usize;
            [back.data[off], back.data[off + 1], back.data[off + 2]]
        };
        assert_eq!(
            px(8, 4),
            [255, 127, 127],
            "脏区内：应为白底一次混合值（修复前此处为二次混合 (255,63,63)）"
        );
        assert_eq!(px(0, 0), [255, 127, 127], "脏区外：应保留上一帧像素不变");
    }

    /// M3-c HiDPI 变体：device_scale_factor=2 下 partial 必须与全量逐位一致——
    /// 覆盖白底重铺边界（逻辑脏区 ×scale = 物理矩形）与 region 渲染的物理像素
    /// 契约（fill 裁剪此前按 CSS 逻辑与物理 region 直接求交，scale≠1 错位；
    /// 2 的幂 scale 换算逐位精确，可作逐位断言）。
    #[test]
    fn partial_dirty_semi_transparent_matches_full_at_scale_2() {
        let logical_w = 20u32;
        let logical_h = 10u32;
        let (w, h) = (40u32, 20u32); // 物理视口
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);

        let paint_full = PaintSnapshotParams {
            viewport_width: logical_w,
            viewport_height: logical_h,
            document_height: logical_h as f32,
            device_scale_factor: 2.0,
            fills: vec![zero_protocol::paint_snapshot::IpcFill {
                rect: IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: logical_w as f32,
                    height: logical_h as f32,
                },
                color: zero_protocol::paint_snapshot::IpcColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 128,
                },
            }],
            dirty_rects: Vec::new(),
            ..Default::default()
        };
        let primitives = zero_paint_convert::to_render_primitives(paint_full.clone());

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        let mut expected = FrameBuffer::new(w, h);
        rasterize::rasterize_into_back(
            &paint_full,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut expected,
            false,
        );

        let mut back = FrameBuffer::new(w, h);
        back.data.copy_from_slice(&expected.data);
        let mut paint_partial = paint_full.clone();
        paint_partial.dirty_rects = vec![IpcRect {
            x: 5.0,
            y: 2.0,
            width: 8.0,
            height: 6.0,
        }];
        rasterize::rasterize_into_back(
            &paint_partial,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            true,
        );
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }

        assert_eq!(
            back.data, expected.data,
            "scale=2 partial 重绘相同内容必须与全量逐位一致（重铺边界与 region 契约按物理像素对齐）"
        );
        let px = |x: u32, y: u32| -> [u8; 3] {
            let off = ((y * w + x) * 4) as usize;
            [back.data[off], back.data[off + 1], back.data[off + 2]]
        };
        // 物理 (16,8) = 逻辑 (8,4)，在归一化脏区的物理投影 [8,28)×[2,18) 内。
        assert_eq!(px(16, 8), [255, 127, 127], "脏区内（物理坐标）：白底一次混合值");
        assert_eq!(px(0, 0), [255, 127, 127], "脏区外（物理坐标）：保留上一帧");
    }

    /// M3-c 残留族收口：跨界图元（rounded_rect 无逐像素裁剪）在 partial 重绘时
    /// 整绘越界——直绘臂的区外写入叠上旧帧，半透明逐帧二次混合漂移（脏区边界
    /// 外圈 (255,127,127)→(255,63,63)）。scratch 白底渲染 + blit_region 精确列
    /// 拷回后越界写入被丢弃，partial==full 逐位一致。
    #[test]
    fn partial_dirty_straddling_rounded_rect_matches_full_render() {
        let w = 20u32;
        let h = 10u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);

        // 全视口半透明圆角矩形：与任何局部脏区都跨界（无裁剪整绘越界）。
        let paint_full = PaintSnapshotParams {
            viewport_width: w,
            viewport_height: h,
            document_height: h as f32,
            rounded_rects: vec![zero_protocol::paint_snapshot::IpcRoundedRect {
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
                    a: 128,
                },
                top_left_radius: 3.0,
                top_right_radius: 3.0,
                bottom_right_radius: 3.0,
                bottom_left_radius: 3.0,
            }],
            dirty_rects: Vec::new(),
            ..Default::default()
        };
        let primitives = zero_paint_convert::to_render_primitives(paint_full.clone());

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        // 帧 1：全量渲染（白底一次混合）——既是上一帧，也是期望结果。
        let mut expected = FrameBuffer::new(w, h);
        rasterize::rasterize_into_back(
            &paint_full,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut expected,
            false,
        );

        // 帧 2：copy_front_to_back 后按局部脏区 partial 重绘。
        let mut back = FrameBuffer::new(w, h);
        back.data.copy_from_slice(&expected.data);
        let mut paint_partial = paint_full.clone();
        paint_partial.dirty_rects = vec![IpcRect {
            x: 5.0,
            y: 2.0,
            width: 8.0,
            height: 6.0,
        }];
        rasterize::rasterize_into_back(
            &paint_partial,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            true,
        );
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }

        assert_eq!(
            back.data, expected.data,
            "partial 重绘跨界图元必须与全量逐位一致（越界写入不得叠上区外旧帧）"
        );
        let px = |x: u32, y: u32| -> [u8; 3] {
            let off = ((y * w + x) * 4) as usize;
            [back.data[off], back.data[off + 1], back.data[off + 2]]
        };
        // 归一化脏区 [4,14)×[1,9) 外的右下角：修前直绘越界二次混合为 (255,63,63)。
        assert_eq!(
            px(17, 9),
            [255, 127, 127],
            "脏区外（跨界图元投影带内）：必须保留上一帧一次混合值"
        );
        assert_eq!(px(8, 4), [255, 127, 127], "脏区内：白底一次混合值");
    }

    /// M3-c 多脏区串区防护：scratch 跨脏区复用时，前一脏区渲染的越界残留不得
    /// 落入后一脏区的拷回列——无逐像素裁剪的全视口圆角矩形在脏区 A 渲染时把墨迹
    /// 铺满整个 scratch（含脏区 B 的列），若不复铺白底，脏区 B 渲染再叠加即二次
    /// 混合（(255,127,127)→(255,63,63)）。逐脏区重铺白底后 partial==full 逐位一致。
    #[test]
    fn partial_dirty_multi_rect_straddling_matches_full_render() {
        let w = 20u32;
        let h = 10u32;
        let loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let mut image_cache = ImageCache::new(8, 1 << 20);

        // 全视口半透明圆角矩形：与任何局部脏区都跨界（无裁剪整绘越界）。
        let paint_full = PaintSnapshotParams {
            viewport_width: w,
            viewport_height: h,
            document_height: h as f32,
            rounded_rects: vec![zero_protocol::paint_snapshot::IpcRoundedRect {
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
                    a: 128,
                },
                top_left_radius: 3.0,
                top_right_radius: 3.0,
                bottom_right_radius: 3.0,
                bottom_left_radius: 3.0,
            }],
            dirty_rects: Vec::new(),
            ..Default::default()
        };
        let primitives = zero_paint_convert::to_render_primitives(paint_full.clone());

        unsafe {
            std::env::set_var("ZW_RENDER_THREAD", "0");
        }
        // 帧 1：全量渲染（白底一次混合）——既是上一帧，也是期望结果。
        let mut expected = FrameBuffer::new(w, h);
        rasterize::rasterize_into_back(
            &paint_full,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut expected,
            false,
        );

        // 帧 2：copy_front_to_back 后按两个不相交脏区 partial 重绘——脏区 A 先渲染，
        // 其越界墨迹覆盖脏区 B 的 scratch 列，恰构成串区向量。
        // A (1,1,6,4) → 归一化 [0,8)×[0,6)；B (11,5,6,4) → 归一化 [10,18)×[4,10)。
        let mut back = FrameBuffer::new(w, h);
        back.data.copy_from_slice(&expected.data);
        let mut paint_partial = paint_full.clone();
        paint_partial.dirty_rects = vec![
            IpcRect {
                x: 1.0,
                y: 1.0,
                width: 6.0,
                height: 4.0,
            },
            IpcRect {
                x: 11.0,
                y: 5.0,
                width: 6.0,
                height: 4.0,
            },
        ];
        rasterize::rasterize_into_back(
            &paint_partial,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &mut image_cache,
            &mut back,
            true,
        );
        unsafe {
            std::env::remove_var("ZW_RENDER_THREAD");
        }

        assert_eq!(
            back.data, expected.data,
            "多脏区 partial 重绘必须与全量逐位一致（后一脏区不得混入前一脏区的 scratch 残留）"
        );
        let px = |x: u32, y: u32| -> [u8; 3] {
            let off = ((y * w + x) * 4) as usize;
            [back.data[off], back.data[off + 1], back.data[off + 2]]
        };
        // 脏区 B 内：若串区未防，此处为 A 残留 + B 墨迹二次混合 (255,63,63)。
        assert_eq!(
            px(14, 7),
            [255, 127, 127],
            "后一脏区内：必须为白底一次混合值（不得含前一脏区残留）"
        );
        assert_eq!(px(4, 3), [255, 127, 127], "前一脏区内：白底一次混合值");
        // 两脏区外且在圆角矩形墨迹带内（(19,0) 落在半径 3 圆角削去区，不取）。
        assert_eq!(px(19, 7), [255, 127, 127], "两脏区外：保留上一帧像素不变");
    }
}
