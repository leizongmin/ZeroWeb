// 右键上下文菜单动作（show_context_menu / activate_context_menu_item /
// context_menu_menu_item_activatable）。
// 从 app_input.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs / app_render_address.rs 同模式；方法保留在 `impl BrowserApp { }` 内。

impl BrowserApp {
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
