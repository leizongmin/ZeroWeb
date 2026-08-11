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
    let digit_width = estimate_char_width('5', font_size, false);
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
    let tab_width = estimate_char_width('\t', font_size, false);
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
    let nl_width = estimate_char_width('\n', font_size, false);
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
    let width = estimate_char_width('A', -16.0, false);
    // 负 font_size 应产生负宽度，不 panic
    assert!(width < 0.0, "负 font_size 应产生负宽度，实际 {}", width);
}

/// 测试 estimate_char_width：Unicode 非 CJK 字符使用默认宽度。
#[test]
fn test_estimate_char_width_unicode_non_cjk() {
    let font_size = 16.0;
    // U+00E9 = é，既不是 ASCII 也不是 CJK
    let width = estimate_char_width('\u{00E9}', font_size, false);
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
    let width = estimate_string_width("A 1.中", font_size, false);
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
    let width = estimate_string_width("ABC", -10.0, false);
    assert!(width < 0.0, "负 font_size 应产生负宽度，实际 {}", width);
}

/// 测试 estimate_string_width：仅包含空格的字符串。
#[test]
fn test_estimate_string_width_spaces_only() {
    let width = estimate_string_width("   ", 16.0, false);
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
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
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
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
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
            baseline: 30.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
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
            padding_top: 0.0,
            padding_bottom: 0.0,
            border_top: 0.0,
            border_bottom: 0.0,
            is_ahem_font: false,
            font_id: None,
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
            padding_top: 0.0,
            padding_bottom: 0.0,
            border_top: 0.0,
            border_bottom: 0.0,
            is_ahem_font: false,
            font_id: None,
        }),
        InlineItem::InlineBlock(InlineBlockBox {
            width: 50.0,
            height: 0.0,
            node_id: NodeId::default(),
            vertical_align: VerticalAlignValue::Baseline,
            baseline: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
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

/// R1720：inline-block 垂直 margin **扩展 line-box 高度**但**不下移 border box**
///（chromium 行为：border box 按 valign 定位，margin 在 border box 外扩 line box）。
/// 旧 R536 实现把 border box 下移 margin_top（`run.y += margin_top`），致 valign:middle
/// 叠加 margin over-shift（R1719 vspace 根因）。A/B 实证该 shift inert（css-flexbox/multicol
/// with/without 全等），故移除。本测钉死正确语义：run.y 不含 margin_top，line.height 含。
#[test]
fn test_inline_block_margin_top_offsets_box_y() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![InlineItem::InlineBlock(InlineBlockBox {
        width: 50.0,
        height: 30.0,
        node_id: NodeId::default(),
        vertical_align: VerticalAlignValue::Baseline,
        baseline: 30.0,
        margin_top: 16.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
    })];
    ctx.break_items_into_lines(items);

    assert_eq!(ctx.lines.len(), 1);
    let run = &ctx.lines[0].runs[0];
    // R1720：margin_top 不下移 border box——run.y 应 == 0.0（valign:baseline 定位），
    // 非 16.0（旧 R536 shift）。border box 留在 valign 位，margin 在其外扩 line box。
    assert!(
        run.y.abs() < 0.01,
        "inline-block margin_top 不应下移 border box，run.y 应 == 0，实际 {}",
        run.y
    );
    // 行盒高度应含 margin box（box 30 + margin_top 16 = 46）——margin 扩 line-box。
    assert!(
        ctx.lines[0].height >= 46.0,
        "行盒高度应含 margin_top，实际 {}",
        ctx.lines[0].height
    );
}

/// 辅助：构造一个仅含单个空格（collapse_whitespace 折叠后）的 TextRun。
fn space_run() -> TextRun {
    TextRun {
        text: " ".to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 19.2,
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
    }
}

/// 辅助：构造一个给定宽高的 inline-block 盒。
fn ib_box(width: f32, height: f32) -> InlineBlockBox {
    InlineBlockBox {
        width,
        height,
        node_id: NodeId::default(),
        vertical_align: VerticalAlignValue::Baseline,
        baseline: height,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
    }
}

/// 两个宽度之和恰好等于容器宽的 inline-block，之间夹一个空格 run，
/// 空格宽度应使第二个盒放不下而换行（whitespace-001 场景）。
#[test]
fn test_whitespace_between_inline_blocks_wraps() {
    // 容器 500，两个 250 宽 inline-block 恰好占满；空格使其溢出 → 第二个换行。
    let mut ctx = InlineFormattingContext::new(500.0);
    let items = vec![
        InlineItem::Text(space_run()),
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
        InlineItem::Text(space_run()),
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
        InlineItem::Text(space_run()),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(
        ctx.lines.len(),
        2,
        "两个恰好占满容器的 inline-block 之间的空格应使第二个换行"
    );
    // 每行各一个盒，都在行首（行首空格被移除）
    assert_eq!(ctx.lines[0].runs.len(), 1, "第一行应只有第一个 inline-block");
    assert_eq!(ctx.lines[1].runs.len(), 1, "第二行应只有第二个 inline-block");
    assert!((ctx.lines[0].runs[0].x - 0.0).abs() < 0.01, "第一个盒应在行首");
    assert!((ctx.lines[1].runs[0].x - 0.0).abs() < 0.01, "第二个盒应换到第二行行首");
}

/// 两个 inline-block 加上空格仍能放进容器时，空格应贡献间距使第二个盒
/// 紧贴第一个盒之后而非重叠（确认空格宽度被计入 advance）。
#[test]
fn test_whitespace_between_inline_blocks_fits() {
    // 容器 800，两个 250 宽 inline-block + 一个空格远小于 800 → 同一行。
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
        InlineItem::Text(space_run()),
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(ctx.lines.len(), 1, "两个 250 宽 inline-block 在 800 宽容器应同行");
    assert_eq!(ctx.lines[0].runs.len(), 2, "同行应有两个盒");
    let first_end = ctx.lines[0].runs[0].x + ctx.lines[0].runs[0].width;
    let second_x = ctx.lines[0].runs[1].x;
    assert!(
        second_x > first_end,
        "空格应使第二个盒在第一个盒之后（second_x={} 应大于 first_end={}），而非紧贴重叠",
        second_x,
        first_end
    );
}

/// 两个 inline-block 之间夹多个连续空格 run（如被注释节点分隔），
/// 按 CSS Text §4.1 应折叠为单个空格，只贡献一个空格宽度。
#[test]
fn test_consecutive_whitespace_runs_collapse() {
    // 容器 504：250 + 一个空格(4) + 250 = 504 恰好放得下；两个空格(8) 会溢出换行。
    // 折叠为单空格时应同行（不换行），验证连续空格 run 被折叠。
    let mut ctx = InlineFormattingContext::new(504.0);
    let items = vec![
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
        InlineItem::Text(space_run()),
        InlineItem::Text(space_run()),
        InlineItem::InlineBlock(ib_box(250.0, 19.0)),
    ];
    ctx.break_items_into_lines(items);

    assert_eq!(
        ctx.lines.len(),
        1,
        "两个连续空格 run 应折叠为单个空格，250+4+250=504 恰好放入 504 宽容器"
    );
}

// ── split_into_words 边界条件 ──

/// 测试 split_into_words：单个单词。
#[test]
fn test_split_into_words_single_word() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("Hello", false);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0], "Hello");
}

/// 测试 split_into_words：仅空白字符。
#[test]
fn test_split_into_words_whitespace_only() {
    let ctx = InlineFormattingContext::new(800.0);
    let words = ctx.split_into_words("   ", false);
    assert!(words.is_empty(), "仅空白字符不应产生单词");
}

/// R1927：white-space: pre-line（break_at_newline）应在换行符 `\n` 处强制断行
///（CSS Text 3 §4.2），同时折叠空白序列（区别于 pre-wrap 保留空白）。
/// split_into_words 对 break_at_newline 按 `\n` 切段，段间插入空串强制断行标记
///（break_items_into_lines 消费空串为强制断行，gate 已扩到 break_at_newline）。
/// normal 模式 `\n` 被折叠为普通词界（无断行标记）。
#[test]
fn test_preline_break_at_newline_markers() {
    // pre-line：单个 `\n` 产生 1 个空串强制断行标记。
    let ctx_preline = InlineFormattingContext::new(800.0).with_break_at_newline(true);
    let words = ctx_preline.split_into_words("a\nb", false);
    assert_eq!(
        words.iter().filter(|w| w.is_empty()).count(),
        1,
        "pre-line 单个 \\n 应产生 1 个断行标记，got {:?}",
        words
    );
    assert_eq!(
        words.iter().filter(|w| !w.is_empty()).count(),
        2,
        "pre-line a\\nb 应有 2 个非空词，got {:?}",
        words
    );

    // pre-line：多个 `\n` 各产生断行标记。
    let words_multi = ctx_preline.split_into_words("a\nb\nc", false);
    assert_eq!(
        words_multi.iter().filter(|w| w.is_empty()).count(),
        2,
        "pre-line 两个 \\n 应产生 2 个断行标记，got {:?}",
        words_multi
    );

    // normal（break_at_newline=false）：`\n` 折叠为词界，无断行标记。
    let ctx_normal = InlineFormattingContext::new(800.0);
    let words_normal = ctx_normal.split_into_words("a\nb", false);
    assert!(
        !words_normal.iter().any(|w| w.is_empty()),
        "normal 模式 \\n 不应产生断行标记，got {:?}",
        words_normal
    );
    assert_eq!(
        words_normal.iter().filter(|w| !w.is_empty()).count(),
        2,
        "normal a\\nb 应折叠为 2 个词，got {:?}",
        words_normal
    );
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
    let expected_lh = 10000.0 * 1.164;
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
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }
}

fn assert_bidi_fragment_sources(ctx: &InlineFormattingContext, logical_text: &str, expected_mapped_text: &str) {
    let fragments = ctx
        .all_fragments()
        .into_iter()
        .filter(|fragment| !fragment.text.is_empty())
        .collect::<Vec<_>>();
    let first_source = fragments
        .first()
        .and_then(|fragment| fragment.source.as_ref())
        .expect("reordered fragment must carry logical source");
    assert_eq!(&*first_source.text, logical_text);

    let mut ranges = Vec::new();
    for fragment in fragments {
        let source = fragment.source.as_ref().expect("all reordered fragments carry source");
        assert!(std::sync::Arc::ptr_eq(&source.text, &first_source.text));
        assert_eq!(
            source.visual_to_logical.len(),
            fragment.text.chars().count(),
            "source map must align with visual fragment {:?}",
            fragment.text
        );
        assert_eq!(
            source.visual_is_rtl.len(),
            fragment.text.chars().count(),
            "resolved direction must align with visual fragment {:?}",
            fragment.text
        );
        ranges.extend(source.visual_to_logical.iter().flatten().cloned());
    }
    ranges.sort_by_key(|range| (range.start, range.end));
    ranges.dedup();
    let mapped_text = ranges
        .iter()
        .map(|range| &logical_text[range.clone()])
        .collect::<String>();
    assert_eq!(mapped_text, expected_mapped_text);
}

/// https://www.w3.org/TR/css-writing-modes-3/#bidi-algo
#[test]
fn test_bidi_fragment_source_survives_normal_word_splitting() {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(vec![make_run("אבג")]);
    assert_eq!(ctx.all_fragments()[0].text, "גבא");
    assert_eq!(
        ctx.all_fragments()[0]
            .source
            .as_ref()
            .and_then(TextFragmentSource::uniform_resolved_rtl),
        Some(true)
    );
    assert_bidi_fragment_sources(&ctx, "אבג", "אבג");
}

/// https://www.w3.org/TR/css-text-3/#white-space-phase-1
#[test]
fn test_bidi_fragment_source_survives_pre_wrap_spaces() {
    let mut ctx = InlineFormattingContext::new(800.0).with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("אב  גד")]);
    assert_bidi_fragment_sources(&ctx, "אב  גד", "אב  גד");
}

/// https://www.w3.org/TR/css-text-3/#line-break-details
#[test]
fn test_bidi_fragment_source_survives_cjk_per_char_splitting() {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(vec![make_run("אב中ג")]);
    assert!(
        ctx.all_fragments().len() > 1,
        "CJK must create a separate break fragment"
    );
    assert_bidi_fragment_sources(&ctx, "אב中ג", "אב中ג");
}

/// https://www.w3.org/TR/css-text-3/#white-space-phase-1
#[test]
fn test_bidi_fragment_source_skips_newline_marker_without_losing_later_ranges() {
    let mut ctx = InlineFormattingContext::new(800.0).with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("אב\nגד")]);
    assert_eq!(ctx.lines.len(), 2);
    assert_bidi_fragment_sources(&ctx, "אב\nגד", "אבגד");
}

/// https://www.w3.org/TR/css-writing-modes-3/#vertical-modes
#[test]
fn test_bidi_fragment_source_survives_vertical_word_splitting() {
    let mut ctx = InlineFormattingContext::new(800.0).with_vertical(true);
    ctx.break_into_lines(vec![make_run("אב中ג")]);
    assert_bidi_fragment_sources(&ctx, "אב中ג", "אב中ג");
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

/// nowrap 容器内多个 inline-block 总宽超过容器也不换行（溢出）。
/// flexbox_flex-*-shrink REF 回归：此前 no_wrap 未传给 inline-block 定位 IFC
/// （engine.rs adjust_inline_block_positions），致 nowrap 容器内 inline-block 被错误换行。
#[test]
fn test_nowrap_inline_blocks_do_not_wrap() {
    let mut ctx = InlineFormattingContext::new(50.0).with_no_wrap(true);
    let items = vec![
        InlineItem::InlineBlock(ib_box(40.0, 20.0)),
        InlineItem::InlineBlock(ib_box(40.0, 20.0)),
    ];
    ctx.break_items_into_lines(items);
    // 两个 40px inline-block 总宽 80 > 容器 50，但 nowrap 不换行 → 单行
    assert_eq!(ctx.lines.len(), 1, "nowrap 容器内 inline-block 超宽不应换行");
    assert_eq!(ctx.lines[0].runs.len(), 2);
    // 第二个盒应在第一个之后（x=40），而非换到新行（x=0）
    assert!(
        (ctx.lines[0].runs[1].x - 40.0).abs() < 0.01,
        "第二个 inline-block 应在 x=40（溢出同行），实际 x={}",
        ctx.lines[0].runs[1].x
    );
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

/// R1338：pre-wrap 词间应为**单**空格（非双空格）。
///
/// 回归用例：`split_into_words` 的 preserve 分支旧实现给每个词追加 `format!("{w} ")`
/// 尾随空格 + 又 push 独立 " " → 词间双空格。修复后词不带尾随空格，间距完全由独立
/// " " 片段承载。Ahem 20px：`a` 与 `b` 之间的水平间距应 = 1 个空格宽（20px），非 40px。
#[test]
fn r1338_prewrap_single_interword_space() {
    let run = TextRun {
        text: "a b".to_string(),
        node_id: NodeId::default(),
        font_size: 20.0,
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
        is_ahem_font: true,
        font_id: None,
    };
    let mut ctx = InlineFormattingContext::new(800.0).with_preserve_whitespace(true);
    ctx.break_into_lines(vec![run]);
    let line = &ctx.lines[0];
    // 找到 "a" 与 "b" 片段
    let a = line.runs.iter().find(|r| r.text == "a").expect("a 片段");
    let b = line.runs.iter().find(|r| r.text == "b").expect("b 片段");
    let gap = b.x - (a.x + a.width);
    assert!(
        (gap - 20.0).abs() < 0.5,
        "pre-wrap 词间应为单空格 (20px)，实际 gap={gap:.1}（双空格 bug 回归？）"
    );
}

/// R1338：pre-wrap + text-align:right 下行尾保留空格应 "hang"（CSS Text §3.1.4 phase II）。
///
/// 复现 pre-wrap-align-right-001：Ahem 20px，容器 15ch=300px，pre-wrap，right 对齐。
/// 内容 "one two three four five" 在 300px 内换行：行 0 = "one two three" + 换行点
/// 尾随空格。期望：可见内容（"three" 右缘）贴容器右缘（x≈300），尾随空格 hang 到
/// 行缘外（x≥300）。旧实现把尾随空格计入 content_width → 内容整体左移 1 空格。
#[test]
fn r1338_prewrap_right_align_trailing_space_hangs() {
    let run = TextRun {
        text: "one two three four five".to_string(),
        node_id: NodeId::default(),
        font_size: 20.0,
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
        is_ahem_font: true,
        font_id: None,
    };
    let mut ctx = InlineFormattingContext::new(300.0)
        .with_preserve_whitespace(true)
        .with_text_align(TextAlign::Right);
    ctx.break_into_lines(vec![run]);
    // 行 0 应包含 "three"（换行点在 three 之后）
    let line0 = &ctx.lines[0];
    let three = line0.runs.iter().find(|r| r.text == "three").expect("行 0 应含 three");
    // 可见内容右缘应贴容器右缘（300），误差 < 半个 Ahem 像素
    let three_right = three.x + three.width;
    assert!(
        (three_right - 300.0).abs() < 0.5,
        "right-align 后 'three' 右缘应=300（hang），实际={three_right:.1}（尾随空格未 hang）"
    );
    // 行 0 末片段应为尾随空格，且其 x ≥ 300（hang 到行缘外）
    let last = line0.runs.last().expect("行 0 应有片段");
    assert!(
        last.text.trim().is_empty() && last.x >= 299.5,
        "行 0 末应为悬挂空格 x≥300，实际 text={:?} x={:.1}",
        last.text,
        last.x
    );
}

/// 测试 white-space: pre / pre-wrap 下显式换行符 `\n` 强制换行。
///
/// 回归用例：`split_into_words` 在 preserve_whitespace 模式下为每个 `\n`
/// 推入空字符串作为强制换行标记，但单词循环旧实现对空字符串只 `continue`
/// 静默丢弃换行标记 → 多行 `<pre>` 内容塌缩为一行（morning-work 文章代码块
/// 整体垂直压缩的根因）。修复后每个 `\n` 应产生一个新行盒（CSS Text §3.1：
/// pre/pre-wrap 下换行符是强制断行机会）。
#[test]
fn test_white_space_pre_newline_forces_break() {
    let mut ctx = InlineFormattingContext::new(800.0).with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("line one\nline two\nline three")]);
    assert_eq!(
        ctx.lines.len(),
        3,
        "pre-wrap 模式下 3 个 \\n 分隔的行应产生 3 行，实际 {} 行",
        ctx.lines.len()
    );
    // 每行应包含对应文本（片段可能因词间距拆分，故用 contains 判定）
    let texts: Vec<String> = ctx
        .lines
        .iter()
        .map(|l| l.runs.iter().map(|r| r.text.clone()).collect::<String>())
        .collect();
    assert!(
        texts[0].contains("line") && texts[0].contains("one"),
        "第 1 行 {:?}",
        texts[0]
    );
    assert!(
        texts[1].contains("line") && texts[1].contains("two"),
        "第 2 行 {:?}",
        texts[1]
    );
    assert!(
        texts[2].contains("line") && texts[2].contains("three"),
        "第 3 行 {:?}",
        texts[2]
    );
}

/// pre（no_wrap + preserve）模式下 `\n` 同样应强制换行。
#[test]
fn test_white_space_pre_newline_forces_break_no_wrap() {
    let mut ctx = InlineFormattingContext::new(50.0)
        .with_no_wrap(true)
        .with_preserve_whitespace(true);
    ctx.break_into_lines(vec![make_run("aaaa\nbbbb")]);
    assert_eq!(ctx.lines.len(), 2, "pre 模式下 \\n 仍应强制换行");
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
    let words = ctx.split_into_words("hello  world", false);
    // 在保留模式下，多空格应该被保留
    let joined: String = words.join("");
    assert!(joined.contains("hello"), "应包含 hello");
    assert!(joined.contains("world"), "应包含 world");
}

/// 测试 split_into_words 在普通模式下折叠空白。
#[test]
fn test_split_into_words_normal_collapses() {
    let ctx = InlineFormattingContext::new(200.0);
    let words = ctx.split_into_words("hello  world", false);
    // 普通模式：split_whitespace 折叠多空格
    assert_eq!(words.len(), 2, "普通模式应折叠空白为 2 个单词");
    assert_eq!(words[0], "hello ");
    assert_eq!(words[1], "world");
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

// ── 垂直书写模式测试 ──────────────────────────────────────────────────

/// 测试垂直模式下短文本放入单列。
#[test]
fn test_vertical_single_column() {
    let mut ctx = InlineFormattingContext::new(800.0).with_vertical(true);
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
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    }];
    ctx.break_into_lines(runs);
    // 短文本应在单列中
    assert_eq!(ctx.lines.len(), 1, "短文本应在单列中");
    // 字符沿 y 向下推进，fragments 的 y 应递增
    let frags: Vec<_> = ctx.all_fragments();
    assert!(!frags.is_empty());
    for i in 1..frags.len() {
        assert!(
            frags[i].y >= frags[i - 1].y,
            "垂直模式下片段 y 应递增: frags[{}].y={} >= frags[{}].y={}",
            i,
            frags[i].y,
            i - 1,
            frags[i - 1].y
        );
    }
}

/// 测试垂直模式下多列换列。
#[test]
fn test_vertical_column_breaking() {
    // 使用 Ahem 字体 + break-word 模式：每个字符宽度 = font_size
    let mut ctx = InlineFormattingContext::new(50.0)
        .with_vertical(true)
        .with_break_word(true);
    // 10 个字符，每个 16px 宽 = 160px 总深度，但 max_depth=50px
    let runs = vec![TextRun {
        text: "AAAAAAAAAA".to_string(), // 10 chars × 16px = 160px depth
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
        is_ahem_font: true,
        font_id: None,
    }];
    ctx.break_into_lines(runs);
    // 应产生多列（max_depth=50px，每个字符 16px，第 4 个字符开始换列）
    assert!(ctx.lines.len() > 1, "长文本应产生多列，实际 {} 列", ctx.lines.len());
}

/// 测试垂直模式下列沿 x 轴排列。
#[test]
fn test_vertical_columns_advance_along_x() {
    let mut ctx = InlineFormattingContext::new(40.0).with_vertical(true);
    let runs = vec![TextRun {
        text: "AAAAAA".to_string(), // 6 chars × 16px = 96px depth
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
        is_ahem_font: true,
        font_id: None,
    }];
    ctx.break_into_lines(runs);
    // 列的 y 值（实际是 x 坐标）应递增
    for i in 1..ctx.lines.len() {
        assert!(
            ctx.lines[i].y > ctx.lines[i - 1].y,
            "列应沿 x 递增: lines[{}].y={} > lines[{}].y={}",
            i,
            ctx.lines[i].y,
            i - 1,
            ctx.lines[i - 1].y
        );
    }
}

/// R1456：垂直模式下 `all_fragments_with_line_y` 的片段 y 必须是列内深度（run.y），
/// **不可**加 line.y。垂直模式 line.y 是**列 x 坐标**（break_items_into_columns /
/// break_into_lines 的 vertical 轴交换把列 x 存进 line.y），已在 run.x（= 列 x）中体现。
/// 旧实现 `run.y + line_y` 把列 x（如 764）误加到深度（0）→ frag_y=764 → 文本推到
/// viewport 外（block-flow-direction-vrl-011 全 0 可见）。WM gate：vertical 不加 line.y。
#[test]
fn test_r1456_vertical_fragment_y_is_depth_not_column_x() {
    let mut ctx = InlineFormattingContext::new(50.0)
        .with_vertical(true)
        .with_break_word(true);
    let runs = vec![TextRun {
        text: "AAAAAAAAAA".to_string(), // 10 chars × 16px = 160px depth > 50 max_depth → 多列
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
        is_ahem_font: true,
        font_id: None,
    }];
    ctx.break_into_lines(runs);
    // 须多列且存在 line.y>0 的列（line.y = 列 x），否则无法暴露「加 line.y」bug。
    assert!(ctx.lines.len() > 1, "须多列");
    assert!(ctx.lines.iter().any(|l| l.y > 0.0), "须有 line.y>0 的列（列 x 非零）");
    let with_line_y = ctx.all_fragments_with_line_y();
    let plain = ctx.all_fragments();
    assert_eq!(with_line_y.len(), plain.len());
    // 垂直模式：line.y（列 x）不可加到片段 y → 两路径 y 必须相等（= 列内深度 run.y）。
    // 旧实现（run.y+line_y）会使 with_line_y.y 比 plain.y 大 line.y → 断言失败。
    for (w, p) in with_line_y.iter().zip(plain.iter()) {
        assert_eq!(
            w.y, p.y,
            "垂直模式片段 y 应=深度(run.y)，不加 line.y（列 x）：with_line_y.y={} plain.y={}",
            w.y, p.y
        );
    }
}

/// 测试垂直模式下片段的 width 等于 line-height（列宽）。
#[test]
fn test_vertical_fragment_width_is_line_height() {
    let mut ctx = InlineFormattingContext::new(800.0).with_vertical(true);
    let runs = vec![TextRun {
        text: "Hi".to_string(),
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
    }];
    ctx.break_into_lines(runs);
    let frags: Vec<_> = ctx.all_fragments();
    for frag in &frags {
        assert_eq!(
            frag.width, 24.0,
            "垂直模式下片段 width 应等于 line-height: got {}",
            frag.width
        );
    }
}

/// 测试垂直模式下 Br 强制换列。
#[test]
fn test_vertical_br_forces_new_column() {
    let mut ctx = InlineFormattingContext::new(800.0).with_vertical(true);
    let items = vec![
        InlineItem::Text(TextRun {
            text: "A".to_string(),
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
        }),
        InlineItem::Br,
        InlineItem::Text(TextRun {
            text: "B".to_string(),
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
        }),
    ];
    ctx.break_items_into_lines(items);
    assert!(ctx.lines.len() >= 2, "Br 应强制换列");
}

/// 测试垂直模式下水平模式不受影响（回归测试）。
#[test]
fn test_horizontal_mode_unaffected_by_vertical_impl() {
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
    }];
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "水平模式：短文本应在单行中");
    assert_eq!(ctx.lines[0].runs.len(), 2, "水平模式：两个单词");
    // 水平模式下 x 递增，y 为 0（直到 vertical-align 调整）
    let frags = ctx.all_fragments();
    assert!(frags[0].x < frags[1].x, "水平模式下片段 x 应递增");
}

#[test]
fn test_empty_inline_element_applies_margin_right() {
    // CSS 2.1: 空 inline 元素的 margin-left 和 margin-right 都应被消费
    // 验证空元素后，后续元素的 x 坐标应包含空元素的 margin-left + margin-right
    let mut ctx = InlineFormattingContext::new(800.0);
    let empty_run = TextRun {
        text: String::new(), // 空 inline 元素
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 50.0,
        margin_right: 30.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: false,
        font_id: None,
    };
    let text_run = TextRun {
        text: "after".to_string(),
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
    };
    let items = vec![InlineItem::Text(empty_run), InlineItem::Text(text_run)];
    ctx.break_items_into_lines(items);

    // 验证：空元素的 line-height 贡献到行盒高度
    assert_eq!(ctx.lines.len(), 1, "空元素和文本应在同一行");
    assert!(ctx.lines[0].height >= 20.0, "行高应包含空元素的 line-height");

    // 验证：后续文本的 x 坐标应包含空元素的 margin-left + margin-right
    let frags = ctx.all_fragments();
    assert!(!frags.is_empty(), "应有文本片段");
    let after_frag = frags
        .iter()
        .find(|frag| frag.text == "after")
        .expect("应找到后续文本片段");
    // 空元素消费了 margin-left (50) + margin-right (30) = 80px
    // 所以后续文本的 x 应至少为 80
    assert!(
        after_frag.x >= 80.0,
        "空元素后文本 x 应包含 margin-left+margin-right (期望>=80, 实际={})",
        after_frag.x
    );
}

// ── 空行内元素 line-height 贡献和空白保留测试 ──

/// 测试空 inline 元素的高 line-height 贡献到行盒高度。
/// CSS 2.1 §10.8.1：空 inline 元素的 line-height 仍贡献到行盒高度。
#[test]
fn test_empty_inline_tall_line_height_contribution() {
    let mut ctx = InlineFormattingContext::new(200.0);

    // 空 inline 元素 line-height = 80px（5 × 16px）
    let empty_run = TextRun::simple(
        String::new(),
        NodeId::default(),
        16.0, // font_size
        80.0, // line_height: 5 × font_size
        VerticalAlignValue::Baseline,
    );

    // 正常文本 line-height = 16px
    let text_run = TextRun::simple(
        "X".to_string(),
        NodeId::default(),
        16.0, // font_size
        16.0, // line_height: 1 × font_size
        VerticalAlignValue::Baseline,
    );

    ctx.break_into_lines(vec![empty_run, text_run]);

    // 行盒高度应取 max(80, 16) = 80
    assert_eq!(ctx.lines.len(), 1, "应生成一行");
    assert!(
        (ctx.lines[0].height - 80.0).abs() < 0.01,
        "行盒高度应为空元素 line-height (期望≈80, 实际={})",
        ctx.lines[0].height
    );
    assert_eq!(ctx.total_height(), 80.0, "总高度应为 80");
}

/// 测试 collapse_whitespace 函数的基本行为。
#[test]
fn test_collapse_whitespace() {
    // 空输入
    assert_eq!(collapse_whitespace(""), "");

    // 纯空白 → 折叠为单个空格
    assert_eq!(collapse_whitespace("   "), " ");
    assert_eq!(collapse_whitespace("\n\t  "), " ");

    // 无空白
    assert_eq!(collapse_whitespace("hello"), "hello");

    // 内部多个空白折叠为单个
    assert_eq!(collapse_whitespace("hello  world"), "hello world");
    assert_eq!(collapse_whitespace("a\n\tb"), "a b");

    // 首尾空白保留（行级剥离由 IFC 处理）
    assert_eq!(collapse_whitespace(" hello "), " hello ");

    // 混合空白
    assert_eq!(collapse_whitespace("  a  b  c  "), " a b c ");
}

/// 测试 IFC 行首空格剥离。
/// CSS 2.1 §16.6.1：行首空格不渲染。
#[test]
fn test_line_start_space_stripping() {
    let mut ctx = InlineFormattingContext::new(200.0);

    // 文本以空格开头（模拟 inline-block 之间的空白节点）
    let run = TextRun::simple(
        " X".to_string(),
        NodeId::default(),
        16.0,
        16.0,
        VerticalAlignValue::Baseline,
    );

    ctx.break_into_lines(vec![run]);

    assert_eq!(ctx.lines.len(), 1, "应生成一行");
    // 行首空格被剥离，片段文本应为 "X"
    assert_eq!(ctx.lines[0].runs.len(), 1);
    assert_eq!(ctx.lines[0].runs[0].text, "X", "行首空格应被剥离");
    // x 坐标应从 0 开始（无前导空格宽度）
    assert!(
        ctx.lines[0].runs[0].x < 1.0,
        "行首空格剥离后 x 应接近 0 (实际={})",
        ctx.lines[0].runs[0].x
    );
}

/// strut ascent 必须基于块容器自身的 font-size（CSS 2.1 §10.8.1），而非行盒实测高度。
///
/// 当行盒被高大的原子行内盒（inline-block/inline-flex）撑高时，旧行为用
/// `line_height * 0.8` 算 strut，会把 strut ascent 错误放大。这导致合成 baseline
/// 偏低的原子盒（baseline < 放大后的 strut）被压到行盒下方，与同容器其它盒错位。
///
/// 本测试构造：容器 font-size=20（strut ascent 应=16），一个高 35、baseline=20 的
/// 原子盒单独成行（行高被撑到 35，旧 strut=28）。baseline 对齐后该盒应位于行顶
/// （y≈0），而非被旧 strut(28) 压下 8px。
#[test]
fn test_strut_ascent_uses_container_font_not_line_height() {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.container_font_size = 20.0;
    let items = vec![InlineItem::InlineBlock(InlineBlockBox {
        width: 100.0,
        height: 35.0, // 撑高行盒 → 旧 line_height*0.8 = 28
        node_id: NodeId::default(),
        vertical_align: VerticalAlignValue::Baseline,
        baseline: 20.0, // 合成 baseline：>= 新 strut(16) 但 < 旧 strut(28)
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
    })];
    ctx.break_items_into_lines(items);

    assert_eq!(ctx.lines.len(), 1, "单个原子盒应单独成行");
    let run = &ctx.lines[0].runs[0];
    assert!(
        run.y.abs() < 0.5,
        "baseline 偏低的原子盒应位于行顶 (y≈0)，实际 y={:.2}（strut 不应基于被撑高的行高）",
        run.y
    );
}

/// 对照测试：当原子盒的 baseline 低于容器 strut（基于 font-size）时，它确实会被
/// 压到行顶之下——证明 strut 仍在生效，只是基于正确的（容器字体）基准。
#[test]
fn test_strut_still_applies_when_baseline_below_container_strut() {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.container_font_size = 20.0; // strut ascent = 16
    let items = vec![InlineItem::InlineBlock(InlineBlockBox {
        width: 100.0,
        height: 35.0,
        node_id: NodeId::default(),
        vertical_align: VerticalAlignValue::Baseline,
        baseline: 10.0, // < strut(16) → 应被压下 6px
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
    })];
    ctx.break_items_into_lines(items);

    let run = &ctx.lines[0].runs[0];
    assert!(
        (run.y - 6.0).abs() < 0.5,
        "baseline(10) 低于 strut(16) 的盒应被压下 6px (y≈6)，实际 y={:.2}",
        run.y
    );
}

// ── AdvanceSource 抽象 seam（R223 plumbing R1）──

/// EstimateAdvance 默认实现必须与 estimate_char_width 完全等价（零行为变更），
/// 保证 advance-width plumbing 的 seam 不改变现有 IFC 度量结果。
#[test]
fn test_estimate_advance_matches_estimate_char_width() {
    use crate::inline::{AdvanceSource, EstimateAdvance};
    let src = EstimateAdvance;
    let font_size = 16.0f32;
    // font_id 为 None 时（R1 默认），EstimateAdvance 必须回退到 estimate_char_width
    for ch in ['W', 'i', 'm', '5', '.', ' ', '中', 'A', 't'] {
        for &is_ahem in &[false, true] {
            let via_trait = src.measure(ch, None, font_size, is_ahem);
            let direct = estimate_char_width(ch, font_size, is_ahem);
            assert!(
                (via_trait - direct).abs() < 1e-6,
                "EstimateAdvance.measure({ch}, ahem={is_ahem}) = {via_trait} != estimate_char_width {direct}"
            );
        }
    }
    // font_id 非 None 时，默认实现忽略它（仍等价 estimate）——R3 注入真实实现才用 font_id
    let with_id = src.measure('W', Some(42), font_size, false);
    let no_id = src.measure('W', None, font_size, false);
    assert!((with_id - no_id).abs() < 1e-6, "EstimateAdvance 应忽略 font_id");
}

// ── C3 advance plumbing（R2 dormant seam）：IFC.advance_source 注入与消费 ──

/// 测试用桩 advance 源：所有非-Ahem 字符返回 `font_size × multiplier`（multiplier 远
/// 大于 estimate 的 0.55），用于证明 IFC 度量点确实经 `advance_of` 消费注入的源。
struct WideAdvance(f32);
impl AdvanceSource for WideAdvance {
    fn measure(&self, _ch: char, _font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        if is_ahem { font_size } else { font_size * self.0 }
    }
}

/// 默认 IFC.advance_source = None（零回归：度量点回退 estimate_char_width）。
#[test]
fn ifc_advance_source_defaults_none() {
    let ctx = InlineFormattingContext::new(100.0);
    assert!(
        ctx.advance_source.is_none(),
        "IFC must default to advance_source = None (estimate path active = zero-regression)"
    );
}

/// 注入 advance 源后，IFC 换行决策必须消费它（proof-of-seam）。
///
/// 容器宽 80px、字号 10px、文本 "aa aa"（两词 + 空格）：
/// - 默认（estimate 0.55）：'a'=5.5px，整行 ~24.5px < 80 → 1 行。
/// - 注入 WideAdvance(2.0)：'a'=20px，整行 40+20+40=100px > 80 → 换行 ≥2 行。
#[test]
fn ifc_advance_source_injected_is_consulted_in_wrapping() {
    let make_ctx = |source: Option<Rc<dyn AdvanceSource>>| {
        let mut ctx = InlineFormattingContext::new(80.0);
        if let Some(s) = source {
            ctx = ctx.with_advance_source(s);
        }
        let items = vec![InlineItem::Text(TextRun {
            text: "aa aa".to_string(),
            node_id: NodeId::default(),
            font_size: 10.0,
            line_height: 10.0,
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
        })];
        ctx.break_items_into_lines(items);
        ctx
    };

    // 默认 estimate：文本一行放得下。
    let default_ctx = make_ctx(None);
    assert_eq!(
        default_ctx.lines.len(),
        1,
        "estimate (0.55): \"aa aa\" ~24.5px < 80px fits one line"
    );

    // 注入 WideAdvance(2.0)：每字符 20px，整行 100px > 80px → 换行。
    let wide: Rc<dyn AdvanceSource> = Rc::new(WideAdvance(2.0));
    let wide_ctx = make_ctx(Some(wide));
    assert!(
        wide_ctx.lines.len() > 1,
        "injected WideAdvance(2.0): \"aa aa\" 100px > 80px must wrap (advance_of consulted)"
    );
}

/// 整串测量覆写必须驱动 fragment width，不能退回逐字符累加。
#[test]
fn ifc_advance_source_uses_contextual_text_measurement() {
    struct ContextAdvance;
    impl AdvanceSource for ContextAdvance {
        fn measure(&self, _ch: char, _font_id: Option<u32>, _font_size: f32, _is_ahem: bool) -> f32 {
            10.0
        }

        fn measure_text(&self, text: &str, _font_id: Option<u32>, _font_size: f32, _is_ahem: bool) -> f32 {
            assert_eq!(text, "AV");
            15.0
        }
    }

    let mut ctx = InlineFormattingContext::new(100.0).with_advance_source(Rc::new(ContextAdvance));
    ctx.break_into_lines(vec![TextRun {
        text: "AV".to_string(),
        node_id: NodeId::default(),
        font_size: 10.0,
        line_height: 10.0,
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
        font_id: Some(7),
    }]);

    assert_eq!(ctx.lines[0].runs[0].width, 15.0);
}

#[test]
fn paint_ifc_font_id_override_restores_shaping_id_without_styles() {
    let node = NodeId::default();
    let ctx = InlineFormattingContext::new(100.0).with_font_id_overrides(HashMap::from([(node, 7)]));

    assert_eq!(
        ctx.shaping_font_id_for_style(Some(node), None, false, 0.0, 0.0, false),
        Some(7)
    );
    assert_eq!(
        ctx.shaping_font_id_for_style(Some(node), None, false, 1.0, 0.0, false),
        None
    );
}

#[test]
fn ifc_advance_source_receives_ordered_font_ids() {
    struct OrderedAdvance;
    impl AdvanceSource for OrderedAdvance {
        fn measure(&self, _ch: char, _font_id: Option<u32>, _font_size: f32, _is_ahem: bool) -> f32 {
            10.0
        }

        fn measure_text_with_fonts(&self, text: &str, font_ids: &[u32], _font_size: f32, _is_ahem: bool) -> f32 {
            assert_eq!(text, "xA");
            assert_eq!(font_ids, &[7, 9]);
            21.0
        }
    }

    let node = NodeId::default();
    let mut ctx = InlineFormattingContext::new(100.0)
        .with_advance_source(Rc::new(OrderedAdvance))
        .with_font_ids_overrides(HashMap::from([(node, vec![7, 9])]));
    ctx.break_into_lines(vec![TextRun {
        text: "xA".to_string(),
        node_id: node,
        font_size: 10.0,
        line_height: 10.0,
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
        font_id: Some(7),
    }]);

    assert_eq!(ctx.lines[0].runs[0].width, 21.0);
}

// ── R990：apply_vertical_alignment 的 ascent ratio 按 is_ahem 区分 ──

/// 构造单文本运行 IFC（line-height:1，即 line_height == font_size，half-leading=0），
/// 使 baseline_y == font_size × ascent_ratio，便于直接断言 ratio。
fn build_single_text_line(is_ahem: bool) -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(800.0);
    let items = vec![InlineItem::Text(TextRun {
        text: "Text".to_string(),
        node_id: NodeId::default(),
        font_size: 100.0,
        line_height: 100.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        margin_left: 0.0,
        margin_right: 0.0,
        padding_top: 0.0,
        padding_bottom: 0.0,
        border_top: 0.0,
        border_bottom: 0.0,
        is_ahem_font: is_ahem,
        font_id: None,
    })];
    ctx.break_items_into_lines(items);
    ctx
}

/// R990：Ahem 文本 ascent ratio = 0.8（精确，upem 1000/ascent 800）。
/// line-height:1 → baseline_y = font_size × 0.8 = 80。
#[test]
fn test_r990_ahem_ascent_ratio_0p8() {
    let ctx = build_single_text_line(true);
    assert_eq!(ctx.lines.len(), 1);
    let baseline_y = ctx.lines[0].baseline_y;
    assert!(
        (baseline_y - 80.0).abs() < 0.01,
        "Ahem 文本 baseline_y 应 = fs×0.8 = 80（实测 {baseline_y}）"
    );
}

/// R990：非-Ahem 文本 ascent ratio = 0.928（system-ui/DejaVuSans 真实 ascent，R885 实测）。
/// 旧实现硬编码 0.8 致非-Ahem 行盒偏矮、基线偏低（font-wall 主因之一）。
#[test]
fn test_r990_non_ahem_ascent_ratio_0p928() {
    let ctx = build_single_text_line(false);
    assert_eq!(ctx.lines.len(), 1);
    let baseline_y = ctx.lines[0].baseline_y;
    assert!(
        (baseline_y - 92.8).abs() < 0.01,
        "非-Ahem 文本 baseline_y 应 = fs×0.928 = 92.8（实测 {baseline_y}）"
    );
}

// ── R1004：ascent_ratio_overrides bypass 基础设施（dormant，零回归）──
// Phase A §12.6 step-2 解锁机制：layout IFC 经 provider 算出每文本节点真实 ascent
// ratio，存入 LayoutBox → paint Path B 经 ascent_ratio_overrides 读取，绕过 R890
// 实证的空 styles 墙。空 map（默认）回退 R990 常数 → 零回归。下两项断言覆盖优先级。

/// R1004：ascent_ratio_overrides 真实 per-font ratio 优先于 R990 is_ahem 常数。
/// 非-Ahem 文本 + 覆盖 0.95（模拟 NotoSansCJK 真实 ascent）→ baseline_y = 100×0.95 = 95
/// （无覆盖时为 92.8 = 0.928）。
#[test]
fn test_r1004_ascent_ratio_override_supersedes_r990_constant() {
    let node_id = NodeId::default();
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(node_id, 0.95);
    let mut ctx = InlineFormattingContext::new(800.0).with_ascent_ratio_overrides(overrides);
    let items = vec![InlineItem::Text(TextRun {
        text: "Text".to_string(),
        node_id,
        font_size: 100.0,
        line_height: 100.0,
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
    })];
    ctx.break_items_into_lines(items);
    assert_eq!(ctx.lines.len(), 1);
    let baseline_y = ctx.lines[0].baseline_y;
    assert!(
        (baseline_y - 95.0).abs() < 0.01,
        "覆盖 0.95 时 baseline_y 应 = fs×0.95 = 95（实测 {baseline_y}）；证明 override 优先于 R990 常数"
    );
}

/// R1004：override ratio ≤ 0（无效/未填充）回退 R990 is_ahem 常数。
/// 保证 dormant 默认（空 map 或 step-2 未填充节点）= R990 行为，零回归。
#[test]
fn test_r1004_ascent_ratio_override_zero_or_absent_falls_back() {
    let node_id = NodeId::default();
    // 覆盖 0.0（无效）→ 回退 0.928（非-Ahem）
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(node_id, 0.0);
    let ratio = ascent_ratio_lookup(&overrides, node_id, false);
    assert!(
        (ratio - 0.928).abs() < 1e-6,
        "override=0.0 应回退非-Ahem 常数 0.928（实测 {ratio}）"
    );
    // 空 map（默认 dormant）→ 回退 0.8（Ahem）
    let empty: std::collections::HashMap<NodeId, f32> = std::collections::HashMap::new();
    let ratio_ahem = ascent_ratio_lookup(&empty, NodeId::default(), true);
    assert!(
        (ratio_ahem - 0.8).abs() < 1e-6,
        "空 map + Ahem 应回退 0.8（实测 {ratio_ahem}）"
    );
}

// ── R1012：text-transform 行断前应用（Phase A IFC 统一首切）──
// text-transform 须在 collect_inline_items 期应用，使 layout 用转换后文本宽度行断
// （与 chromium 一致）。layout IFC（有 styles）读父元素 computed text-transform；
// paint Path B（空 styles）走 text_transform_overrides 覆盖（re-key 到父元素）。
// 以下两项分别覆盖两条路径，断言 frag.text 已转换。

/// R1012：layout IFC（有真实 styles）在 collect_inline_items 期应用 text-transform。
/// `<p style="text-transform:uppercase">hello</p>` → 片段文本应为 "HELLO"。
#[test]
fn test_r1012_text_transform_applied_via_style() {
    use std::collections::HashMap;
    use zero_dom::parse_html;
    use zero_style_system::{ComputedStyle, TextTransformValue};

    let doc = parse_html("<p>hello</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut style = ComputedStyle::default();
    style.text_transform = TextTransformValue::Uppercase;
    let mut styles = HashMap::new();
    styles.insert(p, style);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    let all_text: String = ctx.all_fragments().iter().map(|f| f.text.clone()).collect();
    assert_eq!(
        all_text, "HELLO",
        "layout IFC 应在行断前应用 uppercase（实测 {all_text}）"
    );
}

/// R1012：paint Path B 空 styles IFC 经 text_transform_overrides 应用 text-transform。
/// 模拟 Path B：styles 为空，但 override map 携带父元素 transform → collect 仍转换文本。
#[test]
fn test_r1012_text_transform_applied_via_override_map() {
    use std::collections::HashMap;
    use zero_dom::parse_html;
    use zero_style_system::TextTransformValue;

    let doc = parse_html("<p>hello</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    // 空 styles（模拟 paint Path B），但 override map 携带 p 的 transform。
    let mut overrides = HashMap::new();
    overrides.insert(p, TextTransformValue::Uppercase);
    let mut ctx = InlineFormattingContext::new(800.0).with_text_transform_overrides(overrides);
    ctx.layout(&doc, p, &HashMap::new());

    let all_text: String = ctx.all_fragments().iter().map(|f| f.text.clone()).collect();
    assert_eq!(
        all_text, "HELLO",
        "空 styles + override map 应仍应用 uppercase（实测 {all_text}）；证明 Path B 绕过空 styles 墙"
    );
}

/// R1012：默认（空 override map + 无 style text-transform）= None = 原文，零回归。
#[test]
fn test_r1012_text_transform_default_is_noop() {
    use std::collections::HashMap;
    use zero_dom::parse_html;

    let doc = parse_html("<p>hello</p>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let p = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    let all_text: String = ctx.all_fragments().iter().map(|f| f.text.clone()).collect();
    assert_eq!(
        all_text, "hello",
        "默认（无 transform）应原样，零回归（实测 {all_text}）"
    );
}

/// R1022：`<ruby>` inline 收集排除 `<rt>`/`<rp>` 文本。
///
/// `<ruby><rb>Fi</rb><rt>●●</rt><rp>(注)</rp></ruby>l` → inline 文本应为 "Fil"
/// （rb "Fi" + 尾 "l"），rt/rp 文本不混入 inline 流（由 paint 期作 annotation 上移）。
#[test]
fn test_r1022_ruby_excludes_rt_rp_from_inline_flow() {
    use std::collections::HashMap;
    use zero_dom::parse_html;

    let doc = parse_html("<div><ruby><rb>Fi</rb><rt>\u{25CF}\u{25CF}</rt><rp>(\u{6ce8})</rp></ruby>l</div>");
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    let div = doc.first_child(body).unwrap();

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, div, &HashMap::new());

    let all_text: String = ctx.all_fragments().iter().map(|f| f.text.clone()).collect();
    assert!(
        !all_text.contains('\u{25CF}'),
        "rt 文本 ● 不应出现在 inline 流（实测 {all_text:?}）"
    );
    assert!(
        !all_text.contains('\u{6ce8}'),
        "rp 文本「注」不应出现在 inline 流（实测 {all_text:?}）"
    );
    assert!(
        all_text.contains("Fi") && all_text.contains('l'),
        "rb 文本 + 尾文本应保留在 inline 流（实测 {all_text:?}）"
    );
}
