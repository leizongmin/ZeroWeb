//! Forms 分组的预览 painter（DC-17）。
//!
//! 包含：form_demo。

use super::{PreviewPainter, border_of};
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::PaintCtx;

pub struct FormPainter;
impl PreviewPainter for FormPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let labels_y = [30.0, 76.0];
        let labels = ["Email", "Password"];
        for (i, label) in labels.iter().enumerate() {
            ctx.recorder
                .draw_text(label, Point::new(24.0, labels_y[i]), 12.0, tokens.on_background);
            let field = Rect::from_origin_size(Point::new(24.0, labels_y[i] + 8.0), Size::new(360.0, 32.0));
            ctx.recorder.fill_rect(field, tokens.background);
            ctx.recorder
                .stroke_rect(field, border_of(tokens, tokens.background), 1.0);
        }
        ctx.recorder
            .draw_text("user@example.com", Point::new(36.0, 56.0), 13.0, tokens.on_background);
        ctx.recorder
            .draw_text("••••••••", Point::new(36.0, 102.0), 13.0, tokens.on_background);
        let submit = Rect::from_origin_size(Point::new(24.0, 130.0), Size::new(120.0, 36.0));
        ctx.recorder.fill_rect(submit, tokens.primary);
        ctx.recorder
            .draw_text("Sign in", Point::new(56.0, 154.0), 14.0, tokens.on_primary);
    }
}
