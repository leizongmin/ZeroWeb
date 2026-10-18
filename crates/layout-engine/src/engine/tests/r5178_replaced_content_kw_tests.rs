//! R5178：替换元素 min/max-width/height 的 content 关键字 = transferred size
//!（css-sizing-3 §5.1 + csswg #3973）。converter 把 min/max 关键字映射 length(0)/auto
//! 且 R4149 主臂 `!b.is_replaced` 排除替换元素 → 无人钳。本组钉住：min/max-content
//! 块尺寸 = 固有比 × definite 正交轴尺寸，无有效比退 HTML attr 固有尺寸。
//! driving: WPT css-sizing/replaced-{min,max}-{width,height}-min-content、
//! css-sizing/svg-no-ar-{min,max}-height-min-content。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

/// 在布局树中找指定 tag 的第一个 LayoutBox，返回其 (width, height)。
fn find_first(doc: &zero_dom::Document, root: &LayoutBox, tag: &str) -> Option<(f32, f32)> {
    let mut stack: Vec<&LayoutBox> = vec![root];
    while let Some(b) = stack.pop() {
        let is_target = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == tag))
            .unwrap_or(false);
        if is_target {
            return Some((b.width, b.height));
        }
        stack.extend(&b.children);
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(&format!("<!DOCTYPE html>\n{html}"));
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone()))
}

/// canvas attr 10×10（固有比 1）+ `width:10px; height:100px; min-width:min-content`
/// → min-content 宽 = transferred 100，floor 掉 10px 声明宽（replaced-min-width-min-content）。
#[test]
fn r5178_replaced_min_width_min_content_transferred() {
    let html = r#"<canvas width="10" height="10" style="width: 10px; height: 100px; min-width: min-content; background: green;"></canvas>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "canvas").expect("canvas not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "min-width:min-content must floor to transferred 100x100, got {w}x{h}"
    );
}

/// canvas attr 10×10 + `width:1000px; height:100px; max-width:min-content`
/// → cap 到 transferred 100（replaced-max-width-min-content）。
#[test]
fn r5178_replaced_max_width_min_content_transferred() {
    let html = r#"<canvas width="10" height="10" style="width: 1000px; height: 100px; max-width: min-content; background: green;"></canvas>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "canvas").expect("canvas not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "max-width:min-content must cap to transferred 100x100, got {w}x{h}"
    );
}

/// canvas attr 10×10 + `width:100px; height:10px; min-height:min-content`
/// → min-content 高 = transferred 100（definite 宽 ÷ 比），floor 掉 10px 声明高
///（replaced-min-height-min-content）。
#[test]
fn r5178_replaced_min_height_min_content_transferred() {
    let html = r#"<canvas width="10" height="10" style="width: 100px; height: 10px; min-height: min-content; background: green;"></canvas>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "canvas").expect("canvas not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "min-height:min-content must floor to transferred 100x100, got {w}x{h}"
    );
}

/// canvas attr 10×10 + `width:100px; height:1000px; max-height:min-content`
/// → cap 到 transferred 100（replaced-max-height-min-content）。
#[test]
fn r5178_replaced_max_height_min_content_transferred() {
    let html = r#"<canvas width="10" height="10" style="width: 100px; height: 1000px; max-height: min-content; background: green;"></canvas>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "canvas").expect("canvas not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "max-height:min-content must cap to transferred 100x100, got {w}x{h}"
    );
}

/// svg attr 0×100（width=0 → 无有效固有比）+ `width:100px; height:10px;
/// min-height:min-content` → 无比退固有高 100，floor 掉 10px 声明高
///（svg-no-ar-min-height-min-content）。
#[test]
fn r5178_svg_no_ar_min_height_attr_fallback() {
    let html = r#"<svg width="0" height="100" style="width: 100px; height: 10px; min-height: min-content; background: green;"></svg>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "svg").expect("svg not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "svg min-height:min-content without ratio must floor to attr intrinsic height 100, got {w}x{h}"
    );
}

/// svg attr 0×100 + `width:100px; height:1000px; max-height:min-content`
/// → cap 到固有高 100（svg-no-ar-max-height-min-content）。
#[test]
fn r5178_svg_no_ar_max_height_attr_fallback() {
    let html = r#"<svg width="0" height="100" style="width: 100px; height: 1000px; max-height: min-content; background: green;"></svg>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "svg").expect("svg not found");
    assert!(
        (w - 100.0).abs() < 1.0 && (h - 100.0).abs() < 1.0,
        "svg max-height:min-content without ratio must cap to attr intrinsic height 100, got {w}x{h}"
    );
}

/// flex/grid item 排除守卫（flex-minimum-height-flex-items-023 案形）：column flex
/// item 的 `min-height:min-content` 走 css-flexbox §4.5 自动最小尺寸（content-based，
/// min(内容建议 50, 指定建议 100) = 50），非 css-sizing §5.1 固有/转递——R5178 臂
/// 不得把 img（attr 300×150 + 拉伸宽 100）floor 到固有块尺寸 150。本 harness 无解码
/// 像素（flex 自动最小正空间 = 0），只断言负空间：R5178 臂未写入 min_size。
/// 完整语义由上游常驻案 flex-minimum-height-flex-items-023（corpus A/B）钉住。
#[test]
fn r5178_flex_item_min_height_kw_excluded() {
    let html = r#"<div style="display: flex; flex-direction: column; width: 100px; height: 0px;">
  <img src="support/300x150-green.png" style="min-height: min-content; height: 100px;" />
</div>"#;
    let (doc, root) = layout(html);
    let (w, h) = find_first(&doc, &root, "img").expect("img not found");
    assert!(
        (w - 100.0).abs() < 1.0 && h < 149.0,
        "flex item min-height:min-content must not be floored by R5178 intrinsic arm (got {w}x{h}, want w=100 h<150)"
    );
}
