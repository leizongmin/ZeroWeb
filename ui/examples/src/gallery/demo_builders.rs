//! Demo area real-widget subtree builders (P2-11 / P2-12 / P2-13).
//!
//! Hosts `impl GalleryApp` extension block with one `build_<page>_demo` per page id.
//! Each demo is composed of Column/Row containers + real widgets (Button / ToggleWidget /
//! TextInputWidget) so the host -> widget -> action -> reducer -> props loop is real.
//!
//! P2-13 namespace-ization: each demo reads/writes state via `self.current_demo_read()` /
//! `self.current_demo()` which key on `current_page`, so cross-page state pollution is impossible.

use zero_ui_core::binding::Value;
use zero_ui_core::widget::{WidgetId, WidgetSpec};

use crate::gallery::GalleryApp;
use crate::gallery::model::DemoPage;

impl GalleryApp {
    /// Dispatch demo preview subtree by page id.
    pub(crate) fn build_demo_preview(&self, page: &'static DemoPage) -> WidgetSpec {
        match page.id {
            // widgets
            "button" => self.build_button_demo(),
            "toggle" => self.build_toggle_demo(),
            "text_input" => self.build_text_input_demo(),
            "icon_button" => self.build_icon_button_demo(),
            "badge" => self.build_badge_demo(),
            "progress" => self.build_progress_demo(),
            "tabs" => self.build_tabs_demo(),
            "tooltip" => self.build_tooltip_demo(),
            "list_view" => self.build_list_view_demo(),
            "menu" => self.build_menu_demo(),
            "search_field" => self.build_search_field_demo(),
            "status_bubble" => self.build_status_bubble_demo(),
            "toolbar" => self.build_toolbar_demo(),
            "popover" => self.build_popover_demo(),
            "popup" => self.build_popup_demo(),
            // patterns
            "search_field_demo" => self.build_search_field_demo(),
            "data_list" => self.build_data_list_demo(),
            "command_palette" => self.build_command_palette_demo(),
            "status_bubble_demo" => self.build_status_bubble_demo(),
            "tab_bar" => self.build_tabs_demo(),
            "dialog_scaffold" => self.build_dialog_scaffold_demo(),
            // forms / gestures / animation / collections
            "form_demo" => self.build_form_demo(),
            "gesture_demo" => self.build_gesture_demo(),
            "animation_demo" => self.build_animation_demo(),
            "collection_demo" => self.build_collection_demo(),
            // theme / i18n / dsl / nav
            "theme_demo" => self.build_theme_demo(),
            "i18n_demo" => self.build_i18n_demo(),
            "dsl_demo" => self.build_dsl_demo(),
            "nav_demo" => self.build_nav_demo(),
            _ => self.build_fallback_preview(page),
        }
    }

    fn build_fallback_preview(&self, page: &'static DemoPage) -> WidgetSpec {
        let mut label = self.themed_container("Column", "demo_fallback");
        label
            .props
            .insert("text", Value::Text(format!("(no demo builder for page: {})", page.id)));
        label
    }

    fn themed_container(&self, kind: &str, id: &str) -> WidgetSpec {
        let mut c = WidgetSpec::new(kind);
        c.id = Some(WidgetId::new(id));
        c.props.insert("theme", Value::Text(self.theme.as_str().into()));
        c
    }

    // ── widgets group ───────────────────────────────────────────────────────

    /// 3 Buttons: Default / Secondary / Disabled. Clicks 1/2 update pressed.
    fn build_button_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut row = self.themed_container("Row", "demo_button_row");
        row.props.insert("gap", Value::Float(12.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        let labels: [&str; 3] = [
            if st.pressed == 1 { "Clicked!" } else { "Default" },
            if st.pressed == 2 { "Clicked!" } else { "Secondary" },
            "Disabled",
        ];
        for (i, label) in labels.iter().enumerate() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_btn_{}", i + 1)));
            btn.props.insert("label", Value::Text((*label).into()));
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            btn.props.insert("enabled", Value::Bool(i < 2));
            row.children.push(btn);
        }
        row
    }

    /// 3 Toggles: first two interactive, third disabled. Bit i of toggles.
    fn build_toggle_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_toggle_col");
        col.props.insert("gap", Value::Float(8.0));

        let labels = ["Enable notifications", "Dark mode", "Disabled option"];
        for (i, label) in labels.iter().enumerate() {
            let mut row = self.themed_container("Row", &format!("demo_toggle_row_{}", i));
            row.props.insert("gap", Value::Float(12.0));
            row.props.insert("cross_axis_align", Value::Text("center".into()));

            let mut toggle = WidgetSpec::new("ToggleWidget");
            toggle.id = Some(WidgetId::new(&format!("demo_toggle_{}", i)));
            toggle
                .props
                .insert("checked", Value::Bool((st.toggles & (1 << i)) != 0));
            toggle
                .props
                .insert("action", Value::Text(format!("gallery.demo.toggle.{}", i)));
            toggle.props.insert("label", Value::Text((*label).into()));
            toggle.props.insert("enabled", Value::Bool(i < 2));
            row.children.push(toggle);
            col.children.push(row);
        }
        col
    }

    /// TextInput + live mirror of current text.
    fn build_text_input_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_text_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_text_input"));
        input.props.insert("text", Value::Text(st.text.clone()));
        input
            .props
            .insert("placeholder", Value::Text("Type something...".into()));
        col.children.push(input);

        let mut mirror = WidgetSpec::new("Text");
        mirror.id = Some(WidgetId::new("demo_text_mirror"));
        let display = if st.text.is_empty() {
            "(empty)".to_string()
        } else {
            format!("You typed: {}", st.text)
        };
        mirror.props.insert("text", Value::Text(display));
        col.children.push(mirror);

        col
    }

    /// 4 IconButtons; last-clicked highlighted with ColoredBox marker + real Icon glyph.
    fn build_icon_button_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_icon_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_icon_row");
        row.props.insert("gap", Value::Float(12.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        // P3-4-6：用真 Icon widget（Unicode glyph）替代 ASCII 字符。
        let icons = [
            ("Back", "back"),
            ("Fwd", "forward"),
            ("Reload", "play"),
            ("Close", "close"),
        ];
        for (name, icon_name) in icons.iter() {
            let pos = icons.iter().position(|(n, _)| n == name).unwrap();
            let active = (pos as u32 + 1) == st.pressed;
            let mut icon = WidgetSpec::new("Icon");
            icon.id = Some(WidgetId::new(&format!("demo_icon_glyph_{}", name)));
            icon.props.insert("name", Value::Text((*icon_name).into()));
            icon.props.insert("size", Value::Float(24.0));
            // 选中态用 primary 色，未选中用 muted；让 icon 视觉强化与按钮 hover 联动。
            icon.props
                .insert("color", Value::Text(if active { "primary" } else { "muted" }.into()));
            icon.props.insert("label", Value::Text((*name).into()));
            row.children.push(icon);

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_icon_btn_{}", name)));
            btn.props.insert("label", Value::Text((*name).to_string()));
            btn.props.insert(
                "variant",
                Value::Text(if active { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", pos + 1)));
            row.children.push(btn);
        }
        col.children.push(row);

        let mut hint = WidgetSpec::new("Text");
        hint.id = Some(WidgetId::new("demo_icon_hint"));
        hint.props
            .insert("text", Value::Text(format!("Last clicked: #{}", st.pressed)));
        col.children.push(hint);
        col
    }

    /// Badge: Inbox Button + 真彩色 ColoredBox 徽标（danger 色块包数字）。
    fn build_badge_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_badge_col");
        col.props.insert("gap", Value::Float(8.0));

        let count = st.pressed.min(99);
        let display = if count >= 99 {
            "99+".to_string()
        } else {
            count.to_string()
        };
        let mut row = self.themed_container("Row", "demo_badge_row");
        row.props.insert("gap", Value::Float(8.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        let mut inc_btn = WidgetSpec::new("Button");
        inc_btn.id = Some(WidgetId::new("demo_badge_inc"));
        inc_btn.props.insert("label", Value::Text("Inbox".into()));
        inc_btn
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        row.children.push(inc_btn);

        // 真彩色徽标：ColoredBox（radius 让它变成胶囊形）+ 内嵌 SourceLabel 文本（数字）。
        let badge_w = if count >= 99 { 36.0 } else { 24.0 };
        let mut badge_dot = WidgetSpec::new("ColoredBox");
        badge_dot.id = Some(WidgetId::new("demo_badge_dot"));
        badge_dot.props.insert("color", Value::Text("danger".into()));
        badge_dot.props.insert("width", Value::Float(badge_w));
        badge_dot.props.insert("height", Value::Float(20.0));
        // P3-4-6：radius = height/2 让徽标变成胶囊形（圆角全覆盖）。
        badge_dot.props.insert("radius", Value::Float(10.0));
        badge_dot
            .props
            .insert("label", Value::Text(format!("Unread: {}", display)));
        row.children.push(badge_dot);

        let mut badge_label = WidgetSpec::new("Text");
        badge_label.id = Some(WidgetId::new("demo_badge_count"));
        badge_label.props.insert("text", Value::Text(display));
        row.children.push(badge_label);

        col.children.push(row);
        let mut hint = WidgetSpec::new("Text");
        hint.id = Some(WidgetId::new("demo_badge_hint"));
        hint.props
            .insert("text", Value::Text("Click Inbox to +1 unread (capped at 99)".into()));
        col.children.push(hint);
        col
    }

    /// Progress: 真彩色进度条（filled ColoredBox + track ColoredBox）+ +/- 按钮。
    fn build_progress_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_progress_col");
        col.props.insert("gap", Value::Float(8.0));

        let pct = (st.pressed * 10).min(100);
        // 进度条：用 Row 容纳 filled 段（primary 色）+ track 段（muted 色）。
        let mut bar_row = self.themed_container("Row", "demo_progress_bar");
        bar_row.props.insert("gap", Value::Float(0.0));
        // filled 段：宽度按 pct 比例（用 max_width 约束近似；这里用固定 200px 总宽）。
        let total_w = 200.0_f32;
        let filled_w = total_w * (pct as f32) / 100.0;
        let track_w = total_w - filled_w;
        let mut filled = WidgetSpec::new("ColoredBox");
        filled.id = Some(WidgetId::new("demo_progress_filled"));
        filled.props.insert("color", Value::Text("primary".into()));
        filled.props.insert("label", Value::Text(format!("{}%", pct)));
        filled.props.insert("width", Value::Float((filled_w.max(2.0)) as f64));
        // P3-4-6：进度条圆角（与按钮一致的 6px）。
        filled.props.insert("radius", Value::Float(6.0));
        bar_row.children.push(filled);
        if track_w > 0.0 {
            let mut track = WidgetSpec::new("ColoredBox");
            track.id = Some(WidgetId::new("demo_progress_track"));
            track.props.insert("color", Value::Text("muted".into()));
            track.props.insert("width", Value::Float(track_w as f64));
            track.props.insert("radius", Value::Float(6.0));
            bar_row.children.push(track);
        }
        col.children.push(bar_row);

        // 百分比标签
        let mut pct_label = WidgetSpec::new("Text");
        pct_label.id = Some(WidgetId::new("demo_progress_label"));
        pct_label.props.insert("text", Value::Text(format!("{}%", pct)));
        col.children.push(pct_label);

        let mut row = self.themed_container("Row", "demo_progress_row");
        row.props.insert("gap", Value::Float(12.0));
        let mut plus = WidgetSpec::new("Button");
        plus.id = Some(WidgetId::new("demo_progress_plus"));
        plus.props.insert("label", Value::Text("+10%".into()));
        plus.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        row.children.push(plus);

        let mut minus = WidgetSpec::new("Button");
        minus.id = Some(WidgetId::new("demo_progress_minus"));
        minus.props.insert("label", Value::Text("-10%".into()));
        minus
            .props
            .insert("action", Value::Text("gallery.demo.button_click.2".into()));
        row.children.push(minus);
        col.children.push(row);
        col
    }

    /// Tabs: 3 tab buttons + content panel driven by selected index.
    fn build_tabs_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_tabs_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_tabs_row");
        row.props.insert("gap", Value::Float(4.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));
        let tabs = ["General", "Privacy", "Security"];
        let selected = (st.pressed as usize).saturating_sub(1).min(2);
        for (i, label) in tabs.iter().enumerate() {
            // P3-5-2：选中 tab 前加 check Icon（替代 ASCII "> "）。
            if i == selected {
                let mut check = WidgetSpec::new("Icon");
                check.id = Some(WidgetId::new(&format!("demo_tab_check_{}", i)));
                check.props.insert("name", Value::Text("check".into()));
                check.props.insert("size", Value::Float(16.0));
                check.props.insert("color", Value::Text("primary".into()));
                check.props.insert("label", Value::Text("Active tab".into()));
                row.children.push(check);
            }
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_tab_{}", i)));
            btn.props.insert("label", Value::Text((*label).to_string()));
            btn.props.insert(
                "variant",
                Value::Text(if i == selected { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            row.children.push(btn);
        }
        col.children.push(row);

        let contents = [
            "General settings: appearance, language, font.",
            "Privacy: cookies, tracking, permissions.",
            "Security: HTTPS-only, certificate exceptions.",
        ];
        let mut body = WidgetSpec::new("Text");
        body.id = Some(WidgetId::new("demo_tab_content"));
        body.props.insert("text", Value::Text(contents[selected].into()));
        col.children.push(body);
        col
    }

    /// Tooltip: hover over button shows hint label; leave hides it (真 hover)。
    fn build_tooltip_demo(&self) -> WidgetSpec {
        // P3-5-1：tooltip 改用真浮层（host.overlay_root），不再线性追加 bubble。
        // 这里只画触发按钮；bubble 由 GalleryApp::overlay() + build_tooltip_overlay() 提供。
        let mut col = self.themed_container("Column", "demo_tooltip_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut btn = WidgetSpec::new("Button");
        btn.id = Some(WidgetId::new("demo_tooltip_btn"));
        btn.props.insert("label", Value::Text("Hover me".into()));
        btn.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        // P2-14：用真 hover_action 替代 click toggle。
        btn.props
            .insert("hover_action", Value::Text("gallery.demo.hover".into()));
        col.children.push(btn);

        // 提示文字（常驻）：解释 tooltip 行为，让用户知道要 hover。
        let mut hint = WidgetSpec::new("Text");
        hint.id = Some(WidgetId::new("demo_tooltip_hint"));
        hint.props.insert(
            "text",
            Value::Text("Hover the button → a tooltip floats above (real overlay, not inline).".into()),
        );
        col.children.push(hint);
        col
    }

    /// ListView: 5 selectable rows; selected row prefixed with check Icon.
    fn build_list_view_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_list_col");
        col.props.insert("gap", Value::Float(4.0));
        let selected = (st.pressed as usize).saturating_sub(1).min(4);
        for i in 0..5usize {
            let mut row = self.themed_container("Row", &format!("demo_list_row_{}", i));
            row.props.insert("gap", Value::Float(8.0));
            row.props.insert("cross_axis_align", Value::Text("center".into()));

            // P3-5-2：选中项前加 check Icon（替代 ASCII "> "）。
            if i == selected {
                let mut check = WidgetSpec::new("Icon");
                check.id = Some(WidgetId::new(&format!("demo_list_check_{}", i)));
                check.props.insert("name", Value::Text("check".into()));
                check.props.insert("size", Value::Float(16.0));
                check.props.insert("color", Value::Text("primary".into()));
                check.props.insert("label", Value::Text("Selected".into()));
                row.children.push(check);
            } else {
                // 未选中留空位（保持宽度一致，避免选中切换时跳变）。
                let mut spacer = WidgetSpec::new("ColoredBox");
                spacer.id = Some(WidgetId::new(&format!("demo_list_spacer_{}", i)));
                spacer.props.insert("color", Value::Text("muted".into()));
                spacer.props.insert("width", Value::Float(16.0));
                spacer.props.insert("height", Value::Float(16.0));
                // 透明色块（muted 但实际不渲染影响视觉——保留占位宽度）。
                row.children.push(spacer);
            }

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_list_item_{}", i)));
            btn.props.insert("label", Value::Text(format!("Item {}", i + 1)));
            btn.props.insert(
                "variant",
                Value::Text(if i == selected { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            row.children.push(btn);
            col.children.push(row);
        }
        col
    }

    /// Menu: vertical items; selected item prefixed with check Icon.
    fn build_menu_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_menu_col");
        col.props.insert("gap", Value::Float(4.0));
        let items = [
            ("Open...", "open", 1u32),
            ("Save", "save", 2),
            ("Save As...", "save_as", 3),
            ("Exit", "exit", 4),
        ];
        let selected = st.pressed;
        for (label, name, idx) in items.iter() {
            let mut row = self.themed_container("Row", &format!("demo_menu_row_{}", name));
            row.props.insert("gap", Value::Float(8.0));
            row.props.insert("cross_axis_align", Value::Text("center".into()));

            // P3-5-2：选中项前加 check Icon（替代 ASCII "> "）。
            if *idx == selected {
                let mut check = WidgetSpec::new("Icon");
                check.id = Some(WidgetId::new(&format!("demo_menu_check_{}", name)));
                check.props.insert("name", Value::Text("check".into()));
                check.props.insert("size", Value::Float(16.0));
                check.props.insert("color", Value::Text("primary".into()));
                check.props.insert("label", Value::Text("Selected".into()));
                row.children.push(check);
            } else {
                let mut spacer = WidgetSpec::new("ColoredBox");
                spacer.id = Some(WidgetId::new(&format!("demo_menu_spacer_{}", name)));
                spacer.props.insert("color", Value::Text("muted".into()));
                spacer.props.insert("width", Value::Float(16.0));
                spacer.props.insert("height", Value::Float(16.0));
                row.children.push(spacer);
            }

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_menu_{}", name)));
            btn.props.insert("label", Value::Text((*label).to_string()));
            btn.props.insert(
                "variant",
                Value::Text(if *idx == selected { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            row.children.push(btn);
            col.children.push(row);
        }
        col
    }

    /// SearchField: TextInput + live suggestion list.
    fn build_search_field_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_search_col");
        col.props.insert("gap", Value::Float(8.0));

        // 搜索输入框 + search Icon 前缀（更像真实搜索框）。
        let mut input_row = self.themed_container("Row", "demo_search_input_row");
        input_row.props.insert("gap", Value::Float(8.0));
        input_row.props.insert("cross_axis_align", Value::Text("center".into()));

        let mut search_icon = WidgetSpec::new("Icon");
        search_icon.id = Some(WidgetId::new("demo_search_icon"));
        search_icon.props.insert("name", Value::Text("search".into()));
        search_icon.props.insert("size", Value::Float(20.0));
        search_icon.props.insert("color", Value::Text("muted".into()));
        search_icon.props.insert("label", Value::Text("Search".into()));
        input_row.children.push(search_icon);

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_search_input"));
        input.props.insert("text", Value::Text(st.text.clone()));
        input
            .props
            .insert("placeholder", Value::Text("Search components...".into()));
        input_row.children.push(input);
        col.children.push(input_row);

        // P3-5-3：建议列表用真 Button 行（替代纯文本 SourceLabel），每行配 check Icon。
        let query = st.text.trim().to_lowercase();
        let candidates = ["button", "toggle", "text_input", "menu", "tabs"];
        let matches: Vec<&str> = candidates
            .iter()
            .copied()
            .filter(|c| c.starts_with(query.as_str()))
            .collect();
        if query.is_empty() {
            let mut hint = WidgetSpec::new("Text");
            hint.id = Some(WidgetId::new("demo_search_hint"));
            hint.props
                .insert("text", Value::Text("(type to filter suggestions)".into()));
            col.children.push(hint);
        } else if matches.is_empty() {
            let mut no_match = WidgetSpec::new("Text");
            no_match.id = Some(WidgetId::new("demo_search_no_match"));
            no_match.props.insert("text", Value::Text("No match".into()));
            col.children.push(no_match);
        } else {
            // 每个建议一行：check Icon + Button（点击选中）。
            for (i, candidate) in matches.iter().enumerate() {
                let mut row = self.themed_container("Row", &format!("demo_search_row_{}", i));
                row.props.insert("gap", Value::Float(8.0));
                row.props.insert("cross_axis_align", Value::Text("center".into()));

                let mut check = WidgetSpec::new("Icon");
                check.id = Some(WidgetId::new(&format!("demo_search_check_{}", i)));
                check.props.insert("name", Value::Text("check".into()));
                check.props.insert("size", Value::Float(14.0));
                check.props.insert("color", Value::Text("primary".into()));
                check.props.insert("label", Value::Text("Suggestion".into()));
                row.children.push(check);

                let mut btn = WidgetSpec::new("Button");
                btn.id = Some(WidgetId::new(&format!("demo_search_item_{}", i)));
                btn.props.insert("label", Value::Text((*candidate).to_string()));
                btn.props.insert("variant", Value::Text("neutral".into()));
                btn.props
                    .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
                row.children.push(btn);
                col.children.push(row);
            }
        }
        col
    }

    /// StatusBubble: ColoredBox 状态色块（success/warning/danger）+ cycle button。
    fn build_status_bubble_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_status_col");
        col.props.insert("gap", Value::Float(8.0));

        let (color_name, label_text) = match (st.pressed as usize).saturating_sub(1) % 3 {
            0 => ("success", "Saved"),
            1 => ("warning", "Pending"),
            2 => ("danger", "Failed"),
            _ => unreachable!(),
        };

        let mut row = self.themed_container("Row", "demo_status_dot_row");
        row.props.insert("gap", Value::Float(8.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        // 真彩色圆点（窄 ColoredBox + radius = 一半宽度变正圆）。
        let mut dot = WidgetSpec::new("ColoredBox");
        dot.id = Some(WidgetId::new("demo_status_dot"));
        dot.props.insert("color", Value::Text(color_name.into()));
        dot.props.insert("width", Value::Float(16.0));
        dot.props.insert("height", Value::Float(16.0));
        // P3-4-6：radius = 8（宽高一半）让色块变成正圆状态点。
        dot.props.insert("radius", Value::Float(8.0));
        dot.props.insert("label", Value::Text(label_text.into()));
        row.children.push(dot);

        let mut label = WidgetSpec::new("Text");
        label.id = Some(WidgetId::new("demo_status_label"));
        label.props.insert("text", Value::Text(label_text.into()));
        row.children.push(label);
        col.children.push(row);

        let mut next = WidgetSpec::new("Button");
        next.id = Some(WidgetId::new("demo_status_next"));
        next.props.insert("label", Value::Text("Next status".into()));
        next.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(next);
        col
    }

    /// Toolbar: horizontal Buttons with real Icon markers; last-clicked highlighted.
    fn build_toolbar_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut row = self.themed_container("Row", "demo_toolbar_row");
        row.props.insert("gap", Value::Float(4.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));
        let actions = [
            ("back", "Back", 1u32),
            ("forward", "Forward", 2),
            ("play", "Reload", 3),
            ("home", "Home", 4),
        ];
        for (icon_name, label, idx) in actions.iter() {
            let active = *idx == st.pressed;
            let mut icon = WidgetSpec::new("Icon");
            icon.id = Some(WidgetId::new(&format!("demo_toolbar_glyph_{}", idx)));
            icon.props.insert("name", Value::Text((*icon_name).into()));
            icon.props.insert("size", Value::Float(20.0));
            icon.props
                .insert("color", Value::Text(if active { "primary" } else { "muted" }.into()));
            icon.props.insert("label", Value::Text((*label).into()));
            row.children.push(icon);

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_toolbar_{}", idx)));
            btn.props.insert("label", Value::Text((*label).to_string()));
            btn.props.insert(
                "variant",
                Value::Text(if active { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            row.children.push(btn);
        }
        row
    }

    /// Popover: trigger Button toggles a floating content block.
    fn build_popover_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_popover_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = st.pressed == 1;
        let mut trigger = WidgetSpec::new("Button");
        trigger.id = Some(WidgetId::new("demo_popover_trigger"));
        trigger.props.insert(
            "label",
            Value::Text(if open { "Close popover" } else { "Open popover" }.into()),
        );
        trigger
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(trigger);

        if open {
            let mut content = WidgetSpec::new("Text");
            content.id = Some(WidgetId::new("demo_popover_content"));
            content
                .props
                .insert("text", Value::Text("Popover: floats above other content.".into()));
            col.children.push(content);
        }
        col
    }

    /// Popup: trigger + OK / Cancel buttons (modal-like).
    fn build_popup_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_popup_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = st.pressed == 1;
        let mut trigger = WidgetSpec::new("Button");
        trigger.id = Some(WidgetId::new("demo_popup_trigger"));
        trigger.props.insert(
            "label",
            Value::Text(if open { "Close popup" } else { "Open popup" }.into()),
        );
        trigger
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(trigger);

        // P1-6 修复：OK/Cancel 只在 overlay（build_popup_overlay）中渲染，
        // 主树不放（避免重复 + modal barrier 下主树按钮不可点）。
        let mut hint = WidgetSpec::new("Text");
        hint.id = Some(WidgetId::new("demo_popup_hint"));
        hint.props.insert(
            "text",
            Value::Text("Click trigger to toggle modal popup. OK/Cancel buttons appear in the overlay.".into()),
        );
        col.children.push(hint);
        col
    }

    // ── patterns group ──────────────────────────────────────────────────────

    /// DataList: TextInput + Add button + 8-row state (toggle bitmask).
    fn build_data_list_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_data_list_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_data_list_input_row");
        row.props.insert("gap", Value::Float(8.0));
        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_data_list_input"));
        input.props.insert("text", Value::Text(st.text.clone()));
        input.props.insert("placeholder", Value::Text("New item...".into()));
        row.children.push(input);

        let mut add = WidgetSpec::new("Button");
        add.id = Some(WidgetId::new("demo_data_list_add"));
        add.props.insert("label", Value::Text("Add".into()));
        add.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        row.children.push(add);
        col.children.push(row);

        let mut list_col = self.themed_container("Column", "demo_data_list_items");
        list_col.props.insert("gap", Value::Float(4.0));
        for i in 0..8 {
            let on = (st.toggles & (1 << i)) != 0;
            let mut toggle = WidgetSpec::new("ToggleWidget");
            toggle.id = Some(WidgetId::new(&format!("demo_data_list_t_{}", i)));
            toggle.props.insert("checked", Value::Bool(on));
            toggle
                .props
                .insert("action", Value::Text(format!("gallery.demo.toggle.{}", i)));
            toggle.props.insert("label", Value::Text(format!("Item {}", i + 1)));
            toggle.props.insert("enabled", Value::Bool(true));
            list_col.children.push(toggle);
        }
        col.children.push(list_col);
        col
    }

    /// CommandPalette: TextInput + filtered command list.
    fn build_command_palette_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_cmd_palette_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_cmd_input"));
        input.props.insert("text", Value::Text(st.text.clone()));
        input
            .props
            .insert("placeholder", Value::Text("Type a command...".into()));
        col.children.push(input);

        let cmds = [
            "open file",
            "save",
            "close tab",
            "reload",
            "toggle theme",
            "search",
            "open settings",
            "quit",
        ];
        let q = st.text.trim().to_lowercase();
        let filtered: Vec<&str> = if q.is_empty() {
            cmds.to_vec()
        } else {
            cmds.iter().copied().filter(|c| c.contains(q.as_str())).collect()
        };

        let mut list_col = self.themed_container("Column", "demo_cmd_list");
        list_col.props.insert("gap", Value::Float(4.0));
        // 选中索引：用 pressed 表示（点 button i 会触发 gallery.demo.button_click.{i+1}，pressed=i+1）。
        let selected = st.pressed.saturating_sub(1) as usize;
        for (i, cmd) in filtered.iter().take(5).enumerate() {
            let mut row = self.themed_container("Row", &format!("demo_cmd_row_{}", i));
            row.props.insert("gap", Value::Float(8.0));
            row.props.insert("cross_axis_align", Value::Text("center".into()));

            // 选中项前加 primary 色块，未选中加 muted 色块作为视觉 marker。
            let mut marker = WidgetSpec::new("ColoredBox");
            marker.id = Some(WidgetId::new(&format!("demo_cmd_marker_{}", i)));
            marker.props.insert(
                "color",
                Value::Text(if i == selected { "primary" } else { "muted" }.into()),
            );
            marker.props.insert("width", Value::Float(8.0));
            marker.props.insert("height", Value::Float(16.0));
            marker.props.insert("label", Value::Text((*cmd).into()));
            row.children.push(marker);

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_cmd_item_{}", i)));
            btn.props.insert("label", Value::Text((*cmd).to_string()));
            btn.props.insert(
                "variant",
                Value::Text(if i == selected { "selected" } else { "neutral" }.into()),
            );
            // 命令索引复用 button_click slot 1..5；filtered 内容是动态的，dispatch 只记位置。
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            row.children.push(btn);

            list_col.children.push(row);
        }
        col.children.push(list_col);

        if filtered.is_empty() {
            let mut empty = WidgetSpec::new("Text");
            empty.id = Some(WidgetId::new("demo_cmd_empty"));
            empty.props.insert("text", Value::Text("(no match)".into()));
            col.children.push(empty);
        }
        col
    }

    /// DialogScaffold: trigger only (dialog body + OK/Cancel in overlay).
    fn build_dialog_scaffold_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_dialog_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = st.pressed == 1;
        let mut trigger = WidgetSpec::new("Button");
        trigger.id = Some(WidgetId::new("demo_dialog_trigger"));
        trigger.props.insert(
            "label",
            Value::Text(if open { "Close dialog" } else { "Open dialog" }.into()),
        );
        trigger
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(trigger);

        // P1-6 修复：dialog body + OK/Cancel 只在 overlay（build_dialog_overlay）。
        let mut hint = WidgetSpec::new("Text");
        hint.id = Some(WidgetId::new("demo_dialog_hint"));
        hint.props.insert(
            "text",
            Value::Text("Click trigger to toggle modal dialog. Content appears in the overlay.".into()),
        );
        col.children.push(hint);
        col
    }

    // ── forms / gestures / animation / collections ────────────────────────

    /// Form: name TextInput + subscribe Toggle + Submit Button.
    fn build_form_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_form_col");
        col.props.insert("gap", Value::Float(12.0));

        let mut name_input = WidgetSpec::new("TextInputWidget");
        name_input.id = Some(WidgetId::new("demo_form_name"));
        name_input.props.insert("text", Value::Text(st.text.clone()));
        name_input.props.insert("placeholder", Value::Text("Your name".into()));
        col.children.push(name_input);

        let mut sub_toggle = WidgetSpec::new("ToggleWidget");
        sub_toggle.id = Some(WidgetId::new("demo_form_subscribe"));
        sub_toggle.props.insert("checked", Value::Bool((st.toggles & 1) != 0));
        sub_toggle
            .props
            .insert("action", Value::Text("gallery.demo.toggle.0".into()));
        sub_toggle
            .props
            .insert("label", Value::Text("Subscribe to newsletter".into()));
        col.children.push(sub_toggle);

        let mut submit = WidgetSpec::new("Button");
        submit.id = Some(WidgetId::new("demo_form_submit"));
        submit.props.insert(
            "label",
            Value::Text(if st.pressed == 1 {
                "Submitted!".into()
            } else {
                "Submit".into()
            }),
        );
        submit
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(submit);
        col
    }

    /// Gesture: Buttons for Tap / Double-tap / Long press with ColoredBox active marker.
    fn build_gesture_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_gesture_col");
        col.props.insert("gap", Value::Float(8.0));

        let labels = ["Tap", "Double tap", "Long press"];
        for (i, label) in labels.iter().enumerate() {
            let active = (i + 1) as u32 == st.pressed;
            let mut row = self.themed_container("Row", &format!("demo_gesture_row_{}", i));
            row.props.insert("gap", Value::Float(8.0));
            row.props.insert("cross_axis_align", Value::Text("center".into()));

            let mut marker = WidgetSpec::new("ColoredBox");
            marker.id = Some(WidgetId::new(&format!("demo_gesture_marker_{}", i)));
            marker
                .props
                .insert("color", Value::Text(if active { "primary" } else { "muted" }.into()));
            marker.props.insert("width", Value::Float(8.0));
            marker.props.insert("height", Value::Float(20.0));
            marker.props.insert("label", Value::Text((*label).into()));
            row.children.push(marker);

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_gesture_{}", i)));
            btn.props.insert(
                "label",
                Value::Text(if active {
                    format!("{} (active)", label)
                } else {
                    (*label).to_string()
                }),
            );
            btn.props.insert(
                "variant",
                Value::Text(if active { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            row.children.push(btn);
            col.children.push(row);
        }
        col
    }

    /// Animation: Buttons to switch a ColoredBox "indicator" (color + width varies by state).
    /// 注：真动画需要 render-foundation 时间线 API；这里用"状态切换"演示视觉变化。
    fn build_animation_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_anim_col");
        col.props.insert("gap", Value::Float(8.0));

        let states = [
            ("Idle", "muted", 60.0_f32),
            ("Fade in", "primary", 120.0),
            ("Slide", "success", 180.0),
            ("Spin", "warning", 240.0),
        ];
        let cur = (st.pressed as usize).min(states.len() - 1);
        let (state_name, color, width) = states[cur];

        let mut indicator = WidgetSpec::new("ColoredBox");
        indicator.id = Some(WidgetId::new("demo_anim_indicator"));
        indicator.props.insert("color", Value::Text(color.into()));
        indicator.props.insert("width", Value::Float(width as f64));
        indicator.props.insert("height", Value::Float(24.0));
        indicator.props.insert("radius", Value::Float(6.0));
        // P3-4-5：所有状态都启用 pulse，让颜色明度连续振荡——验证 host 每 frame 推进 clock
        // + widget 用 ctx.request_frame() 拉下一帧的完整动画环路。
        indicator.props.insert("pulse", Value::Bool(true));
        indicator.props.insert("label", Value::Text(state_name.into()));
        col.children.push(indicator);

        let mut state_label = WidgetSpec::new("Text");
        state_label.id = Some(WidgetId::new("demo_anim_state"));
        state_label
            .props
            .insert("text", Value::Text(format!("State: {}", state_name)));
        col.children.push(state_label);

        let mut row = self.themed_container("Row", "demo_anim_row");
        row.props.insert("gap", Value::Float(4.0));
        for (i, name) in states.iter().enumerate() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_anim_btn_{}", i)));
            btn.props.insert("label", Value::Text(name.0.into()));
            btn.props.insert(
                "variant",
                Value::Text(if i == cur { "selected" } else { "neutral" }.into()),
            );
            // button_click slot 从 1 开始（0 用于"无选择"），dispatch 处 0 不计数。
            // 这里用 i (0-based) 作 slot 是与现有 animation demo 一致：pressed==0 → Idle。
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i)));
            row.children.push(btn);
        }
        col.children.push(row);
        col
    }

    /// Collection: 8 Toggles + count summary. Each toggle has its own action 0..7.
    fn build_collection_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_collection_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut summary = WidgetSpec::new("Text");
        summary.id = Some(WidgetId::new("demo_collection_summary"));
        let count = st.toggles.count_ones();
        summary
            .props
            .insert("text", Value::Text(format!("Selected items: {}/8", count)));
        col.children.push(summary);

        for i in 0..8 {
            let mut toggle = WidgetSpec::new("ToggleWidget");
            toggle.id = Some(WidgetId::new(&format!("demo_collection_t_{}", i)));
            toggle
                .props
                .insert("checked", Value::Bool((st.toggles & (1 << i)) != 0));
            toggle
                .props
                .insert("action", Value::Text(format!("gallery.demo.toggle.{}", i)));
            toggle.props.insert("label", Value::Text(format!("Item {}", i + 1)));
            col.children.push(toggle);
        }
        col
    }

    // ── theme / i18n / dsl / nav ──────────────────────────────────────────

    /// Theme: shows current theme + Button that triggers gallery.theme.toggle.
    fn build_theme_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_theme_col");
        col.props.insert("gap", Value::Float(8.0));

        let current = match self.theme {
            crate::gallery::model::ThemeKind::Light => "Light",
            crate::gallery::model::ThemeKind::Dark => "Dark",
        };
        let mut cur = WidgetSpec::new("Text");
        cur.id = Some(WidgetId::new("demo_theme_cur"));
        cur.props
            .insert("text", Value::Text(format!("Current theme: {}", current)));
        col.children.push(cur);

        let mut toggle = WidgetSpec::new("Button");
        toggle.id = Some(WidgetId::new("demo_theme_toggle"));
        toggle.props.insert("label", Value::Text("Toggle theme".into()));
        toggle
            .props
            .insert("action", Value::Text("gallery.theme.toggle".into()));
        col.children.push(toggle);
        col
    }

    /// i18n: shows localized greeting + Button to toggle locale.
    fn build_i18n_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_i18n_col");
        col.props.insert("gap", Value::Float(8.0));

        let hello = match self.locale {
            crate::gallery::model::Locale::En => "Hello, world!",
            crate::gallery::model::Locale::Zh => "Hello, world! (zh)",
        };
        let mut label = WidgetSpec::new("Text");
        label.id = Some(WidgetId::new("demo_i18n_text"));
        label.props.insert("text", Value::Text(hello.into()));
        col.children.push(label);

        let mut toggle = WidgetSpec::new("Button");
        toggle.id = Some(WidgetId::new("demo_i18n_toggle"));
        toggle.props.insert("label", Value::Text("Toggle language".into()));
        toggle
            .props
            .insert("action", Value::Text("gallery.locale.toggle".into()));
        col.children.push(toggle);
        col
    }

    /// DSL: static YAML sample + Apply button.
    fn build_dsl_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_dsl_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut label = WidgetSpec::new("Text");
        label.id = Some(WidgetId::new("demo_dsl_text"));
        label.props.insert(
            "text",
            Value::Text("Button:\n  label: Click me\n  action: button.clicked\nToggle:\n  checked: true".into()),
        );
        col.children.push(label);

        let mut apply = WidgetSpec::new("Button");
        apply.id = Some(WidgetId::new("demo_dsl_apply"));
        apply.props.insert("label", Value::Text("Apply DSL".into()));
        apply
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(apply);
        col
    }

    /// Nav: Buttons for Back / Forward / Home; current selection highlighted.
    fn build_nav_demo(&self) -> WidgetSpec {
        let st = self.current_demo_read();
        let mut col = self.themed_container("Column", "demo_nav_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_nav_row");
        row.props.insert("gap", Value::Float(4.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));
        let items = [("back", "Back", 1u32), ("forward", "Forward", 2), ("home", "Home", 3)];
        for (icon_name, label, idx) in items.iter() {
            let active = *idx == st.pressed;
            let mut icon = WidgetSpec::new("Icon");
            icon.id = Some(WidgetId::new(&format!("demo_nav_marker_{}", idx)));
            icon.props.insert("name", Value::Text((*icon_name).into()));
            icon.props.insert("size", Value::Float(20.0));
            icon.props
                .insert("color", Value::Text(if active { "primary" } else { "muted" }.into()));
            icon.props.insert("label", Value::Text((*label).into()));
            row.children.push(icon);

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_nav_{}", idx)));
            btn.props.insert(
                "label",
                Value::Text(if active {
                    format!("{} (active)", label)
                } else {
                    (*label).to_string()
                }),
            );
            btn.props.insert(
                "variant",
                Value::Text(if active { "selected" } else { "neutral" }.into()),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            row.children.push(btn);
        }
        col.children.push(row);

        let mut state = WidgetSpec::new("Text");
        state.id = Some(WidgetId::new("demo_nav_state"));
        state.props.insert(
            "text",
            Value::Text(format!(
                "Current: {}",
                match st.pressed {
                    1 => "Back",
                    2 => "Forward",
                    3 => "Home",
                    _ => "(none)",
                }
            )),
        );
        col.children.push(state);
        col
    }
}
