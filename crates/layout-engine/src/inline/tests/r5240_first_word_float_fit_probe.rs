//! R5240：行首词放不下 float 缩短带 → 行位下移（CSS2 §9.5 rule 8）锚。
//!
//! fit 门 `!current_line.runs.is_empty()` 守卫使行首词永不折行——float 缩短首行带后
//! 行首词强放溢出（floats-132：container 160px、float 20×20、Ahem 词 8×20=160px；
//! 首行带 140 < 160 应下移到 float 带底，第二行满宽 160 恰好容纳；旧实现强放首行
//! x=20 溢出、行位 y=0，container 红底露出 ✗1.33%）。R4234 已为行首原子盒建同款
//! 下绕（shift_new_line_below_floats），本刀补齐行首词臂。案驱动：
//! css/CSS2/floats-132.xht。
//!
//! kill-switch（`ZW_FIRST_WORD_FLOAT_FIT=0`）反证不由单测承担：flag 走
//! selected 快照路径，进程内 set_var 不可靠（R3858 教训）——由 corpus OFF 轮覆盖。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue;

fn ahem_word(text: &str) -> TextRun {
    let node = zero_dom::Document::new().create_text_node("");
    let mut run = TextRun::simple(text.to_string(), node, 20.0, 20.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run
}

/// floats-132 形态：160px 词在 float 缩短的 140px 首行带放不下 → 行下移到 float
/// 带底（y=20），满宽 160 恰好容纳，行首 x 回到 0。
#[test]
fn first_word_shifted_below_float_band_when_shortened_line_cannot_fit() {
    let mut ctx = InlineFormattingContext::new(160.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 20.0,
        width: 20.0,
        x: 0.0,
        is_left: true,
    }]);
    ctx.break_into_lines(vec![ahem_word("xxxxxxxx")]);

    assert_eq!(ctx.lines.len(), 1, "词应单行（下移后满宽容纳）");
    assert!(
        (ctx.lines[0].y - 20.0).abs() < 0.5,
        "行应下移到 float 带底 y=20，实际 {}",
        ctx.lines[0].y
    );
    assert!(
        (ctx.lines[0].runs[0].x - 0.0).abs() < 0.5,
        "行首 x 应对齐下移后带左缘 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 守卫（镜像 R4234 场景②）：排除带与本行带无重叠（y=500）→ 纯容器溢出不推、
/// 行首停留带左缘 0，防「无 float 环绕被误下移」。
#[test]
fn no_overlap_float_band_keeps_old_placement() {
    let mut ctx = InlineFormattingContext::new(160.0).with_float_exclusions(vec![FloatExclusion {
        y: 500.0,
        height: 20.0,
        width: 20.0,
        x: 0.0,
        is_left: true,
    }]);
    ctx.break_into_lines(vec![ahem_word("xxxxxxxxxx")]);

    assert_eq!(ctx.lines.len(), 1);
    assert!(
        (ctx.lines[0].y - 0.0).abs() < 0.5,
        "守卫：无重叠带不下移，实际 y={}",
        ctx.lines[0].y
    );
    assert!(
        (ctx.lines[0].runs[0].x - 0.0).abs() < 0.5,
        "守卫：行首 x 应停留 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}
