//! ZeroBrowser — 基于 Rust 的跨平台浏览器应用
//!
//! M11 里程碑：完整浏览器应用，连接 BrowserShell（数据模型）、
//! WebView（页面渲染）和 HostRuntime（窗口管理）。

#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
// Windows 子系统选择（编译期）：
// - 默认（打包发布）：GUI 子系统，双击 .exe 不弹控制台黑窗。
// - 启用 `windows-console` feature（开发脚本 browser.ps1 / browser-cpu.ps1）：
//   console 子系统，tracing 日志直接输出到调用方控制台，Ctrl+C 可终止。
// 测试构建始终保留 console 子系统（否则 test 输出不可见）。
// 注意：zero-renderer.exe 始终用 GUI 子系统（由 browser 通过管道 spawn，不能弹窗）。
#![cfg_attr(
    all(windows, not(test), not(feature = "windows-console")),
    windows_subsystem = "windows"
)]
#![allow(unused_comparisons)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::absurd_extreme_comparisons)]
// WIP：多进程后端（ProcessTabBackend 部分 API）与 tab_js_worker 线程 worker 脚本路径
// 尚未全量接线，存在未用方法/字段/变体；T4/T5 统一脚本/帧后评估删除或接线。
#![allow(dead_code)]

mod app;
mod clipboard;
mod colors;
mod compositor_client;
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
mod shutdown_signal;
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
mod tests;

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

/// 按平台调整窗口配置。
///
/// - Wayland：禁用 CSD，避免失焦时 subsurface commit 导致 compositor 断开
/// - Windows：禁用系统装饰，改用自绘标题栏（控制按钮 + 拖拽区），
///   依赖 winit 0.30 对无边框窗口的 Aero Snap 支持（WS_THICKFRAME 保留）
/// - macOS：使用一体化标题栏（系统 traffic lights 与标签栏同排）
fn browser_window_config() -> WindowConfig {
    let mut config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true)
        .with_maximized(true);
    if app::is_wayland() {
        tracing::warn!("Wayland: disabling client-side decorations (CSD subsurface crash on focus switch)");
        config = config.with_decorations(false);
    } else if cfg!(target_os = "windows") {
        tracing::info!("Windows: using custom titlebar (system decorations disabled, Aero Snap retained via winit)");
        config = config.with_decorations(false);
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
            // 必须在 process::exit 前 kill 子进程：process::exit 跳过 Drop，
            // 否则 zero-renderer.exe 孤儿会锁住自身 exe，下次 cargo build 报 os error 5。
            app.shutdown_child_processes();
            std::process::exit(0);
        }
        WindowChromeAction::ToggleFullscreen => {
            if window.fullscreen().is_some() {
                window.set_fullscreen(None);
            } else {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
            app.mark_surface_stale();
            sync_window_size_from_window(app, window);
            sync_window_chrome_icon(app, window);
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

/// 同步窗口标题为当前活跃标签页的标题（任务栏可见）。
fn sync_window_title(app: &mut BrowserApp, window: &winit::window::Window) {
    if let Some(title) = app.window_title_if_changed() {
        window.set_title(&title);
        app.confirm_window_title(&title);
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    // 注册 Ctrl+C handler：把信号转成 flag，事件循环检测到后走
    // shutdown_child_processes 正常退出路径，避免孤儿 renderer 锁住 exe。
    // 必须在进入事件循环前安装。
    shutdown_signal::install();

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
        // Ctrl+C / 系统关机信号：走和窗口关闭按钮一样的清理路径，
        // 避免 process::exit 跳过 Drop 导致 zero-renderer 子进程成为孤儿。
        if shutdown_signal::is_set() {
            tracing::info!("Shutdown signal received, exiting gracefully...");
            app.persist_user_data();
            app.shutdown_child_processes();
            std::process::exit(0);
        }

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
            sync_window_title(&mut app, win);
        }
    }) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }

    tracing::info!("ZeroBrowser exited");
}
