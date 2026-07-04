//! Theme 分组的预览 painter（DC-17）。
//!
//! 包含：theme_demo —— semantic token 色板列表。

use super::PreviewPainter;
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::PaintCtx;

pub struct ThemePainter;
impl PreviewPainter for ThemePainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let entries: &[(&str, Color)] = &[
            ("background", tokens.background),
            ("surface", tokens.surface),
            ("primary", tokens.primary),
            ("on_primary", tokens.on_primary),
            ("on_background", tokens.on_background),
            ("error", tokens.error),
        ];
        for (i, (name, color)) in entries.iter().enumerate() {
            let y = 20.0 + i as f32 * 22.0;
            let swatch = Rect::from_origin_size(Point::new(24.0, y), Size::new(32.0, 16.0));
            ctx.recorder.fill_rect(swatch, *color);
            ctx.recorder.stroke_rect(
                swatch,
                Color::rgb(
                    tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
                    tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
                    tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
                ),
                1.0,
            );
            ctx.recorder
                .draw_text(name, Point::new(70.0, y + 14.0), 12.0, tokens.on_background);
        }
    }
}
