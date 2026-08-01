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
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, RoundedRectPrimitive};
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

/// R2392/R2394：按 `@counter-style` 的 system 算法生成计数器表示（marker body，不含 prefix/suffix）。
/// CSS Counter Styles 3 §3.1.4。`None` = 该值无法表示（超出 range / 系统不支持）→ 调用方走 fallback。
/// R2394 注：additive/range/extends 应用经 A/B 量证为 net-negative（driving WPT 全 font-wall dice/
/// triangle 字形 + system-additive ref 依赖 document.write JS + system-extends nbsp/marker 渲染差），
/// 故**应用 defer**（parse-retain，见 ast.rs 字段 + parser.rs 解析）；本函数仅消费已落地 5 系统 +
/// cyclic 数学取模修正。
fn counter_style_body(rule: &zero_css_parser::ast::CounterStyleRule, value: i32) -> Option<String> {
    use zero_css_parser::ast::CounterSystem;
    let syms = &rule.symbols;
    let len = syms.len();
    if len == 0 {
        return None;
    }
    match rule.system {
        // R2394：cyclic 用数学取模（rem_euclid），表示任意整数（含 0/负数）；CSS §3.1.4 cyclic
        // 不限值域。旧 `value < 1 → None` 致 disclosure-* 等 cyclic value 0 永远 fallback。
        CounterSystem::Cyclic => Some(syms[(value - 1).rem_euclid(len as i32) as usize].clone()),
        // fixed [N]：symbols[value - first]；超出 symbols 范围走 fallback。
        CounterSystem::Fixed(first) => {
            let first = first.unwrap_or(1);
            if value < first {
                return None;
            }
            let offset = value - first;
            if (offset as usize) < len {
                Some(syms[offset as usize].clone())
            } else {
                None
            }
        }
        // symbolic：symbols[(value-1) % len] × ceil(value/len) 次（value >= 1）。
        CounterSystem::Symbolic => {
            if value < 1 {
                return None;
            }
            let idx = ((value - 1) as usize) % len;
            let reps = ((value - 1) as usize) / len + 1;
            Some(syms[idx].repeat(reps))
        }
        // alphabetic：双射 base-len（无零位，类似 Excel 列名）。value >= 1。
        CounterSystem::Alphabetic => {
            if value < 1 || len < 2 {
                // len < 2 无法表示 > 1 的值（spec：alphabetic 须 ≥2 symbols）。
                return if len == 1 && value == 1 {
                    Some(syms[0].clone())
                } else {
                    None
                };
            }
            let mut n = value as usize;
            let mut out = String::new();
            while n > 0 {
                n -= 1;
                out.insert(0, syms[n % len].chars().next().unwrap_or(' '));
                n /= len;
            }
            Some(out)
        }
        // numeric：标准 base-len（含零位）。value >= 0；value 0 → symbols[0]。
        CounterSystem::Numeric => {
            if value < 0 || len < 2 {
                return if len == 1 && (0..=1).contains(&value) {
                    Some(syms[0].clone())
                } else {
                    None
                };
            }
            let mut n = value as usize;
            if n == 0 {
                return Some(syms[0].clone());
            }
            let mut digits: Vec<usize> = Vec::new();
            while n > 0 {
                digits.push(n % len);
                n /= len;
            }
            // digits 收集为低位在前，输出需反转并取每 symbol 首字符。
            let out: String = digits
                .iter()
                .rev()
                .map(|&d| syms[d].chars().next().unwrap_or(' '))
                .collect();
            Some(out)
        }
        // additive / extends：应用 defer（R2394 A/B 量证 net-negative，见函数注释）→ None（fallback）。
        CounterSystem::Additive | CounterSystem::Extends(_) => None,
    }
}

/// R2392：从 stylesheets 收集 `@counter-style` 定义为注册表（name → rule，大小写敏感保留）。
/// 镜像 `animation::register_from_stylesheets` 的 @keyframes 收集模式。
pub(crate) fn build_counter_style_registry(
    stylesheets: &[zero_css_parser::Stylesheet],
) -> std::collections::HashMap<String, zero_css_parser::ast::CounterStyleRule> {
    use zero_css_parser::ast::Rule;
    let mut map = std::collections::HashMap::new();
    for ss in stylesheets {
        for rule in &ss.rules {
            if let Rule::CounterStyle(cs) = rule {
                // CSS Counter Styles 3：计数器名大小写敏感（counter-name-case-sensitive）。
                map.entry(cs.name.clone()).or_insert_with(|| cs.clone());
            }
        }
    }
    map
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

        match &style.list_style_type {
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
                // R1883：circle 是空心圆 outline（CSS §12.5 / chromium），非水平线胶囊。
                // 旧 add_stroke（length=width=marker_size + Round cap）实为 2:1 胶囊（椭圆），
                // 非圆。改用 PathStroke 多边形（24 点圆周）描真圆，line_width 细（~0.2em）。
                let cx = actual_marker_x + marker_size / 2.0;
                let cy = marker_y + font_size * 0.3;
                let radius = marker_size / 2.0;
                let line_w = (marker_size * 0.2).max(1.0);
                let steps = 24;
                let mut verts: Vec<f32> = Vec::with_capacity(steps * 2);
                for i in 0..steps {
                    let theta = (i as f32) * (2.0 * std::f32::consts::PI / steps as f32);
                    verts.push(cx + radius * theta.cos());
                    verts.push(cy + radius * theta.sin());
                }
                self.primitives.add_path_stroke(verts, color, line_w, true);
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
            // R2392：自定义计数器样式（@counter-style）。查注册表 → 按 system 生成 body
            // → prefix+body+suffix。未定义 / 超出 range → fallback（decimal "N."）。
            // R2394 注：additive/extends 应用 defer（A/B net-negative，见 counter_style_body 注释）。
            ListStyleTypeValue::Custom(name) => {
                let index = self
                    .get_counter("list-item")
                    .map(|v| v as usize)
                    .unwrap_or_else(|| self.compute_list_item_index(doc, node_id));
                let value = index as i32;
                let text = match self.counter_styles.get(name) {
                    Some(rule) => match counter_style_body(rule, value) {
                        Some(body) => format!("{}{}{}", rule.prefix, body, rule.suffix),
                        // body 超出 range → fallback（decimal）。
                        None => format!("{index}."),
                    },
                    // 未定义的自定义名 → fallback decimal（CSS Counter Styles 3 §3.1.3）。
                    None => format!("{index}."),
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
    use super::counter_style_body;
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

    /// R2392：构造最小 CounterStyleRule（仅 system + symbols）用于生成测试。
    fn cs_rule(
        system: zero_css_parser::ast::CounterSystem,
        symbols: &[&str],
    ) -> zero_css_parser::ast::CounterStyleRule {
        zero_css_parser::ast::CounterStyleRule {
            name: "test".to_string(),
            system,
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            additive_symbols: Vec::new(),
            prefix: String::new(),
            suffix: ". ".to_string(),
            fallback: "decimal".to_string(),
            range: None,
        }
    }

    #[test]
    fn test_counter_style_cyclic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Cyclic, &["a", "b", "c"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "c");
        assert_eq!(counter_style_body(&r, 4).unwrap(), "a"); // 循环回 a
        assert_eq!(counter_style_body(&r, 6).unwrap(), "c");
        // R2394：cyclic 表示任意整数（数学取模）；value 0 → syms[(-1)%3]=syms[2]="c"。
        assert_eq!(counter_style_body(&r, 0).unwrap(), "c");
        assert_eq!(counter_style_body(&r, -1).unwrap(), "b"); // (-2)%3=1 → "b"
    }

    #[test]
    fn test_counter_style_fixed() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Fixed(Some(5)), &["v", "w", "x"]);
        assert_eq!(counter_style_body(&r, 5).unwrap(), "v"); // first=5
        assert_eq!(counter_style_body(&r, 7).unwrap(), "x");
        assert!(counter_style_body(&r, 8).is_none(), "超出 symbols 范围走 fallback");
        assert!(counter_style_body(&r, 4).is_none(), "< first 应 None");
        // Fixed(None) 默认 first=1。
        let r2 = cs_rule(zero_css_parser::ast::CounterSystem::Fixed(None), &["a", "b"]);
        assert_eq!(counter_style_body(&r2, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r2, 2).unwrap(), "b");
    }

    #[test]
    fn test_counter_style_symbolic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Symbolic, &["*"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "*");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "**");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "***");
        // 多 symbol：idx 循环 + reps 递增。
        let r2 = cs_rule(zero_css_parser::ast::CounterSystem::Symbolic, &["*", "†"]);
        assert_eq!(counter_style_body(&r2, 1).unwrap(), "*");
        assert_eq!(counter_style_body(&r2, 2).unwrap(), "†");
        assert_eq!(counter_style_body(&r2, 3).unwrap(), "**");
        assert_eq!(counter_style_body(&r2, 4).unwrap(), "††");
    }

    #[test]
    fn test_counter_style_alphabetic() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Alphabetic, &["a", "b"]);
        assert_eq!(counter_style_body(&r, 1).unwrap(), "a");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "b");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "aa"); // 双射进位
        assert_eq!(counter_style_body(&r, 4).unwrap(), "ab");
        assert_eq!(counter_style_body(&r, 5).unwrap(), "ba");
        assert_eq!(counter_style_body(&r, 6).unwrap(), "bb");
    }

    #[test]
    fn test_counter_style_numeric() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Numeric, &["0", "1", "2"]);
        assert_eq!(counter_style_body(&r, 0).unwrap(), "0");
        assert_eq!(counter_style_body(&r, 1).unwrap(), "1");
        assert_eq!(counter_style_body(&r, 2).unwrap(), "2");
        assert_eq!(counter_style_body(&r, 3).unwrap(), "10"); // base-3 进位
        assert_eq!(counter_style_body(&r, 4).unwrap(), "11");
        assert_eq!(counter_style_body(&r, 6).unwrap(), "20");
    }

    #[test]
    /// R2394：additive 系统 → 应用 defer（None，走 fallback）。parse-retain 见 parser 测试。
    fn test_counter_style_additive_deferred() {
        let r = cs_rule(zero_css_parser::ast::CounterSystem::Additive, &["a"]);
        assert!(counter_style_body(&r, 1).is_none());
    }
}
