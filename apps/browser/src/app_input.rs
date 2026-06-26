// 输入处理方法（键盘、鼠标、IME、自动补全、上下文菜单）
// 从 app.rs 拆分以控制 app.rs 体积

impl BrowserApp {
    /// 处理鼠标滚轮滚动
    pub fn handle_scroll(
        &mut self,
        delta: zero_host_runtime::event::MouseScrollDelta,
        at_x: f64,
        at_y: f64,
    ) {
        // 上下文菜单打开时不滚动
        if self.context_menu.visible {
            return;
        }

        self.mouse_pos = (at_x, at_y);

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        // 仅在 WebView 内容区响应滚轮
        if !self.point_in_page_content(at_x, at_y) {
            return;
        }

        // 提取滚动量（滚轮向下增大 scroll offset，与 Linux/winit 符号相反故取反）
        let (delta_x, delta_y) = match delta {
            zero_host_runtime::event::MouseScrollDelta::PixelDelta(x, y) => (-(x as f32), -(y as f32)),
            zero_host_runtime::event::MouseScrollDelta::LineDelta(x, y) => (-(x * 40.0), -(y * 40.0)),
        };

        self.apply_page_scroll_delta(tab_id, delta_x, delta_y);
    }

    /// 处理触摸板/触摸屏平移手势（winit `PanGesture`）
    pub fn handle_pan_gesture(&mut self, delta_x: f32, delta_y: f32, x: f64, y: f64) {
        if self.context_menu.visible {
            return;
        }

        self.mouse_pos = (x, y);
        if !self.point_in_page_content(x, y) {
            return;
        }

        let Some(tab_id) = self.shell.active_tab_id() else {
            return;
        };

        // 与 PixelDelta 滚轮保持同一符号约定
        self.apply_page_scroll_delta(tab_id, -delta_x, -delta_y);
    }

    /// 处理触摸屏单指拖拽滚动
    pub fn handle_touch(&mut self, touch: &zero_host_runtime::event::TouchEvent) {
        use zero_host_runtime::event::TouchPhase;

        self.mouse_pos = (touch.x, touch.y);

        if self.context_menu.visible {
            return;
        }

        match touch.phase {
            TouchPhase::Started => {
                if self.point_in_page_content(touch.x, touch.y) {
                    self.touch_scroll = Some((touch.id, touch.y));
                }
            }
            TouchPhase::Moved => {
                let Some((id, last_y)) = self.touch_scroll else {
                    return;
                };
                if id != touch.id {
                    return;
                }
                let delta_y = (last_y - touch.y) as f32;
                if delta_y != 0.0 && let Some(tab_id) = self.shell.active_tab_id() {
                    self.apply_page_scroll_delta(tab_id, 0.0, delta_y);
                }
                if let Some(state) = &mut self.touch_scroll {
                    state.1 = touch.y;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if self.touch_scroll.is_some_and(|(id, _)| id == touch.id) {
                    self.touch_scroll = None;
                }
            }
        }
    }

    /// 触摸/滚轮坐标是否落在 WebView 内容区（物理像素）
    fn point_in_page_content(&self, x: f64, y: f64) -> bool {
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let xf = x as f32;
        let yf = y as f32;
        xf >= content_x && xf < content_x + content_w && yf >= content_y && yf < content_y + content_h
    }

    /// 按物理像素增量更新当前标签页滚动偏移
    fn apply_page_scroll_delta(
        &mut self,
        tab_id: zero_browser_shell::TabId,
        delta_x: f32,
        delta_y: f32,
    ) {
        if delta_x == 0.0 && delta_y == 0.0 {
            return;
        }

        self.ensure_webview(tab_id);

        let layout = self.page_scroll_layout(tab_id);
        let entry = self.scroll.entry(tab_id).or_default();
        entry.x = (entry.x + delta_x).clamp(0.0, layout.max_scroll_x);
        entry.y = (entry.y + delta_y).clamp(0.0, layout.max_scroll_y);

        self.needs_redraw = true;
    }

    fn scrollbar_hit_at(&self, x: f32, y: f32) -> Option<(zero_browser_shell::TabId, page_scroll::ScrollbarHit)> {
        let tab_id = self.shell.active_tab_id()?;
        let (cx, cy, cw, ch) = self.page_content_rect();
        let layout = self.page_scroll_layout(tab_id);
        if !layout.show_vertical && !layout.show_horizontal {
            return None;
        }
        let scroll = self.tab_scroll_state(tab_id);
        let geometry =
            page_scroll::scrollbar_geometry(&layout, scroll, cx, cy, cw, ch, self.scale_factor);
        page_scroll::hit_test_scrollbar(x, y, &geometry)
            .map(|hit| (tab_id, hit))
    }

    fn start_scrollbar_interaction(
        &mut self,
        tab_id: zero_browser_shell::TabId,
        hit: page_scroll::ScrollbarHit,
        x: f32,
        y: f32,
    ) {
        let (cx, cy, cw, ch) = self.page_content_rect();
        let layout = self.page_scroll_layout(tab_id);
        let scroll = self.tab_scroll_state(tab_id);
        let geometry = page_scroll::scrollbar_geometry(
            &layout,
            scroll,
            cx,
            cy,
            cw,
            ch,
            self.scale_factor,
        );

        match hit {
            page_scroll::ScrollbarHit::VerticalThumb => {
                let (_, thumb_y, _, _) = geometry.vertical_thumb.expect("vertical thumb");
                let grab_offset = y - thumb_y;
                let new_y = page_scroll::scroll_y_from_pointer(
                    &layout,
                    cy,
                    ch,
                    self.scale_factor,
                    y,
                    grab_offset,
                );
                self.scroll.entry(tab_id).or_default().y = new_y;
                self.scrollbar_drag = Some(ScrollbarDrag {
                    tab_id,
                    axis: page_scroll::ScrollbarAxis::Vertical,
                    grab_offset,
                });
            }
            page_scroll::ScrollbarHit::VerticalTrack => {
                let thumb_h = page_scroll::vertical_thumb_len(
                    &layout,
                    page_scroll::vertical_track_len(&layout, ch),
                    self.scale_factor,
                );
                let grab_offset = thumb_h * 0.5;
                let new_y = page_scroll::scroll_y_from_pointer(
                    &layout,
                    cy,
                    ch,
                    self.scale_factor,
                    y,
                    grab_offset,
                );
                self.scroll.entry(tab_id).or_default().y = new_y;
                self.scrollbar_drag = Some(ScrollbarDrag {
                    tab_id,
                    axis: page_scroll::ScrollbarAxis::Vertical,
                    grab_offset,
                });
            }
            page_scroll::ScrollbarHit::HorizontalThumb => {
                let (thumb_x, _, _, _) = geometry.horizontal_thumb.expect("horizontal thumb");
                let grab_offset = x - thumb_x;
                let new_x = page_scroll::scroll_x_from_pointer(
                    &layout,
                    cx,
                    cw,
                    self.scale_factor,
                    x,
                    grab_offset,
                );
                self.scroll.entry(tab_id).or_default().x = new_x;
                self.scrollbar_drag = Some(ScrollbarDrag {
                    tab_id,
                    axis: page_scroll::ScrollbarAxis::Horizontal,
                    grab_offset,
                });
            }
            page_scroll::ScrollbarHit::HorizontalTrack => {
                let thumb_w = page_scroll::horizontal_thumb_len(
                    &layout,
                    page_scroll::horizontal_track_len(&layout, cw),
                    self.scale_factor,
                );
                let grab_offset = thumb_w * 0.5;
                let new_x = page_scroll::scroll_x_from_pointer(
                    &layout,
                    cx,
                    cw,
                    self.scale_factor,
                    x,
                    grab_offset,
                );
                self.scroll.entry(tab_id).or_default().x = new_x;
                self.scrollbar_drag = Some(ScrollbarDrag {
                    tab_id,
                    axis: page_scroll::ScrollbarAxis::Horizontal,
                    grab_offset,
                });
            }
        }

        self.page_selection_drag = false;
        self.content_pointer_drag = None;
        self.needs_redraw = true;
    }

    fn update_scrollbar_drag(&mut self, x: f32, y: f32) {
        let Some(drag) = self.scrollbar_drag else {
            return;
        };
        let tab_id = drag.tab_id;
        let (cx, cy, cw, ch) = self.page_content_rect();
        let layout = self.page_scroll_layout(tab_id);
        let entry = self.scroll.entry(tab_id).or_default();
        match drag.axis {
            page_scroll::ScrollbarAxis::Vertical => {
                entry.y = page_scroll::scroll_y_from_pointer(
                    &layout,
                    cy,
                    ch,
                    self.scale_factor,
                    y,
                    drag.grab_offset,
                );
            }
            page_scroll::ScrollbarAxis::Horizontal => {
                entry.x = page_scroll::scroll_x_from_pointer(
                    &layout,
                    cx,
                    cw,
                    self.scale_factor,
                    x,
                    drag.grab_offset,
                );
            }
        }
        self.needs_redraw = true;
    }
    fn content_scroll_drag_threshold(&self) -> f64 {
        8.0 * self.scale_factor as f64
    }

    fn update_content_pointer_drag(&mut self, x: f64, y: f64) {
        if self.context_menu.visible || self.content_pointer_drag.is_none() {
            return;
        }

        let threshold = self.content_scroll_drag_threshold();
        let mut start_scrolling = false;
        let mut scroll_delta = 0.0f32;
        let mut clear_selection_for: Option<zero_browser_shell::TabId> = None;

        if let Some(drag) = self.content_pointer_drag.as_mut() {
            if !drag.scrolling {
                let dy = (y - drag.start_y).abs();
                let dx = (x - drag.start_x).abs();
                if dy >= threshold && dy >= dx {
                    drag.scrolling = true;
                    start_scrolling = true;
                    clear_selection_for = self.shell.active_tab_id();
                }
            }

            if drag.scrolling {
                scroll_delta = (drag.last_y - y) as f32;
                drag.last_y = y;
            }
        }

        if start_scrolling {
            self.page_selection_drag = false;
            if let Some(tab_id) = clear_selection_for {
                self.page_selection.remove(&tab_id);
            }
        }

        if scroll_delta != 0.0 && let Some(tab_id) = self.shell.active_tab_id() {
            self.apply_page_scroll_delta(tab_id, 0.0, scroll_delta);
        }
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: &str, pressed: bool) {
        // 追踪修饰键状态
        match key {
            "Control" => {
                self.ctrl_pressed = pressed;
                return;
            }
            "Meta" | "MetaLeft" | "MetaRight" | "Super" | "SuperLeft" | "SuperRight" => {
                self.cmd_pressed = pressed;
                return;
            }
            "Shift" => {
                self.shift_pressed = pressed;
                return;
            }
            _ => {}
        }

        // 只处理按键按下事件
        if !pressed {
            return;
        }

        // 上下文菜单打开时，Escape 关闭菜单，其他按键忽略
        if self.context_menu.visible {
            match key {
                "Escape" => {
                    self.context_menu.close();
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Up") && !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| {
                            if i > 0 {
                                i - 1
                            } else {
                                self.context_menu.items.len() - 1
                            }
                        })
                        .unwrap_or(self.context_menu.items.len() - 1);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Down") && !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| (i + 1) % self.context_menu.items.len())
                        .unwrap_or(0);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                "Enter" => {
                    self.activate_context_menu_item();
                }
                _ => {}
            }
            return;
        }

        if self.shell.find_state().is_active() {
            self.handle_find_key(key);
        } else if self.address_bar_focused {
            self.handle_address_bar_key(key);
        } else {
            self.handle_global_key(key);
        }
    }

    fn handle_find_key(&mut self, key: &str) {
        match key {
            "Enter" => {
                if self.find_input.is_empty() {
                    self.shell.find_close();
                } else if self.shell.find_state().total_matches() == 0 {
                    self.shell.find_start(&self.find_input.clone());
                } else {
                    self.shell.find_next();
                }
                self.needs_redraw = true;
            }
            "Escape" => {
                self.shell.find_close();
                self.find_input.clear();
                self.needs_redraw = true;
            }
            "Backspace" => {
                self.find_input.pop();
                if self.find_input.is_empty() {
                    self.shell.find_close();
                } else {
                    self.shell.find_start(&self.find_input);
                }
                self.needs_redraw = true;
            }
            _ => {
                if key.len() == 1 {
                    self.find_input.push_str(key);
                    self.shell.find_start(&self.find_input);
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_address_bar_key(&mut self, key: &str) {
        let extend = self.shift_pressed;
        if self.is_modifier_pressed() {
            match key {
                "a" | "A" => {
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                    return;
                }
                "c" | "C" => {
                    let _ = self.address_bar.copy_selection();
                    return;
                }
                "x" | "X" => {
                    if self.address_bar.cut_selection() {
                        self.update_autocomplete();
                        self.needs_redraw = true;
                    }
                    return;
                }
                "v" | "V" => {
                    if self.address_bar.paste_from_clipboard() {
                        self.update_autocomplete();
                        self.needs_redraw = true;
                    }
                    return;
                }
                _ => {}
            }
        }

        match key {
            "Enter" => {
                let url = self.address_bar.text().trim().to_string();
                if !url.is_empty() {
                    let nav_url = if let Some(idx) = self.autocomplete.hovered_index {
                        self.autocomplete
                            .suggestions
                            .get(idx)
                            .map(|s| s.url().to_string())
                            .unwrap_or(url)
                    } else {
                        url
                    };
                    self.navigate_to(&nav_url);
                }
                self.address_bar_focused = false;
                self.address_bar_ime_preedit.clear();
                self.autocomplete.clear();
            }
            "Escape" => {
                self.address_bar_focused = false;
                self.address_bar_ime_preedit.clear();
                self.autocomplete.clear();
                self.update_address_bar_from_active_tab();
            }
            "Backspace" => {
                self.address_bar.delete_backward();
                self.update_autocomplete();
                self.needs_redraw = true;
            }
            "Delete" => {
                self.address_bar.delete_forward();
                self.update_autocomplete();
                self.needs_redraw = true;
            }
            k if key_matches(k, "Left") => {
                self.address_bar.move_left(extend);
                self.needs_redraw = true;
            }
            k if key_matches(k, "Right") => {
                self.address_bar.move_right(extend);
                self.needs_redraw = true;
            }
            "Home" => {
                self.address_bar.move_home(extend);
                self.needs_redraw = true;
            }
            "End" => {
                self.address_bar.move_end(extend);
                self.needs_redraw = true;
            }
            k if key_matches(k, "Down") => {
                if !self.autocomplete.suggestions.is_empty() {
                    let next = self
                        .autocomplete
                        .hovered_index
                        .map(|i| (i + 1).min(self.autocomplete.suggestions.len() - 1))
                        .unwrap_or(0);
                    self.autocomplete.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
            }
            k if key_matches(k, "Up") => {
                if let Some(i) = self.autocomplete.hovered_index {
                    if i > 0 {
                        self.autocomplete.hovered_index = Some(i - 1);
                    } else {
                        self.autocomplete.hovered_index = None;
                    }
                    self.needs_redraw = true;
                }
            }
            "Tab" => {
                if let Some(sug) = self.autocomplete.suggestions.first() {
                    self.address_bar.set_text(sug.url().to_string());
                    self.autocomplete.clear();
                    self.needs_redraw = true;
                }
            }
            _ => {
                if key.len() == 1 {
                    self.address_bar.insert_str(key);
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_global_key(&mut self, key: &str) {
        // Ctrl 修饰键快捷键
        if self.is_modifier_pressed() {
            match key {
                "l" | "L" => {
                    self.address_bar_focused = true;
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                }
                "c" | "C" => {
                    if self.address_bar_focused {
                        let _ = self.address_bar.copy_selection();
                    } else if self.copy_page_selection() {
                        self.needs_redraw = true;
                    }
                }
                "v" | "V" if self.address_bar_focused && self.address_bar.paste_from_clipboard() => {
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
                "x" | "X" if self.address_bar_focused && self.address_bar.cut_selection() => {
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
                "a" | "A" if self.address_bar_focused => {
                    self.address_bar.select_all();
                    self.needs_redraw = true;
                }
                "t" | "T" => {
                    self.new_tab(None);
                }
                "w" | "W" => {
                    self.close_active_tab();
                }
                "r" | "R" => {
                    self.refresh_page();
                }
                "f" | "F" => {
                    self.find_input.clear();
                    self.shell.find_close();
                    self.needs_redraw = true;
                }
                "d" | "D" => {
                    let was_visible = self.bookmarks_bar_visible();
                    self.shell.add_bookmark();
                    if self.bookmarks_bar_visible() != was_visible {
                        self.sync_webview_viewport();
                    }
                    self.needs_redraw = true;
                }
                "+" | "=" => {
                    self.shell.zoom_in();
                    self.needs_redraw = true;
                }
                "-" => {
                    self.shell.zoom_out();
                    self.needs_redraw = true;
                }
                "0" => {
                    self.shell.zoom_reset();
                    self.needs_redraw = true;
                }
                "," => {
                    // Ctrl+, 打开设置页面
                    self.open_settings_page();
                }
                _ => {}
            }
            return;
        }

        // 无修饰键的全局快捷键（保留兼容无 Ctrl 的单键模式）
        match key {
            "l" => {
                self.address_bar_focused = true;
                self.needs_redraw = true;
            }
            "t" => {
                self.new_tab(None);
            }
            "w" => {
                self.close_active_tab();
            }
            "r" => {
                self.refresh_page();
            }
            k if key_matches(k, "Left") => {
                self.go_back();
            }
            k if key_matches(k, "Right") => {
                self.go_forward();
            }
            "Home" => {
                self.navigate_to("https://example.com");
            }
            "f" => {
                self.find_input.clear();
                self.shell.find_close();
                self.needs_redraw = true;
            }
            "+" | "=" => {
                self.shell.zoom_in();
                self.needs_redraw = true;
            }
            "-" => {
                self.shell.zoom_out();
                self.needs_redraw = true;
            }
            "0" => {
                self.shell.zoom_reset();
                self.needs_redraw = true;
            }
            "n" => {
                self.shell.find_next();
                self.find_input = self.shell.find_state().query().to_string();
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    /// 更新自动补全建议
    fn update_autocomplete(&mut self) {
        let query = self.address_bar.text().trim();
        if query.is_empty() {
            self.autocomplete.clear();
            return;
        }
        self.autocomplete.suggestions = self.shell.suggest(query);
        self.autocomplete.hovered_index = None;
    }

    /// 处理鼠标移动
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = (x, y);

        // 上下文菜单悬停检测
        if self.context_menu.visible {
            let hovered = self.context_menu_hit_test(x, y);
            if hovered != self.context_menu.hovered_index {
                self.context_menu.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        // 自动补全悬停
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            let hovered = self.autocomplete_hit_test(x, y);
            if hovered != self.autocomplete.hovered_index {
                self.autocomplete.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        let s = self.scale_factor;
        let y_f = y as f32;
        let chrome_bottom = self.chrome_top_y_for(s);
        if y_f < chrome_bottom {
            self.needs_redraw = true;
        }

        if (old_pos.0 - x).abs() > 1.0 || (old_pos.1 - y).abs() > 1.0 {
            if self.address_bar_drag && self.address_bar_focused && self.left_button_down {
                let s = self.scale_factor;
                let font_size = layout::CHROME_FONT_SIZE * s;
                let (bar_x, _, _, _) = self.address_bar_layout();
                let rel_x = (x as f32 - bar_x - 10.0 * s).max(0.0);
                let idx = self
                    .address_bar
                    .x_to_cursor(rel_x, |t| self.measure_ui_text_width(t, font_size));
                self.address_bar.set_cursor(idx, true);
                self.needs_redraw = true;
            }

            let toolbar_h = self.chrome_top_y_for(self.scale_factor);
            if (y as f32) < toolbar_h {
                self.needs_redraw = true;
            }
            if (y as f32) < layout::TAB_STRIP_HEIGHT * self.scale_factor {
                self.update_window_control_hover(x, y);
            }
            self.update_tab_bar_drag(x, y);
        }

        if self.left_button_down {
            if self.scrollbar_drag.is_some() {
                self.update_scrollbar_drag(x as f32, y as f32);
            } else {
                self.update_content_pointer_drag(x, y);
            }
        }

        if self.page_selection_drag
            && self.left_button_down
            && self.scrollbar_drag.is_none()
            && self.content_pointer_drag.as_ref().is_none_or(|d| !d.scrolling)
            && let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32)
            && let Some(glyphs) = self.page_glyphs(tab_id)
            && let Some(idx) = hit_test_glyph(&glyphs, doc_x, doc_y)
            && let Some(sel) = self.page_selection.get_mut(&tab_id)
        {
            sel.focus = idx;
            self.needs_redraw = true;
        }

        self.update_hovered_link_at(x, y);
    }

    /// 处理鼠标点击（物理像素坐标）
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool, button: &str) {
        if button == "Left" {
            if pressed {
                self.left_button_down = true;
            } else {
                self.left_button_down = false;
                let was_scrollbar_drag = self.scrollbar_drag.is_some();
                self.scrollbar_drag = None;
                let was_scroll_drag = self
                    .content_pointer_drag
                    .as_ref()
                    .is_some_and(|d| d.scrolling);
                self.content_pointer_drag = None;
                self.tab_bar_drag_press = None;
                self.address_bar_drag = false;
                if was_scrollbar_drag || was_scroll_drag {
                    self.page_selection_drag = false;
                    return;
                }
                if self.page_selection_drag {
                    self.page_selection_drag = false;
                    if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32) {
                        let collapsed = self.page_selection.get(&tab_id).is_none_or(|s| s.is_collapsed());
                        if collapsed
                            && let Some(href) = self.tabs.hit_test_link(tab_id, doc_x, doc_y)
                        {
                            self.navigate_to(&href);
                        }
                    }
                }
                return;
            }
        } else if !pressed {
            return;
        }

        // 右键 → 上下文菜单
        if button == "Right" {
            self.show_context_menu(x, y);
            return;
        }

        // 左键点击时关闭上下文菜单
        if self.context_menu.visible {
            if let Some(idx) = self.context_menu_hit_test(x, y) {
                // 点击菜单项
                self.context_menu.hovered_index = Some(idx);
                self.activate_context_menu_item();
                return;
            }
            // 点击菜单外关闭
            self.context_menu.close();
            self.needs_redraw = true;
            return;
        }

        let s = self.scale_factor;
        let y_f = y as f32;
        let x_f = x as f32;
        let width = self.physical_size.0 as f32;

        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        let chrome_top = self.chrome_top_y_for(s);
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let nav_btn_w = layout::NAV_BUTTON_WIDTH * s;
        let addr_padding = layout::ADDRESS_BAR_PADDING * s;
        let tab_close_size = layout::TAB_CLOSE_SIZE * s;
        let autocomplete_row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;

        // 1. 自动补全下拉区域点击
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            if let Some(idx) = self.autocomplete_hit_test(x, y) {
                let url = self.autocomplete.suggestions.get(idx).map(|s| s.url().to_string());
                if let Some(url) = url {
                    self.navigate_to(&url);
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                    return;
                }
            }
            let autocomplete_top = toolbar_h;
            let autocomplete_height = self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE) as f32
                * autocomplete_row_h;
            if y_f >= autocomplete_top && y_f < autocomplete_top + autocomplete_height {
                return;
            }
            self.autocomplete.clear();
        }

        // 2. 标签栏区域点击
        if y_f < tab_strip_h {
            if y_f >= tab_y
                && let Some(action) = self.window_control_hit_test(x_f, y_f, width, s)
            {
                self.pending_window_chrome_action = Some(action);
                self.needs_redraw = true;
                return;
            }

            if y_f >= tab_y {
                let new_tab_x = self.new_tab_button_x();
                if x_f >= new_tab_x && x_f < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
                    self.new_tab(None);
                    return;
                }

                for &(id, tab_x, tab_w) in &self.tab_layout {
                    if x_f >= tab_x && x_f < tab_x + tab_w {
                        let close_x = tab_x + tab_w - 24.0 * s;
                        let close_y_center = tab_y + tab_bar_h / 2.0;
                        if x_f >= close_x
                            && x_f <= close_x + tab_close_size
                            && (y_f - close_y_center).abs() <= tab_close_size / 2.0
                        {
                            self.close_tab_by_id(id);
                            return;
                        }
                        if Some(id) != self.shell.active_tab_id() {
                            self.shell.switch_tab(id);
                            self.tabs.on_active_tab_changed(self.shell.active_tab_id());
                            self.set_hovered_link_url(None);
                            self.update_address_bar_from_active_tab();
                            self.needs_redraw = true;
                        }
                        return;
                    }
                }
            }

            if self.supports_tab_bar_window_drag() && self.is_tab_bar_blank_hit(x_f, y_f, width, s) {
                self.handle_tab_bar_blank_press(x, y);
            }
            return;
        }

        // 3. 地址栏区域点击
        if y_f < toolbar_h {
            let addr_bar_x = nav_w + addr_padding;

            if x_f < nav_w {
                let button_index = ((x_f - 8.0 * s) / nav_btn_w) as i32;
                match button_index {
                    0 => self.go_back(),
                    1 => self.go_forward(),
                    2 => self.refresh_page(),
                    3 => {
                        let home = self.shell.settings().home_url.clone();
                        self.navigate_to(&home);
                    }
                    _ => {}
                }
                return;
            }

            if x_f >= addr_bar_x && x_f <= width - addr_padding {
                self.handle_address_bar_press(x, y);
                return;
            }
        }

        // 4. 书签栏区域点击
        if y_f >= toolbar_h && y_f < chrome_top {
            self.handle_bookmark_bar_click(x_f, y_f, toolbar_h, width, s);
            return;
        }

        // 5. 查找栏区域点击
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        if self.shell.find_state().is_active() && y_f >= content_y && y_f < content_y + layout::FIND_BAR_HEIGHT * s {
            let bar_w = 320.0 * s;
            let bar_x = width - bar_w - 10.0 * s;
            if x_f >= bar_x && x_f <= bar_x + bar_w {
                let close_x = bar_x + bar_w - 40.0 * s;
                if x_f >= close_x {
                    self.shell.find_close();
                    self.find_input.clear();
                    self.needs_redraw = true;
                    return;
                }
                let prev_x = bar_x + bar_w - 100.0 * s;
                let next_x = bar_x + bar_w - 70.0 * s;
                if x_f >= prev_x && x_f < prev_x + 28.0 * s {
                    self.shell.find_previous();
                    self.needs_redraw = true;
                    return;
                }
                if x_f >= next_x && x_f < next_x + 28.0 * s {
                    self.shell.find_next();
                    self.needs_redraw = true;
                    return;
                }
                return;
            }
        }

        // 6. 页面内容区域 — 链接点击 / 取消地址栏焦点
        let find_bar_h = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };
        let page_top = content_y + find_bar_h;

        if y_f >= content_y
            && y_f < content_y + content_h
            && x_f >= content_x
            && x_f < content_x + content_w
            && y_f >= page_top
        {
            if button == "Left" {
                if let Some((tab_id, hit)) = self.scrollbar_hit_at(x_f, y_f) {
                    self.start_scrollbar_interaction(tab_id, hit, x_f, y_f);
                    return;
                }
                if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f)
                    && let Some(glyphs) = self.page_glyphs(tab_id)
                {
                self.content_pointer_drag = Some(ContentPointerDrag {
                    start_x: x,
                    start_y: y,
                    last_y: y,
                    scrolling: false,
                });
                let idx = hit_test_glyph(&glyphs, doc_x, doc_y).unwrap_or(0);
                if self.shift_pressed {
                    if let Some(sel) = self.page_selection.get_mut(&tab_id) {
                        sel.focus = idx;
                    } else {
                        self.page_selection.insert(tab_id, GlyphSelection::collapsed(idx));
                    }
                } else {
                    self.page_selection.insert(tab_id, GlyphSelection::collapsed(idx));
                }
                self.page_selection_drag = true;
                self.needs_redraw = true;
                }
            }

            if self.address_bar_focused {
                self.address_bar_focused = false;
                self.autocomplete.clear();
                self.needs_redraw = true;
            }
        }
    }

    /// 处理书签栏点击
    fn handle_bookmark_bar_click(&mut self, x: f32, _y: f32, _bar_y: f32, _width: f32, s: f32) {
        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let mut target_url: Option<String> = None;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;
            if x >= bx && x < bx + item_w {
                target_url = Some(bm.url().to_string());
                break;
            }
            bx += item_w + 8.0 * s;
        }

        if let Some(url) = target_url {
            self.navigate_to(&url);
        }
    }

    /// 处理 IME 输入（地址栏）
    pub fn handle_ime(&mut self, event: zero_host_runtime::event::ImeEvent) {
        if !self.address_bar_focused {
            return;
        }
        match event {
            zero_host_runtime::event::ImeEvent::Preedit { text, .. } => {
                self.address_bar_ime_preedit = text;
                self.needs_redraw = true;
            }
            zero_host_runtime::event::ImeEvent::Commit(text) => {
                self.address_bar_ime_preedit.clear();
                if !text.is_empty() {
                    self.address_bar.insert_str(&text);
                    self.update_autocomplete();
                }
                self.needs_redraw = true;
            }
            zero_host_runtime::event::ImeEvent::Enabled | zero_host_runtime::event::ImeEvent::Disabled => {}
        }
    }

    fn copy_page_selection(&self) -> bool {
        let Some(tab_id) = self.shell.active_tab_id() else {
            return false;
        };
        let Some(sel) = self.page_selection.get(&tab_id) else {
            return false;
        };
        if sel.is_collapsed() {
            return false;
        }
        let Some(glyphs) = self.page_glyphs(tab_id) else {
            return false;
        };
        let text = GlyphSelection::selected_text(&glyphs, sel);
        if text.is_empty() {
            return false;
        }
        crate::clipboard::write_text(&text)
    }

    fn page_doc_point(&self, x_f: f32, y_f: f32) -> Option<(TabId, f32, f32)> {
        let s = self.scale_factor;
        let tab_id = self.shell.active_tab_id()?;
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let find_bar_h = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };
        let page_top = content_y + find_bar_h;
        let content_bottom = content_y + content_h;
        if x_f < content_x || x_f >= content_x + content_w || y_f < page_top || y_f >= content_bottom {
            return None;
        }
        let scroll = self.tab_scroll_state(tab_id);
        Some((tab_id, (x_f - content_x) / s + scroll.x, (y_f - page_top + scroll.y) / s))
    }

    /// 与渲染一致的页面 glyph 列表。
    fn page_glyphs(&self, tab_id: TabId) -> Option<Vec<zero_render_foundation::primitive::GlyphPrimitive>> {
        Some(self.tabs.last_render(tab_id)?.primitives.glyphs.clone())
    }

    fn address_bar_layout(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = self.physical_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let inset = layout::ADDRESS_BAR_INPUT_V_INSET * s;
        let bar_y = layout::TAB_STRIP_HEIGHT * s + inset;
        let bar_h = layout::ADDRESS_BAR_HEIGHT * s - 2.0 * inset;
        (bar_x, bar_y, bar_w, bar_h)
    }

    fn address_bar_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        let s = self.scale_factor;
        if y_f >= layout::TOOLBAR_HEIGHT * s {
            return false;
        }
        let (bar_x, _, bar_w, _) = self.address_bar_layout();
        x_f >= bar_x && x_f <= bar_x + bar_w
    }

    fn handle_address_bar_press(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let font_size = layout::CHROME_FONT_SIZE * s;
        let (bar_x, _, _, _) = self.address_bar_layout();
        let rel_x = (x as f32 - bar_x - 10.0 * s).max(0.0);
        let measure = |t: &str| self.measure_ui_text_width(t, font_size);
        let idx = self.address_bar.x_to_cursor(rel_x, measure);
        let extend = self.shift_pressed;
        if let Some((last_t, last_x, last_y)) = self.address_bar_last_click
            && last_t.elapsed() < TAB_BAR_DOUBLE_CLICK_INTERVAL
            && (x - last_x).abs() < 5.0
            && (y - last_y).abs() < 5.0
        {
            self.address_bar.select_word_at(idx);
            self.address_bar_last_click = None;
            self.address_bar_focused = true;
            self.address_bar_drag = false;
            self.needs_redraw = true;
            return;
        }
        self.address_bar_last_click = Some((Instant::now(), x, y));
        self.address_bar.set_cursor(idx, extend);
        self.address_bar_focused = true;
        self.address_bar_drag = true;
        self.autocomplete.clear();
        self.needs_redraw = true;
    }

    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let y_f = y as f32;
        let x_f = x as f32;
        let chrome_top = self.chrome_top_y_for(s);

        let context_type = if self.address_bar_hit_test(x_f, y_f) {
            ContextType::Editable
        } else if y_f < chrome_top {
            return;
        } else if let Some(tab_id) = self.shell.active_tab_id()
            && self.page_selection.get(&tab_id).is_some_and(|sel| !sel.is_collapsed())
        {
            ContextType::Selection
        } else if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f)
            && self.tabs.hit_test_link(tab_id, doc_x, doc_y).is_some()
        {
            ContextType::Link
        } else {
            ContextType::Page
        };

        let menu = ContextMenu::new(context_type);
        let items: Vec<String> = menu
            .items()
            .iter()
            .map(|mi| {
                if mi.is_separator() {
                    "---".to_string()
                } else {
                    mi.label().to_string()
                }
            })
            .collect();

        self.context_menu = ContextMenuState {
            visible: true,
            context_type,
            items,
            hovered_index: None,
            x: x as f32,
            y: y as f32,
        };
        self.needs_redraw = true;
    }

    /// 激活上下文菜单中选中的项
    fn activate_context_menu_item(&mut self) {
        let idx = match self.context_menu.hovered_index {
            Some(i) => i,
            None => return,
        };

        let label = match self.context_menu.items.get(idx) {
            Some(l) => l.clone(),
            None => return,
        };

        self.context_menu.close();
        self.needs_redraw = true;

        match label.as_str() {
            "后退" => self.go_back(),
            "前进" => self.go_forward(),
            "重新加载" => self.refresh_page(),
            "复制" => {
                if self.context_menu.context_type == ContextType::Editable {
                    let _ = self.address_bar.copy_selection();
                } else {
                    let _ = self.copy_page_selection();
                }
            }
            "剪切" if self.address_bar.cut_selection() => {
                self.update_autocomplete();
            }
            "粘贴" if self.address_bar.paste_from_clipboard() => {
                self.update_autocomplete();
            }
            "全选" => {
                self.address_bar.select_all();
            }
            _ => {}
        }
        self.needs_redraw = true;
    }

    /// 上下文菜单命中检测
    fn context_menu_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        if !self.context_menu.visible {
            return None;
        }

        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = 28.0 * s;
        let menu_w = 200.0 * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;

        let x_f = x as f32;
        let y_f = y as f32;

        if x_f < menu_x || x_f > menu_x + menu_w || y_f < menu_y || y_f > menu_y + menu_h {
            return None;
        }

        let idx = ((y_f - menu_y) / row_h) as usize;
        if idx < self.context_menu.items.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// 自动补全下拉命中检测（物理像素坐标）
    fn autocomplete_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let s = self.scale_factor;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = self.physical_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;

        let autocomplete_top = layout::TOOLBAR_HEIGHT * s;
        let y_f = y as f32;
        let x_f = x as f32;

        if x_f < bar_x || x_f > bar_x + bar_w || y_f < autocomplete_top {
            return None;
        }

        let row_offset = y_f - autocomplete_top;
        if row_offset < 0.0 {
            return None;
        }

        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let index = (row_offset / row_h) as usize;
        if index
            < self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE)
        {
            Some(index)
        } else {
            None
        }
    }

    /// 从活跃标签更新地址栏文本
    fn update_address_bar_from_active_tab(&mut self) {
        if let Some(tab) = self.shell.active_tab() {
            self.address_bar.set_text(tab.url().unwrap_or("").to_string());
        }
    }
}
