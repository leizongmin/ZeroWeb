//! 文本和列表绘制 — 文本内容、列表标记、列分隔线。
//!
//! 包含 paint_text、paint_list_marker、compute_list_item_index、paint_column_rules。

use std::collections::HashMap;

use zero_css_parser::values::{ColorValue, FloatValue, LengthValue, ListStyleTypeValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::{
    FloatExclusion, InlineFormattingContext, LayoutBox, TextAlign, WordBreakMode, estimate_char_width,
};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, LineCap, StrokePrimitive};
use zero_style_system::{
    ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue, ColumnWidthComputedValue,
    ComputedStyle, ContentComputedValue, ObjectFitComputedValue, TabSizeValue, TextAlignLastValue, TextAlignValue,
    TextOverflowValue, WhiteSpaceValue,
};

use super::super::color::color_value_to_render;
use super::super::helpers::apply_text_transform;

impl super::Painter {
    /// 绘制多列布局的 column-rule（列之间的分隔线）。
    pub(super) fn paint_column_rules(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        use zero_render_foundation::geometry::Rect;
        use zero_render_foundation::primitive::LineCap;

        // 计算 column-count
        let count = match &style.column_count {
            ColumnCountComputedValue::Auto => match &style.column_width {
                ColumnWidthComputedValue::Auto => return,
                ColumnWidthComputedValue::Length(LengthValue::Px(w)) => {
                    let content_w = box_node.content_width;
                    if content_w <= 0.0 || *w <= 0.0 {
                        return;
                    }
                    let gap: f32 = match style.column_gap {
                        LengthValue::Px(g) => g as f32,
                        _ => 0.0,
                    };
                    ((content_w + gap) / (*w as f32 + gap)).max(1.0).floor() as u32
                }
                _ => return,
            },
            ColumnCountComputedValue::Number(n) => *n,
        };

        if count < 2 {
            return;
        }

        if matches!(
            style.column_rule_style,
            ColumnRuleStyleComputedValue::None | ColumnRuleStyleComputedValue::Hidden
        ) {
            return;
        }

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        let content_w = box_node.content_width;
        let content_h = box_node.content_height;

        if content_w <= 0.0 || content_h <= 0.0 {
            return;
        }

        let gap: f32 = match style.column_gap {
            LengthValue::Px(g) => g as f32,
            _ => 0.0,
        };

        let rule_w: f32 = match &style.column_rule_width {
            ColumnRuleWidthComputedValue::Medium => 2.0,
            ColumnRuleWidthComputedValue::Thin => 1.0,
            ColumnRuleWidthComputedValue::Thick => 3.0,
            ColumnRuleWidthComputedValue::Length(LengthValue::Px(w)) => *w as f32,
            _ => 1.0,
        };

        let rule_color = color_value_to_render(&style.column_rule_color);
        let col_w = (content_w - (count as f32 - 1.0) * gap) / count as f32;
        if col_w <= 0.0 {
            return;
        }

        for i in 1..count {
            let rule_x = content_x + i as f32 * col_w + (i as f32 - 0.5) * gap - rule_w / 2.0;
            let rule_x = rule_x.max(content_x);
            match style.column_rule_style {
                ColumnRuleStyleComputedValue::Solid => {
                    self.primitives
                        .add_fill(Rect::new(rule_x, content_y, rule_w, content_h), rule_color);
                }
                ColumnRuleStyleComputedValue::Dotted => {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: rule_x + rule_w / 2.0,
                        y1: content_y,
                        x2: rule_x + rule_w / 2.0,
                        y2: content_y + content_h,
                        width: rule_w,
                        color: rule_color,
                        style: zero_render_foundation::primitive::LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
                ColumnRuleStyleComputedValue::Dashed => {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: rule_x + rule_w / 2.0,
                        y1: content_y,
                        x2: rule_x + rule_w / 2.0,
                        y2: content_y + content_h,
                        width: rule_w,
                        color: rule_color,
                        style: zero_render_foundation::primitive::LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
                _ => {
                    self.primitives
                        .add_fill(Rect::new(rule_x, content_y, rule_w, content_h), rule_color);
                }
            }
        }
    }

    /// 收集浮动子元素的排除区域（带样式映射版本）。
    ///
    /// 遍历 `box_node` 的直接子元素，找出带有 `float: left/right` 样式的子元素，
    /// 计算它们相对于容器内容区域的位置和尺寸。
    pub(super) fn collect_float_exclusions_with_styles(
        &self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<FloatExclusion> {
        let mut exclusions = Vec::new();

        // 容器内容区域的原点 y 偏移（浮动排除区域相对于内容区域顶部计算）
        let content_offset_y = box_node.border_top + box_node.padding_top;

        for child in &box_node.children {
            // 跳过绝对定位子元素（不参与浮动流）
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if let Some(node_id) = child.node_id
                && let Some(child_style) = styles.get(&node_id)
            {
                let is_left = matches!(child_style.float, FloatValue::Left | FloatValue::InlineStart);
                let is_right = matches!(child_style.float, FloatValue::Right | FloatValue::InlineEnd);

                if is_left || is_right {
                    // 浮动子元素相对于容器内容区域的位置
                    let rel_y = child.y - content_offset_y;
                    exclusions.push(FloatExclusion {
                        y: rel_y,
                        height: child.height,
                        width: child.width,
                        is_left,
                    });
                }
            }
        }

        exclusions
    }

    /// 绘制列表标记（disc/circle/square/decimal 等）。
    pub(super) fn paint_list_marker(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        use zero_render_foundation::geometry::Rect;
        use zero_render_foundation::image_cache::ImageKey;
        use zero_render_foundation::primitive::ImagePrimitive;

        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        match &node.kind {
            NodeKind::Element(elem) if elem.local_name() == "li" => {}
            _ => return,
        }

        // list-style-image 优先
        match &style.list_style_image {
            zero_style_system::ListStyleImageComputedValue::Url(url) => {
                let font_size: f32 = match style.font_size {
                    LengthValue::Px(s) => s as f32,
                    _ => 16.0,
                };
                let img_size = font_size;
                let marker_x = abs_x + box_node.border_left - img_size * 1.5;
                let marker_y = abs_y + box_node.border_top + box_node.padding_top;
                self.primitives.add_image(ImagePrimitive {
                    rect: Rect::new(marker_x, marker_y, img_size, img_size),
                    image_key: ImageKey::new(super::super::helpers::simple_hash(url)),
                });
                return;
            }
            zero_style_system::ListStyleImageComputedValue::None => {}
        }

        if style.list_style_type == ListStyleTypeValue::None {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => 16.0,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = color_value_to_render(&style.color);
        let default_font_id = self.resolve_font_id(&style.font_family);
        let marker_size = font_size * 0.4;
        let marker_x = abs_x + box_node.border_left;
        let marker_y = abs_y + box_node.border_top + box_node.padding_top;

        let actual_marker_x = match style.list_style_position {
            zero_css_parser::values::ListStylePositionValue::Outside => marker_x - marker_size * 2.5,
            zero_css_parser::values::ListStylePositionValue::Inside => marker_x + marker_size * 0.5,
        };

        match style.list_style_type {
            ListStyleTypeValue::Disc => {
                self.primitives.add_fill(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                );
            }
            ListStyleTypeValue::Circle => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: actual_marker_x,
                    y1: marker_y + font_size * 0.3 - marker_size / 2.0 + marker_size / 2.0,
                    x2: actual_marker_x + marker_size,
                    y2: marker_y + font_size * 0.3 - marker_size / 2.0 + marker_size / 2.0,
                    width: marker_size,
                    color,
                    style: zero_render_foundation::primitive::LineStyle::Solid,
                    cap: LineCap::Round,
                });
            }
            ListStyleTypeValue::Square => {
                self.primitives.add_fill(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                );
            }
            ListStyleTypeValue::Decimal | ListStyleTypeValue::DecimalLeadingZero => {
                // 优先使用 CSS counter "list-item"，回退到兄弟索引
                let index = self
                    .get_counter("list-item")
                    .map(|v| v as usize)
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let text = if matches!(style.list_style_type, ListStyleTypeValue::DecimalLeadingZero) && index < 10 {
                    format!("0{index}.")
                } else {
                    format!("{index}.")
                };
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85, false);
                }
            }
            ListStyleTypeValue::LowerAlpha | ListStyleTypeValue::UpperAlpha => {
                let index = self
                    .get_counter("list-item")
                    .map(|v| v as usize)
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let ch = if index > 0 && index <= 26 {
                    let base = if matches!(style.list_style_type, ListStyleTypeValue::LowerAlpha) {
                        b'a'
                    } else {
                        b'A'
                    };
                    (base + (index - 1) as u8) as char
                } else {
                    '?'
                };
                let text = format!("{ch}.");
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85, false);
                }
            }
            ListStyleTypeValue::LowerRoman | ListStyleTypeValue::UpperRoman => {
                let index = self
                    .get_counter("list-item")
                    .map(|v| v as usize)
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let roman = to_roman(index);
                let text = if matches!(style.list_style_type, ListStyleTypeValue::LowerRoman) {
                    format!("{}.", roman.to_lowercase())
                } else {
                    format!("{roman}.")
                };
                let mut char_x = actual_marker_x;
                let char_y = marker_y + font_size;
                for ch in text.chars() {
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: char_y,
                        font_size: font_size * 0.85,
                        color,
                        glyph_id: ch as u32,
                        font_id: default_font_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                    });
                    char_x += estimate_char_width(ch, font_size * 0.85, false);
                }
            }
            ListStyleTypeValue::None => {}
        }
    }

    /// 计算当前列表项在其兄弟中的 1-based 索引。
    fn compute_list_item_index(&self, doc: &Document, node_id: NodeId) -> usize {
        let parent_id = match doc.parent_node(node_id) {
            Some(id) => id,
            None => return 1,
        };

        let mut index = 0;
        let mut found = false;
        for child_id in doc.child_nodes(parent_id) {
            if child_id == node_id {
                found = true;
                break;
            }
            if let Some(child) = doc.get(child_id)
                && let NodeKind::Element(elem) = &child.kind
                && elem.local_name() == "li"
            {
                index += 1;
            }
        }

        if found { index + 1 } else { 1 }
    }

    /// 绘制 CSS `content` 属性生成的文本内容。
    ///
    /// 当元素的 `content` 属性为 `String` 或 `Counter` 时，
    /// 在元素的内容区域起始位置绘制对应的文本。
    /// 支持计数器值的十进制、小写字母、大写字母、小写罗马、大写罗马格式化。
    pub(crate) fn paint_content(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let text = match &style.content {
            ContentComputedValue::Normal | ContentComputedValue::None | ContentComputedValue::Attr(_) => return,
            ContentComputedValue::String(s) => s.clone(),
            ContentComputedValue::Counter {
                name,
                style: counter_style,
            } => {
                let value = self.get_counter(name).unwrap_or(0);
                match counter_style.as_deref() {
                    Some("lower-alpha") | Some("lower-latin") => format_counter_alpha(value, false),
                    Some("upper-alpha") | Some("upper-latin") => format_counter_alpha(value, true),
                    Some("lower-roman") => format_counter_roman(value, false),
                    Some("upper-roman") => format_counter_roman(value, true),
                    _ => value.to_string(), // decimal (default)
                }
            }
        };

        if text.is_empty() {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = super::super::color::color_value_to_render(&style.color);
        let default_font_id = self.resolve_font_id(&style.font_family);
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let mut char_x = content_x;
        let char_y = content_y + font_size;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: char_y,
                font_size,
                color,
                glyph_id: ch as u32,
                font_id: default_font_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
            });
            char_x += estimate_char_width(ch, font_size, false);
        }
    }

    /// 绘制 `<img>` 元素，根据 `object-fit` 属性决定图片如何适配容器。
    ///
    /// - `fill`：拉伸图片填满容器（默认）
    /// - `contain`：等比缩放，完整显示图片
    /// - `cover`：等比缩放，完全覆盖容器
    /// - `none`：原始尺寸
    /// - `scale-down`：取 none 和 contain 中较小的结果
    pub(crate) fn paint_img_element(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // 仅处理 <img> 元素
        let elem = match &node.kind {
            NodeKind::Element(elem) if elem.local_name() == "img" => elem,
            _ => return,
        };

        // 获取 src URL 作为图片键
        let src = elem.get_attribute("src").unwrap_or_default();
        if src.is_empty() {
            return;
        }

        let container_w = box_node.content_width;
        let container_h = box_node.content_height;
        if container_w <= 0.0 || container_h <= 0.0 {
            return;
        }

        // 尝试获取图片的固有尺寸（从 width/height 属性或回退到容器尺寸）
        let (intrinsic_w, intrinsic_h) = get_img_intrinsic_size(node, container_w, container_h);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let image_key = ImageKey::new(super::super::helpers::simple_hash(&src));

        let (img_x, img_y, img_w, img_h) = compute_object_fit_rect(
            &style.object_fit,
            container_w,
            container_h,
            intrinsic_w,
            intrinsic_h,
            content_x,
            content_y,
        );

        self.primitives.add_image(ImagePrimitive {
            rect: Rect::new(img_x, img_y, img_w, img_h),
            image_key,
        });
    }

    /// 绘制文本内容（生成多字符 GlyphPrimitive）。
    pub fn paint_text(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
        styles: Option<&HashMap<NodeId, ComputedStyle>>,
    ) {
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };

        if font_size <= 0.0 {
            return;
        }

        if style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);

        let letter_spacing: f32 = match style.letter_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };
        let word_spacing: f32 = match style.word_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };

        let text_shadow = &style.text_shadow;
        let has_text_shadow =
            text_shadow.offset_x != 0.0 || text_shadow.offset_y != 0.0 || text_shadow.blur_radius != 0.0;
        let shadow_ox = text_shadow.offset_x;
        let shadow_oy = text_shadow.offset_y;
        let shadow_color = color_value_to_render(&text_shadow.color);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let (tx, ty) = super::super::helpers::apply_transform_offset(style, abs_x, abs_y);

        let default_font_id = self.resolve_font_id(&style.font_family);

        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            if self.painted_inline_nodes.contains(&node_id) || !has_direct_paintable_text(doc, node_id) {
                return;
            }

            let container_width = box_node.content_width;
            let break_word = matches!(
                style.overflow_wrap,
                zero_style_system::OverflowWrapValue::BreakWord | zero_style_system::OverflowWrapValue::Anywhere
            );

            // 根据 white-space 属性设置换行和空白保留行为
            let (mut no_wrap, preserve_whitespace) = match style.white_space {
                WhiteSpaceValue::Normal => (false, false),
                WhiteSpaceValue::Nowrap => (true, false),
                WhiteSpaceValue::Pre => (true, true),
                WhiteSpaceValue::PreWrap => (false, true),
                WhiteSpaceValue::PreLine => (false, false),
                WhiteSpaceValue::BreakSpaces => (false, true),
            };

            // CSS text-wrap: nowrap 覆盖换行行为
            if let Some(wrap_override) = super::Painter::resolve_text_wrap(style) {
                no_wrap = wrap_override;
            }

            // CSS line-clamp: 限制最大行数
            let max_lines = super::Painter::resolve_line_clamp(style);

            // 将 CSS word-break 映射到布局引擎的 WordBreakMode
            let word_break_mode = match style.word_break {
                zero_style_system::WordBreakValue::BreakAll => WordBreakMode::BreakAll,
                zero_style_system::WordBreakValue::KeepAll => WordBreakMode::KeepAll,
                _ => WordBreakMode::Normal,
            };

            // 将 CSS text-align 映射到布局引擎的 TextAlign
            let text_align = match style.text_align {
                TextAlignValue::Left | TextAlignValue::Start => TextAlign::Left,
                TextAlignValue::Right | TextAlignValue::End => TextAlign::Right,
                TextAlignValue::Center => TextAlign::Center,
                TextAlignValue::Justify => TextAlign::Justify,
            };

            // 将 CSS text-align-last 映射到布局引擎（Auto = 跟随 text-align）
            let text_align_last = match &style.text_align_last {
                TextAlignLastValue::Auto => None,
                TextAlignLastValue::Left | TextAlignLastValue::Start => Some(TextAlign::Left),
                TextAlignLastValue::Right | TextAlignLastValue::End => Some(TextAlign::Right),
                TextAlignLastValue::Center => Some(TextAlign::Center),
                TextAlignLastValue::Justify => Some(TextAlign::Justify),
            };

            // text-indent 首行缩进（px）
            let text_indent_px: f32 = match style.text_indent {
                LengthValue::Px(v) => v as f32,
                LengthValue::Em(v) => v as f32 * font_size,
                _ => 0.0,
            };

            // CSS tab-size — 制表符展开宽度
            // Number(n) 表示 n 个空格宽度，Length 表示具体像素值
            let tab_size_px: f32 = match &style.tab_size {
                TabSizeValue::Number(n) => {
                    // 空格宽度约 font_size * 0.25，乘以空格数
                    *n as f32 * font_size * 0.25
                }
                TabSizeValue::Length(LengthValue::Px(v)) => *v as f32,
                TabSizeValue::Length(LengthValue::Em(v)) => *v as f32 * font_size,
                _ => font_size * 0.25 * 8.0, // 默认 8 个空格宽度
            };

            // 收集浮动子元素的排除区域
            let float_exclusions = styles
                .map(|s| self.collect_float_exclusions_with_styles(box_node, s))
                .unwrap_or_default();

            let mut inline_ctx = InlineFormattingContext::new(container_width)
                .with_text_align(text_align)
                .with_text_align_last(text_align_last)
                .with_break_word(break_word)
                .with_no_wrap(no_wrap)
                .with_preserve_whitespace(preserve_whitespace)
                .with_word_break(word_break_mode)
                .with_text_indent(text_indent_px)
                .with_float_exclusions(float_exclusions)
                .with_tab_size(tab_size_px);
            inline_ctx.layout(doc, node_id, &HashMap::new());

            let fragments = inline_ctx.all_fragments();

            let needs_ellipsis = matches!(style.text_overflow, TextOverflowValue::Ellipsis)
                && !matches!(style.overflow_x, zero_css_parser::values::OverflowValue::Visible);

            if !fragments.is_empty() {
                let glyphs_before_fragments = self.primitives.glyphs.len();

                // writing-mode: vertical-rl/vertical-lr 时字符旋转 90°
                let is_vertical = matches!(
                    style.writing_mode,
                    zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
                );
                let rotation = if is_vertical { std::f32::consts::FRAC_PI_2 } else { 0.0 };

                for fragment in fragments.iter() {
                    self.painted_inline_nodes.insert(fragment.node_id);

                    // text-indent 已在 InlineFormattingContext 中处理，fragment.x 包含缩进
                    let frag_base_x = content_x + fragment.x + tx;
                    let frag_base_y = content_y + fragment.y + fragment.font_size + ty;
                    let mut char_x = frag_base_x;

                    let transformed = apply_text_transform(&fragment.text, &style.text_transform);

                    for ch in transformed.chars() {
                        if has_text_shadow {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: char_x + shadow_ox,
                                y: frag_base_y + shadow_oy,
                                font_size: fragment.font_size,
                                color: shadow_color,
                                glyph_id: ch as u32,
                                font_id: default_font_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation,
                            });
                        }

                        self.primitives.add_glyph(GlyphPrimitive {
                            x: char_x,
                            y: frag_base_y,
                            font_size: fragment.font_size,
                            color,
                            glyph_id: ch as u32,
                            font_id: default_font_id,
                            bitmap_width: None,
                            bitmap_height: None,
                            rotation,
                        });
                        char_x += estimate_char_width(ch, fragment.font_size, false);
                        char_x += letter_spacing;
                        if ch == ' ' {
                            char_x += word_spacing;
                        }
                    }

                    let text_width: f32 = transformed
                        .chars()
                        .map(|ch| {
                            let w = estimate_char_width(ch, fragment.font_size, false) + letter_spacing;
                            if ch == ' ' { w + word_spacing } else { w }
                        })
                        .sum();
                    self.paint_text_decoration_from_style(
                        frag_base_x,
                        frag_base_y,
                        fragment.font_size,
                        text_width,
                        color,
                        style,
                    );
                }

                // text-overflow: ellipsis 后处理
                if needs_ellipsis && container_width > 0.0 {
                    let content_right = content_x + container_width + tx;

                    let glyphs = &mut self.primitives.glyphs;
                    let fragment_glyphs = &mut glyphs[glyphs_before_fragments..];

                    let mut last_visible_idx: Option<usize> = None;
                    let mut has_overflow = false;

                    for (i, g) in fragment_glyphs.iter().enumerate() {
                        if g.font_size == 0.0 {
                            continue;
                        }
                        if g.x >= content_right {
                            has_overflow = true;
                            last_visible_idx = if i > 0 { Some(i - 1) } else { None };
                            break;
                        }
                        last_visible_idx = Some(i);
                    }

                    if has_overflow {
                        let ellipsis_char_width = estimate_char_width('.', font_size, false);
                        let total_ellipsis_width = ellipsis_char_width * 3.0 + letter_spacing * 2.0;
                        let ellipsis_end_x = content_right;
                        let ellipsis_start_x = ellipsis_end_x - total_ellipsis_width;

                        let cutoff_start = if let Some(idx) = last_visible_idx {
                            let mut cut = idx + 1;
                            for j in (0..=idx).rev() {
                                if fragment_glyphs[j].x < ellipsis_start_x && fragment_glyphs[j].font_size > 0.0 {
                                    cut = j + 1;
                                    break;
                                }
                                cut = j;
                            }
                            cut
                        } else {
                            0
                        };

                        for g in fragment_glyphs.iter_mut().skip(cutoff_start) {
                            g.glyph_id = 0;
                            g.font_size = 0.0;
                        }

                        let first_glyph = fragment_glyphs.iter().find(|g| g.font_size > 0.0);
                        let base_y = first_glyph.map(|g| g.y).unwrap_or(content_y + font_size + ty);

                        for (i, ch) in ['.', '.', '.'].iter().enumerate() {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: ellipsis_start_x + ellipsis_char_width * i as f32 + letter_spacing * i as f32,
                                y: base_y,
                                font_size,
                                color,
                                glyph_id: *ch as u32,
                                font_id: default_font_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                            });
                        }
                    }
                }

                // CSS line-clamp 后处理：限制可见行数并在截断处添加省略号
                if let Some(max) = max_lines {
                    let glyphs = &self.primitives.glyphs;
                    let fragment_glyphs = &glyphs[glyphs_before_fragments..];

                    // 收集唯一的行 Y 坐标（用于计算总行数）
                    let mut line_ys: Vec<f32> = fragment_glyphs
                        .iter()
                        .filter(|g| g.font_size > 0.0)
                        .map(|g| g.y)
                        .collect();
                    line_ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    line_ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);

                    if line_ys.len() > max as usize {
                        // 需要截断：找到第 max+1 行的 Y 坐标
                        let cutoff_y = line_ys[max as usize];

                        // 移除截断行及之后的所有 glyph
                        let glyphs = &mut self.primitives.glyphs;
                        for g in glyphs[glyphs_before_fragments..].iter_mut() {
                            if g.y >= cutoff_y - 0.5 {
                                g.font_size = 0.0;
                                g.glyph_id = 0;
                            }
                        }

                        // 在最后一行末尾添加省略号
                        let last_line_y = line_ys[max as usize - 1];
                        let last_glyph_x = glyphs[glyphs_before_fragments..]
                            .iter()
                            .filter(|g| g.font_size > 0.0 && (g.y - last_line_y).abs() < 0.5)
                            .map(|g| g.x)
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            .unwrap_or(content_x + tx);

                        let ellipsis_width = estimate_char_width('.', font_size, false);
                        let default_font_id = self.resolve_font_id(&style.font_family);
                        for i in 0..3 {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: last_glyph_x + ellipsis_width * (i as f32 + 1.0),
                                y: last_line_y,
                                font_size,
                                color,
                                glyph_id: '.' as u32,
                                font_id: default_font_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                            });
                        }
                    }
                }

                return;
            }
        }

        // 退化为单个占位 glyph
        let glyph_x = content_x + tx;
        let glyph_y = content_y + ty;

        if has_text_shadow {
            self.primitives.add_glyph(GlyphPrimitive {
                x: glyph_x + shadow_ox,
                y: glyph_y + font_size + shadow_oy,
                font_size,
                color: shadow_color,
                glyph_id: 0,
                font_id: default_font_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
            });
        }

        self.primitives.add_glyph(GlyphPrimitive {
            x: glyph_x,
            y: glyph_y + font_size,
            font_size,
            color,
            glyph_id: 0,
            font_id: default_font_id,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
        });

        self.paint_text_decoration_from_style(
            glyph_x,
            glyph_y + font_size,
            font_size,
            estimate_char_width('A', font_size, false),
            color,
            style,
        );
    }
}

/// 将数字转换为罗马数字字符串（1-based）。
fn to_roman(mut num: usize) -> String {
    if num == 0 {
        return "0".to_string();
    }
    let pairs = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for (value, symbol) in &pairs {
        while num >= *value {
            result.push_str(symbol);
            num -= value;
        }
    }
    result
}

fn has_direct_paintable_text(doc: &Document, node_id: NodeId) -> bool {
    doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    })
}

/// 将计数器值格式化为字母序列（a/b/.../z/aa/ab/...）。
fn format_counter_alpha(value: i32, upper: bool) -> String {
    if value <= 0 {
        return value.to_string();
    }
    let mut v = value as u32;
    let mut result = String::new();
    while v > 0 {
        v -= 1;
        let ch = (b'a' + (v % 26) as u8) as char;
        result.push(ch);
        v /= 26;
    }
    let s: String = result.chars().rev().collect();
    if upper { s.to_uppercase() } else { s }
}

/// 将计数器值格式化为罗马数字。
fn format_counter_roman(value: i32, upper: bool) -> String {
    let s = to_roman(value.max(0) as usize);
    if upper { s } else { s.to_lowercase() }
}

/// 获取 `<img>` 元素的固有尺寸（从 width/height 属性）。
fn get_img_intrinsic_size(node: &zero_dom::NodeData, fallback_w: f32, fallback_h: f32) -> (f32, f32) {
    let elem = match &node.kind {
        NodeKind::Element(e) => e,
        _ => return (fallback_w, fallback_h),
    };
    let w = elem
        .get_attribute("width")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(fallback_w);
    let h = elem
        .get_attribute("height")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(fallback_h);
    (w.max(1.0), h.max(1.0))
}

/// 根据 `object-fit` 计算图片在容器内的绘制矩形。
pub(super) fn compute_object_fit_rect(
    fit: &ObjectFitComputedValue,
    container_w: f32,
    container_h: f32,
    intrinsic_w: f32,
    intrinsic_h: f32,
    content_x: f32,
    content_y: f32,
) -> (f32, f32, f32, f32) {
    match fit {
        ObjectFitComputedValue::Fill => {
            // 拉伸填满容器
            (content_x, content_y, container_w, container_h)
        }
        ObjectFitComputedValue::Contain => {
            // 等比缩放，完整显示
            let scale = (container_w / intrinsic_w).min(container_h / intrinsic_h);
            let w = intrinsic_w * scale;
            let h = intrinsic_h * scale;
            let x = content_x + (container_w - w) / 2.0;
            let y = content_y + (container_h - h) / 2.0;
            (x, y, w, h)
        }
        ObjectFitComputedValue::Cover => {
            // 等比缩放，完全覆盖
            let scale = (container_w / intrinsic_w).max(container_h / intrinsic_h);
            let w = intrinsic_w * scale;
            let h = intrinsic_h * scale;
            let x = content_x + (container_w - w) / 2.0;
            let y = content_y + (container_h - h) / 2.0;
            (x, y, w, h)
        }
        ObjectFitComputedValue::None => {
            // 原始尺寸，居中
            let x = content_x + (container_w - intrinsic_w) / 2.0;
            let y = content_y + (container_h - intrinsic_h) / 2.0;
            (x, y, intrinsic_w, intrinsic_h)
        }
        ObjectFitComputedValue::ScaleDown => {
            // 取 none 和 contain 中较小的结果
            let none_w = intrinsic_w;
            let contain_scale = (container_w / intrinsic_w).min(container_h / intrinsic_h);
            let contain_w = intrinsic_w * contain_scale;
            if none_w <= contain_w {
                // none 更小，使用原始尺寸居中
                let x = content_x + (container_w - intrinsic_w) / 2.0;
                let y = content_y + (container_h - intrinsic_h) / 2.0;
                (x, y, intrinsic_w, intrinsic_h)
            } else {
                // contain 更小
                let w = contain_w;
                let h = intrinsic_h * contain_scale;
                let x = content_x + (container_w - w) / 2.0;
                let y = content_y + (container_h - h) / 2.0;
                (x, y, w, h)
            }
        }
    }
}
