//! Navigation 分组的预览 painter（DC-17）。
//!
//! 包含：nav_demo / dialog_scaffold / popover / popup / toolbar。

use super::{PreviewPainter, border_of};
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::PaintCtx;

macro_rules! painter {
    ($name:ident) => {
        pub struct $name;
        impl PreviewPainter for $name {
            fn paint(&self, state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
                self.paint(state, tokens, ctx)
            }
        }
    };
}

painter!(NavPainter);
impl NavPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        for i in 0..3 {
            let offset = i as f32 * 16.0;
            let card = Rect::from_origin_size(
                Point::new(60.0 + offset, 30.0 + offset),
                Size::new(240.0 - offset, 120.0 - offset),
            );
            ctx.recorder.fill_rect(card, tokens.surface);
            ctx.recorder.stroke_rect(card, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                &format!("Screen {}", 3 - i),
                Point::new(76.0 + offset, 56.0 + offset),
                14.0,
                tokens.on_surface,
            );
        }
        ctx.recorder.draw_text(
            "Navigation stack: push / pop / modal present",
            Point::new(24.0, 170.0),
            11.0,
            tokens.on_background,
        );
    }
}

painter!(DialogPainter);
impl DialogPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let dlg = Rect::from_origin_size(Point::new(60.0, 20.0), Size::new(280.0, 150.0));
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(384.0, 192.0)),
            Color::rgb(0.0, 0.0, 0.0),
        );
        ctx.recorder.fill_rect(dlg, tokens.surface);
        ctx.recorder.stroke_rect(dlg, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Confirm", Point::new(76.0, 44.0), 15.0, tokens.on_surface);
        ctx.recorder
            .draw_text("Are you sure?", Point::new(76.0, 74.0), 13.0, tokens.on_background);
        let ok = Rect::from_origin_size(Point::new(76.0, 120.0), Size::new(110.0, 32.0));
        ctx.recorder.fill_rect(ok, tokens.primary);
        ctx.recorder
            .draw_text("OK", Point::new(116.0, 142.0), 13.0, tokens.on_primary);
        let cancel = Rect::from_origin_size(Point::new(200.0, 120.0), Size::new(110.0, 32.0));
        ctx.recorder.fill_rect(cancel, tokens.background);
        ctx.recorder
            .stroke_rect(cancel, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("Cancel", Point::new(228.0, 142.0), 13.0, tokens.on_background);
    }
}

painter!(PopoverPainter);
impl PopoverPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let anchor = Rect::from_origin_size(Point::new(60.0, 70.0), Size::new(120.0, 32.0));
        ctx.recorder.fill_rect(anchor, tokens.surface);
        ctx.recorder.stroke_rect(anchor, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Share ▾", Point::new(82.0, 92.0), 13.0, tokens.on_surface);
        let pop = Rect::from_origin_size(Point::new(60.0, 20.0), Size::new(200.0, 44.0));
        ctx.recorder.fill_rect(pop, tokens.surface);
        ctx.recorder.stroke_rect(pop, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("Copy link", Point::new(76.0, 42.0), 13.0, tokens.on_surface);
    }
}

painter!(PopupPainter);
impl PopupPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 4.0), Size::new(384.0, 192.0)),
            Color::rgb(0.0, 0.0, 0.0),
        );
        let popup = Rect::from_origin_size(Point::new(40.0, 30.0), Size::new(320.0, 140.0));
        ctx.recorder.fill_rect(popup, tokens.surface);
        ctx.recorder.stroke_rect(popup, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Popup (modal)", Point::new(56.0, 54.0), 15.0, tokens.on_surface);
        ctx.recorder.draw_text(
            "Blocks underlying UI until dismissed",
            Point::new(56.0, 80.0),
            12.0,
            tokens.on_background,
        );
    }
}

painter!(ToolbarPainter);
impl ToolbarPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let bar = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 44.0));
        ctx.recorder.fill_rect(bar, tokens.surface);
        ctx.recorder.stroke_rect(bar, border_of(tokens, tokens.surface), 1.0);
        let icons = ["◀", "▶", "⟳", "⌂", "⋮"];
        for (i, icon) in icons.iter().enumerate() {
            let x = 40.0 + i as f32 * 64.0;
            ctx.recorder
                .draw_text(icon, Point::new(x, 58.0), 18.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Toolbar: row of IconButtons with optional overflow menu",
            Point::new(24.0, 100.0),
            11.0,
            tokens.on_background,
        );
    }
}
