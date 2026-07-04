//! Animation 分组的预览 painter（DC-17）。
//!
//! 包含：animation_demo。

use super::{PreviewPainter, border_of};
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::PaintCtx;

pub struct AnimationPainter;
impl AnimationPainter {
    fn fill_w(&self, i: usize) -> f32 {
        240.0
            * match i {
                0 => 0.5,
                1 => 0.7,
                _ => 0.85,
            }
    }
}
impl PreviewPainter for AnimationPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let curves = ["Linear", "EaseOut", "Spring"];
        for (i, name) in curves.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let track = Rect::from_origin_size(Point::new(120.0, y), Size::new(240.0, 12.0));
            ctx.recorder.fill_rect(track, tokens.background);
            ctx.recorder
                .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
            let fill_w = self.fill_w(i);
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::new(120.0, y), Size::new(fill_w, 12.0)),
                tokens.primary,
            );
            ctx.recorder
                .draw_text(name, Point::new(24.0, y + 10.0), 12.0, tokens.on_background);
        }
    }
}
