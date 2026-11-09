//! R5454（css-overflow-4 §block-ellipsis）：justify 容器 clamp 末保留行再对齐把
//! ellipsis 计入 justify 分配项——gap_count = kept_count（末 gap 落 ellipsis 槽位
//! 前，与 run 间隔同宽 expansion），并置旗标 `clamp_ellipsis_justify_flush`（paint
//! ellipsis 定位落线尾 [line_limit − ell, line_limit) 的盒旗标镜像源，随
//! `clamp_strut_applied` 同站点镜像）。032 box3 数值定谳：ref `text-align-last:
//! justify` 行（'…' 为独立分配项，正常 justify pass runs 含 ellipsis）remaining
//! = 42.1、gap_count = 4，与 clamp realign D/k 逐位同值；旧 gap_count = kept−1
//! 口径差 1/gap_count 级（每 gap 13.3 vs 10.5，词位累积偏 +3.3/j）。

use crate::inline::text_metrics::AdvanceSource;
use crate::inline::{InlineFormattingContext, TextAlign, TextRun};
use std::rc::Rc;
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

/// 按规则区分度量的注入源（同 r5453 套路）：font_id Some(7) 时 '…' = 16.0（真面
/// 全角），其余字符 estimate；None/其他 face = estimate。
struct FaceEllipsisAdvance;

impl AdvanceSource for FaceEllipsisAdvance {
    fn measure(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        if ch == '\u{2026}' && font_id == Some(7) {
            return 16.0;
        }
        crate::inline::estimate_char_width(ch, font_size, is_ahem)
    }
}

/// 单 TextRun（缺省 16px 常规字体；estimate 度量：字母 8.8 / 空格 4 / '…' 8）。
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

/// pre-wrap + 强制断行 IFC：三行，cap 2 后末保留行 = 第 2 行（cap 前为中间行，
/// 行内已被 justify 均布——与真实管线一致）；kept 含词间纯空白 run（justify 臂
/// kept ≥ 2 门），行尾悬挂空格触发 trim + realign；容器 face id Some(7) 使 '…'
/// 预留走真面 16.0。
fn justify_two_kept_lines(align: TextAlign) -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_align(align)
        .with_advance_source(Rc::new(FaceEllipsisAdvance));
    ctx.container_font_id = Some(7);
    ctx.break_into_lines(vec![run("aa bb \ncc dd \nee ff ")]);
    ctx
}

/// justify realign：ellipsis 计入分配项（gap_count = kept_count）——末 gap 落
/// ellipsis 槽位前，与 run 间隔同宽 expansion；各 kept run 位移 +extra×j（含词间
/// 纯空白 run，位移按 run 下标）。旧口径（gap_count = kept − 1）末 gap = 0（kept
/// 紧贴 ellipsis 槽位起点）。
#[test]
fn justify_realign_counts_ellipsis_as_final_item() {
    let mut ctx = justify_two_kept_lines(TextAlign::Justify);
    // cap 前快照末保留行（cap 2 → 下标 1）：每 run (x, width, 尾随空格数)；
    // kept 口径与臂同源，mixed run 尾空格 advance 先收缩（trim，R5450 同式）再入
    // kept_w。
    let kept_runs = &ctx.lines[1].runs;
    let pre: Vec<(f32, f32, usize)> = kept_runs
        .iter()
        .map(|r| (r.x, r.width, r.text.chars().rev().take_while(|&c| c == ' ').count()))
        .collect();
    let (pure_count, mixed_index) = InlineFormattingContext::scan_hangable_tail(kept_runs);
    let space_adv = ctx.advance_of(' ', None, 16.0, false);
    let mut w_pre: Vec<f32> = pre.iter().map(|p| p.1).collect();
    if let Some(i) = mixed_index {
        w_pre[i] -= pre[i].2 as f32 * space_adv;
    }
    ctx.apply_line_clamp_cap(2);
    let runs = &ctx.lines.last().unwrap().runs;
    let kept_count = pre.len() - pure_count;
    assert!(kept_count >= 2, "前置：kept ≥ 2（justify 臂门），实得 {kept_count}");
    let kept_w = pre[kept_count - 1].0 + w_pre[kept_count - 1] - pre[0].0;
    let extra = (400.0 - kept_w - 16.0) / kept_count as f32;
    let extra_old = (400.0 - kept_w - 16.0) / (kept_count as f32 - 1.0);
    for (j, r) in runs.iter().enumerate().take(kept_count) {
        let want = pre[j].0 + extra * j as f32;
        assert!(
            (r.x - want).abs() < 0.5,
            "kept run {j} 应位移 +extra×{j}（ellipsis 末项口径 extra = D/{kept_count}）落 {want}，实得 {}（旧口径 extra = D/{} 则 {}）",
            r.x,
            kept_count - 1,
            pre[j].0 + extra_old * j as f32
        );
    }
    let last_end = runs[kept_count - 1].x + runs[kept_count - 1].width;
    let end_gap = (400.0 - 16.0) - last_end;
    assert!(
        (end_gap - extra).abs() < 0.5,
        "ellipsis 槽位前末 gap 应与 run 间隔同宽 expansion = {extra}，实得 {end_gap}（旧口径紧贴槽位起点 = 0）"
    );
}

/// realign 触发（justify + kept ≥ 2）置旗标 `clamp_ellipsis_justify_flush`——paint
/// ellipsis 定位落线尾的盒旗标镜像源。
#[test]
fn justify_realign_sets_ellipsis_flush_flag() {
    let mut ctx = justify_two_kept_lines(TextAlign::Justify);
    ctx.apply_line_clamp_cap(2);
    assert!(
        ctx.clamp_ellipsis_justify_flush,
        "justify realign 触发须置旗标（paint 线尾定位镜像源）"
    );
}

/// 非 justify（right realign 臂）不置旗标——线尾定位仅 justify 分配语义需要
///（right 落位 kept+ellipsis 合宽右对齐，ellipsis 本就紧贴槽位起点，paint 默认
/// 相邻定位与之重合）。
#[test]
fn right_realign_does_not_set_ellipsis_flush_flag() {
    let mut ctx = justify_two_kept_lines(TextAlign::Right);
    ctx.apply_line_clamp_cap(2);
    assert!(
        !ctx.clamp_ellipsis_justify_flush,
        "right 臂不得置旗标（线尾定位仅 justify 分配语义需要）"
    );
}
