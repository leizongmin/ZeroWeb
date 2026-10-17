//! R5161（CSS Text 3 §5.2 / UAX#14）：跨 run 词中拼接边界无软换行机会。
//!
//! 旧实现逐 run 独立断行：`PASS<span>FAIL</span>` 的 S|F 边界（无空白相邻的字母对）
//! 被当断点，FAIL 在 4ch 容器内误折第 2 行（contain-paint-047 族红底外露 4.83%）；
//! chromium 按 UAX#14 无断点，FAIL 随前词横向溢出，被 contain:paint 裁净。
//! 判据镜像 run 内分词（collapse_split_words）：断点仅存在于空白与 CJK/SEA
//! per-char 字符侧。kill-switch `ZW_CROSS_RUN_GLUE=0`。
//!
//! Ahem 基准：font:100px 每字符 advance = 100px，单行高 100px。

use crate::engine::LayoutEngine;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 布局并返回 body 第一个 div 的 (width, height)。
fn probe_div(html: &str) -> (f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let div = &result.root.children[1].children[0];
    (div.width, div.height)
}

/// 跨 run 词中拼接边界（S|F，无空白）不断行——FAIL 随前词横向溢出，div 单行高。
/// contain-paint-047 复现形（400px 容器恰容 4 字符）。
#[test]
fn test_cross_run_word_boundary_does_not_wrap() {
    let (w, h) = probe_div(
        r#"<html><body style="margin:0"><div style="width:400px;font:100px Ahem">PASS<span>FAIL</span></div></body></html>"#,
    );
    assert_eq!(w, 400.0);
    assert!(h < 150.0, "glued cross-run word must stay on one line (h≈100), got {h}");
}

/// 对照基座：单 run 无空格长词同形溢出，单行高（同边界字符对，无元素分隔也单行）。
#[test]
fn test_single_run_long_word_overflows_one_line() {
    let (w, h) = probe_div(
        r#"<html><body style="margin:0"><div style="width:400px;font:100px Ahem">PASSFAIL</div></body></html>"#,
    );
    assert_eq!(w, 400.0);
    assert!(h < 150.0, "single unbreakable word overflows on one line, got {h}");
}

/// 空白边界（`PASS ` + span）断行保持——词尾可折叠空白仍是合法断点（不伤常规折行）。
#[test]
fn test_whitespace_boundary_still_wraps() {
    let (w, h) = probe_div(
        r#"<html><body style="margin:0"><div style="width:400px;font:100px Ahem">PASS <span>FAIL</span></div></body></html>"#,
    );
    assert_eq!(w, 400.0);
    assert!(
        h > 150.0,
        "whitespace boundary must keep the soft wrap opportunity, got {h}"
    );
}

/// CJK 跨 run 边界（汉|字）per-char 断点保持——窄容器仍逐字符断行。
#[test]
fn test_cjk_cross_run_boundary_still_breaks() {
    let (w, h) = probe_div(
        r#"<html><body style="margin:0"><div style="width:150px;font:100px Ahem">汉<span>字</span>汉</div></body></html>"#,
    );
    assert_eq!(w, 150.0);
    assert!(
        h > 250.0,
        "CJK per-char break opportunities must survive across run boundaries, got {h}"
    );
}

/// 宽容器跨 run 单行（放得下时不受 glue 影响）。
#[test]
fn test_cross_run_fits_on_wide_line() {
    let (w, h) = probe_div(
        r#"<html><body style="margin:0"><div style="width:900px;font:100px Ahem">PASS<span>FAIL</span></div></body></html>"#,
    );
    assert_eq!(w, 900.0);
    assert!(h < 150.0, "fits on one line, got {h}");
}
