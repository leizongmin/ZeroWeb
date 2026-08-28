//! R3765：`aspect-ratio` + definite width + height:auto + **非 visible overflow** 的块盒，
//! used block-size 由 ratio 传递确定（css-sizing-4 §5.2：scroll container 无 content-based
//! minimum），内容不撑盒——R1743 父高回填不得把它长到子 max-bottom。
//! driving: WPT css-sizing/aspect-ratio/block-aspect-ratio-010/011/012（100×100 ratio 盒
//! 含 100+500 子，曾渲 600/1000，红子溢出可见）。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;

fn layout_ar_box(overflow_y: zero_css_parser::values::OverflowValue, with_children: bool) -> crate::LayoutBox {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let mut children = vec![];
    if with_children {
        for _ in 0..2 {
            let c = doc.create_element("div");
            doc.append_child(parent, c).unwrap();
            children.push(c);
        }
    }

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.width = LengthValue::Px(100.0);
    parent_style.aspect_ratio = Some(1.0);
    parent_style.overflow_y = overflow_y;

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    for &c in &children {
        let mut s = ComputedStyle::default();
        s.display = zero_style_system::DisplayValue::Block;
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(500.0);
        styles.insert(c, s);
    }

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    find(&result.root, parent).expect("parent box").clone()
}

#[test]
/// overflow:hidden ratio 盒：子 1000px 不撑盒（R1743 回填豁免），保持 ratio 传递的 100。
fn r3765_ar_scroll_container_children_do_not_grow_box() {
    let p = layout_ar_box(zero_css_parser::values::OverflowValue::Hidden, true);
    assert_eq!(
        p.height, 100.0,
        "aspect-ratio transferred height (100) must not be backfilled to children max-bottom (1000)"
    );
}

#[test]
/// overflow:visible ratio 盒：保持既有回填行为（ZW 无溢出绘制，回填近似溢出生长）。
fn r3765_ar_visible_overflow_keeps_backfill() {
    let p = layout_ar_box(zero_css_parser::values::OverflowValue::Visible, true);
    assert_eq!(
        p.height, 1000.0,
        "visible-overflow ratio box keeps legacy content backfill (approximates overflow growth)"
    );
}

#[test]
/// 无子元素时 ratio 盒两态均为 100（taffy 传递本就正确，回归哨兵）。
fn r3765_ar_childless_box_uses_ratio_height() {
    for ov in [
        zero_css_parser::values::OverflowValue::Hidden,
        zero_css_parser::values::OverflowValue::Visible,
    ] {
        let p = layout_ar_box(ov, false);
        assert_eq!(p.height, 100.0, "childless ratio box height = width/ratio");
    }
}
