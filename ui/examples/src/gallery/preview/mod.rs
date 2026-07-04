//! DemoPreview 的拆分实现（DC-17 refactor）。
//!
//! 每个 demo page 的视觉绘制逻辑独立到一个 painter 函数，
//! 通过 `PreviewPainter` trait + `painter_for` 分发器查找。
//! 新增 demo 只需在对应分组文件加 painter 并在 `painter_for` 注册，
//! 不再修改 DemoPreview 主体（开闭原则）。
//!
//! 共享辅助：
//! - `border_of`：根据 fill 反算边框色（保持各 painter 视觉一致）
//! - `y_text_center`：行内文本垂直居中近似

use zero_ui_core::theme::{Color, SemanticTokens};

pub mod animation;
pub mod forms;
pub mod gestures;
pub mod i18n;
pub mod navigation;
pub mod patterns;
pub mod theme;
pub mod widgets;

/// 单个 demo 预览的绘制函数签名。
///
/// - `state`：DemoPreview 持有的交互状态（如 toggle on/off 位掩码），按需消费
/// - `tokens`：当前主题的 semantic token 集合
/// - `recorder`：场景 recorder（调用方提供，封装了 ctx.recorder）
///
/// 因为 `PaintCtx` 含 lifetime 且目前不可序列化，painter 直接接受 `&mut PaintCtx`。
/// 这里改用泛型 + trait bound 让 painter 可以是自由函数。
pub trait PreviewPainter {
    fn paint(&self, state: u64, tokens: &SemanticTokens, ctx: &mut zero_ui_core::widget::PaintCtx);
}

/// 由 page_id 查找对应的 painter。
///
/// 未注册的 page_id 返回 None，调用方回落到默认 "{page} preview" 文案。
pub fn painter_for(page_id: &str) -> Option<Box<dyn PreviewPainter>> {
    Some(match page_id {
        // widgets
        "button" => Box::new(widgets::ButtonPainter),
        "toggle" => Box::new(widgets::TogglePainter),
        "icon_button" => Box::new(widgets::IconButtonPainter),
        "badge" => Box::new(widgets::BadgePainter),
        "progress" => Box::new(widgets::ProgressPainter),
        "text_input" => Box::new(widgets::TextInputPainter),
        "tabs" => Box::new(widgets::TabsPainter),
        "tooltip" => Box::new(widgets::TooltipPainter),
        "list_view" => Box::new(widgets::ListViewPainter),
        "menu" => Box::new(widgets::MenuPainter),
        "search_field" => Box::new(widgets::SearchFieldPainter),
        "status_bubble" => Box::new(widgets::StatusBubblePainter),
        // patterns
        "collection_demo" => Box::new(patterns::CollectionPainter),
        "dsl_demo" => Box::new(patterns::DslPainter),
        "data_list" => Box::new(patterns::DataListPainter),
        "command_palette" => Box::new(patterns::CommandPalettePainter),
        "tab_bar" => Box::new(patterns::TabBarPainter),
        // forms
        "form_demo" => Box::new(forms::FormPainter),
        // gestures
        "gesture_demo" => Box::new(gestures::GesturePainter),
        // animation
        "animation_demo" => Box::new(animation::AnimationPainter),
        // navigation
        "nav_demo" => Box::new(navigation::NavPainter),
        "dialog_scaffold" => Box::new(navigation::DialogPainter),
        "popover" => Box::new(navigation::PopoverPainter),
        "popup" => Box::new(navigation::PopupPainter),
        "toolbar" => Box::new(navigation::ToolbarPainter),
        // theme / i18n
        "theme_demo" => Box::new(theme::ThemePainter),
        "i18n_demo" => Box::new(i18n::I18nPainter),
        _ => return None,
    })
}

/// 根据填充色反算 1px 边框色：on_background 25% + fill 75%。
/// 各 painter 共享，保证边框对比度一致。
pub fn border_of(tokens: &SemanticTokens, fill: Color) -> Color {
    Color::rgb(
        tokens.on_background.r * 0.25 + fill.r * 0.75,
        tokens.on_background.g * 0.25 + fill.g * 0.75,
        tokens.on_background.b * 0.25 + fill.b * 0.75,
    )
}

/// 13px 字体在 [row_y, row_y + row_h] 区间内的基线 y（垂直居中近似）。
pub fn y_text_center(row_y: f32, row_h: f32) -> f32 {
    row_y + (row_h + 13.0) * 0.5
}
