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
            let nav_w = layout::NAV_SECTION_LEADING_PAD
                + layout::NAV_BUTTON_WIDTH * 4.0
                + layout::NAV_SECTION_TRAILING_GAP;
            let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
            let bar_y = layout::TAB_STRIP_HEIGHT + layout::ADDRESS_BAR_INPUT_V_INSET;
            window.set_ime_cursor_area(
                LogicalPosition::new(bar_x, bar_y),
                LogicalSize::new(480.0, layout::ADDRESS_BAR_HEIGHT),
            );
        } else if self.shell.find_state().is_active() {
            let (bar_x, bar_y, bar_w, bar_h) =
                self.find_bar_rect_for(self.physical_size.0, self.physical_size.1);
            let s = self.scale_factor;
            window.set_ime_cursor_area(
                LogicalPosition::new(bar_x / s, bar_y / s),
                LogicalSize::new(bar_w / s, bar_h / s),
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
            let scene = self.build_scene(width, height);
        let ChromeScene {
            chrome_fills,
            chrome_glyphs,
            page_fills,
            page_glyphs,
            chrome_overlay_fills,
            chrome_overlay_glyphs,
            overlay_fills,
            overlay_glyphs,
            chrome_shadows,
            overlay_rounded_rects,
        } = scene;

            // 获取 WebView 额外图元（渐变、阴影、圆角矩形、线段、路径等）
            // DC-3 phase-2（feature `sdk-chrome`）：webview_extras 作为 surface 注册，
            // 手绘 chrome_fills/glyphs 被 SDK chrome 替换。
            #[cfg(feature = "sdk-chrome")]
            let _ = (&chrome_fills, &chrome_glyphs); // 手绘 chrome 被 SDK 替换

            let webview_extras = self.get_webview_extra_primitives();

            // 合并 chrome fills + chrome shadows + webview 图元。
            // DC-3 phase-2（feature `sdk-chrome`）：webview_extras 作为 surface 注册到 bridge，
            // 由 draw_external_surface 合成，不直接拼入 scene_primitives。
            #[cfg(not(feature = "sdk-chrome"))]
            let mut scene_primitives = {
                let mut p = webview_extras;
                p.shadows = [chrome_shadows, p.shadows].concat();
                p
            };
            #[cfg(feature = "sdk-chrome")]
            let mut scene_primitives = {
                // DC-14: SDK chrome 不继承手绘 chrome 的 viewport drop shadow（不同布局几何）。
                // 手绘 chrome 的 page_bg（app_render.rs step 9）会覆盖该阴影内部，但 SDK 替换
                // 路径丢弃了 chrome_fills，致 chrome_shadows 的 viewport 阴影内部染灰整片页面
                // （DC-14 page-region 99.70% diff 根因，2026-07-04 诊断）。故 SDK 路径不带
                // chrome_shadows；真实 chrome Widget 落地后由组件自身画正确阴影。
                let _ = &chrome_shadows;
                RenderPrimitives::default()
            };

            // 取活跃标签页 webview 的 ImageCache，供渲染器绘制 <img> 图元
            // （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）
            // self.shell / self.webviews / self.font_loader / self.glyph_cache 为不相交字段借用
            #[cfg_attr(not(feature = "sdk-chrome"), allow(unused_mut))]
            let mut image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
                Some(id) => self.tabs.image_cache_mut(id),
                None => None,
            };

            // DC-14 SDK chrome 替换手绘 chrome（feature `sdk-chrome`）。
            // DC-3 phase-2（webview surface 变体）：webview_extras 作为 surface 注册，
            // draw_external_surface 在 ExternalSurface marker 位置合成页面内容；
            // page_fills 进入 surface→返回空 Vec；page_glyphs 保留在渲染管线原有路径。
            // feature-off：手绘 chrome 路径不变（bit-identical）。
            #[cfg(feature = "sdk-chrome")]
            let (page_fills, page_glyphs) = {
                compose_sdk_chrome_replacement_with_webview(
                    &self.shell, width, height,
                    page_fills, page_glyphs,
                    Some(webview_extras), // DC-3 phase-2: webview surface
                    &mut scene_primitives, &mut image_cache,
                )
            };

            // DC-14：拼接 fills/glyphs。
            // feature-off：chrome 主层 → 页面内容 → chrome 浮层（bit-identical）。
            // feature-on：跳过 chrome 主层（SDK chrome 已在 scene_primitives.fills 最底层）。
            #[cfg(not(feature = "sdk-chrome"))]
            let fills = [chrome_fills, page_fills, chrome_overlay_fills].concat();
            #[cfg(not(feature = "sdk-chrome"))]
            let glyphs = [chrome_glyphs, page_glyphs, chrome_overlay_glyphs].concat();
            #[cfg(feature = "sdk-chrome")]
            let fills = [page_fills, chrome_overlay_fills].concat();
            #[cfg(feature = "sdk-chrome")]
            let glyphs = [page_glyphs, chrome_overlay_glyphs].concat();

            scene_primitives.fills = [fills, scene_primitives.fills].concat();

            // 使用全量 GPU 渲染管线
            renderer.render_full_scene_gpu(
                &scene_primitives,
                &self.font_loader,
                &mut self.glyph_cache,
                image_cache,
                &glyphs,
                &overlay_fills,
                &overlay_glyphs,
                &overlay_rounded_rects,
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

        let scene = self.build_scene(width, height);
        let ChromeScene {
            chrome_fills,
            chrome_glyphs,
            page_fills,
            page_glyphs,
            chrome_overlay_fills,
            chrome_overlay_glyphs,
            overlay_fills,
            overlay_glyphs,
            chrome_shadows,
            overlay_rounded_rects,
        } = scene;

        // DC-3 phase-2（feature `sdk-chrome`）：webview_extras 作为 surface 注册，
        // 手绘 chrome 被 SDK chrome 替换。
        #[cfg(feature = "sdk-chrome")]
        let _ = (&chrome_fills, &chrome_glyphs); // 手绘 chrome 被 SDK 替换

        // 获取 WebView 的额外图元类型（渐变、阴影、线段等）
        let webview_extras = self.get_webview_extra_primitives();

        // 合并：chrome fills + chrome shadows + webview fills + webview 额外图元。
        // DC-3 phase-2（feature `sdk-chrome`）：webview_extras 作为 surface 注册到 bridge。
        #[cfg(not(feature = "sdk-chrome"))]
        let mut scene_primitives = {
            let mut p = webview_extras;
            p.shadows = [chrome_shadows, p.shadows].concat();
            p
        };
        #[cfg(feature = "sdk-chrome")]
        let mut scene_primitives = {
            // DC-14: SDK 路径不带手绘 chrome viewport drop shadow（见 render_frame 注释）。
            let _ = &chrome_shadows;
            RenderPrimitives::default()
        };

        // 取活跃标签页 webview 的 ImageCache，供渲染器绘制 <img> 图元
        // （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）
        // self.shell / self.webviews / self.font_loader / self.glyph_cache 为不相交字段借用
        #[cfg_attr(not(feature = "sdk-chrome"), allow(unused_mut))]
        let mut image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };

        // DC-14 SDK chrome 替换手绘 chrome（feature `sdk-chrome`）。
        // DC-3 phase-2（webview surface 变体）：webview_extras 作为 surface 注册。
        // feature-off：手绘 chrome 路径不变（bit-identical）。
        #[cfg(feature = "sdk-chrome")]
        let (page_fills, page_glyphs) = {
            compose_sdk_chrome_replacement_with_webview(
                &self.shell, width, height,
                page_fills, page_glyphs,
                Some(webview_extras), // DC-3 phase-2: webview surface
                &mut scene_primitives, &mut image_cache,
            )
        };

        // DC-14：拼接 fills/glyphs。
        // feature-off：chrome 主层 → 页面内容 → chrome 浮层（bit-identical）。
        // feature-on：跳过 chrome 主层（SDK chrome 已在 scene_primitives.fills 最底层）。
        #[cfg(not(feature = "sdk-chrome"))]
        let fills = [chrome_fills, page_fills, chrome_overlay_fills].concat();
        #[cfg(not(feature = "sdk-chrome"))]
        let glyphs = [chrome_glyphs, page_glyphs, chrome_overlay_glyphs].concat();
        #[cfg(feature = "sdk-chrome")]
        let fills = [page_fills, chrome_overlay_fills].concat();
        #[cfg(feature = "sdk-chrome")]
        let glyphs = [page_glyphs, chrome_overlay_glyphs].concat();

        scene_primitives.fills = [fills, scene_primitives.fills].concat();

        let fb = render_full_scene(
            width,
            height,
            1.0,
            &scene_primitives,
            &self.font_loader,
            &mut self.glyph_cache,
            image_cache,
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
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
        let scene = self.build_scene(width, height);
        let ChromeScene {
            chrome_fills,
            chrome_glyphs,
            page_fills,
            page_glyphs,
            chrome_overlay_fills,
            chrome_overlay_glyphs,
            overlay_fills,
            overlay_glyphs,
            chrome_shadows,
            overlay_rounded_rects,
        } = scene;
        // DC-14：feature-off 按 chrome 主层 → 页面内容 → chrome 浮层顺序拼接 fills/glyphs，
        // 与历史单 fills/glyphs（chrome+页面+autocomplete/floating）逐位等价（bit-identical）。
        let fills = [chrome_fills, page_fills, chrome_overlay_fills].concat();
        let glyphs = [chrome_glyphs, page_glyphs, chrome_overlay_glyphs].concat();
        let webview_extras = self.get_webview_extra_primitives();
        let mut scene_primitives = webview_extras;
        scene_primitives.fills = [fills, scene_primitives.fills].concat();
        scene_primitives.shadows = [chrome_shadows, scene_primitives.shadows].concat();

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
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
        )
    }

    /// 测试用（feature `sdk-chrome`）：与 `render_cpu` 的 **feature-on 装配逐位一致**
    /// （`build_scene` → `compose_sdk_chrome_replacement_with_webview` 用 SDK chrome 替换
    /// 手绘 chrome + webview surface 合成 → `render_full_scene`），但返回 [`FrameBuffer`]
    /// 而非 present 到 softbuffer 表面。
    ///
    /// **DC-14 headless 可视验收通道**：把 SDK chrome **替换式迁移**（`render_cpu` 真实消费
    /// 的路径）的完整浏览器帧（SDK chrome bars + 页面 surface 合成 + overlays）在无 GUI
    /// 环境光栅为像素。既有 `compose_overlay_rasterizes_to_visible_framebuffer` 只覆盖
    /// additive overlay 路径；`compose_replacement_*` 只验证 scene 构造（未光栅）——本方法
    /// 闭合「替换路径完整帧 → 像素」的 headless 验收缺口。
    #[cfg(all(test, feature = "sdk-chrome"))]
    pub fn render_full_scene_sdk_chrome_for_test(
        &mut self,
        width: u32,
        height: u32,
    ) -> zero_render_foundation::surface::FrameBuffer {
        let scene = self.build_scene(width, height);
        let ChromeScene {
            chrome_fills,
            chrome_glyphs,
            page_fills,
            page_glyphs,
            chrome_overlay_fills,
            chrome_overlay_glyphs,
            overlay_fills,
            overlay_glyphs,
            chrome_shadows,
            overlay_rounded_rects,
        } = scene;
        // SDK chrome 替换手绘 chrome 主层（与 render_cpu feature-on 一致）。
        let _ = (chrome_fills, chrome_glyphs);
        let webview_extras = self.get_webview_extra_primitives();
        let mut scene_primitives = {
            // DC-14: SDK 路径不带手绘 chrome viewport drop shadow（见 render_frame 注释）。
            let _ = &chrome_shadows;
            RenderPrimitives::default()
        };
        let mut image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };
        let (page_fills, page_glyphs) = compose_sdk_chrome_replacement_with_webview(
            &self.shell,
            width,
            height,
            page_fills,
            page_glyphs,
            Some(webview_extras),
            &mut scene_primitives,
            &mut image_cache,
        );
        let fills = [page_fills, chrome_overlay_fills].concat();
        let glyphs = [page_glyphs, chrome_overlay_glyphs].concat();
        scene_primitives.fills = [fills, scene_primitives.fills].concat();
        render_full_scene(
            width,
            height,
            1.0,
            &scene_primitives,
            &self.font_loader,
            &mut self.glyph_cache,
            image_cache,
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
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

/// 读取首个可用的 Chrome UI 主字体原始字节 + 其路径（DC-11 字体共享入口）。
///
/// render-foundation `FontLoader` 与 foundation/text `FontdueBackend` 经本函数取同一份字体
/// 字节，使 UI SDK chrome 文本（`render_chrome_via_sdk`）与浏览器页面文本共享字体数据 →
/// 字形一致（spec FR-014 / DC-11）。本函数只读字体文件，不触碰 render-foundation 字体栈，
/// 故不影响页面渲染（无 product-smoke 风险）。
pub fn chrome_ui_primary_font_data() -> Option<(Vec<u8>, &'static str)> {
    chrome_ui_primary_font_paths().iter().find_map(|path| {
        let data = std::fs::read(path).ok().filter(|d| !d.is_empty())?;
        Some((data, *path))
    })
}

/// 加载系统字体（主字体 + CJK/Emoji 回退链）
pub fn load_system_fonts(font_loader: &mut FontLoader) -> Option<u32> {
    let (primary_data, loaded_path) = chrome_ui_primary_font_data()?;
    let primary = font_loader.load_font(&primary_data).ok()?;
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

// ── DC-14 SDK chrome 接线（feature `sdk-chrome`，默认关闭）──────────────────────
//
// 两个接线模式：
// - `compose_sdk_chrome_overlay`：additive overlay（SDK chrome 叠在手绘 chrome 之上，灰度验证用）。
// - `compose_sdk_chrome_replacement`：替换式迁移（跳过手绘 chrome，SDK chrome 整体替换 chrome 层；
//   页面内容从手绘 chrome viewport 位置翻译到 SDK chrome viewport 位置；SDK chrome fills 置于最底层，
//   页面内容在上层覆盖 viewport 区域）。

/// 懒构造 + 缓存 SDK chrome 文本用的共享 `FontdueBackend`（DC-11 字体共享）。
///
/// 经 [`chrome_ui_primary_font_data`] 读取系统主字体字节喂 foundation/text `FontdueBackend`，
/// 进程内 OnceLock 缓存（避免每帧重复 parse 字体）。无系统字体时返回空 backend（chrome 文本
/// no-op，几何仍渲染）。
#[cfg(feature = "sdk-chrome")]
fn sdk_font_backend() -> std::sync::Arc<zero_text_foundation::FontdueBackend> {
    use std::sync::{Arc, OnceLock};
    static BACKEND: OnceLock<Arc<zero_text_foundation::FontdueBackend>> = OnceLock::new();
    BACKEND
        .get_or_init(|| {
            let mut backend = zero_text_foundation::FontdueBackend::new();
            if let Some((data, _path)) = chrome_ui_primary_font_data() {
                let _ = backend.load_family("ChromeUI", &data);
            }
            Arc::new(backend)
        })
        .clone()
}

/// DC-11 字体栈统一：把 [`FontLoader`] 链接到共享 [`FontdueBackend`]。
///
/// 经 [`chrome_ui_primary_font_data`] 取系统主字体字节，调用
/// [`FontLoader::init_shared_backend`] 创建并设置共享后端（同时 backfill 所有已加载
/// 字体到共享后端）。设置后，生产渲染路径（`rasterize_glyph_with_fallback` →
/// `rasterize_glyph_shared`）把 glyph 光栅委托给共享后端，使 render-foundation 与
/// UI SDK / zero-webview 共享同一字体栈（DC-11 关键不变量）。fontdue 在同字节同 glyph
/// 上确定性光栅 → 页面像素逐位不变（由 `make product-smoke` 守门）。
///
/// [`FontLoader`]: zero_render_foundation::font::loader::FontLoader
/// [`FontLoader::init_shared_backend`]: zero_render_foundation::font::loader::FontLoader::init_shared_backend
/// [`FontdueBackend`]: zero_text_foundation::FontdueBackend
pub fn link_font_loader_to_shared_backend(font_loader: &mut zero_render_foundation::font::loader::FontLoader) {
    // 取系统主字体数据，直接初始化共享后端（每次 BrowserApp::new 调用一次，轻量）。
    if let Some((data, _path)) = chrome_ui_primary_font_data() {
        font_loader.init_shared_backend("ChromeUI", &data);
    }
}

/// 把 SDK chrome 渲染产出并入帧（DC-14 浏览器接线）。
///
/// 经 [`render_chrome_via_sdk`](zero_browser_chrome::sdk_render::render_chrome_via_sdk) 渲染
/// desktop chrome（shell → model → DesktopBrowserShell → WidgetHost → Scene → bridge），再经
/// [`merge_into_frame`](zero_ui_adapter_render_foundation::merge_into_frame) 把 fills + 文本
/// ImagePrimitive（image_key 重映射到帧 cache）合并进 `scene_primitives` + `image_cache`。
///
/// - `width`/`height`：帧物理像素尺寸（SDK chrome 在此坐标空间渲染，与帧对齐）。
/// - `image_cache`：`None`（无活跃 tab）时跳过——不阻断渲染，仅本帧不叠加 SDK chrome。
#[cfg(feature = "sdk-chrome")]
fn compose_sdk_chrome_overlay(
    shell: &zero_browser_shell::BrowserShell,
    width: u32,
    height: u32,
    scene_primitives: &mut zero_render_foundation::primitive::RenderPrimitives,
    image_cache: Option<&mut zero_render_foundation::image_cache::ImageCache>,
) {
    use zero_browser_chrome::sdk_render::render_chrome_via_sdk;
    use zero_ui_adapter_render_foundation::merge_into_frame;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::theme::SemanticTokens;

    let Some(image_cache) = image_cache else {
        return;
    };
    let logical_size = Size::new(width as f32, height as f32);
    let metrics = WindowMetrics {
        logical_size,
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: 1.0,
        density: 1.0,
        orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
    };
    let backend = sdk_font_backend();
    let bridge = render_chrome_via_sdk(shell, &metrics, &SemanticTokens::light(), backend);
    let (sdk_prims, sdk_cache) = bridge.into_primitives_and_cache();
    merge_into_frame(sdk_prims, &sdk_cache, scene_primitives, image_cache);
}

/// SDK chrome 替换手绘 chrome（DC-14 替换式迁移，feature `sdk-chrome`）。
///
/// 与 [`compose_sdk_chrome_overlay`]（additive overlay）不同，本函数**替换**手绘 chrome 层：
/// 1. 调用 [`render_chrome_via_sdk_with_layout`] 获取 SDK chrome + viewport rect。
/// 2. 按 viewport rect 与手绘 chrome 的 `chrome_top` 差值翻译 `page_fills`/`page_glyphs`（页面内容
///    从手绘 chrome viewport 位置迁到 SDK chrome viewport 位置）。
/// 3. 把 SDK chrome fills/images 预置（prepend）到 `scene_primitives` **最底层**，使页面内容
///    在 SDK chrome viewport 区域之上绘制（覆盖 PageViewportFrame 的 viewport 底色）。
/// 4. SDK chrome 的 `ImageCache` 经 `extend_from_other` 合并进帧 cache（text glyph 键重映射）。
///
/// 返回翻译后的 `(page_fills, page_glyphs)`。
/// - `image_cache`：`None`（无活跃 tab）时跳过——返回未翻译的 page 内容。
#[cfg(feature = "sdk-chrome")]
#[allow(clippy::too_many_arguments)]
fn compose_sdk_chrome_replacement(
    shell: &zero_browser_shell::BrowserShell,
    width: u32,
    height: u32,
    chrome_top: f32,
    page_fills: Vec<FillPrimitive>,
    page_glyphs: Vec<GlyphDraw>,
    scene_primitives: &mut RenderPrimitives,
    image_cache: &mut Option<&mut ImageCache>,
) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
    use zero_browser_chrome::sdk_render::render_chrome_via_sdk_with_layout;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::theme::SemanticTokens;

    let Some(ic) = image_cache.as_mut() else {
        return (page_fills, page_glyphs);
    };

    let backend = sdk_font_backend();
    let logical_size = Size::new(width as f32, height as f32);
    let metrics = WindowMetrics {
        logical_size,
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: 1.0,
        density: 1.0,
        orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
    };
    let (bridge, viewport_rect) =
        render_chrome_via_sdk_with_layout(shell, &metrics, &SemanticTokens::light(), backend);
    let (sdk_prims, sdk_cache) = bridge.into_primitives_and_cache();

    // 翻译页面内容：从手绘 chrome viewport 位置 → SDK chrome viewport 位置。
    let mut page_fills = page_fills;
    let mut page_glyphs = page_glyphs;
    if let Some(vp) = viewport_rect {
        let dy = vp.origin.y - chrome_top;
        if dy.abs() > 0.5 {
            page_fills = translate_fills(&page_fills, dy);
            page_glyphs = translate_glyphs(&page_glyphs, dy);
            // 翻译 WebView 额外图元（shadows/gradients/images 等）——这些从 get_webview_extra_primitives
            // 返回，定位在手绘 chrome viewport 位置，需同步迁到 SDK chrome viewport 位置。
            translate_scene_primitives_y(scene_primitives, dy);
        }
    }

    // 合并 SDK chrome 的 ImageCache 到帧 cache（text glyph 键分配新 key 避免碰撞）。
    let rekey = ic.extend_from_other(&sdk_cache);

    // 重映射 SDK image_primitives 的 image_key → 帧 cache key。
    let mut sdk_images = sdk_prims.images;
    if !rekey.is_empty() {
        for img in &mut sdk_images {
            if let Some(new_key) = rekey.get(&img.image_key) {
                img.image_key = new_key.clone();
            }
        }
    }

    // 预置 SDK chrome fills 到 scene_primitives 最底层（先于页面内容绘制）。
    let mut new_fills = sdk_prims.fills;
    new_fills.append(&mut scene_primitives.fills);
    scene_primitives.fills = new_fills;

    // 预置 SDK chrome images（text glyph）。
    if !sdk_images.is_empty() {
        let mut new_images = sdk_images;
        new_images.append(&mut scene_primitives.images);
        scene_primitives.images = new_images;
    }

    (page_fills, page_glyphs)
}

/// 把 [`RenderPrimitives`] 中所有 Y 坐标平移 `dy`（DC-14 替换式迁移：
/// WebView 额外图元（shadows/gradients 等）从手绘 chrome viewport 位置翻译到 SDK chrome viewport）。
#[cfg(feature = "sdk-chrome")]
fn translate_scene_primitives_y(prims: &mut RenderPrimitives, dy: f32) {
    if dy.abs() <= 0.5 {
        return;
    }
    // shadows: rect + offset_y
    for s in &mut prims.shadows {
        s.rect.origin.y += dy;
        s.offset_y += dy;
    }
    // images: rect + clip
    for img in &mut prims.images {
        img.rect.origin.y += dy;
        if let Some(ref mut clip) = img.clip {
            clip.origin.y += dy;
        }
    }
    // gradients: rect + kind y params
    for g in &mut prims.gradients {
        g.rect.origin.y += dy;
        match &mut g.kind {
            GradientKind::Linear { x0: _, y0, x1: _, y1 } => {
                *y0 += dy;
                *y1 += dy;
            }
            GradientKind::Radial { cx: _, cy, .. } => {
                *cy += dy;
            }
            GradientKind::Conic { cx: _, cy, .. } => {
                *cy += dy;
            }
        }
    }
    // rounded_rects
    for r in &mut prims.rounded_rects {
        r.rect.origin.y += dy;
    }
    // strokes
    for s in &mut prims.strokes {
        s.y1 += dy;
        s.y2 += dy;
    }
    // path_strokes: vertices flat [x0,y0,x1,y1,...], y at odd indices
    for ps in &mut prims.path_strokes {
        for i in (1..ps.vertices.len()).step_by(2) {
            ps.vertices[i] += dy;
        }
    }
    // path_fills: vertices flat [x0,y0,...], y at odd indices
    for pf in &mut prims.path_fills {
        for i in (1..pf.vertices.len()).step_by(2) {
            pf.vertices[i] += dy;
        }
    }
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

/// 根据用户主题偏好、窗口主题与系统探测结果解析实际配色方案。
pub fn resolve_effective_color_scheme(
    preference: zero_browser_shell::ColorThemePreference,
    window_theme: Option<winit::window::Theme>,
    detected: PrefersColorSchemeValue,
) -> PrefersColorSchemeValue {
    if let Some(scheme) = color_scheme_from_env() {
        return scheme;
    }
    match preference {
        zero_browser_shell::ColorThemePreference::Light => PrefersColorSchemeValue::Light,
        zero_browser_shell::ColorThemePreference::Dark => PrefersColorSchemeValue::Dark,
        zero_browser_shell::ColorThemePreference::Auto => window_theme
            .map(|theme| match theme {
                winit::window::Theme::Dark => PrefersColorSchemeValue::Dark,
                winit::window::Theme::Light => PrefersColorSchemeValue::Light,
            })
            .unwrap_or(detected),
    }
}

/// 从页面内容构建 WebView 表面 RenderPrimitives（DC-3 phase-2）。
///
/// 收集页面 fills 与 webview 额外图元（shadows/gradients/images/rounded_rects 等），
/// 构造为单个 `RenderPrimitives` 供 `set_surface` 注册。页面 glyphs 不走 surface 通路：
/// `GlyphDraw`（GPU 渲染用）≠ `GlyphPrimitive`（RenderPrimitives 字段），
/// glyphs 保留在渲染管线原有路径绘制。
#[cfg(feature = "sdk-chrome")]
fn build_webview_surface_primitives(
    page_fills: Vec<FillPrimitive>,
    webview_extras: RenderPrimitives,
) -> RenderPrimitives {
    let mut surface = webview_extras;
    surface.fills = [page_fills, surface.fills].concat();
    surface
}

/// DC-14 SDK chrome 替换手绘 chrome（WebView surface 变体，DC-3 phase-2）。
///
/// 使用 `render_chrome_via_sdk_with_webview_surface`：把 WebView 页面内容（fills + 额外图元）
/// 作为 surface 注册到 bridge，由 `draw_external_surface` 在 WebViewWidget 的 `ExternalSurface`
/// marker 位置合成进 SDK chrome scene。
///
/// `webview_extras`：WebView 额外图元（shadows/gradients/images/rounded_rects 等），
/// **不含** chrome 阴影（chrome_shadows 应保持为单独的顶层 primitives）。
/// 若为 `None` 则回落：使用 `render_chrome_via_sdk_with_layout`（无 surface，DC-14 路径）。
#[cfg(feature = "sdk-chrome")]
#[allow(clippy::too_many_arguments)]
fn compose_sdk_chrome_replacement_with_webview(
    shell: &zero_browser_shell::BrowserShell,
    width: u32,
    height: u32,
    page_fills: Vec<FillPrimitive>,
    page_glyphs: Vec<GlyphDraw>,
    webview_extras: Option<RenderPrimitives>,
    scene_primitives: &mut RenderPrimitives,
    image_cache: &mut Option<&mut ImageCache>,
) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
    use zero_browser_chrome::sdk_render::render_chrome_via_sdk_with_webview_surface;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::theme::{ResolvedColorScheme, SemanticTokens};

    // image_cache 可为 None（fresh tab 无 image cache）。SDK chrome 仍须渲染——仅跳过
    // ImageCache 合并（text glyph image 暂不解析，几何/chrome bars 正常画）。**此前 None
    // 时 early-return 致 SDK chrome 完全不渲染**（帧只剩 chrome_shadows），是 DC-14
    // chrome/page region 大面积 diff 的根因（2026-07-04 诊断）。
    let backend = sdk_font_backend();
    let logical_size = Size::new(width as f32, height as f32);
    let metrics = WindowMetrics {
        logical_size,
        scale_factor: 1.0,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: 1.0,
        density: 1.0,
        orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
    };

    // 构建 WebView surface：页面 fills + webview 额外图元 + 图像数据。
    // surface_id 必须与 WebViewWidget 工厂读取的 surface_id（render.rs
    // `register_chrome_factories_with_webview`，默认 0）**一致**——WebViewWidget 在
    // ExternalSurface marker 上携带此 id，paint_scene 经 draw_external_surface(rect, id)
    // 取回本 surface 合成。此前为 1（与工厂默认 0 错配）→ 页面内容永不合成（DC-14
    // page-region 99.70% diff 根因，2026-07-04 诊断）。
    const SURFACE_ID: u64 = 0;
    // DC-3 phase-2：把 WebView ImageCache 经 surface pipeline 传递（真实纹理合成）。
    // snapshot 帧 cache 条目 → surface ImageCache（insert_with_key 保留原键）；
    // 帧 cache 保留原条目不清理（scene_primitives.images 仍引用原键）。
    // surface 内的 images 经 bridge merge_surface_with_cache → extend → remap 后
    // 在帧 cache 产生新副本（暂时冗余，但证明 pipeline 端到端，零 product-smoke 风险）。
    let frame_snapshot: Vec<_> = match image_cache.as_mut() {
        Some(ic) => ic.snapshot_entries(),
        None => Vec::new(),
    };
    let webview_surface = webview_extras.map(|extras| {
        let prims = build_webview_surface_primitives(page_fills, extras);
        let surface_cache = if frame_snapshot.is_empty() {
            None
        } else {
            let mut sc = ImageCache::new(frame_snapshot.len().max(64), 16 * 1024 * 1024);
            for (key, data) in &frame_snapshot {
                sc.insert_with_key(key.clone(), data.clone());
            }
            Some(sc)
        };
        (SURFACE_ID, prims, surface_cache)
    });

    let (bridge, viewport_rect) = render_chrome_via_sdk_with_webview_surface(
        shell, &metrics, &SemanticTokens::light(), ResolvedColorScheme::Light, backend, webview_surface,
    );
    let (sdk_prims, sdk_cache) = bridge.into_primitives_and_cache();

    // page_fills 已移入 surface → 返回空 Vec；page_glyphs 保留在渲染管线原有路径绘制。
    let remaining_page_fills: Vec<FillPrimitive> = Vec::new();

    // 合并 SDK chrome 的 ImageCache 到帧 cache（text glyph 键重映射，collision-safe）。
    // image_cache 为 None 时跳过——SDK text image 暂不解析（fallback），几何仍画。
    let mut sdk_images = sdk_prims.images;
    if let Some(ic) = image_cache.as_mut() {
        let rekey = ic.extend_from_other(&sdk_cache);
        if !rekey.is_empty() {
            for img in &mut sdk_images {
                if let Some(new_key) = rekey.get(&img.image_key) {
                    img.image_key = new_key.clone();
                }
            }
        }
    }

    // 预置 SDK chrome fills 到 scene_primitives 最底层（页面内容在 surface 内，由
    // draw_external_surface 在 ExternalSurface marker 位置合成到正确 z-order）。
    let mut new_fills = sdk_prims.fills;
    let _ = viewport_rect;
    new_fills.append(&mut scene_primitives.fills);
    scene_primitives.fills = new_fills;

    // 预置 SDK chrome images（text glyph）。
    if !sdk_images.is_empty() {
        let mut new_images = sdk_images;
        new_images.append(&mut scene_primitives.images);
        scene_primitives.images = new_images;
    }

    (remaining_page_fills, page_glyphs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
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
    fn chrome_ui_primary_font_data_returns_bytes_when_available() {
        // 与 load_system_fonts_loads_primary 同前提：本平台至少一个候选字体可读。
        // DC-11 字体共享入口：返回主字体原始字节 + 路径（供 FontLoader 与 FontdueBackend 共享）。
        let (data, path) = chrome_ui_primary_font_data()
            .expect("expected at least one Chrome UI font on this platform");
        assert!(!data.is_empty(), "font bytes non-empty");
        assert!(!path.is_empty());
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

    #[test]
    fn resolve_effective_color_scheme_respects_preference() {
        use zero_browser_shell::ColorThemePreference;

        assert_eq!(
            resolve_effective_color_scheme(
                ColorThemePreference::Light,
                Some(winit::window::Theme::Dark),
                PrefersColorSchemeValue::Dark,
            ),
            PrefersColorSchemeValue::Light,
        );
        assert_eq!(
            resolve_effective_color_scheme(
                ColorThemePreference::Auto,
                None,
                PrefersColorSchemeValue::Light,
            ),
            PrefersColorSchemeValue::Light,
        );
    }

    /// 验证浏览器渲染路径消费活跃标签页 WebView 的 ImageCache 绘制 `<img>` 图元
    /// （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）。
    ///
    /// 差异法：基线（ImageCache 为空）→ 图片颜色应为 0；填充缓存（键与 engine 生成的
    /// `simple_hash(src)` 一致）后 → 图片颜色应出现 > 0。证明 webview ImageCache 经
    /// 浏览器 render 路径传入渲染器并被消费。
    #[test]
    #[serial]
    fn render_path_consumes_webview_image_cache() {
        let _guard = crate::test_sync::tab_runtime_test_guard();
        use zero_engine::RenderPipeline;
        use zero_render_foundation::image_cache::ImageData;

        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.new_tab(None);
        let tab_id = app.shell.active_tab_id().expect("active tab");

        let src = "r215-wiring.png";
        let html = format!(
            "<img src=\"{src}\" style=\"display:block;width:40px;height:40px\">"
        );

        // 同步 engine 渲染，避免并行测试下 worker 时序干扰；仍走 browser render 路径验证 ImageCache。
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        let result = pipeline.render_html(&html, "");
        let image_key = result
            .primitives
            .images
            .first()
            .expect("engine should emit image primitive for <img>")
            .image_key
            .clone();

        app.tabs.ensure_tab(tab_id);
        if let Some(snap) = app.tabs.snapshot_mut(tab_id) {
            snap.last_render = Some(zero_webview::WebViewRenderResult {
                primitives: result.primitives,
                timings: zero_engine::PipelineTimings::default(),
            });
            snap.document_height = pipeline.document_height();
        }

        // 区别于 chrome UI 与白色背景的鲜明颜色
        let (pr, pg, pb, pa) = (220u8, 30, 180, 255);
        let pixels = [pr, pg, pb, pa].repeat(40 * 40);
        let img = ImageData::from_rgba(pixels, 40, 40).unwrap();

        // 基线：ImageCache 为空 → 缓存 miss → 图片不被绘制 → 该颜色计数为 0
        let fb0 = app.render_full_scene_with_webview_for_test(800, 600);
        let count0 = count_color(&fb0, pr, pg, pb, pa);
        assert_eq!(count0, 0, "baseline: image color must be absent when cache empty");

        let count1 = {
            let _guard = crate::test_sync::tab_runtime_test_guard();
            app.tabs
                .image_cache_mut(tab_id)
                .expect("tab snapshot")
                .insert_with_key(image_key, img);
            let fb1 = app.render_full_scene_with_webview_for_test(800, 600);
            count_color(&fb1, pr, pg, pb, pa)
        };
        assert!(
            count1 > 0,
            "after populating cache, image color must be drawn (got 0 pixels)"
        );
    }

    /// 右键菜单打开时，页面文字不得绘制在菜单背景之上（`render_full_scene` ui_glyphs / overlay 顺序）。
    #[test]
    fn context_menu_covers_page_glyphs_in_full_scene() {
        let _guard = crate::test_sync::tab_runtime_test_guard();
        use zero_engine::RenderPipeline;

        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.physical_size = (1280, 900);
        app.scale_factor = 1.0;

        let tab_id = app.shell.active_tab_id().expect("active tab");

        let (cx, cy, cw, ch) = app.page_content_rect();
        let mut pipeline = RenderPipeline::new(cw, ch);
        let result = pipeline.render_html(
            "<html><body style='margin:0;background:#fff;padding-top:40px'><p style='font-size:20px;line-height:24px;color:#000;margin:0'>\
             Black page text under context menu overlay regression padding\
             Black page text under context menu overlay regression padding\
             </p></body></html>",
            "",
        );
        assert!(
            !result.primitives.glyphs.is_empty(),
            "engine should emit page text glyphs for overlap regression"
        );

        app.inject_tab_render_for_test(
            tab_id,
            zero_webview::WebViewRenderResult {
                primitives: result.primitives,
                timings: zero_engine::PipelineTimings::default(),
            },
            pipeline.document_height().unwrap_or(ch),
        );
        let engine_glyph_count = app
            .tabs
            .last_render(tab_id)
            .map(|r| r.primitives.glyphs.len())
            .unwrap_or(0);
        assert!(
            engine_glyph_count > 100,
            "injected snapshot should carry page glyphs (got {engine_glyph_count})"
        );

        let menu_x = cx + 24.0;
        let menu_y = cy + 24.0;
        let menu_w = 200_u32;
        let menu_h = 224_u32;

        let fb_plain = app.render_full_scene_with_webview_for_test(1280, 900);
        let black_before = count_rect_pixels(&fb_plain, menu_x, menu_y, menu_w, menu_h, is_near_black);
        assert!(
            black_before > 80,
            "page should render black glyphs under future menu rect (got {black_before}, engine_glyphs={engine_glyph_count})"
        );

        app.show_context_menu_for_test(menu_x, menu_y);
        assert!(app
            .build_scene_for_test(1280, 900)
            .overlay_fills
            .iter()
            .any(|f| { f.color == app.chrome_palette().context_menu_bg }));

        let fb_menu = app.render_full_scene_with_webview_for_test(1280, 900);
        let black_after = count_rect_pixels(&fb_menu, menu_x, menu_y, menu_w, menu_h, is_near_black);
        let white_after = count_rect_pixels(&fb_menu, menu_x, menu_y, menu_w, menu_h, is_near_white);

        assert!(
            black_after < black_before / 3,
            "menu overlay should hide most page text (before={black_before}, after={black_after})"
        );
        assert!(
            white_after > black_before / 2,
            "menu background should dominate overlap region (white={white_after}, page_black={black_before})"
        );
    }

    fn count_rect_pixels(
        fb: &zero_render_foundation::surface::FrameBuffer,
        x0: f32,
        y0: f32,
        w: u32,
        h: u32,
        pred: fn(u8, u8, u8) -> bool,
    ) -> usize {
        let x_start = x0.round().max(0.0) as u32;
        let y_start = y0.round().max(0.0) as u32;
        let x_end = (x_start + w).min(fb.width);
        let y_end = (y_start + h).min(fb.height);
        (y_start..y_end)
            .flat_map(|y| (x_start..x_end).map(move |x| (x, y)))
            .filter(|(x, y)| {
                let p = fb.get_pixel(*x, *y);
                pred(p[0], p[1], p[2])
            })
            .count()
    }

    fn is_near_black(r: u8, g: u8, b: u8) -> bool {
        r < 48 && g < 48 && b < 48
    }

    fn is_near_white(r: u8, g: u8, b: u8) -> bool {
        r > 240 && g > 240 && b > 240
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

// ── DC-14 SDK chrome 接线测试（feature `sdk-chrome`）─────────────────────────────
#[cfg(all(test, feature = "sdk-chrome"))]
mod sdk_chrome_tests {
    use super::*;
    use zero_browser_shell::BrowserShell;
    use zero_render_foundation::image_cache::ImageCache;
    use zero_render_foundation::primitive::{
        GradientKind, GradientPrimitive, ImagePrimitive, LineCap, LineStyle, RenderPrimitives,
        RoundedRectPrimitive, ShadowPrimitive, StrokePrimitive,
    };

    #[test]
    fn compose_overlay_merges_sdk_chrome_into_scene() {
        // 真实 BrowserShell（含 URL tab）→ compose_sdk_chrome_overlay 把 SDK chrome fills
        // 并入空 scene_primitives；文本 image（系统字体可用时）键经帧 image_cache 解析。
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);

        compose_sdk_chrome_overlay(&shell, 1280, 800, &mut scene, Some(&mut image_cache));

        assert!(!scene.fills.is_empty(), "SDK chrome fills merged into scene");
        // SDK chrome 文本 image（系统字体可用时）键全部可在帧 image_cache 解析。
        for img in &scene.images {
            assert!(
                image_cache.get(&img.image_key).is_some(),
                "merged SDK chrome image key resolvable in frame cache"
            );
        }
    }

    #[test]
    fn compose_overlay_sdk_fills_have_nonzero_area() {
        // 验证 SDK chrome 经 compose 产出的 fills 几何非零（SDK 布局正确，产出有效 fill rect）。
        // overlay 实际光栅像素验证留待 GUI 可视验收（render_full_scene draw_order/clip 路径调查）。
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);
        compose_sdk_chrome_overlay(&shell, 1280, 800, &mut scene, Some(&mut image_cache));

        let nonzero = scene
            .fills
            .iter()
            .any(|f| f.rect.size.width > 0.0 && f.rect.size.height > 0.0);
        assert!(nonzero, "SDK chrome fills include at least one non-zero-area rect");
    }

    #[test]
    fn compose_overlay_rasterizes_to_visible_framebuffer() {
        // DC-14 overlay 像素渲染验证（bridge stateful-clip 修复后）：SDK chrome scene 经
        // render_full_scene 光栅，与空 scene 帧缓冲有像素差异 → SDK chrome 实际渲染可见像素。
        // 此前 bridge apply_clip 映射到 render-foundation 破坏性 apply_clip（clear-clip-外），
        // paint_scene 每 entry 一个 clip 逐个擦除兄弟 fill → 全白。修复后 apply_clip stateful
        // intersect，draw_order 只含 fill/image，累积渲染。
        use zero_render_foundation::cpu::render_full_scene;
        use zero_render_foundation::font::cache::GlyphCache;
        use zero_render_foundation::font::loader::FontLoader;

        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);
        compose_sdk_chrome_overlay(&shell, 1280, 800, &mut scene, Some(&mut image_cache));

        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let fb = render_full_scene(
            1280,
            800,
            1.0,
            &scene,
            &font_loader,
            &mut glyph_cache,
            Some(&mut image_cache),
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!((fb.width, fb.height), (1280, 800));

        let empty = RenderPrimitives::default();
        let empty_fb = render_full_scene(
            1280,
            800,
            1.0,
            &empty,
            &font_loader,
            &mut glyph_cache,
            None,
            &[],
            &[],
            &[],
            &[],
        );
        // SDK chrome 贡献了像素 → 两帧缓冲存在差异（.any() 首个差异像素短路）。
        let differs = (0..fb.height)
            .flat_map(|y| (0..fb.width).map(move |x| (x, y)))
            .any(|(x, y)| fb.get_pixel(x, y) != empty_fb.get_pixel(x, y));
        assert!(
            differs,
            "SDK chrome rasterized to visible pixels (framebuffer differs from empty scene)"
        );
    }

    #[test]
    fn compose_replacement_prepends_sdk_chrome_and_translates_page() {
        // DC-14 替换式迁移核心行为：SDK chrome 替换手绘 chrome，fills 置于最底层；
        // 页面内容从手绘 chrome viewport 位置翻译到 SDK chrome viewport 位置。
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);

        // 构造模拟页面内容 fills（代表手绘 chrome viewport 位置的页面内容）。
        let page_fills = vec![FillPrimitive {
            rect: Rect::new(16.0, 140.0, 1248.0, 740.0),
            color: Color::rgb(255, 0, 0),
        }];
        let page_glyphs = vec![];

        // SDK chrome viewport y ≈ 96（toolbar36+tab32+bookmarks28），手绘 chrome ≈ 140。
        // dy = 96 - 140 = -44，页面内容应上移 44px。
        let chrome_top = 140.0; // 模拟手绘 chrome 高度

        let (page_fills, _page_glyphs) = compose_sdk_chrome_replacement(
            &shell,
            1280,
            800,
            chrome_top,
            page_fills,
            page_glyphs,
            &mut scene,
            &mut Some(&mut image_cache),
        );

        // SDK chrome fills 已置于 scene 最底层。
        assert!(!scene.fills.is_empty(), "SDK chrome fills prepended to scene");

        // 页面内容 fills 从手绘 chrome viewport 位置 (y=140) 翻译到 SDK viewport (≈96)。
        if !page_fills.is_empty() {
            let translated_y = page_fills[0].rect.origin.y;
            assert!(
                translated_y < 140.0,
                "page fills translated upward: y={translated_y} < chrome_top=140"
            );
        }
    }

    #[test]
    fn compose_replacement_emits_no_viewport_background_fill() {
        // DC-14「双 viewport frame 叠加清理」回归守卫：SDK chrome 的 PageViewportFrame 必须
        // fill_background=false（不填底色）——否则会发出覆盖页面内容区的 viewport-sized fill，
        // 与页面内容（surface 合成）叠加成双 viewport 帧。SDK chrome fills 应只含顶部 bars
        // （toolbar/tab/bookmarks，bottom ≤ ~104），不含延伸到帧底的 viewport fill。
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);
        let (_page_fills, _page_glyphs) = compose_sdk_chrome_replacement(
            &shell,
            1280,
            800,
            140.0, // 手绘 chrome 高度（仅影响翻译，不影响 SDK chrome fills）
            vec![],
            vec![],
            &mut scene,
            &mut Some(&mut image_cache),
        );
        assert!(!scene.fills.is_empty(), "SDK chrome bars present");
        // 所有 SDK chrome fills 限制在顶部 chrome bar 区（bottom < 150），无 viewport-sized fill
        // （viewport fill 的 bottom 会到帧底 800，覆盖页面内容）。
        for f in &scene.fills {
            assert!(
                f.rect.bottom() < 150.0,
                "DC-14 dual-viewport-frame guard: SDK chrome fill extends below bars (bottom={}) \
                 — PageViewportFrame must not fill (would cover page content)",
                f.rect.bottom()
            );
        }
    }

    #[test]
    fn translate_scene_primitives_y_shifts_shadows_and_gradients() {
        // DC-14 WebView 额外图元翻译：shadows/gradients/images 等从手绘 chrome viewport
        // 位置（y=140）翻译到 SDK chrome viewport（≈96，dy=-44）。
        let mut prims = RenderPrimitives::default();
        let dy = -44.0;

        // shadow: rect.y + offset_y
        prims.shadows.push(ShadowPrimitive {
            rect: Rect::new(16.0, 140.0, 1248.0, 740.0),
            color: Color::rgb(0, 0, 0),
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
        });
        // image: rect.y + clip.y
        prims.images.push(ImagePrimitive {
            rect: Rect::new(0.0, 140.0, 100.0, 100.0),
            image_key: zero_render_foundation::image_cache::ImageKey(1),
            clip: Some(Rect::new(4.0, 144.0, 92.0, 92.0)),
        });
        // gradient: rect.y + kind
        prims.gradients.push(GradientPrimitive {
            rect: Rect::new(0.0, 140.0, 100.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0, y0: 140.0, x1: 100.0, y1: 240.0,
            },
            stops: vec![],
            repeating: false,
        });
        // stroke
        prims.strokes.push(StrokePrimitive {
            x1: 0.0, y1: 140.0, x2: 10.0, y2: 150.0,
            width: 2.0, color: Color::rgb(0, 0, 0),
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        // rounded_rect
        prims.rounded_rects.push(RoundedRectPrimitive {
            rect: Rect::new(0.0, 140.0, 100.0, 100.0),
            color: Color::rgb(0, 0, 0),
            top_left_radius: 8.0,
            top_right_radius: 8.0,
            bottom_right_radius: 8.0,
            bottom_left_radius: 8.0,
        });

        translate_scene_primitives_y(&mut prims, dy);

        // shadow translated
        assert!((prims.shadows[0].rect.origin.y - 96.0).abs() < 0.01);
        assert!((prims.shadows[0].offset_y - (4.0 + dy)).abs() < 0.01);
        // image translated
        assert!((prims.images[0].rect.origin.y - 96.0).abs() < 0.01);
        assert!((prims.images[0].clip.unwrap().origin.y - 100.0).abs() < 0.01);
        // gradient translated
        assert!((prims.gradients[0].rect.origin.y - 96.0).abs() < 0.01);
        if let GradientKind::Linear { y0, y1, .. } = &prims.gradients[0].kind {
            assert!((*y0 - 96.0).abs() < 0.01);
            assert!((*y1 - 196.0).abs() < 0.01);
        }
        // stroke translated
        assert!((prims.strokes[0].y1 - 96.0).abs() < 0.01);
        assert!((prims.strokes[0].y2 - 106.0).abs() < 0.01);
        // rounded_rect translated
        assert!((prims.rounded_rects[0].rect.origin.y - 96.0).abs() < 0.01);
    }

    #[test]
    fn compose_replacement_with_webview_surface_registers_and_merges() {
        // DC-3 phase-2：WebView surface 变体——页面 fills + webview 额外图元作为 surface
        // 注册到 bridge，draw_external_surface 在 ExternalSurface marker 位置合成。
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut scene = RenderPrimitives::default();
        let mut image_cache = ImageCache::new(64, 16 * 1024 * 1024);

        // 模拟页面内容 fills（WebView 渲染输出）
        let page_fills = vec![FillPrimitive {
            rect: Rect::new(16.0, 140.0, 1248.0, 740.0),
            color: Color::rgb(255, 0, 0),
        }];
        let page_glyphs: Vec<GlyphDraw> = vec![];
        let glyph_len = page_glyphs.len(); // 在 move 之前缓存

        // 模拟 webview 额外图元（shadow 在 viewport 区域）
        let mut extras = RenderPrimitives::default();
        extras.shadows.push(ShadowPrimitive {
            rect: Rect::new(16.0, 140.0, 1248.0, 740.0),
            color: Color::rgb(0, 0, 0),
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
        });

        // webview surface 变体：页面内容作为 surface 注册，而非翻译后手动合并。
        let (result_fills, result_glyphs) = compose_sdk_chrome_replacement_with_webview(
            &shell, 1280, 800,
            page_fills, page_glyphs,
            Some(extras), // 传入 webview 额外图元
            &mut scene,
            &mut Some(&mut image_cache),
        );

        // page_fills 已移入 surface → 返回空 Vec（不再手动翻译合并）。
        assert!(
            result_fills.is_empty(),
            "page_fills consumed into webview surface, should return empty"
        );
        // page_glyphs 保留（不进入 surface，由渲染管线原有路径绘制）。
        assert_eq!(result_glyphs.len(), glyph_len);

        // SDK chrome fills 已置于 scene 最底层。
        assert!(!scene.fills.is_empty(), "SDK chrome fills prepended to scene");

        // 验证 scene 不包含原来的 shadows（已移入 surface）。
        assert!(
            scene.shadows.is_empty(),
            "webview extras consumed into surface, scene shadows should be empty"
        );
    }

    /// DC-14 headless 可视验收：SDK chrome **替换式迁移**的完整浏览器帧（`render_cpu`
    /// 真实消费的路径：`build_scene` → `compose_sdk_chrome_replacement_with_webview` →
    /// `render_full_scene`）在无 GUI 环境光栅为像素，顶部 chrome 区有可见 SDK chrome 像素。
    ///
    /// 既有覆盖：`compose_overlay_rasterizes_to_visible_framebuffer` 只测 additive overlay
    /// 路径；`compose_replacement_*` 只测 scene 构造（未光栅）。本测闭合「替换路径完整帧 →
    /// 顶部 chrome 区像素」的 headless 验收缺口。
    #[test]
    fn sdk_chrome_replacement_full_frame_rasterizes_visible_chrome_region() {
        use zero_render_foundation::cpu::render_full_scene;
        use zero_render_foundation::font::cache::GlyphCache;
        use zero_render_foundation::font::loader::FontLoader;

        let mut app = BrowserApp::new(crate::app::RenderMode::Cpu);
        app.physical_size = (1280, 800);
        app.scale_factor = 1.0;

        // 默认 active tab → image_cache 为 Some（compose_sdk_chrome_replacement_with_webview
        // 要求非 None，否则提前 return 不渲染 SDK chrome）。
        assert!(app.shell.active_tab_id().is_some(), "default active tab present");

        let fb = app.render_full_scene_sdk_chrome_for_test(1280, 800);
        assert_eq!((fb.width, fb.height), (1280, 800));

        // 空 scene 基线（fresh font/glyph cache，避免借用 app 字段）。
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let empty = RenderPrimitives::default();
        let empty_fb = render_full_scene(
            1280,
            800,
            1.0,
            &empty,
            &font_loader,
            &mut glyph_cache,
            None,
            &[],
            &[],
            &[],
            &[],
        );

        // 整帧与空 scene 有差异（SDK chrome bars + 页面 surface 贡献像素）。
        let total_differs = (0..fb.height)
            .flat_map(|y| (0..fb.width).map(move |x| (x, y)))
            .any(|(x, y)| fb.get_pixel(x, y) != empty_fb.get_pixel(x, y));
        assert!(
            total_differs,
            "SDK chrome replacement full frame rasterizes visible pixels (differs from empty scene)"
        );

        // 顶部 chrome 区（SDK chrome bars ≈ toolbar36 + tab32 + bookmarks28 ≈ 96px）与空 scene
        // 有差异 —— 证明 SDK chrome 在真实 render_cpu 装配下渲染到顶部 chrome 区像素，
        // 而非只在页面区/overlay 区贡献。
        let chrome_region_differs = (0..90)
            .flat_map(|y| (0..fb.width).map(move |x| (x, y)))
            .any(|(x, y)| fb.get_pixel(x, y) != empty_fb.get_pixel(x, y));
        assert!(
            chrome_region_differs,
            "SDK chrome bars render visible pixels in top chrome region (y < 90) via replacement path"
        );
    }

    /// DC-14 chrome-region pixel-diff **baseline 量化**（headless 可视验收反馈环）。
    ///
    /// 用户确认的 DC-14 终局验收（2026-07-04，commit 8248c604）：`cargo run --bin zero-browser
    /// --features sdk-chrome` 与 `cargo run --bin zero-browser` 视觉像素级等价，**chrome 区 diff ≤
    /// 2%**、页面区 diff ≈ 0%。此前只证明了「SDK chrome 替换路径能光栅出可见像素」，从未**量化**
    /// 两路径的差异百分比 —— 本测闭合该缺口。
    ///
    /// 在 headless 下逐像素比较：
    /// - 手绘 chrome 路径：[`BrowserApp::render_full_scene_with_webview_for_test`]
    /// - SDK chrome 替换路径：[`BrowserApp::render_full_scene_sdk_chrome_for_test`]
    ///
    /// 报告顶部 chrome 区（y < 96，≈ toolbar 36 + tab strip 32 + bookmarks 28）与页面区的 diff
    /// 百分比（用 `--nocapture` 查看）。当前 12 个 chrome 组件仍为 `ChromePanel` 占位，预期 chrome
    /// 区 diff 远高于 2% —— 本测把该缺口量化为后续真实 Widget 实现的反馈环（每实现一个真实
    /// 组件，重跑本测看 diff 收敛）。当前不断言 chrome ≤ 2%（baseline 阶段），仅 sanity 断言。
    #[test]
    fn dc14_chrome_region_pixel_diff_baseline() {
        let width = 1280u32;
        let height = 800u32;
        let mut app = BrowserApp::new(crate::app::RenderMode::Cpu);
        app.physical_size = (width, height);
        app.scale_factor = 1.0;
        assert!(app.shell.active_tab_id().is_some(), "default active tab present");

        // 两路径共享同一 app 状态（build_scene 从当前 shell 状态重建；glyph_cache 缓存幂等，
        // 不影响输出像素），顺序调用避免跨实例字体/状态差异。
        let hand_fb = app.render_full_scene_with_webview_for_test(width, height);
        let sdk_fb = app.render_full_scene_sdk_chrome_for_test(width, height);
        // 控制组：再次渲染手绘路径，证明 harness 确定性（hand-vs-hand 必须为 0%，否则
        // 上面的 diff 是测试顺序/状态污染伪影而非真实路径差异）。
        let hand_fb2 = app.render_full_scene_with_webview_for_test(width, height);
        assert_eq!((hand_fb.width, hand_fb.height), (width, height));
        assert_eq!((sdk_fb.width, sdk_fb.height), (width, height));
        let control_diff = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|(x, y)| hand_fb.get_pixel(*x, *y) != hand_fb2.get_pixel(*x, *y))
            .count();
        let control_pct = (control_diff as f64 / (width * height) as f64) * 100.0;
        eprintln!(
            "DC-14 control (hand-vs-hand): {control_diff}/{} = {control_pct:.3}% (must be 0%)",
            width * height
        );
        assert_eq!(control_diff, 0, "hand-vs-hand render must be deterministic");

        // 顶部 chrome 区 ≈ 96px；其余为页面（viewport）区。
        let chrome_bottom = 96u32;
        let mut chrome_diff = 0usize;
        let mut chrome_total = 0usize;
        let mut page_diff = 0usize;
        let mut page_total = 0usize;
        for y in 0..height {
            let in_chrome = y < chrome_bottom;
            for x in 0..width {
                let differs = hand_fb.get_pixel(x, y) != sdk_fb.get_pixel(x, y);
                if in_chrome {
                    chrome_total += 1;
                    if differs {
                        chrome_diff += 1;
                    }
                } else {
                    page_total += 1;
                    if differs {
                        page_diff += 1;
                    }
                }
            }
        }
        let chrome_pct = (chrome_diff as f64 / chrome_total as f64) * 100.0;
        let page_pct = (page_diff as f64 / page_total as f64) * 100.0;
        eprintln!(
            "DC-14 pixel-diff baseline: chrome region (y<{chrome_bottom}) {chrome_diff}/{chrome_total} = {chrome_pct:.2}%, page region {page_diff}/{page_total} = {page_pct:.2}%"
        );

        // sanity：区域非空；页面区两路径共用 build_scene 的页面内容，不应整体错位全差。
        assert!(chrome_total > 0 && page_total > 0);
        assert!(
            page_pct < 100.0,
            "page region fully differs — render paths diverged unexpectedly"
        );
    }

    /// DC-14 page-region 99.70% diff 根因定位（diagnostic）。
    ///
    /// [`dc14_chrome_region_pixel_diff_baseline`] 测出页面区 99.70% diff（应 ≈0%）。本测把
    /// 「页面区差异」拆解为可定位的指标：对 hand-drawn / SDK / empty 三条帧，分别统计 chrome 区
    /// 与页面区的 **ink 像素数**（与 empty scene 不同的像素 = 该路径实际画出的内容）。
    ///
    /// - 若 SDK 页面区 ink ≈ 0 而 hand-drawn 页面区 ink 高 → SDK 替换路径**丢失了页面内容**
    ///   （surface 合成未把 page_fills 还原为像素，draw_external_surface 路径断裂）。
    /// - 若 SDK 页面区 ink 与 hand-drawn 接近但两者互相 diff 高 → 内容存在但**位置错位**
    ///   （SDK viewport rect 与手绘 viewport y 起点不同 → 整体平移）。
    /// - 此外统计 SDK 路径在页面区的「非黑像素分布」首末 y，判断页面内容是否被画到了
    ///   错误的纵向位置。
    #[test]
    fn dc14_page_region_ink_diagnostic() {
        use zero_render_foundation::cpu::render_full_scene;
        use zero_render_foundation::font::cache::GlyphCache;
        use zero_render_foundation::font::loader::FontLoader;

        let width = 1280u32;
        let height = 800u32;
        let mut app = BrowserApp::new(crate::app::RenderMode::Cpu);
        app.physical_size = (width, height);
        app.scale_factor = 1.0;

        let hand_fb = app.render_full_scene_with_webview_for_test(width, height);
        let sdk_fb = app.render_full_scene_sdk_chrome_for_test(width, height);

        // 空 scene 基线（fresh font/glyph cache）。
        let empty_fb = {
            let fl = FontLoader::new();
            let mut gc = GlyphCache::new(64);
            render_full_scene(width, height, 1.0, &RenderPrimitives::default(), &fl, &mut gc, None, &[], &[], &[], &[])
        };

        let chrome_bottom = 96u32;
        let ink_against_empty = |fb: &zero_render_foundation::surface::FrameBuffer| -> (usize, usize, usize, usize) {
            // 返回 (chrome_ink, chrome_total, page_ink, page_total)。
            let mut ci = 0usize;
            let mut ct = 0usize;
            let mut pi = 0usize;
            let mut pt = 0usize;
            for y in 0..height {
                let in_chrome = y < chrome_bottom;
                for x in 0..width {
                    let ink = fb.get_pixel(x, y) != empty_fb.get_pixel(x, y);
                    if in_chrome {
                        ct += 1;
                        if ink { ci += 1; }
                    } else {
                        pt += 1;
                        if ink { pi += 1; }
                    }
                }
            }
            (ci, ct, pi, pt)
        };
        let (hci, hct, hpi, hpt) = ink_against_empty(&hand_fb);
        let (sci, _sct, spi, _spt) = ink_against_empty(&sdk_fb);

        // SDK 路径页面区「ink 首末 y」：判断页面内容纵向位置。
        let mut sdk_page_y_min = None::<u32>;
        let mut sdk_page_y_max = None::<u32>;
        for y in chrome_bottom..height {
            let row_has_ink = (0..width).any(|x| sdk_fb.get_pixel(x, y) != empty_fb.get_pixel(x, y));
            if row_has_ink {
                sdk_page_y_min = Some(sdk_page_y_min.map_or(y, |m: u32| m.min(y)));
                sdk_page_y_max = Some(sdk_page_y_max.map_or(y, |m: u32| m.max(y)));
            }
        }
        let mut hand_page_y_min = None::<u32>;
        let mut hand_page_y_max = None::<u32>;
        for y in chrome_bottom..height {
            let row_has_ink = (0..width).any(|x| hand_fb.get_pixel(x, y) != empty_fb.get_pixel(x, y));
            if row_has_ink {
                hand_page_y_min = Some(hand_page_y_min.map_or(y, |m: u32| m.min(y)));
                hand_page_y_max = Some(hand_page_y_max.map_or(y, |m: u32| m.max(y)));
            }
        }

        eprintln!(
            "DC-14 page ink diagnostic: hand chrome ink {hci}/{hct} ({:.1}%), page ink {hpi}/{hpt} ({:.1}%), y[{:?}..{:?}]; \
             SDK chrome ink {sci}/{} ({:.1}%), page ink {spi}/{} ({:.1}%), y[{:?}..{:?}]",
            100.0 * hci as f64 / hct as f64,
            100.0 * hpi as f64 / hpt as f64,
            hand_page_y_min, hand_page_y_max,
            hct, 100.0 * sci as f64 / hct as f64,
            hpt, 100.0 * spi as f64 / hpt as f64,
            sdk_page_y_min, sdk_page_y_max,
        );

        // 像素采样：页面中心 + 四角，定位 SDK 路径给页面区涂了什么色。
        let hand_ctr = hand_fb.get_pixel(width / 2, (height + chrome_bottom) / 2);
        let sdk_ctr = sdk_fb.get_pixel(width / 2, (height + chrome_bottom) / 2);
        let sdk_tl = sdk_fb.get_pixel(2, chrome_bottom + 2);
        let sdk_br = sdk_fb.get_pixel(width - 3, height - 3);
        eprintln!(
            "DC-14 page pixel sample: hand center {:?}; SDK center {:?}, SDK page TL {:?}, SDK page BR {:?}",
            hand_ctr, sdk_ctr, sdk_tl, sdk_br
        );

        // 两条路径都应渲染出非空帧（sanity）。
        assert!(hci > 0, "hand-drawn renders chrome ink");
        assert!(sci > 0, "SDK renders chrome ink");
    }

}
