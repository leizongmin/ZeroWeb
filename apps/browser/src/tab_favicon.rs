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

const FAVICON_BASE_CODEPOINT: u32 = 0xF100;
const DEFAULT_FAVICON_SVG: &[u8] = include_bytes!("../assets/icons/globe.svg");

fn favicon_glyph_id(tab_id: TabId) -> u32 {
    FAVICON_BASE_CODEPOINT + (tab_id.0 as u32 & 0x7FF)
}

fn favicon_char(tab_id: TabId) -> char {
    char::from_u32(favicon_glyph_id(tab_id)).unwrap_or('\0')
}

pub fn clear_tab_favicon(font_loader: &mut FontLoader, tab_id: TabId, size_px: f32) {
    font_loader.clear_bitmap_glyph(FAVICON_FONT_ID, favicon_glyph_id(tab_id), size_px);
}

pub fn has_tab_favicon(font_loader: &FontLoader, tab_id: TabId, size_px: f32) -> bool {
    font_loader.has_bitmap_glyph(FAVICON_FONT_ID, favicon_glyph_id(tab_id), size_px)
}

/// 书签 favicon 专用 codepoint 区间（与标签 favicon 区分，避免哈希碰撞）。
const BOOKMARK_FAVICON_BASE_CODEPOINT: u32 = 0xF800;

fn bookmark_favicon_glyph_id(url: &str) -> u32 {
    BOOKMARK_FAVICON_BASE_CODEPOINT + (fnv1a_url(url) & 0x3FF)
}

fn fnv1a_url(url: &str) -> u32 {
    let mut hash = 0x811C_9DC5u32;
    for byte in url.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn bookmark_favicon_char(url: &str) -> char {
    char::from_u32(bookmark_favicon_glyph_id(url)).unwrap_or('\0')
}

/// 为书签注册已抓取的 favicon 位图（按 URL 缓存）。
pub fn register_bookmark_favicon_bitmap(font_loader: &mut FontLoader, url: &str, size_px: f32, bitmap: GlyphBitmap) {
    font_loader.register_bitmap_glyph(FAVICON_FONT_ID, bookmark_favicon_glyph_id(url), size_px, bitmap);
}

/// 书签是否已有真实 favicon（非兜底）。
pub fn has_bookmark_favicon(font_loader: &FontLoader, url: &str, size_px: f32) -> bool {
    font_loader.has_bitmap_glyph(FAVICON_FONT_ID, bookmark_favicon_glyph_id(url), size_px)
}

/// 渲染书签 favicon：优先用已缓存的真实 favicon，否则用 globe 兜底。
/// 返回的 glyph 字符可直接用于 GlyphDraw。
pub fn bookmark_favicon_glyph(font_loader: &mut FontLoader, url: &str, size_px: f32) -> char {
    let glyph_id = bookmark_favicon_glyph_id(url);
    if !font_loader.has_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px) {
        let bitmap = rasterize_svg(DEFAULT_FAVICON_SVG, size_px).unwrap_or_else(|| default_favicon_bitmap(size_px));
        font_loader.register_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px, bitmap);
    }
    bookmark_favicon_char(url)
}

/// 注册已解码的 favicon 位图。
pub fn register_tab_favicon_bitmap(font_loader: &mut FontLoader, tab_id: TabId, size_px: f32, bitmap: GlyphBitmap) {
    font_loader.register_bitmap_glyph(FAVICON_FONT_ID, favicon_glyph_id(tab_id), size_px, bitmap);
}

/// 确保标签有占位 favicon（默认 globe），不进行网络请求。
pub fn ensure_tab_favicon_placeholder(font_loader: &mut FontLoader, tab_id: TabId, size_px: f32) -> char {
    let glyph_id = favicon_glyph_id(tab_id);
    if !font_loader.has_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px) {
        let bitmap = rasterize_svg(DEFAULT_FAVICON_SVG, size_px).unwrap_or_else(|| default_favicon_bitmap(size_px));
        font_loader.register_bitmap_glyph(FAVICON_FONT_ID, glyph_id, size_px, bitmap);
    }
    favicon_char(tab_id)
}

/// 为标签注册 favicon 并返回绘制用的 glyph 字符（仅使用缓存或占位，不阻塞网络）。
#[allow(clippy::too_many_arguments)]
pub fn ensure_tab_favicon(
    font_loader: &mut FontLoader,
    tab_id: TabId,
    _page_url: Option<&str>,
    _html: Option<&str>,
    size_px: f32,
) -> char {
    ensure_tab_favicon_placeholder(font_loader, tab_id, size_px)
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
        rotation: 0.0,
    });
}

fn resolve_favicon_bitmap(page_url: Option<&str>, html: Option<&str>, size_px: f32) -> Option<GlyphBitmap> {
    let page_url = page_url?;
    if should_skip_favicon_fetch(page_url) {
        return None;
    }
    let favicon_url = pick_favicon_url(page_url, html)?;
    fetch_favicon_bitmap(&favicon_url, size_px)
}

/// 解析 favicon URL（供异步拉取使用）。
pub fn pick_favicon_url(page_url: &str, html: Option<&str>) -> Option<String> {
    if let Some(html) = html
        && let Some(href) = extract_link_icon_href(html)
        && let Some(resolved) = resolve_href(page_url, &href)
    {
        return Some(resolved);
    }
    resolve_href(page_url, "/favicon.ico")
}

fn should_skip_favicon_fetch(page_url: &str) -> bool {
    page_url.starts_with("zero://") || page_url.starts_with("about:") || page_url.starts_with("file:")
}

/// 是否应跳过 favicon 网络拉取。
pub fn skip_favicon_fetch(page_url: &str) -> bool {
    should_skip_favicon_fetch(page_url)
}

/// 拉取并解码 favicon（在后台线程调用）。
pub fn fetch_favicon_bitmap(favicon_url: &str, size_px: f32) -> Option<GlyphBitmap> {
    let bytes = fetch_bytes(favicon_url)?;
    decode_icon_bytes(&bytes, size_px)
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
    if bytes.len() >= 4 && bytes[0..4] == [0, 0, 1, 0] {
        return decode_ico(bytes, size_px);
    }
    decode_png(bytes, size_px)
        .or_else(|| decode_ico(bytes, size_px))
        .or_else(|| {
            debug!("unsupported favicon format, using default");
            None
        })
}

fn decode_ico(bytes: &[u8], size_px: f32) -> Option<GlyphBitmap> {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes)).ok()?;
    let entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|entry| entry.width().saturating_mul(entry.height()))?;
    let image = entry.decode().ok()?;
    let width = image.width();
    let height = image.height();
    let rgba = image.rgba_data();
    sample_alpha_grid(
        width,
        height,
        rgba,
        4,
        |idx| rgba.get(idx + 3).copied().unwrap_or(0),
        size_px,
    )
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
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return None,
    };
    sample_alpha_grid(
        width,
        height,
        &buf,
        channels,
        |idx| match info.color_type {
            png::ColorType::Rgba => buf.get(idx + 3).copied().unwrap_or(0),
            png::ColorType::Rgb => 255,
            png::ColorType::GrayscaleAlpha => buf.get(idx + 1).copied().unwrap_or(0),
            png::ColorType::Grayscale => 255,
            _ => 0,
        },
        size_px,
    )
}

fn sample_alpha_grid(
    width: u32,
    height: u32,
    _data: &[u8],
    _channels: usize,
    alpha_at: impl Fn(usize) -> u8,
    size_px: f32,
) -> Option<GlyphBitmap> {
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
            let idx = (src_y * width + src_x) as usize * _channels;
            alpha[(row * side + col) as usize] = alpha_at(idx);
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

/// 兜底 favicon 位图（globe 图标）。所有加载失败 / 内部页 / 未命中网络的情况统一用它。
pub fn default_favicon_bitmap(size_px: f32) -> GlyphBitmap {
    rasterize_svg(DEFAULT_FAVICON_SVG, size_px).unwrap_or_else(|| blank_favicon_bitmap(size_px))
}

/// 完全无法光栅化 globe 时的最终兜底（透明，避免渲染成实心方块）。
fn blank_favicon_bitmap(size_px: f32) -> GlyphBitmap {
    let side = size_px.ceil().max(1.0) as u16;
    GlyphBitmap {
        data: vec![0; side as usize * side as usize],
        width: side,
        height: side,
        x_offset: 0,
        y_offset: 0,
        advance: size_px,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_browser_shell::TabId;
    use zero_render_foundation::color::Color;

    #[test]
    fn extract_link_icon_href_reads_head_link() {
        let html = r#"<!doctype html><head><link rel="icon" href="/assets/app.ico"></head><body></body>"#;
        assert_eq!(extract_link_icon_href(html).as_deref(), Some("/assets/app.ico"));
    }

    #[test]
    fn pick_favicon_url_prefers_html_link_over_default_ico() {
        let html = r#"<link rel="shortcut icon" href="https://cdn.example.com/favicon.png">"#;
        let url = pick_favicon_url("https://example.com/page", Some(html)).expect("favicon url");
        assert_eq!(url, "https://cdn.example.com/favicon.png");
    }

    #[test]
    fn pick_favicon_url_falls_back_to_favicon_ico() {
        let url = pick_favicon_url("https://example.com", None).expect("favicon url");
        assert_eq!(url, "https://example.com/favicon.ico");
    }

    #[test]
    fn favicon_register_and_lookup_use_same_codepoint() {
        let mut loader = FontLoader::new();
        let tab_id = TabId(3);
        let size = 14.0;
        let ch = ensure_tab_favicon(&mut loader, tab_id, None, None, size);
        assert_eq!(ch, favicon_char(tab_id));
        assert!(loader.has_bitmap_glyph(FAVICON_FONT_ID, favicon_glyph_id(tab_id), size));
        let (resolved, bitmap) = loader
            .rasterize_glyph_with_fallback(FAVICON_FONT_ID, ch, size)
            .expect("favicon bitmap should resolve");
        assert_eq!(resolved, FAVICON_FONT_ID);
        assert!(bitmap.width > 0 && bitmap.height > 0);
        assert!(bitmap.data.iter().any(|&a| a > 0));
    }

    #[test]
    fn render_tab_favicon_produces_drawable_glyph() {
        let mut loader = FontLoader::new();
        let mut glyphs = Vec::new();
        render_tab_favicon(
            &mut loader,
            &mut glyphs,
            TabId(1),
            None,
            None,
            20.0,
            20.0,
            14.0,
            Color::BLACK,
        );
        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].font_id, FAVICON_FONT_ID);
        assert!(
            loader
                .rasterize_glyph_with_fallback(FAVICON_FONT_ID, glyphs[0].ch, 14.0)
                .is_ok()
        );
    }

    #[test]
    fn bookmark_favicon_glyph_caches_and_resolves() {
        let mut loader = FontLoader::new();
        let url = "https://example.com/page";
        let size = 14.0;
        let ch = bookmark_favicon_glyph(&mut loader, url, size);
        assert_eq!(ch, bookmark_favicon_char(url));
        assert!(has_bookmark_favicon(&loader, url, size));
        // 不同的 URL 应映射到不同 codepoint（哈希不同）
        let other = bookmark_favicon_glyph(&mut loader, "https://other.test", size);
        assert_ne!(ch, other);
    }

    #[test]
    fn default_favicon_bitmap_is_globe_not_solid_block() {
        let bitmap = default_favicon_bitmap(32.0);
        assert_eq!(bitmap.width, 32);
        assert_eq!(bitmap.height, 32);
        // globe 是描线图标，必有透明像素（非全不透明实心块）
        let transparent = bitmap.data.iter().filter(|&&a| a < 32).count();
        assert!(transparent > 0, "default favicon should not be a solid block");
        let opaque = bitmap.data.iter().filter(|&&a| a > 200).count();
        assert!(opaque > 0, "default favicon should have ink");
    }
}
