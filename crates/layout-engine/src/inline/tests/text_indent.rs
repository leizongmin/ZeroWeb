// text-indent 行内布局测试 — 从 advanced.rs 拆分（R1712，CLAUDE.md §5 文件减负）。
use super::super::*;
use zero_css_parser::values::VerticalAlignValue as VA;

// ── text-indent 行内布局测试 ──

/// 测试 text-indent: 40px 时，首行第一个片段 x 偏移 40px。
#[test]
fn test_text_indent_first_line_offset() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_indent(40.0);
    let runs = vec![TextRun {
        text: "Hello World Foo Bar".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty(), "应产生行盒");

    // 首行第一个片段 x 应为 40.0（缩进值）
    assert!(
        (ctx.lines[0].runs[0].x - 40.0).abs() < 0.01,
        "首行首片段 x 应为 40.0（text-indent），实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试 text-indent 仅影响首行，后续行 x 从 0 开始。
#[test]
fn test_text_indent_only_first_line() {
    // 窄容器强制换行
    let mut ctx = InlineFormattingContext::new(80.0).with_text_indent(40.0);
    let runs = vec![TextRun {
        text: "alpha beta gamma delta epsilon zeta eta theta".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应产生至少 2 行");

    // 首行 x 应有缩进
    assert!(
        ctx.lines[0].runs[0].x > 0.0,
        "首行 x 应 > 0（有缩进），实际 {}",
        ctx.lines[0].runs[0].x
    );

    // 第二行 x 应为 0（无缩进）
    assert!(
        ctx.lines[1].runs[0].x.abs() < 0.01,
        "第二行 x 应为 0（无缩进），实际 {}",
        ctx.lines[1].runs[0].x
    );
}

/// 测试 text-indent: 0 时无偏移。
#[test]
fn test_text_indent_zero_no_offset() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_indent(0.0);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    assert!(
        ctx.lines[0].runs[0].x.abs() < 0.01,
        "text-indent=0 时首片段 x 应为 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试负的 text-indent 使首行向左偏移（悬挂缩进效果）。
#[test]
fn test_text_indent_negative() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_indent(-20.0);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 负缩进：首片段 x 应为 -20.0
    assert!(
        (ctx.lines[0].runs[0].x - (-20.0)).abs() < 0.01,
        "负 text-indent 时首片段 x 应为 -20.0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试 text-indent + text-align: center 组合。
///
/// 首行缩进后，text-align 应基于缩进后的内容宽度计算居中偏移。
#[test]
fn test_text_indent_with_text_align_center() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_text_indent(40.0)
        .with_text_align(TextAlign::Center);
    let runs = vec![TextRun {
        text: "Hello".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    // 首片段 x 应包含缩进 + 居中偏移，值应 > 40.0（因为居中额外偏移）
    let x = ctx.lines[0].runs[0].x;
    assert!(x > 40.0, "text-indent + center 时 x 应 > 40（缩进+居中），实际 {}", x);
}
