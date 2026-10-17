//! R5163（CSS 2.2 §17.4.1 + CSS Containment §3.1）：caption 的 auto 宽在
//! size containment 下维持 fill 表宽。
//!
//! 旧路径把 caption 归入 CIS-or-0 臂：`contain: paint size` / `strict` 的 caption
//! auto 宽塌 0（206 宽表的 caption 只剩 40px 纯边框碎条）——contain-paint-size-001 /
//! contain-strict-001。§17.4.1：caption 宽「as if the caption were a block box child
//! of the table wrapper」——fill 行为非 content-based，containment 只折叠 content
//! 贡献（R2257 block 臂同语义）；块轴（高）仍 CIS-or-0 → content 高 0、纯边框。
//! kill-switch `ZW_CAPTION_CONTAIN_FILL=0`。
//!
//! 探针无 FontLoader：baseline 文本行高随环境浮动，断言用界宽而非钉值。

use crate::engine::LayoutEngine;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 布局并返回首个 caption 盒 (x, y, width, height)。
fn probe_caption(html: &str) -> (f32, f32, f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn walk(box_: &crate::types::LayoutBox, doc: &zero_dom::Document, found: &mut Option<(f32, f32, f32, f32)>) {
        if found.is_none()
            && let Some(nid) = box_.node_id
            && let Some(el) = doc.get(nid)
            && let zero_dom::NodeKind::Element(e) = &el.kind
            && e.tag_name().eq_ignore_ascii_case("caption")
        {
            *found = Some((box_.x, box_.y, box_.width, box_.height));
        }
        for c in &box_.children {
            walk(c, doc, found);
        }
    }
    let mut found = None;
    walk(&result.root, &doc, &mut found);
    found.expect("caption box expected")
}

/// contain-paint-size-001 / contain-strict-001 复现形：caption (0,0,206,40)
/// ——宽填满表宽、高纯边框（content 高 0），行组在其下。
fn assert_border_only_caption(contain: &str) {
    let (x, y, w, h) = probe_caption(&format!(
        r#"<html><body style="margin:0">
        <table style="table-layout: fixed; width: 206px">
        <caption style="border: green solid 20px; contain: {contain}; font-size: 20px">FAIL FAIL FAIL FAIL FAIL FAIL</caption>
        <tr><td>&nbsp;<td>&nbsp;
        <tr><td>&nbsp;<td>&nbsp;
        </table>
        </body></html>"#
    ));
    assert_eq!(
        (x, y, w, h),
        (0.0, 0.0, 206.0, 40.0),
        "contain:{contain} caption must fill table width with border-only height, got ({x}, {y}, {w}, {h})"
    );
}

/// `contain: paint size`：宽 206（fill 表宽）+ 高 40（纯边框）。
#[test]
fn test_caption_contain_paint_size_fills_table_width() {
    assert_border_only_caption("paint size");
}

/// `contain: strict`（size+layout+paint+style）同形。
#[test]
fn test_caption_contain_strict_fills_table_width() {
    assert_border_only_caption("strict");
}

/// 无 containment baseline 不变：caption 内容驱动（宽仍 206，高 > 纯边框 40）。
#[test]
fn test_caption_no_contain_keeps_content_sized_height() {
    let (x, y, w, h) = probe_caption(
        r#"<html><body style="margin:0">
        <table style="table-layout: fixed; width: 206px">
        <caption style="border: green solid 20px; font-size: 20px">FAIL FAIL FAIL FAIL FAIL FAIL</caption>
        <tr><td>&nbsp;<td>&nbsp;
        <tr><td>&nbsp;<td>&nbsp;
        </table>
        </body></html>"#,
    );
    assert_eq!(
        (x, y, w),
        (0.0, 0.0, 206.0),
        "no-contain caption still fills table width"
    );
    assert!(
        h > 40.0 + 8.0,
        "no-contain caption height must be content-driven (text line + 40px borders), got {h}"
    );
}
