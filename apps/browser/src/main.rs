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
mod gui_smoke;
mod headless;
mod input_keys;
mod layout;
mod page_scroll;
mod page_selection;
mod pages;
mod paint_ipc;
mod parity_smoke;
mod process_backend;
mod service_worker_owner;
mod shutdown_signal;
mod smoke_capture;
mod tab_chrome;
mod tab_favicon;
#[cfg(any(test, feature = "test-support"))]
mod tab_js_worker;
mod tab_manager;
#[cfg(any(test, feature = "test-support"))]
mod tab_scripts;
mod tab_snapshot;
#[cfg(any(test, feature = "test-support"))]
mod tab_worker;
#[cfg(not(any(test, feature = "test-support")))]
#[path = "tab_worker_stub.rs"]
mod tab_worker;
mod text_input;
mod text_metrics;
mod ui_icons;
#[cfg(target_os = "windows")]
mod windows_titlebar;

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::config::RenderMode;

use app::BrowserApp;
use app::WindowChromeAction;

/// 单个 Browser 日志文件的最大大小；达到后轮转，避免 GUI 版日志无限增长。
const BROWSER_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// 保留的历史 Browser 日志数量（`.1` 最新，`.5` 最旧）。
const BROWSER_LOG_BACKUP_COUNT: usize = 5;

// --- CLI 参数 ---

struct CliArgs {
    render_mode: RenderMode,
    scale_override: Option<f32>,
    headless: bool,
    remote_debugging_port: u16,
    viewport_width: f32,
    viewport_height: f32,
    /// 与 WPT reftest 对齐：CPU 光栅化 + 1.0 缩放（便于肉眼对比 product-smoke）。
    wpt_parity: bool,
    /// 显式启用真实窗口最终帧产品 smoke，并在成功呈现后写入 PNG。
    smoke_capture: Option<PathBuf>,
    /// 显式启用真实网站 compositor GUI 操作 smoke。
    gui_smoke: Option<gui_smoke::GuiSmokeConfig>,
    /// 显式启用 Chrome 一致性真实窗口交互场景。
    parity_smoke: Option<parity_smoke::ParitySmokeConfig>,
}

fn parse_args() -> Result<CliArgs, String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        std::process::exit(0);
    }
    parse_args_from(args, RenderMode::from_env()?)
}

fn parse_args_from(
    args: impl IntoIterator<Item = String>,
    env_render_mode: Option<RenderMode>,
) -> Result<CliArgs, String> {
    let mut args = args.into_iter();
    let mut render_mode = None;
    let mut scale_override = None;
    let mut headless = false;
    let mut remote_debugging_port = 0u16;
    let mut viewport_width = 800.0f32;
    let mut viewport_height = 600.0f32;
    let mut wpt_parity = false;
    let mut smoke_capture = None;
    let mut gui_smoke_url = None;
    let mut gui_smoke_dir = None;
    let mut parity_scenario = None;
    let mut parity_output_dir = None;

    while let Some(arg) = args.next() {
        if arg == "--single-process" || arg == "--multi-process" {
            return Err(format!(
                "{arg} is no longer supported; ZeroBrowser always uses isolated renderer processes"
            ));
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

        if arg == "--wpt-parity" {
            wpt_parity = true;
        }

        if let Some(value) = arg.strip_prefix("--viewport-width=") {
            viewport_width = value.parse::<f32>().map_err(|_| format!("invalid width: {value}"))?;
            if viewport_width <= 0.0 || !viewport_width.is_finite() {
                return Err(format!("viewport width must be positive: {viewport_width}"));
            }
        }

        if let Some(value) = arg.strip_prefix("--viewport-height=") {
            viewport_height = value.parse::<f32>().map_err(|_| format!("invalid height: {value}"))?;
            if viewport_height <= 0.0 || !viewport_height.is_finite() {
                return Err(format!("viewport height must be positive: {viewport_height}"));
            }
        }

        if let Some(value) = arg.strip_prefix("--smoke-capture=") {
            if value.is_empty() {
                return Err("--smoke-capture requires a PNG path".to_string());
            }
            smoke_capture = Some(PathBuf::from(value));
        }

        if arg == "--smoke-capture" {
            let value = args
                .next()
                .ok_or_else(|| "--smoke-capture requires a PNG path".to_string())?;
            if value.is_empty() {
                return Err("--smoke-capture requires a PNG path".to_string());
            }
            smoke_capture = Some(PathBuf::from(value));
        }

        if let Some(value) = arg.strip_prefix("--gui-smoke-url=") {
            gui_smoke_url = Some(value.to_string());
        }

        if arg == "--gui-smoke-url" {
            gui_smoke_url = Some(
                args.next()
                    .ok_or_else(|| "--gui-smoke-url requires an HTTP(S) URL".to_string())?,
            );
        }

        if let Some(value) = arg.strip_prefix("--gui-smoke-dir=") {
            gui_smoke_dir = Some(PathBuf::from(value));
        }

        if arg == "--gui-smoke-dir" {
            gui_smoke_dir =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--gui-smoke-dir requires a directory path".to_string()
                })?));
        }

        if let Some(value) = arg.strip_prefix("--parity-scenario=") {
            parity_scenario = Some(PathBuf::from(value));
        }

        if arg == "--parity-scenario" {
            parity_scenario = Some(PathBuf::from(
                args.next()
                    .ok_or_else(|| "--parity-scenario requires a JSON path".to_string())?,
            ));
        }

        if let Some(value) = arg.strip_prefix("--parity-output-dir=") {
            parity_output_dir = Some(PathBuf::from(value));
        }

        if arg == "--parity-output-dir" {
            parity_output_dir =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--parity-output-dir requires a directory path".to_string()
                })?));
        }
    }

    let cli_render_mode = render_mode;
    let cli_scale = scale_override;
    let mut render_mode = cli_render_mode.or(env_render_mode).unwrap_or_default();
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
    let gui_smoke = match (gui_smoke_url, gui_smoke_dir) {
        (Some(url), Some(output_dir)) => Some(gui_smoke::GuiSmokeConfig::new(url, output_dir)?),
        (None, None) => None,
        _ => {
            return Err("--gui-smoke-url and --gui-smoke-dir must be provided together".to_string());
        }
    };
    let parity_smoke = match (parity_scenario, parity_output_dir) {
        (Some(scenario), Some(output_dir)) => Some(parity_smoke::ParitySmokeConfig::load(scenario, output_dir)?),
        (None, None) => None,
        _ => {
            return Err("--parity-scenario and --parity-output-dir must be provided together".to_string());
        }
    };
    let smoke_modes =
        usize::from(smoke_capture.is_some()) + usize::from(gui_smoke.is_some()) + usize::from(parity_smoke.is_some());
    if smoke_modes > 1 {
        return Err("smoke capture, GUI smoke, and parity smoke are mutually exclusive".to_string());
    }
    if smoke_modes > 0 {
        if headless {
            return Err("GUI smoke requires a real window".to_string());
        }
        let gpu_dmabuf_smoke = zero_runtime_config::enabled_when_true("ZERO_BROWSER_GPU_DMABUF_SMOKE");
        if gpu_dmabuf_smoke && render_mode == RenderMode::Cpu {
            return Err("GPU dma-buf smoke requires --renderer=gpu|auto --scale=1".to_string());
        }
        if scale_override != Some(1.0) {
            return Err("GUI smoke requires --scale=1".to_string());
        }
    }
    Ok(CliArgs {
        render_mode,
        scale_override,
        headless,
        remote_debugging_port,
        viewport_width,
        viewport_height,
        wpt_parity,
        smoke_capture,
        gui_smoke,
        parity_smoke,
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
  --viewport-width=<px>          Headless/GUI smoke page viewport width (default: 800)
  --viewport-height=<px>         Headless/GUI smoke page viewport height (default: 600)
  --wpt-parity                   Match WPT/product-smoke: CPU renderer and 1.0 scale (make browser-cpu default)
  --smoke-capture=<png>          Capture the real presented window frame, emit region stats, then exit
  --gui-smoke-url=<url>          Run compositor GUI actions against a real HTTP(S) website
  --gui-smoke-dir=<dir>          Write GUI smoke step screenshots into this directory
  --parity-scenario=<json>       Run a real-window Chrome parity interaction scenario
  --parity-output-dir=<dir>      Write ZeroWeb parity evidence into this directory
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
/// - Windows：使用自绘 caption buttons；最大化命中区交由系统显示 Snap Layout
/// - macOS：使用一体化标题栏（系统 traffic lights 与标签栏同排）
fn browser_window_config(app: &BrowserApp, smoke_viewport: Option<(u32, u32)>) -> WindowConfig {
    let mut config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true)
        .with_maximized(true);
    if let Some((viewport_width, viewport_height)) = smoke_viewport {
        let (_, _, current_width, current_height) = app.page_content_rect_for(1024, 768);
        let window_width = (1024.0 + viewport_width as f32 - current_width).round().max(1.0) as u32;
        let window_height = (768.0 + viewport_height as f32 - current_height).round().max(1.0) as u32;
        config = config.with_size(window_width, window_height).with_maximized(false);
    }
    if app::is_wayland() {
        tracing::warn!("Wayland: disabling client-side decorations (CSD subsurface crash on focus switch)");
        config = config.with_decorations(false);
    } else if cfg!(target_os = "windows") {
        tracing::info!("Windows: using Chrome-style custom caption buttons with system Snap Layout hit testing");
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

fn browser_log_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
        .map(|base| base.join("ZeroWeb").join("logs").join("zero-browser.log"))
}

struct RollingLogWriter {
    path: PathBuf,
    file: Option<File>,
    max_bytes: u64,
    backup_count: usize,
}

impl RollingLogWriter {
    fn open(path: PathBuf, max_bytes: u64, backup_count: usize) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file: Some(file),
            max_bytes,
            backup_count,
        })
    }

    fn backup_path(&self, index: usize) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        name.push(format!(".{index}"));
        self.path.with_file_name(name)
    }

    fn rotate(&mut self) -> io::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }

        if self.backup_count > 0 {
            for index in (1..=self.backup_count).rev() {
                let source = if index == 1 {
                    self.path.clone()
                } else {
                    self.backup_path(index - 1)
                };
                let destination = self.backup_path(index);
                if source.exists() {
                    if destination.exists() {
                        fs::remove_file(&destination)?;
                    }
                    fs::rename(source, destination)?;
                }
            }
        }

        self.file = Some(OpenOptions::new().create(true).append(true).open(&self.path)?);
        Ok(())
    }
}

impl Write for RollingLogWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let current_size = self.file.as_ref().expect("log file is open").metadata()?.len();
        if current_size > 0 && current_size.saturating_add(buffer.len() as u64) > self.max_bytes {
            self.rotate()?;
        }
        self.file.as_mut().expect("log file is open").write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().expect("log file is open").flush()
    }
}

/// 将浏览器日志同时写到持久化日志和启动它的控制台。
///
/// GUI 子系统没有关联控制台时，标准错误输出会失败；这不应影响文件日志。
struct TeeLogWriter<W> {
    file: RollingLogWriter,
    console: W,
}

impl<W> TeeLogWriter<W> {
    fn new(file: RollingLogWriter, console: W) -> Self {
        Self { file, console }
    }
}

impl<W: Write> Write for TeeLogWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let _ = self.console.write_all(buffer);
        self.file.write_all(buffer)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let _ = self.console.flush();
        self.file.flush()
    }
}

fn init_logging() {
    let Some(path) = browser_log_path() else {
        tracing_subscriber::fmt().init();
        return;
    };
    let Some(parent) = path.parent() else {
        tracing_subscriber::fmt().init();
        return;
    };
    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!("无法创建浏览器日志目录: {error}");
        tracing_subscriber::fmt().init();
        return;
    }
    match RollingLogWriter::open(path.clone(), BROWSER_LOG_MAX_BYTES, BROWSER_LOG_BACKUP_COUNT) {
        Ok(file) => tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(TeeLogWriter::new(file, io::stderr())))
            .init(),
        Err(error) => {
            eprintln!("无法打开浏览器日志文件 {}: {error}", path.display());
            tracing_subscriber::fmt().init();
        }
    }
}

fn main() {
    init_logging();

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
    tracing::info!("Tabs use zero-renderer child processes");
    if cli.wpt_parity {
        tracing::info!("WPT parity mode: CPU renderer, scale 1.0 (aligned with product-smoke / reftest)");
    }
    tracing::info!("Renderer mode: {}", cli.render_mode);
    #[cfg(target_os = "linux")]
    if cli.render_mode == RenderMode::Cpu || cli.parity_smoke.is_some() {
        // CPU present 与 parity GPU readback 都需要 compositor RGBA 交付。
        // parity 仍会把 GPU 合成帧上传到 Browser GPU 场景。
        // SAFETY: 须在 compositor worker / 子进程启动前设置。
        unsafe {
            std::env::set_var("ZW_BROWSER_GPU_DMABUF_IMPORT", "0");
        }
    }
    if cli.smoke_capture.is_some() || cli.gui_smoke.is_some() || cli.parity_smoke.is_some() {
        // SAFETY: 设置发生在任何 renderer/compositor 子进程和工作线程启动之前。
        unsafe {
            std::env::set_var("ZERO_BROWSER_PRODUCT_SMOKE", "1");
        }
        let fixture = cli
            .gui_smoke
            .as_ref()
            .map(|config| config.url.as_str())
            .or_else(|| cli.parity_smoke.as_ref().map(parity_smoke::ParitySmokeConfig::url))
            .unwrap_or("zero://newtab");
        tracing::info!("SMOKE_EVENT component=browser event=enabled fixture={fixture}");
    }

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

    let mut app = BrowserApp::new(cli.render_mode);
    let smoke_viewport = cli
        .parity_smoke
        .as_ref()
        .map(parity_smoke::ParitySmokeConfig::viewport)
        .or_else(|| {
            cli.gui_smoke
                .as_ref()
                .map(|_| (cli.viewport_width.round() as u32, cli.viewport_height.round() as u32))
        });
    let config = browser_window_config(&app, smoke_viewport);
    let scale_override = cli.scale_override;
    let forced_physical_size = scale_override.map(|_| (config.width, config.height));
    let runtime = HostRuntime::new(config);
    let smoke_capture_path = cli.smoke_capture;
    let mut gui_smoke = cli.gui_smoke.map(gui_smoke::GuiSmoke::new);
    let mut parity_smoke = cli.parity_smoke.map(parity_smoke::ParitySmoke::new);

    tracing::info!("Entering event loop...");

    // CPU surface 由 main 管理生命周期
    let mut cpu_surface: Option<softbuffer::Surface<Arc<winit::window::Window>, Arc<winit::window::Window>>> = None;
    #[cfg(target_os = "windows")]
    let mut windows_caption_hit_test_installed = false;
    let poll_active = Arc::new(AtomicBool::new(true));

    if let Err(e) = runtime.run_with_window_polling(Duration::from_millis(16), poll_active, move |event, window| {
        // Ctrl+C / 系统关机信号：走和窗口关闭按钮一样的清理路径，
        // 避免 process::exit 跳过 Drop 导致 zero-renderer 子进程成为孤儿。
        if shutdown_signal::is_set() {
            tracing::info!("Shutdown signal received, exiting gracefully...");
            app.persist_user_data();
            app.shutdown_child_processes();
            std::process::exit(0);
        }
        if let Some(smoke) = gui_smoke.as_ref()
            && let Err(error) = smoke.check_timeout()
        {
            tracing::error!("GUI_SMOKE_FAILURE error={error}");
            app.shutdown_child_processes();
            std::process::exit(3);
        }
        if let Some(smoke) = parity_smoke.as_ref()
            && let Err(error) = smoke.check_timeout()
        {
            tracing::error!("PARITY_SMOKE_FAILURE error={error}");
            app.shutdown_child_processes();
            std::process::exit(3);
        }

        app.poll_tab_fetch();
        app.expire_scrollbar_overlay();
        #[cfg(target_os = "windows")]
        app.sync_windows_caption_hover();
        match event {
            AppEvent::RedrawRequested => {
                if !app.window_focused && smoke_capture_path.is_none() && gui_smoke.is_none() && parity_smoke.is_none()
                {
                    app.needs_redraw = false;
                } else {
                    if !app.surface_configured {
                        if let Some(ref win) = window {
                            if let Some((width, height)) = forced_physical_size
                                && win.inner_size() != winit::dpi::PhysicalSize::new(width, height)
                            {
                                let _ = win.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
                                return;
                            }
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
                                let physical_size = win.inner_size();
                                let scale_factor =
                                    scale_override.unwrap_or_else(|| normalized_window_scale(win.scale_factor()));
                                let logical_size = (
                                    ((physical_size.width as f32 / scale_factor).round() as u32).max(1),
                                    ((physical_size.height as f32 / scale_factor).round() as u32).max(1),
                                );
                                app.set_window_size(logical_size);
                                app.physical_size = (physical_size.width, physical_size.height);
                                app.scale_factor = scale_factor;
                                app.sync_color_scheme_from_window(win);
                                app.ensure_startup_tab();
                                sync_window_chrome_icon(&mut app, win);
                                app.sync_webview_viewport();
                                #[cfg(target_os = "windows")]
                                if !windows_caption_hit_test_installed {
                                    match windows_titlebar::install(win) {
                                        Ok(()) => {
                                            windows_caption_hit_test_installed = true;
                                            tracing::info!("Windows: native maximize hit testing installed");
                                        }
                                        Err(error) => {
                                            tracing::error!(
                                                "Windows: cannot install native maximize hit testing: {error}"
                                            );
                                        }
                                    }
                                }
                                if let Some(smoke) = gui_smoke.as_mut() {
                                    smoke.start(&mut app);
                                }
                                if let Some(smoke) = parity_smoke.as_mut() {
                                    smoke.start(&mut app);
                                }
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

                    // 必须在场景装配前锁定来源；render 后的 poll 可能采用新快照，
                    // 不能把新状态误配到刚呈现的上一张 framebuffer。
                    let presented_source = app.product_smoke_frame_source();
                    let capture_gpu_frame =
                        app.gpu_renderer_is_some() && (smoke_capture_path.is_some() || gui_smoke.is_some());
                    let presented_frame = if app.gpu_renderer_is_some() {
                        app.render_frame(app.physical_size.0, app.physical_size.1, true);
                        if capture_gpu_frame {
                            match app.render_full_scene_gpu_capture(app.physical_size.0, app.physical_size.1) {
                                Ok(frame) => Some(frame),
                                Err(error) => {
                                    tracing::error!("GPU_SMOKE_CAPTURE_FAILURE error={error}");
                                    app.shutdown_child_processes();
                                    std::process::exit(3);
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        app.render_cpu(app.physical_size.0, app.physical_size.1, &mut cpu_surface, true)
                    };
                    app.needs_redraw = false;
                    app.poll_tab_fetch();
                    app.begin_tab_fetch_after_paint();
                    if let (Some(path), Some(frame), Some(source)) = (
                        smoke_capture_path.as_deref(),
                        presented_frame.as_ref(),
                        presented_source,
                    ) {
                        let mode = if zero_runtime_config::enabled_when_true("ZERO_BROWSER_GPU_DMABUF_SMOKE") {
                            "gpu-dmabuf"
                        } else if compositor_client::enabled() {
                            "compositor"
                        } else {
                            "legacy"
                        };
                        let chrome_height = app.page_content_rect_for(frame.width, frame.height).1.ceil() as u32;
                        let (page_x, page_y, page_width, page_height) =
                            app.page_content_rect_for(frame.width, frame.height);
                        let result = smoke_capture::capture_presented_frame(
                            path,
                            frame,
                            smoke_capture::PixelRegion {
                                x: 0,
                                y: 0,
                                width: frame.width,
                                height: chrome_height,
                            },
                            smoke_capture::PixelRegion {
                                x: page_x.floor().max(0.0) as u32,
                                y: page_y.floor().max(0.0) as u32,
                                width: page_width.ceil().max(0.0) as u32,
                                height: page_height.ceil().max(0.0) as u32,
                            },
                            mode,
                            "zero://newtab",
                            source,
                        );
                        match result {
                            Ok(()) => {
                                tracing::info!(
                                    "SMOKE_EVENT component=browser event=frame_captured source={source} fallback=false"
                                );
                                app.shutdown_child_processes();
                                std::process::exit(0);
                            }
                            Err(error) => {
                                tracing::error!("SMOKE_FAILURE error={error}");
                                app.shutdown_child_processes();
                                std::process::exit(3);
                            }
                        }
                    }
                    if let (Some(smoke), Some(frame), Some(source)) =
                        (gui_smoke.as_mut(), presented_frame.as_ref(), presented_source)
                    {
                        match smoke.on_presented_frame(&mut app, frame, source) {
                            Ok(true) => {
                                app.shutdown_child_processes();
                                std::process::exit(0);
                            }
                            Ok(false) => {}
                            Err(error) => {
                                tracing::error!("GUI_SMOKE_FAILURE error={error}");
                                app.shutdown_child_processes();
                                std::process::exit(3);
                            }
                        }
                    }
                }
            }
            AppEvent::Resized { width, height } if width > 0 && height > 0 => {
                tracing::debug!("Window resized: {width}x{height}");
                app.physical_size = (width, height);
                if let Some(ref win) = window {
                    let scale_factor = scale_override.unwrap_or_else(|| normalized_window_scale(win.scale_factor()));
                    let logical_size = (
                        ((width as f32 / scale_factor).round() as u32).max(1),
                        ((height as f32 / scale_factor).round() as u32).max(1),
                    );
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
                    let normalized_scale =
                        scale_override.unwrap_or_else(|| normalized_window_scale(win.scale_factor()));
                    let logical_size = (
                        ((physical_size.width as f32 / normalized_scale).round() as u32).max(1),
                        ((physical_size.height as f32 / normalized_scale).round() as u32).max(1),
                    );
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
                    app.scale_factor = scale_override.unwrap_or_else(|| normalized_window_scale(scale_factor));
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

        if app.surface_configured
            && app.gpu_renderer_is_some()
            && let Some(smoke) = parity_smoke.as_mut()
        {
            // compositor 完成可能在本轮窗口事件处理期间到达；证据 tick 前再 poll 一次，
            // 保证状态与最新可显示页面帧来自同一输入事务。
            app.poll_tab_fetch();
            let source = app.product_smoke_frame_source();
            if let Some(source) = source {
                match app.render_full_scene_gpu_capture(app.physical_size.0, app.physical_size.1) {
                    Ok(frame) => {
                        let result = smoke.on_presented_frame(&mut app, &frame, source);
                        match result {
                            Ok(true) => {
                                app.shutdown_child_processes();
                                std::process::exit(0);
                            }
                            Ok(false) => {}
                            Err(error) => {
                                tracing::error!("PARITY_SMOKE_FAILURE error={error}");
                                app.shutdown_child_processes();
                                std::process::exit(3);
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!("GPU_SMOKE_CAPTURE_FAILURE error={error}");
                        app.shutdown_child_processes();
                        std::process::exit(3);
                    }
                }
            }
        }

        if (app.needs_redraw || parity_smoke.is_some())
            && (app.window_focused || smoke_capture_path.is_some() || gui_smoke.is_some() || parity_smoke.is_some())
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
