// 浏览器输入处理（书签栏/标签拖拽/上下文菜单/鼠标 hover 等，从 app_input.rs 拆分）。
// 拆分目的：app_input.rs 单文件 ≤2000 行合规（DC-16）。经 app.rs include! 同模块作用域，零行为变化。

impl BrowserApp {
    /// 书签栏命中检测：返回点中的书签 (url, title)。复用于左键导航与右键菜单。
    fn bookmark_bar_item_at(&self, x: f32, s: f32) -> Option<(String, String)> {
        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        for bm in self.shell.bookmarks().list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;
            if x >= bx && x < bx + item_w {
                return Some((bm.url().to_string(), label.to_string()));
            }
            bx += item_w + 8.0 * s;
        }
        None
    }

    /// 处理书签栏点击
    fn handle_bookmark_bar_click(&mut self, x: f32, _y: f32, _bar_y: f32, _width: f32, s: f32) {
        if let Some((url, _)) = self.bookmark_bar_item_at(x, s) {
            // Ctrl/Cmd+点击 → 后台新标签打开（Chrome 行为）。
            if self.ctrl_pressed || self.cmd_pressed {
                self.new_tab_background(&url);
            } else {
                self.navigate_to(&url);
            }
        }
    }

    /// 显示书签上下文菜单（书签栏右键）。
    fn show_bookmark_context_menu(&mut self, url: String, title: String, x: f64, y: f64) {
        let language = UiLanguage::detect_from_env();
        let open_label = match language {
            UiLanguage::ZhCn => "打开",
            UiLanguage::EnUs => "Open",
        };
        let copy_label = match language {
            UiLanguage::ZhCn => "复制链接",
            UiLanguage::EnUs => "Copy link",
        };
        let delete_label = match language {
            UiLanguage::ZhCn => "删除",
            UiLanguage::EnUs => "Delete",
        };
        self.context_menu = ContextMenuState {
            visible: true,
            context_type: ContextType::Page,
            items: vec![
                MenuItem::action("bookmark_open", open_label),
                MenuItem::action("bookmark_copy_link", copy_label),
                MenuItem::separator(),
                MenuItem::action("bookmark_delete", delete_label),
            ],
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: x as f32,
            y: y as f32,
            source_tab_id: self.shell.active_tab_id(),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
            bookmark_url: Some(url),
            bookmark_title: Some(title),
            image_url: None,
            link_url: None,
        };
        self.needs_redraw = true;
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
        match self.page_selection_text() {
            Some(text) if !text.is_empty() => crate::clipboard::write_text(&text),
            _ => false,
        }
    }

    /// 取当前页面选中的文本（不写入剪贴板）。无选区返回 None。
    fn page_selection_text(&self) -> Option<String> {
        let tab_id = self.shell.active_tab_id()?;
        let sel = self.page_selection.get(&tab_id)?;
        if sel.is_collapsed() {
            return None;
        }
        let glyphs = self.page_glyphs(tab_id)?;
        let text = GlyphSelection::selected_text(&glyphs, sel);
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
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
            )
            .with_shortcut(format!("{}H", mod_prefix())));
            items.push(MenuItem::action(
                "history_clear_all",
                match language {
                    UiLanguage::ZhCn => "清除历史…",
                    UiLanguage::EnUs => "Clear history…",
                },
            ));
            // 最近关闭的标签（id 编码为 closed_tab:<url>）。
            let closed: Vec<_> = self.shell.recently_closed().take(5).collect();
            if !closed.is_empty() {
                items.push(MenuItem::separator());
                let header_label = match language {
                    UiLanguage::ZhCn => "最近关闭的标签",
                    UiLanguage::EnUs => "Recently closed",
                };
                items.push(MenuItem::action_disabled("closed_header", header_label));
                for c in closed {
                    let url = c.url.as_deref().unwrap_or("");
                    if url.is_empty() {
                        continue;
                    }
                    let label = c.title.as_deref().filter(|t| !t.is_empty()).unwrap_or(url);
                    items.push(MenuItem::action(&format!("closed_tab:{url}"), label));
                }
            }
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
            )
            .with_shortcut(format!("{}D", mod_prefix())));
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
            )
            .with_shortcut(format!("{}J", mod_prefix())));
            items.push(MenuItem::action("downloads_clear_completed", clear_label));
            items
        };

        self.context_menu = ContextMenuState {
            visible: true,
            context_type: ContextType::Page,
            items: vec![
                MenuItem::action("browser_menu_new_tab", browser_menu_label(BrowserMenuLabel::NewTab, language))
                    .with_shortcut(format!("{}T", mod_prefix())),
                MenuItem::action("browser_menu_new_private_tab", browser_menu_label(BrowserMenuLabel::NewPrivateTab, language))
                    .with_shortcut(format!("{}Shift+N", mod_prefix())),
                MenuItem::separator(),
                MenuItem::action(
                    "browser_menu_reload_all",
                    match language {
                        UiLanguage::ZhCn => "重新加载所有标签",
                        UiLanguage::EnUs => "Reload all tabs",
                    },
                ),
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
                MenuItem::action("browser_menu_toggle_bookmarks_bar", bookmarks_bar_label)
                    .with_shortcut(format!("{}Shift+B", mod_prefix())),
                MenuItem::separator(),
                MenuItem::action("browser_menu_about", browser_menu_label(BrowserMenuLabel::AboutBrowser, language)),
                MenuItem::separator(),
                MenuItem::action("browser_menu_settings", browser_menu_label(BrowserMenuLabel::Settings, language))
                    .with_shortcut(format!("{},", mod_prefix())),
            ],
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: bx + bw - layout::CONTEXT_MENU_WIDTH * s,
            y: by + bh + 4.0 * s,
            source_tab_id: self.shell.active_tab_id(),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
            bookmark_url: None,
            bookmark_title: None,
            image_url: None,
            link_url: None,
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
            bookmark_url: None,
            bookmark_title: None,
            image_url: None,
            link_url: None,
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
        // close_others：仅当多于 1 个标签时可用。
        // close_to_right：仅当目标标签右侧还有其他标签时可用。
        let tab_count = self.shell.tab_count();
        let can_close_others = tab_count > 1;
        // 收集所有 tab id，判断目标是否是最后一个。
        let all_ids: Vec<TabId> = self.shell.tabs().map(|t| t.id()).collect();
        let can_close_to_right = all_ids
            .iter()
            .position(|&id| id == tab_id)
            .is_some_and(|idx| idx + 1 < all_ids.len());
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
                MenuItem::action("tab_reload", tab_menu_label(TabMenuLabel::Reload, language))
                    .with_shortcut(format!("{}R", mod_prefix())),
                MenuItem::action("tab_pin", pin_label),
                MenuItem::action("tab_mute", mute_label),
                MenuItem::separator(),
                MenuItem::action("tab_duplicate", tab_menu_label(TabMenuLabel::Duplicate, language))
                    .with_shortcut(format!("{}Shift+D", mod_prefix())),
                MenuItem::action(
                    "tab_copy_url",
                    match language {
                        UiLanguage::ZhCn => "复制标签 URL",
                        UiLanguage::EnUs => "Copy tab URL",
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("tab_close", tab_menu_label(TabMenuLabel::Close, language))
                    .with_shortcut(format!("{}W", mod_prefix())),
                {
                    let mut item = MenuItem::action(
                        "tab_close_others",
                        tab_menu_label(TabMenuLabel::CloseOthers, language),
                    );
                    item.set_enabled(can_close_others);
                    item
                },
                {
                    let mut item = MenuItem::action(
                        "tab_close_to_right",
                        tab_menu_label(TabMenuLabel::CloseToRight, language),
                    );
                    item.set_enabled(can_close_to_right);
                    item
                },
            ],
            hovered_index: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: x as f32,
            y: y as f32,
            source_tab_id: Some(tab_id),
            page_doc_x: 0.0,
            page_doc_y: 0.0,
            bookmark_url: None,
            bookmark_title: None,
            image_url: None,
            link_url: None,
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
        // 未聚焦时单击：全选 URL（Chrome 行为）；已聚焦时定位光标（便于二次编辑）。
        if !self.address_bar_focused {
            self.address_bar.select_all();
        } else {
            self.address_bar.set_cursor(idx, extend);
            self.address_bar_drag = true;
        }
        self.address_bar_focused = true;
        self.autocomplete.clear();
        self.needs_redraw = true;
    }

    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let language = UiLanguage::detect_from_env();
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

        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        // 书签栏区域右键：弹出书签上下文菜单（打开 / 复制链接 / 删除）。
        if y_f >= toolbar_h && y_f < chrome_top {
            if let Some((url, title)) = self.bookmark_bar_item_at(x_f, s) {
                self.show_bookmark_context_menu(url, title, x, y);
            }
            return;
        }

        // 预先解析页面点击点的图片 src 与链接 href，用于上下文菜单判定。
        let (image_url, link_url): (Option<String>, Option<String>) =
            if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f) {
                let img = self.tabs.hit_test_image(tab_id, doc_x, doc_y);
                let lnk = self.tabs.hit_test_link(tab_id, doc_x, doc_y);
                (img, lnk)
            } else {
                (None, None)
            };

        let context_type = if self.address_bar_hit_test(x_f, y_f) {
            ContextType::Editable
        } else if y_f < chrome_top {
            return;
        } else if let Some(tab_id) = self.shell.active_tab_id()
            && self.page_selection.get(&tab_id).is_some_and(|sel| !sel.is_collapsed())
        {
            ContextType::Selection
        } else if image_url.is_some() {
            // 图片优先于链接（点中 img 时显示图片菜单，即使 img 在 a 内）。
            ContextType::Image
        } else if link_url.is_some() {
            ContextType::Link
        } else {
            ContextType::Page
        };

        let menu = ContextMenu::new(context_type);
        let mut items: Vec<MenuItem> = menu.items().to_vec();

        // 地址栏（Editable 场景）：根据当前选区/内容状态动态禁用编辑项，
        // 并移除地址栏不支持的 undo/redo（单行文本框无撤销栈）。
        if context_type == ContextType::Editable {
            let has_sel = self.address_bar.has_selection();
            let is_empty = self.address_bar.text().is_empty();
            items.retain(|it| !matches!(it.id(), "undo" | "redo"));
            for item in items.iter_mut() {
                match item.id() {
                    "cut" | "copy" if !has_sel => item.set_enabled(false),
                    "select_all" if is_empty => item.set_enabled(false),
                    _ => {}
                }
            }
            // 在 paste 后插入"粘贴并转到 / 粘贴并搜索"（剪贴板有内容时）。
            if let Some(clip) = crate::clipboard::read_text() {
                let clip = clip.trim().to_string();
                if !clip.is_empty() {
                    // 启发式判断：含 "." 且无空格视为 URL，否则搜索词。
                    let looks_like_url = clip.contains('.') && !clip.contains(' ');
                    let label = match (looks_like_url, language) {
                        (true, UiLanguage::ZhCn) => "粘贴并转到",
                        (true, UiLanguage::EnUs) => "Paste and go",
                        (false, UiLanguage::ZhCn) => "粘贴并搜索",
                        (false, UiLanguage::EnUs) => "Paste and search",
                    };
                    if let Some(idx) = items.iter().position(|it| it.id() == "paste") {
                        items.insert(idx + 1, MenuItem::action("paste_and_go", label));
                    }
                }
            }
        }

        // 根据当前页面能力禁用部分菜单项。
        // about:blank / 无 URL / zero:// 内部页 没有"源代码"或"可审查 DOM"概念，
        // 应禁用 view_source / inspect / save_as / print，避免无效操作。
        let active_url = self.shell.active_tab().and_then(|t| t.url());
        let page_inspectable = match active_url {
            None => false,
            Some(u) if u.is_empty() || u == "about:blank" => false,
            Some(u) if u.starts_with("zero://") => false,
            _ => true,
        };
        if !page_inspectable {
            for item in items.iter_mut() {
                if matches!(item.id(), "view_source" | "inspect" | "save_as" | "print") {
                    item.set_enabled(false);
                }
            }
        }

        // 需要文件对话框或图片原始数据的 action 暂未实现，统一禁用。
        for item in items.iter_mut() {
            if matches!(
                item.id(),
                "save_link" | "save_image" | "copy_image" | "print"
            ) {
                item.set_enabled(false);
            }
        }

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
            bookmark_url: None,
            bookmark_title: None,
            image_url,
            link_url,
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
        let bookmark_url = self.context_menu.bookmark_url.clone();
        let link_url = self.context_menu.link_url.clone();
        let image_url = self.context_menu.image_url.clone();

        self.context_menu.close();
        self.needs_redraw = true;

        match item_id.as_str() {
            "back" => self.go_back(),
            "forward" => self.go_forward(),
            "reload" => self.refresh_page(),
            "browser_menu_reload_all" => self.reload_all_tabs(),
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
            "tab_copy_url" => {
                if let Some(tab_id) = source_tab_id
                    && let Some(url) = self.shell.tab(tab_id).and_then(|t| t.url())
                {
                    crate::clipboard::write_text(url);
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
            // 粘贴剪贴板内容并立即导航（不保留聚焦态）。
            "paste_and_go" if self.address_bar.paste_from_clipboard() => {
                let text = self.address_bar.text().trim().to_string();
                if !text.is_empty() {
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                    self.navigate_to(&text);
                }
            }
            "select_all" => {
                self.address_bar.select_all();
            }
            // ── 链接右键菜单 ──
            "open_link" => {
                if let Some(href) = &link_url {
                    // 后台新标签打开链接（Chrome 默认行为）。
                    self.new_tab_background(href);
                }
            }
            "copy_link" => {
                if let Some(href) = &link_url {
                    crate::clipboard::write_text(href);
                }
            }
            "bookmark_link" => {
                if let Some(href) = &link_url {
                    self.shell.add_bookmark_with_url(href);
                    self.needs_redraw = true;
                }
            }
            // ── 图片右键菜单 ──
            "open_image" => {
                if let Some(src) = &image_url {
                    self.new_tab_background(src);
                }
            }
            "copy_image_url" => {
                if let Some(src) = &image_url {
                    crate::clipboard::write_text(src);
                }
            }
            // ── 选中文本右键菜单 ──
            "search_selection" => {
                // 用默认搜索引擎在新标签打开选中文本（前台聚焦）。
                if let Some(text) = self.page_selection_text() {
                    let search_url = self.shell.settings().search(&text);
                    self.new_tab(Some(&search_url));
                }
            }
            "bookmark_open" => {
                if let Some(url) = &bookmark_url {
                    self.navigate_to(url);
                }
            }
            "bookmark_copy_link" => {
                if let Some(url) = &bookmark_url {
                    crate::clipboard::write_text(url);
                }
            }
            "bookmark_delete" => {
                if let Some(url) = &bookmark_url {
                    self.shell.remove_bookmark_by_url(url);
                }
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
        let normal_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu_total_height();

        let x_f = x as f32;
        let y_f = y as f32;

        if x_f < menu_x || x_f > menu_x + menu_w || y_f < menu_y || y_f > menu_y + menu_h {
            return None;
        }

        // 累积行高定位：separator 用紧凑高度，普通项用 normal_h。
        let mut cur_y = menu_y;
        for (idx, item) in self.context_menu.items.iter().enumerate() {
            let h = if item.is_separator() { sep_h } else { normal_h };
            if y_f >= cur_y && y_f < cur_y + h {
                if item.enabled() && !item.is_separator() {
                    return Some(idx);
                } else {
                    return None;
                }
            }
            cur_y += h;
        }
        None
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
        let normal_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        // 子面板矩形由 sub_menu_panel_rect 决定，右侧空间不足时自动翻转到左侧。
        let (sub_x, sub_y, _sub_w, sub_h) = self.sub_menu_panel_rect(parent_idx);

        let x_f = x as f32;
        let y_f = y as f32;
        if x_f < sub_x || x_f > sub_x + layout::CONTEXT_MENU_WIDTH * s || y_f < sub_y || y_f > sub_y + sub_h {
            return None;
        }
        let mut cur_y = sub_y;
        for (ci, child) in children.iter().enumerate() {
            let h = if child.is_separator() { sep_h } else { normal_h };
            if y_f >= cur_y && y_f < cur_y + h {
                if child.enabled() && !child.is_separator() {
                    return Some(ci);
                } else {
                    return None;
                }
            }
            cur_y += h;
        }
        None
    }

    /// 判断点是否在「主菜单 ∪ 子菜单面板 ∪ 两者桥接区」内。
    /// 用于鼠标从主菜单项移向子菜单面板时保留子菜单展开，避免经过间隙时闪烁收起。
    fn point_in_sub_menu_bridge(&self, x: f64, y: f64) -> bool {
        let Some(parent_idx) = self.context_menu.open_sub_menu else {
            return false;
        };
        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu_total_height();
        let (sub_x, sub_y, _sub_w, sub_h) = self.sub_menu_panel_rect(parent_idx);
        let sub_w = layout::CONTEXT_MENU_WIDTH * s;

        let xf = x as f32;
        let yf = y as f32;

        // 主菜单矩形内
        if xf >= menu_x && xf <= menu_x + menu_w && yf >= menu_y && yf <= menu_y + menu_h {
            return true;
        }
        // 子菜单面板矩形内
        if xf >= sub_x && xf <= sub_x + sub_w && yf >= sub_y && yf <= sub_y + sub_h {
            return true;
        }
        // 桥接区：主菜单与子菜单面板之间的水平间隙，
        // y 范围取两者 y 区间的并集（宽松，方便斜向移动）
        let (left_x, right_x) = if sub_x >= menu_x + menu_w {
            (menu_x + menu_w, sub_x)
        } else {
            (sub_x + sub_w, menu_x)
        };
        let bridge_y_top = menu_y.min(sub_y);
        let bridge_y_bot = (menu_y + menu_h).max(sub_y + sub_h);
        if xf >= left_x && xf <= right_x && yf >= bridge_y_top && yf <= bridge_y_bot {
            return true;
        }
        false
    }

    /// 激活子菜单中当前 hover 的子项。
    fn activate_sub_menu_item(&mut self) {        let Some(parent_idx) = self.context_menu.open_sub_menu else {
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
        if let Some(url) = item_id.strip_prefix("closed_tab:") {
            self.shell.reopen_closed_by_url(url);
            self.needs_redraw = true;
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
