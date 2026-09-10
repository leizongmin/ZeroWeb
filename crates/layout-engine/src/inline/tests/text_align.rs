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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
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

// ── R4213：text-group-align 行组对齐测试（CSS Text 4 #text-group-align-property）──

/// 构造一个多行 pre 文本 run（\n 强制断行）。
fn group_align_runs() -> Vec<TextRun> {
    vec![TextRun {
        text: "AAAAAAAAAAAAAAAA\nAA\nA".to_string(),
        node_id: NodeId::default(),
        font_size: 10.0,
        line_height: 10.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_left: 0.0,
        padding_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
    }]
}

/// 组盒测量：多行内容的 group_box = 最宽行的包围盒。
#[test]
fn test_group_box_measures_widest_line() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true);
    ctx.break_into_lines(group_align_runs());
    assert_eq!(ctx.lines.len(), 3, "pre + 断行应得 3 行");

    let (group_left, group_width) = ctx.group_box();
    assert!(
        (group_left - 0.0).abs() < 0.5,
        "未对齐行组盒左缘应为 0，实际 {group_left}"
    );
    // Ahem 10px：最宽行 16 个 A = 160px。
    assert!(
        (group_width - 160.0).abs() < 1.0,
        "组宽应 = 最宽行 160px，实际 {group_width}"
    );
}

/// text-group-align: center——组内各行左对齐（text-align 默认），组整体在容器居中。
#[test]
fn test_group_align_center_shifts_group() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_group_align(TextGroupAlign::Center);
    ctx.break_into_lines(group_align_runs());

    // 组宽 160，容器 300 → 位移 = (300-160)/2 = 70。各行均左对齐于组内（x 同为 70）。
    for (i, line) in ctx.lines.iter().enumerate() {
        let x = line.runs[0].x;
        assert!((x - 70.0).abs() < 0.5, "第 {i} 行首片段 x 应为 70（组居中），实际 {x}");
    }
}

/// text-group-align: right——组右缘贴容器右缘。
#[test]
fn test_group_align_right_pins_group_to_container_edge() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_group_align(TextGroupAlign::Right);
    ctx.break_into_lines(group_align_runs());

    // 位移 = 300-160 = 140：各行起点 140，最宽行右缘 = 140+160 = 300。
    let first = &ctx.lines[0];
    let max_right = first.runs.iter().map(|r| r.x + r.width).fold(f32::MIN, f32::max);
    assert!((max_right - 300.0).abs() < 0.5, "组右缘应贴容器 300，实际 {max_right}");
}

/// text-group-align + text-align:center 组合——text-align 在**组盒内**对齐各行，
/// 非容器轴（组内对齐后组定位不再改变组盒边界）。
#[test]
fn test_group_align_with_text_align_center_aligns_within_group() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_align(TextAlign::Center)
        .with_text_group_align(TextGroupAlign::Center);
    ctx.break_into_lines(group_align_runs());

    // 最宽行（16A，= 组宽）在组内居中 remaining=0，组居中位移 70 → 起点应 70。
    let widest_x = ctx.lines[0].runs[0].x;
    assert!((widest_x - 70.0).abs() < 0.5, "最宽行起点应 70，实际 {widest_x}");
    // 窄行（AA，20px 宽）在组内居中：组内起点 (160-20)/2 = 70，加组位移 70 → 140。
    let narrow_x = ctx.lines[1].runs[0].x;
    assert!(
        (narrow_x - 140.0).abs() < 0.5,
        "窄行组内居中起点应 140，实际 {narrow_x}"
    );
}

/// text-group-align: none 不位移（默认行为回归守卫）。
#[test]
fn test_group_align_none_is_noop() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true);
    ctx.break_into_lines(group_align_runs());
    for line in &ctx.lines {
        assert!(line.runs[0].x.abs() < 0.5, "none 不应位移");
    }
}

/// 组窄于容器、右对齐已把组贴右缘后，组盒边界不受组定位影响（可重复应用幂等）。
#[test]
fn test_group_align_idempotent_after_positioning() {
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_group_align(TextGroupAlign::Center);
    ctx.break_into_lines(group_align_runs());
    let x_first_pass = ctx.lines[0].runs[0].x;
    // 再跑一次组定位（模拟重复调用）：组盒已位移，group_left=70 → offset = 70-70 = 0。
    ctx.apply_text_group_alignment();
    assert!((ctx.lines[0].runs[0].x - x_first_pass).abs() < 0.5, "组定位应幂等");
}
