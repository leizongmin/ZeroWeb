// 浏览器 UI 渲染方法（从 app.rs 通过 include! 引入）
//
// 此文件在编译时被 app.rs include，共享同一个模块作用域。

// --- BrowserApp 渲染 impl ---

impl BrowserApp {
    /// 构建浏览器 UI 渲染图元（物理像素坐标）
    fn build_scene(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
        let s = self.scale_factor;
        let mut fills = Vec::new();
        let mut glyphs = Vec::new();
        let font_size = 14.0 * s;

        // 1. 整体背景
        fills.push(rect_fill(0.0, 0.0, width as f32, height as f32, colors::BACKGROUND));

        // 2. 标签栏背景（macOS 左侧为系统 traffic lights 留白）
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let leading = self.tab_bar_leading_inset() * s;
        if leading > 0.0 {
            fills.push(rect_fill(
                leading,
                0.0,
                width as f32 - leading,
                tab_bar_h,
                colors::TAB_BAR_BG,
            ));
        } else {
            fills.push(rect_fill(0.0, 0.0, width as f32, tab_bar_h, colors::TAB_BAR_BG));
        }

        // 3. 标签内容（带布局缓存）
        self.render_tabs(&mut fills, &mut glyphs, width, font_size, s);

        // 4. 地址栏背景
        let addr_y = tab_bar_h;
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT * s,
            colors::TAB_BAR_BG,
        ));

        // 5. 导航按钮
        self.render_nav_buttons(&mut glyphs, addr_y, font_size, s);

        // 6. 地址栏
        self.render_address_bar(&mut fills, &mut glyphs, width, addr_y, font_size, s);

        // 7. 分隔线
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        fills.push(rect_fill(0.0, toolbar_h - s, width as f32, s, colors::SEPARATOR));

        // 8. 书签栏
        let bookmarks_bar_y = toolbar_h;
        self.render_bookmarks_bar(&mut fills, &mut glyphs, width, bookmarks_bar_y, s);

        // 9. 页面内容区域
        let chrome_top = toolbar_h + layout::BOOKMARKS_BAR_HEIGHT * s;
        let page_h = height as f32 - chrome_top - layout::STATUS_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, chrome_top, width as f32, page_h, colors::PAGE_BG));

        // 10. 加载指示器
        if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
            fills.push(rect_fill(
                0.0,
                chrome_top,
                width as f32,
                2.0 * s,
                colors::LOADING_INDICATOR,
            ));
        }

        // 11. 页面内容（含滚动偏移）
        self.render_page_content(&mut fills, &mut glyphs, width, chrome_top, font_size, s);

        // 12. 查找栏（覆盖在页面内容上方）
        if self.shell.find_state().is_active() {
            self.render_find_bar(&mut fills, &mut glyphs, width, chrome_top, font_size, s);
        }

        // 13. 自动补全下拉
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut fills, &mut glyphs, width, font_size, s);
        }

        // 14. 上下文菜单（最上层覆盖）
        if self.context_menu.visible {
            self.render_context_menu(&mut fills, &mut glyphs, s);
        }

        // 15. 下载进度条（有活跃下载时显示在状态栏上方）
        if self.shell.downloads().active_count() > 0 {
            self.render_download_bar(&mut fills, &mut glyphs, width, height, font_size, s);
        }

        // 16. 状态栏
        self.render_status_bar(&mut fills, &mut glyphs, width, height, font_size, s);

        (fills, glyphs)
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

        for tab in self.shell.tabs() {
            let is_active = Some(tab.id()) == active_id;
            let is_hovered = !is_active && {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= x && mx < x + tab_w && my < layout::TAB_BAR_HEIGHT * s
            };

            let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
            let bg = if is_active {
                colors::TAB_ACTIVE_BG
            } else if is_hovered {
                colors::TAB_HOVER_BG
            } else {
                colors::TAB_BAR_BG
            };
            fills.push(rect_fill(x, 0.0, tab_w - s, tab_bar_h, bg));

            if self.font_id.is_some() {
                let label = tab.title().unwrap_or_else(|| tab.url().unwrap_or("New Tab"));
                let text_area_w = tab_w - 40.0 * s;
                let truncated = self.truncate_ui_text(label, text_area_w, font_size);
                self.draw_ui_text(
                    &truncated,
                    x + 10.0 * s,
                    8.0 * s,
                    font_size,
                    colors::TAB_TEXT,
                    glyphs,
                );
            }

            if let Some(fid) = self.font_id {
                let close_x = x + tab_w - 24.0 * s;
                let close_size = font_size * 0.8;
                let close_advance = self.font_loader.measure_advance(fid, '×', close_size);
                glyphs.push(GlyphDraw {
                    ch: '×',
                    x: close_x + (24.0 * s - close_advance) / 2.0,
                    baseline_y: 8.0 * s + font_size,
                    color: colors::TAB_CLOSE,
                    font_id: fid,
                    font_size: close_size,
                });
            }

            self.tab_layout.push((tab.id(), x, tab_w));
            x += tab_w;
        }

        // 新建标签按钮 (+)，紧跟最后一个标签
        if self.font_id.is_some() {
            let btn_x = x;
            let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
            let is_hovered = {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= btn_x && mx < btn_x + new_tab_btn_w && my < tab_bar_h
            };
            if is_hovered {
                fills.push(rect_fill(btn_x, 0.0, new_tab_btn_w, tab_bar_h, colors::TAB_HOVER_BG));
            }
            let plus_advance = self.measure_ui_text_width("+", font_size);
            let text_x = btn_x + (new_tab_btn_w - plus_advance) / 2.0;
            self.draw_ui_text("+", text_x, 8.0 * s, font_size, colors::NEW_TAB_BUTTON, glyphs);
        }

        if self.uses_custom_window_controls() {
            self.render_window_controls(fills, glyphs, width, s);
        }
    }

    /// 渲染窗口控制按钮（最小化 / 最大化 / 关闭）
    fn render_window_controls(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        s: f32,
    ) {
        let btn_w = layout::WINDOW_CONTROL_BTN_WIDTH * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let x0 = self.window_controls_origin_x(width as f32, s);
        let icon = colors::WINDOW_CONTROL_ICON;
        let thickness = (1.0 * s).max(1.0);

        for i in 0..3 {
            let bx = x0 + i as f32 * btn_w;
            let hovered = self.window_control_hover == Some(i);
            let bg = if i == 2 && hovered {
                colors::WINDOW_CONTROL_CLOSE_HOVER
            } else if hovered {
                colors::WINDOW_CONTROL_HOVER
            } else {
                colors::TAB_BAR_BG
            };
            fills.push(rect_fill(bx, 0.0, btn_w, tab_bar_h, bg));

            let cx = bx + btn_w / 2.0;
            let cy = tab_bar_h / 2.0;

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
                    if let Some(fid) = self.font_id {
                        let close_size = 14.0 * s;
                        let advance = self.font_loader.measure_advance(fid, '×', close_size);
                        glyphs.push(GlyphDraw {
                            ch: '×',
                            x: cx - advance / 2.0,
                            baseline_y: cy + close_size * 0.35,
                            color: icon,
                            font_id: fid,
                            font_size: close_size,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// 渲染导航按钮
    fn render_nav_buttons(&mut self, glyphs: &mut Vec<GlyphDraw>, y: f32, font_size: f32, s: f32) {
        let Some(fid) = self.font_id else {
            return;
        };

        let baseline_y = y + (layout::ADDRESS_BAR_HEIGHT * s + font_size) / 2.0;
        let x = 8.0 * s;
        let btn_w = layout::NAV_BUTTON_WIDTH * s;

        for (i, ch) in ['←', '→', '↻', '⌂'].iter().enumerate() {
            let bx = x + btn_w * i as f32;
            let advance = self.font_loader.measure_advance(fid, *ch, font_size);
            glyphs.push(GlyphDraw {
                ch: *ch,
                x: bx + (btn_w - advance) / 2.0,
                baseline_y,
                color: colors::NAV_BUTTON,
                font_id: fid,
                font_size,
            });
        }
    }

    /// 渲染地址栏
    fn render_address_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        y: f32,
        font_size: f32,
        s: f32,
    ) {
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let bar_y = y + 4.0 * s;
        let bar_h = layout::ADDRESS_BAR_HEIGHT * s - 8.0 * s;

        let bg = if self.address_bar_focused {
            colors::ADDRESS_BAR_BG_FOCUSED
        } else {
            colors::ADDRESS_BAR_BG
        };
        fills.push(rect_fill(bar_x, bar_y, bar_w, bar_h, bg));

        let text = self.address_bar.text();
        let show_placeholder =
            text.is_empty() && !self.address_bar_focused && self.address_bar_ime_preedit.is_empty();

        if self.font_id.is_some() {
            let text_x = bar_x + 10.0 * s;
            let text_y = bar_y + 3.0 * s;

            if self.address_bar_focused && self.address_bar.has_selection() {
                let (sel_start, sel_end) = self.address_bar.selection_char_range();
                let before = Self::chars_slice(text, 0, sel_start);
                let selected = Self::chars_slice(text, sel_start, sel_end);
                let sel_x = text_x + self.measure_ui_text_width(before, font_size);
                let sel_w = self.measure_ui_text_width(selected, font_size).max(1.0);
                fills.push(rect_fill(sel_x, bar_y + 2.0 * s, sel_w, bar_h - 4.0 * s, colors::ADDRESS_BAR_SELECTION_BG));
            }

            let color = if show_placeholder {
                colors::ADDRESS_BAR_PLACEHOLDER
            } else {
                colors::ADDRESS_BAR_TEXT
            };
            let visible = if show_placeholder {
                "Search or enter URL..."
            } else {
                text
            };
            self.draw_ui_text(visible, text_x, text_y, font_size, color, glyphs);

            if !self.address_bar_ime_preedit.is_empty() {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let preedit_x = text_x + self.measure_ui_text_width(before, font_size);
                self.draw_ui_text(
                    &self.address_bar_ime_preedit,
                    preedit_x,
                    text_y,
                    font_size,
                    colors::ADDRESS_BAR_TEXT,
                    glyphs,
                );
            }

            if self.address_bar_focused && !self.address_bar.has_selection() && self.address_bar_ime_preedit.is_empty()
            {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let cursor_x = text_x + self.measure_ui_text_width(before, font_size);
                fills.push(rect_fill(
                    cursor_x,
                    bar_y + 4.0 * s,
                    1.5 * s,
                    bar_h - 8.0 * s,
                    colors::ADDRESS_BAR_TEXT,
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
        fills.push(rect_fill(0.0, y, width as f32, bar_h, colors::BOOKMARKS_BAR_BG));

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
                fills.push(rect_fill(bx, y, item_w, bar_h, colors::BOOKMARKS_BAR_HOVER_BG));
            }

            // 书签图标
            self.draw_ui_text("★", bx, by, font_size, colors::BOOKMARKS_BAR_ICON, glyphs);
            // 标签文本
            self.draw_ui_text(
                label,
                bx + icon_w + 6.0 * s,
                by,
                font_size,
                colors::BOOKMARKS_BAR_TEXT,
                glyphs,
            );

            bx += item_w + 8.0 * s;
            if bx > width as f32 - 40.0 * s {
                break;
            }
        }
    }

    /// 渲染页面内容
    fn render_page_content(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
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

        if !is_loading && self.render_active_webview(fills, glyphs, y, fid, scroll_y) {
            return;
        }

        if !title.is_empty() {
            self.draw_ui_text(
                &title,
                20.0 * s,
                y + 20.0 * s,
                24.0 * s,
                colors::PAGE_TITLE,
                glyphs,
            );
            y += 52.0 * s;
        }

        if !url.is_empty() {
            self.draw_ui_text(&url, 20.0 * s, y, 12.0 * s, colors::PAGE_URL, glyphs);
            y += 28.0 * s;
        }

        if is_loading {
            self.draw_ui_text("Loading...", 20.0 * s, y, font_size, colors::PAGE_HINT, glyphs);
        } else if title.is_empty() && url.is_empty() {
            self.draw_ui_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                20.0 * s,
                y,
                font_size,
                colors::PAGE_HINT,
                glyphs,
            );
        }
    }

    /// 渲染活跃 WebView 的页面图元。
    fn render_active_webview(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
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
        let clip_bottom = y_offset - find_offset + self.content_physical_size().1 as f32;
        let content_y = y_offset - scroll_y;
        let s = self.scale_factor;

        if let Some(sel) = self.page_selection.get(&tab_id)
            && !sel.is_collapsed()
        {
            let (start, end) = sel.normalized();
            let end = end.min(page_primitives.glyphs.len().saturating_sub(1));
            if start <= end {
                for glyph in &page_primitives.glyphs[start..=end] {
                    let x = glyph.x * s;
                    let top = glyph.y * s + content_y - glyph.font_size * s;
                    let w = glyph.font_size * s * 0.55;
                    let h = glyph.font_size * s;
                    if top + h <= y_offset || top >= clip_bottom {
                        continue;
                    }
                    fills.push(rect_fill(x, top, w.max(1.0), h, colors::TEXT_SELECTION_BG));
                }
            }
        }

        append_webview_primitives(
            &page_primitives,
            fills,
            glyphs,
            0.0,
            y_offset - scroll_y,
            fallback_font_id,
            self.scale_factor,
            Some((y_offset, clip_bottom)),
        )
    }

    /// 渲染查找栏
    fn render_find_bar(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        chrome_top: f32,
        font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let y = chrome_top;
        let bar_w = 320.0 * s;
        let bar_x = width as f32 - bar_w - 10.0 * s;

        fills.push(rect_fill(
            bar_x,
            y,
            bar_w,
            layout::FIND_BAR_HEIGHT * s,
            colors::FIND_BAR_BG,
        ));

        let display = if self.find_input.is_empty() {
            "Find...".to_string()
        } else {
            self.find_input.clone()
        };
        let text_color = if self.find_input.is_empty() {
            colors::FIND_MATCH_TEXT
        } else {
            colors::FIND_BAR_TEXT
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
                colors::FIND_MATCH_TEXT,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0 * s;
            self.draw_ui_text(
                "No matches",
                no_match_x,
                y + 5.0 * s,
                font_size,
                colors::FIND_MATCH_TEXT,
                glyphs,
            );
        }

        let btn_y = y + 5.0 * s;
        let btn_size = font_size;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
        let btn_w = 24.0 * s;
        for (ch, bx) in [('↑', prev_x), ('↓', next_x), ('×', close_x)] {
            let advance = self.font_loader.measure_advance(fid, ch, btn_size);
            glyphs.push(GlyphDraw {
                ch,
                x: bx + (btn_w - advance) / 2.0,
                baseline_y: btn_y + btn_size,
                color: colors::FIND_BAR_TEXT,
                font_id: fid,
                font_size: btn_size,
            });
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
        let dropdown_y = (layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT) * s;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let dropdown_h = visible_count as f32 * row_h;

        fills.push(rect_fill(bar_x, dropdown_y, bar_w, dropdown_h, colors::AUTOCOMPLETE_BG));

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * row_h;
            let is_hovered = self.autocomplete.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(bar_x, row_y, bar_w, row_h, colors::AUTOCOMPLETE_HOVER_BG));
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
                    colors::AUTOCOMPLETE_BOOKMARK
                } else {
                    colors::AUTOCOMPLETE_URL
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
                colors::AUTOCOMPLETE_TEXT,
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
                colors::AUTOCOMPLETE_URL,
                glyphs,
            );
        }

        fills.push(rect_fill(bar_x, dropdown_y + dropdown_h, bar_w, s, colors::SEPARATOR));
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
        fills.push(rect_fill(menu_x, menu_y, menu_w, menu_h, colors::CONTEXT_MENU_BG));

        // 菜单边框
        let border_w = 1.0 * s;
        fills.push(rect_fill(
            menu_x,
            menu_y,
            menu_w,
            border_w,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y + menu_h - border_w,
            menu_w,
            border_w,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y,
            border_w,
            menu_h,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x + menu_w - border_w,
            menu_y,
            border_w,
            menu_h,
            colors::CONTEXT_MENU_SEPARATOR,
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
                    colors::CONTEXT_MENU_HOVER_BG,
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
                    colors::CONTEXT_MENU_SEPARATOR,
                ));
                continue;
            }

            self.draw_ui_text(
                label,
                menu_x + 16.0 * s,
                row_y + 6.0 * s,
                font_size,
                colors::CONTEXT_MENU_TEXT,
                glyphs,
            );
        }
    }

    /// 渲染状态栏
    fn render_status_bar(
        &mut self,
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

        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let status_y = height as f32 - status_h;

        fills.push(rect_fill(0.0, status_y, width as f32, status_h, colors::BACKGROUND));
        fills.push(rect_fill(0.0, status_y, width as f32, s, colors::SEPARATOR));

        let zoom = self.shell.zoom();
        if (zoom - 1.0).abs() > f32::EPSILON {
            let zoom_text = format!("{}%", (zoom * 100.0) as u32);
            self.draw_ui_text(
                &zoom_text,
                10.0 * s,
                status_y + 3.0 * s,
                11.0 * s,
                colors::STATUS_TEXT,
                glyphs,
            );
        }

        let tab_count = self.shell.tab_count();
        let tabs_text = format!("Tabs: {tab_count}");
        let tabs_width = self.measure_ui_text_width(&tabs_text, 11.0 * s);
        self.draw_ui_text(
            &tabs_text,
            width as f32 - tabs_width - 10.0 * s,
            status_y + 3.0 * s,
            11.0 * s,
            colors::STATUS_TEXT,
            glyphs,
        );
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
                baseline_y: start_y + font_size,
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
        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let bar_y = height as f32 - status_h - bar_h;

        // 背景
        fills.push(rect_fill(0.0, bar_y, width as f32, bar_h, colors::DOWNLOAD_BAR_BG));

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
                colors::DOWNLOAD_BAR_TEXT,
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
                colors::SEPARATOR,
            ));
            // 进度条填充
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width * progress,
                bar_inner_h,
                colors::DOWNLOAD_BAR_FILL,
            ));

            // 百分比文字
            let pct_text = format!("{:.0}%", progress * 100.0);
            self.draw_ui_text(
                &pct_text,
                bar_start_x + bar_width + 8.0 * s,
                bar_y + 6.0 * s,
                font_size,
                colors::DOWNLOAD_BAR_TEXT,
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

fn draw_hollow_square(fills: &mut Vec<FillPrimitive>, x: f32, y: f32, size: f32, thickness: f32, color: Color) {
    fills.push(rect_fill(x, y, size, thickness, color));
    fills.push(rect_fill(x, y + size - thickness, size, thickness, color));
    fills.push(rect_fill(x, y, thickness, size, color));
    fills.push(rect_fill(x + size - thickness, y, thickness, size, color));
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
        let mut translated = fill.clone();
        translated.rect.origin.x = x;
        translated.rect.origin.y = y;
        translated.rect.size.width = w;
        translated.rect.size.height = h;
        fills.push(translated);
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
