//! Patterns 分组的预览 painter（DC-17）。
//!
//! 包含：collection_demo / dsl_demo / data_list / command_palette / tab_bar。

use super::{PreviewPainter, border_of, y_text_center};
use zero_ui_core::geometry::{Point, Rect, Size};
use zero_ui_core::theme::SemanticTokens;
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

painter!(CollectionPainter);
impl CollectionPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        for row in 0..3 {
            for col in 0..4 {
                let x = 24.0 + col as f32 * 92.0;
                let y = 20.0 + row as f32 * 56.0;
                let cell = Rect::from_origin_size(Point::new(x, y), Size::new(84.0, 48.0));
                let is_selected = row == 1 && col == 2;
                ctx.recorder
                    .fill_rect(cell, if is_selected { tokens.primary } else { tokens.surface });
                ctx.recorder.stroke_rect(cell, border_of(tokens, tokens.surface), 1.0);
                ctx.recorder.draw_text(
                    &format!("{row}-{col}"),
                    Point::new(x + 28.0, y + 28.0),
                    11.0,
                    if is_selected {
                        tokens.on_primary
                    } else {
                        tokens.on_background
                    },
                );
            }
        }
    }
}

painter!(DslPainter);
impl DslPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let left = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(170.0, 110.0));
        let right = Rect::from_origin_size(Point::new(210.0, 20.0), Size::new(170.0, 110.0));
        ctx.recorder.fill_rect(left, tokens.background);
        ctx.recorder.fill_rect(right, tokens.background);
        ctx.recorder
            .stroke_rect(left, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .stroke_rect(right, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("Row:", Point::new(32.0, 38.0), 11.0, tokens.primary);
        ctx.recorder
            .draw_text("  - Text: Hi", Point::new(32.0, 56.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("  - Spacer", Point::new(32.0, 74.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("Row::new()", Point::new(218.0, 38.0), 11.0, tokens.primary);
        ctx.recorder
            .draw_text("  .child(Text)", Point::new(218.0, 56.0), 11.0, tokens.on_background);
        ctx.recorder
            .draw_text("  .child(Spacer)", Point::new(218.0, 74.0), 11.0, tokens.on_background);
    }
}

painter!(DataListPainter);
impl DataListPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        for i in 0..5 {
            let y = 20.0 + i as f32 * 32.0;
            let row = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 30.0));
            ctx.recorder.fill_rect(row, tokens.background);
            ctx.recorder.stroke_rect(row, border_of(tokens, tokens.background), 1.0);
            match i {
                0 => {
                    ctx.recorder.fill_rect(
                        Rect::from_origin_size(Point::new(36.0, y + 13.0), Size::new(60.0, 6.0)),
                        tokens.surface,
                    );
                }
                3 => {
                    ctx.recorder
                        .draw_text("⚠ Failed to load", Point::new(36.0, y + 20.0), 12.0, tokens.error);
                }
                _ => {
                    ctx.recorder.draw_text(
                        &format!("Row {}", i),
                        Point::new(36.0, y + 20.0),
                        12.0,
                        tokens.on_background,
                    );
                }
            }
        }
    }
}

painter!(CommandPalettePainter);
impl CommandPalettePainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let input = Rect::from_origin_size(Point::new(24.0, 20.0), Size::new(360.0, 32.0));
        ctx.recorder.fill_rect(input, tokens.background);
        ctx.recorder.stroke_rect(input, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("> opa", Point::new(36.0, 42.0), 13.0, tokens.on_background);
        let list = Rect::from_origin_size(Point::new(24.0, 56.0), Size::new(360.0, 120.0));
        ctx.recorder.fill_rect(list, tokens.surface);
        ctx.recorder.stroke_rect(list, border_of(tokens, tokens.surface), 1.0);
        let cmds = ["file.open  Open File", "file.save  Save", "go.back  Go Back"];
        for (i, c) in cmds.iter().enumerate() {
            let y = 56.0 + 8.0 + i as f32 * 30.0;
            if i == 0 {
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(24.0, y - 4.0), Size::new(360.0, 28.0)),
                    tokens.primary,
                );
                ctx.recorder
                    .draw_text(c, Point::new(36.0, y + 16.0), 12.0, tokens.on_primary);
            } else {
                ctx.recorder
                    .draw_text(c, Point::new(36.0, y + 16.0), 12.0, tokens.on_background);
            }
        }
    }
}

painter!(TabBarPainter);
impl TabBarPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let tabs = ["Home", "Docs", "About"];
        for (i, t) in tabs.iter().enumerate() {
            let x = 24.0 + i as f32 * 120.0;
            let is_sel = i == 0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(110.0, 32.0));
            ctx.recorder
                .fill_rect(rect, if is_sel { tokens.surface } else { tokens.background });
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                t,
                Point::new(x + 12.0, y_text_center(30.0, 32.0)),
                13.0,
                if is_sel {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
            ctx.recorder
                .draw_text("×", Point::new(x + 88.0, 52.0), 14.0, tokens.on_background);
        }
        ctx.recorder.draw_text(
            "TabBar: selected tab + per-tab close button + reorderable",
            Point::new(24.0, 90.0),
            11.0,
            tokens.on_background,
        );
    }
}
