// 地址栏 UI 渲染（地址栏本体；输入提示/书签栏等其他 chrome 仍在 app_render.rs）。
//
// 从 app_render.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs 同模式；方法保留在 `impl BrowserApp { }` 内，`Self::` 关联
// 函数与 self 字段直接可达。

impl BrowserApp {
    /// 渲染地址栏
    fn render_address_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        _y: f32,
        font_size: f32,
        s: f32,
    ) {
        let (bar_x, bar_y, bar_w, bar_h) = self.address_bar_layout();
        // 聚焦态边框加粗到 2px，强化焦点感知（参考 Chrome）。
        // 窗口失焦时不加粗，避免在非激活窗口留下误导性焦点高亮。
        let border = if self.address_bar_focused && self.window_focused {
            (2.0 * s).max(1.0)
        } else {
            s.max(1.0)
        };
        let radius = bar_h * 0.5;

        let bg = if self.address_bar_focused {
            self.chrome_palette.address_bar_bg_focused
        } else {
            self.chrome_palette.address_bar_bg
        };
        // 窗口失焦时地址栏边框用弱化的 inactive 色（即使地址栏本身处于聚焦态，
        // 只要窗口整体失焦就应弱化焦点感知）。
        let border_color = if !self.window_focused {
            self.chrome_palette.address_bar_border_inactive
        } else if self.address_bar_focused {
            self.chrome_palette.address_bar_border_focused
        } else {
            self.chrome_palette.address_bar_border
        };
        push_rounded_rect_fill(fills, bar_x, bar_y, bar_w, bar_h, radius, border_color);
        push_rounded_rect_fill(
            fills,
            bar_x + border,
            bar_y + border,
            bar_w - 2.0 * border,
            bar_h - 2.0 * border,
            (radius - border).max(0.0),
            bg,
        );

        let text = self.address_bar.text();
        let show_placeholder = text.is_empty() && !self.address_bar_focused && self.address_bar_ime_preedit.is_empty();

        if self.font_id.is_some() {
            let inner_x = bar_x + border;
            let inner_y = bar_y + border;
            let inner_w = bar_w - 2.0 * border;
            let inner_h = bar_h - 2.0 * border;
            let text_x = self.address_bar_text_origin_x();
            let text_pad = layout::ADDRESS_BAR_TEXT_V_PAD * s;
            let (text_top, text_ascent) =
                self.ui_text_top_in_box(inner_y + text_pad, inner_h - 2.0 * text_pad, font_size);
            let status_slot_w = layout::ADDRESS_BAR_LEADING_SLOT_WIDTH * s;
            let status_icon_size = 14.0 * s;
            let status_cx = inner_x + layout::ADDRESS_BAR_INNER_PAD_H * s + status_slot_w * 0.5;
            let status_cy = bar_y + bar_h * 0.5;
            if !self.address_bar_focused {
                fills.push(rect_fill(
                    inner_x + layout::ADDRESS_BAR_INNER_PAD_H * s + status_slot_w,
                    inner_y + 6.0 * s,
                    s.max(1.0),
                    (inner_h - 12.0 * s).max(s.max(1.0)),
                    self.chrome_palette.separator,
                ));
            }

            let status_url = self.shell.active_tab().and_then(|tab| tab.url());
            let status_hint = Self::tab_html_hint(status_url);
            let page_kind = Self::address_bar_page_kind(status_url);
            let is_loading = self.shell.active_tab().is_some_and(|t| t.is_loading());
            if self.address_bar_focused {
                // 地址栏编辑态固定显示 ZeroWeb 标识，避免安全状态位留下空白。
                push_zero_web_icon(fills, status_cx, status_cy, status_icon_size, self.chrome_palette.address_bar_bg_focused);
            } else if is_loading {
                let angle = self.chrome_anim_start.elapsed().as_secs_f32() * 3.5;
                crate::tab_chrome::push_loading_spinner(
                    fills,
                    status_cx,
                    status_cy,
                    status_icon_size,
                    angle,
                    self.chrome_palette.loading_indicator,
                );
            } else if !self.address_bar_focused {
                let status_label = match page_kind {
                    AddressBarPageKind::Secure => None,
                    AddressBarPageKind::Insecure => Some(("!", self.chrome_palette.address_bar_insecure)),
                    AddressBarPageKind::Internal | AddressBarPageKind::Local => {
                        Some(("i", self.chrome_palette.address_bar_internal))
                    }
                    _ => None,
                };
                if page_kind == AddressBarPageKind::Secure {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Lock,
                        status_cx,
                        status_cy,
                        status_icon_size,
                        self.chrome_palette.address_bar_secure,
                    );
                } else if let Some((label, color)) = status_label {
                    let label_w = self.measure_ui_text_width(label, status_icon_size * 0.85);
                    self.draw_ui_text(
                        label,
                        status_cx - label_w * 0.5,
                        text_top,
                        status_icon_size * 0.85,
                        color,
                        glyphs,
                    );
                }
                // favicon 绘制策略：
                // - Secure：已画 Lock，不画 favicon。
                // - Insecure：已画警告标记，且不安全页不应让 favicon 抢占安全语义 → 不画 favicon。
                // - Internal/Local：已画 i 标记，不再叠加 favicon（标签上已显示 favicon）。
                // - Unknown（非 http/https/file/zero）：画 favicon 作为站点识别。
                if matches!(page_kind, AddressBarPageKind::Unknown) {
                    if let Some(tab_id) = self.shell.active_tab_id() {
                        crate::tab_favicon::render_tab_favicon(
                            &mut self.font_loader,
                            glyphs,
                            tab_id,
                            status_url,
                            status_hint,
                            status_cx,
                            status_cy,
                            status_icon_size,
                            self.chrome_palette.page_url,
                        );
                    }
                }
            }

            if self.address_bar_focused && self.address_bar.has_selection() {
                let (sel_start, sel_end) = self.address_bar.selection_char_range();
                let before = Self::chars_slice(text, 0, sel_start);
                let selected = Self::chars_slice(text, sel_start, sel_end);
                let sel_x = text_x + self.measure_ui_text_width(before, font_size);
                let sel_w = self.measure_ui_text_width(selected, font_size).max(1.0);
                let (_, descent) = self.ui_line_metrics(font_size);
                let selection_h = text_ascent - descent;
                fills.push(rect_fill(
                    sel_x,
                    text_top,
                    sel_w,
                    selection_h,
                    self.chrome_palette.address_bar_selection_bg,
                ));
            }

            let color = if show_placeholder {
                self.chrome_palette.address_bar_placeholder
            } else {
                self.chrome_palette.address_bar_text
            };
            let visible = if show_placeholder {
                if self.address_bar_focused && Self::looks_like_search_query(text) {
                    let engine = match self.shell.settings().search_engine {
                        zero_browser_shell::SearchEngine::Google => "Google",
                        zero_browser_shell::SearchEngine::Bing => "Bing",
                        zero_browser_shell::SearchEngine::DuckDuckGo => "DuckDuckGo",
                        zero_browser_shell::SearchEngine::Baidu => "Baidu",
                    };
                    format!("Search {engine} or enter URL...")
                } else {
                    "Search or enter URL...".to_string()
                }
            } else {
                text.to_string()
            };
            let available_text_w = (inner_x + inner_w
                - layout::ADDRESS_BAR_TRAILING_PAD * s
                - layout::ADDRESS_BAR_TRAILING_SLOTS * s
                - text_x)
                .max(0.0);
            let visible = self.truncate_ui_text(&visible, available_text_w, font_size);
            self.draw_ui_text(&visible, text_x, text_top, font_size, color, glyphs);

            if !self.address_bar_ime_preedit.is_empty() {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let preedit_x = text_x + self.measure_ui_text_width(before, font_size);
                self.draw_ui_text(
                    &self.address_bar_ime_preedit,
                    preedit_x,
                    text_top,
                    font_size,
                    self.chrome_palette.address_bar_text,
                    glyphs,
                );
            }

            let cursor_visible = (self.chrome_anim_start.elapsed().as_millis() / 500) & 1 == 0;
            if self.address_bar_focused
                && !self.address_bar.has_selection()
                && self.address_bar_ime_preedit.is_empty()
                && cursor_visible
            {
                let before = Self::chars_slice(text, 0, self.address_bar.cursor());
                let cursor_x = text_x + self.measure_ui_text_width(before, font_size);
                fills.push(rect_fill(
                    cursor_x,
                    text_top + text_ascent * 0.12,
                    1.5 * s,
                    text_ascent * 0.76,
                    self.chrome_palette.address_bar_text,
                ));
            }
            if self.address_bar_focused {
                self.needs_redraw = true;
            }

            let trailing_slots_w = layout::ADDRESS_BAR_TRAILING_SLOTS * s;
            let slots_x = inner_x + inner_w - layout::ADDRESS_BAR_TRAILING_PAD * s - trailing_slots_w;
            let slot_w = layout::ADDRESS_BAR_ACTION_SLOT_WIDTH * s;
            let current_bookmarked = self.shell.is_current_page_bookmarked();
            let star_icon = if current_bookmarked {
                crate::ui_icons::Icon::StarFilled
            } else {
                crate::ui_icons::Icon::Star
            };
            let star_color = if current_bookmarked {
                self.chrome_palette.tab_attention
            } else {
                self.chrome_palette.nav_button
            };
            let slot_specs = [
                (star_icon, star_color),
                (crate::ui_icons::Icon::Shield, self.chrome_palette.page_hint),
            ];
            for (index, (icon, base_color)) in slot_specs.iter().enumerate() {
                let slot_x = slots_x + index as f32 * slot_w;
                let slot_cx = slot_x + slot_w * 0.5;
                let slot_cy = bar_y + bar_h * 0.5;
                let slot_hovered = self.pointer_in_rect(slot_x, inner_y, slot_w, inner_h);
                if slot_hovered {
                    push_circle_fill(
                        fills,
                        slot_cx,
                        slot_cy,
                        layout::NAV_BUTTON_HOVER_DIAMETER * s,
                        self.chrome_palette.tab_hover_bg,
                    );
                }
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    *icon,
                    slot_cx,
                    slot_cy,
                    layout::CHROME_ICON_SIZE * s,
                    if slot_hovered {
                        self.chrome_palette.address_bar_text
                    } else {
                        *base_color
                    },
                );
            }
        }

        if self.font_id.is_some() {
            let (dl_x, dl_y, dl_w, dl_h) = self.toolbar_download_button_rect();
            let dl_cx = dl_x + dl_w * 0.5;
            let dl_cy = dl_y + dl_h * 0.5;
            let dl_hovered = self.pointer_in_rect(dl_x, dl_y, dl_w, dl_h);
            let dl_pressed = dl_hovered && self.left_button_down;
            if dl_pressed {
                push_circle_fill(
                    fills,
                    dl_cx,
                    dl_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.nav_button_pressed,
                );
            } else if dl_hovered {
                push_circle_fill(
                    fills,
                    dl_cx,
                    dl_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                crate::ui_icons::Icon::Download,
                dl_cx,
                dl_cy,
                layout::CHROME_ICON_SIZE * s,
                if dl_hovered {
                    self.chrome_palette.address_bar_text
                } else {
                    self.chrome_palette.nav_button
                },
            );
            if self.shell.downloads().active_count() > 0 {
                push_circle_fill(
                    fills,
                    dl_x + dl_w - 6.0 * s,
                    dl_y + 6.0 * s,
                    4.0 * s,
                    self.chrome_palette.tab_attention,
                );
            }

            let (theme_x, theme_y, theme_w, theme_h) = self.toolbar_theme_button_rect();
            let theme_cx = theme_x + theme_w * 0.5;
            let theme_cy = theme_y + theme_h * 0.5;
            let theme_hovered = self.pointer_in_rect(theme_x, theme_y, theme_w, theme_h);
            let theme_pressed = theme_hovered && self.left_button_down;
            if theme_pressed {
                push_circle_fill(
                    fills,
                    theme_cx,
                    theme_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.nav_button_pressed,
                );
            } else if theme_hovered {
                push_circle_fill(
                    fills,
                    theme_cx,
                    theme_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let theme_icon = self.theme_button_icon();
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                theme_icon,
                theme_cx,
                theme_cy,
                layout::CHROME_ICON_SIZE * s,
                if theme_hovered {
                    self.chrome_palette.address_bar_text
                } else {
                    self.chrome_palette.nav_button
                },
            );
        }

        let (menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h) = self.toolbar_menu_button_rect();
        let menu_btn_cx = menu_btn_x + menu_btn_w * 0.5;
        let menu_btn_cy = menu_btn_y + menu_btn_h * 0.5;
        let menu_hovered = self.pointer_in_rect(menu_btn_x, menu_btn_y, menu_btn_w, menu_btn_h);
        let menu_pressed = menu_hovered && self.left_button_down;
        if menu_pressed {
            push_circle_fill(
                fills,
                menu_btn_cx,
                menu_btn_cy,
                layout::NAV_BUTTON_HOVER_DIAMETER * s,
                self.chrome_palette.nav_button_pressed,
            );
        } else if menu_hovered {
            push_circle_fill(
                fills,
                menu_btn_cx,
                menu_btn_cy,
                layout::NAV_BUTTON_HOVER_DIAMETER * s,
                self.chrome_palette.tab_hover_bg,
            );
        }
        crate::ui_icons::render_icon(
            &mut self.font_loader,
            glyphs,
            crate::ui_icons::Icon::MoreVertical,
            menu_btn_cx,
            menu_btn_cy,
            layout::CHROME_ICON_SIZE * s,
            if menu_hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.nav_button
            },
        );
    }
}
