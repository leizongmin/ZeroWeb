//! Gallery 通用 chrome 控件（DC-17 refactor）。
//!
//! 这里集中了 sidebar/header/demo pane 用到的「外壳」控件（不参与 DemoPreview 派发）：
//! - HeaderTitle / HeaderButton / NavItem / NavSearch / GroupHeader / DemoTitle / Spacer
//! - 共享 helper：theme_from_props / locale_from_props / tokens_for
//!
//! 这些控件原本散在 app.rs 中（占 ~500 行），拆出来后 app.rs 只保留 GalleryApp 主体、
//! DemoPreview dispatcher 和 SourceCode/SourceLabel。

use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{KeyAction, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

use super::model::{Locale, ThemeKind};

// ========== 共享 helper ==========

/// 从 props 解析 theme（缺失或非法时回落 Light，与默认主题 token 一致）。
pub(crate) fn theme_from_props(props: &Props) -> ThemeKind {
    match props.get("theme") {
        Some(Value::Text(s)) => ThemeKind::parse_str(s).unwrap_or(ThemeKind::Light),
        _ => ThemeKind::Light,
    }
}

/// 从 `locale` prop 解析当前语言（非法/缺省回落 En）。
pub(crate) fn locale_from_props(props: &Props) -> Locale {
    match props.get("locale") {
        Some(Value::Text(s)) => Locale::parse_str(s).unwrap_or(Locale::En),
        _ => Locale::En,
    }
}

/// 由 ThemeKind 取 semantic token 色板（与 host 持有的 tokens 同口径）。
pub(crate) fn tokens_for(theme: ThemeKind) -> SemanticTokens {
    match theme {
        ThemeKind::Light => SemanticTokens::light(),
        ThemeKind::Dark => SemanticTokens::dark(),
    }
}

/// 便利 helper：从 props 读 theme 写回 `field`，返回是否变化。
///
/// 用来减少每个 chrome widget update 中重复的 4 行 `theme_from_props + 比对 + 写字段` 代码。
/// 调用方负责把变化标到 NEEDS_PAINT（因为同时还可能要看其它字段是否变化）。
pub(crate) fn sync_theme(props: &Props, field: &mut ThemeKind) -> bool {
    let new_theme = theme_from_props(props);
    let changed = new_theme != *field;
    *field = new_theme;
    changed
}

/// 便利 helper：从 props 读文本字段写回 `field`，返回是否变化。
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

/// 便利 helper：变化时标记 NEEDS_PAINT（多 helper 结果合并）。
pub(crate) fn mark_paint_if_changed(ctx: &mut UpdateCtx, changed: bool) {
    if changed {
        *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
    }
}

/// 便利 helper：变化时同时标记 NEEDS_LAYOUT + NEEDS_PAINT。
///
/// 用于 layout 依赖该字段的场景（如 chrome widget 的 label 文本长度决定按钮宽度）。
/// 标 NEEDS_LAYOUT 会连带上溯重算父级布局。
pub(crate) fn mark_layout_if_changed(ctx: &mut UpdateCtx, changed: bool) {
    if changed {
        *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT
            | zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
    }
}

// ========== HeaderTitle ==========

/// Header 标题
pub struct HeaderTitle {
    pub(crate) text: String,
    pub(crate) theme: ThemeKind,
}

impl Widget for HeaderTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        // 文本长度决定 layout 宽度 → sync_text 走 layout；theme 仅色变走 paint。
        let text_changed = sync_text(props, "text", &mut self.text);
        let theme_changed = sync_theme(props, &mut self.theme);
        mark_layout_if_changed(ctx, text_changed);
        mark_paint_if_changed(ctx, theme_changed);
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        let w = (self.text.chars().count() as f32 * 9.0).clamp(c.min_width, c.max_width);
        Size::new(w, 40.0_f32.clamp(c.min_height, c.max_height))
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = tokens_for(self.theme);
        ctx.recorder
            .draw_text(&self.text, Point::new(12.0, 26.0), 18.0, tokens.on_background);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ========== Spacer ==========

/// 弹性占位（与 `flex: 1` prop 配合吃剩余主轴空间）。
///
/// 由于 layout 引擎不会把 clamp 后的尺寸写回子节点 `cached_size`（仅用于父级排列），
/// Spacer 必须自己控制返回的尺寸：仅吃主轴、cross 维度返回 0，避免把父容器
/// 的 cross（常常是窗口另一维）拉满导致兄弟容器被挤出视口。
/// `axis` prop 标识所在容器（"horizontal"=Row / "vertical"=Column）。
pub struct Spacer {
    pub(crate) axis: String,
}

impl Widget for Spacer {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(a)) = props.get("axis")
            && a != &self.axis
        {
            self.axis = a.clone();
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ========== HeaderButton ==========

/// Header 按钮（语言/主题切换等）
pub struct HeaderButton {
    pub(crate) label: String,
    pub(crate) action: ActionId,
    pub(crate) pressed: bool,
    pub(crate) theme: ThemeKind,
}

impl Widget for HeaderButton {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, "label", &mut self.label);
        let theme_changed = sync_theme(props, &mut self.theme);
        if let Some(Value::Text(a)) = props.get("action") {
            self.action = ActionId::new(a);
        }
        mark_layout_if_changed(ctx, label_changed);
        mark_paint_if_changed(ctx, theme_changed);
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
        let tokens = tokens_for(self.theme);
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

// ========== NavItem ==========

/// 导航项
pub struct NavItem {
    pub(crate) label: String,
    pub(crate) page_id: String,
    pub(crate) selected: bool,
    pub(crate) pressed: bool,
    pub(crate) theme: ThemeKind,
}

impl Widget for NavItem {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, "label", &mut self.label);
        let theme_changed = sync_theme(props, &mut self.theme);
        if let Some(Value::Text(p)) = props.get("page_id") {
            self.page_id = p.clone();
        }
        let mut paint_changed = theme_changed;
        if let Some(Value::Bool(s)) = props.get("selected")
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
                    ActionId::new("gallery.nav.select"),
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
        let tokens = tokens_for(self.theme);
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

// ========== GroupHeader ==========

/// 分组标题
pub struct GroupHeader {
    pub(crate) label: String,
    /// 序列化进 action payload 的 group 标识（与 dispatch 端的 `{:?}` 解析对应）。
    pub(crate) group: String,
    pub(crate) collapsed: bool,
    pub(crate) pressed: bool,
    pub(crate) theme: ThemeKind,
}

impl Widget for GroupHeader {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let label_changed = sync_text(props, "label", &mut self.label);
        let theme_changed = sync_theme(props, &mut self.theme);
        if let Some(Value::Text(g)) = props.get("group") {
            self.group = g.clone();
        }
        let mut paint_changed = theme_changed;
        if let Some(Value::Bool(c)) = props.get("collapsed")
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
                    ActionId::new("gallery.group.toggle"),
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
        let tokens = tokens_for(self.theme);
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

// ========== NavSearch ==========

/// 导航搜索框
pub struct NavSearch {
    pub(crate) query: String,
    pub(crate) theme: ThemeKind,
    pub(crate) locale: Locale,
}

impl Widget for NavSearch {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let new_locale = locale_from_props(props);
        let mut changed = new_locale != self.locale;
        self.locale = new_locale;
        changed |= sync_theme(props, &mut self.theme) || sync_text(props, "query", &mut self.query);
        mark_paint_if_changed(ctx, changed);
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
                EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(q))
            }
            "Enter" | "Escape" => {
                if self.query.is_empty() {
                    EventResult::Ignored
                } else {
                    EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(String::new()))
                }
            }
            _ => match text {
                Some(ch) => {
                    if ch.chars().any(|c| c.is_control()) {
                        return EventResult::Ignored;
                    }
                    let q = format!("{}{}", self.query, ch);
                    EventResult::EmitWithPayload(ActionId::new("gallery.search"), ActionPayload::Text(q))
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
        let tokens = tokens_for(self.theme);
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
            self.locale.search_placeholder().to_string()
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

// ========== DemoTitle ==========

/// Demo 标题区域
pub struct DemoTitle {
    pub(crate) text: String,
    pub(crate) desc: String,
    pub(crate) theme: ThemeKind,
}

impl Widget for DemoTitle {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let changed = sync_theme(props, &mut self.theme)
            || sync_text(props, "text", &mut self.text)
            || sync_text(props, "desc", &mut self.desc);
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
        let tokens = tokens_for(self.theme);
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
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}
