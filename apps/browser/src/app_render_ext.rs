// 浏览器 chrome 浮层 / 装饰渲染方法（从 app_render.rs 拆分）。
// 拆分目的：app_render.rs 单文件 ≤2000 行合规（DC-16）+ chrome 渲染模块化（DC-14 替换式迁移 prep）。
// 经 app.rs `include!` 与 app_render.rs 共享同一模块作用域，零行为变化（纯代码搬迁）。

impl BrowserApp {
    /// 渲染浮动查找栏
    fn render_find_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (bar_x, y, bar_w, bar_h) = self.find_bar_rect_for(width, height);
        let radius = layout::FIND_BAR_FLOAT_RADIUS * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, bar_x, y, bar_w, bar_h, radius, self.chrome_palette.find_bar_border);
        push_rounded_rect_fill(
            fills,
            bar_x + border,
            y + border,
            bar_w - 2.0 * border,
            bar_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.find_bar_bg,
        );

        let display = if self.find_input.is_empty() {
            "Find...".to_string()
        } else {
            self.find_input.clone()
        };
        let text_color = if self.find_input.is_empty() {
            self.chrome_palette.find_match_text
        } else {
            self.chrome_palette.find_bar_text
        };
        self.draw_ui_text(
            &display,
            bar_x + 12.0 * s,
            y + (bar_h - font_size) * 0.5,
            font_size,
            text_color,
            glyphs,
        );

        let find_state = self.shell.find_state();
        // 选项切换按钮（区分大小写 / 全字匹配）位置，位于匹配数与 prev 按钮之间。
        let case_btn_x = bar_x + bar_w - 160.0 * s;
        let whole_btn_x = bar_x + bar_w - 130.0 * s;
        if find_state.total_matches() > 0 {
            // 色阶：当前项用主文本色强调，分隔符与总数用次要色，符合 Chrome 习惯。
            let current_str = find_state.current_match().to_string();
            let rest_str = format!("/{}", find_state.total_matches());
            let match_x = bar_x + bar_w - 200.0 * s;
            let match_y = y + (bar_h - font_size) * 0.5;
            let current_w = self.measure_ui_text_width(&current_str, font_size);
            self.draw_ui_text(&current_str, match_x, match_y, font_size, self.chrome_palette.find_bar_text, glyphs);
            self.draw_ui_text(
                &rest_str,
                match_x + current_w,
                match_y,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
            // 循环提示后缀
            if let Some(wrap) = find_state.last_wrap() {
                let suffix = match wrap {
                    FindWrapHint::WrappedToStart => "  ↻",
                    FindWrapHint::WrappedToEnd => "  ↺",
                };
                let rest_w = self.measure_ui_text_width(&rest_str, font_size);
                self.draw_ui_text(
                    suffix,
                    match_x + current_w + rest_w,
                    match_y,
                    font_size,
                    self.chrome_palette.find_match_text,
                    glyphs,
                );
            }
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 200.0 * s;
            self.draw_ui_text(
                "No matches",
                no_match_x,
                y + (bar_h - font_size) * 0.5,
                font_size,
                self.chrome_palette.find_match_text,
                glyphs,
            );
        }

        // 选项切换按钮：激活态用主色背景 + 白字，未激活态用次要文字。
        let opt_btn_w = 28.0 * s;
        let opt_font = font_size * 0.82;
        let opt_y = y + (bar_h - opt_font) * 0.5;
        for (bx, label, active) in [
            (case_btn_x, "Aa", find_state.case_sensitive()),
            (whole_btn_x, "W•", find_state.whole_word()),
        ] {
            let hovered = self.pointer_in_rect(bx, y, opt_btn_w, bar_h);
            if active {
                push_rounded_rect_fill(
                    fills,
                    bx + 2.0 * s,
                    y + 3.0 * s,
                    opt_btn_w - 4.0 * s,
                    bar_h - 6.0 * s,
                    4.0 * s,
                    self.chrome_palette.find_active_option_bg,
                );
            } else if hovered {
                push_circle_fill(
                    fills,
                    bx + opt_btn_w / 2.0,
                    y + bar_h / 2.0,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let color = if active {
                self.chrome_palette.find_active_option_text
            } else if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.find_match_text
            };
            let lw = self.measure_ui_text_width(label, opt_font);
            self.draw_ui_text(label, bx + (opt_btn_w - lw) / 2.0, opt_y, opt_font, color, glyphs);
        }

        let btn_y = y + (bar_h - font_size) * 0.5;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
        let btn_w = 24.0 * s;
        let icon_size = 16.0 * s;
        let btn_cy = btn_y + font_size * 0.5;

        for (bx, icon) in [
            (prev_x, crate::ui_icons::Icon::ChevronUp),
            (next_x, crate::ui_icons::Icon::ChevronDown),
            (close_x, crate::ui_icons::Icon::Close),
        ] {
            let icon_cx = bx + btn_w / 2.0;
            let hovered = self.pointer_in_rect(bx, y, btn_w, bar_h);
            if hovered {
                // 图标按钮类统一使用圆形 hover（与导航按钮、地址栏 slot 一致）。
                push_circle_fill(
                    fills,
                    icon_cx,
                    btn_cy,
                    layout::NAV_BUTTON_HOVER_DIAMETER * s,
                    self.chrome_palette.tab_hover_bg,
                );
            }
            let icon_color = if hovered {
                self.chrome_palette.address_bar_text
            } else {
                self.chrome_palette.find_bar_text
            };
            crate::ui_icons::render_icon(
                &mut self.font_loader,
                glyphs,
                icon,
                icon_cx,
                btn_cy,
                icon_size,
                icon_color,
            );
        }
    }

    /// 渲染自动补全下拉
    fn render_autocomplete(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (bar_x, _, bar_w, _) = self.address_bar_layout();
        let dropdown_y = layout::TOOLBAR_HEIGHT * s;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let dropdown_h = visible_count as f32 * row_h;
        let radius = layout::AUTOCOMPLETE_DROPDOWN_RADIUS * s;
        let pad_h = layout::AUTOCOMPLETE_ROW_PAD_H * s;
        let pad_v = layout::AUTOCOMPLETE_ROW_PAD_V * s;
        let title_size = font_size * 0.92;
        let url_size = font_size * 0.78;

        push_rounded_rect_fill(
            fills,
            bar_x,
            dropdown_y,
            bar_w,
            dropdown_h,
            radius,
            self.chrome_palette.separator,
        );
        push_rounded_rect_fill(
            fills,
            bar_x + s.max(1.0),
            dropdown_y + s.max(1.0),
            bar_w - 2.0 * s.max(1.0),
            dropdown_h - 2.0 * s.max(1.0),
            (radius - s.max(1.0)).max(0.0),
            self.chrome_palette.autocomplete_bg,
        );

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * row_h;
            let is_hovered = self.autocomplete.hovered_index == Some(i);
            let is_selected = self.autocomplete.hovered_index.is_none() && self.autocomplete.selected_index == Some(i);
            if is_hovered {
                fills.push(rect_fill(
                    bar_x + s.max(1.0),
                    row_y,
                    bar_w - 2.0 * s.max(1.0),
                    row_h,
                    self.chrome_palette.autocomplete_hover_bg,
                ));
            } else if is_selected {
                fills.push(rect_fill(
                    bar_x + s.max(1.0),
                    row_y,
                    bar_w - 2.0 * s.max(1.0),
                    row_h,
                    self.chrome_palette.autocomplete_selected_bg,
                ));
            }

            // 来源标记：图标化（搜索=放大镜，书签=实心星，历史=时钟字符）
            let icon_size = font_size * 0.95;
            let icon_cx = bar_x + pad_h + icon_size * 0.5;
            let icon_cy = row_y + row_h * 0.5;
            let text_x = bar_x + pad_h + icon_size + 10.0 * s;
            let title_area_w = (bar_w - (text_x - bar_x) - pad_h).max(0.0);

            match sug.source() {
                SuggestionSource::Search => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Search,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_url,
                    );
                    // 主标题：搜索词（深色，加粗感由字号体现）
                    let title = sug.title();
                    let truncated = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    // 副标题："<query> — 在 <Engine> 中搜索"（灰色）
                    let engine_name = self.shell.settings().search_engine.display_name();
                    let hint = format!("{truncated} — {engine_name} 搜索");
                    let truncated_hint = self.truncate_ui_text(&hint, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_hint,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
                SuggestionSource::Bookmark => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::StarFilled,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_bookmark,
                    );
                    let title = sug.title();
                    let truncated_title = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated_title,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    let url = sug.url();
                    let truncated_url = self.truncate_ui_text(url, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_url,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
                SuggestionSource::History => {
                    crate::ui_icons::render_icon(
                        &mut self.font_loader,
                        glyphs,
                        crate::ui_icons::Icon::Clock,
                        icon_cx,
                        icon_cy,
                        icon_size,
                        self.chrome_palette.autocomplete_url,
                    );
                    let title = sug.title();
                    let truncated_title = self.truncate_ui_text(title, title_area_w, title_size);
                    self.draw_ui_text(
                        &truncated_title,
                        text_x,
                        row_y + pad_v,
                        title_size,
                        self.chrome_palette.autocomplete_text,
                        glyphs,
                    );
                    let url = sug.url();
                    let truncated_url = self.truncate_ui_text(url, title_area_w, url_size);
                    self.draw_ui_text(
                        &truncated_url,
                        text_x,
                        row_y + pad_v + title_size + 2.0 * s,
                        url_size,
                        self.chrome_palette.autocomplete_url,
                        glyphs,
                    );
                }
            }
        }
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&mut self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        if self.font_id.is_none() {
            return;
        }

        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let menu_w = layout::CONTEXT_MENU_WIDTH * s;
        let menu_h = self.context_menu_total_height();
        let font_size = layout::CHROME_FONT_SIZE * s;
        let radius = layout::CONTEXT_MENU_RADIUS * s;
        let border = s.max(1.0);
        let pad_h = layout::CONTEXT_MENU_PAD_H * s;

        push_rounded_rect_fill(fills, menu_x, menu_y, menu_w, menu_h, radius, self.chrome_palette.context_menu_separator);
        push_rounded_rect_fill(
            fills,
            menu_x + border,
            menu_y + border,
            menu_w - 2.0 * border,
            menu_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.context_menu_bg,
        );

        for (i, item) in self.context_menu.items.iter().enumerate() {
            let row_y = menu_y + self.context_menu_row_y(i);

            if item.is_separator() {
                // separator 占紧凑高度，绘制居中的细线。
                let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
                let sep_y = row_y + sep_h / 2.0;
                fills.push(rect_fill(
                    menu_x + pad_h,
                    sep_y,
                    menu_w - 2.0 * pad_h,
                    border,
                    self.chrome_palette.context_menu_separator,
                ));
                continue;
            }

            let is_hovered = self.context_menu.hovered_index == Some(i);
            let is_disabled = !item.enabled();
            let is_sub_open = self.context_menu.open_sub_menu == Some(i);

            // hover 或子菜单展开时高亮（submenu 父项展开时保持高亮）。
            if (is_hovered || is_sub_open) && !is_disabled {
                fills.push(rect_fill(
                    menu_x + border,
                    row_y,
                    menu_w - 2.0 * border,
                    row_h,
                    self.chrome_palette.context_menu_hover_bg,
                ));
            }

            let text_color = if is_disabled {
                self.chrome_palette.page_hint
            } else {
                self.chrome_palette.context_menu_text
            };

            let label = {
                let raw = item.label();
                if !item.is_sub_menu() && item.shortcut().is_some() {
                    let shortcut_w = self.measure_ui_text_width(item.shortcut().unwrap_or(""), font_size);
                    let max_w = menu_w - 2.0 * pad_h - shortcut_w - 8.0 * s;
                    self.truncate_ui_text(raw, max_w, font_size)
                } else {
                    raw.to_string()
                }
            };
            self.draw_ui_text(
                &label,
                menu_x + pad_h,
                row_y + (row_h - font_size) * 0.5,
                font_size,
                text_color,
                glyphs,
            );

            // 快捷键提示（右对齐，子菜单项不显示以免与箭头冲突）。
            if !item.is_sub_menu() {
                if let Some(shortcut) = item.shortcut() {
                    let w = self.measure_ui_text_width(shortcut, font_size);
                    let x = menu_x + menu_w - pad_h - w;
                    self.draw_ui_text(
                        shortcut,
                        x,
                        row_y + (row_h - font_size) * 0.5,
                        font_size,
                        self.chrome_palette.page_hint,
                        glyphs,
                    );
                }
            }

            // 子菜单右侧箭头。
            if item.is_sub_menu() {
                let arrow_cx = menu_x + menu_w - pad_h - 6.0 * s;
                crate::ui_icons::render_icon(
                    &mut self.font_loader,
                    glyphs,
                    crate::ui_icons::Icon::ChevronRight,
                    arrow_cx,
                    row_y + row_h * 0.5,
                    12.0 * s,
                    text_color,
                );
            }
        }

        // 渲染展开的子菜单面板（默认右侧浮层；右侧空间不足时自动翻转到左侧）。
        if let Some(parent_idx) = self.context_menu.open_sub_menu
            && let Some(parent) = self.context_menu.items.get(parent_idx)
            && let Some(children) = parent.children()
            && !children.is_empty()
        {
            let (sub_x, sub_y, sub_w, sub_h) = self.sub_menu_panel_rect(parent_idx);
            push_rounded_rect_fill(fills, sub_x, sub_y, sub_w, sub_h, radius, self.chrome_palette.context_menu_separator);
            push_rounded_rect_fill(
                fills,
                sub_x + border,
                sub_y + border,
                sub_w - 2.0 * border,
                sub_h - 2.0 * border,
                (radius - border).max(0.0),
                self.chrome_palette.context_menu_bg,
            );

            for (ci, child) in children.iter().enumerate() {
                let crow_y = sub_y + self.sub_menu_row_y(parent_idx, ci);
                if child.is_separator() {
                    let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
                    let sep_y = crow_y + sep_h / 2.0;
                    fills.push(rect_fill(
                        sub_x + pad_h,
                        sep_y,
                        sub_w - 2.0 * pad_h,
                        border,
                        self.chrome_palette.context_menu_separator,
                    ));
                    continue;
                }
                let c_hovered = self.context_menu.sub_menu_hovered == Some(ci);
                let c_disabled = !child.enabled();
                if c_hovered && !c_disabled {
                    fills.push(rect_fill(
                        sub_x + border,
                        crow_y,
                        sub_w - 2.0 * border,
                        row_h,
                        self.chrome_palette.context_menu_hover_bg,
                    ));
                }
                let c_color = if c_disabled {
                    self.chrome_palette.page_hint
                } else {
                    self.chrome_palette.context_menu_text
                };
                self.draw_ui_text(
                    child.label(),
                    sub_x + pad_h,
                    crow_y + (row_h - font_size) * 0.5,
                    font_size,
                    c_color,
                    glyphs,
                );
            }
        }
    }

    /// 渲染链接悬停浮动状态栏（Chrome 风格：左下角胶囊，宽度随 URL 内容）
    fn render_floating_link_status(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        s: f32,
    ) {
        let Some(url) = self.hovered_link_url.as_deref() else {
            return;
        };
        if self.font_id.is_none() {
            return;
        }

        let (cx, cy, cw, ch) = self.page_content_rect_for(width, height);
        if cw <= f32::EPSILON || ch <= f32::EPSILON {
            return;
        }

        let margin = layout::STATUS_BAR_FLOAT_MARGIN * s;
        let pad_h = layout::STATUS_BAR_FLOAT_PAD_H * s;
        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let font_size = 11.0 * s;
        let max_text_w = (cw - 2.0 * margin - 2.0 * pad_h).max(0.0);
        let text = self.truncate_ui_text(url, max_text_w, font_size);
        let text_w = self.measure_ui_text_width(&text, font_size);
        let pill_w = text_w + 2.0 * pad_h;
        let pill_x = cx + margin;
        let pill_y = cy + ch - status_h - margin;
        let radius = layout::STATUS_BAR_FLOAT_RADIUS * s;
        let border = s;

        push_rounded_rect_fill(
            fills,
            pill_x,
            pill_y,
            pill_w,
            status_h,
            radius,
            self.chrome_palette.separator,
        );
        push_rounded_rect_fill(
            fills,
            pill_x + border,
            pill_y + border,
            (pill_w - 2.0 * border).max(0.0),
            (status_h - 2.0 * border).max(0.0),
            (radius - border).max(0.0),
            self.chrome_palette.tab_active_bg,
        );

        self.draw_ui_text(
            &text,
            pill_x + pad_h,
            pill_y + 3.0 * s,
            font_size,
            self.chrome_palette.status_text,
            glyphs,
        );
    }

    /// fontdue 行 metrics；无字体时回退为 `(font_size, 0)`。
    fn ui_line_metrics(&self, font_size: f32) -> (f32, f32) {
        let Some(primary) = self.font_id else {
            return (font_size, 0.0);
        };
        self.font_loader
            .line_metrics(primary, font_size)
            .unwrap_or((font_size, 0.0))
    }

    /// 在给定高度内垂直居中 UI 文本，返回 `(text_top, baseline_y)`。
    fn ui_text_centered_in_height(&self, height: f32, font_size: f32) -> (f32, f32) {
        let (text_top, ascent) = self.ui_text_top_in_box(0.0, height, font_size);
        (text_top, text_top + ascent)
    }

    /// 在给定矩形高度内垂直居中 UI 文本，返回 `(text_top, ascent)`。
    fn ui_text_top_in_box(&self, box_y: f32, box_h: f32, font_size: f32) -> (f32, f32) {
        let (ascent, descent) = self.ui_line_metrics(font_size);
        let line_h = ascent - descent;
        let text_top = box_y + (box_h - line_h) / 2.0;
        (text_top, ascent)
    }

    /// 绘制 UI 文本（使用字体回退链和真实 advance 宽度）
    fn draw_ui_text(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        font_size: f32,
        color: Color,
        glyphs: &mut Vec<GlyphDraw>,
    ) {
        let Some(primary) = self.font_id else {
            return;
        };
        let (ascent, _) = self.ui_line_metrics(font_size);
        let baseline_y = start_y + ascent;
        let mut x = start_x;
        for ch in text.chars() {
            let font_id = self
                .font_loader
                .rasterize_glyph_with_fallback(primary, ch, font_size)
                .map(|(id, _)| id)
                .unwrap_or(primary);
            glyphs.push(GlyphDraw {
                ch,
                x,
                baseline_y,
                color,
                font_id,
                font_size,
            });
            x += self.font_loader.measure_advance(primary, ch, font_size);
        }
    }

    /// 测量 UI 文本总宽度
    fn measure_ui_text_width(&self, text: &str, font_size: f32) -> f32 {
        let Some(primary) = self.font_id else {
            return 0.0;
        };
        text.chars()
            .map(|ch| self.font_loader.measure_advance(primary, ch, font_size))
            .sum()
    }

    /// 按像素宽度截断 UI 文本
    fn truncate_ui_text(&self, text: &str, max_width: f32, font_size: f32) -> String {
        let Some(primary) = self.font_id else {
            return text.to_string();
        };
        let mut result = String::new();
        let mut width = 0.0;
        let ellipsis_advance = self.font_loader.measure_advance(primary, '…', font_size);
        for ch in text.chars() {
            let advance = self.font_loader.measure_advance(primary, ch, font_size);
            if width + advance + ellipsis_advance > max_width && !result.is_empty() {
                result.push('…');
                break;
            }
            result.push(ch);
            width += advance;
        }
        result
    }

    /// 渲染浮动下载面板（右下角）
    fn render_download_panel(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        _font_size: f32,
        s: f32,
    ) {
        if self.font_id.is_none() {
            return;
        }

        let (panel_x, panel_y, panel_w, panel_h) = self.download_panel_rect_for(width, height);
        let radius = layout::DOWNLOAD_PANEL_RADIUS * s;
        let border = s.max(1.0);

        push_rounded_rect_fill(fills, panel_x, panel_y, panel_w, panel_h, radius, self.chrome_palette.separator);
        push_rounded_rect_fill(
            fills,
            panel_x + border,
            panel_y + border,
            panel_w - 2.0 * border,
            panel_h - 2.0 * border,
            (radius - border).max(0.0),
            self.chrome_palette.download_bar_bg,
        );

        let downloads = self.shell.downloads();
        let active: Vec<_> = downloads.iter().filter(|d| d.is_active()).collect();
        if let Some(dl) = active.first() {
            let font_size = 11.0 * s;
            let title_size = 12.0 * s;

            self.draw_ui_text(
                "Downloading",
                panel_x + 12.0 * s,
                panel_y + 10.0 * s,
                title_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );

            let name_text = dl.filename();
            self.draw_ui_text(
                name_text,
                panel_x + 12.0 * s,
                panel_y + 28.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );

            let progress = dl.progress();
            let bar_width = panel_w - 24.0 * s;
            let bar_start_x = panel_x + 12.0 * s;
            let bar_top = panel_y + panel_h - 18.0 * s;
            let bar_inner_h = 6.0 * s;

            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width,
                bar_inner_h,
                self.chrome_palette.separator,
            ));
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width * progress,
                bar_inner_h,
                self.chrome_palette.download_bar_fill,
            ));

            let pct_text = format!("{:.0}%", progress * 100.0);
            let pct_w = self.measure_ui_text_width(&pct_text, font_size);
            self.draw_ui_text(
                &pct_text,
                panel_x + panel_w - 12.0 * s - pct_w,
                panel_y + 10.0 * s,
                font_size,
                self.chrome_palette.download_bar_text,
                glyphs,
            );
        }
    }
}

/// 从渲染图元估算文档高度（CSS 逻辑像素，fills + glyphs 下界）。
pub fn primitives_content_height(primitives: &RenderPrimitives) -> f32 {
    crate::page_scroll::primitives_content_height(primitives)
}

/// 将 WebView 输出的基础图元追加到浏览器场景。
///
/// `clip_y` 为物理像素坐标 `(top, bottom)`，fill 与该区间求交后绘制，glyph 完全落在区间外则跳过。
/// `clip_rounded` 为 `(x, y, w, h, radius)`，将内容裁剪到圆角矩形内。
#[allow(clippy::too_many_arguments)]
pub fn append_webview_primitives(
    primitives: &RenderPrimitives,
    fills: &mut Vec<FillPrimitive>,
    glyphs: &mut Vec<GlyphDraw>,
    x_offset: f32,
    y_offset: f32,
    fallback_font_id: u32,
    s: f32,
    clip_y: Option<(f32, f32)>,
    clip_rounded: Option<(f32, f32, f32, f32, f32)>,
) -> bool {
    let fill_start = fills.len();
    let glyph_start = glyphs.len();

    for fill in &primitives.fills {
        let x = fill.rect.origin.x * s + x_offset;
        let mut y = fill.rect.origin.y * s + y_offset;
        let w = fill.rect.size.width * s;
        let mut h = fill.rect.size.height * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let bottom = y + h;
            if bottom <= clip_top || y >= clip_bottom {
                continue;
            }
            if y < clip_top {
                h -= clip_top - y;
                y = clip_top;
            }
            let bottom = y + h;
            if bottom > clip_bottom {
                h -= bottom - clip_bottom;
            }
            if h <= 0.0 {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            push_fill_clipped_to_rounded_rect(fills, x, y, w, h, fill.color, rx, ry, rw, rh, radius);
        } else {
            let mut translated = fill.clone();
            translated.rect.origin.x = x;
            translated.rect.origin.y = y;
            translated.rect.size.width = w;
            translated.rect.size.height = h;
            fills.push(translated);
        }
    }

    for glyph in &primitives.glyphs {
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        let x = glyph.x * s + x_offset;
        let baseline_y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some((clip_top, clip_bottom)) = clip_y {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            if bottom <= clip_top || top >= clip_bottom || top < clip_top {
                continue;
            }
        }
        if let Some((rx, ry, rw, rh, radius)) = clip_rounded {
            let top = baseline_y - font_size;
            let bottom = baseline_y + font_size * 0.25;
            let width = font_size * 0.6;
            if !axis_rect_intersects_rounded_rect(x, top, width, bottom - top, rx, ry, rw, rh, radius) {
                continue;
            }
        }
        glyphs.push(GlyphDraw {
            ch,
            x,
            baseline_y,
            color: glyph.color,
            font_id: if glyph.font_id.0 == 0 {
                fallback_font_id
            } else {
                glyph.font_id.0
            },
            font_size,
        });
    }

    fills.len() > fill_start || glyphs.len() > glyph_start
}

/// 页面视口裁剪区（物理像素）。
#[derive(Debug, Clone, Copy)]
pub struct ViewportClip {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl ViewportClip {
    /// 由 `(x, y, w, h)` 构造视口裁剪矩形。
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + w,
            bottom: y + h,
        }
    }

    fn excludes(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x + w <= self.left || x >= self.right || y + h <= self.top || y >= self.bottom
    }
}

fn clip_axis_aligned_rect(x: f32, y: f32, w: f32, h: f32, clip: ViewportClip) -> Option<(f32, f32, f32, f32)> {
    if clip.excludes(x, y, w, h) {
        return None;
    }
    let left = x.max(clip.left);
    let top = y.max(clip.top);
    let right = (x + w).min(clip.right);
    let bottom = (y + h).min(clip.bottom);
    let w = right - left;
    let h = bottom - top;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some((left, top, w, h))
}

fn clamp_rounded_rect_radii(rr: &mut zero_render_foundation::primitive::RoundedRectPrimitive) {
    let max_r = rr.rect.size.width.min(rr.rect.size.height) * 0.5;
    rr.top_left_radius = rr.top_left_radius.min(max_r);
    rr.top_right_radius = rr.top_right_radius.min(max_r);
    rr.bottom_right_radius = rr.bottom_right_radius.min(max_r);
    rr.bottom_left_radius = rr.bottom_left_radius.min(max_r);
}

fn clip_rect_field(rect: &mut Rect, clip: ViewportClip) -> bool {
    let Some((x, y, w, h)) =
        clip_axis_aligned_rect(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height, clip)
    else {
        return false;
    };
    rect.origin.x = x;
    rect.origin.y = y;
    rect.size.width = w;
    rect.size.height = h;
    true
}

fn path_vertices_bbox(vertices: &[f32]) -> Option<(f32, f32, f32, f32)> {
    if vertices.len() < 2 {
        return None;
    }
    let mut min_x = vertices[0];
    let mut max_x = vertices[0];
    let mut min_y = vertices[1];
    let mut max_y = vertices[1];
    for chunk in vertices.chunks(2).skip(1) {
        if chunk.len() < 2 {
            continue;
        }
        min_x = min_x.min(chunk[0]);
        max_x = max_x.max(chunk[0]);
        min_y = min_y.min(chunk[1]);
        max_y = max_y.max(chunk[1]);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

/// 将 WebView 的所有 13 种图元类型转换为浏览器坐标（应用 scale、offset、clip）。
///
/// 返回的 `RenderPrimitives` 中的图元坐标为物理像素，
/// 已经应用了 `scale_factor`、`offset` 和视口裁剪。
pub fn transform_webview_primitives(
    primitives: &RenderPrimitives,
    x_offset: f32,
    y_offset: f32,
    s: f32,
    clip_viewport: Option<ViewportClip>,
) -> RenderPrimitives {
    let mut out = RenderPrimitives::new();

    // 1. 阴影
    for shadow in &primitives.shadows {
        let mut s_clone = shadow.clone();
        s_clone.rect.origin.x = s_clone.rect.origin.x * s + x_offset;
        s_clone.rect.origin.y = s_clone.rect.origin.y * s + y_offset;
        s_clone.rect.size.width *= s;
        s_clone.rect.size.height *= s;
        s_clone.offset_x *= s;
        s_clone.offset_y *= s;
        s_clone.blur_radius *= s;
        s_clone.spread_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut s_clone.rect, clip)
        {
            continue;
        }
        out.shadows.push(s_clone);
    }

    // 2. 填充矩形
    for fill in &primitives.fills {
        let x = fill.rect.origin.x * s + x_offset;
        let y = fill.rect.origin.y * s + y_offset;
        let w = fill.rect.size.width * s;
        let h = fill.rect.size.height * s;
        let Some((x, y, w, h)) = clip_viewport
            .and_then(|clip| clip_axis_aligned_rect(x, y, w, h, clip))
            .or_else(|| {
                if clip_viewport.is_some() {
                    None
                } else {
                    Some((x, y, w, h))
                }
            })
        else {
            continue;
        };
        out.fills.push(FillPrimitive {
            rect: Rect::new(x, y, w, h),
            color: fill.color,
        });
    }

    // 3. 圆角矩形
    for rr in &primitives.rounded_rects {
        let mut r_clone = rr.clone();
        r_clone.rect.origin.x = r_clone.rect.origin.x * s + x_offset;
        r_clone.rect.origin.y = r_clone.rect.origin.y * s + y_offset;
        r_clone.rect.size.width *= s;
        r_clone.rect.size.height *= s;
        r_clone.top_left_radius *= s;
        r_clone.top_right_radius *= s;
        r_clone.bottom_right_radius *= s;
        r_clone.bottom_left_radius *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut r_clone.rect, clip)
        {
            continue;
        }
        clamp_rounded_rect_radii(&mut r_clone);
        out.rounded_rects.push(r_clone);
    }

    // 4. 渐变
    for gradient in &primitives.gradients {
        let mut g_clone = gradient.clone();
        g_clone.rect.origin.x = g_clone.rect.origin.x * s + x_offset;
        g_clone.rect.origin.y = g_clone.rect.origin.y * s + y_offset;
        g_clone.rect.size.width *= s;
        g_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut g_clone.rect, clip)
        {
            continue;
        }
        g_clone.kind = match g_clone.kind {
            GradientKind::Linear { x0, y0, x1, y1 } => GradientKind::Linear {
                x0: x0 * s + x_offset,
                y0: y0 * s + y_offset,
                x1: x1 * s + x_offset,
                y1: y1 * s + y_offset,
            },
            GradientKind::Radial {
                cx,
                cy,
                inner_radius,
                outer_radius,
            } => GradientKind::Radial {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                inner_radius: inner_radius * s,
                outer_radius: outer_radius * s,
            },
            GradientKind::Conic { cx, cy, start_angle } => GradientKind::Conic {
                cx: cx * s + x_offset,
                cy: cy * s + y_offset,
                start_angle,
            },
        };
        out.gradients.push(g_clone);
    }

    // 5. 图片（裁剪须用 `clip` 字段，不可缩小 `rect`，否则会拉伸纹理）
    for image in &primitives.images {
        let x = image.rect.origin.x * s + x_offset;
        let y = image.rect.origin.y * s + y_offset;
        let w = image.rect.size.width * s;
        let h = image.rect.size.height * s;
        let full_rect = Rect::new(x, y, w, h);

        if let Some(clip) = clip_viewport
            && clip.excludes(
                full_rect.origin.x,
                full_rect.origin.y,
                full_rect.size.width,
                full_rect.size.height,
            )
        {
            continue;
        }

        let mut i_clone = image.clone();
        i_clone.rect = full_rect;
        if let Some(clip) = &image.clip {
            i_clone.clip = Some(Rect::new(
                clip.origin.x * s + x_offset,
                clip.origin.y * s + y_offset,
                clip.size.width * s,
                clip.size.height * s,
            ));
        } else {
            i_clone.clip = None;
        }

        if let Some(clip) = clip_viewport {
            let window = Rect::new(clip.left, clip.top, clip.right - clip.left, clip.bottom - clip.top);
            i_clone.clip = match i_clone.clip {
                Some(existing) => existing.intersection(&window),
                None => Some(window),
            };
            if i_clone.clip.is_none() {
                continue;
            }
        }

        out.images.push(i_clone);
    }

    // 6. 线段
    for stroke in &primitives.strokes {
        let mut st = stroke.clone();
        st.x1 = st.x1 * s + x_offset;
        st.y1 = st.y1 * s + y_offset;
        st.x2 = st.x2 * s + x_offset;
        st.y2 = st.y2 * s + y_offset;
        st.width *= s;
        if let Some(clip) = clip_viewport {
            let pad = st.width * 0.5;
            let min_x = st.x1.min(st.x2) - pad;
            let min_y = st.y1.min(st.y2) - pad;
            let max_x = st.x1.max(st.x2) + pad;
            let max_y = st.y1.max(st.y2) + pad;
            if clip.excludes(min_x, min_y, max_x - min_x, max_y - min_y) {
                continue;
            }
        }
        out.strokes.push(st);
    }

    // 7. 路径填充
    for pf in &primitives.path_fills {
        let mut p_clone = pf.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
            && clip.excludes(x, y, w, h)
        {
            continue;
        }
        out.path_fills.push(p_clone);
    }

    // 8. 路径描边
    for ps in &primitives.path_strokes {
        let mut p_clone = ps.clone();
        for i in (0..p_clone.vertices.len()).step_by(2) {
            p_clone.vertices[i] = p_clone.vertices[i] * s + x_offset;
            if i + 1 < p_clone.vertices.len() {
                p_clone.vertices[i + 1] = p_clone.vertices[i + 1] * s + y_offset;
            }
        }
        p_clone.line_width *= s;
        if let Some(clip) = clip_viewport
            && let Some((x, y, w, h)) = path_vertices_bbox(&p_clone.vertices)
        {
            let pad = p_clone.line_width * 0.5;
            if clip.excludes(x - pad, y - pad, w + pad * 2.0, h + pad * 2.0) {
                continue;
            }
        }
        out.path_strokes.push(p_clone);
    }

    // 9. 文字
    for glyph in &primitives.glyphs {
        let x = glyph.x * s + x_offset;
        let y = glyph.y * s + y_offset;
        let font_size = glyph.font_size * s;
        if let Some(clip) = clip_viewport {
            let top = y - font_size;
            let bottom = y + font_size * 0.25;
            let width = font_size * 0.6;
            if clip.excludes(x, top, width, bottom - top) {
                continue;
            }
        }
        out.glyphs.push(GlyphPrimitive {
            x,
            y,
            font_size,
            color: glyph.color,
            glyph_id: glyph.glyph_id,
            font_id: glyph.font_id,
            bitmap_width: glyph.bitmap_width,
            bitmap_height: glyph.bitmap_height,
            rotation: glyph.rotation,
        });
    }

    // 10. 裁剪
    for clip in &primitives.clips {
        let mut c_clone = clip.clone();
        c_clone.rect.origin.x = c_clone.rect.origin.x * s + x_offset;
        c_clone.rect.origin.y = c_clone.rect.origin.y * s + y_offset;
        c_clone.rect.size.width *= s;
        c_clone.rect.size.height *= s;
        if let Some(viewport) = clip_viewport
            && !clip_rect_field(&mut c_clone.rect, viewport)
        {
            continue;
        }
        out.clips.push(c_clone);
    }

    // 11. 变换
    for transform in &primitives.transforms {
        let mut t_clone = transform.clone();
        t_clone.rect.origin.x = t_clone.rect.origin.x * s + x_offset;
        t_clone.rect.origin.y = t_clone.rect.origin.y * s + y_offset;
        t_clone.rect.size.width *= s;
        t_clone.rect.size.height *= s;
        t_clone.origin_x = t_clone.origin_x * s + x_offset;
        t_clone.origin_y = t_clone.origin_y * s + y_offset;
        t_clone.tx *= s;
        t_clone.ty *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut t_clone.rect, clip)
        {
            continue;
        }
        out.transforms.push(t_clone);
    }

    // 12. 滤镜
    for filter in &primitives.filters {
        let mut f_clone = filter.clone();
        f_clone.rect.origin.x = f_clone.rect.origin.x * s + x_offset;
        f_clone.rect.origin.y = f_clone.rect.origin.y * s + y_offset;
        f_clone.rect.size.width *= s;
        f_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut f_clone.rect, clip)
        {
            continue;
        }
        out.filters.push(f_clone);
    }

    // 13. 混合模式
    for blend in &primitives.blend_modes {
        let mut b_clone = blend.clone();
        b_clone.rect.origin.x = b_clone.rect.origin.x * s + x_offset;
        b_clone.rect.origin.y = b_clone.rect.origin.y * s + y_offset;
        b_clone.rect.size.width *= s;
        b_clone.rect.size.height *= s;
        if let Some(clip) = clip_viewport
            && !clip_rect_field(&mut b_clone.rect, clip)
        {
            continue;
        }
        out.blend_modes.push(b_clone);
    }

    out
}
