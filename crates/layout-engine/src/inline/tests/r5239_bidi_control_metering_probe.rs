//! R5239：bidi 控制码（U+202A-202E / U+2066-2069）呈现层零计量锚。
//!
//! UAX #9 X9 / css-writing-modes-3 #bidi-control-codes：控制码驱动重排后不产生
//! 盒子、不参与计量、不呈现字形。修复前 `BidiFragmentCursor` 视觉输出携带控制码
//! → estimate_char_width 计字宽 + letter-spacing 按 char_count 计费 + paint 绘出
//! 字形——bidi-005/006 test 段（直系文本+控制码构造）与 ref 段（连续 span 构造）
//! 恒差平移（✗1.35%，R5204 定谳 166px 量级）。案驱动：css/CSS2/bidi-005.xht。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue;

fn ahem_run(text: &str) -> TextRun {
    let node = zero_dom::Document::new().create_text_node("");
    let mut run = TextRun::simple(text.to_string(), node, 16.0, 16.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run.ws_override = Some(crate::inline::RunWhiteSpace {
        preserve: true,
        break_at_newline: false,
        no_wrap: false,
        hang_trailing: false,
    });
    run
}

/// 控制码零计量：含控制码 run 与等可见内容 run 的行宽一致（字宽+ls 双计费消灭）。
#[test]
fn bidi_control_char_contributes_zero_advance_and_spacing() {
    let mut with_ctrl = ahem_run("a\u{202E}b");
    with_ctrl.letter_spacing = 32.0;
    let mut plain = ahem_run("ab");
    plain.letter_spacing = 32.0;

    let mut ctx_ctrl = InlineFormattingContext::new(800.0);
    ctx_ctrl.break_into_lines(vec![with_ctrl]);
    let w_ctrl: f32 = ctx_ctrl.lines[0].runs.iter().map(|r| r.width).sum();

    let mut ctx_plain = InlineFormattingContext::new(800.0);
    ctx_plain.break_into_lines(vec![plain]);
    let w_plain: f32 = ctx_plain.lines[0].runs[0].width;

    assert!(
        (w_ctrl - w_plain).abs() < 0.5,
        "控制码须零计量：ctrl={w_ctrl} plain={w_plain}"
    );
}

/// 控制码不落入 fragment 文本（paint 消费端不再绘出字形）。
#[test]
fn bidi_control_char_absent_from_fragments() {
    let run = ahem_run("a\u{202E}b\u{202C}c");
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.break_into_lines(vec![run]);
    let text: String = ctx.lines[0].runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(text, "abc", "控制码须从视觉流剥除，实际 {text:?}");
}

/// RLO 反转语义不受剥除影响——控制码驱动的方向反转仍生效（levels 奇偶已消费）。
#[test]
fn rlo_reversal_semantics_survive_stripping() {
    // logical `ab[RLO]cde`：RLO 使 cde 反转 → 视觉序 cde 倒序（edc）。
    let run = ahem_run("ab\u{202E}cde");
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.bidi_override_direction = None;
    ctx.break_into_lines(vec![run]);
    let text: String = ctx.lines[0].runs.iter().map(|r| r.text.as_str()).collect();
    let (pc, pd, pe) = (
        text.find('c').expect("c 在视觉流"),
        text.find('d').expect("d 在视觉流"),
        text.find('e').expect("e 在视觉流"),
    );
    assert!(pe < pd && pd < pc, "RLO 反转语义须保留（edc 序），实际 {text:?}");
}
