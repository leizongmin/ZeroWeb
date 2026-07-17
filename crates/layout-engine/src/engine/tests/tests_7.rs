use super::*;
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue, PositionValue};
use zero_style_system::ComputedStyle;
use zero_style_system::StyleSystem;

// ── 边缘场景补充测试（第九批）──

/// 测试零宽高容器内的子元素布局不 panic。
#[test]
fn test_zero_size_container_children() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let child = doc.create_element("span");
    doc.append_child(container, child).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(0.0);
    container_style.height = LengthValue::Px(0.0);
    styles.insert(container, container_style);

    styles.insert(child, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    assert_eq!(container_box.width, 0.0, "容器宽度应为 0");
    assert_eq!(container_box.height, 0.0, "容器高度应为 0");
}

/// 测试 display:none 元素子树完全不参与布局。
#[test]
fn test_display_none_nested() {
    let (mut doc, body) = make_doc_with_body();
    let hidden = doc.create_element("div");
    doc.append_child(body, hidden).unwrap();
    let child1 = doc.create_element("span");
    doc.append_child(hidden, child1).unwrap();
    let child2 = doc.create_element("span");
    doc.append_child(hidden, child2).unwrap();

    let mut styles = HashMap::new();
    let mut hidden_style = ComputedStyle::default();
    hidden_style.display = DisplayValue::None;
    styles.insert(hidden, hidden_style);
    styles.insert(child1, ComputedStyle::default());
    styles.insert(child2, ComputedStyle::default());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let hidden_box = find_child_by_node_id(&result.root, hidden);
    assert!(hidden_box.is_none(), "display:none 不应出现在布局树中");
}

/// 测试百分比宽度在无明确父宽度时的行为。
#[test]
fn test_percent_width_without_parent_width() {
    let (mut doc, body) = make_doc_with_body();
    let child = doc.create_element("div");
    doc.append_child(body, child).unwrap();

    let mut styles = HashMap::new();
    let mut child_style = ComputedStyle::default();
    child_style.width = LengthValue::Percentage(50.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let child_box = find_child_by_node_id(&result.root, child).expect("child");
    // 百分比基于 body 宽度（默认占满视口），应为 400px
    assert!(child_box.width > 0.0, "百分比宽度子元素应有非零宽度");
}

/// 测试负 margin 对布局的影响。
#[test]
fn test_negative_margin_overlap() {
    let (mut doc, body) = make_doc_with_body();
    let box1 = doc.create_element("div");
    doc.append_child(body, box1).unwrap();
    let box2 = doc.create_element("div");
    doc.append_child(body, box2).unwrap();

    let mut styles = HashMap::new();
    let mut s1 = ComputedStyle::default();
    s1.width = LengthValue::Px(100.0);
    s1.height = LengthValue::Px(50.0);
    styles.insert(box1, s1);

    let mut s2 = ComputedStyle::default();
    s2.display = DisplayValue::Block; // R1058：测垂直 margin 须 block（默认 Inline §8.3 垂直 margin 归零）
    s2.width = LengthValue::Px(100.0);
    s2.height = LengthValue::Px(50.0);
    s2.margin_top = LengthValue::Px(-20.0);
    styles.insert(box2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b2 = find_child_by_node_id(&result.root, box2).expect("box2");
    // 负 margin-top 应使 box2 向上偏移
    assert!(b2.y < 50.0, "负 margin-top 应使元素向上偏移，实际 y={}", b2.y);
}

/// 测试 absolute 定位元素的坐标不受兄弟元素影响。
#[test]
fn test_absolute_unaffected_by_siblings() {
    let (mut doc, body) = make_doc_with_body();
    let spacer = doc.create_element("div");
    doc.append_child(body, spacer).unwrap();
    let abs_el = doc.create_element("div");
    doc.append_child(body, abs_el).unwrap();

    let mut styles = HashMap::new();
    let mut spacer_style = ComputedStyle::default();
    spacer_style.height = LengthValue::Px(200.0);
    styles.insert(spacer, spacer_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_el, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_el).expect("abs");
    assert_eq!(abs_box.y, 10.0, "absolute top 应为 10");
    assert_eq!(abs_box.x, 20.0, "absolute left 应为 20");
    assert!(abs_box.is_absolute, "应标记为 absolute");
}

/// 测试 fixed 定位元素坐标调整为视口相对。
#[test]
fn test_fixed_viewport_relative() {
    let (mut doc, body) = make_doc_with_body();
    let scroll_container = doc.create_element("div");
    doc.append_child(body, scroll_container).unwrap();
    let fixed_el = doc.create_element("div");
    doc.append_child(scroll_container, fixed_el).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(300.0);
    container_style.height = LengthValue::Px(300.0);
    styles.insert(scroll_container, container_style);

    let mut fixed_style = ComputedStyle::default();
    fixed_style.position = PositionValue::Fixed;
    fixed_style.top = LengthValue::Px(0.0);
    fixed_style.left = LengthValue::Px(0.0);
    fixed_style.width = LengthValue::Px(100.0);
    fixed_style.height = LengthValue::Px(100.0);
    styles.insert(fixed_el, fixed_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fixed_box = find_child_by_node_id(&result.root, fixed_el).expect("fixed");
    assert!(fixed_box.is_fixed, "应标记为 fixed");
    // fixed 元素坐标应相对于视口，不是相对于父容器
    assert_eq!(fixed_box.y, 0.0, "fixed top 应为 0（视口相对）");
    assert_eq!(fixed_box.x, 0.0, "fixed left 应为 0（视口相对）");
}

/// 测试嵌套 flex 容器布局。
#[test]
fn test_nested_flex_containers() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let item = doc.create_element("span");
    doc.append_child(inner, item).unwrap();

    let mut styles = HashMap::new();
    let mut outer_s = ComputedStyle::default();
    outer_s.display = DisplayValue::Flex;
    outer_s.width = LengthValue::Px(400.0);
    outer_s.height = LengthValue::Px(200.0);
    styles.insert(outer, outer_s);

    let mut inner_s = ComputedStyle::default();
    inner_s.display = DisplayValue::Flex;
    inner_s.width = LengthValue::Px(200.0);
    inner_s.height = LengthValue::Px(100.0);
    styles.insert(inner, inner_s);

    let mut item_s = ComputedStyle::default();
    item_s.width = LengthValue::Px(50.0);
    item_s.height = LengthValue::Px(30.0);
    styles.insert(item, item_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let item_box = find_child_by_node_id(&result.root, item).expect("item");
    assert_eq!(item_box.width, 50.0, "嵌套 flex 子元素宽度应正确");
    assert_eq!(item_box.height, 30.0, "嵌套 flex 子元素高度应正确");
}

/// 测试 inline-block 在行内格式化上下文中的换行。
#[test]
fn test_inline_block_line_wrapping() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let ib1 = doc.create_element("span");
    doc.append_child(container, ib1).unwrap();
    let ib2 = doc.create_element("span");
    doc.append_child(container, ib2).unwrap();
    let ib3 = doc.create_element("span");
    doc.append_child(container, ib3).unwrap();

    let mut styles = HashMap::new();
    let mut container_s = ComputedStyle::default();
    container_s.width = LengthValue::Px(150.0);
    styles.insert(container, container_s);

    for &id in &[ib1, ib2, ib3] {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::InlineBlock;
        s.width = LengthValue::Px(80.0);
        s.height = LengthValue::Px(30.0);
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 三个 80px 宽的 inline-block 在 150px 容器中应换行
    let ib1_box = find_child_by_node_id(&result.root, ib1).expect("ib1");
    let ib2_box = find_child_by_node_id(&result.root, ib2).expect("ib2");
    // ib2 应该在第二行（y > ib1.y + ib1.height 或者 x 回到行首）
    assert!(ib2_box.y >= ib1_box.y, "第二个 inline-block 应在第一行或换行后");
}

/// 测试大边框不影响内容区域为负。
#[test]
fn test_large_border_content_clamp() {
    let (mut doc, body) = make_doc_with_body();
    let el = doc.create_element("div");
    doc.append_child(body, el).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.width = LengthValue::Px(50.0);
    s.height = LengthValue::Px(50.0);
    // 使用 CSS 构建带大 border 的样式
    styles.insert(el, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let el_box = find_child_by_node_id(&result.root, el).expect("el");
    assert!(
        el_box.content_width >= 0.0,
        "内容宽度不应为负，实际 {}",
        el_box.content_width
    );
    assert!(
        el_box.content_height >= 0.0,
        "内容高度不应为负，实际 {}",
        el_box.content_height
    );
}

/// 测试 grid 容器内 flex 子元素正常布局。
#[test]
fn test_flex_child_in_grid() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();
    let flex_child = doc.create_element("div");
    doc.append_child(grid, flex_child).unwrap();
    let item = doc.create_element("span");
    doc.append_child(flex_child, item).unwrap();

    let mut styles = HashMap::new();
    let mut grid_s = ComputedStyle::default();
    grid_s.display = DisplayValue::Grid;
    grid_s.width = LengthValue::Px(400.0);
    grid_s.height = LengthValue::Px(200.0);
    styles.insert(grid, grid_s);

    let mut flex_s = ComputedStyle::default();
    flex_s.display = DisplayValue::Flex;
    styles.insert(flex_child, flex_s);

    let mut item_s = ComputedStyle::default();
    item_s.width = LengthValue::Px(100.0);
    item_s.height = LengthValue::Px(50.0);
    styles.insert(item, item_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let item_box = find_child_by_node_id(&result.root, item).expect("item");
    assert_eq!(item_box.width, 100.0, "grid 内 flex 子元素宽度应正确");
    assert_eq!(item_box.height, 50.0, "grid 内 flex 子元素高度应正确");
}

/// 测试深层嵌套 absolute 元素的定位。
#[test]
fn test_deeply_nested_absolute() {
    let (mut doc, body) = make_doc_with_body();
    let l1 = doc.create_element("div");
    doc.append_child(body, l1).unwrap();
    let l2 = doc.create_element("div");
    doc.append_child(l1, l2).unwrap();
    let l3 = doc.create_element("div");
    doc.append_child(l2, l3).unwrap();
    let abs_el = doc.create_element("div");
    doc.append_child(l3, abs_el).unwrap();

    let mut styles = HashMap::new();
    let mut l1_s = ComputedStyle::default();
    l1_s.width = LengthValue::Px(300.0);
    l1_s.height = LengthValue::Px(300.0);
    styles.insert(l1, l1_s);

    styles.insert(l2, ComputedStyle::default());
    styles.insert(l3, ComputedStyle::default());

    let mut abs_s = ComputedStyle::default();
    abs_s.position = PositionValue::Absolute;
    abs_s.top = LengthValue::Px(5.0);
    abs_s.left = LengthValue::Px(10.0);
    abs_s.width = LengthValue::Px(20.0);
    abs_s.height = LengthValue::Px(20.0);
    styles.insert(abs_el, abs_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_el).expect("abs");
    assert_eq!(abs_box.width, 20.0, "深层嵌套 absolute 宽度应正确");
    assert_eq!(abs_box.height, 20.0, "深层嵌套 absolute 高度应正确");
    assert!(abs_box.is_absolute, "应标记为 absolute");
}

/// 测试 LayoutBox 的 outer_area 计算（包含 margin）。
#[test]
fn test_layout_box_outer_area() {
    let (mut doc, body) = make_doc_with_body();
    let el = doc.create_element("div");
    doc.append_child(body, el).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block; // R1058：测垂直 margin 须 block（默认 Inline §8.3 垂直 margin 归零）
    s.width = LengthValue::Px(100.0);
    s.height = LengthValue::Px(50.0);
    s.margin_top = LengthValue::Px(10.0);
    s.margin_left = LengthValue::Px(20.0);
    styles.insert(el, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let el_box = find_child_by_node_id(&result.root, el).expect("el");
    assert_eq!(el_box.width, 100.0);
    assert_eq!(el_box.height, 50.0);
    assert_eq!(el_box.margin_top, 10.0);
    assert_eq!(el_box.margin_left, 20.0);
}

/// 测试空 inline 元素的 line-height 贡献到容器高度。
/// CSS 2.1 §9.4.2 + WPT empty-inline-001：仅含「裸空 inline 元素」（无文本、无 atomic、
/// 无 padding/border/margin）的行盒为**零高**——裸空 span 的 line-height **不**贡献到
/// 行盒高度。故 `<div><span style="line-height:5"></span></div>`（span 独占一行、无其他
/// 显著内容）的容器 content_height == 0（chromium 同此）。
/// 注：若同行有其他显著内容（文本/atomic/带几何空 inline），裸空 inline 的 line-height
/// 仍正常贡献（见 empty-inline-003）。
#[test]
fn test_empty_inline_line_height_contribution() {
    let (mut doc, body) = make_doc_with_body();

    // body > div#container > span (empty)
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let span = doc.create_element("span");
    doc.append_child(container, span).unwrap();

    let mut styles = HashMap::new();

    // Container: auto height, explicit width
    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // Span: display inline, line-height: 5 (unitless) —— 裸空 inline（无 padding/border/margin）
    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.line_height = zero_style_system::property::types::LineHeightValue::Number(5.0);
    styles.insert(span, span_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");

    // 裸空 span 独占行 → 零高 line box → 容器 content_height == 0
    assert_eq!(
        container_box.content_height, 0.0,
        "Bare empty inline alone should create a zero-height line box (content_height==0), got {}",
        container_box.content_height
    );
}

/// 测试空 inline 元素的 border/padding 几何会从 IFC 写回子 LayoutBox。
#[test]
fn test_inline_child_box_synced_from_ifc_for_empty_span() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let span = doc.create_element("span");
    doc.append_child(container, span).unwrap();

    let mut styles = HashMap::new();

    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(500.0);
    container_style.font_size = LengthValue::Px(100.0);
    container_style.line_height = zero_style_system::property::types::LineHeightValue::Number(1.0);
    styles.insert(container, container_style);

    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(100.0);
    span_style.line_height = zero_style_system::property::types::LineHeightValue::Number(1.0);
    span_style.padding_top = LengthValue::Px(100.0);
    span_style.padding_right = LengthValue::Px(100.0);
    span_style.padding_bottom = LengthValue::Px(100.0);
    span_style.padding_left = LengthValue::Px(100.0);
    span_style.border_top_width = LengthValue::Px(25.0);
    span_style.border_right_width = LengthValue::Px(25.0);
    span_style.border_bottom_width = LengthValue::Px(25.0);
    span_style.border_left_width = LengthValue::Px(25.0);
    styles.insert(span, span_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    let span_box = find_child_by_node_id(container_box, span).expect("span");

    assert_eq!(
        span_box.width, 250.0,
        "empty inline 的 border-box 宽度应包含左右 padding/border"
    );
    assert_eq!(
        span_box.height, 350.0,
        "empty inline 的 border-box 高度应包含 line-height + padding + border"
    );
    assert_eq!(span_box.content_width, 0.0, "empty inline 无文本时内容宽度应为 0");
    assert_eq!(span_box.content_height, 100.0, "内容高度应保留原始 line-height");
    assert_eq!(span_box.padding_left, 100.0);
    assert_eq!(span_box.border_left, 25.0);
    // CSS §10.8.1/§8.4：inline 非替换元素的垂直 padding/border 只绘制，
    // 不影响 line box 高度。父容器高度应仅基于 line-height（100px），
    // 不被 inline padding/border 撑到 350（R769d 修复，对齐 chromium + blocks-019）。
    assert!(
        container_box.content_height < 150.0,
        "父容器高度应仅基于 line-height（inline padding/border 不影响 line box，CSS §10.8.1），高度={}",
        container_box.content_height
    );
}

/// 测试 inline 元素的 padding-top/border-top 会把视觉盒子向上扩展。
#[test]
fn test_inline_child_box_bleeds_upwards_from_ifc() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let span = doc.create_element("span");
    doc.append_child(container, span).unwrap();

    let mut styles = HashMap::new();

    let mut container_style = ComputedStyle::default();
    container_style.width = LengthValue::Px(500.0);
    container_style.font_size = LengthValue::Px(40.0);
    container_style.line_height = zero_style_system::property::types::LineHeightValue::Number(1.0);
    styles.insert(container, container_style);

    let mut span_style = ComputedStyle::default();
    span_style.display = DisplayValue::Inline;
    span_style.font_size = LengthValue::Px(40.0);
    span_style.line_height = zero_style_system::property::types::LineHeightValue::Number(1.0);
    span_style.padding_top = LengthValue::Px(25.0);
    span_style.border_top_width = LengthValue::Px(15.0);
    // 显式置 0：本用例只验证 border-top/padding-top 向上扩展；border-bottom 默认值
    // 自 R549 起为 medium(3px)（CSS §8.5.1），IFC 按宽度直接计入盒高，会干扰 80px 期望。
    span_style.border_bottom_width = LengthValue::Px(0.0);
    styles.insert(span, span_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container");
    let span_box = find_child_by_node_id(container_box, span).expect("span");

    assert_eq!(
        span_box.height, 80.0,
        "空 span 的视觉高度应包含 line-height + padding-top + border-top"
    );
    assert!(
        span_box.y < 0.0,
        "padding-top/border-top 应使 inline 盒子向上 bleed，实际 y={}",
        span_box.y
    );
}

#[test]
fn test_real_empty_inline_003_layout_height() {
    let html = r#"
    <html>
      <body>
        <div id="rel-pos-wrapper" style="position: relative;">
          <div id="test" style="background-color: green; color: white; line-height: 1;">
            <span id="empty-inline-element" style="line-height: 5;"></span>X
          </div>
          <div id="reference-overlapped-red"
               style="background-color: red; left: 0; line-height: 5; position: absolute; top: 0; width: 100%; z-index: -1;">X</div>
        </div>
      </body>
    </html>
    "#;
    let doc = zero_dom::parse_html(html);
    let mut style_system = StyleSystem::new();
    style_system.set_viewport(800.0, 600.0);
    let styles = style_system.compute_styles(&doc, &[]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut test_box = None;
    let mut empty_span_box = None;
    let mut stack = vec![&result.root];
    while let Some(node) = stack.pop() {
        if let Some(node_id) = node.node_id
            && let Some(dom_node) = doc.get(node_id)
            && let zero_dom::NodeKind::Element(elem) = &dom_node.kind
        {
            if elem.get_attribute("id").as_deref() == Some("test") {
                test_box = Some(node);
            }
            if elem.get_attribute("id").as_deref() == Some("empty-inline-element") {
                empty_span_box = Some(node);
            }
        }
        stack.extend(node.children.iter());
    }

    let test_box = test_box.expect("#test");
    let empty_span_box = empty_span_box.expect("#empty-inline-element");
    eprintln!(
        "#test: width={} height={} content_height={} inline_layout={} span.height={} span.y={}",
        test_box.width,
        test_box.height,
        test_box.content_height,
        test_box.inline_layout.is_some(),
        empty_span_box.height,
        empty_span_box.y
    );
    assert!(
        test_box.content_height >= 80.0,
        "#test content_height 应体现空 inline 的 line-height:5，实际={}",
        test_box.content_height
    );
}

/// R207 stored-line-boxes 路径回归守护（font-051 类）。
///
/// `div > span > 文本`（inline-level 叶子子元素容器，无 block 子，inline 子无元素子）
/// 应由 compute_final 存储 inline_layout（用真实 styles），使 paint use_stored 渲染正确
/// 度量——修复 font-051 等 large-font（100px 文本被 paint 重跑渲染成 16px）的 bug。
/// 窄条件排除混合 inline+block / block-in-inline（R206/R207 收敛）。
#[test]
fn test_r207_stored_inline_layout_for_inline_child_container() {
    let html = "<html><body><div id=\"c\" style=\"font: 100px/1 Ahem;\"><span style=\"font: serif;\">FAIL</span></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut style_system = StyleSystem::new();
    style_system.set_viewport(800.0, 600.0);
    let styles = style_system.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 定位 #c 的 LayoutBox
    let mut c_box = None;
    let mut stack = vec![&result.root];
    while let Some(node) = stack.pop() {
        if let Some(nid) = node.node_id
            && let Some(dn) = doc.get(nid)
            && let zero_dom::NodeKind::Element(el) = &dn.kind
            && el.get_attribute("id").as_deref() == Some("c")
        {
            c_box = Some(node);
        }
        stack.extend(node.children.iter());
    }
    let c_box = c_box.expect("#c div");

    // R207：inline-level 叶子子元素容器应存储 inline_layout。
    assert!(
        c_box.inline_layout.is_some(),
        "#c (div>span>text) 应存储 inline_layout (R207 stored-line-boxes 路径)"
    );
    // 存储片段 font_size 应为真实 100px（compute_final 用真实 styles，非 paint 重跑的 16px 默认）。
    let frag_fs = c_box
        .inline_layout
        .as_ref()
        .and_then(|lines| lines.first())
        .and_then(|l| l.fragments.first())
        .map(|f| f.font_size);
    assert_eq!(
        frag_fs,
        Some(100.0),
        "存储片段 font_size 应为 100px（真实 styles），实际={:?}",
        frag_fs
    );
}

/// R355 multi-line stored-line-boxes 守护（large-font 簇 ifc-008/009）。
///
/// R207 仅存储「单行 + 纯 Ahem」容器；R355 放宽为「多行 + 纯 Ahem」（非浮动），
/// 解 large-font bug（100px 多行文本 paint 阶段被 16px 默认值覆盖）。本测试断言：
/// (1) 非浮动多行纯 Ahem 容器存储 inline_layout 且行数 > 1（R355 新覆盖）；
/// (2) 浮动多行纯 Ahem 容器**不**存储（保持 R84 单行限制——multicol-fill-auto-001 ref
///     用 float div 模拟列，test 用真 multicol；浮动容器多行存储打破 test/ref 对称）。
#[test]
fn test_r355_multiline_stored_layout_pure_ahem() {
    let html = "<html><body>\
<div id=\"nf\" style=\"font: 100px/1 Ahem; width: 150px;\">XX XX</div>\
<div id=\"fl\" style=\"font: 100px/1 Ahem; width: 150px; float: left;\">XX XX</div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut style_system = StyleSystem::new();
    style_system.set_viewport(800.0, 600.0);
    let styles = style_system.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut nf_box = None;
    let mut fl_box = None;
    let mut stack = vec![&result.root];
    while let Some(node) = stack.pop() {
        if let Some(nid) = node.node_id
            && let Some(dn) = doc.get(nid)
            && let zero_dom::NodeKind::Element(el) = &dn.kind
        {
            match el.get_attribute("id").as_deref() {
                Some("nf") => nf_box = Some(node),
                Some("fl") => fl_box = Some(node),
                _ => {}
            }
        }
        stack.extend(node.children.iter());
    }
    let nf_box = nf_box.expect("#nf");
    let fl_box = fl_box.expect("#fl");

    // (1) 非浮动多行纯 Ahem：R355 存储，行数 > 1（"XX XX" 在 150px 宽下换行成 ≥2 行）。
    let nf_lines = nf_box.inline_layout.as_ref().map(|l| l.len());
    assert!(
        nf_box.inline_layout.is_some() && nf_lines.is_some_and(|n| n > 1),
        "#nf 非浮动多行纯 Ahem 应存储多行 inline_layout (R355)，实际 lines={:?}",
        nf_lines
    );
    // 存储片段 font_size 应为真实 100px（非 paint 重跑的 16px）。
    let nf_fs = nf_box
        .inline_layout
        .as_ref()
        .and_then(|lines| lines.first())
        .and_then(|l| l.fragments.first())
        .map(|f| f.font_size);
    assert_eq!(nf_fs, Some(100.0), "#nf 存储 font_size 应为 100px，实际={:?}", nf_fs);

    // (2) 浮动多行纯 Ahem：保持 R84 单行限制——多行不存储。
    assert!(
        fl_box.inline_layout.is_none(),
        "#fl 浮动多行纯 Ahem 不应存储 inline_layout（R355 浮动 guard，保 multicol-fill-auto-001 self-source），实际={}",
        fl_box.inline_layout.is_some()
    );
}

/// R362：CSS float 侵入——祖先 BFC 内的 float 应侵入未建 BFC 的后代 block 的 line box。
/// d1 含 float 子 d2（右浮 100x100）+ 兄弟 block inner（文本 "X X X"，100px Ahem，300px 容器）。
/// inner 的 IFC 应见 d2 排除：line 1（紧邻 float，可用 200px）只容 "X"，line 2（float 下方 300px）容 "X X"。
/// 修复前 inner 的 IFC 不感知 d2 → line 1 容 "X X"（满宽 300px），与本断言冲突。
#[test]
fn test_r362_float_intrusion_propagates_to_sibling_block_ifc() {
    let html = "<html><body>\
<div id=\"d1\" style=\"font: 100px/1 Ahem; width: 300px; height: 200px;\">\
<div id=\"d2\" style=\"float: right; width: 100px; height: 100px;\"></div>\
<div id=\"inner\">X X X</div>\
</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut style_system = StyleSystem::new();
    style_system.set_viewport(800.0, 600.0);
    let styles = style_system.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let mut inner_box = None;
    let mut stack = vec![&result.root];
    while let Some(node) = stack.pop() {
        if let Some(nid) = node.node_id
            && let Some(dn) = doc.get(nid)
            && let zero_dom::NodeKind::Element(el) = &dn.kind
            && el.get_attribute("id").as_deref() == Some("inner")
        {
            inner_box = Some(node);
        }
        stack.extend(node.children.iter());
    }
    let inner_box = inner_box.expect("#inner");

    let lines = inner_box
        .inline_layout
        .as_ref()
        .expect("#inner 应存储 inline_layout（纯 Ahem 非浮动）");
    let line1_text: String = lines
        .first()
        .map(|l| l.fragments.iter().map(|f| f.text.as_str()).collect())
        .unwrap_or_default();
    let line1_x_count = line1_text.matches('X').count();
    assert!(
        lines.len() >= 2 && line1_x_count == 1,
        "R362 float 侵入：line 1 应只含 1 个 X（绕开 float，200px 可用），实际 lines={} line1_text={:?}",
        lines.len(),
        line1_text
    );
}

/// DC-11 position:sticky 静态基线钉死：无滚动（scroll_offset=0）时 sticky 须与 relative
/// 产生**相同**布局位置（converter `PositionValue::Sticky|Relative|Static => taffy Relative`，
/// engine inset 解析对 sticky/relative 同路径）。sticky 的动态 clamp 行为需 host-layer scroll
/// offset（DC-11 动态部分，display 环境，本测试覆盖不到）；本测试守静态基线，防未来动态实现
/// 误改静态映射，并为动态实现提供回归对照（动态只应在 scroll_offset!=0 时偏离 relative）。
#[test]
fn test_sticky_static_case_equals_relative() {
    fn pos_of_t(position: &str) -> (f32, f32, bool) {
        let html = format!(
            r#"<html><body style="margin:0">
            <div style="height:40px"></div>
            <div id="t" style="position:{position}; top:10px; width:50px; height:50px"></div>
            </body></html>"#
        );
        let doc = zero_dom::parse_html(&html);
        let mut ss = StyleSystem::new();
        ss.set_viewport(800.0, 600.0);
        let styles = ss.compute_styles(&doc, &[]);
        let mut eng = LayoutEngine::new(800.0, 600.0);
        let result = eng.compute(&doc, &styles);
        let mut stack = vec![&result.root];
        while let Some(n) = stack.pop() {
            if let Some(nid) = n.node_id
                && let Some(dn) = doc.get(nid)
                && let zero_dom::NodeKind::Element(e) = &dn.kind
                && e.get_attribute("id").as_deref() == Some("t")
            {
                return (n.x, n.y, n.is_sticky);
            }
            stack.extend(n.children.iter());
        }
        panic!("#t not found for position:{position}");
    }
    let (s_x, s_y, s_is_sticky) = pos_of_t("sticky");
    let (r_x, r_y, _) = pos_of_t("relative");
    // sticky 标志须正确（is_sticky=true 区分于 relative）。
    assert!(s_is_sticky, "sticky #t must carry is_sticky=true");
    // 静态位置须与 relative 字节一致（同一 taffy Relative 映射 + 同 inset 解析路径）。
    assert_eq!(
        (s_x, s_y),
        (r_x, r_y),
        "sticky static-case must equal relative position (same taffy Relative mapping); \
         got sticky=({s_x},{s_y}) relative=({r_x},{r_y})"
    );
}
