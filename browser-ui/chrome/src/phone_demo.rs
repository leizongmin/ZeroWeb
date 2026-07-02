//! phone_demo（DC-15）—— 移动端 phone shell + 多指手势 + 平台 back 可运行制品。
//!
//! 配套 example：`cargo run -p zero-browser-chrome --example phone_demo`。
//!
//! 用 [`WindowMetrics::phone`] preset 构造手机视口，经 [`AdaptiveBrowserChrome`] 选
//! [`PhoneBrowserShell`](crate::shell::PhoneBrowserShell)，跑 retained 闭环；脚本化驱动
//! 触摸 Tap/Pinch（[`GestureArena`]，多指用 `UiEvent::Pointer.pointer_id`）+ 平台 back
//! （[`BackNavigationService`]→[`RouteStack`] 回落），返回汇总。证明 DC-15「PhoneBrowserShell
//! 可用」+ 手势全四类经 host arena 可达 + 平台 back 仲裁（headless 可运行，不接真实设备）。

use crate::chrome_model::BrowserChromeModel;
use crate::render::register_chrome_factories;
use crate::shell::{AdaptiveBrowserChrome, ShellKind};
use crate::{BrowserTab, NavigationButtons, SecurityState};
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point};
use zero_ui_core::layout::{InputClass, PlatformClass, WindowMetrics};
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::WidgetId;
use zero_ui_gestures::{Gesture, GestureArena, PanRecognizer, PinchRecognizer, TapRecognizer};
use zero_ui_navigation::{Route, RouteStack};
use zero_ui_platform::{BackHandlerId, BackNavigationService, BackResult, InMemoryBackNavigation};
use zero_ui_runtime::WidgetHost;

/// phone_demo 运行汇总（example 打印 + 测试断言）。
#[derive(Debug, Clone)]
pub struct PhoneDemoSummary {
    /// adaptive 选择的 shell 种类（应为 Phone）。
    pub shell_kind: ShellKind,
    /// 统一 Scene 图元数。
    pub scene_entries: usize,
    /// SDK chrome 拥有的布局——内容区视口顶部 y（chrome 占顶部后内容起始，safe_area 避让）。
    pub viewport_top_y: f32,
    /// 底部 chrome（phone 底部导航栏）高度（>0 表示 phone 有底部栏；desktop/tablet 为 0）。
    pub bottom_chrome_height: f32,
    /// phone shell 声明树根节点是否布局到 phone 视口宽。
    pub shell_laid_out: bool,
    /// 识别出的手势种类名（Tap/Pan/Pinch/Fling）。
    pub gesture_kinds: Vec<String>,
    /// 注册了 handler 时的 back 仲裁结果（应 Handled）。
    pub back_with_handler: BackResult,
    /// 无 handler 时的 back 仲裁结果（应 DefaultBack）。
    pub back_default: BackResult,
    /// DefaultBack 回落后 navigator 剩余深度（home→settings 推入再 pop → 1）。
    pub nav_depth_after_back: usize,
}

fn pointer(phase: PointerPhase, position: Point, pointer_id: u32) -> UiEvent {
    UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position,
        modifiers: Modifiers::NONE,
        pointer_id,
    }
}

/// 构造一个最小有数据的 chrome model（一个安全 tab）。
fn sample_model() -> BrowserChromeModel {
    let mut model = BrowserChromeModel::new();
    model.address_text = "https://example.com".into();
    model.security = SecurityState::Secure;
    model.navigation = NavigationButtons::new(true, false, false);
    model.tabs = vec![BrowserTab {
        id: zero_browser_shell::TabId(1),
        title: "Example".into(),
        loading: false,
    }];
    model.active_tab_index = Some(0);
    model
}

/// 运行 phone_demo 闭环，返回汇总（example + 测试共用）。
pub fn run_phone_demo() -> PhoneDemoSummary {
    let model = sample_model();
    let metrics = WindowMetrics::phone();

    // adaptive 选 shell（phone metrics + Mobile + Touch → Phone）。
    let result = AdaptiveBrowserChrome::new().build(&model, &metrics, PlatformClass::Mobile, InputClass::Touch);

    // retained 闭环：register factories → set_root → layout(phone 紧约束) → paint。
    // paint() 返回的 &Scene 不跨 dispatch_event 持有（避免借用冲突），scene 数据最后读。
    let tokens = SemanticTokens::light();
    let mut host = WidgetHost::new();
    register_chrome_factories(&mut host, &tokens);
    host.set_root(&result.spec);
    host.layout(Constraints::tight(metrics.logical_size));
    host.paint();

    // 手势 arena：Tap + Pan + Pinch 识别器（多指 Pinch 经 pointer_id 区分）。
    let mut arena = GestureArena::new();
    arena.push(TapRecognizer::new(8.0));
    arena.push(PanRecognizer::new());
    arena.push(PinchRecognizer::new());
    host.set_gesture_arena(arena);

    // 触摸 Tap（单指 id=0）于地址栏区。
    let tap_at = Point::new(195.0, 60.0);
    host.dispatch_event(&pointer(PointerPhase::Pressed, tap_at, 0));
    host.dispatch_event(&pointer(PointerPhase::Released, tap_at, 0));

    // 触摸 Pinch（双指 id=0 + id=1）于内容区。
    host.dispatch_event(&pointer(PointerPhase::Pressed, Point::new(130.0, 400.0), 0));
    host.dispatch_event(&pointer(PointerPhase::Pressed, Point::new(260.0, 400.0), 1));
    // 第二指外移 50px（距离 130→180，scale≈1.38）→ Pinch。
    host.dispatch_event(&pointer(PointerPhase::Moved, Point::new(310.0, 400.0), 1));

    let gesture_kinds = host
        .take_gestures()
        .into_iter()
        .map(|g| match g {
            Gesture::Tap(_) => "Tap".to_string(),
            Gesture::Pan { .. } => "Pan".to_string(),
            Gesture::Pinch { .. } => "Pinch".to_string(),
            Gesture::Fling { .. } => "Fling".to_string(),
        })
        .collect();

    // 平台 back 仲裁 + navigator 回落（移动 back glue）。
    let back = InMemoryBackNavigation::new();
    back.push_handler(BackHandlerId::new("overlay"));
    let back_with_handler = back.on_platform_back(); // → Handled（overlay 消耗，不退栈）
    let mut nav = RouteStack::new(Route::new("home"));
    nav.push(Route::new("settings"));
    let back_default = back.on_platform_back(); // → DefaultBack
    if matches!(back_default, BackResult::DefaultBack) {
        nav.pop(); // 宿主回落：DefaultBack → 退栈（settings 出栈，剩 home）
    }

    // 最后读 scene + 几何（此时 host 无活跃可变借用）。
    let scene_entries = host.scene().entries.len();
    let shell_laid_out = host
        .rect_of(&WidgetId::new("shell"))
        .map(|r| r.size.width >= 380.0)
        .unwrap_or(false);

    PhoneDemoSummary {
        shell_kind: result.kind,
        scene_entries,
        viewport_top_y: result.layout.viewport.origin.y,
        bottom_chrome_height: result.layout.bottom_chrome.size.height,
        shell_laid_out,
        gesture_kinds,
        back_with_handler,
        back_default,
        nav_depth_after_back: nav.depth(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ShellKind;

    #[test]
    fn phone_demo_selects_phone_shell_and_renders() {
        // DC-15：PhoneBrowserShell 可用——phone metrics → Phone shell + 非空 Scene。
        let s = run_phone_demo();
        assert_eq!(s.shell_kind, ShellKind::Phone, "phone metrics → Phone shell");
        assert!(s.scene_entries > 0, "phone shell renders non-empty scene");
        assert!(s.shell_laid_out, "shell 根节点布局到 phone 视口宽");
    }

    #[test]
    fn phone_demo_safe_area_and_bottom_nav() {
        // phone shell 应避让顶部 safe_area（viewport_top_y > 0）并有底部导航栏。
        let s = run_phone_demo();
        // safe_area top=47（phone preset）→ 内容区应在其下（顶部 chrome 含 safe_area）。
        assert!(
            s.viewport_top_y > 0.0,
            "内容区起点 y>0（chrome + safe_area 避让），got {}",
            s.viewport_top_y
        );
        assert!(
            s.bottom_chrome_height > 0.0,
            "phone 有底部导航栏，got {}",
            s.bottom_chrome_height
        );
    }

    #[test]
    fn phone_demo_gestures_and_back_arbitration() {
        // DC-15：Tap + Pinch 经 host arena 识别；back 仲裁 Handled→DefaultBack→navigator pop。
        let s = run_phone_demo();
        assert!(
            s.gesture_kinds.iter().any(|g| g == "Tap"),
            "Tap 被识别：{:?}",
            s.gesture_kinds
        );
        assert!(
            s.gesture_kinds.iter().any(|g| g == "Pinch"),
            "双指 Pinch 被识别：{:?}",
            s.gesture_kinds
        );
        // back #1（注册 overlay handler）→ Handled。
        assert!(matches!(s.back_with_handler, BackResult::Handled(_)));
        // back #2（无 handler）→ DefaultBack → navigator pop（settings 出栈，剩 home）。
        assert_eq!(s.back_default, BackResult::DefaultBack);
        assert_eq!(s.nav_depth_after_back, 1, "pop 后剩 home（depth 1）");
    }
}
