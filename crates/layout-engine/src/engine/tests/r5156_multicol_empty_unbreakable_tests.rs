//! R5156（css-break-3 §5.1 × css-multicol-1 column breaking）：multicol 子盒若为
//! **零内容区间**空盒（DOM 无内容子节点不产行盒 + content_height=0，盒高全来自
//! padding/border），内部无 Class A/B 断点机会，整体不可断——超高时单片段原地溢出
//!（Class C3），不跨列几何切片。chromium contain-size-breaks-001-ref 实证：空 div
//! padding-top 5em 整体留 col0 溢出成 1×5em 矩形（此前跨切成 2×2.5em）。对照可断形
//!（definite 内容高空盒）亦入测。kill-switch `ZW_MC_EMPTY_UNBREAKABLE=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 列片段表条目（x, y, col_x, col_w, col_top, col_h）。
type Fragments = Vec<(f32, f32, f32, f32, f32, f32)>;

/// 返回目标盒几何 + 列片段表。
fn layout_fragments(html: &str, css: &str, selector_tag: &str, index: usize) -> (f32, f32, f32, f32, Fragments) {
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
    fn find(root: &LayoutBox, id: NodeId) -> Option<(f32, f32, f32, f32, Fragments)> {
        let mut stack = vec![root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(id) {
                return Some((b.x, b.y, b.width, b.height, b.column_span_offsets.clone()));
            }
            stack.extend(b.children.iter());
        }
        None
    }
    find(&result.root, node).unwrap_or_else(|| panic!("{selector_tag}#{index} 无布局盒"))
}

/// corpus ref 形：column-fill:auto + 明确列高下的**空** div（padding-top 200 无内容）
/// 超高 → 不可断，单片段全高原地溢出（col_h = 200 = 盒全高，非 100 切片）。
#[test]
fn r5156_empty_box_not_sliced_across_columns() {
    let (_x, _y, w, h, cso) = layout_fragments(
        r#"<html><body style="margin:0"><article><div></div></article></body></html>"#,
        "article { height: 100px; column-gap: 0; columns: 3 40px; width: 120px; column-fill: auto; }\ndiv { background: orange; padding-top: 200px; margin: 0; }",
        "div",
        0,
    );
    assert!((w - 40.0).abs() < 0.5, "空盒宽应为一列 40，实际 {w}");
    assert!((h - 200.0).abs() < 0.5, "盒高应含 padding-top 全 200，实际 {h}");
    assert_eq!(cso.len(), 1, "空盒不可断应单片段，实际片段 {:?}", cso);
    assert!(
        (cso[0].5 - 200.0).abs() < 0.5,
        "片段应为全高 200（原地溢出 Class C3），实际 {}",
        cso[0].5
    );
}

/// 对照：含流内子的盒（有断点机会）超高照常跨列切片——既有 breaking 行为不变。
#[test]
fn r5156_box_with_child_still_sliced() {
    let (_x, _y, _w, h, cso) = layout_fragments(
        r#"<html><body style="margin:0"><article><div><i></i></div></article></body></html>"#,
        "article { height: 100px; column-gap: 0; columns: 3 40px; width: 120px; column-fill: auto; }\ndiv { background: orange; height: 200px; margin: 0; }\ndiv i { display: block; height: 50px; margin: 0; }",
        "div",
        0,
    );
    assert!((h - 200.0).abs() < 0.5, "盒高应为显式 200，实际 {h}");
    assert_eq!(cso.len(), 2, "有流内子的盒可断，应仍跨切为 2 片段，实际 {:?}", cso);
}

/// 窄门对照：definite 内容高的**空**盒（显式 height，无 padding）超高照常切片——
/// chromium change-fragmentainer-size-002 定谳（400px 空 div 切满 4 列成绿方块），
/// 不可断仅限零内容区间（盒高全来自 padding/border）形。
#[test]
fn r5156_explicit_height_empty_box_still_sliced() {
    let (_x, _y, _w, h, cso) = layout_fragments(
        r#"<html><body style="margin:0"><article><div></div></article></body></html>"#,
        "article { height: 100px; column-gap: 0; columns: 3 40px; width: 120px; column-fill: auto; }\ndiv { background: orange; height: 200px; margin: 0; }",
        "div",
        0,
    );
    assert!((h - 200.0).abs() < 0.5, "盒高应为显式 200，实际 {h}");
    assert_eq!(cso.len(), 2, "显式内容高空盒可断，应仍跨切为 2 片段，实际 {:?}", cso);
}
