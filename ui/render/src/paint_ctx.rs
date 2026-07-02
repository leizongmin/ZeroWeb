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

    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32) {
        self.push(RenderPrimitive::StrokeRect {
            rect,
            color,
            stroke_width,
            rounding: Rounding::ZERO,
        });
    }
}

impl SceneRecorder {
    /// 记录文本图元（M1 直接承载字符串；M2 改为引用 foundation/text 的 TextBlob）。
    pub fn draw_text(&mut self, text: &str, position: Point, size_px: f32, color: Color) {
        self.push(RenderPrimitive::Text {
            text: text.to_string(),
            position,
            size_px,
            color,
        });
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
    fn clip_is_attached_to_entries() {
        let mut r = SceneRecorder::new(WidgetId::new("card"));
        let clip = Rect::from_ltrb(0.0, 0.0, 10.0, 10.0);
        r.set_clip(Some(clip));
        r.fill_rect(Rect::ZERO, Color::WHITE);
        let scene = r.finish();
        assert_eq!(scene.entries[0].clip, Some(clip));
    }
}
