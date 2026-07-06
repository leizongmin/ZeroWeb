//! CodeBlock — 通用代码块 widget（P3-6-2，从 gallery SourceCode 提升到 ui-sdk）。
//!
//! 显示带语法高亮的代码片段，支持 yaml/rust 两种语言。卡片背景 + 按行渲染 + 段间 measure_text 推进。
//! 不响应事件（纯展示）。

use zero_ui_core::action::EventResult;
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::prop_keys;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

use crate::highlight::{highlight_rust, highlight_yaml, token_color};

/// 代码块 widget。
pub struct CodeBlock {
    /// `source` prop：代码内容。
    source: String,
    /// `lang` prop：语言（"yaml" / "rust" / 其它；未知语言用默认色）。
    lang: String,
    /// 上次 layout 算出的尺寸。
    size: Size,
}

impl Default for CodeBlock {
    fn default() -> Self {
        CodeBlock::new()
    }
}

impl CodeBlock {
    pub fn new() -> CodeBlock {
        CodeBlock {
            source: String::new(),
            lang: String::from("yaml"),
            size: Size::new(400.0, 80.0),
        }
    }
}

impl Widget for CodeBlock {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut source_changed = false;
        let mut lang_changed = false;
        if let Some(Value::Text(s)) = props.get(prop_keys::SOURCE)
            && s != &self.source
        {
            self.source = s.clone();
            source_changed = true;
        }
        if let Some(Value::Text(l)) = props.get(prop_keys::LANG)
            && l != &self.lang
        {
            self.lang = l.clone();
            lang_changed = true;
        }
        if source_changed {
            *ctx.invalidation |= InvalidationFlags::NEEDS_LAYOUT;
        } else if lang_changed {
            *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 行数决定高度；宽度吃满 max。
        let line_h = 16.0_f32;
        let lines = self.source.lines().count().max(1) as f32;
        let h = (lines * line_h).clamp(c.min_height, c.max_height).max(40.0);
        let w = c.max_width;
        self.size = Size::new(w, h);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(self.size);
        // 卡片背景：比 surface 略亮（light）/ 略深（dark）的混合色。
        let card = Color::rgb(
            tokens.surface.r * 0.97 + tokens.background.r * 0.03,
            tokens.surface.g * 0.97 + tokens.background.g * 0.03,
            tokens.surface.b * 0.97 + tokens.background.b * 0.03,
        );
        ctx.recorder.fill_rect(
            Rect::from_origin_size(Point::new(8.0, 0.0), Size::new(size.width - 16.0, size.height)),
            card,
        );

        // 语法高亮 token 渲染：token_color 与 on_background 混合保证两种主题下可读。
        let base = tokens.on_background;
        let mix = |c: (f32, f32, f32)| {
            Color::rgb(
                c.0 * 0.85 + base.r * 0.15,
                c.1 * 0.85 + base.g * 0.15,
                c.2 * 0.85 + base.b * 0.15,
            )
        };
        let code_tokens = match self.lang.as_str() {
            "yaml" => highlight_yaml(&self.source),
            "rust" => highlight_rust(&self.source),
            _ => vec![(&self.source as &str, "default")],
        };

        // 按字符遍历，遇换行重置 x；同色段累计成字符串，整段一次 draw_text 调用。
        let mut x = 16.0_f32;
        let mut y = 14.0_f32;
        let line_h = 16.0_f32;
        for (text, kind) in &code_tokens {
            let color = mix(token_color(kind));
            let mut first_segment = true;
            for segment in text.split('\n') {
                if !first_segment {
                    x = 16.0;
                    y += line_h;
                }
                first_segment = false;
                if segment.is_empty() {
                    continue;
                }
                ctx.recorder.draw_text(segment, Point::new(x, y), 12.0, color);
                x += ctx.recorder.measure_text(segment, 12.0);
            }
        }
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("code_block"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::NONE,
            label: Some(SemanticsLabel::Literal(self.source.clone().into())),
            value: None,
            children: Vec::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::highlight::{highlight_rust, highlight_yaml};

    #[test]
    fn code_block_default_lang_yaml() {
        let cb = CodeBlock::new();
        assert_eq!(cb.lang, "yaml");
    }

    #[test]
    fn layout_height_grows_with_line_count() {
        let mut cb = CodeBlock::new();
        cb.source = "line1\nline2\nline3".into();
        let s = cb.layout(
            &mut LayoutCtx {
                scale_factor: 1.0,
                text_measure: None,
                font_metrics: None,
            },
            Constraints {
                min_width: 0.0,
                max_width: 400.0,
                min_height: 0.0,
                max_height: 1000.0,
            },
        );
        // 3 行 × 16 = 48
        assert!((s.height - 48.0).abs() < 0.5, "3 行高度应≈48, got {}", s.height);
    }

    #[test]
    fn highlight_yaml_and_rust_dont_panic() {
        // 简单回归：高亮不应 panic。
        let _ = highlight_yaml("key: value\n# comment");
        let _ = highlight_rust("let x = 1; // hi");
    }
}
