//! R5323（css-text-3 §4.1 phase-1 collapse 跨 inline 边界）：纯空白 run 的空格与
//! 上一内容词尾随空格同属一个空白序列——只保留前者 advance。
//!
//! 修复前：纯空白 run 臂门只看「上一 run 是否纯空白 run」（last_was_collapsible_ws），
//! 内容词尾随空格（trailing_space_width，R5297 词臂只覆盖了后继 run 的**词内前导**
//! 空格）不同步该状态——`<span>X </span> <span>X </span>` 词间双计 2 空格
//!（white-space-007 每行 4 X 共 3 行 vs chromium 折叠单空格 5 X 共 2 行）。
//!
//! 修复后：门增加 `!last_ended_collapsible_ws`（纯空白 run 同吞 advance，preserve
//! run 不参与）；owner/no_wrap 状态照常更新——断行机会治理（R5115 经 last_ws_no_wrap）
//! 维持「序列最后一个空格所属 run」口径，折行位不变、仅间距归一。

use super::super::*;

/// 铸 n 个互异 node id（boundary_ws_break 治理依赖 owner 互异）。
fn ids(n: usize) -> Vec<NodeId> {
    let mut doc = zero_dom::Document::new();
    (0..n).map(|_| doc.create_element("span")).collect()
}

fn run_with(text: &str, node_id: NodeId, ws: Option<RunWhiteSpace>) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id,
        font_size: 50.0,
        line_height: 50.0,
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
        ws_override: ws,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

fn nowrap() -> Option<RunWhiteSpace> {
    Some(RunWhiteSpace {
        preserve: false,
        break_at_newline: false,
        no_wrap: true,
        hang_trailing: false,
    })
}

/// 单行场景末片段 x（单词无空白，x 即净 advance，Ahem 50px 字号下 1 字/空格 = 50）。
fn last_frag_x(runs: Vec<TextRun>) -> f32 {
    let mut ctx = InlineFormattingContext::new(2000.0);
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 1, "锚场景应单行");
    ctx.lines[0].runs.last().expect("至少一个片段").x
}

/// 主判例：`z ` + ` `（纯空白 run）+ `x` 应折叠单空格——末词 x 与
/// `z ` + `x`（单空格锚）等 x。修复前双空格多一个字宽。
#[test]
fn r5323_pure_ws_run_collapses_with_prev_trailing_ws_across_runs() {
    let id = ids(3);
    let one_space = last_frag_x(vec![run_with("z ", id[0], None), run_with("x", id[1], None)]);
    let collapsed = last_frag_x(vec![
        run_with("z ", id[0], None),
        run_with(" ", id[2], None),
        run_with("x", id[1], None),
    ]);
    assert!(
        (collapsed - one_space).abs() < 0.01,
        "跨边界双空白应折叠单空格（x={one_space}），实得 {collapsed}"
    );
}

/// white-space-007 真镜像：`<span>X </span> <span>X </span>` nowrap span × normal
/// 容器（Ahem 50px，500px 宽）——折叠后每行 5 X（2 行）；修复前每行 4 X（3 行）。
/// 断行治理腿：序列最后空格（容器 normal）治理折行——nowrap 不得外溢禁止折行。
#[test]
fn r5323_ws007_mirror_nowrap_span_boundary_two_lines() {
    let id = ids(21);
    let mut runs = Vec::new();
    for i in 0..10 {
        if i > 0 {
            runs.push(run_with(" ", id[10 + i], None));
        }
        runs.push(run_with("X ", id[i], nowrap()));
    }
    let mut ctx = InlineFormattingContext::new(500.0);
    ctx.break_into_lines(runs);
    assert_eq!(ctx.lines.len(), 2, "应 2 行（每行 5 X），实得行数 {}", ctx.lines.len());
    for (li, line) in ctx.lines.iter().enumerate() {
        let words: Vec<_> = line.runs.iter().filter(|f| !f.text.is_empty()).collect();
        assert_eq!(words.len(), 5, "行 {li} 应 5 个 X 片段");
        // 行内 X 间距 = 折叠后单空格 50px（修复前 100px 双空格）。
        for pair in words.windows(2) {
            let gap = pair[1].x - (pair[0].x + pair[0].width);
            assert!(
                (gap - 50.0).abs() < 0.01,
                "行 {li} X 间距应 50px（折叠单空格），实得 {gap}"
            );
        }
    }
}

/// 对照守卫：上一词无尾随空白时纯空白 run advance 照常计（门不得卡死为真）——
/// `zz` + ` ` + `x` 与 `zz` + `x` 应差一个空格 advance。
#[test]
fn r5323_pure_ws_run_kept_when_prev_word_has_no_trailing_ws() {
    let id = ids(3);
    let with_ws = last_frag_x(vec![
        run_with("zz", id[0], None),
        run_with(" ", id[2], None),
        run_with("x", id[1], None),
    ]);
    let without = last_frag_x(vec![run_with("zz", id[0], None), run_with("x", id[1], None)]);
    assert!(
        with_ws - without > 1.0,
        "纯空白 run 应计一次 advance，实得 Δ={}",
        with_ws - without
    );
}

/// preserve run 空格不可折叠：preserve 语境的纯空白 run 不吞 advance（run_preserve 门）。
#[test]
fn r5323_preserve_pure_ws_run_not_collapsed() {
    let id = ids(3);
    let pre = RunWhiteSpace {
        preserve: true,
        break_at_newline: true,
        no_wrap: true,
        hang_trailing: false,
    };
    // preserve 尾空格词 + preserve 纯空白 run：空格均真实 advance（2 空格）。
    let two_spaces = last_frag_x(vec![
        run_with("z ", id[0], Some(pre)),
        run_with(" ", id[2], Some(pre)),
        run_with("x", id[1], Some(pre)),
    ]);
    // 正常折叠语境单空格锚。
    let one_space = last_frag_x(vec![run_with("z ", id[0], None), run_with("x", id[1], None)]);
    assert!(
        two_spaces - one_space > 1.0,
        "preserve 空格不参与折叠（Δ 应 ≥1 空格），实得 Δ={}",
        two_spaces - one_space
    );
}
