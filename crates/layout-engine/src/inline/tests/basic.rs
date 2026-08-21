// Auto-generated test file — split from layout-engine/inline.rs
use super::super::*;
use zero_style_system::TextAutospaceValue;

/// 测试文本分割为单词。
#[test]
fn test_split_into_words() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("Hello World Foo", false);
    assert_eq!(words.len(), 3);
    assert_eq!(words[0], "Hello ");
}

/// R645：SEA 词典分词文字（Thai/Lao/Myanmar/Khmer）分类正确。
#[test]
fn test_r645_sea_word_script_classification() {
    // Thai / Lao / Myanmar / Khmer 各取代表字符
    assert!(is_sea_word_script('\u{0E01}')); // Thai ก
    assert!(is_sea_word_script('\u{0E81}')); // Lao ກ
    assert!(is_sea_word_script('\u{1000}')); // Myanmar ႀ
    assert!(is_sea_word_script('\u{1780}')); // Khmer ក
    // 非 SEA：ASCII / CJK / 其他文字
    assert!(!is_sea_word_script('a'));
    assert!(!is_sea_word_script(' '));
    assert!(!is_sea_word_script('\u{4E00}')); // CJK
    // 组合判定：CJK 与 SEA 都允许 per-char 断行
    assert!(is_per_char_break_script('\u{4E00}')); // CJK
    assert!(is_per_char_break_script('\u{0E01}')); // SEA
    assert!(!is_per_char_break_script('a')); // 拉丁字符不 per-char 断行
}

/// R645：SEA 文字在 normal 模式下按字符断行（fallback line breaking）。
/// CSS Text 3 §line-break-details：无空格分词的 SEA 文字须 fallback 断行（不允许溢出）。
/// 验证：连续 Thai 文本被拆成单字符"单词"，从而允许在任意字符间断行。
#[test]
fn test_r645_sea_text_per_char_break() {
    let ctx = InlineFormattingContext::new(96.0); // 6em 容器
    // 3 个 Thai 字符（无空格）——normal 模式应拆成 3 个独立单词（每个 1 字符）
    let words = ctx.split_into_words("\u{0E21}\u{0E19}\u{0E38}", false);
    assert_eq!(words.len(), 3, "SEA 文本应按字符拆为独立断行点");
    for w in &words {
        // 每个单词恰好 1 个非空白字符（末尾可能带词间距空格，同 test_split_into_words）
        let non_ws: Vec<char> = w.chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(non_ws.len(), 1, "每个 SEA 单词为单字符：{w:?}");
    }
}

/// R1214：is_ahem（cjk_contiguous）时 CJK per-char 拆分不加词间空格（连续）。
/// 旧 post-loop 给同一 step-1 词内 CJK per-char 也加 1em 词间距 → Ahem CJK 2em 发散
///（text-autospace ideograph-numeric/alpha-001）。CJK 每字符可断行不意味着产生空格；
/// 普通字体的连续模式由独立 gate 隔离，避免 layout/paint advance 未同源时扩大回归。
#[test]
fn test_r1214_cjk_per_char_contiguous_when_ahem() {
    let ctx = InlineFormattingContext::new(800.0);
    // is_ahem=true：同一 step-1 词内 CJK per-char 连续无词间空格
    let words = ctx.split_into_words("4水水", true);
    assert_eq!(words, vec!["4".to_string(), "水".to_string(), "水".to_string()]);
    // 真实空格处仍加尾部空格（word-spacing advance）
    let words2 = ctx.split_into_words("hello 水", true);
    assert_eq!(words2, vec!["hello ".to_string(), "水".to_string()]);
    // CJK-Latin-CJK 交替（无空格）→ 全连续
    let words3 = ctx.split_into_words("水4水", true);
    assert_eq!(words3, vec!["水".to_string(), "4".to_string(), "水".to_string()]);
    // 真实空格分隔的两个 CJK 词：第一个词末尾加空格
    let words4 = ctx.split_into_words("水 水", true);
    assert_eq!(words4, vec!["水 ".to_string(), "水".to_string()]);
    // 普通字体默认保留旧 advance；连续模式仍可通过纯 helper 独立验证。
    let words5 = ctx.split_into_words("4水水", false);
    assert_eq!(words5, vec!["4 ".to_string(), "水 ".to_string(), "水".to_string()]);
    let words6 = ctx.collapse_split_words_with_mode("姓名 ", true);
    assert_eq!(words6, vec!["姓".to_string(), "名 ".to_string()]);
}

/// R1215：text-autospace 字符类别判定。
#[test]
fn test_r1215_char_autospace_category() {
    assert_eq!(char_autospace_category('国'), AutospaceCategory::Ideograph);
    assert_eq!(char_autospace_category('水'), AutospaceCategory::Ideograph);
    assert_eq!(char_autospace_category('X'), AutospaceCategory::Letter);
    assert_eq!(char_autospace_category('a'), AutospaceCategory::Letter);
    assert_eq!(char_autospace_category('4'), AutospaceCategory::Numeric);
    assert_eq!(char_autospace_category('0'), AutospaceCategory::Numeric);
    // 标点。不在 ideograph 类别（不触发 autospacing）
    assert_eq!(char_autospace_category('。'), AutospaceCategory::Other);
    assert_eq!(char_autospace_category(' '), AutospaceCategory::Other);
    assert_eq!(char_autospace_category('、'), AutospaceCategory::Other);
}

/// R1215：autospace_gap_for 在 ideograph↔letter/numeric 边界返回 0.125em，标点/同类别返回 0。
#[test]
fn test_r1215_autospace_gap_for_boundaries() {
    let fs = 40.0;
    let normal = TextAutospaceValue::Normal;
    let none = TextAutospaceValue::NoAutospace;
    let alpha_only = TextAutospaceValue::IdeographAlpha;
    let numeric_only = TextAutospaceValue::IdeographNumeric;
    // ideograph-alpha（Normal 启用）：国→X / X→国 = 5px
    assert!((autospace_gap_for('国', 'X', normal, fs) - 5.0).abs() < 0.01);
    assert!((autospace_gap_for('X', '国', normal, fs) - 5.0).abs() < 0.01);
    // ideograph-numeric（Normal 启用）：4→水 / 水→4 = 5px
    assert!((autospace_gap_for('4', '水', normal, fs) - 5.0).abs() < 0.01);
    assert!((autospace_gap_for('水', '4', normal, fs) - 5.0).abs() < 0.01);
    // 同类别（国→国）、标点边界（。→X / X→。）：0
    assert!((autospace_gap_for('国', '国', normal, fs)).abs() < 0.01);
    assert!((autospace_gap_for('。', 'X', normal, fs)).abs() < 0.01);
    assert!((autospace_gap_for('X', '。', normal, fs)).abs() < 0.01);
    // NoAutospace：全 0
    assert!((autospace_gap_for('国', 'X', none, fs)).abs() < 0.01);
    // ideograph-alpha only：国→4 = 0（numeric 未启），国→X = 5
    assert!((autospace_gap_for('国', '4', alpha_only, fs)).abs() < 0.01);
    assert!((autospace_gap_for('国', 'X', alpha_only, fs) - 5.0).abs() < 0.01);
    // ideograph-numeric only：国→X = 0（alpha 未启），国→4 = 5
    assert!((autospace_gap_for('国', 'X', numeric_only, fs)).abs() < 0.01);
    assert!((autospace_gap_for('国', '4', numeric_only, fs) - 5.0).abs() < 0.01);
}

/// R1215：text-autospace: normal 在 "国国XX国" 的 ideograph↔letter 边界插 0.125em 间隙。
/// "国国XX国" is_ahem → R1214 split ["国","国","XX","国"] contiguous；normal → 国→XX 与
/// XX→国 各插 5px（0.125em@40px）。期望 frag x：国@0, 国@40, XX@85, 国@170。
#[test]
fn test_r1215_text_autospace_normal_applies_gaps() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_autospace(TextAutospaceValue::Normal);
    let runs = vec![TextRun {
        text: "国国XX国".to_string(),
        node_id: NodeId::default(),
        font_size: 40.0,
        line_height: 40.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);
    let frags = ctx.all_fragments();
    assert_eq!(frags.len(), 4, "4 fragments: 国 国 XX 国");
    // 国@0, 国@40（CJK-CJK 无 gap）
    assert!((frags[0].x - 0.0).abs() < 0.5, "frag0 国 @0, got {}", frags[0].x);
    assert!((frags[1].x - 40.0).abs() < 0.5, "frag1 国 @40, got {}", frags[1].x);
    // XX@85（国@40 + 国宽 40 + 5px autospacing）
    assert!((frags[2].x - 85.0).abs() < 0.5, "frag2 XX @85, got {}", frags[2].x);
    // 国@170（XX@85 + XX宽 80 + 5px autospacing）
    assert!((frags[3].x - 170.0).abs() < 0.5, "frag3 国 @170, got {}", frags[3].x);
}

/// R1215：text-autospace: no-autospace（默认）不插间隙——"国国XX国" 连续 1em（R1214 行为不变）。
#[test]
fn test_r1215_text_autospace_no_autospace_no_gap() {
    let mut ctx = InlineFormattingContext::new(800.0).with_text_autospace(TextAutospaceValue::NoAutospace);
    let runs = vec![TextRun {
        text: "国国XX国".to_string(),
        node_id: NodeId::default(),
        font_size: 40.0,
        line_height: 40.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);
    let frags = ctx.all_fragments();
    assert_eq!(frags.len(), 4);
    // 无 autospacing：国@0, 国@40, XX@80, 国@160（纯 1em 连续）
    assert!(
        (frags[2].x - 80.0).abs() < 0.5,
        "frag2 XX @80 (no gap), got {}",
        frags[2].x
    );
    assert!(
        (frags[3].x - 160.0).abs() < 0.5,
        "frag3 国 @160 (no gap), got {}",
        frags[3].x
    );
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);
    let fragments: Vec<_> = ctx.all_fragments();
    assert!(fragments.is_empty());
}

/// R1086: word-spacing 作为词间前导间隙——非首词的 fragment.x 必须含 word_spacing gap。
/// 旧实现把 word_spacing 计入 word_width → 仅推进 current_x 给下一词，本词 fragment.x 缺 gap
///（word-spacing-007 第二 x 落在无 gap 处）。Ahem 'x'=16px、space=16px、word_spacing=96px
/// → 第二词应在 16(x)+16(space)+96(ws)=128px。
#[test]
fn test_r1086_word_spacing_applied_to_position() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun {
        text: "x x".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 96.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: true,
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "应在单行");
    assert_eq!(ctx.lines[0].runs.len(), 2, "两个单词");
    let gap = ctx.lines[0].runs[1].x - ctx.lines[0].runs[0].x;
    // 第二词位移 = 首词宽(16) + 空格(16) + word_spacing(96) = 128；旧 bug 下 ~32（无 gap）。
    assert!(
        gap >= 110.0,
        "word_spacing 应作前导间隙，gap={gap} 应 ~128（含 96px word_spacing）"
    );
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
    }];
    ctx.break_into_lines(runs);

    // 容器宽度为 0 时，第一个单词放入第一行（即使溢出），
    // 后续每个单词都换新行
    assert!(!ctx.lines.is_empty(), "即使容器宽度为 0，也应产生行盒");
    assert!(ctx.lines.len() >= 2, "零宽度容器中多个单词应产生多行");
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
        (line_height - 18.624).abs() < 0.01,
        "默认 line_height 应为 16.0 * 1.164 = 18.624，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 从 ComputedStyle 中读取 font-size。
#[test]
fn test_resolve_font_metrics_with_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(24.0);

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 24.0).abs() < 0.01, "font_size 应为 24.0，实际 {font_size}");
    // line-height: Normal → 24.0 * 1.164 = 27.936
    assert!(
        (line_height - 27.936).abs() < 0.01,
        "line_height 应为 28.8，实际 {line_height}"
    );
}

/// R3622：IFC font metrics 的 font-size used value 也要解析 residual real length。
/// direct `ComputedStyle` 下 `font-size:2em` 应等价 32px；旧逻辑只接受 `Px`，会回退 16px。
#[test]
fn r3622_resolve_font_metrics_resolves_relative_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Em(2.0);

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!(
        (font_size - 32.0).abs() < 0.01,
        "font_size 应解析 2em@16px 为 32，实际 {font_size}"
    );
    assert!(
        (line_height - 37.248).abs() < 0.01,
        "line_height 应使用 resolved font_size 32 * 1.164 = 37.248，实际 {line_height}"
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
        (line_height - 37.248).abs() < 0.01,
        "line_height 应为 32.0 * 1.164 = 37.248，实际 {line_height}"
    );
}

/// 测试 resolve_font_metrics 中 Ahem 字体 line-height: Normal 使用字体实际度量比率 1.0
/// （非默认 1.2）—— Chromium 对 line-height:normal 用字体 OS/2 度量，Ahem 度量=1.0。
#[test]
fn test_resolve_font_metrics_line_height_normal_ahem() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(32.0);
    style.font_family = vec!["Ahem".to_string()];
    // 默认 line-height 就是 Normal

    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 32.0).abs() < 0.01);
    assert!(
        (line_height - 32.0).abs() < 0.01,
        "Ahem line-height:normal 应为 32.0 * 1.0 = 32.0（字体实际度量），实际 {line_height}"
    );

    // font-family 列表中含 Ahem（多字体回退）也应触发
    style.font_family = vec!["serif".to_string(), "Ahem".to_string()];
    let (_, line_height) = resolve_font_metrics(Some(&style));
    assert!(
        (line_height - 32.0).abs() < 0.01,
        "含 Ahem 的 font-family 列表 line-height:normal 应为 32.0，实际 {line_height}"
    );

    // 大小写不敏感
    style.font_family = vec!["ahem".to_string()];
    let (_, line_height) = resolve_font_metrics(Some(&style));
    assert!(
        (line_height - 32.0).abs() < 0.01,
        "大小写不敏感：ahem line-height:normal 应为 32.0，实际 {line_height}"
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

    // 行盒高度应为 16.0 * 1.164 = 18.624
    for line in &ctx.lines {
        assert!(
            (line.height - 18.624).abs() < 0.01,
            "无样式时行盒高度应为 18.624，实际 {}",
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
            font_id: None,
            is_rtl: false,
            is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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
        font_id: None,
        is_rtl: false,
        is_plaintext_bidi: false,
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

#[test]
fn r3634_inline_text_margin_resolves_residual_real_length() {
    use zero_css_parser::values::{DisplayValue, LengthValue};
    use zero_style_system::ComputedStyle;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    let text = doc.create_text_node("x");
    doc.append_child(root, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut styles = std::collections::HashMap::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Inline;
    style.font_size = LengthValue::Px(20.0);
    style.margin_left = LengthValue::Em(2.0);
    style.margin_right = LengthValue::Ch(3.0);
    styles.insert(span, style);

    let ctx = InlineFormattingContext::new(800.0);
    let items = ctx.collect_inline_items(&doc, root, &styles);

    let Some(InlineItem::Text(run)) = items.first() else {
        panic!("expected inline text item, got {items:?}");
    };
    assert!(
        (run.margin_left - 40.0).abs() < 0.01,
        "margin-left:2em at 20px should resolve to 40px, got {}",
        run.margin_left
    );
    assert!(
        (run.margin_right - 30.0).abs() < 0.01,
        "margin-right:3ch at 20px should resolve to 30px, got {}",
        run.margin_right
    );
}

#[test]
fn r3634_inline_block_margin_resolves_residual_real_length() {
    use zero_css_parser::values::{DisplayValue, LengthValue};
    use zero_style_system::ComputedStyle;

    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    doc.append_child(root, span).unwrap();

    let mut styles = std::collections::HashMap::new();
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineBlock;
    style.font_size = LengthValue::Px(20.0);
    style.width = LengthValue::Px(10.0);
    style.height = LengthValue::Px(5.0);
    style.margin_top = LengthValue::Em(1.0);
    style.margin_right = LengthValue::Ch(2.0);
    style.margin_bottom = LengthValue::Em(0.5);
    style.margin_left = LengthValue::Em(1.5);
    styles.insert(span, style);

    let ctx = InlineFormattingContext::new(800.0);
    let items = ctx.collect_inline_items(&doc, root, &styles);

    let Some(InlineItem::InlineBlock(block)) = items.first() else {
        panic!("expected inline-block item, got {items:?}");
    };
    assert!((block.margin_top - 20.0).abs() < 0.01);
    assert!((block.margin_right - 20.0).abs() < 0.01);
    assert!((block.margin_bottom - 10.0).abs() < 0.01);
    assert!((block.margin_left - 30.0).abs() < 0.01);
    assert!(
        (block.baseline - 15.0).abs() < 0.01,
        "empty inline-block baseline should include resolved margin-bottom, got {}",
        block.baseline
    );
}

/// R816 linebox 度量统一 Phase 1：`break_into_lines` 后行盒的 baseline_y/ascent/descent
/// 必须被 `apply_vertical_alignment` 填充（非默认 0）。验证字段存储正确，供后续 Phase
/// paint 复用。
#[test]
fn r816_linebox_metrics_populated() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun::simple(
        "Hello".to_string(),
        NodeId::default(),
        20.0, // font-size
        24.0, // line-height
        zero_css_parser::values::VerticalAlignValue::Baseline,
    )];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1);
    let line = &ctx.lines[0];
    // ascent 应为正（half-leading + ascent），baseline_y == ascent
    assert!(line.ascent > 0.0, "ascent 应被填充，实际 {}", line.ascent);
    assert!(
        (line.baseline_y - line.ascent).abs() < 0.01,
        "baseline_y 应等于 ascent，实际 baseline_y={} ascent={}",
        line.baseline_y,
        line.ascent
    );
    // descent = height - ascent（含 half-leading 下半）
    assert!(
        (line.descent - (line.height - line.ascent)).abs() < 0.01,
        "descent 应 = height - ascent，实际 descent={} height={} ascent={}",
        line.descent,
        line.height,
        line.ascent
    );
    assert!(line.descent >= 0.0, "descent 非负");
}

/// R817 linebox 度量统一 Phase 2：验证 IFC 片段的 `is_ahem`（实际字体）按**每个片段**传播，
/// 而非容器级。这是 paint Phase 2 基线定位公式（仅对真正 Ahem 方块 `baseline_y - font_size`
/// 应用）所依赖的数据——Ahem 容器内的非 Ahem 片段（如 font-051 的 serif span）必须为 false，
/// 否则按 ascent=font_size 错移导致回归。
#[test]
fn r817_is_ahem_font_propagated_per_fragment() {
    let mut ctx = InlineFormattingContext::new(800.0);
    // 同一行的两个片段：一个真正 Ahem，一个 serif（同容器，不同实际字体）。
    let mut ahem_run = TextRun::simple(
        "AAAA".to_string(),
        NodeId::default(),
        40.0,
        130.0, // line-height 3.25
        zero_css_parser::values::VerticalAlignValue::Baseline,
    );
    ahem_run.is_ahem_font = true;
    let serif_run = TextRun::simple(
        "b".to_string(),
        NodeId::default(),
        16.0,
        19.2, // line-height 1.2
        zero_css_parser::values::VerticalAlignValue::Baseline,
    );
    // serif_run.is_ahem_font 保持 false（simple 默认）。
    ctx.break_into_lines(vec![ahem_run, serif_run]);
    assert_eq!(ctx.lines.len(), 1);
    let frags = &ctx.lines[0].runs;
    assert!(frags.len() >= 2, "应至少 2 片段，实际 {}", frags.len());
    // 每个片段的 is_ahem 反映其自身实际字体（run.is_ahem_font），非容器级。
    assert!(
        frags[0].is_ahem,
        "Ahem 片段 is_ahem 应为 true，实际 {}",
        frags[0].is_ahem
    );
    assert!(
        !frags.iter().any(|f| f.text == "b" && f.is_ahem),
        "serif 片段 is_ahem 应为 false（实际字体非 Ahem）"
    );

    // A3 不变量：line-height:1（run.height == font_size）时 Phase 2 v_offset 公式退化为 0。
    // v_offset = baseline_y_abs - font_size - frag.y；line-height:1 下 frag.y = baseline_y - font_size，
    // line.y=0 → baseline_y_abs = baseline_y → v_offset = baseline_y - font_size - (baseline_y - font_size) = 0。
    let mut single = InlineFormattingContext::new(800.0);
    let mut r = TextRun::simple(
        "X".to_string(),
        NodeId::default(),
        100.0,
        100.0, // line-height:1
        zero_css_parser::values::VerticalAlignValue::Baseline,
    );
    r.is_ahem_font = true;
    single.break_into_lines(vec![r]);
    let f = &single.lines[0].runs[0];
    let baseline_y = single.lines[0].baseline_y;
    let v_offset = baseline_y - f.font_size - f.y; // line.y = 0
    assert!(
        v_offset.abs() < 0.001,
        "line-height:1 时 Phase 2 v_offset 应退化为 0（A3），实际 {v_offset}"
    );
}

/// R822：line-box 高度 = strut ∪ valign 偏移 inline box（CSS §10.8.1）。text-bottom 把 inline
/// box 移到 strut 顶之上，line-box 须向上扩展（va-117a ZW line-box 130 而 REF 175）。验证一个含
/// text-bottom run 的行经 break_into_lines + apply_vertical_alignment 后 line.height > strut
/// line-height（扩展生效），且 baseline-aligned run 不触发扩展（line.height == line-height）。
#[test]
fn r822_linebox_grows_for_valign_extension() {
    // text-bottom 行：font 40px, line-height 130（half-leading 45）。text-bottom run 的 box
    // 越过 strut 顶 45px → line-box 应从 130 扩到 ~175。
    let mut ctx = InlineFormattingContext::new(800.0);
    let baseline_run = TextRun::simple(
        "TTTT".to_string(),
        NodeId::default(),
        40.0,
        130.0,
        zero_css_parser::values::VerticalAlignValue::Baseline,
    );
    let mut tb_run = TextRun::simple(
        "Above".to_string(),
        NodeId::default(),
        40.0,
        130.0,
        zero_css_parser::values::VerticalAlignValue::TextBottom,
    );
    tb_run.is_ahem_font = true;
    ctx.break_into_lines(vec![baseline_run, tb_run]);
    assert_eq!(ctx.lines.len(), 1);
    let line = &ctx.lines[0];
    assert!(
        line.height > 130.0 + 40.0,
        "text-bottom 行 line-box 应扩展超 130+40，实际 {}",
        line.height
    );
    assert!(
        line.height <= 130.0 + 45.0 + 1.0,
        "text-bottom 扩展量应≈half_leading 45，实际 height={}",
        line.height
    );
    // baseline_y 随 top_extend 下移（strut 在更高 line-box 内下移）。
    assert!(
        line.baseline_y > 77.0,
        "baseline_y 应随扩展下移，实际 {}",
        line.baseline_y
    );

    // 对照：纯 baseline 行不扩展。
    let mut ctx2 = InlineFormattingContext::new(800.0);
    ctx2.break_into_lines(vec![TextRun::simple(
        "X".to_string(),
        NodeId::default(),
        40.0,
        130.0,
        zero_css_parser::values::VerticalAlignValue::Baseline,
    )]);
    assert!(
        (ctx2.lines[0].height - 130.0).abs() < 0.5,
        "纯 baseline 行 line-box 应=strut 130 不扩展，实际 {}",
        ctx2.lines[0].height
    );
}

/// R1099 Slice α-1：`subtree_has_text_decoration` decoration-gate 正确性。
/// gate 决定 vertical 容器是否应用 container_width WM-aware fix（回避 Layer 4 装饰耦合）。
#[test]
fn test_r1099_subtree_has_text_decoration() {
    use crate::inline_finalization::subtree_has_text_decoration;
    use zero_dom::parse_html;
    use zero_style_system::property::types::TextDecorationLineValue;

    // 子树有 text-decoration（descendant span 设 underline）
    let doc = parse_html("<div><p>text</p></div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();
    let p = doc.first_child(div).unwrap();
    let mut styles = HashMap::new();
    let mut p_style = ComputedStyle::default();
    p_style.text_decoration_line = TextDecorationLineValue {
        underline: true,
        overline: false,
        line_through: false,
    };
    styles.insert(p, p_style);
    assert!(
        subtree_has_text_decoration(&doc, &styles, div),
        "子树含 underline 应返回 true"
    );

    // 子树无任何 text-decoration/emphasis
    let doc2 = parse_html("<div><p>text</p></div>");
    let html2 = doc2.first_child(doc2.root()).unwrap();
    let body2 = doc2.last_child(html2).unwrap();
    let div2 = doc2.first_child(body2).unwrap();
    let styles2 = HashMap::new(); // 全 default（text_decoration_line = None）
    assert!(
        !subtree_has_text_decoration(&doc2, &styles2, div2),
        "子树无装饰应返回 false"
    );
}

/// R1100 Slice α-2 验证：vertical-mode 下 col_width（= line.height）是否 = line-height × fs。
/// R1052 VIFCDUMP（pre-α-1 tree）实测 col_width=16（应 80，line-height:5 × fs16）。
/// 本测验证 current tree（post-α-1）run.line_height → col_width 链路是否正确。
#[test]
fn test_r1100_alpha2_vertical_line_height_column_width() {
    use zero_dom::parse_html;
    use zero_style_system::WritingModeValue;
    use zero_style_system::property::types::LineHeightValue;

    let doc = parse_html("<div>AAAAA</div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.writing_mode = WritingModeValue::VerticalRl;
    div_style.font_size = LengthValue::Px(16.0);
    div_style.line_height = LineHeightValue::Number(5.0); // line-height:5 → 80px
    styles.insert(div, div_style);

    // vertical: container_width 模拟 α-1 的 content_height（200px 可用深度）。
    let mut ctx = InlineFormattingContext::new(200.0).with_vertical(true);
    ctx.layout(&doc, div, &styles);

    assert!(!ctx.lines.is_empty(), "vertical IFC 应产出至少一列");
    let col_width = ctx.lines[0].height;
    assert!(
        (col_width - 80.0).abs() < 1.0,
        "vertical 列宽（line.height）应为 line-height:5 × fs16 = 80，实际 {} \
         （若 =16 则 line-height 未传 vertical 列宽，α-2 须修）",
        col_width
    );
}
