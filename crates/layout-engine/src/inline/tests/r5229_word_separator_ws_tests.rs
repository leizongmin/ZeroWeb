//! R5229（css-text-3 §8 word-separator）：word-spacing 施于分隔字符自身 advance，
//! 与 paint 侧 per-char 口径（text.rs is_word_separator(ch) += frag_word_spacing）
//! 对齐。旧模型（R1086 lead_gap 承载 ws）在四处丢失：跨 run 边界空格（前 run 尾词 +
//! 后 run 首词 word_idx==0 无 lead_gap）、纯空白 run、run 首可折叠空格；同 run 词间
//! 净 advance 不变（ws 由词尾空格臂承载，lead_gap 项撤销）。corpus 驱动案
//! c541-word-sp-001 div4/div5（chromium CDP 探针逐字符真值）。

use super::super::*;

fn run(text: &str, word_spacing: f32) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id: NodeId::default(),
        font_size: 25.0,
        line_height: 25.0,
        vertical_align: VerticalAlignValue::Baseline,
        letter_spacing: 0.0,
        word_spacing,
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

/// 后继首词片段 x（单行单词场景，取最后一段）。
fn second_run_x(runs: Vec<TextRun>) -> f32 {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "锚场景应单行");
    let frags = &ctx.lines[0].runs;
    frags.last().expect("至少一个片段").x
}

/// 跨 run 边界空格的 ws：run1 "x "（ws=W）+ run2 "y"（ws=0）——
/// 空格归属 run1，其 ws 应计入空格 advance（chromium 探针 SP w=50.02）。
/// 修前边界空格丢 ws（c541 div4 span 左移 25）。
#[test]
fn r5229_boundary_space_carries_word_spacing() {
    let base = second_run_x(vec![run("x ", 0.0), run("y", 0.0)]);
    let spaced = second_run_x(vec![run("x ", 10.0), run("y", 0.0)]);
    assert!(
        (spaced - base - 10.0).abs() < 0.01,
        "跨 run 边界空格应携带 run1 的 ws（Δx=10），实得 Δ={}",
        spaced - base
    );
}

/// 同 run 词间净 advance 不变（防双计回归守卫）：run "x y"（ws=W）的
/// 词间 gap = 空格 advance + W，与旧 lead_gap 口径等值（R1086 形态保持）。
#[test]
fn r5229_intra_run_gap_net_unchanged() {
    let base = second_run_x(vec![run("x y", 0.0)]);
    let spaced = second_run_x(vec![run("x y", 10.0)]);
    assert!(
        (spaced - base - 10.0).abs() < 0.01,
        "同 run 词间 gap 应恰增 ws（净 advance 不变、无双计），实得 Δ={}",
        spaced - base
    );
}

/// run 首可折叠空格的 ws：run1 "x" + run2 " y"（ws=W）——空格归属 run2，
/// 计 run2 的 ws。修前丢 ws（c541 div5 前导空格 25 应 0 形态）。
#[test]
fn r5229_leading_space_carries_word_spacing() {
    let base = second_run_x(vec![run("x", 0.0), run(" y", 0.0)]);
    let spaced = second_run_x(vec![run("x", 0.0), run(" y", 10.0)]);
    assert!(
        (spaced - base - 10.0).abs() < 0.01,
        "run 首可折叠空格应携带 run2 的 ws（Δx=10），实得 Δ={}",
        spaced - base
    );
}

/// 纯空白 run 的 ws（run 间空格载体）：run1 "x" + run2 " "（ws=W）+ run3 "y"——
/// 折叠单空格计 run2 的 ws（R4233 ls 同位对称）。
#[test]
fn r5229_whitespace_only_run_carries_word_spacing() {
    let base = second_run_x(vec![run("x", 0.0), run(" ", 0.0), run("y", 0.0)]);
    let spaced = second_run_x(vec![run("x", 0.0), run(" ", 10.0), run("y", 0.0)]);
    assert!(
        (spaced - base - 10.0).abs() < 0.01,
        "纯空白 run 应携带自身 ws（Δx=10），实得 Δ={}",
        spaced - base
    );
}
