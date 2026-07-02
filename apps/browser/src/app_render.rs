// 浏览器 UI 渲染方法（从 app.rs 通过 include! 引入）
//
// 此文件在编译时被 app.rs include，共享同一个模块作用域。

// --- BrowserApp 渲染 impl ---

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
        // DC-14 替换式迁移分离点：页面内容（render_page_content）与 chrome 浮层（自动补全/
        // 链接状态栏）独立成层，使 feature-on 可用 SDK chrome 替换 chrome 主层、保留页面内容。
        let mut page_fills = Vec::new();
        let mut page_glyphs = Vec::new();
        let mut chrome_overlay_fills = Vec::new();
        let mut chrome_overlay_glyphs = Vec::new();
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

        // 11. 页面内容（含滚动偏移）—— DC-14 分离点：路由到 page_fills/page_glyphs。
        self.render_page_content(&mut page_fills, &mut page_glyphs, width, content_x, content_y, font_size, s);

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

        // 13. 自动补全下拉 —— DC-14 分离点：路由到 chrome_overlay_*（覆盖于页面之上的 chrome 浮层）。
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut chrome_overlay_fills, &mut chrome_overlay_glyphs, width, font_size, s);
        }

        // 14. 链接悬停浮动状态栏（覆盖在页面内容上方，不占布局高度）
        self.render_floating_link_status(&mut chrome_overlay_fills, &mut chrome_overlay_glyphs, width, height, s);

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

        ChromeScene {
            chrome_fills: fills,
            chrome_glyphs: glyphs,
            page_fills,
            page_glyphs,
            chrome_overlay_fills,
            chrome_overlay_glyphs,
            overlay_fills,
            overlay_glyphs,
            chrome_shadows,
            overlay_rounded_rects,
        }
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
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return RenderPrimitives::new(),
        };

        let layout = self.page_scroll_layout_for(tab_id, self.physical_size.0, self.physical_size.1);
        let scroll = self.tab_scroll_state(tab_id);

        let primitives = match self.tabs.last_render(tab_id).map(|render| &render.primitives) {
            Some(p) => p,
            None => return RenderPrimitives::new(),
        };

        let s = self.scale_factor;
        let clip_viewport = ViewportClip::new(layout.viewport_x, layout.viewport_y, layout.viewport_w, layout.viewport_h);

        let y_offset = layout.viewport_y - scroll.y;
        let x_offset = layout.viewport_x - scroll.x;

        let mut transformed = transform_webview_primitives(primitives, x_offset, y_offset, s, Some(clip_viewport));

        // fills 和 glyphs 已通过 append_webview_primitives 混入 chrome 层，此处清空避免重复
        transformed.fills.clear();
        transformed.glyphs.clear();

        let _ = (layout.viewport_w,);
        transformed
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

    /// 渲染地址栏
    fn render_address_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        _y: f32,
        font_size: f32,
        s: f32,
    ) {
        let (bar_x, bar_y, bar_w, bar_h) = self.address_bar_layout();
        // 聚焦态边框加粗到 2px，强化焦点感知（参考 Chrome）。
        // 窗口失焦时不加粗，避免在非激活窗口留下误导性焦点高亮。
        let border = if self.address_bar_focused && self.window_focused {
            (2.0 * s).max(1.0)
        } else {
            s.max(1.0)
        };
        let radius = bar_h * 0.5;

        let bg = if self.address_bar_focused {
            self.chrome_palette.address_bar_bg_focused
        } else {
            self.chrome_palette.address_bar_bg
        };
        // 窗口失焦时地址栏边框用弱化的 inactive 色（即使地址栏本身处于聚焦态，
        // 只要窗口整体失焦就应弱化焦点感知）。
        let border_color = if !self.window_focused {
            self.chrome_palette.address_bar_border_inactive
        } else if self.address_bar_focused {
            self.chrome_palette.address_bar_border_focused
        } else {
            self.chrome_palette.address_bar_border
        };
        push_rounded_rect_fill(fills, bar_x, bar_y, bar_w, bar_h, radius, border_color);
        push_rounded_rect_fill(
            fills,
            bar_x + border,
            bar_y + border,
            bar_w - 2.0 * border,
            bar_h - 2.0 * border,
            (radius - border).max(0.0),
            bg,
        );

        let text = self.address_bar.text();
        let show_placeholder = text.is_empty() && !self.address_bar_focused && self.address_bar_ime_preedit.is_empty();

        if self.font_id.is_some() {
            let inner_x = bar_x + border;
            let inner_y = bar_y + border;
            let inner_w = bar_w - 2.0 * border;
            let inner_h = bar_h - 2.0 * border;
            let text_x = self.address_bar_text_origin_x();
            let text_pad = layout::ADDRESS_BAR_TEXT_V_PAD * s;
            let (text_top, text_ascent) =
                self.ui_text_top_in_box(inner_y + text_pad, inner_h - 2.0 * text_pad, font_size);
            let status_slot_w = layout::ADDRESS_BAR_LEADING_SLOT_WIDTH * s;
            let status_icon_size = 14.0 * s;
            let status_cx = inner_x + layout::ADDRESS_BAR_INNER_PAD_H * s + status_slot_w * 0.5;
            let status_cy = bar_y + bar_h * 0.5;
            let slot_divider = if self.address_bar_focused {
                self.chrome_palette.address_bar_border_focused
            } else {
                self.chrome_palette.separator
            };
            fills.push(rect_fill(
                inner_x + layout::ADDRESS_BAR_INNER_PAD_H * s + status_slot_w,
                inner_y + 6.0 * s,
                s.max(1.0),
                (inner_h - 12.0 * s).max(s.max(1.0)),
                slot_divider,
            ));

            let status_url = self.shell.active_tab().and_then(|tab| tab.url());
            let status_hint = Self::tab_html_hint(status_url);
            let page_kind = Self::address_bar_page_kind(status_url);
            let is_loading = self.shell.active_tab().is_some_and(|t| t.is_loading());
            if is_loading && !self.address_bar_focused {
                let angle = self.chrome_anim_start.elapsed().as_secs_f32() * 3.5;
                crate::tab_chrome::push_loading_spinner(
                    fills,
                    status_cx,
                    status_cy,
                    status_icon_size,
                    angle,
                    self.chrome_palette.loading_indicator,
                );
            } else if !self.address_bar_focused {
                let status_label = match page_kind {
                    AddressBarPageKind::Secure => None,
                    AddressBarPageKind::Insecure => Some(("!", self.chrome_palette.address_bar_insecure)),
                    AddressBarPageKind::Internal | AddressBarPageKind::Local => {
                        Some(("i", self.chrome_palette.address_bar_internal))
                    }
                    _ => None,
                };
                if page_kind == AddressBarPageKind::Secure {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Lock,
                        status_cx,
                        status_cy,
                        status_icon_size,
                        self.chrome_palette.address_bar_secure,
                    );
                } else if let Some((label, color)) = status_label {
                    let label_w = self.measure_ui_text_width(label, status_icon_size * 0.85);
                    self.draw_ui_text(
                        label,
                        status_cx - label_w * 0.5,
                        text_top,
                        status_icon_size * 0.85,
                        color,
                        glyphs,
                    );
                }
                // favicon 绘制策略：
                // - Secure：已画 Lock，不画 favicon。
                // - Insecure：已画警告标记，且不安全页不应让 favicon 抢占安全语义 → 不画 favicon。
                // - Internal/Local：已画 i 标记，不再叠加 favicon（标签上已显示 favicon）。
                // - Unknown（非 http/https/file/zero）：画 favicon 作为站点识别。
                if matches!(page_kind, AddressBarPageKind::Unknown) {
                    if let Some(tab_id) = self.shell.active_tab_id() {
                        crate::tab_favicon::render_tab_favicon(
                            &mut self.font_loader,
                            glyphs,
                            tab_id,
                            status_url,
                            status_hint,
                            status_cx,
                            status_cy,
                            status_icon_size,
                            self.chrome_palette.page_url,
                        );
                    }
                }
            }

            if self.address_bar_focused && self.address_bar.has_selection() {
                let (sel_start, sel_end) = self.address_bar.selection_char_range();
                let before = Self::chars_slice(text, 0, sel_start);
                let selected = Self::chars_slice(text, sel_start, sel_end);
                let sel_x = text_x + self.measure_ui_text_width(before, font_size);
                let sel_w = self.measure_ui_text_width(selected, font_size).max(1.0);
                let (_, descent) = self.ui_line_metrics(font_size);
                let selection_h = text_ascent - descent;
                fills.push(rect_fill(
                    sel_x,
                    text_top,
                    sel_w,
                    selection_h,
                    self.chrome_palette.address_bar_selection_bg,
                ));
            }

            let color = if show_placeholder {
                self.chrome_palette.address_bar_placeholder
            } else {
                self.chrome_palette.address_bar_text
            };
            let visible = if show_placeholder {
                if self.address_bar_focused && Self::looks_like_search_query(text) {
                    let engine = match self.shell.settings().search_engine {
                        zero_browser_shell::SearchEngine::Google => "Google",
                        zero_browser_shell::SearchEngine::Bing => "Bing",
                        zero_browser_shell::SearchEngine::DuckDuckGo => "DuckDuckGo",
                        zero_browser_shell::SearchEngine::Baidu => "Baidu",
                    };
                    format!("Search {engine} or enter URL...")
                } else {
                    "Search or enter URL...".to_string()
                }
            } else {
                text.to_string()
            };
            let available_text_w = (inner_x + inner_w
                - layout::ADDRESS_BAR_TRAILING_PAD * s
                - layout::ADDRESS_BAR_TRAILING_SLOTS * s
                - text_x)
                .max(0.0);
            let visible = self.truncate_ui_text(&visible, available_text_w, font_size);
            self.draw_ui_text(&visible, text_x, text_top, font_size, color, glyphs);

            if !self.address_bar_ime_preedit.is_empty() {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let preedit_x = text_x + self.measure_ui_text_width(before, font_size);
                self.draw_ui_text(
                    &self.address_bar_ime_preedit,
                    preedit_x,
                    text_top,
                    font_size,
                    self.chrome_palette.address_bar_text,
                    glyphs,
                );
            }

            if self.address_bar_focused && !self.address_bar.has_selection() && self.address_bar_ime_preedit.is_empty()
            {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let cursor_x = text_x + self.measure_ui_text_width(before, font_size);
                fills.push(rect_fill(
                    cursor_x,
                    text_top + text_ascent * 0.12,
                    1.5 * s,
                    text_ascent * 0.76,
                    self.chrome_palette.address_bar_text,
                ));
            }

            let trailing_slots_w = layout::ADDRESS_BAR_TRAILING_SLOTS * s;
            let slots_x = inner_x + inner_w - layout::ADDRESS_BAR_TRAILING_PAD * s - trailing_slots_w;
            let slot_w = layout::ADDRESS_BAR_ACTION_SLOT_WIDTH * s;
            let current_bookmarked = self.shell.is_current_page_bookmarked();
            let star_icon = if current_bookmarked {
                crate::ui_icons::Icon::StarFilled
            } else {
                crate::ui_icons::Icon::Star
            };
            let star_color = if current_bookmarked {
                self.chrome_palette.tab_attention
            } else {
                self.chrome_palette.nav_button
            };
            let slot_specs = [
                (star_icon, star_color),
                (crate::ui_icons::Icon::Shield, self.chrome_palette.page_hint),
            ];
            for (index, (icon, base_color)) in slot_specs.iter().enumerate() {
                let slot_x = slots_x + index as f32 * slot_w;
                let slot_cx = slot_x + slot_w * 0.5;
                let slot_cy = bar_y + bar_h * 0.5;
                let slot_hovered = self.pointer_in_rect(slot_x, inner_y, slot_w, inner_h);
                if slot_hovered {
                    push_circle_fill(
                        fills,
                        slot_cx,
                        slot_cy,
                        layout::NAV_BUTTON_HOVER_DIAMETER * s,
                        self.chrome_palette.tab_hover_bg,
                    );
                }
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    *icon,
                    slot_cx,
                    slot_cy,
                    layout::CHROME_ICON_SIZE * s,
                    if slot_hovered {
                        self.chrome_palette.address_bar_text
                    } else {
                        *base_color
                    },
                );
            }
        }

        if self.font_id.is_some() {
            let (dl_x, dl_y, dl_w, dl_h) = self.toolbar_download_button_rect();
            let dl_cx = dl_x + dl_w * 0.5;
            let dl_cy = dl_y + dl_h * 0.5;
            let dl_hovered = self.pointer_in_rect(dl_x, dl_y, dl_w, dl_h);
            let dl_pressed = dl_hovered && self.left_button_down;
            if dl_pressed {
                push_circle_fill(
                    fills,
                    dl_cx,
                    dl_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.nav_button_pressed,
                );
            } else if dl_hovered {
                push_circle_fill(
                    fills,
                    dl_cx,
                    dl_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                crate::ui_icons::Icon::Download,
                dl_cx,
                dl_cy,
                layout::CHROME_ICON_SIZE * s,
                if dl_hovered {
                    self.chrome_palette.address_bar_text
                } else {
                    self.chrome_palette.nav_button
                },
            );
            if self.shell.downloads().active_count() > 0 {
                push_circle_fill(
                    fills,
                    dl_x + dl_w - 6.0 * s,
                    dl_y + 6.0 * s,
                    4.0 * s,
                    self.chrome_palette.tab_attention,
                );
            }

            let (theme_x, theme_y, theme_w, theme_h) = self.toolbar_theme_button_rect();
            let theme_cx = theme_x + theme_w * 0.5;
            let theme_cy = theme_y + theme_h * 0.5;
            let theme_hovered = self.pointer_in_rect(theme_x, theme_y, theme_w, theme_h);
            let theme_pressed = theme_hovered && self.left_button_down;
            if theme_pressed {
                push_circle_fill(
                    fills,
                    theme_cx,
                    theme_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.nav_button_pressed,
                );
            } else if theme_hovered {
                push_circle_fill(
                    fills,
                    theme_cx,
                    theme_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let theme_icon = self.theme_button_icon();
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                theme_icon,
                theme_cx,
                theme_cy,
                layout::CHROME_ICON_SIZE * s,
                if theme_hovered {
                    self.chrome_palette.address_bar_text
                } else {
                    self.chrome_palette.nav_button
                },
            );
        }

        let (menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h) = self.toolbar_menu_button_rect();
        let menu_btn_cx = menu_btn_x + menu_btn_w * 0.5;
        let menu_btn_cy = menu_btn_y + menu_btn_h * 0.5;
        let menu_hovered = self.pointer_in_rect(menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h);
        let menu_pressed = menu_hovered && self.left_button_down;
        if menu_pressed {
            push_circle_fill(
                fills,
                menu_btn_cx,
                menu_btn_cy,
                layout::NAV_BUTTON_HOVER_DIAMETER * s,
                self.chrome_palette.nav_button_pressed,
            );
        } else if menu_hovered {
            push_circle_fill(
                fills,
                menu_btn_cx,
                menu_btn_cy,
                layout::NAV_BUTTON_HOVER_DIAMETER * s,
                self.chrome_palette.tab_hover_bg,
            );
        }
        crate::ui_icons::render_icon(
            &mut self.font_loader,
            glyphs,
            crate::ui_icons::Icon::MoreVertical,
            menu_btn_cx,
            menu_btn_cy,
            layout::CHROME_ICON_SIZE * s,
            if menu_hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.nav_button
            },
        );
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

        if has_composite_paint
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

        let page_primitives = primitives.clone();

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
            &page_primitives,
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

}
