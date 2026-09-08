//! R4149：块盒 min/max-width 的 content 关键字（min/max-content/fit-content）=
//! content-based 尺寸（css-sizing-3 §5.2 + csswg #3973）。converter 把 min_width
//! 关键字映射 length(0)、max_width 关键字映射 auto（taffy 无 content-keyword 概念），
//! `apply_intrinsic_content_sizing` pass 在此测 intrinsic 后经 taffy 重跑钳制：
//! min 侧 floor（dynamic-011：`min-width:min-content; width:0px` 内子传宽 100 应撑到
//! 100），max 侧 cap（block-aspect-ratio 域 border-box-and-max-content-002：AR +
//! content-box 盒的 `max-width:max-content` 应按 border-box cap）。
//! driving: WPT css-sizing/intrinsic-percent-replaced-dynamic-011、
//! css-sizing/block-size-with-min-or-max-content-6、css-sizing/border-box-and-max-content-002。

use super::*;

/// `min-width:min-content; width:0px` 的 abspos 盒含 100px 宽子 → 宽须撑到 100
///（converter 把 min_width 关键字塌成 0，无此钳制时盒宽 0）。
#[test]
fn r4149_min_width_min_content_floors_to_content() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.position = zero_css_parser::values::PositionValue::Absolute;
    parent_style.width = LengthValue::Px(0.0);
    parent_style.min_width = LengthValue::MinContent;

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(100.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let p = find(&result.root, parent).expect("parent box");
    assert!(
        (p.width - 100.0).abs() < 1.0,
        "min-width:min-content must floor box width to content width 100, got {}",
        p.width
    );
}

/// 无 AR 的普通块 `max-width:min-content; width:200px` 含 100px 定宽子：R4151 gate
/// 放宽后 max 关键字 cap 臂触发——max-width:min-content 按 spec 语义钳到子 min-content
/// 贡献 100（dynamic-012 同语义；R4151 wrapper 的 min-content 近似 = 显式定宽子宽）。
#[test]
fn r4151_max_width_min_content_caps_plain_block() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.width = LengthValue::Px(200.0);
    parent_style.max_width = LengthValue::MinContent;

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let p = find(&result.root, parent).expect("parent box");
    assert!(
        (p.width - 100.0).abs() < 1.0,
        "plain block with max-width:min-content caps to child min-content contribution (100), got {}",
        p.width
    );
}

/// `max-width:max-content` 的 AR 盒不塌：cap = max-content 自身（非 min-content）——
/// border-box-and-max-content-002 的 .item（无子、AR 1/1 + height:500 + border-box
/// padding）经 AR-aware 测量（own_ar transferred 500）cap no-op，宽保持 500。
#[test]
fn r4151_max_width_max_content_ar_box_not_collapsed() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let item = doc.create_element("div");
    doc.append_child(parent, item).unwrap();

    let mut parent_style = ComputedStyle::default();
    parent_style.display = zero_style_system::DisplayValue::Block;
    parent_style.width = LengthValue::Px(600.0);

    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;
    item_style.max_width = LengthValue::MaxContent;
    item_style.height = LengthValue::Px(500.0);
    item_style.aspect_ratio = Some(1.0);
    item_style.padding_left = LengthValue::Px(20.0);
    item_style.padding_right = LengthValue::Px(20.0);
    item_style.box_sizing = zero_css_parser::values::BoxSizingValue::BorderBox;

    let mut styles = HashMap::new();
    styles.insert(parent, parent_style);
    styles.insert(item, item_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let it = find(&result.root, item).expect("item box");
    assert!(
        (it.height - 500.0).abs() < 1.0 && (it.width - 500.0).abs() < 1.0,
        "AR box with max-width:max-content must keep transferred 500x500, got {}x{}",
        it.width,
        it.height
    );
}

/// R4149 二轮（约束机制）：flex item 的 `min-width:min-content` + `flex-basis:0`——
/// 关键字写入 taffy min_size 约束（非定宽），flex 主轴求解按 min 钳制撑到内容 100
///（flex-item-min-width-min-content：vertical-rl item 亦同，width 是物理水平轴属性）。
#[test]
fn r4149_flex_item_min_width_keyword_clamps_main_axis() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();
    let child = doc.create_element("div");
    doc.append_child(item, child).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(0.0);
    container_style.height = LengthValue::Px(100.0);

    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;
    item_style.min_width = LengthValue::MinContent;

    let mut child_style = ComputedStyle::default();
    child_style.display = zero_style_system::DisplayValue::Block;
    child_style.width = LengthValue::Px(100.0);
    child_style.height = LengthValue::Px(50.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);
    styles.insert(child, child_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let it = find(&result.root, item).expect("item box");
    assert!(
        (it.width - 100.0).abs() < 1.0,
        "flex item min-width:min-content must clamp main axis to content 100 (flex-basis:0 + min floor), got {}",
        it.width
    );
}

/// R4150（CSS Flexbox §4）：flex item 建立独立格式化上下文——auto 高度包含 float 子。
/// 容器 100 宽下两个 float 100×50 竖排（第二个放不下换行），item 高应 100
///（旧按 §10.5.1 排除 float 收缩到 0）。
#[test]
fn r4150_flex_item_auto_height_contains_floats() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();
    let f1 = doc.create_element("div");
    doc.append_child(item, f1).unwrap();
    let f2 = doc.create_element("div");
    doc.append_child(item, f2).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);
    container_style.flex_direction = zero_css_parser::values::FlexDirectionValue::Column;

    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;

    let mut float_style = ComputedStyle::default();
    float_style.display = zero_style_system::DisplayValue::Block;
    float_style.width = LengthValue::Px(100.0);
    float_style.height = LengthValue::Px(50.0);
    float_style.float = zero_css_parser::values::FloatValue::Left;

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);
    styles.insert(f1, float_style.clone());
    styles.insert(f2, float_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let it = find(&result.root, item).expect("item box");
    assert!(
        (it.height - 100.0).abs() < 1.0,
        "flex item auto height must contain float children (100), got {}",
        it.height
    );
}

/// R4152（css-sizing-4 §4.1）：flex 容器 definite width + AR + Auto height——内容
/// 高于 transferred 时容器高 = 内容（item h:100 → 容器 100，非 transferred 50）。
#[test]
fn r4152_flex_container_ar_height_yields_to_content() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(100.0);
    container_style.aspect_ratio = Some(2.0);

    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;
    item_style.height = LengthValue::Px(100.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let c = find(&result.root, container).expect("container box");
    assert!(
        (c.height - 100.0).abs() < 1.0,
        "flex container AR transferred height (50) must yield to content height (100), got {}",
        c.height
    );
}

/// R4152b（CSS2 §10.4）：max-height 上限参与——flex 容器内容 200 被 max-block-size
/// 100 钳（flex-aspect-ratio-043：容器高 = min(max(transferred,content), max) = 100）。
#[test]
fn r4152b_flex_container_ar_content_clamped_by_max_height() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.display = zero_style_system::DisplayValue::Flex;
    container_style.width = LengthValue::Px(100.0);
    container_style.aspect_ratio = Some(2.0);
    container_style.max_height = LengthValue::Px(100.0);

    let mut item_style = ComputedStyle::default();
    item_style.display = zero_style_system::DisplayValue::Block;
    item_style.height = LengthValue::Px(200.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(item, item_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let c = find(&result.root, container).expect("container box");
    assert!(
        (c.height - 100.0).abs() < 1.0,
        "flex container AR content height (200) must clamp to max-height (100), got {}",
        c.height
    );
}
