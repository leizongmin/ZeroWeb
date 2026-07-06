//! Icon — 内置图标 widget（P3-4-2）。
//!
//! 用 Unicode 符号 + draw_text 渲染常用图标，无需 SVG 解析、无需 AssetProvider、无需 ImageRef
//! 注册——gallery 与外部宿主零依赖开箱即用。
//!
//! 设计权衡：
//! - 选 Unicode 而非 SVG path：PaintRecorder 当前没有 draw_path/draw_svg API；扩展它需要改
//!   ui/render + ui/core trait + 后端实现。Unicode 符号已被字体栈支持，零扩展成本。
//! - 字符选型：优先 Unicode 几何符号（← → ✕ ✓ ☰ ⚙ ⚠ ★ ♥ ⌘），它们在大多数字体中都有
//!   字形且语义清晰；emoji 风格符号（🎉）避免，跨平台渲染不一致。
//! - tint：默认用 tokens.on_surface；可由 prop `color` 覆盖（primary/success/danger 等命名）。
//!
//! 不响应事件；交互由 sibling Button 承担。Icon 纯视觉。

use zero_ui_core::action::EventResult;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget};

/// 内置图标枚举（20 个常用图标）。
///
/// 字符选型基于 Unicode 1.1-7.0 几何符号区段，覆盖主流字体（Segoe UI / SF Pro / Noto Sans）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    /// ← 返回。
    Back,
    /// → 前进。
    Forward,
    /// ✕ 关闭。
    Close,
    /// ✓ 完成/确认。
    Check,
    /// ☰ 菜单。
    Menu,
    /// ⚙ 设置/齿轮。
    Settings,
    /// ⚠ 警告。
    Warning,
    /// ★ 收藏/星标。
    Star,
    /// ♥ 喜欢/心。
    Heart,
    /// ⌘ 命令键（macOS 语义）/ 通用动作。
    Command,
    /// ⏵ 播放。
    Play,
    /// ⏸ 暂停。
    Pause,
    /// ⏹ 停止。
    Stop,
    /// ⏭ 下一首。
    Next,
    /// ⏮ 上一首。
    Previous,
    /// 🔍 放大镜（搜索）。注：可能是 emoji 渲染，但搜索图标极常用。
    Search,
    /// 🏠 主页。
    Home,
    /// ✉ 邮件。
    Mail,
    /// 🕐 时钟（最近/历史）。
    Clock,
    /// ℹ 信息。
    Info,
}

impl IconKind {
    /// 对应的 Unicode 字符。
    pub fn glyph(self) -> &'static str {
        match self {
            IconKind::Back => "←",
            IconKind::Forward => "→",
            IconKind::Close => "✕",
            IconKind::Check => "✓",
            IconKind::Menu => "☰",
            IconKind::Settings => "⚙",
            IconKind::Warning => "⚠",
            IconKind::Star => "★",
            IconKind::Heart => "♥",
            IconKind::Command => "⌘",
            IconKind::Play => "▶",
            IconKind::Pause => "⏸",
            IconKind::Stop => "⏹",
            IconKind::Next => "⏭",
            IconKind::Previous => "⏮",
            IconKind::Search => "🔍",
            IconKind::Home => "🏠",
            IconKind::Mail => "✉",
            IconKind::Clock => "🕐",
            IconKind::Info => "ℹ",
        }
    }

    /// 从字符串名解析（与 prop `name` 对应；未知名回退到 Info）。
    pub fn from_name(name: &str) -> IconKind {
        match name {
            "back" => IconKind::Back,
            "forward" => IconKind::Forward,
            "close" => IconKind::Close,
            "check" => IconKind::Check,
            "menu" => IconKind::Menu,
            "settings" => IconKind::Settings,
            "warning" => IconKind::Warning,
            "star" => IconKind::Star,
            "heart" => IconKind::Heart,
            "command" => IconKind::Command,
            "play" => IconKind::Play,
            "pause" => IconKind::Pause,
            "stop" => IconKind::Stop,
            "next" => IconKind::Next,
            "previous" => IconKind::Previous,
            "search" => IconKind::Search,
            "home" => IconKind::Home,
            "mail" => IconKind::Mail,
            "clock" => IconKind::Clock,
            "info" => IconKind::Info,
            _ => IconKind::Info,
        }
    }
}

/// Icon widget 控件实例。
pub struct Icon {
    /// 当前图标种类（从 prop `name` 解析）。
    kind: IconKind,
    /// `size` prop：图标尺寸（逻辑像素，正方形）；默认 20。
    size_px: f32,
    /// `color` prop：命名预设或 `#rrggbb`；默认 None = 用 tokens.on_surface。
    color_raw: Option<String>,
    /// 上次 layout 算出的尺寸。
    size: Size,
    /// 可选 a11y 标签（默认用 IconKind 的英文名）。
    label: String,
}

impl Default for Icon {
    fn default() -> Self {
        Icon::new()
    }
}

impl Icon {
    pub fn new() -> Icon {
        Icon {
            kind: IconKind::Info,
            size_px: 20.0,
            color_raw: None,
            size: Size::new(20.0, 20.0),
            label: String::new(),
        }
    }
}

impl Widget for Icon {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        let mut changed = false;
        if let Some(zero_ui_core::binding::Value::Text(name)) = props.get("name") {
            let k = IconKind::from_name(name);
            if k != self.kind {
                self.kind = k;
                changed = true;
            }
        }
        if let Some(zero_ui_core::binding::Value::Float(s)) = props.get("size") {
            let s = *s as f32;
            if s != self.size_px && s > 0.0 {
                self.size_px = s;
                *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_LAYOUT;
            }
        }
        match props.get("color") {
            Some(zero_ui_core::binding::Value::Text(c)) => {
                if self.color_raw.as_deref() != Some(c.as_str()) {
                    self.color_raw = Some(c.clone());
                    changed = true;
                }
            }
            _ => {
                if self.color_raw.is_some() {
                    self.color_raw = None;
                    changed = true;
                }
            }
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
        let s = self
            .size_px
            .clamp(c.min_width.min(c.min_height), c.max_width.min(c.max_height));
        self.size = Size::new(s, s);
        self.size
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let tokens = ctx.tokens;
        let color = match &self.color_raw {
            Some(raw) => resolve_icon_color(raw, tokens),
            None => tokens.on_surface,
        };
        // 字号与 size_px 一致；baseline 居中：icon 字符通常视觉中心略高于几何中心，
        // 用 height * 0.5 + size_px * 0.35 近似（与 button label baseline 同思路）。
        let baseline = self.size.height * 0.5 + self.size_px * 0.35;
        let x = 0.0;
        ctx.recorder
            .draw_text(self.kind.glyph(), Point::new(x, baseline), self.size_px, color);
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        let label = if self.label.is_empty() {
            format!("{:?}", self.kind)
        } else {
            self.label.clone()
        };
        ctx.nodes.push(SemanticsNode {
            id: zero_ui_core::widget::WidgetId::new("icon"),
            rect: Rect::ZERO,
            flags: SemanticsFlags::NONE,
            label: Some(SemanticsLabel::Literal(label.into())),
            value: None,
            children: Vec::new(),
        });
    }
}

/// 与 ColoredBox::resolve_color 一致的命名预设解析；unknown 回退到 on_surface。
fn resolve_icon_color(raw: &str, tokens: &zero_ui_core::theme::SemanticTokens) -> Color {
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
            Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        _ => tokens.on_surface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_nonempty_for_all_variants() {
        // 所有 IconKind 必须有非空 glyph（否则 paint 会画空字符串）。
        let all = [
            IconKind::Back,
            IconKind::Forward,
            IconKind::Close,
            IconKind::Check,
            IconKind::Menu,
            IconKind::Settings,
            IconKind::Warning,
            IconKind::Star,
            IconKind::Heart,
            IconKind::Command,
            IconKind::Play,
            IconKind::Pause,
            IconKind::Stop,
            IconKind::Next,
            IconKind::Previous,
            IconKind::Search,
            IconKind::Home,
            IconKind::Mail,
            IconKind::Clock,
            IconKind::Info,
        ];
        for k in all {
            assert!(!k.glyph().is_empty(), "{:?} glyph should be non-empty", k);
        }
    }

    #[test]
    fn from_name_roundtrip_for_known_names() {
        assert_eq!(IconKind::from_name("back"), IconKind::Back);
        assert_eq!(IconKind::from_name("forward"), IconKind::Forward);
        assert_eq!(IconKind::from_name("close"), IconKind::Close);
        assert_eq!(IconKind::from_name("check"), IconKind::Check);
        assert_eq!(IconKind::from_name("search"), IconKind::Search);
    }

    #[test]
    fn from_name_unknown_falls_back_to_info() {
        assert_eq!(IconKind::from_name("nonexistent"), IconKind::Info);
    }

    #[test]
    fn size_prop_drives_layout_square() {
        use zero_ui_core::geometry::Constraints;
        use zero_ui_core::widget::{LayoutCtx, Props, Widget};
        let mut icon = Icon::new();
        let mut props = Props::new();
        props.insert("size", zero_ui_core::binding::Value::Float(32.0));
        let mut inval = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        icon.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut inval,
            },
            &props,
        );
        let s = icon.layout(
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
        assert_eq!(s.width, 32.0);
        assert_eq!(s.height, 32.0, "icon 应为正方形");
    }
}
