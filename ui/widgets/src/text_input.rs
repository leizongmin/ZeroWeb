//! TextInput — 文本输入控件（spec FR-009 / DC-8 IME）。
//!
//! 本模块提供两层 API：
//! - **retained 编辑状态** [`TextInputState`]：光标/选区/插入/退格等纯数据操作，
//!   可由应用层或 patterns 复用（与 host 树无关）。
//! - **完整 Widget** [`TextInputWidget`]：实现 `Widget` trait，处理键盘/点击事件、
//!   paint caret + 文本、focus + IME caret rect。受控模式下 `text` 从 props 同步。

use zero_ui_core::action::{ActionId, EventResult};
use zero_ui_core::event::{KeyAction, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// 文本变更 action 的 payload：携带最新完整文本。
///
/// 受控单向数据流：应用接收 action 后回写 `text` props（如有差异），驱动 reconcile。
pub const ACTION_TEXT_CHANGED: &str = "text_input.changed";

/// TextInput 的 retained 编辑状态（无 Widget 依赖，可独立使用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputState {
    pub text: String,
    /// 光标字节偏移（= 选区锚点）。
    pub cursor: usize,
    /// 选区（起止字节偏移）；None = 无选区（collapsed caret）。
    pub selection: Option<(usize, usize)>,
}

impl TextInputState {
    pub fn empty() -> TextInputState {
        TextInputState {
            text: String::new(),
            cursor: 0,
            selection: None,
        }
    }

    pub fn from_text(text: &str) -> TextInputState {
        TextInputState {
            text: text.to_string(),
            cursor: text.len(),
            selection: None,
        }
    }

    fn clamp_cursor(&self, c: usize) -> usize {
        c.min(self.text.len())
    }

    /// 在光标处插入文本（替换当前选区）。
    pub fn insert(&mut self, s: &str) {
        if let Some((start, end)) = self.selection {
            let (lo, hi) = (start.min(end), start.max(end));
            self.text.replace_range(lo..hi, s);
            self.cursor = lo + s.len();
        } else {
            self.text.insert_str(self.cursor, s);
            self.cursor += s.len();
        }
        self.selection = None;
    }

    /// 向前删除（backspace）。
    pub fn backspace(&mut self) {
        if let Some((start, end)) = self.selection {
            let (lo, hi) = (start.min(end), start.max(end));
            self.text.replace_range(lo..hi, "");
            self.cursor = lo;
            self.selection = None;
            return;
        }
        if self.cursor == 0 {
            return;
        }
        // 退到前一个字符边界。
        let prev = self.text[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.replace_range(prev..self.cursor, "");
        self.cursor = prev;
    }

    /// 移动光标（dir = -1/1）。
    pub fn move_cursor(&mut self, dir: i32) {
        if dir < 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if let Some((_, ch)) = self.text[self.cursor..].char_indices().next() {
            self.cursor += ch.len_utf8();
        }
        self.selection = None;
    }

    /// 由 caret 字节偏移推算的 IME 光标屏幕 rect（M1：等宽启发式；M2 用真实 shaped metrics）。
    pub fn ime_caret_rect(&self, origin: Rect) -> Rect {
        let char_w = 8.0_f32;
        let char_count = self.text[..self.clamp_cursor(self.cursor)].chars().count();
        Rect::from_ltrb(
            origin.left() + char_count as f32 * char_w,
            origin.top(),
            origin.left() + char_count as f32 * char_w + 2.0,
            origin.bottom(),
        )
    }
}

/// TextInput Widget 实例。
///
/// 内部 [`TextInputState`] 持有 caret/selection；`text` 通过 props 受控同步：
/// 当 props.text 与 state.text 不一致（应用层 setValue 等），update 阶段强制覆盖。
pub struct TextInputWidget {
    state: TextInputState,
    /// placeholder 文本（state.text 为空时显示）。
    placeholder: String,
    /// 上次 layout 缓存的尺寸（paint 用）。
    size: Size,
    /// 是否聚焦（由 host 焦点系统驱动：focused 节点 paint caret）。
    focused: bool,
    hover: bool,
}

impl TextInputWidget {
    pub fn new() -> TextInputWidget {
        TextInputWidget {
            state: TextInputState::empty(),
            placeholder: String::new(),
            size: Size::new(200.0, 28.0),
            focused: false,
            hover: false,
        }
    }

    pub fn with_placeholder(mut self, p: &str) -> TextInputWidget {
        self.placeholder = p.to_string();
        self
    }

    /// 由工厂调用：把 props.text 写入内部 state（用于初始挂载时同步受控值）。
    pub fn set_text_from_props(&mut self, text: &str) {
        if !text.is_empty() {
            self.state = TextInputState::from_text(text);
        }
    }

    /// 文本色（聚焦时高亮）。
    fn text_color(&self, tokens: &SemanticTokens) -> Color {
        tokens.on_surface
    }

    /// caret 垂直条 rect（基于上次 paint 的 size 与 state.cursor）。
    fn caret_rect(&self) -> Rect {
        let char_w = 8.0_f32;
        let char_count = self.state.text[..self.state.cursor.min(self.state.text.len())]
            .chars()
            .count();
        let x = 8.0 + char_count as f32 * char_w;
        Rect::from_origin_size(Point::new(x, 4.0), Size::new(2.0, self.size.height - 8.0))
    }
}

impl Default for TextInputWidget {
    fn default() -> TextInputWidget {
        TextInputWidget::new()
    }
}

impl Widget for TextInputWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut layout_changed = false;
        // 受控同步：props.text 与 state 不一致时强制覆盖（应用 setValue）。
        if let Some(zero_ui_core::binding::Value::Text(t)) = props.get("text")
            && t != &self.state.text
        {
            self.state = TextInputState::from_text(t);
            layout_changed = true;
        }
        if let Some(zero_ui_core::binding::Value::Text(p)) = props.get("placeholder") {
            self.placeholder = p.clone();
        }
        if layout_changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
        }
        // 任何 props 更新都重画（保险）。
        *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
    }

    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer { phase, position, .. } => {
                use zero_ui_core::event::PointerPhase;
                match phase {
                    PointerPhase::Moved => {
                        self.hover = true;
                        EventResult::Consumed
                    }
                    PointerPhase::Pressed => {
                        // 点击定位光标：按 x 反推字符偏移（启发式 8px/char）。
                        let char_w = 8.0_f32;
                        let x_in = (position.x - 8.0).max(0.0);
                        let idx_bytes: usize = self
                            .state
                            .text
                            .char_indices()
                            .take((x_in / char_w).round() as usize)
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        self.state.cursor = idx_bytes;
                        self.state.selection = None;
                        EventResult::Consumed
                    }
                    PointerPhase::Exited => {
                        self.hover = false;
                        EventResult::Consumed
                    }
                    _ => EventResult::Ignored,
                }
            }
            UiEvent::Focus(zero_ui_core::event::FocusEvent::Gained) => {
                self.focused = true;
                EventResult::Consumed
            }
            UiEvent::Focus(zero_ui_core::event::FocusEvent::Lost) => {
                self.focused = false;
                EventResult::Consumed
            }
            UiEvent::Key {
                action: KeyAction::Pressed,
                code,
                text,
                ..
            } => {
                let old_text = self.state.text.clone();
                match code.0.as_str() {
                    "Backspace" => self.state.backspace(),
                    "ArrowLeft" => self.state.move_cursor(-1),
                    "ArrowRight" => self.state.move_cursor(1),
                    "Enter" => {
                        // Enter 视为提交，不再插入换行（单行输入框语义）。
                        return EventResult::Consumed;
                    }
                    "Escape" => return EventResult::Consumed,
                    _ => {
                        if let Some(ch) = text
                            && !ch.chars().any(|c| c.is_control())
                        {
                            self.state.insert(ch);
                        } else {
                            return EventResult::Ignored;
                        }
                    }
                }
                // 文本若有变化 → emit text_changed action（应用回写或更新业务状态）。
                if self.state.text != old_text {
                    EventResult::EmitWithPayload(
                        ActionId::new(ACTION_TEXT_CHANGED),
                        zero_ui_core::action::ActionPayload::Text(self.state.text.clone()),
                    )
                } else {
                    EventResult::Consumed
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        // 单行高度 28；宽度按 max_width 收敛，min 80。
        let w = c.max_width.min(c.max_width).max(c.min_width.max(80.0));
        let h = 28.0_f32.max(c.min_height).min(c.max_height.max(28.0));
        let size = Size::new(w, h);
        self.size = size;
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        // 背景 + 边框（聚焦/悬停用 primary，否则 on_background*0.4 混色）。
        let border = if self.focused {
            tokens.primary
        } else if self.hover {
            Color::rgb(
                tokens.on_background.r * 0.6 + tokens.background.r * 0.4,
                tokens.on_background.g * 0.6 + tokens.background.g * 0.4,
                tokens.on_background.b * 0.6 + tokens.background.b * 0.4,
            )
        } else {
            Color::rgb(
                tokens.on_background.r * 0.4 + tokens.background.r * 0.6,
                tokens.on_background.g * 0.4 + tokens.background.g * 0.6,
                tokens.on_background.b * 0.4 + tokens.background.b * 0.6,
            )
        };
        let bg = tokens.surface;
        let frame = Rect::from_origin_size(Point::ZERO, self.size);
        ctx.recorder.fill_rect(frame, bg);
        ctx.recorder
            .stroke_rect(frame, border, if self.focused { 2.0 } else { 1.0 });

        // 文本或 placeholder。
        let display = if self.state.text.is_empty() {
            self.placeholder.clone()
        } else {
            self.state.text.clone()
        };
        let fg = if self.state.text.is_empty() {
            Color::rgb(
                tokens.on_background.r * 0.5 + tokens.background.r * 0.5,
                tokens.on_background.g * 0.5 + tokens.background.g * 0.5,
                tokens.on_background.b * 0.5 + tokens.background.b * 0.5,
            )
        } else {
            self.text_color(tokens)
        };
        let baseline = (self.size.height + 14.0) * 0.5;
        ctx.recorder.draw_text(&display, Point::new(8.0, baseline), 14.0, fg);

        // caret（仅聚焦时画）。
        if self.focused {
            ctx.recorder.fill_rect(self.caret_rect(), tokens.primary);
        }
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("text_input"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::TEXT_FIELD | SemanticsFlags::FOCUSABLE,
            label: Some(SemanticsLabel::Literal(self.placeholder.clone().into())),
            value: Some(self.state.text.as_str().into()),
            children: Vec::new(),
        });
    }

    fn focusable(&self) -> bool {
        true
    }

    fn ime_rect(&self) -> Option<Rect> {
        Some(self.caret_rect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_move_backspace() {
        let mut st = TextInputState::empty();
        st.insert("abc");
        assert_eq!(st.text, "abc");
        assert_eq!(st.cursor, 3);
        st.move_cursor(-1);
        st.insert("X");
        assert_eq!(st.text, "abXc");
        st.move_cursor(1);
        st.backspace();
        assert_eq!(st.text, "abX");
    }

    #[test]
    fn replace_selection() {
        let mut st = TextInputState::empty();
        st.insert("hello");
        st.selection = Some((1, 4)); // "ell"
        st.insert("EL");
        assert_eq!(st.text, "hELo");
        assert_eq!(st.cursor, 3);
        assert_eq!(st.selection, None);
    }

    #[test]
    fn ime_caret_rect_advances_with_cursor() {
        let mut st = TextInputState::empty();
        st.insert("abcd");
        st.cursor = 2;
        let r = st.ime_caret_rect(Rect::from_ltrb(0.0, 0.0, 100.0, 20.0));
        // 2 个字符 → caret x = 16。
        assert_eq!(r.left(), 16.0);
    }

    #[test]
    fn widget_default_focusable() {
        let w = TextInputWidget::new();
        assert!(w.focusable());
        assert!(w.ime_rect().is_some());
    }
}
