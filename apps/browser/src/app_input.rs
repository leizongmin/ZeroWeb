// 输入处理方法（键盘、鼠标、IME、自动补全、上下文菜单）
// 从 app.rs 拆分以控制 app.rs 体积

impl BrowserApp {
    /// 处理鼠标滚轮滚动
    pub fn handle_scroll(&mut self, delta: zero_host_runtime::event::MouseScrollDelta, at_x: f64, at_y: f64) {
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
                if delta_y != 0.0
                    && let Some(tab_id) = self.shell.active_tab_id()
                {
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
    fn apply_page_scroll_delta(&mut self, tab_id: zero_browser_shell::TabId, delta_x: f32, delta_y: f32) {
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
        let geometry = page_scroll::scrollbar_geometry(&layout, scroll, cx, cy, cw, ch, self.scale_factor);
        page_scroll::hit_test_scrollbar(x, y, &geometry).map(|hit| (tab_id, hit))
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
        let geometry = page_scroll::scrollbar_geometry(&layout, scroll, cx, cy, cw, ch, self.scale_factor);

        match hit {
            page_scroll::ScrollbarHit::VerticalThumb => {
                let (_, thumb_y, _, _) = geometry.vertical_thumb.expect("vertical thumb");
                let grab_offset = y - thumb_y;
                let new_y = page_scroll::scroll_y_from_pointer(&layout, cy, ch, self.scale_factor, y, grab_offset);
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
                let new_y = page_scroll::scroll_y_from_pointer(&layout, cy, ch, self.scale_factor, y, grab_offset);
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
                let new_x = page_scroll::scroll_x_from_pointer(&layout, cx, cw, self.scale_factor, x, grab_offset);
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
                let new_x = page_scroll::scroll_x_from_pointer(&layout, cx, cw, self.scale_factor, x, grab_offset);
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
                entry.y = page_scroll::scroll_y_from_pointer(&layout, cy, ch, self.scale_factor, y, drag.grab_offset);
            }
            page_scroll::ScrollbarAxis::Horizontal => {
                entry.x = page_scroll::scroll_x_from_pointer(&layout, cx, cw, self.scale_factor, x, drag.grab_offset);
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

        if scroll_delta != 0.0
            && let Some(tab_id) = self.shell.active_tab_id()
        {
            self.apply_page_scroll_delta(tab_id, 0.0, scroll_delta);
        }
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: &str, pressed: bool, text: Option<&str>) {
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

        if !self.address_bar_focused && key.len() == 1 && !self.ctrl_pressed && !self.cmd_pressed {
            if let Some(tab_id) = self.shell.active_tab_id() {
                let event = if pressed { "keydown" } else { "keyup" };
                if self.tabs.dispatch_key_event(tab_id, event, key, key) {
                    self.needs_redraw = true;
                }
            }
        }

        // 只处理按键按下事件
        if !pressed {
            return;
        }

        // 上下文菜单打开时，Escape 关闭菜单，其他按键忽略
        if self.context_menu.visible {
            match key {
                "Escape" => {
                    // 子菜单展开时优先收起子菜单，再次 Escape 才关闭整个菜单
                    if self.context_menu.open_sub_menu.is_some() {
                        self.context_menu.open_sub_menu = None;
                        self.context_menu.sub_menu_hovered = None;
                    } else {
                        self.context_menu.close();
                    }
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Up") && self.context_menu.open_sub_menu.is_some() => {
                    // 子菜单内向上
                    let parent = self.context_menu.open_sub_menu.and_then(|i| self.context_menu.items.get(i));
                    if let Some(children) = parent.and_then(|p| p.children()) {
                        let len = children.len();
                        let start = self.context_menu.sub_menu_hovered.unwrap_or(len);
                        let mut next = start;
                        for step in 1..=len {
                            let candidate = (start + len - step) % len;
                            if children.get(candidate).is_some_and(|c| c.enabled() && !c.is_separator()) {
                                next = candidate;
                                break;
                            }
                        }
                        self.context_menu.sub_menu_hovered = Some(next);
                        self.needs_redraw = true;
                    }
                }
                k if key_matches(k, "Down") && self.context_menu.open_sub_menu.is_some() => {
                    // 子菜单内向下
                    let parent = self.context_menu.open_sub_menu.and_then(|i| self.context_menu.items.get(i));
                    if let Some(children) = parent.and_then(|p| p.children()) {
                        let len = children.len();
                        let start = self.context_menu.sub_menu_hovered.map(|i| i + 1).unwrap_or(0);
                        let mut next = self.context_menu.sub_menu_hovered.unwrap_or(0);
                        for step in 0..len {
                            let candidate = (start + step) % len;
                            if children.get(candidate).is_some_and(|c| c.enabled() && !c.is_separator()) {
                                next = candidate;
                                break;
                            }
                        }
                        self.context_menu.sub_menu_hovered = Some(next);
                        self.needs_redraw = true;
                    }
                }
                k if key_matches(k, "Up") && !self.context_menu.items.is_empty() => {
                    let len = self.context_menu.items.len();
                    let start = self.context_menu.hovered_index.unwrap_or(len);
                    let mut next = start;
                    for step in 1..=len {
                        let candidate = (start + len - step) % len;
                        if self.context_menu_menu_item_activatable(candidate) {
                            next = candidate;
                            break;
                        }
                    }
                    self.context_menu.hovered_index = Some(next);
                    // 移动到 sub_menu 项时展开；移到普通项时收起
                    self.sync_open_sub_menu_with_hover();
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Down") && !self.context_menu.items.is_empty() => {
                    let len = self.context_menu.items.len();
                    let start = self.context_menu.hovered_index.map(|i| i + 1).unwrap_or(0);
                    let mut next = self.context_menu.hovered_index.unwrap_or(0);
                    for step in 0..len {
                        let candidate = (start + step) % len;
                        if self.context_menu_menu_item_activatable(candidate) {
                            next = candidate;
                            break;
                        }
                    }
                    self.context_menu.hovered_index = Some(next);
                    self.sync_open_sub_menu_with_hover();
                    self.needs_redraw = true;
                }
                k if key_matches(k, "Right") => {
                    // 在 sub_menu 父项上按右键：展开子菜单并选中第一个可激活子项
                    if let Some(idx) = self.context_menu.hovered_index
                        && let Some(item) = self.context_menu.items.get(idx)
                        && item.is_sub_menu()
                    {
                        self.context_menu.open_sub_menu = Some(idx);
                        self.context_menu.sub_menu_hovered = self
                            .context_menu
                            .items
                            .get(idx)
                            .and_then(|p| p.children())
                            .and_then(|chs| chs.iter().position(|c| c.enabled() && !c.is_separator()));
                        self.needs_redraw = true;
                    }
                }
                k if key_matches(k, "Left")
                    // 子菜单展开时按左键：收起，焦点回到父项
                    && self.context_menu.open_sub_menu.is_some() =>
                {
                    self.context_menu.open_sub_menu = None;
                    self.context_menu.sub_menu_hovered = None;
                    self.needs_redraw = true;
                }
                "Enter" => {
                    if self.context_menu.open_sub_menu.is_some() && self.context_menu.sub_menu_hovered.is_some() {
                        self.activate_sub_menu_item();
                    } else if let Some(idx) = self.context_menu.hovered_index
                        && let Some(item) = self.context_menu.items.get(idx)
                        && item.is_sub_menu()
                    {
                        // Enter 在 sub_menu 父项：展开
                        self.context_menu.open_sub_menu = Some(idx);
                        self.context_menu.sub_menu_hovered = item
                            .children()
                            .and_then(|chs| chs.iter().position(|c| c.enabled() && !c.is_separator()));
                        self.needs_redraw = true;
                    } else {
                        self.activate_context_menu_item();
                    }
                }
                _ => {}
            }
            return;
        }

        if self.shell.find_state().is_active() {
            self.handle_find_key(key, text);
        } else if self.address_bar_focused {
            self.handle_address_bar_key(key, text);
        } else {
            self.handle_global_key(key);
        }
    }

    fn handle_find_key(&mut self, key: &str, text: Option<&str>) {
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
                if let Some(inserted) = Self::extract_typed_text(key, text) {
                    self.find_input.push_str(&inserted);
                    self.shell.find_start(&self.find_input);
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_address_bar_key(&mut self, key: &str, text: Option<&str>) {
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
                    let nav_url = if let Some(idx) = self.autocomplete.highlight_index() {
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
                    self.autocomplete.hovered_index = None;
                    let next = self
                        .autocomplete
                        .selected_index
                        .map(|i| (i + 1).min(self.autocomplete.suggestions.len() - 1))
                        .unwrap_or(0);
                    self.autocomplete.selected_index = Some(next);
                    self.needs_redraw = true;
                }
            }
            k if key_matches(k, "Up") => {
                if !self.autocomplete.suggestions.is_empty() {
                    self.autocomplete.hovered_index = None;
                    if let Some(i) = self.autocomplete.selected_index {
                        if i > 0 {
                            self.autocomplete.selected_index = Some(i - 1);
                        } else {
                            self.autocomplete.selected_index = None;
                        }
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
                if let Some(inserted) = Self::extract_typed_text(key, text) {
                    self.address_bar.insert_str(&inserted);
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn extract_typed_text(key: &str, text: Option<&str>) -> Option<String> {
        if let Some(raw) = text.filter(|t| !t.is_empty()) {
            let sanitized: String = raw.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            if !sanitized.is_empty() {
                return Some(sanitized);
            }
        }
        if key.len() == 1 {
            return Some(key.to_string());
        }
        None
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
                "t" | "T" if self.shift_pressed => {
                    self.new_private_tab(None);
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
                "h" | "H" => {
                    self.open_history_page();
                }
                "j" | "J" => {
                    self.open_downloads_page();
                }
                k if key_matches(k, "Tab") => {
                    self.cycle_active_tab(self.shift_pressed);
                }
                k if key_matches(k, "PageDown") => {
                    self.cycle_active_tab(false);
                }
                k if key_matches(k, "PageUp") => {
                    self.cycle_active_tab(true);
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
                let home = self.shell.settings().home_url.clone();
                self.navigate_to(&home);
            }
            "F5" => {
                self.refresh_page();
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

    /// 循环切换活跃标签（`reverse=true` 向前，否则向后）。
    fn cycle_active_tab(&mut self, reverse: bool) {
        let active_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        let ids: Vec<TabId> = self.shell.tabs().map(|t| t.id()).collect();
        if ids.len() < 2 {
            return;
        }
        let current = match ids.iter().position(|&id| id == active_id) {
            Some(i) => i,
            None => return,
        };
        let count = ids.len();
        let next = if reverse {
            (current + count - 1) % count
        } else {
            (current + 1) % count
        };
        let target = ids[next];
        self.shell.switch_tab(target);
        self.shell.set_tab_needs_attention(target, false);
        self.needs_redraw = true;
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
        self.autocomplete.selected_index = None;
    }

    /// 处理鼠标移动
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = (x, y);

        // 上下文菜单悬停检测（含子菜单面板）
        if self.context_menu.visible {
            // 先检测子菜单面板
            if let Some(sub_idx) = self.sub_menu_hit_test(x, y) {
                if self.context_menu.sub_menu_hovered != Some(sub_idx) {
                    self.context_menu.sub_menu_hovered = Some(sub_idx);
                    self.needs_redraw = true;
                }
            } else {
                if self.context_menu.sub_menu_hovered.is_some() {
                    self.context_menu.sub_menu_hovered = None;
                    self.needs_redraw = true;
                }
                // 主菜单 hit-test
                let hovered = self.context_menu_hit_test(x, y);
                if hovered != self.context_menu.hovered_index {
                    self.context_menu.hovered_index = hovered;
                    // hover 到 sub_menu 项时自动展开，hover 离开则收起
                    let new_open = hovered.and_then(|i| {
                        let item = self.context_menu.items.get(i)?;
                        if item.is_sub_menu() && item.enabled() { Some(i) } else { None }
                    });
                    if new_open != self.context_menu.open_sub_menu {
                        self.context_menu.open_sub_menu = new_open;
                        self.context_menu.sub_menu_hovered = None;
                    }
                    self.needs_redraw = true;
                } else if hovered.is_none() && self.context_menu.open_sub_menu.is_some() {
                    // 鼠标移出主菜单且不在子面板：收起子菜单
                    self.context_menu.open_sub_menu = None;
                    self.context_menu.sub_menu_hovered = None;
                    self.needs_redraw = true;
                }
            }
        }

        // 自动补全悬停
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            let hovered = self.autocomplete_hit_test(x, y);
            if hovered != self.autocomplete.hovered_index {
                self.autocomplete.hovered_index = hovered;
                if hovered.is_some() {
                    self.autocomplete.selected_index = None;
                }
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
                let text_x = self.address_bar_text_origin_x();
                let rel_x = (x as f32 - text_x).max(0.0);
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
        self.update_scrollbar_hover(x, y);
    }

    fn update_scrollbar_hover(&mut self, x: f64, y: f64) {
        if self.scrollbar_drag.is_some() {
            return;
        }
        let width = self.physical_size.0;
        let height = self.physical_size.1;
        let hover = self
            .shell
            .active_tab_id()
            .and_then(|tab_id| {
                let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
                let layout = self.page_scroll_layout_for(tab_id, width, height);
                if !layout.show_vertical && !layout.show_horizontal {
                    return None;
                }
                let scroll = self.tab_scroll_state(tab_id);
                let geometry = crate::page_scroll::scrollbar_geometry(
                    &layout,
                    scroll,
                    cx,
                    cy,
                    cw,
                    ch,
                    self.scale_factor,
                );
                crate::page_scroll::hit_test_scrollbar(x as f32, y as f32, &geometry)
            });
        if hover != self.scrollbar_hover {
            self.scrollbar_hover = hover;
            self.needs_redraw = true;
        }
    }

    fn find_bar_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        if !self.shell.find_state().is_active() {
            return false;
        }
        let (bar_x, bar_y, bar_w, bar_h) =
            self.find_bar_rect_for(self.physical_size.0, self.physical_size.1);
        x_f >= bar_x && x_f <= bar_x + bar_w && y_f >= bar_y && y_f <= bar_y + bar_h
    }

    /// 处理鼠标点击（物理像素坐标）
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool, button: &str) {
        if button == "Left" {
            if pressed {
                self.left_button_down = true;
            } else {
                self.left_button_down = false;
                if self.context_menu.visible && self.context_menu_suppress_left_up {
                    self.context_menu_suppress_left_up = false;
                    self.scrollbar_drag = None;
                    self.content_pointer_drag = None;
                    self.tab_bar_drag_press = None;
                    self.address_bar_drag = false;
                    self.page_selection_drag = false;
                    return;
                }
                let was_scrollbar_drag = self.scrollbar_drag.is_some();
                self.scrollbar_drag = None;
                let was_scroll_drag = self.content_pointer_drag.as_ref().is_some_and(|d| d.scrolling);
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
                        if collapsed {
                            let allowed = self.tabs.dispatch_page_click(tab_id, doc_x, doc_y);
                            if allowed && let Some(href) = self.tabs.hit_test_link(tab_id, doc_x, doc_y) {
                                self.navigate_to(&href);
                            }
                            if allowed {
                                self.needs_redraw = true;
                            }
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

        // 中键点击「+」→ 无痕标签页
        if button == "Middle" {
            let s = self.scale_factor;
            let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
            let tab_y = layout::TAB_BAR_TOP_INSET * s;
            let y_f = y as f32;
            if y_f >= tab_y && y_f < tab_strip_h {
                let new_tab_x = self.new_tab_button_x();
                if (x as f32) >= new_tab_x && (x as f32) < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
                    self.new_private_tab(None);
                    return;
                }
            }
        }

        // 左键点击时处理上下文菜单
        if self.context_menu.visible {
            // 先检测子菜单面板点击
            if let Some(sub_idx) = self.sub_menu_hit_test(x, y) {
                self.context_menu.sub_menu_hovered = Some(sub_idx);
                self.activate_sub_menu_item();
                self.context_menu_suppress_left_up = false;
                return;
            }
            if let Some(idx) = self.context_menu_hit_test(x, y) {
                // 点击 sub_menu 父项：切换展开状态，不触发动作
                if let Some(item) = self.context_menu.items.get(idx)
                    && item.is_sub_menu()
                {
                    let cur = self.context_menu.open_sub_menu;
                    self.context_menu.open_sub_menu = if cur == Some(idx) { None } else { Some(idx) };
                    self.context_menu.hovered_index = Some(idx);
                    self.context_menu.sub_menu_hovered = None;
                    self.context_menu_suppress_left_up = false;
                    self.needs_redraw = true;
                    return;
                }
                self.context_menu.hovered_index = Some(idx);
                self.context_menu_suppress_left_up = false;
                self.activate_context_menu_item();
                return;
            }
            self.context_menu.close();
            self.context_menu_suppress_left_up = false;
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
                            self.shell.set_tab_needs_attention(id, false);
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
            let nav_w = self.nav_section_width();
            let addr_bar_x = nav_w + addr_padding;

            if x_f < nav_w {
                let button_index = ((x_f - layout::NAV_SECTION_LEADING_PAD * s) / nav_btn_w) as i32;
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

            if self.toolbar_download_hit_test(x_f, y_f) {
                self.download_panel_open = !self.download_panel_open;
                self.needs_redraw = true;
                return;
            }

            if self.toolbar_theme_hit_test(x_f, y_f) {
                self.cycle_color_theme();
                return;
            }

            if self.toolbar_menu_hit_test(x_f, y_f) {
                self.show_browser_menu();
                return;
            }

            if x_f >= addr_bar_x && x_f <= addr_bar_x + self.address_bar_layout().2 {
                if self.address_bar_trailing_slot_hit_test(x_f, y_f, 0) {
                    self.shell.toggle_current_bookmark();
                    self.needs_redraw = true;
                    return;
                }
                if self.address_bar_trailing_slot_hit_test(x_f, y_f, 1) {
                    self.show_site_permissions_menu();
                    return;
                }
                self.handle_address_bar_press(x, y);
                return;
            }
        }

        // 4. 书签栏区域点击
        if y_f >= toolbar_h && y_f < chrome_top {
            self.handle_bookmark_bar_click(x_f, y_f, toolbar_h, width, s);
            return;
        }

        // 5. 浮动查找栏点击
        if self.find_bar_hit_test(x_f, y_f) {
            let (bar_x, _bar_y, bar_w, _bar_h) =
                self.find_bar_rect_for(self.physical_size.0, self.physical_size.1);
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

        // 6. 页面内容区域 — 链接点击 / 取消地址栏焦点
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let page_top = content_y;

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
                    self.tabs.dispatch_page_mousedown(tab_id, doc_x, doc_y);
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

    /// 处理 IME 输入（地址栏 / 查找框）
    pub fn handle_ime(&mut self, event: zero_host_runtime::event::ImeEvent) {
        let in_address_bar = self.address_bar_focused;
        let in_find_bar = self.shell.find_state().is_active();
        if !in_address_bar && !in_find_bar {
            return;
        }
        match event {
            zero_host_runtime::event::ImeEvent::Preedit { text, .. } => {
                if in_address_bar {
                    self.address_bar_ime_preedit = text;
                    self.needs_redraw = true;
                }
            }
            zero_host_runtime::event::ImeEvent::Commit(text) => {
                if in_address_bar {
                    self.address_bar_ime_preedit.clear();
                }
                if !text.is_empty() {
                    if in_address_bar {
                        self.address_bar.insert_str(&text);
                        self.update_autocomplete();
                    } else {
                        self.find_input.push_str(&text);
                        self.shell.find_start(&self.find_input);
                    }
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
        if self.find_bar_hit_test(x_f, y_f) {
            return None;
        }
        let (content_x, content_y, content_w, content_h) = self.page_content_rect();
        let page_top = content_y;
        let content_bottom = content_y + content_h;
        if x_f < content_x || x_f >= content_x + content_w || y_f < page_top || y_f >= content_bottom {
            return None;
        }
        let scroll = self.tab_scroll_state(tab_id);
        Some((
            tab_id,
            (x_f - content_x) / s + scroll.x,
            (y_f - page_top + scroll.y) / s,
        ))
    }

    /// 与渲染一致的页面 glyph 列表。
    fn page_glyphs(&self, tab_id: TabId) -> Option<Vec<zero_render_foundation::primitive::GlyphPrimitive>> {
        Some(self.tabs.last_render(tab_id)?.primitives.glyphs.clone())
    }

    fn nav_section_width(&self) -> f32 {
        let s = self.scale_factor;
        (layout::NAV_SECTION_LEADING_PAD + layout::NAV_BUTTON_WIDTH * 4.0 + layout::NAV_SECTION_TRAILING_GAP) * s
    }

    fn address_bar_layout(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let bar_x = self.nav_section_width() + layout::ADDRESS_BAR_PADDING * s;
        let download_w = layout::TOOLBAR_DOWNLOAD_BUTTON_WIDTH * s;
        let theme_w = layout::TOOLBAR_THEME_BUTTON_WIDTH * s;
        let menu_w = layout::TOOLBAR_MENU_BUTTON_WIDTH * s;
        let trailing_reserved = layout::ADDRESS_BAR_PADDING * s
            + layout::TOOLBAR_TRAILING_GAP * s
            + download_w
            + layout::TOOLBAR_TRAILING_GAP * s
            + theme_w
            + layout::TOOLBAR_TRAILING_GAP * s
            + menu_w;
        let bar_w = self.physical_size.0 as f32 - bar_x - trailing_reserved;
        let inset = layout::ADDRESS_BAR_INPUT_V_INSET * s;
        let bar_y = layout::TAB_STRIP_HEIGHT * s + inset;
        let bar_h = layout::ADDRESS_BAR_HEIGHT * s - 2.0 * inset;
        (bar_x, bar_y, bar_w, bar_h)
    }

    fn address_bar_text_origin_x(&self) -> f32 {
        let s = self.scale_factor;
        let (bar_x, _, _, _) = self.address_bar_layout();
        let border = s.max(1.0);
        bar_x + border + layout::ADDRESS_BAR_INNER_PAD_H * s + layout::ADDRESS_BAR_LEADING_SLOT_WIDTH * s
    }

    fn toolbar_download_button_rect(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (theme_x, theme_y, _, theme_h) = self.toolbar_theme_button_rect();
        let btn_w = layout::TOOLBAR_DOWNLOAD_BUTTON_WIDTH * s;
        let btn_x = theme_x - layout::TOOLBAR_TRAILING_GAP * s - btn_w;
        (btn_x, theme_y, btn_w, theme_h)
    }

    fn toolbar_download_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        let (bx, by, bw, bh) = self.toolbar_download_button_rect();
        x_f >= bx && x_f <= bx + bw && y_f >= by && y_f <= by + bh
    }

    fn toolbar_theme_button_rect(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (menu_x, menu_y, _, menu_h) = self.toolbar_menu_button_rect();
        let btn_w = layout::TOOLBAR_THEME_BUTTON_WIDTH * s;
        let btn_x = menu_x - layout::TOOLBAR_TRAILING_GAP * s - btn_w;
        (btn_x, menu_y, btn_w, menu_h)
    }

    fn toolbar_theme_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        let (bx, by, bw, bh) = self.toolbar_theme_button_rect();
        x_f >= bx && x_f <= bx + bw && y_f >= by && y_f <= by + bh
    }

    #[cfg(test)]
    pub fn toolbar_theme_button_rect_for_test(&self) -> (f32, f32, f32, f32) {
        self.toolbar_theme_button_rect()
    }

    fn toolbar_menu_button_rect(&self) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let btn_w = layout::TOOLBAR_MENU_BUTTON_WIDTH * s;
        // 最右侧按钮：紧贴窗口右内边距
        let btn_x = self.physical_size.0 as f32 - layout::ADDRESS_BAR_PADDING * s - btn_w;
        let (_, bar_y, _, bar_h) = self.address_bar_layout();
        (btn_x, bar_y, btn_w, bar_h)
    }

    #[cfg(test)]
    pub fn toolbar_menu_button_rect_for_test(&self) -> (f32, f32, f32, f32) {
        self.toolbar_menu_button_rect()
    }

    #[cfg(test)]
    pub fn toolbar_theme_button_rect_for_test_full(&self) -> (f32, f32, f32, f32) {
        self.toolbar_theme_button_rect()
    }

    #[cfg(test)]
    pub fn toolbar_download_button_rect_for_test(&self) -> (f32, f32, f32, f32) {
        self.toolbar_download_button_rect()
    }

    #[cfg(test)]
    pub fn address_bar_layout_for_test(&self) -> (f32, f32, f32, f32) {
        self.address_bar_layout()
    }

    fn toolbar_menu_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        let (bx, by, bw, bh) = self.toolbar_menu_button_rect();
        x_f >= bx && x_f <= bx + bw && y_f >= by && y_f <= by + bh
    }

    fn address_bar_trailing_slot_hit_test(&self, x_f: f32, y_f: f32, slot_index: u32) -> bool {
        let (slot_x, slot_y, slot_w, slot_h) = self.address_bar_trailing_slot_rect(slot_index);
        x_f >= slot_x && x_f < slot_x + slot_w && y_f >= slot_y && y_f < slot_y + slot_h
    }

    fn address_bar_trailing_slot_rect(&self, slot_index: u32) -> (f32, f32, f32, f32) {
        let s = self.scale_factor;
        let (bar_x, bar_y, bar_w, bar_h) = self.address_bar_layout();
        let border = s.max(1.0);
        let inner_x = bar_x + border;
        let inner_y = bar_y + border;
        let inner_w = bar_w - 2.0 * border;
        let inner_h = bar_h - 2.0 * border;
        let trailing_slots_w = layout::ADDRESS_BAR_TRAILING_SLOTS * s;
        let slots_x = inner_x + inner_w - layout::ADDRESS_BAR_TRAILING_PAD * s - trailing_slots_w;
        let slot_w = layout::ADDRESS_BAR_ACTION_SLOT_WIDTH * s;
        let slot_x = slots_x + slot_index as f32 * slot_w;
        (slot_x, inner_y, slot_w, inner_h)
    }

    fn address_bar_bookmark_hit_test(&self, x_f: f32, y_f: f32) -> bool {
        self.address_bar_trailing_slot_hit_test(x_f, y_f, 0)
    }

    fn show_browser_menu(&mut self) {
        let s = self.scale_factor;
        let (bx, by, bw, bh) = self.toolbar_menu_button_rect();
        let language = UiLanguage::detect_from_env();
        let bookmarks_bar_label = if self.shell.settings().show_bookmarks_bar {
            browser_menu_label(BrowserMenuLabel::HideBookmarksBar, language)
        } else {
            browser_menu_label(BrowserMenuLabel::ShowBookmarksBar, language)
        };

        // 历史子菜单：取最近 8 条历史记录，每条作为可点击项（id 编码为 history:<url>）。
        // 末尾追加"查看全部历史"与"清除历史"动作项。
        let history_children: Vec<MenuItem> = {
            let mut entries: Vec<_> = self.shell.history().iter().take(8).collect();
            entries.reverse(); // 最近访问展示在顶部
            let mut items: Vec<MenuItem> = if entries.is_empty() {
                let empty_label = match language {
                    UiLanguage::ZhCn => "无历史记录",
                    UiLanguage::EnUs => "No history",
                };
                vec![MenuItem::action_disabled("history_empty", empty_label)]
            } else {
                entries
                    .iter()
                    .map(|e| {
                        let title = e.title();
                        let label = if title.is_empty() { e.url() } else { title };
                        MenuItem::action(&format!("history:{}", e.url()), label)
                    })
                    .collect()
            };
            items.push(MenuItem::separator());
            items.push(MenuItem::action(
                "history_view_all",
                match language {
                    UiLanguage::ZhCn => "查看全部历史",
                    UiLanguage::EnUs => "View all history",
                },
            ));
            items.push(MenuItem::action(
                "history_clear_all",
                match language {
                    UiLanguage::ZhCn => "清除历史…",
                    UiLanguage::EnUs => "Clear history…",
                },
            ));
            items
        };

        // 书签子菜单：列出最近添加的书签（id 编码为 bookmark:<url>），
        // 末尾追加"管理书签"与"添加书签"动作项。
        let bookmarks_children: Vec<MenuItem> = {
            let mut bm_entries: Vec<_> = self.shell.bookmarks().iter().take(8).collect();
            bm_entries.reverse();
            let mut items: Vec<MenuItem> = if bm_entries.is_empty() {
                let empty_label = match language {
                    UiLanguage::ZhCn => "无书签",
                    UiLanguage::EnUs => "No bookmarks",
                };
                vec![MenuItem::action_disabled("bookmarks_empty", empty_label)]
            } else {
                bm_entries
                    .iter()
                    .map(|b| {
                        let label = if b.title().is_empty() { b.url() } else { b.title() };
                        MenuItem::action(&format!("bookmark:{}", b.url()), label)
                    })
                    .collect()
            };
            items.push(MenuItem::separator());
            items.push(MenuItem::action(
                "browser_menu_bookmarks",
                match language {
                    UiLanguage::ZhCn => "管理书签",
                    UiLanguage::EnUs => "Manage bookmarks",
                },
            ));
            items.push(MenuItem::action(
                "browser_menu_add_bookmark",
                match language {
                    UiLanguage::ZhCn => "为此标签页添加书签",
                    UiLanguage::EnUs => "Bookmark this tab",
                },
            ));
            items
        };

        // 下载子菜单：列出最近下载条目（点击打开下载页），末尾追加动作项。
        let downloads_children: Vec<MenuItem> = {
            let dl_entries: Vec<_> = self.shell.downloads().iter().take(5).collect();
            let (empty_label, clear_label) = match language {
                UiLanguage::ZhCn => ("无下载", "清空已完成下载"),
                UiLanguage::EnUs => ("No downloads", "Clear finished downloads"),
            };
            let mut items: Vec<MenuItem> = if dl_entries.is_empty() {
                vec![MenuItem::action_disabled("downloads_empty", empty_label)]
            } else {
                dl_entries
                    .iter()
                    .map(|d| MenuItem::action("browser_menu_downloads", d.filename()))
                    .collect()
            };
            items.push(MenuItem::separator());
            items.push(MenuItem::action(
                "downloads_view_all",
                match language {
                    UiLanguage::ZhCn => "查看全部下载",
                    UiLanguage::EnUs => "View all downloads",
                },
            ));
            items.push(MenuItem::action("downloads_clear_completed", clear_label));
            items
        };

        self.context_menu = ContextMenuState {
            visible: true,
            context_type: ContextType::Page,
            items: vec![
                MenuItem::action("browser_menu_new_tab", browser_menu_label(BrowserMenuLabel::NewTab, language)),
                MenuItem::action("browser_menu_new_private_tab", browser_menu_label(BrowserMenuLabel::NewPrivateTab, language)),
                MenuItem::separator(),
                MenuItem::sub_menu(
                    "browser_menu_history",
                    browser_menu_label(BrowserMenuLabel::History, language),
                    history_children,
                ),
                MenuItem::sub_menu(
                    "browser_menu_downloads",
                    browser_menu_label(BrowserMenuLabel::Downloads, language),
                    downloads_children,
                ),
                MenuItem::sub_menu(
                    "browser_menu_bookmarks",
                    browser_menu_label(BrowserMenuLabel::BookmarksManager, language),
                    bookmarks_children,
                ),
                MenuItem::separator(),
                MenuItem::action("browser_menu_toggle_bookmarks_bar", bookmarks_bar_label),
                MenuItem::separator(),
                MenuItem::action("browser_menu_about", browser_menu_label(BrowserMenuLabel::AboutBrowser, language)),
                MenuItem::separator(),
                MenuItem::action("browser_menu_settings", browser_menu_label(BrowserMenuLabel::Settings, language)),
            ],
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: bx + bw - layout::CONTEXT_MENU_WIDTH * s,
            y: by + bh + 4.0 * s,
            source_tab_id: self.shell.active_tab_id(),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
        };
        self.context_menu_suppress_left_up = true;
        self.needs_redraw = true;
    }

    fn show_site_permissions_menu(&mut self) {
        let s = self.scale_factor;
        let (slot_x, slot_y, slot_w, slot_h) = self.address_bar_trailing_slot_rect(1);
        let language = UiLanguage::detect_from_env();

        // 取当前标签页 URL → Origin，查询真实权限状态。
        let current_url: Option<String> = self
            .shell
            .active_tab()
            .and_then(|t| t.url())
            .map(|u| u.to_string());
        let permissions: Vec<(zero_security::permission::PermissionName, zero_security::permission::PermissionState)> =
            if let Some(url_str) = current_url.as_deref() {
                match zero_security::origin::Origin::parse(url_str) {
                    Ok(origin) => self.permissions.get_all_for_origin(&origin),
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            };

        let items: Vec<MenuItem> = if permissions.is_empty() {
            vec![MenuItem::action_disabled(
                "no_permissions",
                match language {
                    UiLanguage::ZhCn => "此站点尚未请求任何权限",
                    UiLanguage::EnUs => "This site has not requested any permissions",
                },
            )]
        } else {
            permissions
                .iter()
                .map(|(name, state)| {
                    let label = permission_label(name, language);
                    let state_str = match (state, language) {
                        (zero_security::permission::PermissionState::Granted, UiLanguage::ZhCn) => "已允许",
                        (zero_security::permission::PermissionState::Granted, UiLanguage::EnUs) => "Allowed",
                        (zero_security::permission::PermissionState::Denied, UiLanguage::ZhCn) => "已阻止",
                        (zero_security::permission::PermissionState::Denied, UiLanguage::EnUs) => "Blocked",
                        (zero_security::permission::PermissionState::Prompt, UiLanguage::ZhCn) => "每次询问",
                        (zero_security::permission::PermissionState::Prompt, UiLanguage::EnUs) => "Ask",
                    };
                    MenuItem::action_disabled("permission_entry", &format!("{label}：{state_str}"))
                })
                .collect()
        };

        self.context_menu = ContextMenuState {
            visible: true,
            context_type: ContextType::Page,
            items,
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: slot_x + slot_w - layout::CONTEXT_MENU_WIDTH * s,
            y: slot_y + slot_h + 4.0 * s,
            source_tab_id: self.shell.active_tab_id(),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
        };
        self.context_menu_suppress_left_up = true;
        self.needs_redraw = true;
    }

    fn tab_hit_test(&self, x_f: f32, y_f: f32) -> Option<TabId> {
        let s = self.scale_factor;
        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
        if y_f < tab_y || y_f >= tab_strip_h {
            return None;
        }
        for &(id, tab_x, tab_w) in &self.tab_layout {
            if x_f >= tab_x && x_f < tab_x + tab_w {
                return Some(id);
            }
        }
        None
    }

    fn show_tab_context_menu(&mut self, tab_id: TabId, x: f64, y: f64) {
        let language = UiLanguage::detect_from_env();
        let pinned = self.shell.tab(tab_id).is_some_and(|t| t.is_pinned());
        let muted = self.shell.tab(tab_id).is_some_and(|t| t.is_muted());
        let pin_label = if pinned {
            tab_menu_label(TabMenuLabel::Unpin, language)
        } else {
            tab_menu_label(TabMenuLabel::Pin, language)
        };
        let mute_label = if muted {
            tab_menu_label(TabMenuLabel::Unmute, language)
        } else {
            tab_menu_label(TabMenuLabel::Mute, language)
        };
        self.context_menu = ContextMenuState {
            visible: true,
            context_type: ContextType::Page,
            items: vec![
                MenuItem::action("tab_reload", tab_menu_label(TabMenuLabel::Reload, language)),
                MenuItem::action("tab_pin", pin_label),
                MenuItem::action("tab_mute", mute_label),
                MenuItem::separator(),
                MenuItem::action("tab_duplicate", tab_menu_label(TabMenuLabel::Duplicate, language)),
                MenuItem::separator(),
                MenuItem::action("tab_close", tab_menu_label(TabMenuLabel::Close, language)),
                MenuItem::action("tab_close_others", tab_menu_label(TabMenuLabel::CloseOthers, language)),
                MenuItem::action("tab_close_to_right", tab_menu_label(TabMenuLabel::CloseToRight, language)),
            ],
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: x as f32,
            y: y as f32,
            source_tab_id: Some(tab_id),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
        };
        self.needs_redraw = true;
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
        let text_x = self.address_bar_text_origin_x();
        let rel_x = (x as f32 - text_x).max(0.0);
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
        let tab_y = layout::TAB_BAR_TOP_INSET * s;
        let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;

        if y_f >= tab_y && y_f < tab_strip_h {
            if let Some(tab_id) = self.tab_hit_test(x_f, y_f) {
                self.show_tab_context_menu(tab_id, x, y);
            }
            return;
        }

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
        let items: Vec<MenuItem> = menu.items().to_vec();

        let (page_doc_x, page_doc_y) = if context_type == ContextType::Page
            || context_type == ContextType::Link
            || context_type == ContextType::Selection
        {
            self.page_doc_point(x_f, y_f)
                .map(|(_, dx, dy)| (dx, dy))
                .unwrap_or((0.0, 0.0))
        } else {
            (0.0, 0.0)
        };

        self.context_menu = ContextMenuState {
            visible: true,
            context_type,
            items,
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: x as f32,
            y: y as f32,
            source_tab_id: self.shell.active_tab_id(),
            page_doc_x,
            page_doc_y,
        };
        self.needs_redraw = true;
    }

    /// 激活上下文菜单中选中的项
    fn activate_context_menu_item(&mut self) {
        let idx = match self.context_menu.hovered_index {
            Some(i) => i,
            None => return,
        };

        let item_id = match self.context_menu.items.get(idx) {
            Some(item) if item.enabled() && !item.is_separator() && !item.is_sub_menu() => item.id().to_string(),
            _ => return,
        };

        let source_tab_id = self.context_menu.source_tab_id;
        let page_doc_x = self.context_menu.page_doc_x;
        let page_doc_y = self.context_menu.page_doc_y;
        let context_type = self.context_menu.context_type;

        self.context_menu.close();
        self.needs_redraw = true;

        match item_id.as_str() {
            "back" => self.go_back(),
            "forward" => self.go_forward(),
            "reload" => self.refresh_page(),
            "browser_menu_new_tab" => self.new_tab(None),
            "browser_menu_new_private_tab" => self.new_private_tab(None),
            "browser_menu_add_bookmark" => {
                let was_visible = self.bookmarks_bar_visible();
                self.shell.add_bookmark();
                if self.bookmarks_bar_visible() != was_visible {
                    self.sync_webview_viewport();
                }
            }
            "browser_menu_toggle_bookmarks_bar" => {
                let was_visible = self.bookmarks_bar_visible();
                let show = !self.shell.settings().show_bookmarks_bar;
                self.shell.apply_settings(|settings| settings.show_bookmarks_bar = show);
                if self.bookmarks_bar_visible() != was_visible {
                    self.sync_webview_viewport();
                }
            }
            "browser_menu_about" => {
                let html = pages::generate_about_browser_html();
                self.open_internal_document_tab(html, "zero://about", "About ZeroBrowser");
            }
            "browser_menu_settings" => self.open_settings_page(),
            "browser_menu_history" => self.open_history_page(),
            "browser_menu_downloads" => {
                self.download_panel_open = true;
                self.open_downloads_page();
            }
            "browser_menu_bookmarks" => self.open_bookmarks_page(),
            "tab_reload" => {
                if let Some(tab_id) = source_tab_id {
                    if self.shell.active_tab_id() != Some(tab_id) {
                        self.shell.switch_tab(tab_id);
                        self.tabs.on_active_tab_changed(self.shell.active_tab_id());
                        self.update_address_bar_from_active_tab();
                    }
                    self.refresh_page();
                }
            }
            "tab_pin" => {
                if let Some(tab_id) = source_tab_id {
                    let pinned = self.shell.tab(tab_id).is_some_and(|t| t.is_pinned());
                    self.shell.set_tab_pinned(tab_id, !pinned);
                }
            }
            "tab_mute" => {
                if let Some(tab_id) = source_tab_id {
                    let muted = self.shell.tab(tab_id).is_some_and(|t| t.is_muted());
                    self.shell.set_tab_muted(tab_id, !muted);
                }
            }
            "tab_close" => {
                if let Some(tab_id) = source_tab_id {
                    self.close_tab_by_id(tab_id);
                }
            }
            "tab_duplicate" => {
                if let Some(tab_id) = source_tab_id {
                    self.duplicate_tab_by_id(tab_id);
                }
            }
            "tab_close_others" => {
                if let Some(tab_id) = source_tab_id {
                    self.close_other_tabs_by_id(tab_id);
                }
            }
            "tab_close_to_right" => {
                if let Some(tab_id) = source_tab_id {
                    self.close_tabs_to_right_by_id(tab_id);
                }
            }
            "view_source" => {
                if let Some(tab_id) = source_tab_id {
                    self.view_page_source(tab_id);
                }
            }
            "inspect" => {
                if let Some(tab_id) = source_tab_id {
                    self.inspect_element_at(tab_id, page_doc_x, page_doc_y);
                }
            }
            "copy" => {
                if context_type == ContextType::Editable {
                    let _ = self.address_bar.copy_selection();
                } else {
                    let _ = self.copy_page_selection();
                }
            }
            "cut" if self.address_bar.cut_selection() => {
                self.update_autocomplete();
            }
            "paste" if self.address_bar.paste_from_clipboard() => {
                self.update_autocomplete();
            }
            "select_all" => {
                self.address_bar.select_all();
            }
            _ => {}
        }
        self.needs_redraw = true;
    }

    /// 上下文菜单命中检测
    /// 菜单项是否可激活（非分隔线、非 disabled）。键盘导航和点击共用。
    fn context_menu_menu_item_activatable(&self, idx: usize) -> bool {
        self.context_menu
            .items
            .get(idx)
            .is_some_and(|item| !item.is_separator() && item.enabled())
    }

    /// 键盘上下移动时，根据 hovered_index 同步子菜单展开状态：
    /// hover 到 sub_menu 项 → 展开；hover 到普通项 → 收起。
    fn sync_open_sub_menu_with_hover(&mut self) {
        let new_open = self.context_menu.hovered_index.and_then(|i| {
            let item = self.context_menu.items.get(i)?;
            if item.is_sub_menu() && item.enabled() { Some(i) } else { None }
        });
        if new_open != self.context_menu.open_sub_menu {
            self.context_menu.open_sub_menu = new_open;
            self.context_menu.sub_menu_hovered = None;
        }
    }

    fn context_menu_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        if !self.context_menu.visible {
            return None;
        }

        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;

        let x_f = x as f32;
        let y_f = y as f32;

        if x_f < menu_x || x_f > menu_x + menu_w || y_f < menu_y || y_f > menu_y + menu_h {
            return None;
        }

        let idx = ((y_f - menu_y) / row_h) as usize;
        if let Some(item) = self.context_menu.items.get(idx)
            && item.enabled()
            && !item.is_separator()
        {
            Some(idx)
        } else {
            None
        }
    }

    /// 子菜单面板命中检测，返回命中的子项索引。
    fn sub_menu_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let parent_idx = self.context_menu.open_sub_menu?;
        let parent = self.context_menu.items.get(parent_idx)?;
        let children = parent.children()?;
        if children.is_empty() {
            return None;
        }

        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        // 子面板紧贴主菜单右侧（与 render_context_menu 中一致）。
        let sub_x = menu_x + menu_w + 1.0 * s;
        let sub_y = menu_y + parent_idx as f32 * row_h;
        let sub_h = children.len() as f32 * row_h;

        let x_f = x as f32;
        let y_f = y as f32;
        if x_f < sub_x || x_f > sub_x + menu_w || y_f < sub_y || y_f > sub_y + sub_h {
            return None;
        }
        let ci = ((y_f - sub_y) / row_h) as usize;
        if let Some(child) = children.get(ci)
            && child.enabled()
            && !child.is_separator()
        {
            Some(ci)
        } else {
            None
        }
    }

    /// 激活子菜单中当前 hover 的子项。
    fn activate_sub_menu_item(&mut self) {
        let Some(parent_idx) = self.context_menu.open_sub_menu else {
            return;
        };
        let Some(ci) = self.context_menu.sub_menu_hovered else {
            return;
        };
        // 取出子项 id 后立即关闭整个菜单（避免持有 items 借用）。
        let item_id = self
            .context_menu
            .items
            .get(parent_idx)
            .and_then(|p| p.children())
            .and_then(|chs| chs.get(ci))
            .filter(|c| c.enabled() && !c.is_separator())
            .map(|c| c.id().to_string());
        let Some(item_id) = item_id else {
            return;
        };
        self.context_menu.close();
        self.needs_redraw = true;

        // 子菜单项 id 派发
        if let Some(url) = item_id.strip_prefix("history:") {
            self.navigate_to(url);
            return;
        }
        if let Some(url) = item_id.strip_prefix("bookmark:") {
            self.navigate_to(url);
            return;
        }
        match item_id.as_str() {
            "history_view_all" => self.open_history_page(),
            "history_clear_all" => {
                self.shell.history_mut().clear();
            }
            "downloads_view_all" => {
                self.download_panel_open = true;
                self.open_downloads_page();
            }
            "downloads_clear_completed" => {
                self.shell.downloads_mut().clear_completed();
            }
            "browser_menu_downloads" => {
                self.download_panel_open = true;
                self.open_downloads_page();
            }
            "browser_menu_bookmarks" => self.open_bookmarks_page(),
            "browser_menu_add_bookmark" => {
                let was_visible = self.bookmarks_bar_visible();
                self.shell.add_bookmark();
                if self.bookmarks_bar_visible() != was_visible {
                    self.sync_webview_viewport();
                }
            }
            _ => {}
        }
    }

    /// 自动补全下拉命中检测（物理像素坐标）
    fn autocomplete_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let s = self.scale_factor;
        let (bar_x, _, bar_w, _) = self.address_bar_layout();

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

/// 权限名称的本地化标签（用于站点权限菜单展示）。
fn permission_label(
    name: &zero_security::permission::PermissionName,
    language: UiLanguage,
) -> &'static str {
    use zero_security::permission::PermissionName::*;
    match (name, language) {
        (Camera, UiLanguage::ZhCn) => "摄像头",
        (Camera, UiLanguage::EnUs) => "Camera",
        (Microphone, UiLanguage::ZhCn) => "麦克风",
        (Microphone, UiLanguage::EnUs) => "Microphone",
        (Geolocation, UiLanguage::ZhCn) => "位置信息",
        (Geolocation, UiLanguage::EnUs) => "Location",
        (Notifications, UiLanguage::ZhCn) => "通知",
        (Notifications, UiLanguage::EnUs) => "Notifications",
        (ClipboardRead, UiLanguage::ZhCn) => "剪贴板读取",
        (ClipboardRead, UiLanguage::EnUs) => "Clipboard read",
        (ClipboardWrite, UiLanguage::ZhCn) => "剪贴板写入",
        (ClipboardWrite, UiLanguage::EnUs) => "Clipboard write",
        (Fullscreen, UiLanguage::ZhCn) => "全屏",
        (Fullscreen, UiLanguage::EnUs) => "Fullscreen",
        (PointerLock, UiLanguage::ZhCn) => "鼠标指针锁定",
        (PointerLock, UiLanguage::EnUs) => "Pointer lock",
        (ScreenCapture, UiLanguage::ZhCn) => "屏幕录制",
        (ScreenCapture, UiLanguage::EnUs) => "Screen capture",
        (BackgroundSync, UiLanguage::ZhCn) => "后台同步",
        (BackgroundSync, UiLanguage::EnUs) => "Background sync",
        (PersistentStorage, UiLanguage::ZhCn) => "持久化存储",
        (PersistentStorage, UiLanguage::EnUs) => "Persistent storage",
    }
}
