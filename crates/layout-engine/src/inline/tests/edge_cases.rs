// 边界条件和极端值测试 — inline 模块私有函数和公共函数。
use super::super::*;

// ── is_cjk_character 边界条件 ──

/// 测试 is_cjk_character：CJK 兼容表意文字（U+F900..U+FAFF）。
#[test]
fn test_is_cjk_compatibility_ideographs() {
    assert!(is_cjk_character('\u{F900}'), "U+F900 应为 CJK");
    assert!(is_cjk_character('\u{FAFF}'), "U+FAFF 应为 CJK");
    assert!(is_cjk_character('\u{F92F}'), "U+F92F 应为 CJK（兼容表意文字中间值）");
}

/// 测试 is_cjk_character：CJK 符号和标点（U+3000..U+303F）。
#[test]
fn test_is_cjk_symbols_and_punctuation() {
    assert!(is_cjk_character('\u{3000}'), "U+3000 全角空格应为 CJK");
    assert!(is_cjk_character('\u{3001}'), "U+3001 、应为 CJK");
    assert!(is_cjk_character('\u{3002}'), "U+3002 。应为 CJK");
    assert!(is_cjk_character('\u{303F}'), "U+303F 应为 CJK（符号末尾）");
}

/// 测试 is_cjk_character：CJK 部首补充（U+2E80..U+2EFF）。
#[test]
fn test_is_cjk_radicals_supplement() {
    assert!(is_cjk_character('\u{2E80}'), "U+2E80 应为 CJK（部首补充起始）");
    assert!(is_cjk_character('\u{2EFF}'), "U+2EFF 应为 CJK（部首补充末尾）");
}

/// 测试 is_cjk_character：CJK 基本区和扩展 A 之间的边界。
#[test]
fn test_is_cjk_boundary_between_extension_a_and_basic() {
    // U+4DBF 是扩展 A 的末尾
    assert!(is_cjk_character('\u{4DBF}'), "U+4DBF 扩展 A 末尾应为 CJK");
    // U+4E00 是基本区的起始
    assert!(is_cjk_character('\u{4E00}'), "U+4E00 基本区起始应为 CJK");
    // U+4DC0 不在任何范围内
    assert!(!is_cjk_character('\u{4DC0}'), "U+4DC0 不在 CJK 范围内");
}

/// 测试 is_cjk_character：拉丁扩展字符不为 CJK。
#[test]
fn test_is_cjk_latin_extended() {
    assert!(!is_cjk_character('\u{0100}'), "Ā 不应为 CJK");
    assert!(!is_cjk_character('\u{024F}'), "ɏ 不应为 CJK");
}

/// 测试 is_cjk_character：阿拉伯文字不为 CJK。
#[test]
fn test_is_cjk_arabic() {
    assert!(!is_cjk_character('\u{0627}'), "ا 不应为 CJK");
    assert!(!is_cjk_character('\u{0649}'), "ى 不应为 CJK");
}

/// 测试 is_cjk_character：西里尔字母不为 CJK。
#[test]
fn test_is_cjk_cyrillic() {
    assert!(!is_cjk_character('\u{0410}'), "А 不应为 CJK");
    assert!(!is_cjk_character('\u{044F}'), "я 不应为 CJK");
}

// ── estimate_char_width 边界条件 ──

/// 测试 estimate_char_width：数字宽度比例。
#[test]
fn test_estimate_char_width_digit_ratio() {
    let font_size = 16.0;
    let digit_width = estimate_char_width('5', font_size);
    let expected = font_size * 0.5;
    assert!(
        (digit_width - expected).abs() < 0.01,
        "数字 '5' 宽度应为 {}，实际 {}",
        expected,
        digit_width
    );
}

/// 测试 estimate_char_width：制表符按默认 Unicode 字符宽度计算。
#[test]
fn test_estimate_char_width_tab_character() {
    let font_size = 16.0;
    let tab_width = estimate_char_width('\t', font_size);
    // '\t' 不是 ASCII 空白字符中的空格，但 is_ascii_whitespace 返回 true
    assert!(
        (tab_width - font_size * 0.25).abs() < 0.01,
        "制表符应被视为空白字符，宽度应为 {}，实际 {}",
        font_size * 0.25,
        tab_width
    );
}

/// 测试 estimate_char_width：换行符按空白字符计算。
#[test]
fn test_estimate_char_width_newline_character() {
    let font_size = 16.0;
    let nl_width = estimate_char_width('\n', font_size);
    // '\n' 是 ASCII 空白字符
    assert!(
        (nl_width - font_size * 0.25).abs() < 0.01,
        "换行符应被视为空白字符，宽度应为 {}，实际 {}",
        font_size * 0.25,
        nl_width
    );
}

/// 测试 estimate_char_width：font_size 为负值时的行为。
#[test]
fn test_estimate_char_width_negative_font_size() {
    let width = estimate_char_width('A', -16.0);
    // 负 font_size 应产生负宽度，不 panic
    assert!(width < 0.0, "负 font_size 应产生负宽度，实际 {}", width);
}

/// 测试 estimate_char_width：Unicode 非 CJK 字符使用默认宽度。
#[test]
fn test_estimate_char_width_unicode_non_cjk() {
    let font_size = 16.0;
    // U+00E9 = é，既不是 ASCII 也不是 CJK
    let width = estimate_char_width('\u{00E9}', font_size);
    let expected = font_size * 0.5;
    assert!(
        (width - expected).abs() < 0.01,
        "Unicode 非 CJK 字符应使用默认宽度 {}，实际 {}",
        expected,
        width
    );
}

// ── estimate_string_width 边界条件 ──

/// 测试 estimate_string_width：包含多种字符类型的混合字符串。
#[test]
fn test_estimate_string_width_mixed_types() {
    let font_size = 16.0;
    // 'A' (ASCII字母) + ' ' (空格) + '1' (数字) + '.' (标点) + '中' (CJK)
    let width = estimate_string_width("A 1.中", font_size);
    let expected = 16.0 * 0.55 + 16.0 * 0.25 + 16.0 * 0.5 + 16.0 * 0.4 + 16.0;
    assert!(
        (width - expected).abs() < 0.01,
        "混合类型宽度应为 {}，实际 {}",
        expected,
        width
    );
}

/// 测试 estimate_string_width：负 font_size 产生负总宽度。
#[test]
fn test_estimate_string_width_negative_font_size() {
    let width = estimate_string_width("ABC", -10.0);
    assert!(width < 0.0, "负 font_size 应产生负宽度，实际 {}", width);
}

/// 测试 estimate_string_width：仅包含空格的字符串。
#[test]
fn test_estimate_string_width_spaces_only() {
    let width = estimate_string_width("   ", 16.0);
    let expected = 3.0 * 16.0 * 0.25;
    assert!(
        (width - expected).abs() < 0.01,
        "仅空格字符串宽度应为 {}，实际 {}",
        expected,
        width
    );
}

// ── InlineFormattingContext 边界条件 ──

/// 测试负容器宽度时不会 panic。
#[test]
fn test_negative_container_width_no_panic() {
    let mut ctx = InlineFormattingContext::new(-100.0);
    let runs = vec![TextRun {
        text: "Hello".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
    }];
    // 不应 panic
    ctx.break_into_lines(runs);
    // 负宽度下行为未定义，只要不 panic 即可
    assert!(!ctx.lines.is_empty(), "即使容器宽度为负也应产生行盒");
}

/// 测试极端窄容器（接近 0 但不为 0）中单字符换行。
#[test]
fn test_very_narrow_container_single_char_per_line() {
    let mut ctx = InlineFormattingContext::new(1.0);
    let runs = vec![TextRun {
        text: "a b c d".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
    }];
    ctx.break_into_lines(runs);
    // 极窄容器中每个单词应单独一行
    assert!(
        ctx.lines.len() >= 4,
        "极窄容器中每个单词应单独一行，实际 {} 行",
        ctx.lines.len()
    );
}

/// 测试 TextAlign 枚举的 Default trait。
#[test]
fn test_text_align_default() {
    assert_eq!(TextAlign::default(), TextAlign::Left);
}

/// 测试 inline-block 宽度为零时不影响布局。
#[test]
fn test_zero_width_inline_block() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::InlineBlock(InlineBlockBox {
            width: 0.0,
            height: 30.0,
            node_id: NodeId::default(),
            vertical_align: VerticalAlignValue::Baseline,
        }),
        InlineItem::Text(TextRun {
            text: "After".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
        }),
    ];
    ctx.break_items_into_lines(items);

    // 零宽 inline-block + 文本应在同一行
    assert_eq!(ctx.lines.len(), 1, "零宽 inline-block 不应触发换行");
    assert_eq!(ctx.lines[0].runs.len(), 2, "应有 2 个片段");
}

/// 测试 inline-block 高度为零时行盒高度由文本决定。
#[test]
fn test_zero_height_inline_block() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::Text(TextRun {
            text: "Text".to_string(),
            node_id: NodeId::default(),
            font_size: 16.0,
            line_height: 20.0,
            vertical_align: VerticalAlignValue::Baseline,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 0.0,
            node_id: NodeId::default(),
            vertical_align: VerticalAlignValue::Baseline,
        }),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(ctx.lines.len(), 1);
    // 行盒高度应取 max(20, 0) = 20
    assert!(
        (ctx.lines[0].height - 20.0).abs() < 0.01,
        "行盒高度应取 max(文本行高20, inline-block高度0) = 20，实际 {}",
        ctx.lines[0].height
    );
}

// ── split_into_words 边界条件 ──

/// 测试 split_into_words：单个单词。
#[test]
fn test_split_into_words_single_word() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("Hello");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0], "Hello ");
}

/// 测试 split_into_words：仅空白字符。
#[test]
fn test_split_into_words_whitespace_only() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("   ");
    assert!(words.is_empty(), "仅空白字符不应产生单词");
}

// ── overflow-wrap: break-word 测试 ──

/// break_word=false 时，超长单词不应在字符边界断行。
#[test]
fn test_break_word_false_long_word_no_split() {
    use crate::{InlineItem, TextRun};
    use zero_style_system::VerticalAlignValue;
    let mut ctx = InlineFormattingContext::new(50.0);
    let items = vec![InlineItem::Text(TextRun::simple(
        "Supercalifragilistic".to_string(),
        zero_dom::NodeId::default(),
        14.0,
        18.0,
        VerticalAlignValue::Baseline,
    ))];
    ctx.break_items_into_lines(items);
    // 不应拆分，整行一个 fragment
    let frags = ctx.all_fragments();
    assert_eq!(frags.len(), 1, "break_word=false 时不拆分长单词");
}

/// break_word=true 时，超长单词应在字符边界断行。
#[test]
fn test_break_word_true_long_word_splits() {
    use crate::{InlineItem, TextRun};
    use zero_style_system::VerticalAlignValue;
    let mut ctx = InlineFormattingContext::new(50.0).with_break_word(true);
    let items = vec![InlineItem::Text(TextRun::simple(
        "Supercalifragilistic".to_string(),
        zero_dom::NodeId::default(),
        14.0,
        18.0,
        VerticalAlignValue::Baseline,
    ))];
    ctx.break_items_into_lines(items);
    let frags = ctx.all_fragments();
    assert!(
        frags.len() > 1,
        "break_word=true 时应将超长单词拆分为多个 fragment，实际 {} 个",
        frags.len()
    );
    // 应产生多行
    assert!(
        ctx.lines.len() > 1,
        "break_word=true 时应产生多行，实际 {} 行",
        ctx.lines.len()
    );
}

// ── resolve_font_metrics 边界条件 ──

/// 测试 resolve_font_metrics：零值 font-size。
#[test]
fn test_resolve_font_metrics_zero_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(0.0);
    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 0.0).abs() < 0.01, "font_size 应为 0");
    assert!((line_height - 0.0).abs() < 0.01, "line_height 应为 0 * 1.2 = 0");
}

/// 测试 resolve_font_metrics：极大 font-size。
#[test]
fn test_resolve_font_metrics_large_font_size() {
    let mut style = ComputedStyle::default();
    style.font_size = LengthValue::Px(10000.0);
    let (font_size, line_height) = resolve_font_metrics(Some(&style));
    assert!((font_size - 10000.0).abs() < 0.01);
    let expected_lh = 10000.0 * 1.2;
    assert!(
        (line_height - expected_lh).abs() < 0.01,
        "line_height 应为 {}，实际 {}",
        expected_lh,
        line_height
    );
}

// ── white-space 属性测试 ──

/// 辅助：创建测试用 TextRun。
fn make_run(text: &str) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 19.2,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
    }
}

/// 测试 white-space: normal — 默认行为，自动换行。
#[test]
fn test_white_space_normal_wraps() {
    let mut ctx = InlineFormattingContext::new(50.0);
    ctx.break_into_lines(vec![make_run("hello world test")]);
    // 50px 容器 + 16px 字号，"hello world test" 应被分成多行
    assert!(
        ctx.lines.len() >= 2,
        "normal 模式应自动换行，实际 {} 行",
        ctx.lines.len()
    );
}

/// 测试 white-space: nowrap — 不换行，所有文本在一行。
#[test]
fn test_white_space_nowrap_no_wrap() {
    let mut ctx = InlineFormattingContext::new(50.0).with_no_wrap(true);
    ctx.break_into_lines(vec![make_run("hello world test")]);
    // nowrap：即使超出容器宽度也不换行
    assert_eq!(ctx.lines.len(), 1, "nowrap 模式应只有一行，实际 {} 行", ctx.lines.len());
}

/// 测试 white-space: pre — 保留空白且不换行。
#[test]
fn test_white_space_pre_preserves_and_no_wrap() {
    let mut ctx = InlineFormattingContext::new(50.0)
        .with_no_wrap(true)
        .with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("hello  world")]);
    // pre 模式：不换行
    assert_eq!(ctx.lines.len(), 1, "pre 模式应只有一行");
    // 保留空白：所有文本应该包含多空格
    let all_text: String = ctx
        .lines
        .iter()
        .flat_map(|l| l.runs.iter())
        .map(|r| r.text.clone())
        .collect();
    assert!(all_text.contains("  "), "pre 模式应保留多空格");
}

/// 测试 white-space: pre-wrap — 保留空白且自动换行。
#[test]
fn test_white_space_pre_wrap_preserves_and_wraps() {
    let mut ctx = InlineFormattingContext::new(50.0).with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("hello  world  test  data")]);
    // pre-wrap 模式：保留空白，但超宽时换行
    assert!(ctx.lines.len() >= 2, "pre-wrap 模式应换行，实际 {} 行", ctx.lines.len());
    // 保留空白
    let all_text: String = ctx
        .lines
        .iter()
        .flat_map(|l| l.runs.iter())
        .map(|r| r.text.clone())
        .collect();
    assert!(all_text.contains("  "), "pre-wrap 模式应保留多空格");
}

/// 测试 no_wrap=false + preserve_whitespace=false = normal 行为。
#[test]
fn test_white_space_default_equals_normal() {
    let mut c1 = InlineFormattingContext::new(80.0);
    let mut c2 = InlineFormattingContext::new(80.0)
        .with_no_wrap(false)
        .with_preserve_whitespace(false);
    let runs = vec![make_run("hello world")];
    c1.break_into_lines(runs.clone());
    c2.break_into_lines(runs);
    assert_eq!(c1.lines.len(), c2.lines.len(), "默认行为应与 normal 一致");
}

/// 测试 split_into_words 在 preserve_whitespace 模式下保留多空格。
#[test]
fn test_split_into_words_preserve_whitespace() {
    let ctx = InlineFormattingContext::new(200.0).with_preserve_whitespace(true);
    let words = ctx.split_into_words("hello  world");
    // 在保留模式下，多空格应该被保留
    let joined: String = words.join("");
    assert!(joined.contains("hello"), "应包含 hello");
    assert!(joined.contains("world"), "应包含 world");
}

/// 测试 split_into_words 在普通模式下折叠空白。
#[test]
fn test_split_into_words_normal_collapses() {
    let ctx = InlineFormattingContext::new(200.0);
    let words = ctx.split_into_words("hello  world");
    // 普通模式：split_whitespace 折叠多空格
    assert_eq!(words.len(), 2, "普通模式应折叠空白为 2 个单词");
    assert_eq!(words[0], "hello ");
    assert_eq!(words[1], "world ");
}

/// 测试 no_wrap 模式下长文本不换行。
#[test]
fn test_no_wrap_long_text_single_line() {
    let mut ctx = InlineFormattingContext::new(100.0).with_no_wrap(true);
    let long_text = "This is a very long text that should not wrap even though it exceeds the container width";
    ctx.break_into_lines(vec![make_run(long_text)]);
    assert_eq!(ctx.lines.len(), 1, "no_wrap 长文本应保持单行");
    let total_width: f32 = ctx.lines[0].runs.iter().map(|r| r.width).sum();
    assert!(total_width > 100.0, "文本宽度应超出容器，实际 {}", total_width);
}

// ── word-break: break-all 测试 ──

/// 测试 word-break: break-all 允许在任意字符间断行。
#[test]
fn test_word_break_break_all_splits_long_word() {
    // 窄容器（60px），一个长单词 "ABCDEFGHIJ"，break-all 应逐字符拆分
    let mut ctx = InlineFormattingContext::new(60.0).with_word_break(WordBreakMode::BreakAll);
    ctx.break_into_lines(vec![make_run("ABCDEFGHIJ")]);
    // break-all 应产生多行，因为长单词超过容器宽度
    assert!(
        ctx.lines.len() > 1,
        "break-all 应将长单词拆分到多行，实际 {} 行",
        ctx.lines.len()
    );
    // 每行的宽度不应超过容器宽度（容差 1px）
    for (i, line) in ctx.lines.iter().enumerate() {
        let line_end_x = line.runs.last().map(|r| r.x + r.width).unwrap_or(0.0);
        assert!(
            line_end_x <= 62.0,
            "第 {} 行宽度 {} 应不超过容器宽度 60（+容差），实际 {}",
            i,
            line_end_x,
            60.0
        );
    }
}

/// 测试 word-break: break-all 短单词不拆分。
#[test]
fn test_word_break_break_all_short_word_stays() {
    // 宽容器，短单词不应被拆分
    let mut ctx = InlineFormattingContext::new(800.0).with_word_break(WordBreakMode::BreakAll);
    ctx.break_into_lines(vec![make_run("Hello")]);
    assert_eq!(ctx.lines.len(), 1, "短单词应在一行中");
    assert_eq!(ctx.lines[0].runs.len(), 1, "短单词不应被拆分");
}

// ── word-break: keep-all 测试 ──

/// 测试 word-break: keep-all 保持 CJK 文本为单词。
#[test]
fn test_word_break_keep_all_cjk_stays_together() {
    // keep-all 模式下，连续 CJK 文本应作为一个整体
    let mut ctx = InlineFormattingContext::new(50.0).with_word_break(WordBreakMode::KeepAll);
    ctx.break_into_lines(vec![make_run("中文文本测试")]);
    // keep-all 应将 CJK 文本保持为单个单词，不拆分
    assert_eq!(ctx.lines.len(), 1, "keep-all 下 CJK 文本应作为单个单词（溢出）");
    assert_eq!(ctx.lines[0].runs.len(), 1, "CJK 文本不应被拆分");
}

/// 测试 word-break: keep-all 拉丁文本正常断行。
#[test]
fn test_word_break_keep_all_latin_breaks_at_spaces() {
    let mut ctx = InlineFormattingContext::new(80.0).with_word_break(WordBreakMode::KeepAll);
    ctx.break_into_lines(vec![make_run("Hello World Foo Bar")]);
    // keep-all 不影响拉丁文本（本来就在空格处断行）
    assert!(ctx.lines.len() >= 2, "keep-all 下拉丁文本应在空格处正常换行");
}

/// 测试 word-break: keep-all 空白处断行。
#[test]
fn test_word_break_keep_all_breaks_at_whitespace() {
    let mut ctx = InlineFormattingContext::new(50.0).with_word_break(WordBreakMode::KeepAll);
    // CJK 文本之间有空格，可以在空格处断行
    ctx.break_into_lines(vec![make_run("中文 文本 测试")]);
    // 有空格时可以断行
    assert_eq!(ctx.lines.len(), 3, "keep-all 应在空白处断行");
}

/// 测试 word-break 默认值 (Normal)。
#[test]
fn test_word_break_default_is_normal() {
    let ctx = InlineFormattingContext::new(800.0);
    assert_eq!(ctx.word_break, WordBreakMode::Normal);
}

/// 测试 word-break: break-all 多行内容所有字符都布局。
#[test]
fn test_word_break_break_all_preserves_all_chars() {
    let mut ctx = InlineFormattingContext::new(40.0).with_word_break(WordBreakMode::BreakAll);
    let text = "ABCDEFGH";
    ctx.break_into_lines(vec![make_run(text)]);
    // 验证所有字符都被布局了
    let all_text: String = ctx.all_fragments().iter().map(|f| f.text.as_str()).collect();
    assert_eq!(all_text.replace(' ', ""), text, "所有字符都应被布局");
}
