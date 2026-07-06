//! Toggle — 开关控件（spec FR-009；权限/设置项等用）。
//!
//! 双态（on/off）控件。本模块提供两层 API：
//! - **数据模型** [`Toggle`] / [`Toggle::flip`]（向后兼容，spec FR-003 单向数据流）。
//! - **完整 Widget** [`ToggleWidget`]：实现 `Widget` trait，处理 hover/pressed、paint、semantics、
//!   受控模式下从 props.checked 同步状态。点击 emit action 由应用回写。
//!
//! 推荐新代码用 [`ToggleSpec`] + [`ToggleWidget`] 组合（事件回路完整接入 host）。

use zero_ui_core::action::{ActionId, EventResult};
use zero_ui_core::event::{PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// 开关声明（props；传给 [`ToggleWidget`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleSpec {
    pub checked: bool,
    pub action: ActionId,
    /// 可选标签（设置项文案；生产走 i18n message id）。
    pub label: Option<String>,
    pub enabled: bool,
}

impl ToggleSpec {
    pub fn new(checked: bool, action: &str) -> ToggleSpec {
        ToggleSpec {
            checked,
            action: ActionId::new(action),
            label: None,
            enabled: true,
        }
    }

    pub fn with_label(mut self, label: &str) -> ToggleSpec {
        self.label = Some(label.to_string());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> ToggleSpec {
        self.enabled = enabled;
        self
    }
}

/// 旧 Toggle 数据模型（保留以兼容 `permission_prompt` 等 patterns 调用方）。
///
/// 不实现 `Widget` trait——若需挂入 host 树请用 [`ToggleWidget`]。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toggle {
    pub checked: bool,
    pub action: ActionId,
    pub label: Option<String>,
}

impl Toggle {
    pub fn new(checked: bool, action: &str) -> Toggle {
        Toggle {
            checked,
            action: ActionId::new(action),
            label: None,
        }
    }

    pub fn with_label(mut self, label: &str) -> Toggle {
        self.label = Some(label.to_string());
        self
    }

    /// 翻转状态并返回要派发的 action（单向数据流：应用接收 action 后回写 checked）。
    pub fn flip(&mut self) -> ActionId {
        self.checked = !self.checked;
        self.action.clone()
    }

    /// 转换为 spec（用于构造 [`ToggleWidget`]）。
    pub fn to_spec(&self) -> ToggleSpec {
        let mut s = ToggleSpec::new(self.checked, self.action.0.as_str());
        s.label = self.label.clone();
        s
    }
}

/// Toggle Widget 实例（retained 临时 hover/pressed 态）。
///
/// 完整接入 host：event 派发、layout/paint、semantics、focusable。点击 emit `spec.action`，
/// 由应用 reducer 回写 `checked` props，再经 `Widget::update` 同步回实例状态。
pub struct ToggleWidget {
    spec: ToggleSpec,
    hover: bool,
    pressed: bool,
    /// 上次 layout 算出的尺寸；paint 据此摆放 track + label。
    size: Size,
}

impl ToggleWidget {
    pub fn new(spec: ToggleSpec) -> ToggleWidget {
        ToggleWidget {
            spec,
            hover: false,
            pressed: false,
            size: Size::new(180.0, 28.0),
        }
    }

    /// track 背景色：disabled 中性灰；on = primary；off = on_background*0.3 + background*0.7。
    fn track_color(&self, tokens: &SemanticTokens) -> Color {
        if !self.spec.enabled {
            return Color::rgb(
                tokens.surface.r * 0.6 + tokens.background.r * 0.4,
                tokens.surface.g * 0.6 + tokens.background.g * 0.4,
                tokens.surface.b * 0.6 + tokens.background.b * 0.4,
            );
        }
        if self.spec.checked {
            tokens.primary
        } else {
            Color::rgb(
                tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
                tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
                tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
            )
        }
    }
}

impl Widget for ToggleWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut paint_changed = false;
        if let Some(zero_ui_core::binding::Value::Bool(c)) = props.get("checked")
            && *c != self.spec.checked
        {
            self.spec.checked = *c;
            paint_changed = true;
        }
        if let Some(zero_ui_core::binding::Value::Bool(en)) = props.get("enabled") {
            self.spec.enabled = *en;
            paint_changed = true;
        }
        if let Some(zero_ui_core::binding::Value::Text(l)) = props.get("label") {
            // P0-3 修复：只在 label 真变化时 clone + 标 layout，避免每帧无意义重排。
            if self.spec.label.as_deref() != Some(l.as_str()) {
                self.spec.label = Some(l.clone());
                *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
            }
        }
        if paint_changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        if !self.spec.enabled {
            return EventResult::Ignored;
        }
        let UiEvent::Pointer { phase, button, .. } = event else {
            return EventResult::Ignored;
        };
        match phase {
            PointerPhase::Moved => {
                self.hover = true;
                EventResult::Consumed
            }
            PointerPhase::Pressed if matches!(button, Some(PointerButton::Primary)) => {
                self.pressed = true;
                EventResult::Consumed
            }
            PointerPhase::Released if matches!(button, Some(PointerButton::Primary)) => {
                let was_pressed = self.pressed;
                self.pressed = false;
                if was_pressed {
                    // 单向数据流：emit action，应用回写 checked → update 同步回 self.spec.checked。
                    EventResult::Emit(self.spec.action.clone())
                } else {
                    EventResult::Consumed
                }
            }
            PointerPhase::Exited => {
                self.pressed = false;
                self.hover = false;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 宽度：label（若有）+ gap + track(48)；高度：max(label 行高, track 24)。
        const TRACK_W: f32 = 48.0;
        const TRACK_H: f32 = 24.0;
        const GAP: f32 = 12.0;
        let label_w = self
            .spec
            .label
            .as_ref()
            .map(|l| ctx.measure_text(l, 13.0).width)
            .unwrap_or(0.0);
        let w = if label_w > 0.0 {
            label_w + GAP + TRACK_W
        } else {
            TRACK_W
        };
        let h = TRACK_H.max(20.0);
        let size = Size::new(w.max(constraints.min_width).min(constraints.max_width), h);
        self.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        const TRACK_W: f32 = 48.0;
        const TRACK_H: f32 = 24.0;
        // track 右对齐（label 在左），或居中（无 label）。
        let track_x = if self.spec.label.is_some() {
            self.size.width - TRACK_W
        } else {
            (self.size.width - TRACK_W) * 0.5
        };
        let track_y = (self.size.height - TRACK_H) * 0.5;
        let track_rect = Rect::from_origin_size(Point::new(track_x, track_y), Size::new(TRACK_W, TRACK_H));
        ctx.recorder.fill_rect(track_rect, self.track_color(tokens));

        // thumb：on 偏右，off 偏左。
        let thumb_w = 20.0;
        let thumb_x = if self.spec.checked {
            track_x + TRACK_W - thumb_w - 2.0
        } else {
            track_x + 2.0
        };
        let thumb_y = track_y + 2.0;
        let thumb_rect = Rect::from_origin_size(Point::new(thumb_x, thumb_y), Size::new(thumb_w, 20.0));
        ctx.recorder.fill_rect(thumb_rect, tokens.background);
        let thumb_border = if self.hover && self.spec.enabled {
            tokens.primary
        } else {
            tokens.on_background
        };
        ctx.recorder.stroke_rect(thumb_rect, thumb_border, 1.0);

        // label：左侧，垂直居中（13px 字体，近似行高 18）。
        if let Some(label) = &self.spec.label {
            let label_color = if self.spec.enabled {
                tokens.on_surface
            } else {
                Color::rgb(
                    tokens.on_surface.r * 0.5 + tokens.surface.r * 0.5,
                    tokens.on_surface.g * 0.5 + tokens.surface.g * 0.5,
                    tokens.on_surface.b * 0.5 + tokens.surface.b * 0.5,
                )
            };
            let baseline = track_y + (TRACK_H + 13.0) * 0.5;
            ctx.recorder
                .draw_text(label, Point::new(0.0, baseline), 13.0, label_color);
        }
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        let mut flags = SemanticsFlags::FOCUSABLE;
        if self.spec.checked {
            flags |= SemanticsFlags::FOCUSED;
        }
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("toggle"),
            rect: Rect::ZERO,
            flags,
            label: Some(SemanticsLabel::Literal(
                self.spec.label.clone().unwrap_or_default().into(),
            )),
            value: Some(if self.spec.checked { "on".into() } else { "off".into() }),
            children: Vec::new(),
        });
    }

    fn focusable(&self) -> bool {
        self.spec.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_with_label_and_enabled() {
        let s = ToggleSpec::new(false, "perm.geolocation.toggle")
            .with_label("settings.dark_mode")
            .with_enabled(false);
        assert!(!s.checked);
        assert!(!s.enabled);
        assert_eq!(s.label.as_deref(), Some("settings.dark_mode"));
    }

    #[test]
    fn track_color_off_when_unchecked() {
        let t = ToggleWidget::new(ToggleSpec::new(false, "x"));
        let tokens = SemanticTokens::light();
        let c = t.track_color(&tokens);
        let expected = Color::rgb(
            tokens.on_background.r * 0.3 + tokens.background.r * 0.7,
            tokens.on_background.g * 0.3 + tokens.background.g * 0.7,
            tokens.on_background.b * 0.3 + tokens.background.b * 0.7,
        );
        assert!((c.r - expected.r).abs() < 0.001);
    }

    #[test]
    fn track_color_primary_when_checked() {
        let t = ToggleWidget::new(ToggleSpec::new(true, "x"));
        let tokens = SemanticTokens::light();
        assert_eq!(t.track_color(&tokens), tokens.primary);
    }

    #[test]
    fn track_color_neutral_when_disabled() {
        let t = ToggleWidget::new(ToggleSpec::new(true, "x").with_enabled(false));
        let tokens = SemanticTokens::light();
        let c = t.track_color(&tokens);
        let expected = Color::rgb(
            tokens.surface.r * 0.6 + tokens.background.r * 0.4,
            tokens.surface.g * 0.6 + tokens.background.g * 0.4,
            tokens.surface.b * 0.6 + tokens.background.b * 0.4,
        );
        assert!((c.r - expected.r).abs() < 0.001);
    }

    #[test]
    fn flip_toggles_and_emits_action() {
        let mut t = Toggle::new(false, "perm.geolocation.toggle");
        let a1 = t.flip();
        assert!(t.checked);
        assert_eq!(a1, ActionId::new("perm.geolocation.toggle"));
        let _ = t.flip();
        assert!(!t.checked);
    }

    #[test]
    fn label_optional() {
        let t = Toggle::new(true, "x").with_label("settings.dark_mode");
        assert_eq!(t.label.as_deref(), Some("settings.dark_mode"));
    }
}
