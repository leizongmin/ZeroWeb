//! Paint context — 把 widget 的 paint 调用记录为图元（spec IF-001 `PaintCtx` 具体实现）。
//!
//! `SceneRecorder` 实现 [`zero_ui_core::widget::PaintRecorder`]，作为 `Widget::paint` 的后端；
//! 记录的图元进入 [`crate::Scene`]。

use crate::render_node::RenderPrimitive;
use crate::scene::{Scene, SceneEntry};
use zero_ui_core::geometry::{Point, Rect, Rounding};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{PaintRecorder, WidgetId};

/// 记录 paint 调用为场景图元的 recorder。
pub struct SceneRecorder {
    source: WidgetId,
    clip: Option<Rect>,
    scene: Scene,
}

impl Default for SceneRecorder {
    fn default() -> SceneRecorder {
        SceneRecorder::new(WidgetId::new("__root__"))
    }
}

impl SceneRecorder {
    pub fn new(source: WidgetId) -> SceneRecorder {
        SceneRecorder {
            source,
            clip: None,
            scene: Scene::new(),
        }
    }

    pub fn set_clip(&mut self, clip: Option<Rect>) {
        self.clip = clip;
    }

    pub fn finish(self) -> Scene {
        self.scene
    }

    fn push(&mut self, primitive: RenderPrimitive) {
        self.scene.push(SceneEntry {
            source: self.source.clone(),
            clip: self.clip,
            primitive,
        });
    }
}

impl PaintRecorder for SceneRecorder {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.push(RenderPrimitive::FillRect {
            rect,
            color,
            rounding: Rounding::ZERO,
        });
    }

    fn fill_rounded_rect(&mut self, rect: Rect, corner_radius: f32, color: Color) {
        // corner_radius ≤ 0 → 等价 fill_rect（无圆角）；> 0 → 四角同半径。
        let rounding = if corner_radius <= 0.0 {
            Rounding::ZERO
        } else {
            Rounding::all(corner_radius)
        };
        self.push(RenderPrimitive::FillRect { rect, color, rounding });
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32) {
        self.push(RenderPrimitive::StrokeRect {
            rect,
            color,
            stroke_width,
            rounding: Rounding::ZERO,
        });
    }

    fn draw_text(&mut self, text: &str, position: Point, size_px: f32, color: Color) {
        self.push(RenderPrimitive::Text {
            text: text.to_string(),
            position,
            size_px,
            color,
        });
    }

    fn draw_external_surface(&mut self, rect: Rect, surface_id: u64) {
        self.push(RenderPrimitive::ExternalSurface { rect, surface_id });
    }
}

impl SceneRecorder {
    /// 记录预 shape 文本图元（DC-11：调用方先用 foundation/text 的 `TextShaper`+`TextMeasurer`
    /// 产出 `TextBlob`，后端直接光栅 glyph，不再 reshape）。
    pub fn draw_text_blob(&mut self, blob: zero_text_foundation::TextBlob, position: Point, color: Color) {
        self.push(RenderPrimitive::TextBlob { blob, position, color });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_collects_primitives_into_scene() {
        let mut r = SceneRecorder::new(WidgetId::new("btn"));
        r.fill_rect(Rect::ZERO, Color::BLACK);
        r.draw_text("OK", Point::ZERO, 14.0, Color::WHITE);
        let scene = r.finish();
        assert_eq!(scene.entries.len(), 2);
        assert_eq!(scene.entries[0].source, WidgetId::new("btn"));
        assert!(matches!(scene.entries[1].primitive, RenderPrimitive::Text { .. }));
    }

    #[test]
    fn fill_rounded_rect_emits_fillrect_with_rounding() {
        // fill_rounded_rect(radius > 0) → FillRect 带四角同半径 rounding；
        // radius ≤ 0 → 等价 fill_rect（rounding ZERO）。
        let mut r = SceneRecorder::new(WidgetId::new("disk"));
        r.fill_rounded_rect(Rect::ZERO, 8.0, Color::WHITE);
        r.fill_rounded_rect(Rect::ZERO, 0.0, Color::BLACK);
        let scene = r.finish();
        match &scene.entries[0].primitive {
            RenderPrimitive::FillRect { rounding, color, .. } => {
                assert_eq!(rounding.top_left, 8.0);
                assert_eq!(rounding.top_right, 8.0);
                assert_eq!(rounding.bottom_left, 8.0);
                assert_eq!(*color, Color::WHITE);
            }
            other => panic!("expected FillRect, got {other:?}"),
        }
        match &scene.entries[1].primitive {
            RenderPrimitive::FillRect { rounding, .. } => {
                assert_eq!(*rounding, Rounding::ZERO, "radius<=0 should be ZERO rounding");
            }
            other => panic!("expected FillRect, got {other:?}"),
        }
    }

    #[test]
    fn clip_is_attached_to_entries() {
        let mut r = SceneRecorder::new(WidgetId::new("card"));
        let clip = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
        r.set_clip(Some(clip));
        r.fill_rect(Rect::ZERO, Color::WHITE);
        let scene = r.finish();
        assert_eq!(scene.entries[0].clip, Some(clip));
    }

    #[test]
    fn draw_text_blob_records_textblob_primitive() {
        // DC-11：draw_text_blob 把预 shape 的 TextBlob 记录为 TextBlob 图元。
        let blob = zero_text_foundation::TextBlob::new(
            zero_text_foundation::ShapedText {
                runs: Vec::new(),
                total_advance_x: 0.0,
                total_advance_y: 0.0,
            },
            zero_text_foundation::TextMetrics {
                width: 0.0,
                height: 0.0,
                ascent: 0.0,
                descent: 0.0,
                line_count: 0,
            },
        );
        let mut r = SceneRecorder::new(WidgetId::new("lbl"));
        r.draw_text_blob(blob, Point::ZERO, Color::WHITE);
        let scene = r.finish();
        assert_eq!(scene.entries.len(), 1);
        assert!(matches!(scene.entries[0].primitive, RenderPrimitive::TextBlob { .. }));
    }
}
