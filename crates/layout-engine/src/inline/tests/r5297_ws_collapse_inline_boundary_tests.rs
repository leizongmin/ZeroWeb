//! R5297（css-text-3 §4.1 白空折叠跨 inline 边界）：上一内容词以可折叠空白
//! 收尾时，后继 run 的前导空白与之同属一个空白序列——只保留前者 advance。
//!
//! 修复前：词臂前导空格门只看「上一 run 是否纯空白 run」（last_was_collapsible_ws），
//! 内容词尾随空格（trailing_space_width > 0）不同步该状态——`z z <em> x` 双计
//! 2 空格（c42-ibx-ht-000 实证 em 首词右移一字宽、全行错位；chromium 1 空格）。
//!
//! 修复后：门增加 `!last_ended_collapsible_ws`（词臂按 trailing_space_width 维护），
//! 跨 inline 边界空白序列折叠为单空格；纯空白 run 路径（last_was_collapsible_ws）
//! 与行首语义不变。

use super::super::*;

fn run(text: &str) -> TextRun {
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
        is_ahem_font: false,
        font_id: None,
        is_rtl: false,
        bidi_override: None,
        is_plaintext_bidi: false,
        ws_override: None,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

/// 单行场景末片段 x（单词无空白，x 即净 advance）。
fn last_frag_x(runs: Vec<TextRun>) -> f32 {
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "锚场景应单行");
    ctx.lines[0].runs.last().expect("至少一个片段").x
}

/// 主判例：`z ` + ` x`（跨 run 边界双空白）应折叠为单空格——
/// 末词 x 与 `z` + ` x`（单空格锚）等 x。修复前双空格多一个字宽。
#[test]
fn r5297_leading_ws_collapses_with_prev_trailing_ws_across_runs() {
    let one_space = last_frag_x(vec![run("z"), run(" x")]);
    let collapsed = last_frag_x(vec![run("z "), run(" x")]);
    assert!(
        (collapsed - one_space).abs() < 0.01,
        "跨边界双空白应折叠单空格（x={one_space}），实得 {collapsed}"
    );
}

/// 对照守卫：上一词无尾随空白时前导空白照常计 advance（门不得卡死为真）——
/// `zz` + ` x` 与 `zz` + `x` 应差一个空格 advance。
#[test]
fn r5297_leading_ws_kept_when_prev_word_has_no_trailing_ws() {
    let with_ws = last_frag_x(vec![run("zz"), run(" x")]);
    let without = last_frag_x(vec![run("zz"), run("x")]);
    assert!(
        with_ws - without > 1.0,
        "前导空白应计一次 advance，实得 Δ={}",
        with_ws - without
    );
}

/// 纯空白 run 路径不变：`z` + ` `（纯空白 run）+ ` x` 仍单空格
///（last_was_collapsible_ws 原门保留）。
#[test]
fn r5297_pure_ws_run_path_unchanged() {
    let one_space = last_frag_x(vec![run("z"), run(" x")]);
    let via_ws_run = last_frag_x(vec![run("z"), run(" "), run(" x")]);
    assert!(
        (via_ws_run - one_space).abs() < 0.01,
        "纯空白 run 折叠应保持单空格（x={one_space}），实得 {via_ws_run}"
    );
}
