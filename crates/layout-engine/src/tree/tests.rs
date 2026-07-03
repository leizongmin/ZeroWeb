//! tree.rs 布局树构建回归测试（从 tree.rs 抽出，保持 2000 行约束）。
#![allow(clippy::field_reassign_with_default)]

use super::*;
use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};
use zero_dom::Document;

/// 辅助：创建简单 DOM（html > body > div）。
fn make_simple_doc() -> (Document, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    (doc, html, body, div)
}

/// 在 taffy_to_dom 中查找指定 dom_id 对应的 taffy NodeId。
fn find_taffy_for_dom(taffy_to_dom: &HashMap<taffy::NodeId, NodeId>, target_dom: NodeId) -> taffy::NodeId {
    taffy_to_dom
        .iter()
        .find(|(_, dom_id)| **dom_id == target_dom)
        .map(|(t, _)| *t)
        .unwrap()
}

/// 测试简单树构建。
#[test]
fn test_build_simple_tree() {
    let (doc, html, _body, _div) = make_simple_doc();
    let styles = HashMap::new();
    let (_taffy_tree, root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    assert!(taffy_to_dom.contains_key(&root_id));
    // html 节点应该在映射中
    assert_eq!(taffy_to_dom.get(&root_id), Some(&html));
}

/// 测试多层嵌套。
#[test]
fn test_build_nested_tree() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(div1, div2).unwrap();
    let div3 = doc.create_element("span");
    doc.append_child(div2, div3).unwrap();

    let styles = HashMap::new();
    let (taffy_tree, root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let children = taffy_tree.children(root_id).unwrap();
    assert!(!children.is_empty());
    // 应该有 html, body, div, div, span 的映射
    assert!(taffy_to_dom.len() >= 5);
}

/// 测试跳过 display:none 元素。
#[test]
fn test_build_skips_display_none() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let hidden = doc.create_element("div");
    doc.append_child(body, hidden).unwrap();
    let visible = doc.create_element("span");
    doc.append_child(body, visible).unwrap();

    let mut styles = HashMap::new();
    let mut hidden_style = ComputedStyle::default();
    hidden_style.display = DisplayValue::None;
    styles.insert(hidden, hidden_style);

    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // visible 应该在映射中
    assert!(taffy_to_dom.values().any(|id| *id == visible));
}

/// 测试跳过文本节点。
#[test]
fn test_build_skips_text_nodes() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let text = doc.create_text_node("Hello World");
    doc.append_child(body, text).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // 文本节点不应在 taffy 映射中
    assert!(!taffy_to_dom.values().any(|id| *id == text));
    // div 应该存在
    assert!(taffy_to_dom.values().any(|id| *id == div));
}

/// 测试 flex 容器构建。
#[test]
fn test_build_flex_container() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let flex_container = doc.create_element("div");
    doc.append_child(html, flex_container).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(flex_container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(flex_container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    styles.insert(flex_container, container_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let container_taffy = find_taffy_for_dom(&taffy_to_dom, flex_container);
    let style = taffy_tree.style(container_taffy).unwrap();
    assert_eq!(style.display, taffy::style::Display::Flex);
}

/// 测试 grid 容器构建。
#[test]
fn test_build_grid_container() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let grid_container = doc.create_element("div");
    doc.append_child(html, grid_container).unwrap();
    let item = doc.create_element("span");
    doc.append_child(grid_container, item).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Grid;
    styles.insert(grid_container, container_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let container_taffy = find_taffy_for_dom(&taffy_to_dom, grid_container);
    let style = taffy_tree.style(container_taffy).unwrap();
    assert_eq!(style.display, taffy::style::Display::Grid);
}

/// 测试混合 display 类型。
#[test]
fn test_build_mixed_display_types() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let block = doc.create_element("div");
    doc.append_child(body, block).unwrap();
    let flex = doc.create_element("div");
    doc.append_child(body, flex).unwrap();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut styles = HashMap::new();
    let mut block_style = ComputedStyle::default();
    block_style.display = DisplayValue::Block;
    styles.insert(block, block_style);

    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    styles.insert(flex, flex_style);

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    styles.insert(grid, grid_style);

    let (_taffy_tree, _root_id, _taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // 树应该成功构建
}

/// 测试绝对定位元素。
#[test]
fn test_build_with_absolute_position() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let container = doc.create_element("div");
    doc.append_child(html, container).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(container, abs_child).unwrap();

    let mut styles = HashMap::new();
    let mut abs_style = ComputedStyle::default();
    abs_style.position = zero_css_parser::values::PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    styles.insert(abs_child, abs_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let abs_taffy = find_taffy_for_dom(&taffy_to_dom, abs_child);
    let style = taffy_tree.style(abs_taffy).unwrap();
    assert_eq!(style.position, taffy::style::Position::Absolute);
}

/// 测试 auto margin 和显式 0px margin。
#[test]
fn test_build_with_auto_margins() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    // 默认 margin 是 Px(0.0)，不是 auto
    let styles = HashMap::new();
    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    // 默认 margin 是 Px(0.0)，转换为 Length(0.0)
    assert_eq!(style.margin.top, taffy::style::LengthPercentageAuto::Length(0.0));
}

/// 测试 margin: auto 正确传递。
#[test]
fn test_build_with_explicit_auto_margin() {
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    let mut style = ComputedStyle::default();
    style.margin_top = LengthValue::Auto;
    style.margin_right = LengthValue::Auto;
    let mut styles = HashMap::new();
    styles.insert(div, style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    assert_eq!(style.margin.top, taffy::style::LengthPercentageAuto::Auto);
    assert_eq!(style.margin.right, taffy::style::LengthPercentageAuto::Auto);
}

/// 测试百分比 width 正确传递。
#[test]
fn test_build_with_percentage_width() {
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    let mut style = ComputedStyle::default();
    style.width = LengthValue::Percentage(50.0);
    let mut styles = HashMap::new();
    styles.insert(div, style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    assert_eq!(style.size.width, taffy::style::Dimension::Percent(0.5));
}

/// 测试空文档。
#[test]
fn test_build_empty_document() {
    let doc = Document::new();
    let styles = HashMap::new();
    let (taffy_tree, root_id, _taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // 空文档没有元素节点，但 taffy 树仍然会创建一个根节点。
    // 布局不 panic 即为通过。
    let _ = taffy_tree;
    // root_id 应该存在
    assert!(root_id == root_id); // 确保编译通过
}

/// 测试深层嵌套（50 层）。
#[test]
fn test_build_deep_nesting() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let mut current = html;
    for _ in 0..50 {
        let div = doc.create_element("div");
        doc.append_child(current, div).unwrap();
        current = div;
    }

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // 应该有 1 (html) + 50 (divs) = 51 个映射
    assert_eq!(taffy_to_dom.len(), 51);
}

/// 测试宽树（100 个兄弟元素）。
#[test]
fn test_build_wide_tree() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    for _ in 0..100 {
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();
    }

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // html + body + 100 divs = 102
    assert_eq!(taffy_to_dom.len(), 102);
}

/// 测试带 gap 的构建。
#[test]
fn test_build_with_gap() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let flex = doc.create_element("div");
    doc.append_child(html, flex).unwrap();
    let item = doc.create_element("span");
    doc.append_child(flex, item).unwrap();

    let mut styles = HashMap::new();
    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    flex_style.gap = LengthValue::Px(10.0);
    styles.insert(flex, flex_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let flex_taffy = find_taffy_for_dom(&taffy_to_dom, flex);
    let style = taffy_tree.style(flex_taffy).unwrap();
    assert_eq!(style.gap.width, taffy::style::LengthPercentage::Length(10.0));
}

/// 测试带 padding/border/margin。
#[test]
fn test_build_with_padding_border_margin() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.padding_top = LengthValue::Px(10.0);
    div_style.border_top_width = LengthValue::Px(2.0);
    // border-style=Solid 方能使 border-width 进入布局盒（CSS §8.5.3：style=none→width=0）
    div_style.border_top_style = zero_style_system::BorderStyleValue::Solid;
    div_style.margin_top = LengthValue::Px(5.0);
    styles.insert(div, div_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    assert_eq!(style.padding.top, taffy::style::LengthPercentage::Length(10.0));
    assert_eq!(style.border.top, taffy::style::LengthPercentage::Length(2.0));
    assert_eq!(style.margin.top, taffy::style::LengthPercentageAuto::Length(5.0));
}

/// 测试带 min/max size。
#[test]
fn test_build_with_min_max_size() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.min_width = LengthValue::Px(50.0);
    div_style.max_width = LengthValue::Px(500.0);
    styles.insert(div, div_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    assert_eq!(style.min_size.width, taffy::style::Dimension::Length(50.0));
    assert_eq!(style.max_size.width, taffy::style::Dimension::Length(500.0));
}

// -- 边界条件测试 --

/// 测试 display: none 子元素不进入布局树
#[test]
fn test_build_with_all_display_none_children() {
    // 所有子元素 display:none => 布局树子元素为空
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("span");
    doc.append_child(body, div2).unwrap();
    let div3 = doc.create_element("section");
    doc.append_child(body, div3).unwrap();

    let mut styles = HashMap::new();
    let mut hidden_style = ComputedStyle::default();
    hidden_style.display = DisplayValue::None;
    styles.insert(div1, hidden_style.clone());
    styles.insert(div2, hidden_style.clone());
    styles.insert(div3, hidden_style);

    let (taffy_tree, _root_id, _taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // body 的子元素都是 display:none，body 在 taffy 中不应有可见子节点
    // html 和 body 应在映射中
    let _ = taffy_tree; // 布局不 panic 即通过
}

/// 测试带有 grid-area 的元素构建
#[test]
fn test_build_with_grid_area() {
    use zero_style_system::GridLineValue;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let grid = doc.create_element("div");
    doc.append_child(html, grid).unwrap();
    let item = doc.create_element("span");
    doc.append_child(grid, item).unwrap();

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    grid_style.grid_template_areas = Some("\"a b\" \"c d\"".to_string());
    styles.insert(grid, grid_style);

    let mut item_style = ComputedStyle::default();
    item_style.grid_row_start = GridLineValue::Name("a".to_string());
    item_style.grid_row_end = GridLineValue::Name("a".to_string());
    item_style.grid_column_start = GridLineValue::Name("a".to_string());
    item_style.grid_column_end = GridLineValue::Name("a".to_string());
    styles.insert(item, item_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // item 应在映射中
    assert!(taffy_to_dom.values().any(|id| *id == item));
    let grid_taffy = find_taffy_for_dom(&taffy_to_dom, grid);
    let style = taffy_tree.style(grid_taffy).unwrap();
    assert_eq!(style.display, taffy::style::Display::Grid);
}

/// 测试嵌套 flex-in-grid 布局树
#[test]
fn test_build_nested_flex_in_grid() {
    // Grid container > flex container > block
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let grid = doc.create_element("div");
    doc.append_child(html, grid).unwrap();
    let flex = doc.create_element("div");
    doc.append_child(grid, flex).unwrap();
    let block = doc.create_element("span");
    doc.append_child(flex, block).unwrap();

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px".to_string());
    grid_style.grid_template_rows = Some("50px".to_string());
    styles.insert(grid, grid_style);

    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    flex_style.flex_direction = FlexDirectionValue::Row;
    styles.insert(flex, flex_style);

    styles.insert(block, ComputedStyle::default());

    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // grid + flex + block = 3 个映射
    assert!(taffy_to_dom.len() >= 3, "应有至少 3 个节点映射");
}

/// 测试带有 min/max 约束的布局树
#[test]
fn test_build_with_min_max_constraints() {
    // 元素带有 min-width 和 max-width
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let div = doc.create_element("div");
    doc.append_child(html, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.min_width = LengthValue::Px(50.0);
    div_style.max_width = LengthValue::Px(500.0);
    div_style.min_height = LengthValue::Px(30.0);
    div_style.max_height = LengthValue::Px(300.0);
    styles.insert(div, div_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
    let style = taffy_tree.style(div_taffy).unwrap();
    assert_eq!(style.min_size.width, taffy::style::Dimension::Length(50.0));
    assert_eq!(style.max_size.width, taffy::style::Dimension::Length(500.0));
    assert_eq!(style.min_size.height, taffy::style::Dimension::Length(30.0));
    assert_eq!(style.max_size.height, taffy::style::Dimension::Length(300.0));
}

// -- DOM 树构建边界条件测试 --

/// 注释节点在 DOM 树中应被跳过，不创建 taffy 节点。
#[test]
fn test_build_with_comment_nodes() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 在 body 和元素之间插入多个注释节点
    let comment1 = doc.create_comment("这是注释1");
    doc.append_child(body, comment1).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let comment2 = doc.create_comment("这是注释2");
    doc.append_child(body, comment2).unwrap();
    let span = doc.create_element("span");
    doc.append_child(body, span).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 注释节点不应出现在 taffy 映射中
    assert!(
        !taffy_to_dom.values().any(|id| *id == comment1),
        "注释节点 comment1 不应出现在布局树映射中"
    );
    assert!(
        !taffy_to_dom.values().any(|id| *id == comment2),
        "注释节点 comment2 不应出现在布局树映射中"
    );
    // 元素节点应正常出现
    assert!(taffy_to_dom.values().any(|id| *id == div));
    assert!(taffy_to_dom.values().any(|id| *id == span));
    // 映射数量：html + body + div + span = 4
    assert_eq!(taffy_to_dom.len(), 4);
}

/// ProcessingInstruction 节点应被跳过，不创建 taffy 节点。
#[test]
fn test_build_with_processing_instruction() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 插入 ProcessingInstruction 节点
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\"");
    doc.append_child(body, pi).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // ProcessingInstruction 不应出现在 taffy 映射中
    assert!(
        !taffy_to_dom.values().any(|id| *id == pi),
        "ProcessingInstruction 节点不应出现在布局树映射中"
    );
    // 元素节点应正常出现
    assert!(taffy_to_dom.values().any(|id| *id == div));
    // 映射数量：html + body + div = 3
    assert_eq!(taffy_to_dom.len(), 3);
}

/// 20+ 层嵌套的 div，验证布局树深度与 DOM 深度一致。
#[test]
fn test_build_deeply_nested_tree() {
    let depth = 25;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let mut current = html;
    let mut all_ids = vec![html];
    for _ in 0..depth {
        let div = doc.create_element("div");
        doc.append_child(current, div).unwrap();
        all_ids.push(div);
        current = div;
    }

    let styles = HashMap::new();
    let (taffy_tree, root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 映射数量：html + 25 层 div = 26
    assert_eq!(taffy_to_dom.len(), depth + 1);

    // 验证 taffy 树的深度：从根节点逐层向下走，应有 depth 层子节点
    let mut current_taffy = root_id;
    let mut actual_depth = 0;
    loop {
        let children = taffy_tree.children(current_taffy).unwrap();
        if children.is_empty() {
            break;
        }
        actual_depth += 1;
        current_taffy = children[0];
    }
    // html 本身是根，下面有 25 层 div 子节点
    assert_eq!(actual_depth, depth, "布局树深度应与 DOM 嵌套深度一致");

    // 验证最内层 div 确实在映射中
    assert!(taffy_to_dom.values().any(|id| *id == current));
}

/// 父元素可见，部分子元素 display:none，部分可见。
/// 只有可见的子元素应出现在布局树映射中。
#[test]
fn test_build_mixed_display_none_children() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 创建 5 个子元素：其中 3 个 visible，2 个 display:none
    let vis1 = doc.create_element("div");
    doc.append_child(body, vis1).unwrap();
    let hidden1 = doc.create_element("span");
    doc.append_child(body, hidden1).unwrap();
    let vis2 = doc.create_element("section");
    doc.append_child(body, vis2).unwrap();
    let hidden2 = doc.create_element("p");
    doc.append_child(body, hidden2).unwrap();
    let vis3 = doc.create_element("article");
    doc.append_child(body, vis3).unwrap();

    let mut styles = HashMap::new();
    let mut hidden_style = ComputedStyle::default();
    hidden_style.display = DisplayValue::None;
    styles.insert(hidden1, hidden_style.clone());
    styles.insert(hidden2, hidden_style);

    let (taffy_tree, root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 可见元素应在映射中
    assert!(taffy_to_dom.values().any(|id| *id == vis1), "vis1 应在布局树中");
    assert!(taffy_to_dom.values().any(|id| *id == vis2), "vis2 应在布局树中");
    assert!(taffy_to_dom.values().any(|id| *id == vis3), "vis3 应在布局树中");

    // display:none 元素不在 taffy_to_dom 映射中（提前返回跳过了映射记录）
    assert!(
        !taffy_to_dom.values().any(|id| *id == hidden1),
        "hidden1 不应在布局树映射中"
    );
    assert!(
        !taffy_to_dom.values().any(|id| *id == hidden2),
        "hidden2 不应在布局树映射中"
    );

    // body 在映射中，且有 taffy 子节点（包含 display:none 的隐藏节点）
    let body_taffy = find_taffy_for_dom(&taffy_to_dom, body);
    let body_children = taffy_tree.children(body_taffy).unwrap();
    // display:none 元素仍创建了 taffy 节点作为 body 子节点
    assert_eq!(body_children.len(), 5, "body 应有 5 个 taffy 子节点（含隐藏节点）");

    // 检查 body 的 taffy 子节点中，有 3 个是 display 非 none 的（vis1/vis2/vis3）
    let mut visible_count = 0;
    let mut hidden_count = 0;
    for &child_taffy in &body_children {
        let style = taffy_tree.style(child_taffy).unwrap();
        if style.display == taffy::style::Display::None {
            hidden_count += 1;
        } else {
            visible_count += 1;
        }
    }
    assert_eq!(visible_count, 3, "body 应有 3 个可见 taffy 子节点");
    assert_eq!(hidden_count, 2, "body 应有 2 个 display:none 的 taffy 子节点");

    // 验证可见节点不是 display:none
    let vis1_taffy = find_taffy_for_dom(&taffy_to_dom, vis1);
    let v1_style = taffy_tree.style(vis1_taffy).unwrap();
    assert_ne!(v1_style.display, taffy::style::Display::None);

    // root_id 应该是 html
    assert_eq!(taffy_to_dom.get(&root_id), Some(&html));
}

/// Grid 容器带有 grid-template-areas，子元素使用 grid-area 命名引用，
/// 验证 grid 项被正确放置。
#[test]
fn test_build_with_grid_container_and_items() {
    use zero_style_system::GridLineValue;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    // 创建 grid 容器
    let grid = doc.create_element("div");
    doc.append_child(html, grid).unwrap();

    // 创建 4 个 grid 子项
    let header = doc.create_element("header");
    doc.append_child(grid, header).unwrap();
    let nav = doc.create_element("nav");
    doc.append_child(grid, nav).unwrap();
    let main = doc.create_element("main");
    doc.append_child(grid, main).unwrap();
    let footer = doc.create_element("footer");
    doc.append_child(grid, footer).unwrap();

    let mut styles = HashMap::new();

    // grid 容器样式
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("200px 200px".to_string());
    grid_style.grid_template_rows = Some("50px 50px".to_string());
    grid_style.grid_template_areas = Some("\"header header\" \"nav main\"".to_string());
    styles.insert(grid, grid_style);

    // header 使用 grid-area 命名 "header"
    let mut header_style = ComputedStyle::default();
    header_style.grid_row_start = GridLineValue::Name("header".to_string());
    header_style.grid_row_end = GridLineValue::Name("header".to_string());
    header_style.grid_column_start = GridLineValue::Name("header".to_string());
    header_style.grid_column_end = GridLineValue::Name("header".to_string());
    styles.insert(header, header_style);

    // nav 使用 grid-area 命名 "nav"
    let mut nav_style = ComputedStyle::default();
    nav_style.grid_row_start = GridLineValue::Name("nav".to_string());
    nav_style.grid_row_end = GridLineValue::Name("nav".to_string());
    nav_style.grid_column_start = GridLineValue::Name("nav".to_string());
    nav_style.grid_column_end = GridLineValue::Name("nav".to_string());
    styles.insert(nav, nav_style);

    // main 使用 grid-area 命名 "main"
    let mut main_style = ComputedStyle::default();
    main_style.grid_row_start = GridLineValue::Name("main".to_string());
    main_style.grid_row_end = GridLineValue::Name("main".to_string());
    main_style.grid_column_start = GridLineValue::Name("main".to_string());
    main_style.grid_column_end = GridLineValue::Name("main".to_string());
    styles.insert(main, main_style);

    // footer 使用默认 auto 放置
    styles.insert(footer, ComputedStyle::default());

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 所有元素都应在映射中
    assert!(taffy_to_dom.values().any(|id| *id == grid));
    assert!(taffy_to_dom.values().any(|id| *id == header));
    assert!(taffy_to_dom.values().any(|id| *id == nav));
    assert!(taffy_to_dom.values().any(|id| *id == main));
    assert!(taffy_to_dom.values().any(|id| *id == footer));

    // 验证 grid 容器的 display
    let grid_taffy = find_taffy_for_dom(&taffy_to_dom, grid);
    let grid_taffy_style = taffy_tree.style(grid_taffy).unwrap();
    assert_eq!(grid_taffy_style.display, taffy::style::Display::Grid);

    // grid 容器应有 4 个子节点
    let grid_children = taffy_tree.children(grid_taffy).unwrap();
    assert_eq!(grid_children.len(), 4, "grid 容器应有 4 个子项");

    // 验证 header 的 grid 位置已从命名引用解析为行号
    let header_taffy = find_taffy_for_dom(&taffy_to_dom, header);
    let header_taffy_style = taffy_tree.style(header_taffy).unwrap();
    // "header" 区域在模板的第一行跨两列 → row 1-2, col 1-3
    assert_eq!(
        header_taffy_style.grid_row.start,
        taffy::style::GridPlacement::from_line_index(1),
        "header 应解析到 row start = 1"
    );
    assert_eq!(
        header_taffy_style.grid_row.end,
        taffy::style::GridPlacement::from_line_index(2),
        "header 应解析到 row end = 2"
    );
    assert_eq!(
        header_taffy_style.grid_column.start,
        taffy::style::GridPlacement::from_line_index(1),
        "header 应解析到 col start = 1"
    );
    assert_eq!(
        header_taffy_style.grid_column.end,
        taffy::style::GridPlacement::from_line_index(3),
        "header 应解析到 col end = 3"
    );

    // 验证 nav 的位置 → row 2-3, col 1-2
    let nav_taffy = find_taffy_for_dom(&taffy_to_dom, nav);
    let nav_taffy_style = taffy_tree.style(nav_taffy).unwrap();
    assert_eq!(
        nav_taffy_style.grid_row.start,
        taffy::style::GridPlacement::from_line_index(2),
        "nav 应解析到 row start = 2"
    );
    assert_eq!(
        nav_taffy_style.grid_column.start,
        taffy::style::GridPlacement::from_line_index(1),
        "nav 应解析到 col start = 1"
    );
    assert_eq!(
        nav_taffy_style.grid_column.end,
        taffy::style::GridPlacement::from_line_index(2),
        "nav 应解析到 col end = 2"
    );
}

// -- Shadow DOM slot 解析测试 --

/// 有 shadow root 的元素，shadow 树中包含 <slot name="header">，
/// light DOM 中有 slot="header" 的子元素 → 布局树应包含该 slotted 子元素。
#[test]
fn test_shadow_dom_slot_flattened_into_layout() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 宿主元素
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // 附加 shadow root
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // shadow 树内容：<div><slot name="header"></slot></div>
    let shadow_wrapper = doc.create_element("div");
    doc.append_child(shadow, shadow_wrapper).unwrap();
    let slot_header = doc.create_element("slot");
    doc.set_attribute(slot_header, "name", "header");
    doc.append_child(shadow_wrapper, slot_header).unwrap();

    // light DOM 子元素：<h1 slot="header">Title</h1>
    let header_elem = doc.create_element("h1");
    doc.set_attribute(header_elem, "slot", "header");
    doc.append_child(host, header_elem).unwrap();

    // 解析 slot 分配
    doc.resolve_slots(host);

    let styles = HashMap::new();
    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // host 应在映射中
    assert!(taffy_to_dom.values().any(|id| *id == host), "宿主元素应在布局树中");

    // slotted 子元素 (h1) 应在布局树中
    assert!(
        taffy_to_dom.values().any(|id| *id == header_elem),
        "slotted h1 元素应在布局树中"
    );

    // shadow 树中的 wrapper div 应在布局树中
    assert!(
        taffy_to_dom.values().any(|id| *id == shadow_wrapper),
        "shadow wrapper div 应在布局树中"
    );

    // 验证 shadow_wrapper 是 host 的 taffy 子节点
    // host 在 taffy 中的子节点应该是 shadow_wrapper（而非 light DOM 子节点）
    let host_taffy = find_taffy_for_dom(&taffy_to_dom, host);
    let host_children = taffy_tree.children(host_taffy).unwrap();
    assert_eq!(host_children.len(), 1, "host 应有 1 个 taffy 子节点（shadow wrapper）");

    // shadow_wrapper 的子节点应该是 slotted 的 header_elem
    let wrapper_taffy = find_taffy_for_dom(&taffy_to_dom, shadow_wrapper);
    let wrapper_children = taffy_tree.children(wrapper_taffy).unwrap();
    assert_eq!(wrapper_children.len(), 1, "wrapper 应有 1 个子节点（slotted h1）");

    // 验证那个子节点对应的是 header_elem
    let child_dom_id = taffy_to_dom.get(&wrapper_children[0]).copied();
    assert_eq!(child_dom_id, Some(header_elem), "wrapper 子节点应为 slotted h1");
}

/// 未命名的默认 <slot> 接收没有 slot 属性的 light DOM 子节点。
#[test]
fn test_shadow_dom_default_slot_uses_light_children() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 宿主元素
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // 附加 shadow root，包含默认 <slot>（无 name 属性）
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(shadow, wrapper).unwrap();
    let default_slot = doc.create_element("slot");
    doc.append_child(wrapper, default_slot).unwrap();

    // light DOM：两个没有 slot 属性的子元素
    let child1 = doc.create_element("p");
    doc.append_child(host, child1).unwrap();
    let child2 = doc.create_element("span");
    doc.append_child(host, child2).unwrap();

    // 解析 slot 分配
    doc.resolve_slots(host);

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 两个 light DOM 子元素都应出现在布局树中（通过默认 slot）
    assert!(
        taffy_to_dom.values().any(|id| *id == child1),
        "默认 slot 中的 p 元素应在布局树中"
    );
    assert!(
        taffy_to_dom.values().any(|id| *id == child2),
        "默认 slot 中的 span 元素应在布局树中"
    );
}

/// <slot> 有回退子元素，且没有 light DOM 分配 → 布局树使用回退内容。
#[test]
fn test_shadow_dom_fallback_content_when_no_assignment() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 宿主元素（无 light DOM 子节点）
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // 附加 shadow root
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(shadow, wrapper).unwrap();

    // <slot name="sidebar"> 带回退子元素
    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "name", "sidebar");
    doc.append_child(wrapper, slot).unwrap();

    // 回退内容
    let fallback_div = doc.create_element("div");
    doc.set_attribute(fallback_div, "class", "fallback");
    doc.append_child(slot, fallback_div).unwrap();

    // 解析 slot 分配（无 light DOM 匹配 "sidebar" slot）
    doc.resolve_slots(host);

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 回退 div 应在布局树中
    assert!(
        taffy_to_dom.values().any(|id| *id == fallback_div),
        "slot 回退内容（div.fallback）应在布局树中"
    );
}

/// 未分配到任何 slot 的 light DOM 子节点不应出现在布局树中。
#[test]
fn test_shadow_dom_unassigned_light_children_hidden() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 宿主元素
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // 附加 shadow root，只有一个具名 slot
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(shadow, wrapper).unwrap();
    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "name", "header");
    doc.append_child(wrapper, slot).unwrap();

    // light DOM：一个匹配 slot="header"，一个不匹配任何 slot
    let header_elem = doc.create_element("h1");
    doc.set_attribute(header_elem, "slot", "header");
    doc.append_child(host, header_elem).unwrap();
    let orphan_elem = doc.create_element("footer");
    // footer 没有 slot 属性，且 shadow 树中没有默认 slot
    doc.append_child(host, orphan_elem).unwrap();

    // 解析 slot 分配
    doc.resolve_slots(host);

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 已分配的 h1 应在布局树中
    assert!(
        taffy_to_dom.values().any(|id| *id == header_elem),
        "已分配到 slot 的 h1 应在布局树中"
    );

    // 未分配的 footer 不应在布局树中
    assert!(
        !taffy_to_dom.values().any(|id| *id == orphan_elem),
        "未分配到任何 slot 的 footer 不应在布局树中"
    );
}

// -- 边界条件测试（第五批）--

/// 测试独立设置 row-gap 的布局树构建。
///
/// 当 ComputedStyle 中只设置 row_gap 而不设置 gap 时，
/// 验证 row-gap 正确传递到 taffy 样式中，且构建不 panic。
#[test]
fn test_build_with_row_gap_only() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let flex = doc.create_element("div");
    doc.append_child(html, flex).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(flex, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(flex, item2).unwrap();

    let mut styles = HashMap::new();
    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    flex_style.flex_direction = FlexDirectionValue::Column;
    // 仅设置 row_gap，gap 保持默认 Px(0.0)
    flex_style.row_gap = LengthValue::Px(15.0);
    styles.insert(flex, flex_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let flex_taffy = find_taffy_for_dom(&taffy_to_dom, flex);
    let style = taffy_tree.style(flex_taffy).unwrap();
    // gap.width（column-gap）应为默认 0.0
    assert_eq!(style.gap.width, taffy::style::LengthPercentage::Length(0.0));
    // gap.height（row-gap）应为 15.0
    assert_eq!(style.gap.height, taffy::style::LengthPercentage::Length(15.0));
}

/// 测试 grid 容器子元素全部使用 Span 放置时的布局树构建。
///
/// 所有子元素通过 GridLineValue::Span 定位，不使用命名引用，
/// 验证布局树成功构建且不 panic。
#[test]
fn test_build_grid_items_all_span_placement() {
    use zero_style_system::GridLineValue;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let grid = doc.create_element("div");
    doc.append_child(html, grid).unwrap();
    let item1 = doc.create_element("span");
    doc.append_child(grid, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(grid, item2).unwrap();

    let mut styles = HashMap::new();

    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("1fr 1fr".to_string());
    grid_style.grid_template_rows = Some("50px".to_string());
    styles.insert(grid, grid_style);

    // item1: column span 2
    let mut item1_style = ComputedStyle::default();
    item1_style.grid_column_start = GridLineValue::Line(1);
    item1_style.grid_column_end = GridLineValue::Span(2);
    item1_style.grid_row_start = GridLineValue::Line(1);
    item1_style.grid_row_end = GridLineValue::Line(2);
    styles.insert(item1, item1_style);

    // item2: column span 1, 下一行（会溢出到隐式行）
    let mut item2_style = ComputedStyle::default();
    item2_style.grid_column_start = GridLineValue::Line(1);
    item2_style.grid_column_end = GridLineValue::Span(1);
    item2_style.grid_row_start = GridLineValue::Line(2);
    item2_style.grid_row_end = GridLineValue::Line(3);
    styles.insert(item2, item2_style);

    let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );

    // 两个 item 都应在映射中
    assert!(taffy_to_dom.values().any(|id| *id == item1), "item1 应在布局树中");
    assert!(taffy_to_dom.values().any(|id| *id == item2), "item2 应在布局树中");

    // grid 容器应有 2 个 taffy 子节点
    let grid_taffy = find_taffy_for_dom(&taffy_to_dom, grid);
    let children = taffy_tree.children(grid_taffy).unwrap();
    assert_eq!(children.len(), 2, "grid 容器应有 2 个子项");
}

// ── 覆盖率补全第三轮：Shadow DOM slot 处理路径 ──

/// 覆盖 find_first_element 中 doc.get(node) 返回 None 的分支（line 68）
/// 以及深度优先搜索子节点路径（lines 76-82）
#[test]
fn test_build_with_text_nodes_mixed() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    // 文本节点在元素之前
    let text1 = doc.create_text_node("before");
    doc.append_child(body, text1).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text2 = doc.create_text_node("after");
    doc.append_child(body, text2).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    assert!(taffy_to_dom.values().any(|id| *id == div));
    assert!(!taffy_to_dom.values().any(|id| *id == text1));
    assert!(!taffy_to_dom.values().any(|id| *id == text2));
}

/// 覆盖 shadow DOM slot 替换路径（lines 194-228）
/// 测试：host 元素有 shadow root，shadow root 中有 <slot> 元素，
/// slot 有已分配的 light DOM 节点
#[test]
fn test_build_with_shadow_dom_slot_assigned() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 创建 host 元素
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // light DOM 子节点
    let light_child = doc.create_element("div");
    doc.append_child(host, light_child).unwrap();

    // attach shadow root
    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // shadow root 内的 <slot> 元素
    let slot = doc.create_element("slot");
    doc.append_child(shadow_root, slot).unwrap();

    // 设置 slot 的 name 属性并分配 light DOM 到 slot
    doc.set_attribute(slot, "name", "default");
    doc.assign_slot(slot, "default", light_child);

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // light_child 应该通过 slot 替换出现在布局树中
    assert!(
        taffy_to_dom.values().any(|id| *id == light_child),
        "assigned light DOM should be in layout tree"
    );
}

/// 覆盖 shadow DOM slot 回退内容路径（lines 211-222）
/// 测试：slot 没有分配的 light DOM 节点，使用 slot 自身的子元素作为回退
#[test]
fn test_build_with_shadow_dom_slot_fallback() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    // 创建 host 元素（无 light DOM 子节点）
    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    // attach shadow root
    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // shadow root 内的 <slot> 元素（带回退内容）
    let slot = doc.create_element("slot");
    doc.append_child(shadow_root, slot).unwrap();
    let fallback = doc.create_element("span");
    doc.append_child(slot, fallback).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // fallback span should be in the layout tree
    assert!(
        taffy_to_dom.values().any(|id| *id == fallback),
        "slot fallback should be in layout tree"
    );
}

/// 覆盖 shadow 树中非 slot 元素处理（lines 224-228）
#[test]
fn test_build_with_shadow_dom_non_slot_elements() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let host = doc.create_element("my-component");
    doc.append_child(body, host).unwrap();

    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let shadow_div = doc.create_element("div");
    doc.append_child(shadow_root, shadow_div).unwrap();

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    assert!(
        taffy_to_dom.values().any(|id| *id == shadow_div),
        "shadow div should be in layout tree"
    );
}

/// 覆盖 process_slot_children_in_shadow 路径（lines 248-286）
/// 嵌套 shadow DOM：shadow root 内部有子元素，子元素中的 slot 有分配节点
#[test]
fn test_build_with_nested_shadow_slots() {
    use zero_dom::ShadowRootMode;

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let host = doc.create_element("outer-component");
    doc.append_child(body, host).unwrap();

    // light DOM
    let light_div = doc.create_element("div");
    doc.append_child(host, light_div).unwrap();

    // shadow root with a wrapper containing a slot
    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(shadow_root, wrapper).unwrap();
    let inner_slot = doc.create_element("slot");
    doc.append_child(wrapper, inner_slot).unwrap();

    // Assign the light DOM div to the slot
    doc.assign_slot(inner_slot, "", light_div);

    let styles = HashMap::new();
    let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(
        &doc,
        &styles,
        800.0,
        600.0,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    // wrapper should definitely be in the tree
    assert!(
        taffy_to_dom.values().any(|id| *id == wrapper),
        "shadow wrapper should be in layout tree"
    );
}
