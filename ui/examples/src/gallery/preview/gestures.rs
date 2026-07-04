//! Gestures 分组的预览 painter（DC-17）。
//!
//! 包含：gesture_demo。

use super::{PreviewPainter, border_of};
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::PaintCtx;

pub struct GesturePainter;
impl PreviewPainter for GesturePainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let pad = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(360.0, 110.0));
        ctx.recorder.fill_rect(pad, tokens.background);
        ctx.recorder.stroke_rect(pad, border_of(tokens, tokens.background), 1.0);
        let gestures = [("Tap", 60.0), ("Pan", 150.0), ("Pinch", 240.0), ("Long press", 320.0)];
        for (label, x) in gestures.iter() {
            ctx.recorder
                .draw_text(label, Point::new(*x, 70.0), 13.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Gesture arena: tap / pan / pinch / long-press recognition",
            Point::new(24.0, 160.0),
            11.0,
            tokens.on_background,
        );
    }
}
