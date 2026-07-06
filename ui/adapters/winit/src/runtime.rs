//! winit 后端的 PlatformRuntime 占位实现（spec IF-006）+ [`WinitRuntime::launch`]（DC-2
//! 终端阻塞壳的可测试 setup 核心）。
//!
//! M1 提供空 `run` 满足 trait 边界与编译期依赖隔离验证；真实事件循环/窗口/surface/IME
//! 在 M2（桌面）与 M4（移动）落地。`launch` 把「建树 + 工厂注册 + 首帧」从阻塞的 GUI
//! run loop 中抽离为可单测的 setup 核心（headless 可验证）；GUI-gated 的 `EventLoop::new`/
//! `Window`/surface + `event_loop.run` 包壳是剩余运行时件（需 GUI 验证首帧）。

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GpuRenderer;
use zero_text_foundation::FontdueBackend;
use zero_ui_adapter_render_foundation::RenderFoundationBackend;
use zero_ui_core::geometry::Rect;
use zero_ui_core::geometry::Size as UiSize;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::{ResolvedColorScheme, SystemThemeSnapshot};
use zero_ui_render::paint_scene;
use zero_ui_runtime::app::UiApp;
use zero_ui_runtime::host::WidgetHost;
use zero_ui_runtime::platform::{PlatformRuntime, RuntimeError, UiResult, WindowId};

use crate::driver::WinitDriver;
use crate::event_map::{
    map_cursor_moved, map_ime, map_key_event, map_mouse_input, map_mouse_wheel, map_touch, map_window_metrics,
    to_logical_point, to_logical_size,
};

type RegisterFn = Box<dyn FnOnce(&mut WidgetHost)>;

/// 字体资源（DC-17 FontConfig API）。
///
/// 调用方通过 `WinitRuntime::add_font` 注册字体；run() 启动时按注册顺序加载到
/// `FontdueBackend`，作为 per-character fallback 链（has_char 顺序匹配）。
///
/// - `family`：字体族名（如 "UI" / "CJK"），同时作为 fallback 顺序标识
/// - `data`：字体原始字节。WOFF 容器会自动解码；TTF/OTF 直送后端
/// - `container`：声明容器格式，避免自动探测歧义
#[derive(Clone, Debug)]
pub struct FontAsset {
    pub family: &'static str,
    pub data: &'static [u8],
    pub container: FontContainer,
}

/// 字体容器格式（DC-17 FontConfig API）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontContainer {
    /// 裸 TTF/OTF 字节，直送后端 `load_family`。
    Ttf,
    /// WOFF 容器，需先 `decode_woff` 解包再送后端。
    Woff,
}

/// winit 平台运行时（M1 占位）。
pub struct WinitRuntime {
    system_scheme: ResolvedColorScheme,
    register: Option<RegisterFn>,
    /// 调用方注册的字体资源列表。空时回落到内置默认（Noto Sans + M+ 1P + Ahem），
    /// 保持向后兼容（gallery 之外的 example 不需要改）。
    fonts: Vec<FontAsset>,
}

impl Default for WinitRuntime {
    fn default() -> WinitRuntime {
        WinitRuntime {
            system_scheme: ResolvedColorScheme::Light,
            register: None,
            fonts: Vec::new(),
        }
    }
}

impl WinitRuntime {
    pub fn new() -> WinitRuntime {
        WinitRuntime::default()
    }

    /// 测试/注入用：设置系统主题快照。
    pub fn set_system_scheme(&mut self, scheme: ResolvedColorScheme) {
        self.system_scheme = scheme;
    }

    /// 注册控件工厂闭包，将在 `run()` 中调用（`host.register(...)`）。
    ///
    /// 必须在调用 `run()` 前设置；调用后 consume 内部 Option。
    pub fn set_register(&mut self, register: impl FnOnce(&mut WidgetHost) + 'static) {
        self.register = Some(Box::new(register));
    }

    /// 注册一个字体资源（DC-17 FontConfig API）。
    ///
    /// 调用方按 fallback 顺序依次添加（先注册的优先匹配 has_char）。
    /// 一旦注册任意字体，内置默认字体栈（Noto Sans + M+ 1P + Ahem）不再加载，
    /// 调用方需自行覆盖所有渲染场景的字符集。
    pub fn add_font(&mut self, asset: FontAsset) {
        self.fonts.push(asset);
    }

    /// 真实 `EventLoop::run` 的**可测试 setup 核心**（DC-2 终端阻塞壳前置）。
    ///
    /// 构造 [`WinitDriver`]、经 `register` 闭包注册应用控件工厂、`begin` 产出首帧，返回
    /// driver 供真实 run loop 继续喂事件（`pump_event`/`pump_frame`）。这把建树、工厂注册、
    /// 首帧三步从阻塞的 GUI run loop 中抽离，使其可在无窗口环境单测——解决了「driver 需在
    /// `begin` 前注册工厂」的设计 blocker（`register` 闭包承载应用特定工厂注册）。
    ///
    /// GUI-gated 的 `EventLoop::new` / `Window` / GPU surface 与 `event_loop.run` 包壳是剩余
    /// 运行时件（需 GUI 验证首帧）。真实 `run` 实现会是：
    /// ```text
    /// let mut driver = WinitRuntime::launch(app, metrics, register);
    /// event_loop.run(|ev, _| {
    ///     driver.pump_event(&event_map::map_*(ev));
    ///     driver.pump_frame();
    ///     render(driver.host().scene());  // 经 zero-ui-adapter-render-foundation 光栅
    /// });
    /// ```
    pub fn launch<'app>(
        app: &'app mut dyn UiApp,
        metrics: WindowMetrics,
        register: impl FnOnce(&mut WidgetHost),
    ) -> WinitDriver<'app> {
        let mut driver = WinitDriver::new(app, metrics);
        register(driver.host_mut());
        driver.begin();
        driver
    }
}

impl PlatformRuntime for WinitRuntime {
    fn run(&mut self, app: &mut dyn UiApp) -> UiResult<()> {
        let event_loop = EventLoop::new().map_err(|e| RuntimeError::Platform(e.to_string()))?;

        let window_attrs = Window::default_attributes()
            .with_title("UI SDK")
            .with_inner_size(LogicalSize::new(1024.0, 768.0));

        // SAFETY: 事件循环在本函数内同步运行，app 引用在 run() 返回前始终有效。
        // 事件循环退出后 handler 不再访问 app_ptr，run() 返回即丢弃 handler。
        let app_ptr: *mut dyn UiApp = unsafe { std::mem::transmute::<&mut dyn UiApp, *mut dyn UiApp>(app) };

        let mut ui_text_backend = FontdueBackend::new();
        // 字体加载（DC-17 FontConfig API）：
        // - 若调用方通过 add_font 注册了任意字体 → 仅加载用户列表（调用方负责覆盖字符集）
        // - 否则回落到内置默认栈：Noto Sans（拉丁） + M+ 1P（CJK） + Ahem（fallback 占位）
        //   保持向后兼容，gallery 之外的 example 不需要改。
        let user_fonts = std::mem::take(&mut self.fonts);
        if !user_fonts.is_empty() {
            for asset in &user_fonts {
                load_font_asset(&mut ui_text_backend, asset);
            }
        } else {
            load_default_fonts(&mut ui_text_backend);
        }
        let mut handler = SdkGpuApp {
            window_attrs: Some(window_attrs),
            register: self.register.take(),
            window: None,
            scale_factor: 1.0,
            cursor_pos: (0.0, 0.0),
            app_ptr,
            driver: None,
            gpu: None,
            rf_backend: None,
            text_backend: Arc::new(ui_text_backend),
            font_loader: FontLoader::new(),
            glyph_cache: GlyphCache::new(1024),
            needs_render: true,
        };

        event_loop
            .run_app(&mut handler)
            .map_err(|e| RuntimeError::Platform(e.to_string()))?;

        Ok(())
    }
    fn request_redraw(&mut self, _window: WindowId) {}
    fn set_ime_area(&mut self, _window: WindowId, _rect: Option<Rect>) {}
    fn system_theme(&self) -> SystemThemeSnapshot {
        SystemThemeSnapshot {
            system_scheme: self.system_scheme,
            high_contrast: false,
        }
    }
}

// ── winit 0.30 ApplicationHandler：UI SDK GPU demo ──

struct SdkGpuApp {
    window_attrs: Option<WindowAttributes>,
    register: Option<RegisterFn>,
    window: Option<Arc<Window>>,
    scale_factor: f32,
    cursor_pos: (f64, f64),
    app_ptr: *mut dyn UiApp,
    driver: Option<WinitDriver<'static>>,
    gpu: Option<GpuRenderer>,
    rf_backend: Option<RenderFoundationBackend>,
    text_backend: Arc<FontdueBackend>,
    font_loader: FontLoader,
    glyph_cache: GlyphCache,
    needs_render: bool,
}

/// Ahem 测试字体（证明文本渲染的最小字体）。
const AHEM: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

/// 内置默认字体栈（DC-17：调用方未注册任何字体时使用）。
///
/// - Noto Sans（拉丁，gallery UI 文案）
/// - M+ 1P（CJK，与 Noto Sans 互补 per-char fallback）
/// - Ahem（最后兜底，保证 scene 有 glyph 不全帧空白）
///
/// WOFF 容器先解码为 TTF 字节再送后端 `load_family`。
fn load_default_fonts(backend: &mut FontdueBackend) {
    let noto = include_bytes!("../../../../tests/wpt-runner/wpt-data/fonts/noto/noto-sans-v8-latin-regular.woff");
    match zero_render_foundation::font::decode_woff(noto) {
        Some(ttf) => match backend.load_family("UI", &ttf) {
            Ok(fid) => tracing::debug!(family = "UI", id = ?fid, loaded = backend.len(), "default font loaded"),
            Err(e) => tracing::warn!(family = "UI", error = ?e, "default font load failed"),
        },
        None => tracing::warn!(family = "UI", "default font woff decode failed"),
    }
    let mplus = include_bytes!("../../../../tests/wpt-runner/wpt-data/fonts/mplus-1p-regular.woff");
    match zero_render_foundation::font::decode_woff(mplus) {
        Some(ttf) => match backend.load_family("CJK", &ttf) {
            Ok(fid) => tracing::debug!(family = "CJK", id = ?fid, loaded = backend.len(), "default font loaded"),
            Err(e) => tracing::warn!(family = "CJK", error = ?e, "default font load failed"),
        },
        None => tracing::warn!(family = "CJK", "default font woff decode failed"),
    }
    match backend.load_family("Ahem", AHEM) {
        Ok(fid) => tracing::debug!(family = "Ahem", id = ?fid, loaded = backend.len(), "default font loaded"),
        Err(e) => tracing::warn!(family = "Ahem", error = ?e, "default font load failed"),
    }
}

/// 加载调用方注册的字体（DC-17 FontConfig API）。
///
/// WOFF 容器自动解码；TTF/OTF 直送后端。失败时 tracing::warn 警告但不 panic
/// （P2-10：原 eprintln! 改为结构化日志，便于 release 关闭 / 开发期 RUST_LOG=debug 看明细）。
fn load_font_asset(backend: &mut FontdueBackend, asset: &FontAsset) {
    let bytes: Vec<u8> = match asset.container {
        FontContainer::Ttf => asset.data.to_vec(),
        FontContainer::Woff => match zero_render_foundation::font::decode_woff(asset.data) {
            Some(ttf) => ttf,
            None => {
                tracing::warn!(family = %asset.family, "registered font woff decode failed");
                return;
            }
        },
    };
    match backend.load_family(asset.family, &bytes) {
        Ok(fid) => tracing::debug!(family = %asset.family, id = ?fid, loaded = backend.len(), "registered font loaded"),
        Err(e) => tracing::warn!(family = %asset.family, error = ?e, "registered font load failed"),
    }
}

impl ApplicationHandler<()> for SdkGpuApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Some(attrs) = self.window_attrs.take()
        {
            let win = event_loop.create_window(attrs).expect("Failed to create window");
            let window = Arc::new(win);

            let size = window.inner_size();
            let sf = window.scale_factor() as f32;
            self.scale_factor = sf;

            // GPU 渲染器
            match GpuRenderer::new_for_window(window.clone()) {
                Ok(mut gpu) => {
                    gpu.configure_surface(size.width, size.height);
                    self.gpu = Some(gpu);
                }
                Err(e) => {
                    tracing::error!("GPU renderer init failed: {e}");
                    event_loop.exit();
                    return;
                }
            }

            // UI SDK 驱动器
            let metrics = map_window_metrics(size, sf);
            // SAFETY: app_ptr 来自 run() 的参数 &mut dyn UiApp，该引用在事件循环
            // 退出前始终有效（run() 同步阻塞）。
            let app_ref: &'static mut dyn UiApp = unsafe { &mut *self.app_ptr };
            let mut driver = WinitDriver::new(app_ref, metrics);
            if let Some(register) = self.register.take() {
                register(driver.host_mut());
            }
            // P3-7 核心修复：把 FontdueBackend 注入 WidgetHost，让所有 widget 的
            // layout/paint 都用真实字体度量（修复中文间距乱、关键字染色错、按钮截断等）。
            {
                let measurer = crate::FontdueTextMeasure::new(self.text_backend.clone());
                if let Some((ascent_ratio, descent_ratio)) = measurer.line_metrics_ratio(12.0) {
                    driver.host_mut().set_font_metrics(ascent_ratio, descent_ratio);
                }
                driver.host_mut().set_text_measure(Box::new(measurer));
            }
            driver.begin();
            self.driver = Some(driver);

            // 绘制后端
            let logical = to_logical_size(size, sf);
            let mut backend = RenderFoundationBackend::new_with_text_size(
                UiSize::new(logical.width, logical.height),
                self.text_backend.clone(),
            );
            backend.set_scale_factor(sf);
            self.rf_backend = Some(backend);

            self.window = Some(window);
            // P0-2 (CJK 修复)：启用 IME 事件。winit 0.30 文档明确要求必须调
            // `set_ime_allowed(true)`，否则 WindowEvent::Ime 永远不会被发送——
            // 即使我们在 window_event 里加了 Ime 分支也收不到。这是中文输入的硬前提。
            if let Some(ref w) = self.window {
                w.set_ime_allowed(true);
            }
            self.needs_render = true;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        let sf = self.scale_factor;
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let Some(ref mut gpu) = self.gpu {
                    gpu.configure_surface(size.width, size.height);
                }
                let logical = to_logical_size(size, sf);
                if let Some(ref mut driver) = self.driver {
                    driver.set_metrics(map_window_metrics(size, sf));
                }
                let mut backend = RenderFoundationBackend::new_with_text_size(
                    UiSize::new(logical.width, logical.height),
                    self.text_backend.clone(),
                );
                backend.set_scale_factor(sf);
                self.rf_backend = Some(backend);
                self.needs_render = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let (width, height) = if let Some(ref window) = self.window {
                    let size = window.inner_size();
                    (size.width, size.height)
                } else {
                    return;
                };

                if let Some(ref mut driver) = self.driver {
                    driver.pump_frame();
                    let scene = driver.host().scene();
                    if let Some(ref mut backend) = self.rf_backend {
                        paint_scene(scene, backend);

                        if let Some(ref mut gpu) = self.gpu {
                            let logical = to_logical_size(PhysicalSize::new(width, height), sf);
                            let mut fresh = RenderFoundationBackend::new_with_text_size(
                                UiSize::new(logical.width, logical.height),
                                self.text_backend.clone(),
                            );
                            fresh.set_scale_factor(sf);
                            let (primitives, mut image_cache) =
                                std::mem::replace(backend, fresh).into_primitives_and_cache();

                            let fillrects = scene
                                .entries
                                .iter()
                                .filter(|e| {
                                    matches!(
                                        &e.primitive,
                                        zero_ui_render::render_node::RenderPrimitive::FillRect { .. }
                                    )
                                })
                                .count();
                            let strs = scene
                                .entries
                                .iter()
                                .filter(|e| {
                                    matches!(&e.primitive, zero_ui_render::render_node::RenderPrimitive::Text { .. })
                                })
                                .count();
                            let imgs = scene
                                .entries
                                .iter()
                                .filter(|e| {
                                    matches!(&e.primitive, zero_ui_render::render_node::RenderPrimitive::Image { .. })
                                })
                                .count();
                            let strokes = scene
                                .entries
                                .iter()
                                .filter(|e| {
                                    matches!(
                                        &e.primitive,
                                        zero_ui_render::render_node::RenderPrimitive::StrokeRect { .. }
                                    )
                                })
                                .count();
                            let blobs = scene
                                .entries
                                .iter()
                                .filter(|e| {
                                    matches!(
                                        &e.primitive,
                                        zero_ui_render::render_node::RenderPrimitive::TextBlob { .. }
                                    )
                                })
                                .count();
                            tracing::trace!(
                                scene_entries = scene.entries.len(),
                                fillrects,
                                strs,
                                imgs,
                                strokes,
                                blobs,
                                primitives_fills = primitives.fills.len(),
                                primitives_rounded = primitives.rounded_rects.len(),
                                primitives_images = primitives.images.len(),
                                primitives_glyphs = primitives.glyphs.len(),
                                "render scene stats"
                            );

                            gpu.render_full_scene_gpu(
                                &primitives,
                                &self.font_loader,
                                &mut self.glyph_cache,
                                Some(&mut image_cache),
                                &[],
                                &[],
                                &[],
                                &[],
                                sf,
                            );
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
                if let Some(ref mut driver) = self.driver {
                    let pt = to_logical_point(position, sf);
                    driver.pump_event(&map_cursor_moved(pt, zero_ui_core::event::Modifiers::NONE));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(ref mut driver) = self.driver {
                    let pt = to_logical_point(PhysicalPosition::new(self.cursor_pos.0, self.cursor_pos.1), sf);
                    driver.pump_event(&map_mouse_input(
                        button,
                        state,
                        pt,
                        zero_ui_core::event::Modifiers::NONE,
                    ));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ref mut driver) = self.driver {
                    let pt = to_logical_point(PhysicalPosition::new(self.cursor_pos.0, self.cursor_pos.1), sf);
                    driver.pump_event(&map_mouse_wheel(delta, sf, pt, zero_ui_core::event::Modifiers::NONE));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Touch(touch) => {
                // U-2 修复：触摸事件转 Pointer 事件，让所有 widget 都支持触摸（tap/drag/scroll）。
                // 触摸 Started = Primary 按键 Pressed（等同鼠标左键，供 click-to-focus）。
                if let Some(ref mut driver) = self.driver {
                    driver.pump_event(&map_touch(
                        touch.phase,
                        touch.location,
                        touch.id as u32,
                        sf,
                        zero_ui_core::event::Modifiers::NONE,
                    ));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // U1-2/U3-3 根因修复：键盘事件转发到 host，TextInput/NavSearch 才能输入。
                // 之前 runtime 只处理鼠标/触摸/滚轮，键盘事件被默认 `_ => {}` 吃掉。
                if let Some(ref mut driver) = self.driver {
                    driver.pump_event(&map_key_event(&event, zero_ui_core::event::Modifiers::NONE));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                // IME（中文/日文等复合输入）转发到 host。
                if let Some(ref mut driver) = self.driver {
                    driver.pump_event(&map_ime(ime));
                    self.needs_render = true;
                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor as f32;
                if let Some(ref window) = self.window {
                    let size = window.inner_size();
                    let logical = to_logical_size(size, self.scale_factor);
                    if let Some(ref mut driver) = self.driver {
                        driver.set_metrics(map_window_metrics(size, self.scale_factor));
                    }
                    if let Some(ref mut gpu) = self.gpu {
                        gpu.configure_surface(size.width, size.height);
                    }
                    let mut backend = RenderFoundationBackend::new_with_text_size(
                        UiSize::new(logical.width, logical.height),
                        self.text_backend.clone(),
                    );
                    backend.set_scale_factor(sf);
                    self.rf_backend = Some(backend);
                    self.needs_render = true;
                    window.request_redraw();
                }
            }
            WindowEvent::Focused(true) => {
                self.needs_render = true;
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 连续渲染循环：每帧请求重绘（vsync 节流），避免首次 acquire 失败后永久空白。
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
    use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
    use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::{
        EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget, WidgetId, WidgetSpec,
    };

    #[test]
    fn runtime_stubs_compile_and_default() {
        let mut rt = WinitRuntime::new();
        rt.set_system_scheme(ResolvedColorScheme::Dark);
        rt.request_redraw(WindowId(0));
        rt.set_ime_area(WindowId(0), None);
        assert_eq!(rt.system_theme().system_scheme, ResolvedColorScheme::Dark);
    }

    #[test]
    fn run_rejects_test_thread() {
        // run() 现在创建 winit EventLoop，需要在主线程运行。在测试线程上调用会 panic。
        // 验证错误而非段错误。
        let mut rt = WinitRuntime::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = rt.run(&mut _Noop);
        }));
        assert!(result.is_err(), "run() should panic on non-main thread");
    }

    struct _Noop;
    impl UiApp for _Noop {
        fn root_spec(&self) -> zero_ui_core::widget::WidgetSpec {
            zero_ui_core::widget::WidgetSpec::new("Empty")
        }
        fn dispatch(
            &mut self,
            _action: &zero_ui_core::action::ActionId,
            _payload: Option<zero_ui_core::action::ActionPayload>,
        ) -> zero_ui_core::action::ActionResult {
            zero_ui_core::action::ActionResult::Handled
        }
    }

    // ── launch() 测试用品：最小可交互控件 + 计数 UiApp ──

    /// 占位叶子控件：Pressed emit "box.click"；paint 一个 fill（证明经工厂挂载）。
    struct _Box;
    impl Widget for _Box {
        fn mount(&mut self, _ctx: &mut MountCtx) {}
        fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
        fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
            if let UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } = event
            {
                EventResult::Emit(ActionId::new("box.click"))
            } else {
                EventResult::Ignored
            }
        }
        fn layout(&mut self, _ctx: &mut LayoutCtx, _c: Constraints) -> Size {
            Size::new(100.0, 40.0)
        }
        fn paint(&mut self, ctx: &mut PaintCtx) {
            ctx.recorder
                .fill_rect(Rect::from_ltrb(0.0, 0.0, 100.0, 40.0), Color::rgb(0.3, 0.5, 0.7));
        }
        fn focusable(&self) -> bool {
            true
        }
    }

    /// 计数 "box.click" 的最小 UiApp（证明 launch 返回的 driver 能驱动 reducer）。
    struct _App {
        clicks: u32,
    }
    impl UiApp for _App {
        fn root_spec(&self) -> WidgetSpec {
            let mut s = WidgetSpec::new("Box");
            s.id = Some(WidgetId::new("box"));
            s
        }
        fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
            if action.0.as_str() == "box.click" {
                self.clicks += 1;
                ActionResult::Handled
            } else {
                ActionResult::UnknownAction(action.clone())
            }
        }
    }

    fn press_at(x: f32, y: f32) -> UiEvent {
        UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Primary),
            position: Point::new(x, y),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }
    }

    #[test]
    fn launch_registers_factories_before_begin_and_returns_drivable_driver() {
        // DC-2 终端阻塞壳的可测试 setup 核心：launch = driver 构造 + register 工厂 + begin 首帧。
        let mut app = _App { clicks: 0 };
        // launch 返回的 driver 持 &mut app → app.clicks 在作用域外读（下方最终断言）。
        {
            let mut driver = WinitRuntime::launch(&mut app, WindowMetrics::desktop(), |host| {
                host.register("Box", |_spec| Box::new(_Box));
            });
            // register 在 begin 前调用 → _Box 挂载 + 首帧 paint → scene 非空。
            assert!(
                !driver.host().scene().entries.is_empty(),
                "launch 注册工厂在 begin 前 → 首帧非空 scene"
            );
            // 返回的 driver 可继续驱动：点击 → emit → reducer → Handled → 重建。
            let out = driver.pump_event(&press_at(10.0, 10.0));
            assert_eq!(out.emitted_actions, 1);
            assert!(out.spec_rebuilt, "Handled → driver 重建声明树");
            driver.pump_frame();
        }
        assert_eq!(app.clicks, 1, "reducer 被 launch 返回的 driver 驱动");
    }
}
