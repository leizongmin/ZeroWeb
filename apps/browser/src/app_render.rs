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

        // 2. 标签栏背景
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, 0.0, width as f32, tab_bar_h, colors::TAB_BAR_BG));

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

        let new_tab_btn_w = 32.0 * s;
        let available_width = width as f32 - new_tab_btn_w;
        let tab_w = (available_width / tab_count as f32).clamp(layout::TAB_MIN_WIDTH * s, layout::TAB_MAX_WIDTH * s);

        self.tab_layout.clear();
        let mut x = 0.0_f32;

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

            if let Some(fid) = self.font_id {
                let label = tab.title().unwrap_or_else(|| tab.url().unwrap_or("New Tab"));
                let max_chars = ((tab_w - 40.0 * s) / (font_size * 0.6)).max(3.0) as usize;
                let truncated: String = label.chars().take(max_chars).collect();
                draw_text(
                    &truncated,
                    x + 10.0 * s,
                    8.0 * s,
                    font_size,
                    colors::TAB_TEXT,
                    fid,
                    glyphs,
                );
            }

            if let Some(fid) = self.font_id {
                let close_x = x + tab_w - 24.0 * s;
                glyphs.push(GlyphDraw {
                    ch: '×',
                    x: close_x,
                    baseline_y: 8.0 * s + font_size,
                    color: colors::TAB_CLOSE,
                    font_id: fid,
                    font_size: font_size * 0.8,
                });
            }

            self.tab_layout.push((tab.id(), x, tab_w));
            x += tab_w;
        }

        // 新建标签按钮 (+)
        if let Some(fid) = self.font_id {
            let btn_x = width as f32 - new_tab_btn_w;
            let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
            let is_hovered = {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= btn_x && my < tab_bar_h
            };
            if is_hovered {
                fills.push(rect_fill(btn_x, 0.0, new_tab_btn_w, tab_bar_h, colors::TAB_HOVER_BG));
            }
            let text_x = btn_x + (new_tab_btn_w - font_size * 0.6) / 2.0;
            draw_text("+", text_x, 8.0 * s, font_size, colors::NEW_TAB_BUTTON, fid, glyphs);
        }
    }

    /// 渲染导航按钮
    fn render_nav_buttons(&mut self, glyphs: &mut Vec<GlyphDraw>, y: f32, font_size: f32, s: f32) {
        if let Some(fid) = self.font_id {
            let baseline_y = y + (layout::ADDRESS_BAR_HEIGHT * s + font_size) / 2.0;
            let x = 8.0 * s;
            let w = layout::NAV_BUTTON_WIDTH * s;

            for (i, ch) in ['←', '→', '↻', '⌂'].iter().enumerate() {
                glyphs.push(GlyphDraw {
                    ch: *ch,
                    x: x + w * i as f32,
                    baseline_y,
                    color: colors::NAV_BUTTON,
                    font_id: fid,
                    font_size,
                });
            }
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

        let display_text = if self.address_bar_text.is_empty() && !self.address_bar_focused {
            "Search or enter URL...".to_string()
        } else {
            self.address_bar_text.clone()
        };

        if let Some(fid) = self.font_id {
            let color = if self.address_bar_focused {
                colors::ADDRESS_BAR_TEXT
            } else if self.address_bar_text.is_empty() {
                colors::ADDRESS_BAR_PLACEHOLDER
            } else {
                colors::ADDRESS_BAR_TEXT
            };
            draw_text(
                &display_text,
                bar_x + 10.0 * s,
                bar_y + 3.0 * s,
                font_size,
                color,
                fid,
                glyphs,
            );

            if self.address_bar_focused {
                let cursor_x = bar_x + 10.0 * s + self.address_bar_text.len() as f32 * font_size * 0.6;
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

    /// 渲染书签栏
    fn render_bookmarks_bar(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        y: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let bar_h = layout::BOOKMARKS_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, y, width as f32, bar_h, colors::BOOKMARKS_BAR_BG));

        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let by = y + 3.0 * s;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;

            // 悬停效果
            let mx = self.mouse_pos.0 as f32;
            let my = self.mouse_pos.1 as f32;
            if mx >= bx && mx < bx + item_w && my >= y && my < y + bar_h {
                fills.push(rect_fill(bx, y, item_w, bar_h, colors::BOOKMARKS_BAR_HOVER_BG));
            }

            // 书签图标
            draw_text("★", bx, by, font_size, colors::BOOKMARKS_BAR_ICON, fid, glyphs);
            // 标签文本
            draw_text(
                label,
                bx + 14.0 * s,
                by,
                font_size,
                colors::BOOKMARKS_BAR_TEXT,
                fid,
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
            draw_text(
                &title,
                20.0 * s,
                y + 20.0 * s,
                24.0 * s,
                colors::PAGE_TITLE,
                fid,
                glyphs,
            );
            y += 52.0 * s;
        }

        if !url.is_empty() {
            draw_text(&url, 20.0 * s, y, 12.0 * s, colors::PAGE_URL, fid, glyphs);
            y += 28.0 * s;
        }

        if is_loading {
            draw_text("Loading...", 20.0 * s, y, font_size, colors::PAGE_HINT, fid, glyphs);
        } else if title.is_empty() && url.is_empty() {
            draw_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                20.0 * s,
                y,
                font_size,
                colors::PAGE_HINT,
                fid,
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

        append_webview_primitives(
            primitives,
            fills,
            glyphs,
            0.0,
            y_offset - scroll_y,
            fallback_font_id,
            1.0,
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
        draw_text(
            &display,
            bar_x + 10.0 * s,
            y + 5.0 * s,
            font_size,
            text_color,
            fid,
            glyphs,
        );

        let find_state = self.shell.find_state();
        if find_state.total_matches() > 0 {
            let match_text = format!("{}/{}", find_state.current_match(), find_state.total_matches());
            let match_x = bar_x + bar_w - 130.0 * s;
            draw_text(
                &match_text,
                match_x,
                y + 5.0 * s,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0 * s;
            draw_text(
                "No matches",
                no_match_x,
                y + 5.0 * s,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        }

        let btn_y = y + 5.0 * s;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
        draw_text("↑", prev_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
        draw_text("↓", next_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
        draw_text("×", close_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
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
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

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
            let text_x = bar_x + 10.0 * s;
            draw_text(
                source_label,
                text_x,
                row_y + 5.0 * s,
                font_size * 0.85,
                if sug.source() == SuggestionSource::Bookmark {
                    colors::AUTOCOMPLETE_BOOKMARK
                } else {
                    colors::AUTOCOMPLETE_URL
                },
                fid,
                glyphs,
            );

            let title = sug.title();
            let max_title_chars = ((bar_w - 180.0 * s) / (font_size * 0.6)).max(10.0) as usize;
            let truncated_title: String = title.chars().take(max_title_chars).collect();
            draw_text(
                &truncated_title,
                text_x + 24.0 * s,
                row_y + 5.0 * s,
                font_size * 0.85,
                colors::AUTOCOMPLETE_TEXT,
                fid,
                glyphs,
            );

            let url = sug.url();
            let url_x = bar_x + bar_w - 10.0 * s;
            let max_url_chars = ((bar_w * 0.4) / (font_size * 0.5)).max(8.0) as usize;
            let truncated_url: String = url.chars().take(max_url_chars).collect();
            let url_display_width = truncated_url.len() as f32 * font_size * 0.5;
            draw_text(
                &truncated_url,
                url_x - url_display_width,
                row_y + 5.0 * s,
                font_size * 0.75,
                colors::AUTOCOMPLETE_URL,
                fid,
                glyphs,
            );
        }

        fills.push(rect_fill(bar_x, dropdown_y + dropdown_h, bar_w, s, colors::SEPARATOR));
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

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

            draw_text(
                label,
                menu_x + 16.0 * s,
                row_y + 6.0 * s,
                font_size,
                colors::CONTEXT_MENU_TEXT,
                fid,
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
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let status_y = height as f32 - status_h;

        fills.push(rect_fill(0.0, status_y, width as f32, status_h, colors::BACKGROUND));
        fills.push(rect_fill(0.0, status_y, width as f32, s, colors::SEPARATOR));

        let zoom = self.shell.zoom();
        if (zoom - 1.0).abs() > f32::EPSILON {
            let zoom_text = format!("{}%", (zoom * 100.0) as u32);
            draw_text(
                &zoom_text,
                10.0 * s,
                status_y + 3.0 * s,
                11.0 * s,
                colors::STATUS_TEXT,
                fid,
                glyphs,
            );
        }

        let tab_count = self.shell.tab_count();
        let tabs_text = format!("Tabs: {tab_count}");
        let tabs_width = tabs_text.len() as f32 * 11.0 * s * 0.6;
        draw_text(
            &tabs_text,
            width as f32 - tabs_width - 10.0 * s,
            status_y + 3.0 * s,
            11.0 * s,
            colors::STATUS_TEXT,
            fid,
            glyphs,
        );
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
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

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
            draw_text(
                name_text,
                10.0 * s,
                bar_y + 6.0 * s,
                font_size,
                colors::DOWNLOAD_BAR_TEXT,
                fid,
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
            draw_text(
                &pct_text,
                bar_start_x + bar_width + 8.0 * s,
                bar_y + 6.0 * s,
                font_size,
                colors::DOWNLOAD_BAR_TEXT,
                fid,
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

/// 绘制文本（估算字符宽度）
fn draw_text(
    text: &str,
    start_x: f32,
    start_y: f32,
    font_size: f32,
    color: Color,
    font_id: u32,
    glyphs: &mut Vec<GlyphDraw>,
) {
    let mut x = start_x;
    for ch in text.chars() {
        glyphs.push(GlyphDraw {
            ch,
            x,
            baseline_y: start_y + font_size,
            color,
            font_id,
            font_size,
        });
        x += if ch.is_ascii() { font_size * 0.6 } else { font_size };
    }
}

/// 将 WebView 输出的基础图元追加到浏览器场景。
pub fn append_webview_primitives(
    primitives: &RenderPrimitives,
    fills: &mut Vec<FillPrimitive>,
    glyphs: &mut Vec<GlyphDraw>,
    x_offset: f32,
    y_offset: f32,
    fallback_font_id: u32,
    s: f32,
) -> bool {
    let fill_start = fills.len();
    let glyph_start = glyphs.len();

    for fill in &primitives.fills {
        let mut translated = fill.clone();
        translated.rect.origin.x = fill.rect.origin.x * s + x_offset;
        translated.rect.origin.y = fill.rect.origin.y * s + y_offset;
        translated.rect.size.width *= s;
        translated.rect.size.height *= s;
        fills.push(translated);
    }

    for glyph in &primitives.glyphs {
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        glyphs.push(GlyphDraw {
            ch,
            x: glyph.x * s + x_offset,
            baseline_y: glyph.y * s + y_offset,
            color: glyph.color,
            font_id: if glyph.font_id.0 == 0 {
                fallback_font_id
            } else {
                glyph.font_id.0
            },
            font_size: glyph.font_size * s,
        });
    }

    fills.len() > fill_start || glyphs.len() > glyph_start
}
