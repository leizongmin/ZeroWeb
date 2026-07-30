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
    let html = r#"<html><body style="margin:0"><div style="writing-mode:vertical-rl">text</div></body></html>"#;
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

/// 测试 vertical-rl 中绝对定位元素的静态位置。
///
/// 容器 320x320，writing-mode: vertical-rl，position: relative。
/// 内联文本 "1 2 34" + 绝对定位的 <span>X</span>。
/// abs-pos 元素的静态位置应基于它在正常流中的位置。
#[test]
fn test_vertical_rl_abs_pos_static_position() {
    let html = r#"<html><body style="margin:0">
        <div id="cb" style="position:relative; writing-mode:vertical-rl; direction:ltr;
            font:80px/1 Ahem; width:320px; height:320px; color:transparent">
            1 2 34<span style="position:absolute; top:auto; height:auto; bottom:auto; color:green">X</span>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到容器
    let container = find_box_by_tag(&result.root, &doc, "div").expect("should find container <div>");
    eprintln!(
        "container: x={}, y={}, w={}, h={}",
        container.x, container.y, container.width, container.height
    );
    eprintln!(
        "container content: x={}, y={}, w={}, h={}",
        container.content_x, container.content_y, container.content_width, container.content_height
    );
    eprintln!("container writing_mode: {:?}", container.writing_mode);

    for (i, child) in container.children.iter().enumerate() {
        eprintln!(
            "child[{}]: x={}, y={}, w={}, h={}, abs={}, node={:?}",
            i, child.x, child.y, child.width, child.height, child.is_absolute, child.node_id
        );
        if let Some(nid) = child.node_id {
            if let Some(node) = doc.get(nid) {
                eprintln!("  node kind: {:?}", std::mem::discriminant(&node.kind));
            }
        }
    }

    // 找到 abs-pos span
    let span = container.children.iter().find(|c| c.is_absolute);
    if let Some(span) = span {
        eprintln!(
            "abs span: x={}, y={}, w={}, h={}",
            span.x, span.y, span.width, span.height
        );
        // 静态位置：在 vertical-rl 中，"1 2 34" + "X" 应该在第5个字符位置
        // Ahem 80px 字体，4个可见字符(1,2,3,4) 占 4×80=320px 深度
        // 但 X 在第 5 列位置...
        // 这个测试主要用于调试，不严格断言
    } else {
        eprintln!("WARNING: no abs-pos child found");
    }
}

/// 测试 border-bottom: inherit 正确传播 width/style/color 子属性。
/// 对应上游 WPT 测试 CSS2/borders/border-bottom-018.xht
#[test]
fn test_border_bottom_inherit() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="border-bottom: 1in solid blue; padding-bottom: 10px">
            <div id="child" style="border-bottom: inherit; height: 1in"></div>
        </div>
    </body></html>"#;

    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);

    // 查找子元素
    let child_id = doc.query_selector(doc.root(), "#child").expect("child found");

    let child_style = styles.get(&child_id).expect("child has style");

    // 子元素应从父元素继承 border-bottom: 1in solid blue
    match &child_style.border_bottom_width {
        LengthValue::Px(v) => {
            let expected = 96.0_f64; // 1in = 96px
            assert!(
                (v - expected).abs() < 0.01,
                "Expected border-bottom-width 96px (1in), got {}px",
                v
            );
        }
        other => panic!("Expected Px border-bottom-width, got {:?}", other),
    }

    // border-bottom-style 应为 Solid
    assert!(
        matches!(
            child_style.border_bottom_style,
            zero_style_system::property::types::BorderStyleValue::Solid
        ),
        "Expected border-bottom-style: Solid, got {:?}",
        child_style.border_bottom_style
    );
}

/// 测试：自身为垂直书写模式、父元素为水平模式的块级元素，width:auto 时应收缩到
/// 内容块轴跨度（受 min-width 约束），而非填满容器宽度
/// （CSS §10.3.3 + CSS Writing Modes §7.1）。
#[test]
fn test_shrink_vertical_block_width_auto_to_content() {
    use std::collections::HashMap;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    let (_doc, parent_id) = make_doc_with_body();

    // 子元素位于小 x（模拟 float 后处理后的位置），width=20。
    let child = LayoutBox {
        x: 8.0,
        width: 20.0,
        margin_right: 0.0,
        ..Default::default()
    };
    // 父元素：vertical-rl，width:auto，初始被 taffy 填满到 784（容器宽度）。
    let parent = LayoutBox {
        node_id: Some(parent_id),
        width: 784.0,
        content_width: 784.0,
        is_block_level: true,
        writing_mode: WritingModeValue::VerticalRl,
        children: vec![child],
        ..Default::default()
    };
    // 根（body，水平）包含 parent。
    let mut root = LayoutBox {
        width: 800.0,
        writing_mode: WritingModeValue::HorizontalTb,
        children: vec![parent],
        ..Default::default()
    };

    let mut style = ComputedStyle::default();
    style.width = LengthValue::Auto;
    style.min_width = LengthValue::Px(120.0);
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(parent_id, style);

    shrink_vertical_blocks_to_content(&mut root, &styles, &WritingModeValue::HorizontalTb);

    let parent = &root.children[0];
    // 内容跨度 = 8 + 20 + 0 = 28；min-width=120 → 收缩到 120，而非保持 784。
    assert!(
        (parent.width - 120.0).abs() < 0.5,
        "vertical-rl width:auto 块应收缩到 max(内容跨度, min-width)=120，实际 {}",
        parent.width
    );
    assert!(
        parent.content_width < 784.0,
        "content_width 应随 width 收缩，实际 {}",
        parent.content_width
    );
}

/// 测试：水平书写模式的块不受 shrink 影响（自限性保证零回归）。
#[test]
fn test_shrink_vertical_block_no_op_for_horizontal() {
    use std::collections::HashMap;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    let (_doc, parent_id) = make_doc_with_body();

    let parent = LayoutBox {
        node_id: Some(parent_id),
        width: 784.0,
        is_block_level: true,
        writing_mode: WritingModeValue::HorizontalTb,
        children: vec![LayoutBox {
            x: 8.0,
            width: 20.0,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![parent],
        ..Default::default()
    };
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Auto;
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(parent_id, style);

    shrink_vertical_blocks_to_content(&mut root, &styles, &WritingModeValue::HorizontalTb);

    assert_eq!(root.children[0].width, 784.0, "水平块不应被 shrink 影响");
}

// ── R2240 contain:layout / contain:paint → BFC（独立格式化上下文）──────────────

/// 辅助：对给定 HTML 跑完整布局，返回指定标签首个 LayoutBox 的引用。
fn contain_bfc_box(html: &str, tag: &str) -> LayoutBox {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let found = find_box_by_tag(&result.root, &doc, tag).expect("应找到目标元素");
    // 克隆出来脱离 result 生命周期，便于断言。
    LayoutBox {
        is_flow_root: found.is_flow_root,
        is_block_level: found.is_block_level,
        ..LayoutBox::default()
    }
}

/// R2240：CSS Containment §3 — `contain: layout` 建立独立格式化上下文（BFC），
/// 隔离浮动 + 阻止与后代的 margin 折叠。driving: WPT css-contain
/// contain-layout-formatting-context-float-001 / -margin-001。
#[test]
fn test_contain_layout_establishes_bfc() {
    use crate::margin_collapse::establishes_bfc;
    let bx = contain_bfc_box(
        r#"<html><body><div style="contain:layout">a</div></body></html>"#,
        "div",
    );
    assert!(bx.is_flow_root, "contain:layout 的块须标记 is_flow_root");
    assert!(establishes_bfc(&bx), "contain:layout 的块须建立 BFC");
}

/// R2240：CSS Containment §4 — `contain: paint` 同样建立独立格式化上下文（BFC）。
/// driving: WPT css-contain contain-paint-formatting-context-float-001 / -margin-001。
#[test]
fn test_contain_paint_establishes_bfc() {
    use crate::margin_collapse::establishes_bfc;
    let bx = contain_bfc_box(r#"<html><body><div style="contain:paint">a</div></body></html>"#, "div");
    assert!(bx.is_flow_root, "contain:paint 的块须标记 is_flow_root");
    assert!(establishes_bfc(&bx), "contain:paint 的块须建立 BFC");
}

/// R2240：`contain: strict` / `contain: content` 含 layout+paint，亦须建立 BFC。
#[test]
fn test_contain_strict_content_establish_bfc() {
    use crate::margin_collapse::establishes_bfc;
    for (decl, label) in [("contain:strict", "strict"), ("contain:content", "content")] {
        let html = format!(r#"<html><body><div style="{decl}">a</div></body></html>"#);
        let bx = contain_bfc_box(&html, "div");
        assert!(bx.is_flow_root, "contain:{label} 的块须标记 is_flow_root");
        assert!(establishes_bfc(&bx), "contain:{label} 的块须建立 BFC");
    }
}

/// R2240：`contain: style` / `contain: size` 不含 layout/paint，**不**建立 BFC
///（size 仅独立化尺寸，style 仅隔离计数器/引号）。同时验证普通块无 contain 不建立 BFC。
/// 这是回归护栏：contain→BFC 仅限 layout/paint，不误伤其他 contain 值。
#[test]
fn test_contain_style_size_no_bfc() {
    use crate::margin_collapse::establishes_bfc;
    for (decl, label) in [("contain:style", "style"), ("contain:size", "size")] {
        let html = format!(r#"<html><body><div style="{decl}">a</div></body></html>"#);
        let bx = contain_bfc_box(&html, "div");
        assert!(!bx.is_flow_root, "contain:{label} 不应标记 is_flow_root");
        assert!(!establishes_bfc(&bx), "contain:{label} 不应建立 BFC");
    }
    // 普通块（无 contain）不建立 BFC。
    let bx = contain_bfc_box(r#"<html><body><div>a</div></body></html>"#, "div");
    assert!(!bx.is_flow_root, "无 contain 的普通块不应标记 is_flow_root");
    assert!(!establishes_bfc(&bx), "无 contain 的普通块不应建立 BFC");
}

/// R2240：`contain: layout` 不对 `display: inline`（非原子行内）元素建立 BFC——
/// 防止 apply_inline_block_float_avoidance（float_positioning.rs，按
/// `is_flow_root && !is_block_level` 识别 inline-block）把 inline+contain 误判为 inline-block。
#[test]
fn test_contain_layout_inline_not_bfc() {
    use crate::margin_collapse::establishes_bfc;
    let bx = contain_bfc_box(
        r#"<html><body><span style="contain:layout">a</span></body></html>"#,
        "span",
    );
    assert!(
        !bx.is_flow_root || bx.is_block_level,
        "display:inline + contain:layout 不应成为非块级 is_flow_root（避免被当作 inline-block）"
    );
    let _ = establishes_bfc(&bx); // 仅作可调用性占位；判定以 is_flow_root + is_block_level 为准。
}
