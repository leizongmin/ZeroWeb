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
use zero_render_foundation::primitive::{
    GlyphPrimitive, ImagePrimitive, LineCap, RoundedRectPrimitive, StrokePrimitive,
};
use zero_style_system::ComputedStyle;

use crate::measure_char_for_paint;
use crate::paint::color::color_value_to_render;
use crate::paint::helpers::image_resource_key;

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
                    image_key: ImageKey::new(image_resource_key(url, self.document_url.as_deref())),
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
                // R1882：disc 是实心圆（CSS §12.5 / chromium），非方块。用圆角矩形
                //（radius = marker_size/2 = 正方形四角全圆 → 圆）近似实心圆 marker。
                self.primitives.add_rounded_rect(RoundedRectPrimitive::uniform(
                    Rect::new(
                        actual_marker_x,
                        marker_y + font_size * 0.3 - marker_size / 2.0,
                        marker_size,
                        marker_size,
                    ),
                    color,
                    marker_size / 2.0,
                ));
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
        list_item_counter(doc, node_id) as usize
    }
}

/// R1701：计算 `<li>` 的列表序号（counter），尊重 HTML4 `<ol start=N>` 起始值
/// 与 `<li value=N>` 重置值（后续 li 从 value+1 继续）。无属性时等价 1-based 兄弟
/// 位置（向后兼容）。fixture 22 `<ol start="3" type="A">` → C/D/J(`value=10`)/K。
pub(super) fn list_item_counter(doc: &Document, node_id: NodeId) -> i64 {
    let parent_id = match doc.parent_node(node_id) {
        Some(id) => id,
        None => return 1,
    };
    // <ol start=N>：默认 1；负数/非数字忽略（HTML4 start 须为整数）。
    let start: i64 = doc
        .get_attribute(parent_id, "start")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(1);
    let mut counter = start;
    let mut found = false;
    for child_id in doc.child_nodes(parent_id) {
        if child_id == node_id {
            found = true;
            break;
        }
        if is_li(doc, child_id) {
            // 该 li 兄弟消耗一个序号；若带 value=N，先把 counter 重置为 N。
            if let Some(v) = doc
                .get_attribute(child_id, "value")
                .and_then(|s| s.trim().parse::<i64>().ok())
            {
                counter = v;
            }
            counter += 1;
        }
    }
    if !found {
        return 1;
    }
    // 目标 li 自身 value= 设其序号（其上方兄弟的循环已从 value+1 继续）。
    if let Some(v) = doc
        .get_attribute(node_id, "value")
        .and_then(|s| s.trim().parse::<i64>().ok())
    {
        return v.max(1);
    }
    counter.max(1)
}

fn is_li(doc: &Document, id: NodeId) -> bool {
    doc.get(id)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "li"))
}

#[cfg(test)]
mod tests {
    use super::list_item_counter;
    use zero_dom::parse_html;

    fn li(doc: &zero_dom::Document, n: usize) -> zero_dom::NodeId {
        doc.get_elements_by_tag_name("li")[n]
    }

    /// R1701：ol start= 与 li value= 计数器语义（fixture 22 ol[start=3] type=A → C/D/J/K）。
    #[test]
    fn list_counter_respects_start_and_value_attrs() {
        let doc = parse_html("<ol start=\"3\"><li>a</li><li>b</li><li value=\"10\">c</li><li>d</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 3); // C
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 4); // D
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 10); // J（value=10）
        assert_eq!(list_item_counter(&doc, li(&doc, 3)), 11); // K（从 10+1 继续）
    }

    /// 无 start=/value= 时等价 1-based 兄弟位置（向后兼容，R1701 前行为）。
    #[test]
    fn list_counter_default_is_one_based_position() {
        let doc = parse_html("<ol><li>a</li><li>b</li><li>c</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 1);
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 2);
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 3);
    }

    /// li value= 在中间重置后续计数（value=5 后续 6/7）。
    #[test]
    fn list_counter_value_attr_resets_running_counter() {
        let doc = parse_html("<ol><li>a</li><li value=\"5\">b</li><li>c</li></ol>");
        assert_eq!(list_item_counter(&doc, li(&doc, 0)), 1);
        assert_eq!(list_item_counter(&doc, li(&doc, 1)), 5); // value=5
        assert_eq!(list_item_counter(&doc, li(&doc, 2)), 6); // 从 5+1 继续
    }
}
