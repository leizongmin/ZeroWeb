//! R5325（css-text-4 §text-wrap-balance）：`text-wrap: balance` 断点重排——纯文本段
//! 保持贪心行数、词间重排最小化行宽参差（min-Σrendered² DP）。
//!
//! 实现前：layout 不 balance（tree.rs container_has_balancing_text_wrap 同口径注释），
//! balance 段按贪心断行——balance-002 渲染与无 balance mis-ref 逐像素同像
//!（mismatch 需 diff > 0，恒红）。实现后：balance-002 贪心 [380,60] 重排 [220,220]；
//! balance-004 强制断行分组独立均衡。非文本段（原子/浮动/Br 外形态）与贪心已参差
//! 最优的段零扰动（scope 门 + 非严格更优不改断点）。

use super::super::*;

/// 铸 n 个互异 node id。
fn ids(n: usize) -> Vec<NodeId> {
    let mut doc = zero_dom::Document::new();
    (0..n).map(|_| doc.create_element("span")).collect()
}

/// Ahem 精确字宽 run（1 字/空格 advance 恰 = font_size；行高 32）。
fn ahem_run(text: &str, node_id: NodeId, font_size: f32) -> TextRun {
    TextRun {
        text: text.to_string(),
        node_id,
        font_size,
        line_height: 32.0,
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
        ws_override: None,
        ruby_rt_ascent: 0.0,
        glyph_ascent: 0.0,
        glyph_descent: 0.0,
    }
}

/// 行盒可视右缘（最后 fragment x + width；空行 = 0）。
fn line_right_edge(line: &LineBox) -> f32 {
    line.runs.last().map(|f| f.x + f.width).unwrap_or(0.0)
}

/// 行盒 fragment (x, width) 序列快照（断点位/置位全等比较用）。
fn line_shape(line: &LineBox) -> Vec<(f32, f32)> {
    line.runs.iter().map(|f| (f.x, f.width)).collect()
}

// 主判例（balance-002 镜像）：20ch Ahem 容器 400px，「123 567 901 345 789 123」
// 贪心 [380, 60] → balance 重排 [220, 220]（3+3 词，行宽参差最小）。
#[test]
fn r5325_balance_redistributes_breaks_min_raggedness() {
    let id = ids(1);
    let mut ctx = InlineFormattingContext::new(400.0).with_balance(true);
    ctx.break_items_into_lines(vec![InlineItem::Text(ahem_run("123 567 901 345 789 123", id[0], 20.0))]);
    assert_eq!(ctx.lines.len(), 2, "balance 不改变行数（贪心 2 行）");
    let right: Vec<f32> = ctx.lines.iter().map(line_right_edge).collect();
    assert!(
        right.iter().zip([220.0, 220.0]).all(|(a, b)| (a - b).abs() < 1e-3),
        "R5325: balance 应重排为 [220, 220]（贪心 [380, 60]），实际 {right:?}"
    );
}

// 强制断行分组（balance-004 镜像）：Br 分隔的段独立均衡——组 1 贪心 [220, 40]
// 在同 2 行内重排为参差最优 [100, 160]，组 2（单词条）不参与跨组重排。
#[test]
fn r5325_balance_groups_forced_breaks_separately() {
    let id = ids(2);
    let mut ctx = InlineFormattingContext::new(240.0).with_balance(true);
    ctx.break_items_into_lines(vec![
        InlineItem::Text(ahem_run("aa bb cc dd ee", id[0], 20.0)),
        InlineItem::Br,
        InlineItem::Text(ahem_run("x", id[1], 20.0)),
    ]);
    let right: Vec<f32> = ctx.lines.iter().map(line_right_edge).collect();
    assert_eq!(right.len(), 3, "Br 分组后组 1 两行 + 组 2 一行");
    assert!(
        right
            .iter()
            .zip([100.0, 160.0, 20.0])
            .all(|(a, b)| (a - b).abs() < 1e-3),
        "R5325: 组 1 应重排 [100, 160]（贪心 [220, 40]）、组 2 独立单行，实际 {right:?}"
    );
}

// 已参差最优段零扰动（balance-align-001 同形）：贪心 [340, 340] 即最小参差，
// balance 开关两侧断行逐 fragment 全等（非严格更优不改断点）。
#[test]
fn r5325_balance_noop_when_greedy_already_even() {
    let id = ids(1);
    let items = vec![InlineItem::Text(ahem_run(
        "123456789 123 567 123456789 123 567",
        id[0],
        20.0,
    ))];
    let mut plain = InlineFormattingContext::new(400.0);
    plain.break_items_into_lines(items.clone());
    let mut balanced = InlineFormattingContext::new(400.0).with_balance(true);
    balanced.break_items_into_lines(items);
    let plain_shape: Vec<Vec<(f32, f32)>> = plain.lines.iter().map(line_shape).collect();
    let balanced_shape: Vec<Vec<(f32, f32)>> = balanced.lines.iter().map(line_shape).collect();
    assert_eq!(plain_shape, balanced_shape, "贪心已参差最优时 balance 零扰动");
}

// scope 回退控制（overflow-wrap: break-word）：break-word 语境不在规划面
//（词中逐字符断行未被 span 模型覆盖），balance 开关两侧行为全等。
#[test]
fn r5325_balance_scope_fallback_break_word() {
    let id = ids(1);
    let items = vec![InlineItem::Text(ahem_run("CONTROLLING YOUR BU BU", id[0], 20.0))];
    let mut plain = InlineFormattingContext::new(200.0).with_break_word(true);
    plain.break_items_into_lines(items.clone());
    let mut balanced = InlineFormattingContext::new(200.0)
        .with_break_word(true)
        .with_balance(true);
    balanced.break_items_into_lines(items);
    let plain_shape: Vec<Vec<(f32, f32)>> = plain.lines.iter().map(line_shape).collect();
    let balanced_shape: Vec<Vec<(f32, f32)>> = balanced.lines.iter().map(line_shape).collect();
    assert_eq!(plain_shape, balanced_shape, "break-word 语境回退贪心");
}

// 单行段零扰动：贪心 1 行（N=1 无重排面），balance 下不新造断行。
#[test]
fn r5325_balance_single_line_stays_one_line() {
    let id = ids(1);
    let mut ctx = InlineFormattingContext::new(400.0).with_balance(true);
    ctx.break_items_into_lines(vec![InlineItem::Text(ahem_run("123 567 901", id[0], 20.0))]);
    assert_eq!(ctx.lines.len(), 1, "可容纳单行时 balance 不增行");
    assert!((line_right_edge(&ctx.lines[0]) - 220.0).abs() < 1e-3);
}

// 多行段重排（balance-004 group1 镜像，Ahem 8px ≡ monospace 8px/字、50ch=400px）：
// 贪心 5 行 [400,384,384,392,256] 参差大，balance 在同 5 行内重排为 min-Σrendered²
// [312,392,352,376,384]（首行让出 adipiscing，尾行收拢短词）。
#[test]
fn r5325_balance_multiline_reduces_raggedness() {
    let id = ids(1);
    let mut plain = InlineFormattingContext::new(400.0);
    plain.break_items_into_lines(vec![InlineItem::Text(ahem_run(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
        id[0],
        8.0,
    ))]);
    let greedy: Vec<f32> = plain.lines.iter().map(line_right_edge).collect();
    assert_eq!(greedy.len(), 5, "贪心 5 行是重排前提");
    let mut ctx = InlineFormattingContext::new(400.0).with_balance(true);
    ctx.break_items_into_lines(vec![InlineItem::Text(ahem_run(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
        id[0],
        8.0,
    ))]);
    let right: Vec<f32> = ctx.lines.iter().map(line_right_edge).collect();
    assert_eq!(right.len(), 5, "balance 不改变行数");
    let cost = |ws: &[f32]| -> f64 { ws.iter().map(|w| f64::from(*w) * f64::from(*w)).sum() };
    assert!(
        cost(&right) + 0.01 < cost(&greedy),
        "balance 应严格降低参差（贪心 {greedy:?} → 实际 {right:?}）"
    );
    assert!(
        right
            .iter()
            .zip([312.0, 392.0, 352.0, 376.0, 384.0])
            .all(|(a, b)| (a - b).abs() < 1e-3),
        "R5325: 5 行重排应为 [312, 392, 352, 376, 384]，实际 {right:?}"
    );
}
