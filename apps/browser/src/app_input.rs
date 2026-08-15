// 输入处理方法（键盘、鼠标、IME、自动补全、上下文菜单）
// 从 app.rs 拆分以控制 app.rs 体积

/// 平台主修饰键前缀（macOS: ⌘，其他: Ctrl+），用于菜单快捷键提示。
fn mod_prefix() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl+"
    }
}

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

        // Ctrl+滚轮 → 页面缩放（Chrome 行为）。delta_y < 0（向上滚）放大，> 0 缩小。
        if self.ctrl_pressed || self.cmd_pressed {
            if delta_y < 0.0 {
                self.shell.zoom_in();
            } else if delta_y > 0.0 {
                self.shell.zoom_out();
            }
            self.show_zoom_indicator();
            self.apply_page_zoom_change();
            return;
        }

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
            // 上下文菜单打开时：touch 走 tap 合成路径（点菜单项），
            // 但页面内容滚动不适用。
        }

        match touch.phase {
            TouchPhase::Started => {
                if self.point_in_page_content(touch.x, touch.y) {
                    // 页面内容区：单指滚动 + 长按候选（500ms 不动弹右键菜单）。
                    self.touch_scroll = Some((touch.id, touch.y));
                    self.touch_tap_candidate = None;
                    self.touch_long_press = Some((touch.id, touch.x, touch.y, Instant::now()));
                } else {
                    // chrome UI 区：记录 tap 候选，Ended 时判定。
                    self.touch_tap_candidate = Some((touch.id, touch.x, touch.y));
                    self.touch_long_press = None;
                }
            }
            TouchPhase::Moved => {
                // 页面内容滚动
                if let Some((id, last_y)) = self.touch_scroll {
                    if id == touch.id {
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
                }
                // 长按候选：移动超过阈值则取消（判定为滚动意图）。
                if let Some((id, sx, sy, _)) = self.touch_long_press {
                    if id == touch.id && ((touch.x - sx).abs() > 8.0 || (touch.y - sy).abs() > 8.0) {
                        self.touch_long_press = None;
                    }
                }
                // chrome 区 tap 候选：移动超过阈值则取消（判定为非 tap 意图）。
                if let Some((id, sx, sy)) = self.touch_tap_candidate {
                    if id == touch.id && ((touch.x - sx).abs() > 10.0 || (touch.y - sy).abs() > 10.0) {
                        self.touch_tap_candidate = None;
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if self.touch_scroll.is_some_and(|(id, _)| id == touch.id) {
                    self.touch_scroll = None;
                }
                // 长按候选在 Ended 时清除（若已触发则在 poll 中清空，此处兜底）。
                if self.touch_long_press.is_some_and(|(id, _, _, _)| id == touch.id) {
                    self.touch_long_press = None;
                }
                // chrome 区 tap 完成：合成左键 press + release。
                if let Some((id, _sx, _sy)) = self.touch_tap_candidate {
                    if id == touch.id {
                        self.touch_tap_candidate = None;
                        self.handle_mouse_click(touch.x, touch.y, true, "Left");
                        self.handle_mouse_click(touch.x, touch.y, false, "Left");
                    }
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
        self.show_scrollbar_overlay();

        // R3293（S0）：派发用户滚动 'scroll' 事件到页面 JS（闭合 R3253 主路径不可达 gap）。
        // 主线程已完成视觉滚动（entry.x/y 更新 + 重绘），本调用仅补「页面 JS 可观察」半边——
        // infinite scroll / lazy load / sticky nav / parallax 的用户滚动触发依赖。kill-switch
        // `ZW_USER_SCROLL_EVENT=0` 关闭（default-on）。fire-and-forget，无回执不阻塞 UI。
        //
        // R3298（S1）：附带滚轮发生处的视口光标坐标（相对 WebView 内容区原点），供 renderer S2/S4
        // 命中可滚动祖先容器（元素级滚动视觉依赖 S3 layout 几何，渲染流域协调点，未实现）。
        // 光标取 `self.mouse_pos`（各滚动入口——滚轮/触摸/平移/滚动条拖拽——均已更新），减去
        // 内容区原点得内容相对视口坐标。
        if std::env::var("ZW_USER_SCROLL_EVENT").as_deref() != Ok("0") {
            let (content_x, content_y, _, _) = self.page_content_rect();
            let (mx, my) = self.mouse_pos;
            self.tabs.dispatch_user_scroll(
                tab_id,
                delta_x,
                delta_y,
                (mx as f32 - content_x).max(0.0),
                (my as f32 - content_y).max(0.0),
            );
        }

        if crate::compositor_client::async_scroll_enabled() {
            if let Some(submission) = self
                .tabs
                .snapshot(tab_id)
                .and_then(|snap| snap.compositor_submission)
            {
                let scroll = self.tab_scroll_state(tab_id);
                crate::compositor_client::set_scroll(submission.surface_id, scroll.x, scroll.y);
            }
        }

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
                    page_scroll::vertical_track_len(&layout, ch, self.scale_factor),
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
                    page_scroll::horizontal_track_len(&layout, cw, self.scale_factor),
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


    /// 按像素量滚动当前活动标签页（正值向下）。
    fn scroll_active_page_by_px(&mut self, dy: f32) {
        if dy == 0.0 {
            return;
        }
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.apply_page_scroll_delta(tab_id, 0.0, dy);
        }
    }

    /// 按当前内容区高度的比例滚动当前活动标签页（Space/PageDown 用）。
    fn scroll_active_page_by_viewport_ratio(&mut self, ratio: f32) {
        let (_, _, _, ch) = self.page_content_rect();
        self.scroll_active_page_by_px(ch * ratio);
    }

    /// R3254-M9：滚动键的延迟滚动幅度（CSS 像素，正 = 向下）；非滚动键返回 None。
    /// 与 keydown 派发同时注册（挂回执），替代原先的立即滚动。
    fn scroll_delta_for_key(&self, key: &str) -> Option<f32> {
        match key {
            " " | "Space" => {
                let ratio = if self.shift_pressed { -0.85 } else { 0.85 };
                let (_, _, _, ch) = self.page_content_rect();
                Some(ch * ratio)
            }
            "PageDown" => {
                let (_, _, _, ch) = self.page_content_rect();
                Some(ch * 0.85)
            }
            "PageUp" => {
                let (_, _, _, ch) = self.page_content_rect();
                Some(-ch * 0.85)
            }
            // 方向键步长 40 CSS px（含页面 zoom 换算）。
            "ArrowDown" => Some(40.0 * self.page_render_scale()),
            "ArrowUp" => Some(-40.0 * self.page_render_scale()),
            _ => None,
        }
    }

    /// 滚动活动标签页到页面顶部（Home 键）。
    fn scroll_active_page_to_top(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            let current = self.tab_scroll_state(tab_id);
            self.apply_page_scroll_delta(tab_id, -current.x, -current.y);
        }
    }

    /// 滚动活动标签页到页面底部（End 键）。
    fn scroll_active_page_to_bottom(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            let layout = self.page_scroll_layout(tab_id);
            let current = self.tab_scroll_state(tab_id);
            self.apply_page_scroll_delta(tab_id, layout.max_scroll_x - current.x, layout.max_scroll_y - current.y);
        }
    }

    /// 循环切换活跃标签（`reverse=true` 向前，否则向后）。
    fn cycle_active_tab(&mut self, reverse: bool) {        let active_id = match self.shell.active_tab_id() {
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
        // zoom 按标签存储——切换后按新活动标签重推 CSS 视口。
        self.sync_webview_viewport();
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
                // 鼠标不在主菜单项上、但有子菜单展开时，判断是否在桥接区
                // （主菜单与子菜单面板之间）。若是则视为"仍在前往子菜单的路上"，
                // 保留当前展开状态，不触发 hover 重算导致子菜单收起。
                let in_bridge = hovered.is_none()
                    && self.context_menu.open_sub_menu.is_some()
                    && self.point_in_sub_menu_bridge(x, y);
                if !in_bridge && hovered != self.context_menu.hovered_index {
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
                } else if !in_bridge
                    && hovered.is_none()
                    && self.context_menu.open_sub_menu.is_some()
                {
                    // 鼠标移出主菜单且不在桥接区：收起子菜单
                    self.context_menu.open_sub_menu = None;
                    self.context_menu.sub_menu_hovered = None;
                    self.needs_redraw = true;
                }
            }
        }

        // 标签拖拽：左键按住时移动超过阈值即激活拖拽，实时更新 current_x。
        if let Some(drag) = self.tab_drag.as_mut() {
            drag.current_x = x as f32;
            // 阈值 4 物理像素，避免普通点击被误判为拖拽。
            if !drag.active && (drag.current_x - drag.press_x).abs() > 4.0 {
                drag.active = true;
            }
            if drag.active {
                self.needs_redraw = true;
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
            && let Some(idx) = hit_test_caret(&glyphs, doc_x, doc_y)
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

    /// 标签拖拽释放：按鼠标 x 计算目标 index 并调用 move_tab 重排序。
    fn finish_tab_drag(&mut self, drag: &crate::app::TabDragState) {
        // 用当前 tab_layout 计算目标 index。
        // 鼠标 x 落在哪个 tab 中点之前，就插入到那个 tab 的位置。
        let mut target_index: Option<usize> = None;
        for (i, &(_id, tx, tw)) in self.tab_layout.iter().enumerate() {
            let mid = tx + tw * 0.5;
            if drag.current_x < mid {
                target_index = Some(i);
                break;
            }
        }
        let target_index = target_index.unwrap_or(self.tab_layout.len().saturating_sub(1));
        self.shell.move_tab(drag.tab_id, target_index);
        self.needs_redraw = true;
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
                let pointer_drag = self.content_pointer_drag.take();
                let was_scroll_drag = pointer_drag.as_ref().is_some_and(|d| d.scrolling);
                let moved_beyond_click_threshold = pointer_drag.as_ref().is_some_and(|d| {
                    let dx = x - d.start_x;
                    let dy = y - d.start_y;
                    dx.hypot(dy) >= self.content_scroll_drag_threshold()
                });
                self.tab_bar_drag_press = None;
                self.address_bar_drag = false;
                // 标签拖拽释放：若已激活，按鼠标 x 计算目标 index 重排序。
                if let Some(drag) = self.tab_drag.take() {
                    if drag.active {
                        self.finish_tab_drag(&drag);
                        self.page_selection_drag = false;
                        return;
                    }
                }
                if was_scrollbar_drag || was_scroll_drag {
                    self.page_selection_drag = false;
                    return;
                }
                if self.page_selection_drag {
                    self.page_selection_drag = false;
                    // 拖拽释放落在地址栏：把选中文本填入地址栏（拖拽填充语义）。
                    if self.address_bar_hit_test(x as f32, y as f32) {
                        if let Some(text) = self.page_selection_text() {
                            self.address_bar.set_text(text.clone());
                            self.address_bar.set_cursor(
                                self.address_bar.text().len(),
                                false,
                            );
                            self.address_bar_focused = true;
                            self.address_bar.select_all();
                            self.update_autocomplete();
                            self.needs_redraw = true;
                        }
                        return;
                    }
                    if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x as f32, y as f32) {
                        if !moved_beyond_click_threshold {
                            // https://w3c.github.io/uievents/#event-type-click
                            // click 由同一次按下/释放及移动阈值决定；文字选区是否折叠不能作为
                            // 激活条件，否则输入框内一次轻微抖动就会吞掉 textarea/button click。
                            // 异步派发 click；若页面未 preventDefault，链接导航会通过
                            // `take_pending_actions` 在下一帧 poll 后执行（仿 Chrome 延迟导航）。
                            self.tabs
                                .dispatch_page_click(tab_id, doc_x, doc_y, self.ctrl_pressed || self.cmd_pressed);
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

        // 中键点击：
        // - 点「+」→ 无痕标签页
        // - 点标签 → 关闭该标签（Chrome/Firefox 标配交互）
        // - 点书签栏 → 后台新标签打开该书签
        if button == "Middle" {
            let s = self.scale_factor;
            let tab_strip_h = layout::TAB_STRIP_HEIGHT * s;
            let tab_y = layout::TAB_BAR_TOP_INSET * s;
            let toolbar_h = layout::TOOLBAR_HEIGHT * s;
            let y_f = y as f32;
            let x_f = x as f32;
            if y_f >= tab_y && y_f < tab_strip_h {
                // 先检测是否点中「+」按钮
                let new_tab_x = self.new_tab_button_x();
                if x_f >= new_tab_x && x_f < new_tab_x + layout::NEW_TAB_BTN_WIDTH * s {
                    self.new_blank_tab_focused(true);
                    return;
                }
                // 再检测是否点中某个标签
                for &(id, tab_x, tab_w) in &self.tab_layout {
                    if x_f >= tab_x && x_f < tab_x + tab_w {
                        self.close_tab_by_id(id);
                        return;
                    }
                }
            }
            // 书签栏中键 → 后台新标签打开
            if y_f >= toolbar_h && y_f < self.chrome_top_y_for(s) {
                if let Some((url, _)) = self.bookmark_bar_item_at(x_f, s) {
                    self.new_tab_background(&url);
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
                    self.new_blank_tab_focused(false);
                    return;
                }

                for &(id, tab_x, tab_w) in &self.tab_layout {
                    if x_f >= tab_x && x_f < tab_x + tab_w {
                        // 仅当标签宽度 >= COMPRESSED 时才判定 close 按钮命中，
                        // 极限压缩模式下不绘制 close 按钮，也不应触发关闭。
                        if tab_w >= layout::TAB_MIN_WIDTH_COMPRESSED * s {
                            let close_x = tab_x + tab_w - 24.0 * s;
                            let close_y_center = tab_y + tab_bar_h / 2.0;
                            if x_f >= close_x
                                && x_f <= close_x + tab_close_size
                                && (y_f - close_y_center).abs() <= tab_close_size / 2.0
                            {
                                self.close_tab_by_id(id);
                                return;
                            }
                        }
                        // 双击同一标签则关闭（参考 Chrome/Firefox）。
                        let now = Instant::now();
                        let is_double_click = self.last_tab_click_id == Some(id)
                            && self
                                .last_tab_click_time
                                .is_some_and(|t| now.duration_since(t).as_millis() < 400);
                        if is_double_click {
                            self.last_tab_click_time = None;
                            self.last_tab_click_id = None;
                            self.tab_drag = None;
                            self.close_tab_by_id(id);
                            return;
                        }
                        self.last_tab_click_time = Some(now);
                        self.last_tab_click_id = Some(id);
                        // 记录标签拖拽候选：active=false，待鼠标移动超过阈值才激活。
                        // 释放前若未激活则按普通点击处理。
                        self.tab_drag = Some(crate::app::TabDragState {
                            tab_id: id,
                            press_x: x_f,
                            tab_origin_x: tab_x,
                            tab_w,
                            current_x: x_f,
                            active: false,
                        });
                        if Some(id) != self.shell.active_tab_id() {
                            self.shell.switch_tab(id);
                            self.shell.set_tab_needs_attention(id, false);
                            self.tabs.on_active_tab_changed(self.shell.active_tab_id());
                            self.set_hovered_link_url(None);
                            self.update_address_bar_from_active_tab();
                            // zoom 按标签存储——切换后按新活动标签重推 CSS 视口。
                            self.sync_webview_viewport();
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
                    2 => {
                        // 加载中点击刷新按钮 → 停止加载（Chrome/Firefox 标配）。
                        if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
                            self.stop_loading_page();
                        } else {
                            self.refresh_page();
                        }
                    }
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
            // 选项切换按钮（区分大小写 / 全字匹配），位于 prev 按钮左侧。
            let case_x = bar_x + bar_w - 160.0 * s;
            let whole_x = bar_x + bar_w - 130.0 * s;
            if x_f >= case_x && x_f < case_x + 28.0 * s {
                self.shell.find_toggle_case_sensitive();
                if !self.find_input.is_empty() {
                    self.shell.find_start(&self.find_input.clone());
                }
                self.needs_redraw = true;
                return;
            }
            if x_f >= whole_x && x_f < whole_x + 28.0 * s {
                self.shell.find_toggle_whole_word();
                if !self.find_input.is_empty() {
                    self.shell.find_start(&self.find_input.clone());
                }
                self.needs_redraw = true;
                return;
            }
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
                if let Some((tab_id, doc_x, doc_y)) = self.page_doc_point(x_f, y_f) {
                    self.tabs.dispatch_page_mousedown(tab_id, doc_x, doc_y);
                    self.content_pointer_drag = Some(ContentPointerDrag {
                        start_x: x,
                        start_y: y,
                        last_y: y,
                        scrolling: false,
                    });
                    let idx = self
                        .page_glyphs(tab_id)
                        .and_then(|glyphs| hit_test_caret(&glyphs, doc_x, doc_y))
                        .unwrap_or(0);
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
        match event {
            zero_host_runtime::event::ImeEvent::Preedit { text, cursor } => {
                if in_address_bar {
                    self.address_bar_ime_preedit = text;
                    self.needs_redraw = true;
                } else if !in_find_bar
                    && let Some(tab_id) = self.shell.active_tab_id()
                {
                    self.tabs.dispatch_ime_event(
                        tab_id,
                        zero_protocol::message::ImeEventParams {
                            event_type: zero_protocol::message::ImeEventType::Preedit,
                            text,
                            cursor_start: cursor.map(|range| range.0),
                            cursor_end: cursor.map(|range| range.1),
                        },
                    );
                }
            }
            zero_host_runtime::event::ImeEvent::Commit(text) => {
                if in_address_bar {
                    self.address_bar_ime_preedit.clear();
                }
                // R3254-L4：空文本 Commit 也转发页面（部分 Linux IM 以空 Commit 表示取消合成）——
                // renderer/worker 的空 Commit 分支（compositionend + 清 preedit）此前不可达。
                if in_address_bar {
                    if !text.is_empty() {
                        self.address_bar.insert_str(&text);
                        self.update_autocomplete();
                    }
                } else if in_find_bar {
                    if !text.is_empty() {
                        self.find_input.push_str(&text);
                        self.shell.find_start(&self.find_input);
                    }
                } else if let Some(tab_id) = self.shell.active_tab_id() {
                    // https://w3c.github.io/uievents/#events-composition-input-events
                    self.tabs.dispatch_ime_event(
                        tab_id,
                        zero_protocol::message::ImeEventParams {
                            event_type: zero_protocol::message::ImeEventType::Commit,
                            text,
                            cursor_start: None,
                            cursor_end: None,
                        },
                    );
                }
                self.needs_redraw = true;
            }
            zero_host_runtime::event::ImeEvent::Enabled if !in_address_bar && !in_find_bar => {
                if let Some(tab_id) = self.shell.active_tab_id() {
                    self.tabs.dispatch_ime_event(
                        tab_id,
                        zero_protocol::message::ImeEventParams {
                            event_type: zero_protocol::message::ImeEventType::Enabled,
                            text: String::new(),
                            cursor_start: None,
                            cursor_end: None,
                        },
                    );
                }
            }
            zero_host_runtime::event::ImeEvent::Disabled if !in_address_bar && !in_find_bar => {
                if let Some(tab_id) = self.shell.active_tab_id() {
                    self.tabs.dispatch_ime_event(
                        tab_id,
                        zero_protocol::message::ImeEventParams {
                            event_type: zero_protocol::message::ImeEventType::Disabled,
                            text: String::new(),
                            cursor_start: None,
                            cursor_end: None,
                        },
                    );
                }
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
        // CSS 坐标换算含页面 zoom（device scale × zoom）。
        let s = self.page_render_scale();
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
            (x_f - content_x) / s + scroll.x / s,
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
        (bar_x.round(), bar_y.round(), bar_w.round(), bar_h.round())
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
