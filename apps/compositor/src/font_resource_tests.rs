use std::sync::Arc;
use zero_paint_convert::fonts::PaintFonts;
use zero_protocol::{IpcColor, IpcFontPayload, IpcGlyph, PaintSnapshotParams};
use zero_render_foundation::{
    font::{FontLoader, GlyphCache},
    image_cache::ImageCache,
    rendering_thread::RenderingThread,
    surface::FrameBuffer,
};

#[test]
fn downloaded_font_snapshot_rasterizes_like_local_font_with_or_without_worker() {
    let bytes = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");
    let mut paint = PaintSnapshotParams {
        viewport_width: 80,
        viewport_height: 80,
        font_payloads: vec![IpcFontPayload {
            font_id: 42,
            face_index: 0,
            data: bytes.to_vec(),
        }],
        glyphs: vec![IpcGlyph {
            x: 10.0,
            y: 50.0,
            font_size: 40.0,
            glyph_id: 'X' as u32,
            font_glyph_index: None,
            source: None,
            font_id: 42,
            font_variation_id: None,
            color: IpcColor {
                r: 0,
                g: 128,
                b: 0,
                a: 255,
            },
            rotation: 0.0,
            synthetic_italic: false,
        }],
        ..Default::default()
    };
    // The first resource-bearing frame may have been coalesced away. The newest
    // complete snapshot alone must be sufficient to reconstruct the font.
    let mut imports = PaintFonts::new(Arc::new(FontLoader::new()));
    imports.update(&paint.font_payloads).unwrap();
    imports.remap(&mut paint);
    assert_ne!(paint.glyphs[0].font_id, 42);
    let primitives = zero_paint_convert::to_render_primitives(paint.clone());
    let render = |loader: &FontLoader, worker: Option<&RenderingThread>| {
        let mut frame = FrameBuffer::new(1, 1);
        crate::rasterize::rasterize_into_back(
            &paint,
            &primitives,
            loader,
            &mut GlyphCache::new(64),
            worker,
            &mut ImageCache::new(8, 1 << 20),
            &mut frame,
            false,
        );
        frame.data
    };
    let mut local = FontLoader::new();
    local.load_font(bytes).unwrap();
    let expected = render(&local, None);
    assert!(
        expected.as_chunks::<4>().0.iter().filter(|p| p[1] < 200).count() >= 1400,
        "missing fonts must not pass as two identical blank frames"
    );
    assert_eq!(render(&imports.loader, None), expected);
    let worker = RenderingThread::spawn(imports.loader.clone(), 64);
    assert_eq!(render(&imports.loader, Some(&worker)), expected);
}
