//! 页面视口滚动状态与滚动条布局。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};

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
    /// WebView 可见区域宽度（已扣除滚动条占位）。
    pub viewport_w: f32,
    /// WebView 可见区域高度（已扣除滚动条占位）。
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
        .map(|g| g.x + g.font_size * 0.6)
        .fold(0.0f32, f32::max);
    let image_max = primitives
        .images
        .iter()
        .map(|i| i.rect.origin.x + i.rect.size.width)
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
    scale: f32,
) -> PageScrollLayout {
    if content_w <= 0.0 || content_h <= 0.0 {
        return PageScrollLayout::from_content_rect(content_x, content_y, content_w, content_h);
    }

    let sb = layout::SCROLLBAR_THICKNESS * scale;

    let mut viewport_w = content_w;
    let mut viewport_h = content_h;

    let mut need_v = doc_h_physical > viewport_h + 0.5;
    if need_v {
        viewport_w = (content_w - sb).max(0.0);
    }
    let need_h = doc_w_physical > viewport_w + 0.5;
    if need_h {
        viewport_h = (content_h - sb).max(0.0);
    }
    need_v = doc_h_physical > viewport_h + 0.5;
    if need_v && viewport_w > content_w - sb {
        viewport_w = (content_w - sb).max(0.0);
    }

    let max_scroll_x = (doc_w_physical - viewport_w).max(0.0);
    let max_scroll_y = (doc_h_physical - viewport_h).max(0.0);

    PageScrollLayout {
        viewport_x: content_x,
        viewport_y: content_y,
        viewport_w,
        viewport_h,
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
            layout.viewport_h
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
            layout.viewport_w
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
pub fn push_scrollbar_fills(
    geometry: &ScrollbarGeometry,
    track_color: Color,
    thumb_color: Color,
    fills: &mut Vec<FillPrimitive>,
) {
    for rect in [geometry.vertical_track, geometry.horizontal_track, geometry.corner]
        .into_iter()
        .flatten()
    {
        fills.push(scroll_rect_fill(rect.0, rect.1, rect.2, rect.3, track_color));
    }
    for rect in [geometry.vertical_thumb, geometry.horizontal_thumb]
        .into_iter()
        .flatten()
    {
        fills.push(scroll_rect_fill(rect.0, rect.1, rect.2, rect.3, thumb_color));
    }
}

/// 将滚动偏移限制在当前布局允许范围内。
pub fn clamp_scroll(scroll: TabScrollState, layout: &PageScrollLayout) -> TabScrollState {
    TabScrollState {
        x: scroll.x.clamp(0.0, layout.max_scroll_x),
        y: scroll.y.clamp(0.0, layout.max_scroll_y),
    }
}
