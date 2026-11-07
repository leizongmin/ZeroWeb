//! R5451（css-overflow-4 §block-ellipsis：ellipsis 计入末保留行参与对齐）：clamp
//! trim 生效后末保留行再对齐。对齐 pass 先于 clamp cap 执行且 content_width 含
//! 被剪 hang 宽（R1338 豁免是 IFC 级），kept-last 截断前恒非末行 → 对齐值取
//! text_align；right/center 整行附加位移、justify kept run 重均布且 cleared
//! tail 重折叠到末 kept run 新末端。引擎级端到端钉见
//! tests/r5451_clamp_hang_realign.rs；kill-switch 与 R5449/R5450 同门
//!（ZW_CLAMP_HANG_TRIM，OFF 钉见 r5450_clamp_hang_x_collapse_kill_switch.rs）。

use crate::inline::{InlineFormattingContext, TextAlign, TextRun};
use zero_css_parser::values::VerticalAlignValue as VA;
use zero_dom::NodeId;

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

/// pre-wrap + 强制断行 IFC：三行（强制断行使行构成与宽度估计无关，断言确定性
/// 来源），各行以保留空格片段收尾。
fn trailing_space_lines(align: TextAlign) -> InlineFormattingContext {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_align(align);
    ctx.break_into_lines(vec![run("aa \nbb \ncc ")]);
    ctx
}

/// 词片段（非空白文本）下标集合。
fn word_indices(runs: &[crate::inline::TextFragment]) -> Vec<usize> {
    runs.iter()
        .enumerate()
        .filter(|(_, f)| f.text.chars().any(|c| !c.is_whitespace()))
        .map(|(i, _)| i)
        .collect()
}

/// right：trim 后整行附加位移使 kept + ellipsis 合宽右对齐到 line_limit
///（400 − 17.6 − 8 = 374.4 行起点）；非 kept 行不受再对齐影响。
#[test]
fn right_clamped_line_realigned_with_ellipsis_width() {
    let mut ctx = trailing_space_lines(TextAlign::Right);
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    let pre_line0_x = ctx.lines[0].runs[0].x;
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    assert_eq!(
        ctx.lines[0].runs[0].x, pre_line0_x,
        "非 kept 行不受再对齐影响（只动末保留行）"
    );
    let runs = &ctx.lines.last().unwrap().runs;
    let words = word_indices(runs);
    assert_eq!(words.len(), 1, "前置：kept 词单片段");
    let ell = ctx.advance_of('…', None, 16.0, false);
    assert!((ell - 8.0).abs() < 0.01, "前置：estimate '…' = 8，实得 {ell}");
    let first = &runs[words[0]];
    assert!(
        (first.x - (400.0 - first.width - ell)).abs() < 0.01,
        "right 再对齐：kept 起点 = line_limit − kept_w − ell_w，实得 {}",
        first.x
    );
    let kept_end = first.x + first.width;
    let cleared: Vec<_> = runs.iter().filter(|f| f.text.is_empty()).collect();
    assert!(
        cleared.iter().all(|f| (f.x - kept_end).abs() < 0.01),
        "清零片段 glued 到 kept 末端（整行位移后不脱 glue），实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
    let band_end = runs.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    assert!(
        (band_end - (400.0 - ell)).abs() < 0.01,
        "行 band 上界 = line_limit − ell_w（ellipsis 预留位），实得 {band_end}"
    );
}

/// center：kept + ellipsis 合宽居中（400 − 17.6 − 8 = 374.4，行起点 187.2）。
#[test]
fn center_clamped_line_realigned_with_ellipsis_width() {
    let mut ctx = trailing_space_lines(TextAlign::Center);
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    let runs = &ctx.lines.last().unwrap().runs;
    let words = word_indices(runs);
    let first = &runs[words[0]];
    let ell = ctx.advance_of('…', None, 16.0, false);
    let desired = (400.0 - first.width - ell) / 2.0;
    assert!(
        (first.x - desired).abs() < 0.01,
        "center 再对齐：kept 起点 = (line_limit − kept_w − ell_w)/2，实得 {}",
        first.x
    );
}

/// justify：kept run 重均布至 [left, line_limit − ell_w]，cleared tail 重折叠到
/// 末 kept run 新末端（原 justify 把 hang run 也当内容分 gap，折叠位与 kept 端
/// 之间隔着旧 gap——uniform 位移不复 glued，R5297 band 域零延伸口径不变）。
#[test]
fn justify_clamped_line_redisatributed_and_cleared_reglued() {
    let mut ctx = InlineFormattingContext::new(400.0)
        .with_preserve_whitespace(true)
        .with_break_at_newline(true)
        .with_text_align(TextAlign::Justify);
    ctx.break_into_lines(vec![run("aa bb cc \ndd ee ff \ngg hh ii ")]);
    assert_eq!(ctx.lines.len(), 3, "前置：强制断行 3 行");
    ctx.apply_line_clamp_cap(2);
    assert!(ctx.clamped, "截断生效");
    let runs = &ctx.lines.last().unwrap().runs;
    let words = word_indices(runs);
    assert!(words.len() >= 2, "前置：kept 词多片段，实得 {words:?}");
    let ell = ctx.advance_of('…', None, 16.0, false);
    let first = &runs[words[0]];
    let last = &runs[*words.last().unwrap()];
    // justify j=0 不位移（首 kept run 原位），kept 铺满 [0, line_limit − ell_w]。
    assert!((first.x).abs() < 0.01, "justify 首 kept run 原位，实得 {}", first.x);
    let kept_end = last.x + last.width;
    assert!(
        (kept_end - (400.0 - ell)).abs() < 0.01,
        "justify 再均布：kept 末端 = line_limit − ell_w，实得 {kept_end}"
    );
    let cleared: Vec<_> = runs.iter().filter(|f| f.text.is_empty()).collect();
    assert!(
        !cleared.is_empty() && cleared.iter().all(|f| (f.x - kept_end).abs() < 0.01),
        "清零片段重折叠到末 kept run 新末端，实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
    let band_end = runs.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    assert!(
        (band_end - kept_end).abs() < 0.01,
        "行 band 上界不越过 kept 末端（幻影点零延伸）：{band_end} vs {kept_end}"
    );
}
