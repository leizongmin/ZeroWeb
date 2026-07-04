//! Widgets 分组的预览 painter（DC-17）。
//!
//! 包含：button / toggle / icon_button / badge / progress / text_input / tabs /
//! tooltip / list_view / menu / search_field / status_bubble。
//! 视觉规则与原 `DemoPreview::paint_*_preview` 一致，仅迁移到独立 painter struct
//! 以便 DemoPreview 主体保持精简、易于新增 demo。

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

painter!(ButtonPainter);
impl ButtonPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let labels = ["Default", "Pressed", "Disabled"];
        let colors = [
            tokens.surface,
            tokens.primary,
            Color::rgb(
                tokens.surface.r * 0.7 + tokens.background.r * 0.3,
                tokens.surface.g * 0.7 + tokens.background.g * 0.3,
                tokens.surface.b * 0.7 + tokens.background.b * 0.3,
            ),
        ];
        let fg_colors = [tokens.on_surface, tokens.on_primary, tokens.on_surface];
        for (i, label) in labels.iter().enumerate() {
            let x = 24.0 + i as f32 * 130.0;
            let rect = Rect::from_origin_size(Point::new(x, 40.0), Size::new(110.0, 36.0));
            ctx.recorder.fill_rect(rect, colors[i]);
            ctx.recorder.stroke_rect(rect, border_of(tokens, colors[i]), 1.0);
            ctx.recorder
                .draw_text(label, Point::new(x + 12.0, 64.0), 14.0, fg_colors[i]);
        }
        ctx.recorder.draw_text(
            "Click → emit Action (state held by parent app)",
            Point::new(24.0, 110.0),
            12.0,
            tokens.on_surface,
        );
    }
}

painter!(TogglePainter);
impl TogglePainter {
    fn paint(&self, state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let labels = ["On/Off (interactive)", "On/Off (interactive)", "Disabled"];
        for (i, label) in labels.iter().enumerate() {
            let y = 28.0 + i as f32 * 40.0;
            ctx.recorder
                .draw_text(label, Point::new(24.0, y + 18.0), 13.0, tokens.on_surface);
            let track_x = 200.0;
            let is_on = i < 2 && (state & (1 << i)) != 0;
            let track_color = if i == 2 {
                Color::rgb(
                    tokens.surface.r * 0.6 + tokens.background.r * 0.4,
                    tokens.surface.g * 0.6 + tokens.background.g * 0.4,
                    tokens.surface.b * 0.6 + tokens.background.b * 0.4,
                )
            } else if is_on {
                tokens.primary
            } else {
                Color::rgb(
                    tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
                    tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
                    tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
                )
            };
            let track_rect = Rect::from_origin_size(Point::new(track_x, y), Size::new(48.0, 24.0));
            ctx.recorder.fill_rect(track_rect, track_color);
            let thumb_x = if is_on { track_x + 26.0 } else { track_x + 2.0 };
            let thumb_rect = Rect::from_origin_size(Point::new(thumb_x, y + 2.0), Size::new(20.0, 20.0));
            ctx.recorder.fill_rect(thumb_rect, tokens.background);
            ctx.recorder.stroke_rect(thumb_rect, tokens.on_background, 1.0);
        }
    }
}

painter!(IconButtonPainter);
impl IconButtonPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let icons = ["◀", "▶", "⟳", "✕"];
        for (i, icon) in icons.iter().enumerate() {
            let x = 24.0 + i as f32 * 70.0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(56.0, 40.0));
            ctx.recorder.fill_rect(rect, tokens.surface);
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder
                .draw_text(icon, Point::new(x + 18.0, 56.0), 18.0, tokens.on_surface);
        }
        ctx.recorder.draw_text(
            "Icon-only buttons; emit action on click",
            Point::new(24.0, 100.0),
            12.0,
            tokens.on_surface,
        );
    }
}

painter!(BadgePainter);
impl BadgePainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let samples = ["3", "99+"];
        for (i, count) in samples.iter().enumerate() {
            let x = 32.0 + i as f32 * 130.0;
            let icon_rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(56.0, 56.0));
            ctx.recorder.fill_rect(icon_rect, tokens.surface);
            ctx.recorder
                .stroke_rect(icon_rect, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder
                .draw_text("▣", Point::new(x + 18.0, 66.0), 22.0, tokens.on_surface);
            let badge_rect = Rect::from_origin_size(Point::new(x + 40.0, 22.0), Size::new(28.0, 20.0));
            ctx.recorder.fill_rect(badge_rect, tokens.error);
            ctx.recorder
                .draw_text(count, Point::new(x + 45.0, 36.0), 12.0, tokens.on_primary);
        }
        ctx.recorder.draw_text(
            "Count badge clamped to max (default 99)",
            Point::new(24.0, 110.0),
            12.0,
            tokens.on_surface,
        );
    }
}

painter!(ProgressPainter);
impl ProgressPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let fracs = [0.3_f32, 0.7_f32];
        for (i, frac) in fracs.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let track = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 12.0));
            ctx.recorder.fill_rect(track, tokens.background);
            let fill_w = track.size.width * frac;
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::new(24.0, y), Size::new(fill_w, 12.0)),
                tokens.primary,
            );
            ctx.recorder
                .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
        }
        // indeterminate：动画条（静态位置占位）。
        let y = 30.0 + 2.0 * 32.0;
        let track = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 12.0));
        ctx.recorder.fill_rect(track, tokens.background);
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(60.0, y), Size::new(120.0, 12.0)),
            tokens.primary,
        );
        ctx.recorder
            .stroke_rect(track, border_of(tokens, tokens.background), 1.0);
        let labels = ["Determinate 30%", "Determinate 70%", "Indeterminate"];
        for (i, label) in labels.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            ctx.recorder
                .draw_text(label, Point::new(24.0, y + 28.0), 11.0, tokens.on_background);
        }
    }
}

painter!(TextInputPainter);
impl TextInputPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let placeholder_rect = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(placeholder_rect, tokens.background);
        ctx.recorder
            .stroke_rect(placeholder_rect, border_of(tokens, tokens.background), 1.0);
        let ph_color = Color::rgb(
            tokens.on_background.r * 0.4 + tokens.background.r * 0.6,
            tokens.on_background.g * 0.4 + tokens.background.g * 0.6,
            tokens.on_background.b * 0.4 + tokens.background.b * 0.6,
        );
        ctx.recorder
            .draw_text("Placeholder...", Point::new(36.0, 54.0), 14.0, ph_color);

        let filled_rect = Rect::from_origin_size(Point::new(24.0, 76.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(filled_rect, tokens.background);
        ctx.recorder.stroke_rect(filled_rect, tokens.primary, 2.0);
        ctx.recorder
            .draw_text("Hello", Point::new(36.0, 100.0), 14.0, tokens.on_background);
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(70.0, 80.0), Size::new(1.5, 28.0)),
            tokens.primary,
        );
    }
}

painter!(TabsPainter);
impl TabsPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let tabs = ["General", "Privacy", "Security"];
        for (i, label) in tabs.iter().enumerate() {
            let x = 24.0 + i as f32 * 110.0;
            let is_selected = i == 0;
            let rect = Rect::from_origin_size(Point::new(x, 30.0), Size::new(100.0, 36.0));
            ctx.recorder
                .fill_rect(rect, if is_selected { tokens.surface } else { tokens.background });
            ctx.recorder.stroke_rect(rect, border_of(tokens, tokens.surface), 1.0);
            if is_selected {
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(x, 64.0), Size::new(100.0, 2.0)),
                    tokens.primary,
                );
            }
            ctx.recorder.draw_text(
                label,
                Point::new(x + 12.0, 54.0),
                13.0,
                if is_selected {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
        }
        let content = Rect::from_origin_size(Point::new(24.0, 70.0), Size::new(336.0, 60.0));
        ctx.recorder.fill_rect(content, tokens.surface);
        ctx.recorder
            .stroke_rect(content, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Selected tab content", Point::new(36.0, 100.0), 13.0, tokens.on_surface);
    }
}

painter!(TooltipPainter);
impl TooltipPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let btn = Rect::from_origin_size(Point::new(60.0, 50.0), Size::new(80.0, 36.0));
        ctx.recorder.fill_rect(btn, tokens.surface);
        ctx.recorder.stroke_rect(btn, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("Hover me", Point::new(72.0, 74.0), 13.0, tokens.on_surface);
        let tip = Rect::from_origin_size(Point::new(70.0, 8.0), Size::new(110.0, 28.0));
        ctx.recorder.fill_rect(tip, tokens.on_background);
        ctx.recorder
            .draw_text("Helpful hint", Point::new(80.0, 26.0), 12.0, tokens.background);
        ctx.recorder.draw_text(
            "Tooltip anchored above target on hover (delay 300ms)",
            Point::new(24.0, 110.0),
            11.0,
            tokens.on_background,
        );
    }
}

painter!(ListViewPainter);
impl ListViewPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        for i in 0..5 {
            let y = 20.0 + i as f32 * 32.0;
            let row = Rect::from_origin_size(Point::new(24.0, y), Size::new(360.0, 30.0));
            let is_selected = i == 1;
            if is_selected {
                let washed = Color::rgb(
                    tokens.primary.r * 0.3 + tokens.surface.r * 0.7,
                    tokens.primary.g * 0.3 + tokens.surface.g * 0.7,
                    tokens.primary.b * 0.3 + tokens.surface.b * 0.7,
                );
                ctx.recorder.fill_rect(row, washed);
            }
            ctx.recorder.stroke_rect(row, border_of(tokens, tokens.surface), 1.0);
            ctx.recorder.draw_text(
                &format!("Item {}", i + 1),
                Point::new(36.0, y + 20.0),
                13.0,
                if is_selected {
                    tokens.on_surface
                } else {
                    tokens.on_background
                },
            );
        }
    }
}

painter!(MenuPainter);
impl MenuPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let items = [
            "Open...  ⌘O",
            "Save      ⌘S",
            "Save As...",
            "──────────",
            "Exit      ⌘Q",
        ];
        let menu_rect = Rect::from_origin_size(Point::new(40.0, 20.0), Size::new(220.0, 160.0));
        ctx.recorder.fill_rect(menu_rect, tokens.surface);
        ctx.recorder
            .stroke_rect(menu_rect, border_of(tokens, tokens.surface), 1.0);
        for (i, item) in items.iter().enumerate() {
            let y = 20.0 + 8.0 + i as f32 * 30.0;
            if i == 0 {
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(40.0, y - 4.0), Size::new(220.0, 28.0)),
                    tokens.primary,
                );
                ctx.recorder
                    .draw_text(item, Point::new(56.0, y + 16.0), 13.0, tokens.on_primary);
            } else {
                ctx.recorder
                    .draw_text(item, Point::new(56.0, y + 16.0), 13.0, tokens.on_background);
            }
        }
    }
}

painter!(SearchFieldPainter);
impl SearchFieldPainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let field = Rect::from_origin_size(Point::new(24.0, 30.0), Size::new(360.0, 36.0));
        ctx.recorder.fill_rect(field, tokens.background);
        ctx.recorder
            .stroke_rect(field, border_of(tokens, tokens.background), 1.0);
        ctx.recorder
            .draw_text("🔍", Point::new(36.0, 54.0), 14.0, tokens.on_background);
        ctx.recorder
            .draw_text("compo", Point::new(64.0, 54.0), 14.0, tokens.on_background);
        let sugg = Rect::from_origin_size(Point::new(24.0, 70.0), Size::new(360.0, 60.0));
        ctx.recorder.fill_rect(sugg, tokens.surface);
        ctx.recorder.stroke_rect(sugg, border_of(tokens, tokens.surface), 1.0);
        ctx.recorder
            .draw_text("component gallery", Point::new(36.0, 90.0), 12.0, tokens.on_surface);
        ctx.recorder
            .draw_text("composer pattern", Point::new(36.0, 110.0), 12.0, tokens.on_surface);
    }
}

painter!(StatusBubblePainter);
impl StatusBubblePainter {
    fn paint(&self, _state: u64, tokens: &SemanticTokens, ctx: &mut PaintCtx) {
        let samples = [
            ("✓ Saved", tokens.primary, tokens.on_primary),
            ("! Pending", Color::rgb(0.9, 0.7, 0.2), Color::rgb(0.0, 0.0, 0.0)),
            ("✗ Failed", tokens.error, tokens.on_primary),
        ];
        for (i, (text, bg, fg)) in samples.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let bubble = Rect::from_origin_size(Point::new(24.0, y), Size::new(200.0, 24.0));
            ctx.recorder.fill_rect(bubble, *bg);
            ctx.recorder.draw_text(text, Point::new(36.0, y + 16.0), 12.0, *fg);
        }
    }
}
