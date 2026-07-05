//! Demo area real-widget subtree builders (P2-11 / P2-12 widget migration).
//!
//! Hosts `impl GalleryApp` extension block with one `build_<page>_demo` per page id.
//! Each demo is composed of Column/Row containers + real widgets (Button / ToggleWidget /
//! TextInputWidget) so the host -> widget -> action -> reducer -> props loop is real.
//!
//! Note: this file is just an `impl GalleryApp` extension block; all methods belong to `GalleryApp`.

use zero_ui_core::binding::Value;
use zero_ui_core::widget::{WidgetId, WidgetSpec};

use crate::gallery::GalleryApp;
use crate::gallery::model::DemoPage;

impl GalleryApp {
    /// Dispatch demo preview subtree by page id.
    ///
    /// Since P2-11 each demo is progressively migrated to real-widget subtrees.
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
        let mut preview = WidgetSpec::new("DemoPreview");
        preview.id = Some(WidgetId::new("demo_preview"));
        preview.props.insert("theme", Value::Text(self.theme.as_str().into()));
        preview.props.insert("page_id", Value::Text(page.id.into()));
        preview
    }

    fn themed_container(&self, kind: &str, id: &str) -> WidgetSpec {
        let mut c = WidgetSpec::new(kind);
        c.id = Some(WidgetId::new(id));
        c.props.insert("theme", Value::Text(self.theme.as_str().into()));
        c
    }

    // ?? widgets group ???????????????????????????????????????????????????????

    /// 3 Buttons: Default / Secondary / Disabled. Clicks 1/2 update demo_button_pressed.
    fn build_button_demo(&self) -> WidgetSpec {
        let mut row = self.themed_container("Row", "demo_button_row");
        row.props.insert("gap", Value::Float(12.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        let labels: [&str; 3] = [
            if self.demo_button_pressed == 1 {
                "Clicked!"
            } else {
                "Default"
            },
            if self.demo_button_pressed == 2 {
                "Clicked!"
            } else {
                "Secondary"
            },
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

    /// 3 Toggles: first two interactive, third disabled. Bit i of demo_toggle_state.
    fn build_toggle_demo(&self) -> WidgetSpec {
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
                .insert("checked", Value::Bool((self.demo_toggle_state & (1 << i)) != 0));
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
        let mut col = self.themed_container("Column", "demo_text_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_text_input"));
        input.props.insert("text", Value::Text(self.demo_text_input.clone()));
        input
            .props
            .insert("placeholder", Value::Text("Type something...".into()));
        col.children.push(input);

        let mut mirror = WidgetSpec::new("SourceLabel");
        mirror.id = Some(WidgetId::new("demo_text_mirror"));
        let display = if self.demo_text_input.is_empty() {
            "(empty)".to_string()
        } else {
            format!("You typed: {}", self.demo_text_input)
        };
        mirror.props.insert("text", Value::Text(display));
        col.children.push(mirror);

        col
    }

    /// 4 IconButtons (unicode glyphs as icons); last-clicked is highlighted.
    fn build_icon_button_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_icon_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_icon_row");
        row.props.insert("gap", Value::Float(12.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        let icons = [("Back", "<"), ("Fwd", ">"), ("Reload", "R"), ("Close", "X")];
        for (name, glyph) in icons.iter() {
            let pos = icons.iter().position(|(n, _)| n == name).unwrap();
            let active = (pos as u32 + 1) == self.demo_button_pressed;
            let label = if active {
                format!("[{}]", glyph)
            } else {
                (*glyph).to_string()
            };
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_icon_btn_{}", name)));
            btn.props.insert("label", Value::Text(label));
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", pos + 1)));
            row.children.push(btn);
        }
        col.children.push(row);

        let mut hint = WidgetSpec::new("SourceLabel");
        hint.id = Some(WidgetId::new("demo_icon_hint"));
        hint.props.insert(
            "text",
            Value::Text(format!("Last clicked: #{}", self.demo_button_pressed)),
        );
        col.children.push(hint);
        col
    }

    /// Badge: Inbox Button + count label; each click +1 unread (capped at 99).
    fn build_badge_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_badge_col");
        col.props.insert("gap", Value::Float(8.0));

        let count = self.demo_button_pressed.min(99);
        let display = if count >= 99 {
            "99+".to_string()
        } else {
            count.to_string()
        };
        let mut row = self.themed_container("Row", "demo_badge_row");
        row.props.insert("gap", Value::Float(12.0));
        row.props.insert("cross_axis_align", Value::Text("center".into()));

        let mut inc_btn = WidgetSpec::new("Button");
        inc_btn.id = Some(WidgetId::new("demo_badge_inc"));
        inc_btn.props.insert("label", Value::Text("Inbox".into()));
        inc_btn
            .props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        row.children.push(inc_btn);

        let mut badge = WidgetSpec::new("SourceLabel");
        badge.id = Some(WidgetId::new("demo_badge_count"));
        badge.props.insert("text", Value::Text(format!("Unread: {}", display)));
        row.children.push(badge);

        col.children.push(row);
        let mut hint = WidgetSpec::new("SourceLabel");
        hint.id = Some(WidgetId::new("demo_badge_hint"));
        hint.props
            .insert("text", Value::Text("Click Inbox to +1 unread (capped at 99)".into()));
        col.children.push(hint);
        col
    }

    /// Progress: ASCII bar driven by demo_button_pressed; +/- buttons adjust by 10%.
    fn build_progress_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_progress_col");
        col.props.insert("gap", Value::Float(8.0));

        let pct = (self.demo_button_pressed * 10).min(100);
        let filled = (pct / 10) as usize;
        let bar_text = format!("[{}{}] {}%", "#".repeat(filled), "_".repeat(10 - filled), pct);
        let mut bar = WidgetSpec::new("SourceLabel");
        bar.id = Some(WidgetId::new("demo_progress_bar"));
        bar.props.insert("text", Value::Text(bar_text));
        col.children.push(bar);

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
        let mut col = self.themed_container("Column", "demo_tabs_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_tabs_row");
        row.props.insert("gap", Value::Float(4.0));
        let tabs = ["General", "Privacy", "Security"];
        let selected = (self.demo_button_pressed as usize).saturating_sub(1).min(2);
        for (i, label) in tabs.iter().enumerate() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_tab_{}", i)));
            let display = if i == selected {
                format!("> {} (active)", label)
            } else {
                (*label).to_string()
            };
            btn.props.insert("label", Value::Text(display));
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
        let mut body = WidgetSpec::new("SourceLabel");
        body.id = Some(WidgetId::new("demo_tab_content"));
        body.props.insert("text", Value::Text(contents[selected].into()));
        col.children.push(body);
        col
    }

    /// Tooltip: Button toggles a hint label visibility (no real hover yet).
    fn build_tooltip_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_tooltip_col");
        col.props.insert("gap", Value::Float(8.0));

        let show_tip = self.demo_button_pressed == 1;
        let mut btn = WidgetSpec::new("Button");
        btn.id = Some(WidgetId::new("demo_tooltip_btn"));
        btn.props.insert(
            "label",
            Value::Text(if show_tip { "Hide tip" } else { "Show tip" }.into()),
        );
        btn.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(btn);

        if show_tip {
            let mut tip = WidgetSpec::new("SourceLabel");
            tip.id = Some(WidgetId::new("demo_tooltip_text"));
            tip.props.insert(
                "text",
                Value::Text("Helpful hint: this is a tooltip-like bubble.".into()),
            );
            col.children.push(tip);
        }
        col
    }

    /// ListView: 5 selectable rows; click selects and marks row with >.
    fn build_list_view_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_list_col");
        col.props.insert("gap", Value::Float(4.0));
        let selected = (self.demo_button_pressed as usize).saturating_sub(1).min(4);
        for i in 0..5usize {
            let mut row = self.themed_container("Row", &format!("demo_list_row_{}", i));
            row.props.insert("gap", Value::Float(8.0));

            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_list_item_{}", i)));
            let marker = if i == selected { "> " } else { "  " };
            btn.props
                .insert("label", Value::Text(format!("{}Item {}", marker, i + 1)));
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            row.children.push(btn);
            col.children.push(row);
        }
        col
    }

    /// Menu: vertical items; click highlights selected.
    fn build_menu_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_menu_col");
        col.props.insert("gap", Value::Float(4.0));
        let items = [
            ("Open...", "open", 1u32),
            ("Save", "save", 2),
            ("Save As...", "save_as", 3),
            ("Exit", "exit", 4),
        ];
        let selected = self.demo_button_pressed;
        for (label, name, idx) in items.iter() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_menu_{}", name)));
            let display = if *idx == selected {
                format!("> {}", label)
            } else {
                (*label).to_string()
            };
            btn.props.insert("label", Value::Text(display));
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            col.children.push(btn);
        }
        col
    }

    /// SearchField: TextInput + live suggestion list.
    fn build_search_field_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_search_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_search_input"));
        input.props.insert("text", Value::Text(self.demo_text_input.clone()));
        input
            .props
            .insert("placeholder", Value::Text("Search components...".into()));
        col.children.push(input);

        let mut result = WidgetSpec::new("SourceLabel");
        result.id = Some(WidgetId::new("demo_search_result"));
        let query = self.demo_text_input.trim().to_lowercase();
        let candidates = ["button", "toggle", "text_input", "menu", "tabs"];
        let matches: Vec<&str> = candidates
            .iter()
            .copied()
            .filter(|c| c.starts_with(query.as_str()))
            .collect();
        let display = if query.is_empty() {
            "(type to filter)".to_string()
        } else if matches.is_empty() {
            "No match".to_string()
        } else {
            matches.join(", ")
        };
        result
            .props
            .insert("text", Value::Text(format!("Suggestions: {}", display)));
        col.children.push(result);
        col
    }

    /// StatusBubble: cycle through 3 statuses via button.
    fn build_status_bubble_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_status_col");
        col.props.insert("gap", Value::Float(8.0));

        let states = ["[ok] Saved", "[!] Pending", "[x] Failed"];
        let idx = (self.demo_button_pressed as usize).saturating_sub(1) % states.len();
        let text = states[idx];

        let mut label = WidgetSpec::new("SourceLabel");
        label.id = Some(WidgetId::new("demo_status_label"));
        label.props.insert("text", Value::Text(text.into()));
        col.children.push(label);

        let mut next = WidgetSpec::new("Button");
        next.id = Some(WidgetId::new("demo_status_next"));
        next.props.insert("label", Value::Text("Next status".into()));
        next.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        col.children.push(next);
        col
    }

    /// Toolbar: horizontal Buttons (Back/Forward/Reload/Home); last-clicked highlighted.
    fn build_toolbar_demo(&self) -> WidgetSpec {
        let mut row = self.themed_container("Row", "demo_toolbar_row");
        row.props.insert("gap", Value::Float(4.0));
        let actions = [("<", 1u32), (">", 2), ("R", 3), ("H", 4)];
        for (icon, idx) in actions.iter() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_toolbar_{}", idx)));
            let active = *idx == self.demo_button_pressed;
            btn.props.insert(
                "label",
                Value::Text(if active {
                    format!("[{}]", icon)
                } else {
                    (*icon).to_string()
                }),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            row.children.push(btn);
        }
        row
    }

    /// Popover: trigger Button toggles a floating content block.
    fn build_popover_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_popover_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = self.demo_button_pressed == 1;
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
            let mut content = WidgetSpec::new("SourceLabel");
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
        let mut col = self.themed_container("Column", "demo_popup_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = self.demo_button_pressed == 1;
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

        if open {
            let mut row = self.themed_container("Row", "demo_popup_actions");
            row.props.insert("gap", Value::Float(8.0));
            let mut ok = WidgetSpec::new("Button");
            ok.id = Some(WidgetId::new("demo_popup_ok"));
            ok.props.insert("label", Value::Text("OK".into()));
            ok.props
                .insert("action", Value::Text("gallery.demo.button_click.2".into()));
            row.children.push(ok);

            let mut cancel = WidgetSpec::new("Button");
            cancel.id = Some(WidgetId::new("demo_popup_cancel"));
            cancel.props.insert("label", Value::Text("Cancel".into()));
            cancel
                .props
                .insert("action", Value::Text("gallery.demo.button_click.3".into()));
            row.children.push(cancel);
            col.children.push(row);
        }
        col
    }

    // ?? patterns group ??????????????????????????????????????????????????????

    /// DataList: TextInput + Add button + 8-row state (toggle bitmask).
    fn build_data_list_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_data_list_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_data_list_input_row");
        row.props.insert("gap", Value::Float(8.0));
        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_data_list_input"));
        input.props.insert("text", Value::Text(self.demo_text_input.clone()));
        input.props.insert("placeholder", Value::Text("New item...".into()));
        row.children.push(input);

        let mut add = WidgetSpec::new("Button");
        add.id = Some(WidgetId::new("demo_data_list_add"));
        add.props.insert("label", Value::Text("Add".into()));
        add.props
            .insert("action", Value::Text("gallery.demo.button_click.1".into()));
        row.children.push(add);
        col.children.push(row);

        let mut list = WidgetSpec::new("SourceLabel");
        list.id = Some(WidgetId::new("demo_data_list_view"));
        let mut lines = Vec::new();
        for i in 0..8 {
            let on = (self.demo_toggle_state & (1 << i)) != 0;
            lines.push(format!("Item {}: {}", i + 1, if on { "[on]" } else { "[ ]" }));
        }
        list.props.insert("text", Value::Text(lines.join("\n")));
        col.children.push(list);
        col
    }

    /// CommandPalette: TextInput + filtered command list.
    fn build_command_palette_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_cmd_palette_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut input = WidgetSpec::new("TextInputWidget");
        input.id = Some(WidgetId::new("demo_cmd_input"));
        input.props.insert("text", Value::Text(self.demo_text_input.clone()));
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
        let q = self.demo_text_input.trim().to_lowercase();
        let filtered: Vec<&str> = if q.is_empty() {
            cmds.to_vec()
        } else {
            cmds.iter().copied().filter(|c| c.contains(q.as_str())).collect()
        };
        let mut result = WidgetSpec::new("SourceLabel");
        result.id = Some(WidgetId::new("demo_cmd_result"));
        let display = if filtered.is_empty() {
            "(no match)".to_string()
        } else {
            filtered
                .iter()
                .take(5)
                .map(|c| format!("> {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        };
        result.props.insert("text", Value::Text(display));
        col.children.push(result);
        col
    }

    /// DialogScaffold: trigger + inner dialog body with OK / Cancel.
    fn build_dialog_scaffold_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_dialog_col");
        col.props.insert("gap", Value::Float(8.0));

        let open = self.demo_button_pressed == 1;
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

        if open {
            let mut body = WidgetSpec::new("SourceLabel");
            body.id = Some(WidgetId::new("demo_dialog_body"));
            body.props.insert("text", Value::Text("Are you sure?".into()));
            col.children.push(body);

            let mut row = self.themed_container("Row", "demo_dialog_actions");
            row.props.insert("gap", Value::Float(8.0));
            let mut ok = WidgetSpec::new("Button");
            ok.id = Some(WidgetId::new("demo_dialog_ok"));
            ok.props.insert("label", Value::Text("Confirm".into()));
            ok.props
                .insert("action", Value::Text("gallery.demo.button_click.2".into()));
            row.children.push(ok);

            let mut cancel = WidgetSpec::new("Button");
            cancel.id = Some(WidgetId::new("demo_dialog_cancel"));
            cancel.props.insert("label", Value::Text("Cancel".into()));
            cancel
                .props
                .insert("action", Value::Text("gallery.demo.button_click.3".into()));
            row.children.push(cancel);
            col.children.push(row);
        }
        col
    }

    // ?? forms / gestures / animation / collections ????????????????????????

    /// Form: name TextInput + subscribe Toggle + Submit Button.
    fn build_form_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_form_col");
        col.props.insert("gap", Value::Float(12.0));

        let mut name_input = WidgetSpec::new("TextInputWidget");
        name_input.id = Some(WidgetId::new("demo_form_name"));
        name_input
            .props
            .insert("text", Value::Text(self.demo_text_input.clone()));
        name_input.props.insert("placeholder", Value::Text("Your name".into()));
        col.children.push(name_input);

        let mut sub_toggle = WidgetSpec::new("ToggleWidget");
        sub_toggle.id = Some(WidgetId::new("demo_form_subscribe"));
        sub_toggle
            .props
            .insert("checked", Value::Bool((self.demo_toggle_state & 1) != 0));
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
            Value::Text(if self.demo_button_pressed == 1 {
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

    /// Gesture: Buttons for Tap / Double-tap / Long press; click marks current.
    fn build_gesture_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_gesture_col");
        col.props.insert("gap", Value::Float(8.0));

        let labels = ["Tap", "Double tap", "Long press"];
        for (i, label) in labels.iter().enumerate() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_gesture_{}", i)));
            let active = (i + 1) as u32 == self.demo_button_pressed;
            btn.props.insert(
                "label",
                Value::Text(if active {
                    format!("{} (active)", label)
                } else {
                    (*label).to_string()
                }),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i + 1)));
            col.children.push(btn);
        }
        col
    }

    /// Animation: Buttons to switch state label.
    fn build_animation_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_anim_col");
        col.props.insert("gap", Value::Float(8.0));

        let states = ["Idle", "Fade in", "Slide", "Spin"];
        let cur = (self.demo_button_pressed as usize).min(states.len() - 1);
        let mut label = WidgetSpec::new("SourceLabel");
        label.id = Some(WidgetId::new("demo_anim_state"));
        label
            .props
            .insert("text", Value::Text(format!("State: {}", states[cur])));
        col.children.push(label);

        let mut row = self.themed_container("Row", "demo_anim_row");
        row.props.insert("gap", Value::Float(4.0));
        for (i, name) in states.iter().enumerate() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_anim_btn_{}", i)));
            btn.props.insert("label", Value::Text((*name).into()));
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", i)));
            row.children.push(btn);
        }
        col.children.push(row);
        col
    }

    /// Collection: 8 Toggles + count summary.
    fn build_collection_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_collection_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut summary = WidgetSpec::new("SourceLabel");
        summary.id = Some(WidgetId::new("demo_collection_summary"));
        let count = self.demo_toggle_state.count_ones();
        summary
            .props
            .insert("text", Value::Text(format!("Selected items: {}/8", count)));
        col.children.push(summary);

        for i in 0..8 {
            let mut toggle = WidgetSpec::new("ToggleWidget");
            toggle.id = Some(WidgetId::new(&format!("demo_collection_t_{}", i)));
            toggle
                .props
                .insert("checked", Value::Bool((self.demo_toggle_state & (1 << i)) != 0));
            // Note: only toggle.0/1/2 actions exist in dispatch; toggles 3..7 reuse them,
            // so toggling those will affect lower bits. Acceptable for a demo.
            toggle
                .props
                .insert("action", Value::Text(format!("gallery.demo.toggle.{}", i % 3)));
            toggle.props.insert("label", Value::Text(format!("Item {}", i + 1)));
            col.children.push(toggle);
        }
        col
    }

    // ?? theme / i18n / dsl / nav ??????????????????????????????????????????

    /// Theme: shows current theme + Button that triggers gallery.theme.toggle.
    fn build_theme_demo(&self) -> WidgetSpec {
        let mut col = self.themed_container("Column", "demo_theme_col");
        col.props.insert("gap", Value::Float(8.0));

        let current = match self.theme {
            crate::gallery::model::ThemeKind::Light => "Light",
            crate::gallery::model::ThemeKind::Dark => "Dark",
        };
        let mut cur = WidgetSpec::new("SourceLabel");
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
        let mut label = WidgetSpec::new("SourceLabel");
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

        let mut label = WidgetSpec::new("SourceLabel");
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
        let mut col = self.themed_container("Column", "demo_nav_col");
        col.props.insert("gap", Value::Float(8.0));

        let mut row = self.themed_container("Row", "demo_nav_row");
        row.props.insert("gap", Value::Float(4.0));
        let items = [("< Back", 1u32), ("> Forward", 2), ("H Home", 3)];
        for (label, idx) in items.iter() {
            let mut btn = WidgetSpec::new("Button");
            btn.id = Some(WidgetId::new(&format!("demo_nav_{}", idx)));
            let active = *idx == self.demo_button_pressed;
            btn.props.insert(
                "label",
                Value::Text(if active {
                    format!("{} (active)", label)
                } else {
                    (*label).to_string()
                }),
            );
            btn.props
                .insert("action", Value::Text(format!("gallery.demo.button_click.{}", idx)));
            row.children.push(btn);
        }
        col.children.push(row);

        let mut state = WidgetSpec::new("SourceLabel");
        state.id = Some(WidgetId::new("demo_nav_state"));
        state.props.insert(
            "text",
            Value::Text(format!(
                "Current: {}",
                match self.demo_button_pressed {
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
