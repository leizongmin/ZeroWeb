//! R5110：替换元素 max 侧 min-content/max-content 关键字钳制测试（css-sizing-3 §5.2
//! + csswg #3973）。
//!
//! converter 把 max 侧关键字映射 taffy auto（替换元素无内容可测，taffy 不钳）——
//! chrome 语义：有内在比且另一轴确定 → 按比转递；无比 → 本轴 attr 内在值。
//! driving: replaced-max-width-min-content / replaced-max-height-min-content /
//! svg-no-ar-max-height-min-content（全量 fail 带 14.42%/9.35%/9.35%）。

use crate::engine::LayoutEngine;

fn compute_root(html: &str) -> std::sync::Arc<crate::types::LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles).root
}

fn replaced_box(root: &crate::types::LayoutBox) -> &crate::types::LayoutBox {
    root.children
        .iter()
        .flat_map(|b| b.children.iter())
        .find(|b| b.is_replaced)
        .expect("replaced 盒")
}

/// 有内在比（canvas attr 10×10）+ 另一轴 CSS 确定（height:100px）→ max-width:
/// min-content 按比转递 100：width 1000→100（replaced-max-width-min-content）。
#[test]
fn test_r5110_canvas_max_width_min_content_transfers_from_height() {
    let root = compute_root(
        r#"<html><body style="margin:0">
      <canvas width="10" height="10" style="width: 1000px; height: 100px; max-width: min-content; background: green;"></canvas>
    </body></html>"#,
    );
    let canvas = replaced_box(&root);
    assert!((canvas.width - 100.0).abs() < 0.5, "w={}", canvas.width);
    assert!((canvas.height - 100.0).abs() < 0.5, "h={}", canvas.height);
}

/// 有内在比 + 另一轴 CSS 确定（width:100px）→ max-height: min-content 转递 100：
/// height 1000→100（replaced-max-height-min-content）。
#[test]
fn test_r5110_canvas_max_height_min_content_transfers_from_width() {
    let root = compute_root(
        r#"<html><body style="margin:0">
      <canvas width="10" height="10" style="width: 100px; height: 1000px; max-height: min-content; background: green;"></canvas>
    </body></html>"#,
    );
    let canvas = replaced_box(&root);
    assert!((canvas.width - 100.0).abs() < 0.5, "w={}", canvas.width);
    assert!((canvas.height - 100.0).abs() < 0.5, "h={}", canvas.height);
}

/// 无内在比（svg attr width=0）→ max-height: min-content 取本轴 attr 内在高 100：
/// height 1000→100（svg-no-ar-max-height-min-content）。
#[test]
fn test_r5110_svg_no_ratio_max_height_min_content_uses_attr_intrinsic() {
    let root = compute_root(
        r#"<html><body style="margin:0">
      <svg width="0" height="100" style="width: 100px; height: 1000px; max-height: min-content; background: green;"></svg>
    </body></html>"#,
    );
    let svg = replaced_box(&root);
    assert!((svg.width - 100.0).abs() < 0.5, "w={}", svg.width);
    assert!((svg.height - 100.0).abs() < 0.5, "h={}", svg.height);
}
