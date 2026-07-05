//! Gallery 视觉回归测试（P3-2）—— 滚动 + DemoPreview 不回归。
//!
//! 验证两个之前出过 bug 的关键路径：
//! 1. **侧栏滚动**：sidebar 内容超出视口时滚动 offset 正确偏移子节点 + clamp（DC-16）。
//! 2. **DemoPreview**：切换页面后预览区有内容（不回归到「只画一行文字」的早期 bug）。
//!
//! 经 [`WinitDriver`] 驱动完整 retained 闭环，模拟真实运行环境。

use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, ScrollPhase, UiEvent};
use zero_ui_core::geometry::{Point, Rect, Vec2};
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{GalleryApp, register_gallery_factories};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;

fn pointer(phase: PointerPhase, position: Point) -> UiEvent {
    UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position,
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    }
}

fn scroll(delta_y: f32, position: Point) -> UiEvent {
    UiEvent::Scroll {
        delta: Vec2::new(0.0, delta_y),
        phase: ScrollPhase::Discrete,
        position,
        modifiers: Modifiers::NONE,
    }
}

/// 桌面 metrics 但窗口高度故意压缩到 300，让 sidebar 内容（28 个 nav 项 + 5 个 group header
/// + search ≈ 远超 300）必然溢出，触发滚动。
fn small_desktop() -> WindowMetrics {
    let mut m = WindowMetrics::desktop();
    m.logical_size.height = 300.0;
    m
}

#[test]
fn sidebar_scroll_offset_shifts_children_and_clamps() {
    // P3-2 回归测试：sidebar 在小窗口下内容必然溢出，Wheel 事件应推进 scroll_offset，
    // 子节点 y 上移；越界滚到末尾应 clamp（不超 content 高度）。
    let mut app = GalleryApp::new();
    let mut driver = WinitDriver::new(&mut app, small_desktop());
    register_gallery_factories(driver.host_mut());
    driver.begin();

    let sidebar_rect = driver
        .host()
        .rect_of(&WidgetId::new("sidebar"))
        .expect("sidebar node must exist");
    let viewport_h = sidebar_rect.size.height;

    // 取一个靠下的 nav 项作为参考，记录初始 y。
    // dsl_demo / nav_demo / theme_demo 是后段 page id，必然在 sidebar 下方。
    let pick_low_nav = |host: &zero_ui_runtime::WidgetHost| -> Option<Rect> {
        ["nav_dsl_demo", "nav_nav_demo", "nav_theme_demo", "nav_i18n_demo"]
            .iter()
            .find_map(|id| host.rect_of(&WidgetId::new(id)))
    };
    let nav_last = pick_low_nav(driver.host()).expect("至少有一个靠下的 nav 节点存在");
    let y_before = nav_last.origin.y;

    // 在 sidebar 内发 Wheel down。
    let inside = Point::new(sidebar_rect.origin.x + 10.0, sidebar_rect.origin.y + 10.0);
    driver.pump_event(&scroll(80.0, inside));
    driver.pump_frame();

    let nav_after = pick_low_nav(driver.host()).expect("nav 节点仍存在（不应因滚动消失）");
    assert!(
        nav_after.origin.y < y_before,
        "向下滚 80 后 nav 节点 y 应上移：before={}, after={}",
        y_before,
        nav_after.origin.y
    );

    // 越界滚：delta 远大于 content，应 clamp。
    driver.pump_event(&scroll(100_000.0, inside));
    driver.pump_frame();
    let nav_clamped = pick_low_nav(driver.host()).expect("nav 节点仍存在");
    // clamp 后 nav 节点应位于 viewport 内（origin.y + size.height ≈ viewport 底或更小）。
    let bottom = nav_clamped.origin.y + nav_clamped.size.height;
    assert!(
        bottom <= viewport_h + 1.0,
        "clamp 后 nav 底部应不超 viewport 底 {viewport_h}：实际 bottom={bottom}"
    );
}

#[test]
fn demo_preview_renders_non_trivial_content_for_each_page() {
    // P3-2 回归测试：早期 bug 是 DemoPreview 只画一行文字，没画实际组件内容。
    // 这里遍历几个代表性页面，断言切换后预览区 scene 图元数 > 1（至少有背景 + 内容）。
    //
    // P2-11 真控件化后：button/toggle/text_input 改用真控件子树（各自 source id 前缀），
    // 其余页面仍走旧 DemoPreview painter 架构（统一 "demo_preview" 前缀）。
    for (page_id, source_prefix) in [
        ("button", "demo_btn_"),
        ("toggle", "demo_toggle_"),
        ("text_input", "demo_text_"),
        ("badge", "demo_preview"),
        ("tabs", "demo_preview"),
        ("i18n_demo", "demo_preview"),
        ("theme_demo", "demo_preview"),
    ] {
        let mut app = GalleryApp::new();
        app.current_page = page_id.into();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let scene = driver.host().scene();
        let entry_count = count_demo_preview_entries(scene, source_prefix);
        assert!(
            entry_count >= 2,
            "page={page_id} 预览区图元数应 ≥ 2（背景 + 内容），实际 {entry_count}；可能回归到「只画一行文字」"
        );
    }
}

fn count_demo_preview_entries(scene: &Scene, source_prefix: &str) -> usize {
    // 按传入 source 前缀统计该 demo 的图元。
    scene
        .entries
        .iter()
        .filter(|e| e.source.0.starts_with(source_prefix))
        .count()
}

#[test]
fn click_nav_item_switches_page_via_driver() {
    // P3-2 回归测试：早期 bug 是 nav item 不可点击 / 不切换 page。
    // 验证：点击 nav_toggle 后 app.current_page 切到 "toggle"。
    let mut app = GalleryApp::new();
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        let nav_rect = driver
            .host()
            .rect_of(&WidgetId::new("nav_toggle"))
            .expect("nav_toggle must be laid out");
        let center = Point::new(
            nav_rect.origin.x + nav_rect.size.width / 2.0,
            nav_rect.origin.y + nav_rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(app.current_page, "toggle", "点击 nav_toggle 后应切换到 toggle 页");
}

#[test]
fn theme_toggle_changes_token_colors_in_scene() {
    // P3-2 回归测试：早期 bug 是 theme toggle 后文字消失 / 颜色不切换。
    // 验证：Light vs Dark 场景的 fill 颜色集合不同。
    let mut app_light = GalleryApp::new();
    let scene_light = {
        let mut driver = WinitDriver::new(&mut app_light, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        driver.host().scene().clone()
    };

    let mut app_dark = GalleryApp::new();
    app_dark.theme = zero_ui_examples::gallery::model::ThemeKind::Dark;
    let scene_dark = {
        let mut driver = WinitDriver::new(&mut app_dark, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        driver.host().scene().clone()
    };

    let light_colors: std::collections::HashSet<_> = fill_colors(&scene_light);
    let dark_colors: std::collections::HashSet<_> = fill_colors(&scene_dark);
    assert!(
        light_colors != dark_colors,
        "Light / Dark 主题 scene fill 颜色集合应不同（否则 theme 切换未生效）"
    );
}

fn fill_colors(scene: &Scene) -> std::collections::HashSet<[u8; 4]> {
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::FillRect { color, .. } => Some([
                (color.r.clamp(0.0, 1.0) * 255.0) as u8,
                (color.g.clamp(0.0, 1.0) * 255.0) as u8,
                (color.b.clamp(0.0, 1.0) * 255.0) as u8,
                (color.a.clamp(0.0, 1.0) * 255.0) as u8,
            ]),
            _ => None,
        })
        .collect()
}

// ── P2-11 真控件交互回归 ──────────────────────────────────────────────────

#[test]
fn click_button_in_demo_area_updates_state() {
    // 验证 button demo 中点击 "Default" 按钮后 demo_button_pressed 变为 1。
    let mut app = GalleryApp::new();
    app.current_page = "button".into();
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_btn_1"))
            .expect("demo_btn_1 must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(
        app.demo_button_pressed, 1,
        "点击 Default 按钮后 demo_button_pressed 应为 1"
    );
}

#[test]
fn click_toggle_in_demo_area_updates_bitmask() {
    // 验证 toggle demo 中点击第 0 个 toggle 后 demo_toggle_state 第 0 位翻转。
    let mut app = GalleryApp::new();
    app.current_page = "toggle".into();
    let initial = app.demo_toggle_state;
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_toggle_0"))
            .expect("demo_toggle_0 must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(app.demo_toggle_state, initial ^ 0b001, "点击 toggle 0 后第 0 位应翻转");
}

#[test]
fn disabled_button_does_not_emit_action() {
    // 验证 disabled 按钮（index 2）点击后 demo_button_pressed 不变。
    let mut app = GalleryApp::new();
    app.current_page = "button".into();
    app.demo_button_pressed = 0;
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_btn_3"))
            .expect("demo_btn_3 must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(
        app.demo_button_pressed, 0,
        "Disabled 按钮不应触发 action；demo_button_pressed 应保持 0"
    );
}

// 防止未使用 Rect 导入告警（在某些 toolchain 上 rect_of 返回值用到 Rect）。
#[allow(dead_code)]
fn _rect_unused(_: Rect) {}
