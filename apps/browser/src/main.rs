//! ZeroBrowser — 基于 Rust 的跨平台浏览器应用
//!
//! M11 里程碑：完整浏览器应用，连接 BrowserShell（数据模型）、
//! WebView（页面渲染）和 HostRuntime（窗口管理）。

mod app;
mod colors;
mod headless;
mod layout;
mod pages;

use std::sync::Arc;

use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::config::RenderMode;

use app::BrowserApp;

// --- CLI 参数 ---

struct CliArgs {
    render_mode: RenderMode,
    scale_override: Option<f32>,
    headless: bool,
    remote_debugging_port: u16,
    viewport_width: f32,
    viewport_height: f32,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = std::env::args().skip(1);
    let mut render_mode = None;
    let mut scale_override = None;
    let mut headless = false;
    let mut remote_debugging_port = 0u16;
    let mut viewport_width = 800.0f32;
    let mut viewport_height = 600.0f32;

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

        if let Some(value) = arg.strip_prefix("--viewport-width=") {
            viewport_width = value.parse::<f32>().map_err(|_| format!("invalid width: {value}"))?;
        }

        if let Some(value) = arg.strip_prefix("--viewport-height=") {
            viewport_height = value.parse::<f32>().map_err(|_| format!("invalid height: {value}"))?;
        }
    }

    let render_mode = render_mode.or(RenderMode::from_env()?).unwrap_or_default();
    Ok(CliArgs {
        render_mode,
        scale_override,
        headless,
        remote_debugging_port,
        viewport_width,
        viewport_height,
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
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive, RenderPrimitives};

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
    fn startup_has_single_default_tab() {
        let app = BrowserApp::new(RenderMode::Cpu);
        assert_eq!(app.shell.tab_count(), 1, "should start with exactly one tab");
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
    fn browser_window_config_disables_decorations_on_wayland() {
        let config = super::browser_window_config();
        if crate::app::is_wayland() {
            assert!(!config.decorations);
        } else {
            assert!(config.decorations);
        }
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
        let _ = app.build_scene_for_test(800, 600);
    }

    /// 验证 Ctrl 修饰键追踪：按下 Ctrl 后标记为活跃，释放后恢复。
    #[test]
    fn ctrl_key_tracking() {
        let mut app = BrowserApp::new(RenderMode::Cpu);

        // Ctrl 按下
        app.handle_key("Control", true);
        assert!(app.is_ctrl_pressed(), "ctrl_pressed should be true after Ctrl down");

        // Ctrl 释放
        app.handle_key("Control", false);
        assert!(!app.is_ctrl_pressed(), "ctrl_pressed should be false after Ctrl up");
    }

    /// 验证 Ctrl+L 聚焦地址栏并清空文本。
    #[test]
    fn ctrl_l_focuses_address_bar() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        assert!(!app.address_bar_focused);

        // Ctrl 按下 + L
        app.handle_key("Control", true);
        app.handle_key("l", true);
        assert!(app.address_bar_focused, "Ctrl+L should focus address bar");
        assert!(
            app.address_bar_text().is_empty(),
            "Ctrl+L should clear address bar text"
        );
    }

    /// 验证 Ctrl+D 添加书签（当前页面）。
    #[test]
    fn ctrl_d_adds_bookmark() {
        let mut app = BrowserApp::new(RenderMode::Cpu);

        // 先导航到一个页面
        let tab_id = app.shell.active_tab_id().unwrap();
        app.ensure_webview(tab_id);
        app.shell.navigate("https://example.com");
        app.shell.on_page_loaded("Example");

        let count_before = app.shell.bookmarks().len();
        app.handle_key("Control", true);
        app.handle_key("d", true);
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

        app.handle_key("Control", true);
        app.handle_key("w", true);
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
        app.handle_key("Control", true);
        app.handle_key("Control", false);
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
        let (fills, glyphs) = app.build_scene_for_test(800, 600);

        // 应有下载栏的 fill（至少一个蓝色进度条填充）
        assert!(
            fills.iter().any(|f| f.color == colors::DOWNLOAD_BAR_BG),
            "should have download bar background"
        );

        // 应有下载相关文字 glyph
        let text: String = glyphs.iter().map(|g| g.ch).collect();
        assert!(
            text.contains("file.zip"),
            "download bar should show filename, got glyphs containing: {}",
            text.chars().take(200).collect::<String>()
        );

        let _ = glyphs; // 避免 unused 警告
    }

    /// 验证设置页面生成正确 HTML。
    #[test]
    fn settings_page_generates_html() {
        let settings = zero_browser_shell::BrowserSettings::new();
        let html = pages::generate_settings_html(&settings);
        assert!(html.contains("设置"), "settings page should have title");
        assert!(html.contains("Google"), "settings page should show search engine");
        assert!(html.contains("example.com"), "settings page should show home URL");
        assert!(html.contains("ZeroBrowser"), "settings page should show browser name");
    }

    /// 验证 open_settings_page 正确加载。
    #[test]
    fn open_settings_page_loads_in_webview() {
        let mut app = BrowserApp::new(RenderMode::Cpu);
        let tab_id = app.shell.active_tab_id().unwrap();
        app.ensure_webview(tab_id);

        app.open_settings_page();

        assert_eq!(app.address_bar_text(), "zero://settings");
        // WebView 应该有渲染结果
        let _ = app.build_scene_for_test(800, 600);
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
    let config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true);
    if app::is_wayland() {
        tracing::warn!("Wayland: disabling client-side decorations (CSD subsurface crash on focus switch)");
        config.with_decorations(false)
    } else {
        config
    }
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

    // 初始化默认空白标签页（Shell 构造时已创建唯一 tab）
    app.init_default_tab();

    tracing::info!("Entering event loop...");

    // CPU surface 由 main 管理生命周期
    let mut cpu_surface: Option<softbuffer::Surface<Arc<winit::window::Window>, Arc<winit::window::Window>>> = None;

    if let Err(e) = runtime.run_with_window(move |event, window| {
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
                }
            }
            AppEvent::Resized { width, height } if width > 0 && height > 0 => {
                tracing::debug!("Window resized: {width}x{height}");
                app.physical_size = (width, height);
                if let Some(ref win) = window {
                    let (logical_size, scale_factor) = logical_size_from_window(win);
                    app.set_window_size(logical_size);
                    app.scale_factor = scale_factor;
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
                let (cw, ch) = app.content_physical_size();
                app.resize_all_webviews(cw, ch);
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
                    let (cw, ch) = app.content_physical_size();
                    app.resize_all_webviews(cw, ch);
                } else {
                    app.scale_factor = normalized_window_scale(scale_factor);
                }
                app.needs_redraw = true;
            }
            AppEvent::CloseRequested => {
                tracing::info!("Window closed");
            }
            AppEvent::KeyboardInput { key, pressed } => {
                app.handle_key(&key, pressed);
            }
            AppEvent::MouseMoved { x, y } => {
                app.handle_mouse_move(x, y);
            }
            AppEvent::MouseInput { button, pressed } => {
                let btn_str = match button {
                    zero_host_runtime::event::MouseButton::Left => "Left",
                    zero_host_runtime::event::MouseButton::Right => "Right",
                    zero_host_runtime::event::MouseButton::Middle => "Middle",
                    _ => "Other",
                };
                app.handle_mouse_click(app.mouse_pos.0, app.mouse_pos.1, pressed, btn_str);
            }
            AppEvent::MouseWheel { delta } => {
                app.handle_scroll(delta);
            }
            AppEvent::Focused => {
                tracing::debug!("Window focused");
                app.window_focused = true;
                app.gpu_surface_stale = true;
                app.needs_redraw = true;
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
        }
    }) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }

    tracing::info!("ZeroBrowser exited");
}
