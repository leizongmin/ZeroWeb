//! 页面视口滚动状态与滚动条布局。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives, RoundedRectPrimitive};

use crate::layout;

/// 单标签页滚动偏移（物理像素）。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TabScrollState {
    /// 水平偏移。
    pub x: f32,
    /// 垂直偏移。
    pub y: f32,
}

/// 当前帧的视口与滚动条几何（物理像素）。
#[derive(Debug, Clone, Copy)]
pub struct PageScrollLayout {
    /// WebView 可见区域原点 x。
    pub viewport_x: f32,
    /// WebView 可见区域原点 y。
    pub viewport_y: f32,
    /// WebView 可见区域宽度。
    pub viewport_w: f32,
    /// WebView 可见区域高度。
    pub viewport_h: f32,
    /// 是否显示垂直滚动条。
    pub show_vertical: bool,
    /// 是否显示水平滚动条。
    pub show_horizontal: bool,
    /// 最大水平滚动量。
    pub max_scroll_x: f32,
    /// 最大垂直滚动量。
    pub max_scroll_y: f32,
}

impl PageScrollLayout {
    /// 无滚动条时的默认布局（视口等于内容区）。
    pub fn from_content_rect(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            viewport_x: x,
            viewport_y: y,
            viewport_w: w,
            viewport_h: h,
            show_vertical: false,
            show_horizontal: false,
            max_scroll_x: 0.0,
            max_scroll_y: 0.0,
        }
    }
}

/// 滚动条轨道与滑块几何。
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarGeometry {
    pub vertical_track: Option<(f32, f32, f32, f32)>,
    pub horizontal_track: Option<(f32, f32, f32, f32)>,
    pub vertical_thumb: Option<(f32, f32, f32, f32)>,
    pub horizontal_thumb: Option<(f32, f32, f32, f32)>,
    pub corner: Option<(f32, f32, f32, f32)>,
}

/// 滚动条轴向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    /// 垂直滚动条。
    Vertical,
    /// 水平滚动条。
    Horizontal,
}

/// 滚动条命中区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarHit {
    /// 垂直滑块。
    VerticalThumb,
    /// 垂直轨道（滑块外）。
    VerticalTrack,
    /// 水平滑块。
    HorizontalThumb,
    /// 水平轨道（滑块外）。
    HorizontalTrack,
}

fn point_in_rect(px: f32, py: f32, rect: (f32, f32, f32, f32)) -> bool {
    let (x, y, w, h) = rect;
    px >= x && px < x + w && py >= y && py < y + h
}

/// 命中测试：优先滑块，其次轨道。
pub fn hit_test_scrollbar(px: f32, py: f32, geometry: &ScrollbarGeometry) -> Option<ScrollbarHit> {
    if geometry.vertical_thumb.is_some_and(|r| point_in_rect(px, py, r)) {
        return Some(ScrollbarHit::VerticalThumb);
    }
    if geometry.horizontal_thumb.is_some_and(|r| point_in_rect(px, py, r)) {
        return Some(ScrollbarHit::HorizontalThumb);
    }
    if geometry.vertical_track.is_some_and(|r| point_in_rect(px, py, r)) {
        return Some(ScrollbarHit::VerticalTrack);
    }
    if geometry.horizontal_track.is_some_and(|r| point_in_rect(px, py, r)) {
        return Some(ScrollbarHit::HorizontalTrack);
    }
    None
}

pub(crate) fn vertical_track_len(layout: &PageScrollLayout, content_h: f32, scale: f32) -> f32 {
    if layout.show_horizontal {
        (content_h - layout::SCROLLBAR_THICKNESS * scale).max(0.0)
    } else {
        content_h
    }
}

pub(crate) fn horizontal_track_len(layout: &PageScrollLayout, content_w: f32, scale: f32) -> f32 {
    if layout.show_vertical {
        (content_w - layout::SCROLLBAR_THICKNESS * scale).max(0.0)
    } else {
        content_w
    }
}

pub(crate) fn vertical_thumb_len(layout: &PageScrollLayout, track_len: f32, scale: f32) -> f32 {
    let min_thumb = layout::SCROLLBAR_MIN_THUMB * scale;
    let doc_h = layout.max_scroll_y + layout.viewport_h;
    (track_len * layout.viewport_h / doc_h).max(min_thumb).min(track_len)
}

pub(crate) fn horizontal_thumb_len(layout: &PageScrollLayout, track_len: f32, scale: f32) -> f32 {
    let min_thumb = layout::SCROLLBAR_MIN_THUMB * scale;
    let doc_w = layout.max_scroll_x + layout.viewport_w;
    (track_len * layout.viewport_w / doc_w).max(min_thumb).min(track_len)
}

/// 由指针在垂直轨道上的位置计算 `scroll.y`（`grab_offset` 为指针相对滑块顶边的偏移）。
pub fn scroll_y_from_pointer(
    layout: &PageScrollLayout,
    content_y: f32,
    content_h: f32,
    scale: f32,
    pointer_y: f32,
    grab_offset: f32,
) -> f32 {
    if layout.max_scroll_y <= 0.0 {
        return 0.0;
    }
    let track_len = vertical_track_len(layout, content_h, scale);
    let thumb_h = vertical_thumb_len(layout, track_len, scale);
    let travel = (track_len - thumb_h).max(0.0);
    if travel <= 0.0 {
        return 0.0;
    }
    let thumb_top = (pointer_y - grab_offset - content_y).clamp(0.0, travel);
    (thumb_top / travel) * layout.max_scroll_y
}

/// 由指针在水平轨道上的位置计算 `scroll.x`。
pub fn scroll_x_from_pointer(
    layout: &PageScrollLayout,
    content_x: f32,
    content_w: f32,
    scale: f32,
    pointer_x: f32,
    grab_offset: f32,
) -> f32 {
    if layout.max_scroll_x <= 0.0 {
        return 0.0;
    }
    let track_len = horizontal_track_len(layout, content_w, scale);
    let thumb_w = horizontal_thumb_len(layout, track_len, scale);
    let travel = (track_len - thumb_w).max(0.0);
    if travel <= 0.0 {
        return 0.0;
    }
    let thumb_left = (pointer_x - grab_offset - content_x).clamp(0.0, travel);
    (thumb_left / travel) * layout.max_scroll_x
}

/// 从渲染图元估算文档宽度（CSS 逻辑像素）。
pub fn primitives_content_width(primitives: &RenderPrimitives) -> f32 {
    let fill_max = primitives
        .fills
        .iter()
        .map(|f| f.rect.origin.x + f.rect.size.width)
        .fold(0.0f32, f32::max);
    let glyph_max = primitives
        .glyphs
        .iter()
        .filter(|g| g.glyph_id != 0 && g.font_size > 0.0)
        .map(|g| g.x + g.font_size * 0.6)
        .fold(0.0f32, f32::max);
    // 图片为保持 crop 语义会保留原始 rect，文档宽度只能使用实际可见的 clip 交集。
    // https://drafts.csswg.org/css-overflow-3/#scrollable
    let image_max = primitives
        .images
        .iter()
        .filter_map(|i| match i.clip {
            Some(clip) => i.rect.intersection(&clip).map(|visible| visible.right()),
            None => Some(i.rect.right()),
        })
        .fold(0.0f32, f32::max);
    fill_max.max(glyph_max).max(image_max)
}

/// 从渲染图元估算文档高度（CSS 逻辑像素）。
pub fn primitives_content_height(primitives: &RenderPrimitives) -> f32 {
    let fill_max = primitives
        .fills
        .iter()
        .map(|f| f.rect.origin.y + f.rect.size.height)
        .fold(0.0f32, f32::max);
    let glyph_max = primitives
        .glyphs
        .iter()
        .map(|g| g.y + g.font_size)
        .fold(0.0f32, f32::max);
    let image_max = primitives
        .images
        .iter()
        .map(|i| i.rect.origin.y + i.rect.size.height)
        .fold(0.0f32, f32::max);
    fill_max.max(glyph_max).max(image_max)
}

/// 计算视口布局与滚动上限。
pub fn compute_page_scroll_layout(
    content_x: f32,
    content_y: f32,
    content_w: f32,
    content_h: f32,
    doc_w_physical: f32,
    doc_h_physical: f32,
    _scale: f32,
) -> PageScrollLayout {
    if content_w <= 0.0 || content_h <= 0.0 {
        return PageScrollLayout::from_content_rect(content_x, content_y, content_w, content_h);
    }

    let need_v = doc_h_physical > content_h + 0.5;
    let need_h = doc_w_physical > content_w + 0.5;
    let max_scroll_x = (doc_w_physical - content_w).max(0.0);
    let max_scroll_y = (doc_h_physical - content_h).max(0.0);

    PageScrollLayout {
        viewport_x: content_x,
        viewport_y: content_y,
        viewport_w: content_w,
        viewport_h: content_h,
        show_vertical: need_v && max_scroll_y > 0.0,
        show_horizontal: need_h && max_scroll_x > 0.0,
        max_scroll_x,
        max_scroll_y,
    }
}

/// 根据布局与当前滚动偏移计算滚动条几何。
pub fn scrollbar_geometry(
    layout: &PageScrollLayout,
    scroll: TabScrollState,
    content_x: f32,
    content_y: f32,
    content_w: f32,
    content_h: f32,
    scale: f32,
) -> ScrollbarGeometry {
    let sb = layout::SCROLLBAR_THICKNESS * scale;
    let min_thumb = layout::SCROLLBAR_MIN_THUMB * scale;

    let mut vertical_track = None;
    let mut horizontal_track = None;
    let mut vertical_thumb = None;
    let mut horizontal_thumb = None;
    let mut corner = None;

    if layout.show_vertical {
        let track_h = if layout.show_horizontal {
            (content_h - sb).max(0.0)
        } else {
            content_h
        };
        let track_x = content_x + content_w - sb;
        vertical_track = Some((track_x, content_y, sb, track_h));

        let doc_h = layout.max_scroll_y + layout.viewport_h;
        let thumb_h = (track_h * layout.viewport_h / doc_h).max(min_thumb).min(track_h);
        let travel = (track_h - thumb_h).max(0.0);
        let thumb_y = if layout.max_scroll_y > 0.0 {
            content_y + travel * (scroll.y / layout.max_scroll_y)
        } else {
            content_y
        };
        vertical_thumb = Some((track_x, thumb_y, sb, thumb_h));
    }

    if layout.show_horizontal {
        let track_w = if layout.show_vertical {
            (content_w - sb).max(0.0)
        } else {
            content_w
        };
        let track_y = content_y + content_h - sb;
        horizontal_track = Some((content_x, track_y, track_w, sb));

        let doc_w = layout.max_scroll_x + layout.viewport_w;
        let thumb_w = (track_w * layout.viewport_w / doc_w).max(min_thumb).min(track_w);
        let travel = (track_w - thumb_w).max(0.0);
        let thumb_x = if layout.max_scroll_x > 0.0 {
            content_x + travel * (scroll.x / layout.max_scroll_x)
        } else {
            content_x
        };
        horizontal_thumb = Some((thumb_x, track_y, thumb_w, sb));
    }

    if layout.show_vertical && layout.show_horizontal {
        corner = Some((content_x + content_w - sb, content_y + content_h - sb, sb, sb));
    }

    ScrollbarGeometry {
        vertical_track,
        horizontal_track,
        vertical_thumb,
        horizontal_thumb,
        corner,
    }
}

fn scroll_rect_fill(x: f32, y: f32, w: f32, h: f32, color: Color) -> FillPrimitive {
    FillPrimitive {
        rect: Rect::new(x, y, w, h),
        color,
    }
}

/// 绘制滚动条轨道与滑块。
///
/// Overlay 风格：track 透明，thumb 带圆角与内边距，更克制不喧宾夺主。
#[allow(clippy::too_many_arguments)]
pub fn push_scrollbar_fills(
    geometry: &ScrollbarGeometry,
    track_color: Color,
    thumb_color: Color,
    thumb_hover_color: Color,
    thumb_active_color: Color,
    hover: Option<ScrollbarHit>,
    dragging: Option<ScrollbarAxis>,
    fills: &mut Vec<FillPrimitive>,
    rounded_rects: &mut Vec<RoundedRectPrimitive>,
) {
    // Track：仅在传入颜色不透明时绘制（默认透明 → overlay 风格）
    if track_color.a > 0 {
        for rect in [geometry.vertical_track, geometry.horizontal_track, geometry.corner]
            .into_iter()
            .flatten()
        {
            fills.push(scroll_rect_fill(rect.0, rect.1, rect.2, rect.3, track_color));
        }
    }

    let vertical_thumb_color = match dragging {
        Some(ScrollbarAxis::Vertical) => thumb_active_color,
        _ => match hover {
            Some(ScrollbarHit::VerticalThumb) => thumb_hover_color,
            _ => thumb_color,
        },
    };
    let horizontal_thumb_color = match dragging {
        Some(ScrollbarAxis::Horizontal) => thumb_active_color,
        _ => match hover {
            Some(ScrollbarHit::HorizontalThumb) => thumb_hover_color,
            _ => thumb_color,
        },
    };

    // Thumb 内缩 2px 留呼吸感，用圆角矩形绘制（Chrome 风格柔和圆角）
    let pad = 2.0;
    // 圆角半径取 thumb 厚度的一半，形成胶囊形（参考 Chrome overlay 滚动条）
    let radius_ratio = 0.5;
    if let Some(rect) = geometry.vertical_thumb {
        let (x, y, w, h) = (rect.0, rect.1, rect.2, rect.3);
        let inner_w = (w - pad).max(1.0);
        let inner_x = x + (w - inner_w) * 0.5;
        let r = (inner_w * radius_ratio).min(h * 0.5).max(0.0);
        rounded_rects.push(scroll_rounded_rect(inner_x, y, inner_w, h, r, vertical_thumb_color));
    }
    if let Some(rect) = geometry.horizontal_thumb {
        let (x, y, w, h) = (rect.0, rect.1, rect.2, rect.3);
        let inner_h = (h - pad).max(1.0);
        let inner_y = y + (h - inner_h) * 0.5;
        let r = (inner_h * radius_ratio).min(w * 0.5).max(0.0);
        rounded_rects.push(scroll_rounded_rect(x, inner_y, w, inner_h, r, horizontal_thumb_color));
    }
}

/// 构造圆角矩形（uniform 圆角）。
fn scroll_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) -> RoundedRectPrimitive {
    RoundedRectPrimitive {
        rect: Rect::new(x, y, w, h),
        color,
        top_left_radius: radius,
        top_right_radius: radius,
        bottom_right_radius: radius,
        bottom_left_radius: radius,
    }
}

/// 将滚动偏移限制在当前布局允许范围内。
pub fn clamp_scroll(scroll: TabScrollState, layout: &PageScrollLayout) -> TabScrollState {
    TabScrollState {
        x: scroll.x.clamp(0.0, layout.max_scroll_x),
        y: scroll.y.clamp(0.0, layout.max_scroll_y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::image_cache::ImageKey;
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive, ImagePrimitive};

    #[test]
    fn clipped_glyphs_do_not_expand_document_width() {
        let mut primitives = RenderPrimitives::new();
        primitives.add_glyph(GlyphPrimitive {
            x: 20.0,
            y: 20.0,
            font_size: 10.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 'A' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        primitives.add_glyph(GlyphPrimitive {
            x: 1_600.0,
            y: 20.0,
            font_size: 0.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 0,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });

        assert_eq!(primitives_content_width(&primitives), 26.0);
    }

    #[test]
    fn clipped_images_do_not_expand_document_width() {
        let mut primitives = RenderPrimitives::new();
        primitives.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 1_600.0, 100.0),
            image_key: ImageKey::new(1),
            clip: Some(Rect::new(0.0, 0.0, 100.0, 100.0)),
        });

        assert_eq!(primitives_content_width(&primitives), 100.0);
    }

    #[test]
    fn vertical_overlay_scrollbar_does_not_create_horizontal_overflow() {
        let layout = compute_page_scroll_layout(0.0, 0.0, 100.0, 100.0, 100.0, 200.0, 1.0);

        assert!(layout.show_vertical);
        assert!(!layout.show_horizontal);
        assert_eq!(layout.viewport_w, 100.0);
        assert_eq!(layout.max_scroll_x, 0.0);
    }
}
