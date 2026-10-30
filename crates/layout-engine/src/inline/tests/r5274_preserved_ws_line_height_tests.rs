//! R5274（CSS2 §9.2.1.1「preserved white space」+ css-pseudo-4 §highlight-painting）：
//! preserve 模式（pre/pre-wrap/break-spaces）下**纯空白行盒计 line-height**——R57
//! 「全可折叠空白塌 0 高」收尾判定未区分保留空白：tab 片段 text=""（R1447 宽度由
//! x 推进承载）、保留空格 text=" " 均被判塌 0 → pre 纯 tab 行的块/float 盒高 0 整盒
//! 不可见（active-selection-063 驱动：选中 tab 行高亮必须可见）。collapse 模式行为
//! 不变（R57 canvas-grid 形态锁死）。

use super::super::*;

fn pre_run(text: &str) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 25.0,
        line_height: 25.0,
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
        is_isolated_bidi: false,
        ws_override: Some(RunWhiteSpace {
            preserve: true,
            break_at_newline: true,
            no_wrap: true,
            hang_trailing: false,
        }),
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

fn collapse_run(text: &str) -> TextRun {
    let mut run = pre_run(text);
    run.ws_override = None;
    run
}

fn line_heights(runs: Vec<TextRun>) -> Vec<f32> {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(runs);
    ctx.lines.iter().map(|l| l.height).collect()
}

/// pre 纯 tab 行（R1447 tab 片段 text=""）→ 行高 = line-height，不塌 0
/// （active-selection-063 形：tab_size:4 × 25px → 行宽 100）。
#[test]
fn r5274_pre_tab_only_line_keeps_height() {
    let heights = line_heights(vec![pre_run("\t")]);
    assert_eq!(heights.len(), 1, "单 tab 应产出单行");
    assert!(
        (heights[0] - 25.0).abs() < 0.01,
        "pre 纯 tab 行应计 line-height 25，实得 {:?}",
        heights
    );
}

/// pre 纯空格行 → 行高 = line-height（§9.2.1.1 preserved white space）。
#[test]
fn r5274_pre_space_only_line_keeps_height() {
    let heights = line_heights(vec![pre_run(" ")]);
    assert_eq!(heights.len(), 1, "单空格应产出单行");
    assert!(
        (heights[0] - 25.0).abs() < 0.01,
        "pre 纯空格行应计 line-height 25，实得 {:?}",
        heights
    );
}

/// collapse 模式纯空白 run → 维持旧行为零高（R57 canvas-grid 形态不回退）。
#[test]
fn r5274_collapse_ws_only_line_still_zero_height() {
    let heights = line_heights(vec![collapse_run("   ")]);
    assert!(
        heights.iter().all(|&h| h == 0.0),
        "collapse 纯空白行应维持 0 高，实得 {:?}",
        heights
    );
}
