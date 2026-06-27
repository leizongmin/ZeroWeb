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
    /// 返回 `(fills, glyphs, overlay_fills, overlay_glyphs)`：
    /// `overlay_fills` 和 `overlay_glyphs` 在所有 fills/glyphs 之后绘制，
    /// 用于确保上下文菜单等浮层不被其他内容覆盖。
    fn build_scene(
        &mut self,
        width: u32,
        height: u32,
    ) -> (Vec<FillPrimitive>, Vec<GlyphDraw>, Vec<FillPrimitive>, Vec<GlyphDraw>) {
        let s = self.scale_factor;
        let mut fills = Vec::new();
        let mut glyphs = Vec::new();
        let mut overlay_fills = Vec::new();
        let mut overlay_glyphs = Vec::new();
        let font_size = layout::CHROME_FONT_SIZE * s;

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

        // 4. 地址栏背景（与激活标签同色，形成一体工具栏）
        let addr_y = tab_strip_h;
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT * s,
            self.chrome_palette.toolbar_bg,
        ));
        fills.push(rect_fill(0.0, addr_y, width as f32, s, self.chrome_palette.separator));

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
            self.render_bookmarks_bar(&mut fills, &mut glyphs, width, toolbar_h, s);
        }

        // 9. 页面内容区域（圆角边框视口）
        let chrome_top = self.chrome_top_y_for(s);
        let frame_bottom_y = self.page_frame_bottom_y_for(width, height);
        let page_gutter_h = frame_bottom_y - chrome_top;
        fills.push(rect_fill(
            0.0,
            chrome_top,
            width as f32,
            page_gutter_h,
            self.chrome_palette.tab_active_bg,
        ));
        let (frame_x, frame_y, frame_w, frame_h) = self.page_frame_rect_for(width, height);
        self.render_page_frame(&mut fills, frame_x, frame_y, frame_w, frame_h, s);
        let (content_x, content_y, content_w, _) = self.page_content_rect_for(width, height);

        // 10. 加载指示器
        if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
            fills.push(rect_fill(
                content_x,
                content_y,
                content_w,
                2.0 * s,
                self.chrome_palette.loading_indicator,
            ));
        }

        // 11. 页面内容（含滚动偏移）
        self.render_page_content(&mut fills, &mut glyphs, width, content_x, content_y, font_size, s);

        // 11b. 页面滚动条（overlay，始终显示于溢出时）
        self.render_page_scrollbars(&mut overlay_fills, width, height);

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

        // 18–19. 圆角遮罩与视口边框（无圆角时跳过）
        self.render_page_frame_corner_masks(&mut overlay_fills, width, height, s);
        self.render_page_frame_border(&mut overlay_fills, frame_x, frame_y, frame_w, frame_h, s);
        // 19. Wayland 非最大化：自绘窗口外框（无系统装饰时与桌面区分）
        self.render_custom_window_frame_border(&mut overlay_fills, width, height, s);

        (fills, glyphs, overlay_fills, overlay_glyphs)
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
    fn render_page_scrollbars(&self, overlay_fills: &mut Vec<FillPrimitive>, width: u32, height: u32) {
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
                ideal.clamp(layout::TAB_MIN_WIDTH_COMPRESSED * s, layout::TAB_MIN_WIDTH * s)
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
            if tab.is_muted() {
                label = format!("静音 · {label}");
            }
            tabs.push(TabPaint {
                id: tab.id(),
                x,
                tab_w,
                tab_body_w: tab_w - s,
                bg,
                is_active,
                is_loading: tab.is_loading(),
                is_pinned: tab.is_pinned(),
                is_muted: tab.is_muted(),
                is_crashed: tab.is_crashed(),
                needs_attention: tab.needs_attention(),
                label,
                page_url: tab.url().map(str::to_string),
                html_hint: Self::tab_html_hint(tab.url()),
            });
            x += tab_w;
        }

        for tab in tabs.iter().filter(|t| !t.is_active) {
            crate::tab_chrome::push_inactive_tab_fill(fills, tab.x, tab_y, tab.tab_body_w, tab_bar_h, s, tab.bg);
        }

        if let Some(active) = tabs.iter().find(|t| t.is_active) {
            crate::tab_chrome::push_active_tab_fill(fills, active.x, tab_y, active.tab_body_w, tab_bar_h, s, active.bg);
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
        let thickness = (1.0 * s).max(1.0);

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
                    let size = 8.0 * s;
                    let off = 3.0 * s;
                    let back_left = cx - size / 2.0 - off / 2.0;
                    let back_top = cy - size / 2.0 - off / 2.0;
                    let front_left = cx - size / 2.0 + off / 2.0;
                    let front_top = cy - size / 2.0 + off / 2.0;
                    draw_hollow_square(fills, back_left, back_top, size, thickness, icon);
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
            crate::ui_icons::Icon::Refresh,
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
        let border = s.max(1.0);
        let radius = bar_h * 0.5;

        let bg = if self.address_bar_focused {
            self.chrome_palette.address_bar_bg_focused
        } else {
            self.chrome_palette.address_bar_bg
        };
        let border_color = if self.address_bar_focused {
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
                if page_kind != AddressBarPageKind::Secure {
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
            let slot_specs = [
                (crate::ui_icons::Icon::Star, self.chrome_palette.nav_button),
                (crate::ui_icons::Icon::Shield, self.chrome_palette.page_hint),
                (crate::ui_icons::Icon::MoreVertical, self.chrome_palette.nav_button),
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
            if dl_hovered {
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
        }

        let (menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h) = self.toolbar_menu_button_rect();
        let menu_btn_cx = menu_btn_x + menu_btn_w * 0.5;
        let menu_btn_cy = menu_btn_y + menu_btn_h * 0.5;
        let menu_hovered = self.pointer_in_rect(menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h);
        if menu_hovered {
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
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                crate::ui_icons::Icon::Star,
                icon_cx,
                icon_cy,
                icon_size,
                self.chrome_palette.bookmarks_bar_icon,
            );
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

    /// 渲染页面视口背景（外圈边框色圆角 + 内圈页面底色）
    fn render_page_frame(&self, fills: &mut Vec<FillPrimitive>, x: f32, y: f32, w: f32, h: f32, s: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        if layout::PAGE_FRAME_RADIUS <= 0.0 && layout::PAGE_FRAME_BORDER <= 0.0 {
            fills.push(rect_fill(x, y, w, h, self.chrome_palette.page_bg));
            return;
        }
        let border = layout::PAGE_FRAME_BORDER * s;
        let radius = layout::PAGE_FRAME_RADIUS * s;
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
        if layout::PAGE_FRAME_RADIUS <= 0.0 {
            return;
        }
        let (fx, fy, fw, fh) = self.page_frame_rect_for(width, height);
        let outer_r = layout::PAGE_FRAME_RADIUS * s;
        push_rounded_rect_outside_corner_masks(fills, fx, fy, fw, fh, outer_r, self.chrome_palette.tab_active_bg);

        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let border = layout::PAGE_FRAME_BORDER * s;
        let inner_r = (layout::PAGE_FRAME_RADIUS * s - border).max(0.0);
        push_rounded_rect_outside_corner_masks(fills, cx, cy, cw, ch, inner_r, self.chrome_palette.separator);
    }

    /// 渲染页面视口灰色描边（在内容之上绘制，避免圆角处被内容污染）
    fn render_page_frame_border(&self, fills: &mut Vec<FillPrimitive>, x: f32, y: f32, w: f32, h: f32, s: f32) {
        if w <= 0.0 || h <= 0.0 || layout::PAGE_FRAME_RADIUS <= 0.0 || layout::PAGE_FRAME_BORDER <= 0.0 {
            return;
        }
        let border = layout::PAGE_FRAME_BORDER * s;
        let radius = layout::PAGE_FRAME_RADIUS * s;
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
        let color = self.chrome_palette.separator;

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
        let border = layout::PAGE_FRAME_BORDER * s;
        let radius = (layout::PAGE_FRAME_RADIUS * s - border).max(0.0);
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
        if find_state.total_matches() > 0 {
            let match_text = format!("{}/{}", find_state.current_match(), find_state.total_matches());
            let match_x = bar_x + bar_w - 130.0 * s;
            self.draw_ui_text(
                &match_text,
                match_x,
                y + (bar_h - font_size) * 0.5,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0 * s;
            self.draw_ui_text(
                "No matches",
                no_match_x,
                y + (bar_h - font_size) * 0.5,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
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
                push_circle_fill(
                    fills,
                    icon_cx,
                    btn_cy,
                    icon_size + 8.0 * s,
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

            let source_label = match sug.source() {
                SuggestionSource::Bookmark => "★",
                SuggestionSource::History => "◷",
            };
            let source_size = font_size * 0.82;
            let text_x = bar_x + pad_h;
            self.draw_ui_text(
                source_label,
                text_x,
                row_y + pad_v,
                source_size,
                if sug.source() == SuggestionSource::Bookmark {
                    self.chrome_palette.autocomplete_bookmark
                } else {
                    self.chrome_palette.autocomplete_url
                },
                glyphs,
            );

            let title = sug.title();
            let title_x = text_x + 22.0 * s;
            let title_area_w = bar_w - pad_h * 2.0 - 22.0 * s;
            let truncated_title = self.truncate_ui_text(title, title_area_w, title_size);
            self.draw_ui_text(
                &truncated_title,
                title_x,
                row_y + pad_v,
                title_size,
                self.chrome_palette.autocomplete_text,
                glyphs,
            );

            let url = sug.url();
            let truncated_url = self.truncate_ui_text(url, title_area_w, url_size);
            self.draw_ui_text(
                &truncated_url,
                title_x,
                row_y + pad_v + title_size + 2.0 * s,
                url_size,
                self.chrome_palette.autocomplete_url,
                glyphs,
            );
        }
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        if self.font_id.is_none() {
            return;
        }

        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;
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

        for (i, label) in self.context_menu.items.iter().enumerate() {
            let row_y = menu_y + i as f32 * row_h;
            let is_hovered = self.context_menu.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(
                    menu_x + border,
                    row_y,
                    menu_w - 2.0 * border,
                    row_h,
                    self.chrome_palette.context_menu_hover_bg,
                ));
            }

            if label == "---" {
                let sep_y = row_y + row_h / 2.0;
                fills.push(rect_fill(
                    menu_x + pad_h,
                    sep_y,
                    menu_w - 2.0 * pad_h,
                    border,
                    self.chrome_palette.context_menu_separator,
                ));
                continue;
            }

            self.draw_ui_text(
                label,
                menu_x + pad_h,
                row_y + (row_h - font_size) * 0.5,
                font_size,
                self.chrome_palette.context_menu_text,
                glyphs,
            );
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

    /// fontdue 行 metrics；无字体时回退为 `(font_size, 0)`。
    fn ui_line_metrics(&self, font_size: f32) -> (f32, f32) {
        let Some(primary) = self.font_id else {
            return (font_size, 0.0);
        };
        self.font_loader
            .line_metrics(primary, font_size)
            .unwrap_or((font_size, 0.0))
    }

    /// 在给定高度内垂直居中 UI 文本，返回 `(text_top, baseline_y)`。
    fn ui_text_centered_in_height(&self, height: f32, font_size: f32) -> (f32, f32) {
        let (text_top, ascent) = self.ui_text_top_in_box(0.0, height, font_size);
        (text_top, text_top + ascent)
    }

    /// 在给定矩形高度内垂直居中 UI 文本，返回 `(text_top, ascent)`。
    fn ui_text_top_in_box(&self, box_y: f32, box_h: f32, font_size: f32) -> (f32, f32) {
        let (ascent, descent) = self.ui_line_metrics(font_size);
        let line_h = ascent - descent;
        let text_top = box_y + (box_h - line_h) / 2.0;
        (text_top, ascent)
    }

    /// 绘制 UI 文本（使用字体回退链和真实 advance 宽度）
    fn draw_ui_text(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        font_size: f32,
        color: Color,
        glyphs: &mut Vec<GlyphDraw>,
    ) {
        let Some(primary) = self.font_id else {
            return;
        };
        let (ascent, _) = self.ui_line_metrics(font_size);
        let baseline_y = start_y + ascent;
        let mut x = start_x;
        for ch in text.chars() {
            let font_id = self
                .font_loader
                .rasterize_glyph_with_fallback(primary, ch, font_size)
                .map(|(id, _)| id)
                .unwrap_or(primary);
            glyphs.push(GlyphDraw {
                ch,
                x,
                baseline_y,
                color,
                font_id,
                font_size,
            });
            x += self.font_loader.measure_advance(primary, ch, font_size);
        }
    }

    /// 测量 UI 文本总宽度
    fn measure_ui_text_width(&self, text: &str, font_size: f32) -> f32 {
        let Some(primary) = self.font_id else {
            return 0.0;
        };
        text.chars()
            .map(|ch| self.font_loader.measure_advance(primary, ch, font_size))
            .sum()
    }

    /// 按像素宽度截断 UI 文本
    fn truncate_ui_text(&self, text: &str, max_width: f32, font_size: f32) -> String {
        let Some(primary) = self.font_id else {
            return text.to_string();
        };
        let mut result = String::new();
        let mut width = 0.0;
        let ellipsis_advance = self.font_loader.measure_advance(primary, '…', font_size);
        for ch in text.chars() {
            let advance = self.font_loader.measure_advance(primary, ch, font_size);
            if width + advance + ellipsis_advance > max_width && !result.is_empty() {
                result.push('…');
                break;
            }
            result.push(ch);
            width += advance;
        }
        result
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

/// 从渲染图元估算文档高度（CSS 逻辑像素，fills + glyphs 下界）。
pub fn primitives_content_height(primitives: &RenderPrimitives) -> f32 {
    crate::page_scroll::primitives_content_height(primitives)
}

/// 将 WebView 输出的基础图元追加到浏览器场景。
///
/// `clip_y` 为物理像素坐标 `(top, bottom)`，fill 与该区间求交后绘制，glyph 完全落在区间外则跳过。
/// `clip_rounded` 为 `(x, y, w, h, radius)`，将内容裁剪到圆角矩形内。
#[allow(clippy::too_many_arguments)]
pub fn append_webview_primitives(
    primitives: &RenderPrimitives,
    fills: &mut Vec<FillPrimitive>,
    glyphs: &mut Vec<GlyphDraw>,
    x_offset: f32,
    y_offset: f32,
    fallback_font_id: u32,
    s: f32,
    clip_y: Option<(f32, f32)>,
    clip_rounded: Option<(f32, f32, f32, f32, f32)>,
) -> bool {
    let fill_start = fills.len();
    let glyph_start = glyphs.len();

    for fill in &primitives.fills {
        let x = fill.rect.origin.x * s + x_offset;
        let mut y = fill.rect.origin.y * s + y_offset;
        let w = fill.rect.size.width * s;
        let mut h = fill.rect.size.height * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let bottom = y + h;
            if bottom <= clip_top || y >= clip_bottom {
                continue;
            }
            if y < clip_top {
                h -= clip_top - y;
                y = clip_top;
            }
            let bottom = y + h;
            if bottom > clip_bottom {
                h -= bottom - clip_bottom;
            }
            if h <= 0.0 {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            push_fill_clipped_to_rounded_rect(fills, x, y, w, h, fill.color, rx, ry, rw, rh, radius);
        } else {
            let mut translated = fill.clone();
            translated.rect.origin.x = x;
            translated.rect.origin.y = y;
            translated.rect.size.width = w;
            translated.rect.size.height = h;
            fills.push(translated);
        }
    }

    for glyph in &primitives.glyphs {
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        let x = glyph.x * s + x_offset;
        let baseline_y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            if bottom <= clip_top || top >= clip_bottom || top < clip_top {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            let width = font_size * 0.6;
            if !axis_rect_intersects_rounded_rect(x, top, width, bottom - top, rx, ry, rw, rh, radius) {
                continue;
            }
        }
        glyphs.push(GlyphDraw {
            ch,
            x,
            baseline_y,
            color: glyph.color,
            font_id: if glyph.font_id.0 == 0 {
                fallback_font_id
            } else {
                glyph.font_id.0
            },
            font_size,
        });
    }

    fills.len() > fill_start || glyphs.len() > glyph_start
}

/// 页面视口裁剪区（物理像素）。
#[derive(Debug, Clone, Copy)]
pub struct ViewportClip {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl ViewportClip {
    /// 由 `(x, y, w, h)` 构造视口裁剪矩形。
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + w,
            bottom: y + h,
        }
    }

    fn excludes(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w <= self.left || x >= self.right || y + h <= self.top || y >= self.bottom
    }
}

fn clip_axis_aligned_rect(x: f32, y: f32, w: f32, h: f32, clip: ViewportClip) -> Option<(f32, f32, f32, f32)> {
    if clip.excludes(x, y, w, h) {
        return None;
    }
    let left = x.max(clip.left);
    let top = y.max(clip.top);
    let right = (x + w).min(clip.right);
    let bottom = (y + h).min(clip.bottom);
    let w = right - left;
    let h = bottom - top;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some((left, top, w, h))
}

fn clamp_rounded_rect_radii(rr: &mut zero_render_foundation::primitive::RoundedRectPrimitive) {
    let max_r = rr.rect.size.width.min(rr.rect.size.height) * 0.5;
    rr.top_left_radius = rr.top_left_radius.min(max_r);
    rr.top_right_radius = rr.top_right_radius.min(max_r);
    rr.bottom_right_radius = rr.bottom_right_radius.min(max_r);
    rr.bottom_left_radius = rr.bottom_left_radius.min(max_r);
}

fn clip_rect_field(rect: &mut Rect, clip: ViewportClip) -> bool {
    let Some((x, y, w, h)) =
        clip_axis_aligned_rect(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height, clip)
    else {
        return false;
    };
    rect.origin.x = x;
    rect.origin.y = y;
    rect.size.width = w;
    rect.size.height = h;
    true
}

fn path_vertices_bbox(vertices: &[f32]) -> Option<(f32, f32, f32, f32)> {
    if vertices.len() < 2 {
        return None;
    }
    let mut min_x = vertices[0];
    let mut max_x = vertices[0];
    let mut min_y = vertices[1];
    let mut max_y = vertices[1];
    for chunk in vertices.chunks(2).skip(1) {
        if chunk.len() < 2 {
            continue;
        }
        min_x = min_x.min(chunk[0]);
        max_x = max_x.max(chunk[0]);
        min_y = min_y.min(chunk[1]);
        max_y = max_y.max(chunk[1]);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

/// 将 WebView 的所有 13 种图元类型转换为浏览器坐标（应用 scale、offset、clip）。
///
/// 返回的 `RenderPrimitives` 中的图元坐标为物理像素，
/// 已经应用了 `scale_factor`、`offset` 和视口裁剪。
pub fn transform_webview_primitives(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_viewport: Option<ViewportClip>,
) -> RenderPrimitives {
    let mut out = RenderPrimitives::new();

    // 1. 阴影
    for shadow in &primitives.shadows {
        let mut s_clone = shadow.clone();
        s_clone.rect.origin.x = s_clone.rect.origin.x * s + x_offset;
        s_clone.rect.origin.y = s_clone.rect.origin.y * s + y_offset;
        s_clone.rect.size.width *= s;
        s_clone.rect.size.height *= s;
        s_clone.offset_x *= s;
        s_clone.offset_y *= s;
        s_clone.blur_radius *= s;
        s_clone.spread_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut s_clone.rect, clip)
        {
            continue;
        }
        out.shadows.push(s_clone);
    }

    // 2. 填充矩形
    for fill in &primitives.fills {
        let x = fill.rect.origin.x * s + x_offset;
        let y = fill.rect.origin.y * s + y_offset;
        let w = fill.rect.size.width * s;
        let h = fill.rect.size.height * s;
        let Some((x, y, w, h)) = clip_viewport
            .and_then(|clip| clip_axis_aligned_rect(x, y, w, h, clip))
            .or_else(|| {
                if clip_viewport.is_some() {
                    None
                } else {
                    Some((x, y, w, h))
                }
            })
        else {
            continue;
        };
        out.fills.push(FillPrimitive {
            rect: Rect::new(x, y, w, h),
            color: fill.color,
        });
    }

    // 3. 圆角矩形
    for rr in &primitives.rounded_rects {
        let mut r_clone = rr.clone();
        r_clone.rect.origin.x = r_clone.rect.origin.x * s + x_offset;
        r_clone.rect.origin.y = r_clone.rect.origin.y * s + y_offset;
        r_clone.rect.size.width *= s;
        r_clone.rect.size.height *= s;
        r_clone.top_left_radius *= s;
        r_clone.top_right_radius *= s;
        r_clone.bottom_right_radius *= s;
        r_clone.bottom_left_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut r_clone.rect, clip)
        {
            continue;
        }
        clamp_rounded_rect_radii(&mut r_clone);
        out.rounded_rects.push(r_clone);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        let mut g_clone = gradient.clone();
        g_clone.rect.origin.x = g_clone.rect.origin.x * s + x_offset;
        g_clone.rect.origin.y = g_clone.rect.origin.y * s + y_offset;
        g_clone.rect.size.width *= s;
        g_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut g_clone.rect, clip)
        {
            continue;
        }
        g_clone.kind = match g_clone.kind {
            GradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear {
                x0: x0 * s + x_offset,
                y0: y0 * s + y_offset,
                x1: x1 * s + x_offset,
                y1: y1 * s + y_offset,
            },
            GradientKind::Radial {
                cx,
                cy,
                inner_radius,
                outer_radius,
            } => GradientKind::Radial {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                inner_radius: inner_radius * s,
                outer_radius: outer_radius * s,
            },
            GradientKind::Conic { cx, cy, start_angle } => GradientKind::Conic {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                start_angle,
            },
        };
        out.gradients.push(g_clone);
    }

    // 5. 图片（裁剪须用 `clip` 字段，不可缩小 `rect`，否则会拉伸纹理）
    for image in &primitives.images {
        let x = image.rect.origin.x * s + x_offset;
        let y = image.rect.origin.y * s + y_offset;
        let w = image.rect.size.width * s;
        let h = image.rect.size.height * s;
        let full_rect = Rect::new(x, y, w, h);

        if let Some(clip) = clip_viewport
            && clip.excludes(
                full_rect.origin.x,
                full_rect.origin.y,
                full_rect.size.width,
                full_rect.size.height,
            )
        {
            continue;
        }

        let mut i_clone = image.clone();
        i_clone.rect = full_rect;
        if let Some(clip) = &image.clip {
            i_clone.clip = Some(Rect::new(
                clip.origin.x * s + x_offset,
                clip.origin.y * s + y_offset,
                clip.size.width * s,
                clip.size.height * s,
            ));
        } else {
            i_clone.clip = None;
        }

        if let Some(clip) = clip_viewport {
            let window = Rect::new(clip.left, clip.top, clip.right - clip.left, clip.bottom - clip.top);
            i_clone.clip = match i_clone.clip {
                Some(existing) => existing.intersection(&window),
                None => Some(window),
            };
            if i_clone.clip.is_none() {
                continue;
            }
        }

        out.images.push(i_clone);
    }

    // 6. 线段
    for stroke in &primitives.strokes {
        let mut st = stroke.clone();
        st.x1 = st.x1 * s + x_offset;
        st.y1 = st.y1 * s + y_offset;
        st.x2 = st.x2 * s + x_offset;
        st.y2 = st.y2 * s + y_offset;
        st.width *= s;
        if let Some(clip) = clip_viewport {
            let pad = st.width * 0.5;
            let min_x = st.x1.min(st.x2) - pad;
            let min_y = st.y1.min(st.y2) - pad;
            let max_x = st.x1.max(st.x2) + pad;
            let max_y = st.y1.max(st.y2) + pad;
            if clip.excludes(min_x, min_y, max_x - min_x, max_y - min_y) {
                continue;
            }
        }
        out.strokes.push(st);
    }

    // 7. 路径填充
    for pf in &primitives.path_fills {
        let mut p_clone = pf.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
            && clip.excludes(x, y, w, h)
        {
            continue;
        }
        out.path_fills.push(p_clone);
    }

    // 8. 路径描边
    for ps in &primitives.path_strokes {
        let mut p_clone = ps.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        p_clone.line_width *= s;
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
        {
            let pad = p_clone.line_width * 0.5;
            if clip.excludes(x - pad, y - pad, w + pad * 2.0, h + pad * 2.0) {
                continue;
            }
        }
        out.path_strokes.push(p_clone);
    }

    // 9. 文字
    for glyph in &primitives.glyphs {
        let x = glyph.x * s + x_offset;
        let y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some(clip) = clip_viewport {
            let top = y - font_size;
            let bottom = y + font_size * 0.25;
            let width = font_size * 0.6;
            if clip.excludes(x, top, width, bottom - top) {
                continue;
            }
        }
        out.glyphs.push(GlyphPrimitive {
            x,
            y,
            font_size,
            color: glyph.color,
            glyph_id: glyph.glyph_id,
            font_id: glyph.font_id,
            bitmap_width: glyph.bitmap_width,
            bitmap_height: glyph.bitmap_height,
            rotation: glyph.rotation,
        });
    }

    // 10. 裁剪
    for clip in &primitives.clips {
        let mut c_clone = clip.clone();
        c_clone.rect.origin.x = c_clone.rect.origin.x * s + x_offset;
        c_clone.rect.origin.y = c_clone.rect.origin.y * s + y_offset;
        c_clone.rect.size.width *= s;
        c_clone.rect.size.height *= s;
        if let Some(viewport) = clip_viewport
            && !clip_rect_field(&mut c_clone.rect, viewport)
        {
            continue;
        }
        out.clips.push(c_clone);
    }

    // 11. 变换
    for transform in &primitives.transforms {
        let mut t_clone = transform.clone();
        t_clone.rect.origin.x = t_clone.rect.origin.x * s + x_offset;
        t_clone.rect.origin.y = t_clone.rect.origin.y * s + y_offset;
        t_clone.rect.size.width *= s;
        t_clone.rect.size.height *= s;
        t_clone.origin_x = t_clone.origin_x * s + x_offset;
        t_clone.origin_y = t_clone.origin_y * s + y_offset;
        t_clone.tx *= s;
        t_clone.ty *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut t_clone.rect, clip)
        {
            continue;
        }
        out.transforms.push(t_clone);
    }

    // 12. 滤镜
    for filter in &primitives.filters {
        let mut f_clone = filter.clone();
        f_clone.rect.origin.x = f_clone.rect.origin.x * s + x_offset;
        f_clone.rect.origin.y = f_clone.rect.origin.y * s + y_offset;
        f_clone.rect.size.width *= s;
        f_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut f_clone.rect, clip)
        {
            continue;
        }
        out.filters.push(f_clone);
    }

    // 13. 混合模式
    for blend in &primitives.blend_modes {
        let mut b_clone = blend.clone();
        b_clone.rect.origin.x = b_clone.rect.origin.x * s + x_offset;
        b_clone.rect.origin.y = b_clone.rect.origin.y * s + y_offset;
        b_clone.rect.size.width *= s;
        b_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut b_clone.rect, clip)
        {
            continue;
        }
        out.blend_modes.push(b_clone);
    }

    out
}
