/// Writing-mode 布局测试。
///
/// 验证垂直书写模式下轴交换和坐标还原的正确性。
use super::*;
use zero_style_system::StyleSystem;
use zero_style_system::WritingModeValue;

/// 辅助函数：在布局树中按标签名查找第一个匹配的 LayoutBox。
fn find_box_by_tag<'a>(root: &'a LayoutBox, doc: &Document, tag: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = root.node_id {
        if let Some(node) = doc.get(nid) {
            if let zero_dom::NodeKind::Element(elem) = &node.kind {
                if elem.local_name() == tag {
                    return Some(root);
                }
            }
        }
    }
    for child in &root.children {
        if let Some(found) = find_box_by_tag(child, doc, tag) {
            return Some(found);
        }
    }
    None
}

/// 测试水平书写模式（默认）下布局不受影响。
#[test]
fn test_writing_mode_horizontal_tb_no_swap() {
    let html = r#"<html><body style="margin:0">
        <div style="width:100px; height:50px; writing-mode:horizontal-tb">hello</div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");
    assert_eq!(div.width, 100.0, "div width should be 100px, got {}", div.width);
    assert_eq!(div.height, 50.0, "div height should be ~50px, got {}", div.height);
    assert!(
        matches!(div.writing_mode, WritingModeValue::HorizontalTb),
        "div writing_mode should be HorizontalTb"
    );
}

/// 测试 writing-mode 属性解析和存储。
#[test]
fn test_writing_mode_vertical_rl_parsed() {
    let html = r#"<html><body style="margin:0">
        <div style="writing-mode:vertical-rl">text</div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");
    assert!(
        matches!(div.writing_mode, WritingModeValue::VerticalRl),
        "div writing_mode should be VerticalRl"
    );
}

/// 测试 writing-mode 在元素上显式设置时正确传播到 LayoutBox。
/// TODO: 待垂直布局完整实现后，测试隐式继承和显式 inherit 关键字。
#[test]
fn test_writing_mode_explicit_on_child() {
    let html =
        r#"<html><body style="margin:0"><div style="writing-mode:vertical-rl">text</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // body 应为 horizontal-tb（默认）
    let body = find_box_by_tag(&result.root, &doc, "body").expect("should find <body>");
    assert!(
        matches!(body.writing_mode, WritingModeValue::HorizontalTb),
        "body should have horizontal-tb"
    );

    // div 显式设置 writing-mode:vertical-rl
    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");
    assert!(
        matches!(div.writing_mode, WritingModeValue::VerticalRl),
        "div should have vertical-rl (explicitly set)"
    );
}

/// 测试 vertical-rl 容器的尺寸轴交换。
#[test]
fn test_writing_mode_vertical_rl_container_size_swap() {
    let html = r#"<html><body style="margin:0">
        <div style="writing-mode:vertical-rl; width:200px; height:100px"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");

    // extract_layout 应交换回视觉坐标：width=200px, height=100px
    assert_eq!(div.width, 200.0, "visual width should be 200px, got {}", div.width);
    assert_eq!(div.height, 100.0, "visual height should be 100px, got {}", div.height);
}

/// 测试 vertical-rl 容器中 border 的轴交换。
#[test]
fn test_writing_mode_vertical_rl_border_swap() {
    let html = r#"<html><body style="margin:0">
        <div style="writing-mode:vertical-rl; border-left:5px solid; border-top:10px solid; width:200px; height:100px"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");

    // CSS border-left=5px, border-top=10px
    // extract_layout 应交换回来
    assert_eq!(
        div.border_left, 5.0,
        "visual border_left should be 5px (swapped back), got {}",
        div.border_left
    );
    assert_eq!(
        div.border_top, 10.0,
        "visual border_top should be 10px (swapped back), got {}",
        div.border_top
    );
}

/// 测试 vertical-rl 子元素的尺寸轴交换。
#[test]
fn test_writing_mode_vertical_rl_child_size_swap() {
    let html = r#"<html><body style="margin:0">
        <div style="writing-mode:vertical-rl; width:300px; height:400px">
            <div style="width:100px; height:50px">child</div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 vertical-rl 容器
    let container = find_box_by_tag(&result.root, &doc, "div").expect("should find container <div>");
    assert!(
        matches!(container.writing_mode, WritingModeValue::VerticalRl),
        "container writing_mode should be VerticalRl"
    );

    // 子 div 也应为 VerticalRl（继承）
    let child = &container.children[0];
    assert_eq!(
        child.width, 100.0,
        "child visual width should be 100px (swapped back), got {}",
        child.width
    );
    assert_eq!(
        child.height, 50.0,
        "child visual height should be 50px (swapped back), got {}",
        child.height
    );
}
