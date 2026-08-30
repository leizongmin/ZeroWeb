//! R3836：容器级 RTL bidi-override 的行级 run 序反转单测。
//!
//! `break_items_into_lines` 逐 run 反转字符（`BidiFragmentCursor::with_override`）但
//! run 以逻辑序进入行盒；多 run 行（相邻 inline 元素边界，bidi-box-model-013：
//! `<span>dnoceS</span> tsriF`）需要整行反转显示序（UAX #9 L2 按行重排）。
//! chromium 真值：反转后视觉序 = 逻辑序反向，行内相邻间隙随 run 一起镜像。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue;
use zero_dom::{NodeKind, parse_html};

/// 单 run 行不受行级反转影响（<2 runs 跳过）——字符级反转已在 break 阶段完成。
#[test]
fn rtl_override_single_run_line_is_untouched() {
    let mut ctx = InlineFormattingContext::new(800.0).with_bidi_override_direction(Some(true));
    let node = zero_dom::Document::new().create_text_node("");
    let mut run = TextRun::simple("abc".to_string(), node, 16.0, 20.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    ctx.break_into_lines(vec![run]);
    // break 阶段已按 override 反转字符；行级反转跳过，几何不被二次改动。
    assert_eq!(ctx.lines[0].runs[0].text, "cba");
    let x_before = ctx.lines[0].runs[0].x;
    ctx.reverse_lines_for_rtl_override();
    assert_eq!(ctx.lines[0].runs[0].text, "cba");
    assert_eq!(ctx.lines[0].runs[0].x, x_before);
}

/// 多 run 行：整行反转显示序 + 相邻间隙随 run 镜像（013 场景）。
#[test]
fn rtl_override_multi_run_line_reverses_display_order_with_gaps() {
    let mut doc = zero_dom::Document::new();
    let n1 = doc.create_text_node("");
    let n2 = doc.create_text_node("");
    let n3 = doc.create_text_node("");
    // Ahem：advance = font_size，宽度确定。三 run 等宽 20px，中 run 前有 10px gap
    //（模拟折叠空格/margin）。
    let mut a = TextRun::simple("aaa".to_string(), n1, 20.0, 20.0, VerticalAlignValue::Baseline);
    a.is_ahem_font = true;
    let mut b = TextRun::simple("bbb".to_string(), n2, 20.0, 20.0, VerticalAlignValue::Baseline);
    b.is_ahem_font = true;
    b.margin_left = 10.0;
    let mut c = TextRun::simple("ccc".to_string(), n3, 20.0, 20.0, VerticalAlignValue::Baseline);
    c.is_ahem_font = true;
    let mut ctx = InlineFormattingContext::new(800.0).with_bidi_override_direction(Some(true));
    ctx.break_into_lines(vec![a, b, c]);

    assert_eq!(ctx.lines.len(), 1);
    let line = &ctx.lines[0];
    assert_eq!(line.runs.len(), 3);
    // 反转前：逻辑序 a(0-20) b(30-50) c(50-70)，b 的 margin 保留为其前导 gap。
    assert_eq!(line.runs[0].text, "aaa");
    assert_eq!(line.runs[1].text, "bbb");
    assert_eq!(line.runs[2].text, "ccc");

    ctx.reverse_lines_for_rtl_override();
    let line = &ctx.lines[0];
    // 反转后：显示序 c b a；gap 随 run 镜像（b 的 margin 现在在 b 与 a 之间）。
    // Ahem advance = font_size = 20px/字符，3 字符 run 宽 60。
    assert_eq!(line.runs[0].text, "ccc");
    assert_eq!(line.runs[1].text, "bbb");
    assert_eq!(line.runs[2].text, "aaa");
    assert_eq!(line.runs[0].x, 0.0);
    assert_eq!(line.runs[1].x, 70.0);
    assert_eq!(line.runs[2].x, 130.0);
    // 总占宽不变（镜像不改变行内容宽度）。
    let last = line.runs.last().unwrap();
    assert_eq!(last.x + last.width, 190.0);
}

/// LTR override（`direction: ltr` + `unicode-bidi: bidi-override`）不走行级反转。
#[test]
fn ltr_override_does_not_reverse_lines() {
    let mut doc = zero_dom::Document::new();
    let n1 = doc.create_text_node("");
    let n2 = doc.create_text_node("");
    let mut a = TextRun::simple("aaa".to_string(), n1, 20.0, 20.0, VerticalAlignValue::Baseline);
    a.is_ahem_font = true;
    let mut b = TextRun::simple("bbb".to_string(), n2, 20.0, 20.0, VerticalAlignValue::Baseline);
    b.is_ahem_font = true;
    // LTR override 也逐 run 反转字符（override 语义），但行级 run 序保持逻辑序：
    // `layout()` 的反转门控是 `== Some(true)`（仅 RTL）。
    let mut ctx = InlineFormattingContext::new(800.0).with_bidi_override_direction(Some(false));
    ctx.break_into_lines(vec![a, b]);
    let line = &ctx.lines[0];
    assert_eq!(line.runs[0].text, "aaa");
    assert_eq!(line.runs[0].x, 0.0);
    assert_eq!(line.runs[1].text, "bbb");
    assert_eq!(line.runs[1].x, 60.0);
}

/// 端到端：013 DOM 形状（div.rtol 容器 + span + 文本），反转后 span 在右、首词在左。
#[test]
fn bidi_box_model_013_dom_reverses_visual_order() {
    let doc = parse_html("<div style=\"direction: rtl; unicode-bidi: bidi-override\"><span>dnoceS</span> tsriF</div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc
        .child_nodes(body)
        .iter()
        .copied()
        .find(|id| doc.get(*id).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
        .expect("div");
    let span = doc
        .child_nodes(div)
        .iter()
        .copied()
        .find(|id| doc.get(*id).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
        .expect("span");

    let mut div_style = zero_style_system::ComputedStyle::default();
    div_style.unicode_bidi = zero_style_system::UnicodeBidiValue::BidiOverride;
    div_style.direction = zero_style_system::DirectionValue::Rtl;
    let styles = std::collections::HashMap::from([(div, div_style)]);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, div, &styles);
    assert_eq!(ctx.bidi_override_direction, Some(true));

    let span_x = ctx
        .all_fragments()
        .iter()
        .filter(|f| f.node_id == span && !f.text.is_empty())
        .map(|f| f.x)
        .fold(f32::INFINITY, f32::min);
    let first_x = ctx
        .all_fragments()
        .iter()
        .filter(|f| f.node_id != span && !f.text.is_empty())
        .map(|f| f.x)
        .fold(f32::INFINITY, f32::min);
    // chromium 真值（013）：tsriF 视觉在左、dnoceS（span）在右。
    assert!(
        first_x < span_x,
        "expected text (tsriF) left of span (dnoceS): first_x={first_x} span_x={span_x}"
    );
}

/// R3837 BUG D：inline 水平 padding 参与 inline 轴推进（CSS2.1 §8.4）——
/// bidi-box-model-033：span padding-left:40 → 后续片段 x 右移 40。
#[test]
fn inline_padding_left_advances_inline_axis() {
    let mut doc = zero_dom::Document::new();
    let n = doc.create_text_node("");
    let mut run = TextRun::simple("ab".to_string(), n, 20.0, 20.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run.padding_left = 40.0;
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(vec![run]);
    let frag = &ctx.lines[0].runs[0];
    // 片段 x = 内容起点（padding 在片段之前消费），占宽推进给后续内容。
    assert_eq!(frag.x, 40.0);
    assert_eq!(ctx.lines[0].runs[0].width, 40.0);
}

/// R3837 BUG A：行尾 inline 盒的尾随 margin/padding 计入对齐宽度
///（bidi-box-model-019：text-align:right + span margin-right:40 → 行整体左移 40）。
#[test]
fn trailing_margin_counts_into_right_align_width() {
    let mut doc = zero_dom::Document::new();
    let n = doc.create_text_node("");
    let mut run = TextRun::simple("ab".to_string(), n, 20.0, 20.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run.margin_right = 40.0;
    let mut ctx = InlineFormattingContext::new(400.0).with_text_align(TextAlign::Right);
    ctx.break_into_lines(vec![run]);
    let frag = &ctx.lines[0].runs[0];
    // 内容宽 40 + margin 40 = 80 → right-align 起点 x = 400 − 80 = 320。
    assert_eq!(frag.x, 320.0);
}
