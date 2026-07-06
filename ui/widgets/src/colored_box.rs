//! ColoredBox — 纯彩色矩形 widget（P2-15 视觉精度恢复）。
//!
//! 用途：把 badge 计数、progress 进度填充、status bubble 状态色块、icon button 字形背景
//! 这类原本用 ASCII 字符（`[ok]`、`###`、`<`）拼出来的"伪图形"换成真彩色像素块。
//!
//! 设计：
//! - 整个 layout 区域填 `color` prop 指定的颜色；
//! - `color` 支持三种命名预设（`primary` / `success` / `warning` / `danger` / `muted`）以及
//!   `#rrggbb` 十六进制字面量；默认 `muted`（中性灰）。
//! - 不响应事件，不发出 action；交互仍由 sibling 的 Button / ToggleWidget 承担。

use zero_ui_core::action::EventResult;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Rect, Size};
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// ColoredBox 控件实例。
pub struct ColoredBox {
    /// `color` prop 原始字符串（命名预设或 `#rrggbb`）。
    color_raw: String,
    /// `width` prop：固定宽度（>0 时优先），否则吃满 max_width。
    width: f32,
    /// `height` prop：固定高度（>0 时优先），否则默认 24。
    height: f32,
    /// `radius` prop：圆角半径（默认 0 = 直角；>0 圆角，让 badge/dot 看起来更真实）。
    radius: f32,
    /// `pulse` prop：true 时让颜色按 sin(now_ms) 轻微振荡（animation_demo 用）。
    /// 需要 host 注入 now_ms；未注入则忽略。
    pulse: bool,
    /// 上次 layout 算出的尺寸；paint 据此填充（paint 不接收 size）。
    size: Size,
    /// 可选 a11y 标签（用于屏幕阅读器，不影响视觉）。
    label: String,
}

impl ColoredBox {
    pub fn new() -> ColoredBox {
        ColoredBox {
            color_raw: "muted".into(),
            width: 0.0,
            height: 0.0,
            radius: 0.0,
            pulse: false,
            size: Size::new(64.0, 24.0),
            label: String::new(),
        }
    }
}

impl Default for ColoredBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ColoredBox {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut changed = false;
        if let Some(zero_ui_core::binding::Value::Text(raw)) = props.get("color")
            && raw != &self.color_raw
        {
            self.color_raw = raw.clone();
            changed = true;
        }
        if let Some(zero_ui_core::binding::Value::Float(w)) = props.get("width") {
            let w = *w as f32;
            if w != self.width {
                self.width = w;
                // 宽度变化要重 layout。
                *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
            }
        }
        if let Some(zero_ui_core::binding::Value::Float(h)) = props.get("height") {
            let h = *h as f32;
            if h != self.height {
                self.height = h;
                *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
            }
        }
        if let Some(zero_ui_core::binding::Value::Float(r)) = props.get("radius") {
            let r = *r as f32;
            if r != self.radius {
                self.radius = r;
                changed = true;
            }
        }
        if let Some(zero_ui_core::binding::Value::Bool(p)) = props.get("pulse")
            && *p != self.pulse
        {
            self.pulse = *p;
            changed = true;
        }
        if let Some(zero_ui_core::binding::Value::Text(label)) = props.get("label")
            && label != &self.label
        {
            self.label = label.clone();
            changed = true;
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 优先 width/height prop；否则吃满 constraints。
        let w = if self.width > 0.0 {
            self.width.clamp(c.min_width, c.max_width)
        } else {
            c.max_width
        };
        let default_h = 24.0_f32;
        let h = if self.height > 0.0 { self.height } else { default_h };
        let h = h.clamp(c.min_height, c.max_height);
        self.size = Size::new(w, h);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(self.size);
        let rect = Rect::from_origin_size(zero_ui_core::geometry::Point::ZERO, size);
        let mut color = resolve_color(&self.color_raw, tokens);
        // P3-4-5：pulse 模式下让明度按 sin(now_ms / 600) 振荡 ±15%（连续动画）。
        // 需要外部 driver 推进 clock；若 now_ms 为 None（无动画时钟），直接画静态色。
        if self.pulse
            && let Some(now) = ctx.now_ms
        {
            let phase = (now as f32 / 600.0).sin(); // -1..1
            let lighten = 0.15 * phase;
            color = if lighten >= 0.0 {
                color.lighten(lighten)
            } else {
                color.darken(-lighten)
            };
            // 声明需要下一帧（动画未完成——永远不完成，直到 pulse=false）。
            ctx.request_frame();
        }
        // P3-4-1：radius>0 用 fill_rounded_rect，让徽标/状态点变圆形/胶囊。
        if self.radius > 0.0 {
            ctx.recorder.fill_rounded_rect(rect, self.radius, color);
        } else {
            ctx.recorder.fill_rect(rect, color);
        }
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        if self.label.is_empty() {
            return;
        }
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("colored_box"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::NONE,
            label: Some(SemanticsLabel::Literal(self.label.clone().into())),
            value: None,
            children: Vec::new(),
        });
    }
}

/// 把 `color` prop（命名预设或 `#rrggbb`）解析成 `Color`。
/// 命名预设从 `tokens` 派生，保证 light/dark 主题下对比度都合适。
fn resolve_color(raw: &str, tokens: &zero_ui_core::theme::SemanticTokens) -> Color {
    let from_u8 = |r: u8, g: u8, b: u8| Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    match raw {
        "primary" => tokens.primary,
        "success" => Color::rgb(0.20, 0.70, 0.35),
        "warning" => Color::rgb(0.95, 0.75, 0.20),
        "danger" => Color::rgb(0.85, 0.30, 0.30),
        "muted" => Color::rgb(
            tokens.on_background.r * 0.5 + tokens.background.r * 0.5,
            tokens.on_background.g * 0.5 + tokens.background.g * 0.5,
            tokens.on_background.b * 0.5 + tokens.background.b * 0.5,
        ),
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0x80);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0x80);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0x80);
            from_u8(r, g, b)
        }
        _ => Color::rgb(0.5, 0.5, 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_named_presets() {
        let t = zero_ui_core::theme::SemanticTokens::light();
        assert_eq!(resolve_color("success", &t), Color::rgb(0.20, 0.70, 0.35));
        assert_eq!(resolve_color("warning", &t), Color::rgb(0.95, 0.75, 0.20));
        assert_eq!(resolve_color("danger", &t), Color::rgb(0.85, 0.30, 0.30));
        assert_eq!(resolve_color("primary", &t), t.primary);
    }

    #[test]
    fn resolve_hex_literal() {
        let t = zero_ui_core::theme::SemanticTokens::light();
        let red = resolve_color("#ff0000", &t);
        assert_eq!(red, Color::rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn resolve_invalid_falls_back_to_gray() {
        let t = zero_ui_core::theme::SemanticTokens::light();
        let bad = resolve_color("not-a-color", &t);
        assert_eq!(bad, Color::rgb(0.5, 0.5, 0.5));
    }

    #[test]
    fn width_prop_drives_layout() {
        use zero_ui_core::geometry::Constraints;
        use zero_ui_core::widget::{LayoutCtx, Props, Widget};
        let mut b = ColoredBox::new();
        let mut props = Props::new();
        props.insert("width", zero_ui_core::binding::Value::Float(120.0));
        props.insert("height", zero_ui_core::binding::Value::Float(40.0));
        let mut inval = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        b.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut inval,
            },
            &props,
        );
        let size = b.layout(
            &mut LayoutCtx {
                scale_factor: 1.0,
                text_measure: None,
                font_metrics: None,
            },
            Constraints {
                min_width: 0.0,
                max_width: 1000.0,
                min_height: 0.0,
                max_height: 1000.0,
            },
        );
        assert_eq!(size.width, 120.0, "width prop 应被 layout 采纳");
        assert_eq!(size.height, 40.0, "height prop 应被 layout 采纳");
    }

    #[test]
    fn width_prop_default_fills_max() {
        use zero_ui_core::geometry::Constraints;
        use zero_ui_core::widget::{LayoutCtx, Widget};
        let mut b = ColoredBox::new();
        let size = b.layout(
            &mut LayoutCtx {
                scale_factor: 1.0,
                text_measure: None,
                font_metrics: None,
            },
            Constraints {
                min_width: 0.0,
                max_width: 300.0,
                min_height: 0.0,
                max_height: 1000.0,
            },
        );
        assert_eq!(size.width, 300.0, "无 width prop 时吃满 max_width");
        assert_eq!(size.height, 24.0, "默认高度 24");
    }
}
