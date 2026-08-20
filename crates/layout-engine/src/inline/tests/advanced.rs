// Auto-generated test file — split from layout-engine/inline.rs
use super::super::*;
use zero_css_parser::values::VerticalAlignValue as VA;

/// 测试 vertical-align: middle 在行盒中的 y 偏移。
#[test]
fn test_vertical_align_middle_in_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "Hello".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
        vertical_align: VerticalAlignValue::Middle,
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
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let fragment = &ctx.lines[0].runs[0];
    // middle 对齐: y = (line_height - fragment_height) / 2
    let expected_y = (30.0 - fragment.height) / 2.0;
    assert!(
        (fragment.y - expected_y).abs() < 0.01,
        "vertical-align: middle 片段 y 应为 {}，实际 {}",
        expected_y,
        fragment.y
    );
}

// ── 新增边界测试 ──

/// 测试 vertical-align: sub — 片段基线下移 font_size × 0.3。
///
/// 公式: y = baseline_y - height + (font_size * 0.3)
/// baseline_y = line_height * 0.8
#[test]
fn test_vertical_align_sub_in_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let font_size = 16.0_f32;
    let line_height = 30.0_f32;
    let runs = vec![TextRun {
        text: "sub".to_string(),
        node_id: NodeId::default(),
        font_size,
        line_height,
        vertical_align: VA::Sub,
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
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let fragment = &ctx.lines[0].runs[0];
    // R800/R990：行盒基线 = half-leading + ascent（CSS §10.8.1）= (line_height - em)/2 + ratio*em，
    // em = font_size（行内单文本运行，dominant_fs = font_size）；ratio 按 is_ahem 区分
    // （Ahem 0.8 / 非-Ahem 0.928，R990）。此处 is_ahem_font=false → 0.928。
    let baseline_y = (line_height - font_size) / 2.0 + font_size * 0.928;
    let offset = font_size * 0.3;
    let expected_y = baseline_y - fragment.height + offset;
    assert!(
        (fragment.y - expected_y).abs() < 0.01,
        "vertical-align: sub 片段 y 应为 {}，实际 {}",
        expected_y,
        fragment.y
    );
}

/// 测试 vertical-align: super — 片段基线上移 font_size × 0.3。
///
/// 公式: y = baseline_y - height - (font_size * 0.3)
#[test]
fn test_vertical_align_super_in_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let font_size = 16.0_f32;
    let line_height = 30.0_f32;
    let runs = vec![TextRun {
        text: "super".to_string(),
        node_id: NodeId::default(),
        font_size,
        line_height,
        vertical_align: VA::Super,
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
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let fragment = &ctx.lines[0].runs[0];
    // R800/R990：行盒基线 = half-leading + ascent（CSS §10.8.1）= (line_height - em)/2 + ratio*em，
    // em = font_size（行内单文本运行，dominant_fs = font_size）；ratio 按 is_ahem 区分
    // （Ahem 0.8 / 非-Ahem 0.928，R990）。此处 is_ahem_font=false → 0.928。
    let baseline_y = (line_height - font_size) / 2.0 + font_size * 0.928;
    let offset = font_size * 0.3;
    let expected_y = baseline_y - fragment.height - offset;
    assert!(
        (fragment.y - expected_y).abs() < 0.01,
        "vertical-align: super 片段 y 应为 {}，实际 {}",
        expected_y,
        fragment.y
    );
}

/// 测试 vertical-align: text-top 与 top 行为一致 — y = 0.0。
#[test]
fn test_vertical_align_text_top_same_as_top() {
    let mut ctx_text_top = InlineFormattingContext::new(800.0);
    let runs_text_top = vec![TextRun {
        text: "Text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
        vertical_align: VA::TextTop,
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
        is_plaintext_bidi: false,
    }];
    ctx_text_top.break_into_lines(runs_text_top);

    let mut ctx_top = InlineFormattingContext::new(800.0);
    let runs_top = vec![TextRun {
        text: "Text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
        vertical_align: VA::Top,
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
        is_plaintext_bidi: false,
    }];
    ctx_top.break_into_lines(runs_top);

    let y_text_top = ctx_text_top.lines[0].runs[0].y;
    let y_top = ctx_top.lines[0].runs[0].y;
    assert!(
        (y_text_top - y_top).abs() < 0.01,
        "text-top y ({}) 应与 top y ({}) 一致",
        y_text_top,
        y_top
    );
    assert!(y_text_top.abs() < 0.01, "text-top 片段 y 应为 0.0，实际 {}", y_text_top);
}

/// 测试 vertical-align: text-bottom 与 bottom 行为一致 — y = line_height - height。
#[test]
fn test_vertical_align_text_bottom_same_as_bottom() {
    let line_height = 30.0_f32;
    let mut ctx_text_bottom = InlineFormattingContext::new(800.0);
    let runs_text_bottom = vec![TextRun {
        text: "Text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height,
        vertical_align: VA::TextBottom,
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
        is_plaintext_bidi: false,
    }];
    ctx_text_bottom.break_into_lines(runs_text_bottom);

    let mut ctx_bottom = InlineFormattingContext::new(800.0);
    let runs_bottom = vec![TextRun {
        text: "Text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height,
        vertical_align: VA::Bottom,
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
        is_plaintext_bidi: false,
    }];
    ctx_bottom.break_into_lines(runs_bottom);

    let y_text_bottom = ctx_text_bottom.lines[0].runs[0].y;
    let y_bottom = ctx_bottom.lines[0].runs[0].y;
    assert!(
        (y_text_bottom - y_bottom).abs() < 0.01,
        "text-bottom y ({}) 应与 bottom y ({}) 一致",
        y_text_bottom,
        y_bottom
    );
    // text-bottom: y = line_height - fragment_height
    let height = ctx_text_bottom.lines[0].runs[0].height;
    let expected = line_height - height;
    assert!(
        (y_text_bottom - expected).abs() < 0.01,
        "text-bottom y 应为 {}，实际 {}",
        expected,
        y_text_bottom
    );
}

/// 测试 resolve_font_metrics 中 LineHeightValue::Length(LengthValue::Em(1.5)) 按 font-size 折算。
#[test]
fn test_resolve_font_metrics_line_height_em_length() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.line_height = LineHeightValue::Length(LengthValue::Em(1.5));

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 20.0).abs() < 0.01, "font_size 应为 20.0，实际 {font_size}");
    assert!(
        (line_height - 30.0).abs() < 0.01,
        "line-height:1.5em 应解析为 30px，实际 {line_height}"
    );
}

#[test]
fn test_resolve_font_metrics_line_height_percentage_length() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.line_height = LineHeightValue::Length(LengthValue::Percentage(150.0));

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 20.0).abs() < 0.01, "font_size 应为 20.0，实际 {font_size}");
    assert!(
        (line_height - 30.0).abs() < 0.01,
        "line-height:150% 应解析为 30px，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 中非 Px font_size 回退到 DEFAULT_FONT_SIZE (16.0)。
#[test]
fn test_resolve_font_metrics_non_px_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Em(2.0);
    // line-height 默认 Normal

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    // 非 Px font_size 回退到 16.0
    assert!(
        (font_size - 16.0).abs() < 0.01,
        "非 Px font_size 应回退到 16.0，实际 {font_size}"
    );
    // line_height = 16.0 * 1.164 = 18.624
    assert!(
        (line_height - 18.624).abs() < 0.01,
        "line_height 应为 18.624，实际 {line_height}"
    );
}

/// 测试 break_into_lines 调用两次后状态完全重置，不残留第一次调用的结果。
#[test]
fn test_break_into_lines_called_twice_resets_state() {
    // 使用窄容器确保第一次调用产生多行
    let mut ctx = InlineFormattingContext::new(60.0);

    // 第一次调用：产生多行的长文本
    let first_runs = vec![TextRun {
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(first_runs);
    let first_line_count = ctx.lines.len();
    assert!(first_line_count > 1, "第一次调用应产生多行");

    // 切换到宽容器，第二次调用：短文本，只产生单行
    ctx.container_width = 800.0;
    let second_runs = vec![TextRun {
        text: "Short".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(second_runs);

    // 第二次调用后应只有 1 行，无第一次调用的残留
    assert_eq!(
        ctx.lines.len(),
        1,
        "第二次调用后应只有 1 行，无残留，实际 {} 行",
        ctx.lines.len()
    );
    assert_eq!(ctx.lines[0].runs.len(), 1, "应只有 1 个片段");
    assert!(
        ctx.lines[0].runs[0].text.contains("Short"),
        "片段文本应为 'Short'，实际 {}",
        ctx.lines[0].runs[0].text
    );
}

/// 测试 font_size=0.0 时不会 panic。
///
/// 零字体大小意味着估算字符宽度为 0，所有单词放入单行。
#[test]
fn test_zero_font_size_no_panic() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 0.0,
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    // 不应 panic
    ctx.break_into_lines(runs);

    // 零字体大小 → 零字符宽度 → 所有单词放入单行
    assert_eq!(ctx.lines.len(), 1, "零字体大小应产生 1 行");
    // 片段宽度应为 0
    for f in ctx.all_fragments() {
        assert!(f.width.abs() < 0.01, "零字体大小片段宽度应为 0，实际 {}", f.width);
    }
}

/// 测试 line_height=0.0 时不会 panic。
///
/// 零行高意味着行盒高度为 0，y 坐标不会递增。
#[test]
fn test_zero_line_height_no_panic() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "Hello World".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 0.0,
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    // 不应 panic
    ctx.break_into_lines(runs);

    // 行盒高度应为 0.0（所有片段 line_height 为 0）
    assert_eq!(ctx.lines.len(), 1, "零行高应产生 1 行");
    assert!(
        ctx.lines[0].height.abs() < 0.01,
        "零行高行盒高度应为 0，实际 {}",
        ctx.lines[0].height
    );
    assert!(
        ctx.total_height().abs() < 0.01,
        "零行高总高度应为 0，实际 {}",
        ctx.total_height()
    );
}

// ── 新增边界测试：行内格式化上下文边界场景 ──

/// 测试同一行中混合不同字体大小的 TextRun。
///
/// 两个 TextRun 分别使用 10px 和 20px 字体大小放在同一行，
/// 验证行盒高度取两者行高的最大值，且两个片段都被正确放置。
#[test]
fn test_mixed_font_sizes_on_same_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "Small".to_string(),
            node_id: NodeId::default(),
            font_size: 10.0,
            line_height: 12.0,
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
            is_rtl: false,
            is_plaintext_bidi: false,
        },
        TextRun {
            text: "Large".to_string(),
            node_id: NodeId::default(),
            font_size: 20.0,
            line_height: 24.0,
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
            is_rtl: false,
            is_plaintext_bidi: false,
        },
    ];
    ctx.break_into_lines(runs);

    // 两个短词应在同一行
    assert_eq!(ctx.lines.len(), 1, "两个短词应在同一行");

    // 行盒高度应取最大行高 24.0
    assert!(
        (ctx.lines[0].height - 24.0).abs() < 0.01,
        "行盒高度应取 max(12.0, 24.0) = 24.0，实际 {}",
        ctx.lines[0].height
    );

    // 应有 2 个片段
    let fragments = ctx.all_fragments();
    assert_eq!(fragments.len(), 2, "应有 2 个片段");

    // 第一个片段（10px）x 从 0 开始，第二个片段紧随其后
    assert!(
        fragments[0].x.abs() < 0.01,
        "第一个片段 x 应为 0，实际 {}",
        fragments[0].x
    );
    assert!(
        fragments[1].x >= fragments[0].x + fragments[0].width - 0.01,
        "第二个片段 x 应在第一个片段之后"
    );

    // 两个片段的 font_size 各自保留
    assert!(
        (fragments[0].font_size - 10.0).abs() < 0.01,
        "第一个片段 font_size 应为 10.0"
    );
    assert!(
        (fragments[1].font_size - 20.0).abs() < 0.01,
        "第二个片段 font_size 应为 20.0"
    );
}

/// 测试单个超长单词超过容器宽度时仍被放置在第一行。
///
/// 行内格式化的首单词总是放入当前行，即使宽度超过 container_width
/// （因为 current_line.runs 为空，不会触发换行条件）。
#[test]
fn test_single_word_exceeds_container_width() {
    let mut ctx = InlineFormattingContext::new(100.0);
    // 构造一个超长单词：50 个字符 × 16×0.6 = 480px，远超 100px
    let long_word = "supercalifragilisticexpialidocious_and_then_some";
    let runs = vec![TextRun {
        text: long_word.to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    // 首单词总是放置在第一行（即使溢出）
    assert_eq!(
        ctx.lines.len(),
        1,
        "单个超长单词应在第一行，实际 {} 行",
        ctx.lines.len()
    );
    assert_eq!(ctx.lines[0].runs.len(), 1, "应只有 1 个片段");

    let fragment = &ctx.lines[0].runs[0];
    // 片段宽度应超过容器宽度
    assert!(
        fragment.width > ctx.container_width,
        "片段宽度 {} 应超过容器宽度 {}",
        fragment.width,
        ctx.container_width
    );

    // 片段 x 应为 0（行首）
    assert!(fragment.x.abs() < 0.01, "首单词片段 x 应为 0，实际 {}", fragment.x);
}

/// 测试 container_width=0.0 时不会 panic，且首单词仍被放置。
///
/// 零宽度容器下，第一个单词因 current_line.runs 为空而总是放入当前行，
/// 后续每个单词因 current_x + word_width > 0.0 且行非空而触发换行。
#[test]
fn test_empty_container_width_first_word_still_placed() {
    let mut ctx = InlineFormattingContext::new(0.0);
    let runs = vec![TextRun {
        text: "Hello World Foo".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    // 不应 panic
    ctx.break_into_lines(runs);

    // 至少有一行（首单词总是放置）
    assert!(!ctx.lines.is_empty(), "container_width=0 时仍应产生行盒");

    // 首单词应被放置
    assert!(!ctx.lines[0].runs.is_empty(), "第一行应至少有一个片段");
    assert!(
        ctx.lines[0].runs[0].text.contains("Hello"),
        "首单词应为 'Hello'，实际 {}",
        ctx.lines[0].runs[0].text
    );

    // 多个单词应产生多行（每个后续单词都溢出零宽度容器）
    assert!(
        ctx.lines.len() >= 2,
        "零宽度容器中多个单词应产生多行，实际 {} 行",
        ctx.lines.len()
    );
}

/// 测试同一行中多个不同行高的 TextRun，行盒高度取最大值。
///
/// 三个 TextRun：font_size=12 line_height=16、font_size=20 line_height=28、
/// font_size=14 line_height=18。在宽容器中放入同一行时，
/// 行盒高度应取 max(16, 28, 18) = 28。
#[test]
fn test_multiple_lines_line_height_max_per_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "Small".to_string(),
            node_id: NodeId::default(),
            font_size: 12.0,
            line_height: 16.0,
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
            is_rtl: false,
            is_plaintext_bidi: false,
        },
        TextRun {
            text: "Big".to_string(),
            node_id: NodeId::default(),
            font_size: 20.0,
            line_height: 28.0,
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
            is_rtl: false,
            is_plaintext_bidi: false,
        },
        TextRun {
            text: "Med".to_string(),
            node_id: NodeId::default(),
            font_size: 14.0,
            line_height: 18.0,
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
            is_rtl: false,
            is_plaintext_bidi: false,
        },
    ];
    ctx.break_into_lines(runs);

    // 宽容器中三个短词应在同一行
    assert_eq!(ctx.lines.len(), 1, "三个短词应在同一行，实际 {} 行", ctx.lines.len());

    // 行盒高度应取 max(16, 28, 18) = 28
    assert!(
        (ctx.lines[0].height - 28.0).abs() < 0.01,
        "行盒高度应取 max(16, 28, 18) = 28.0，实际 {}",
        ctx.lines[0].height
    );

    // 所有片段都应存在
    let fragments = ctx.all_fragments();
    assert_eq!(fragments.len(), 3, "应有 3 个片段");
}

/// 测试纯空白文本不产生任何行盒。
///
/// TextRun 的文本为 "   "（仅空格），break_into_lines 通过
/// split_into_words → split_whitespace 得到零个单词，
/// 因此不应产生任何行盒或片段。
#[test]
fn test_whitespace_only_text_no_lines() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "   ".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    // 纯空白文本不应产生行盒
    assert!(
        ctx.lines.is_empty(),
        "纯空白文本不应产生行盒，实际 {} 行",
        ctx.lines.len()
    );

    // 不应有任何片段
    let fragments = ctx.all_fragments();
    assert!(
        fragments.is_empty(),
        "纯空白文本不应有片段，实际 {} 个",
        fragments.len()
    );

    // 总高度应为 0
    assert!(
        ctx.total_height().abs() < 0.01,
        "纯空白文本总高度应为 0，实际 {}",
        ctx.total_height()
    );
}

// ── inline-block 布局测试 ──

/// 测试文本 + inline-block + 文本在同一行排列。
///
/// 宽容器中，文本片段、一个 50x50 的 inline-block、再接文本片段，
/// 三者应在同一行内水平排列，x 坐标递增。
#[test]
fn test_inline_block_on_same_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 50.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 50.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
        InlineItem::Text(TextRun {
            text: "World".to_string(),
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
    ];
    ctx.break_items_into_lines(items);

    // 所有内容应在同一行
    assert_eq!(
        ctx.lines.len(),
        1,
        "文本 + inline-block + 文本应在同一行，实际 {} 行",
        ctx.lines.len()
    );

    let fragments = ctx.all_fragments();
    // "Hello " (1 word) + inline-block (1) + "World " (1) = 3 fragments
    assert!(
        fragments.len() >= 3,
        "应至少有 3 个片段（Hello、inline-block、World），实际 {}",
        fragments.len()
    );

    // 验证 x 坐标递增
    for i in 1..fragments.len() {
        assert!(
            fragments[i].x >= fragments[i - 1].x + fragments[i - 1].width - 0.01,
            "片段 {} 的 x 坐标应紧随片段 {} 之后",
            i,
            i - 1
        );
    }

    // inline-block 片段（font_size=0）应有 50x50 尺寸
    let ib_fragment = fragments.iter().find(|f| f.font_size < 0.01 && f.width > 40.0);
    assert!(
        ib_fragment.is_some(),
        "应包含 inline-block 片段（font_size≈0, width≈50）"
    );
    let ib = ib_fragment.unwrap();
    assert!(
        (ib.width - 50.0).abs() < 0.01,
        "inline-block 宽度应为 50，实际 {}",
        ib.width
    );
    assert!(
        (ib.height - 50.0).abs() < 0.01,
        "inline-block 高度应为 50，实际 {}",
        ib.height
    );
}

/// 测试 inline-block 在当前行放不下时换到下一行。
///
/// 窄容器（80px）中，先放一个文本片段占满大部分宽度，
/// 再放一个 50x30 的 inline-block，它应换到第二行。
#[test]
fn test_inline_block_wraps_to_next_line() {
    let mut ctx = InlineFormattingContext::new(80.0);
    let items = vec![
        InlineItem::Text(TextRun {
            text: "WideText".to_string(),
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 30.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 30.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
    ];
    ctx.break_items_into_lines(items);

    // 应产生至少 2 行（文本 + inline-block 换行）
    assert!(
        ctx.lines.len() >= 2,
        "inline-block 应换到下一行，实际 {} 行",
        ctx.lines.len()
    );

    // 第二行应有 inline-block 片段
    let last_line = ctx.lines.last().unwrap();
    let has_ib = last_line.runs.iter().any(|r| r.width > 40.0 && r.font_size < 0.01);
    assert!(has_ib, "最后一行应包含 inline-block 片段");

    // 第二行 y 坐标应大于第一行
    assert!(
        ctx.lines[1].y > ctx.lines[0].y,
        "第二行 y({}) 应大于第一行 y({})",
        ctx.lines[1].y,
        ctx.lines[0].y
    );
}

/// 测试 inline-block 高度比文本高时，行盒高度增加。
///
/// 文本行高 20px，inline-block 高度 60px，放在同一行时，
/// 行盒高度应取 max(20, 60) = 60。
#[test]
fn test_inline_block_height_contributes_to_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
            text: "Short".to_string(),
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 40.0,
            height: 60.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 60.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(ctx.lines.len(), 1, "应在同一行");

    // 行盒高度应取 max(20, 60) = 60
    assert!(
        (ctx.lines[0].height - 60.0).abs() < 0.01,
        "行盒高度应取 max(文本行高20, inline-block高度60) = 60，实际 {}",
        ctx.lines[0].height
    );
}

/// 测试两个 inline-block 在同一行并排排列。
///
/// 两个 50x40 的 inline-block 在 800px 宽容器中，
/// 应在同一行水平排列，x 坐标递增。
#[test]
fn test_multiple_inline_blocks_on_same_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 40.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 40.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 40.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 40.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
    ];
    ctx.break_items_into_lines(items);

    // 两个 inline-block 应在同一行
    assert_eq!(
        ctx.lines.len(),
        1,
        "两个小 inline-block 应在同一行，实际 {} 行",
        ctx.lines.len()
    );

    let fragments = ctx.all_fragments();
    assert_eq!(fragments.len(), 2, "应有 2 个片段");

    // x 坐标递增
    assert!(
        fragments[1].x >= fragments[0].x + fragments[0].width - 0.01,
        "第二个 inline-block (x={}) 应在第一个 (x={}, w={}) 之后",
        fragments[1].x,
        fragments[0].x,
        fragments[0].width
    );

    // 行盒高度应为 40（两个 inline-block 高度相同）
    assert!(
        (ctx.lines[0].height - 40.0).abs() < 0.01,
        "行盒高度应为 40，实际 {}",
        ctx.lines[0].height
    );

    // 总高度应为 40（单行）
    assert!(
        (ctx.total_height() - 40.0).abs() < 0.01,
        "总高度应为 40，实际 {}",
        ctx.total_height()
    );
}

// ── 按字符宽度估算测试 ──

/// CJK 字符宽度约为 ASCII 字母的 2 倍。
///
/// CJK 全角字符宽度 ≈ font_size，ASCII 字母宽度 ≈ font_size × 0.55，
/// 比值约 1.0 / 0.55 ≈ 1.8，接近 2 倍。
#[test]
fn test_cjk_char_wider_than_ascii() {
    let font_size = 16.0;
    let cjk_width = estimate_char_width('中', font_size, false);
    let ascii_width = estimate_char_width('A', font_size, false);

    assert!(
        cjk_width > ascii_width * 1.5,
        "CJK 字符宽度 ({}) 应至少为 ASCII 字母宽度 ({}) 的 1.5 倍",
        cjk_width,
        ascii_width
    );
    // CJK 宽度应约为 font_size
    assert!(
        (cjk_width - font_size).abs() < 0.01,
        "CJK 字符宽度应约为 font_size ({})，实际 {}",
        font_size,
        cjk_width
    );
}

/// 空格宽度比字母 W 窄。
#[test]
fn test_space_narrower_than_letter() {
    let font_size = 16.0;
    let space_width = estimate_char_width(' ', font_size, false);
    let w_width = estimate_char_width('W', font_size, false);

    assert!(
        space_width < w_width,
        "空格宽度 ({}) 应小于字母 W 宽度 ({})",
        space_width,
        w_width
    );
    // 空格应为 font_size * 0.25
    assert!(
        (space_width - font_size * 0.25).abs() < 0.01,
        "空格宽度应为 {}，实际 {}",
        font_size * 0.25,
        space_width
    );
}

/// 混合 ASCII/CJK 文本在窄容器中正确换行。
///
/// CJK 字符较宽，应比纯 ASCII 文本更早触发换行。
#[test]
fn test_mixed_ascii_cjk_line_breaking() {
    let font_size = 16.0;
    // 纯 ASCII 文本
    let mut ctx_ascii = InlineFormattingContext::new(100.0);
    let runs_ascii = vec![TextRun {
        text: "Hello World Foo Bar".to_string(),
        node_id: NodeId::default(),
        font_size,
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx_ascii.break_into_lines(runs_ascii);

    // 混合 CJK 文本（CJK 字符更宽，应产生更多行）
    let mut ctx_mixed = InlineFormattingContext::new(100.0);
    let runs_mixed = vec![TextRun {
        text: "Hello 世界 Foo 测试 Bar".to_string(),
        node_id: NodeId::default(),
        font_size,
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx_mixed.break_into_lines(runs_mixed);

    // 混合 CJK 文本应产生至少和纯 ASCII 相同或更多的行数
    assert!(
        ctx_mixed.lines.len() >= ctx_ascii.lines.len(),
        "混合 CJK 文本行数 ({}) 应不少于纯 ASCII 行数 ({})",
        ctx_mixed.lines.len(),
        ctx_ascii.lines.len()
    );
}

/// 验证各类字符的具体宽度估算值。
///
/// - 'W' (ASCII 字母): font_size × 0.55
/// - 'i' (ASCII 字母): font_size × 0.55
/// - '中' (CJK): font_size × 1.0
/// - ' ' (空格): font_size × 0.25
/// - '.' (标点): font_size × 0.4
#[test]
fn test_estimate_char_width_various_chars() {
    let font_size = 16.0;

    // ASCII 字母
    let w = estimate_char_width('W', font_size, false);
    assert!(
        (w - font_size * 0.55).abs() < 0.01,
        "'W' 宽度应为 {}，实际 {}",
        font_size * 0.55,
        w
    );

    let i = estimate_char_width('i', font_size, false);
    assert!(
        (i - font_size * 0.55).abs() < 0.01,
        "'i' 宽度应为 {}，实际 {}",
        font_size * 0.55,
        i
    );

    // CJK 字符
    let cjk = estimate_char_width('中', font_size, false);
    assert!(
        (cjk - font_size).abs() < 0.01,
        "CJK '中' 宽度应为 {}，实际 {}",
        font_size,
        cjk
    );

    // 空格
    let space = estimate_char_width(' ', font_size, false);
    assert!(
        (space - font_size * 0.25).abs() < 0.01,
        "空格宽度应为 {}，实际 {}",
        font_size * 0.25,
        space
    );

    // 标点
    let period = estimate_char_width('.', font_size, false);
    assert!(
        (period - font_size * 0.4).abs() < 0.01,
        "'.' 宽度应为 {}，实际 {}",
        font_size * 0.4,
        period
    );
}

// ── br 元素测试 ──

/// 测试两个文本运行之间插入 Br 产生两行。
///
/// "Hello" + Br + "World" 应产生两行：
/// 第一行包含 "Hello "，第二行包含 "World "。
#[test]
fn test_br_forces_line_break() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::Br,
        InlineItem::Text(TextRun {
            text: "World".to_string(),
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(
        ctx.lines.len(),
        2,
        "Hello + Br + World 应产生 2 行，实际 {} 行",
        ctx.lines.len()
    );

    // 第一行应包含 "Hello "
    assert!(
        ctx.lines[0].runs[0].text.contains("Hello"),
        "第一行应包含 'Hello'，实际 {}",
        ctx.lines[0].runs[0].text
    );

    // 第二行应包含 "World "
    assert!(
        ctx.lines[1].runs[0].text.contains("World"),
        "第二行应包含 'World'，实际 {}",
        ctx.lines[1].runs[0].text
    );

    // 第二行 y 应在第一行之后
    assert!(
        ctx.lines[1].y >= ctx.lines[0].y + ctx.lines[0].height - 0.01,
        "第二行 y({}) 应在第一行 (y={}, h={}) 之后",
        ctx.lines[1].y,
        ctx.lines[0].y,
        ctx.lines[0].height
    );
}

/// 测试 Br 作为首个条目产生空的第一行。
///
/// Br + "Hello" 应产生两行：第一行为空（height=0），第二行包含 "Hello "。
/// 第一行被 Br 强制推入，此时 current_line.runs 为空但仍然被 push。
#[test]
fn test_br_at_start_of_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Br,
        InlineItem::Text(TextRun {
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
    ];
    ctx.break_items_into_lines(items);

    // Br 强制换行产生第一行（空行），然后 "Hello" 在第二行
    assert!(
        ctx.lines.len() >= 2,
        "Br + Hello 应产生至少 2 行，实际 {} 行",
        ctx.lines.len()
    );

    // 第一行应为空（由 Br 强制产生）
    assert!(
        ctx.lines[0].runs.is_empty(),
        "第一行（Br 产生）应为空，实际有 {} 个片段",
        ctx.lines[0].runs.len()
    );

    // 第二行应包含 "Hello "
    assert!(
        ctx.lines[1].runs[0].text.contains("Hello"),
        "第二行应包含 'Hello'，实际 {}",
        ctx.lines[1].runs[0].text
    );
}

/// 测试文本后跟 Br 产生换行。
///
/// "Hello" + Br 应产生一行（包含 "Hello "），Br 强制将该行推入结果。
/// 最终行列表只有 1 行（Br 之后没有更多内容，空行不会被推入）。
#[test]
fn test_br_at_end_of_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::Br,
    ];
    ctx.break_items_into_lines(items);

    // "Hello" 被放入行，然后 Br 强制推入该行。
    // Br 之后没有内容，空行不会被推入。
    assert_eq!(
        ctx.lines.len(),
        1,
        "Hello + Br 应产生 1 行（Br 之后无内容不产生空行），实际 {} 行",
        ctx.lines.len()
    );

    // 该行应包含 "Hello "
    assert!(
        ctx.lines[0].runs[0].text.contains("Hello"),
        "第一行应包含 'Hello'，实际 {}",
        ctx.lines[0].runs[0].text
    );
}

/// 测试连续三个 Br 产生空行。
///
/// Br + Br + Br 产生三行空行：第一个 Br 推入空行1，
/// 第二个 Br 推入空行2，第三个 Br 推入空行3。
/// 最后无内容，不会再推入一行。
#[test]
fn test_multiple_br_elements() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![InlineItem::Br, InlineItem::Br, InlineItem::Br];
    ctx.break_items_into_lines(items);

    // 每个 Br 推入一行（当前行），然后开启新行。
    // 3 个 Br → 3 行被推入，第 4 行（空）不被推入。
    assert_eq!(
        ctx.lines.len(),
        3,
        "3 个连续 Br 应产生 3 行空行，实际 {} 行",
        ctx.lines.len()
    );

    // 所有行都应为空
    for (i, line) in ctx.lines.iter().enumerate() {
        assert!(
            line.runs.is_empty(),
            "第 {} 行应为空，实际有 {} 个片段",
            i,
            line.runs.len()
        );
    }

    // R1286：空 Br 行须有 strut 高度（line-height，CSS §10.8.1）——空 line box 仍含 strut，
    // 非内容高度 0。旧行为（height=0）致 `<p><br></p>` 等塌缩（chromium 给一行 line-height）。
    // default_line_height=20（NORMAL 行高近似）。
    for (i, line) in ctx.lines.iter().enumerate() {
        assert!(
            line.height > 10.0,
            "第 {} 行（空 Br）应有 strut 高度，实际 {}",
            i,
            line.height
        );
    }
}

/// 测试文本 + inline-block + Br + 文本的正确布局。
///
/// "Hello" + inline-block(50x50) + Br + "World" 应产生两行：
/// 第一行包含 "Hello " 和 inline-block，第二行包含 "World "。
#[test]
fn test_br_with_inline_blocks() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 50.0,
            node_id: NodeId::default(),
            vertical_align: VA::Baseline,
            baseline: 50.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
        }),
        InlineItem::Br,
        InlineItem::Text(TextRun {
            text: "World".to_string(),
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
            is_rtl: false,
            is_plaintext_bidi: false,
        }),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(
        ctx.lines.len(),
        2,
        "Text + InlineBlock + Br + Text 应产生 2 行，实际 {} 行",
        ctx.lines.len()
    );

    // 第一行应有 "Hello " 片段和 inline-block 片段
    assert!(
        ctx.lines[0].runs.len() >= 2,
        "第一行应至少有 2 个片段（文本 + inline-block），实际 {}",
        ctx.lines[0].runs.len()
    );

    // 第一行应包含 inline-block（font_size=0, width=50）
    let has_ib = ctx.lines[0]
        .runs
        .iter()
        .any(|r| r.font_size < 0.01 && (r.width - 50.0).abs() < 0.01);
    assert!(has_ib, "第一行应包含 inline-block 片段");

    // 第一行高度应取 max(20, 50) = 50
    assert!(
        (ctx.lines[0].height - 50.0).abs() < 0.01,
        "第一行高度应取 max(文本行高20, inline-block高度50) = 50，实际 {}",
        ctx.lines[0].height
    );

    // 第二行应包含 "World "
    assert!(
        ctx.lines[1].runs[0].text.contains("World"),
        "第二行应包含 'World'，实际 {}",
        ctx.lines[1].runs[0].text
    );

    // 第二行 y 应在第一行之后
    assert!(
        ctx.lines[1].y >= ctx.lines[0].y + ctx.lines[0].height - 0.01,
        "第二行 y({}) 应在第一行 (y={}, h={}) 之后",
        ctx.lines[1].y,
        ctx.lines[0].y,
        ctx.lines[0].height
    );
}

// ── is_cjk_character / estimate_string_width 边界条件测试 ──

/// 测试 is_cjk_character：CJK 统一表意文字基本区（U+4E00）。
#[test]
fn test_is_cjk_unified_ideographs() {
    assert!(is_cjk_character('\u{4E00}'), "U+4E00 一 应为 CJK");
    assert!(is_cjk_character('\u{9FFF}'), "U+9FFF 龠 应为 CJK");
    assert!(is_cjk_character('中'), "'中' 应为 CJK");
}

/// 测试 is_cjk_character：CJK 扩展 A（U+3400..U+4DBF）。
#[test]
fn test_is_cjk_extension_a() {
    assert!(is_cjk_character('\u{3400}'), "U+3400 应为 CJK");
    assert!(is_cjk_character('\u{4DBF}'), "U+4DBF 应为 CJK");
}

/// 测试 is_cjk_character：平假名和片假名。
#[test]
fn test_is_cjk_hiragana_katakana() {
    assert!(is_cjk_character('\u{3040}'), "U+3040 应为 CJK");
    assert!(is_cjk_character('\u{309F}'), "U+309F 应为 CJK（平假名末尾）");
    assert!(is_cjk_character('\u{30A0}'), "U+30A0 应为 CJK（片假名起始）");
    assert!(is_cjk_character('\u{30FF}'), "U+30FF 应为 CJK（片假名末尾）");
}

/// 测试 is_cjk_character：韩文音节（U+AC00..U+D7AF）。
#[test]
fn test_is_cjk_korean_syllables() {
    assert!(is_cjk_character('\u{AC00}'), "U+AC00 가 应为 CJK");
    assert!(is_cjk_character('\u{D7AF}'), "U+D7AF 힣 应为 CJK");
}

/// 测试 is_cjk_character：全角形式（U+FF00..U+FFEF）。
#[test]
fn test_is_cjk_fullwidth_forms() {
    assert!(is_cjk_character('\u{FF00}'), "U+FF00 全角感叹号应为 CJK");
    assert!(is_cjk_character('\u{FFEF}'), "U+FFEF 应为 CJK");
}

/// 测试 is_cjk_character：非 CJK 字符返回 false。
#[test]
fn test_is_cjk_non_cjk_returns_false() {
    assert!(!is_cjk_character('A'), "ASCII 大写字母不应为 CJK");
    assert!(!is_cjk_character('z'), "ASCII 小写字母不应为 CJK");
    assert!(!is_cjk_character('0'), "数字不应为 CJK");
    assert!(!is_cjk_character(' '), "空格不应为 CJK");
    assert!(!is_cjk_character('.'), "标点不应为 CJK");
    assert!(!is_cjk_character('\u{00E9}'), "é 不应为 CJK");
}

/// 测试 estimate_string_width：空字符串宽度为 0。
#[test]
fn test_estimate_string_width_empty() {
    assert!(
        estimate_string_width("", 16.0, false).abs() < 0.001,
        "空字符串宽度应为 0"
    );
}

/// 测试 estimate_string_width：纯 ASCII 字符串。
#[test]
fn test_estimate_string_width_ascii() {
    let width = estimate_string_width("Hello", 16.0, false);
    // 5 个字母 × 16 × 0.55 = 44.0
    let expected = 5.0 * 16.0 * 0.55;
    assert!(
        (width - expected).abs() < 0.001,
        "纯 ASCII 宽度应为 {}，实际 {}",
        expected,
        width
    );
}

/// 测试 estimate_string_width：纯 CJK 字符串。
#[test]
fn test_estimate_string_width_cjk() {
    let width = estimate_string_width("中文", 16.0, false);
    // 2 个 CJK 字符 × 16.0 = 32.0
    let expected = 2.0 * 16.0;
    assert!(
        (width - expected).abs() < 0.001,
        "纯 CJK 宽度应为 {}，实际 {}",
        expected,
        width
    );
}

/// 测试 estimate_string_width：中英混合。
#[test]
fn test_estimate_string_width_mixed() {
    let width = estimate_string_width("A中", 16.0, false);
    // 'A' = 16×0.55 = 8.8, '中' = 16.0, 总 = 24.8
    let expected = 16.0 * 0.55 + 16.0;
    assert!(
        (width - expected).abs() < 0.001,
        "混合宽度应为 {}，实际 {}",
        expected,
        width
    );
}

/// 测试 estimate_string_width：font_size 为 0 时所有宽度为 0。
#[test]
fn test_estimate_string_width_zero_font_size() {
    let width = estimate_string_width("Hello世界", 0.0, false);
    assert!(width.abs() < 0.001, "零 font_size 宽度应为 0，实际 {}", width);
}

// ── text-align-last 行为测试 ──

/// 测试 text-align-last 默认（None）时，justify 的最后一行回退到左对齐。
///
/// 3 行文本使用 text-align: justify，最后一行不应两端对齐，
/// 而是默认回退到左对齐（x = 0）。
#[test]
fn test_text_align_last_none_justify_falls_back_to_left() {
    let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Justify);
    // 构造多行文本
    let runs = vec![TextRun {
        text: "alpha beta gamma delta epsilon zeta".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应产生至少 2 行，实际 {} 行", ctx.lines.len());

    // 最后一行的第一个片段 x 应为 0（回退到左对齐，不做 justify 偏移）
    let last_line = ctx.lines.last().unwrap();
    if !last_line.runs.is_empty() {
        assert!(
            last_line.runs[0].x.abs() < 0.01,
            "最后一行（justify 回退左对齐）x 应为 0，实际 {}",
            last_line.runs[0].x
        );
    }
}

/// 测试 text-align-last: center 时，最后一行居中。
///
/// 多行文本使用 text-align: left，text-align-last: center。
/// 最后一行的片段应有正偏移（居中效果）。
#[test]
fn test_text_align_last_center_on_last_line() {
    let mut ctx = InlineFormattingContext::new(100.0)
        .with_text_align(TextAlign::Left)
        .with_text_align_last(Some(TextAlign::Center));
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应产生至少 2 行，实际 {} 行", ctx.lines.len());

    // 最后一行应有正 x 偏移（居中效果）
    let last_line = ctx.lines.last().unwrap();
    if !last_line.runs.is_empty() {
        assert!(
            last_line.runs[0].x > 0.0,
            "最后一行（center）x 应 > 0（居中偏移），实际 {}",
            last_line.runs[0].x
        );
    }

    // 非最后一行 x 应为 0（text-align: left）
    assert!(
        ctx.lines[0].runs[0].x.abs() < 0.01,
        "第一行（left）x 应为 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试 text-align-last: right 时，最后一行右对齐。
///
/// 多行文本，最后一行的片段应有较大 x 偏移（右对齐效果）。
#[test]
fn test_text_align_last_right_on_last_line() {
    let mut ctx = InlineFormattingContext::new(100.0)
        .with_text_align(TextAlign::Left)
        .with_text_align_last(Some(TextAlign::Right));
    let runs = vec![TextRun {
        text: "alpha beta gamma delta epsilon zeta eta".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应产生至少 2 行");

    let last_line = ctx.lines.last().unwrap();
    if !last_line.runs.is_empty() {
        // 右对齐：x 偏移应 > 0 且比较大
        assert!(
            last_line.runs[0].x > 0.0,
            "最后一行（right）x 应 > 0（右对齐偏移），实际 {}",
            last_line.runs[0].x
        );
    }
}

/// 测试 text-align-last: justify 时，最后一行也两端对齐。
///
/// 正常 justify 的最后一行回退到左对齐，但显式设置
/// text-align-last: justify 后，最后一行也应均匀分配空间。
#[test]
fn test_text_align_last_justify_on_last_line() {
    let mut ctx = InlineFormattingContext::new(100.0)
        .with_text_align(TextAlign::Left)
        .with_text_align_last(Some(TextAlign::Justify));
    let runs = vec![TextRun {
        text: "alpha beta gamma delta epsilon zeta eta theta iota".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应产生至少 2 行");

    // 最后一行如果有 2+ 个片段，justify 会分配间距
    // 第一个片段 x 应为 0（justify 从行首开始）
    let last_line = ctx.lines.last().unwrap();
    if last_line.runs.len() >= 2 {
        assert!(
            last_line.runs[0].x.abs() < 0.01,
            "最后一行（justify）第一个片段 x 应为 0，实际 {}",
            last_line.runs[0].x
        );
        // justify 时后续片段应有间距
        assert!(
            last_line.runs[1].x > last_line.runs[0].width + 1.0,
            "最后一行（justify）片段间应有间距分配"
        );
    }
}

/// 测试单行文本 + text-align-last 时，该行视为最后一行。
///
/// 单行就是最后一行，text-align-last 应直接作用于它。
#[test]
fn test_text_align_last_single_line_treated_as_last() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_text_align(TextAlign::Left)
        .with_text_align_last(Some(TextAlign::Center));
    let runs = vec![TextRun {
        text: "Short".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1, "应只有 1 行");

    // 单行 = 最后一行，text-align-last: center 应生效
    let line = &ctx.lines[0];
    assert!(
        line.runs[0].x > 0.0,
        "单行（center）x 应 > 0（居中偏移），实际 {}",
        line.runs[0].x
    );
}

/// 测试 text-align: left + text-align-last 为 None 时提前返回，
/// 不应用任何对齐偏移。
#[test]
fn test_text_align_left_no_align_last_no_offset() {
    let mut ctx = InlineFormattingContext::new(100.0).with_text_align(TextAlign::Left);
    // 不设置 text_align_last（默认 None）
    let runs = vec![TextRun {
        text: "alpha beta gamma delta epsilon".to_string(),
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
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    // 所有行左对齐，每行第一个片段 x 应为 0
    for (i, line) in ctx.lines.iter().enumerate() {
        if !line.runs.is_empty() {
            assert!(
                line.runs[0].x.abs() < 0.01,
                "行 {} 第一个片段 x 应为 0（左对齐无偏移），实际 {}",
                i,
                line.runs[0].x
            );
        }
    }
}
