//! R5152（CSS Containment 1 §3.1 × css-multicol-1）：contain:size 的 multicol 按
//! 「无内容」尺寸化，但列几何（count×col-width + (n-1)×gap）仍参与——
//! ① abs shrink-to-fit 宽不被 R4034b 抑制（contain-size-multicol-001：2×40+20=100）；
//! ② auto 高随内容涨回被 R4508 写回门控（contain-size-multicol-002：红底多绘）；
//! ③ min/max-content 内在宽 = 纯列几何、内容贡献归零（contain-size-multicol-003）。
//! kill-switch `ZW_MC_CONTAIN_SIZE_SIZE=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout_box(html: &str, css: &str, selector_tag: &str, index: usize) -> (f32, f32, f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let node = doc
        .get_elements_by_tag_name(selector_tag)
        .into_iter()
        .nth(index)
        .unwrap_or_else(|| panic!("{selector_tag}#{index} 不存在"));
    find_box(&result.root, node).unwrap_or_else(|| panic!("{selector_tag}#{index} 无布局盒"))
}

/// ① contained abs multicol shrink-to-fit 宽 = 列几何（chromium oracle：100×100 绿块
/// 覆盖红块；ZW 旧版 R4034b 抑制补测 → w=0，红块全露恰 10,000px 差异）。
#[test]
fn r5152_contained_abs_multicol_shrink_to_fit_column_geometry() {
    let (x, _y, w, h) = layout_box(
        r#"<html><body style="margin:0"><p>x</p><div id="red"></div><div id="test">filler filler filler</div></body></html>"#,
        r#"div { position: absolute; margin: 0; }
#red { background: red; width: 100px; height: 100px; }
#test { background: green; contain: size; columns: 2 40px; column-gap: 20px; min-height: 100px; }"#,
        "div",
        1,
    );
    assert!(
        (w - 100.0).abs() < 0.5,
        "contained abs multicol shrink-to-fit 宽应为列几何 100（2×40+20），实际 {w}"
    );
    assert!((h - 100.0).abs() < 0.5, "min-height:100px 应保持，实际 {h}");
    let _ = x;
}

/// ② contained multicol auto 高按无内容尺寸化——50×50 子不涨高（盒高 = 上下 border
/// 32；旧版平衡列高写回 → 82，红底随内容多绘 50px）。min-height 钳制不受影响。
#[test]
fn r5152_contained_multicol_auto_height_ignores_content() {
    let (_x, _y, w1, h1) = layout_box(
        r#"<html><body style="margin:0"><div id="t1"><div class="inner">inner</div></div><div id="t2" class="min-h"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t1, #t2 { contain: size; border: 16px solid green; background: red; column-count: 3; }
#t2 { min-height: 20px; }
.inner { width: 50px; height: 50px; }"#,
        "div",
        0,
    );
    assert!(
        (h1 - 32.0).abs() < 0.5,
        "contained multicol auto 高应只含边框 32（内容 50px 不参与），实际 {h1}"
    );
    let (_x2, _y2, _w2, h2) = layout_box(
        r#"<html><body style="margin:0"><div id="t1"><div class="inner">inner</div></div><div id="t2" class="min-h"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t1, #t2 { contain: size; border: 16px solid green; background: red; column-count: 3; }
#t2 { min-height: 20px; }
.inner { width: 50px; height: 50px; }"#,
        "div",
        2,
    );
    assert!(
        (h2 - 52.0).abs() < 0.5,
        "min-height:20px 钳制应保持（20+32=52），实际 {h2}"
    );
    let _ = w1;
}

/// ③ contained multicol min-content 内在宽 = 纯列几何——col-width auto 列塌零宽，
/// 只剩 (n-1)×gap=10 + 边框 32 = 42，内容贡献（100×100 子）归零——与 empty multicol
/// ref 同几何（contain-size-multicol-003 ref 末两案；旧版内容撑到 3×100+10+32=342）。
#[test]
fn r5152_contained_multicol_min_content_is_column_geometry() {
    let (_x, _y, w, _h) = layout_box(
        r#"<html><body style="margin:0"><div class="min"><div id="t"><div class="inner">inner</div></div></div></body></html>"#,
        r#"div { margin: 0; }
.min { width: min-content; }
#t { contain: size; border: 16px solid green; column-count: 3; column-gap: 5px; }
.inner { width: 100px; height: 100px; }"#,
        "div",
        0,
    );
    assert!(
        (w - 42.0).abs() < 0.5,
        "contained multicol min-content 应为列几何 (3-1)×5+边框 32=42，实际 {w}"
    );
}
