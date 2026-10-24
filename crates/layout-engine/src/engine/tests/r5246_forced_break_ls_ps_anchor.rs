//! R5246：U+2028 LS / U+2029 PS 进 white-space 强制断行类 + LS 段落连续行级 L2 锚点。
//!
//! 修前 pre/pre-line 下的 PS/LS 双缺陷（bidi-breaking-003 set 2/3 ✗2.24%）：
//! ① `collapse_whitespace_keep_newlines` 与 `split_into_words_with_ws` 的强制断行类
//!    只有 `\n`——PS/LS 在 pre-lines collect 阶段被折叠为空格、在 pre 词切分层留为
//!    普通字符（画 tofu、不折行）；② unicode-bidi 按 UAX #9 class B 把 LS 一并切段
//!    （CSS 语义：段落分隔只有 PS；LS 只强制断行，段落 levels 跨 LS 连续、行级 L2）。
//! 修后 LS/PS 与 `\n` 同为强制断行点（本体不渲染），LS 不断段、行序 = 逻辑行序。
//!
//! kill-switch：`ZW_FORCED_BREAK_LS_PS=0` 回退①（锚翻红单行 tofu 签名）；锚②
//! 对 `ZW_BIDI_LS_LINE_ONLY=0` 翻红（LS 切段重排仍无断行标记，单行）。
//! OFF 反证以 reftest bidi-breaking-003 实证（① OFF ✗2.22 / ② OFF ✓0.06，
//! 双开 ✓0.00）；单测进程内不设 env。

use super::*;

/// 找 body 直下首个块盒。
fn first_block(result: &LayoutResult) -> &LayoutBox {
    result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| b.is_block_level)
        .expect("块容器存在")
}

/// R5246 锚①：pre 容器 LS/PS 强制折行——3 行盒（Ahem 100px/1 每行 100px），
/// 修前单行（LS/PS 不断行）。
#[test]
fn r5246_pre_ls_ps_forced_line_breaks() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:100px/1 Ahem; white-space:pre\">aaa\u{2028}bbb\u{2029}ccc</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = first_block(&result);
    // LS 折行 + PS 折行 = 3 行 × 100。
    assert!(
        (div.height - 300.0).abs() < 1.0,
        "pre 容器应 3 行高 300（LS/PS 各一强制断行），实 {}",
        div.height
    );
}

/// R5246 锚②：pre 容器 RTL 混排跨 LS——段落 levels 跨 LS 连续（行级 L2 后仍按
/// 逻辑行折 2 行），修前（LS 切段或不断行）行数签名不同。
#[test]
fn r5246_pre_rtl_ls_two_lines() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:100px/1 Ahem; white-space:pre\">אא\u{2028}בב</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = first_block(&result);
    // LS 强制断行 → 2 行 × 100（段落连续性不影响行数，行序/行内形态由
    // text_metrics 单测 r5246_ls_line_break_not_paragraph_line_scoped_l2 钉死）。
    assert!(
        (div.height - 200.0).abs() < 1.0,
        "pre 容器应 2 行高 200（LS 强制断行），实 {}",
        div.height
    );
}
