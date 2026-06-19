//! table layout / img intrinsic / float shrink 回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_css_parser::values::DisplayValue;
use zero_style_system::StyleSystem;

#[test]
fn test_table_styles_correct() {
    let html = r#"<html><body><table><tr><td>cell</td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);

    let root = doc.root();
    let mut stack = vec![root];
    let mut found_table = false;
    while let Some(nid) = stack.pop() {
        if let Some(style) = styles.get(&nid) {
            if let Some(n) = doc.get(nid) {
                if let zero_dom::NodeKind::Element(elem) = &n.kind {
                    if elem.local_name() == "table" {
                        found_table = true;
                        assert_eq!(style.display, DisplayValue::Table, "table should have display:table");
                    }
                }
            }
        }
        if let Some(n) = doc.get(nid) {
            stack.extend(n.children.iter().copied());
        }
    }

    assert!(found_table, "should find <table> element");
}

#[test]
fn test_table_layout_runs() {
    let html = r#"<html><body style="margin:0"><table style="width:200px"><tr><td style="width:100px;height:40px"></td><td style="width:100px;height:40px"></td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // Should not crash, and root should have non-zero size
    assert!(result.root.width > 0.0);
    assert!(result.root.height > 0.0);
}

/// `<img>` 无 width/height 属性时应使用解码固有尺寸（DC-11 替换元素固有尺寸）。
#[test]
fn test_img_intrinsic_size_from_decoded() {
    let html = r#"<html><body style="margin:0"><img src="logo.jpg"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);

    // 模拟解码后的固有尺寸
    let img_id = doc
        .get_elements_by_tag_name("img")
        .into_iter()
        .next()
        .expect("img element exists");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (120.0, 90.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);

    // 在布局树中找到 img 盒，断言其尺寸 ≈ 解码固有尺寸
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found in layout tree");
    assert!((w - 120.0).abs() < 1.0, "img width should use intrinsic 120, got {w}");
    assert!((h - 90.0).abs() < 1.0, "img height should use intrinsic 90, got {h}");
}

/// 回归：CSS §10.3/§10.6 替换元素——`<img>` 仅设 CSS width（height auto）时，
/// height 应按固有宽高比从 width 推导，而非用固有绝对高度。
/// 旧 bug：正方形 SVG（intrinsic 441×441）+ width:80px 渲染成 80×441（巨高），
/// 致真实页面 logo（仅设 width 或 height）严重变形（wintertc logo）。
#[test]
fn test_img_width_set_height_auto_preserves_aspect() {
    let html = r#"<html><body style="margin:0"><img src="logo.svg" style="width:80px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (441.0, 441.0)); // 正方形固有尺寸
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((w - 80.0).abs() < 1.0, "img width should be 80 (CSS), got {w}");
    assert!(
        (h - 80.0).abs() < 1.5,
        "img height should be aspect-preserved ~80 (square @ width 80), got {h}"
    );
}

/// 对称：仅设 CSS height（width auto）时，width 按固有比例推导。
#[test]
fn test_img_height_set_width_auto_preserves_aspect() {
    let html = r#"<html><body style="margin:0"><img src="logo.svg" style="height:48px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (200.0, 100.0)); // 2:1 宽图
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((h - 48.0).abs() < 1.0, "img height should be 48 (CSS), got {h}");
    assert!(
        (w - 96.0).abs() < 1.5,
        "img width should be aspect-preserved ~96 (2:1 @ height 48), got {w}"
    );
}

/// R325：CSS §10 替换元素——`<img>` 同时显式设置 width 与 height 时，两者都必须生效，
/// 不得用固有宽高比强制（否则 taffy 会把 height 拉到 width 比例，忽略显式 height）。
/// 旧实现 `<img style="width:200px;height:50px">` 渲染成 200×200（height 被忽略）。
#[test]
fn test_img_both_width_height_set_no_aspect_enforcement() {
    let html = r#"<html><body style="margin:0"><img src="red.png" style="width:200px;height:50px"></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (100.0, 100.0)); // 正方形 intrinsic（ratio 1:1）
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes);
    let mut found = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(img_id) {
            found = Some((b.width, b.height));
            break;
        }
        stack.extend(b.children.iter());
    }
    let (w, h) = found.expect("img box found");
    assert!((w - 200.0).abs() < 1.0, "img width should be 200 (CSS), got {w}");
    assert!(
        (h - 50.0).abs() < 1.0,
        "img height should be 50 (CSS, not aspect-forced 200), got {h}"
    );
}

/// width:auto 的浮动元素，块级子元素全 0 宽（如 visibility:collapse 的 flex item
/// 主尺寸归零，或空内容块）时，应 shrink-to-fit 收缩到 padding+border，
/// 而非撑满容器全宽。旧实现 `content_max_w > 0.0` 条件在此跳过收缩（R300/R301）。
#[test]
fn test_float_with_zero_width_block_child_shrinks() {
    use zero_css_parser::values::FloatValue;
    let html = r#"<html><body style="margin:0"><div style="float:left"><div style="width:0px;height:10px"></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 float div（float != None，含 width:0 块级子元素）
    let mut float_w = None;
    fn walk(b: &LayoutBox, out: &mut Option<f32>) {
        if out.is_some() {
            return;
        }
        if b.float != FloatValue::None && b.children.iter().any(|c| c.is_block_level) {
            *out = Some(b.width);
            return;
        }
        for c in &b.children {
            walk(c, out);
        }
    }
    walk(&result.root, &mut float_w);
    let w = float_w.expect("should find a float with block-level children");
    assert!(
        w < 50.0,
        "width:auto float with 0-width block child should shrink (<<800), got {w} (old bug left it full-width)"
    );
}
