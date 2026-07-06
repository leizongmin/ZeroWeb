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
        let bg = if self.pressed { tokens.primary } else { tokens.surface };
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
                EventResult::EmitWithPayload(
                    self.action.clone(),
                    ActionPayload::Text(self.page_id.clone()),
                )
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
        if self.selected {
            let washed = Color::rgb(
                tokens.primary.r * 0.35 + tokens.surface.r * 0.65,
                tokens.primary.g * 0.35 + tokens.surface.g * 0.65,
                tokens.primary.b * 0.35 + tokens.surface.b * 0.65,
            );
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), washed);
        }
        if self.pressed {
            ctx.recorder
                .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.primary);
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
                EventResult::EmitWithPayload(
                    self.action.clone(),
                    ActionPayload::Text(self.group.clone()),
                )
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
        let prefix = if self.collapsed { "▸ " } else { "▾ " };
        let display = format!("{}{}", prefix, self.label);
        let fg = if self.pressed {
            tokens.primary
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
        }
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
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        let UiEvent::Key {
            code,
            action: KeyAction::Pressed,
            text,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
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
        let border = Color::rgb(
            tokens.on_background.r * 0.4 + tokens.background.r * 0.6,
            tokens.on_background.g * 0.4 + tokens.background.g * 0.6,
            tokens.on_background.b * 0.4 + tokens.background.b * 0.6,
        );
        ctx.recorder
            .fill_rect(Rect::from_origin_size(Point::ZERO, size), tokens.background);
        ctx.recorder
            .stroke_rect(Rect::from_origin_size(Point::ZERO, size), border, 1.0);
        let display = if self.query.is_empty() {
            self.placeholder.clone()
        } else {
            format!("🔍 {}", self.query)
        };
        let fg = Color::rgb(
            tokens.on_background.r * 0.5 + tokens.background.r * 0.5,
            tokens.on_background.g * 0.5 + tokens.background.g * 0.5,
            tokens.on_background.b * 0.5 + tokens.background.b * 0.5,
        );
        ctx.recorder.draw_text(&display, Point::new(8.0, 22.0), 13.0, fg);
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
