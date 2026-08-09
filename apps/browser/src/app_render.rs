// 浏览器 UI 渲染方法（从 app.rs 通过 include! 引入）
//
// 此文件在编译时被 app.rs include，共享同一个模块作用域。

// --- BrowserApp 渲染 impl ---

/// compositor client 状态是否禁止 Browser 使用 legacy 页面图元。
pub(crate) fn compositor_controls_page(status: crate::compositor_client::CompositorStatus) -> bool {
    matches!(
        status,
        crate::compositor_client::CompositorStatus::Starting
            | crate::compositor_client::CompositorStatus::Healthy
    )
}

impl BrowserApp {
    /// 鼠标指针是否落在矩形区域内（物理像素坐标）
    fn pointer_in_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let mx = self.mouse_pos.0 as f32;
        let my = self.mouse_pos.1 as f32;
        mx >= x && mx < x + w && my >= y && my < y + h
    }

    /// 构建浏览器 UI 渲染图元（物理像素坐标）。
    ///
    /// 返回 `ChromeScene`：`(fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows)`。
    /// `overlay_fills` 和 `overlay_glyphs` 在所有 fills/glyphs 之后绘制，
    /// 用于确保上下文菜单等浮层不被其他内容覆盖。
    /// `chrome_shadows` 是浏览器壳层产生的阴影（如页面视口 drop shadow）。
    fn build_scene(&mut self, width: u32, height: u32) -> ChromeScene {
        let s = self.scale_factor;
        let mut fills = Vec::new();
        let mut glyphs = Vec::new();
        let mut overlay_fills = Vec::new();
        let mut overlay_glyphs = Vec::new();
        let mut chrome_shadows: Vec<ShadowPrimitive> = Vec::new();
        let mut overlay_rounded_rects: Vec<RoundedRectPrimitive> = Vec::new();
        let font_size = layout::CHROME_FONT_SIZE * s;

        // 0. 同步加载动画起始时刻：is_loading 从 false→true 时记录，
        //    从 true→false 时清除。用于渲染模拟进度条。
        let is_loading = self.shell.active_tab().is_some_and(|t| t.is_loading());
        if is_loading && self.loading_anim_start.is_none() {
            self.loading_anim_start = Some(Instant::now());
        } else if !is_loading {
            self.loading_anim_start = None;
        }
        // 加载期间持续请求重绘，驱动进度条动画。
        if is_loading {
            self.needs_redraw = true;
        }

        // 1. 整体背景
        fills.push(rect_fill(
            0.0,
            0.0,
            width as f32,
            height as f32,
            self.chrome_palette.background,
        ));

        // 2. 标签栏背景（macOS 左侧为 traffic lights 留白，标签从 inset 起绘）
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let tab_strip_bg = self.chrome_tab_strip_bg();
        fills.push(rect_fill(0.0, 0.0, width as f32, tab_strip_h, tab_strip_bg));

        // 3. 标签内容（带布局缓存）
        self.render_tabs(&mut fills, &mut glyphs, width, font_size, s);

        // 4. 地址栏背景（与激活标签同色，形成一体工具栏；窗口失焦时统一变灰）
        let addr_y = tab_strip_h;
        let toolbar_bg = if self.window_focused {
            self.chrome_palette.toolbar_bg
        } else {
            self.chrome_palette.chrome_inactive_bg
        };
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT * s,
            toolbar_bg,
        ));

        // 5. 导航按钮
        self.render_nav_buttons(&mut fills, &mut glyphs, addr_y, font_size, s);

        // 6. 地址栏
        self.render_address_bar(&mut fills, &mut glyphs, width, addr_y, font_size, s);

        // 7. 分隔线
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        fills.push(rect_fill(
            0.0,
            toolbar_h - s,
            width as f32,
            s,
            self.chrome_palette.separator,
        ));

        // 8. 书签栏（有书签且设置开启时显示；否则不占高度）
        if self.bookmarks_bar_visible() {
            self.refresh_bookmark_favicons();
            self.render_bookmarks_bar(&mut fills, &mut glyphs, width, toolbar_h, s);
        }

        // 9. 页面内容区域（与工具栏无缝衔接：page_bg 直接填充到 chrome 边界）
        let chrome_top = self.chrome_top_y_for(s);
        let frame_bottom_y = self.page_frame_bottom_y_for(width, height);
        let page_gutter_h = frame_bottom_y - chrome_top;
        fills.push(rect_fill(
            0.0,
            chrome_top,
            width as f32,
            page_gutter_h,
            self.chrome_palette.page_bg,
        ));
        let (frame_x, frame_y, frame_w, frame_h) = self.page_frame_rect_for(width, height);
        self.render_page_frame_shadow(&mut chrome_shadows, frame_x, frame_y, frame_w, frame_h);
        self.render_page_frame(&mut fills, frame_x, frame_y, frame_w, frame_h, s);
        let (content_x, content_y, content_w, _) = self.page_content_rect_for(width, height);

        // 10. 加载进度条（模拟 Chrome 风格：快速到 30%，缓慢逼近 85%）
        if let Some(start) = self.loading_anim_start {
            let elapsed = start.elapsed().as_secs_f32();
            // 进度曲线：前 0.3s 快速到 30%，之后向 85% 渐近。
            let progress = if elapsed < 0.3 {
                (elapsed / 0.3) * 0.30
            } else {
                // 指数渐近：0.30 + 0.55 * (1 - exp(-(t-0.3)/2.0))
                0.30 + 0.55 * (1.0 - (-(elapsed - 0.3) / 2.0).exp())
            };
            let bar_w = (content_w * progress).min(content_w);
            fills.push(rect_fill(
                content_x,
                content_y,
                bar_w,
                2.0 * s,
                self.chrome_palette.loading_indicator,
            ));
        }

        // 11. 页面内容（含滚动偏移）
        self.render_page_content(&mut fills, &mut glyphs, width, content_x, content_y, font_size, s);

        // 11b. 页面滚动条（overlay，始终显示于溢出时）
        self.render_page_scrollbars(
            &mut overlay_fills,
            &mut overlay_rounded_rects,
            width,
            height,
        );

        // 11c. 装饰层（视口圆角遮罩 / 边框 / 窗口外框）必须先绘制，
        //      随后所有交互浮层（自动补全 / 链接状态栏 / 查找栏 / 下载面板 /
        //      上下文菜单）才不会被这些装饰盖住。装饰仅用于边界视觉，
        //      不参与交互，因此应位于交互浮层之下。
        self.render_page_frame_corner_masks(&mut overlay_fills, width, height, s);
        self.render_page_frame_border(&mut overlay_fills, frame_x, frame_y, frame_w, frame_h, s);
        self.render_custom_window_frame_border(&mut overlay_fills, width, height, s);

        // 12. 查找栏与下载面板在 overlay 层绘制（浮动，不占布局高度）

        // 13. 自动补全下拉
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut fills, &mut glyphs, width, font_size, s);
        }

        // 14. 链接悬停浮动状态栏（覆盖在页面内容上方，不占布局高度）
        self.render_floating_link_status(&mut fills, &mut glyphs, width, height, s);

        // 15–17. 浮动查找栏、下载面板、上下文菜单（overlay 顶层）
        if self.shell.find_state().is_active() {
            self.render_find_bar(&mut overlay_fills, &mut overlay_glyphs, width, height, font_size, s);
        }
        if self.should_show_download_panel() {
            self.render_download_panel(&mut overlay_fills, &mut overlay_glyphs, width, height, font_size, s);
        }
        if self.context_menu.visible {
            self.render_context_menu(&mut overlay_fills, &mut overlay_glyphs, s);
        }

        // 18. 缩放百分比浮层（右下角，缩放后 3 秒内显示）
        self.render_zoom_indicator(&mut overlay_fills, &mut overlay_glyphs, width, height, s);

        // 19. 打印预览徽标（右下角，@media print 激活期间持久显示；Ctrl+P 切换。R1994）
        self.render_print_preview_indicator(&mut overlay_fills, &mut overlay_glyphs, width, height, s);

        (fills, glyphs, overlay_fills, overlay_glyphs, chrome_shadows, overlay_rounded_rects)
    }

    /// 渲染打印预览徽标（DC-12 @media print；R1994）。
    ///
    /// `Ctrl+P` 切换打印预览（`toggle_print_preview`）后，`media_type == Print` 期间在页面
    /// 右下角持久显示「Print Preview — Ctrl+P to exit」徽标，给用户明确反馈当前为打印
    /// 媒体渲染（@media print 样式生效）。镜像 `render_zoom_indicator` 的右下角圆角浮层，
    /// 但**无 3 秒自动隐藏**（持久态，随 toggle 消失）。
    fn render_print_preview_indicator(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        height: u32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }
        // 仅打印媒体类型激活时显示（Screen = 默认，不显示）。
        if self.tabs.media_type() != zero_engine::MediaType::Print {
            return;
        }
        let label = "Print Preview — Ctrl+P to exit";
        let font_size = layout::CHROME_FONT_SIZE * s;
        let label_w = self.measure_ui_text_width(label, font_size);
        let pad_h = 12.0 * s;
        let pad_v = 8.0 * s;
        let box_w = label_w + pad_h * 2.0;
        let box_h = font_size + pad_v * 2.0;
        let margin = 16.0 * s;
        // 左下角（避开右下角缩放浮层 + 滚动条）。
        let box_x = margin;
        let box_y = height as f32 - box_h - margin;
        let radius = 8.0 * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, box_x, box_y, box_w, box_h, radius, self.chrome_palette.find_bar_border);
        push_rounded_rect_fill(
            fills,
            box_x + border,
            box_y + border,
            box_w - 2.0 * border,
            box_h - 2.0 * border,
            radius - border,
            self.chrome_palette.find_bar_bg,
        );
        self.draw_ui_text(
            label,
            box_x + pad_h,
            box_y + pad_v,
            font_size,
            self.chrome_palette.find_bar_text,
            glyphs,
        );
    }

    /// 渲染缩放百分比浮层。zoom 操作后 3 秒内显示在页面右下角。
    /// 超时自动清除（清除时也请求重绘以移除浮层）。
    fn render_zoom_indicator(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }
        let Some(start) = self.zoom_indicator_start else {
            return;
        };
        // 3 秒后清除。
        if start.elapsed().as_secs_f32() > 3.0 {
            self.zoom_indicator_start = None;
            return;
        }
        // 显示期间持续重绘（用于倒计时消失）。
        self.needs_redraw = true;

        let zoom = self.shell.zoom();
        // 100% 时不显示（zoom_reset 到 100% 仍短暂提示，便于用户确认复位）。
        let label = if (zoom - 1.0).abs() < 0.001 {
            "100%".to_string()
        } else {
            format!("{:.0}%", (zoom * 100.0).round())
        };
        let font_size = layout::CHROME_FONT_SIZE * s;
        let label_w = self.measure_ui_text_width(&label, font_size);
        let pad_h = 12.0 * s;
        let pad_v = 8.0 * s;
        let box_w = label_w + pad_h * 2.0;
        let box_h = font_size + pad_v * 2.0;
        let margin = 16.0 * s;
        // 右下角，避开滚动条。
        let scrollbar_w = layout::SCROLLBAR_THICKNESS * s;
        let box_x = width as f32 - box_w - margin - scrollbar_w;
        let box_y = height as f32 - box_h - margin;
        let radius = 8.0 * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, box_x, box_y, box_w, box_h, radius, self.chrome_palette.find_bar_border);
        push_rounded_rect_fill(
            fills,
            box_x + border,
            box_y + border,
            box_w - 2.0 * border,
            box_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.find_bar_bg,
        );
        if self.font_id.is_some() {
            let (text_top, _) = self.ui_text_centered_in_height(box_h, font_size);
            self.draw_ui_text(
                &label,
                box_x + (box_w - label_w) * 0.5,
                box_y + text_top,
                font_size,
                self.chrome_palette.find_bar_text,
                glyphs,
            );
        }
    }

    /// 获取 WebView 的额外图元（渐变、阴影、圆角矩形、线段、路径、变换、裁剪、滤镜、混合模式）。
    ///
    /// 注意：fills 和 glyphs 不在此返回中（它们已通过 `append_webview_primitives` 混入 chrome 层）。
    /// 仅返回 render_full_scene() 需要的其他 11 种图元类型。
    fn get_webview_extra_primitives(&self) -> RenderPrimitives {
        self.get_webview_extra_primitives_for_status(self.compositor_status())
    }

    fn get_webview_extra_primitives_for_status(
        &self,
        compositor_status: crate::compositor_client::CompositorStatus,
    ) -> RenderPrimitives {
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return RenderPrimitives::new(),
        };

        let layout = self.page_scroll_layout_for(tab_id, self.physical_size.0, self.physical_size.1);
        let scroll = self.tab_scroll_state(tab_id);
        let s = self.scale_factor;
        let clip_viewport = ViewportClip::new(layout.viewport_x, layout.viewport_y, layout.viewport_w, layout.viewport_h);
        let y_offset = layout.viewport_y - scroll.y;
        let x_offset = layout.viewport_x - scroll.x;

        if compositor_controls_page(compositor_status) {
            if compositor_status != crate::compositor_client::CompositorStatus::Healthy {
                return RenderPrimitives::new();
            }
            return self
                .tabs
                .compositor_frame(tab_id)
                .map_or_else(RenderPrimitives::new, |frame| {
                    compositor_frame_primitives(frame, x_offset, y_offset, s, clip_viewport)
                });
        }

        let primitives = match self.tabs.last_render(tab_id).map(|render| &render.primitives) {
            Some(p) => p,
            None => return RenderPrimitives::new(),
        };

        // fills/glyphs 已由 append_webview_primitives 混入 chrome 层——
        // extra 层只变换其余 11 类图元（性能门禁优化 S2，2026-08-08：
        // 旧实现先变换再清空，4400 元素页每帧白白克隆 ~11k fills + ~22k glyphs）
        let transformed = transform_webview_primitives_extra(primitives, x_offset, y_offset, s, Some(clip_viewport));

        let _ = (layout.viewport_w,);
        transformed
    }

    /// 测试用：按指定 compositor 状态构建页面额外图元。
    #[cfg(test)]
    pub fn compositor_primitives_for_test(
        &self,
        status: crate::compositor_client::CompositorStatus,
    ) -> RenderPrimitives {
        self.get_webview_extra_primitives_for_status(status)
    }

    /// 绘制页面滚动条（内容溢出时）。
    fn render_page_scrollbars(
        &self,
        overlay_fills: &mut Vec<FillPrimitive>,
        overlay_rounded_rects: &mut Vec<RoundedRectPrimitive>,
        width: u32,
        height: u32,
    ) {
        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };
        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let layout = self.page_scroll_layout_for(tab_id, width, height);
        if !layout.show_vertical && !layout.show_horizontal {
            return;
        }
        let scroll = self.tab_scroll_state(tab_id);
        let geometry = crate::page_scroll::scrollbar_geometry(&layout, scroll, cx, cy, cw, ch, self.scale_factor);
        let dragging = self.scrollbar_drag.map(|d| d.axis);
        crate::page_scroll::push_scrollbar_fills(
            &geometry,
            self.chrome_palette.scrollbar_track,
            self.chrome_palette.scrollbar_thumb,
            self.chrome_palette.scrollbar_thumb_hover,
            self.chrome_palette.scrollbar_thumb_active,
            self.scrollbar_hover,
            dragging,
            overlay_fills,
            overlay_rounded_rects,
        );
    }

    /// 渲染标签页
    fn render_tabs(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        font_size: f32,
        s: f32,
    ) {
        let active_id = self.shell.active_tab_id();
        let tab_count = self.shell.tab_count();
        if tab_count == 0 {
            return;
        }

        let new_tab_btn_w = layout::NEW_TAB_BTN_WIDTH * s;
        let window_controls_w = if self.uses_custom_window_controls() {
            layout::WINDOW_CONTROLS_WIDTH * s
        } else {
            0.0
        };
        let leading = self.tab_bar_leading_inset() * s;
        let tabs_max_width = width as f32 - window_controls_w - new_tab_btn_w - leading;
        let pinned_count = self.shell.tabs().filter(|t| t.is_pinned()).count();
        let normal_count = tab_count.saturating_sub(pinned_count);
        let pinned_total = pinned_count as f32 * layout::TAB_PINNED_WIDTH * s;
        let normal_available = (tabs_max_width - pinned_total).max(0.0);
        let normal_tab_w = if normal_count > 0 {
            let ideal = normal_available / normal_count as f32;
            if ideal < layout::TAB_MIN_WIDTH * s {
                // 拥挤区：理想宽度低于正常最小宽度。优先压缩到 COMPRESSED（保留 favicon+close）；
                // 若仍不够（ideal < COMPRESSED），进一步压缩到 ABSOLUTE_MIN（只保留 favicon）。
                let lower = if ideal < layout::TAB_MIN_WIDTH_COMPRESSED * s {
                    layout::TAB_ABSOLUTE_MIN_WIDTH * s
                } else {
                    layout::TAB_MIN_WIDTH_COMPRESSED * s
                };
                ideal.clamp(lower, layout::TAB_MIN_WIDTH * s)
            } else {
                ideal.clamp(layout::TAB_MIN_WIDTH * s, layout::TAB_MAX_WIDTH * s)
            }
        } else {
            0.0
        };

        self.tab_layout.clear();
        let mut x = leading;
        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let icon_size = layout::TAB_ICON_SIZE * s;
        let spinner_angle = self.chrome_anim_start.elapsed().as_secs_f32() * 3.5;
        let text_left_inset = 12.0 * s + icon_size + 8.0 * s;
        let (text_top, text_baseline) = self.ui_text_centered_in_height(tab_bar_h, font_size);
        let text_y = tab_y + text_top;
        let icon_cy = tab_y + text_baseline - font_size * 0.48;

        struct TabPaint {
            id: TabId,
            x: f32,
            tab_w: f32,
            tab_body_w: f32,
            bg: Color,
            is_active: bool,
            is_loading: bool,
            is_pinned: bool,
            is_muted: bool,
            is_crashed: bool,
            needs_attention: bool,
            is_dragging: bool,
            label: String,
            page_url: Option<String>,
            html_hint: Option<&'static str>,
        }

        let mut tabs: Vec<TabPaint> = Vec::new();
        for tab in self.shell.tabs() {
            let tab_w = if tab.is_pinned() {
                layout::TAB_PINNED_WIDTH * s
            } else {
                normal_tab_w
            };
            let is_active = Some(tab.id()) == active_id;
            let is_hovered = !is_active && {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= x && mx < x + tab_w && my >= tab_y && my < tab_strip_h
            };
            let bg = if is_active {
                self.chrome_palette.tab_active_bg
            } else if is_hovered {
                self.chrome_palette.tab_hover_bg
            } else {
                self.chrome_palette.tab_bar_bg
            };
            let mut label = tab.title().or(tab.url()).unwrap_or("New Tab").to_string();
            if tab.is_private() {
                label = format!("无痕 · {label}");
            }
            // 被拖拽的标签：x 跟随鼠标，绘制时置于顶层。
            let (paint_x, is_dragging) = match &self.tab_drag {
                Some(d) if d.active && d.tab_id == tab.id() => {
                    let offset = d.current_x - d.press_x;
                    (x + offset, true)
                }
                _ => (x, false),
            };
            tabs.push(TabPaint {
                id: tab.id(),
                x: paint_x,
                tab_w,
                tab_body_w: tab_w - s,
                bg: if is_dragging { self.chrome_palette.tab_active_bg } else { bg },
                is_active,
                is_loading: tab.is_loading(),
                is_pinned: tab.is_pinned(),
                is_muted: tab.is_muted(),
                is_crashed: tab.is_crashed(),
                needs_attention: tab.needs_attention(),
                is_dragging,
                label,
                page_url: tab.url().map(str::to_string),
                html_hint: Self::tab_html_hint(tab.url()),
            });
            x += tab_w;
        }

        for tab in tabs.iter().filter(|t| !t.is_active && !t.is_dragging) {
            crate::tab_chrome::push_inactive_tab_fill(fills, tab.x, tab_y, tab.tab_body_w, tab_bar_h, s, tab.bg);
        }

        if let Some(active) = tabs.iter().find(|t| t.is_active) {
            crate::tab_chrome::push_active_tab_fill(fills, active.x, tab_y, active.tab_body_w, tab_bar_h, s, active.bg);
        }

        // 被拖拽的标签最后绘制（顶层），用 active 样式突出。
        for tab in tabs.iter().filter(|t| t.is_dragging) {
            crate::tab_chrome::push_active_tab_fill(fills, tab.x, tab_y, tab.tab_body_w, tab_bar_h, s, tab.bg);
        }

        // 相邻非激活标签之间的竖线（在标签底色之上、文本图标之下）
        for i in 0..tabs.len().saturating_sub(1) {
            if !tabs[i].is_active && !tabs[i + 1].is_active {
                let gap_center = tabs[i].x + tabs[i].tab_w - s * 0.5;
                let inset = layout::TAB_SEPARATOR_INSET * s;
                let sep_w = s.max(1.0);
                fills.push(rect_fill(
                    gap_center - sep_w * 0.5,
                    tab_y + inset,
                    sep_w,
                    tab_bar_h - 2.0 * inset,
                    self.chrome_palette.tab_separator,
                ));
            }
        }

        for tab in &tabs {
            let tab_text_color = if tab.is_crashed {
                self.chrome_palette.tab_crashed
            } else if tab.is_active {
                self.chrome_palette.tab_text
            } else {
                self.chrome_palette.page_hint
            };
            let icon_cx = tab.x + 12.0 * s + icon_size * 0.5;
            if tab.needs_attention && !tab.is_loading {
                push_circle_fill(
                    fills,
                    tab.x + 8.0 * s,
                    tab_y + 8.0 * s,
                    4.0 * s,
                    self.chrome_palette.tab_attention,
                );
            }
            if tab.is_loading {
                crate::tab_chrome::push_loading_spinner(
                    fills,
                    icon_cx,
                    icon_cy,
                    icon_size,
                    spinner_angle,
                    self.chrome_palette.loading_indicator,
                );
            } else if self.font_id.is_some() {
                let favicon_url = tab.page_url.as_deref().or(Some("zero://newtab"));
                crate::tab_favicon::render_tab_favicon(
                    &mut self.font_loader,
                    glyphs,
                    tab.id,
                    favicon_url,
                    tab.html_hint,
                    icon_cx,
                    icon_cy,
                    icon_size,
                    tab_text_color,
                );
            }

            if self.font_id.is_some() && !tab.is_pinned && tab.tab_w >= layout::TAB_TITLE_HIDE_WIDTH * s {
                let text_area_w = tab.tab_w - text_left_inset - 32.0 * s;
                let truncated = self.truncate_ui_text(&tab.label, text_area_w.max(0.0), font_size);
                self.draw_ui_text(
                    &truncated,
                    tab.x + text_left_inset,
                    text_y,
                    font_size,
                    tab_text_color,
                    glyphs,
                );
            }

            if tab.is_pinned {
                continue;
            }

            // muted / crashed 状态图标（位于 close 按钮左侧）。loading/attention 已单独绘制。
            if tab.is_crashed {
                let st_x = tab.x + tab.tab_w - 52.0 * s;
                let st_hit = 24.0 * s;
                let st_cx = st_x + st_hit / 2.0;
                let st_cy = tab_y + tab_bar_h / 2.0;
                let st_hovered = self.pointer_in_rect(st_x, tab_y, st_hit, tab_bar_h);
                if st_hovered {
                    push_circle_fill(fills, st_cx, st_cy, st_hit, self.chrome_palette.tab_hover_bg);
                }
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    crate::ui_icons::Icon::AlertTriangle,
                    st_cx,
                    st_cy,
                    14.0 * s,
                    self.chrome_palette.tab_crashed,
                );
            } else if tab.is_muted {
                let st_x = tab.x + tab.tab_w - 52.0 * s;
                let st_hit = 24.0 * s;
                let st_cx = st_x + st_hit / 2.0;
                let st_cy = tab_y + tab_bar_h / 2.0;
                let st_hovered = self.pointer_in_rect(st_x, tab_y, st_hit, tab_bar_h);
                if st_hovered {
                    push_circle_fill(fills, st_cx, st_cy, st_hit, self.chrome_palette.tab_hover_bg);
                }
                let st_color = if tab.is_active {
                    self.chrome_palette.tab_text
                } else {
                    self.chrome_palette.page_hint
                };
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    crate::ui_icons::Icon::VolumeOff,
                    st_cx,
                    st_cy,
                    14.0 * s,
                    st_color,
                );
            }

            // close 按钮：仅当标签宽度 >= COMPRESSED 时绘制。
            // 极限压缩模式下（仅 favicon）省略 close 按钮，点击行为通过中键关闭或右键菜单兜底。
            if tab.tab_w >= layout::TAB_MIN_WIDTH_COMPRESSED * s {
                let close_x = tab.x + tab.tab_w - 28.0 * s;
                let close_hit = 24.0 * s;
                let close_cx = close_x + close_hit / 2.0;
                let close_cy = tab_y + tab_bar_h / 2.0;
                let close_hovered = self.pointer_in_rect(close_x, tab_y, close_hit, tab_bar_h);
                if close_hovered {
                    push_circle_fill(fills, close_cx, close_cy, close_hit, self.chrome_palette.tab_hover_bg);
                }
                let close_color = if close_hovered {
                    self.chrome_palette.address_bar_text
                } else if tab.is_active {
                    self.chrome_palette.tab_close
                } else {
                    self.chrome_palette.page_hint
                };
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    crate::ui_icons::Icon::Close,
                    close_cx,
                    close_cy,
                    12.0 * s,
                    close_color,
                );
            }

            self.tab_layout.push((tab.id, tab.x, tab.tab_w));
        }

        // 新建标签按钮 (+)，紧跟最后一个标签
        {
            let btn_x = x;
            let is_hovered = {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= btn_x && mx < btn_x + new_tab_btn_w && my >= tab_y && my < tab_strip_h
            };
            if is_hovered {
                push_circle_fill(
                    fills,
                    btn_x + new_tab_btn_w / 2.0,
                    tab_y + tab_bar_h / 2.0,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let plus_color = if is_hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.new_tab_button
            };
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                crate::ui_icons::Icon::Plus,
                btn_x + new_tab_btn_w / 2.0,
                tab_y + tab_bar_h / 2.0,
                layout::CHROME_ICON_SIZE * s,
                plus_color,
            );
        }

        if self.uses_custom_window_controls() {
            self.render_window_controls(fills, glyphs, width, tab_y, s);
        }
    }

    /// 渲染窗口控制按钮（最小化 / 最大化 / 关闭）
    fn render_window_controls(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        tab_y: f32,
        s: f32,
    ) {
        let btn_w = layout::WINDOW_CONTROL_BTN_WIDTH * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let x0 = self.window_controls_origin_x(width as f32, s);
        let icon = self.chrome_palette.window_control_icon;
        // 窗口图标描边 1px，与 CPU/GPU backend 一致（CPU 的 floor/ceil 取整对 1px 矩形无放大效应）。
        let thickness = 1.0 * s;

        for i in 0..3 {
            let bx = x0 + i as f32 * btn_w;
            let hovered = self.window_control_hover == Some(i);
            let bg = if i == 2 && hovered {
                self.chrome_palette.window_control_close_hover
            } else if hovered {
                self.chrome_palette.window_control_hover
            } else {
                self.chrome_palette.tab_bar_bg
            };
            fills.push(rect_fill(bx, tab_y, btn_w, tab_bar_h, bg));

            let cx = bx + btn_w / 2.0;
            let cy = tab_y + tab_bar_h / 2.0;

            match i {
                0 => {
                    let line_w = 10.0 * s;
                    fills.push(rect_fill(
                        cx - line_w / 2.0,
                        cy - thickness / 2.0,
                        line_w,
                        thickness,
                        icon,
                    ));
                }
                1 if self.window_is_maximized => {
                    // 还原态：前框在左下、后框在右上，偏移约 30% 边长。
                    // 后框被前框遮挡左下，只露上+右两条边。
                    // size/off 取整数，避免亚像素导致四条边粗细不一致。
                    let size = 8.0 * s;
                    let off = (size * 0.3).round();
                    let back_left = cx - size / 2.0 + off / 2.0;
                    let back_top = cy - size / 2.0 - off / 2.0;
                    let front_left = cx - size / 2.0 - off / 2.0;
                    let front_top = cy - size / 2.0 + off / 2.0;
                    draw_hollow_square_top_right_only(fills, back_left, back_top, size, thickness, icon);
                    draw_hollow_square(fills, front_left, front_top, size, thickness, icon);
                }
                1 => {
                    let size = 10.0 * s;
                    draw_hollow_square(fills, cx - size / 2.0, cy - size / 2.0, size, thickness, icon);
                }
                2 => {
                    let close_color = if hovered {
                        Color {
                            r: 255,
                            g: 255,
                            b: 255,
                            a: 255,
                        }
                    } else {
                        icon
                    };
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Close,
                        cx,
                        cy,
                        12.0 * s,
                        close_color,
                    );
                }
                _ => {}
            }
        }
    }

    /// 渲染导航按钮（矢量图标，不依赖字体符号覆盖）
    fn render_nav_buttons(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        y: f32,
        _font_size: f32,
        s: f32,
    ) {
        let x = layout::NAV_SECTION_LEADING_PAD * s;
        let btn_w = layout::NAV_BUTTON_WIDTH * s;
        let btn_h = layout::ADDRESS_BAR_HEIGHT * s;
        let cy = y + btn_h / 2.0;

        let nav_icons = [
            crate::ui_icons::Icon::ChevronLeft,
            crate::ui_icons::Icon::ChevronRight,
            // 加载中显示停止（X）图标，点击停止加载。
            if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
                crate::ui_icons::Icon::Close
            } else {
                crate::ui_icons::Icon::Refresh
            },
            crate::ui_icons::Icon::Home,
        ];
        let can_back = self.shell.active_tab().is_some_and(|tab| tab.history_index() > 0);
        let can_forward = self.shell.active_tab().is_some_and(|tab| {
            let history = tab.navigation_history();
            !history.is_empty() && tab.history_index() < history.len() - 1
        });
        let nav_enabled = [can_back, can_forward, true, true];
        let hover_diameter = layout::NAV_BUTTON_HOVER_DIAMETER * s;

        for (i, &icon) in nav_icons.iter().enumerate() {
            let bx = x + btn_w * i as f32;
            let cx = bx + btn_w / 2.0;
            let enabled = nav_enabled[i];
            let hovered = enabled && self.pointer_in_rect(bx, y, btn_w, btn_h);
            let pressed = hovered && self.left_button_down && enabled;
            if pressed {
                push_circle_fill(fills, cx, cy, hover_diameter, self.chrome_palette.nav_button_pressed);
            } else if hovered {
                push_circle_fill(fills, cx, cy, hover_diameter, self.chrome_palette.tab_hover_bg);
            }
            let color = if !enabled {
                self.chrome_palette.nav_button_disabled
            } else if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.nav_button
            };
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                icon,
                cx,
                cy,
                layout::CHROME_ICON_SIZE * s,
                color,
            );
        }
    }


    fn byte_at_char(text: &str, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        text.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(text.len())
    }

    fn chars_slice(text: &str, start: usize, end: usize) -> &str {
        let b0 = Self::byte_at_char(text, start);
        let b1 = Self::byte_at_char(text, end);
        &text[b0..b1]
    }

    /// 渲染书签栏
    fn render_bookmarks_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        y: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let bar_h = layout::BOOKMARKS_BAR_HEIGHT * s;
        fills.push(rect_fill(
            0.0,
            y,
            width as f32,
            bar_h,
            self.chrome_palette.bookmarks_bar_bg,
        ));
        fills.push(rect_fill(
            0.0,
            y + bar_h - s.max(1.0),
            width as f32,
            s.max(1.0),
            self.chrome_palette.separator,
        ));

        let font_size = layout::BOOKMARKS_BAR_FONT_SIZE * s;
        let icon_size = layout::BOOKMARKS_BAR_ICON_SIZE * s;
        let mut bx = layout::BOOKMARKS_BAR_PAD_H * s;
        let item_h = (bar_h - 4.0 * s).max(0.0);
        let item_y = y + (bar_h - item_h) * 0.5;
        let icon_cy = y + bar_h * 0.5;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let label_w = self.measure_ui_text_width(label, font_size);
            let item_w = layout::BOOKMARKS_BAR_ITEM_PAD_H * s * 2.0
                + icon_size
                + layout::BOOKMARKS_BAR_ICON_GAP * s
                + label_w;

            let mx = self.mouse_pos.0 as f32;
            let my = self.mouse_pos.1 as f32;
            if mx >= bx && mx < bx + item_w && my >= y && my < y + bar_h {
                push_rounded_rect_fill(
                    fills,
                    bx,
                    item_y,
                    item_w,
                    item_h,
                    layout::BOOKMARKS_BAR_ITEM_RADIUS * s,
                    self.chrome_palette.bookmarks_bar_hover_bg,
                );
            }

            let icon_cx = bx + layout::BOOKMARKS_BAR_ITEM_PAD_H * s + icon_size * 0.5;
            let bm_url = bm.url();
            let favicon_ch =
                crate::tab_favicon::bookmark_favicon_glyph(&mut self.font_loader, bm_url, icon_size);
            glyphs.push(GlyphDraw {
                ch: favicon_ch,
                x: icon_cx - icon_size * 0.5,
                baseline_y: icon_cy + icon_size * 0.5,
                color: self.chrome_palette.bookmarks_bar_icon,
                font_id: crate::tab_favicon::FAVICON_FONT_ID,
                font_size: icon_size,
                rotation: 0.0,
            });
            let text_x = bx + layout::BOOKMARKS_BAR_ITEM_PAD_H * s + icon_size + layout::BOOKMARKS_BAR_ICON_GAP * s;
            let (text_top, _) = self.ui_text_centered_in_height(bar_h, font_size);
            self.draw_ui_text(
                label,
                text_x,
                y + text_top,
                font_size,
                self.chrome_palette.bookmarks_bar_text,
                glyphs,
            );

            bx += item_w + layout::BOOKMARKS_BAR_ITEM_GAP * s;
            if bx > width as f32 - 40.0 * s {
                break;
            }
        }
    }

    /// 渲染页面视口 drop shadow（非最大化时；最大化/全屏时跳过）。
    ///
    /// 在 `render_page_frame` 之前调用，阴影绘制在页面背景之下、chrome 背景之上。
    fn render_page_frame_shadow(&self, shadows: &mut Vec<ShadowPrimitive>, x: f32, y: f32, w: f32, h: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        if self.window_is_maximized || self.window_is_fullscreen {
            return;
        }
        let s = self.scale_factor;
        let blur = layout::PAGE_FRAME_SHADOW_BLUR * s;
        let offset_y = layout::PAGE_FRAME_SHADOW_OFFSET_Y * s;
        let alpha = layout::PAGE_FRAME_SHADOW_ALPHA;
        // 阴影颜色：黑色按 alpha 叠加。亮色/暗色主题都用黑底低不透明度，避免额外配色。
        let color = Color { r: 0, g: 0, b: 0, a: alpha };
        shadows.push(ShadowPrimitive {
            rect: Rect::new(x, y, w, h),
            color,
            offset_x: 0.0,
            offset_y,
            blur_radius: blur,
            spread_radius: 0.0,
            inset: false,
        });
    }

    /// 渲染页面视口背景（外圈边框色圆角 + 内圈页面底色）
    fn render_page_frame(&self, fills: &mut Vec<FillPrimitive>, x: f32, y: f32, w: f32, h: f32, s: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let radius = self.effective_page_frame_radius();
        let border = self.effective_page_frame_border();
        if radius <= 0.0 && border <= 0.0 {
            fills.push(rect_fill(x, y, w, h, self.chrome_palette.page_bg));
            return;
        }
        let _ = s;
        let inner_r = (radius - border).max(0.0);
        push_rounded_rect_fill(fills, x, y, w, h, radius, self.chrome_palette.separator);
        push_rounded_rect_fill(
            fills,
            x + border,
            y + border,
            w - 2.0 * border,
            h - 2.0 * border,
            inner_r,
            self.chrome_palette.page_bg,
        );
    }

    /// 清掉圆角外溢出的页面像素（内圈用边框色、外圈用 gutter 色）。
    fn render_page_frame_corner_masks(&self, fills: &mut Vec<FillPrimitive>, width: u32, height: u32, s: f32) {
        let outer_r = self.effective_page_frame_radius();
        if outer_r <= 0.0 {
            return;
        }
        let _ = s;
        let (fx, fy, fw, fh) = self.page_frame_rect_for(width, height);
        push_rounded_rect_outside_corner_masks(fills, fx, fy, fw, fh, outer_r, self.chrome_palette.tab_active_bg);

        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let border = self.effective_page_frame_border();
        let inner_r = (outer_r - border).max(0.0);
        push_rounded_rect_outside_corner_masks(fills, cx, cy, cw, ch, inner_r, self.chrome_palette.separator);
    }

    /// 渲染页面视口灰色描边（在内容之上绘制，避免圆角处被内容污染）
    fn render_page_frame_border(&self, fills: &mut Vec<FillPrimitive>, x: f32, y: f32, w: f32, h: f32, s: f32) {
        let radius = self.effective_page_frame_radius();
        let border = self.effective_page_frame_border();
        if w <= 0.0 || h <= 0.0 || radius <= 0.0 || border <= 0.0 {
            return;
        }
        let _ = s;
        push_rounded_rect_border(fills, x, y, w, h, radius, border, self.chrome_palette.separator);
    }

    /// Wayland 无系统装饰时，为非最大化窗口绘制 1px 外框描边。
    fn render_custom_window_frame_border(&self, fills: &mut Vec<FillPrimitive>, width: u32, height: u32, s: f32) {
        if !self.uses_custom_window_controls() || self.window_is_maximized {
            return;
        }

        let border = layout::WINDOW_FRAME_BORDER * s;
        let w = width as f32;
        let h = height as f32;
        // 使用专用的 window_frame_border 颜色，与 separator 区分，
        // 保证亮/暗色主题下窗口外框都有明确的视觉边界。
        // 注：窗口几何本身为直角（Wayland client-side rounded window 需 compositor 支持），
        // 故自绘描边保持直角，避免与实际窗口形状冲突产生透明缺口。
        let color = self.chrome_palette.window_frame_border;

        fills.push(rect_fill(0.0, 0.0, w, border, color));
        fills.push(rect_fill(0.0, h - border, w, border, color));
        fills.push(rect_fill(0.0, border, border, h - 2.0 * border, color));
        fills.push(rect_fill(w - border, border, border, h - 2.0 * border, color));
    }

    /// 渲染页面内容
    #[allow(clippy::too_many_arguments)]
    fn render_page_content(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        content_x: f32,
        page_y: f32,
        font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let content_y_offset = 0.0;

        let (title, url, is_loading) = match self.shell.active_tab() {
            Some(tab) => (
                tab.title().unwrap_or("").to_string(),
                tab.url().unwrap_or("").to_string(),
                tab.is_loading(),
            ),
            None => return,
        };

        let mut y = page_y + content_y_offset;

        // 获取当前标签的滚动偏移
        let tab_id = self.shell.active_tab_id().unwrap();
        let scroll = self.tab_scroll_state(tab_id);
        let layout = self.page_scroll_layout(tab_id);
        let has_composite_paint = self.tabs.snapshot(tab_id).is_some_and(|s| s.should_composite_paint());
        let compositor_status = self.compositor_status();

        if compositor_controls_page(compositor_status) {
            if compositor_status == crate::compositor_client::CompositorStatus::Healthy
                && self
                    .tabs
                    .compositor_frame(tab_id)
                    .is_some()
            {
                return;
            }
        } else if has_composite_paint
            && self.render_active_webview(
                fills,
                glyphs,
                layout.viewport_x,
                layout.viewport_y,
                fid,
                scroll.x,
                scroll.y,
                layout.viewport_w,
                layout.viewport_h,
            )
        {
            return;
        }

        if !title.is_empty() {
            self.draw_ui_text(
                &title,
                content_x + 20.0 * s,
                y + 20.0 * s,
                24.0 * s,
                self.chrome_palette.page_title,
                glyphs,
            );
            y += 52.0 * s;
        }

        if !url.is_empty() {
            self.draw_ui_text(
                &url,
                content_x + 20.0 * s,
                y,
                12.0 * s,
                self.chrome_palette.page_url,
                glyphs,
            );
            y += 28.0 * s;
        }

        if is_loading {
            self.draw_ui_text(
                "Loading...",
                content_x + 20.0 * s,
                y,
                font_size,
                self.chrome_palette.page_hint,
                glyphs,
            );
        } else if title.is_empty() && url.is_empty() {
            self.draw_ui_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                content_x + 20.0 * s,
                y,
                font_size,
                self.chrome_palette.page_hint,
                glyphs,
            );
        }
    }

    /// 渲染活跃 WebView 的页面图元。
    #[allow(clippy::too_many_arguments)]
    fn render_active_webview(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        viewport_x: f32,
        viewport_y: f32,
        fallback_font_id: u32,
        scroll_x: f32,
        scroll_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> bool {
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return false,
        };

        let primitives = match self.tabs.last_render(tab_id).map(|render| &render.primitives) {
            Some(primitives) => primitives,
            None => return false,
        };

        // 性能门禁优化 S2（2026-08-08）：旧实现每帧全量深克隆页面图元只为只读的
        // 选区高亮循环 + append_webview_primitives（均只需 &）——改为直接借用
        let page_primitives = primitives;

        let clip_top = viewport_y;
        let clip_bottom = viewport_y + viewport_h;
        let content_y_draw = viewport_y - scroll_y;
        let content_x_draw = viewport_x - scroll_x;
        let s = self.scale_factor;
        let border = self.effective_page_frame_border();
        let radius = (self.effective_page_frame_radius() - border).max(0.0);
        let clip_rounded = Some((viewport_x, viewport_y, viewport_w, viewport_h, radius));

        if let Some(sel) = self.page_selection.get(&tab_id)
            && !sel.is_collapsed()
        {
            let (start, end) = sel.normalized();
            let end = end.min(page_primitives.glyphs.len().saturating_sub(1));
            if start <= end {
                for glyph in &page_primitives.glyphs[start..=end] {
                    let x = glyph.x * s + content_x_draw;
                    let top = glyph.y * s + content_y_draw - glyph.font_size * s;
                    let w = glyph.font_size * s * 0.55;
                    let h = glyph.font_size * s;
                    if top + h <= clip_top || top >= clip_bottom {
                        continue;
                    }
                    if let Some((rx, ry, rw, rh, rr)) = clip_rounded {
                        push_fill_clipped_to_rounded_rect(
                            fills,
                            x,
                            top,
                            w.max(1.0),
                            h,
                            self.chrome_palette.text_selection_bg,
                            rx,
                            ry,
                            rw,
                            rh,
                            rr,
                        );
                    } else {
                        fills.push(rect_fill(x, top, w.max(1.0), h, self.chrome_palette.text_selection_bg));
                    }
                }
            }
        }

        append_webview_primitives(
            page_primitives,
            fills,
            glyphs,
            content_x_draw,
            content_y_draw,
            fallback_font_id,
            self.scale_factor,
            Some((clip_top, clip_bottom)),
            clip_rounded,
        )
    }

    /// 渲染浮动查找栏
    fn render_find_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (bar_x, y, bar_w, bar_h) = self.find_bar_rect_for(width, height);
        let radius = layout::FIND_BAR_FLOAT_RADIUS * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, bar_x, y, bar_w, bar_h, radius, self.chrome_palette.find_bar_border);
        push_rounded_rect_fill(
            fills,
            bar_x + border,
            y + border,
            bar_w - 2.0 * border,
            bar_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.find_bar_bg,
        );

        let display = if self.find_input.is_empty() {
            "Find...".to_string()
        } else {
            self.find_input.clone()
        };
        let text_color = if self.find_input.is_empty() {
            self.chrome_palette.find_match_text
        } else {
            self.chrome_palette.find_bar_text
        };
        self.draw_ui_text(
            &display,
            bar_x + 12.0 * s,
            y + (bar_h - font_size) * 0.5,
            font_size,
            text_color,
            glyphs,
        );

        let find_state = self.shell.find_state();
        // 选项切换按钮（区分大小写 / 全字匹配）位置，位于匹配数与 prev 按钮之间。
        let case_btn_x = bar_x + bar_w - 160.0 * s;
        let whole_btn_x = bar_x + bar_w - 130.0 * s;
        if find_state.total_matches() > 0 {
            // 色阶：当前项用主文本色强调，分隔符与总数用次要色，符合 Chrome 习惯。
            let current_str = find_state.current_match().to_string();
            let rest_str = format!("/{}", find_state.total_matches());
            let match_x = bar_x + bar_w - 200.0 * s;
            let match_y = y + (bar_h - font_size) * 0.5;
            let current_w = self.measure_ui_text_width(&current_str, font_size);
            self.draw_ui_text(&current_str, match_x, match_y, font_size, self.chrome_palette.find_bar_text, glyphs);
            self.draw_ui_text(
                &rest_str,
                match_x + current_w,
                match_y,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
            // 循环提示后缀
            if let Some(wrap) = find_state.last_wrap() {
                let suffix = match wrap {
                    FindWrapHint::WrappedToStart => "  ↻",
                    FindWrapHint::WrappedToEnd => "  ↺",
                };
                let rest_w = self.measure_ui_text_width(&rest_str, font_size);
                self.draw_ui_text(
                    suffix,
                    match_x + current_w + rest_w,
                    match_y,
                    font_size,
                    self.chrome_palette.find_match_text,
                    glyphs,
                );
            }
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 200.0 * s;
            self.draw_ui_text(
                "No matches",
                no_match_x,
                y + (bar_h - font_size) * 0.5,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
        }

        // 选项切换按钮：激活态用主色背景 + 白字，未激活态用次要文字。
        let opt_btn_w = 28.0 * s;
        let opt_font = font_size * 0.82;
        let opt_y = y + (bar_h - opt_font) * 0.5;
        for (bx, label, active) in [
            (case_btn_x, "Aa", find_state.case_sensitive()),
            (whole_btn_x, "W•", find_state.whole_word()),
        ] {
            let hovered = self.pointer_in_rect(bx, y, opt_btn_w, bar_h);
            if active {
                push_rounded_rect_fill(
                    fills,
                    bx + 2.0 * s,
                    y + 3.0 * s,
                    opt_btn_w - 4.0 * s,
                    bar_h - 6.0 * s,
                    4.0 * s,
                    self.chrome_palette.find_active_option_bg,
                );
            } else if hovered {
                push_circle_fill(
                    fills,
                    bx + opt_btn_w / 2.0,
                    y + bar_h / 2.0,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let color = if active {
                self.chrome_palette.find_active_option_text
            } else if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.find_match_text
            };
            let lw = self.measure_ui_text_width(label, opt_font);
            self.draw_ui_text(label, bx + (opt_btn_w - lw) / 2.0, opt_y, opt_font, color, glyphs);
        }

        let btn_y = y + (bar_h - font_size) * 0.5;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
        let btn_w = 24.0 * s;
        let icon_size = 16.0 * s;
        let btn_cy = btn_y + font_size * 0.5;

        for (bx, icon) in [
            (prev_x, crate::ui_icons::Icon::ChevronUp),
            (next_x, crate::ui_icons::Icon::ChevronDown),
            (close_x, crate::ui_icons::Icon::Close),
        ] {
            let icon_cx = bx + btn_w / 2.0;
            let hovered = self.pointer_in_rect(bx, y, btn_w, bar_h);
            if hovered {
                // 图标按钮类统一使用圆形 hover（与导航按钮、地址栏 slot 一致）。
                push_circle_fill(
                    fills,
                    icon_cx,
                    btn_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let icon_color = if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.find_bar_text
            };
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                icon,
                icon_cx,
                btn_cy,
                icon_size,
                icon_color,
            );
        }
    }

    /// 渲染自动补全下拉
    fn render_autocomplete(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (bar_x, _, bar_w, _) = self.address_bar_layout();
        let dropdown_y = layout::TOOLBAR_HEIGHT * s;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let dropdown_h = visible_count as f32 * row_h;
        let radius = layout::AUTOCOMPLETE_DROPDOWN_RADIUS * s;
        let pad_h = layout::AUTOCOMPLETE_ROW_PAD_H * s;
        let pad_v = layout::AUTOCOMPLETE_ROW_PAD_V * s;
        let title_size = font_size * 0.92;
        let url_size = font_size * 0.78;

        push_rounded_rect_fill(
            fills,
            bar_x,
            dropdown_y,
            bar_w,
            dropdown_h,
            radius,
            self.chrome_palette.separator,
        );
        push_rounded_rect_fill(
            fills,
            bar_x + s.max(1.0),
            dropdown_y + s.max(1.0),
            bar_w - 2.0 * s.max(1.0),
            dropdown_h - 2.0 * s.max(1.0),
            (radius - s.max(1.0)).max(0.0),
            self.chrome_palette.autocomplete_bg,
        );

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * row_h;
            let is_hovered = self.autocomplete.hovered_index == Some(i);
            let is_selected = self.autocomplete.hovered_index.is_none() && self.autocomplete.selected_index == Some(i);
            if is_hovered {
                fills.push(rect_fill(
                    bar_x + s.max(1.0),
                    row_y,
                    bar_w - 2.0 * s.max(1.0),
                    row_h,
                    self.chrome_palette.autocomplete_hover_bg,
                ));
            } else if is_selected {
                fills.push(rect_fill(
                    bar_x + s.max(1.0),
                    row_y,
                    bar_w - 2.0 * s.max(1.0),
                    row_h,
                    self.chrome_palette.autocomplete_selected_bg,
                ));
            }

            // 来源标记：图标化（搜索=放大镜，书签=实心星，历史=时钟字符）
            let icon_size = font_size * 0.95;
            let icon_cx = bar_x + pad_h + icon_size * 0.5;
            let icon_cy = row_y + row_h * 0.5;
            let text_x = bar_x + pad_h + icon_size + 10.0 * s;
            let title_area_w = (bar_w - (text_x - bar_x) - pad_h).max(0.0);

            match sug.source() {
                SuggestionSource::Search => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Search,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_url,
                    );
                    // 主标题：搜索词（深色，加粗感由字号体现）
                    let title = sug.title();
                    let truncated = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    // 副标题："<query> — 在 <Engine> 中搜索"（灰色）
                    let engine_name = self.shell.settings().search_engine.display_name();
                    let hint = format!("{truncated} — {engine_name} 搜索");
                    let truncated_hint = self.truncate_ui_text(&hint, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_hint,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
                SuggestionSource::Bookmark => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::StarFilled,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_bookmark,
                    );
                    let title = sug.title();
                    let truncated_title = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated_title,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    let url = sug.url();
                    let truncated_url = self.truncate_ui_text(url, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_url,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
                SuggestionSource::History => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Clock,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_url,
                    );
                    let title = sug.title();
                    let truncated_title = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated_title,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    let url = sug.url();
                    let truncated_url = self.truncate_ui_text(url, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_url,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
            }
        }
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&mut self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        if self.font_id.is_none() {
            return;
        }

        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu_total_height();
        let font_size = layout::CHROME_FONT_SIZE * s;
        let radius = layout::CONTEXT_MENU_RADIUS * s;
        let border = s.max(1.0);
        let pad_h = layout::CONTEXT_MENU_PAD_H * s;

        push_rounded_rect_fill(fills, menu_x, menu_y, menu_w, menu_h, radius, self.chrome_palette.context_menu_separator);
        push_rounded_rect_fill(
            fills,
            menu_x + border,
            menu_y + border,
            menu_w - 2.0 * border,
            menu_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.context_menu_bg,
        );

        for (i, item) in self.context_menu.items.iter().enumerate() {
            let row_y = menu_y + self.context_menu_row_y(i);

            if item.is_separator() {
                // separator 占紧凑高度，绘制居中的细线。
                let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
                let sep_y = row_y + sep_h / 2.0;
                fills.push(rect_fill(
                    menu_x + pad_h,
                    sep_y,
                    menu_w - 2.0 * pad_h,
                    border,
                    self.chrome_palette.context_menu_separator,
                ));
                continue;
            }

            let is_hovered = self.context_menu.hovered_index == Some(i);
            let is_disabled = !item.enabled();
            let is_sub_open = self.context_menu.open_sub_menu == Some(i);

            // hover 或子菜单展开时高亮（submenu 父项展开时保持高亮）。
            if (is_hovered || is_sub_open) && !is_disabled {
                fills.push(rect_fill(
                    menu_x + border,
                    row_y,
                    menu_w - 2.0 * border,
                    row_h,
                    self.chrome_palette.context_menu_hover_bg,
                ));
            }

            let text_color = if is_disabled {
                self.chrome_palette.page_hint
            } else {
                self.chrome_palette.context_menu_text
            };

            let label = {
                let raw = item.label();
                if !item.is_sub_menu() && item.shortcut().is_some() {
                    let shortcut_w = self.measure_ui_text_width(item.shortcut().unwrap_or(""), font_size);
                    let max_w = menu_w - 2.0 * pad_h - shortcut_w - 8.0 * s;
                    self.truncate_ui_text(raw, max_w, font_size)
                } else {
                    raw.to_string()
                }
            };
            self.draw_ui_text(
                &label,
                menu_x + pad_h,
                row_y + (row_h - font_size) * 0.5,
                font_size,
                text_color,
                glyphs,
            );

            // 快捷键提示（右对齐，子菜单项不显示以免与箭头冲突）。
            if !item.is_sub_menu() {
                if let Some(shortcut) = item.shortcut() {
                    let w = self.measure_ui_text_width(shortcut, font_size);
                    let x = menu_x + menu_w - pad_h - w;
                    self.draw_ui_text(
                        shortcut,
                        x,
                        row_y + (row_h - font_size) * 0.5,
                        font_size,
                        self.chrome_palette.page_hint,
                        glyphs,
                    );
                }
            }

            // 子菜单右侧箭头。
            if item.is_sub_menu() {
                let arrow_cx = menu_x + menu_w - pad_h - 6.0 * s;
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    crate::ui_icons::Icon::ChevronRight,
                    arrow_cx,
                    row_y + row_h * 0.5,
                    12.0 * s,
                    text_color,
                );
            }
        }

        // 渲染展开的子菜单面板（默认右侧浮层；右侧空间不足时自动翻转到左侧）。
        if let Some(parent_idx) = self.context_menu.open_sub_menu
            && let Some(parent) = self.context_menu.items.get(parent_idx)
            && let Some(children) = parent.children()
            && !children.is_empty()
        {
            let (sub_x, sub_y, sub_w, sub_h) = self.sub_menu_panel_rect(parent_idx);
            push_rounded_rect_fill(fills, sub_x, sub_y, sub_w, sub_h, radius, self.chrome_palette.context_menu_separator);
            push_rounded_rect_fill(
                fills,
                sub_x + border,
                sub_y + border,
                sub_w - 2.0 * border,
                sub_h - 2.0 * border,
                (radius - border).max(0.0),
                self.chrome_palette.context_menu_bg,
            );

            for (ci, child) in children.iter().enumerate() {
                let crow_y = sub_y + self.sub_menu_row_y(parent_idx, ci);
                if child.is_separator() {
                    let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
                    let sep_y = crow_y + sep_h / 2.0;
                    fills.push(rect_fill(
                        sub_x + pad_h,
                        sep_y,
                        sub_w - 2.0 * pad_h,
                        border,
                        self.chrome_palette.context_menu_separator,
                    ));
                    continue;
                }
                let c_hovered = self.context_menu.sub_menu_hovered == Some(ci);
                let c_disabled = !child.enabled();
                if c_hovered && !c_disabled {
                    fills.push(rect_fill(
                        sub_x + border,
                        crow_y,
                        sub_w - 2.0 * border,
                        row_h,
                        self.chrome_palette.context_menu_hover_bg,
                    ));
                }
                let c_color = if c_disabled {
                    self.chrome_palette.page_hint
                } else {
                    self.chrome_palette.context_menu_text
                };
                self.draw_ui_text(
                    child.label(),
                    sub_x + pad_h,
                    crow_y + (row_h - font_size) * 0.5,
                    font_size,
                    c_color,
                    glyphs,
                );
            }
        }
    }

    /// 渲染链接悬停浮动状态栏（Chrome 风格：左下角胶囊，宽度随 URL 内容）
    fn render_floating_link_status(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        s: f32,
    ) {
        let Some(url) = self.hovered_link_url.as_deref() else {
            return;
        };
        if self.font_id.is_none() {
            return;
        }

        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        if cw <= f32::EPSILON || ch <= f32::EPSILON {
            return;
        }

        let margin = layout::STATUS_BAR_FLOAT_MARGIN * s;
        let pad_h = layout::STATUS_BAR_FLOAT_PAD_H * s;
        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let font_size = 11.0 * s;
        let max_text_w = (cw - 2.0 * margin - 2.0 * pad_h).max(0.0);
        let text = self.truncate_ui_text(url, max_text_w, font_size);
        let text_w = self.measure_ui_text_width(&text, font_size);
        let pill_w = text_w + 2.0 * pad_h;
        let pill_x = cx + margin;
        let pill_y = cy + ch - status_h - margin;
        let radius = layout::STATUS_BAR_FLOAT_RADIUS * s;
        let border = s;

        push_rounded_rect_fill(
            fills,
            pill_x,
            pill_y,
            pill_w,
            status_h,
            radius,
            self.chrome_palette.separator,
        );
        push_rounded_rect_fill(
            fills,
            pill_x + border,
            pill_y + border,
            (pill_w - 2.0 * border).max(0.0),
            (status_h - 2.0 * border).max(0.0),
            (radius - border).max(0.0),
            self.chrome_palette.tab_active_bg,
        );

        self.draw_ui_text(
            &text,
            pill_x + pad_h,
            pill_y + 3.0 * s,
            font_size,
            self.chrome_palette.status_text,
            glyphs,
        );
    }


    /// 渲染浮动下载面板（右下角）
    fn render_download_panel(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        _font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (panel_x, panel_y, panel_w, panel_h) = self.download_panel_rect_for(width, height);
        let radius = layout::DOWNLOAD_PANEL_RADIUS * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, panel_x, panel_y, panel_w, panel_h, radius, self.chrome_palette.separator);
        push_rounded_rect_fill(
            fills,
            panel_x + border,
            panel_y + border,
            panel_w - 2.0 * border,
            panel_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.download_bar_bg,
        );

        let downloads = self.shell.downloads();
        let active: Vec<_> = downloads.iter().filter(|d| d.is_active()).collect();
        if let Some(dl) = active.first() {
            let font_size = 11.0 * s;
            let title_size = 12.0 * s;

            self.draw_ui_text(
                "Downloading",
                panel_x + 12.0 * s,
                panel_y + 10.0 * s,
                title_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );

            let name_text = dl.filename();
            self.draw_ui_text(
                name_text,
                panel_x + 12.0 * s,
                panel_y + 28.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );

            let progress = dl.progress();
            let bar_width = panel_w - 24.0 * s;
            let bar_start_x = panel_x + 12.0 * s;
            let bar_top = panel_y + panel_h - 18.0 * s;
            let bar_inner_h = 6.0 * s;

            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width,
                bar_inner_h,
                self.chrome_palette.separator,
            ));
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width * progress,
                bar_inner_h,
                self.chrome_palette.download_bar_fill,
            ));

            let pct_text = format!("{:.0}%", progress * 100.0);
            let pct_w = self.measure_ui_text_width(&pct_text, font_size);
            self.draw_ui_text(
                &pct_text,
                panel_x + panel_w - 12.0 * s - pct_w,
                panel_y + 10.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );
        }
    }
}
