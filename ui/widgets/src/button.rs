//! Button — 通用按钮控件（spec FR-009 / IF-001 Widget 范例）。
//!
//! 点击只发出 `Action`，由应用层更新状态（spec FR-003 单向数据流）。
//! 控件内部仅保存临时 UI 状态（hover/pressed）；业务状态由应用持有。

use zero_ui_core::action::{ActionId, EventResult};
use zero_ui_core::event::{PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Rect, Size};
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// 按钮声明。
#[derive(Debug, Clone)]
pub struct ButtonSpec {
    pub label: String,
    pub action: ActionId,
    pub enabled: bool,
}

impl ButtonSpec {
    pub fn new(label: &str, action: &str) -> ButtonSpec {
        ButtonSpec {
            label: label.to_string(),
            action: ActionId::new(action),
            enabled: true,
        }
    }
}

/// Button 控件实例（retained 临时状态）。
pub struct Button {
    spec: ButtonSpec,
    hover: bool,
    pressed: bool,
    /// 上次 `layout()` 算出的尺寸；`paint()` 据此填满背景（DC-7：避免硬编码宽度截断长标签）。
    /// Widget trait 的 `paint` 不接收尺寸，故控件须在 layout 缓存。
    size: Size,
}

impl Button {
    pub fn new(spec: ButtonSpec) -> Button {
        Button {
            spec,
            hover: false,
            pressed: false,
            size: Size::new(96.0, 32.0),
        }
    }

    /// 按钮背景色（DC-5：从 semantic token 派生，不硬编码浏览器色值）。
    ///
    /// - default = `primary`
    /// - hover = `primary.lighten(0.12)`（变亮，交互态；WCAG 对瞬态放宽）
    /// - pressed = `primary.darken(0.12)`（变暗）
    /// - disabled = `on_surface` 与 `surface` 中和的中性灰（WCAG 豁免禁用态，仍 token 派生）
    fn background(&self, tokens: &SemanticTokens) -> Color {
        if !self.spec.enabled {
            return tokens.on_surface.mix(tokens.surface, 0.55);
        }
        let primary = tokens.primary;
        if self.pressed {
            primary.darken(0.12)
        } else if self.hover {
            primary.lighten(0.12)
        } else {
            primary
        }
    }
}

impl Widget for Button {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(zero_ui_core::binding::Value::Text(label)) = props.get("label") {
            self.spec.label = label.clone();
        }
        if let Some(zero_ui_core::binding::Value::Bool(enabled)) = props.get("enabled") {
            self.spec.enabled = *enabled;
        }
        // 标签文本可能变长 → 委托上层 layout 决定；此处只标记 paint。
        *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
    }

    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        let UiEvent::Pointer { phase, button, .. } = event else {
            return EventResult::Ignored;
        };
        if !self.spec.enabled {
            return EventResult::Ignored;
        }
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

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // M1：用字符数启发式估算宽度（M2 接 foundation/text 真实测量）。
        // 按字符数（非字节长度）计宽——多字节标签（CJK/重音）此前被字节数高估，
        // 与 text_input::ime_caret_rect 用 chars().count() 一致（DC-7/i18n）。
        let char_w = 8.0_f32;
        let padding = 16.0_f32;
        let desired = Size::new(self.spec.label.chars().count() as f32 * char_w + padding, 32.0);
        let size = Size::new(
            desired.width.clamp(constraints.min_width, constraints.max_width),
            desired.height.clamp(constraints.min_height, constraints.max_height),
        );
        self.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // 填满 layout 算出的节点矩形（DC-7：用缓存 size，避免硬编码 96 截断长标签背景）。
        // 文本绘制在 M2 接 text foundation 后补。
        ctx.recorder.fill_rect(
            Rect::from_ltrb(0.0, 0.0, self.size.width, self.size.height),
            self.background(ctx.tokens),
        );
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("button"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::BUTTON | SemanticsFlags::FOCUSABLE,
            label: Some(SemanticsLabel::Literal(self.spec.label.clone().into())),
            value: None,
            children: Vec::new(),
        });
    }

    fn focusable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::binding::Value;
    use zero_ui_core::event::{KeyAction, KeyCode, Modifiers};
    use zero_ui_core::geometry::{Point, Vec2};
    use zero_ui_core::invalidation::InvalidationFlags;
    use zero_ui_core::semantics::SemanticsFlags;
    use zero_ui_core::widget::PaintRecorder;

    fn press() -> UiEvent {
        UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Primary),
            position: Point::new(5.0, 5.0),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }
    }

    fn release() -> UiEvent {
        UiEvent::Pointer {
            phase: PointerPhase::Released,
            button: Some(PointerButton::Primary),
            position: Point::new(5.0, 5.0),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }
    }

    /// 记录 paint 调用，用于断言背景色随状态变化。
    #[derive(Default)]
    struct MockRecorder {
        fills: Vec<(Rect, Color)>,
    }
    impl PaintRecorder for MockRecorder {
        fn fill_rect(&mut self, rect: Rect, color: Color) {
            self.fills.push((rect, color));
        }
        fn stroke_rect(&mut self, _rect: Rect, _color: Color, _stroke_width: f32) {}
        fn draw_text(&mut self, _text: &str, _position: Point, _size_px: f32, _color: Color) {}
        fn draw_external_surface(&mut self, _rect: Rect, _surface_id: u64) {}
        fn draw_image(&mut self, _rect: Rect, _image_ref: zero_ui_core::image::ImageRef, _tint: Color) {}
    }

    fn paint_into(btn: &mut Button) -> Color {
        let tokens = SemanticTokens::light();
        paint_into_with(btn, &tokens)
    }

    fn paint_into_with(btn: &mut Button, tokens: &SemanticTokens) -> Color {
        let mut rec = MockRecorder::default();
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: None,
            offset: Vec2::ZERO,
            tokens,
        };
        btn.paint(&mut ctx);
        rec.fills[0].1
    }

    #[test]
    fn click_emits_action() {
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        let mut flags = InvalidationFlags::CLEAN;
        // press
        let _ = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &press(),
        );
        assert!(btn.pressed);
        // release → emit action
        let result = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &release(),
        );
        assert_eq!(result, EventResult::Emit(ActionId::new("app.confirm")));
    }

    #[test]
    fn disabled_button_ignores_events() {
        let mut spec = ButtonSpec::new("OK", "app.confirm");
        spec.enabled = false;
        let mut btn = Button::new(spec);
        let mut flags = InvalidationFlags::CLEAN;
        let result = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &press(),
        );
        assert_eq!(result, EventResult::Ignored);
    }

    #[test]
    fn paint_background_reflects_state() {
        let t = SemanticTokens::light();
        // default = primary
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        assert_eq!(paint_into(&mut btn), t.primary, "默认背景 = tokens.primary");
        // hover = primary.lighten(0.12)
        let mut flags = InvalidationFlags::CLEAN;
        let _ = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &UiEvent::Pointer {
                phase: PointerPhase::Moved,
                button: None,
                position: Point::new(1.0, 1.0),
                modifiers: Modifiers::NONE,
                pointer_id: 0,
            },
        );
        assert_eq!(paint_into(&mut btn), t.primary.lighten(0.12), "hover 背景");
        // pressed = primary.darken(0.12)
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        let _ = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &press(),
        );
        assert_eq!(paint_into(&mut btn), t.primary.darken(0.12), "pressed 背景");
        // disabled = on_surface.mix(surface, 0.55)
        let mut spec = ButtonSpec::new("OK", "app.confirm");
        spec.enabled = false;
        let mut btn = Button::new(spec);
        assert_eq!(paint_into(&mut btn), t.on_surface.mix(t.surface, 0.55), "disabled 背景");
    }

    #[test]
    fn default_background_is_wcag_aa_with_primary_text() {
        // DC-5 闭环：Button 默认背景（= tokens.primary）+ on_primary 文字 ≥ WCAG AA 4.5。
        // 验证控件消费 token 后继承主题可访问性（非硬编码不可访问色）。
        use zero_ui_core::theme::{contrast_ratio, passes_wcag_aa};
        for tokens in [SemanticTokens::light(), SemanticTokens::dark()] {
            let mut btn = Button::new(ButtonSpec::new("OK", "app.x"));
            let bg = paint_into_with(&mut btn, &tokens);
            assert!(
                passes_wcag_aa(tokens.on_primary, bg, false),
                "button default bg {:?} + on_primary {:?} ratio {:.2} < 4.5",
                bg,
                tokens.on_primary,
                contrast_ratio(tokens.on_primary, bg)
            );
        }
    }

    #[test]
    fn dark_theme_background_uses_dark_tokens() {
        // 确认 Button 经 PaintCtx.tokens 消费当前主题（dark primary），而非固定 light 值。
        let dark = SemanticTokens::dark();
        let mut btn = Button::new(ButtonSpec::new("OK", "app.x"));
        assert_eq!(paint_into_with(&mut btn, &dark), dark.primary);
    }

    #[test]
    fn release_without_prior_press_consumes_no_emit() {
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        let mut flags = InvalidationFlags::CLEAN;
        // 直接 release（未先 press）→ Consumed 但不发 action。
        let result = btn.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &release(),
        );
        assert_eq!(result, EventResult::Consumed);
    }

    #[test]
    fn non_primary_pointer_and_non_pointer_events_ignored() {
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        let mut flags = InvalidationFlags::CLEAN;
        // 非主键 press / release → Ignored（落到 `_` 分支）。
        let secondary = UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Secondary),
            position: Point::new(0.0, 0.0),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        };
        assert_eq!(
            btn.event(
                &mut EventCtx {
                    invalidation: &mut flags,
                },
                &secondary,
            ),
            EventResult::Ignored
        );
        let secondary_release = UiEvent::Pointer {
            phase: PointerPhase::Released,
            button: Some(PointerButton::Secondary),
            position: Point::new(0.0, 0.0),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        };
        assert_eq!(
            btn.event(
                &mut EventCtx {
                    invalidation: &mut flags,
                },
                &secondary_release,
            ),
            EventResult::Ignored
        );
        // 非指针事件（键盘）→ Ignored。
        let key = UiEvent::Key {
            code: KeyCode::new("Space"),
            action: KeyAction::Pressed,
            modifiers: Modifiers::NONE,
            text: None,
        };
        assert_eq!(
            btn.event(
                &mut EventCtx {
                    invalidation: &mut flags,
                },
                &key,
            ),
            EventResult::Ignored
        );
    }

    #[test]
    fn update_applies_props_and_marks_paint() {
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        let mut props = Props::new();
        props.insert("label", Value::Text("Cancel".into()));
        props.insert("enabled", Value::Bool(false));
        let mut flags = InvalidationFlags::CLEAN;
        btn.update(
            &mut UpdateCtx {
                invalidation: &mut flags,
            },
            &props,
        );
        assert_eq!(btn.spec.label, "Cancel");
        assert!(!btn.spec.enabled);
        assert!(flags.contains(InvalidationFlags::NEEDS_PAINT));
    }

    #[test]
    fn layout_clamps_to_constraints() {
        let mut btn = Button::new(ButtonSpec::new("OK", "app.confirm"));
        // "OK" 期望 32×32；tight 50×20 → clamp 到 50×20。
        let tight = btn.layout(
            &mut LayoutCtx { scale_factor: 1.0 },
            Constraints::tight(Size::new(50.0, 20.0)),
        );
        assert_eq!((tight.width, tight.height), (50.0, 20.0));
        // loose 200×200 → 期望尺寸不被裁剪。
        let loose = btn.layout(
            &mut LayoutCtx { scale_factor: 1.0 },
            Constraints::loose(Size::new(200.0, 200.0)),
        );
        assert_eq!((loose.width, loose.height), (32.0, 32.0));
    }

    #[test]
    fn semantics_emits_button_node() {
        let btn = Button::new(ButtonSpec::new("Save", "app.save"));
        let mut nodes: Vec<SemanticsNode> = Vec::new();
        btn.semantics(&mut SemanticsCtx { nodes: &mut nodes });
        assert_eq!(nodes.len(), 1);
        assert!(
            nodes[0]
                .flags
                .contains(SemanticsFlags::BUTTON | SemanticsFlags::FOCUSABLE)
        );
        assert!(nodes[0].label.is_some());
    }

    // ── 深度审查（lei-deep-review）：paint 覆盖 layout 尺寸 + i18n 字符计宽 ──

    #[test]
    fn paint_covers_full_laid_out_width_for_wide_label() {
        // DC-7 修复：paint 背景应填满 layout 算出的节点宽度，而非硬编码 96。
        // 长标签（>10 字符 → 宽度 >96）此前背景被截断为 96px，右侧透明。
        let mut btn = Button::new(ButtonSpec::new("Save Settings Now", "app.save"));
        let size = btn.layout(
            &mut LayoutCtx { scale_factor: 1.0 },
            Constraints::loose(Size::new(400.0, 400.0)),
        );
        // 标签 17 字符 → 期望宽 17×8+16 = 152。
        assert!((size.width - 152.0).abs() < 0.5, "laid out width {}", size.width);
        let mut rec = MockRecorder::default();
        let tokens = SemanticTokens::light();
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: None,
            offset: Vec2::ZERO,
            tokens: &tokens,
        };
        btn.paint(&mut ctx);
        assert_eq!(rec.fills.len(), 1);
        let fill_width = rec.fills[0].0.right();
        assert!(
            (fill_width - size.width).abs() < 0.5,
            "paint fill width {} should cover laid-out width {} (was hardcoded 96)",
            fill_width,
            size.width
        );
    }

    #[test]
    fn layout_sizes_by_char_count_not_byte_length() {
        // DC-7/i18n：多字节标签按字符数计宽，而非字节数（与 text_input ime_caret_rect 用 chars().count() 一致）。
        // "保存" = 2 字符 / 6 字节 → 宽度应为 2×8+16=32，而非 6×8+16=64。
        let mut btn = Button::new(ButtonSpec::new("保存", "app.save"));
        let size = btn.layout(
            &mut LayoutCtx { scale_factor: 1.0 },
            Constraints::loose(Size::new(400.0, 400.0)),
        );
        assert!(
            (size.width - 32.0).abs() < 0.5,
            "2-char CJK label width {} should be 32 (char count), not 64 (byte length)",
            size.width
        );
    }

    #[test]
    fn exited_clears_pressed_and_hover() {
        // F1：Exited 清除 Button pressed/hover 态（曾因缺 Exited 粘滞）。
        let mut btn = Button::new(ButtonSpec::new("Click", "app.click"));
        let mut inval = InvalidationFlags::CLEAN;
        let mut ctx = EventCtx {
            invalidation: &mut inval,
        };

        // 按下。
        let _ = btn.event(&mut ctx, &press());
        // 断言可经 pressed bg 颜色验证（后续用 Released 是否 emit 间接证）。
        // 派发 Exited → pressed 应被清除。
        let exited = UiEvent::Pointer {
            phase: PointerPhase::Exited,
            button: None,
            position: Point::ZERO,
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        };
        let _ = btn.event(&mut ctx, &exited);
        // 释放 → 不应 emit（pressed 已被 Exited 清除）。
        let res = btn.event(&mut ctx, &release());
        assert!(
            !matches!(res, EventResult::Emit(_)),
            "Exited clears pressed → release should not emit"
        );
    }
}
