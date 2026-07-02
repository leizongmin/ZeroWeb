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

/// Packed scroll pointer-context (reduces arg count for clippy `too-many-arguments`).
struct ScrollPtrCtx<'a> {
    layout: &'a page_scroll::PageScrollLayout,
    scroll: page_scroll::TabScrollState,
    cx: f32,
    cy: f32,
    cw: f32,
    ch: f32,
}

impl<'a> ScrollPtrCtx<'a> {
    fn new(
        layout: &'a page_scroll::PageScrollLayout,
        scroll: page_scroll::TabScrollState,
        cx: f32,
        cy: f32,
        cw: f32,
        ch: f32,
    ) -> Self {
        Self {
            layout,
            scroll,
            cx,
            cy,
            cw,
            ch,
        }
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
        self.do_hit_test_scrollbar(x, y, &geometry).map(|hit| (tab_id, hit))
    }

    #[cfg(not(feature = "sdk-chrome"))]
    fn do_hit_test_scrollbar(&self, x: f32, y: f32, geometry: &page_scroll::ScrollbarGeometry) -> Option<page_scroll::ScrollbarHit> {
        page_scroll::hit_test_scrollbar(x, y, geometry)
    }

    #[cfg(feature = "sdk-chrome")]
    fn do_hit_test_scrollbar(&self, _x: f32, _y: f32, geometry: &page_scroll::ScrollbarGeometry) -> Option<page_scroll::ScrollbarHit> {
        crate::sdk_scrollbar::sdk_hit_test_scrollbar(_x, _y, geometry)
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
        let ctx = ScrollPtrCtx::new(&layout, scroll, cx, cy, cw, ch);

        match hit {
            page_scroll::ScrollbarHit::VerticalThumb => {
                let (_, thumb_y, _, _) = geometry.vertical_thumb.expect("vertical thumb");
                let grab_offset = y - thumb_y;
                let new_y = self.scroll_y_from_ptr(ctx, y, grab_offset);
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
                let new_y = self.scroll_y_from_ptr(ctx, y, grab_offset);
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
                let new_x = self.scroll_x_from_ptr(ctx, x, grab_offset);
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
                let new_x = self.scroll_x_from_ptr(ctx, x, grab_offset);
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
        let scroll = self.tab_scroll_state(tab_id);
        let ctx = ScrollPtrCtx::new(&layout, scroll, cx, cy, cw, ch);
        let new_value = match drag.axis {
            page_scroll::ScrollbarAxis::Vertical => Some((0, self.scroll_y_from_ptr(ctx, y, drag.grab_offset))),
            page_scroll::ScrollbarAxis::Horizontal => Some((1, self.scroll_x_from_ptr(ctx, x, drag.grab_offset))),
        };
        if let Some((axis, val)) = new_value {
            let entry = self.scroll.entry(tab_id).or_default();
            if axis == 0 {
                entry.y = val;
            } else {
                entry.x = val;
            }
        }
        self.needs_redraw = true;
    }

    // ── Pointer → scroll helpers (dispatched to hand-rolled or SDK per feature) ──

    #[cfg(not(feature = "sdk-chrome"))]
    fn scroll_y_from_ptr(&self, ctx: ScrollPtrCtx, pointer_y: f32, grab_offset: f32) -> f32 {
        page_scroll::scroll_y_from_pointer(ctx.layout, ctx.cy, ctx.ch, self.scale_factor, pointer_y, grab_offset)
    }

    #[cfg(feature = "sdk-chrome")]
    fn scroll_y_from_ptr(&self, ctx: ScrollPtrCtx, pointer_y: f32, grab_offset: f32) -> f32 {
        let content_rect = zero_ui_core::geometry::Rect::from_ltrb(ctx.cx, ctx.cy, ctx.cx + ctx.cw, ctx.cy + ctx.ch);
        crate::sdk_scrollbar::sdk_scroll_y_from_pointer(
            ctx.layout, ctx.scroll, content_rect,
            pointer_y - grab_offset, pointer_y,
        )
    }

    #[cfg(not(feature = "sdk-chrome"))]
    fn scroll_x_from_ptr(&self, ctx: ScrollPtrCtx, pointer_x: f32, grab_offset: f32) -> f32 {
        page_scroll::scroll_x_from_pointer(ctx.layout, ctx.cx, ctx.cw, self.scale_factor, pointer_x, grab_offset)
    }

    #[cfg(feature = "sdk-chrome")]
    fn scroll_x_from_ptr(&self, ctx: ScrollPtrCtx, pointer_x: f32, grab_offset: f32) -> f32 {
        let content_rect = zero_ui_core::geometry::Rect::from_ltrb(ctx.cx, ctx.cy, ctx.cx + ctx.cw, ctx.cy + ctx.ch);
        crate::sdk_scrollbar::sdk_scroll_x_from_pointer(
            ctx.layout, ctx.scroll, content_rect,
            pointer_x - grab_offset, pointer_x,
        )
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

    /// 滚动活动标签页到页面顶部（Home 键）。
    fn scroll_active_page_to_top(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            let entry = self.scroll.entry(tab_id).or_default();
            entry.y = 0.0;
            entry.x = 0.0;
            self.needs_redraw = true;
        }
    }

    /// 滚动活动标签页到页面底部（End 键）。
    fn scroll_active_page_to_bottom(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            let layout = self.page_scroll_layout(tab_id);
            let entry = self.scroll.entry(tab_id).or_default();
            entry.y = layout.max_scroll_y;
            self.needs_redraw = true;
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
                let was_scroll_drag = self.content_pointer_drag.as_ref().is_some_and(|d| d.scrolling);
                self.content_pointer_drag = None;
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
                        let collapsed = self.page_selection.get(&tab_id).is_none_or(|s| s.is_collapsed());
                        if collapsed {
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

}
