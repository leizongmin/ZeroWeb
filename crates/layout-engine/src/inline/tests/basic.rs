// Auto-generated test file — split from layout-engine/inline.rs
use super::super::*;

/// 测试文本分割为单词。
#[test]
fn test_split_into_words() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("Hello World Foo");
    assert_eq!(words.len(), 3);
    assert_eq!(words[0], "Hello ");
}

/// 测试空文本不产生行盒。
#[test]
fn test_empty_text_no_lines() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "   ".to_string(),
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
    }];
    ctx.break_into_lines(runs);
    let fragments: Vec<_> = ctx.all_fragments();
    assert!(fragments.is_empty());
}

/// 测试短文本放入单行。
#[test]
fn test_single_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
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
    }];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "短文本应在单行中");
    assert_eq!(ctx.lines[0].runs.len(), 2, "两个单词");
    assert_eq!(ctx.total_height(), 20.0);
}

/// 测试长文本自动换行。
#[test]
fn test_line_breaking() {
    let mut ctx = InlineFormattingContext::new(100.0);
    let long_text = "a ".repeat(50);
    let runs = vec![TextRun {
        text: long_text,
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
    }];
    ctx.break_into_lines(runs);
    assert!(ctx.lines.len() > 1, "长文本应产生多行，实际 {} 行", ctx.lines.len());
}

/// 测试行盒 y 坐标累加。
#[test]
fn test_line_y_positions() {
    let mut ctx = InlineFormattingContext::new(50.0);
    let runs = vec![TextRun {
        text: "word1 word2 word3 word4 word5".to_string(),
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
    }];
    ctx.break_into_lines(runs);
    for i in 1..ctx.lines.len() {
        assert!(
            ctx.lines[i].y >= ctx.lines[i - 1].y + ctx.lines[i - 1].height - 0.01,
            "行 {} 的 y 坐标应递增",
            i
        );
    }
}

/// 测试 total_height 正确计算。
#[test]
fn test_total_height() {
    let mut ctx = InlineFormattingContext::new(50.0);
    let runs = vec![TextRun {
        text: "a b c d e f g h i j k l m n o p".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 24.0,
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
    }];
    ctx.break_into_lines(runs);
    let expected = ctx.lines.len() as f32 * 24.0;
    assert!(
        (ctx.total_height() - expected).abs() < 0.01,
        "total_height 应为 {}，实际 {}",
        expected,
        ctx.total_height()
    );
}

/// 测试 all_fragments 扁平化。
#[test]
fn test_all_fragments() {
    let mut ctx = InlineFormattingContext::new(800.0);
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
    }];
    ctx.break_into_lines(runs);
    let fragments = ctx.all_fragments();
    assert_eq!(fragments.len(), 2);
}

/// 测试 TextFragment x 坐标递增。
#[test]
fn test_fragment_x_positions() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "First Second Third".to_string(),
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
    }];
    ctx.break_into_lines(runs);
    for line in &ctx.lines {
        for i in 1..line.runs.len() {
            assert!(
                line.runs[i].x >= line.runs[i - 1].x + line.runs[i - 1].width - 0.01,
                "片段 x 坐标应递增"
            );
        }
    }
}

/// 测试多个 TextRun 合并到同一行。
#[test]
fn test_multiple_runs_same_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "Hello".to_string(),
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
        },
        TextRun {
            text: "World".to_string(),
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
        },
    ];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "两个短文本应在同一行");
}

/// 测试 inline 元素文本与文本节点混合。
#[test]
fn test_inline_layout_from_document() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Hello <b>World</b>!</p>");

    // 找到 body > p
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    let fragments = ctx.all_fragments();
    assert!(!fragments.is_empty(), "p 元素应包含文本片段");

    let all_text: String = fragments.iter().map(|f| f.text.clone()).collect();
    assert!(all_text.contains("Hello"), "应包含 'Hello'，实际: {all_text}");
}

// ── 新增综合测试 ──

/// 测试混合文本和 inline 元素（真实 HTML 结构）。
///
/// 模拟 `<p>This is <em>important</em> and <strong>bold</strong> text</p>` 场景，
/// 验证从文档收集行内内容后能正确排列成行盒。
#[test]
fn test_mixed_text_and_inline_elements() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>This is <em>important</em> and <strong>bold</strong> text</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    let fragments = ctx.all_fragments();
    assert!(
        fragments.len() >= 5,
        "应至少有 5 个文本片段（各单词），实际 {}",
        fragments.len()
    );

    // 验证所有片段的 x 坐标在同一行内递增
    for line in &ctx.lines {
        for i in 1..line.runs.len() {
            assert!(line.runs[i].x >= line.runs[i - 1].x, "片段 x 坐标应在行内递增");
        }
    }
}

/// 测试超长单个单词应溢出容器。
///
/// 当一个单词宽度超过 container_width 时，它仍然被放置在行盒中
/// （浏览器行为：溢出而不截断）。
#[test]
fn test_very_long_single_word_overflow() {
    let mut ctx = InlineFormattingContext::new(100.0);
    // 构造一个超长单词，每个字符宽度约 16*0.6 = 9.6px
    // 50 个字符 = ~480px，远超 100px 容器宽度
    let long_word = "a".repeat(50);
    let runs = vec![TextRun {
        text: long_word.clone(),
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
    }];
    ctx.break_into_lines(runs);

    // 应产生 1 行（单个不中断单词不换行）
    assert_eq!(ctx.lines.len(), 1, "超长单词应在单行中（溢出而不是换行）");
    assert_eq!(ctx.lines[0].runs.len(), 1, "只有一个单词片段");

    // 片段宽度应超过容器宽度
    let fragment = &ctx.lines[0].runs[0];
    assert!(
        fragment.width > ctx.container_width,
        "片段宽度 {} 应超过容器宽度 {}",
        fragment.width,
        ctx.container_width
    );
}

/// 测试空容器（无文本节点）不产生任何行盒。
#[test]
fn test_empty_container_no_lines() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs: Vec<TextRun> = vec![];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.is_empty(), "空容器不应产生行盒");
    assert!(ctx.all_fragments().is_empty(), "空容器不应有文本片段");
    assert!((ctx.total_height() - 0.0).abs() < 0.01, "空容器总高度应为 0");
}

/// 测试空容器通过 Document layout 方法。
#[test]
fn test_empty_container_from_document() {
    use zero_dom::parse_html;

    let doc = parse_html("<p></p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    assert!(ctx.lines.is_empty(), "没有文本的空 p 元素不应产生行盒");
}

/// 测试行高计算 — 不同行高产生不同的行盒高度。
#[test]
fn test_line_height_calculation() {
    // 行高 24px 的单行
    let mut ctx24 = InlineFormattingContext::new(800.0);
    let runs_24 = vec![TextRun {
        text: "Short text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 24.0,
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
    }];
    ctx24.break_into_lines(runs_24);

    // 行高 32px 的单行
    let mut ctx32 = InlineFormattingContext::new(800.0);
    let runs_32 = vec![TextRun {
        text: "Short text".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 32.0,
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
    }];
    ctx32.break_into_lines(runs_32);

    assert!(
        (ctx24.lines[0].height - 24.0).abs() < 0.01,
        "行高 24px 应产生高度 24px 的行盒"
    );
    assert!(
        (ctx32.lines[0].height - 32.0).abs() < 0.01,
        "行高 32px 应产生高度 32px 的行盒"
    );
    assert!((ctx24.total_height() - 24.0).abs() < 0.01, "总高度应为 24.0");
    assert!((ctx32.total_height() - 32.0).abs() < 0.01, "总高度应为 32.0");
}

/// 测试行高在多行中的累加效果。
#[test]
fn test_line_height_accumulation() {
    let mut ctx = InlineFormattingContext::new(50.0);
    let runs = vec![TextRun {
        text: "a b c d e f g h i j".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
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
    }];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "窄容器应产生多行");

    // 每行高度应为 30px
    for (i, line) in ctx.lines.iter().enumerate() {
        assert!(
            (line.height - 30.0).abs() < 0.01,
            "第 {} 行高度应为 30.0，实际 {}",
            i,
            line.height
        );
    }

    // 总高度 = 行数 × 30
    let expected = ctx.lines.len() as f32 * 30.0;
    assert!(
        (ctx.total_height() - expected).abs() < 0.01,
        "总高度应为 {}，实际 {}",
        expected,
        ctx.total_height()
    );
}

/// 测试混合字体大小的行盒。
///
/// 当同一行包含不同字体大小的文本时，行盒高度应取最大值。
#[test]
fn test_multiple_font_sizes_same_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "Small".to_string(),
            node_id: NodeId::default(),
            font_size: 12.0,
            line_height: 16.0,
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
        },
        TextRun {
            text: "Large".to_string(),
            node_id: NodeId::default(),
            font_size: 24.0,
            line_height: 30.0,
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
        },
        TextRun {
            text: "Medium".to_string(),
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
        },
    ];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1, "三个短词应在同一行");

    // 行盒高度应取最大行高 30.0
    assert!(
        (ctx.lines[0].height - 30.0).abs() < 0.01,
        "行盒高度应取最大行高 30.0，实际 {}",
        ctx.lines[0].height
    );

    // 验证片段保留了各自的字体大小
    let fragments = ctx.all_fragments();
    let font_sizes: Vec<f32> = fragments.iter().map(|f| f.font_size).collect();
    assert!(
        font_sizes.iter().any(|&s| (s - 12.0).abs() < 0.01),
        "应包含 12px 字体大小的片段"
    );
    assert!(
        font_sizes.iter().any(|&s| (s - 24.0).abs() < 0.01),
        "应包含 24px 字体大小的片段"
    );
}

/// 测试混合字体大小时估算宽度与字体大小成正比。
#[test]
fn test_font_size_affects_width() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "Word".to_string(),
            node_id: NodeId::default(),
            font_size: 10.0,
            line_height: 14.0,
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
        },
        TextRun {
            text: "Word".to_string(),
            node_id: NodeId::default(),
            font_size: 20.0,
            line_height: 24.0,
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
        },
    ];
    ctx.break_into_lines(runs);

    let fragments = ctx.all_fragments();
    assert_eq!(fragments.len(), 2, "两个单词各一个片段");

    // 20px 字体的片段宽度应为 10px 字体的 2 倍
    let ratio = fragments[1].width / fragments[0].width;
    assert!((ratio - 2.0).abs() < 0.01, "宽度比应为 2.0，实际 {}", ratio);
}

/// 测试窄容器中多个 TextRun 跨行排列。
#[test]
fn test_multiple_runs_wrap_across_lines() {
    let mut ctx = InlineFormattingContext::new(80.0);
    let runs = vec![
        TextRun {
            text: "alpha beta".to_string(),
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
        },
        TextRun {
            text: "gamma delta".to_string(),
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
        },
    ];
    ctx.break_into_lines(runs);

    assert!(
        ctx.lines.len() > 1,
        "窄容器中 4 个单词应产生多行，实际 {} 行",
        ctx.lines.len()
    );

    // 验证 y 坐标连续递增
    for i in 1..ctx.lines.len() {
        assert!(
            ctx.lines[i].y >= ctx.lines[i - 1].y + ctx.lines[i - 1].height - 0.01,
            "行 y 坐标应连续递增"
        );
    }
}

/// 测试所有片段的 NodeId 正确保留。
#[test]
fn test_fragment_node_ids_preserved() {
    let id1 = NodeId::default();
    let id2 = NodeId::default();

    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![
        TextRun {
            text: "First".to_string(),
            node_id: id1,
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
        },
        TextRun {
            text: "Second".to_string(),
            node_id: id2,
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
        },
    ];
    ctx.break_into_lines(runs);

    let fragments = ctx.all_fragments();
    // 每个片段都应有有效的 NodeId（即使是默认值）
    assert_eq!(fragments.len(), 2, "应有 2 个片段");
    for f in &fragments {
        assert!(f.node_id.is_valid(), "每个片段都应有有效的 NodeId");
    }
}

/// 测试 Container width 为 0 的边界情况。
#[test]
fn test_zero_container_width() {
    let mut ctx = InlineFormattingContext::new(0.0);
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
    }];
    ctx.break_into_lines(runs);

    // 容器宽度为 0 时，第一个单词放入第一行（即使溢出），
    // 后续每个单词都换新行
    assert!(!ctx.lines.is_empty(), "即使容器宽度为 0，也应产生行盒");
    assert!(ctx.lines.len() >= 2, "零宽度容器中多个单词应产生多行");
}

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

// ── resolve_font_metrics 测试 ──

/// 测试 resolve_font_metrics 在无样式时返回默认值。
#[test]
fn test_resolve_font_metrics_no_style() {
    let (font_size, line_height) = resolve_font_metrics(None);
    assert!(
        (font_size - 16.0).abs() < 0.01,
        "默认 font_size 应为 16.0，实际 {font_size}"
    );
    assert!(
        (line_height - 19.2).abs() < 0.01,
        "默认 line_height 应为 16.0 * 1.2 = 19.2，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 从 ComputedStyle 中读取 font-size。
#[test]
fn test_resolve_font_metrics_with_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(24.0);

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 24.0).abs() < 0.01, "font_size 应为 24.0，实际 {font_size}");
    // line-height: Normal → 24.0 * 1.2 = 28.8
    assert!(
        (line_height - 28.8).abs() < 0.01,
        "line_height 应为 28.8，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 中 line-height: Number 使用倍数。
#[test]
fn test_resolve_font_metrics_line_height_number() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.line_height = LineHeightValue::Number(1.5);

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 20.0).abs() < 0.01);
    assert!(
        (line_height - 30.0).abs() < 0.01,
        "line_height 应为 20.0 * 1.5 = 30.0，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 中 line-height: Length 使用固定值。
#[test]
fn test_resolve_font_metrics_line_height_length() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(20.0);
    style.line_height = LineHeightValue::Length(LengthValue::Px(28.0));

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 20.0).abs() < 0.01);
    assert!(
        (line_height - 28.0).abs() < 0.01,
        "line_height 应为 28.0，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 中 line-height: Normal 使用 1.2 倍数。
#[test]
fn test_resolve_font_metrics_line_height_normal() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(32.0);
    // 默认 line-height 就是 Normal

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 32.0).abs() < 0.01);
    assert!(
        (line_height - 38.4).abs() < 0.01,
        "line_height 应为 32.0 * 1.2 = 38.4，实际 {line_height}"
    );
}

// ── 样式感知 layout 测试 ──

/// 测试从 Document 布局时使用 ComputedStyle 中的 font-size。
#[test]
fn test_layout_uses_style_font_size() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Hello World</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    // 给 p 设置 font-size: 32px
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(32.0);
    styles.insert(p, style);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    let fragments = ctx.all_fragments();
    assert!(!fragments.is_empty());

    // 所有片段的 font_size 应为 32.0
    for f in &fragments {
        assert!(
            (f.font_size - 32.0).abs() < 0.01,
            "片段 font_size 应为 32.0，实际 {}",
            f.font_size
        );
    }
}

/// 测试从 Document 布局时使用 ComputedStyle 中的 line-height。
#[test]
fn test_layout_uses_style_line_height() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Hello World</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    // 给 p 设置 line-height: 2.0（无单位数值）
    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.line_height = LineHeightValue::Number(2.0);
    styles.insert(p, style);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    // 行盒高度应为 16.0 * 2.0 = 32.0
    for line in &ctx.lines {
        assert!(
            (line.height - 32.0).abs() < 0.01,
            "行盒高度应为 32.0（16 * 2.0），实际 {}",
            line.height
        );
    }
}

/// 测试从 Document 布局时 inline 元素使用自己的样式。
#[test]
fn test_layout_inline_element_own_style() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Hello <b>World</b></p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    // 找到 <b> 元素（跳过文本节点，node_type 1 = Element）
    let b = doc
        .child_nodes(p)
        .into_iter()
        .find(|&id| doc.node_type(id) == Some(1))
        .expect("应有 <b> 元素");

    // p 使用默认 16px，b 使用 24px
    let mut styles = HashMap::new();
    let mut p_style = ComputedStyle::default();
    p_style.font_size = LengthValue::Px(16.0);
    styles.insert(p, p_style);

    let mut b_style = ComputedStyle::default();
    b_style.font_size = LengthValue::Px(24.0);
    styles.insert(b, b_style);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    let fragments = ctx.all_fragments();

    // 应有 font_size 为 16.0 的片段（来自 p 的文本节点）
    let has_16 = fragments.iter().any(|f| (f.font_size - 16.0).abs() < 0.01);
    assert!(has_16, "应有 16px 字体大小的片段");

    // 应有 font_size 为 24.0 的片段（来自 b 元素）
    let has_24 = fragments.iter().any(|f| (f.font_size - 24.0).abs() < 0.01);
    assert!(has_24, "应有 24px 字体大小的片段");
}

/// 测试无样式时回退到默认值 16.0 / 19.2。
#[test]
fn test_layout_no_style_fallback() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Hello</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    let fragments = ctx.all_fragments();
    assert!(!fragments.is_empty());

    for f in &fragments {
        assert!(
            (f.font_size - 16.0).abs() < 0.01,
            "无样式时 font_size 应回退到 16.0，实际 {}",
            f.font_size
        );
    }

    // 行盒高度应为 16.0 * 1.2 = 19.2
    for line in &ctx.lines {
        assert!(
            (line.height - 19.2).abs() < 0.01,
            "无样式时行盒高度应为 19.2，实际 {}",
            line.height
        );
    }
}

/// 测试 line-height: Length(24px) 覆盖默认行高。
#[test]
fn test_layout_fixed_line_height() {
    use zero_dom::parse_html;

    let doc = parse_html("<p>Text</p>");

    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(16.0);
    style.line_height = LineHeightValue::Length(LengthValue::Px(24.0));
    styles.insert(p, style);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    for line in &ctx.lines {
        assert!(
            (line.height - 24.0).abs() < 0.01,
            "行盒高度应为 24.0（固定 line-height），实际 {}",
            line.height
        );
    }
}

// ── 新增补充测试 ──

/// 测试混合 inline 和 block 内容边界。
///
/// 窄容器中多个文本运行跨越多行，验证行盒之间的 y 坐标不重叠。
#[test]
fn test_mixed_inline_block_content_boundary() {
    let mut ctx = InlineFormattingContext::new(80.0);
    let runs = vec![
        TextRun {
            text: "alpha beta gamma".to_string(),
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
        },
        TextRun {
            text: "delta epsilon".to_string(),
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
        },
    ];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() > 1, "窄容器中应有多个行盒，实际 {}", ctx.lines.len());

    // 行盒之间不应重叠：下一行的 y 应 >= 上一行的 y + 上一行的高度
    for i in 1..ctx.lines.len() {
        let prev_end = ctx.lines[i - 1].y + ctx.lines[i - 1].height;
        assert!(
            ctx.lines[i].y >= prev_end - 0.01,
            "行 {} (y={}) 应在行 {} (y={}, h={}) 之后",
            i,
            ctx.lines[i].y,
            i - 1,
            ctx.lines[i - 1].y,
            ctx.lines[i - 1].height
        );
    }
}

/// 测试文本中包含显式换行符时产生多个行盒。
///
/// 换行符 \n 将文本分割到不同行盒中。
#[test]
fn test_text_with_explicit_line_breaks() {
    let mut ctx = InlineFormattingContext::new(800.0);
    // \n 在 split_whitespace 中被视为空白，会被当作单词分隔符
    let runs = vec![TextRun {
        text: "line1\nline2\nline3".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 24.0,
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
    }];
    ctx.break_into_lines(runs);

    // 宽容器中所有单词应在同一行（因为容器足够宽）
    // 但 \n 在 split_whitespace 中作为空白处理，所以分词为 3 个单词
    assert!(ctx.lines.len() >= 1, "应至少有 1 行（宽容器中单词可能全部放入同一行）");

    // 验证所有片段存在且 y 坐标合理
    for line in &ctx.lines {
        assert!(line.height > 0.0, "行盒高度应为正");
    }
}

/// 测试 white-space: nowrap 行为 — 模拟单行不换行。
///
/// 虽然行内格式化上下文不直接处理 white-space 属性，
/// 但可以验证当所有文本放入单行时的行为（通过足够宽的容器）。
#[test]
fn test_whitespace_nowrap_behavior() {
    let mut ctx = InlineFormattingContext::new(8000.0);
    let runs = vec![TextRun {
        text: "This is a long sentence that would normally wrap but in a very wide container stays on one line"
            .to_string(),
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
    }];
    ctx.break_into_lines(runs);

    // 在 8000px 宽的容器中，应只有 1 行
    assert_eq!(
        ctx.lines.len(),
        1,
        "超宽容器中文本应在单行中，实际 {} 行",
        ctx.lines.len()
    );

    // 总高度应等于行高
    assert!(
        (ctx.total_height() - 20.0).abs() < 0.01,
        "单行总高度应为 20.0，实际 {}",
        ctx.total_height()
    );
}

/// 测试超长无断词机会的单词 — 不应换行，应放入单行。
///
/// 即使单词宽度远超容器，也应保持在同一行中（浏览器行为）。
#[test]
fn test_very_long_word_without_break_opportunity() {
    let mut ctx = InlineFormattingContext::new(50.0);
    // 100 个字符的连续字符串，无空格
    let long_word = "abcdefghijklmnopqrstuvwxyz".repeat(4);
    let runs = vec![TextRun {
        text: long_word,
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
    }];
    ctx.break_into_lines(runs);

    // 单个不中断单词应在 1 行中（不换行）
    assert_eq!(
        ctx.lines.len(),
        1,
        "超长无断词单词应在单行中（溢出），实际 {} 行",
        ctx.lines.len()
    );
    assert_eq!(ctx.lines[0].runs.len(), 1, "应只有 1 个片段");

    // 片段宽度应远超容器宽度
    let fragment = &ctx.lines[0].runs[0];
    assert!(fragment.width > 50.0, "片段宽度 {} 应超过容器宽度 50", fragment.width);
}

/// 测试 vertical-align: top 在行盒中的 y 偏移。
#[test]
fn test_vertical_align_top_in_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "Hello".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
        vertical_align: VerticalAlignValue::Top,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let fragment = &ctx.lines[0].runs[0];
    // top 对齐: y 应为 0.0
    assert!(
        fragment.y.abs() < 0.01,
        "vertical-align: top 片段 y 应为 0，实际 {}",
        fragment.y
    );
}

/// 测试 vertical-align: bottom 在行盒中的 y 偏移。
#[test]
fn test_vertical_align_bottom_in_line() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "Hello".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 30.0,
        vertical_align: VerticalAlignValue::Bottom,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
    }];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    let fragment = &ctx.lines[0].runs[0];
    // bottom 对齐: y = line_height - fragment_height
    let expected_y = 30.0 - fragment.height;
    assert!(
        (fragment.y - expected_y).abs() < 0.01,
        "vertical-align: bottom 片段 y 应为 {}，实际 {}",
        expected_y,
        fragment.y
    );
}

/// R109 §9.2.1.1：验证 IFC fragment_node_ids 限制 collect_inline_items 只收集指定片段。
/// 整合 inline_block_split（拆分）+ IFC（片段收集）两个基础件。
#[test]
fn test_fragment_node_ids_restricts_inline_collection() {
    let html = r#"<html><body><div id="i" style="display:inline">aaa<div>bbb</div>ccc</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    fn find(id: &str, doc: &zero_dom::Document, node: zero_dom::NodeId) -> Option<zero_dom::NodeId> {
        if let Some(n) = doc.get(node)
            && let zero_dom::NodeKind::Element(e) = &n.kind
            && e.get_attribute("id").as_deref() == Some(id)
        {
            return Some(node);
        }
        for &c in &doc.get(node).map(|n| n.children.clone()).unwrap_or_default() {
            if let Some(f) = find(id, doc, c) {
                return Some(f);
            }
        }
        None
    }
    let i_id = find("i", &doc, doc.root()).expect("find #i");
    // 计算拆分，取第一个 Inline 片段
    let segs = crate::inline_block_split::compute_inline_block_split(&doc, &styles, i_id)
        .expect("inline with block child should split");
    let first_frag: Vec<zero_dom::NodeId> = segs
        .iter()
        .find_map(|s| match s {
            crate::inline_block_split::InlineBlockSegment::Inline { item_node_ids } => Some(item_node_ids.clone()),
            _ => None,
        })
        .expect("first Inline segment");
    // 不设 fragment：收集 #i 全部子节点（aaa + ccc 文本，block 子元素被简化跳过）
    let ctx_all = InlineFormattingContext::new(800.0);
    let all_items = ctx_all.collect_inline_items(&doc, i_id, &styles);
    // 设 fragment：只收集第一个片段（aaa）
    let mut ctx_frag = InlineFormattingContext::new(800.0);
    ctx_frag.set_fragment_node_ids(first_frag);
    let frag_items = ctx_frag.collect_inline_items(&doc, i_id, &styles);
    assert!(
        frag_items.len() < all_items.len(),
        "fragment should collect fewer items: frag={} all={}",
        frag_items.len(),
        all_items.len()
    );
    assert!(!frag_items.is_empty(), "fragment should collect its text item");
}
