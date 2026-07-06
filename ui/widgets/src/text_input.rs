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
///
/// 选区模型：`cursor` + `anchor`（与浏览器地址栏 `apps/browser/src/text_input.rs` 一致）。
/// - `cursor == anchor`：collapsed caret（无选区）。
/// - `cursor != anchor`：有选区，选区 = `[min(cursor,anchor), max(cursor,anchor)]`。
///
/// 字节偏移（非 char 索引），与 `String::replace_range` 直接兼容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputState {
    pub text: String,
    /// 光标字节偏移（编辑插入点 / 选区活动端）。
    pub cursor: usize,
    /// 选区锚点字节偏移（选区另一端；= cursor 表示无选区）。
    pub anchor: usize,
}

impl TextInputState {
    pub fn empty() -> TextInputState {
        TextInputState {
            text: String::new(),
            cursor: 0,
            anchor: 0,
        }
    }

    pub fn from_text(text: &str) -> TextInputState {
        TextInputState {
            text: text.to_string(),
            cursor: text.len(),
            anchor: text.len(),
        }
    }

    fn clamp_cursor(&self, c: usize) -> usize {
        c.min(self.text.len())
    }

    /// 是否有选区（cursor != anchor）。
    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    /// 选区字节范围 `(min, max)`。无选区时返回 collapsed `(cursor, cursor)`。
    pub fn selection_range(&self) -> (usize, usize) {
        if self.cursor <= self.anchor {
            (self.cursor, self.anchor)
        } else {
            (self.anchor, self.cursor)
        }
    }

    /// 选中文本（无选区返回空串）。
    pub fn selected_text(&self) -> &str {
        if !self.has_selection() {
            return "";
        }
        let (lo, hi) = self.selection_range();
        &self.text[lo..hi]
    }

    /// 全选：anchor=0，cursor=末尾。
    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
    }

    /// 设置光标位置。`extend=true` 时移动 cursor 但保留 anchor（拖拽选择）；
    /// `extend=false` 时 cursor 与 anchor 一起移动（取消选区）。
    pub fn set_cursor(&mut self, byte_idx: usize, extend: bool) {
        let i = self.clamp_cursor(byte_idx);
        self.cursor = i;
        if !extend {
            self.anchor = i;
        }
    }

    /// 在光标处插入文本（替换当前选区）。
    pub fn insert(&mut self, s: &str) {
        if self.has_selection() {
            let (lo, hi) = self.selection_range();
            self.text.replace_range(lo..hi, s);
            self.cursor = lo + s.len();
        } else {
            self.text.insert_str(self.cursor, s);
            self.cursor += s.len();
        }
        self.anchor = self.cursor;
    }

    /// 向前删除（backspace）。有选区时删选区；无选区删前一字符。
    pub fn backspace(&mut self) {
        if self.has_selection() {
            let (lo, hi) = self.selection_range();
            self.text.replace_range(lo..hi, "");
            self.cursor = lo;
            self.anchor = self.cursor;
            return;
        }
        if self.cursor == 0 {
            return;
        }
        let prev = self.text[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        self.anchor = self.cursor;
    }

    /// 移动光标（dir = -1/1）。`extend=true` 保留 anchor（Shift+方向键扩展选区）。
    pub fn move_cursor(&mut self, dir: i32, extend: bool) {
        if dir < 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if let Some((_, ch)) = self.text[self.cursor..].char_indices().next() {
            self.cursor += ch.len_utf8();
        }
        if !extend {
            self.anchor = self.cursor;
        }
    }

    /// 双击选词：选中包含 `byte_idx` 的单词（alphanumeric / _ / - / . / / 视为词字符）。
    /// 与浏览器地址栏 `select_word_at` 同口径。
    pub fn select_word_at(&mut self, byte_idx: usize) {
        let chars: Vec<(usize, char)> = self.text.char_indices().collect();
        if chars.is_empty() {
            return;
        }
        // 把字节偏移转 char 索引：找到第一个 char 起始字节 > byte_idx 的位置，
        // 然后回退一个 = 点击落点对应的 char。
        // 用 > 而非 >=：byte_idx 正好落在字符边界时，position 跳过该字符，
        // saturating_sub 后正好回到该字符（边界点击取当前字符而非前一字符）。
        let char_idx = chars
            .iter()
            .position(|(b, _)| *b > byte_idx)
            .unwrap_or(chars.len());
        let idx = char_idx.saturating_sub(1).min(chars.len().saturating_sub(1));
        let mut start = idx;
        let mut end = (idx + 1).min(chars.len());
        while start > 0 && is_word_char(chars[start - 1].1) {
            start -= 1;
        }
        while end < chars.len() && is_word_char(chars[end].1) {
            end += 1;
        }
        // anchor = chars[start] 字节偏移；cursor = chars[end] 字节偏移（end==len 时为 text.len()）。
        self.anchor = chars.get(start).map(|(b, _)| *b).unwrap_or(0);
        self.cursor = chars.get(end).map(|(b, _)| *b).unwrap_or(self.text.len());
    }

    /// 由 caret 字节偏移推算的 IME 光标屏幕 rect（启发式 8px/char；widget 层用真实度量覆盖）。
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

/// 词字符判定（与浏览器 `is_word_char` 同口径）。
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'
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
    /// CJK/IME 修复：缓存每个字符边界对应的累计 x 偏移（含起始 0.0）。
    /// `char_x[i]` = text 前 i 个字符的累计像素宽度（左侧 padding 8.0 已加）。
    /// 由 paint 阶段用 measure_text 实测；event（点击定位）复用。
    /// 长度 = chars count + 1；首元素 = 8.0（padding）。
    char_x: Vec<f32>,
    /// 上次 paint 的文本（用于检测 char_x 是否需要重建）。
    painted_text: String,
    /// 拖拽选择中（鼠标按住拖动时移动 cursor 但保留 anchor）。
    dragging: bool,
    /// 上次点击的 (Instant, x, y) 用于双击检测（None = 无上次点击或已过期）。
    last_click: Option<std::time::Instant>,
    last_click_pos: (f32, f32),
}

impl TextInputWidget {
    pub fn new() -> TextInputWidget {
        TextInputWidget {
            state: TextInputState::empty(),
            placeholder: String::new(),
            size: Size::new(200.0, 28.0),
            focused: false,
            hover: false,
            char_x: vec![8.0],
            painted_text: String::new(),
            dragging: false,
            last_click: None,
            last_click_pos: (0.0, 0.0),
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

    /// caret 垂直条 rect（基于缓存 char_x 与 state.cursor）。
    /// 若 char_x 尚未对齐当前 text（首次 paint 前），回落到末尾。
    fn caret_rect(&self) -> Rect {
        let char_idx = self.state.text[..self.state.cursor.min(self.state.text.len())]
            .chars()
            .count();
        let x = if char_idx < self.char_x.len() {
            self.char_x[char_idx]
        } else {
            *self.char_x.last().unwrap_or(&8.0)
        };
        Rect::from_origin_size(Point::new(x, 4.0), Size::new(2.0, self.size.height - 8.0))
    }

    /// 由 x 坐标反推光标字节偏移（点击定位）。基于缓存 char_x 二分查找最近的边界。
    fn cursor_from_x(&self, x: f32) -> usize {
        if self.char_x.len() <= 1 || self.painted_text.is_empty() {
            return 0;
        }
        // 找到 char_x 中 <= x 的最大索引（最近边界）。
        let mut best_idx = 0usize;
        for (i, &cx) in self.char_x.iter().enumerate() {
            if cx <= x {
                best_idx = i;
            } else {
                break;
            }
        }
        // best_idx 是字符边界索引（0..=chars_count），转字节偏移。
        self.painted_text
            .char_indices()
            .nth(best_idx)
            .map(|(b, _)| b)
            .unwrap_or_else(|| self.painted_text.len())
    }

    /// 用 measure_text 重建 char_x 缓存（paint 阶段调用）。
    fn rebuild_char_x(&mut self, ctx: &mut PaintCtx, font_size: f32, display: &str) {
        // 注意：display 可能是 placeholder（空 text 时），但 caret 只在非空时显示，
        // 所以只在 display == state.text 时缓存才有意义。
        if display == self.state.text && display == self.painted_text {
            return; // 已缓存且一致。
        }
        let mut xs = Vec::with_capacity(display.chars().count() + 1);
        xs.push(8.0); // 左 padding
        let mut buf = String::new();
        for ch in display.chars() {
            buf.push(ch);
            let w = ctx.measure_text(&buf, font_size).width;
            xs.push(8.0 + w);
        }
        self.char_x = xs;
        self.painted_text = display.to_string();
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
            UiEvent::Pointer { phase, position, modifiers, .. } => {
                use zero_ui_core::event::PointerPhase;
                match phase {
                    PointerPhase::Moved => {
                        self.hover = true;
                        // 拖拽选择中：移动 cursor 但保留 anchor（extend=true）。
                        if self.dragging {
                            let idx = self.cursor_from_x(position.x);
                            self.state.set_cursor(idx, true);
                            return EventResult::Consumed;
                        }
                        EventResult::Consumed
                    }
                    PointerPhase::Pressed => {
                        // 双击检测：上次点击 < 500ms 且位移 < 5px → 双击选词。
                        // 与浏览器地址栏 TAB_BAR_DOUBLE_CLICK_INTERVAL 同口径。
                        let now = std::time::Instant::now();
                        let is_double = self
                            .last_click
                            .map(|t| {
                                now.duration_since(t).as_millis() < 500
                                    && (position.x - self.last_click_pos.0).abs() < 5.0
                                    && (position.y - self.last_click_pos.1).abs() < 5.0
                            })
                            .unwrap_or(false);

                        if is_double {
                            // 双击：选词，取消 drag（双击不进入拖拽）。
                            let idx = self.cursor_from_x(position.x);
                            self.state.select_word_at(idx);
                            self.dragging = false;
                            self.last_click = None; // 三击重新开始
                        } else {
                            // 单击：定位光标。Shift+单击 = 扩展选区（保留 anchor）。
                            let idx = self.cursor_from_x(position.x);
                            let extend = modifiers.contains(zero_ui_core::event::Modifiers::SHIFT);
                            self.state.set_cursor(idx, extend);
                            self.dragging = true;
                            self.last_click = Some(now);
                            self.last_click_pos = (position.x, position.y);
                        }
                        EventResult::Consumed
                    }
                    PointerPhase::Released => {
                        // 释放鼠标：结束拖拽。
                        self.dragging = false;
                        EventResult::Consumed
                    }
                    PointerPhase::Exited => {
                        self.hover = false;
                        self.dragging = false;
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
                self.dragging = false;
                EventResult::Consumed
            }
            UiEvent::Ime(ime) => {
                // CJK 输入修复：IME Commit 把合成完成的文本插入到光标位置。
                // 中文/日文等复合输入走这条路径，不走 Key.text。
                match ime {
                    zero_ui_core::event::ImeEvent::Commit(s) => {
                        let old_text = self.state.text.clone();
                        if !s.is_empty() {
                            self.state.insert(s);
                        }
                        if self.state.text != old_text {
                            EventResult::EmitWithPayload(
                                ActionId::new(ACTION_TEXT_CHANGED),
                                zero_ui_core::action::ActionPayload::Text(self.state.text.clone()),
                            )
                        } else {
                            EventResult::Consumed
                        }
                    }
                    // Preedit/Enabled/Disabled 暂不处理（M2：合成预览浮层）。
                    _ => EventResult::Consumed,
                }
            }
            UiEvent::Key {
                action,
                code,
                text,
                modifiers,
            } => {
                // P1-3：按住键重复输入——KeyAction::Repeat 与 Pressed 同样处理。
                if !matches!(action, KeyAction::Pressed | KeyAction::Repeat) {
                    return EventResult::Ignored;
                }
                let old_text = self.state.text.clone();
                let shift = modifiers.contains(zero_ui_core::event::Modifiers::SHIFT);
                // P1-5：Ctrl+A/C/V/X 快捷键。
                // **键码契约**：winit `Key::Character("a")` → `KeyCode("a")`（小写字面值，
                // 不是 "KeyA"）。Ctrl 组合时 winit 传小写字面值，与浏览器 `app_input.rs`
                // `"t" | "T"` 匹配方式一致。大小写都匹配以兼容 CapsLock。
                if modifiers.contains(zero_ui_core::event::Modifiers::CONTROL) {
                    let key_lower = code.0.to_ascii_lowercase();
                    match key_lower.as_str() {
                        "a" => {
                            self.state.select_all();
                            return EventResult::Consumed;
                        }
                        "c" => {
                            let payload = if self.state.has_selection() {
                                self.state.selected_text().to_string()
                            } else {
                                self.state.text.clone()
                            };
                            return EventResult::EmitWithPayload(
                                ActionId::new("text_input.clipboard_copy"),
                                zero_ui_core::action::ActionPayload::Text(payload),
                            );
                        }
                        "x" => {
                            let payload = if self.state.has_selection() {
                                self.state.selected_text().to_string()
                            } else {
                                self.state.text.clone()
                            };
                            if self.state.has_selection() {
                                let (lo, hi) = self.state.selection_range();
                                self.state.text.replace_range(lo..hi, "");
                                self.state.cursor = lo;
                                self.state.anchor = lo;
                            }
                            return EventResult::EmitWithPayload(
                                ActionId::new("text_input.clipboard_copy"),
                                zero_ui_core::action::ActionPayload::Text(payload),
                            );
                        }
                        "v" => {
                            return EventResult::Emit(ActionId::new("text_input.clipboard_paste"));
                        }
                        _ => {} // 其它 Ctrl 组合键不在此处理。
                    }
                }
                match code.0.as_str() {
                    "Backspace" => self.state.backspace(),
                    "ArrowLeft" => self.state.move_cursor(-1, shift),
                    "ArrowRight" => self.state.move_cursor(1, shift),
                    "Enter" => return EventResult::Consumed,
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
                // 文本变化或选区变化 → 需要 repaint。文本变化 emit text_changed。
                if self.state.text != old_text {
                    EventResult::EmitWithPayload(
                        ActionId::new(ACTION_TEXT_CHANGED),
                        zero_ui_core::action::ActionPayload::Text(self.state.text.clone()),
                    )
                } else {
                    // 选区变化或纯 caret 移动 → 重画但不 emit。
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

        // CJK 修复：用真实字体度量重建字符 x 偏移表，供 caret/选区/click 定位复用。
        // 只对 state.text 缓存（placeholder 不需要 caret）。
        if !self.state.text.is_empty() {
            self.rebuild_char_x(ctx, 14.0, &self.state.text.clone());
        }

        // P1-4：选区高亮（在文本之前画，文本叠在上层）。
        // 选区色 = primary 与 surface 混合的半透明（与浏览器地址栏 selection 背景 同口径）。
        if self.state.has_selection() && !self.state.text.is_empty() {
            let (lo, hi) = self.state.selection_range();
            // 字节偏移 → char 索引 → char_x 索引。
            let lo_char = self.state.text[..lo].chars().count();
            let hi_char = self.state.text[..hi].chars().count();
            let x_start = self
                .char_x
                .get(lo_char)
                .copied()
                .unwrap_or_else(|| self.char_x.first().copied().unwrap_or(8.0));
            let x_end = self
                .char_x
                .get(hi_char)
                .copied()
                .unwrap_or_else(|| self.char_x.last().copied().unwrap_or(8.0));
            let sel_rect = Rect::from_ltrb(
                x_start,
                4.0,
                x_end.max(x_start + 1.0),
                self.size.height - 4.0,
            );
            let sel_color = Color::rgba(
                tokens.primary.r * 0.7 + tokens.surface.r * 0.3,
                tokens.primary.g * 0.7 + tokens.surface.g * 0.3,
                tokens.primary.b * 0.7 + tokens.surface.b * 0.3,
                0.35,
            );
            ctx.recorder.fill_rect(sel_rect, sel_color);
        }

        let baseline = (self.size.height + 14.0) * 0.5;
        ctx.recorder.draw_text(&display, Point::new(8.0, baseline), 14.0, fg);

        // caret（仅聚焦且无选区时画——有选区时光标隐藏，与浏览器地址栏一致）。
        // 500ms 周期闪烁，闪烁周期内调 request_frame 触发下一帧重 paint。
        if self.focused && !self.state.has_selection() {
            let caret_visible = match ctx.now_ms {
                Some(ms) => {
                    // 1068ms 周期：534ms 显 + 534ms 隐（避免与帧率整除导致卡死）。
                    let phase = (ms % 1068) < 534;
                    ctx.request_frame();
                    phase
                }
                None => true,
            };
            if caret_visible {
                ctx.recorder.fill_rect(self.caret_rect(), tokens.primary);
            }
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
    use zero_ui_core::action::ActionPayload;

    #[test]
    fn insert_move_backspace() {
        let mut st = TextInputState::empty();
        st.insert("abc");
        assert_eq!(st.text, "abc");
        assert_eq!(st.cursor, 3);
        st.move_cursor(-1, false);
        st.insert("X");
        assert_eq!(st.text, "abXc");
        st.move_cursor(1, false);
        st.backspace();
        assert_eq!(st.text, "abX");
    }

    #[test]
    fn replace_selection() {
        let mut st = TextInputState::empty();
        st.insert("hello");
        // 选区 "ell"：anchor=1, cursor=4。
        st.anchor = 1;
        st.cursor = 4;
        st.insert("EL");
        assert_eq!(st.text, "hELo");
        assert_eq!(st.cursor, 3);
        assert_eq!(st.anchor, 3);
        assert!(!st.has_selection());
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

    /// CJK 输入回归：IME Commit("你") 把中文插入到光标位置，emit 带 text 的 action。
    #[test]
    fn ime_commit_inserts_cjk_text() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx {
            invalidation: &mut flags,
        };
        let res = w.event(
            &mut ec,
            &UiEvent::Ime(zero_ui_core::event::ImeEvent::Commit("你".into())),
        );
        match res {
            EventResult::EmitWithPayload(_, ActionPayload::Text(t)) => {
                assert_eq!(t, "你");
                assert_eq!(w.state.text, "你");
                // 字节偏移 = 3（"你" 是 3 字节 UTF-8），但 char count = 1。
                assert_eq!(w.state.cursor, 3);
            }
            other => panic!("期望 EmitWithPayload，实际 {other:?}"),
        }
    }

    /// CJK 输入回归：IME Commit 多次追加，文本累积正确。
    #[test]
    fn ime_commit_appends_multiple_cjk() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx {
            invalidation: &mut flags,
        };
        let _ = w.event(
            &mut ec,
            &UiEvent::Ime(zero_ui_core::event::ImeEvent::Commit("你".into())),
        );
        let _ = w.event(
            &mut ec,
            &UiEvent::Ime(zero_ui_core::event::ImeEvent::Commit("好".into())),
        );
        assert_eq!(w.state.text, "你好");
        assert_eq!(w.state.cursor, 6); // 两个汉字 = 6 字节
    }

    /// char_x 缓存：cursor_from_x 在空缓存时退到末尾（首次点击场景）。
    #[test]
    fn cursor_from_x_returns_end_when_cache_empty() {
        let w = TextInputWidget::new();
        // 默认 char_x = [8.0]，text 空 → 退到 len()=0。
        assert_eq!(w.cursor_from_x(50.0), 0);
    }

    /// P1-3 回归：KeyAction::Repeat 必须与 Pressed 同样插入字符（按住键重复输入）。
    #[test]
    fn key_repeat_inserts_character() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        // 第一次 Pressed
        let _ = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("a"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::NONE,
            text: Some("a".into()),
        });
        // 后续 Repeat（用户按住键）
        let _ = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("a"),
            action: KeyAction::Repeat,
            modifiers: zero_ui_core::event::Modifiers::NONE,
            text: Some("a".into()),
        });
        // 应该插入了 2 个 a
        assert_eq!(w.state.text, "aa", "Pressed + Repeat 应各插一次字符");
    }

    /// P1-5 回归：Ctrl+A 全选。anchor=0，cursor=len。
    /// **键码契约**：winit Character("a") → KeyCode("a")（小写字面值），不是 "KeyA"。
    #[test]
    fn ctrl_a_selects_all() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        w.state.text = "hello".into();
        w.state.cursor = 2;
        w.state.anchor = 2;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        let res = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("a"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::CONTROL,
            text: Some("a".into()),
        });
        assert_eq!(w.state.anchor, 0, "Ctrl+A 应 anchor=0");
        assert_eq!(w.state.cursor, 5, "Ctrl+A 后 cursor=末尾");
        assert!(w.state.has_selection(), "Ctrl+A 后应有选区");
        assert!(matches!(res, EventResult::Consumed), "全选不应 emit");
    }

    /// P1-5 回归：Ctrl+C 复制选区文本（emit clipboard_copy action）。
    #[test]
    fn ctrl_c_emits_clipboard_copy() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        w.state.text = "hello".into();
        // 选区 "ell"：anchor=1, cursor=4。
        w.state.anchor = 1;
        w.state.cursor = 4;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        let res = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("c"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::CONTROL,
            text: None,
        });
        match res {
            EventResult::EmitWithPayload(id, ActionPayload::Text(t)) => {
                assert_eq!(t, "ell", "Ctrl+C 应复制选区内容");
                assert_eq!(id.0.as_str(), "text_input.clipboard_copy");
            }
            other => panic!("期望 EmitWithPayload(clipboard_copy)，实际 {other:?}"),
        }
        // 复制不修改文本/选区。
        assert_eq!(w.state.text, "hello");
        assert!(w.state.has_selection());
    }

    /// P1-5 回归：Ctrl+X 剪切——既 emit 复制 action，又删除选区。
    #[test]
    fn ctrl_x_cuts_selection() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        w.state.text = "hello".into();
        w.state.anchor = 1;
        w.state.cursor = 4; // "ell"
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        let res = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("x"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::CONTROL,
            text: None,
        });
        match res {
            EventResult::EmitWithPayload(_, ActionPayload::Text(t)) => assert_eq!(t, "ell"),
            other => panic!("期望 EmitWithPayload，实际 {other:?}"),
        }
        // 剪切后文本只剩 "ho"，cursor=1，anchor=1（无选区）。
        assert_eq!(w.state.text, "ho");
        assert_eq!(w.state.cursor, 1);
        assert_eq!(w.state.anchor, 1);
        assert!(!w.state.has_selection());
    }

    /// P1-5 回归：Ctrl+V emit clipboard_paste 请求（host 回填）。
    #[test]
    fn ctrl_v_emits_paste_request() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        w.state.text = "abc".into();
        w.state.cursor = 3;
        w.state.anchor = 3;
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        let res = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("v"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::CONTROL,
            text: None,
        });
        match res {
            EventResult::Emit(id) => assert_eq!(id.0.as_str(), "text_input.clipboard_paste"),
            other => panic!("期望 Emit(clipboard_paste)，实际 {other:?}"),
        }
        // 粘贴请求本身不修改文本（等 host 读剪贴板后回填 text prop）。
        assert_eq!(w.state.text, "abc");
    }

    /// P1-3 回归：Shift+方向键扩展选区（anchor 不动，cursor 移动）。
    #[test]
    fn shift_arrow_extends_selection() {
        let mut st = TextInputState::from_text("hello");
        st.cursor = 3; // 在 "llo" 第二个 l 后
        st.anchor = 3;
        // Shift+Left：cursor 左移，anchor 不动 → 产生选区。
        st.move_cursor(-1, true);
        assert_eq!(st.cursor, 2);
        assert_eq!(st.anchor, 3, "Shift+Left 应保留 anchor");
        assert!(st.has_selection());
        assert_eq!(st.selected_text(), "l");
        // Shift+Right 再回：cursor 右移到 3，与 anchor 重合 → 选区消失。
        st.move_cursor(1, true);
        assert_eq!(st.cursor, 3);
        assert!(!st.has_selection());
    }

    /// P1-3 回归：select_word_at 双击选词（与浏览器地址栏同口径）。
    #[test]
    fn select_word_at_picks_word() {
        let mut st = TextInputState::from_text("hello world");
        // 点击落在 "world" 的 'o' 字节偏移（"hello " = 6 字节 + 'w'=1 + 'o'=1 = 8）。
        st.select_word_at(8);
        assert_eq!(st.selected_text(), "world");
    }

    /// P1-3 回归：select_word_at 对 CJK 字符串，连续汉字视为一词（与浏览器一致）。
    #[test]
    fn select_word_at_single_cjk_char() {
        let mut st = TextInputState::from_text("你好");
        // 点击第一个字"你"（字节偏移 0）。
        st.select_word_at(0);
        // CJK 字符 is_alphanumeric()=true，连续汉字合并为"一词"（浏览器行为）。
        assert_eq!(st.selected_text(), "你好");
    }

    /// P1-3 回归：select_word_at 在 CJK + 空格 + CJK 场景按词分隔。
    #[test]
    fn select_word_at_cjk_with_space() {
        let mut st = TextInputState::from_text("你好 世界");
        // 点击"世界"区域（字节偏移 = "你好 " = 7 字节，落在 '世' 上）。
        st.select_word_at(7);
        assert_eq!(st.selected_text(), "世界");
    }

    /// P1-5 回归：Ctrl+A 大小写兼容（CapsLock 时 winit 传 "A" 大写）。
    #[test]
    fn ctrl_a_case_insensitive() {
        let mut w = TextInputWidget::new();
        w.focused = true;
        w.state.text = "hello".into();
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let mut ec = EventCtx { invalidation: &mut flags };
        // 大写 "A"（CapsLock 或 Shift+Ctrl+A）
        let _ = w.event(&mut ec, &UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("A"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::CONTROL,
            text: Some("a".into()),
        });
        assert_eq!(w.state.anchor, 0, "Ctrl+A 大写也应全选");
        assert_eq!(w.state.cursor, 5);
        assert!(w.state.has_selection());
    }
}
