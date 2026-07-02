//! Chrome → Scene 渲染桥（DC-7 完整验收）。
//!
//! 把 adaptive shell 产出的 `browser.*` [`WidgetSpec`] 声明树，经 [`WidgetHost`] 注册的
//! chrome 绘制工厂 paint 进统一 [`Scene`]（走 `ui/render` 的 `PaintRecorder`，**不绕过** ui/render）。
//!
//! - **容器节点**（shell 根 / ToolbarRow / Phone 顶部）由 shell 在 `props.layout` 声明为
//!   `column`/`row`（见 `ui/runtime::host` 的 `node_container_kind`），无 widget —— host 据此布局子节点。
//! - **叶子节点**（AddressBar/SecurityBadge/NavigationButtons/BrowserMenu/BrowserTabStrip/
//!   BookmarksBar/PageViewportFrame/FindBar）注册绘制工厂 [`ChromePanel`]，按 props 语义色 + 文案 paint。
//!
//! 本模块是「chrome 组件 → 可渲染 Scene」的桥，证明浏览器领域组件输出进统一 SDK scene（DC-7）。
//! 真实主题色解析在 DC-5 主题系统接入后由 semantic token 解析；此处用 [`chrome_color`] 语义色名映射。

use zero_ui_core::action::EventResult;
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetSpec,
};
use zero_ui_render::Scene;
use zero_ui_runtime::WidgetHost;

/// 语义色名 → [`Color`]（组件只消费 semantic token；DC-5 主题接入后由 ThemeResolver 解析）。
pub fn chrome_color(name: &str) -> Color {
    match name {
        "secure" => Color::rgb(0.10, 0.70, 0.30),
        "insecure" | "mixed" => Color::rgb(0.90, 0.60, 0.10),
        "dangerous" => Color::rgb(0.90, 0.20, 0.20),
        "accent" => Color::rgb(0.13, 0.58, 0.95),
        "viewport" => Color::rgb(1.0, 1.0, 1.0),
        _ => Color::rgb(0.92, 0.92, 0.93),
    }
}

/// chrome 安全状态 → 语义色名。
pub fn security_color_name(state: crate::security_badge::SecurityState) -> &'static str {
    use crate::security_badge::SecurityState;
    match state {
        SecurityState::Secure => "secure",
        SecurityState::Insecure => "insecure",
        SecurityState::Mixed => "mixed",
        SecurityState::Dangerous => "dangerous",
    }
}

/// chrome 叶子绘制控件：填充背景（语义色）+ 可选文案。
///
/// `fill_viewport = true` 时占满可用宽高（用于 PageViewportFrame）；否则占满宽、固定高（bars）。
/// paint 读 `ctx.clip` 得节点可视尺寸，以局部坐标 (0,0) 填充 —— host 按节点 origin 平移后覆盖节点 rect。
pub struct ChromePanel {
    bg: Color,
    text: Option<String>,
    text_color: Color,
    height: f32,
    fill_viewport: bool,
}

impl ChromePanel {
    /// 由声明节点构造：`bg`/`text`/`text_color`/`height` 从 props 读，缺省用传入默认。
    pub fn from_spec(spec: &WidgetSpec, default_bg: &str, default_height: f32, fill_viewport: bool) -> ChromePanel {
        let bg = match spec.props.get("bg") {
            Some(Value::Text(s)) => chrome_color(s),
            _ => chrome_color(default_bg),
        };
        let text = match spec.props.get("text") {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        };
        let text_color = match spec.props.get("text_color") {
            Some(Value::Text(s)) => chrome_color(s),
            _ => Color::rgb(0.12, 0.12, 0.12),
        };
        let height = match spec.props.get("height") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => default_height,
        };
        ChromePanel {
            bg,
            text,
            text_color,
            height,
            fill_viewport,
        }
    }
}

impl Widget for ChromePanel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        if self.fill_viewport {
            Size::new(constraints.max_width, constraints.max_height)
        } else {
            Size::new(
                constraints.max_width,
                self.height.clamp(constraints.min_height, constraints.max_height),
            )
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // ctx.clip = 祖先 clip ∩ 节点 rect（绝对坐标）；其 size 即节点可视尺寸。
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, self.height));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        if let Some(text) = &self.text {
            // 文本基线：bars 在 height-8 处；viewport 顶部 18px。
            let baseline = if self.fill_viewport {
                18.0
            } else {
                (self.height - 8.0).max(10.0)
            };
            ctx.recorder
                .draw_text(text, Point::new(6.0, baseline), 14.0, self.text_color);
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 把 chrome `browser.*` 叶子组件工厂注册到 host。
///
/// 容器节点（shell 根 / ToolbarRow）不注册 widget —— 它们经 `props.layout` 由 host 布局。
/// 调用方仍需先 `host.set_root(&shell.build(...))` 注入声明树。
pub fn register_chrome_factories(host: &mut WidgetHost) {
    // bars：固定高度的语义色条 + 可选文案。
    host.register("browser.AddressBar", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 36.0, false))
    });
    host.register("browser.NavigationButtons", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 36.0, false))
    });
    host.register("browser.BrowserMenu", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 36.0, false))
    });
    host.register("browser.BrowserTabStrip", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 32.0, false))
    });
    host.register("browser.BookmarksBar", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 28.0, false))
    });
    host.register("browser.SecurityBadge", |s| {
        Box::new(ChromePanel::from_spec(s, "secure", 28.0, false))
    });
    host.register("browser.FindBar", |s| {
        Box::new(ChromePanel::from_spec(s, "chrome", 32.0, false))
    });
    // 视口：占满剩余区域，白色背景（WebView 容器；真实网页由 WebViewWidget 在 M2 后期接入）。
    host.register("browser.PageViewportFrame", |s| {
        Box::new(ChromePanel::from_spec(s, "viewport", 0.0, true))
    });
}

/// 取 Scene 中所有文本图元的文案（断言用）。
pub fn scene_texts(scene: &Scene) -> Vec<String> {
    use zero_ui_render::render_node::RenderPrimitive;
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrowserChromeShell;
    use crate::chrome_model::BrowserChromeModel;
    use crate::shell::DesktopBrowserShell;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::render_node::RenderPrimitive;

    fn metrics(w: f32, h: f32) -> WindowMetrics {
        WindowMetrics {
            logical_size: Size::new(w, h),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
        }
    }

    #[test]
    fn desktop_shell_paints_into_unified_scene() {
        // DC-7：chrome 组件输出进统一 Scene（不绕过 ui/render）。
        let mut model = BrowserChromeModel::new();
        model.address_text = "https://example.com".into();
        model.security = crate::security_badge::SecurityState::Secure;
        model.tabs = vec![crate::BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "Example".into(),
            loading: false,
        }];
        model.active_tab_index = Some(0);

        let m = metrics(1280.0, 800.0);
        let spec = DesktopBrowserShell.build(&model, &m);

        let mut host = WidgetHost::new();
        register_chrome_factories(&mut host);
        host.set_root(&spec);
        let root_size = host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(
            m.logical_size.width,
            m.logical_size.height,
        )));
        // shell 根为 column 容器，铺满视口。
        assert!((root_size.width - 1280.0).abs() < 0.01);
        let scene = host.paint().clone();

        // 1. 地址栏 URL 文案进 Scene。
        let texts = scene_texts(&scene);
        assert!(
            texts.iter().any(|t| t.contains("example.com")),
            "address URL must render into scene, got {texts:?}"
        );
        // 2. 至少有 toolbar/tabstrip/bookmarks/address 的背景 fill + 视口 fill。
        let fills = scene
            .entries
            .iter()
            .filter(|e| matches!(e.primitive, RenderPrimitive::FillRect { .. }))
            .count();
        assert!(fills >= 4, "expected several chrome bar fills, got {fills}");
        // 3. 视口节点（fill_viewport）铺满剩余高度：rect 高接近 800-顶部bars。
        let vp = host.rect_of(&WidgetId::new("viewport")).expect("viewport laid out");
        assert!(
            vp.size.height > 600.0,
            "viewport should fill remaining height, got {}",
            vp.size.height
        );
    }

    #[test]
    fn security_state_drives_badge_color() {
        // 安全状态 → 语义色名 → Scene 中该节点 fill 的颜色。
        assert_eq!(
            security_color_name(crate::security_badge::SecurityState::Secure),
            "secure"
        );
        assert_eq!(
            security_color_name(crate::security_badge::SecurityState::Dangerous),
            "dangerous"
        );
        assert_eq!(chrome_color("dangerous"), Color::rgb(0.90, 0.20, 0.20));
        assert_eq!(
            chrome_color("unknown"),
            chrome_color("chrome"),
            "未知色名回落 chrome 灰"
        );
    }
}
