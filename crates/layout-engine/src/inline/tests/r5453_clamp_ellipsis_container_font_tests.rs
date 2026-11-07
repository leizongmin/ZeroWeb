//! R5453（css-overflow-4 §block-ellipsis）：clamp 末保留行再对齐的 ellipsis
//! advance 度量源三臂——paint Path B 注入值 `clamp_ellipsis_advance` 直取（R5453②，
//! 与 paint ellipsis 截断门同表达式）、容器 face 捕获（`container_font_id`，
//! `layout()` 容器 styles 在手处捕获，R5453①）、estimate 回退——旧
//! `advance_of('…', None)` 恒回 estimate：runner 环境 mono 真面全角 16px vs
//! estimate 半宽 8px，预留差把末保留字符挤进 paint cut-back 臂被清零
//!（block-ellipsis-032 末字符 'd' 截失实证）。paint 侧 ellipsis 定位/截断门按
//! 容器 face/字号/Ahem 判定测宽（text.rs `measure_char_cached(default_font_id,
//! '…')`），layout 预留须同源。注入源按 font_id 区分度量，验证 realign 预留消费
//! `Some(container_font_id)` 真面值而非 estimate；None（Path B/estimate 环境）仍
//! 回 estimate 零回归。runner 环境端到端由 corpus block-ellipsis-032 双窗 A/B
//! 判定（estimate 环境无 FontLoader，不可复现真面 advance）。

use crate::inline::text_metrics::AdvanceSource;
use crate::inline::{InlineFormattingContext, TextAlign, TextRun};
use std::rc::Rc;
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

/// 按规则区分度量的注入源：font_id Some(7) 时 '…' = 16.0（真面全角），其余字符
/// estimate；None/其他 face = estimate（镜像 ShapedAdvanceSource 的 font_id 分派）。
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

/// pre-wrap + 强制断行 IFC：三行（强制断行使行构成与宽度估计无关），末行以保留
/// 空格片段收尾，注入 face 区分 advance 源。
fn trailing_space_lines(align: TextAlign) -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_align(align)
        .with_advance_source(Rc::new(FaceEllipsisAdvance));
    ctx.container_font_id = Some(7);
    ctx.break_into_lines(vec![run("aa \nbb \ncc ")]);
    ctx
}

/// right + `container_font_id` Some：trim 后整行 Δ 位移使 kept + ellipsis 合宽
/// 右对齐——ellipsis 预留 = 注入源真面 16.0（行起点 400 − 17.6 − 16 = 366.4），
/// 非 estimate 8（374.4）。Δ = 366.4 − 0（kept 首片段 pre-alignment 原点）。
#[test]
fn right_realign_reserves_container_face_ellipsis_advance() {
    let mut ctx = trailing_space_lines(TextAlign::Right);
    ctx.apply_line_clamp_cap(2);
    let last = ctx.lines.last().expect("kept 行");
    let kept: Vec<_> = last.runs.iter().filter(|f| !f.text.is_empty()).collect();
    assert_eq!(kept.len(), 1, "前置：末行 kept = \"cc\" 单片段");
    let first_x = kept[0].x;
    let kept_w = kept[0].width;
    assert!(
        (first_x - (400.0 - kept_w - 16.0)).abs() < 0.5,
        "行起点应按真面 ellipsis 16 预留 = {}，实得 {first_x}（estimate 8 则 {}）",
        400.0 - kept_w - 16.0,
        400.0 - kept_w - 8.0
    );
}

/// `container_font_id` None（Path B / 无法解析）：预留回 estimate 8，旧语义不变。
#[test]
fn right_realign_falls_back_to_estimate_without_container_font() {
    let mut ctx = trailing_space_lines(TextAlign::Right);
    ctx.container_font_id = None;
    ctx.apply_line_clamp_cap(2);
    let last = ctx.lines.last().expect("kept 行");
    let kept: Vec<_> = last.runs.iter().filter(|f| !f.text.is_empty()).collect();
    let first_x = kept[0].x;
    let kept_w = kept[0].width;
    assert!(
        (first_x - (400.0 - kept_w - 8.0)).abs() < 0.5,
        "无容器 face 时应回 estimate 8 预留 = {}，实得 {first_x}",
        400.0 - kept_w - 8.0
    );
}

/// paint Path B 注入值直取（R5453②）：`clamp_ellipsis_advance` Some 时优先于
/// 容器 face 捕获/estimate——注入与 paint ellipsis 截断门同表达式，预留与截断门
/// 逐位同源（runner 环境 Path B 空 styles 捕获臂空转的修复臂）。
#[test]
fn right_realign_prefers_injected_ellipsis_advance() {
    let mut ctx = trailing_space_lines(TextAlign::Right);
    ctx.container_font_id = None;
    ctx.clamp_ellipsis_advance = Some(16.0);
    ctx.apply_line_clamp_cap(2);
    let last = ctx.lines.last().expect("kept 行");
    let kept: Vec<_> = last.runs.iter().filter(|f| !f.text.is_empty()).collect();
    let first_x = kept[0].x;
    let kept_w = kept[0].width;
    assert!(
        (first_x - (400.0 - kept_w - 16.0)).abs() < 0.5,
        "注入值 16 应直取预留 = {}，实得 {first_x}",
        400.0 - kept_w - 16.0
    );
}
