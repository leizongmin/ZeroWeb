//! 多列（multicol）布局 paint helper — 列参数计算 + balance 列高 + column-rule 绘制。
//!
//! R1694 从 painter/text.rs 抽离（text.rs 减负，单文件超 2000 行 guideline）。
//! 独立纯函数 + paint_column_rules Painter 方法 + 专属单测。

use zero_css_parser::values::LengthValue;
use zero_layout_engine::LayoutBox;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{LineCap, StrokePrimitive};
use zero_style_system::{
    ColumnCountComputedValue, ColumnRuleStyleComputedValue, ColumnRuleWidthComputedValue, ColumnWidthComputedValue,
    ComputedStyle,
};

use crate::paint::color::color_value_to_render;

/// 多列布局的列信息（用于 inline 内容的列分布）。
pub(super) struct MulticolInfo {
    /// 列数
    pub(super) col_count: usize,
    /// 单列宽度
    pub(super) col_width: f32,
    /// 列间距
    pub(super) gap: f32,
}

/// 从 ComputedStyle 计算多列参数。
///
/// 返回 `None` 表示不需要多列布局（column-count: auto 且 column-width: auto）。
/// `font_size_px` 用于将 em/rem 单位的 gap 和 column-width 转换为像素。
pub(super) fn compute_multicol_info_for_paint(
    style: &ComputedStyle,
    container_width: f32,
    font_size_px: f32,
) -> Option<MulticolInfo> {
    let gap: f32 = match &style.column_gap {
        // R2192：column-gap:normal（Auto sentinel）→ 1em（CSS Multicol §4.1 default），对齐 layout
        //（multicol.rs:161 R1040）。旧 `_ => 0.0` 致 paint 重算列几何 gap=0 与 layout 的 1em 分歧。
        LengthValue::Auto => font_size_px,
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

/// R1424：multicol balance 列高 target_h（ceil(行数/列数) × 平均行高）。
///
/// 使各列填到 `ceil(N/C)` 行（front-loaded，末列收尾），匹配 chromium LayoutNG balancing
/// （如 44 行/6 列 → ceil=8 行/列 → 8,8,8,8,8,4）。旧 `total_height/col_count`（=7.33 行）
/// 给 8,7,8,7,7,7（更均匀但非 chromium）。仅当行数不能整除列数时二者不同（整除时 ceil=N/C
/// 等价）。非均匀行高用平均行高近似。
pub(super) fn multicol_balance_target_height(num_lines: usize, col_count: usize, total_height: f32) -> f32 {
    if num_lines > 0 && col_count > 0 {
        let lines_per_col = num_lines.div_ceil(col_count) as f32;
        lines_per_col * (total_height / num_lines as f32)
    } else if col_count > 0 {
        total_height / col_count as f32
    } else {
        total_height
    }
}

/// R1862：判定 inline-content multicol 的哪些列实际有内容。
///
/// `store_inline_multicol_columns`（inline_finalization.rs）把每行 fragment.x 加上
/// `col_idx×(col_w+gap)` 列偏移后存入 `inline_layout`，故某列是否有内容 = 是否存在
/// fragment.x 落在该列范围 `[col_idx×stride, col_idx×stride+col_w]`（stride = col_w+gap）。
///
/// 供 `paint_column_rules` 实现 CSS Multicol §5.2「列分隔线仅在相邻两列都有内容时绘制」：
/// `column-fill:auto` 容器内容只填满部分列时（如 columnfill-auto-max-height-001 内容仅
/// 在 col1），空列侧的 rule 不应绘制（旧实现因无 block 子而跳过检查 → 误画 red rule）。
pub(super) fn inline_multicol_used_columns(
    inline_layout: &[zero_layout_engine::types::InlineLayoutLine],
    col_w: f32,
    gap: f32,
    count: usize,
) -> Vec<bool> {
    let stride = col_w + gap;
    let mut used = vec![false; count];
    if stride <= 0.0 || col_w <= 0.0 || count == 0 {
        return used;
    }
    for line in inline_layout {
        for frag in &line.fragments {
            let col_idx = (frag.x / stride).floor();
            if col_idx < 0.0 {
                continue;
            }
            let col_idx = col_idx as usize;
            if col_idx >= count {
                continue;
            }
            let start = col_idx as f32 * stride;
            if frag.x >= start - 0.5 && frag.x < start + col_w + 0.5 {
                used[col_idx] = true;
            }
        }
    }
    used
}

impl super::super::Painter {
    /// 绘制多列布局的 column-rule（列之间的分隔线）。
    pub(crate) fn paint_column_rules(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
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
        // R1429：column-fill:auto + 明确高度 + inline 内容溢出时，layout 侧（store_inline_multicol_columns）
        // 创建了溢出列（实际列数 > style column-count，存于 multicol_overflow_column_count）。column-rule
        // 须在每个间隙（含溢出间隙）绘制——CSS Multicol §8.2：溢出列在容器内容边外水平延伸。
        // col_w 仍按 style count 计算（溢出列保持原列宽），仅循环范围扩到 actual_count。
        let actual_count = box_node.multicol_overflow_column_count.unwrap_or(count);

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

        for i in 1..actual_count {
            // CSS Multi-column §5.2：列分隔线仅在两列都有内容时绘制。
            // 检查第 i-1 列（left）和第 i 列（right）是否有内容：
            // - block 子：按子元素 x（position_multicol_children 已设为列内坐标）判定列归属；
            // - inline 内容（column-fill:auto + 纯文本，无 block 子）：按 store_inline_multicol_columns
            //   存的 inline_layout fragment.x 判定（每个 fragment.x 已加 col_idx×(col_w+gap) 列偏移）。
            //   修 columnfill-auto-max-height-001：内容仅填满 col1 时不应在 col1/col2 间隙画 rule。
            // - 无子元素信息（单元测试场景）：默认绘制。
            let col_left_start = (i - 1) as f32 * (col_w + gap);
            let col_right_start = i as f32 * (col_w + gap);
            let in_col = |x: f32, start: f32| x >= start - 0.5 && x < start + col_w + 0.5;
            let (has_left_content, has_right_content) = if !box_node.children.is_empty() {
                let l = box_node
                    .children
                    .iter()
                    .any(|c| !c.is_absolute && !c.is_fixed && in_col(c.x, col_left_start));
                let r = box_node
                    .children
                    .iter()
                    .any(|c| !c.is_absolute && !c.is_fixed && in_col(c.x, col_right_start));
                (l, r)
            } else if let Some(inline_layout) = &box_node.inline_layout {
                let used = inline_multicol_used_columns(inline_layout, col_w, gap, actual_count as usize);
                let l = used.get((i - 1) as usize).copied().unwrap_or(false);
                let r = used.get(i as usize).copied().unwrap_or(false);
                (l, r)
            } else {
                (true, true)
            };
            if !has_left_content || !has_right_content {
                continue; // 跳过空列的分隔线
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
}

#[cfg(test)]
mod r1424_tests {
    use super::{inline_multicol_used_columns, multicol_balance_target_height};
    use zero_layout_engine::types::{InlineLayoutFragment, InlineLayoutLine};

    /// R1424：ceil(行数/列数) × 平均行高 → front-loaded 分布（匹配 chromium）。
    /// 驱动案 multicol-columns-001（44 行/6 列，Ahem 20px/行，total 880）：旧 total/col_count
    /// =146.67（7.33 行）→ 8,7,8,7,7,7；新 ceil(44/6)*20=160（8 行）→ 8,8,8,8,8,4（chromium）。
    #[test]
    fn r1424_target_height_ceil_front_loads() {
        // 44 行/6 列/total 880 → ceil(44/6)=8 行 × 20px = 160（非 880/6=146.67）。
        assert!(
            (multicol_balance_target_height(44, 6, 880.0) - 160.0).abs() < 0.01,
            "44 行/6 列应 front-load 到 8 行/列（target_h=160），匹配 chromium"
        );
        // 44 行/4 列 → ceil(44/4)=11 × 20 = 220。
        assert!(
            (multicol_balance_target_height(44, 4, 880.0) - 220.0).abs() < 0.01,
            "44 行/4 列应 11 行/列（target_h=220）"
        );
    }

    /// 整除时 ceil(N/C)=N/C，target_h 与旧 total/col_count 等价（无变化，零回归基线）。
    #[test]
    fn r1424_target_height_exact_division_unchanged() {
        // 48 行/6 列/total 960 → ceil(48/6)=8 × 20 = 160 == 960/6=160。
        assert!(
            (multicol_balance_target_height(48, 6, 960.0) - 160.0).abs() < 0.01,
            "整除时 target_h 应与 total/col_count 等价（160）"
        );
        // 100 行/4 列/total 400 → ceil(100/4)=25 × 4 = 100 == 400/4=100。
        assert!(
            (multicol_balance_target_height(100, 4, 400.0) - 100.0).abs() < 0.01,
            "整除时无 ceil 差异"
        );
    }

    /// 边界：0 行回退 total/col_count；col_count=0 回退 total（防 div-by-zero）。
    #[test]
    fn r1424_target_height_edge_cases() {
        assert!(
            (multicol_balance_target_height(0, 6, 880.0) - (880.0 / 6.0)).abs() < 0.01,
            "0 行回退 total/col_count"
        );
        assert!(
            (multicol_balance_target_height(44, 0, 880.0) - 880.0).abs() < 0.01,
            "col_count=0 回退 total（防 panic）"
        );
    }

    /// 构造一个仅含给定列偏移处文本片段的 inline_layout（模拟 store_inline_multicol_columns
    /// 的存储：fragment.x = in-column x + col_idx×stride）。
    fn layout_with_fragments(col_w: f32, gap: f32, col_indices: &[usize]) -> Vec<InlineLayoutLine> {
        let stride = col_w + gap;
        col_indices
            .iter()
            .map(|&c| InlineLayoutLine {
                y: 0.0,
                height: 25.0,
                baseline_y: 20.0,
                ascent: 20.0,
                descent: 5.0,
                fragments: vec![InlineLayoutFragment {
                    x: c as f32 * stride, // 列起始处（in-column x=0）
                    y: 0.0,
                    width: col_w,
                    height: 25.0,
                    font_size: 25.0,
                    is_ahem: true,
                    is_ahem_font: true,
                    text: "Abcd".into(),
                    node_id: None,
                    baseline_y: 20.0,
                }],
            })
            .collect()
    }

    /// R1862：columnfill-auto-max-height-001 —— 内容仅填满 col1（declared col_count=2），
    /// col2 为空。used = [true, false]，故 col1/col2 间隙不应画 rule。
    #[test]
    fn r1862_inline_used_columns_single_column_filled() {
        // col_w=100, gap=100（4em@25px），declared count=2，4 行全在 col1。
        let layout = layout_with_fragments(100.0, 100.0, &[0, 0, 0, 0]);
        let used = inline_multicol_used_columns(&layout, 100.0, 100.0, 2);
        assert_eq!(used, vec![true, false], "仅 col1 有内容，col2 应为空");
    }

    /// R1862：内容跨所有声明列 → 全部 true（rule 正常绘制，零回归基线）。
    #[test]
    fn r1862_inline_used_columns_all_columns_filled() {
        let layout = layout_with_fragments(100.0, 100.0, &[0, 1]);
        let used = inline_multicol_used_columns(&layout, 100.0, 100.0, 2);
        assert_eq!(used, vec![true, true], "两列均有内容");
    }

    /// R1862：空 inline_layout → 全 false（无任何 rule）。
    #[test]
    fn r1862_inline_used_columns_empty_layout() {
        let used = inline_multicol_used_columns(&[], 100.0, 10.0, 3);
        assert_eq!(used, vec![false, false, false], "空 layout 无列有内容");
    }

    /// R1862：退化几何（col_w=0 / stride=0）不 panic，返回全 false。
    #[test]
    fn r1862_inline_used_columns_degenerate_geometry() {
        let layout = layout_with_fragments(100.0, 10.0, &[0]);
        assert_eq!(
            inline_multicol_used_columns(&layout, 0.0, 10.0, 2),
            vec![false, false],
            "col_w=0 不 panic"
        );
        assert_eq!(
            inline_multicol_used_columns(&layout, 0.0, 0.0, 2),
            vec![false, false],
            "stride=0 不 panic"
        );
    }
}
