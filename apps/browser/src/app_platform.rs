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

        // ZeroUI winit adapter 的 CJK 前提：未调用 set_ime_allowed(true) 时平台不会产生
        // WindowEvent::Ime。页面 renderer 自行决定非文本焦点是否消费事件。
        let needs_ime = needs_ime_enabled(
            self.window_focused,
            self.address_bar_focused,
            self.shell.find_state().is_active(),
            self.shell.active_tab_id().is_some(),
        );
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
        } else {
            // 页面 caret 精确矩形将在 renderer focus 回执接入；在此之前至少把候选窗约束在页面区，
            // 避免沿用地址栏的陈旧位置。
            let tab_id = self.shell.active_tab_id().expect("page IME requires an active tab");
            let scroll = self.tab_scroll_state(tab_id);
            let (content_x, content_y, _, _) = self.page_content_rect();
            let s = self.scale_factor;
            let (x, y, width, height) = self.tabs.page_ime_rect(tab_id).unwrap_or((0.0, 0.0, 1.0, 20.0));
            window.set_ime_cursor_area(
                LogicalPosition::new(content_x / s + x - scroll.x / s, content_y / s + y - scroll.y / s),
                LogicalSize::new(width, height),
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

    fn skip_local_composite_for_owned_present(&self) -> bool {
        crate::compositor_client::owned_present_enabled()
            && crate::compositor_client::present_enabled()
            && self.compositor_status() == crate::compositor_client::CompositorStatus::Healthy
    }

    /// Linux：将 tab 上 pending dma-buf 导入 GPU 纹理。
    #[cfg(target_os = "linux")]
    fn apply_compositor_dmabuf_import(&mut self, _width: u32, _height: u32) {
        use zero_render_foundation::gpu::{ExportedGpuFrame, try_import_linear_dmabuf};

        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        let Some(dmabuf) = self.tabs.snapshot_mut(tab_id).and_then(|s| s.take_compositor_dmabuf()) else {
            return;
        };
        let Some(gpu) = self.gpu_renderer_as_mut() else {
            return;
        };
        let export = ExportedGpuFrame {
            fd: dmabuf.fd,
            width: dmabuf.width,
            height: dmabuf.height,
            stride: dmabuf.stride,
            drm_fourcc: dmabuf.drm_fourcc,
            drm_modifier: dmabuf.drm_modifier,
            sync_fd: None,
        };
        match try_import_linear_dmabuf(gpu.device(), gpu.queue(), &export) {
            Ok(texture) => {
                gpu.set_compositor_import(texture, dmabuf.width, dmabuf.height, dmabuf.dst_x, dmabuf.dst_y);
            }
            Err(error) => {
                tracing::warn!("compositor dma-buf 导入失败，回退 RGBA: {error}");
                gpu.clear_compositor_import();
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn apply_compositor_dmabuf_import(&mut self, _width: u32, _height: u32) {}

    /// GPU 渲染一帧（使用全量 13 种图元管线）
    pub fn render_frame(&mut self, width: u32, height: u32, present: bool) {
        if !present || !self.window_focused {
            return;
        }
        // R3254-M4：标签切换时清 GPU 图片纹理缓存（旧标签纹理滞留会累积显存）。
        let active_tab = self.shell.active_tab_id();
        if active_tab != self.last_rendered_tab
            && let Some(renderer) = self.gpu_renderer.as_mut()
        {
            renderer.clear_image_texture_cache();
        }
        self.last_rendered_tab = active_tab;
        if self.skip_local_composite_for_owned_present() {
            self.forward_compositor_chrome_ui(width, height);
            self.maybe_request_compositor_present(width, height);
            return;
        }
        self.apply_compositor_dmabuf_import(width, height);
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            if renderer.is_present_suspended() {
                self.gpu_renderer = gpu;
                return;
            }
            let (fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows, overlay_rounded_rects) = self.build_scene(width, height);

            // 获取 WebView 额外图元（渐变、阴影、圆角矩形、线段、路径等）
            let webview_extras = self.get_webview_extra_primitives();

            // 合并 chrome fills + chrome shadows + webview 图元
            let mut scene_primitives = webview_extras;
            scene_primitives.fills = [fills, scene_primitives.fills].concat();
            // chrome 阴影（页面视口 drop shadow）置于 webview 阴影之前，
            // 确保页面阴影绘制在网页内容阴影之下、chrome 背景之上。
            scene_primitives.shadows = [chrome_shadows, scene_primitives.shadows].concat();

            // 取活跃标签页 webview 的 ImageCache，供渲染器绘制 <img> 图元
            // （goal doc DC-13 P1「图片子资源/ImageCache 未贯通」最后消费 hop）
            // self.shell / self.webviews / self.font_loader / self.glyph_cache 为不相交字段借用
            let image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
                Some(id) => self.tabs.image_cache_mut(id),
                None => None,
            };

            // 使用全量 GPU 渲染管线
            if !renderer.render_full_scene_gpu(
                &scene_primitives,
                &self.font_loader,
                &mut self.glyph_cache,
                image_cache,
                &glyphs,
                &overlay_fills,
                &overlay_glyphs,
                &overlay_rounded_rects,
                // P2-8 HiDPI：scene_primitives 为 CSS 逻辑坐标，GPU 顶点按
                // scale_factor 缩放（与 CPU 路径 render_scene_cpu 一致）；
                // 旧传 1.0 致高分屏下文字/图片按 1x 光栅。
                self.scale_factor,
            ) {
                // P0-1：GPU 不支持本帧特性（clips/blend_modes/半透明/带模糊阴影/
                // 窗口模式滤镜变换）→ CPU 整帧渲染后上传 blit（慢但对，避免静默画错）。
                // 基线：docs/learnings/bugs/cpu-gpu-path-divergence.md
                let fb = self.render_scene_cpu(
                    width,
                    height,
                    &scene_primitives,
                    &glyphs,
                    &overlay_fills,
                    &overlay_glyphs,
                    &overlay_rounded_rects,
                );
                let texture = renderer.upload_frame(fb.width, fb.height, &fb.data);
                renderer.set_compositor_import(texture, fb.width, fb.height, 0.0, 0.0);
                let empty = zero_render_foundation::primitive::RenderPrimitives::default();
                renderer.render_full_scene_gpu(
                    &empty,
                    &self.font_loader,
                    &mut self.glyph_cache,
                    None,
                    &[],
                    &[],
                    &[],
                    &[],
                    1.0,
                );
                renderer.clear_compositor_import();
            }
            // R3254-M5：GPU 光栅化路径同样每帧回收零引用图片（此前 GPU 路径从不 gc，
            // ImageCache 的 2048 条目/256MB 上限形同虚设）。
            if let Some(id) = active_tab
                && let Some(cache) = self.tabs.image_cache_mut(id)
            {
                cache.gc();
            }
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
    ) -> Option<zero_render_foundation::surface::FrameBuffer> {
        if !present {
            return None;
        }

        if let Some(fb) = self.try_blit_compositor_present(width, height, cpu_surface) {
            self.forward_compositor_chrome_ui(width, height);
            self.maybe_request_compositor_present(width, height);
            return Some(fb);
        }

        if self.skip_local_composite_for_owned_present() {
            self.forward_compositor_chrome_ui(width, height);
            self.maybe_request_compositor_present(width, height);
            let mut fb = zero_render_foundation::surface::FrameBuffer::new(width, height);
            fb.clear(255, 255, 255, 255);
            if present_rgba_to_softbuffer(cpu_surface, fb.width, fb.height, &fb.data) {
                return Some(fb);
            }
            return None;
        }

        let (fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows, overlay_rounded_rects) = self.build_scene(width, height);

        // 获取 WebView 的额外图元类型（渐变、阴影、线段等）
        let webview_extras = self.get_webview_extra_primitives();

        // 合并：chrome fills + chrome shadows + webview fills (已在 fills 中) + webview 额外图元
        let mut scene_primitives = webview_extras;
        // fills 和 glyphs 已通过 append_webview_primitives 混入 chrome 的 fills/glyphs
        // 所以只需把 chrome fills 放入 scene_primitives.fills 的前面
        scene_primitives.fills = [fills, scene_primitives.fills].concat();
        scene_primitives.shadows = [chrome_shadows, scene_primitives.shadows].concat();

        // ImageCache 在 render_scene_cpu 内部获取（避免 &mut self 调用前的借用冲突）
        let fb = self.render_scene_cpu(
            width,
            height,
            &scene_primitives,
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
        );
        self.forward_compositor_chrome_ui(width, height);
        self.maybe_request_compositor_present(width, height);
        if present_rgba_to_softbuffer(cpu_surface, fb.width, fb.height, &fb.data) {
            Some(fb)
        } else {
            None
        }
    }

    /// RFC 4.4-S3：有 compositor present 帧时直接 blit 全窗口位图。
    fn try_blit_compositor_present(
        &mut self,
        width: u32,
        height: u32,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
    ) -> Option<zero_render_foundation::surface::FrameBuffer> {
        if !crate::compositor_client::present_enabled() {
            return None;
        }
        let tab_id = self.shell.active_tab_id()?;
        let (w, h, pixels) = self.tabs.compositor_present_pixels(tab_id)?;
        if w != width || h != height {
            return None;
        }
        if !present_rgba_to_softbuffer(cpu_surface, w, h, &pixels) {
            return None;
        }
        Some(zero_render_foundation::surface::FrameBuffer {
            width: w,
            height: h,
            data: pixels,
        })
    }

    /// 向 compositor 提交 Chrome UI 位图（present 时页面区透明）。
    fn forward_compositor_chrome_ui(&mut self, width: u32, height: u32) {
        if !crate::compositor_client::enabled() {
            return;
        }
        if !crate::compositor_client::ui_frames_enabled()
            && !crate::compositor_client::present_enabled()
        {
            return;
        }
        let (fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows, overlay_rounded_rects) =
            self.build_scene(width, height);
        let mut scene_primitives = zero_render_foundation::primitive::RenderPrimitives::new();
        scene_primitives.fills = fills;
        scene_primitives.shadows = chrome_shadows;
        let fb = rasterize_full_scene(
            width,
            height,
            1.0,
            &scene_primitives,
            &self.font_loader,
            &mut self.glyph_cache,
            None,
            &glyphs,
            &overlay_fills,
            &overlay_glyphs,
            &overlay_rounded_rects,
        );
        let mut ui = fb.data;
        if crate::compositor_client::present_enabled() {
            let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
            Self::clear_rect_alpha_zero(&mut ui, width, height, cx, cy, cw, ch);
        }
        crate::compositor_client::forward_ui_frame(
            crate::compositor_client::CHROME_UI_SURFACE_ID,
            width,
            height,
            ui,
        );
    }

    fn maybe_request_compositor_present(&self, width: u32, height: u32) {
        if !crate::compositor_client::present_enabled() {
            return;
        }
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        let Some(frame) = self.tabs.compositor_frame(tab_id) else {
            return;
        };
        crate::compositor_client::request_present_frame(
            frame.surface_id,
            crate::compositor_client::CHROME_UI_SURFACE_ID,
            width,
            height,
        );
    }

    fn clear_rect_alpha_zero(
        rgba: &mut [u8],
        fb_width: u32,
        fb_height: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let ix0 = x.round().max(0.0) as u32;
        let iy0 = y.round().max(0.0) as u32;
        let ix1 = (x + w).round().min(fb_width as f32) as u32;
        let iy1 = (y + h).round().min(fb_height as f32) as u32;
        if ix1 <= ix0 || iy1 <= iy0 {
            return;
        }
        let row = (fb_width * 4) as usize;
        for py in iy0..iy1 {
            for px in ix0..ix1 {
                let i = py as usize * row + px as usize * 4;
                if i + 3 < rgba.len() {
                    rgba[i..i + 4].fill(0);
                }
            }
        }
    }

    /// 光栅化场景（性能门禁优化 S1，2026-08-08）。
    ///
    /// 纯滚动帧走 **translate-blit**：平移保留帧缓冲的内容像素 + 只重绘新露出的
    /// 条带（`render_full_scene_region_into`，不清全帧）；任何其他状态变更
    /// （新快照 / 缩放 / 窗口尺寸 / 选区 / 滚动条拖拽 / 亚像素滚动 / 水平滚动）
    /// 走全量渲染并刷新保留帧。kill-switch：`ZERO_SCROLL_BLIT=0` 回退全量。
    #[allow(clippy::too_many_arguments)] // 光栅化全参数（渲染路径同款）
    fn render_scene_cpu(
        &mut self,
        width: u32,
        height: u32,
        scene_primitives: &zero_render_foundation::primitive::RenderPrimitives,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        overlay_glyphs: &[GlyphDraw],
        overlay_rounded_rects: &[RoundedRectPrimitive],
    ) -> zero_render_foundation::surface::FrameBuffer {
        let tab_id = self.shell.active_tab_id();
        let scroll = tab_id.map(|id| self.tab_scroll_state(id)).unwrap_or_default();
        let epoch = (
            tab_id.map(|id| self.tabs.snapshot_seq(id)).unwrap_or(0),
            width,
            height,
            self.scale_factor,
        );
        // 先取全部 &self 读取（页面内容矩形 / 选区 / 拖拽状态），再进入 &mut 借用区
        let page_rect = tab_id.map(|_| self.page_content_rect_for(width, height));
        let selection_active = tab_id.is_some_and(|id| {
            self.page_selection
                .get(&id)
                .is_some_and(|sel| !sel.is_collapsed())
        });
        let compositor_status = self.compositor_status();
        let page_has_content = tab_id.is_some_and(|id| {
            if compositor_controls_page(compositor_status) {
                compositor_status == crate::compositor_client::CompositorStatus::Healthy
                    && self.tabs.compositor_frame(id).is_some()
            } else {
                self.tabs.last_render(id).is_some()
            }
        });

        let blit_enabled = std::env::var("ZERO_SCROLL_BLIT").as_deref() != Ok("0");
        let dy = (scroll.y - self.fb_cache_scroll.1).round() as i32;
        let same_fraction = (scroll.y - scroll.y.floor()) == (self.fb_cache_scroll.1 - self.fb_cache_scroll.1.floor());
        // 页面内容与上一帧完全一致（无新快照、滚动未变、无选区/浮层/拖拽）→ 保留
        // 页面区像素，只重绘 chrome 条带（S1b，2026-08-08）：加载期间的动画帧
        //（spinner/进度条转动）每帧全量光栅 → 仅重绘页面区外的 chrome，动画流畅。
        let can_reuse = blit_enabled
            && self.retained_fb.is_some()
            && epoch == self.fb_cache_epoch
            && page_has_content
            && scroll.x == self.fb_cache_scroll.0
            && scroll.y == self.fb_cache_scroll.1
            && !selection_active
            && !self.page_selection_drag
            && self.scrollbar_drag.is_none()
            && self.touch_scroll.is_none()
            && !self.shell.find_state().is_active()
            && !self.context_menu.visible;
        let can_blit = blit_enabled
            && self.retained_fb.is_some()
            && epoch == self.fb_cache_epoch
            && page_has_content
            && scroll.x == self.fb_cache_scroll.0
            && same_fraction
            && dy != 0
            && page_rect.is_some_and(|(_, _, _, ch)| (dy.abs() as f32) < ch)
            && !selection_active
            && !self.page_selection_drag
            && self.scrollbar_drag.is_none()
            && self.touch_scroll.is_none()
            // 部分高度 overlay（查找栏 / 上下文菜单）在平移后会留下残影——
            // 全高滚动条自愈（overlay 每帧重画全覆盖），部分区域 overlay 需禁 blit
            && !self.shell.find_state().is_active()
            && !self.context_menu.visible;

        // 活跃标签页 webview 的 ImageCache（绘制 <img> 图元消费）——所有 &self 读取
        // 完成后、渲染调用前获取（self.tabs 与 font_loader/glyph_cache/retained_fb
        // 为不相交字段借用，可共存）
        let mut image_cache: Option<&mut ImageCache> = match tab_id {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };

        let fb = if can_blit {
            let (cx, cy, cw, ch) = page_rect.unwrap();
            let mut fb = self.retained_fb.take().unwrap();
            let ix0 = cx.round() as usize;
            let ix1 = (cx + cw).round() as usize;
            let iy0 = cy.round() as usize;
            let iy1 = (cy + ch).round() as usize;
            let row_bytes = width as usize * 4;
            let span = ix1.saturating_sub(ix0) * 4;
            let ady = dy.unsigned_abs() as usize;
            if dy > 0 {
                // 内容上移（滚动向下）：行 y ← 行 y+ady
                for y in iy0..iy1.saturating_sub(ady) {
                    let src = (y + ady) * row_bytes + ix0 * 4;
                    let dst = y * row_bytes + ix0 * 4;
                    fb.data.copy_within(src..src + span, dst);
                }
            } else {
                // 内容下移（滚动向上）：自下而上复制避免覆盖
                for y in (iy0.saturating_add(ady)..iy1).rev() {
                    let src = (y - ady) * row_bytes + ix0 * 4;
                    let dst = y * row_bytes + ix0 * 4;
                    fb.data.copy_within(src..src + span, dst);
                }
            }
            // 只重绘新露出的条带。region 语义是「剔除不相交图元」而非裁剪——
            // 穿过条带的高图元会完整绘制并污染条带外像素，故渲染到 scratch 帧缓冲
            // 后仅把条带行拷回保留帧（scratch 内越界绘制无害）。
            let strip_top = if dy > 0 { iy1.saturating_sub(ady) } else { iy0 };
            let strip_bottom = strip_top + ady;
            let strip = zero_render_foundation::geometry::Rect::new(
                ix0 as f32,
                strip_top as f32,
                (ix1 - ix0) as f32,
                ady as f32,
            );
            let mut scratch = zero_render_foundation::surface::FrameBuffer::new(width, height);
            scratch.clear(255, 255, 255, 255);
            zero_render_foundation::cpu::render_full_scene_region_into(
                &mut scratch,
                scene_primitives,
                &self.font_loader,
                &mut self.glyph_cache,
                image_cache,
                glyphs,
                overlay_fills,
                overlay_glyphs,
                overlay_rounded_rects,
                Some(strip),
                1.0,
            );
            for y in strip_top..strip_bottom {
                let row = y * row_bytes;
                fb.data[row..row + row_bytes]
                    .copy_from_slice(&scratch.data[row..row + row_bytes]);
            }
            fb
        } else if can_reuse {
            // S1b：页面区保留，重绘页面区外的 chrome 条带（顶部 + 底部）
            let (_cx, cy, _cw, ch) = page_rect.unwrap();
            let mut fb = self.retained_fb.take().unwrap();
            let top_strip = zero_render_foundation::geometry::Rect::new(
                0.0,
                0.0,
                width as f32,
                cy,
            );
            if top_strip.size.height > 0.0 {
                zero_render_foundation::cpu::render_full_scene_region_into(
                    &mut fb,
                    scene_primitives,
                    &self.font_loader,
                    &mut self.glyph_cache,
                    image_cache.as_deref_mut(),
                    glyphs,
                    overlay_fills,
                    overlay_glyphs,
                    overlay_rounded_rects,
                    Some(top_strip),
                    1.0,
                );
            }
            let bottom_top = cy + ch;
            let bottom_h = (height as f32 - bottom_top).max(0.0);
            if bottom_h > 0.0 {
                zero_render_foundation::cpu::render_full_scene_region_into(
                    &mut fb,
                    scene_primitives,
                    &self.font_loader,
                    &mut self.glyph_cache,
                    image_cache.as_deref_mut(),
                    glyphs,
                    overlay_fills,
                    overlay_glyphs,
                    overlay_rounded_rects,
                    Some(zero_render_foundation::geometry::Rect::new(
                        0.0,
                        bottom_top,
                        width as f32,
                        bottom_h,
                    )),
                    1.0,
                );
            }
            fb
        } else {
            let fb = rasterize_full_scene(
                width,
                height,
                1.0,
                scene_primitives,
                &self.font_loader,
                &mut self.glyph_cache,
                image_cache,
                glyphs,
                overlay_fills,
                overlay_glyphs,
                overlay_rounded_rects,
            );
            self.retained_fb = Some(fb.clone());
            self.fb_cache_scroll = (scroll.x, scroll.y);
            self.fb_cache_epoch = epoch;
            fb
        };

        if can_blit {
            self.retained_fb = Some(fb.clone());
            self.fb_cache_scroll = (scroll.x, scroll.y);
            self.fb_cache_epoch = epoch;
        }
        fb
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
        let (fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows, overlay_rounded_rects) = self.build_scene(width, height);
        let webview_extras = self.get_webview_extra_primitives();
        let mut scene_primitives = webview_extras;
        scene_primitives.fills = [fills, scene_primitives.fills].concat();
        scene_primitives.shadows = [chrome_shadows, scene_primitives.shadows].concat();

        // 与 render_cpu / render_frame 完全一致的 ImageCache 装配（不相交字段借用）
        let image_cache: Option<&mut ImageCache> = match self.shell.active_tab_id() {
            Some(id) => self.tabs.image_cache_mut(id),
            None => None,
        };

        rasterize_full_scene(
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

fn needs_ime_enabled(window_focused: bool, address_bar: bool, find_bar: bool, page_active: bool) -> bool {
    window_focused && (address_bar || find_bar || page_active)
}

#[cfg(test)]
mod ime_tests {
    use super::needs_ime_enabled;

    #[test]
    fn active_page_enables_platform_ime() {
        assert!(needs_ime_enabled(true, false, false, true));
        assert!(!needs_ime_enabled(false, false, false, true));
        assert!(!needs_ime_enabled(true, false, false, false));
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
) -> bool {
    use std::num::NonZeroU32;

    let Some(surface) = cpu_surface.as_mut() else {
        return false;
    };

    let sw = match NonZeroU32::new(width.max(1)) {
        Some(w) => w,
        None => return false,
    };
    let sh = match NonZeroU32::new(height.max(1)) {
        Some(h) => h,
        None => return false,
    };

    if let Err(err) = surface.resize(sw, sh) {
        tracing::error!("CPU surface resize failed: {err}");
        return false;
    }

    let mut buffer = match surface.buffer_mut() {
        Ok(b) => b,
        Err(err) => {
            tracing::error!("CPU surface buffer failed: {err}");
            return false;
        }
    };

    for (dst, chunk) in buffer.iter_mut().zip(rgba.chunks_exact(4)) {
        *dst = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
    }

    if let Err(err) = buffer.present() {
        tracing::error!("CPU surface present failed: {err}");
        return false;
    }
    true
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

/// Resolve a link clicked within a document against that document's URL.
///
/// This deliberately differs from address-bar normalization: a relative value
/// such as `guide/intro.html` is a document-relative link, not a host name.
pub fn resolve_clicked_link_url(input: &str, document_url: Option<&str>) -> String {
    let input = input.trim();
    let Some(base) = document_url.filter(|url| !url.starts_with("zero://")) else {
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

/// 进程级共享的系统字体 base 集（生产与测试同路径）。
///
/// `BrowserApp::new` 与每个 TabWorker 线程各自 `load_system_fonts`（含 19MB CJK
/// fallback 解析 ~0.5-2.9s）；每进程首个调用解析一次，其余 `duplicate()` 复用
/// （Arc 共享字体数据，见 FontLoader::duplicate）——多标签页免重复解析与重复
/// 内存（~40-60MB/份）。`duplicate` 保持 font_id 序号与字体顺序一致 → 各调用方
/// 内容与独立加载等价。`&self` 只读 + fontdue 无内部可变性 → 并发（多线程 +
/// worker 线程）安全。
///
/// 限制：进程生命周期内系统字体文件视为稳定（真实浏览器启动时扫描字体一次，
/// 与 chromium 行为一致；运行中系统字体变更不感知）。
pub(crate) fn shared_system_fonts() -> (FontLoader, Option<u32>) {
    static CACHED: std::sync::OnceLock<(FontLoader, Option<u32>)> = std::sync::OnceLock::new();
    let cached = CACHED.get_or_init(|| {
        let mut loader = FontLoader::new();
        let id = load_system_fonts(&mut loader);
        (loader, id)
    });
    (cached.0.duplicate(), cached.1)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use zero_render_foundation::font::loader::FontLoader;

    #[test]
    fn clicked_relative_link_resolves_against_file_document_url() {
        assert_eq!(
            resolve_clicked_link_url(
                "docs/intro.html",
                Some("file:///C:/site/website/index.html"),
            ),
            "file:///C:/site/website/docs/intro.html"
        );
    }

    #[test]
    fn clicked_relative_link_resolves_against_http_document_url() {
        assert_eq!(
            resolve_clicked_link_url("next.html", Some("https://example.com/guide/index.html")),
            "https://example.com/guide/next.html"
        );
    }

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
        use zero_engine::RenderPipeline;
        use zero_render_foundation::image_cache::ImageData;

        let mut app = BrowserApp::new(RenderMode::Cpu);
        app.tabs.disable_multiprocess_for_test();
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
            .primitives()
            .images
            .first()
            .expect("engine should emit image primitive for <img>")
            .image_key
            .clone();

        app.tabs.ensure_tab(tab_id);
        if let Some(snap) = app.tabs.snapshot_mut(tab_id) {
            snap.last_render = Some(zero_webview::WebViewRenderResult {
                primitives: result.primitives().clone(),
                dirty_rects: result.display_list.dirty_rects.clone(),
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
    #[ignore = "macOS CI flaky (context menu rendering timing)"]
    #[test]
    fn context_menu_covers_page_glyphs_in_full_scene() {
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
            !result.primitives().glyphs.is_empty(),
            "engine should emit page text glyphs for overlap regression"
        );

        app.inject_tab_render_for_test(
            tab_id,
            zero_webview::WebViewRenderResult {
                primitives: result.primitives().clone(),
                dirty_rects: result.display_list.dirty_rects.clone(),
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
        assert!(app.build_scene_for_test(1280, 900).2.iter().any(|f| {
            f.color == app.chrome_palette().context_menu_bg
        }));

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
