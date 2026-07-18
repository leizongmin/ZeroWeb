//! 列表标记（list marker）渲染 + 计数器格式化 helper。
//!
//! R1694 从 painter/text.rs 抽离（text.rs 减负，单文件超 2000 行 guideline）。
//! 计数器格式化（Roman / Latin 字母序号）+ `<li>` 的 paint_list_marker Painter 方法 +
//! compute_list_item_index 兄弟索引。paint_content（CSS content 计数器）通过
//! `use super::text_list::{format_counter_alpha, format_counter_roman}` 复用格式化函数。

use zero_css_parser::values::{LengthValue, ListStyleTypeValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, LineCap, StrokePrimitive};
use zero_style_system::ComputedStyle;

use crate::measure_char_for_paint;
use crate::paint::color::color_value_to_render;
use crate::paint::helpers::simple_hash;

/// 将正整数转为大写罗马数字（lowercase 由调用方 `to_lowercase()`）。
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

/// 将计数器值格式化为字母序列（a/b/.../z/aa/ab/...）。
pub(super) fn format_counter_alpha(value: i32, upper: bool) -> String {
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
pub(super) fn format_counter_roman(value: i32, upper: bool) -> String {
    let s = to_roman(value.max(0) as usize);
    if upper { s } else { s.to_lowercase() }
}

impl super::super::Painter {
    /// 绘制列表项标记（disc / circle / square / decimal / alpha / roman / list-style-image）。
    pub(crate) fn paint_list_marker(
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
                    image_key: ImageKey::new(simple_hash(url)),
                    clip: None,
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
        let default_font_id = self.resolve_font_id(&style.font_family, &style.font_weight);
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
                    char_x += measure_char_for_paint(ch, font_size * 0.85, false);
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
                    char_x += measure_char_for_paint(ch, font_size * 0.85, false);
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
                    char_x += measure_char_for_paint(ch, font_size * 0.85, false);
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
}
