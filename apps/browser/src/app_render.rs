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
        fills.push(rect_fill(0.0, 0.0, width as f32, height as f32, self.chrome_palette.background));

        // 2. 标签栏背景（macOS 左侧为系统 traffic lights 留白）
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let leading = self.tab_bar_leading_inset() * s;
        if leading > 0.0 {
            fills.push(rect_fill(
                leading,
                0.0,
                width as f32 - leading,
                tab_strip_h,
                self.chrome_palette.tab_bar_bg,
            ));
        } else {
            fills.push(rect_fill(0.0, 0.0, width as f32, tab_strip_h, self.chrome_palette.tab_bar_bg));
        }

        // 3. 标签内容（带布局缓存）
        self.render_tabs(&mut fills, &mut glyphs, width, font_size, s);

        // 4. 地址栏背景（与激活标签同色，形成一体工具栏）
        let addr_y = tab_strip_h;
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT * s,
            self.chrome_palette.tab_active_bg,
        ));

        // 5. 导航按钮
        self.render_nav_buttons(&mut fills, &mut glyphs, addr_y, font_size, s);

        // 6. 地址栏
        self.render_address_bar(&mut fills, &mut glyphs, width, addr_y, font_size, s);

        // 7. 分隔线
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        fills.push(rect_fill(0.0, toolbar_h - s, width as f32, s, self.chrome_palette.separator));

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

        // 12. 查找栏（覆盖在页面内容上方）
        if self.shell.find_state().is_active() {
            self.render_find_bar(&mut fills, &mut glyphs, width, content_y, font_size, s);
        }

        // 13. 自动补全下拉
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut fills, &mut glyphs, width, font_size, s);
        }

        // 14. 下载进度条（有活跃下载时显示在状态栏上方）
        if self.shell.downloads().active_count() > 0 {
            self.render_download_bar(&mut fills, &mut glyphs, width, height, font_size, s);
        }

        // 15. 链接悬停浮动状态栏（覆盖在页面内容上方，不占布局高度）
        self.render_floating_link_status(&mut fills, &mut glyphs, width, height, s);

        // 16. 上下文菜单（overlay 图层，始终在最顶层）
        if self.context_menu.visible {
            self.render_context_menu(&mut overlay_fills, &mut overlay_glyphs, s);
        }

        // 17–18. 圆角遮罩与视口边框（overlay：在 WebView glyphs 之后绘制）
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

        let primitives = match self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.last_render())
            .map(|render| &render.primitives)
        {
            Some(p) => p,
            None => return RenderPrimitives::new(),
        };

        let s = self.scale_factor;
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let chrome_top = self.chrome_top_y_for(s);
        let scroll_y = self.scroll_offset.get(&tab_id).copied().unwrap_or(0.0);
        let y_offset = chrome_top - scroll_y;

        let find_offset = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };
        let clip_top = content_y + find_offset;
        let (_, fy, _, fh) = self.page_frame_rect();
        let clip_bottom = (content_y + content_h).min(fy + fh);

        let mut transformed = transform_webview_primitives(
            primitives,
            content_x,
            y_offset,
            s,
            Some((clip_top, clip_bottom)),
        );

        // fills 和 glyphs 已通过 append_webview_primitives 混入 chrome 层，此处清空避免重复
        transformed.fills.clear();
        transformed.glyphs.clear();

        let _ = (content_w,); // 避免未使用变量警告
        transformed
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
        let tab_w = (tabs_max_width / tab_count as f32).clamp(layout::TAB_MIN_WIDTH * s, layout::TAB_MAX_WIDTH * s);

        self.tab_layout.clear();
        let mut x = leading;
        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let icon_size = layout::TAB_ICON_SIZE * s;
        let spinner_angle = self.chrome_anim_start.elapsed().as_secs_f32() * 3.5;
        let text_left_inset = 10.0 * s + icon_size + 6.0 * s;
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
            label: String,
            page_url: Option<String>,
            html_hint: Option<&'static str>,
        }

        let mut tabs: Vec<TabPaint> = Vec::new();
        for tab in self.shell.tabs() {
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
            let label = tab
                .title()
                .or(tab.url())
                .unwrap_or("New Tab")
                .to_string();
            tabs.push(TabPaint {
                id: tab.id(),
                x,
                tab_w,
                tab_body_w: tab_w - s,
                bg,
                is_active,
                is_loading: tab.is_loading(),
                label,
                page_url: tab.url().map(str::to_string),
                html_hint: Self::tab_html_hint(tab.url()),
            });
            x += tab_w;
        }

        for tab in tabs.iter().filter(|t| !t.is_active) {
            crate::tab_chrome::push_inactive_tab_fill(
                fills,
                tab.x,
                tab_y,
                tab.tab_body_w,
                tab_bar_h,
                s,
                tab.bg,
            );
        }

        if let Some(active) = tabs.iter().find(|t| t.is_active) {
            crate::tab_chrome::push_active_tab_fill(
                fills,
                active.x,
                tab_y,
                active.tab_body_w,
                tab_bar_h,
                s,
                active.bg,
            );
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
            let icon_cx = tab.x + 10.0 * s + icon_size * 0.5;
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
                let favicon_url = tab
                    .page_url
                    .as_deref()
                    .or(Some("zero://newtab"));
                crate::tab_favicon::render_tab_favicon(
                    &mut self.font_loader,
                    glyphs,
                    tab.id,
                    favicon_url,
                    tab.html_hint,
                    icon_cx,
                    icon_cy,
                    icon_size,
                    self.chrome_palette.tab_text,
                );
            }

            if self.font_id.is_some() {
                let text_area_w = tab.tab_w - text_left_inset - 28.0 * s;
                let truncated = self.truncate_ui_text(&tab.label, text_area_w.max(0.0), font_size);
                self.draw_ui_text(
                    &truncated,
                    tab.x + text_left_inset,
                    text_y,
                    font_size,
                    self.chrome_palette.tab_text,
                    glyphs,
                );
            }

            let close_x = tab.x + tab.tab_w - 24.0 * s;
            let close_hit = 24.0 * s;
            let close_cx = close_x + close_hit / 2.0;
            let close_cy = tab_y + tab_bar_h / 2.0;
            let close_hovered = self.pointer_in_rect(close_x, tab_y, close_hit, tab_bar_h);
            if close_hovered && tab.is_active {
                push_circle_fill(fills, close_cx, close_cy, close_hit, self.chrome_palette.tab_hover_bg);
            }
            let close_color = if close_hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.tab_close
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
                    24.0 * s,
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
                16.0 * s,
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
                    fills.push(rect_fill(cx - line_w / 2.0, cy - thickness / 2.0, line_w, thickness, icon));
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
        let x = 8.0 * s;
        let btn_w = layout::NAV_BUTTON_WIDTH * s;
        let btn_h = layout::ADDRESS_BAR_HEIGHT * s;
        let cy = y + btn_h / 2.0;

        let nav_icons = [
            crate::ui_icons::Icon::ChevronLeft,
            crate::ui_icons::Icon::ChevronRight,
            crate::ui_icons::Icon::Refresh,
            crate::ui_icons::Icon::Home,
        ];
        let can_back = self
            .shell
            .active_tab()
            .is_some_and(|tab| tab.history_index() > 0);
        let can_forward = self.shell.active_tab().is_some_and(|tab| {
            let history = tab.navigation_history();
            !history.is_empty() && tab.history_index() < history.len() - 1
        });
        let nav_enabled = [can_back, can_forward, true, true];
        let hover_diameter = 28.0 * s;

        for (i, &icon) in nav_icons.iter().enumerate() {
            let bx = x + btn_w * i as f32;
            let cx = bx + btn_w / 2.0;
            let enabled = nav_enabled[i];
            let hovered = enabled && self.pointer_in_rect(bx, y, btn_w, btn_h);
            if hovered {
                push_circle_fill(fills, cx, cy, hover_diameter, self.chrome_palette.tab_hover_bg);
            }
            let color = if !enabled {
                self.chrome_palette.nav_button_disabled
            } else if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.nav_button
            };
            crate::ui_icons::render_icon(&mut self.font_loader, glyphs, icon, cx, cy, 16.0 * s, color);
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
        push_rounded_rect_fill(
            fills,
            bar_x,
            bar_y,
            bar_w,
            bar_h,
            radius,
            self.chrome_palette.separator,
        );
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
        let show_placeholder =
            text.is_empty() && !self.address_bar_focused && self.address_bar_ime_preedit.is_empty();

        if self.font_id.is_some() {
            let inner_x = bar_x + border;
            let inner_y = bar_y + border;
            let inner_h = bar_h - 2.0 * border;
            let text_x = inner_x + 10.0 * s;
            let text_pad = layout::ADDRESS_BAR_TEXT_V_PAD * s;
            let (text_top, text_ascent) =
                self.ui_text_top_in_box(inner_y + text_pad, inner_h - 2.0 * text_pad, font_size);

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
                "Search or enter URL..."
            } else {
                text
            };
            self.draw_ui_text(visible, text_x, text_top, font_size, color, glyphs);

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
        }
    }

    fn byte_at_char(text: &str, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }
        text.char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(text.len())
    }

    fn chars_slice(text: &str, start: usize, end: usize) -> &str {
        let b0 = Self::byte_at_char(text, start);
        let b1 = Self::byte_at_char(text, end);
        &text[b0..b1]
    }

    /// 渲染书签栏
    fn render_bookmarks_bar(
        &self,
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
        fills.push(rect_fill(0.0, y, width as f32, bar_h, self.chrome_palette.bookmarks_bar_bg));

        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let by = y + 3.0 * s;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let icon_w = self.measure_ui_text_width("★", font_size);
            let label_w = self.measure_ui_text_width(label, font_size);
            let item_w = icon_w + 6.0 * s + label_w + 16.0 * s;

            // 悬停效果
            let mx = self.mouse_pos.0 as f32;
            let my = self.mouse_pos.1 as f32;
            if mx >= bx && mx < bx + item_w && my >= y && my < y + bar_h {
                fills.push(rect_fill(bx, y, item_w, bar_h, self.chrome_palette.bookmarks_bar_hover_bg));
            }

            // 书签图标
            self.draw_ui_text("★", bx, by, font_size, self.chrome_palette.bookmarks_bar_icon, glyphs);
            // 标签文本
            self.draw_ui_text(
                label,
                bx + icon_w + 6.0 * s,
                by,
                font_size,
                self.chrome_palette.bookmarks_bar_text,
                glyphs,
            );

            bx += item_w + 8.0 * s;
            if bx > width as f32 - 40.0 * s {
                break;
            }
        }
    }

    /// 渲染页面视口背景（外圈边框色圆角 + 内圈页面底色）
    fn render_page_frame(
        &self,
        fills: &mut Vec<FillPrimitive>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s: f32,
    ) {
        if w <= 0.0 || h <= 0.0 {
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
    fn render_page_frame_corner_masks(
        &self,
        fills: &mut Vec<FillPrimitive>,
        width: u32,
        height: u32,
        s: f32,
    ) {
        let (fx, fy, fw, fh) = self.page_frame_rect_for(width, height);
        let outer_r = layout::PAGE_FRAME_RADIUS * s;
        push_rounded_rect_outside_corner_masks(fills, fx, fy, fw, fh, outer_r, self.chrome_palette.tab_active_bg);

        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        let border = layout::PAGE_FRAME_BORDER * s;
        let inner_r = (layout::PAGE_FRAME_RADIUS * s - border).max(0.0);
        push_rounded_rect_outside_corner_masks(fills, cx, cy, cw, ch, inner_r, self.chrome_palette.separator);
    }

    /// 渲染页面视口灰色描边（在内容之上绘制，避免圆角处被内容污染）
    fn render_page_frame_border(
        &self,
        fills: &mut Vec<FillPrimitive>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s: f32,
    ) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let border = layout::PAGE_FRAME_BORDER * s;
        let radius = layout::PAGE_FRAME_RADIUS * s;
        push_rounded_rect_border(
            fills,
            x,
            y,
            w,
            h,
            radius,
            border,
            self.chrome_palette.separator,
        );
    }

    /// Wayland 无系统装饰时，为非最大化窗口绘制 1px 外框描边。
    fn render_custom_window_frame_border(
        &self,
        fills: &mut Vec<FillPrimitive>,
        width: u32,
        height: u32,
        s: f32,
    ) {
        if !self.uses_custom_window_controls() || self.window_is_maximized {
            return;
        }

        let border = layout::WINDOW_FRAME_BORDER * s;
        let w = width as f32;
        let h = height as f32;
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

        let content_y_offset = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };

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
        let scroll_y = self.scroll_offset.get(&tab_id).copied().unwrap_or(0.0);

        if !is_loading && self.render_active_webview(fills, glyphs, content_x, page_y, fid, scroll_y) {
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
    fn render_active_webview(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        content_x: f32,
        y_offset: f32,
        fallback_font_id: u32,
        scroll_y: f32,
    ) -> bool {
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return false,
        };

        let primitives = match self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.last_render())
            .map(|render| &render.primitives)
        {
            Some(primitives) => primitives,
            None => return false,
        };

        let mut page_primitives = primitives.clone();
        if let Some(primary) = self.font_id {
            reflow_webview_glyphs(&mut page_primitives.glyphs, &self.font_loader, primary);
        }

        let find_offset = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * self.scale_factor
        } else {
            0.0
        };
        let (_, content_y, content_w, content_h) = self.page_content_rect();
        let clip_top = content_y + find_offset;
        let (_, fy, _, fh) = self.page_frame_rect();
        let clip_bottom = (content_y + content_h).min(fy + fh);
        let content_y_draw = y_offset - scroll_y;
        let s = self.scale_factor;
        let border = layout::PAGE_FRAME_BORDER * s;
        let radius = (layout::PAGE_FRAME_RADIUS * s - border).max(0.0);
        let clip_rounded = Some((content_x, content_y, content_w, content_h, radius));

        if let Some(sel) = self.page_selection.get(&tab_id)
            && !sel.is_collapsed()
        {
            let (start, end) = sel.normalized();
            let end = end.min(page_primitives.glyphs.len().saturating_sub(1));
            if start <= end {
                for glyph in &page_primitives.glyphs[start..=end] {
                    let x = glyph.x * s + content_x;
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
            content_x,
            y_offset - scroll_y,
            fallback_font_id,
            self.scale_factor,
            Some((clip_top, clip_bottom)),
            clip_rounded,
        )
    }

    /// 渲染查找栏
    fn render_find_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        content_y: f32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let y = content_y;
        let bar_w = 320.0 * s;
        let bar_x = width as f32 - bar_w - 10.0 * s;

        fills.push(rect_fill(
            bar_x,
            y,
            bar_w,
            layout::FIND_BAR_HEIGHT * s,
            self.chrome_palette.find_bar_bg,
        ));

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
            bar_x + 10.0 * s,
            y + 5.0 * s,
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
                y + 5.0 * s,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0 * s;
            self.draw_ui_text(
                "No matches",
                no_match_x,
                y + 5.0 * s,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
        }

        let btn_y = y + 5.0 * s;
        let btn_size = font_size;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
        let btn_w = 24.0 * s;
        let bar_h = layout::FIND_BAR_HEIGHT * s;
        let icon_size = 16.0 * s;
        let btn_cy = btn_y + btn_size * 0.5;

        for (bx, icon) in [
            (prev_x, crate::ui_icons::Icon::ChevronUp),
            (next_x, crate::ui_icons::Icon::ChevronDown),
            (close_x, crate::ui_icons::Icon::Close),
        ] {
            let icon_cx = bx + btn_w / 2.0;
            let hovered = self.pointer_in_rect(bx, y, btn_w, bar_h);
            if hovered {
                push_circle_fill(fills, icon_cx, btn_cy, icon_size + 8.0 * s, self.chrome_palette.tab_hover_bg);
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
        width: u32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let dropdown_y = layout::TOOLBAR_HEIGHT * s;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let dropdown_h = visible_count as f32 * row_h;

        fills.push(rect_fill(bar_x, dropdown_y, bar_w, dropdown_h, self.chrome_palette.autocomplete_bg));

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * row_h;
            let is_hovered = self.autocomplete.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(bar_x, row_y, bar_w, row_h, self.chrome_palette.autocomplete_hover_bg));
            }

            let source_label = match sug.source() {
                SuggestionSource::Bookmark => "★",
                SuggestionSource::History => "🕐",
            };
            let source_size = font_size * 0.85;
            let text_x = bar_x + 10.0 * s;
            self.draw_ui_text(
                source_label,
                text_x,
                row_y + 5.0 * s,
                source_size,
                if sug.source() == SuggestionSource::Bookmark {
                    self.chrome_palette.autocomplete_bookmark
                } else {
                    self.chrome_palette.autocomplete_url
                },
                glyphs,
            );

            let title = sug.title();
            let title_area_w = bar_w - 180.0 * s;
            let truncated_title = self.truncate_ui_text(title, title_area_w, source_size);
            self.draw_ui_text(
                &truncated_title,
                text_x + 24.0 * s,
                row_y + 5.0 * s,
                source_size,
                self.chrome_palette.autocomplete_text,
                glyphs,
            );

            let url = sug.url();
            let url_size = font_size * 0.75;
            let url_area_w = bar_w * 0.4;
            let truncated_url = self.truncate_ui_text(url, url_area_w, url_size);
            let url_display_width = self.measure_ui_text_width(&truncated_url, url_size);
            let url_x = bar_x + bar_w - 10.0 * s;
            self.draw_ui_text(
                &truncated_url,
                url_x - url_display_width,
                row_y + 5.0 * s,
                url_size,
                self.chrome_palette.autocomplete_url,
                glyphs,
            );
        }

        fills.push(rect_fill(bar_x, dropdown_y + dropdown_h, bar_w, s, self.chrome_palette.separator));
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        if self.font_id.is_none() {
            return;
        }

        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = 28.0 * s;
        let menu_w = 200.0 * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;
        let font_size = 13.0 * s;

        // 菜单背景
        fills.push(rect_fill(menu_x, menu_y, menu_w, menu_h, self.chrome_palette.context_menu_bg));

        // 菜单边框
        let border_w = 1.0 * s;
        fills.push(rect_fill(
            menu_x,
            menu_y,
            menu_w,
            border_w,
            self.chrome_palette.context_menu_separator,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y + menu_h - border_w,
            menu_w,
            border_w,
            self.chrome_palette.context_menu_separator,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y,
            border_w,
            menu_h,
            self.chrome_palette.context_menu_separator,
        ));
        fills.push(rect_fill(
            menu_x + menu_w - border_w,
            menu_y,
            border_w,
            menu_h,
            self.chrome_palette.context_menu_separator,
        ));

        for (i, label) in self.context_menu.items.iter().enumerate() {
            let row_y = menu_y + i as f32 * row_h;
            let is_hovered = self.context_menu.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(
                    menu_x + border_w,
                    row_y,
                    menu_w - 2.0 * border_w,
                    row_h,
                    self.chrome_palette.context_menu_hover_bg,
                ));
            }

            // 分隔线项
            if label == "---" {
                let sep_y = row_y + row_h / 2.0;
                fills.push(rect_fill(
                    menu_x + 12.0 * s,
                    sep_y,
                    menu_w - 24.0 * s,
                    border_w,
                    self.chrome_palette.context_menu_separator,
                ));
                continue;
            }

            self.draw_ui_text(
                label,
                menu_x + 16.0 * s,
                row_y + 6.0 * s,
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

        push_rounded_rect_fill(fills, pill_x, pill_y, pill_w, status_h, radius, self.chrome_palette.separator);
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

    /// 渲染下载进度条（状态栏上方）
    fn render_download_bar(
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

        let bar_h = layout::DOWNLOAD_BAR_HEIGHT * s;
        let frame_bottom_y = self.page_frame_bottom_y_for(width, height);
        let bar_y = frame_bottom_y - bar_h;

        // 背景
        fills.push(rect_fill(0.0, bar_y, width as f32, bar_h, self.chrome_palette.download_bar_bg));

        // 显示第一个活跃下载的信息
        let downloads = self.shell.downloads();
        let active: Vec<_> = downloads.iter().filter(|d| d.is_active()).collect();
        if let Some(dl) = active.first() {
            let font_size = 11.0 * s;

            // 文件名
            let name_text = dl.filename();
            self.draw_ui_text(
                name_text,
                10.0 * s,
                bar_y + 6.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );

            // 进度条
            let progress = dl.progress();
            let bar_width = 120.0 * s;
            let bar_start_x = width as f32 - bar_width - 80.0 * s;
            let bar_top = bar_y + 8.0 * s;
            let bar_inner_h = 6.0 * s;

            // 进度条背景
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width,
                bar_inner_h,
                self.chrome_palette.separator,
            ));
            // 进度条填充
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width * progress,
                bar_inner_h,
                self.chrome_palette.download_bar_fill,
            ));

            // 百分比文字
            let pct_text = format!("{:.0}%", progress * 100.0);
            self.draw_ui_text(
                &pct_text,
                bar_start_x + bar_width + 8.0 * s,
                bar_y + 6.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );
        }
    }
}

// --- 渲染工具函数 ---

/// 创建填充矩形图元
fn rect_fill(x: f32, y: f32, w: f32, h: f32, color: Color) -> FillPrimitive {
    FillPrimitive {
        rect: zero_render_foundation::geometry::Rect::new(x, y, w, h),
        color,
    }
}

/// 圆角矩形在指定行的水平可见区间 `(x_start, x_end)`。
fn rounded_rect_x_span_at_y(yf: f32, x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<(f32, f32)> {
    if yf < y || yf >= y + h {
        return None;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        return Some((x, x + w));
    }

    let r_sq = r * r;
    let mut x_start = x;
    let mut x_end = x + w;

    let dy_top = (y + r) - yf;
    if dy_top > 0.0 {
        let dx = (r_sq - dy_top * dy_top).max(0.0).sqrt();
        x_start = x + r - dx;
        x_end = x + w - r + dx;
    } else {
        let dy_bottom = yf - (y + h - r);
        if dy_bottom > 0.0 {
            let dx = (r_sq - dy_bottom * dy_bottom).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        }
    }

    if x_end <= x_start {
        None
    } else {
        Some((x_start, x_end))
    }
}

/// 将轴对齐矩形裁剪到圆角矩形内，按行写入 fill。
#[allow(clippy::too_many_arguments)]
fn push_fill_clipped_to_rounded_rect(
    fills: &mut Vec<FillPrimitive>,
    fx: f32,
    fy: f32,
    fw: f32,
    fh: f32,
    color: Color,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    radius: f32,
) {
    let ix0 = fx.max(rx);
    let iy0 = fy.max(ry);
    let ix1 = (fx + fw).min(rx + rw);
    let iy1 = (fy + fh).min(ry + rh);
    if ix0 >= ix1 || iy0 >= iy1 {
        return;
    }

    let min_row = iy0.floor() as i32;
    let max_row = iy1.ceil() as i32;
    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < iy0 || yf >= iy1 {
            continue;
        }
        let Some((mut xs, mut xe)) = rounded_rect_x_span_at_y(yf, rx, ry, rw, rh, radius) else {
            continue;
        };
        xs = xs.max(ix0);
        xe = xe.min(ix1);
        if xe > xs {
            fills.push(rect_fill(xs, row as f32, xe - xs, 1.0, color));
        }
    }
}

/// 轴对齐矩形是否与圆角矩形有交集（用于 glyph 裁剪）。
#[allow(clippy::too_many_arguments)]
fn axis_rect_intersects_rounded_rect(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    radius: f32,
) -> bool {
    if ax >= rx + rw || ax + aw <= rx || ay >= ry + rh || ay + ah <= ry {
        return false;
    }
    let sample_y = (ay + ah * 0.5).clamp(ry, ry + rh - f32::EPSILON);
    let Some((xs, xe)) = rounded_rect_x_span_at_y(sample_y, rx, ry, rw, rh, radius) else {
        return false;
    };
    ax + aw > xs && ax < xe
}

/// 将圆角矩形外、轴对齐包围盒内的区域用指定颜色覆盖（清除四角溢出）。
fn push_rounded_rect_outside_corner_masks(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        return;
    }

    let min_row = y.floor() as i32;
    let max_row = (y + h).ceil() as i32;

    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }
        let Some((xs, xe)) = rounded_rect_x_span_at_y(yf, x, y, w, h, r) else {
            continue;
        };
        if xs > x {
            fills.push(rect_fill(x, row as f32, xs - x, 1.0, color));
        }
        if x + w > xe {
            fills.push(rect_fill(xe, row as f32, x + w - xe, 1.0, color));
        }
    }
}

/// 圆角矩形描边（在内容之上绘制，仅输出边框环）。
#[allow(clippy::too_many_arguments)]
fn push_rounded_rect_border(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    border: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 || border <= 0.0 {
        return;
    }
    let outer_r = radius.min(w * 0.5).min(h * 0.5);
    let inner_r = (outer_r - border).max(0.0);
    let min_row = y.floor() as i32;
    let max_row = (y + h).ceil() as i32;

    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }
        let Some((ox0, ox1)) = rounded_rect_x_span_at_y(yf, x, y, w, h, outer_r) else {
            continue;
        };
        if inner_r <= f32::EPSILON {
            fills.push(rect_fill(ox0, row as f32, ox1 - ox0, 1.0, color));
            continue;
        }
        let Some((ix0, ix1)) =
            rounded_rect_x_span_at_y(yf, x + border, y + border, w - 2.0 * border, h - 2.0 * border, inner_r)
        else {
            fills.push(rect_fill(ox0, row as f32, ox1 - ox0, 1.0, color));
            continue;
        };
        if ix0 > ox0 {
            fills.push(rect_fill(ox0, row as f32, ix0 - ox0, 1.0, color));
        }
        if ox1 > ix1 {
            fills.push(rect_fill(ix1, row as f32, ox1 - ix1, 1.0, color));
        }
    }
}

/// 四角圆角矩形（`radius = h/2` 时为胶囊形地址栏）。
fn push_rounded_rect_fill(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        fills.push(rect_fill(x, y, w, h, color));
        return;
    }

    let r_sq = r * r;
    let min_y = y.floor() as i32;
    let max_y = (y + h).ceil() as i32;

    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }

        let mut x_start = x;
        let mut x_end = x + w;

        let dy_top = (y + r) - yf;
        if dy_top > 0.0 {
            let dx = (r_sq - dy_top * dy_top).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        } else {
            let dy_bottom = yf - (y + h - r);
            if dy_bottom > 0.0 {
                let dx = (r_sq - dy_bottom * dy_bottom).max(0.0).sqrt();
                x_start = x + r - dx;
                x_end = x + w - r + dx;
            }
        }

        if x_end > x_start {
            fills.push(rect_fill(x_start, row as f32, x_end - x_start, 1.0, color));
        }
    }
}

fn draw_hollow_square(fills: &mut Vec<FillPrimitive>, x: f32, y: f32, size: f32, thickness: f32, color: Color) {
    fills.push(rect_fill(x, y, size, thickness, color));
    fills.push(rect_fill(x, y + size - thickness, size, thickness, color));
    fills.push(rect_fill(x, y, thickness, size, color));
    fills.push(rect_fill(x + size - thickness, y, thickness, size, color));
}

/// 实心圆盘（用于图标 hover 背景等）
fn push_circle_fill(fills: &mut Vec<FillPrimitive>, cx: f32, cy: f32, diameter: f32, color: Color) {
    let r = diameter * 0.5;
    if r <= 0.0 {
        return;
    }
    let min_y = (cy - r).floor() as i32;
    let max_y = (cy + r).ceil() as i32;
    let r_sq = r * r;

    for y in min_y..=max_y {
        let yf = y as f32 + 0.5;
        let dy = yf - cy;
        let dx_max = (r_sq - dy * dy).max(0.0).sqrt();
        if dx_max <= f32::EPSILON {
            continue;
        }
        fills.push(rect_fill(cx - dx_max, y as f32, dx_max * 2.0, 1.0, color));
    }
}

/// 按真实字体 advance 重新排列 WebView 文本 glyph（与 UI 文本一致）
pub(crate) fn reflow_webview_glyphs(
    glyphs: &mut [zero_render_foundation::primitive::GlyphPrimitive],
    font_loader: &FontLoader,
    primary_id: u32,
) {
    use std::collections::HashMap;

    if glyphs.is_empty() {
        return;
    }

    let mut lines: HashMap<i32, Vec<usize>> = HashMap::new();
    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.glyph_id == 0 {
            continue;
        }
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        let key = (glyph.y * 2.0).round() as i32;
        lines.entry(key).or_default().push(i);
    }

    for indices in lines.into_values() {
        let mut indices = indices;
        indices.sort_by(|&a, &b| {
            glyphs[a]
                .x
                .partial_cmp(&glyphs[b].x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut cursor_x = glyphs[indices[0]].x;
        let mut i = 0;
        while i < indices.len() {
            let cluster_x = glyphs[indices[i]].x;
            let font_size = glyphs[indices[i]].font_size;
            let Some(ch) = char::from_u32(glyphs[indices[i]].glyph_id) else {
                i += 1;
                continue;
            };

            let mut j = i + 1;
            while j < indices.len() && (glyphs[indices[j]].x - cluster_x).abs() < 1.0 {
                j += 1;
            }

            for idx in &indices[i..j] {
                let offset = glyphs[*idx].x - cluster_x;
                glyphs[*idx].x = cursor_x + offset;
            }

            cursor_x += font_loader.measure_advance(primary_id, ch, font_size);
            i = j;
        }
    }
}

/// 从渲染图元估算文档高度（逻辑像素，fills + glyphs 下界）。
pub fn primitives_content_height(primitives: &RenderPrimitives) -> f32 {
    let fill_max = primitives
        .fills
        .iter()
        .map(|f| f.rect.origin.y + f.rect.size.height)
        .fold(0.0f32, f32::max);
    let glyph_max = primitives
        .glyphs
        .iter()
        .map(|g| g.y + g.font_size)
        .fold(0.0f32, f32::max);
    fill_max.max(glyph_max)
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

/// 将 WebView 的所有 13 种图元类型转换为浏览器坐标（应用 scale、offset、clip）。
///
/// 返回的 `RenderPrimitives` 中的图元坐标为物理像素，
/// 已经应用了 `scale_factor`、`offset` 和视口裁剪。
pub fn transform_webview_primitives(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_y: Option<(f32, f32)>,
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
        out.shadows.push(s_clone);
    }

    // 2. 填充矩形
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
        out.rounded_rects.push(r_clone);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        let mut g_clone = gradient.clone();
        g_clone.rect.origin.x = g_clone.rect.origin.x * s + x_offset;
        g_clone.rect.origin.y = g_clone.rect.origin.y * s + y_offset;
        g_clone.rect.size.width *= s;
        g_clone.rect.size.height *= s;
        g_clone.kind = match g_clone.kind {
            GradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear {
                x0: x0 * s + x_offset,
                y0: y0 * s + y_offset,
                x1: x1 * s + x_offset,
                y1: y1 * s + y_offset,
            },
            GradientKind::Radial { cx, cy, inner_radius, outer_radius } => GradientKind::Radial {
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

    // 5. 图片
    for image in &primitives.images {
        let mut i_clone = image.clone();
        i_clone.rect.origin.x = i_clone.rect.origin.x * s + x_offset;
        i_clone.rect.origin.y = i_clone.rect.origin.y * s + y_offset;
        i_clone.rect.size.width *= s;
        i_clone.rect.size.height *= s;
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
        out.path_strokes.push(p_clone);
    }

    // 9. 文字
    for glyph in &primitives.glyphs {
        let x = glyph.x * s + x_offset;
        let y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let top = y - font_size;
            let bottom = y + font_size * 0.25;
            if bottom <= clip_top || top >= clip_bottom {
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
        out.transforms.push(t_clone);
    }

    // 12. 滤镜
    for filter in &primitives.filters {
        let mut f_clone = filter.clone();
        f_clone.rect.origin.x = f_clone.rect.origin.x * s + x_offset;
        f_clone.rect.origin.y = f_clone.rect.origin.y * s + y_offset;
        f_clone.rect.size.width *= s;
        f_clone.rect.size.height *= s;
        out.filters.push(f_clone);
    }

    // 13. 混合模式
    for blend in &primitives.blend_modes {
        let mut b_clone = blend.clone();
        b_clone.rect.origin.x = b_clone.rect.origin.x * s + x_offset;
        b_clone.rect.origin.y = b_clone.rect.origin.y * s + y_offset;
        b_clone.rect.size.width *= s;
        b_clone.rect.size.height *= s;
        out.blend_modes.push(b_clone);
    }

    out
}
