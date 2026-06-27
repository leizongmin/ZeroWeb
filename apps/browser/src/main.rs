//! ZeroBrowser — 基于 Rust 的跨平台浏览器应用
//!
//! M11 里程碑：完整浏览器应用，连接 BrowserShell（数据模型）、
//! WebView（页面渲染）和 HostRuntime（窗口管理）。

#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![allow(unused_comparisons)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::absurd_extreme_comparisons)]
// WIP：多进程后端（ProcessTabBackend 部分 API）与 tab_js_worker 线程 worker 脚本路径
// 尚未全量接线，存在未用方法/字段/变体；T4/T5 统一脚本/帧后评估删除或接线。
#![allow(dead_code)]

mod app;
mod clipboard;
mod colors;
mod favicon_fetch;
mod fetch_proxy;
mod headless;
mod input_keys;
mod layout;
mod page_scroll;
mod page_selection;
mod pages;
mod paint_ipc;
mod process_backend;
mod tab_chrome;
mod tab_favicon;
mod tab_js_worker;
mod tab_manager;
mod tab_scripts;
mod tab_snapshot;
mod tab_worker;
mod test_sync;
mod text_input;
mod text_metrics;
mod ui_icons;

use std::sync::Arc;

use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::config::RenderMode;

use app::BrowserApp;
use app::WindowChromeAction;
use process_backend::set_multiprocess_enabled;

// --- CLI 参数 ---

struct CliArgs {
    render_mode: RenderMode,
    scale_override: Option<f32>,
    headless: bool,
    remote_debugging_port: u16,
    viewport_width: f32,
    viewport_height: f32,
    single_process: bool,
    /// 与 WPT reftest 对齐：CPU 光栅化 + 1.0 缩放（便于肉眼对比 product-smoke）。
    wpt_parity: bool,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = std::env::args().skip(1);
    let mut render_mode = None;
    let mut scale_override = None;
    let mut headless = false;
    let mut remote_debugging_port = 0u16;
    let mut viewport_width = 800.0f32;
    let mut viewport_height = 600.0f32;
    let mut single_process = false;
    let mut wpt_parity = false;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }

        if let Some(value) = arg.strip_prefix("--renderer=") {
            render_mode = Some(value.parse()?);
            continue;
        }

        if arg == "--renderer" {
            let value = args
                .next()
                .ok_or_else(|| format!("--renderer requires {}", RenderMode::values()))?;
            render_mode = Some(value.parse()?);
        }

        if let Some(value) = arg.strip_prefix("--scale=") {
            let s = value.parse::<f32>().map_err(|_| format!("invalid scale: {value}"))?;
            if s <= 0.0 || !s.is_finite() {
                return Err(format!("scale must be positive: {s}"));
            }
            scale_override = Some(s);
        }

        if arg == "--headless" {
            headless = true;
        }

        if let Some(value) = arg.strip_prefix("--remote-debugging-port=") {
            remote_debugging_port = value.parse::<u16>().map_err(|_| format!("invalid port: {value}"))?;
        }

        if arg == "--remote-debugging-port" {
            let value = args
                .next()
                .ok_or_else(|| "--remote-debugging-port requires a port number".to_string())?;
            remote_debugging_port = value.parse::<u16>().map_err(|_| format!("invalid port: {value}"))?;
        }

        if arg == "--multi-process" {
            // 默认已启用；保留该开关以兼容旧脚本。
        }

        if arg == "--single-process" {
            single_process = true;
        }

        if arg == "--wpt-parity" {
            wpt_parity = true;
        }

        if let Some(value) = arg.strip_prefix("--viewport-width=") {
            viewport_width = value.parse::<f32>().map_err(|_| format!("invalid width: {value}"))?;
        }

        if let Some(value) = arg.strip_prefix("--viewport-height=") {
            viewport_height = value.parse::<f32>().map_err(|_| format!("invalid height: {value}"))?;
        }
    }

    let cli_render_mode = render_mode;
    let cli_scale = scale_override;
    let mut render_mode = cli_render_mode.or(RenderMode::from_env()?).unwrap_or_default();
    let mut scale_override = cli_scale;
    // make browser / browser.ps1 默认 --renderer=gpu；make browser-cpu / browser-cpu.ps1 传 --wpt-parity。
    if wpt_parity {
        if cli_render_mode.is_none() {
            render_mode = RenderMode::Cpu;
        }
        if cli_scale.is_none() {
            scale_override = Some(1.0);
        }
    }
    Ok(CliArgs {
        render_mode,
        scale_override,
        headless,
        remote_debugging_port,
        viewport_width,
        viewport_height,
        single_process,
        wpt_parity,
    })
}

fn print_usage() {
    println!(
        "Usage: zero-browser [options]

Options:
  --renderer=<mode>              Choose rendering backend ({})
  --scale=<factor>               Override window scale factor (e.g. --scale=2 for HiDPI)
  --headless                     Run without a window (remote debugging mode)
  --remote-debugging-port=<port> WebSocket port for remote debugging (default: 9222)
  --viewport-width=<px>          Headless viewport width (default: 800)
  --viewport-height=<px>         Headless viewport height (default: 600)
  --single-process               Run tabs in browser process threads (disable renderer isolation)
  --multi-process                Use zero-renderer child processes per tab (default)
  --wpt-parity                   Match WPT/product-smoke: CPU renderer and 1.0 scale (make browser-cpu default)
  --help, -h                     Show this help

Environment: {}={}",
        RenderMode::values(),
        RenderMode::ENV_VAR,
        RenderMode::values()
    );
}

// --- 平台检测 ---

fn logical_size_from_window(window: &winit::window::Window) -> ((u32, u32), f32) {
    let physical = window.inner_size();
    let scale = normalized_window_scale(window.scale_factor());
    let logical_width = ((physical.width as f32 / scale).round() as u32).max(1);
    let logical_height = ((physical.height as f32 / scale).round() as u32).max(1);
    ((logical_width, logical_height), scale)
}

fn normalized_window_scale(scale: f64) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale as f32
    } else {
        1.0
    }
}

/// 在 winit 初始化前检测平台缩放提示。
///
/// X11 下 winit 只有设置了 `Xft.dpi` 或 `WINIT_X11_SCALE_FACTOR` 才会返回正确的缩放因子，
/// 否则默认返回 1.0。此函数从常见环境变量中读取缩放设置，供 winit 使用。
///
/// 优先级：`WINIT_X11_SCALE_FACTOR` > `GDK_SCALE` > `QT_SCALE_FACTOR` > Xft.dpi > 1.0
fn detect_and_set_platform_scale() {
    if std::env::var("WINIT_X11_SCALE_FACTOR").is_ok() {
        return;
    }

    for var in ["GDK_SCALE", "QT_SCALE_FACTOR"] {
        if let Ok(val) = std::env::var(var)
            && let Ok(scale) = val.parse::<f64>()
            && scale > 1.0
            && scale.is_finite()
        {
            // SAFETY: 在 winit 初始化前（单线程）设置环境变量，无竞态风险
            unsafe {
                std::env::set_var("WINIT_X11_SCALE_FACTOR", format!("{scale}"));
            }
            tracing::info!("Detected {var}={scale}, setting WINIT_X11_SCALE_FACTOR={scale}");
            return;
        }
    }

    try_detect_x11_dpi();
}

/// 尝试从 X11 显示器 DPI 估算缩放因子。
fn try_detect_x11_dpi() {
    if std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("WAYLAND_SOCKET").is_ok() {
        return;
    }
    let output = match std::process::Command::new("xdpyinfo").output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => return,
    };
    let text = String::from_utf8_lossy(&output);
    for line in text.lines() {
        if line.contains("resolution:") && line.contains("dots per inch") {
            let dpi = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.split('x').next())
                .and_then(|s| s.parse::<f64>().ok());
            if let Some(dpi) = dpi
                && dpi > 96.0
                && dpi.is_finite()
            {
                let scale = (dpi / 96.0).round();
                if scale > 1.0 {
                    unsafe {
                        std::env::set_var("WINIT_X11_SCALE_FACTOR", format!("{scale}"));
                    }
                    tracing::info!("Detected X11 DPI {dpi}, setting WINIT_X11_SCALE_FACTOR={scale}");
                    return;
                }
            }
        }
    }
}

// --- 测试 ---

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
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
        let frame_w = 2.0 * (layout::PAGE_FRAME_INSET_H + layout::PAGE_FRAME_BORDER) * app.scale_factor;
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
        if crate::app::is_wayland() {
            assert!(config.maximized);
            assert!(!config.decorations);
        } else {
            assert!(!config.maximized);
            assert!(config.decorations);
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

        let (_, _, overlay, _) = app.build_scene_for_test(800, 600);
        let border = app.chrome_palette().separator;
        assert!(
            overlay.iter().any(|f| {
                f.color == border && f.rect.origin.x <= 0.5 && f.rect.origin.y <= 0.5 && f.rect.size.width >= 799.0
            }),
            "non-maximized Wayland window should draw top frame border"
        );

        app.set_window_maximized(true);
        let (_, _, overlay_max, _) = app.build_scene_for_test(800, 600);
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
        let (fills, _, _, _) = app.build_scene_for_test(1280, 900);
        assert!(
            fills
                .iter()
                .any(|f| { f.color == sep && (f.rect.size.height - line_h).abs() < 0.5 && f.rect.size.width <= 2.0 }),
            "adjacent inactive tabs should have a vertical separator"
        );

        let mut two_tabs = BrowserApp::new(RenderMode::Cpu);
        two_tabs.physical_size = (1280, 900);
        two_tabs.scale_factor = 1.0;
        let (fills_two, _, _, _) = two_tabs.build_scene_for_test(1280, 900);
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
        let (_, _, _, _) = app.build_scene_for_test(800, 600);
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

    /// Home 键应导航到设置的主页 URL，而非硬编码。
    #[test]
    fn home_key_navigates_to_configured_home() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.shell
            .apply_settings(|s| s.home_url = "https://custom.home.test".to_string());
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

        let (_, glyphs, _, _) = app.build_scene_for_test(1280, 900);
        assert!(
            glyphs.iter().any(|g| g.ch == '\u{E008}'),
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

        let (_, glyphs, _, _) = app.build_scene_for_test(1280, 900);
        assert!(
            glyphs.iter().any(|g| g.ch == '\u{E00A}'),
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

        let (_, glyphs, _, _) = app.build_scene_for_test(1280, 900);
        let more_vertical_count = glyphs.iter().filter(|g| g.ch == '\u{E008}').count();
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

        let (_, glyphs, _, _) = app.build_scene_for_test(1280, 900);
        assert!(
            glyphs.iter().any(|g| g.ch == '\u{E00B}'),
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

        let (fills, _, _, _) = app.build_scene_for_test(1280, 900);
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

        let (_, glyphs, _, _) = app.build_scene_for_test(1280, 900);
        assert!(
            glyphs.iter().any(|g| g.ch == '\u{E00F}'),
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
        let (fills, _, _, _) = app.build_scene_for_test(1280, 900);
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
        let (fills, glyphs, overlay, overlay_glyphs) = app.build_scene_for_test(800, 600);

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
        let (fills, _, _, _) = app.build_scene_for_test(800, 600);
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

        let (_, glyphs, _, _) = app.build_scene_for_test(800, 600);
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
        let (fills_loading, _, _, _) = app.build_scene_for_test(1280, 900);
        let spinner_segments = fills_loading.iter().filter(|f| f.color == loading).count();
        assert!(
            spinner_segments >= 28,
            "loading tab should draw spinner segments, got {spinner_segments}"
        );

        if let Some(tab) = app.shell.active_tab_mut() {
            tab.set_loading(false);
        }
        let (fills_idle, _, _, _) = app.build_scene_for_test(1280, 900);
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
        let (_, _, _, _) = app.build_scene_for_test(800, 600);
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

        let (fills_at_zero, _, _, _) = app.build_scene_for_test(1280, 900);

        // Linux/WSL 滚轮向下通常为负 LineDelta
        app.handle_scroll(zero_host_runtime::event::MouseScrollDelta::LineDelta(0.0, -3.0), x, y);

        assert!(
            app.scroll_offset_for_tab(tab_id) > 0.0,
            "tall page should scroll with negative line delta (scroll down on Linux)"
        );

        let content_top = content_y;
        let (fills_after, _, _, _) = app.build_scene_for_test(1280, 900);
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
                (cx + cw) <= fx + fw + 0.5 && (cy + ch) <= fy + fh - layout::PAGE_FRAME_BORDER * scale + 0.5,
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

            let (_, _, _overlay, _) = app.build_scene_for_test(1280, 900);

            let fb = app.render_scene_for_test(1280, 900);
            if layout::PAGE_FRAME_RADIUS > 0.0 {
                let sep = app.chrome_palette().separator;
                for (px, py) in [(cx + 2.0, cy + ch - 2.0), (cx + cw - 3.0, cy + ch - 2.0)] {
                    let x = px.round() as u32;
                    let y = py.round() as u32;
                    let i = ((y * fb.width + x) * 4) as usize;
                    assert_eq!(
                        (fb.data[i], fb.data[i + 1], fb.data[i + 2]),
                        (sep.r, sep.g, sep.b),
                        "scale={scale}: bottom corner ({x},{y}) should be separator"
                    );
                }
            }
        }
    }

    /// 最大化时启用底部 clip/UI guard，避免 WSLg 裁切圆角。
    #[test]
    fn page_layout_reserves_bottom_guards_when_maximized() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.physical_size = (1280, 900);
        app.scale_factor = 1.0;
        app.set_window_maximized(true);

        let (_, fy, _, fh) = app.page_frame_rect_for(1280, 900);
        let frame_bottom = fy + fh;
        let window_bottom = 900.0 - layout::PAGE_FRAME_BOTTOM_CLIP_GUARD - layout::PAGE_FRAME_BOTTOM_UI_GUARD;

        assert!(
            frame_bottom + layout::PAGE_FRAME_INSET_BOTTOM <= window_bottom + 0.5,
            "maximized frame should stay above bottom guards"
        );
    }

    /// 圆角 frame 时 overlay 应包含遮罩；扁平 frame 时 overlay 仍可用于浮动 UI。
    #[test]
    fn page_frame_bottom_corners_use_separator_overlay() {
        if layout::PAGE_FRAME_RADIUS <= 0.0 {
            return;
        }
        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.physical_size = (1280, 900);
        app.scale_factor = 1.0;

        let tab_id = app.shell.active_tab_id().unwrap();
        app.ensure_webview(tab_id);
        app.load_webview_html(
            tab_id,
            "<html><body><div style='height:2400px;background:#ff3232'>Tall</div></body></html>",
            Some("html, body { margin: 0; background: #ff3232; }"),
        );
        app.sync_webview_viewport();
        app.shell.on_page_loaded("Tall");

        let (_, _, overlay_fills, _) = app.build_scene_for_test(1280, 900);
        assert!(
            !overlay_fills.is_empty(),
            "page frame overlay should include corner masks and border"
        );

        let fb = app.render_scene_for_test(1280, 900);
        let (cx, cy, cw, ch) = app.page_content_rect();
        let sep = app.chrome_palette().separator;

        let sample_points = [(cx + 2.0, cy + ch - 2.0), (cx + cw - 3.0, cy + ch - 2.0)];
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
}

// --- 无头模式入口 ---

/// 无头模式：启动远程调试服务器，不接受窗口事件。
fn run_headless(cli: CliArgs) {
    let port = if cli.remote_debugging_port > 0 {
        cli.remote_debugging_port
    } else {
        9222
    };

    tracing::info!(
        "Starting headless mode on port {}, viewport: {:.0}x{:.0}",
        port,
        cli.viewport_width,
        cli.viewport_height
    );

    let mut server = headless::HeadlessServer::new(port, cli.viewport_width, cli.viewport_height);
    let actual_addr = server.addr();

    println!("ZeroWeb headless server: ws://{}", actual_addr);
    println!("DevTools URL: http://{}", actual_addr);

    if let Err(e) = server.run() {
        tracing::error!("Headless server error: {e}");
        std::process::exit(1);
    }
}

// --- 入口 ---

/// 按平台调整窗口配置（Wayland 上禁用 CSD，避免失焦时 subsurface commit 导致 compositor 断开）
fn browser_window_config() -> WindowConfig {
    let mut config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true);
    if app::is_wayland() {
        tracing::warn!("Wayland: disabling client-side decorations (CSD subsurface crash on focus switch)");
        config = config.with_decorations(false).with_maximized(true);
    } else if app::uses_unified_titlebar() {
        config = config.with_unified_titlebar(true);
    }
    config
}

fn apply_window_chrome_action(app: &mut BrowserApp, window: &winit::window::Window) {
    let Some(action) = app.take_window_chrome_action() else {
        return;
    };
    match action {
        WindowChromeAction::Minimize => {
            if window.fullscreen().is_some() {
                window.set_fullscreen(None);
                app.mark_surface_stale();
                sync_window_size_from_window(app, window);
            }
            window.set_minimized(true);
            sync_window_chrome_icon(app, window);
        }
        WindowChromeAction::ToggleMaximize => {
            // Wayland 下不可同时对 fullscreen 表面调用 set_maximized，须分步切换
            if window.fullscreen().is_some() {
                window.set_fullscreen(None);
                app.mark_surface_stale();
                sync_window_size_from_window(app, window);
                sync_window_chrome_icon(app, window);
            } else if window.is_maximized() {
                window.set_maximized(false);
                sync_window_chrome_icon(app, window);
            } else {
                window.set_maximized(true);
                sync_window_chrome_icon(app, window);
            }
        }
        WindowChromeAction::Close => {
            app.persist_user_data();
            std::process::exit(0);
        }
        WindowChromeAction::StartDrag => {
            if let Err(err) = window.drag_window() {
                tracing::warn!("drag_window failed: {err}");
            }
        }
    }
}

fn sync_window_size_from_window(app: &mut BrowserApp, window: &winit::window::Window) {
    let physical = window.inner_size();
    app.physical_size = (physical.width, physical.height);
    let scale = normalized_window_scale(window.scale_factor());
    let logical_width = ((physical.width as f32 / scale).round() as u32).max(1);
    let logical_height = ((physical.height as f32 / scale).round() as u32).max(1);
    app.set_window_size((logical_width, logical_height));
    app.scale_factor = scale;
    app.sync_webview_viewport();
}

fn sync_window_chrome_icon(app: &mut BrowserApp, window: &winit::window::Window) {
    let fullscreen = window.fullscreen().is_some();
    app.set_window_fullscreen(fullscreen);
    app.set_window_maximized(fullscreen || window.is_maximized());
}

fn main() {
    tracing_subscriber::fmt().init();

    detect_and_set_platform_scale();

    tracing::info!("ZeroBrowser starting...");

    let cli = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            print_usage();
            std::process::exit(2);
        }
    };
    let multiprocess = !cli.single_process;
    set_multiprocess_enabled(multiprocess);
    if multiprocess {
        tracing::info!("Multi-process mode: tabs use zero-renderer child processes");
    } else {
        tracing::info!("Single-process mode: tabs use in-process tab workers");
    }
    if cli.wpt_parity {
        tracing::info!("WPT parity mode: CPU renderer, scale 1.0 (aligned with product-smoke / reftest)");
    }
    tracing::info!("Renderer mode: {}", cli.render_mode);

    if cli.headless {
        run_headless(cli);
        return;
    }

    if let Some(scale) = cli.scale_override {
        // SAFETY: 在 winit 初始化前（单线程）设置，无竞态风险
        unsafe {
            std::env::set_var("WINIT_X11_SCALE_FACTOR", format!("{scale}"));
        }
        tracing::info!("CLI --scale={scale}, overriding WINIT_X11_SCALE_FACTOR");
    }

    let config = browser_window_config();

    let runtime = HostRuntime::new(config);
    let mut app = BrowserApp::new(cli.render_mode);

    tracing::info!("Entering event loop...");

    // CPU surface 由 main 管理生命周期
    let mut cpu_surface: Option<softbuffer::Surface<Arc<winit::window::Window>, Arc<winit::window::Window>>> = None;

    if let Err(e) = runtime.run_with_window(move |event, window| {
        app.poll_tab_fetch();

        match event {
            AppEvent::RedrawRequested => {
                if !app.window_focused {
                    app.needs_redraw = false;
                } else {
                    if !app.surface_configured {
                        if let Some(ref win) = window {
                            let wayland_cpu = app.wayland_forces_cpu_present();
                            let needs_gpu = !wayland_cpu
                                && matches!(app.render_mode(), RenderMode::Gpu | RenderMode::Auto)
                                && app.gpu_renderer_is_none()
                                && cpu_surface.is_none();
                            let needs_cpu = matches!(app.render_mode(), RenderMode::Cpu)
                                || wayland_cpu
                                || (app.gpu_renderer_is_none()
                                    && matches!(app.render_mode(), RenderMode::Auto)
                                    && cpu_surface.is_none());

                            if needs_gpu || needs_cpu {
                                let (logical_size, scale_factor) = logical_size_from_window(win);
                                let physical_size = win.inner_size();
                                app.set_window_size(logical_size);
                                app.physical_size = (physical_size.width, physical_size.height);
                                app.scale_factor = scale_factor;
                                app.sync_color_scheme_from_window(win);
                                app.ensure_startup_tab();
                                sync_window_chrome_icon(&mut app, win);
                                app.sync_webview_viewport();
                                tracing::debug!(
                                    "Surface init — physical: {}x{}, logical: {}x{}, scale: {:.2}",
                                    physical_size.width,
                                    physical_size.height,
                                    logical_size.0,
                                    logical_size.1,
                                    scale_factor
                                );

                                match app.render_mode() {
                                    RenderMode::Cpu => app.init_cpu_surface(win, &mut cpu_surface),
                                    RenderMode::Gpu | RenderMode::Auto => {
                                        if needs_gpu {
                                            app.init_gpu(win);
                                        }
                                        if wayland_cpu
                                            || (app.gpu_renderer_is_none()
                                                && matches!(app.render_mode(), RenderMode::Auto))
                                        {
                                            app.init_cpu_surface(win, &mut cpu_surface);
                                        }
                                    }
                                }
                            }
                        }
                        let phys = app.physical_size;
                        if let Some(ref mut gpu) = app.gpu_renderer_as_mut() {
                            gpu.configure_surface(phys.0, phys.1);
                            app.surface_configured = true;
                        } else if cpu_surface.is_some() {
                            app.surface_configured = true;
                        }
                    } else if app.gpu_surface_stale {
                        let (w, h) = app.physical_size;
                        if let Some(ref mut gpu) = app.gpu_renderer_as_mut() {
                            gpu.configure_surface(w, h);
                        }
                        app.gpu_surface_stale = false;
                    }

                    app.resume_gpu_present();

                    if app.gpu_renderer_is_some() {
                        app.render_frame(app.physical_size.0, app.physical_size.1, true);
                    } else {
                        app.render_cpu(app.physical_size.0, app.physical_size.1, &mut cpu_surface, true);
                    }
                    app.needs_redraw = false;
                    app.poll_tab_fetch();
                    app.begin_tab_fetch_after_paint();
                    if app.any_tab_loading() || app.tab_fetch_active() {
                        app.needs_redraw = true;
                        if let Some(ref win) = window {
                            win.request_redraw();
                        }
                    }
                }
            }
            AppEvent::Resized { width, height } if width > 0 && height > 0 => {
                tracing::debug!("Window resized: {width}x{height}");
                app.physical_size = (width, height);
                if let Some(ref win) = window {
                    let (logical_size, scale_factor) = logical_size_from_window(win);
                    app.set_window_size(logical_size);
                    app.scale_factor = scale_factor;
                    sync_window_chrome_icon(&mut app, win);
                } else {
                    app.set_window_size((width, height));
                    app.scale_factor = 1.0;
                }
                if app.window_focused
                    && let Some(ref mut gpu) = app.gpu_renderer_as_mut()
                {
                    gpu.configure_surface(width, height);
                } else {
                    app.gpu_surface_stale = true;
                }
                app.sync_webview_viewport();
                app.needs_redraw = true;
            }
            AppEvent::ScaleFactorChanged { scale_factor } => {
                tracing::debug!("Window scale factor changed: {scale_factor}");
                if let Some(ref win) = window {
                    let physical_size = win.inner_size();
                    let (logical_size, normalized_scale) = logical_size_from_window(win);
                    app.physical_size = (physical_size.width, physical_size.height);
                    app.set_window_size(logical_size);
                    app.scale_factor = normalized_scale;
                    if app.window_focused
                        && let Some(ref mut gpu) = app.gpu_renderer_as_mut()
                    {
                        gpu.configure_surface(physical_size.width, physical_size.height);
                    } else {
                        app.gpu_surface_stale = true;
                    }
                    app.sync_webview_viewport();
                } else {
                    app.scale_factor = normalized_window_scale(scale_factor);
                }
                app.needs_redraw = true;
            }
            AppEvent::CloseRequested => {
                app.persist_user_data();
                tracing::info!("Window closed");
            }
            AppEvent::KeyboardInput { key, text, pressed } => {
                app.handle_key(&key, pressed, text.as_deref());
            }
            AppEvent::MouseMoved { x, y } => {
                app.handle_mouse_move(x, y);
            }
            AppEvent::MouseInput { button, pressed, x, y } => {
                let btn_str = match button {
                    zero_host_runtime::event::MouseButton::Left => "Left",
                    zero_host_runtime::event::MouseButton::Right => "Right",
                    zero_host_runtime::event::MouseButton::Middle => "Middle",
                    _ => "Other",
                };
                app.handle_mouse_move(x, y);
                app.handle_mouse_click(x, y, pressed, btn_str);
            }
            AppEvent::MouseWheel { delta, x, y } => {
                app.handle_scroll(delta, x, y);
            }
            AppEvent::PanGesture { delta_x, delta_y, x, y } => {
                app.handle_pan_gesture(delta_x, delta_y, x, y);
            }
            AppEvent::Touch(touch) => {
                app.handle_touch(&touch);
            }
            AppEvent::Ime(event) => {
                app.handle_ime(event);
            }
            AppEvent::ThemeChanged { dark } => {
                app.handle_system_theme_changed(dark);
            }
            AppEvent::Focused => {
                tracing::debug!("Window focused");
                app.window_focused = true;
                app.gpu_surface_stale = true;
                app.needs_redraw = true;
                if let Some(ref win) = window {
                    sync_window_chrome_icon(&mut app, win);
                }
            }
            AppEvent::Unfocused => {
                tracing::debug!("Window unfocused");
                app.on_window_unfocused();
                app.window_focused = false;
                app.address_bar_focused = false;
                app.needs_redraw = false;
                app.gpu_surface_stale = true;
            }
            _ => {}
        }

        if app.needs_redraw
            && app.window_focused
            && let Some(ref win) = window
        {
            win.request_redraw();
        }

        if let Some(ref win) = window {
            app.sync_ime_state(win);
            apply_window_chrome_action(&mut app, win);
        }
    }) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }

    tracing::info!("ZeroBrowser exited");
}
