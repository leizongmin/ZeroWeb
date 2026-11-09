//! R5449（css-overflow-4 §block-ellipsis / css-text-3 §3.1.4 hangable whitespace）：
//! line-clamp 截断生效后，末保留行的 hangable 尾随空格不进 ellipsis 前内容——
//! 纯空白 run 清零（无字形零推进）、混合 run 剪尾随空格并收缩 width。
//! kill-switch `ZW_CLAMP_HANG_TRIM`（=0 回退不剪；独立进程钉见
//! tests/r5449_clamp_hang_trim_kill_switch.rs，引擎级 032 形态钉见
//! tests/r5449_clamp_hang_trim.rs）。

use crate::inline::{InlineFormattingContext, TextRun};
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

/// 单 TextRun（缺省 16px 常规字体）。
fn run(text: &str) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 16.0,
        line_height: 20.0,
        vertical_align: VA::Baseline,
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
        is_isolated_bidi: false,
        ws_override: None,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

/// pre-wrap + 强制断行 IFC：`"aa \nbb \nc\n "` 三行，各行以保留空格片段收尾
///（强制断行使行构成与宽度估计无关，断言确定性来源）。
fn trailing_space_lines() -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true);
    ctx.break_into_lines(vec![run("aa \nbb \ncc ")]);
    ctx
}

/// 截断后末保留行的纯空白 run 清零（032 红带主臂：pre-wrap 悬挂空格 span）。
#[test]
fn clamped_last_line_pure_space_run_zeroed() {
    let mut ctx = trailing_space_lines();
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    assert!(
        ctx.lines[1].runs.last().is_some_and(|f| f.text == " "),
        "前置：L2 以纯空格片段收尾"
    );
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    assert_eq!(ctx.lines.len(), 2, "clamp:2 保留 2 行");
    let runs = &ctx.lines.last().unwrap().runs;
    assert!(
        runs.iter().any(|f| f.text == "bb"),
        "内容词保留：{:?}",
        runs.iter().map(|f| &f.text).collect::<Vec<_>>()
    );
    assert!(
        runs.iter().all(|f| !f.text.ends_with(' ')),
        "末保留行不再以空格收尾（纯空白 run 清零）：{:?}",
        runs.iter().map(|f| &f.text).collect::<Vec<_>>()
    );
    let zeroed = runs
        .iter()
        .find(|f| f.text.is_empty())
        .expect("被清零的原空格片段仍在行上");
    assert_eq!(zeroed.width, 0.0, "清零片段零推进（红底 advance 消失）");
}

/// 截断后末保留行混合 run（词 + 尾随空格同片段）剪尾随空格并收回宽度。
///
/// split 恒把保留空格独立成片段，混合形只能来自其他合成路径——直接改写词片段
/// 追加尾随空格以钉混合臂语义。
#[test]
fn clamped_last_line_mixed_run_trailing_spaces_trimmed() {
    let mut ctx = trailing_space_lines();
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    let space_advance = ctx.advance_of(' ', None, 16.0, false);
    let word_width = ctx.lines[1].runs[0].width;
    assert_eq!(ctx.lines[1].runs[0].text, "bb", "前置：L2 首片段为词");
    {
        let frag = &mut ctx.lines[1].runs[0];
        frag.text.push_str("  ");
        frag.width += 2.0 * space_advance;
    }
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    let runs = &ctx.lines.last().unwrap().runs;
    let word = runs.iter().find(|f| f.text.starts_with("bb")).expect("bb 保留");
    assert_eq!(word.text, "bb", "混合 run 剪尾随空格，实得 {:?}", word.text);
    assert!(
        (word.width - word_width).abs() < 0.01,
        "width 收回双空格 advance：{} 应回 {}",
        word.width,
        word_width
    );
    assert!(
        runs.iter().all(|f| !f.text.ends_with(' ')),
        "同行的纯空白 run 同臂清零：{:?}",
        runs.iter().map(|f| &f.text).collect::<Vec<_>>()
    );
}

/// 负向守卫：无 clamp（内容不溢出）不剪——尾随空格片段原样保留。
#[test]
fn no_clamp_keeps_trailing_spaces() {
    let ctx = trailing_space_lines();
    assert!(!ctx.clamped, "未截断");
    for (i, line) in ctx.lines.iter().enumerate() {
        assert!(
            line.runs.last().is_some_and(|f| f.text.ends_with(' ')),
            "L{i} 未 clamp：行尾空格片段不剪，实得 {:?}",
            line.runs.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
    }
}
