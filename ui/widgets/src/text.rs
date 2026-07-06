//! Text — 通用文本 widget（P3-6-1）。
//!
//! 显示一行文本，支持字号、颜色（命名预设或 #rrggbb）、对齐 props。
//! 默认 14px、on_surface 色、左对齐。
//!
//! 替代 gallery 内部的 SourceLabel。区别：
//! - SourceLabel 硬编码 12px / 灰色 / 固定 baseline 16。
//! - Text 支持 size / color / align / weight props，更通用。
//! - 默认值与 SourceLabel 兼容（12px、on_surface 60% 混合 = 灰色）。
//!
//! 不响应事件（纯展示）。多行文本用 CodeBlock（按 `\n` 分行渲染）。

use zero_ui_core::action::EventResult;
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::prop_keys;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// 文本对齐方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// 左对齐（默认）。
    #[default]
    Left,
    /// 居中。
    Center,
    /// 右对齐。
    Right,
}

/// 通用文本 widget。
///
/// 显示文本，支持自动换行（按 `max_width` word-wrap）和 `\n` 硬换行。
/// 不响应事件。
pub struct Text {
    /// `text` prop：要显示的内容。
    text: String,
    /// `size` prop：字号（默认 14）。
    size_px: f32,
    /// `color` prop：命名预设（primary/on_surface/muted 等）或 `#rrggbb`；默认 None = on_surface。
    color_raw: Option<String>,
    /// `align` prop：left/center/right（默认 left）。
    align: TextAlign,
    /// `weight` prop：normal/bold（默认 normal）。bold 用更大字号近似（无字体权重 API）。
    bold: bool,
    /// 上次 layout 算出的尺寸。
    size: Size,
    /// P1-3：layout 阶段计算好的行列表（已 word-wrap + 按硬换行分割），paint 直接用。
    /// 避免 paint 阶段没有 text_measure 的问题。
    laid_lines: Vec<String>,
}

impl Default for Text {
    fn default() -> Self {
        Text::new()
    }
}

impl Text {
    pub fn new() -> Text {
        Text {
            text: String::new(),
            size_px: 14.0,
            color_raw: None,
            align: TextAlign::Left,
            bold: false,
            size: Size::new(0.0, 20.0),
            laid_lines: Vec::new(),
        }
    }
}

impl Widget for Text {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut layout_changed = false;
        let mut paint_changed = false;

        if let Some(Value::Text(t)) = props.get(prop_keys::TEXT)
            && t != &self.text
        {
            self.text = t.clone();
            layout_changed = true; // 文本变了可能影响宽度
        }
        if let Some(Value::Float(s)) = props.get("size") {
            let s = *s as f32;
            if s != self.size_px && s > 0.0 {
                self.size_px = s;
                layout_changed = true;
            }
        }
        match props.get("color") {
            Some(Value::Text(c)) => {
                if self.color_raw.as_deref() != Some(c.as_str()) {
                    self.color_raw = Some(c.clone());
                    paint_changed = true;
                }
            }
            _ => {
                if self.color_raw.is_some() {
                    self.color_raw = None;
                    paint_changed = true;
                }
            }
        }
        if let Some(Value::Text(a)) = props.get("align") {
            let new_align = match a.as_str() {
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                _ => TextAlign::Left,
            };
            if new_align != self.align {
                self.align = new_align;
                paint_changed = true;
            }
        }
        if let Some(Value::Text(w)) = props.get("weight") {
            let new_bold = w == "bold";
            if new_bold != self.bold {
                self.bold = new_bold;
                paint_changed = true;
            }
        }

        if layout_changed {
            *ctx.invalidation |= InvalidationFlags::NEEDS_LAYOUT;
        } else if paint_changed {
            *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // P1-3：自动换行。按 max_width 用 measure_text 计算 word-wrap，
        // 同时保留 \n 硬换行。结果存 self.laid_lines 供 paint 使用。
        let line_h = self.size_px * 1.4;
        let max_w = c.max_width;
        let mut lines: Vec<String> = Vec::new();
        for hard_line in self.text.split('\n') {
            if hard_line.is_empty() {
                lines.push(String::new());
                continue;
            }
            // 简单 word-wrap：按空格分词，累加直到超 max_w。
            // CJK 无空格分隔 → 按字符回退（每个 CJK 字符可单独成行）。
            let mut cur = String::new();
            let mut cur_w = 0.0_f32;
            for token in tokenize_line(hard_line) {
                let token_w = ctx.measure_text(&token, self.size_px).width;
                if cur.is_empty() {
                    // 单个 token 就超宽：硬切（按字符）。
                    if token_w > max_w && token.chars().count() > 1 {
                        for ch in token.chars() {
                            let ch_w = ctx.measure_text(&ch.to_string(), self.size_px).width;
                            if !cur.is_empty() && cur_w + ch_w > max_w {
                                lines.push(std::mem::take(&mut cur));
                                cur_w = 0.0;
                            }
                            cur.push(ch);
                            cur_w += ch_w;
                        }
                        continue;
                    }
                    cur.push_str(&token);
                    cur_w = token_w;
                } else if cur_w + token_w > max_w {
                    lines.push(std::mem::take(&mut cur));
                    cur = token;
                    cur_w = token_w;
                } else {
                    cur.push_str(&token);
                    cur_w += token_w;
                }
            }
            if !cur.is_empty() {
                lines.push(cur);
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        let line_count = lines.len().max(1) as f32;
        let h = (line_count * line_h).clamp(c.min_height, c.max_height);
        let w = c.max_width;
        self.size = Size::new(w, h);
        self.laid_lines = lines;
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let color = resolve_text_color(&self.color_raw, tokens);
        // bold 近似：字号 +1（无真实字体权重 API）。
        let size_px = if self.bold { self.size_px + 1.0 } else { self.size_px };
        let line_h = self.size_px * 1.4;
        let ascent = size_px * 0.92;
        // P1-3：用 layout 算好的 laid_lines（已 word-wrap）。
        for (i, line) in self.laid_lines.iter().enumerate() {
            let baseline_y = (i as f32 + 1.0) * line_h - (line_h - ascent);
            // 水平对齐：用真实 measure_text 算宽度（layout 算过，paint 复算保持一致）。
            let text_w = ctx.measure_text(line, size_px).width;
            let x = match self.align {
                TextAlign::Left => 0.0,
                TextAlign::Center => ((self.size.width - text_w) / 2.0).max(0.0),
                TextAlign::Right => (self.size.width - text_w).max(0.0),
            };
            ctx.recorder.draw_text(line, Point::new(x, baseline_y), size_px, color);
        }
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("text"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::NONE,
            label: Some(SemanticsLabel::Literal(self.text.clone().into())),
            value: None,
            children: Vec::new(),
        });
    }
}

/// P1-3：把一行文本切成 word-wrap 友好的 token（保留空格）。
///
/// 拉丁文按空格分词；CJK 字符（每个字独立）作为独立 token，可在任意位置断行。
fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    for ch in line.chars() {
        let is_cjk = matches!(ch as u32, 0x3000..=0x9FFF | 0xAC00..=0xD7AF | 0xFF00..=0xFFEF);
        if is_cjk {
            if !buf.is_empty() {
                tokens.push(std::mem::take(&mut buf));
            }
            tokens.push(ch.to_string());
        } else if ch == ' ' {
            if !buf.is_empty() {
                tokens.push(std::mem::take(&mut buf));
            }
            tokens.push(" ".to_string());
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        tokens.push(buf);
    }
    tokens
}

/// 与 Icon / ColoredBox 一致的命名预设解析。
fn resolve_text_color(raw: &Option<String>, tokens: &zero_ui_core::theme::SemanticTokens) -> Color {
    match raw.as_deref() {
        Some("primary") => tokens.primary,
        Some("on_primary") => tokens.on_primary,
        Some("on_surface") => tokens.on_surface,
        Some("success") => Color::rgb(0.20, 0.70, 0.35),
        Some("warning") => Color::rgb(0.95, 0.75, 0.20),
        Some("danger") => Color::rgb(0.85, 0.30, 0.30),
        // 默认 / "muted" / "secondary"：on_background 60% + background 40%（灰色辅助文本）。
        Some("muted") | Some("secondary") | None => Color::rgb(
            tokens.on_background.r * 0.6 + tokens.background.r * 0.4,
            tokens.on_background.g * 0.6 + tokens.background.g * 0.4,
            tokens.on_background.b * 0.6 + tokens.background.b * 0.4,
        ),
        Some(hex) if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0x80);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0x80);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0x80);
            Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        Some(_) => tokens.on_surface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Constraints;
    use zero_ui_core::widget::{LayoutCtx, Widget};

    #[test]
    fn text_default_size_and_color() {
        let t = Text::new();
        assert_eq!(t.size_px, 14.0);
        assert!(t.color_raw.is_none());
        assert_eq!(t.align, TextAlign::Left);
        assert!(!t.bold);
    }

    #[test]
    fn text_layout_height_grows_with_lines() {
        // 单行 height = 1 * 14 * 1.4 = 19.6
        // 双行 height = 2 * 14 * 1.4 = 39.2
        let mut t = Text::new();
        let mut props = Props::new();
        props.insert(prop_keys::TEXT, Value::Text("one\ntwo".into()));
        let mut inval = InvalidationFlags::CLEAN;
        t.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut inval,
            },
            &props,
        );
        let s = t.layout(
            &mut LayoutCtx {
                scale_factor: 1.0,
                text_measure: None,
                font_metrics: None,
            },
            Constraints {
                min_width: 0.0,
                max_width: 200.0,
                min_height: 0.0,
                max_height: 1000.0,
            },
        );
        assert!((s.height - 39.2).abs() < 0.5, "双行高度应≈39.2, got {}", s.height);
    }

    #[test]
    fn align_parse_from_prop() {
        let mut t = Text::new();
        let mut props = Props::new();
        props.insert("align", Value::Text("center".into()));
        let mut inval = InvalidationFlags::CLEAN;
        t.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut inval,
            },
            &props,
        );
        assert_eq!(t.align, TextAlign::Center);
    }

    #[test]
    fn resolve_color_named_presets() {
        let tokens = zero_ui_core::theme::SemanticTokens::light();
        assert_eq!(resolve_text_color(&Some("primary".into()), &tokens), tokens.primary);
        assert_eq!(
            resolve_text_color(&Some("on_surface".into()), &tokens),
            tokens.on_surface
        );
        // muted 应是混合灰。
        let muted = resolve_text_color(&Some("muted".into()), &tokens);
        assert_ne!(muted, tokens.on_surface);
        assert_ne!(muted, tokens.background);
    }

    #[test]
    fn resolve_color_hex_literal() {
        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let c = resolve_text_color(&Some("#ff8800".into()), &tokens);
        // r=255/255=1.0, g=136/255≈0.533, b=0/255=0
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.533).abs() < 0.02);
        assert!((c.b - 0.0).abs() < 0.01);
    }
}
