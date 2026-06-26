// 平台相关独立函数（从 app.rs 通过 include! 引入）
//
// 此文件在编译时被 app.rs include，共享同一个模块作用域。

// ── BrowserApp GPU/CPU 渲染 impl ──────────────────────────────────

impl BrowserApp {
    /// Wayland 上是否强制使用 CPU softbuffer present（规避 wgpu swapchain 与 winit CSD 冲突）
    pub fn wayland_forces_cpu_present(&self) -> bool {
        is_wayland() && matches!(self.render_mode, RenderMode::Gpu | RenderMode::Auto)
    }

    /// 初始化 GPU 渲染器（Wayland 上跳过 wgpu 窗口 surface，改走 CPU present）
    pub fn init_gpu(&mut self, window: &std::sync::Arc<winit::window::Window>) {
        if matches!(self.render_mode, RenderMode::Cpu) || self.wayland_forces_cpu_present() {
            if self.wayland_forces_cpu_present() {
                tracing::warn!(
                    "Wayland: wgpu window surface disabled (focus-switch crash); using CPU softbuffer present"
                );
            }
            return;
        }

        match GpuRenderer::new_for_window(std::sync::Arc::clone(window)) {
            Ok(renderer) => {
                tracing::info!("GPU renderer initialized (format: {:?})", renderer.surface_format());
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                if matches!(self.render_mode, RenderMode::Gpu) {
                    tracing::error!("GPU renderer init failed: {e}");
                } else {
                    tracing::warn!("GPU renderer init failed: {e}; using CPU renderer");
                }
            }
        }
    }

    /// 窗口失焦：Wayland 上销毁 GPU 渲染器，避免 swapchain 在失焦后 commit
    pub fn on_window_unfocused(&mut self) {
        if is_wayland() {
            if self.gpu_renderer.is_some() {
                tracing::debug!("Wayland unfocus: releasing GPU renderer");
                self.gpu_renderer = None;
                self.surface_configured = false;
            }
        } else {
            self.suspend_gpu_present();
        }
    }

    /// 初始化 CPU 软件渲染 surface
    pub fn init_cpu_surface(
        &mut self,
        window: &std::sync::Arc<winit::window::Window>,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
    ) {
        if cpu_surface.is_some() {
            return;
        }

        match softbuffer::Context::new(std::sync::Arc::clone(window))
            .and_then(|context| softbuffer::Surface::new(&context, std::sync::Arc::clone(window)))
        {
            Ok(surface) => {
                tracing::info!("CPU renderer initialized");
                *cpu_surface = Some(surface);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(err) => {
                tracing::error!("CPU renderer init failed: {err}");
            }
        }
    }

    /// 同步 IME 状态（Wayland 失焦时必须关闭，否则 subsurface commit 会导致 compositor 断开）
    pub fn sync_ime_state(&self, window: &winit::window::Window) {
        use winit::dpi::{LogicalPosition, LogicalSize};

        let needs_ime = self.window_focused && (self.address_bar_focused || self.shell.find_state().is_active());
        window.set_ime_allowed(needs_ime);

        if !needs_ime {
            return;
        }

        if self.address_bar_focused {
            let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
            let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
            let bar_y = layout::TAB_STRIP_HEIGHT + layout::ADDRESS_BAR_INPUT_V_INSET;
            window.set_ime_cursor_area(
                LogicalPosition::new(bar_x, bar_y),
                LogicalSize::new(480.0, layout::ADDRESS_BAR_HEIGHT),
            );
        } else if self.shell.find_state().is_active() {
            window.set_ime_cursor_area(
                LogicalPosition::new(8.0, layout::TOOLBAR_HEIGHT + 4.0),
                LogicalSize::new(240.0, layout::FIND_BAR_HEIGHT),
            );
        }
    }

    /// 失焦时暂停 GPU swapchain present（非 Wayland，Wayland 直接销毁 renderer）
    pub fn suspend_gpu_present(&mut self) {
        if is_wayland() {
            return;
        }
        if let Some(gpu) = self.gpu_renderer_as_mut() {
            gpu.suspend_present();
        }
    }

    /// 获焦后恢复 GPU swapchain present（非 Wayland）
    pub fn resume_gpu_present(&mut self) {
        if is_wayland() {
            return;
        }
        if let Some(gpu) = self.gpu_renderer_as_mut() {
            gpu.resume_present();
        }
    }

    /// GPU 渲染一帧（使用全量 13 种图元管线）
    pub fn render_frame(&mut self, width: u32, height: u32, present: bool) {
        if !present || !self.window_focused {
            return;
        }
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            if renderer.is_present_suspended() {
                self.gpu_renderer = gpu;
                return;
            }
            let (fills, glyphs, overlay_fills, overlay_glyphs) = self.build_scene(width, height);

            // 获取 WebView 额外图元（渐变、阴影、圆角矩形、线段、路径等）
            let webview_extras = self.get_webview_extra_primitives();

            // 合并 chrome fills + webview 图元
            let mut scene_primitives = webview_extras;
            scene_primitives.fills = [fills, scene_primitives.fills].concat();

            // 取活跃标签页 webview 的 ImageCache，供渲染器绘制 <img> 图元
            // （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）
            // self.shell / self.webviews / self.font_loader / self.glyph_cache 为不相交字段借用
            let image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
                Some(id) => self.tabs.image_cache_mut(id),
                None => None,
            };

            // 使用全量 GPU 渲染管线
            renderer.render_full_scene_gpu(
                &scene_primitives,
                &self.font_loader,
                &mut self.glyph_cache,
                image_cache,
                &overlay_fills,
                &overlay_glyphs.iter().chain(glyphs.iter()).cloned().collect::<Vec<_>>(),
                1.0, // scale_factor: GPU 渲染器内部已通过 surface 尺寸处理
            );
        }
        self.gpu_renderer = gpu;
    }

    /// CPU 软件渲染一帧（`present` 为 false 时跳过）
    pub fn render_cpu(
        &mut self,
        width: u32,
        height: u32,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
        present: bool,
    ) {
        if !present {
            return;
        }

        let (fills, glyphs, overlay_fills, overlay_glyphs) = self.build_scene(width, height);

        // 获取 WebView 的额外图元类型（渐变、阴影、线段等）
        let webview_extras = self.get_webview_extra_primitives();

        // 合并：chrome fills + webview fills (已在 fills 中) + webview 额外图元
        let mut scene_primitives = webview_extras;
        // fills 和 glyphs 已通过 append_webview_primitives 混入 chrome 的 fills/glyphs
        // 所以只需把 chrome fills 放入 scene_primitives.fills 的前面
        scene_primitives.fills = [fills, scene_primitives.fills].concat();

        // 取活跃标签页 webview 的 ImageCache，供渲染器绘制 <img> 图元
        // （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）
        // self.shell / self.webviews / self.font_loader / self.glyph_cache 为不相交字段借用
        let image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };

        let fb = render_full_scene(
            width,
            height,
            1.0,
            &scene_primitives,
            &self.font_loader,
            &mut self.glyph_cache,
            image_cache,
            // overlay_fills: chrome overlay（上下文菜单背景、圆角遮罩等）
            &overlay_fills,
            // overlay_glyphs: chrome overlay 文字 + 所有 GlyphDraw 文字
            // （chrome glyphs 和 webview glyphs 都是 GlyphDraw 格式，通过 append_webview_primitives 混合）
            // 先渲染 overlay_glyphs（chrome overlay 文字），再追加 glyphs（chrome + webview 文字）
            // 由于 Vec 需要合并，直接拼接
            &overlay_glyphs.iter().chain(glyphs.iter()).cloned().collect::<Vec<_>>(),
        );
        present_rgba_to_softbuffer(cpu_surface, fb.width, fb.height, &fb.data);
    }

    /// 测试用：与 `render_cpu` 相同的场景装配（chrome + WebView 图元 + 活跃标签页
    /// WebView 的 ImageCache），但返回 FrameBuffer 而非 present 到 softbuffer 表面。
    ///
    /// 用于验证浏览器渲染路径消费 webview ImageCache 绘制 `<img>` 图元
    /// （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）。
    #[cfg(test)]
    pub fn render_full_scene_with_webview_for_test(
        &mut self,
        width: u32,
        height: u32,
    ) -> zero_render_foundation::surface::FrameBuffer {
        let (fills, glyphs, overlay_fills, overlay_glyphs) = self.build_scene(width, height);
        let webview_extras = self.get_webview_extra_primitives();
        let mut scene_primitives = webview_extras;
        scene_primitives.fills = [fills, scene_primitives.fills].concat();

        // 与 render_cpu / render_frame 完全一致的 ImageCache 装配（不相交字段借用）
        let image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };

        render_full_scene(
            width,
            height,
            1.0,
            &scene_primitives,
            &self.font_loader,
            &mut self.glyph_cache,
            image_cache,
            &overlay_fills,
            &overlay_glyphs
                .iter()
                .chain(glyphs.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

// ── 平台独立函数 ──────────────────────────────────

/// 当前进程是否运行在 Wayland 上
pub fn is_wayland() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("WINIT_UNIX_BACKEND")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// macOS 一体化标题栏模式
pub fn uses_unified_titlebar() -> bool {
    #[cfg(target_os = "macos")]
    {
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// 将 RGBA 像素提交到 softbuffer 表面
pub(crate) fn present_rgba_to_softbuffer(
    cpu_surface: &mut Option<
        softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
    >,
    width: u32,
    height: u32,
    rgba: &[u8],
) {
    use std::num::NonZeroU32;

    let Some(surface) = cpu_surface.as_mut() else {
        return;
    };

    let sw = match NonZeroU32::new(width.max(1)) {
        Some(w) => w,
        None => return,
    };
    let sh = match NonZeroU32::new(height.max(1)) {
        Some(h) => h,
        None => return,
    };

    if let Err(err) = surface.resize(sw, sh) {
        tracing::error!("CPU surface resize failed: {err}");
        return;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(err) => {
            tracing::error!("CPU surface buffer failed: {err}");
            return;
        }
    };

    for (dst, chunk) in buffer.iter_mut().zip(rgba.chunks_exact(4)) {
        *dst = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
    }

    if let Err(err) = buffer.present() {
        tracing::error!("CPU surface present failed: {err}");
    }
}

/// 将以 `/` 开头的根相对路径解析到当前标签页 URL（无可用 base 时原样返回）。
pub fn resolve_path_relative_url(input: &str, shell: &BrowserShell) -> String {
    let input = input.trim();
    if !input.starts_with('/') || input.starts_with("//") {
        return input.to_string();
    }
    let Some(base) = shell.active_tab().and_then(|tab| {
        let url = tab.url()?;
        if url.starts_with("zero://") {
            None
        } else {
            Some(url)
        }
    }) else {
        return input.to_string();
    };
    zero_engine::resolve_document_url(base, input)
}

/// URL 规范化 — 支持 URL 和搜索引擎回退
pub fn normalize_url(input: &str, shell: &BrowserShell) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.starts_with("ftp://") || input.starts_with("file://") || input.starts_with("data:") {
        return input.to_string();
    }
    if input.starts_with("zero://") {
        return input.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{input}");
    }
    shell.settings().search(input)
}

/// Chrome UI 主字体候选路径（按平台 OS Citizenship 优先级，与 Chromium 一致）。
fn chrome_ui_primary_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            // San Francisco（macOS 系统 UI 字体，Chrome 同源）
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFCompact.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &[
            // Segoe UI（Windows 系统 UI 字体）
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            // GTK/Fontconfig 常见 UI sans（Linux 桌面 Chrome 经 fontconfig 解析的同类字体）
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf",
            "/usr/share/fonts/truetype/cantarell/Cantarell-Regular.otf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ]
    }
}

/// 加载系统字体（主字体 + CJK/Emoji 回退链）
pub fn load_system_fonts(font_loader: &mut FontLoader) -> Option<u32> {
    let primary_paths = chrome_ui_primary_font_paths();

    let (primary, loaded_path) = primary_paths.iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = font_loader.load_font(&data).ok()?;
        Some((id, *path))
    })?;
    tracing::info!("Chrome UI primary font: {loaded_path} (id={primary})");

    #[cfg(target_os = "macos")]
    let bold_paths = [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    #[cfg(target_os = "windows")]
    let bold_paths = [
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "C:\\Windows\\Fonts\\segoeuib.ttf",
    ];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let bold_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    ];
    if let Some((bold_id, bold_path)) = bold_paths.iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = font_loader.load_font(&data).ok()?;
        Some((id, *path))
    }) {
        tracing::info!("Bold UI font: {bold_path} (id={bold_id})");
    }

    #[cfg(target_os = "macos")]
    let fallback_paths = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Apple Symbols.ttf",
    ];
    #[cfg(target_os = "windows")]
    let fallback_paths = ["C:\\Windows\\Fonts\\msyh.ttc", "C:\\Windows\\Fonts\\seguiemj.ttf"];
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let fallback_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
    ];

    let mut fallbacks = Vec::new();
    for path in fallback_paths {
        if path.contains("NotoColorEmoji") {
            // fontdue 无法 rasterize CBDT/CBLC 彩色 emoji 字体，跳过以免浪费内存
            continue;
        }
        match std::fs::read(path) {
            Ok(data) => match font_loader.load_font(&data) {
                Ok(id) if id != primary => {
                    tracing::info!("Loaded fallback font: {path} (id={id})");
                    fallbacks.push(id);
                }
                Ok(_) => {}
                Err(e) => tracing::debug!("Failed to load fallback font {path}: {e}"),
            },
            Err(e) => tracing::debug!("Fallback font not found {path}: {e}"),
        }
    }
    font_loader.set_fallback_chain(fallbacks);
    tracing::info!("Font fallback chain: {} fonts", font_loader.fallback_chain().len());

    Some(primary)
}

/// 环境变量 `ZERO_BROWSER_COLOR_SCHEME` 覆盖（`dark` / `light`）。
pub fn color_scheme_from_env() -> Option<PrefersColorSchemeValue> {
    let val = std::env::var("ZERO_BROWSER_COLOR_SCHEME").ok()?;
    if val.eq_ignore_ascii_case("dark") {
        Some(PrefersColorSchemeValue::Dark)
    } else if val.eq_ignore_ascii_case("light") {
        Some(PrefersColorSchemeValue::Light)
    } else {
        tracing::debug!("Ignoring unrecognized ZERO_BROWSER_COLOR_SCHEME={val:?}, expected dark or light");
        None
    }
}

/// 解析 `gsettings get org.gnome.desktop.interface color-scheme` 输出。
fn parse_gnome_color_scheme_stdout(stdout: &str) -> Option<PrefersColorSchemeValue> {
    let value = stdout.trim();
    if value.contains("prefer-dark") {
        return Some(PrefersColorSchemeValue::Dark);
    }
    if value.contains("prefer-light") || value.contains("'default'") || value.contains("'light'") {
        return Some(PrefersColorSchemeValue::Light);
    }
    None
}

fn detect_linux_gnome_color_scheme() -> Option<PrefersColorSchemeValue> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_gnome_color_scheme_stdout(&String::from_utf8_lossy(&output.stdout))
}

/// 检测系统深色/浅色模式（Linux 优先 gsettings，可用 `ZERO_BROWSER_COLOR_SCHEME` 覆盖）。
///
/// 仅在明确识别为深色时返回 [`PrefersColorSchemeValue::Dark`]，无法识别时默认亮色。
pub fn detect_system_color_scheme() -> PrefersColorSchemeValue {
    if let Some(scheme) = color_scheme_from_env() {
        return scheme;
    }

    if let Some(scheme) = detect_linux_gnome_color_scheme() {
        return scheme;
    }

    tracing::debug!("System color scheme unrecognized, defaulting to light");
    PrefersColorSchemeValue::Light
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::font::loader::FontLoader;

    #[test]
    fn load_system_fonts_loads_primary() {
        let mut loader = FontLoader::new();
        assert!(
            load_system_fonts(&mut loader).is_some(),
            "expected at least one Chrome UI font on this platform"
        );
    }

    #[test]
    fn chrome_ui_primary_paths_non_empty() {
        assert!(!chrome_ui_primary_font_paths().is_empty());
    }

    #[test]
    fn parse_gnome_color_scheme_prefers_dark() {
        assert_eq!(
            parse_gnome_color_scheme_stdout("'prefer-dark'\n"),
            Some(PrefersColorSchemeValue::Dark)
        );
    }

    #[test]
    fn parse_gnome_color_scheme_prefers_light_and_default() {
        assert_eq!(
            parse_gnome_color_scheme_stdout("'prefer-light'\n"),
            Some(PrefersColorSchemeValue::Light)
        );
        assert_eq!(
            parse_gnome_color_scheme_stdout("'default'\n"),
            Some(PrefersColorSchemeValue::Light)
        );
    }

    #[test]
    fn parse_gnome_color_scheme_unrecognized_returns_none() {
        assert_eq!(parse_gnome_color_scheme_stdout(""), None);
        assert_eq!(parse_gnome_color_scheme_stdout("invalid\n"), None);
    }

    /// 验证浏览器渲染路径消费活跃标签页 WebView 的 ImageCache 绘制 `<img>` 图元
    /// （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）。
    ///
    /// 差异法：基线（ImageCache 为空）→ 图片颜色应为 0；填充缓存（键与 engine 生成的
    /// `simple_hash(src)` 一致）后 → 图片颜色应出现 > 0。证明 webview ImageCache 经
    /// 浏览器 render 路径传入渲染器并被消费。
    #[test]
    fn render_path_consumes_webview_image_cache() {
        use zero_render_foundation::image_cache::{ImageData, ImageKey};
        use zero_engine::simple_hash;

        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.new_tab(None);
        let tab_id = app.shell.active_tab_id().expect("active tab");

        // 页面含一个 40x40 的 <img>；engine 用 simple_hash(src) 生成 image_key
        let src = "r215-wiring.png";
        let html = format!(
            "<img src=\"{src}\" style=\"display:block;width:40px;height:40px\">"
        );
        app.load_webview_html(tab_id, &html, None);

        // 区别于 chrome UI 与白色背景的鲜明颜色
        let (pr, pg, pb, pa) = (220u8, 30, 180, 255);
        let pixels = [pr, pg, pb, pa].repeat(40 * 40);
        let img = ImageData::from_rgba(pixels, 40, 40).unwrap();

        // 基线：ImageCache 为空 → 缓存 miss → 图片不被绘制 → 该颜色计数为 0
        let fb0 = app.render_full_scene_with_webview_for_test(800, 600);
        let count0 = count_color(&fb0, pr, pg, pb, pa);
        assert_eq!(count0, 0, "baseline: image color must be absent when cache empty");

        // 填充活跃标签页 WebView 的 ImageCache（键 = simple_hash(src)，与 engine 一致）
        app.tabs
            .image_cache_mut(tab_id)
            .expect("tab snapshot")
            .insert_with_key(ImageKey::new(simple_hash(src)), img);

        // 装配后渲染：image_cache 经浏览器渲染路径传入渲染器 → 图片颜色应出现
        let fb1 = app.render_full_scene_with_webview_for_test(800, 600);
        let count1 = count_color(&fb1, pr, pg, pb, pa);
        assert!(
            count1 > 0,
            "after populating cache, image color must be drawn (got 0 pixels)"
        );
    }

    /// 统计 FrameBuffer 中精确匹配某 RGBA 颜色的像素数。
    fn count_color(
        fb: &zero_render_foundation::surface::FrameBuffer,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> usize {
        (0..fb.height)
            .flat_map(|y| (0..fb.width).map(move |x| (x, y)))
            .filter(|(x, y)| {
                let p = fb.get_pixel(*x, *y);
                p[0] == r && p[1] == g && p[2] == b && p[3] == a
            })
            .count()
    }
}
