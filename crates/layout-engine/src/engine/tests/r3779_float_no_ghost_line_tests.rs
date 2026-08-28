//! R3779b：float 子不向 IFC 注入占位 Br（无 R1286 幽灵空行）。
//!
//! CSS2 §9.5：float 脱离常规流、不产生行盒；其后的行通过 float exclusion 缩宽
//! （`effective_content_area`）。旧实现 collect_items 给 float 子发 `InlineItem::Br`，
//! R1286 strut 给行首空行赋 20px 高 → 幽灵空行占据 line-clamp 行预算
//!（driving: css-overflow/line-clamp/line-clamp-with-floats-001，cap=4 只留 3 行真文本），
//! 并抬高含 float 容器的 IFC 总高（floats-wrap-top-below-bfc-001l、floats-zero-height-wrap）。
//!
//! kill-switch `ZW_FLOAT_NO_GHOST_LINE=0` 回退旧行为（collect_items）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn compute_body_height(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn body_height(b: &LayoutBox, doc: &zero_dom::Document) -> Option<f32> {
        for c in &b.children {
            let is_body = c
                .node_id
                .and_then(|id| doc.get(id))
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "body"));
            if is_body {
                return Some(c.height);
            }
            if let Some(h) = body_height(c, doc) {
                return Some(h);
            }
        }
        None
    }
    body_height(&result.root, &doc).expect("body box")
}

/// driving: line-clamp-with-floats-001（test 页）——[float 子 + 5 行 pre 文本] 无 clamp 时
/// 容器高 = 5 行 160px；旧幽灵 Br 首行 20px → 180px。
#[test]
fn r3779_float_child_no_ghost_first_line_height() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; padding: 0 4px; white-space: pre;\">\
<div style=\"float: left; width: 50px; height: 50px; margin: 4px; background-color: skyblue;\"></div>Line 1\nLine 2\nLine 3\nLine 4\nLine 5</div>\
</body></html>";
    assert_eq!(
        compute_body_height(html),
        160.0,
        "float 脱流无行盒，5 行 pre 文本 160px（旧幽灵行 180）"
    );
}

/// 同结构 + line-clamp:4 → cap 后 = 4 行 128px；旧幽灵行占预算 → 只剩 3 行 96px。
#[test]
fn r3779_float_child_line_clamp_keeps_four_text_lines() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 4; font: 16px/32px serif; padding: 0 4px; white-space: pre;\">\
<div style=\"float: left; width: 50px; height: 50px; margin: 4px; background-color: skyblue;\"></div>Line 1\nLine 2\nLine 3\nLine 4\nLine 5</div>\
</body></html>";
    assert_eq!(
        compute_body_height(html),
        128.0,
        "clamp cap=4 计真文本行：4 行 128px（旧幽灵行挤掉第 4 行 96px）"
    );
}

/// 旧 r1733 契约保留：float 后 in-flow block 子 → BlockBreak 同样不产生 strut 行
///（R57 M3），两 float + 文本容器高度只计真行。
#[test]
fn r3779_float_and_text_only_real_lines_counted() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; white-space: pre;\">\
<div style=\"float: left; width: 20px; height: 20px;\"></div>A\nB</div>\
</body></html>";
    assert_eq!(
        compute_body_height(html),
        64.0,
        "2 行真文本 64px（旧幽灵 Br 首行 +20 = 84）"
    );
}

/// float 前无文本、float 后有文本：文本首行仍在容器顶部起排（float exclusion 只缩宽
/// 不推 y——float top = 行盒 top）。
#[test]
fn r3779_text_after_float_starts_at_container_top() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font: 16px/32px serif; white-space: pre;\">\
<div style=\"float: left; width: 20px; height: 20px;\"></div>Only</div>\
</body></html>";
    assert_eq!(
        compute_body_height(html),
        32.0,
        "float 后单行文本 32px（旧幽灵行把文本推到第 2 行 = 52）"
    );
}
