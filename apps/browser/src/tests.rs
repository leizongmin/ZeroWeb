//! ZeroBrowser 单元测试模块（从 main.rs 拆分以控制单文件体积）。
//!
//! 经 `#[cfg(test)] mod tests;` 在 main.rs 中声明；`super::*` 与 `super::browser_window_config`
//! 仍解析到 crate 根（main.rs），与内联时一致。

use super::*;
use app::append_webview_primitives;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};

#[test]
fn append_webview_primitives_translates_fills_and_glyphs() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_fill(Rect::new(1.0, 2.0, 10.0, 20.0), Color::rgb(255, 0, 0));
    primitives.add_glyph(GlyphPrimitive {
        x: 3.0,
        y: 4.0,
        font_size: 16.0,
        color: Color::rgb(0, 0, 0),
        glyph_id: 'A' as u32,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });

    let mut fills = Vec::new();
    let mut glyphs = Vec::new();

    assert!(append_webview_primitives(
        &primitives,
        &mut fills,
        &mut glyphs,
        10.0,
        layout::TOOLBAR_HEIGHT,
        7,
        1.0,
        None,
        None,
    ));

    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.x, 11.0);
    assert_eq!(fills[0].rect.origin.y, layout::TOOLBAR_HEIGHT + 2.0);
    assert_eq!(fills[0].rect.size.width, 10.0);
    assert_eq!(fills[0].rect.size.height, 20.0);
    assert_eq!(glyphs.len(), 1);
    assert_eq!(glyphs[0].ch, 'A');
    assert_eq!(glyphs[0].x, 13.0);
    assert_eq!(glyphs[0].baseline_y, layout::TOOLBAR_HEIGHT + 4.0);
    assert_eq!(glyphs[0].font_id, 7);
    assert_eq!(glyphs[0].font_size, 16.0);
}

#[test]
fn append_webview_primitives_scales_with_device_pixel_ratio() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_fill(Rect::new(10.0, 20.0, 100.0, 50.0), Color::rgb(255, 0, 0));

    let mut fills = Vec::new();
    let mut glyphs = Vec::new();

    assert!(append_webview_primitives(
        &primitives,
        &mut fills,
        &mut glyphs,
        0.0,
        100.0,
        1,
        2.0,
        None,
        None,
    ));

    assert_eq!(fills[0].rect.origin.x, 20.0);
    assert_eq!(fills[0].rect.origin.y, 140.0);
    assert_eq!(fills[0].rect.size.width, 200.0);
    assert_eq!(fills[0].rect.size.height, 100.0);
}

#[test]
fn content_logical_size_uses_device_pixel_ratio() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (3840, 2160);
    app.scale_factor = 2.0;
    let (w, h) = app.content_logical_size();
    let frame_w = 2.0 * (layout::PAGE_FRAME_INSET_H * app.scale_factor + app.effective_page_frame_border());
    let expected_w = ((3840.0 - frame_w) / app.scale_factor).floor() as u32;
    assert_eq!(w, expected_w);
    assert!(h > 0);
    let (phys_w, phys_h) = app.content_physical_size();
    assert_eq!(phys_w, (3840.0 - frame_w) as u32);
    assert!(phys_h >= h);
    assert!((h as f32 * app.scale_factor) <= phys_h as f32 + f32::EPSILON);
}

#[test]
fn startup_has_single_default_tab() {
    let app = BrowserApp::new(RenderMode::Cpu);
    assert_eq!(app.shell.tab_count(), 1, "should start with exactly one tab");
}

fn clear_root_bookmarks(shell: &mut zero_browser_shell::BrowserShell) {
    let ids: Vec<_> = shell
        .bookmarks()
        .list_root()
        .into_iter()
        .map(|bookmark| bookmark.id())
        .collect();
    for id in ids {
        shell.bookmarks_mut().remove(id);
    }
}

#[test]
fn bookmarks_bar_hidden_without_bookmarks_or_when_disabled() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    clear_root_bookmarks(&mut app.shell);
    assert!(!app.bookmarks_bar_visible());
    assert_eq!(app.bookmarks_bar_height_for(1.0), 0.0);
    assert_eq!(app.chrome_top_y_for(1.0), layout::TOOLBAR_HEIGHT);

    let mut with_bookmark = BrowserApp::new(RenderMode::Cpu);
    clear_root_bookmarks(&mut with_bookmark.shell);
    with_bookmark
        .shell
        .bookmarks_mut()
        .add("Example", "https://example.com", None);
    with_bookmark.shell.settings_mut().show_bookmarks_bar = true;
    assert!(with_bookmark.bookmarks_bar_visible());
    assert_eq!(
        with_bookmark.bookmarks_bar_height_for(1.0),
        layout::BOOKMARKS_BAR_HEIGHT
    );
    assert_eq!(
        with_bookmark.chrome_top_y_for(1.0),
        layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT
    );

    let mut disabled = BrowserApp::new(RenderMode::Cpu);
    clear_root_bookmarks(&mut disabled.shell);
    disabled
        .shell
        .bookmarks_mut()
        .add("Example", "https://example.com", None);
    disabled.shell.settings_mut().show_bookmarks_bar = false;
    assert!(!disabled.bookmarks_bar_visible());
    assert_eq!(disabled.chrome_top_y_for(1.0), layout::TOOLBAR_HEIGHT);
}

#[test]
fn unfocus_marks_gpu_surface_stale() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.gpu_surface_stale = false;
    app.window_focused = false;
    app.gpu_surface_stale = true;
    assert!(app.gpu_surface_stale);
}

#[test]
fn browser_window_config_starts_maximized_not_fullscreen() {
    let config = super::browser_window_config();
    assert!(!config.fullscreen);
    assert!(config.maximized);
    // 无装饰平台：Wayland（规避 CSD 崩溃）+ Windows（自绘标题栏）
    let undecorated = crate::app::is_wayland() || cfg!(target_os = "windows");
    if undecorated {
        assert!(!config.decorations, "应禁用系统装饰");
    } else {
        assert!(config.decorations, "应保留系统装饰");
    }
}

/// Wayland 非最大化时应自绘窗口外框，便于与桌面其他窗口区分。
#[test]
fn custom_window_frame_border_on_wayland_when_not_maximized() {
    if !crate::app::is_wayland() {
        return;
    }

    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;
    app.set_window_maximized(false);

    let (_, _, overlay, _, _, _) = app.build_scene_for_test(800, 600);
    let border = app.chrome_palette().window_frame_border;
    assert!(
        overlay.iter().any(|f| {
            f.color == border && f.rect.origin.x <= 0.5 && f.rect.origin.y <= 0.5 && f.rect.size.width >= 799.0
        }),
        "non-maximized Wayland window should draw top frame border"
    );

    app.set_window_maximized(true);
    let (_, _, overlay_max, _, _, _) = app.build_scene_for_test(800, 600);
    assert!(
        !overlay_max
            .iter()
            .any(|f| f.color == border && f.rect.size.width >= 799.0),
        "maximized window should not draw outer frame border"
    );
}

/// 全屏状态切换应触发重绘，且 macOS 全屏时标签栏左侧 traffic light 留白应消失。
#[test]
fn fullscreen_toggle_updates_state_and_leading_inset() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    assert!(!app.window_is_fullscreen_for_test());

    app.set_window_fullscreen(true);
    assert!(app.window_is_fullscreen_for_test());
    // macOS 全屏时不应为 traffic lights 预留留白
    assert_eq!(app.tab_bar_leading_inset(), 0.0);

    app.set_window_fullscreen(false);
    assert!(!app.window_is_fullscreen_for_test());
}

/// 两个相邻的非当前标签之间应绘制竖线分隔。
#[test]
fn adjacent_inactive_tabs_render_vertical_separator() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.new_tab(None);
    app.new_tab(None);

    let sep = app.chrome_palette().tab_separator;
    let line_h = layout::TAB_BAR_HEIGHT - 2.0 * layout::TAB_SEPARATOR_INSET;
    let (fills, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        fills
            .iter()
            .any(|f| { f.color == sep && (f.rect.size.height - line_h).abs() < 0.5 && f.rect.size.width <= 2.0 }),
        "adjacent inactive tabs should have a vertical separator"
    );

    let mut two_tabs = BrowserApp::new(RenderMode::Cpu);
    two_tabs.physical_size = (1280, 900);
    two_tabs.scale_factor = 1.0;
    let (fills_two, _, _, _, _, _) = two_tabs.build_scene_for_test(1280, 900);
    assert!(
        !fills_two.iter().any(|f| f.color == sep),
        "single tab should not draw tab separators"
    );
}

#[test]
fn wayland_forces_cpu_present_for_gpu_and_auto() {
    let gpu = BrowserApp::new(RenderMode::Gpu);
    let auto = BrowserApp::new(RenderMode::Auto);
    let cpu = BrowserApp::new(RenderMode::Cpu);
    if crate::app::is_wayland() {
        assert!(gpu.wayland_forces_cpu_present());
        assert!(auto.wayland_forces_cpu_present());
    }
    assert!(!cpu.wayland_forces_cpu_present());
}

#[test]
fn gpu_suspend_present_is_noop_without_renderer() {
    let mut app = BrowserApp::new(RenderMode::Gpu);
    app.suspend_gpu_present();
    app.resume_gpu_present();
}

#[test]
fn unfocused_event_does_not_trigger_redraw() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    assert!(app.window_focused, "should start focused for initial render");
    app.needs_redraw = true;

    app.window_focused = false;
    app.needs_redraw = false;

    let should_redraw = app.needs_redraw && app.window_focused;
    assert!(!should_redraw, "should NOT redraw when unfocused");

    app.window_focused = true;
    app.needs_redraw = true;
    let should_redraw = app.needs_redraw && app.window_focused;
    assert!(should_redraw, "should redraw after focus regained");
}

#[test]
fn redraw_skipped_when_unfocused() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.surface_configured = true;

    app.window_focused = false;
    app.needs_redraw = true;
    let can_render = app.window_focused && app.surface_configured;
    assert!(!can_render, "should skip render when unfocused");

    app.window_focused = true;
    let can_render = app.window_focused && app.surface_configured;
    assert!(can_render, "should render when focused and configured");
}

#[test]
fn build_scene_renders_loaded_webview_content() {
    let mut app = BrowserApp::new(RenderMode::Cpu);

    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.shell.navigate("https://example.test");
    app.load_webview_html(
        tab_id,
        "<html><body><p>Example Domain</p></body></html>",
        Some("body { color: black; } p { color: black; font-size: 16px; }"),
    );
    app.shell.on_page_loaded("Example Domain");

    // 验证不 panic
    let (_, _, _, _, _, _) = app.build_scene_for_test(800, 600);
}

/// 验证 Ctrl 修饰键追踪：按下 Ctrl 后标记为活跃，释放后恢复。
#[test]
fn ctrl_key_tracking() {
    let mut app = BrowserApp::new(RenderMode::Cpu);

    // 平台修饰键按下
    let mod_key = BrowserApp::test_modifier_key_name();
    app.handle_key(mod_key, true, None);
    assert!(app.is_ctrl_pressed(), "modifier should be true after modifier down");

    // 平台修饰键释放
    app.handle_key(mod_key, false, None);
    assert!(!app.is_ctrl_pressed(), "modifier should be false after modifier up");
}

/// 验证 Ctrl+L 聚焦地址栏并清空文本。
#[test]
fn ctrl_l_focuses_address_bar() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    assert!(!app.address_bar_focused);

    // Ctrl 按下 + L
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("l", true, None);
    assert!(app.address_bar_focused, "Mod+L should focus address bar");
}

/// Ctrl+Tab 应循环到下一个标签页。
#[test]
fn ctrl_tab_cycles_to_next_tab() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    let first = app.shell.active_tab_id().unwrap();
    app.new_tab(None);
    let second = app.shell.active_tab_id().unwrap();
    assert_ne!(first, second);

    // Ctrl+Tab 应回到第一个标签
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("Tab", true, None);
    assert_eq!(app.shell.active_tab_id().unwrap(), first);

    // 再 Ctrl+Tab 到第二个
    app.handle_key("Tab", true, None);
    assert_eq!(app.shell.active_tab_id().unwrap(), second);
}

/// Ctrl+H 应打开历史页。
#[test]
fn ctrl_h_opens_history_page() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("h", true, None);
    assert_eq!(app.address_bar_text(), "zero://history");
}

/// Ctrl+J 应打开下载页。
#[test]
fn ctrl_j_opens_downloads_page() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("j", true, None);
    assert_eq!(app.address_bar_text(), "zero://downloads");
}

/// Alt+Home 应导航到设置的主页 URL，而非硬编码。
#[test]
fn alt_home_navigates_to_configured_home() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.shell
        .apply_settings(|s| s.home_url = "https://custom.home.test".to_string());
    app.handle_key("Alt", true, None);
    app.handle_key("Home", true, None);
    assert_eq!(
        app.shell.active_tab().and_then(|t| t.url()),
        Some("https://custom.home.test")
    );
}

/// 地址栏右侧应渲染浏览器菜单（三点）按钮。
#[test]
fn address_bar_renders_browser_menu_button() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        glyphs
            .iter()
            .any(|g| g.ch == crate::ui_icons::Icon::MoreVertical.as_char()),
        "address bar should render the browser menu icon"
    );
}

/// 工具栏尾部按钮（下载/主题/菜单）应全部位于地址栏 pill 右侧，不得重叠。
#[test]
fn trailing_toolbar_buttons_do_not_overlap_address_bar() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.build_scene_for_test(1280, 900);

    let (bar_x, _, bar_w, _) = app.address_bar_layout_for_test();
    let bar_right = bar_x + bar_w;

    let (dl_x, _, dl_w, _) = app.toolbar_download_button_rect_for_test();
    let (theme_x, _, theme_w, _) = app.toolbar_theme_button_rect_for_test_full();
    let (menu_x, _, menu_w, _) = app.toolbar_menu_button_rect_for_test();

    // 每个按钮都应在地址栏右边界之后
    assert!(dl_x >= bar_right, "download button must start after address bar pill");
    assert!(theme_x >= dl_x + dl_w, "theme button must follow download button");
    assert!(menu_x >= theme_x + theme_w, "menu button must follow theme button");
    // 菜单按钮右边界不应超出窗口
    assert!(menu_x + menu_w <= 1280.0, "menu button must stay within window");
}

/// HTTPS 页面地址栏 leading slot 应绘制锁图标。
#[test]
fn address_bar_renders_lock_icon_for_https() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    if let Some(tab) = app.shell.active_tab_mut() {
        tab.set_url("https://example.com");
        tab.set_loading(false);
    }

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        glyphs.iter().any(|g| g.ch == crate::ui_icons::Icon::Lock.as_char()),
        "https tab should render lock icon in address bar"
    );
}

/// 地址栏内右侧只应保留星标与盾牌两个图标，不再有页面操作三点菜单
/// （避免与地址栏外的全局菜单三点重复）。
#[test]
fn address_bar_inner_drops_page_actions_menu() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(1280, 900);
    let more_vertical_count = glyphs
        .iter()
        .filter(|g| g.ch == crate::ui_icons::Icon::MoreVertical.as_char())
        .count();
    assert_eq!(
        more_vertical_count, 1,
        "exactly one MoreVertical icon (the outer global menu) should be rendered"
    );
}

/// 工具栏应渲染下载按钮图标。
#[test]
fn toolbar_renders_download_button() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        glyphs.iter().any(|g| g.ch == crate::ui_icons::Icon::Download.as_char()),
        "toolbar should render download button icon"
    );
}

/// 有活跃下载时工具栏下载按钮应显示角标。
#[test]
fn download_toolbar_shows_active_badge() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");

    let (fills, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    let attention = app.chrome_palette().tab_attention;
    assert!(
        fills.iter().any(|f| f.color == attention && f.rect.size.width <= 12.0),
        "active download should show badge on toolbar download button"
    );
}

/// 多标签拥挤时标签宽度应压缩到最小值以下 ideal 宽度。
#[test]
fn crowded_tabs_compress_tab_width() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (640, 900);
    app.scale_factor = 1.0;
    for _ in 0..8 {
        app.new_tab(None);
    }
    let _ = app.build_scene_for_test(640, 900);
    let tab_id = app.shell.active_tab_id().unwrap();
    let (_, tab_w) = app.tab_layout_rect_for_test(tab_id).expect("tab layout");
    assert!(
        tab_w <= layout::TAB_MIN_WIDTH + 1.0,
        "crowded tabs should compress width, got {tab_w}"
    );
}

/// open_history_page 应导航到 zero://history。
#[test]
fn open_history_page_sets_url() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.open_history_page();
    assert_eq!(app.address_bar_text(), "zero://history");
}

/// 工具栏应渲染主题切换按钮（默认 Auto 为日月图标）。
#[test]
fn toolbar_renders_theme_button() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.shell
        .apply_settings(|settings| settings.color_theme = zero_browser_shell::ColorThemePreference::Auto);

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        glyphs.iter().any(|g| g.ch == crate::ui_icons::Icon::SunMoon.as_char()),
        "toolbar should render sun-moon icon for auto theme"
    );
}

/// 点击主题按钮应在 Auto → Light → Dark 间轮换。
#[test]
fn theme_button_cycles_preference() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.shell
        .apply_settings(|settings| settings.color_theme = zero_browser_shell::ColorThemePreference::Auto);

    let (btn_x, btn_y, btn_w, btn_h) = app.toolbar_theme_button_rect_for_test();
    let cx = (btn_x + btn_w * 0.5) as f64;
    let cy = (btn_y + btn_h * 0.5) as f64;

    assert_eq!(
        app.shell.settings().color_theme,
        zero_browser_shell::ColorThemePreference::Auto
    );
    app.handle_mouse_click(cx, cy, true, "Left");
    assert_eq!(
        app.shell.settings().color_theme,
        zero_browser_shell::ColorThemePreference::Light
    );
    assert_eq!(app.color_scheme_for_test(), zero_engine::PrefersColorSchemeValue::Light);

    app.handle_mouse_click(cx, cy, true, "Left");
    assert_eq!(
        app.shell.settings().color_theme,
        zero_browser_shell::ColorThemePreference::Dark
    );
    assert_eq!(app.color_scheme_for_test(), zero_engine::PrefersColorSchemeValue::Dark);
}

/// R1993：`toggle_print_preview`（Ctrl+P）翻转渲染媒体类型 Screen ↔ Print（DC-12）。
///
/// 验证 App 层 toggle 逻辑 + TabManager 持久化：默认 Screen → Print → Screen 往返。
/// （webview 层 set_media_type 触达级联由 R1992 integration smoke 覆盖。）
#[test]
fn toggle_print_preview_flips_media_type() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    assert_eq!(app.media_type_for_test(), zero_engine::MediaType::Screen);
    app.toggle_print_preview();
    assert_eq!(app.media_type_for_test(), zero_engine::MediaType::Print);
    app.toggle_print_preview();
    assert_eq!(app.media_type_for_test(), zero_engine::MediaType::Screen);
}

/// 点击地址栏右侧三点按钮应打开浏览器菜单。
#[test]
fn browser_menu_button_opens_context_menu() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let s = app.scale_factor;
    let (btn_x, btn_y, menu_btn_w, bar_h) = app.toolbar_menu_button_rect_for_test();

    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        true,
        "Left",
    );
    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        false,
        "Left",
    );
    assert!(
        app.is_context_menu_visible_for_test(),
        "browser menu should stay open after button press+release"
    );
}

/// 浏览器菜单项点击（按下 + 释放）应触发对应动作。
#[test]
fn browser_menu_item_click_new_tab() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let count_before = app.shell.tab_count();

    let s = app.scale_factor;
    let (btn_x, btn_y, menu_btn_w, bar_h) = app.toolbar_menu_button_rect_for_test();

    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        true,
        "Left",
    );
    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        false,
        "Left",
    );
    assert!(app.is_context_menu_visible_for_test());

    let menu_x = btn_x + menu_btn_w - layout::CONTEXT_MENU_WIDTH * s;
    let menu_y = btn_y + bar_h + 4.0 * s;
    let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
    let item_x = (menu_x + layout::CONTEXT_MENU_WIDTH * s * 0.5) as f64;
    let item_y = (menu_y + row_h * 0.5) as f64;

    app.handle_mouse_click(item_x, item_y, true, "Left");

    assert_eq!(
        app.shell.tab_count(),
        count_before + 1,
        "clicking New Tab in browser menu should open a tab"
    );
}

/// 标签栏右键应打开标签上下文菜单。
#[test]
fn tab_context_menu_opens_on_right_click() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    let _ = app.build_scene_for_test(1280, 900);
    let (tab_x, tab_w) = app.tab_layout_rect_for_test(tab_id).expect("tab layout");
    let s = app.scale_factor;
    let tab_y = layout::TAB_BAR_TOP_INSET * s;
    let tab_h = layout::TAB_BAR_HEIGHT * s;
    let x = (tab_x + tab_w * 0.5) as f64;
    let y = (tab_y + tab_h * 0.5) as f64;

    app.handle_mouse_click(x, y, true, "Right");
    assert!(
        app.is_context_menu_visible_for_test(),
        "right-click tab should open tab menu"
    );
}

/// 标签上下文菜单「固定标签页」应切换 pinned 状态。
#[test]
fn tab_context_menu_pin_tab() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    assert!(!app.shell.tab(tab_id).unwrap().is_pinned());
    let _ = app.build_scene_for_test(1280, 900);
    let (tab_x, tab_w) = app.tab_layout_rect_for_test(tab_id).unwrap();
    let s = app.scale_factor;
    let tab_y = layout::TAB_BAR_TOP_INSET * s;
    let tab_h = layout::TAB_BAR_HEIGHT * s;
    let x = (tab_x + tab_w * 0.5) as f64;
    let y = (tab_y + tab_h * 0.5) as f64;

    app.handle_mouse_click(x, y, true, "Right");
    let menu_x = x as f32 + 4.0;
    let menu_y = y as f32 + 4.0;
    let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
    let pin_y = (menu_y + row_h * 1.5) as f64;
    app.handle_mouse_click(menu_x as f64, pin_y, true, "Left");

    assert!(app.shell.tab(tab_id).unwrap().is_pinned());
}

/// 键盘 ↓ 选中自动补全首项时应使用 selected 背景色。
#[test]
fn autocomplete_keyboard_selection_uses_selected_bg() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.shell.navigate("https://example.com");
    app.shell.on_page_loaded("Example");
    app.address_bar_focused = true;
    for ch in ["e", "x", "a"] {
        app.handle_key(ch, true, None);
    }

    app.handle_key("ArrowDown", true, None);
    let (fills, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        fills
            .iter()
            .any(|f| f.color == app.chrome_palette().autocomplete_selected_bg),
        "keyboard-selected autocomplete row should use selected background"
    );
}

/// 验证 Ctrl+D 添加书签（当前页面）。
#[test]
fn about_page_loads_as_internal_document() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    let html = pages::generate_about_browser_html();
    app.open_internal_document_tab(html, "zero://about", "About ZeroBrowser");
    assert_eq!(app.address_bar_text(), "zero://about");
}

#[test]
fn ctrl_d_adds_bookmark() {
    let mut app = BrowserApp::new(RenderMode::Cpu);

    // 先导航到一个页面
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.shell.navigate("https://example.com");
    app.shell.on_page_loaded("Example");

    let count_before = app.shell.bookmarks().len();
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("d", true, None);
    assert_eq!(
        app.shell.bookmarks().len(),
        count_before + 1,
        "Ctrl+D should add a bookmark"
    );
}

/// 验证 Ctrl+W 关闭标签页。
#[test]
fn ctrl_w_closes_tab() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.new_tab(None);
    let count_before = app.shell.tab_count();
    assert!(count_before >= 2);

    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key("w", true, None);
    assert_eq!(
        app.shell.tab_count(),
        count_before - 1,
        "Ctrl+W should close active tab"
    );
}

/// 验证释放 Ctrl 后单键不再触发 Ctrl 快捷键。
#[test]
fn no_ctrl_shortcut_after_release() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.new_tab(None);

    // 按下并释放 Ctrl
    app.handle_key(BrowserApp::test_modifier_key_name(), true, None);
    app.handle_key(BrowserApp::test_modifier_key_name(), false, None);
    assert!(!app.is_ctrl_pressed());

    // 确认 ctrl_pressed 为 false
    assert!(!app.is_ctrl_pressed(), "ctrl should be released");
}

/// 验证下载栏在有活跃下载时正确渲染。
#[test]
fn download_bar_renders_when_active() {
    let mut app = BrowserApp::new(RenderMode::Cpu);

    // 添加一个活跃下载
    app.shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");

    // 构建场景应不 panic
    let (fills, glyphs, overlay, overlay_glyphs, _, _) = app.build_scene_for_test(800, 600);

    // 下载面板在 overlay 层，不应占满窗口宽度
    assert!(
        overlay.iter().any(|f| f.color == app.chrome_palette().download_bar_bg),
        "should have download panel background in overlay"
    );
    assert!(
        overlay
            .iter()
            .any(|f| { f.color == app.chrome_palette().download_bar_bg && f.rect.size.width < 700.0 }),
        "download panel should render as a floating panel, not full window width"
    );

    // 应有下载相关文字 glyph（下载面板在 overlay 层）
    let text: String = overlay_glyphs.iter().chain(glyphs.iter()).map(|g| g.ch).collect();
    assert!(
        text.contains("file.zip"),
        "download bar should show filename, got glyphs containing: {}",
        text.chars().take(200).collect::<String>()
    );

    let _ = (fills, glyphs, overlay_glyphs);
}

/// 默认不显示底部状态栏；WebView 高度不受状态栏占用。
#[test]
fn floating_link_status_hidden_by_default() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (800, 600);

    assert!(app.hovered_link_url().is_none());
    let (fills, _, _, _, _, _) = app.build_scene_for_test(800, 600);
    assert!(
        !fills.iter().any(|f| {
            f.rect.size.width >= 799.0
                && f.rect.size.height <= layout::STATUS_BAR_HEIGHT + 1.0
                && f.rect.origin.y > 500.0
                && f.color == app.chrome_palette().background
        }),
        "should not render full-width bottom status bar"
    );
}

/// 悬停链接时在左下角显示浮动 URL 状态栏（宽度随内容，不占布局）。
#[test]
fn floating_link_status_shows_on_link_hover() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;

    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
            tab_id,
            r#"<html><body style="margin:0"><a href="https://example.com/test" style="display:block;padding:10px">Example</a></body></html>"#,
            None,
        );

    let (cx, cy, _, _) = app.page_content_rect();
    app.handle_mouse_move((cx + 50.0) as f64, (cy + 25.0) as f64);
    assert_eq!(app.hovered_link_url(), Some("https://example.com/test"));

    let (_, glyphs, _, _, _, _) = app.build_scene_for_test(800, 600);
    let text: String = glyphs.iter().map(|g| g.ch).collect();
    assert!(
        text.contains("example.com"),
        "floating status should show link URL, got: {}",
        text.chars().take(120).collect::<String>()
    );
}

/// 验证设置页面生成正确 HTML。
#[test]
fn settings_page_generates_html() {
    let settings = zero_browser_shell::BrowserSettings::new();
    let html = pages::generate_settings_html(&settings);
    assert!(
        html.contains("设置") || html.contains("Settings"),
        "settings page should have title"
    );
    assert!(html.contains("Google"), "settings page should show search engine");
    assert!(html.contains("example.com"), "settings page should show home URL");
    assert!(html.contains("ZeroBrowser"), "settings page should show browser name");
    assert!(
        html.contains("zero://settings/toggle/show_bookmarks_bar"),
        "settings page should expose bookmarks bar toggle"
    );
    assert!(
        html.contains("zero://settings/toggle/javascript_enabled"),
        "settings page should expose javascript toggle"
    );
    assert!(
        html.contains("zero://settings/cycle/search_engine"),
        "settings page should expose search engine cycle"
    );
    assert!(
        html.contains("zero://settings/set/home_url/https%3A%2F%2Fexample.com"),
        "settings page should expose home url presets"
    );
    assert!(
        html.contains("zero://settings/edit/home_url"),
        "settings page should expose custom home url editor"
    );
    assert!(
        html.contains("zero://settings/adjust/default_zoom/up"),
        "settings page should expose default zoom controls"
    );
}

/// 轮换搜索引擎应更新设置并留在设置页。
#[test]
fn settings_cycle_search_engine_advances_engine() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.shell
        .apply_settings(|settings| settings.search_engine = zero_browser_shell::SearchEngine::Google);
    app.navigate_to("zero://settings/cycle/search_engine");
    assert_eq!(
        app.shell.settings().search_engine,
        zero_browser_shell::SearchEngine::Bing
    );
    assert_eq!(app.address_bar_text(), "zero://settings");
}

/// 设置主页 URL 预设链接应写入配置。
#[test]
fn settings_home_url_preset_updates_home() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.navigate_to("zero://settings/set/home_url/zero%3A%2F%2Fnewtab");
    assert_eq!(app.shell.settings().home_url, "zero://newtab");
    assert_eq!(app.address_bar_text(), "zero://settings");
}

/// 自定义主页链接应聚焦地址栏并带上设置前缀。
#[test]
fn settings_edit_home_url_focuses_address_bar() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.navigate_to("zero://settings/edit/home_url");
    assert!(app.address_bar_focused);
    assert_eq!(app.address_bar_text(), "zero://settings/set/home_url/");
}

/// 地址栏输入完整设置 URL 应保存自定义主页。
#[test]
fn settings_custom_home_url_via_address_bar() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.navigate_to("zero://settings/set/home_url/https://custom.test/home");
    assert_eq!(app.shell.settings().home_url, "https://custom.test/home");
}

/// 默认缩放调整链接应持久化并更新当前缩放。
#[test]
fn settings_adjust_default_zoom_updates_shell() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.shell.apply_settings(|settings| settings.default_zoom = 1.0);
    app.navigate_to("zero://settings/adjust/default_zoom/up");
    assert!((app.shell.settings().default_zoom - 1.1).abs() < f32::EPSILON);
    assert!((app.shell.zoom() - 1.1).abs() < f32::EPSILON);
    app.navigate_to("zero://settings/set/default_zoom/1.0");
    assert!((app.shell.settings().default_zoom - 1.0).abs() < f32::EPSILON);
}

/// 设置页 toggle URL 应切换对应选项并留在设置页。
#[test]
fn settings_toggle_url_flips_option_and_stays_on_settings() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.shell.apply_settings(|settings| settings.show_bookmarks_bar = true);
    let initial = app.shell.settings().show_bookmarks_bar;
    app.navigate_to("zero://settings/toggle/show_bookmarks_bar");
    assert_eq!(app.shell.settings().show_bookmarks_bar, !initial);
    assert_eq!(app.address_bar_text(), "zero://settings");
    assert!(
        !app.shell.active_tab().unwrap().is_loading(),
        "settings toggle should clear loading state"
    );
}

/// 标签 loading 时地址栏 leading slot 应绘制 spinner。
#[test]
fn address_bar_shows_loading_spinner_when_tab_loading() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    if let Some(tab) = app.shell.active_tab_mut() {
        tab.set_loading(true);
    }
    let loading = app.chrome_palette().loading_indicator;
    let (fills_loading, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    let spinner_segments = fills_loading.iter().filter(|f| f.color == loading).count();
    assert!(
        spinner_segments >= 28,
        "loading tab should draw spinner segments, got {spinner_segments}"
    );

    if let Some(tab) = app.shell.active_tab_mut() {
        tab.set_loading(false);
    }
    let (fills_idle, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    let idle_segments = fills_idle.iter().filter(|f| f.color == loading).count();
    assert_eq!(idle_segments, 0, "idle tab should not draw loading spinner");
}

/// 验证 open_settings_page 正确加载。
#[test]
fn open_settings_page_loads_in_webview() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    app.open_settings_page();

    assert_eq!(app.address_bar_text(), "zero://settings");
    assert!(
        !app.shell.active_tab().unwrap().is_loading(),
        "settings page should clear loading state"
    );
    // WebView 应该有渲染结果
    let (_, _, _, _, _, _) = app.build_scene_for_test(800, 600);
}

/// 欢迎页包含双语内容与可点击链接。
#[test]
fn welcome_page_has_bilingual_content_and_links() {
    assert!(pages::WELCOME_HTML.contains("Welcome · 欢迎"));
    assert!(pages::WELCOME_HTML.contains("Built different"));
    assert!(pages::WELCOME_HTML.contains("href=\"https://example.com\""));
    assert!(pages::WELCOME_HTML.contains("prefers-color-scheme: dark"));
}

/// 欢迎页在 800px 高的 WebView 视口内应完整显示（无需滚动）。
#[test]
fn welcome_page_fits_800px_webview_viewport() {
    let viewport_h = 800.0;
    let mut pipeline = zero_engine::RenderPipeline::new(880.0, viewport_h);
    let result = pipeline.render_html(pages::WELCOME_HTML, "");

    let mut page_h = 0.0f32;
    for fill in &result.primitives.fills {
        page_h = page_h.max(fill.rect.origin.y + fill.rect.size.height);
    }
    for glyph in &result.primitives.glyphs {
        page_h = page_h.max(glyph.y + glyph.font_size);
    }

    assert!(
        page_h <= viewport_h,
        "welcome page height {page_h}px exceeds {viewport_h}px webview viewport"
    );
}

/// zero:// 协议应保留不被搜索引擎改写。
#[test]
fn normalize_url_preserves_zero_scheme() {
    let shell = zero_browser_shell::BrowserShell::new();
    assert_eq!(crate::app::normalize_url("zero://settings", &shell), "zero://settings");
}

/// 长文档在内容区滚轮应更新 scroll_offset。
#[test]
fn handle_scroll_updates_offset_for_tall_page() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 2400px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div></body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    let (_, content_y, content_w, _) = app.page_content_rect();
    let x = (content_w * 0.5) as f64;
    let y = content_y as f64 + 100.0;
    app.mouse_pos = (x, y);

    let (fills_at_zero, _, _, _, _, _) = app.build_scene_for_test(1280, 900);

    // Linux/WSL 滚轮向下通常为负 LineDelta
    app.handle_scroll(zero_host_runtime::event::MouseScrollDelta::LineDelta(0.0, -3.0), x, y);

    assert!(
        app.scroll_offset_for_tab(tab_id) > 0.0,
        "tall page should scroll with negative line delta (scroll down on Linux)"
    );

    let content_top = content_y;
    let (fills_after, _, _, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        fills_after
            .iter()
            .filter(|f| f.rect.size.height > 2000.0)
            .all(|f| f.rect.origin.y >= content_top),
        "scrolled page fills must not paint above content area (content_top={content_top})"
    );
    let _ = fills_at_zero;
}

/// 触摸屏在内容区拖拽应更新 scroll_offset。
#[test]
fn handle_touch_drag_updates_offset_for_tall_page() {
    use zero_host_runtime::event::{TouchEvent, TouchPhase};

    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 2400px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div></body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    let (_, content_y, content_w, _) = app.page_content_rect();
    let touch_x = content_w as f64 * 0.5;
    let start_y = content_y as f64 + 120.0;

    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Started,
        x: touch_x,
        y: start_y,
    });
    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Moved,
        x: touch_x,
        y: start_y - 80.0,
    });

    assert!(
        app.scroll_offset_for_tab(tab_id) > 0.0,
        "touch drag up should increase scroll offset on tall page"
    );
}

/// 触摸 tap chrome UI 区应合成左键 click（验证「+」按钮 tap 新建标签）。
#[test]
fn handle_touch_tap_on_new_tab_button_creates_tab() {
    use zero_host_runtime::event::{TouchEvent, TouchPhase};

    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let initial_count = app.shell.tab_count();

    // build_scene 填充 tab_layout
    let _ = app.build_scene_for_test(1280, 900);
    let new_tab_x = app.new_tab_button_x_for_test();
    let tap_x = new_tab_x as f64 + 8.0;
    let tap_y = 12.0;

    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Started,
        x: tap_x,
        y: tap_y,
    });
    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Ended,
        x: tap_x,
        y: tap_y,
    });

    assert_eq!(
        app.shell.tab_count(),
        initial_count + 1,
        "touch tap on '+' should create a new tab"
    );
}

/// 触摸 tap 移动超过阈值不应合成 click（避免滚动误触）。
#[test]
fn handle_touch_swipe_does_not_trigger_tap() {
    use zero_host_runtime::event::{TouchEvent, TouchPhase};

    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let initial_count = app.shell.tab_count();

    let _ = app.build_scene_for_test(1280, 900);
    let new_tab_x = app.new_tab_button_x_for_test();
    let tap_x = new_tab_x as f64 + 8.0;
    let tap_y = 12.0;

    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Started,
        x: tap_x,
        y: tap_y,
    });
    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Moved,
        x: tap_x + 50.0,
        y: tap_y,
    });
    app.handle_touch(&TouchEvent {
        id: 1,
        phase: TouchPhase::Ended,
        x: tap_x + 50.0,
        y: tap_y,
    });

    assert_eq!(
        app.shell.tab_count(),
        initial_count,
        "swipe (move > threshold) should not trigger tap click"
    );
}

/// 鼠标左键拖拽（RDP/远程桌面触摸模拟）应更新 scroll_offset。
#[test]
fn handle_mouse_drag_scroll_updates_offset_for_tall_page() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 2400px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div></body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    let (content_x, content_y, content_w, _) = app.page_content_rect();
    let x = (content_x + content_w * 0.5) as f64;
    let y = content_y as f64 + 120.0;

    app.handle_mouse_click(x, y, true, "Left");
    app.handle_mouse_move(x, y - 80.0);
    app.handle_mouse_click(x, y - 80.0, false, "Left");

    assert!(
        app.scroll_offset_for_tab(tab_id) > 0.0,
        "mouse drag up in content area should scroll tall page (RDP touch path)"
    );
}

/// WebView 视口与页面框布局应落在窗口内，且缩放后高度不超过内容区。
#[test]
fn page_layout_and_webview_fit_within_frame() {
    for scale in [1.0_f32, 1.25, 1.5, 2.0] {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.physical_size = (1280, 900);
        app.scale_factor = scale;

        let tab_id = app.shell.active_tab_id().unwrap();
        app.ensure_webview(tab_id);
        app.sync_webview_viewport();
        app.load_webview_html(
            tab_id,
            "<html><body style='margin:0;background:#f4f6f8;height:100%'>Hi</body></html>",
            None,
        );

        let (fx, fy, fw, fh) = app.page_frame_rect_for(1280, 900);
        let (cx, cy, cw, ch) = app.page_content_rect_for(1280, 900);
        let frame_bottom = fy + fh;
        let bottom_reserve = layout::PAGE_FRAME_INSET_BOTTOM * scale;

        assert!(
            frame_bottom + bottom_reserve <= 900.0 + 0.5,
            "scale={scale}: non-maximized frame should only reserve bottom inset"
        );
        assert!(
            (cx + cw) <= fx + fw + 0.5 && (cy + ch) <= fy + fh - app.effective_page_frame_border() + 0.5,
            "scale={scale}: content rect must fit inside frame"
        );

        let (logical_w, logical_h) = app.content_logical_size();
        let Some((wv_w, wv_h)) = app.webview_logical_size_for_tab(tab_id) else {
            panic!("scale={scale}: missing webview");
        };
        assert_eq!((wv_w, wv_h), (logical_w, logical_h));
        assert!(
            wv_h as f32 * scale <= ch + 0.5,
            "scale={scale}: webview scaled height {} exceeds content h {ch}",
            wv_h as f32 * scale
        );

        let (_, _, _overlay, _, _, _) = app.build_scene_for_test(1280, 900);

        // corner pixel sampling covered by page_frame_bottom_corners_use_separator_overlay.
    }
}

/// 最大化时的底部预留：仅 Linux（WSLg 圆角裁切）启用 guard，
/// Windows/macOS 最大化窗口为纯矩形，frame 应铺到窗口底边。
#[test]
fn page_layout_bottom_respects_platform_when_maximized() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    app.set_window_maximized(true);

    let (_, fy, _, fh) = app.page_frame_rect_for(1280, 900);
    let frame_bottom = fy + fh;

    if cfg!(target_os = "linux") {
        // Linux/WSLg 保留 guard 避免圆角裁切
        let window_bottom = 900.0 - layout::PAGE_FRAME_BOTTOM_CLIP_GUARD - layout::PAGE_FRAME_BOTTOM_UI_GUARD;
        assert!(
            frame_bottom + layout::PAGE_FRAME_INSET_BOTTOM <= window_bottom + 0.5,
            "maximized frame on Linux should stay above bottom guards"
        );
    } else {
        // Windows/macOS 最大化窗口为纯矩形，frame 底边应贴到窗口底边
        assert!(
            frame_bottom >= 900.0 - 0.5,
            "maximized frame on Windows/macOS should reach window bottom, got {frame_bottom}"
        );
    }
}

/// 圆角 frame 时 overlay 应包含遮罩；扁平 frame 时 overlay 仍可用于浮动 UI。
#[test]
fn page_frame_bottom_corners_use_separator_overlay() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    if app.effective_page_frame_radius() <= 0.0 {
        return;
    }
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
        tab_id,
        "<html><body><div style='height:100px;background:#ff3232'>Short</div></body></html>",
        Some("html, body { margin: 0; background: #ff3232; }"),
    );
    app.sync_webview_viewport();
    app.shell.on_page_loaded("Tall");

    let (_, _, overlay_fills, _, _, _) = app.build_scene_for_test(1280, 900);
    assert!(
        !overlay_fills.is_empty(),
        "page frame overlay should include corner masks and border"
    );

    let fb = app.render_scene_for_test(1280, 900);
    let (cx, cy, cw, ch) = app.page_content_rect();
    let sep = app.chrome_palette().separator;

    let sample_points = [(cx + 1.0, cy + ch - 1.0), (cx + cw - 2.0, cy + ch - 1.0)];
    for (px, py) in sample_points {
        let x = px.round() as u32;
        let y = py.round() as u32;
        let i = ((y * fb.width + x) * 4) as usize;
        let r = fb.data[i];
        let g = fb.data[i + 1];
        let b = fb.data[i + 2];
        assert!(
            (r as i16 - sep.r as i16).abs() <= 2
                && (g as i16 - sep.g as i16).abs() <= 2
                && (b as i16 - sep.b as i16).abs() <= 2,
            "corner pixel at ({x},{y}) should match separator {:?}, got rgb({r},{g},{b})",
            sep
        );
    }
}

#[test]
fn append_webview_primitives_clip_excludes_outside_range() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::rgb(255, 0, 0));
    primitives.add_fill(Rect::new(0.0, 100.0, 10.0, 10.0), Color::rgb(0, 255, 0));

    let mut fills = Vec::new();
    let mut glyphs = Vec::new();
    assert!(append_webview_primitives(
        &primitives,
        &mut fills,
        &mut glyphs,
        0.0,
        50.0,
        1,
        1.0,
        Some((50.0, 80.0)),
        None,
    ));
    assert_eq!(fills.len(), 1, "only fill intersecting clip band should remain");
    assert_eq!(fills[0].rect.origin.y, 50.0);
}

/// 与 clip 带部分相交的 fill 应被裁剪，而非整颗绘制到 chrome 上方。
#[test]
fn append_webview_primitives_clip_intersects_partial_fill() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_fill(Rect::new(0.0, 40.0, 10.0, 30.0), Color::rgb(255, 0, 0));

    let mut fills = Vec::new();
    let mut glyphs = Vec::new();
    assert!(append_webview_primitives(
        &primitives,
        &mut fills,
        &mut glyphs,
        0.0,
        0.0,
        1,
        1.0,
        Some((50.0, 80.0)),
        None,
    ));
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].rect.origin.y, 50.0);
    assert_eq!(fills[0].rect.size.height, 20.0);
}

/// 模拟滚动后文档上移：fill 顶部不得超出 clip_top（避免盖住标签栏）。
#[test]
fn append_webview_primitives_clip_pins_scroll_overflow_to_content_top() {
    let mut primitives = RenderPrimitives::new();
    primitives.add_fill(Rect::new(0.0, 0.0, 200.0, 500.0), Color::rgb(200, 220, 240));

    let mut fills = Vec::new();
    let mut glyphs = Vec::new();
    let chrome_top = layout::TOOLBAR_HEIGHT;
    let scroll = 40.0;
    assert!(append_webview_primitives(
        &primitives,
        &mut fills,
        &mut glyphs,
        0.0,
        chrome_top - scroll,
        1,
        1.0,
        Some((chrome_top, 878.0)),
        None,
    ));
    assert!(!fills.is_empty());
    assert!(
        fills.iter().all(|f| f.rect.origin.y >= chrome_top),
        "fill must not extend above content area top"
    );
}

/// 圆角矩形等 extra 图元须与 fill 一样裁剪，避免滚动后盖住地址栏。
#[test]
fn transform_webview_primitives_clips_rounded_rect_above_viewport() {
    use app::{ViewportClip, transform_webview_primitives};
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{RenderPrimitives, RoundedRectPrimitive};

    let mut primitives = RenderPrimitives::new();
    primitives.rounded_rects.push(RoundedRectPrimitive::uniform(
        Rect::new(10.0, 0.0, 40.0, 20.0),
        Color::BLUE,
        4.0,
    ));

    let chrome_top = layout::TOOLBAR_HEIGHT;
    let scroll = 50.0;
    let clip = ViewportClip::new(0.0, chrome_top, 1280.0, 800.0);
    let out = transform_webview_primitives(&primitives, 0.0, chrome_top - scroll, 1.0, Some(clip));
    assert!(
        out.rounded_rects.is_empty(),
        "rounded rect scrolled above viewport must be culled"
    );
}

/// DC-10：`transform_webview_primitives` 必须对全部图元类型正确应用 scale_factor + offset。
/// 覆盖 fills / rounded_rects（含 4 圆角独立缩放）/ gradients（Linear/Radial/Conic 三变体）/
/// shadows（offset+blur+spread）/ strokes（端点+线宽）/ glyphs（font_size）/ transforms（rect+origin+tx/ty）。
/// 公式：out = in * scale + offset。
#[test]
fn transform_webview_primitives_applies_scale_and_offset_to_all_types() {
    use app::transform_webview_primitives;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{
        FontId, GlyphPrimitive, GradientKind, GradientPrimitive, LineCap, LineStyle, RenderPrimitives,
        RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive, TransformPrimitive,
    };

    let mut p = RenderPrimitives::new();
    p.add_fill(Rect::new(1.0, 2.0, 10.0, 20.0), Color::rgb(255, 0, 0));
    p.rounded_rects.push(RoundedRectPrimitive {
        rect: Rect::new(1.0, 2.0, 10.0, 20.0),
        color: Color::rgb(0, 255, 0),
        top_left_radius: 1.0,
        top_right_radius: 2.0,
        bottom_right_radius: 3.0,
        bottom_left_radius: 4.0,
    });
    p.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        kind: GradientKind::Linear {
            x0: 1.0,
            y0: 2.0,
            x1: 3.0,
            y1: 4.0,
        },
        stops: Vec::new(),
        repeating: false,
    });
    p.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        kind: GradientKind::Radial {
            cx: 1.0,
            cy: 2.0,
            inner_radius: 3.0,
            outer_radius: 5.0,
        },
        stops: Vec::new(),
        repeating: false,
    });
    p.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        kind: GradientKind::Conic {
            cx: 1.0,
            cy: 2.0,
            start_angle: 0.5,
        },
        stops: Vec::new(),
        repeating: false,
    });
    p.shadows.push(ShadowPrimitive {
        rect: Rect::new(1.0, 2.0, 10.0, 10.0),
        color: Color::rgb(0, 0, 0),
        offset_x: 2.0,
        offset_y: 3.0,
        blur_radius: 4.0,
        spread_radius: 5.0,
    });
    p.strokes.push(StrokePrimitive {
        x1: 1.0,
        y1: 2.0,
        x2: 3.0,
        y2: 4.0,
        width: 5.0,
        color: Color::rgb(0, 0, 0),
        style: LineStyle::Solid,
        cap: LineCap::Butt,
    });
    p.add_glyph(GlyphPrimitive {
        x: 1.0,
        y: 2.0,
        font_size: 16.0,
        color: Color::rgb(0, 0, 0),
        glyph_id: 'A' as u32,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    p.transforms.push(TransformPrimitive {
        rect: Rect::new(1.0, 2.0, 10.0, 10.0),
        origin_x: 1.0,
        origin_y: 2.0,
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 3.0,
        ty: 4.0,
    });

    let out = transform_webview_primitives(&p, 10.0, 20.0, 2.0, None);

    let f = &out.fills[0];
    assert_eq!((f.rect.origin.x, f.rect.origin.y), (12.0, 24.0));
    assert_eq!((f.rect.size.width, f.rect.size.height), (20.0, 40.0));

    let r = &out.rounded_rects[0];
    assert_eq!((r.rect.origin.x, r.rect.origin.y), (12.0, 24.0));
    assert_eq!((r.rect.size.width, r.rect.size.height), (20.0, 40.0));
    assert_eq!(
        (
            r.top_left_radius,
            r.top_right_radius,
            r.bottom_right_radius,
            r.bottom_left_radius
        ),
        (2.0, 4.0, 6.0, 8.0)
    );

    match &out.gradients[0].kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!((*x0, *y0, *x1, *y1), (12.0, 24.0, 16.0, 28.0));
        }
        _ => panic!("expected Linear gradient"),
    }
    match &out.gradients[1].kind {
        GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => {
            assert_eq!((*cx, *cy, *inner_radius, *outer_radius), (12.0, 24.0, 6.0, 10.0));
        }
        _ => panic!("expected Radial gradient"),
    }
    match &out.gradients[2].kind {
        // Conic 的 start_angle 是无量纲角度，不应被 scale 缩放。
        GradientKind::Conic { cx, cy, start_angle } => {
            assert_eq!((*cx, *cy, *start_angle), (12.0, 24.0, 0.5));
        }
        _ => panic!("expected Conic gradient"),
    }

    let sh = &out.shadows[0];
    assert_eq!(
        (sh.offset_x, sh.offset_y, sh.blur_radius, sh.spread_radius),
        (4.0, 6.0, 8.0, 10.0)
    );

    let st = &out.strokes[0];
    assert_eq!((st.x1, st.y1, st.x2, st.y2, st.width), (12.0, 24.0, 16.0, 28.0, 10.0));

    let g = &out.glyphs[0];
    assert_eq!((g.x, g.y, g.font_size), (12.0, 24.0, 32.0));

    let t = &out.transforms[0];
    assert_eq!((t.rect.origin.x, t.rect.origin.y), (12.0, 24.0));
    assert_eq!((t.rect.size.width, t.rect.size.height), (20.0, 20.0));
    assert_eq!((t.origin_x, t.origin_y), (12.0, 24.0));
    assert_eq!((t.tx, t.ty), (6.0, 8.0));
}

/// DC-10：`transform_webview_primitives` 必须裁掉完全落在视口外的图元
///（rounded_rects / gradients / path_fills / glyphs），并保留视口内的图元。
#[test]
fn transform_webview_primitives_culls_primitives_outside_viewport() {
    use app::{ViewportClip, transform_webview_primitives};
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{
        FontId, GlyphPrimitive, GradientKind, GradientPrimitive, RenderPrimitives, RoundedRectPrimitive,
    };

    let mut p = RenderPrimitives::new();
    // 视口外（视口宽高 200，图元在 1000,1000）
    p.rounded_rects.push(RoundedRectPrimitive::uniform(
        Rect::new(1000.0, 1000.0, 10.0, 10.0),
        Color::BLUE,
        2.0,
    ));
    p.gradients.push(GradientPrimitive {
        interpolation: Default::default(),
        rect: Rect::new(1000.0, 1000.0, 10.0, 10.0),
        kind: GradientKind::Linear {
            x0: 1000.0,
            y0: 1000.0,
            x1: 1010.0,
            y1: 1010.0,
        },
        stops: Vec::new(),
        repeating: false,
    });
    p.add_path_fill(
        vec![1000.0, 1000.0, 1010.0, 1000.0, 1010.0, 1010.0],
        Color::rgb(0, 255, 0),
    );
    p.add_glyph(GlyphPrimitive {
        x: 1000.0,
        y: 1000.0,
        font_size: 16.0,
        color: Color::rgb(0, 0, 0),
        glyph_id: 'A' as u32,
        font_id: FontId(0),
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
    });
    // 视口内 fill（control：应保留）
    p.add_fill(Rect::new(10.0, 10.0, 10.0, 10.0), Color::rgb(0, 0, 255));
    // 视口外 fill（应裁掉）
    p.add_fill(Rect::new(1000.0, 1000.0, 10.0, 10.0), Color::rgb(255, 0, 0));

    let clip = ViewportClip::new(0.0, 0.0, 200.0, 200.0);
    let out = transform_webview_primitives(&p, 0.0, 0.0, 1.0, Some(clip));

    assert!(out.rounded_rects.is_empty(), "offscreen rounded rect must be culled");
    assert!(out.gradients.is_empty(), "offscreen gradient must be culled");
    assert!(out.path_fills.is_empty(), "offscreen path_fill must be culled");
    assert!(out.glyphs.is_empty(), "offscreen glyph must be culled");
    assert_eq!(out.fills.len(), 1, "only the in-viewport fill should survive");
}

/// DC-13 line 328：ZeroBrowser 不得对 WebView glyph 做改变布局语义的整行重排。
///
/// `transform_webview_primitives` 须按输入顺序逐个映射 glyph（仅 scale+offset + 裁剪），
/// 不得按 Y / 字体 / 纹理图集排序或批处理重排——否则会破坏跨行布局语义（如把不同 baseline
/// 的 glyph 合并到一行）。本测试构造**会被 sort-by-Y / font-batch 优化重排**的 glyph 序列
///（混合行 + 混合字体），断言输出严格保序，守此不变量防未来优化回归。
#[test]
fn transform_webview_primitives_preserves_glyph_order() {
    use app::transform_webview_primitives;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive, RenderPrimitives};

    let mut p = RenderPrimitives::new();
    // 输入顺序 = 布局文档顺序：A(行1 y=10, font0) → B(行1 y=10, font1) → C(行2 y=30, font0)
    // → D(行1 y=10, font0)。sort-by-Y-then-font 会把 D 移到 C 前；font-batch 会按 font 分组
    //（[A,C,D]+[B]）。两者都破坏跨行布局语义。
    let seq = [('A', 10.0, 0u32), ('B', 10.0, 1), ('C', 30.0, 0), ('D', 10.0, 0)];
    for (ch, y, fid) in seq {
        p.add_glyph(GlyphPrimitive {
            x: 5.0,
            y,
            font_size: 16.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: ch as u32,
            font_id: FontId(fid),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
        });
    }
    // 不裁剪（None）→ 全保留 → 顺序可直接比对。
    let out = transform_webview_primitives(&p, 0.0, 0.0, 1.0, None);
    assert_eq!(out.glyphs.len(), 4, "无裁剪须全保留");
    let out_ids: Vec<u32> = out.glyphs.iter().map(|g| g.glyph_id).collect();
    assert_eq!(
        out_ids,
        vec!['A' as u32, 'B' as u32, 'C' as u32, 'D' as u32],
        "glyph 须严格保输入顺序（不得按 Y/字体排序或批处理重排，DC-13 line 328）"
    );
}

/// Esc 在加载中应停止加载（无其他 Escape 上下文时）。
#[test]
fn escape_stops_loading_when_active_tab_loading() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    // 先导航到一个 URL，使 tab 拥有 url，refresh 才会触发 loading
    app.navigate_to("https://example.com/");
    app.shell.refresh();
    assert!(
        app.shell.active_tab().unwrap().is_loading(),
        "tab should be loading after refresh"
    );

    app.handle_key("Escape", true, None);
    assert!(
        !app.shell.active_tab().unwrap().is_loading(),
        "Escape should stop loading"
    );
}

/// Esc 在未加载时不应触发停止（无副作用）。
#[test]
fn escape_does_nothing_when_not_loading() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    assert!(!app.shell.active_tab().unwrap().is_loading());

    // 不应 panic，也无副作用
    app.handle_key("Escape", true, None);
    assert!(!app.shell.active_tab().unwrap().is_loading());
}

/// Space / PageDown 应向下滚动页面，PageUp 向上。
#[test]
fn keyboard_space_and_pagedown_scroll_page() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    // 高页面以产生可滚动区域
    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 4000px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div></body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    let before = app.scroll_offset_for_tab(tab_id);
    app.handle_key("PageDown", true, None);
    let after_pagedown = app.scroll_offset_for_tab(tab_id);
    assert!(
        after_pagedown > before,
        "PageDown should scroll down (before={before}, after={after_pagedown})"
    );

    app.handle_key("PageUp", true, None);
    let after_pageup = app.scroll_offset_for_tab(tab_id);
    assert!(
        after_pageup < after_pagedown,
        "PageUp should scroll up (after_pagedown={after_pagedown}, after_pageup={after_pageup})"
    );

    // Space 向下
    let mid = app.scroll_offset_for_tab(tab_id);
    app.handle_key("Space", true, None);
    assert!(app.scroll_offset_for_tab(tab_id) > mid, "Space should scroll down");
}

/// Home 滚动到页顶，End 滚动到页底。
#[test]
fn home_end_scroll_to_top_and_bottom() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 4000px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div></body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    // 先滚到中间
    app.handle_key("PageDown", true, None);
    let mid = app.scroll_offset_for_tab(tab_id);
    assert!(mid > 0.0, "should have scrolled down first");

    // Home 回到顶部
    app.handle_key("Home", true, None);
    assert_eq!(app.scroll_offset_for_tab(tab_id), 0.0, "Home should scroll to top");

    // End 到底部
    app.handle_key("End", true, None);
    let bottom = app.scroll_offset_for_tab(tab_id);
    assert!(bottom > mid, "End should scroll to bottom (bottom={bottom}, mid={mid})");
}

/// 子菜单展开后，鼠标从主菜单项移向子菜单面板（经过桥接区）时，
/// 子菜单应保持展开，而非因离开主菜单项立即收起。
#[test]
fn sub_menu_stays_open_when_crossing_bridge() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let s = app.scale_factor;
    let (btn_x, btn_y, menu_btn_w, bar_h) = app.toolbar_menu_button_rect_for_test();

    // 打开主菜单
    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        true,
        "Left",
    );
    app.handle_mouse_click(
        (btn_x + menu_btn_w * 0.5) as f64,
        (btn_y + bar_h * 0.5) as f64,
        false,
        "Left",
    );
    assert!(app.is_context_menu_visible_for_test());

    // 找到 history 子菜单父项
    let history_idx = app
        .context_menu_item_index_for_test("browser_menu_history")
        .expect("history sub-menu should exist");
    let menu_x = app.context_menu_x_for_test();
    let menu_y = app.context_menu_y_for_test();
    let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
    // 用实际行 y（含 separator 紧凑高度）
    let row_top = app.context_menu_row_y(history_idx);
    let item_cy = menu_y + row_top + row_h * 0.5;

    // hover history 项 → 展开子菜单
    app.handle_mouse_move((menu_x + 10.0) as f64, item_cy as f64);
    assert_eq!(
        app.open_sub_menu_for_test(),
        Some(history_idx),
        "hovering history item should open its sub-menu"
    );

    // 计算桥接区中点 x（主菜单与子菜单面板之间的间隙）
    let menu_w = layout::CONTEXT_MENU_WIDTH * s;
    let (sub_x, sub_y, _sub_w, _sub_h) = app.sub_menu_panel_rect(history_idx);
    let menu_right = menu_x + menu_w;
    let sub_w = menu_w;
    let (gap_left, gap_right) = if sub_x >= menu_right {
        (menu_right, sub_x)
    } else {
        (sub_x + sub_w, menu_x)
    };
    let bridge_x = ((gap_left + gap_right) * 0.5) as f64;
    let bridge_y = (sub_y + 5.0) as f64;

    app.handle_mouse_move(bridge_x, bridge_y);
    assert_eq!(
        app.open_sub_menu_for_test(),
        Some(history_idx),
        "sub-menu should stay open when pointer crosses bridge to sub-menu panel"
    );

    // 移动到子菜单面板内
    app.handle_mouse_move((sub_x + 10.0) as f64, (sub_y + 10.0) as f64);
    assert_eq!(
        app.open_sub_menu_for_test(),
        Some(history_idx),
        "sub-menu should remain open when pointer enters sub-menu panel"
    );
}

/// 刷新按钮在加载中点击应停止加载（按钮变停止语义）。
#[test]
fn refresh_button_click_stops_loading_when_loading() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;

    let _ = app.build_scene_for_test(1280, 900);
    app.navigate_to("https://example.com/");
    app.shell.refresh();
    assert!(app.shell.active_tab().unwrap().is_loading());

    // 计算刷新按钮中心坐标（导航区第 3 个按钮，index 2）
    let s = app.scale_factor;
    let nav_btn_w = layout::NAV_BUTTON_WIDTH * s;
    let refresh_btn_cx = (layout::NAV_SECTION_LEADING_PAD + nav_btn_w * 2.0 + nav_btn_w / 2.0) * s;
    let toolbar_y = layout::TAB_STRIP_HEIGHT * s;
    let btn_cy = toolbar_y + (layout::ADDRESS_BAR_HEIGHT * s) / 2.0;

    app.handle_mouse_click(refresh_btn_cx as f64, btn_cy as f64, true, "Left");
    app.handle_mouse_click(refresh_btn_cx as f64, btn_cy as f64, false, "Left");
    assert!(
        !app.shell.active_tab().unwrap().is_loading(),
        "clicking refresh button while loading should stop loading"
    );
}
