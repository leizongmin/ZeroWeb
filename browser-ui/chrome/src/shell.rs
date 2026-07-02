//! Adaptive shell — desktop/tablet/phone 浏览器 chrome 编排（spec §8.4.4A / IF-009 / FR-015 / DC-12）。
//!
//! 三 shell 共享 [`BrowserChromeModel`] + [`BrowserAction`](crate::BrowserAction)，按
//! `WindowMetrics` + `PlatformClass` + `InputClass` 选择；移动端不是 desktop 等比缩小，
//! 而是按断点 + safe area + 软键盘重排（§8.4.4A 表）。shell 只组装 UI（输出 [`WidgetSpec`]
//! 声明树 + [`ShellLayout`] 具体区域），不持有可变业务状态。

use crate::chrome_model::BrowserChromeModel;
use zero_ui_core::binding::Value;
use zero_ui_core::geometry::Rect;
use zero_ui_core::layout::{AdaptiveBranch, InputClass, PlatformClass, ViewportClass, WindowMetrics};
use zero_ui_core::widget::{WidgetId, WidgetSpec};

/// shell 种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Desktop,
    Tablet,
    Phone,
}

/// shell 布局区域（具体像素矩形，避开 safe area + 软键盘）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellLayout {
    /// 顶部 chrome 区域（toolbar/tab strip/bookmarks 或 phone 顶部地址栏）。
    pub top_chrome: Rect,
    /// WebView 视口（避开所有 chrome + safe area + keyboard）。
    pub viewport: Rect,
    /// 底部 chrome 区域（phone 底部导航栏；desktop/tablet 为 ZERO）。
    pub bottom_chrome: Rect,
    /// 是否因软键盘做了底部避让。
    pub keyboard_avoided: bool,
}

/// 浏览器 chrome shell 接口（spec IF-009 `BrowserChromeShell`；本 M2 取 `&model` 借用）。
pub trait BrowserChromeShell {
    fn kind(&self) -> ShellKind;
    /// 组装声明树（组件 + 稳定 WidgetId；跨断点切换时同 id 保留输入状态）。
    fn build(&self, model: &BrowserChromeModel, metrics: &WindowMetrics) -> WidgetSpec;
    /// 计算具体区域（safe area + keyboard 避让）。
    fn layout(&self, metrics: &WindowMetrics) -> ShellLayout;
}

// ── stable WidgetId 常量（跨 shell 一致；DC-12 responsive-branch 保留输入状态）──────
const ID_SHELL: &str = "shell";
const ID_TOOLBAR: &str = "toolbar";
const ID_ADDRESS_BAR: &str = "address_bar";
const ID_SECURITY_BADGE: &str = "security_badge";
const ID_NAV_BUTTONS: &str = "nav_buttons";
const ID_MENU: &str = "menu";
const ID_TAB_STRIP: &str = "tab_strip";
const ID_BOOKMARKS: &str = "bookmarks";
/// 页面内容视口节点 id（pub(crate)：sdk_render 据此取 SDK chrome 布局后的内容区矩形，
/// 供浏览器替换式迁移把页面内容定位到 SDK chrome 的视口（SDK chrome 拥有布局，浏览器适配）。
pub(crate) const ID_VIEWPORT: &str = "viewport";
const ID_BOTTOM_NAV: &str = "bottom_nav";
const ID_FIND_BAR: &str = "find_bar";

fn node(component: &str, id: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new(component);
    s.id = Some(WidgetId::new(id));
    s
}

/// 容器节点：声明 `props.layout`（host 据此布局子节点，无需硬编码业务组件名）。
fn node_layout(component: &str, id: &str, layout: &str) -> WidgetSpec {
    let mut s = node(component, id);
    s.props.insert("layout", Value::Text(layout.into()));
    s
}

/// 叶子节点：语义色 `bg` + 可选文案 `text`（chrome 绘制工厂据此 paint 进统一 Scene）。
fn leaf(component: &str, id: &str, bg: &str, text: Option<String>) -> WidgetSpec {
    let mut s = node(component, id);
    s.props.insert("bg", Value::Text(bg.into()));
    if let Some(t) = text {
        s.props.insert("text", Value::Text(t));
    }
    s
}

fn tab_titles(model: &BrowserChromeModel) -> String {
    model
        .tabs
        .iter()
        .map(|t| t.title.clone())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 收集声明树中所有稳定 WidgetId（测试用）。
pub fn collect_widget_ids(spec: &WidgetSpec, out: &mut Vec<String>) {
    if let Some(id) = &spec.id {
        out.push(id.0.to_string());
    }
    for c in &spec.children {
        collect_widget_ids(c, out);
    }
}

fn inner_rect(metrics: &WindowMetrics) -> Rect {
    Rect::from_ltrb(
        metrics.safe_area.left,
        metrics.safe_area.top,
        metrics.logical_size.width - metrics.safe_area.right,
        metrics.logical_size.height - metrics.safe_area.bottom,
    )
}

// ── DesktopBrowserShell ─────────────────────────────────────────────────────────
/// 桌面 shell：顶部 toolbar（nav + 地址栏 + 安全徽章 + 菜单）+ 完整 tab strip + 书签栏 + 视口。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopBrowserShell;

impl BrowserChromeShell for DesktopBrowserShell {
    fn kind(&self) -> ShellKind {
        ShellKind::Desktop
    }

    fn build(&self, model: &BrowserChromeModel, _metrics: &WindowMetrics) -> WidgetSpec {
        let mut root = node_layout("browser.DesktopBrowserShell", ID_SHELL, "column");
        let mut toolbar = node_layout("browser.ToolbarRow", ID_TOOLBAR, "row");
        toolbar.children.push(leaf(
            "browser.NavigationButtons",
            ID_NAV_BUTTONS,
            "chrome",
            Some(format!(
                "Back·{} Fwd·{}",
                model.navigation.can_go_back, model.navigation.can_go_forward
            )),
        ));
        toolbar.children.push(leaf(
            "browser.AddressBar",
            ID_ADDRESS_BAR,
            "chrome",
            Some(model.address_text.clone()),
        ));
        toolbar.children.push(leaf(
            "browser.SecurityBadge",
            ID_SECURITY_BADGE,
            crate::render::security_color_name(model.security),
            None,
        ));
        toolbar
            .children
            .push(leaf("browser.BrowserMenu", ID_MENU, "chrome", Some("Menu".into())));
        root.children.push(toolbar);
        root.children.push(leaf(
            "browser.BrowserTabStrip",
            ID_TAB_STRIP,
            "chrome",
            Some(tab_titles(model)),
        ));
        root.children.push(leaf(
            "browser.BookmarksBar",
            ID_BOOKMARKS,
            "chrome",
            Some(format!("{} bookmarks", model.bookmarks.len())),
        ));
        let mut viewport = node("browser.PageViewportFrame", ID_VIEWPORT);
        viewport.props.insert("bg", Value::Text("viewport".into()));
        root.children.push(viewport);
        if model.find.is_some() {
            root.children
                .push(leaf("browser.FindBar", ID_FIND_BAR, "chrome", Some("Find".into())));
        }
        root
    }

    fn layout(&self, metrics: &WindowMetrics) -> ShellLayout {
        let inner = inner_rect(metrics);
        // toolbar(40) + tab strip(36) + bookmarks(28) ≈ 104。
        let top_h = 104.0_f32.min((inner.bottom() - inner.top()) * 0.5);
        let top_chrome = Rect::from_ltrb(inner.left(), inner.top(), inner.right(), inner.top() + top_h);
        let viewport = Rect::from_ltrb(inner.left(), top_chrome.bottom(), inner.right(), inner.bottom());
        ShellLayout {
            top_chrome,
            viewport,
            bottom_chrome: Rect::ZERO,
            keyboard_avoided: false,
        }
    }
}

// ── TabletBrowserShell ──────────────────────────────────────────────────────────
/// 平板 shell：顶部 toolbar（nav + 地址栏 + 安全徽章）+ 可滚动 tab strip + 视口（无书签栏/底部栏）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabletBrowserShell;

impl BrowserChromeShell for TabletBrowserShell {
    fn kind(&self) -> ShellKind {
        ShellKind::Tablet
    }

    fn build(&self, model: &BrowserChromeModel, _metrics: &WindowMetrics) -> WidgetSpec {
        let mut root = node_layout("browser.TabletBrowserShell", ID_SHELL, "column");
        let mut toolbar = node_layout("browser.ToolbarRow", ID_TOOLBAR, "row");
        toolbar.children.push(leaf(
            "browser.NavigationButtons",
            ID_NAV_BUTTONS,
            "chrome",
            Some(format!(
                "Back·{} Fwd·{}",
                model.navigation.can_go_back, model.navigation.can_go_forward
            )),
        ));
        toolbar.children.push(leaf(
            "browser.AddressBar",
            ID_ADDRESS_BAR,
            "chrome",
            Some(model.address_text.clone()),
        ));
        toolbar.children.push(leaf(
            "browser.SecurityBadge",
            ID_SECURITY_BADGE,
            crate::render::security_color_name(model.security),
            None,
        ));
        root.children.push(toolbar);
        root.children.push(leaf(
            "browser.BrowserTabStrip",
            ID_TAB_STRIP,
            "chrome",
            Some(tab_titles(model)),
        ));
        let mut viewport = node("browser.PageViewportFrame", ID_VIEWPORT);
        viewport.props.insert("bg", Value::Text("viewport".into()));
        root.children.push(viewport);
        if model.find.is_some() {
            root.children
                .push(leaf("browser.FindBar", ID_FIND_BAR, "chrome", Some("Find".into())));
        }
        root
    }

    fn layout(&self, metrics: &WindowMetrics) -> ShellLayout {
        let inner = inner_rect(metrics);
        // toolbar(40) + tab strip(36) = 76。
        let top_h = 76.0_f32.min((inner.bottom() - inner.top()) * 0.5);
        let top_chrome = Rect::from_ltrb(inner.left(), inner.top(), inner.right(), inner.top() + top_h);
        let viewport = Rect::from_ltrb(inner.left(), top_chrome.bottom(), inner.right(), inner.bottom());
        ShellLayout {
            top_chrome,
            viewport,
            bottom_chrome: Rect::ZERO,
            keyboard_avoided: false,
        }
    }
}

// ── PhoneBrowserShell ───────────────────────────────────────────────────────────
/// 手机 shell：顶部地址栏 + 全屏视口 + 底部导航栏；视口避开 safe area 与软键盘（§8.4.4A）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneBrowserShell;

impl BrowserChromeShell for PhoneBrowserShell {
    fn kind(&self) -> ShellKind {
        ShellKind::Phone
    }

    fn build(&self, model: &BrowserChromeModel, _metrics: &WindowMetrics) -> WidgetSpec {
        let mut root = node_layout("browser.PhoneBrowserShell", ID_SHELL, "column");
        // 顶部地址栏（row 容器：自身 chrome 背景 + URL 文案，内含安全徽章）。
        let mut top = node_layout("browser.AddressBar", ID_ADDRESS_BAR, "row");
        top.props.insert("bg", Value::Text("chrome".into()));
        top.props.insert("text", Value::Text(model.address_text.clone()));
        top.children.push(leaf(
            "browser.SecurityBadge",
            ID_SECURITY_BADGE,
            crate::render::security_color_name(model.security),
            None,
        ));
        root.children.push(top);
        let mut viewport = node("browser.PageViewportFrame", ID_VIEWPORT);
        viewport.props.insert("bg", Value::Text("viewport".into()));
        root.children.push(viewport);
        root.children.push(leaf(
            "browser.NavigationButtons",
            ID_BOTTOM_NAV,
            "chrome",
            Some(format!(
                "Back·{} Fwd·{}",
                model.navigation.can_go_back, model.navigation.can_go_forward
            )),
        ));
        if model.find.is_some() {
            root.children
                .push(leaf("browser.FindBar", ID_FIND_BAR, "chrome", Some("Find".into())));
        }
        root
    }

    fn layout(&self, metrics: &WindowMetrics) -> ShellLayout {
        let inner = inner_rect(metrics);
        let top_h = 48.0_f32.min((inner.bottom() - inner.top()) * 0.4);
        let top_chrome = Rect::from_ltrb(inner.left(), inner.top(), inner.right(), inner.top() + top_h);

        // 底部导航栏避开 safe area + 软键盘。
        let keyboard = metrics.keyboard_insets.bottom;
        let avail_bottom = inner.bottom() - keyboard;
        let bottom_h = 56.0_f32.min((avail_bottom - top_chrome.bottom()) * 0.4);
        let bottom_chrome = Rect::from_ltrb(inner.left(), avail_bottom - bottom_h, inner.right(), avail_bottom);
        let viewport = Rect::from_ltrb(inner.left(), top_chrome.bottom(), inner.right(), bottom_chrome.top());
        ShellLayout {
            top_chrome,
            viewport,
            bottom_chrome,
            keyboard_avoided: keyboard > 0.0,
        }
    }
}

/// 按 adaptive 分支选 shell 种类（spec §8.4.4A）：
/// Expanded + Pointer → Desktop；Compact → Phone；Medium + Touch/Mobile → Tablet。
pub fn select_shell(branch: AdaptiveBranch) -> ShellKind {
    match (branch.viewport, branch.platform, branch.input) {
        (ViewportClass::Compact, _, _) => ShellKind::Phone,
        (ViewportClass::Expanded, _, _) => ShellKind::Desktop,
        (ViewportClass::Medium, PlatformClass::Mobile, _) => ShellKind::Tablet,
        (ViewportClass::Medium, _, InputClass::Touch) => ShellKind::Tablet,
        (ViewportClass::Medium, _, _) => ShellKind::Desktop,
    }
}

/// Adaptive 浏览器 chrome：持有三 shell，按 metrics + platform + input 选择并组装。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdaptiveBrowserChrome {
    pub desktop: DesktopBrowserShell,
    pub tablet: TabletBrowserShell,
    pub phone: PhoneBrowserShell,
}

/// adaptive 组装结果（shell 种类 + 声明树 + 区域）。
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveChromeResult {
    pub kind: ShellKind,
    pub spec: WidgetSpec,
    pub layout: ShellLayout,
}

impl AdaptiveBrowserChrome {
    pub fn new() -> AdaptiveBrowserChrome {
        AdaptiveBrowserChrome::default()
    }

    /// 按 metrics + platform + input 组装当前 shell。
    pub fn build(
        &self,
        model: &BrowserChromeModel,
        metrics: &WindowMetrics,
        platform: PlatformClass,
        input: InputClass,
    ) -> AdaptiveChromeResult {
        let branch = AdaptiveBranch::from_metrics(metrics, platform, input);
        let kind = select_shell(branch);
        let (spec, layout) = match kind {
            ShellKind::Desktop => (self.desktop.build(model, metrics), self.desktop.layout(metrics)),
            ShellKind::Tablet => (self.tablet.build(model, metrics), self.tablet.layout(metrics)),
            ShellKind::Phone => (self.phone.build(model, metrics), self.phone.layout(metrics)),
        };
        AdaptiveChromeResult { kind, spec, layout }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::find_bar::FindBar;
    use zero_ui_core::geometry::{Insets, Size};

    fn metrics(width: f32, height: f32, safe_bottom: f32, kb_bottom: f32) -> WindowMetrics {
        WindowMetrics {
            logical_size: Size::new(width, height),
            scale_factor: 1.0,
            safe_area: Insets {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: safe_bottom,
            },
            keyboard_insets: Insets {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: kb_bottom,
            },
        }
    }

    #[test]
    fn select_shell_matches_spec_table() {
        // Expanded + Pointer → Desktop。
        let desk = AdaptiveBranch {
            viewport: ViewportClass::Expanded,
            platform: PlatformClass::Desktop,
            input: InputClass::Pointer,
        };
        assert_eq!(select_shell(desk), ShellKind::Desktop);
        // Compact → Phone（不管 platform/input）。
        let phone = AdaptiveBranch {
            viewport: ViewportClass::Compact,
            platform: PlatformClass::Mobile,
            input: InputClass::Touch,
        };
        assert_eq!(select_shell(phone), ShellKind::Phone);
        // Medium + Touch → Tablet。
        let tab = AdaptiveBranch {
            viewport: ViewportClass::Medium,
            platform: PlatformClass::Desktop,
            input: InputClass::Touch,
        };
        assert_eq!(select_shell(tab), ShellKind::Tablet);
        // Medium + Pointer → Desktop（fallback）。
        let tab_desk = AdaptiveBranch {
            viewport: ViewportClass::Medium,
            platform: PlatformClass::Desktop,
            input: InputClass::Pointer,
        };
        assert_eq!(select_shell(tab_desk), ShellKind::Desktop);
    }

    #[test]
    fn phone_layout_avoids_keyboard_and_safe_area() {
        // 800 高窗口，底部 safe area 20，软键盘 300。
        let m = metrics(390.0, 800.0, 20.0, 300.0);
        let lay = PhoneBrowserShell.layout(&m);
        assert!(lay.keyboard_avoided, "应识别软键盘避让");
        // 底部导航栏在键盘上方（avail_bottom = 800-20-300=480）。
        assert!(
            (lay.bottom_chrome.bottom() - 480.0).abs() < 0.01,
            "bottom_chrome 底 = 键盘上方"
        );
        // 视口底部 ≤ bottom_chrome 顶部，不被底部栏/键盘遮挡。
        assert!(lay.viewport.bottom() <= lay.bottom_chrome.top() + 0.01);
        // 视口顶部 = top_chrome 底部（顶部地址栏 48px）。
        assert!((lay.viewport.top() - 48.0).abs() < 0.01);
    }

    #[test]
    fn phone_layout_no_keyboard_not_avoided() {
        let m = metrics(390.0, 800.0, 0.0, 0.0);
        let lay = PhoneBrowserShell.layout(&m);
        assert!(!lay.keyboard_avoided);
        assert_eq!(lay.bottom_chrome.bottom(), 800.0);
    }

    #[test]
    fn desktop_layout_top_chrome_then_viewport() {
        let m = metrics(1280.0, 800.0, 0.0, 0.0);
        let lay = DesktopBrowserShell.layout(&m);
        assert_eq!(lay.bottom_chrome, Rect::ZERO, "desktop 无底部栏");
        assert!((lay.top_chrome.bottom() - 104.0).abs() < 0.01);
        assert!((lay.viewport.top() - 104.0).abs() < 0.01);
        assert_eq!(lay.viewport.bottom(), 800.0);
    }

    #[test]
    fn stable_widget_ids_shared_across_shells() {
        // DC-12：跨断点切换时同 WidgetId 保留输入状态（address_bar / viewport 在三 shell 都存在）。
        let model = BrowserChromeModel::new();
        let m = metrics(390.0, 800.0, 0.0, 0.0);
        for shell in [
            &DesktopBrowserShell as &dyn BrowserChromeShell,
            &TabletBrowserShell,
            &PhoneBrowserShell,
        ] {
            let spec = shell.build(&model, &m);
            let mut ids = Vec::new();
            collect_widget_ids(&spec, &mut ids);
            assert!(
                ids.iter().any(|s| s == ID_ADDRESS_BAR),
                "{:?} 缺 address_bar",
                shell.kind()
            );
            assert!(ids.iter().any(|s| s == ID_VIEWPORT), "{:?} 缺 viewport", shell.kind());
        }
    }

    #[test]
    fn find_bar_appears_only_when_active() {
        let mut model = BrowserChromeModel::new();
        let m = metrics(1280.0, 800.0, 0.0, 0.0);
        let desktop = DesktopBrowserShell;
        let mut ids = Vec::new();
        collect_widget_ids(&desktop.build(&model, &m), &mut ids);
        assert!(!ids.iter().any(|s| s == ID_FIND_BAR), "find 未打开不应出现");
        model.find = Some(FindBar::open());
        let mut ids2 = Vec::new();
        collect_widget_ids(&desktop.build(&model, &m), &mut ids2);
        assert!(ids2.iter().any(|s| s == ID_FIND_BAR), "find 打开应出现");
    }

    #[test]
    fn adaptive_build_dispatches_correct_shell() {
        let chrome = AdaptiveBrowserChrome::new();
        let model = BrowserChromeModel::new();
        // 宽屏 desktop。
        let r_desk = chrome.build(
            &model,
            &metrics(1280.0, 800.0, 0.0, 0.0),
            PlatformClass::Desktop,
            InputClass::Pointer,
        );
        assert_eq!(r_desk.kind, ShellKind::Desktop);
        // 窄屏 phone + 键盘。
        let r_phone = chrome.build(
            &model,
            &metrics(390.0, 800.0, 20.0, 300.0),
            PlatformClass::Mobile,
            InputClass::Touch,
        );
        assert_eq!(r_phone.kind, ShellKind::Phone);
        assert!(r_phone.layout.keyboard_avoided);
        // 中屏 tablet（touch）。
        let r_tab = chrome.build(
            &model,
            &metrics(720.0, 1024.0, 0.0, 0.0),
            PlatformClass::Desktop,
            InputClass::Touch,
        );
        assert_eq!(r_tab.kind, ShellKind::Tablet);
    }
}
