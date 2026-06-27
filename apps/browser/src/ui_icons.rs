//! 浏览器 Chrome 图标：运行时以 `resvg` 光栅化 `assets/icons/*.svg`，
//! 经 glyph atlas 的 alpha 遮罩绘制（与文字相同的抗锯齿路径）。

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use tracing::warn;
use zero_render_foundation::color::Color;
use zero_render_foundation::font::GlyphBitmap;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;

/// 图标专用虚拟 font id（不对应真实 fontdue 字体）。
pub const ICON_FONT_ID: u32 = 0xFFFF_FFFE;

const ICON_BASE_CODEPOINT: u32 = 0xE000;

/// Chrome 工具栏图标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    ChevronLeft,
    ChevronRight,
    Refresh,
    Home,
    Close,
    ChevronUp,
    ChevronDown,
    Plus,
    MoreVertical,
    Star,
}

impl Icon {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Icon::ChevronLeft => include_bytes!("../assets/icons/chevron-left.svg"),
            Icon::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Icon::Refresh => include_bytes!("../assets/icons/refresh.svg"),
            Icon::Home => include_bytes!("../assets/icons/home.svg"),
            Icon::Close => include_bytes!("../assets/icons/close.svg"),
            Icon::ChevronUp => include_bytes!("../assets/icons/chevron-up.svg"),
            Icon::ChevronDown => include_bytes!("../assets/icons/chevron-down.svg"),
            Icon::Plus => include_bytes!("../assets/icons/plus.svg"),
            Icon::MoreVertical => include_bytes!("../assets/icons/more-vertical.svg"),
            Icon::Star => include_bytes!("../assets/icons/star.svg"),
        }
    }

    fn glyph_id(self) -> u32 {
        ICON_BASE_CODEPOINT + self as u32
    }

    fn as_char(self) -> char {
        char::from_u32(self.glyph_id()).unwrap_or('\0')
    }
}

/// 在 `(cx, cy)` 居中绘制图标。`size` 为物理像素边长。
pub fn render_icon(
    font_loader: &mut FontLoader,
    glyphs: &mut Vec<GlyphDraw>,
    icon: Icon,
    cx: f32,
    cy: f32,
    size: f32,
    color: Color,
) {
    if size <= 0.0 {
        return;
    }

    ensure_icon_bitmap(font_loader, icon, size);

    glyphs.push(GlyphDraw {
        ch: icon.as_char(),
        x: cx - size * 0.5,
        baseline_y: cy + size * 0.5,
        color,
        font_id: ICON_FONT_ID,
        font_size: size,
    });
}

fn ensure_icon_bitmap(font_loader: &mut FontLoader, icon: Icon, size_px: f32) {
    let glyph_id = icon.glyph_id();
    if font_loader.has_bitmap_glyph(ICON_FONT_ID, glyph_id, size_px) {
        return;
    }

    match rasterize_icon_svg(icon.svg_bytes(), size_px) {
        Ok(bitmap) => {
            font_loader.register_bitmap_glyph(ICON_FONT_ID, glyph_id, size_px, bitmap);
        }
        Err(err) => {
            warn!(?icon, %size_px, %err, "failed to rasterize chrome icon");
        }
    }
}

fn rasterize_icon_svg(svg: &[u8], size_px: f32) -> Result<GlyphBitmap, String> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg, &options).map_err(|e| e.to_string())?;

    let side = size_px.ceil().max(1.0) as u32;
    let mut pixmap = Pixmap::new(side, side).ok_or_else(|| "pixmap allocation failed".to_string())?;

    let view_w = tree.size().width();
    let view_h = tree.size().height();
    let scale = side as f32 / view_w.max(view_h);
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    let data: Vec<u8> = pixmap.pixels().iter().map(|px| px.alpha()).collect();

    Ok(GlyphBitmap {
        data,
        width: side as u16,
        height: side as u16,
        x_offset: 0,
        y_offset: 0,
        advance: size_px,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_all_icons() {
        let mut loader = FontLoader::new();
        for icon in [
            Icon::ChevronLeft,
            Icon::ChevronRight,
            Icon::Refresh,
            Icon::Home,
            Icon::Close,
            Icon::ChevronUp,
            Icon::ChevronDown,
            Icon::Plus,
            Icon::MoreVertical,
            Icon::Star,
        ] {
            render_icon(&mut loader, &mut Vec::new(), icon, 16.0, 16.0, 24.0, Color::BLACK);
            assert!(loader.has_bitmap_glyph(ICON_FONT_ID, icon.glyph_id(), 24.0));
        }
    }

    #[test]
    fn icon_bitmap_has_alpha_edges() {
        let bitmap = rasterize_icon_svg(Icon::Close.svg_bytes(), 24.0).expect("rasterize close");
        assert!(bitmap.width > 0 && bitmap.height > 0);
        let opaque = bitmap.data.iter().filter(|&&a| a > 200).count();
        let fringe = bitmap.data.iter().filter(|&&a| (1..200).contains(&a)).count();
        assert!(opaque > 0, "icon should have opaque pixels");
        assert!(fringe > 0, "icon should have antialiased fringe pixels");
    }
}
