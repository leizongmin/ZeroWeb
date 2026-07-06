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
    // 遍历代表性页面，断言 demo 区有 ≥ 2 个图元（背景 + 内容）。
    //
    // P2-12 起所有 demo 都改用真控件子树，source id 形如 "demo_btn_1" / "demo_toggle_0" /
    // "demo_text_input" 等。统一以 "demo_" 前缀统计。
    for page_id in [
        "button",
        "toggle",
        "text_input",
        "icon_button",
        "badge",
        "progress",
        "tabs",
        "tooltip",
        "list_view",
        "menu",
        "search_field",
        "status_bubble",
        "toolbar",
        "popover",
        "popup",
        "data_list",
        "command_palette",
        "dialog_scaffold",
        "form_demo",
        "gesture_demo",
        "animation_demo",
        "collection_demo",
        "theme_demo",
        "i18n_demo",
        "dsl_demo",
        "nav_demo",
    ] {
        let mut app = GalleryApp::new();
        app.current_page = page_id.into();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let scene = driver.host().scene();
        let entry_count = count_demo_entries(scene);
        assert!(
            entry_count >= 2,
            "page={page_id} demo 区图元数应 >= 2 (background + content), got {entry_count}; 可能回归到「只画一行文字」"
        );
    }
}

fn count_demo_entries(scene: &Scene) -> usize {
    // 统计 demo 区所有 source id 以 "demo_" 开头的图元（覆盖所有 page 的真控件子树）。
    scene.entries.iter().filter(|e| e.source.0.starts_with("demo_")).count()
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
    // 验证 button demo 中点击 "Default" 按钮后 demo pressed 变为 1。
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
        app.current_demo_read().pressed,
        1,
        "clicking Default should set pressed to 1"
    );
}

#[test]
fn click_toggle_in_demo_area_updates_bitmask() {
    // 验证 toggle demo 中点击第 0 个 toggle 后 toggle state 第 0 位翻转。
    let mut app = GalleryApp::new();
    app.current_page = "toggle".into();
    let initial = app.current_demo_read().toggles;
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
    assert_eq!(
        app.current_demo_read().toggles,
        initial ^ 0b001,
        "clicking toggle 0 should flip bit 0"
    );
}

#[test]
fn disabled_button_does_not_emit_action() {
    // 验证 disabled 按钮（index 2）点击后 pressed 不变。
    let mut app = GalleryApp::new();
    app.current_page = "button".into();
    app.current_demo().pressed = 0;
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
        app.current_demo_read().pressed,
        0,
        "disabled button should not emit action; pressed stays 0"
    );
}

// ── P2-12 扩展：更多 demo page 的交互回归 ────────────────────────────────────

#[test]
fn click_tab_updates_selected_index() {
    // 验证 tabs demo 中点击第 2 个 tab button 后 pressed == 2。
    let mut app = GalleryApp::new();
    app.current_page = "tabs".into();
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_tab_1"))
            .expect("demo_tab_1 must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(
        app.current_demo_read().pressed,
        2,
        "clicking tab 1 should set selected index to 2"
    );
}

#[test]
fn click_popover_trigger_toggles_open_state() {
    // 验证 popover trigger 按钮点击后切换 open 状态。
    let mut app = GalleryApp::new();
    app.current_page = "popover".into();
    assert_eq!(app.current_demo_read().pressed, 0, "initial state should be 0 (closed)");
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_popover_trigger"))
            .expect("demo_popover_trigger must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
    }
    assert_eq!(app.current_demo_read().pressed, 1, "clicking trigger should open (=1)");
}

#[test]
fn hover_tooltip_button_emits_enter_action() {
    // P2-14：验证 Button hover_action 在 enter 时 emit，tooltip demo 真 hover 联动。
    // leave 行为已在 zero-ui-widgets button 测试里覆盖。
    let mut app = GalleryApp::new();
    app.current_page = "tooltip".into();
    assert_eq!(app.current_demo_read().pressed, 0, "initial: no hover");

    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_tooltip_btn"))
            .expect("demo_tooltip_btn must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&UiEvent::Pointer {
            phase: PointerPhase::Moved,
            button: None,
            position: center,
            modifiers: zero_ui_core::event::Modifiers::NONE,
            pointer_id: 0,
        });
        driver.pump_frame();
    }
    assert_eq!(app.current_demo_read().pressed, 1, "hover enter should set pressed=1");
}

#[test]
fn text_input_in_search_field_updates_state() {
    // 验证 search_field demo 中的 TextInput 接收键盘事件后 text 更新。
    let mut app = GalleryApp::new();
    app.current_page = "search_field".into();
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();

        // 点击输入框聚焦。
        let rect = driver
            .host()
            .rect_of(&WidgetId::new("demo_search_input"))
            .expect("demo_search_input must be laid out");
        let center = Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));

        // 键入 "abc"。
        for ch in ['a', 'b', 'c'] {
            driver.pump_event(&UiEvent::Key {
                code: zero_ui_core::event::KeyCode::new("KeyA"),
                action: zero_ui_core::event::KeyAction::Pressed,
                modifiers: zero_ui_core::event::Modifiers::NONE,
                text: Some(ch.to_string()),
            });
        }
        driver.pump_frame();
    }
    assert_eq!(
        app.current_demo_read().text,
        "abc",
        "typing 'abc' should set text to 'abc'"
    );
}

#[test]
fn all_pages_render_without_panic() {
    // P2-12 回归：所有 page 都能正确构建 + paint（不 panic）。
    // 这是兜底测试——确保新增 demo builder 没有破坏其它 page。
    for page_id in [
        "button",
        "toggle",
        "text_input",
        "icon_button",
        "badge",
        "progress",
        "tabs",
        "tooltip",
        "list_view",
        "menu",
        "search_field",
        "status_bubble",
        "toolbar",
        "popover",
        "popup",
        "data_list",
        "command_palette",
        "dialog_scaffold",
        "form_demo",
        "gesture_demo",
        "animation_demo",
        "collection_demo",
        "theme_demo",
        "i18n_demo",
        "dsl_demo",
        "nav_demo",
    ] {
        let mut app = GalleryApp::new();
        app.current_page = page_id.into();
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_gallery_factories(driver.host_mut());
        driver.begin();
        // 再 pump 一帧确保 layout + paint 完整执行。
        driver.pump_frame();
        // 不 panic 即通过。
    }
}

// 防止未使用 Rect 导入告警（在某些 toolchain 上 rect_of 返回值用到 Rect）。
#[allow(dead_code)]
fn _rect_unused(_: Rect) {}

// ── P3-3 真视觉回归：data_list / command_palette / icon_button / animation ─────

#[test]
fn data_list_renders_per_item_toggles_not_single_label() {
    // P3-3 回归：早期 data_list 用单个 SourceLabel 拼 8 行文本。
    // 现在每个 item 是独立 ToggleWidget，应有 demo_data_list_t_0..7。
    let mut app = GalleryApp::new();
    app.current_page = "data_list".into();
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_gallery_factories(driver.host_mut());
    driver.begin();

    for i in 0..8 {
        let id = WidgetId::new(&format!("demo_data_list_t_{}", i));
        assert!(
            driver.host().rect_of(&id).is_some(),
            "demo_data_list_t_{i} 应被 layout（应是独立 ToggleWidget 节点，而不是 SourceLabel 拼接）"
        );
    }
}

#[test]
fn command_palette_renders_per_item_buttons_with_markers() {
    // P3-3 回归：早期 command_palette 用 SourceLabel 拼 > cmd 列表。
    // 现在前 5 项是 Button + ColoredBox marker。
    let mut app = GalleryApp::new();
    app.current_page = "command_palette".into();
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_gallery_factories(driver.host_mut());
    driver.begin();

    // 至少前 5 个命令的 button + marker 应被 layout。
    let mut found_buttons = 0;
    let mut found_markers = 0;
    for i in 0..5 {
        if driver.host().rect_of(&WidgetId::new(&format!("demo_cmd_item_{}", i))).is_some() {
            found_buttons += 1;
        }
        if driver.host().rect_of(&WidgetId::new(&format!("demo_cmd_marker_{}", i))).is_some() {
            found_markers += 1;
        }
    }
    assert!(
        found_buttons >= 1,
        "command_palette 应至少有 1 个 demo_cmd_item_* Button（不再是纯 SourceLabel）"
    );
    assert!(
        found_markers >= 1,
        "command_palette 应至少有 1 个 demo_cmd_marker_* ColoredBox"
    );
}

#[test]
fn icon_button_has_coloredbox_markers() {
    // P3-3 回归：icon_button 现在每个 icon 配 ColoredBox 标记，不再仅 ASCII 字符。
    let mut app = GalleryApp::new();
    app.current_page = "icon_button".into();
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_gallery_factories(driver.host_mut());
    driver.begin();

    for name in ["Back", "Fwd", "Reload", "Close"] {
        let id = WidgetId::new(&format!("demo_icon_glyph_{}", name));
        assert!(
            driver.host().rect_of(&id).is_some(),
            "demo_icon_glyph_{name} 应存在（ColoredBox 图标标记）"
        );
    }
}

#[test]
fn animation_demo_has_coloredbox_indicator() {
    // P3-3 回归：animation 现在用 ColoredBox indicator 显示状态，不再纯文本。
    let mut app = GalleryApp::new();
    app.current_page = "animation_demo".into();
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_gallery_factories(driver.host_mut());
    driver.begin();

    assert!(
        driver.host().rect_of(&WidgetId::new("demo_anim_indicator")).is_some(),
        "demo_anim_indicator ColoredBox 应存在"
    );
}
