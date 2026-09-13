//! 下载字体随完整绘制帧发送，避免任一 latest-wins 队列丢弃资源注册。

use std::collections::{BTreeSet, HashSet};
use zero_protocol::IpcFontPayload;
use zero_render_foundation::{font::FontLoader, primitive::RenderPrimitives};

pub(crate) fn for_primitives(
    loader: &FontLoader,
    downloaded: &HashSet<u32>,
    primitives: &RenderPrimitives,
) -> Result<Vec<IpcFontPayload>, String> {
    let ids: BTreeSet<u32> = primitives
        .glyphs
        .iter()
        .map(|g| g.font_id.0)
        .filter(|id| downloaded.contains(id))
        .collect();
    let mut payloads = Vec::new();
    if ids.len() > zero_protocol::MAX_PAINT_FONTS {
        return Err("too many paint fonts".into());
    }
    let mut total = 0usize;
    for id in ids {
        let data = loader.get_font_data(id).ok_or("missing downloaded font")?;
        total = total.saturating_add(data.len());
        if data.len() > zero_protocol::MAX_PAINT_FONT_BYTES || total > zero_protocol::MAX_PAINT_FONTS_BYTES {
            return Err("paint font byte budget exceeded".into());
        }
        payloads.push(IpcFontPayload {
            font_id: id,
            face_index: loader.face_index(id),
            data: data.to_vec(),
        });
    }
    Ok(payloads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::{
        color::Color,
        primitive::{FontId, GlyphPrimitive},
    };

    #[test]
    fn every_snapshot_repeats_only_referenced_downloaded_fonts() {
        let bytes = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");
        let mut loader = FontLoader::new();
        let system = loader.load_font(bytes).unwrap();
        let downloaded = loader.load_font(bytes).unwrap();
        let unused = loader.load_font(bytes).unwrap();
        let glyph = |id| GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 40.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 'X' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(id),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        };
        let primitives = RenderPrimitives {
            glyphs: vec![glyph(system), glyph(downloaded), glyph(downloaded)],
            ..RenderPrimitives::new()
        };
        let owned = HashSet::from([downloaded, unused]);
        let first = for_primitives(&loader, &owned, &primitives).unwrap();
        let latest = for_primitives(&loader, &owned, &primitives).unwrap();
        assert_eq!(first, latest);
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].font_id, downloaded);
        assert_eq!(latest[0].data, bytes);
        assert!(
            for_primitives(&loader, &owned, &RenderPrimitives::new())
                .unwrap()
                .is_empty()
        );
    }
}
