use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue};

/// 测试 clear:both 将元素推到所有浮动元素下方。
#[test]
fn test_clear_both_after_floats() {
    let (mut doc, body) = make_doc_with_body();
    let float_left = doc.create_element("div");
    doc.append_child(body, float_left).unwrap();
    let float_right = doc.create_element("div");
    doc.append_child(body, float_right).unwrap();
    let clear_elem = doc.create_element("div");
    doc.append_child(body, clear_elem).unwrap();

    let mut styles = HashMap::new();

    // 左浮动：100x50
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Left;
    styles.insert(float_left, fl);

    // 右浮动：100x80
    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.width = LengthValue::Px(100.0);
    fr.height = LengthValue::Px(80.0);
    fr.float = FloatValue::Right;
    styles.insert(float_right, fr);

    // clear: both 元素
    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.width = LengthValue::Px(200.0);
    ce.height = LengthValue::Px(30.0);
    ce.clear = ClearValue::Both;
    styles.insert(clear_elem, ce);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_left).expect("float_left found");
    let fr_box = find_child_by_node_id(&result.root, float_right).expect("float_right found");
    let ce_box = find_child_by_node_id(&result.root, clear_elem).expect("clear_elem found");

    // clear 元素应在最高的浮动元素下方
    // 右浮动 80px > 左浮动 50px
    let max_float_bottom = (fr_box.y + fr_box.height).max(fl_box.y + fl_box.height);
    assert!(
        ce_box.y >= max_float_bottom - 0.5,
        "clear:both 元素应在所有浮动下方: ce.y={}, max_float_bottom={}",
        ce_box.y,
        max_float_bottom
    );
}

/// 测试 clear:left 只清除左浮动。
#[test]
fn test_clear_left_only() {
    let (mut doc, body) = make_doc_with_body();
    let float_left = doc.create_element("div");
    doc.append_child(body, float_left).unwrap();
    let float_right = doc.create_element("div");
    doc.append_child(body, float_right).unwrap();
    let clear_elem = doc.create_element("div");
    doc.append_child(body, clear_elem).unwrap();

    let mut styles = HashMap::new();

    // 左浮动：100x60
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(60.0);
    fl.float = FloatValue::Left;
    styles.insert(float_left, fl);

    // 右浮动：100x100（更高）
    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.width = LengthValue::Px(100.0);
    fr.height = LengthValue::Px(100.0);
    fr.float = FloatValue::Right;
    styles.insert(float_right, fr);

    // clear: left — 只应在左浮动下方
    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.width = LengthValue::Px(200.0);
    ce.height = LengthValue::Px(30.0);
    ce.clear = ClearValue::Left;
    styles.insert(clear_elem, ce);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_left).expect("float_left found");
    let ce_box = find_child_by_node_id(&result.root, clear_elem).expect("clear_elem found");

    // clear:left 元素应在左浮动下方
    assert!(
        ce_box.y >= fl_box.y + fl_box.height - 0.5,
        "clear:left 元素应在左浮动下方: ce.y={}, fl_bottom={}",
        ce_box.y,
        fl_box.y + fl_box.height
    );
}

/// 测试 clear:right 只清除右浮动。
#[test]
fn test_clear_right_only() {
    let (mut doc, body) = make_doc_with_body();
    let float_left = doc.create_element("div");
    doc.append_child(body, float_left).unwrap();
    let float_right = doc.create_element("div");
    doc.append_child(body, float_right).unwrap();
    let clear_elem = doc.create_element("div");
    doc.append_child(body, clear_elem).unwrap();

    let mut styles = HashMap::new();

    // 左浮动：100x120
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(120.0);
    fl.float = FloatValue::Left;
    styles.insert(float_left, fl);

    // 右浮动：100x40
    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.width = LengthValue::Px(100.0);
    fr.height = LengthValue::Px(40.0);
    fr.float = FloatValue::Right;
    styles.insert(float_right, fr);

    // clear: right
    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.width = LengthValue::Px(200.0);
    ce.height = LengthValue::Px(30.0);
    ce.clear = ClearValue::Right;
    styles.insert(clear_elem, ce);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fr_box = find_child_by_node_id(&result.root, float_right).expect("float_right found");
    let ce_box = find_child_by_node_id(&result.root, clear_elem).expect("clear_elem found");

    // clear:right 元素应在右浮动下方
    assert!(
        ce_box.y >= fr_box.y + fr_box.height - 0.5,
        "clear:right 元素应在右浮动下方: ce.y={}, fr_bottom={}",
        ce_box.y,
        fr_box.y + fr_box.height
    );
}

/// 测试 clear:none 不影响布局。
#[test]
fn test_clear_none_no_effect() {
    let (mut doc, body) = make_doc_with_body();
    let float_left = doc.create_element("div");
    doc.append_child(body, float_left).unwrap();
    let block_elem = doc.create_element("div");
    doc.append_child(body, block_elem).unwrap();

    let mut styles = HashMap::new();

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Left;
    styles.insert(float_left, fl);

    // clear: none（默认） — 不应清除浮动
    let mut be = ComputedStyle::default();
    be.display = DisplayValue::Block;
    be.width = LengthValue::Px(200.0);
    be.height = LengthValue::Px(30.0);
    be.clear = ClearValue::None;
    styles.insert(block_elem, be);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_left).expect("float_left found");
    let be_box = find_child_by_node_id(&result.root, block_elem).expect("block_elem found");

    // clear:none 元素不应被推到浮动下方
    // 它应正常流布局（taffy 按正常块布局放置）
    assert!(
        be_box.y < fl_box.y + fl_box.height + 50.0,
        "clear:none 元素不应被大幅推到浮动下方: be.y={}, fl.y+fl.h={}",
        be_box.y,
        fl_box.y + fl_box.height
    );
}

/// 测试 clearance-006 场景：零 clearance / 正 clearance 的 margin 折叠行为
/// 结构：container > before(mb=40) + float(h=40) + clear(mt=40, h=20)
/// container: border-top=20, content-height=80, border-bottom=20
/// 正确行为：float 在 Y=40, clear 在 Y=80（正 clearance）,
///   clear 溢出到 border-bottom 区域，绿色覆盖红色
#[test]
fn test_clearance_with_margin_collapse() {
    let (mut doc, body) = make_doc_with_body();

    // container
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    // .before (empty, margin-bottom=40)
    let before = doc.create_element("div");
    doc.append_child(container, before).unwrap();

    // .float (height=40, no margin)
    let float_elem = doc.create_element("div");
    doc.append_child(container, float_elem).unwrap();

    // .clear (clear:both, margin-top=40, height=20)
    let clear_elem = doc.create_element("div");
    doc.append_child(container, clear_elem).unwrap();

    let mut styles = HashMap::new();

    // container: width=100, border-top=20, height=80, border-bottom=20
    let mut ct = ComputedStyle::default();
    ct.display = DisplayValue::Block;
    ct.width = LengthValue::Px(100.0);
    ct.height = LengthValue::Px(80.0);
    ct.border_top_width = LengthValue::Px(20.0);
    ct.border_bottom_width = LengthValue::Px(20.0);
    styles.insert(container, ct);

    // .before: height=0, margin-bottom=40
    let mut be = ComputedStyle::default();
    be.display = DisplayValue::Block;
    be.width = LengthValue::Px(100.0);
    be.height = LengthValue::Px(0.0);
    be.margin_bottom = LengthValue::Px(40.0);
    styles.insert(before, be);

    // .float: float:left, height=40, no margin
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(40.0);
    fl.float = FloatValue::Left;
    styles.insert(float_elem, fl);

    // .clear: clear:both, margin-top=40, height=20
    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.width = LengthValue::Px(100.0);
    ce.height = LengthValue::Px(20.0);
    ce.clear = ClearValue::Both;
    ce.margin_top = LengthValue::Px(40.0);
    styles.insert(clear_elem, ce);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ct_box = find_child_by_node_id(&result.root, container).expect("container found");
    let _ = find_child_by_node_id(&result.root, before).expect("before found");
    let fl_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let ce_box = find_child_by_node_id(&result.root, clear_elem).expect("clear found");

    // Expected (content-relative, i.e. child.y - container.content_y):
    // float at content-Y = 40 (after .before mb=40 collapses with float mt=0)
    // float bottom = 80
    // clear at content-Y = 80 (clearance pushes it below float bottom)
    // clear bottom = 100 (overflows content area 80)
    let content_y = ct_box.content_y;
    let float_content_y = fl_box.y - content_y;
    let clear_content_y = ce_box.y - content_y;

    assert!(
        (float_content_y - 40.0).abs() < 1.0,
        "float should be at content-Y=40, got {}",
        float_content_y
    );
    assert!(
        (clear_content_y - 80.0).abs() < 1.0,
        "clear should be at content-Y=80 (clearance pushes below float), got {}",
        clear_content_y
    );
}

/// 测试空 cleared block 只应折叠自身上下外边距，不应把后继兄弟多推一段。
#[test]
fn test_empty_cleared_block_collapses_with_next_sibling() {
    let (mut doc, body) = make_doc_with_body();

    let before = doc.create_element("div");
    doc.append_child(body, before).unwrap();

    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();

    let clear_elem = doc.create_element("div");
    doc.append_child(body, clear_elem).unwrap();

    let after = doc.create_element("div");
    doc.append_child(body, after).unwrap();

    let mut styles = HashMap::new();

    let mut be = ComputedStyle::default();
    be.display = DisplayValue::Block;
    be.width = LengthValue::Px(100.0);
    be.height = LengthValue::Px(20.0);
    be.margin_bottom = LengthValue::Px(20.0);
    styles.insert(before, be);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(40.0);
    styles.insert(float_elem, fl);

    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.clear = ClearValue::Both;
    ce.width = LengthValue::Px(100.0);
    ce.height = LengthValue::Px(0.0);
    ce.margin_top = LengthValue::Px(80.0);
    ce.margin_bottom = LengthValue::Px(100.0);
    styles.insert(clear_elem, ce);

    let mut af = ComputedStyle::default();
    af.display = DisplayValue::Block;
    af.width = LengthValue::Px(100.0);
    af.height = LengthValue::Px(20.0);
    styles.insert(after, af);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let clear_box = find_child_by_node_id(&result.root, clear_elem).expect("clear found");
    let after_box = find_child_by_node_id(&result.root, after).expect("after found");

    assert!(
        (clear_box.y - 100.0).abs() < 1.0,
        "empty cleared block should still be placed at the cleared position, got {}",
        clear_box.y
    );
    assert!(
        (after_box.y - 120.0).abs() < 1.0,
        "next sibling should see collapsed empty-block margins, got {}",
        after_box.y
    );
}

/// 测试 BFC 检测：overflow:hidden 建立新的格式化上下文。
#[test]
fn test_bfc_detection_overflow_hidden() {
    use crate::margin_collapse::establishes_bfc;
    use crate::types::OverflowClip;

    let mut bx = LayoutBox::default();
    assert!(!establishes_bfc(&bx), "default should not establish BFC");

    bx.overflow_x = OverflowClip::Hidden;
    assert!(establishes_bfc(&bx), "overflow:hidden should establish BFC");

    bx.overflow_x = OverflowClip::Visible;
    bx.overflow_y = OverflowClip::Scroll;
    assert!(establishes_bfc(&bx), "overflow:scroll should establish BFC");
}

/// 测试 BFC 检测：float 和 position 建立格式化上下文。
#[test]
fn test_bfc_detection_float_and_position() {
    use crate::margin_collapse::establishes_bfc;

    let mut bx = LayoutBox::default();
    bx.float = FloatValue::Left;
    assert!(establishes_bfc(&bx), "float:left should establish BFC");

    bx.float = FloatValue::None;
    bx.is_absolute = true;
    assert!(establishes_bfc(&bx), "position:absolute should establish BFC");

    bx.is_absolute = false;
    bx.is_fixed = true;
    assert!(establishes_bfc(&bx), "position:fixed should establish BFC");
}

/// 测试 BFC 检测：display:flex/grid/table（is_layout_container）建立格式化上下文。
/// CSS：这些容器建立独立格式化上下文（隔离 margin 折叠 + 包含浮动）。
#[test]
fn test_bfc_detection_layout_container() {
    use crate::margin_collapse::establishes_bfc;

    let mut bx = LayoutBox::default();
    assert!(!establishes_bfc(&bx), "普通 block 不建立 BFC");

    bx.is_flow_root = true;
    assert!(establishes_bfc(&bx), "display:flow-root 建立 BFC");

    bx.is_flow_root = false;
    bx.is_layout_container = true;
    assert!(
        establishes_bfc(&bx),
        "display:flex/grid/table（is_layout_container）建立 BFC"
    );

    bx.is_layout_container = false;
    bx.is_multicol = true;
    assert!(establishes_bfc(&bx), "多列容器建立 BFC");

    bx.is_multicol = false;
    bx.is_anon_table_root = true;
    assert!(establishes_bfc(&bx), "孤立 table-internal（匿名 table 根）建立 BFC");
}

/// 测试 mark_anonymous_table_roots：孤立 table-internal 标记为匿名 table 根（建立 BFC）。
#[test]
fn test_mark_anonymous_table_roots_orphan() {
    use crate::engine::mark_anonymous_table_roots;
    use crate::margin_collapse::establishes_bfc;
    use std::collections::HashMap;
    use zero_css_parser::values::DisplayValue;
    use zero_style_system::ComputedStyle;

    let (_doc, orphan_id) = make_doc_with_body();
    let (_doc2, table_id) = make_doc_with_body();
    let (_doc3, nested_id) = make_doc_with_body();

    // 孤立 table-row-group（父为 block，非 table 上下文）
    let orphan = LayoutBox {
        node_id: Some(orphan_id),
        ..Default::default()
    };
    // 嵌套 table-row-group（在 table 内部）
    let nested = LayoutBox {
        node_id: Some(nested_id),
        ..Default::default()
    };
    let table = LayoutBox {
        node_id: Some(table_id),
        children: vec![nested],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![orphan, table],
        ..Default::default()
    };

    let mut s1 = ComputedStyle::default();
    s1.display = DisplayValue::TableRowGroup;
    let mut s2 = ComputedStyle::default();
    s2.display = DisplayValue::Table;
    let mut s3 = ComputedStyle::default();
    s3.display = DisplayValue::TableRowGroup;
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(orphan_id, s1);
    styles.insert(table_id, s2);
    styles.insert(nested_id, s3);

    mark_anonymous_table_roots(&mut root, &styles, false);

    assert!(
        root.children[0].is_anon_table_root,
        "孤立 table-row-group 应标记为匿名 table 根"
    );
    assert!(establishes_bfc(&root.children[0]), "孤立 table-row-group 应建立 BFC");
    assert!(
        !root.children[1].children[0].is_anon_table_root,
        "table 内部的 table-row-group 不应标记为匿名 table 根"
    );
}

/// 测试多个浮动元素 + clear:both 的复杂布局。
#[test]
fn test_multiple_floats_with_clear() {
    let (mut doc, body) = make_doc_with_body();

    // 3 个左浮动，1 个 clear:both，1 个普通块
    let mut float_ids = Vec::new();
    for _ in 0..3 {
        let elem = doc.create_element("div");
        doc.append_child(body, elem).unwrap();
        float_ids.push(elem);
    }
    let clear_elem = doc.create_element("div");
    doc.append_child(body, clear_elem).unwrap();
    let block_elem = doc.create_element("div");
    doc.append_child(body, block_elem).unwrap();

    let mut styles = HashMap::new();

    // 3 个左浮动，每个 80x30
    for &id in &float_ids {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.width = LengthValue::Px(80.0);
        s.height = LengthValue::Px(30.0);
        s.float = FloatValue::Left;
        styles.insert(id, s);
    }

    // clear:both
    let mut ce = ComputedStyle::default();
    ce.display = DisplayValue::Block;
    ce.width = LengthValue::Px(300.0);
    ce.height = LengthValue::Px(20.0);
    ce.clear = ClearValue::Both;
    styles.insert(clear_elem, ce);

    // 普通块
    let mut be = ComputedStyle::default();
    be.display = DisplayValue::Block;
    be.width = LengthValue::Px(300.0);
    be.height = LengthValue::Px(20.0);
    styles.insert(block_elem, be);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let f0 = find_child_by_node_id(&result.root, float_ids[0]).expect("float0");
    let f1 = find_child_by_node_id(&result.root, float_ids[1]).expect("float1");
    let f2 = find_child_by_node_id(&result.root, float_ids[2]).expect("float2");
    let ce = find_child_by_node_id(&result.root, clear_elem).expect("clear");
    let be = find_child_by_node_id(&result.root, block_elem).expect("block");

    // 左浮动元素垂直堆叠（当前实现行为）
    // 后续可扩展为水平排列 + 自动换行
    assert!(f1.y >= f0.y, "f1 应在 f0 下方或同行");
    assert!(f2.y >= f1.y, "f2 应在 f1 下方或同行");

    // clear:both 元素应在浮动元素下方
    assert!(
        ce.y >= f0.y + f0.height - 0.5,
        "clear 元素应在浮动下方: ce.y={}, float_bottom={}",
        ce.y,
        f0.y + f0.height
    );

    // 普通块在 clear 元素之后
    assert!(be.y >= ce.y, "block 应在 clear 元素之后: be.y={}, ce.y={}", be.y, ce.y);
}

/// 测试 <img> 元素的固有尺寸：有 width/height HTML 属性时应正确布局。
#[test]
fn test_img_intrinsic_sizing_with_attributes() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    // 设置 HTML 属性
    {
        let elem = doc.get_mut(img).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "200");
            e.set_attribute("height", "100");
        }
    }
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    // 不设置 CSS width/height — 使用 HTML 属性的固有尺寸
    let mut img_style = ComputedStyle::default();
    img_style.display = DisplayValue::Block;
    styles.insert(img, img_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");

    // img 应使用 HTML 属性的固有尺寸 200x100
    assert!(
        (img_box.width - 200.0).abs() < 1.0,
        "img width should be ~200 (from HTML attribute), got {}",
        img_box.width
    );
    assert!(
        (img_box.height - 100.0).abs() < 1.0,
        "img height should be ~100 (from HTML attribute), got {}",
        img_box.height
    );
}

/// R1683：`<embed>`/`<object>`/`<applet>` 同为替换元素，HTML width/height 属性应给出固有
/// 尺寸（viewport）。此前 `apply_replaced_element_sizing` 仅处理 img/canvas → embed 渲成
/// 784×0、object/applet 按 fallback 内容宽（legacy-html fixture 43 抓到）。本测试钉死三标签
/// 带 width/height 属性时用属性固有尺寸。
#[test]
fn test_embed_object_applet_intrinsic_sizing_from_attributes() {
    for tag in ["embed", "object", "applet"] {
        let (mut doc, body) = make_doc_with_body();
        let el = doc.create_element(tag);
        {
            let elem = doc.get_mut(el).unwrap();
            if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
                e.set_attribute("width", "120");
                e.set_attribute("height", "60");
            }
        }
        doc.append_child(body, el).unwrap();

        let mut styles = HashMap::new();
        let mut el_style = ComputedStyle::default();
        el_style.display = DisplayValue::InlineBlock;
        styles.insert(el, el_style);

        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let el_box = find_child_by_node_id(&result.root, el).expect("element found");
        assert!(
            (el_box.width - 120.0).abs() < 1.0,
            "<{tag}> width should be ~120 (from HTML attribute), got {}",
            el_box.width
        );
        assert!(
            (el_box.height - 60.0).abs() < 1.0,
            "<{tag}> height should be ~60 (from HTML attribute), got {}",
            el_box.height
        );
    }
}

/// R1684：`<details>` 无 `open` 属性（闭合态）时，仅 `<summary>` 子渲染，其余子按 UA
/// `details:not([open]) > *:not(summary) { display: none }` 隐藏。ZW 无 UA CSS 父条件
/// 选择器，故 layout-tree 构建期过滤。本测试钉死闭合态 details 隐藏非 summary 内容、
/// 开启态 details 显示全部内容。
#[test]
fn test_closed_details_hides_non_summary_children() {
    let (mut doc, body) = make_doc_with_body();

    // 闭合 details（无 open）：summary + p
    let closed = doc.create_element("details");
    let closed_summary = doc.create_element("summary");
    let closed_p = doc.create_element("p");
    doc.append_child(closed, closed_summary).unwrap();
    doc.append_child(closed, closed_p).unwrap();
    doc.append_child(body, closed).unwrap();

    // 开启 details（有 open）：summary + p
    let open_details = doc.create_element("details");
    {
        let elem = doc.get_mut(open_details).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("open", "");
        }
    }
    let open_summary = doc.create_element("summary");
    let open_p = doc.create_element("p");
    doc.append_child(open_details, open_summary).unwrap();
    doc.append_child(open_details, open_p).unwrap();
    doc.append_child(body, open_details).unwrap();

    let mut styles = HashMap::new();
    for &el in &[closed, closed_summary, closed_p, open_details, open_summary, open_p] {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        styles.insert(el, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 闭合 details：summary 有 box，p（隐藏内容）无 box（不在 layout 树）。
    assert!(
        find_child_by_node_id(&result.root, closed_summary).is_some(),
        "closed <details> summary must render"
    );
    assert!(
        find_child_by_node_id(&result.root, closed_p).is_none(),
        "closed <details> non-summary content must be hidden (no layout box)"
    );

    // 开启 details：summary + p 都有 box。
    assert!(
        find_child_by_node_id(&result.root, open_summary).is_some(),
        "open <details> summary must render"
    );
    assert!(
        find_child_by_node_id(&result.root, open_p).is_some(),
        "open <details> content must render"
    );
}

/// R784：`<canvas>` 是替换元素，HTML width/height 属性给出固有尺寸，与 `<img>` 一致——
/// CSS 单侧显式时另一侧按固有宽高比推导。旧实现 canvas 未被 apply_replaced_element_sizing
/// 处理（仅 img）→ 当普通 block 拉伸填满父宽；且 HTML-attr 分支 auto 侧用 HTML 绝对值
/// 而非按比例推导（aspect-ratio-intrinsic-size 簇 canvas 渲染错误）。
#[test]
fn test_canvas_one_css_side_explicit_derives_other_via_aspect() {
    let (mut doc, body) = make_doc_with_body();
    let canvas = doc.create_element("canvas");
    {
        let elem = doc.get_mut(canvas).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "10");
            e.set_attribute("height", "20"); // 固有比 10:20 = 1:2
        }
    }
    doc.append_child(body, canvas).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.height = LengthValue::Px(100.0); // CSS height 显式，width auto
    styles.insert(canvas, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cb = find_child_by_node_id(&result.root, canvas).expect("canvas found");
    // height 显式 100，width 按 10:20 固有比推导 = 100 * 10/20 = 50（旧实现会拉满父宽或用 HTML 10）
    assert!(
        (cb.height - 100.0).abs() < 1.0,
        "canvas height should be 100 (CSS explicit), got {}",
        cb.height
    );
    assert!(
        (cb.width - 50.0).abs() < 1.5,
        "canvas width should be aspect-derived ~50 (intrinsic 10:20 @ height 100), got {}",
        cb.width
    );
}

/// 测试 <img> 元素有 CSS 尺寸时覆盖 HTML 属性。
#[test]
fn test_img_css_overrides_html_attributes() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    {
        let elem = doc.get_mut(img).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "200");
            e.set_attribute("height", "100");
        }
    }
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    // CSS 尺寸应覆盖 HTML 属性
    let mut img_style = ComputedStyle::default();
    img_style.display = DisplayValue::Block;
    img_style.width = LengthValue::Px(300.0);
    img_style.height = LengthValue::Px(150.0);
    styles.insert(img, img_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");

    // CSS 尺寸应覆盖 HTML 属性
    assert!(
        (img_box.width - 300.0).abs() < 1.0,
        "img width should be ~300 (from CSS), got {}",
        img_box.width
    );
    assert!(
        (img_box.height - 150.0).abs() < 1.0,
        "img height should be ~150 (from CSS), got {}",
        img_box.height
    );
}

/// R976：CSS `aspect-ratio` 优先于替换元素固有宽高比（css-sizing-4 §4）。无 HTML width/height
/// 属性的 `<img>`（回退到解码固有尺寸）在一侧 CSS 显式、另一侧 auto 时，auto 侧须按 **CSS
/// aspect-ratio**（若设）推导，而非固有 w/h。旧实现恒用固有比，致 `<img style="block-size:55vw;
/// aspect-ratio:2/1">`（固有 8×16）width 算成 440×(8/16)=220 而非 440×2=880
/// （nested-grid-item-block-size-001 oracle 64.36%→13.76%）。
#[test]
fn test_img_css_aspect_ratio_overrides_intrinsic_ratio() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.height = LengthValue::Px(440.0); // 显式 height（如 55vw @800 viewport）
    s.aspect_ratio = Some(2.0); // CSS aspect-ratio: 2/1 (width/height)
    styles.insert(img, s);

    // 注入解码固有尺寸 8×16（固有比 0.5，区别于 CSS aspect-ratio 2.0）
    let mut intrinsic = HashMap::new();
    intrinsic.insert(img, (8.0_f32, 16.0_f32));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, intrinsic, std::collections::HashMap::new());

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    // height 显式 440；width auto 须按 CSS aspect-ratio 2.0 推导 = 440×2 = 880
    // （旧实现按固有 8/16 推导 = 220，是本次修复的 bug）。
    assert!(
        (img_box.height - 440.0).abs() < 2.0,
        "img height should be ~440 (CSS explicit), got {}",
        img_box.height
    );
    assert!(
        (img_box.width - 880.0).abs() < 2.0,
        "img width should be ~880 (CSS aspect-ratio 2/1 @ height 440), got {}; \
         bug = used intrinsic 8/16 ratio → 220",
        img_box.width
    );
}

/// R976 对称分支：CSS width 显式 + aspect-ratio + height auto，height 须按 CSS aspect-ratio
/// 推导（旧实现按固有比）。
#[test]
fn test_img_css_aspect_ratio_overrides_intrinsic_ratio_width_explicit() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.width = LengthValue::Px(200.0);
    s.aspect_ratio = Some(2.0); // width/height = 2/1
    styles.insert(img, s);

    let mut intrinsic = HashMap::new();
    intrinsic.insert(img, (8.0_f32, 16.0_f32)); // 固有比 0.5

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, intrinsic, std::collections::HashMap::new());

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    // width 显式 200；height auto 须按 CSS aspect-ratio 2.0 推导 = 200/2 = 100
    // （旧实现按固有 16/8 推导 = 200×(16/8)=400，是 bug）。
    assert!(
        (img_box.width - 200.0).abs() < 2.0,
        "img width should be ~200 (CSS explicit), got {}",
        img_box.width
    );
    assert!(
        (img_box.height - 100.0).abs() < 2.0,
        "img height should be ~100 (CSS aspect-ratio 2/1 @ width 200), got {}; \
         bug = used intrinsic 16/8 ratio → 400",
        img_box.height
    );
}

/// R1437 no-ratio 替换元素尺寸（CSS §10.3.2）：width-only no-ratio SVG（`width="50"` 无
/// height/viewBox）+ CSS `height:20px width:auto` → width 须用真实固有宽 50（非按 usvg
/// 默认 h=100 算出的 50/100=0.5 比例推导为 20×0.5=10）。驱动案：visudet
/// replaced-elements-height-20（width-50-no-ratio.svg）。
#[test]
fn test_img_no_ratio_width_only_height_explicit() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.height = LengthValue::Px(20.0); // height 显式，width auto
    styles.insert(img, s);

    // no-ratio 信号：真实固有宽 50，无固有高（usvg 默认 h=100 不真实）。
    // 同时入 sizes（pixmap 50×100，供背景图），但 no-ratio 优先消费。
    let mut sizes = HashMap::new();
    sizes.insert(img, (50.0_f32, 100.0_f32));
    let mut no_ratio = HashMap::new();
    no_ratio.insert(img, (Some(50.0_f32), None));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_intrinsic(&doc, &styles, sizes, std::collections::HashMap::new(), no_ratio);

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    // height 显式 20；width auto 须用真实固有宽 50（旧实现按 bogus ratio 0.5 推导 = 10）。
    assert!(
        (img_box.height - 20.0).abs() < 2.0,
        "img height should be ~20 (CSS explicit), got {}",
        img_box.height
    );
    assert!(
        (img_box.width - 50.0).abs() < 2.0,
        "img width should be ~50 (no-ratio intrinsic width), got {}; \
         bug = used bogus 50/100 ratio → 10",
        img_box.width
    );
}

/// R1437 no-ratio：height-only no-ratio SVG（`height="25"`）+ CSS `width:40px height:auto`
/// → height 须用真实固有高 25（驱动案：visudet replaced-elements-width-40 height-25-no-ratio）。
#[test]
fn test_img_no_ratio_height_only_width_explicit() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.width = LengthValue::Px(40.0); // width 显式，height auto
    styles.insert(img, s);

    let mut no_ratio = HashMap::new();
    no_ratio.insert(img, (None, Some(25.0_f32)));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_intrinsic(
        &doc,
        &styles,
        HashMap::new(),
        std::collections::HashMap::new(),
        no_ratio,
    );

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    assert!(
        (img_box.width - 40.0).abs() < 2.0,
        "width ~40 (CSS), got {}",
        img_box.width
    );
    assert!(
        (img_box.height - 25.0).abs() < 2.0,
        "img height should be ~25 (no-ratio intrinsic height), got {}",
        img_box.height
    );
}

/// R1437 no-ratio：零固有维 no-ratio SVG（无 width/height/viewBox）+ width/height 均 auto
/// → 须用 default object size 300×150（驱动案：visudet replaced-elements-all-auto no-ratio.svg）。
#[test]
fn test_img_no_ratio_no_dims_all_auto_default_object_size() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block; // width/height 均 auto
    styles.insert(img, s);

    let mut no_ratio = HashMap::new();
    no_ratio.insert(img, (None, None));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_intrinsic(
        &doc,
        &styles,
        HashMap::new(),
        std::collections::HashMap::new(),
        no_ratio,
    );

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    assert!(
        (img_box.width - 300.0).abs() < 2.0,
        "img width should be ~300 (default object size), got {}",
        img_box.width
    );
    assert!(
        (img_box.height - 150.0).abs() < 2.0,
        "img height should be ~150 (default object size), got {}",
        img_box.height
    );
}

/// R1468 ratio-only 替换元素默认对象尺寸（CSS §10.3.2）：仅有宽高比、无确定固有尺寸的 SVG
///（viewBox-only）+ width/height 均 auto 的**非 flex**上下文 → width 用默认对象宽 300，
/// height 由 aspect_ratio 推导（=300/ratio）。旧实现 ratio-only 不设 size（0 宽）。
/// 驱动案：visudet/normal-flow replaced-elements-{all-auto,min-width-40}（ratio-2.svg）。
/// flex 上下文须保留无确定 size（transferred-size ratio-derivation，R980/R991/R992），故仅
/// 非 flex 设默认对象宽——flex gate 由 apply_replaced_element_sizing 的 is_flex_*_item 守卫。
#[test]
fn test_img_ratio_only_all_auto_default_object_size() {
    let (mut doc, body) = make_doc_with_body();
    let img = doc.create_element("img");
    doc.append_child(body, img).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block; // 非 flex 上下文；width/height 均 auto
    styles.insert(img, s);

    // ratio-only 信号：仅有宽高比 2.0（viewBox-only SVG），无确定固有尺寸。
    let mut ratios = HashMap::new();
    ratios.insert(img, 2.0_f32);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_intrinsic(&doc, &styles, HashMap::new(), ratios, HashMap::new());

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    // width 须用默认对象宽 300；height 由 ratio 2.0 推导 = 300/2 = 150。
    assert!(
        (img_box.width - 300.0).abs() < 2.0,
        "img width should be ~300 (CSS §10.3.2 default object size), got {}; \
         bug = ratio-only SVG had no size → 0",
        img_box.width
    );
    assert!(
        (img_box.height - 150.0).abs() < 2.0,
        "img height should be ~150 (300 / ratio 2.0), got {}",
        img_box.height
    );
}
