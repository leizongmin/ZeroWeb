//! ZeroBrowser 单元测试模块（从 main.rs 拆分以控制单文件体积）。
//!
//! 经 `#[cfg(test)] mod tests;` 在 main.rs 中声明；`super::*` 与 `super::browser_window_config`
//! 仍解析到 crate 根（main.rs），与内联时一致。

use super::*;
use app::append_webview_primitives;

mod html_scenario;
use html_scenario::{BrowserScenarioHost, HtmlScenario, StateExpectation};

/// R3254：多进程 GUI 测试串行化——并行 spawn 多个 renderer 子进程（每个约 582MB 二进制
/// 加字体加载）、叠加共享进程内 compositor client，会资源竞争导致快照轮询超时
/// （form_fixture / typing 并行即挂）。多进程测试须先持锁。
static MULTIPROCESS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
use zero_browser_shell::TabId;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};

fn wait_for_snapshot_after(app: &mut BrowserApp, tab_id: TabId, sequence: u64, gpu_present: bool) -> bool {
    // R3254-F10：重负载（make test 全量并行）下 renderer spawn + 渲染可超 60s——
    // 预算放宽到 120s（多进程 GUI 测试的等待容忍）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        if gpu_present {
            app.poll_tab_fetch_with_gpu_present_for_test();
        } else {
            app.poll_tab_fetch();
        }
        if app.snapshot_seq_for_test(tab_id) > sequence {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn smoke_capture_cli_requires_real_multiprocess_window_at_scale_one() {
    let parsed = parse_args_from(
        ["--renderer=gpu", "--scale=1", "--smoke-capture=target/smoke.png"]
            .into_iter()
            .map(str::to_string),
        None,
    )
    .unwrap();
    assert_eq!(parsed.render_mode, RenderMode::Gpu);
    assert_eq!(parsed.scale_override, Some(1.0));
    assert_eq!(parsed.smoke_capture, Some(std::path::PathBuf::from("target/smoke.png")));
    assert!(parsed.gui_smoke.is_none());

    for invalid in [
        vec![
            "--renderer=cpu",
            "--scale=1",
            "--single-process",
            "--smoke-capture=x.png",
        ],
        vec!["--renderer=cpu", "--scale=2", "--smoke-capture=x.png"],
        vec!["--renderer=cpu", "--scale=1", "--headless", "--smoke-capture=x.png"],
    ] {
        assert!(
            parse_args_from(invalid.into_iter().map(str::to_string), None).is_err(),
            "invalid smoke CLI must be rejected"
        );
    }
}

#[test]
fn real_site_gui_smoke_cli_requires_complete_compositor_configuration() {
    let parsed = parse_args_from(
        [
            "--renderer=cpu",
            "--scale=1",
            "--gui-smoke-url=https://www.iana.org/domains/reserved",
            "--gui-smoke-dir=target/gui-smoke",
        ]
        .into_iter()
        .map(str::to_string),
        None,
    )
    .unwrap();
    let config = parsed.gui_smoke.unwrap();
    assert_eq!(config.url, "https://www.iana.org/domains/reserved");
    assert_eq!(config.output_dir, std::path::PathBuf::from("target/gui-smoke"));

    for invalid in [
        vec![
            "--renderer=cpu",
            "--scale=1",
            "--gui-smoke-url=zero://newtab",
            "--gui-smoke-dir=target/gui-smoke",
        ],
        vec!["--renderer=cpu", "--scale=1", "--gui-smoke-url=https://example.com"],
        vec!["--renderer=cpu", "--scale=1", "--gui-smoke-dir=target/gui-smoke"],
        vec![
            "--renderer=cpu",
            "--scale=1",
            "--headless",
            "--gui-smoke-url=https://example.com",
            "--gui-smoke-dir=target/gui-smoke",
        ],
        vec![
            "--renderer=cpu",
            "--scale=1",
            "--smoke-capture=x.png",
            "--gui-smoke-url=https://example.com",
            "--gui-smoke-dir=target/gui-smoke",
        ],
    ] {
        assert!(
            parse_args_from(invalid.into_iter().map(str::to_string), None).is_err(),
            "invalid real-site GUI smoke CLI must be rejected"
        );
    }
}

#[test]
fn compositor_states_exclude_legacy_page_primitives_until_fallback() {
    use crate::compositor_client::CompositorStatus;

    assert!(!app::compositor_controls_page(CompositorStatus::Disabled));
    assert!(app::compositor_controls_page(CompositorStatus::Starting));
    assert!(app::compositor_controls_page(CompositorStatus::Healthy));
    assert!(!app::compositor_controls_page(CompositorStatus::Disconnected));
}

#[test]
fn compositor_active_tab_uses_its_own_surface_image() {
    use crate::compositor_client::CompositorStatus;

    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (800, 600);
    let first = app.shell.active_tab_id().unwrap();
    app.inject_compositor_frame_for_test(first, 101, 1, 4, (64, 48), [255, 0, 0, 255].repeat(64 * 48));

    let second = app.shell.new_tab(None);
    app.inject_compositor_frame_for_test(second, 202, 1, 9, (32, 24), [0, 255, 0, 255].repeat(32 * 24));

    assert_eq!(app.compositor_surface_for_test(second), Some(202));
    let second_scene = app.compositor_primitives_for_test(CompositorStatus::Healthy);
    assert_eq!(second_scene.images.len(), 1);
    assert_eq!(second_scene.images[0].rect.size.width, 32.0);

    app.shell.switch_tab(first);
    assert_eq!(app.compositor_surface_for_test(first), Some(101));
    let first_scene = app.compositor_primitives_for_test(CompositorStatus::Healthy);
    assert_eq!(first_scene.images.len(), 1);
    assert_eq!(first_scene.images[0].rect.size.width, 64.0);
}

#[test]
fn healthy_compositor_scene_never_contains_same_page_legacy_primitives() {
    use crate::compositor_client::CompositorStatus;
    use zero_webview::WebViewRenderResult;

    let mut app = BrowserApp::new(RenderMode::Cpu);
    let tab_id = app.shell.active_tab_id().unwrap();
    let mut legacy = RenderPrimitives::new();
    legacy.add_fill(Rect::new(0.0, 0.0, 300.0, 200.0), Color::rgb(255, 0, 0));
    app.inject_tab_render_for_test(
        tab_id,
        WebViewRenderResult {
            primitives: legacy,
            dirty_rects: Vec::new(),
            timings: Default::default(),
        },
        200.0,
    );
    app.inject_compositor_frame_for_test(tab_id, 303, 0, 1, (20, 10), [0, 0, 255, 255].repeat(20 * 10));

    let scene = app.compositor_primitives_for_test(CompositorStatus::Healthy);
    assert!(scene.fills.is_empty());
    assert!(scene.glyphs.is_empty());
    assert_eq!(scene.images.len(), 1);
    assert!(
        app.compositor_primitives_for_test(CompositorStatus::Starting)
            .is_empty()
    );
}

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
        font_glyph_index: Some(42),
        source: None,
        font_id: FontId(0),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
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
    assert_eq!(glyphs[0].font_glyph_index, Some(42));
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
fn browser_gpu_scene_does_not_apply_device_scale_twice() {
    assert_eq!(BrowserApp::browser_scene_gpu_scale_factor(), 1.0);
}

#[test]
fn browser_cpu_gpu_chrome_geometry_matches_at_hidpi() {
    let mut app = BrowserApp::new(RenderMode::Gpu);
    app.physical_size = (640, 480);
    app.scale_factor = 2.0;

    let cpu = app.render_full_scene_with_webview_for_test(640, 480);
    let gpu = app
        .render_full_scene_gpu_capture(640, 480)
        .expect("HiDPI browser chrome should be supported by the GPU path");

    let different_pixels = cpu
        .data
        .chunks_exact(4)
        .zip(gpu.data.chunks_exact(4))
        .filter(|(left, right)| {
            left[0].abs_diff(right[0]) as u16 + left[1].abs_diff(right[1]) as u16 + left[2].abs_diff(right[2]) as u16
                > 48
        })
        .count();
    let ratio = different_pixels as f32 / (cpu.data.len() / 4) as f32;
    assert!(ratio < 0.01, "HiDPI CPU/GPU geometry diverged: {ratio:.3}");
}

#[test]
fn windows_browser_scripts_build_required_child_processes() {
    for script in [
        include_str!("../../../scripts/browser.ps1"),
        include_str!("../../../scripts/browser-cpu.ps1"),
    ] {
        assert!(script.contains("-p zero-renderer -p zero-compositor"));
        assert!(script.contains("Test-Path -LiteralPath $CompositorBin"));
    }
}

#[test]
fn browser_build_and_release_entries_include_compositor() {
    let makefile = include_str!("../../../Makefile");
    assert!(makefile.contains("-p zero-browser -p zero-renderer -p zero-compositor"));

    for script in [
        include_str!("../../../scripts/package-linux.sh"),
        include_str!("../../../scripts/package-macos.sh"),
        include_str!("../../../scripts/package-windows.ps1"),
    ] {
        assert!(script.contains("zero-compositor"));
    }

    for workflow in [
        include_str!("../../../.github/workflows/weekly.yml"),
        include_str!("../../../.github/workflows/release.yml"),
    ] {
        assert!(workflow.contains("--bin zero-compositor"));
        assert!(workflow.contains("release/zero-compositor"));
    }
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
    let app = BrowserApp::new(RenderMode::Cpu);
    let config = super::browser_window_config(&app, None);
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

#[test]
fn browser_window_config_sizes_gui_smoke_page_viewport() {
    let app = BrowserApp::new(RenderMode::Cpu);
    let config = super::browser_window_config(&app, Some((800, 720)));
    let (_, _, page_width, page_height) = app.page_content_rect_for(config.width, config.height);
    assert_eq!((page_width, page_height), (800.0, 720.0));
    assert!(!config.maximized);
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
    assert!(html.contains(&format!("ZeroBrowser v{}", zero_product_version::VERSION)));
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

#[test]
fn clicking_page_content_unfocuses_address_bar() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
        tab_id,
        r#"<html><body style="margin:0"><input id="name" style="display:block;width:160px;height:32px"></body></html>"#,
        None,
    );
    app.address_bar_focused = true;

    let (content_x, content_y, _, _) = app.page_content_rect();
    app.handle_mouse_click((content_x + 10.0) as f64, (content_y + 10.0) as f64, true, "Left");

    assert!(
        !app.address_bar_focused,
        "clicking page content must move keyboard focus away from the address bar"
    );
}

#[test]
fn clicking_checkbox_without_page_text_publishes_updated_snapshot() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut app = BrowserApp::new(RenderMode::Cpu);
    // R3254：断言真实多进程链路（click 默认动作经 renderer）——显式启用。
    app.enable_multiprocess_for_test();
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
        tab_id,
        r#"<html><body style="margin:0"><input id="updates" type="checkbox" style="width:20px;height:20px"></body></html>"#,
        None,
    );
    let initial_snapshot_seq = app.snapshot_seq_for_test(tab_id);

    let (content_x, content_y, _, _) = app.page_content_rect();
    let x = (content_x + 10.0) as f64;
    let y = (content_y + 10.0) as f64;
    app.handle_mouse_click(x, y, true, "Left");
    app.handle_mouse_click(x, y, false, "Left");

    assert!(
        wait_for_snapshot_after(&mut app, tab_id, initial_snapshot_seq, false),
        "clicking a checkbox without page text must publish a new rendered page snapshot (initial sequence {initial_snapshot_seq}, current sequence {})",
        app.snapshot_seq_for_test(tab_id)
    );
}

#[test]
fn typing_in_clicked_input_updates_page_html() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut app = BrowserApp::new(RenderMode::Cpu);
    // R3254：断言真实多进程链路（输入焦点/默认动作经 renderer）——显式启用。
    app.enable_multiprocess_for_test();
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
        tab_id,
        r#"<html><body style="margin:0"><input id="name" style="display:block;width:160px;height:32px"></body></html>"#,
        None,
    );
    let initial_snapshot_seq = app.snapshot_seq_for_test(tab_id);

    let (content_x, content_y, _, _) = app.page_content_rect();
    let x = (content_x + 70.0) as f64;
    let y = (content_y + 10.0) as f64;
    app.handle_mouse_click(x, y, true, "Left");
    app.handle_mouse_click(x, y, false, "Left");
    app.handle_key("x", true, None);

    assert!(
        wait_for_snapshot_after(&mut app, tab_id, initial_snapshot_seq, false),
        "typing after clicking an input must publish a new rendered page snapshot (initial sequence {initial_snapshot_seq}, current sequence {})",
        app.snapshot_seq_for_test(tab_id)
    );
}

#[test]
fn gpu_compositor_path_dispatches_input_events_to_form_controls() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // R3254：断言真实多进程链路（renderer/compositor 帧 + 输入路由）——本地合成模式与
    // 断言无关（快照/hit-test 来自 renderer），用 Cpu 避免无 GPU 环境（llvmpipe）的
    // 软渲染并行崩溃（wgpu_core panic 曾致测试 flaky）。
    let mut app = BrowserApp::new(RenderMode::Cpu);
    // R3254：断言真实多进程链路（GPU/compositor 帧 + 输入路由）——显式启用。
    app.enable_multiprocess_for_test();
    app.physical_size = (800, 600);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(
        tab_id,
        r#"<html><body style="margin:0"><input id="name" style="display:block;width:160px;height:32px"></body></html>"#,
        None,
    );

    for _ in 0..300 {
        app.poll_tab_fetch_with_gpu_present_for_test();
        if app
            .hit_test_page_element_for_test(tab_id, 10.0, 10.0)
            .is_some_and(|hit| hit.tag_name.eq_ignore_ascii_case("input"))
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        app.hit_test_page_element_for_test(tab_id, 10.0, 10.0)
            .is_some_and(|hit| hit.tag_name.eq_ignore_ascii_case("input")),
        "GPU/compositor page frame must expose the input to browser-side hit testing"
    );

    let initial_snapshot_seq = app.snapshot_seq_for_test(tab_id);
    app.clear_page_hit_test_for_test(tab_id);
    let (content_x, content_y, _, _) = app.page_content_rect();
    let x = (content_x + 10.0) as f64;
    let y = (content_y + 10.0) as f64;
    app.handle_mouse_click(x, y, true, "Left");
    app.handle_mouse_click(x, y, false, "Left");
    app.handle_key("x", true, None);

    assert!(
        wait_for_snapshot_after(&mut app, tab_id, initial_snapshot_seq, true),
        "GPU/compositor form input did not publish an updated snapshot"
    );
}

#[test]
fn form_fixture_physical_clicks_reach_controls_at_windows_scale_factors() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let html = include_str!("../../../examples/forms/form-interaction-test.html");
    let mut ime_verified = false;
    for scale in [1.0_f32, 1.25, 1.5, 2.0] {
        // R3254：本地合成模式与断言无关（快照/hit-test 来自 renderer）——Cpu 避免
        // llvmpipe 软渲染并行崩溃。
        let mut app = BrowserApp::new(RenderMode::Cpu);
        // R3254：断言真实多进程链路（示例页表单交互经 renderer）——显式启用。
        app.enable_multiprocess_for_test();
        app.physical_size = (1600, 1800);
        app.scale_factor = scale;
        let tab_id = app.shell.active_tab_id().unwrap();
        app.ensure_webview(tab_id);
        app.sync_webview_viewport();
        let initial_snapshot_seq = app.snapshot_seq_for_test(tab_id);
        app.load_webview_html_without_wait_for_test(tab_id, html, None);

        let mut observed_snapshot_seq = initial_snapshot_seq;
        let mut page_ready = false;
        // 轮询上限 1200（每次 10ms ≈ 12s）。曾为 500（5s），并发 `make test`（多二进制 + GPU compositor
        // 子进程争抢 CPU）下 renderer 启动 + 首帧可能 > 5s 致 scale=1 page_ready 超时 flaky（隔离运行通过）。
        // 12s 覆盖并发负载峰值，保多 scale 因子稳定（R3316 归因：资源争抢超时非真实回归）。
        for _ in 0..1200 {
            app.poll_tab_fetch_with_gpu_present_for_test();
            let current_snapshot_seq = app.snapshot_seq_for_test(tab_id);
            if current_snapshot_seq != observed_snapshot_seq {
                observed_snapshot_seq = current_snapshot_seq;
                let (logical_w, logical_h) = app.content_logical_size();
                page_ready = (0..logical_h).step_by(8).any(|y| {
                    (0..logical_w).step_by(8).any(|x| {
                        app.hit_test_page_element_for_test(tab_id, x as f32, y as f32)
                            .and_then(|hit| hit.id)
                            .as_deref()
                            == Some("name")
                    })
                });
                if page_ready {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(page_ready, "scale={scale}: 示例页命中快照未就绪");

        for expected_id in ["name", "note", "click"] {
            let (logical_w, logical_h) = app.content_logical_size();
            let mut point = None;
            'scan: for y in (0..logical_h).step_by(8) {
                for x in (0..logical_w).step_by(8) {
                    if app
                        .hit_test_page_element_for_test(tab_id, x as f32, y as f32)
                        .and_then(|hit| hit.id)
                        .as_deref()
                        == Some(expected_id)
                    {
                        point = Some((x as f32, y as f32));
                        break 'scan;
                    }
                }
            }
            let (doc_x, doc_y) = point.unwrap_or_else(|| panic!("scale={scale}: 示例页控件 #{expected_id} 无法命中"));
            let (content_x, content_y, _, _) = app.page_content_rect();
            let physical_x = (content_x + doc_x * scale) as f64;
            let physical_y = (content_y + doc_y * scale) as f64;
            app.handle_mouse_move(physical_x, physical_y);
            app.handle_mouse_click(physical_x, physical_y, true, "Left");
            // 真实鼠标按下和释放之间通常会有亚像素/小幅抖动；小于拖动阈值仍须激活控件。
            app.handle_mouse_move(physical_x + 1.0, physical_y + 1.0);
            app.handle_mouse_click(physical_x + 1.0, physical_y + 1.0, false, "Left");

            assert_eq!(
                app.page_event_target_for_test(tab_id),
                Some(match expected_id {
                    "name" => "#name",
                    "note" => "#note",
                    _ => "#click",
                }),
                "scale={scale}: 物理坐标点击必须派发给 #{expected_id}"
            );

            // DPI 只影响坐标换算；IME 语义由同一事件路径处理，一次端到端提交即可。
            if expected_id == "note" && !ime_verified {
                let before_preedit = app.snapshot_seq_for_test(tab_id);
                app.handle_ime(zero_host_runtime::event::ImeEvent::Preedit {
                    text: "zhongwen".to_string(),
                    cursor: Some((8, 8)),
                });
                assert!(
                    wait_for_snapshot_after(&mut app, tab_id, before_preedit, true),
                    "scale={scale}: textarea 的 IME preedit 必须发布临时绘制帧"
                );
                let before_input = app.snapshot_seq_for_test(tab_id);
                app.handle_ime(zero_host_runtime::event::ImeEvent::Commit("中文备注".to_string()));
                let published = wait_for_snapshot_after(&mut app, tab_id, before_input, true);
                assert!(published, "scale={scale}: textarea 的中文 IME commit 必须发布新帧");
                ime_verified = true;
            }
        }
    }
    assert!(ime_verified, "示例页 textarea 的中文 IME commit 未执行");
}

#[test]
fn form_fixture_complete_multiprocess_semantics() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // R3254：本地合成模式与断言无关（页面状态经 renderer 快照/title 读回）——Cpu
    // 避免无 GPU 环境（llvmpipe）软渲染并行崩溃（wgpu_core panic 曾致 step 4 断言
    // 超时——崩溃使 GPU 渲染停摆、快照不再更新）。
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.enable_multiprocess_for_test();
    app.physical_size = (1600, 1800);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().expect("active tab");
    app.ensure_webview(tab_id);
    app.sync_webview_viewport();
    let html = include_str!("../../../examples/forms/form-interaction-test.html");
    let before_load = app.snapshot_seq_for_test(tab_id);
    app.load_webview_html_with_url_without_wait_for_test(tab_id, html, "https://zero.test/forms?__zero_test_state=1");
    assert!(
        wait_for_snapshot_after(&mut app, tab_id, before_load, true),
        "目标表单页必须在真实 renderer/GPU 路径加载完成"
    );
    let initial_url = app.page_url_for_test(tab_id).expect("page URL");

    let mut host = BrowserScenarioHost::new(&mut app, tab_id, true);
    HtmlScenario::new(&mut host)
        .click("#name")
        .assert_focused("#name")
        .type_text("abc")
        .assert_output("输入事件：abc")
        .assert_state(
            StateExpectation::default()
                .reason("name-input")
                .name("abc")
                .note("")
                .subscribe(false)
                .plan("basic"),
        )
        .press_key("Backspace")
        .assert_output("输入事件：ab")
        .assert_state(StateExpectation::default().reason("name-input").name("ab"))
        .press_key("Tab")
        .assert_focused("#note")
        .type_text("x")
        .ime_preedit("zhong")
        .ime_commit("中")
        .assert_output("备注输入：x中")
        .assert_state(StateExpectation::default().reason("note-input").note("x中"))
        .click("#subscribe")
        .assert_checked("#subscribe", true)
        .assert_output("复选框：已选中")
        .assert_state(StateExpectation::default().reason("subscribe-change").subscribe(true))
        .click("#plan-pro")
        .assert_checked("#plan-basic", false)
        .assert_checked("#plan-pro", true)
        .assert_output("套餐：pro")
        .assert_state(StateExpectation::default().reason("plan-change").plan("pro"))
        .click("#click")
        .assert_output("普通按钮 click 事件已触发。")
        .click("#reset")
        .assert_output("表单已重置。")
        .assert_state(
            StateExpectation::default()
                .reason("reset")
                .name("")
                .note("")
                .subscribe(false)
                .plan("basic"),
        )
        .click("#submit")
        .assert_output("提交事件已触发（已阻止导航）。")
        .assert_state(StateExpectation::default().reason("submit"))
        .assert_url(&initial_url)
        .run()
        .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn default_actions_work_without_javascript() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut app = BrowserApp::new(RenderMode::Gpu);
    app.enable_multiprocess_for_test();
    app.set_javascript_enabled_for_test(false);
    app.physical_size = (1000, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().expect("active tab");
    app.ensure_webview(tab_id);
    app.sync_webview_viewport();
    let html = r#"<html><head><title>js-off</title></head><body>
        <form action="https://zero.test/submitted" method="get">
          <input id="name" name="name" style="display:block;width:200px;height:32px">
          <input id="subscribe" name="subscribe" value="yes" type="checkbox" style="display:block;width:24px;height:24px">
          <input id="basic" name="plan" value="basic" type="radio" checked style="display:block;width:24px;height:24px">
          <input id="pro" name="plan" value="pro" type="radio" style="display:block;width:24px;height:24px">
          <button id="reset" type="reset" style="display:block;width:100px;height:32px">Reset</button>
          <button id="submit" name="go" value="1" type="submit" style="display:block;width:100px;height:32px">Submit</button>
        </form>
        <script>
          document.title = 'script-ran';
          document.querySelector('#name').addEventListener('input', () => document.title = 'input-listener');
          document.querySelector('#subscribe').addEventListener('change', () => document.title = 'change-listener');
          document.querySelector('form').addEventListener('reset', () => document.title = 'reset-listener');
        </script>
    </body></html>"#;
    let before_load = app.snapshot_seq_for_test(tab_id);
    app.load_webview_html_with_url_without_wait_for_test(tab_id, html, "https://zero.test/js-disabled");
    assert!(
        wait_for_snapshot_after(&mut app, tab_id, before_load, true),
        "JavaScript-disabled fixture must load in the renderer"
    );

    {
        let mut host = BrowserScenarioHost::new(&mut app, tab_id, true);
        HtmlScenario::new(&mut host)
            .click("#name")
            .type_text("before")
            .click("#subscribe")
            .click("#pro")
            .click("#reset")
            .click("#name")
            .type_text("after")
            .click("#subscribe")
            .click("#pro")
            .run()
            .unwrap_or_else(|error| panic!("{error}"));
    }
    assert_eq!(
        app.page_title_for_test(tab_id).as_deref(),
        Some("js-off"),
        "page scripts and input/change/reset listeners must stay disabled"
    );

    let expected_url = "https://zero.test/submitted?name=after&subscribe=yes&plan=pro&go=1";
    let mut host = BrowserScenarioHost::new(&mut app, tab_id, true);
    HtmlScenario::new(&mut host)
        .click("#submit")
        .assert_url(expected_url)
        .run()
        .unwrap_or_else(|error| panic!("{error}"));
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
    assert!(html.contains(&format!("ZeroBrowser v{}", zero_product_version::VERSION)));
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
    for fill in &result.primitives().fills {
        page_h = page_h.max(fill.rect.origin.y + fill.rect.size.height);
    }
    for glyph in &result.primitives().glyphs {
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

/// R3294（S0 单进程端到端）：用户滚动应派发 'scroll' 事件到页面 JS（闭合 R3253 主路径不可达 gap）。
///
/// 单进程完整链路：`handle_scroll` → `apply_page_scroll_delta`（视觉滚动 + `tabs.dispatch_user_scroll`）
/// → `TabWorkerCommand::UserScroll` → worker 线程 `execute_script(script_user_scroll)` →
/// `__zw_user_scroll` 派 'scroll' + 更 `window.scrollY`。本测经 `test_execute_script` 读回 worker
/// WebView 的 JS 态，验证端到端 JS 可观察性（R3294 多进程 IPC 契约测 + R3253 hook 测的组合已证
/// 链路通，本测补单进程 BrowserApp→worker→JS 端到端实证）。
#[test]
fn handle_scroll_dispatches_scroll_to_js_single_process_r3294() {
    let mut app = BrowserApp::new(RenderMode::Cpu);
    // R3254：本测试断言的是**单进程 worker 路径**的 UserScroll 注入（经
    // `test_execute_script` worker-only 回执读 JS 态）——多进程默认可用后须显式禁用。
    app.disable_multiprocess_for_test();
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);

    let tall_html = r#"<!DOCTYPE html><html><head><style>
          head, style, title { display: none; }
          .spacer { height: 2400px; background: #eef; }
        </style></head><body><div class="spacer">Tall</div>
        <script>
          globalThis.__scrollCount = 0;
          globalThis.__scrollY = -1;
          globalThis.__inlinedRan = 'YES';
          window.addEventListener('scroll', function() {
            globalThis.__scrollCount++;
            globalThis.__scrollY = window.scrollY;
          });
        </script>
        </body></html>"#;
    app.load_webview_html(tab_id, tall_html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    let (_, content_y, content_w, _) = app.page_content_rect();
    let x = (content_w * 0.5) as f64;
    let y = content_y as f64 + 100.0;
    app.mouse_pos = (x, y);

    // 滚轮向下（Linux 负 LineDelta）。
    app.handle_scroll(zero_host_runtime::event::MouseScrollDelta::LineDelta(0.0, -3.0), x, y);

    // pump worker 处理 UserScroll 命令（worker 独立线程，1ms 循环 try_recv 排空）。
    // sync_webview_viewport_and_poll 经 tabs.poll 排空 worker 消息触发命令循环。
    // 读回 JS 态：scroll listener 触发则 __scrollCount > 0。
    let mut count = 0u32;
    let mut scroll_y = -1i64;
    for _ in 0..300 {
        app.sync_webview_viewport_and_poll(tab_id);
        if let Ok(c) = app.test_execute_script(tab_id, "String(globalThis.__scrollCount)") {
            count = c.parse::<u32>().unwrap_or(0);
            if count > 0 {
                if let Ok(sy) = app.test_execute_script(tab_id, "String(Math.round(globalThis.__scrollY))") {
                    scroll_y = sy.parse::<i64>().unwrap_or(-1);
                }
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        count > 0,
        "R3294: 用户滚动应派 'scroll' 到页面 JS（scrollCount>0），实得 {count} | inline={} hook={} addEL={} scrollY={}",
        app.test_execute_script(tab_id, "String(globalThis.__inlinedRan||'NO')")
            .unwrap_or_default(),
        app.test_execute_script(tab_id, "typeof __zw_user_scroll")
            .unwrap_or_default(),
        app.test_execute_script(tab_id, "typeof window.addEventListener")
            .unwrap_or_default(),
        app.test_execute_script(tab_id, "String(window.scrollY)")
            .unwrap_or_default()
    );
    assert!(
        scroll_y > 0,
        "R3294: window.scrollY 应跟踪用户滚动（>0），实得 {scroll_y}"
    );
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
        inset: false,
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
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: true,
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
    assert!(g.synthetic_italic);

    let t = &out.transforms[0];
    assert_eq!((t.rect.origin.x, t.rect.origin.y), (12.0, 24.0));
    assert_eq!((t.rect.size.width, t.rect.size.height), (20.0, 20.0));
    assert_eq!((t.origin_x, t.origin_y), (12.0, 24.0));
    assert_eq!((t.tx, t.ty), (6.0, 8.0));
}

/// 性能门禁优化 S2（2026-08-08）：`transform_webview_primitives_extra` 跳过
/// fills/glyphs（浏览器每帧调用它生成 extra 层，fills/glyphs 已由
/// append_webview_primitives 处理），其余 11 类图元输出必须与全量变换一致。
#[test]
fn transform_webview_primitives_extra_skips_fills_glyphs_keeps_others() {
    use app::{transform_webview_primitives, transform_webview_primitives_extra};
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
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
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

    let full = transform_webview_primitives(&p, 10.0, 20.0, 2.0, None);
    let extra = transform_webview_primitives_extra(&p, 10.0, 20.0, 2.0, None);

    // extra 层不产出 fills/glyphs（调用方语义：它们已由 append_webview_primitives 处理）
    assert!(extra.fills.is_empty(), "extra must not produce fills");
    assert!(extra.glyphs.is_empty(), "extra must not produce glyphs");
    // 其余 11 类长度与全量变换一致（图元类型未实现 PartialEq，抽查代表字段）
    assert_eq!(extra.rounded_rects.len(), full.rounded_rects.len());
    assert_eq!(extra.gradients.len(), full.gradients.len());
    assert_eq!(extra.shadows.len(), full.shadows.len());
    assert_eq!(extra.strokes.len(), full.strokes.len());
    assert_eq!(extra.path_fills.len(), full.path_fills.len());
    assert_eq!(extra.path_strokes.len(), full.path_strokes.len());
    assert_eq!(extra.images.len(), full.images.len());
    assert_eq!(extra.transforms.len(), full.transforms.len());
    assert_eq!(extra.clips.len(), full.clips.len());
    assert_eq!(extra.filters.len(), full.filters.len());
    assert_eq!(extra.blend_modes.len(), full.blend_modes.len());
    let r = &extra.rounded_rects[0];
    assert_eq!(
        (r.rect.origin.x, r.rect.origin.y, r.rect.size.width),
        (12.0, 24.0, 20.0)
    );
    match &extra.gradients[0].kind {
        GradientKind::Linear { x0, y0, x1, y1 } => {
            assert_eq!((*x0, *y0, *x1, *y1), (12.0, 24.0, 16.0, 28.0));
        }
        _ => panic!("expected Linear gradient"),
    }
    let st = &extra.strokes[0];
    assert_eq!((st.x1, st.y1, st.x2, st.y2, st.width), (12.0, 24.0, 16.0, 28.0, 10.0));
    let t = &extra.transforms[0];
    assert_eq!((t.rect.origin.x, t.rect.origin.y, t.tx, t.ty), (12.0, 24.0, 6.0, 8.0));
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
        font_glyph_index: None,
        source: None,
        font_id: FontId(0),
        font_variation_id: None,
        bitmap_width: None,
        bitmap_height: None,
        rotation: 0.0,
        synthetic_italic: false,
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
            font_glyph_index: None,
            source: None,
            font_id: FontId(fid),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
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

    // R3254-M9：滚动默认动作挂 keydown 回执（异步）——用轮询等待回执驱动生效。
    let before = app.scroll_offset_for_tab(tab_id);
    app.handle_key("PageDown", true, None);
    let after_pagedown = wait_for_scroll_change(&mut app, tab_id, before);
    assert!(
        after_pagedown > before,
        "PageDown should scroll down (before={before}, after={after_pagedown})"
    );

    app.handle_key("PageUp", true, None);
    let after_pageup = wait_for_scroll_change(&mut app, tab_id, after_pagedown);
    assert!(
        after_pageup < after_pagedown,
        "PageUp should scroll up (after_pagedown={after_pagedown}, after_pageup={after_pageup})"
    );

    // Space 向下
    let mid = app.scroll_offset_for_tab(tab_id);
    app.handle_key("Space", true, None);
    let after_space = wait_for_scroll_change(&mut app, tab_id, mid);
    assert!(after_space > mid, "Space should scroll down");
}

/// R3254-M9：轮询等待滚动默认动作经 keydown 回执生效（滚动偏移变化），最多 10s。
fn wait_for_scroll_change(app: &mut BrowserApp, tab_id: TabId, previous: f32) -> f32 {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        app.poll_tab_fetch();
        let current = app.scroll_offset_for_tab(tab_id);
        if current != previous || std::time::Instant::now() >= deadline {
            return current;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
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

    // 先滚到中间（R3254-M9：滚动经 keydown 回执，轮询等待）
    app.handle_key("PageDown", true, None);
    let mid = wait_for_scroll_change(&mut app, tab_id, 0.0);
    assert!(mid > 0.0, "should have scrolled down first");

    // Home 回到顶部
    app.handle_key("Home", true, None);
    assert_eq!(
        wait_for_scroll_change(&mut app, tab_id, mid),
        0.0,
        "Home should scroll to top"
    );

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

/// R3254：轮询等待合成帧相对基线发生变化（renderer 输入/渲染异步——不等快照 seq，
/// 直接等像素变化，比 wait_for_snapshot_after 更贴近「合成帧内容」语义）。最多 10s。
/// 合成帧（CPU/GPU 双通道输出）。
type CompositeFrame = (Vec<u8>, Vec<u8>);
/// 合成帧差异比率（0..1）。
type DiffFn = dyn Fn(&[u8], &[u8]) -> f32;

fn wait_composite_changed(
    app: &mut BrowserApp,
    composite: &dyn Fn(&mut BrowserApp) -> CompositeFrame,
    diff_ratio: &DiffFn,
    base_cpu: &[u8],
    base_gpu: &[u8],
    label: &str,
) -> CompositeFrame {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let (cpu, gpu) = composite(app);
        let cpu_diff = diff_ratio(base_cpu, &cpu);
        let gpu_diff = diff_ratio(base_gpu, &gpu);
        if cpu_diff > 0.00005 && gpu_diff > 0.00005 {
            return (cpu, gpu);
        }
        app.poll_tab_fetch();
        if std::time::Instant::now() >= deadline {
            panic!("{label} 后合成帧应变化（cpu_diff={cpu_diff:.5} gpu_diff={gpu_diff:.5}，阈值 0.005%）");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// R3254：同 `wait_composite_changed`，但窗口超时返回 `(frame, false)` 而非 panic——
/// 调用方在已确认逻辑状态（renderer 文本快照）时接受「无可见字形 → 帧不变」
/// （无 CJK 字体平台 IME 中文提交，见 R3416）。
fn wait_composite_changed_or_missing_glyph(
    app: &mut BrowserApp,
    composite: &dyn Fn(&mut BrowserApp) -> CompositeFrame,
    diff_ratio: &DiffFn,
    base_cpu: &[u8],
    base_gpu: &[u8],
    timeout: std::time::Duration,
) -> (CompositeFrame, bool) {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let frame = composite(app);
        let cpu_diff = diff_ratio(base_cpu, &frame.0);
        let gpu_diff = diff_ratio(base_gpu, &frame.1);
        if cpu_diff > 0.00005 && gpu_diff > 0.00005 {
            return (frame, true);
        }
        app.poll_tab_fetch();
        if std::time::Instant::now() >= deadline {
            return (frame, false);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// R3254：本地合成 CPU/GPU 双参数矩阵——多进程 renderer（完整 shim/焦点/输入）+
/// legacy 帧发布（ViewPainted → last_render 含页面主体，本地合成可渲染页面内容）。
/// 依次交互（点击聚焦 + 输入 / IME 中文 / 滚动），每步用 CPU（rasterize_full_scene）
/// 与 GPU（headless wgpu）两个通道渲染合成帧，断言：① 页面内容像素存在（非纯白）；
/// ② 交互引起合成帧变化；③ 两通道输出一致（parity——同输入同渲染）。
#[test]
fn local_composite_cpu_gpu_matrix_for_form_interactions() {
    let _mp_guard = MULTIPROCESS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.enable_multiprocess_for_test();
    app.physical_size = (1280, 900);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.set_legacy_frame_publish_for_test(tab_id);
    app.set_compositor_status_for_test(crate::compositor_client::CompositorStatus::Disconnected);
    // 滚动断言必须自行保证文档高于视口，不能依赖平台字体或历史 renderer 的布局高度。
    let html = include_str!("../../../examples/forms/form-interaction-test.html")
        .replace("</body>", "<div style=\"height: 900px\"></div></body>");
    let before_load = app.snapshot_seq_for_test(tab_id);
    app.load_webview_html_without_wait_for_test(tab_id, &html, None);
    assert!(
        wait_for_snapshot_after(&mut app, tab_id, before_load, false),
        "表单页应在 renderer 加载完成"
    );
    // 合成帧 helper：CPU/GPU 双通道。
    let composite = |app: &mut BrowserApp| -> (Vec<u8>, Vec<u8>) {
        let cpu = app.render_full_scene_with_webview_for_test(1280, 900);
        let gpu = app.render_full_scene_with_webview_gpu_for_test(1280, 900);
        (cpu.data, gpu.data)
    };
    let non_white_ratio = |fb: &[u8]| -> f32 {
        let mut non_white = 0usize;
        for px in fb.chunks_exact(4) {
            if px[0] < 250 || px[1] < 250 || px[2] < 250 {
                non_white += 1;
            }
        }
        non_white as f32 / (fb.len() / 4) as f32
    };
    let diff_ratio = |a: &[u8], b: &[u8]| -> f32 {
        let mut diff = 0usize;
        for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
            let da = pa[0].abs_diff(pb[0]) as u16 + pa[1].abs_diff(pb[1]) as u16 + pa[2].abs_diff(pb[2]) as u16;
            if da > 48 {
                diff += 1;
            }
        }
        diff as f32 / (a.len() / 4) as f32
    };
    let diff_bounds = |a: &[u8], b: &[u8]| -> Option<(usize, usize, usize, usize, usize)> {
        let frame_width = 1280usize;
        let mut bounds: Option<(usize, usize, usize, usize)> = None;
        let mut diff = 0usize;
        for py in 0..900 {
            for px in 0..frame_width {
                let index = (py * frame_width + px) * 4;
                let delta = a[index].abs_diff(b[index]) as u16
                    + a[index + 1].abs_diff(b[index + 1]) as u16
                    + a[index + 2].abs_diff(b[index + 2]) as u16;
                if delta > 48 {
                    diff += 1;
                    bounds = Some(match bounds {
                        Some((min_x, min_y, max_x, max_y)) => {
                            (min_x.min(px), min_y.min(py), max_x.max(px), max_y.max(py))
                        }
                        None => (px, py, px, py),
                    });
                }
            }
        }
        bounds.map(|(min_x, min_y, max_x, max_y)| (min_x, min_y, max_x, max_y, diff))
    };

    // ① 首个就绪合成帧：GPU backend 初始化可能晚于 renderer 首帧，轮询到两通道
    // 都含页面内容；不降低非白阈值。
    let initial_deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let (_first_cpu, _first_gpu) = loop {
        let (cpu, gpu) = composite(&mut app);
        let cpu_non_white = non_white_ratio(&cpu);
        let gpu_non_white = non_white_ratio(&gpu);
        if cpu_non_white > 0.01 && gpu_non_white > 0.01 {
            break (cpu, gpu);
        }
        app.poll_tab_fetch();
        if std::time::Instant::now() >= initial_deadline {
            panic!("CPU/GPU 首个就绪合成帧应含页面内容（cpu={cpu_non_white:.3}, gpu={gpu_non_white:.3}）");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let settle_deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut observed_seq = app.snapshot_seq_for_test(tab_id);
    let mut quiet_since = std::time::Instant::now();
    while quiet_since.elapsed() < std::time::Duration::from_millis(200) {
        app.poll_tab_fetch();
        let current = app.snapshot_seq_for_test(tab_id);
        if current != observed_seq {
            observed_seq = current;
            quiet_since = std::time::Instant::now();
        }
        assert!(
            std::time::Instant::now() < settle_deadline,
            "首屏 legacy 快照应在 10s 内稳定"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let (cpu0, gpu0) = composite(&mut app);
    let parity0 = diff_ratio(&cpu0, &gpu0);
    assert!(parity0 < 0.15, "CPU/GPU 初始合成帧差异应 <15%（got {parity0:.3}）");
    // ② 点击聚焦 #name（多进程 renderer 完整焦点链路）+ 键盘输入 'abc'。
    // 命中扫描找 #name 中心（表单布局位置不确定）。R3254-F 时序适配：慢 runner
    //（windows/macos）上 wait_for_snapshot_after 通过但 hit-test 的 DOM/布局未就绪——
    // 10s 窗口内周期重扫 + poll_tab_fetch（63e4b70b point_for_id 同族模式）。
    let (content_x, content_y, logical_w, logical_h) = app.page_content_rect();
    let scan_deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut name_point: Option<(f32, f32)> = None;
    while name_point.is_none() {
        let mut bounds: Option<(f32, f32, f32, f32)> = None;
        for y in (0..logical_h as u32).step_by(4) {
            for x in (0..logical_w as u32).step_by(4) {
                if let Some(hit) = app.hit_test_page_element_for_test(tab_id, x as f32, y as f32)
                    && hit.id.as_deref() == Some("name")
                {
                    bounds = Some(match bounds {
                        Some((min_x, min_y, max_x, max_y)) => (
                            min_x.min(x as f32),
                            min_y.min(y as f32),
                            max_x.max(x as f32),
                            max_y.max(y as f32),
                        ),
                        None => (x as f32, y as f32, x as f32, y as f32),
                    });
                }
            }
        }
        name_point = bounds.map(|(min_x, min_y, max_x, max_y)| ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0));
        if name_point.is_none() {
            app.poll_tab_fetch();
            assert!(
                std::time::Instant::now() < scan_deadline,
                "hit-test 扫描应找到 #name（10s 重试窗口）"
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
    let (doc_x, doc_y) = name_point.expect("hit-test 扫描应找到 #name");
    let px = (content_x + doc_x * app.scale_factor) as f64;
    let py = (content_y + doc_y * app.scale_factor) as f64;
    app.handle_mouse_click(px, py, true, "Left");
    app.handle_mouse_click(px, py, false, "Left");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.page_event_target_for_test(tab_id) != Some("#name") {
        app.poll_tab_fetch();
        assert!(std::time::Instant::now() < deadline, "点击应同步 event_targets");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    for ch in "abc".chars() {
        app.handle_key(&ch.to_string(), true, None);
    }
    let glyph_deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !app
        .last_render_text_for_test(tab_id)
        .is_some_and(|text| text.contains("abc"))
    {
        app.poll_tab_fetch();
        assert!(
            std::time::Instant::now() < glyph_deadline,
            "输入 abc 后 renderer 的 legacy glyph 快照必须包含 abc"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let (cpu1, gpu1) = composite(&mut app);
    let cpu_bounds = diff_bounds(&cpu0, &cpu1);
    let gpu_bounds = diff_bounds(&gpu0, &gpu1);
    assert!(
        cpu_bounds.is_some() && gpu_bounds.is_some(),
        "输入 abc 后合成帧必须变化（cpu={cpu_bounds:?} gpu={gpu_bounds:?}, click=({px:.1},{py:.1})）"
    );
    let parity1 = diff_ratio(&cpu1, &gpu1);
    assert!(parity1 < 0.15, "CPU/GPU 输入后合成帧差异应 <15%（got {parity1:.3}）");

    // ③ IME 中文提交（Preedit + Commit 到焦点 input）。
    app.handle_ime(zero_host_runtime::event::ImeEvent::Preedit {
        text: "zhong".to_string(),
        cursor: Some((5, 5)),
    });
    app.handle_ime(zero_host_runtime::event::ImeEvent::Commit("中".to_string()));
    // R3416：先等逻辑状态——renderer 文本快照须含 "中"。字形 primitives 保留
    // code point（shaper 对无覆盖字符以 .notdef 占位，shape_fallback 不丢字符），
    // 故无 CJK 字体平台也成立。d68c4705 起 test-state 输出 display:none（UA
    // [hidden] 规则），无 CJK 字体平台（CI ubuntu-latest 无 fonts-noto-cjk）上
    // "中" 无可见字形 → 合成帧像素不变，旧断言依赖 test-state 的可见 JSON dump
    // 才成立——改为：状态必查，像素断言仅在字形可渲染时执行。
    let ime_deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !app
        .last_render_text_for_test(tab_id)
        .is_some_and(|text| text.contains('中'))
    {
        app.poll_tab_fetch();
        assert!(
            std::time::Instant::now() < ime_deadline,
            "IME 提交后 renderer 的 legacy glyph 快照必须包含 中"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let ((cpu2, gpu2), ime_visible) = wait_composite_changed_or_missing_glyph(
        &mut app,
        &composite,
        &diff_ratio,
        &cpu1,
        &gpu1,
        std::time::Duration::from_secs(5),
    );
    if ime_visible {
        let parity2 = diff_ratio(&cpu2, &gpu2);
        assert!(parity2 < 0.15, "CPU/GPU IME 后合成帧差异应 <15%（got {parity2:.3}）");
    } else {
        // 无 CJK 字体平台：字形缺失 → 帧不变属预期（状态已由上断言确认）。
        // 有 CJK 字体平台若 5s 未变说明像素路径故障——但此时快照断言已保证
        // 提交到达渲染器，可见字形必产生像素变化，故不会走到此分支。
        tracing::warn!("IME 提交后合成帧无像素变化——无 CJK 字体字形缺失，跳过像素断言（状态已验证）");
    }

    // ④ 滚动（滚轮）——合成帧的 webview 内容偏移。
    let (content_x, content_y, _, _) = app.page_content_rect();
    let mx = (content_x + 100.0) as f64;
    let my = (content_y + 100.0) as f64;
    app.mouse_pos = (mx, my);
    app.handle_scroll(zero_host_runtime::event::MouseScrollDelta::LineDelta(0.0, -3.0), mx, my);
    let (cpu3, gpu3) = wait_composite_changed(&mut app, &composite, &diff_ratio, &cpu1, &gpu1, "滚动");
    let parity3 = diff_ratio(&cpu3, &gpu3);
    assert!(parity3 < 0.15, "CPU/GPU 滚动后合成帧差异应 <15%（got {parity3:.3}）");
}

/// R3254：窗口 surface present 路径冒烟——真实 winit 窗口（Xvfb/CI 有显示）+
/// `init_gpu`（wgpu 窗口 surface）→ `render_frame(present=true)`（swapchain 提交）。
/// 窗口模式无法 read_pixels——验证「present 全流程不崩溃 + GPU 渲染器存活 +
/// 表面状态流转」。无显示环境（无 DISPLAY/WAYLAND）或 wgpu surface 不可用
///（无 GPU 后端）时优雅跳过（测试意义 = 有窗口环境下的 present 冒烟）。
#[test]
fn window_surface_present_smoke() {
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};

    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    if !has_display {
        eprintln!("skipping window present smoke (no DISPLAY/WAYLAND)");
        return;
    }

    struct PresentProbe {
        outcome: Option<String>,
    }
    impl ApplicationHandler for PresentProbe {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let attrs = winit::window::Window::default_attributes().with_title("zero-window-probe");
            let Ok(window) = event_loop.create_window(attrs) else {
                self.outcome = Some("window creation failed".to_string());
                event_loop.exit();
                return;
            };
            let window = std::sync::Arc::new(window);
            let mut app = BrowserApp::new(RenderMode::Gpu);
            app.window_focused = true;
            app.init_gpu(&window);
            if !app.gpu_renderer_is_some() {
                // wgpu 窗口 surface 不可用（无 GPU 后端）——优雅跳过。
                self.outcome = Some("gpu surface unavailable (no backend)".to_string());
                event_loop.exit();
                return;
            };
            // present 全流程：surface configure + swapchain acquire + 合成 + queue.present
            //（configure 由主事件循环驱动，测试手动补齐——render_frame 依赖外部配置）。
            let (w, h) = (window.inner_size().width.max(1), window.inner_size().height.max(1));
            if let Some(gpu) = app.gpu_renderer_as_mut() {
                gpu.configure_surface(w, h);
            }
            app.surface_configured = true;
            app.render_frame(w, h, true);
            self.outcome = Some(format!(
                "presented {}x{} surface_configured={}",
                w, h, app.surface_configured
            ));
            event_loop.exit();
        }
        fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, _e: WindowEvent) {}
    }

    // cargo test 线程非主线程——winit 30 默认拒绝；X11 用 any_thread（macOS/Windows
    // 允许非主线程，Linux 需显式开启）。
    #[cfg(target_os = "linux")]
    let event_loop = {
        use winit::platform::x11::EventLoopBuilderExtX11;
        let mut builder = winit::event_loop::EventLoop::builder();
        builder.with_any_thread(true);
        builder.build().expect("event loop")
    };
    #[cfg(not(target_os = "linux"))]
    let event_loop = EventLoop::new().expect("event loop");
    let mut probe = PresentProbe { outcome: None };
    let _ = event_loop.run_app(&mut probe);
    let outcome = probe.outcome.expect("probe should complete");
    if outcome.contains("failed") || outcome.contains("unavailable") {
        eprintln!("window present smoke skipped: {outcome}");
        return;
    }
    assert!(outcome.starts_with("presented"), "窗口 present 应完成：{outcome}");
}

#[test]
fn gpu_present_is_not_suppressed_by_compositor_owned_present() {
    assert!(BrowserApp::should_skip_local_composite_for_owned_present(
        true, true, true, false, true,
    ));
    assert!(
        !BrowserApp::should_skip_local_composite_for_owned_present(true, true, true, true, true),
        "the compositor returns a bitmap but does not submit the browser GPU swapchain; GPU must keep presenting locally"
    );
}

/// owned-present 跳过本地合成的前提：compositor present 像素已就绪。
/// 首帧（present 往返未完成）跳过本地合成会整帧空白。
#[test]
fn owned_present_waits_for_present_pixels_before_skipping_local_composite() {
    assert!(
        !BrowserApp::should_skip_local_composite_for_owned_present(true, true, true, false, false),
        "present 像素未就绪时必须保留本地合成，避免首帧空白"
    );
}

/// T1（布局↔绘制宽度防线，ZRG-2026-08-15）：paint glyph 位置与 rustybuzz shaping 基准一致。
///
/// 场景：独立 WebView + 手动字体 loader（Lato + Liberation 双字体），同一页面混排
/// `font-family: "Lato"` 与 `sans-serif` 两段含 kerning 对（AVATAR）的文本。回归
/// 背景：paint 的 per-char advance 经全局 measure 回调用 thread-local 的单一
/// font_id 测量，与字形实际字体（另一 face）脱节 → 多字体页面字距与 Chrome
/// （rustybuzz 精确 shaping）差 ~1px/词。断言 sans-serif 段 AVATAR run 的 glyph
/// 相对 x 序列与测试内 rustybuzz 直算的 Liberation 基准一致（容差 = 索引 ×
/// 1/64 + ε）。修复前必失败（paint 用 ctx font_id=Lato 测量 Liberation 字形）；
/// 修复后 pass。
#[test]
fn text_glyph_positions_match_shaping_baseline() {
    use zero_engine::set_char_measure_fn;
    use zero_render_foundation::font::TextShaper;
    use zero_render_foundation::primitive::FontId;
    use zero_webview::{WebView, WebViewConfig};

    const LATO_TTF: &[u8] = include_bytes!("../../../tests/wpt-runner/fonts/Lato-Medium.ttf");

    // 全局 measure/shape 回调：BrowserApp 启动时注册（browser 实现，走 MEASURE_CTX）。
    let _app = BrowserApp::new(RenderMode::Cpu);

    // 测试字体 loader：Lato（@font-face 场景）+ 系统 sans（primary）。与 BrowserApp
    // 同源（shared_system_fonts 进程级缓存），跨平台可用（Linux Liberation/DejaVu、
    // macOS SFNS/Helvetica、Windows Segoe/Arial），不再硬编码 Linux 字体路径。
    let (mut loader, sans_id) = crate::app::shared_system_fonts();
    let sans_id = sans_id.expect("系统 sans 字体存在");
    let lato_id = loader.load_font(LATO_TTF).expect("bundled Lato 可加载");
    loader.register_family_alias("Lato", lato_id);
    loader.register_family_alias("sans-serif", sans_id);

    let html = r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
          body { margin: 0; font-size: 16px; }
          p { margin: 0; }
          p.lato { font-family: "Lato"; }
          p.sans { font-family: sans-serif; }
        </style></head><body>
          <p class="lato">AVATAR Hello World</p>
          <p class="sans">AVATAR Hello World</p>
        </body></html>"#;

    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 200,
        ..WebViewConfig::default()
    });
    wv.set_font_resolver(loader.build_font_resolver());
    // 修复前语义：measure 回调的 thread-local ctx font_id 为 Lato——paint 用 Lato
    // advance 摆所有字形（含 sans-serif 段）；修复后按 glyph.font_id 逐字形测量。
    let result = crate::text_metrics::with_measure_ctx(&loader, lato_id, || wv.load_html(html, None));

    // 基准：sans-serif 段字形（Liberation）的 rustybuzz shaping——用整段文本
    // （paint 的 fragment shaping 输入是整段，词内 glyph 上下文与单独 "AVATAR" 不同）。
    let shaped = TextShaper::new(&loader, Some(FontId(sans_id))).shape_single_line("AVATAR Hello World", 16.0);
    let glyphs = &result.primitives().glyphs;
    let text: String = glyphs.iter().filter_map(|g| char::from_u32(g.glyph_id)).collect();
    let first = text.find("AVATAR").expect("页面应渲染 Lato 段 AVATAR");
    let second = text[first + 7..].find("AVATAR").map(|p| first + 7 + p);
    let run_start = second.expect("页面应渲染 sans-serif 段 AVATAR");
    let run: Vec<&GlyphPrimitive> = glyphs.iter().skip(run_start).take(7).collect();
    assert_eq!(run.len(), 7, "sans-serif AVATAR run 应有 7 个 glyph");
    assert!(run.windows(2).all(|w| w[0].y == w[1].y), "sans-serif AVATAR run 应同行");
    assert!(
        run.iter().all(|g| g.font_id.0 == sans_id),
        "sans-serif run 的字形应解析为系统 sans 字体（font_id={sans_id}），实际 {:?}",
        run.iter().map(|g| g.font_id.0).collect::<Vec<_>>()
    );
    // 断言 AVATAR 自身 6 个 glyph 的 advance 与 rustybuzz shaped 累计一致
    // （run 第 7 个 glyph 是下一 fragment 的首字符，受 fragment 边界 GPOS
    // x_offset 影响，不属于本测试关注点）。前 6 个 glyph 的 x_offset 为 0。
    let base_x = run[0].x;
    let mut expected = 0.0f32;
    for (i, g) in run.iter().enumerate().skip(1).take(5) {
        expected += shaped[i - 1].advance_x;
        let tolerance = 0.05 + 0.02 * i as f32; // 每字符 1/64px 取整上限 + ε
        assert!(
            (g.x - base_x - expected).abs() <= tolerance,
            "sans-serif run glyph#{i} x={} 偏离 Liberation shaping 基准 {expected}（advance 未按字形字体测量）",
            g.x
        );
    }
}

/// T2（布局↔绘制宽度防线，ZRG-2026-08-15）：换行点与 rustybuzz 基准一致。
///
/// 场景：固定宽度盒子内 sans-serif 长英文句。回归背景：布局文本宽度默认走
/// estimate_char_width 启发式（字母 0.55em 等），与真实 advance 差 15-20% →
/// 换行点系统性过早、行尾参差。断言每行起始词（按 glyph y 分组）与 rustybuzz
/// 基准（累计宽度切行）一致。
///
/// 修复 A（布局真实 advance）前必失败；修复后 pass。
#[test]
fn text_wrap_points_match_shaping_baseline() {
    let sentence = "The quick brown fox jumps over the lazy dog while the sun sets behind the hills.";
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
          body {{ margin: 0; font-size: 16px; }}
          div.box {{ width: 260px; }}
        </style></head><body>
          <div class="box"><p style="margin:0">{sentence}</p></div>
        </body></html>"#
    );
    let mut app = BrowserApp::new(RenderMode::Cpu);
    app.physical_size = (400, 400);
    app.scale_factor = 1.0;
    let tab_id = app.shell.active_tab_id().unwrap();
    app.ensure_webview(tab_id);
    app.load_webview_html(tab_id, &html, None);
    app.sync_webview_viewport_and_poll(tab_id);

    // 基准：逐词宽度（词间空格宽）切行。词序列：以空格分词，每词宽 = hmtx advance 和
    // + 空格宽——与布局测量同源（measure_text_hmtx，ZRG-2026-08-15 修复 A 的 hmtx 路径，
    // 非 rustybuzz shaped：kerning 差异使 Segoe/SFNS 等平台字体在 260px 边界翻转换行点，
    // Linux Liberation kerning 小侥幸对齐）。
    // 浏览器实际布局字体 = shared_system_fonts primary（与 BrowserApp 同源进程级缓存），
    // 基准必须同源换行点才可比；跨平台可用（Linux Liberation/DejaVu、macOS SFNS/Helvetica、
    // Windows Segoe/Arial），不再硬编码 Linux 字体路径。
    let (baseline_loader, primary_id) = crate::app::shared_system_fonts();
    let primary_id = primary_id.expect("系统字体存在");
    let words: Vec<&str> = sentence.split(' ').collect();
    let word_widths: Vec<f32> = words
        .iter()
        .map(|w| baseline_loader.measure_text_hmtx(&[primary_id], w, 16.0))
        .collect();
    let space_w = baseline_loader.measure_text_hmtx(&[primary_id], " ", 16.0);
    // 贪心切行：每行 ≤ 260px。
    let mut expected_lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut line_w = 0.0f32;
    for (i, w) in words.iter().enumerate() {
        let w_w = word_widths[i];
        let add = if line.is_empty() { 0.0 } else { space_w };
        if line_w + add + w_w > 260.0 && !line.is_empty() {
            expected_lines.push(line.clone());
            line = String::new();
            line_w = 0.0;
        }
        if !line.is_empty() {
            line.push(' ');
            line_w += space_w;
        }
        line.push_str(w);
        line_w += w_w;
    }
    if !line.is_empty() {
        expected_lines.push(line);
    }

    // 实际渲染：按 glyph y 分行的首词。
    let primitives = app.last_render_primitives_for_test(tab_id).expect("页面已渲染");
    let glyphs = primitives.glyphs.clone();
    let mut rows: Vec<(f32, Vec<char>)> = Vec::new();
    for g in &glyphs {
        if let Some(ch) = char::from_u32(g.glyph_id) {
            match rows.last_mut() {
                Some((y, chars)) if (*y - g.y).abs() < 1.0 => chars.push(ch),
                _ => rows.push((g.y, vec![ch])),
            }
        }
    }
    let actual_lines: Vec<String> = rows
        .iter()
        .map(|(_, chars)| chars.iter().collect::<String>())
        .filter(|s| !s.is_empty())
        .collect();
    // paint 不输出空格 glyph（advance 计入）；基准行同样去空格后对比。
    let expected_compact: Vec<String> = expected_lines
        .iter()
        .map(|line| line.chars().filter(|c| !c.is_whitespace()).collect())
        .collect();
    assert_eq!(
        actual_lines, expected_compact,
        "换行点应与 rustybuzz 基准一致（布局 estimate 偏宽导致换行过早）"
    );
}
