// 文本对齐测试 — 从 basic.rs 拆分（R1711，CLAUDE.md §5 文件减负）。
use super::super::*;

// ── 文本对齐测试 ──

/// 测试默认对齐为 Left。
#[test]
fn test_default_text_align_is_left() {
    let ctx = InlineFormattingContext::new(800.0);
    assert_eq!(ctx.text_align, TextAlign::Left);
}

/// 测试 with_text_align builder 方法。
#[test]
fn test_with_text_align_builder() {
    let ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
    assert_eq!(ctx.text_align, TextAlign::Center);
}

/// 测试 center 对齐 — 片段整体居中。
#[test]
fn test_text_align_center() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let line = &ctx.lines[0];
    // center 对齐不变式：first.x + last.x + last.width = container_width
    // 即首片段到末片段右边界对称居中
    let last = line.runs.last().unwrap();
    let centered_end = line.runs[0].x + last.x + last.width;
    assert!(
        (centered_end - 800.0).abs() < 0.5,
        "center: 内容应居中，首尾边界和 {} 应接近 800",
        centered_end
    );
}

/// 测试 right 对齐 — 片段整体靠右。
#[test]
fn test_text_align_right() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Right);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let line = &ctx.lines[0];
    // right 对齐不变式：最后一个片段的右边界 = container_width
    let last = line.runs.last().unwrap();
    assert!(
        (last.x + last.width - 800.0).abs() < 0.5,
        "right: 最后片段右边界应为 800，实际 {}",
        last.x + last.width
    );
}

/// 测试 left 对齐（默认）— 片段从 x=0 开始。
#[test]
fn test_text_align_left_no_offset() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Left);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    // 第一个片段 x 应为 0
    assert!(
        ctx.lines[0].runs[0].x.abs() < 0.01,
        "left: 第一个片段 x 应为 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试 justify 对齐 — 非最后一行时片段间均匀分配空间。
#[test]
fn test_text_align_justify_distributes_space() {
    // 使用窄容器（60px）确保产生多行
    let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Justify);
    let runs = vec![TextRun {
        text: "aa bb cc dd ee ff gg hh".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "应产生多行用于 justify 测试");

    // 非最后一行：最后一个片段的右边界应接近容器宽度
    for (i, line) in ctx.lines.iter().enumerate() {
        if i < ctx.lines.len() - 1 && line.runs.len() >= 2 {
            let last_run = line.runs.last().unwrap();
            let right_edge = last_run.x + last_run.width;
            assert!(
                (right_edge - 60.0).abs() < 1.0,
                "justify 第 {} 行右边界应接近 60，实际 {}",
                i,
                right_edge
            );
        }
    }
}

/// 测试 justify 最后一行不拉伸（保持左对齐）。
#[test]
fn test_text_align_justify_last_line_not_stretched() {
    let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Justify);
    let runs = vec![TextRun {
        text: "aa bb cc dd ee ff gg".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "应产生多行");
    let last_line = ctx.lines.last().unwrap();
    // 最后一行的第一个片段 x 应为 0（不 justify）
    assert!(
        last_line.runs[0].x.abs() < 0.01,
        "justify 最后一行不应拉伸，x 应为 0，实际 {}",
        last_line.runs[0].x
    );
}

/// 测试 center 对齐在多行中每行都居中。
///
/// 验证每行的第一个片段 x 坐标等于 (container_width - 总宽度) / 2。
/// 总宽度通过所有片段宽度之和计算（不含对齐偏移）。
#[test]
fn test_text_align_center_multiline() {
    let mut ctx = InlineFormattingContext::new(60.0).with_text_align(TextAlign::Center);
    let runs = vec![TextRun {
        text: "aa bb cc dd ee ff".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "应产生多行");
    for (i, line) in ctx.lines.iter().enumerate() {
        if line.runs.is_empty() {
            continue;
        }
        // center 对齐不变式：first.x + last.x + last.width = container_width
        let last = line.runs.last().unwrap();
        let centered_end = line.runs[0].x + last.x + last.width;
        assert!(
            (centered_end - 60.0).abs() < 1.0,
            "center 第 {} 行: 首尾边界和 {} 应接近 60",
            i,
            centered_end
        );
    }
}

/// 测试 right 对齐在多行中每行都靠右。
#[test]
fn test_text_align_right_multiline() {
    let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Right);
    let runs = vec![TextRun {
        text: "aa bb cc dd ee".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "应产生多行");
    for (i, line) in ctx.lines.iter().enumerate() {
        let last = line.runs.last().unwrap();
        let right_edge = last.x + last.width;
        assert!(
            (right_edge - 100.0).abs() < 1.0,
            "right 第 {} 行: 右边界应约 100，实际 {}",
            i,
            right_edge
        );
    }
}

/// 测试 justify 在只有 1 个片段的行不崩溃。
#[test]
fn test_text_align_justify_single_fragment_line() {
    let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
    // 超长单个单词，只会产生 1 个片段的行
    let runs = vec![TextRun {
        text: "aaaaaaaaaaaaaaaaaaaaaa".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];
    ctx.break_into_lines(runs);
    // 不应 panic
    assert_eq!(ctx.lines.len(), 1);
    assert!(ctx.lines[0].runs[0].x.abs() < 0.01, "单片段行 justify 不应调整 x");
}

/// 测试对齐不影响 total_height。
#[test]
fn test_text_align_does_not_affect_total_height() {
    let runs = vec![TextRun {
        text: "aa bb cc dd ee ff gg".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];

    let mut ctx_left = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Left);
    ctx_left.break_into_lines(runs.clone());

    let mut ctx_center = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Center);
    ctx_center.break_into_lines(runs.clone());

    let mut ctx_right = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Right);
    ctx_right.break_into_lines(runs.clone());

    let mut ctx_justify = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
    ctx_justify.break_into_lines(runs);

    let h = ctx_left.total_height();
    assert!((ctx_center.total_height() - h).abs() < 0.01, "center 高度应相同");
    assert!((ctx_right.total_height() - h).abs() < 0.01, "right 高度应相同");
    assert!((ctx_justify.total_height() - h).abs() < 0.01, "justify 高度应相同");
}

/// 测试对齐不影响行数。
#[test]
fn test_text_align_does_not_change_line_count() {
    let runs = vec![TextRun {
        text: "aa bb cc dd ee ff".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
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
        is_rtl: false,
    }];

    let mut ctx_left = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Left);
    ctx_left.break_into_lines(runs.clone());

    let mut ctx_justify = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
    ctx_justify.break_into_lines(runs);

    assert_eq!(ctx_left.lines.len(), ctx_justify.lines.len(), "对齐方式不应改变行数");
}

/// 测试空行盒在对齐时不会崩溃。
#[test]
fn test_text_align_empty_lines_no_panic() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_align(TextAlign::Center);
    let runs: Vec<TextRun> = vec![];
    ctx.break_into_lines(runs);
    assert!(ctx.lines.is_empty());
}
