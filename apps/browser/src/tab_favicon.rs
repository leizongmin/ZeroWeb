//! 标签页 favicon 获取与光栅化。

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use tracing::debug;
use zero_browser_shell::TabId;
use zero_render_foundation::color::Color;
use zero_render_foundation::font::GlyphBitmap;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;

/// favicon 专用虚拟 font id。
pub const FAVICON_FONT_ID: u32 = 0xFFFF_FFFD;

const DEFAULT_FAVICON_SVG: &[u8] = include_bytes!("../assets/icons/globe.svg");

pub fn clear_tab_favicon(font_loader: &mut FontLoader, tab_id: TabId, size_px: f32) {
    font_loader.clear_bitmap_glyph(FAVICON_FONT_ID, tab_id.0 as u32, size_px);
}

/// 为标签注册 favicon 并返回绘制用的 glyph 字符。
#[allow(clippy::too_many_arguments)]
pub fn ensure_tab_favicon(
    font_loader: &mut FontLoader,
    tab_id: TabId,
    page_url: Option<&str>,
    html: Option<&str>,
    size_px: f32,
) -> char {
    let glyph_id = tab_id.0 as u32;
    if font_loader.has_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px) {
        return favicon_char(glyph_id);
    }

    let bitmap = resolve_favicon_bitmap(page_url, html, size_px)
        .unwrap_or_else(|| rasterize_svg(DEFAULT_FAVICON_SVG, size_px).unwrap_or_else(default_bitmap));
    font_loader.register_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px, bitmap);
    favicon_char(glyph_id)
}

#[allow(clippy::too_many_arguments)]
pub fn render_tab_favicon(
    font_loader: &mut FontLoader,
    glyphs: &mut Vec<GlyphDraw>,
    tab_id: TabId,
    page_url: Option<&str>,
    html: Option<&str>,
    cx: f32,
    cy: f32,
    size_px: f32,
    color: Color,
) {
    let ch = ensure_tab_favicon(font_loader, tab_id, page_url, html, size_px);
    glyphs.push(GlyphDraw {
        ch,
        x: cx - size_px * 0.5,
        baseline_y: cy + size_px * 0.5,
        color,
        font_id: FAVICON_FONT_ID,
        font_size: size_px,
    });
}

fn favicon_char(glyph_id: u32) -> char {
    char::from_u32(0xF100 + (glyph_id & 0x7FF)).unwrap_or('\0')
}

fn resolve_favicon_bitmap(page_url: Option<&str>, html: Option<&str>, size_px: f32) -> Option<GlyphBitmap> {
    let page_url = page_url?;
    let favicon_url = pick_favicon_url(page_url, html)?;
    let bytes = fetch_bytes(&favicon_url)?;
    decode_icon_bytes(&bytes, size_px)
}

fn pick_favicon_url(page_url: &str, html: Option<&str>) -> Option<String> {
    if let Some(html) = html
        && let Some(href) = extract_link_icon_href(html)
        && let Some(resolved) = resolve_href(page_url, &href)
    {
        return Some(resolved);
    }
    resolve_href(page_url, "/favicon.ico")
}

fn extract_link_icon_href(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    for needle in [
        "rel=\"icon\"",
        "rel='icon'",
        "rel=\"shortcut icon\"",
        "rel='shortcut icon'",
    ] {
        if let Some(rel_pos) = lower.find(needle) {
            let snippet = &html[rel_pos..rel_pos.saturating_add(240).min(html.len())];
            if let Some(href) = extract_href_attribute(snippet) {
                return Some(href);
            }
        }
    }
    None
}

fn extract_href_attribute(snippet: &str) -> Option<String> {
    for pattern in ["href=\"", "href='"] {
        let start = snippet.find(pattern)? + pattern.len();
        let quote = pattern.chars().last()?;
        let rest = &snippet[start..];
        let end = rest.find(quote)?;
        let href = rest[..end].trim();
        if !href.is_empty() {
            return Some(href.to_string());
        }
    }
    None
}

fn resolve_href(base: &str, href: &str) -> Option<String> {
    if href.starts_with("data:") {
        return Some(href.to_string());
    }
    url::Url::parse(base)
        .ok()
        .and_then(|base_url| base_url.join(href).ok())
        .map(|u| u.to_string())
}

fn fetch_bytes(url: &str) -> Option<Vec<u8>> {
    if let Some(payload) = url.strip_prefix("data:image/svg+xml,") {
        return Some(payload.as_bytes().to_vec());
    }
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?
        .get(url)
        .send()
        .ok()?
        .bytes()
        .ok()
        .map(|b| b.to_vec())
}

fn decode_icon_bytes(bytes: &[u8], size_px: f32) -> Option<GlyphBitmap> {
    if bytes.starts_with(b"<") || bytes.starts_with(b"<?") || bytes.starts_with(b"<svg") {
        return rasterize_svg(bytes, size_px);
    }
    decode_png(bytes, size_px).or_else(|| {
        debug!("unsupported favicon format, using default");
        None
    })
}

fn decode_png(bytes: &[u8], size_px: f32) -> Option<GlyphBitmap> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let width = info.width as u32;
    let height = info.height as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let side = size_px.ceil().max(1.0) as u32;
    let mut alpha = vec![0u8; (side * side) as usize];
    let scale_x = width as f32 / side as f32;
    let scale_y = height as f32 / side as f32;
    for row in 0..side {
        for col in 0..side {
            let src_x = ((col as f32 + 0.5) * scale_x - 0.5)
                .round()
                .clamp(0.0, width as f32 - 1.0) as u32;
            let src_y = ((row as f32 + 0.5) * scale_y - 0.5)
                .round()
                .clamp(0.0, height as f32 - 1.0) as u32;
            let idx = (src_y * width + src_x) as usize;
            let a = match info.color_type {
                png::ColorType::Rgba => buf.get(idx * 4 + 3).copied().unwrap_or(0),
                png::ColorType::Rgb => 255,
                png::ColorType::GrayscaleAlpha => buf.get(idx * 2 + 1).copied().unwrap_or(0),
                png::ColorType::Grayscale => 255,
                _ => 0,
            };
            alpha[(row * side + col) as usize] = a;
        }
    }
    Some(GlyphBitmap {
        data: alpha,
        width: side as u16,
        height: side as u16,
        x_offset: 0,
        y_offset: 0,
        advance: size_px,
    })
}

fn rasterize_svg(svg: &[u8], size_px: f32) -> Option<GlyphBitmap> {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).ok()?;
    let side = size_px.ceil().max(1.0) as u32;
    let mut pixmap = Pixmap::new(side, side)?;
    let scale = side as f32 / tree.size().width().max(tree.size().height());
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let data: Vec<u8> = pixmap.pixels().iter().map(|px| px.alpha()).collect();
    Some(GlyphBitmap {
        data,
        width: side as u16,
        height: side as u16,
        x_offset: 0,
        y_offset: 0,
        advance: size_px,
    })
}

fn default_bitmap() -> GlyphBitmap {
    GlyphBitmap {
        data: vec![255; 256],
        width: 16,
        height: 16,
        x_offset: 0,
        y_offset: 0,
        advance: 16.0,
    }
}
