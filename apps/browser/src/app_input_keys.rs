// 键盘输入处理（handle_key 及 find/address_bar/global/extract_typed_text 等键处理）。
// 从 app_input.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs / app_render_address.rs 同模式；方法保留在 `impl BrowserApp { }` 内。

impl BrowserApp {
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
            "Alt" | "AltLeft" | "AltRight" => {
                self.alt_pressed = pressed;
                return;
            }
            _ => {}
        }

        // Alt+Left/Right：浏览器后退/前进（不转发给页面，地址栏聚焦时也不拦截）
        // Alt+Home：导航到主页
        if self.alt_pressed && pressed {
            match key {
                "ArrowLeft" => {
                    self.go_back();
                    self.needs_redraw = true;
                    return;
                }
                "ArrowRight" => {
                    self.go_forward();
                    self.needs_redraw = true;
                    return;
                }
                "Home" => {
                    let home = self.shell.settings().home_url.clone();
                    self.navigate_to(&home);
                    return;
                }
                _ => {}
            }
        }

        // F11 全局切换全屏（不依赖修饰键，地址栏聚焦时也生效，参考 Chrome）
        if key == "F11" && pressed {
            self.pending_window_chrome_action = Some(WindowChromeAction::ToggleFullscreen);
            return;
        }

        // F5 刷新，Ctrl+F5 强制刷新（绕过缓存）。
        if key == "F5" && pressed {
            if self.ctrl_pressed || self.cmd_pressed {
                self.refresh_page_bypass_cache();
            } else {
                self.refresh_page();
            }
            return;
        }

        // F3 查找下一个（复用上次查询）。Shift+F3 上一个。
        // 查找栏关闭时自动打开并用 last_find_query 查找。
        if key == "F3" && pressed {
            if !self.shell.find_state().is_active() && !self.last_find_query.is_empty() {
                self.find_input = self.last_find_query.clone();
                self.shell.find_start(&self.last_find_query.clone());
            } else if self.shell.find_state().is_active() {
                if self.shift_pressed {
                    self.shell.find_previous();
                } else {
                    self.shell.find_next();
                }
            }
            self.needs_redraw = true;
            return;
        }

        // Escape 取消正在进行的标签拖拽。
        if key == "Escape" && pressed && self.tab_drag.is_some() {
            self.tab_drag = None;
            self.needs_redraw = true;
            return;
        }

        // Escape 停止加载：仅当没有其他 Escape 上下文（菜单/查找栏/地址栏）打开，
        // 且当前活动标签正在加载时。优先级最低，放在各早返回之后。
        if key == "Escape"
            && pressed
            && !self.context_menu.visible
            && !self.shell.find_state().is_active()
            && !self.address_bar_focused
        {
            if let Some(tab) = self.shell.active_tab()
                && tab.is_loading()
            {
                self.stop_loading_page();
                return;
            }
        }

        if !self.address_bar_focused && key.len() == 1 && !self.ctrl_pressed && !self.cmd_pressed {
            if let Some(tab_id) = self.shell.active_tab_id() {
                let event = if pressed { "keydown" } else { "keyup" };
                self.tabs.dispatch_key_event(tab_id, event, key, key);
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
                    self.last_find_query = self.find_input.clone();
                }
                self.needs_redraw = true;
            }
            _ => {
                if let Some(inserted) = Self::extract_typed_text(key, text) {
                    self.find_input.push_str(&inserted);
                    self.shell.find_start(&self.find_input);
                    self.last_find_query = self.find_input.clone();
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
                    // Ctrl+Shift+T：恢复最近关闭的标签。
                    self.needs_redraw |= self.shell.reopen_last_closed_tab().is_some();
                }
                "n" | "N" if self.shift_pressed => {
                    // Ctrl+Shift+N：新建无痕标签页，聚焦地址栏。
                    self.new_blank_tab_focused(true);
                }
                "t" | "T" => {
                    // Ctrl+T：新建标签页，聚焦地址栏。
                    self.new_blank_tab_focused(false);
                }
                "w" | "W" if self.shift_pressed => {
                    // Ctrl+Shift+W：关闭整个窗口。
                    self.pending_window_chrome_action = Some(WindowChromeAction::Close);
                }
                "w" | "W" => {
                    self.close_active_tab();
                }
                // Ctrl+1~8 切换到对应索引标签，Ctrl+9 切换到最后标签。
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" => {
                    if let Ok(idx) = key.parse::<usize>() {
                        if self.shell.switch_to_index(idx.saturating_sub(1)) {
                            self.tabs.on_active_tab_changed(self.shell.active_tab_id());
                            self.update_address_bar_from_active_tab();
                            self.needs_redraw = true;
                        }
                    }
                }
                "9" if self.shell.switch_to_last() => {
                    self.tabs.on_active_tab_changed(self.shell.active_tab_id());
                    self.update_address_bar_from_active_tab();
                    self.needs_redraw = true;
                }
                "r" | "R" if self.shift_pressed => {
                    // Ctrl+Shift+R：强制刷新（绕过 HTTP 缓存）。
                    self.refresh_page_bypass_cache();
                }
                "r" | "R" => {
                    self.refresh_page();
                }
                "p" | "P" => {
                    // Ctrl+P：切换打印预览（@media print 重渲染；DC-12 / R1993）。
                    self.toggle_print_preview();
                }
                "f" | "F" => {
                    // Ctrl+F：切换查找栏。已打开则关闭，未打开则打开（空查询激活）。
                    if self.shell.find_state().is_active() {
                        self.find_input.clear();
                        self.shell.find_close();
                    } else {
                        self.find_input.clear();
                        self.shell.find_start("");
                    }
                    self.needs_redraw = true;
                }
                "g" | "G" => {
                    // Ctrl+G：查找下一个（查找栏打开时）。Shift+Ctrl+G 上一个。
                    let find_active = self.shell.find_state().is_active();
                    if find_active && self.shift_pressed {
                        self.shell.find_previous();
                        self.needs_redraw = true;
                    } else if find_active {
                        self.shell.find_next();
                        self.needs_redraw = true;
                    }
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
                    self.show_zoom_indicator();
                }
                "-" => {
                    self.shell.zoom_out();
                    self.show_zoom_indicator();
                }
                "0" => {
                    self.shell.zoom_reset();
                    self.show_zoom_indicator();
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

        // 键盘页面滚动（Chrome 行为），无修饰键且非地址栏/查找栏聚焦时生效：
        // - Space / PageDown → 向下滚动约一个视口（Shift+Space 反向）
        // - PageUp → 向上滚动
        // - ArrowDown/Up → 小步滚动（约 40px）
        let scroll_handled = match key {
            " " | "Space" => {
                let ratio = if self.shift_pressed { -0.85 } else { 0.85 };
                self.scroll_active_page_by_viewport_ratio(ratio);
                true
            }
            "PageDown" => {
                self.scroll_active_page_by_viewport_ratio(0.85);
                true
            }
            "PageUp" => {
                self.scroll_active_page_by_viewport_ratio(-0.85);
                true
            }
            "ArrowDown" => {
                self.scroll_active_page_by_px(40.0 * self.scale_factor);
                true
            }
            "ArrowUp" => {
                self.scroll_active_page_by_px(-40.0 * self.scale_factor);
                true
            }
            _ => false,
        };
        if scroll_handled {
            return;
        }

        // 无修饰键的全局快捷键（保留兼容无 Ctrl 的单键模式）
        match key {
            "l" => {
                self.address_bar_focused = true;
                self.needs_redraw = true;
            }
            "t" => {
                self.new_blank_tab_focused(false);
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
                // 纯 Home 键滚动到页面顶部（Chrome/标准行为）。
                self.scroll_active_page_to_top();
            }
            "End" => {
                self.scroll_active_page_to_bottom();
            }
            "f" => {
                self.find_input.clear();
                self.shell.find_close();
                self.needs_redraw = true;
            }
            "+" | "=" => {
                self.shell.zoom_in();
                self.show_zoom_indicator();
            }
            "-" => {
                self.shell.zoom_out();
                self.show_zoom_indicator();
            }
            "0" => {
                self.shell.zoom_reset();
                self.show_zoom_indicator();
            }
            "n" => {
                self.shell.find_next();
                self.find_input = self.shell.find_state().query().to_string();
                self.needs_redraw = true;
            }
            _ => {}
        }
    }
}
