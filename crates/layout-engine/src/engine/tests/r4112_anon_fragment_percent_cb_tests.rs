//! R4112（CSS2.1 §9.2.1.1 anonymous block boxes）：匿名块盒在**百分比解析时被忽略**——
//! 匿名片段盒的子元素百分比高相对「最近非匿名祖先盒」的确定高解析，而非匿名片段自身。
//!
//! driving: anonymous-boxes-001a（div height:200px > [inline 内容 + block p] 混排 →
//! 匿名块包住行内内容，img height:50% 应 = 100px；旧实现相对匿名盒（indefinite）不解析，
//! taffy/IFC 膨胀到 784×784 全视口）。
//!
//! 锚：块容器（height:200px 明确）> 混排内容（匿名片段）> 百分比高子块 → 子块高 = 200×%。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn find_boxes_by_style(root: &LayoutBox, pred: &impl Fn(&LayoutBox) -> bool, out: &mut Vec<(f32, f32)>) {
    if pred(root) {
        out.push((root.width, root.height));
    }
    for c in &root.children {
        find_boxes_by_style(c, pred, out);
    }
}

/// 混排块容器（height:200px 明确）：inline 文本 + 100%-高子块 → 子块高应 200
/// （最近非匿名祖先 = 该容器；匿名片段盒高度被忽略）。
#[test]
fn r4112_percent_height_child_of_anon_fragment_uses_non_anon_ancestor() {
    let html = r#"<html><body style="margin:0">
<div style="height: 200px; background: green;">inline text
  <div style="height: 50%; background: blue;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("div { margin: 0; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 内层 50%-高 div：蓝色子块。定位：树中第二个 style 带 height:50% 的 div 盒。
    let inner_id = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("inner div");
    let mut boxes = Vec::new();
    find_boxes_by_style(&result.root, &|b: &LayoutBox| b.node_id == Some(inner_id), &mut boxes);
    let (_, h) = boxes.last().copied().expect("inner div box");
    // 50% × 最近非匿名祖先 200px = 100。旧实现相对匿名片段（indefinite）不解析 → 膨胀值。
    assert!(
        (h - 100.0).abs() < 1.0,
        "匿名片段内 % 高子块应按最近非匿名祖先（200px）解析为 100，got {h}"
    );
}

/// 对照锚（非匿名路径零变化）：同样的 50%-高子块在**普通**块容器（无混排、无匿名
/// 片段）内仍按直接父解析——200px 容器内 50% = 100，不被本 pass 触碰。
#[test]
fn r4112_plain_container_percent_height_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="height: 200px; background: green;">
  <div style="height: 50%; background: blue;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet("div { margin: 0; }");
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let inner_id = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("inner div");
    let mut boxes = Vec::new();
    find_boxes_by_style(&result.root, &|b: &LayoutBox| b.node_id == Some(inner_id), &mut boxes);
    let (_, h) = boxes.last().copied().expect("inner div box");
    assert!((h - 100.0).abs() < 1.0, "普通容器内 % 高照旧解析，got {h}");
}
