//! Chrome widgets — 应用外壳通用控件（P3-6-3/4，从 gallery chrome 提升到 ui-sdk）。
//!
//! 提供 7 个应用层常用 chrome 控件，全部浏览器/业务无关：
//! - [`HeaderTitle`]：应用顶部大标题
//! - [`HeaderButton`]：顶部操作按钮（语言/主题切换等）
//! - [`NavItem`]：侧栏导航项（selected 高亮 + 点击 emit `nav.select` action）
//! - [`NavSearch`]：侧栏搜索框（key 输入 + placeholder）
//! - [`GroupHeader`]：分组标题（折叠/展开 + 点击 emit `group.toggle` action）
//! - [`DemoTitle`]：内容区标题 + 描述
//! - [`Spacer`]：弹性占位
//!
//! 主题色统一从 `PaintCtx.tokens` 取（不存 theme 字段）；action 通过 prop 注入（解耦业务）。

use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{KeyAction, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::prop_keys;
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget};

// ========== 共享 helper ==========

pub(crate) fn sync_text(props: &Props, key: &str, field: &mut String) -> bool {
    if let Some(Value::Text(s)) = props.get(key)
        && s != field
    {
        *field = s.clone();
        true
    } else {
        false
    }
}

pub(crate) fn mark_paint_if_changed(ctx: &mut UpdateCtx, changed: bool) {
    if changed {
        *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
    }
}

pub(crate) fn mark_layout_if_changed(ctx: &mut UpdateCtx, changed: bool) {
    if changed {
        *ctx.invalidation |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
    }
}

// ========== HeaderTitle ==========

/// 应用顶部大标题。
pub struct HeaderTitle {
    pub(crate) text: String,
}

impl Default for HeaderTitle {
    fn default() -> Self {
        HeaderTitle::new()
    }
}

impl HeaderTitle {
    pub fn new() -> HeaderTitle {
        HeaderTitle { text: String::new() }
    }
}

impl Widget for HeaderTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let text_changed = sync_text(props, prop_keys::TEXT, &mut self.text);
        mark_layout_if_changed(ctx, text_changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let w = ctx.measure_text(&self.text, 18.0).width + 24.0;
        Size::new(
            w.clamp(c.min_width, c.max_width),
            40.0_f32.clamp(c.min_height, c.max_height),
        )
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder
            .draw_text(&self.text, Point::new(12.0, 26.0), 18.0, ctx.tokens.on_background);
    }
}

// ========== Spacer ==========

/// 弹性占位（与 `flex: 1` prop 配合吃剩余主轴空间）。
pub struct Spacer {
    pub(crate) axis: String,
}

impl Default for Spacer {
    fn default() -> Self {
        Spacer::new()
    }
}

impl Spacer {
    pub fn new() -> Spacer {
        Spacer {
            axis: String::from("horizontal"),
        }
    }
}

impl Widget for Spacer {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(a)) = props.get(prop_keys::AXIS)
            && a != &self.axis
        {
            self.axis = a.clone();
            *ctx.invalidation |= InvalidationFlags::NEEDS_LAYOUT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        match self.axis.as_str() {
            "vertical" => Size::new(0.0, c.max_height.max(c.min_height)),
            _ => Size::new(c.max_width.max(c.min_width), 0.0),
        }
    }
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
}

// ========== HeaderButton ==========

/// 顶部操作按钮（语言/主题切换等）。
///
/// 点击 emit `action` prop 指定的 action（默认 "noop"）。
pub struct HeaderButton {
    pub(crate) label: String,
    pub(crate) action: ActionId,
    pub(crate) pressed: bool,
    /// P1-5：hover 态（鼠标悬停时背景变化）。
    pub(crate) hover: bool,
}

impl Default for HeaderButton {
    fn default() -> Self {
        HeaderButton::new()
    }
}

impl HeaderButton {
    pub fn new() -> HeaderButton {
        HeaderButton {
            label: String::new(),
            action: ActionId::new("noop"),
            pressed: false,
            hover: false,
        }
    }
}

impl Widget for HeaderButton {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, prop_keys::LABEL, &mut self.label);
        if let Some(Value::Text(a)) = props.get(prop_keys::ACTION) {
            self.action = ActionId::new(a);
        }
        mark_layout_if_changed(ctx, label_changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            } => {
                self.pressed = false;
                EventResult::Emit(self.action.clone())
            }
            // P1-5：hover 态。
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                ..
            } => {
                if !self.hover {
                    self.hover = true;
                }
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Exited,
                ..
            } => {
                self.hover = false;
                self.pressed = false;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let content_w = (self.label.chars().count() as f32 * 10.0 + 24.0).max(64.0);
        Size::new(
            content_w.clamp(c.min_width, c.max_width),
            32.0_f32.clamp(c.min_height, c.max_height),
        )
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(64.0, 32.0));
        // P1-5：pressed > hover > default 三态背景。
        let bg = if self.pressed {
            tokens.primary
        } else if self.hover {
            // hover：surface 与 on_background 8% 混合（轻微高亮）。
            Color::rgb(
                tokens.surface.r * 0.92 + tokens.on_background.r * 0.08,
                tokens.surface.g * 0.92 + tokens.on_background.g * 0.08,
                tokens.surface.b * 0.92 + tokens.on_background.b * 0.08,
            )
        } else {
            tokens.surface
        };
        ctx.recorder.fill_rect(Rect::from_origin_size(Point::ZERO, size), bg);
        let border = Color::rgb(
            tokens.on_background.r * 0.3 + tokens.surface.r * 0.7,
            tokens.on_background.g * 0.3 + tokens.surface.g * 0.7,
            tokens.on_background.b * 0.3 + tokens.surface.b * 0.7,
        );
        ctx.recorder
            .stroke_rect(Rect::from_origin_size(Point::ZERO, size), border, 1.0);
        let on_bg = if self.pressed {
            tokens.on_primary
        } else {
            tokens.on_surface
        };
        ctx.recorder.draw_text(&self.label, Point::new(12.0, 22.0), 14.0, on_bg);
    }
    fn focusable(&self) -> bool {
        true
    }
}

// ========== NavItem ==========

/// 侧栏导航项。
///
/// `action` prop 指定点击 emit 的 action（默认 "nav.select"）；payload 是 `page_id` prop。
pub struct NavItem {
    pub(crate) label: String,
    pub(crate) page_id: String,
    pub(crate) action: ActionId,
    pub(crate) selected: bool,
    pub(crate) pressed: bool,
    /// P1-5：hover 态。
    pub(crate) hover: bool,
}

impl Default for NavItem {
    fn default() -> Self {
        NavItem::new()
    }
}

impl NavItem {
    pub fn new() -> NavItem {
        NavItem {
            label: String::new(),
            page_id: String::new(),
            action: ActionId::new("nav.select"),
            selected: false,
            pressed: false,
            hover: false,
        }
    }
}

impl Widget for NavItem {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, prop_keys::LABEL, &mut self.label);
        if let Some(Value::Text(p)) = props.get(prop_keys::PAGE_ID) {
            self.page_id = p.clone();
        }
        if let Some(Value::Text(a)) = props.get(prop_keys::ACTION) {
            self.action = ActionId::new(a);
        }
        let mut paint_changed = false;
        if let Some(Value::Bool(s)) = props.get(prop_keys::SELECTED)
            && *s != self.selected
        {
            self.selected = *s;
            paint_changed = true;
        }
        mark_layout_if_changed(ctx, label_changed);
        mark_paint_if_changed(ctx, paint_changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            } => {
                self.pressed = false;
                EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(self.page_id.clone()))
            }
            // P1-5：hover 态。
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                ..
            } => {
                self.hover = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Exited,
                ..
            } => {
                self.hover = false;
                self.pressed = false;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let content_w = 16.0 + self.label.chars().count() as f32 * 8.0 + 16.0;
        let w = content_w.clamp(c.min_width, c.max_width);
        Size::new(w, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        // P1-5：selected > pressed > hover > default 四态背景。
        if self.selected {
            let washed = Color::rgb(
                tokens.primary.r * 0.35 + tokens.surface.r * 0.65,
                tokens.primary.g * 0.35 + tokens.surface.g * 0.65,
                tokens.primary.b * 0.35 + tokens.surface.b * 0.65,
            );
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), washed);
        } else if self.pressed {
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.primary);
        } else if self.hover {
            // hover：surface 与 on_background 8% 混合。
            let hov = Color::rgb(
                tokens.surface.r * 0.92 + tokens.on_background.r * 0.08,
                tokens.surface.g * 0.92 + tokens.on_background.g * 0.08,
                tokens.surface.b * 0.92 + tokens.on_background.b * 0.08,
            );
            ctx.recorder.fill_rect(Rect::from_origin_size(Point::ZERO, size), hov);
        }
        if self.pressed && !self.selected {
            ctx.recorder
                .draw_text(&self.label, Point::new(16.0, 22.0), 14.0, tokens.on_primary);
            return;
        }
        ctx.recorder
            .draw_text(&self.label, Point::new(16.0, 22.0), 14.0, tokens.on_surface);
    }
    fn focusable(&self) -> bool {
        true
    }
}

// ========== GroupHeader ==========

/// 分组标题（折叠/展开）。
///
/// `action` prop 指定点击 emit 的 action（默认 "group.toggle"）；payload 是 `group` prop。
pub struct GroupHeader {
    pub(crate) label: String,
    pub(crate) group: String,
    pub(crate) action: ActionId,
    pub(crate) collapsed: bool,
    pub(crate) pressed: bool,
    /// P1-5：hover 态。
    pub(crate) hover: bool,
}

impl Default for GroupHeader {
    fn default() -> Self {
        GroupHeader::new()
    }
}

impl GroupHeader {
    pub fn new() -> GroupHeader {
        GroupHeader {
            label: String::new(),
            group: String::new(),
            action: ActionId::new("group.toggle"),
            collapsed: false,
            pressed: false,
            hover: false,
        }
    }
}

impl Widget for GroupHeader {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, prop_keys::LABEL, &mut self.label);
        if let Some(Value::Text(g)) = props.get(prop_keys::GROUP) {
            self.group = g.clone();
        }
        if let Some(Value::Text(a)) = props.get(prop_keys::ACTION) {
            self.action = ActionId::new(a);
        }
        let mut paint_changed = false;
        if let Some(Value::Bool(c)) = props.get(prop_keys::COLLAPSED)
            && *c != self.collapsed
        {
            self.collapsed = *c;
            paint_changed = true;
        }
        mark_layout_if_changed(ctx, label_changed);
        mark_paint_if_changed(ctx, paint_changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        match event {
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } => {
                self.pressed = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            } => {
                self.pressed = false;
                EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(self.group.clone()))
            }
            // P1-5：hover 态。
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                ..
            } => {
                self.hover = true;
                EventResult::Consumed
            }
            UiEvent::Pointer {
                phase: PointerPhase::Exited,
                ..
            } => {
                self.hover = false;
                self.pressed = false;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let content_w = 8.0 + self.label.chars().count() as f32 * 8.0 + 8.0;
        let w = content_w.clamp(c.min_width, c.max_width);
        Size::new(w, 28.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 28.0));
        // P1-5：hover 背景。
        if self.hover || self.pressed {
            let hov = Color::rgb(
                tokens.surface.r * 0.92 + tokens.on_background.r * 0.08,
                tokens.surface.g * 0.92 + tokens.on_background.g * 0.08,
                tokens.surface.b * 0.92 + tokens.on_background.b * 0.08,
            );
            ctx.recorder.fill_rect(Rect::from_origin_size(Point::ZERO, size), hov);
        }
        let prefix = if self.collapsed { "▸ " } else { "▾ " };
        let display = format!("{}{}", prefix, self.label);
        let fg = if self.pressed || self.hover {
            tokens.on_background
        } else {
            Color::rgb(
                tokens.on_background.r * 0.6 + tokens.surface.r * 0.4,
                tokens.on_background.g * 0.6 + tokens.surface.g * 0.4,
                tokens.on_background.b * 0.6 + tokens.surface.b * 0.4,
            )
        };
        ctx.recorder.draw_text(&display, Point::new(8.0, 18.0), 12.0, fg);
    }
    fn focusable(&self) -> bool {
        true
    }
}

// ========== NavSearch ==========

/// 侧栏搜索框。
///
/// `placeholder` prop 指定空查询时的占位文字（应用层负责 i18n）。
/// `action` prop 指定 keypress emit 的 action（默认 "search"）；payload 是当前 query。
pub struct NavSearch {
    pub(crate) query: String,
    pub(crate) placeholder: String,
    pub(crate) action: ActionId,
    /// U3-3 修复：追踪 focus/hover 状态，focused 时显示 caret（与 TextInput 一致）。
    pub(crate) focused: bool,
    pub(crate) hover: bool,
    /// CJK 修复：缓存每个字符边界对应的累计 x 偏移（含起始 8.0 padding）。
    /// `char_x[i]` = query 前 i 个字符累计像素宽度 + 8.0。长度 = chars + 1。
    pub(crate) char_x: Vec<f32>,
}

impl Default for NavSearch {
    fn default() -> Self {
        NavSearch::new()
    }
}

impl NavSearch {
    pub fn new() -> NavSearch {
        NavSearch {
            query: String::new(),
            placeholder: String::from("Search..."),
            action: ActionId::new("search"),
            focused: false,
            hover: false,
            char_x: vec![8.0],
        }
    }

    /// 用 measure_text 重建 char_x 缓存（paint 阶段调用）。
    fn rebuild_char_x(&mut self, ctx: &mut PaintCtx, font_size: f32) {
        let mut xs = Vec::with_capacity(self.query.chars().count() + 1);
        xs.push(8.0);
        let mut buf = String::new();
        for ch in self.query.chars() {
            buf.push(ch);
            let w = ctx.measure_text(&buf, font_size).width;
            xs.push(8.0 + w);
        }
        self.char_x = xs;
    }
}

impl Widget for NavSearch {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let query_changed = sync_text(props, prop_keys::QUERY, &mut self.query);
        let placeholder_changed = sync_text(props, "placeholder", &mut self.placeholder);
        if let Some(Value::Text(a)) = props.get(prop_keys::ACTION) {
            self.action = ActionId::new(a);
        }
        mark_paint_if_changed(ctx, query_changed || placeholder_changed);
    }
    fn event(&mut self, ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        // U3-3：focus/hover 与 TextInput 同口径。
        match event {
            UiEvent::Focus(zero_ui_core::event::FocusEvent::Gained) => {
                self.focused = true;
                *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
                return EventResult::Consumed;
            }
            UiEvent::Focus(zero_ui_core::event::FocusEvent::Lost) => {
                self.focused = false;
                *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
                return EventResult::Consumed;
            }
            UiEvent::Pointer { phase, position, .. } => match phase {
                PointerPhase::Moved => {
                    if !self.hover {
                        self.hover = true;
                        *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
                    }
                    return EventResult::Consumed;
                }
                PointerPhase::Pressed => {
                    // CJK 修复：点击定位不适用 NavSearch（受控单 query，caret 永远在末尾）。
                    // 忽略点击位置，仅消费事件让 host 给 focus。
                    let _ = position;
                    return EventResult::Consumed;
                }
                PointerPhase::Exited => {
                    self.hover = false;
                    *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
                    return EventResult::Consumed;
                }
                _ => {}
            },
            UiEvent::Ime(ime) => {
                // CJK 输入修复：IME Commit 把合成完成的文本追加到 query（受控 emit）。
                if let zero_ui_core::event::ImeEvent::Commit(s) = ime
                    && !s.is_empty()
                {
                    let q = format!("{}{}", self.query, s);
                    return EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(q));
                }
                return EventResult::Consumed;
            }
            _ => {}
        }
        let UiEvent::Key {
            code,
            action,
            text,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
        // P1-3：接受 Repeat（按住键重复输入），与 Pressed 同路径。
        if !matches!(action, KeyAction::Pressed | KeyAction::Repeat) {
            return EventResult::Ignored;
        }
        match code.0.as_str() {
            "Backspace" => {
                let mut q = self.query.clone();
                q.pop();
                EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(q))
            }
            "Enter" | "Escape" => {
                if self.query.is_empty() {
                    EventResult::Ignored
                } else {
                    EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(String::new()))
                }
            }
            _ => match text {
                Some(ch) => {
                    if ch.chars().any(|c| c.is_control()) {
                        return EventResult::Ignored;
                    }
                    let q = format!("{}{}", self.query, ch);
                    EventResult::EmitWithPayload(self.action.clone(), ActionPayload::Text(q))
                }
                None => EventResult::Ignored,
            },
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let w = 200.0_f32.clamp(c.min_width, c.max_width);
        Size::new(w, 32.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(220.0, 32.0));
        // U3-3：focused 用 primary 高亮，hover 用次级色，否则中性边框（与 TextInput 一致）。
        let border = if self.focused {
            tokens.primary
        } else if self.hover {
            tokens.primary.mix(tokens.surface, 0.4)
        } else {
            tokens.on_background.mix(tokens.background, 0.6)
        };
        ctx.recorder
            .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.background);
        ctx.recorder.stroke_rect(
            Rect::from_origin_size(Point::ZERO, size),
            border,
            if self.focused { 2.0 } else { 1.0 },
        );
        // CJK 修复：去掉 "🔍 " 前缀（emoji 宽度不固定，让 caret 计算更复杂）。
        // 搜索框语义：query 即显示文本，空时显 placeholder。
        let display = if self.query.is_empty() {
            self.placeholder.clone()
        } else {
            self.query.clone()
        };
        let fg = if self.query.is_empty() {
            tokens.on_background.mix(tokens.background, 0.5)
        } else {
            tokens.on_background
        };
        let text_x = 8.0;
        ctx.recorder.draw_text(&display, Point::new(text_x, 22.0), 13.0, fg);
        // CJK 修复：用真实字体度量重建 query 的 char_x（仅在 query 非空时）。
        if !self.query.is_empty() {
            self.rebuild_char_x(ctx, 13.0);
        }
        // U3-3：focused 时显示 caret（与 TextInput 一致）。caret 始终在 query 末尾
        // （NavSearch 是受控追加型，无中间编辑）。
        // P1-4：500ms 周期闪烁（与 TextInput 同周期）。
        if self.focused {
            let caret_visible = match ctx.now_ms {
                Some(ms) => {
                    let phase = (ms % 1068) < 534;
                    ctx.request_frame();
                    phase
                }
                None => true,
            };
            if caret_visible {
                let caret_x = if self.query.is_empty() {
                    text_x
                } else {
                    *self.char_x.last().unwrap_or(&text_x)
                };
                ctx.recorder.fill_rect(
                    Rect::from_origin_size(Point::new(caret_x, 8.0), Size::new(1.5, 16.0)),
                    tokens.primary,
                );
            }
        }
    }
    fn focusable(&self) -> bool {
        true
    }
}

// ========== DemoTitle ==========

/// 内容区标题 + 描述。
pub struct DemoTitle {
    pub(crate) text: String,
    pub(crate) desc: String,
}

impl Default for DemoTitle {
    fn default() -> Self {
        DemoTitle::new()
    }
}

impl DemoTitle {
    pub fn new() -> DemoTitle {
        DemoTitle {
            text: String::new(),
            desc: String::new(),
        }
    }
}

impl Widget for DemoTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let changed =
            sync_text(props, prop_keys::TEXT, &mut self.text) || sync_text(props, prop_keys::DESC, &mut self.desc);
        mark_paint_if_changed(ctx, changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let h = 60.0_f32.clamp(c.min_height, c.max_height);
        Size::new(c.max_width, h)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        ctx.recorder
            .draw_text(&self.text, Point::new(16.0, 24.0), 20.0, tokens.on_background);
        let desc_fg = Color::rgb(
            tokens.on_background.r * 0.55 + tokens.background.r * 0.45,
            tokens.on_background.g * 0.55 + tokens.background.g * 0.45,
            tokens.on_background.b * 0.55 + tokens.background.b * 0.45,
        );
        ctx.recorder
            .draw_text(&self.desc, Point::new(16.0, 46.0), 13.0, desc_fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// U3-3 回归：NavSearch 收到 Focus(Gained) 后进入 focused 状态，且标记 NEEDS_PAINT。
    #[test]
    fn nav_search_focus_sets_focused_and_marks_paint() {
        let mut w = NavSearch::new();
        let mut flags = InvalidationFlags::CLEAN;
        let mut ec = EventCtx {
            invalidation: &mut flags,
        };
        let res = w.event(&mut ec, &UiEvent::Focus(zero_ui_core::event::FocusEvent::Gained));
        assert!(matches!(res, EventResult::Consumed));
        assert!(w.focused, "Focus(Gained) 应进入 focused 态");
        assert!(
            flags.contains(InvalidationFlags::NEEDS_PAINT),
            "Focus(Gained) 应标记 NEEDS_PAINT"
        );
    }

    /// U3-3 回归：NavSearch focused 后，输入字符 emit 带 query 的 action。
    /// （键盘事件转发到 host 后，TextInput / NavSearch 才能真正输入——U1-2/U3-3 的根因。）
    #[test]
    fn nav_search_emits_query_on_keypress_when_focused() {
        let mut w = NavSearch::new();
        w.focused = true;
        let mut flags = InvalidationFlags::CLEAN;
        let mut ec = EventCtx {
            invalidation: &mut flags,
        };
        let ev = UiEvent::Key {
            code: zero_ui_core::event::KeyCode::new("KeyA"),
            action: KeyAction::Pressed,
            modifiers: zero_ui_core::event::Modifiers::NONE,
            text: Some("a".into()),
        };
        let res = w.event(&mut ec, &ev);
        match res {
            EventResult::EmitWithPayload(_, ActionPayload::Text(q)) => assert_eq!(q, "a"),
            other => panic!("期望 EmitWithPayload，实际 {other:?}"),
        }
    }

    /// CJK 输入回归：NavSearch focused 后 IME Commit("搜") 把中文追加到 query 并 emit。
    #[test]
    fn nav_search_ime_commit_appends_cjk_to_query() {
        let mut w = NavSearch::new();
        w.focused = true;
        w.query = "ab".into();
        let mut flags = InvalidationFlags::CLEAN;
        let mut ec = EventCtx {
            invalidation: &mut flags,
        };
        let res = w.event(
            &mut ec,
            &UiEvent::Ime(zero_ui_core::event::ImeEvent::Commit("搜".into())),
        );
        match res {
            EventResult::EmitWithPayload(_, ActionPayload::Text(q)) => assert_eq!(q, "ab搜"),
            other => panic!("期望 EmitWithPayload，实际 {other:?}"),
        }
    }
}
