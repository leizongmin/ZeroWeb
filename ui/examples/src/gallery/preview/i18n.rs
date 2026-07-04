//! i18n 分组的预览 painter（DC-17）。
//!
//! 包含：i18n_demo —— 多语言样例文字。

use super::PreviewPainter;
use zero_ui_core::geometry::Point;
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::PaintCtx;

pub struct I18nPainter;
impl PreviewPainter for I18nPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let rows = [
            ("English", "Hello, world!"),
            ("中文", "你好，世界！"),
            ("RTL", "مرحبا بالعالم"),
        ];
        for (i, (lang, sample)) in rows.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            ctx.recorder
                .draw_text(lang, Point::new(24.0, y + 20.0), 13.0, tokens.on_background);
            ctx.recorder
                .draw_text(sample, Point::new(160.0, y + 20.0), 13.0, tokens.on_surface);
        }
    }
}
