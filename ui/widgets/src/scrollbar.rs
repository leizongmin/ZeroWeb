//! ScrollBar — 通用滚动条几何、命中、拖动命令与视觉（spec FR-006 / DC-4）。
//!
//! 页面内容尺寸/scroll offset 由 WebView 管理；本控件只算**外部**滚动条的几何与绘制。
//! 拖动 thumb / 点击 track 转为 [`ScrollCommand`]（不直接改业务状态）；命中判定区分
//! thumb / track-before / track-after / none。`paint_scrollbar` 把 track+thumb 记录进
//! `PaintRecorder`（可进统一 Scene，与 chrome render 桥一致，不绕过 ui/render）。

use zero_ui_core::geometry::{Point, Rect};
use zero_ui_core::scroll::{ScrollCommand, ScrollMetrics};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::PaintRecorder;

/// 滚动条朝向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollOrientation {
    Vertical,
    Horizontal,
}

/// 滚动条几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarGeometry {
    pub track: Rect,
    pub thumb: Rect,
    pub orientation: ScrollOrientation,
}

/// 滚动条命中区域（hit-test 结果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBarHit {
    /// 不在 track 上。
    None,
    /// thumb（可拖动）。
    Thumb,
    /// thumb 之前（上方/左方）的 track 空白 → 上一页。
    TrackBefore,
    /// thumb 之后（下方/右方）的 track 空白 → 下一页。
    TrackAfter,
}

const THICKNESS: f32 = 10.0;

/// 由视口矩形 + 滚动度量算滚动条几何。
///
/// 返回 `None` 表示内容不超出视口（无需滚动条）。
pub fn layout_scrollbar(
    viewport: Rect,
    metrics: ScrollMetrics,
    orientation: ScrollOrientation,
) -> Option<ScrollBarGeometry> {
    let track = match orientation {
        ScrollOrientation::Vertical => Rect::from_ltrb(
            viewport.right() - THICKNESS,
            viewport.top(),
            viewport.right(),
            viewport.bottom(),
        ),
        ScrollOrientation::Horizontal => Rect::from_ltrb(
            viewport.left(),
            viewport.bottom() - THICKNESS,
            viewport.right(),
            viewport.bottom(),
        ),
    };

    let (content_extent, viewport_extent, scroll, max_scroll) = match orientation {
        ScrollOrientation::Vertical => (
            metrics.content_height,
            metrics.viewport_height,
            metrics.scroll_y,
            metrics.max_scroll_y(),
        ),
        ScrollOrientation::Horizontal => (
            metrics.content_width,
            metrics.viewport_width,
            metrics.scroll_x,
            metrics.max_scroll_x(),
        ),
    };

    if content_extent <= viewport_extent || max_scroll <= 0.0 {
        return None;
    }

    let track_start = match orientation {
        ScrollOrientation::Vertical => track.top(),
        ScrollOrientation::Horizontal => track.left(),
    };
    let track_extent = match orientation {
        ScrollOrientation::Vertical => track.size.height,
        ScrollOrientation::Horizontal => track.size.width,
    };

    let thumb_extent = (track_extent * viewport_extent / content_extent).max(24.0);
    let ratio = scroll / max_scroll;
    let thumb_start = track_start + (track_extent - thumb_extent) * ratio;

    let thumb = match orientation {
        ScrollOrientation::Vertical => {
            Rect::from_ltrb(track.left(), thumb_start, track.right(), thumb_start + thumb_extent)
        }
        ScrollOrientation::Horizontal => {
            Rect::from_ltrb(thumb_start, track.top(), thumb_start + thumb_extent, track.bottom())
        }
    };

    Some(ScrollBarGeometry {
        track,
        thumb,
        orientation,
    })
}

/// 命中测试：判定一个点落在滚动条的哪个区域。
pub fn hit_test(geom: &ScrollBarGeometry, point: Point) -> ScrollBarHit {
    if !geom.track.contains(point) {
        return ScrollBarHit::None;
    }
    if geom.thumb.contains(point) {
        return ScrollBarHit::Thumb;
    }
    match geom.orientation {
        ScrollOrientation::Vertical => {
            if point.y < geom.thumb.top() {
                ScrollBarHit::TrackBefore
            } else {
                ScrollBarHit::TrackAfter
            }
        }
        ScrollOrientation::Horizontal => {
            if point.x < geom.thumb.left() {
                ScrollBarHit::TrackBefore
            } else {
                ScrollBarHit::TrackAfter
            }
        }
    }
}

/// track 空白区点击 → 翻页命令（`TrackBefore` = 上一页，`TrackAfter` = 下一页）。
/// thumb / none 不产生命令（thumb 拖动见 [`drag_to_command`]）。
pub fn hit_to_page_command(geom: &ScrollBarGeometry, hit: ScrollBarHit) -> Option<ScrollCommand> {
    match (hit, geom.orientation) {
        (ScrollBarHit::TrackBefore, ScrollOrientation::Vertical) => Some(ScrollCommand::Page {
            pages_x: 0.0,
            pages_y: -1.0,
        }),
        (ScrollBarHit::TrackAfter, ScrollOrientation::Vertical) => Some(ScrollCommand::Page {
            pages_x: 0.0,
            pages_y: 1.0,
        }),
        (ScrollBarHit::TrackBefore, ScrollOrientation::Horizontal) => Some(ScrollCommand::Page {
            pages_x: -1.0,
            pages_y: 0.0,
        }),
        (ScrollBarHit::TrackAfter, ScrollOrientation::Horizontal) => Some(ScrollCommand::Page {
            pages_x: 1.0,
            pages_y: 0.0,
        }),
        _ => None,
    }
}

/// 把 thumb 上的指针拖动（从 `from` 到 `to`）转为相对滚动命令。
///
/// 仅当起始点落在 thumb 上才视为有效拖动。拖动量按 **thumb 可移动范围**
/// （`track_extent - thumb_extent`）映射到内容滚动量：thumb 在 track 里移动一整段
/// 对应内容滚动 `max_scroll`（由 `metrics` 提供，DC-4 修正：不再用占位常量）。
pub fn drag_to_command(
    geom: &ScrollBarGeometry,
    metrics: ScrollMetrics,
    from: Point,
    to: Point,
) -> Option<ScrollCommand> {
    if !geom.thumb.contains(from) {
        return None;
    }
    match geom.orientation {
        ScrollOrientation::Vertical => {
            let thumb_extent = geom.thumb.size.height.max(1.0);
            let usable = (geom.track.size.height - thumb_extent).max(1.0);
            let dy = to.y - from.y;
            let scroll_delta = dy / usable * metrics.max_scroll_y();
            Some(ScrollCommand::By {
                dx: 0.0,
                dy: scroll_delta,
            })
        }
        ScrollOrientation::Horizontal => {
            let thumb_extent = geom.thumb.size.width.max(1.0);
            let usable = (geom.track.size.width - thumb_extent).max(1.0);
            let dx = to.x - from.x;
            let scroll_delta = dx / usable * metrics.max_scroll_x();
            Some(ScrollCommand::By {
                dx: scroll_delta,
                dy: 0.0,
            })
        }
    }
}

/// 滚动条视觉样式（semantic 色；DC-5 主题接入后由 token 解析）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarStyle {
    pub track_color: Color,
    pub thumb_color: Color,
}

impl ScrollBarStyle {
    /// 从 semantic token 派生（DC-5：track/thumb 由 `on_surface` 与 `surface` 中和，
    /// 自动适配 light/dark——light 得浅 track + 深 thumb，dark 得深 track + 浅 thumb，
    /// 不硬编码浏览器色值）。
    pub fn from_tokens(tokens: &SemanticTokens) -> ScrollBarStyle {
        ScrollBarStyle {
            // track 更接近 surface（更淡），thumb 居中（更显眼）。
            track_color: tokens.on_surface.mix(tokens.surface, 0.85),
            thumb_color: tokens.on_surface.mix(tokens.surface, 0.5),
        }
    }

    /// thumb hover 色（略亮，DC-4 浏览器桥接用）。
    pub fn thumb_hover_color(&self) -> Color {
        self.thumb_color.lighten(0.1)
    }

    /// thumb active/dragging 色（略暗，DC-4 浏览器桥接用）。
    pub fn thumb_active_color(&self) -> Color {
        self.thumb_color.darken(0.1)
    }
}

impl Default for ScrollBarStyle {
    fn default() -> ScrollBarStyle {
        ScrollBarStyle::from_tokens(&SemanticTokens::light())
    }
}

/// 把滚动条（track + thumb）记录进 `PaintRecorder`（进统一 Scene，不绕过 ui/render）。
pub fn paint_scrollbar(recorder: &mut dyn PaintRecorder, geom: &ScrollBarGeometry, style: &ScrollBarStyle) {
    recorder.fill_rect(geom.track, style.track_color);
    recorder.fill_rect(geom.thumb, style.thumb_color);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(scroll_y: f32) -> ScrollMetrics {
        ScrollMetrics {
            content_width: 200.0,
            content_height: 1000.0,
            viewport_width: 200.0,
            viewport_height: 200.0,
            scroll_x: 0.0,
            scroll_y,
        }
    }

    #[test]
    fn vertical_thumb_ratio_and_position() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        // viewport/content = 200/1000 = 0.2 → thumb 高度 = track 高 * 0.2。
        let track_h = geom.track.size.height;
        assert!((geom.thumb.size.height - track_h * 0.2).abs() < 0.5);
        // scroll 400 / max 800 = 0.5 → thumb 顶端 = (track_h - thumb_h) * 0.5 = 80。
        assert!((geom.thumb.top() - 80.0).abs() < 0.5);
    }

    #[test]
    fn no_scrollbar_when_content_fits() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let small = ScrollMetrics {
            content_height: 100.0,
            viewport_height: 200.0,
            content_width: 200.0,
            viewport_width: 200.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        assert!(layout_scrollbar(viewport, small, ScrollOrientation::Vertical).is_none());
    }

    #[test]
    fn hit_test_zones() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        // thumb 顶端 ≈ 80，底端 ≈ 120。
        assert_eq!(
            hit_test(&geom, Point::new(geom.thumb.left() + 1.0, geom.thumb.top() + 1.0)),
            ScrollBarHit::Thumb
        );
        // thumb 之上的 track → TrackBefore。
        assert_eq!(
            hit_test(&geom, Point::new(geom.thumb.left() + 1.0, 10.0)),
            ScrollBarHit::TrackBefore
        );
        // thumb 之下的 track → TrackAfter。
        assert_eq!(
            hit_test(&geom, Point::new(geom.thumb.left() + 1.0, 180.0)),
            ScrollBarHit::TrackAfter
        );
        // track 外 → None。
        assert_eq!(hit_test(&geom, Point::new(0.0, 100.0)), ScrollBarHit::None);
    }

    #[test]
    fn track_click_to_page_command() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        assert_eq!(
            hit_to_page_command(&geom, ScrollBarHit::TrackBefore),
            Some(ScrollCommand::Page {
                pages_x: 0.0,
                pages_y: -1.0
            })
        );
        assert_eq!(
            hit_to_page_command(&geom, ScrollBarHit::TrackAfter),
            Some(ScrollCommand::Page {
                pages_x: 0.0,
                pages_y: 1.0
            })
        );
        assert_eq!(hit_to_page_command(&geom, ScrollBarHit::Thumb), None);
        assert_eq!(hit_to_page_command(&geom, ScrollBarHit::None), None);
    }

    #[test]
    fn drag_maps_thumb_delta_to_content_scroll() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let m = metrics(400.0); // max_scroll_y = 800
        let geom = layout_scrollbar(viewport, m, ScrollOrientation::Vertical).unwrap();
        // thumb 顶端 ≈ 80，thumb_h = track_h*0.2 = 40；usable = track_h - thumb_h = 200-40 = 160。
        let on_thumb = Point::new(geom.thumb.left() + 1.0, geom.thumb.top() + 1.0);
        // 下拖 16px（=usable 的 1/10）→ 内容滚动 800 的 1/10 = 80。
        let cmd = drag_to_command(&geom, m, on_thumb, Point::new(on_thumb.x, on_thumb.y + 16.0));
        match cmd {
            Some(ScrollCommand::By { dx: 0.0, dy }) => assert!((dy - 80.0).abs() < 0.5, "dy={dy}"),
            other => panic!("expected By, got {other:?}"),
        }
        // 起点不在 thumb → None。
        assert!(drag_to_command(&geom, m, Point::new(0.0, 0.0), on_thumb).is_none());
    }

    #[test]
    fn closed_loop_drag_to_clamped_offset() {
        // 端到端：metrics → layout → drag → resolve_target → clamp（apply_scroll_command 同口径）。
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let m = metrics(400.0);
        let geom = layout_scrollbar(viewport, m, ScrollOrientation::Vertical).unwrap();
        let on_thumb = Point::new(geom.thumb.left() + 1.0, geom.thumb.top() + 1.0);
        // 下拖到 thumb 几乎贴底（大幅）→ 内容滚动应被钳到 max_scroll_y(=800)。
        let cmd = drag_to_command(&geom, m, on_thumb, Point::new(on_thumb.x, on_thumb.y + 1000.0)).unwrap();
        let (_tx, ty) = cmd.resolve_target(m);
        let clamped_y = ty.clamp(0.0, m.max_scroll_y());
        assert_eq!(clamped_y, 800.0, "drag beyond track must clamp to max_scroll");
    }

    /// 测试用 recorder：数 fill_rect 调用。
    #[derive(Default)]
    struct CountRecorder {
        fills: Vec<(Rect, Color)>,
    }
    impl PaintRecorder for CountRecorder {
        fn fill_rect(&mut self, rect: Rect, color: Color) {
            self.fills.push((rect, color));
        }
        fn stroke_rect(&mut self, _rect: Rect, _color: Color, _stroke_width: f32) {}
        fn draw_text(&mut self, _text: &str, _position: Point, _size_px: f32, _color: Color) {}
        fn draw_external_surface(&mut self, _rect: Rect, _surface_id: u64) {}
        fn draw_image(&mut self, _rect: Rect, _image_ref: zero_ui_core::image::ImageRef, _tint: Color) {}
    }

    #[test]
    fn paint_records_track_and_thumb() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        let mut rec = CountRecorder::default();
        paint_scrollbar(&mut rec, &geom, &ScrollBarStyle::default());
        // track + thumb = 2 fill。
        assert_eq!(rec.fills.len(), 2);
        // 第一条是 track，第二条是 thumb。
        assert_eq!(rec.fills[0].0, geom.track);
        assert_eq!(rec.fills[1].0, geom.thumb);
        // dark 主题样式（token 派生，非硬编码）生效：paint 用 from_tokens(dark)。
        let style = ScrollBarStyle::from_tokens(&SemanticTokens::dark());
        let mut rec2 = CountRecorder::default();
        paint_scrollbar(&mut rec2, &geom, &style);
        assert_eq!(rec2.fills[0].1, style.track_color, "paint 用 style.track_color");
        assert_eq!(rec2.fills[1].1, style.thumb_color, "paint 用 style.thumb_color");
    }

    #[test]
    fn style_from_tokens_adapts_to_scheme() {
        // DC-5：from_tokens 从 on_surface/surface 派生，light/dark 自适应。
        // light：track（0.85 向 surface）浅于 thumb（0.5 居中）—— thumb 更显眼。
        let light = ScrollBarStyle::from_tokens(&SemanticTokens::light());
        assert_ne!(light.track_color, light.thumb_color, "track/thumb 应可区分");
        // light 近似历史硬编码值（track≈0.85, thumb≈0.5），确认派生合理。
        assert!(light.track_color.r > light.thumb_color.r, "light: track 浅于 thumb");
        // dark：on_surface 浅、surface 深 → mix 后 thumb 浅于 track（反相，dark 自适应）。
        let dark = ScrollBarStyle::from_tokens(&SemanticTokens::dark());
        assert!(dark.thumb_color.r > dark.track_color.r, "dark: thumb 浅于 track");
        // default() == from_tokens(light)（无硬编码）。
        assert_eq!(ScrollBarStyle::default().track_color, light.track_color);
    }

    #[test]
    fn thumb_hover_active_colors_derived_from_thumb() {
        // DC-4：hover=略亮、active=略暗，由 thumb_color 派生（不硬编码）。
        let style = ScrollBarStyle::from_tokens(&SemanticTokens::light());
        let hover = style.thumb_hover_color();
        let active = style.thumb_active_color();
        // hover 亮于 thumb（lighten 0.1）。
        assert!(hover.r > style.thumb_color.r, "hover lighter than thumb");
        // active 暗于 thumb（darken 0.1）。
        assert!(active.r < style.thumb_color.r, "active darker than thumb");
        // hover 与 active 应不同。
        assert_ne!(hover, active, "hover ≠ active");
        // dark 主题：反相，hover 更亮（更接近白）、active 更暗。
        let dark = ScrollBarStyle::from_tokens(&SemanticTokens::dark());
        let dh = dark.thumb_hover_color();
        let da = dark.thumb_active_color();
        assert!(dh.r > dark.thumb_color.r, "dark hover lighter");
        assert!(da.r < dark.thumb_color.r, "dark active darker");
    }
}
