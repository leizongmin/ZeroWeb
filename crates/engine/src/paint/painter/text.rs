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
use super::super::helpers::PrimitiveCounts;
use super::super::helpers::apply_text_transform;

/// 多列布局的列信息（用于 inline 内容的列分布）。
struct MulticolInfo {
    /// 列数
    col_count: usize,
    /// 单列宽度
    col_width: f32,
    /// 列间距
    gap: f32,
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
/// `font_size_px` 用于将 em/rem 单位的 gap 和 column-width 转换为像素。
fn compute_multicol_info_for_paint(
    style: &ComputedStyle,
    container_width: f32,
    font_size_px: f32,
) -> Option<MulticolInfo> {
    let gap: f32 = match &style.column_gap {
        LengthValue::Px(g) => *g as f32,
        LengthValue::Em(e) => *e as f32 * font_size_px,
        LengthValue::Rem(r) => *r as f32 * 16.0_f32, // rem 基于 root font-size
        LengthValue::Percentage(_) => 0.0,           // 百分比 gap 需要容器宽度上下文，暂不支持
        _ => 0.0,
    };

    let col_count_from_count = match &style.column_count {
        ColumnCountComputedValue::Auto => None,
        ColumnCountComputedValue::Number(n) => Some(*n as usize),
    };

    let col_width_hint = match &style.column_width {
        ColumnWidthComputedValue::Auto => None,
        ColumnWidthComputedValue::Length(l) => match l {
            LengthValue::Px(v) => Some(*v as f32),
            LengthValue::Em(e) => Some(*e as f32 * font_size_px),
            LengthValue::Rem(r) => Some(*r as f32 * 16.0_f32),
            _ => None,
        },
    };

    match (col_count_from_count, col_width_hint) {
        (None, None) => None,
        (Some(n), None) => {
            if n == 0 {
                return None;
            }
            let col_width = if container_width > 0.0 {
                (container_width - (n - 1) as f32 * gap) / n as f32
            } else {
                0.0
            };
            Some(MulticolInfo {
                col_count: n,
                col_width: col_width.max(0.0),
                gap,
            })
        }
        (None, Some(min_w)) => {
            if container_width <= 0.0 || min_w <= 0.0 {
                return None;
            }
            let count = ((container_width + gap) / (min_w + gap)).floor() as usize;
            let count = count.max(1);
            let col_width = (container_width - (count - 1) as f32 * gap) / count as f32;
            Some(MulticolInfo {
                col_count: count,
                col_width: col_width.max(0.0),
                gap,
            })
        }
        (Some(_n), Some(min_w)) => {
            // 两者都有值：使用 CSS §3.4 伪算法
            // 取 min(count_from_count, count_from_width)
            let count_from_width = if container_width > 0.0 && min_w > 0.0 {
                ((container_width + gap) / (min_w + gap)).floor() as usize
            } else {
                return None;
            };
            let count = (_n).min(count_from_width).max(1);
            let col_width = (container_width - (count - 1) as f32 * gap) / count as f32;
            Some(MulticolInfo {
                col_count: count,
                col_width: col_width.max(0.0),
                gap,
            })
        }
    }
}

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
            // CSS Multi-column §5.2：列分隔线仅在两列都有内容时绘制。
            // 如果容器有子元素，检查第 i 列和第 i+1 列是否有内容；
            // 如果容器没有子元素（单元测试场景），默认绘制所有分隔线。
            if !box_node.children.is_empty() {
                let col_left_start = (i - 1) as f32 * (col_w + gap);
                let has_left_content = box_node.children.iter().any(|c| {
                    !c.is_absolute && !c.is_fixed && c.x >= col_left_start - 0.5 && c.x < col_left_start + col_w + 0.5
                });
                let col_right_start = i as f32 * (col_w + gap);
                let has_right_content = box_node.children.iter().any(|c| {
                    !c.is_absolute && !c.is_fixed && c.x >= col_right_start - 0.5 && c.x < col_right_start + col_w + 0.5
                });
                if !has_left_content || !has_right_content {
                    continue; // 跳过空列的分隔线
                }
            }

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
            // R109 §9.2.1.1：被 in-flow block 子元素拆分的 inline 父盒自身不渲染文本——
            // 其直接文本已由匿名块片段子盒（带 fragment_node_ids）渲染。避免与片段重叠。
            if box_node.is_r109_split && box_node.fragment_node_ids.is_none() {
                return;
            }
            if !has_direct_paintable_text(doc, node_id) {
                return;
            }
            // R109：匿名块片段跳过 painted_inline_nodes 去重——多个片段共享 inline 的
            // node_id，首个片段渲染后会标记该 id，须放行后续片段各自渲染其片段文本。
            if box_node.fragment_node_ids.is_none() && self.painted_inline_nodes.contains(&node_id) {
                return;
            }

            let container_width = box_node.content_width;

            // 检测是否为多列容器（无块级子元素但有 inline 内容）
            // 如果是，使用列宽创建 IFC，并在渲染时将行分配到各列。
            // 条件：
            // 1. 无 inflow 子元素（纯 inline 内容）
            // 2. column-fill: balance（默认值，非 auto 顺序填充）
            // 注意：对于纯 inline 内容，有明确高度时 balance 模式仍需分配到各列。
            // column-fill: auto 的 inline 内容由 layout 层处理（有 height 限制时），
            // 此处仅处理 balance 模式（无论有无 height）。
            let has_in_flow_children = box_node
                .children
                .iter()
                .any(|c| !c.is_absolute && !c.is_fixed && c.is_block_level);
            let is_balance_mode = !matches!(style.column_fill, zero_style_system::ColumnFillComputedValue::Auto);
            // 仅对 height:auto 的纯行内 multicol 容器做列分配。明确高度的 balance 容器
            // （常见于嵌套 multicol / column-breaking 测试）涉及 column breaking，
            // 当前简单均衡分配会回归这类用例，回退到单块渲染。
            let height_auto = matches!(style.height, LengthValue::Auto);
            let multicol_info = if !has_in_flow_children && is_balance_mode && height_auto {
                compute_multicol_info_for_paint(style, container_width, font_size)
            } else {
                None
            };

            // 多列容器使用列宽创建 IFC
            let ifc_width = if let Some(ref mc) = multicol_info {
                mc.col_width
            } else {
                container_width
            };

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

            let is_vertical = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let is_vertical_rtl = matches!(style.writing_mode, zero_style_system::WritingModeValue::VerticalRl);

            // 尝试使用布局引擎存储的行内布局结果，避免重新运行 IFC。
            // 条件：(1) 非多列模式 (2) 有存储结果 (3) 容器宽度匹配
            // 宽度验证确保 table/multicol 后处理改变宽度时回退到 paint IFC。
            let width_matches = (box_node.inline_layout_width - ifc_width).abs() < 1.0;
            let use_stored = multicol_info.is_none() && box_node.inline_layout.is_some() && width_matches;

            // 从存储结果创建的扁平化片段列表（用于非多列渲染路径）
            struct PaintFragment {
                x: f32,
                y: f32,
                #[allow(dead_code)]
                height: f32,
                font_size: f32,
                is_ahem: bool,
                text: String,
                node_id: NodeId,
            }

            let stored_fragments: Vec<PaintFragment> = if use_stored {
                box_node
                    .inline_layout
                    .as_ref()
                    .unwrap()
                    .iter()
                    .flat_map(|line| {
                        line.fragments.iter().filter_map(|f| {
                            f.node_id.map(|nid| PaintFragment {
                                x: f.x,
                                y: f.y,
                                height: f.height,
                                font_size: f.font_size,
                                is_ahem: f.is_ahem,
                                text: f.text.clone(),
                                node_id: nid,
                            })
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // 非存储模式下运行 IFC
            let inline_ctx = if use_stored {
                InlineFormattingContext::new(ifc_width)
            } else {
                // R72: 恢复 override maps 机制。
                // 传递真实 styles 会导致 4 个测试回归（BFC-004, font-feature-002,
                // position-absolute-in-inline-005/006），虽然修复了 float-003。
                // override maps 方式是经过 R37-R71 验证的安全路径。
                // 仅纳入文本节点片段构建父级映射。
                // text_node_* 中混入了内联元素片段（如 <img>，其 font_size=0、height=96），
                // 它们与文本片段共享同一父元素；直接 collect 时 last-write-wins，
                // 结果随 HashMap 迭代顺序（每进程随机）变化 → 渲染非确定性（flaky reftest）。
                // 过滤为纯文本节点后，同一父元素的文本节点继承一致的字号/行高，结果确定。
                let is_text = |tn: zero_dom::NodeId| matches!(doc.get(tn).map(|n| &n.kind), Some(NodeKind::Text(_)));
                let parent_font_sizes: HashMap<zero_dom::NodeId, f32> = box_node
                    .text_node_font_sizes
                    .iter()
                    .filter_map(|(&tn, &fs)| {
                        if !is_text(tn) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, fs))
                    })
                    .collect();

                let parent_is_ahem: HashMap<zero_dom::NodeId, bool> = box_node
                    .text_node_is_ahem
                    .iter()
                    .filter_map(|(&tn, &is_ahem)| {
                        if !is_text(tn) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, is_ahem))
                    })
                    .collect();

                let parent_letter_spacing: HashMap<zero_dom::NodeId, f32> = box_node
                    .text_node_letter_spacing
                    .iter()
                    .filter_map(|(&tn, &ls)| {
                        if !is_text(tn) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, ls))
                    })
                    .collect();

                let parent_line_heights: HashMap<zero_dom::NodeId, f32> = box_node
                    .text_node_line_heights
                    .iter()
                    .filter_map(|(&tn, &lh)| {
                        if !is_text(tn) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, lh))
                    })
                    .collect();

                let inline_metrics = box_node.inline_element_metrics.clone();
                let margin_overrides = box_node.inline_element_margins.clone();

                let mut ctx = InlineFormattingContext::new(ifc_width)
                    .with_text_align(text_align)
                    .with_text_align_last(text_align_last)
                    .with_break_word(break_word)
                    .with_no_wrap(no_wrap)
                    .with_preserve_whitespace(preserve_whitespace)
                    .with_word_break(word_break_mode)
                    .with_text_indent(text_indent_px)
                    .with_float_exclusions(float_exclusions)
                    .with_tab_size(tab_size_px)
                    .with_vertical(is_vertical)
                    .with_vertical_rtl(is_vertical_rtl)
                    .with_font_size_overrides(parent_font_sizes)
                    .with_is_ahem_overrides(parent_is_ahem)
                    .with_letter_spacing_overrides(parent_letter_spacing)
                    .with_line_height_overrides(parent_line_heights)
                    .with_inline_element_metrics(inline_metrics)
                    .with_margin_overrides(margin_overrides);
                // R109 §9.2.1.1：匿名块盒片段——若此盒是 inline 被 block 子元素拆分后的
                // 匿名块片段，只收集该片段的 inline 内容（而非 inline 元素的全部子节点）。
                if let Some(ref frag) = box_node.fragment_node_ids {
                    ctx.set_fragment_node_ids(frag.clone());
                }
                ctx.layout(doc, node_id, &HashMap::new());
                ctx
            };

            let fragments: Vec<&zero_layout_engine::TextFragment> = if use_stored {
                Vec::new()
            } else {
                inline_ctx.all_fragments()
            };

            let has_content = use_stored && !stored_fragments.is_empty() || !fragments.is_empty();

            let needs_ellipsis = matches!(style.text_overflow, TextOverflowValue::Ellipsis)
                && !matches!(style.overflow_x, zero_css_parser::values::OverflowValue::Visible);

            if has_content {
                let glyphs_before_fragments = self.primitives.glyphs.len();

                // writing-mode: vertical-rl/vertical-lr 时字符旋转 90°
                let rotation = if is_vertical { std::f32::consts::FRAC_PI_2 } else { 0.0 };

                if let Some(ref mc) = multicol_info {
                    // 多列布局：遍历行（带 line.y），将行分配到各列
                    let total_height: f32 = inline_ctx.lines.iter().map(|l| l.height).sum();
                    let target_h = total_height / mc.col_count as f32;

                    // 预计算每列首行 y，用于把每列内容 rebase 到列内 y=0。
                    // 旧实现 col_start_y = col_idx * target_h，当 target_h 不是行高整数倍时
                    // （如 29 行 / 2 列 → target_h=14.5 行）首行不在 y=0，列内内容整体偏移。
                    // 取每列实际首行 y 作 col_start_y 可消除该 fractional offset。
                    let col_first_y: Vec<f32> = (0..mc.col_count)
                        .map(|col_idx| {
                            if target_h <= 0.0 {
                                0.0
                            } else {
                                inline_ctx
                                    .lines
                                    .iter()
                                    .find(|l| ((l.y / target_h).floor() as usize).min(mc.col_count - 1) == col_idx)
                                    .map(|l| l.y)
                                    .unwrap_or(0.0)
                            }
                        })
                        .collect();

                    // 按列分组渲染：先收集每列的行索引，再按列渲染并裁剪
                    // 这样可以对每列独立裁剪，防止内容溢出到相邻列
                    for (col_idx, &col_start_y) in col_first_y.iter().enumerate() {
                        let col_x_offset = col_idx as f32 * (mc.col_width + mc.gap);

                        // 裁剪区域：列宽 + 右半间隙，允许内容延伸到间隙
                        let clip_rect = Rect::new(
                            content_x + col_x_offset,
                            content_y,
                            mc.col_width + mc.gap / 2.0,
                            box_node.content_height.max(0.0) + 1000.0,
                        );
                        let counts_before_col = PrimitiveCounts::snapshot(&self.primitives);

                        for line in &inline_ctx.lines {
                            // 根据行的 y 位置确定所在列
                            let line_col = if target_h > 0.0 {
                                (line.y / target_h).floor() as usize
                            } else {
                                0
                            }
                            .min(mc.col_count - 1);

                            if line_col != col_idx {
                                continue;
                            }

                            for fragment in &line.runs {
                                self.painted_inline_nodes.insert(fragment.node_id);

                                // 颜色：取片段所属 inline 元素的 color，绕过 inline ownership
                                // （多列分支统一绘制全部片段）。fragment.node_id 可能是 inline 元素
                                // 也可能是文本节点——文本节点时取其父元素。同时标记 owner 元素，
                                // 使 span 自身的 paint_text 跳过（避免在非列位置重绘）。
                                let owner_id = if doc
                                    .get(fragment.node_id)
                                    .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                {
                                    doc.parent_node(fragment.node_id).unwrap_or(fragment.node_id)
                                } else {
                                    fragment.node_id
                                };
                                self.painted_inline_nodes.insert(owner_id);
                                let frag_color = styles
                                    .and_then(|s| s.get(&owner_id))
                                    .filter(|s| s.color != ColorValue::CurrentColor)
                                    .map(|s| color_value_to_render(&s.color))
                                    .unwrap_or(color);

                                let frag_base_x = content_x + fragment.x + col_x_offset + tx;
                                // 行盒顶部 = (line.y - col_start_y)；基线偏移 v_offset
                                // （Ahem 完美方块顶部对齐 → 0；普通字体 = font_size ≈ ascent）。
                                // is_ahem 用容器 font-family 判定（多列 IFC 的 fragment.is_ahem 不可靠）。
                                let container_is_ahem =
                                    style.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem"));
                                let v_offset = if container_is_ahem { 0.0 } else { fragment.font_size };
                                let frag_base_y = content_y + (line.y - col_start_y) + v_offset + ty;

                                let transformed = apply_text_transform(&fragment.text, &style.text_transform);
                                let mut char_pos = frag_base_x;
                                let frag_is_ahem = fragment.is_ahem;

                                for ch in transformed.chars() {
                                    let glyph_x = char_pos;
                                    let glyph_y = frag_base_y;

                                    if has_text_shadow {
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: glyph_x + shadow_ox,
                                            y: glyph_y + shadow_oy,
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
                                        x: glyph_x,
                                        y: glyph_y,
                                        font_size: fragment.font_size,
                                        color: frag_color,
                                        glyph_id: ch as u32,
                                        font_id: default_font_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                    });

                                    let advance = estimate_char_width(ch, fragment.font_size, frag_is_ahem)
                                        + letter_spacing
                                        + if ch == ' ' { word_spacing } else { 0.0 };
                                    char_pos += advance;
                                }

                                let text_width: f32 = transformed
                                    .chars()
                                    .map(|ch| {
                                        let w =
                                            estimate_char_width(ch, fragment.font_size, frag_is_ahem) + letter_spacing;
                                        if ch == ' ' { w + word_spacing } else { w }
                                    })
                                    .sum();
                                self.paint_text_decoration_from_style(
                                    frag_base_x,
                                    frag_base_y,
                                    fragment.font_size,
                                    text_width,
                                    frag_color,
                                    style,
                                );
                            }
                        }

                        // 对本列的图元应用裁剪
                        super::super::helpers::clip_all_primitives_to_rect(
                            &mut self.primitives,
                            &counts_before_col,
                            &clip_rect,
                        );
                    }
                } else {
                    // 非多列布局：统一处理存储片段和 IFC 片段
                    // 宏化渲染逻辑，避免重复代码
                    macro_rules! render_fragment {
                        ($frag_x:expr, $frag_y:expr, $baseline_offset:expr, $frag_fs:expr, $frag_text:expr, $frag_nid:expr) => {{
                            render_fragment!(
                                $frag_x,
                                $frag_y,
                                $baseline_offset,
                                $frag_fs,
                                $frag_text,
                                $frag_nid,
                                false
                            )
                        }};
                        ($frag_x:expr, $frag_y:expr, $baseline_offset:expr, $frag_fs:expr, $frag_text:expr, $frag_nid:expr, $is_ahem:expr) => {{
                            self.painted_inline_nodes.insert($frag_nid);

                            let (frag_base_x, frag_base_y, char_advance_is_y) = if is_vertical {
                                (content_x + $frag_x + tx, content_y + $frag_y + ty, true)
                            } else {
                                (
                                    content_x + $frag_x + tx,
                                    content_y + $frag_y + $baseline_offset + ty,
                                    false,
                                )
                            };
                            let mut char_pos = if char_advance_is_y {
                                frag_base_y
                            } else {
                                frag_base_x
                            };

                            let transformed = apply_text_transform(&$frag_text, &style.text_transform);

                            for ch in transformed.chars() {
                                let (glyph_x, glyph_y) = if char_advance_is_y {
                                    (frag_base_x, char_pos)
                                } else {
                                    (char_pos, frag_base_y)
                                };

                                if has_text_shadow {
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: glyph_x + shadow_ox,
                                        y: glyph_y + shadow_oy,
                                        font_size: $frag_fs,
                                        color: shadow_color,
                                        glyph_id: ch as u32,
                                        font_id: default_font_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                    });
                                }

                                self.primitives.add_glyph(GlyphPrimitive {
                                    x: glyph_x,
                                    y: glyph_y,
                                    font_size: $frag_fs,
                                    color,
                                    glyph_id: ch as u32,
                                    font_id: default_font_id,
                                    bitmap_width: None,
                                    bitmap_height: None,
                                    rotation,
                                });

                                let advance = estimate_char_width(ch, $frag_fs, $is_ahem)
                                    + letter_spacing
                                    + if ch == ' ' { word_spacing } else { 0.0 };
                                char_pos += advance;
                            }

                            let text_width: f32 = transformed
                                .chars()
                                .map(|ch| {
                                    let w = estimate_char_width(ch, $frag_fs, $is_ahem) + letter_spacing;
                                    if ch == ' ' { w + word_spacing } else { w }
                                })
                                .sum();
                            self.paint_text_decoration_from_style(
                                frag_base_x,
                                frag_base_y,
                                $frag_fs,
                                text_width,
                                color,
                                style,
                            );
                        }};
                    }

                    if use_stored {
                        for frag in &stored_fragments {
                            // 存储结果：frag.y 是片段框顶部（baseline_y - height）。
                            // 基线偏移：Ahem 字形位图是完美 font_size 方块（无内部 ascent 留白），
                            // 位图顶部应与行盒顶部对齐 → offset=0；普通字体保留 font_size（≈ascent）。
                            // 仅在 stored 路径生效（compute_final 守卫保证 stored 片段为纯 Ahem 或
                            // 其 is_ahem 可靠），非存储路径不在此处调整（见 else 分支）。
                            let v_offset = if frag.is_ahem { 0.0 } else { frag.font_size };
                            render_fragment!(
                                frag.x,
                                frag.y,
                                v_offset,
                                frag.font_size,
                                frag.text,
                                frag.node_id,
                                frag.is_ahem
                            );
                        }
                    } else {
                        for fragment in fragments.iter() {
                            // IFC 片段（空 styles）：frag.y 基于 16px 默认值，
                            // 使用存储的 font_size（来自 layout IFC）计算基线偏移。
                            // 如果无存储值，回退到 16px 默认值（保持原有行为）。
                            let stored_fs = box_node.text_node_font_sizes.get(&fragment.node_id).copied();
                            let baseline_fs = stored_fs.unwrap_or(fragment.font_size);
                            render_fragment!(
                                fragment.x,
                                fragment.y,
                                baseline_fs,
                                stored_fs.unwrap_or(fragment.font_size),
                                fragment.text,
                                fragment.node_id,
                                fragment.is_ahem
                            );
                        }
                    }
                } // end non-multicol else block

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

    /// 绘制匿名文本项（flex/grid 容器中的文本节点）。
    ///
    /// 与 paint_text 不同，此方法直接渲染 node_id 指向的文本节点内容，
    /// 而非查找子文本节点。匿名文本项没有独立的 ComputedStyle，
    /// 使用父元素的样式。
    pub fn paint_anonymous_text_item(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
        node_id: NodeId,
    ) {
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 || style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);
        let default_font_id = self.resolve_font_id(&style.font_family);
        let content_x = abs_x;
        let content_y = abs_y;

        // 直接从文本节点获取内容
        let text = match doc.get(node_id) {
            Some(node) => match &node.kind {
                NodeKind::Text(data) => data.content.trim().to_string(),
                _ => return,
            },
            None => return,
        };
        if text.is_empty() {
            return;
        }

        // 渲染文本字符为 glyph primitives
        let is_ahem = style.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem"));
        let mut char_x = content_x;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: content_y + font_size,
                font_size,
                color,
                glyph_id: ch as u32,
                font_id: default_font_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
            });
            char_x += estimate_char_width(ch, font_size, is_ahem);
        }
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
