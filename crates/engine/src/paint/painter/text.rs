//! 文本和列表绘制 — 文本内容、列表标记、列分隔线。
//!
//! 包含 paint_text、paint_list_marker、compute_list_item_index、paint_column_rules。

use std::collections::HashMap;

use crate::measure_char_for_paint;
use zero_css_parser::values::{ColorValue, FloatValue, LengthValue, ListStyleTypeValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::inline_finalization::subtree_has_text_decoration;
use zero_layout_engine::{FloatExclusion, InlineFormattingContext, LayoutBox, TextAlign, WordBreakMode};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, LineCap, StrokePrimitive};
use zero_style_system::{
    ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue, ColumnWidthComputedValue,
    ComputedStyle, ContentComputedValue, DirectionValue, ObjectFitComputedValue, TabSizeValue, TextAlignLastValue,
    TextAlignValue, TextEmphasisPositionValue, TextEmphasisStyleValue, TextOverflowValue, TextTransformValue,
    WhiteSpaceValue,
};

use super::super::color::color_value_to_render;
use super::super::helpers::PrimitiveCounts;
use super::super::helpers::apply_text_transform;

/// R644：判断是否为 Cc 类控制字符（非空白）。CSS Text 3 §white-space-processing 要求
/// 控制字符（Unicode 类别 Cc）必须可见；但 fontdue 对 Cc 无字形（.notdef 空白），
/// 故 paint 时渲染可见占位框（修 control-chars-* mismatch 测试：test 应 != 空 ref）。
/// 排除空白控制符（U+0009 TAB / U+000A LF / U+000C FF / U+000D CR），它们由换行/空白处理。
pub(super) fn is_cc_control_char(ch: char) -> bool {
    let cp = ch as u32;
    ((cp <= 0x1F) || (0x7F..=0x9F).contains(&cp)) // Cc category
        && !matches!(cp, 0x09 | 0x0A | 0x0C | 0x0D) // 排除空白 TAB/LF/FF/CR
}

/// R841：判定真正 Ahem 方块字形是否使用 em-box 位（glyph 顶 = 基线 − 0.8·fs）而非
/// R817 默认位（基线 − fs）。
///
/// Chromium 的有效 Ahem 方块位**随 line-height 变**（R839 实测）：当 half-leading ≈ 0
/// （即 line-height ≈ font-size，如 lh:1 / lh:1em / Ahem lh:normal=1.0）时，方块填满
/// 整个 line-box，em-box 位才是正确的（修 inline-formatting-context-008、line-height-121）；
/// 当 line-height 偏离 font-size（lh:0 行盒塌缩 / lh>1 含 leading）时，R817 的基线−fs 位
/// 对多数用例更接近 chromium（R839 妥协）。
///
/// R837 全量应用 em-box 位反致 27 个 line-height:0 用例 0.99%→1.02% 越过 1% 阈值
/// （见 evidence/r841-*）；本门控仅对 half-leading≈0 的子集启用，得 +2 零回归。
pub(super) fn ahem_uses_embox_position(line_height: f32, font_size: f32) -> bool {
    let half_leading = (line_height - font_size) / 2.0;
    half_leading.abs() < 0.5
}

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

/// R1022：收集 `<ruby>` owner 的 `<rt>` 后代文本作 annotation。
///
/// 返回非空白字符序列（rt 标记逐字符配对 rb 文本，paint 期上移到 rb 之上）。
/// owner 非 ruby 元素或无 rt 文本时返回 None。
fn ruby_annotation_chars(doc: &Document, owner_id: NodeId) -> Option<Vec<char>> {
    let owner = doc.get(owner_id)?;
    if !matches!(&owner.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("ruby")) {
        return None;
    }
    let mut text = String::new();
    collect_ruby_rt_text(doc, owner_id, &mut text);
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() { None } else { Some(chars) }
}

/// 递归收集 `id` 子树内所有 `<rt>` 元素的文本。
fn collect_ruby_rt_text(doc: &Document, id: NodeId, out: &mut String) {
    for child_id in doc.child_nodes(id) {
        if let Some(node) = doc.get(child_id)
            && let NodeKind::Element(elem) = &node.kind
        {
            if elem.local_name().eq_ignore_ascii_case("rt") {
                if let Some(t) = doc.text_content(child_id) {
                    out.push_str(&t);
                }
            } else {
                // 递归查找嵌套 ruby 中的 rt（如 ruby 嵌套）
                collect_ruby_rt_text(doc, child_id, out);
            }
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

        // R1029：column-span:all spanner 使 column-rule 在 spanner 处中断（CSS Multicol §6.1）。
        // 检测直接子元素中的 spanner（in-flow + column_span_offsets 被清空 + 全宽——非 spanner
        // 的列子元素被 position_multicol_children narrow 到 col_w 且 column_span_offsets 非空），
        // 把 rule 的 [0, content_h] Y 范围按 spanner Y 区间分段，每段独立绘制。
        // 非 spanner 容器 → spanner_ranges 空 → segments = [(0, content_h)] → 行为不变（零回归）。
        let spanner_ranges: Vec<(f32, f32)> = box_node
            .children
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed && c.column_span_offsets.is_empty() && c.width >= content_w - 1.0)
            .map(|c| (c.y, c.y + c.height))
            .collect();
        let mut segments: Vec<(f32, f32)> = vec![(0.0, content_h)];
        for &(s_start, s_end) in &spanner_ranges {
            let mut next = Vec::new();
            for (seg_start, seg_end) in segments {
                if s_end <= seg_start || s_start >= seg_end {
                    // spanner 与 segment 无重叠，保留整段。
                    next.push((seg_start, seg_end));
                } else {
                    // 重叠：保留 spanner 之前/之后的剩余部分。
                    if s_start > seg_start {
                        next.push((seg_start, s_start));
                    }
                    if s_end < seg_end {
                        next.push((s_end, seg_end));
                    }
                }
            }
            segments = next;
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
            // R1029：按 spanner 分段绘制 column-rule（非 spanner 容器 segments 只有一段 [0, content_h]，
            // 与原行为一致）。
            for &(seg_start, seg_end) in &segments {
                let seg_h = seg_end - seg_start;
                if seg_h <= 0.5 {
                    continue;
                }
                let seg_y = content_y + seg_start;
                match style.column_rule_style {
                    ColumnRuleStyleComputedValue::Solid => {
                        self.primitives
                            .add_fill(Rect::new(rule_x, seg_y, rule_w, seg_h), rule_color);
                    }
                    ColumnRuleStyleComputedValue::Dotted => {
                        self.primitives.add_stroke(StrokePrimitive {
                            x1: rule_x + rule_w / 2.0,
                            y1: seg_y,
                            x2: rule_x + rule_w / 2.0,
                            y2: seg_y + seg_h,
                            width: rule_w,
                            color: rule_color,
                            style: zero_render_foundation::primitive::LineStyle::Dotted,
                            cap: LineCap::Round,
                        });
                    }
                    ColumnRuleStyleComputedValue::Dashed => {
                        self.primitives.add_stroke(StrokePrimitive {
                            x1: rule_x + rule_w / 2.0,
                            y1: seg_y,
                            x2: rule_x + rule_w / 2.0,
                            y2: seg_y + seg_h,
                            width: rule_w,
                            color: rule_color,
                            style: zero_render_foundation::primitive::LineStyle::Dashed,
                            cap: LineCap::Square,
                        });
                    }
                    _ => {
                        self.primitives
                            .add_fill(Rect::new(rule_x, seg_y, rule_w, seg_h), rule_color);
                    }
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
        let default_font_id = self.resolve_font_id(&style.font_family, &style.font_weight);
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
            char_x += measure_char_for_paint(ch, font_size, false);
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
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let image_hash = super::super::helpers::image_resource_key(&src, self.document_url.as_deref());
        let image_key = ImageKey::new(image_hash);

        // 与布局阶段保持一致：优先使用解码后的真实图片尺寸；若图片未进入缓存，再回退到
        // HTML width/height 属性，最后才退回容器尺寸。
        let decoded_size = self.get_image_size(image_hash);
        let (intrinsic_w, intrinsic_h) = get_img_intrinsic_size(node, decoded_size, container_w, container_h);

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
            clip: Some(Rect::new(content_x, content_y, container_w, container_h)),
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

        let default_font_id = self.resolve_font_id(&style.font_family, &style.font_weight);

        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            // R109 §9.2.1.1：被 in-flow block 子元素拆分的 inline 父盒自身不渲染文本——
            // 其直接文本已由匿名块片段子盒（带 fragment_node_ids）渲染。避免与片段重叠。
            if box_node.is_r109_split && box_node.fragment_node_ids.is_none() {
                return;
            }
            if !has_direct_paintable_text(doc, node_id, styles) {
                return;
            }
            // R109：匿名块片段跳过 painted_inline_nodes 去重——多个片段共享 inline 的
            // node_id，首个片段渲染后会标记该 id，须放行后续片段各自渲染其片段文本。
            if box_node.fragment_node_ids.is_none() && self.painted_inline_nodes.contains(&node_id) {
                return;
            }

            // R1099 Slice α-1（vertical-mode IFC 四层协调）：container_width WM-aware。
            // vertical-rl/lr 下 IFC 重跑须与 layout 侧（inline_finalization.rs）同取 content_height
            //（竖直 inline 尺寸 = 字符向下推进可用深度），非 content_width。horizontal-tb 零回归。
            // decoration-gate（TBD-2）：vertical 容器子树有 text-decoration/emphasis 时保持
            // content_width（旧行为），回避 Layer 4 装饰坐标耦合（α-3 未实施）。
            let is_vertical_wm = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let vertical_decoration_free = styles.is_some_and(|s| {
                box_node
                    .node_id
                    .is_some_and(|id| !subtree_has_text_decoration(doc, s, id))
            });
            let container_width = if is_vertical_wm && vertical_decoration_free {
                box_node.content_height
            } else {
                box_node.content_width
            };

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
            // CSS Text 3 §5.3：line-break: anywhere → BreakAll（与 inline_finalization 一致，
            // layout/paint 双路径同步，避免 R1004/R989 类发散）。
            let word_break_mode = if matches!(style.line_break, zero_style_system::LineBreakValue::Anywhere) {
                WordBreakMode::BreakAll
            } else {
                word_break_mode
            };

            // 将 CSS text-align 映射到布局引擎的 TextAlign。
            // `start`/`end` 方向感知（CSS Text 3 §6.1）：start = LTR→left / RTL→right，end 反之。
            let is_rtl = matches!(style.direction, DirectionValue::Rtl);
            let text_align = match style.text_align {
                TextAlignValue::Left => TextAlign::Left,
                TextAlignValue::Right => TextAlign::Right,
                TextAlignValue::Center => TextAlign::Center,
                TextAlignValue::Justify => TextAlign::Justify,
                TextAlignValue::Start => {
                    if is_rtl {
                        TextAlign::Right
                    } else {
                        TextAlign::Left
                    }
                }
                TextAlignValue::End => {
                    if is_rtl {
                        TextAlign::Left
                    } else {
                        TextAlign::Right
                    }
                }
            };

            // 将 CSS text-align-last 映射到布局引擎（Auto = 跟随 text-align）。
            // start/end 同样方向感知。
            let text_align_last = match &style.text_align_last {
                TextAlignLastValue::Auto => None,
                TextAlignLastValue::Left => Some(TextAlign::Left),
                TextAlignLastValue::Right => Some(TextAlign::Right),
                TextAlignLastValue::Center => Some(TextAlign::Center),
                TextAlignLastValue::Justify => Some(TextAlign::Justify),
                TextAlignLastValue::Start => Some(if is_rtl { TextAlign::Right } else { TextAlign::Left }),
                TextAlignLastValue::End => Some(if is_rtl { TextAlign::Left } else { TextAlign::Right }),
            };

            // text-indent 首行缩进（px）：Px/Em（×font_size）/Percentage（×container_width，CSS §10.3.1）。
            // 与 layout 路径（inline_finalization::resolve_text_indent）保持一致（IFC 双路径同源）。
            let text_indent_px: f32 = match style.text_indent {
                LengthValue::Px(v) => v as f32,
                LengthValue::Em(v) => v as f32 * font_size,
                LengthValue::Percentage(v) => v as f32 / 100.0 * container_width,
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
                // R817 Phase 2：片段基线绝对 y（container-rel = line.y + line.baseline_y）。
                // 供 is_ahem glyph 定位用（见 stored 渲染循环），paint 非存储路径不读。
                baseline_y_abs: f32,
                height: f32,
                font_size: f32,
                is_ahem: bool,
                is_ahem_font: bool,
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
                        // R355：多行存储需把行盒垂直偏移（line.y）加到片段 y 上——
                        // 存储片段 f.y 是行内相对（恒为 0），line.y 才是行盒在容器内的位置。
                        // R207 单行存储时 line.y==0 故无影响；R355 多行若不加 line.y，
                        // 所有行渲染在容器顶部 y=0 互相覆盖（ifc-008 底半红露白）。
                        let line_y = line.y;
                        line.fragments.iter().filter_map(move |f| {
                            f.node_id.map(|nid| PaintFragment {
                                x: f.x,
                                y: line_y + f.y,
                                baseline_y_abs: line_y + f.baseline_y,
                                height: f.height,
                                font_size: f.font_size,
                                is_ahem: f.is_ahem,
                                is_ahem_font: f.is_ahem_font,
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

                // R1012：text-transform 覆盖（re-key 文本节点 → 父元素），让 paint Path B
                // 空 styles IFC 也能在 collect_inline_items 期应用 transform，使行断用
                // 转换后文本宽度（与 layout IFC / chromium 一致）。None 不插入（保持默认）。
                let parent_text_transforms: HashMap<zero_dom::NodeId, TextTransformValue> = box_node
                    .text_node_text_transform
                    .iter()
                    .filter_map(|(&tn, &tt)| {
                        if !is_text(tn) || matches!(tt, TextTransformValue::None) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, tt))
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
                    .with_text_autospace(style.text_autospace)
                    .with_text_indent(text_indent_px)
                    .with_float_exclusions(float_exclusions)
                    .with_tab_size(tab_size_px)
                    .with_vertical(is_vertical)
                    .with_vertical_rtl(is_vertical_rtl)
                    .with_block_extent(
                        if is_vertical
                            && styles.is_some_and(|s| {
                                box_node.node_id.is_some_and(|id| {
                                    s.get(&id).is_some_and(|st| {
                                        matches!(st.display, zero_css_parser::values::DisplayValue::TableCaption)
                                    })
                                })
                            })
                        {
                            box_node.content_width
                        } else {
                            container_width
                        },
                    )
                    .with_font_size_overrides(parent_font_sizes)
                    .with_is_ahem_overrides(parent_is_ahem)
                    .with_letter_spacing_overrides(parent_letter_spacing)
                    .with_line_height_overrides(parent_line_heights)
                    .with_text_transform_overrides(parent_text_transforms)
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

            // 多行块的片段 y 必须包含行盒 y 偏移（line.y），否则多行文本垂直堆叠。
            // `all_fragments()` 返回行内相对 y（恒 0），`all_fragments_with_line_y()`
            // 把 line.y 加到片段 y。
            //
            // **统一使用 with_line_y（R246 限制解除，2026-06-25）**：R246 曾把此修复限定在
            // preserve_whitespace（pre 族），因 auto-wrap 多行块的 test/ref 此前都堆叠同错，
            // 修后反致同源 reftest 净 -11 回归。但实测确认 auto-wrap 多行堆叠是真实 bug
            //（layout 算对多行 h，paint 把多行画在同一 y）——用户可见的"文字堆叠看不清"。
            // 同源 -11 是「test/ref 同错用例的诚实化暴露」（DC-14 视角为进步），非真退步；
            // product-smoke（真实网站）维度此修复为正收益。故统一对所有 Path B 应用 with_line_y。
            let fragments: Vec<zero_layout_engine::TextFragment> = if use_stored {
                Vec::new()
            } else {
                inline_ctx.all_fragments_with_line_y()
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
                                let owner_style = styles.and_then(|s| s.get(&owner_id));
                                let frag_color = owner_style
                                    .filter(|s| s.color != ColorValue::CurrentColor)
                                    .map(|s| color_value_to_render(&s.color))
                                    .unwrap_or(color);
                                // R1021：text-emphasis 标记取自片段 owner 样式（<span> 上设的属性），
                                // 非容器 style。None/Char 判定 + 位置均来自 owner。
                                let emphasis_mark: Option<char> =
                                    owner_style.and_then(|s| match s.text_emphasis_style {
                                        TextEmphasisStyleValue::Char(c) => Some(c),
                                        TextEmphasisStyleValue::None => None,
                                    });
                                let emphasis_over = owner_style
                                    .map(|s| {
                                        matches!(
                                            s.text_emphasis_position,
                                            TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                        )
                                    })
                                    .unwrap_or(true);
                                // R1022：ruby annotation —— owner 为 <ruby> 时，rt 后代文本逐字符上移。
                                let ruby_marks: Option<Vec<char>> = ruby_annotation_chars(doc, owner_id);

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

                                for (char_idx, ch) in transformed.chars().enumerate() {
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

                                    let advance = measure_char_for_paint(ch, fragment.font_size, frag_is_ahem)
                                        + letter_spacing
                                        + if ch == ' ' { word_spacing } else { 0.0 };
                                    char_pos += advance;

                                    // R1021：text-emphasis 标记（CSS Text Decoration 3 §3）。
                                    // 每个非空白字符上方（over）或下方（under）居中绘一个小标记字符。
                                    if !ch.is_whitespace()
                                        && let Some(mark_ch) = emphasis_mark
                                    {
                                        let mark_fs = fragment.font_size * 0.5;
                                        let mark_advance = measure_char_for_paint(mark_ch, mark_fs, frag_is_ahem);
                                        // 居中于当前字符（char_pos 已前进 advance，故字符中心 = char_pos - advance/2）
                                        let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                        let mark_y = if emphasis_over {
                                            frag_base_y - fragment.font_size
                                        } else {
                                            frag_base_y + fragment.font_size * 0.35
                                        };
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: mark_x,
                                            y: mark_y,
                                            font_size: mark_fs,
                                            color: frag_color,
                                            glyph_id: mark_ch as u32,
                                            font_id: default_font_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                        });
                                    }

                                    // R1022：ruby annotation —— rt[char_idx] 上移到 rb 字符上方
                                    // （类 text-emphasis over，mark 来自 rt 文本而非 style）。
                                    if !ch.is_whitespace()
                                        && let Some(rt_ch) = ruby_marks.as_ref().and_then(|v| v.get(char_idx).copied())
                                    {
                                        let rt_fs = fragment.font_size * 0.5;
                                        let rt_advance = measure_char_for_paint(rt_ch, rt_fs, frag_is_ahem);
                                        let rt_x = char_pos - advance / 2.0 - rt_advance / 2.0;
                                        let rt_y = frag_base_y - fragment.font_size;
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: rt_x,
                                            y: rt_y,
                                            font_size: rt_fs,
                                            color: frag_color,
                                            glyph_id: rt_ch as u32,
                                            font_id: default_font_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                        });
                                    }
                                }

                                let text_width: f32 = transformed
                                    .chars()
                                    .map(|ch| {
                                        let w = measure_char_for_paint(ch, fragment.font_size, frag_is_ahem)
                                            + letter_spacing;
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

                            // R358：per-fragment color（带 abs-pos guard）。
                            // 非多列路径此前所有片段用容器 color（丢失 span 自身 color，
                            // 如 multicol-count-computed-004 彩色 span 被渲成容器黑色）。
                            // 现解析每个片段所属元素的 color，**但 abs-pos/fixed 片段保留容器 color**——
                            // R335 实证 per-fragment color 作用于 abspos 文本会使绿色 X 更显眼地绘在
                            // 错误的 paint-IFC（正常流）位置 → abs-pos-non-replaced-vrl/vlr 4 case 回归。
                            // abspos 文本位置修复需 Phase A（R336 double-path），guard 维持当前行为。
                            let owner_id = if doc
                                .get($frag_nid)
                                .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                            {
                                doc.parent_node($frag_nid).unwrap_or($frag_nid)
                            } else {
                                $frag_nid
                            };
                            let frag_color = styles
                                .and_then(|s| s.get(&owner_id))
                                .filter(|s| {
                                    s.color != ColorValue::CurrentColor
                                        && !matches!(
                                            s.position,
                                            zero_css_parser::values::PositionValue::Absolute
                                                | zero_css_parser::values::PositionValue::Fixed
                                        )
                                })
                                .map(|s| color_value_to_render(&s.color))
                                .unwrap_or(color);

                            // R1021：text-emphasis 取自片段 owner 样式（<span> 上设）。
                            let owner_style_opt = styles.and_then(|s| s.get(&owner_id));
                            let emphasis_mark: Option<char> =
                                owner_style_opt.and_then(|s| match s.text_emphasis_style {
                                    TextEmphasisStyleValue::Char(c) => Some(c),
                                    TextEmphasisStyleValue::None => None,
                                });
                            let emphasis_over = owner_style_opt
                                .map(|s| matches!(
                                    s.text_emphasis_position,
                                    TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                ))
                                .unwrap_or(true);
                            // R1022：ruby annotation —— owner 为 <ruby> 时，rt 后代文本逐字符上移。
                            let ruby_marks: Option<Vec<char>> = ruby_annotation_chars(doc, owner_id);

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

                            // R639：text_width 先算（glyph loop 之前），支持 inline bg 在 glyph 下绘制。
                            let text_width: f32 = transformed
                                .chars()
                                .map(|ch| {
                                    let w = measure_char_for_paint(ch, $frag_fs, $is_ahem) + letter_spacing;
                                    if ch == ' ' { w + word_spacing } else { w }
                                })
                                .sum();

                            // R639 Phase A slice：per-line-fragment inline background，仅对跨多行
                            // 的 inline 生效。关键修复（R638 锁定 blocker）：宏的 box_node 是 **IFC
                            // owner**（文本所在容器）非 inline 本身，故多行门控用 **owner inline 自身
                            // height**（self.inline_heights 按 owner_id 查），而非 box_node.height
                            //（IFC owner 的）——后者在 inline 文本处于父 IFC 时与 paint_node 抑制
                            //（inline 自身 box 上）分歧致 bg 消失。两处现均用 inline 自身 height → 一致。
                            // frag_base_x 已含 text-indent（IFC 首行 current_x=text_indent），首行从缩进后起。
                            let owner_h = self.inline_heights.get(&owner_id).copied().unwrap_or(0.0);
                            if !is_vertical
                                && !box_node.is_absolute
                                && !box_node.is_fixed
                                && owner_h > $frag_fs * 1.5
                                && let Some(owner_style) = styles.and_then(|s| s.get(&owner_id))
                                && matches!(owner_style.display, zero_css_parser::values::DisplayValue::Inline)
                                && owner_style.background_color != ColorValue::Transparent
                            {
                                let line_h = box_node
                                    .text_node_line_heights
                                    .get(&$frag_nid)
                                    .copied()
                                    // 与 layout `NORMAL_LINE_HEIGHT_RATIO`（1.164 = chromium
                                    // DejaVu line-height:normal 真值，R1174）保持一致；仅在
                                    // `text_node_line_heights` 缺该 fragment 时作回退。
                                    .unwrap_or($frag_fs * 1.164);
                                self.primitives.add_fill(
                                    Rect::new(frag_base_x, content_y + $frag_y + ty, text_width, line_h),
                                    color_value_to_render(&owner_style.background_color),
                                );
                            }

                            for (char_idx, ch) in transformed.chars().enumerate() {
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
                                    color: frag_color,
                                    glyph_id: ch as u32,
                                    font_id: default_font_id,
                                    bitmap_width: None,
                                    bitmap_height: None,
                                    rotation,
                                });

                                // R644：Cc 控制字符可见性（CSS Text 3）——fontdue 对 Cc 无字形
                                //（.notdef 空），渲染可见占位框（em 方块），使 control-chars-* mismatch
                                // 测试 test != 空 ref（diff > min_mismatch_ratio 0.5%；fs×fs em 方块
                                // 在 4em=64px 下 ~0.85% diff，超阈值）。
                                if is_cc_control_char(ch) {
                                    self.primitives.add_fill(
                                        Rect::new(glyph_x, glyph_y - $frag_fs, $frag_fs, $frag_fs),
                                        frag_color,
                                    );
                                }

                                let advance = measure_char_for_paint(ch, $frag_fs, $is_ahem)
                                    + letter_spacing
                                    + if ch == ' ' { word_spacing } else { 0.0 };
                                char_pos += advance;

                                // R1021：text-emphasis 标记（水平书写模式；垂直暂不支持）。
                                if !char_advance_is_y
                                    && !ch.is_whitespace()
                                    && let Some(mark_ch) = emphasis_mark
                                {
                                    let mark_fs = $frag_fs * 0.5;
                                    let mark_advance = measure_char_for_paint(mark_ch, mark_fs, $is_ahem);
                                    let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                    // over：mark 基线在文本顶部之上（leading 区）；under：基线之下
                                    let mark_y = if emphasis_over {
                                        frag_base_y - $frag_fs - mark_fs * 0.4
                                    } else {
                                        frag_base_y + $frag_fs * 0.5
                                    };
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: mark_x,
                                        y: mark_y,
                                        font_size: mark_fs,
                                        color: frag_color,
                                        glyph_id: mark_ch as u32,
                                        font_id: default_font_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                    });
                                }

                                // R1022：ruby annotation —— rt[char_idx] 上移到 rb 字符上方
                                // （类 text-emphasis over，mark 来自 rt 文本）。
                                if !char_advance_is_y
                                    && !ch.is_whitespace()
                                    && let Some(rt_ch) =
                                        ruby_marks.as_ref().and_then(|v| v.get(char_idx).copied())
                                {
                                    let rt_fs = $frag_fs * 0.5;
                                    let rt_advance = measure_char_for_paint(rt_ch, rt_fs, $is_ahem);
                                    let rt_x = char_pos - advance / 2.0 - rt_advance / 2.0;
                                    let rt_y = frag_base_y - $frag_fs - rt_fs * 0.4;
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: rt_x,
                                        y: rt_y,
                                        font_size: rt_fs,
                                        color: frag_color,
                                        glyph_id: rt_ch as u32,
                                        font_id: default_font_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                    });
                                }
                            }

                            self.paint_text_decoration_from_style(
                                frag_base_x,
                                frag_base_y,
                                $frag_fs,
                                text_width,
                                frag_color,
                                style,
                            );
                        }};
                    }

                    if use_stored {
                        for frag in &stored_fragments {
                            // R817 linebox 度量统一 Phase 2：is_ahem glyph 位图顶 = 片段基线 - font_size
                            // （Ahem 方块底边齐基线，ascent=font_size）。行基线（container-rel）=
                            // frag.baseline_y_abs。macro glyph_y = content_y + frag.y + v_offset，其中
                            // frag.y = line.y + f.y，故 v_offset = baseline_y_abs - font_size - frag.y
                            // → glyph_y = content_y + line.y + line.baseline_y - font_size（基线处）。
                            // 旧 v_offset=0 把 glyph 放在 f.y（=baseline_y-run.height，line-height>1 时
                            // 为负，glyph 越过行盒顶部错位）。line-height:1 时 f.y=baseline_y-font_size，
                            // v_offset 退化为 0（== 旧行为，A3，font-051 不回归）。
                            // 仅对**真正** Ahem 字形（is_ahem_font，来自 IFC run 实际字体）应用——
                            // 容器为 Ahem 但片段实为其它字体（font-051 的 serif span）时保留旧
                            // 容器级行为（is_ahem?0:font_size），避免按 ascent=font_size 错移。
                            let v_offset = if frag.is_ahem_font {
                                // R841：line-height-aware Ahem 方块位（见 ahem_uses_embox_position）。
                                // half-leading≈0（lh≈fs）→ em-box 位 baseline−0.8·fs；否则 R817 baseline−fs。
                                if ahem_uses_embox_position(frag.height, frag.font_size) {
                                    frag.baseline_y_abs - 0.8 * frag.font_size - frag.y
                                } else {
                                    frag.baseline_y_abs - frag.font_size - frag.y
                                }
                            } else if frag.is_ahem {
                                0.0
                            } else {
                                frag.font_size
                            };
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
                            // R953：非存储路径 glyph 定位修正。glyph 顶（行盒相对）= half-leading =
                            // (line-height − font_size)/2（与 ascent 无关，字形按 em-box 在行盒内居中）。
                            // frag.y = run 顶 = baseline_y − run.height；glyph 顶 = frag.y + offset，需
                            // offset = run.height − ascent = line-height − 0.8·fs（ascent≈0.8·fs 启发式，
                            // 与 apply_vertical_alignment 的 strut_ascent 一致）。旧 offset = font_size
                            // 把 glyph 顶放在 frag.y + fs（基线位），致默认字体文本每行偏低约 9.6px。
                            // A/B（R953）：css-text +60 / css-text-decor +27 / position +3 / tables +3 /
                            // fonts +4 / multicol +4 / writing-modes +1 oracle-pass（≈ +102 case），
                            // 零目录回归；welcome hero title 反而更准（ORA 104-135 / OFF 135-154 / ON 105-124）。
                            // 残余 welcome 净 +0.77pp = 真字体 ascent≠0.8·fs 的字体墙噪声（trend-only，
                            // 理想修须接 fontdue 真 ascent，font-metric 墙多会话）。
                            // 仅文本运行（fs>0）；inline-block/原子盒（fs==0）保留旧 baseline_fs。
                            let baseline_offset = if fragment.font_size > 0.0 {
                                fragment.height - 0.8 * fragment.font_size
                            } else {
                                baseline_fs
                            };
                            render_fragment!(
                                fragment.x,
                                fragment.y,
                                baseline_offset,
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
                        let ellipsis_char_width = measure_char_for_paint('.', font_size, false);
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

                        let ellipsis_width = measure_char_for_paint('.', font_size, false);
                        let default_font_id = self.resolve_font_id(&style.font_family, &style.font_weight);
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
            measure_char_for_paint('A', font_size, false),
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
        let default_font_id = self.resolve_font_id(&style.font_family, &style.font_weight);
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
            char_x += measure_char_for_paint(ch, font_size, is_ahem);
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

pub(super) fn has_direct_paintable_text(
    doc: &Document,
    node_id: NodeId,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
) -> bool {
    let direct = doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    });
    if direct {
        return true;
    }
    // PHASEA stored-line-boxes 路径（默认启用；env PHASEA_STORE_EXT=0 关闭，与 compute_final 存储扩展配套）：仅对
    // **纯 inline 内容**容器（有 inline-level 元素子节点且**无 block-level 元素子节点**）返回
    // true。排除 block 子节点（独立渲染）与混合 inline+block 内容（block-in-inline / span+h4
    // 等存储路径与重跑分歧致回归：inline-box-001/002、multicol-block-no-clip-001）。
    if std::env::var("PHASEA_STORE_EXT").as_deref() != Ok("0")
        && let Some(styles) = styles
    {
        use zero_css_parser::values::DisplayValue;
        let is_inline_display = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let child_ids: Vec<zero_dom::NodeId> = doc.child_nodes(node_id);
        let child_displays: Vec<Option<&DisplayValue>> =
            child_ids.iter().map(|c| styles.get(c).map(|s| &s.display)).collect();
        let has_inline_elem = child_displays.iter().any(|d| d.is_some_and(is_inline_display));
        let has_block_elem = child_displays
            .iter()
            .any(|d| d.is_some_and(|dd| !is_inline_display(dd)));
        // inline-level 子元素须为叶文本容器（无元素子节点），排除 block-in-inline（R109 碎片化）。
        let inline_children_have_elem = child_ids.iter().any(|c| {
            styles.get(c).is_some_and(|s| is_inline_display(&s.display))
                && doc
                    .child_nodes(*c)
                    .iter()
                    .any(|gc| doc.get(*gc).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        });
        has_inline_elem && !has_block_elem && !inline_children_have_elem
    } else {
        false
    }
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

/// 获取 `<img>` 元素的固有尺寸。
///
/// 优先使用解码后的真实尺寸；若图片尚未解码，再回退到 HTML `width`/`height` 属性，
/// 最后使用调用方提供的回退尺寸。
fn get_img_intrinsic_size(
    node: &zero_dom::NodeData,
    decoded_size: Option<(f32, f32)>,
    fallback_w: f32,
    fallback_h: f32,
) -> (f32, f32) {
    if let Some((w, h)) = decoded_size
        && w > 0.0
        && h > 0.0
    {
        return (w, h);
    }

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

#[cfg(test)]
mod r841_tests {
    use super::ahem_uses_embox_position;

    /// R841：line-height ≈ font-size（half-leading≈0）启用 em-box 位（修 ifc-008/line-height-121）。
    #[test]
    fn r841_embox_gate_half_leading_zero() {
        // lh:1（含 1em、Ahem lh:normal=1.0）→ half-leading=0 → em-box 位
        assert!(ahem_uses_embox_position(40.0, 40.0), "lh:1 应启用 em-box 位");
        assert!(
            ahem_uses_embox_position(100.0, 100.0),
            "lh:1em（100px）应启用（ifc-008）"
        );
        // 极小数值误差仍视为 lh≈fs
        assert!(ahem_uses_embox_position(40.0 + 0.1, 40.0), "亚像素偏差应仍启用");
    }

    /// R841：line-height:0（行盒塌缩）与 line-height>1（含 leading）保留 R817 位。
    #[test]
    fn r841_embox_gate_leading_present() {
        // lh:0（line-height:0px 测试簇）→ half-leading=-fs/2 → 不启用（避免 27 用例越过 1%）
        assert!(!ahem_uses_embox_position(0.0, 20.0), "lh:0 不应启用");
        // lh>1（va-117a 等）→ 含正 half-leading → 不启用（R839 妥协位）
        assert!(!ahem_uses_embox_position(130.0, 40.0), "lh>1 不应启用");
        assert!(!ahem_uses_embox_position(80.0, 40.0), "lh:2 不应启用");
        // lh:0.5（<fs）也不启用
        assert!(!ahem_uses_embox_position(10.0, 20.0), "lh:0.5 不应启用");
    }
}
