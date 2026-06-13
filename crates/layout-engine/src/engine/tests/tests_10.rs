use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue, VisibilityValue};

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

/// 测试 inline-only 容器收缩后，后续普通流兄弟应同步上移。
#[test]
fn test_inline_only_container_shrink_reflows_following_sibling() {
    let (mut doc, body) = make_doc_with_body();
    let first = doc.create_element("div");
    doc.append_child(body, first).unwrap();
    let img1 = doc.create_element("img");
    let img2 = doc.create_element("img");
    {
        let elem = doc.get_mut(img1).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "96");
        }
    }
    {
        let elem = doc.get_mut(img2).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "144");
        }
    }
    doc.append_child(first, img1).unwrap();
    doc.append_child(first, img2).unwrap();

    let second = doc.create_element("div");
    doc.append_child(body, second).unwrap();
    let img3 = doc.create_element("img");
    {
        let elem = doc.get_mut(img3).unwrap();
        if let zero_dom::NodeKind::Element(e) = &mut elem.kind {
            e.set_attribute("width", "96");
            e.set_attribute("height", "96");
        }
    }
    doc.append_child(second, img3).unwrap();

    let mut styles = HashMap::new();
    let mut first_style = ComputedStyle::default();
    first_style.display = DisplayValue::Block;
    styles.insert(first, first_style);

    let mut second_style = ComputedStyle::default();
    second_style.display = DisplayValue::Block;
    styles.insert(second, second_style);

    let mut img_style = ComputedStyle::default();
    img_style.display = DisplayValue::Inline;
    styles.insert(img1, img_style.clone());
    styles.insert(img2, img_style.clone());
    styles.insert(img3, img_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first_box = find_child_by_node_id(&result.root, first).expect("first found");
    let second_box = find_child_by_node_id(&result.root, second).expect("second found");

    assert!(
        (first_box.height - 144.0).abs() < 1.0,
        "first inline-only container should shrink to tallest image height, got {}",
        first_box.height
    );
    assert!(
        (second_box.y - (first_box.y + first_box.height)).abs() < 1.0,
        "following sibling should be reflowed after shrink: second.y={}, first.bottom={}",
        second_box.y,
        first_box.y + first_box.height
    );
}

/// 测试 clear-float-003：空普通块的自折叠 margin 不应错误抬高后续 clear:right 浮动。
#[test]
fn test_clear_float_003_negative_margin_clear_float_can_overlap_prior_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let first_float = doc.create_element("div");
    doc.append_child(container, first_float).unwrap();
    let spacer = doc.create_element("div");
    doc.append_child(container, spacer).unwrap();
    let cleared_float = doc.create_element("div");
    doc.append_child(container, cleared_float).unwrap();

    let mut styles = HashMap::new();

    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(192.0);
    container_style.height = LengthValue::Px(192.0);
    styles.insert(container, container_style);

    let mut first_float_style = ComputedStyle::default();
    first_float_style.display = DisplayValue::Block;
    first_float_style.float = FloatValue::Right;
    first_float_style.width = LengthValue::Px(96.0);
    first_float_style.height = LengthValue::Px(96.0);
    styles.insert(first_float, first_float_style);

    let mut spacer_style = ComputedStyle::default();
    spacer_style.display = DisplayValue::Block;
    spacer_style.height = LengthValue::Px(0.0);
    spacer_style.margin_top = LengthValue::Px(96.0);
    spacer_style.margin_bottom = LengthValue::Px(96.0);
    styles.insert(spacer, spacer_style);

    let mut cleared_float_style = ComputedStyle::default();
    cleared_float_style.display = DisplayValue::Block;
    cleared_float_style.float = FloatValue::Right;
    cleared_float_style.clear = ClearValue::Right;
    cleared_float_style.width = LengthValue::Px(96.0);
    cleared_float_style.height = LengthValue::Px(96.0);
    cleared_float_style.margin_top = LengthValue::Px(-96.0);
    styles.insert(cleared_float, cleared_float_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first_box = find_child_by_node_id(&result.root, first_float).expect("first float found");
    let cleared_box = find_child_by_node_id(&result.root, cleared_float).expect("cleared float found");

    assert!(
        (first_box.y - 0.0).abs() < 0.5,
        "first float should stay at top, got y={}",
        first_box.y
    );
    assert!(
        (cleared_box.y - 0.0).abs() < 0.5,
        "cleared float should keep the same top as the prior float, got y={}",
        cleared_box.y
    );
}

/// 测试 BFC 浮动排斥：overflow:hidden 的块级元素不得与左浮动重叠。
/// CSS 2.1 §9.5: BFC 元素的 border box 不得与同一格式化上下文中的浮动元素重叠。
#[test]
fn test_bfc_float_avoidance_left() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();
    let bfc_elem = doc.create_element("div");
    doc.append_child(body, bfc_elem).unwrap();

    let mut styles = HashMap::new();

    // 左浮动：50x50
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(50.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Left;
    styles.insert(float_elem, fl);

    // overflow:hidden（建立 BFC）：100x100
    let mut bfc = ComputedStyle::default();
    bfc.display = DisplayValue::Block;
    bfc.width = LengthValue::Px(100.0);
    bfc.height = LengthValue::Px(100.0);
    bfc.overflow_x = zero_css_parser::values::OverflowValue::Hidden;
    bfc.overflow_y = zero_css_parser::values::OverflowValue::Hidden;
    styles.insert(bfc_elem, bfc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let bfc_box = find_child_by_node_id(&result.root, bfc_elem).expect("bfc found");

    // BFC 元素的左边缘应在浮动元素的右边缘之后或与之间距
    assert!(
        bfc_box.x >= fl_box.x + fl_box.width - 0.5,
        "BFC 元素不得与左浮动重叠: bfc.x={}, float_right={}",
        bfc_box.x,
        fl_box.x + fl_box.width
    );
}

/// 测试 BFC 浮动排斥：overflow:hidden 的块级元素不得与右浮动重叠。
#[test]
fn test_bfc_float_avoidance_right() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();
    let bfc_elem = doc.create_element("div");
    doc.append_child(body, bfc_elem).unwrap();

    let mut styles = HashMap::new();

    // 右浮动：50x50
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.width = LengthValue::Px(50.0);
    fl.height = LengthValue::Px(50.0);
    fl.float = FloatValue::Right;
    styles.insert(float_elem, fl);

    // overflow:hidden（建立 BFC）：200x100
    let mut bfc = ComputedStyle::default();
    bfc.display = DisplayValue::Block;
    bfc.width = LengthValue::Px(200.0);
    bfc.height = LengthValue::Px(100.0);
    bfc.overflow_x = zero_css_parser::values::OverflowValue::Hidden;
    bfc.overflow_y = zero_css_parser::values::OverflowValue::Hidden;
    styles.insert(bfc_elem, bfc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let bfc_box = find_child_by_node_id(&result.root, bfc_elem).expect("bfc found");

    // BFC 元素的右边缘不应超过右浮动的左边缘
    assert!(
        bfc_box.x + bfc_box.width <= fl_box.x + 0.5,
        "BFC 元素不得与右浮动重叠: bfc_right={}, float_left={}",
        bfc_box.x + bfc_box.width,
        fl_box.x
    );
}

/// 测试孤立 table-row-group 作为块级兄弟时仍应布局其匿名行内的 table-cell。
///
/// 该场景对应 clear-applies-to-001：`display: table-row-group` 不应触发 clear，
/// 但其内部匿名行仍必须参与 table 布局，否则单元格会停留在 taffy 的错误位置。
#[test]
fn test_orphan_table_row_group_positions_anonymous_cells() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();

    let row_group = doc.create_element("div");
    doc.append_child(body, row_group).unwrap();

    let cell_a = doc.create_element("div");
    doc.append_child(row_group, cell_a).unwrap();
    let cell_b = doc.create_element("div");
    doc.append_child(row_group, cell_b).unwrap();

    let mut styles = HashMap::new();

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(320.0);
    fl.height = LengthValue::Px(96.0);
    styles.insert(float_elem, fl);

    let mut rg = ComputedStyle::default();
    rg.display = DisplayValue::TableRowGroup;
    rg.clear = ClearValue::Both;
    rg.background_color = zero_css_parser::values::ColorValue::Named("blue".to_string());
    styles.insert(row_group, rg);

    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    cell.width = LengthValue::Px(48.0);
    cell.height = LengthValue::Px(48.0);
    cell.background_color = zero_css_parser::values::ColorValue::Named("blue".to_string());
    styles.insert(cell_a, cell.clone());
    styles.insert(cell_b, cell);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let row_group_box = find_child_by_node_id(&result.root, row_group).expect("row group found");
    let cell_a_box = find_child_by_node_id(&result.root, cell_a).expect("cell_a found");
    let cell_b_box = find_child_by_node_id(&result.root, cell_b).expect("cell_b found");

    assert!(
        row_group_box.x < float_box.x + float_box.width - 0.5,
        "table-row-group clear should not apply: rg.x={}, float_right={}",
        row_group_box.x,
        float_box.x + float_box.width
    );
    assert!(
        (cell_a_box.x - row_group_box.x).abs() < 1.0,
        "first anonymous cell should start at row group left edge: cell_a.x={}, rg.x={}",
        cell_a_box.x,
        row_group_box.x
    );
    assert!(
        cell_b_box.x >= cell_a_box.x + cell_a_box.width - 0.5,
        "second anonymous cell should be positioned after the first: cell_b.x={}, cell_a.right={}",
        cell_b_box.x,
        cell_a_box.x + cell_a_box.width
    );
    assert!(
        (cell_a_box.y - row_group_box.y).abs() < 1.0 && (cell_b_box.y - row_group_box.y).abs() < 1.0,
        "anonymous row cells should align to the row group top: cell_a.y={}, cell_b.y={}, rg.y={}",
        cell_a_box.y,
        cell_b_box.y,
        row_group_box.y
    );
}

/// 测试嵌套 block 上的 clear:both 仍需清除祖先容器中更早的浮动。
///
/// 对应 clear-applies-to-009：float 是 body 的直接子元素，clear:block 在后续 div 内部。
#[test]
fn test_nested_block_clear_sees_ancestor_floats() {
    let (mut doc, body) = make_doc_with_body();
    let float_elem = doc.create_element("div");
    doc.append_child(body, float_elem).unwrap();

    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();

    let clear_block = doc.create_element("div");
    doc.append_child(wrapper, clear_block).unwrap();

    let mut styles = HashMap::new();

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(320.0);
    fl.height = LengthValue::Px(96.0);
    styles.insert(float_elem, fl);

    let mut wr = ComputedStyle::default();
    wr.display = DisplayValue::Block;
    styles.insert(wrapper, wr);

    let mut cb = ComputedStyle::default();
    cb.display = DisplayValue::Block;
    cb.clear = ClearValue::Both;
    cb.width = LengthValue::Px(96.0);
    cb.height = LengthValue::Px(96.0);
    styles.insert(clear_block, cb);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_box = find_child_by_node_id(&result.root, float_elem).expect("float found");
    let clear_box = find_child_by_node_id(&result.root, clear_block).expect("clear block found");
    let (_, float_abs_y) = find_absolute_position_by_node_id(&result.root, float_elem).expect("float abs");
    let (_, clear_abs_y) = find_absolute_position_by_node_id(&result.root, clear_block).expect("clear abs");

    assert!(
        clear_abs_y >= float_abs_y + float_box.height - 0.5,
        "nested clear block should be placed below earlier float: clear_abs_y={}, float_bottom_abs={}",
        clear_abs_y,
        float_abs_y + float_box.height
    );
    assert!(clear_box.height > 0.0, "clear block should still have its own box");
}

#[test]
fn test_border_collapse_table_wins() {
    let html = r#"<html><body style="margin:0"><table style="border: 5px solid green; border-collapse: collapse"><tr><td style="border: 4.95px solid red; width: 50px; height: 50px"></td></tr></table></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // Find the table cell by looking for small boxes with borders.
    // 必须跳过 table 本身（也可能匹配 width < 100 条件），
    // 只返回最内层（叶子级别）的 cell 盒。
    fn find_cell(box_node: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
        for child in &box_node.children {
            // 先递归搜索子元素，优先返回更深的 cell
            if let Some(c) = find_cell(child) {
                return Some(c);
            }
            // 如果子元素中没有 cell，检查当前 child 是否匹配
            if child.border_top > 0.0 && child.width < 100.0 && child.width > 10.0 {
                return Some(child);
            }
        }
        None
    }

    if let Some(cell) = find_cell(&result.root) {
        // Table border (5px) should win over cell border (4.95px)
        // After resolve_collapsed_borders, cell border_top should be ~5.0
        assert!(
            cell.border_top >= 4.9,
            "cell border_top should be ~5.0 (table wins), got {}",
            cell.border_top
        );
        // Color override should be set for top edge (green from table)
        let top_color_override = cell.collapsed_border_color_overrides[0];
        assert!(
            top_color_override.is_some(),
            "top color override should be set (table's green), got None"
        );
        // Green = Rgba(0, 128, 0, 255) = 0x008000FF
        if let Some(c) = top_color_override {
            assert_eq!(
                c, 0x008000FF,
                "top color override should be green (0x008000FF), got {:#010X}",
                c
            );
        }
    }
}

/// 测试 table column 的 visibility:collapse。
///
/// 对应 visibility-collapse-colspan-003：中间列被 `visibility:collapse` 折叠，
/// 其宽度应为 0，且最后一行的 colspan 单元格应只占非折叠列宽度并裁剪溢出内容。
/// 非折叠列的显式 width 不应被 colspan 单元格的长内容撑开。
#[test]
fn test_table_column_visibility_collapse() {
    use zero_css_parser::values::LengthValue;
    let (mut doc, body) = make_doc_with_body();

    // <table>
    let table = doc.create_element("table");
    doc.append_child(body, table).unwrap();

    // <col> x3, 中间一个 visibility:collapse
    let col0 = doc.create_element("col");
    doc.append_child(table, col0).unwrap();
    let col1 = doc.create_element("col");
    doc.append_child(table, col1).unwrap();
    let col2 = doc.create_element("col");
    doc.append_child(table, col2).unwrap();

    // <tr> with 3 <td>, firstCol=65, thirdCol=160
    let row = doc.create_element("tr");
    doc.append_child(table, row).unwrap();
    let td0 = doc.create_element("td");
    doc.append_child(row, td0).unwrap();
    let td1 = doc.create_element("td");
    doc.append_child(row, td1).unwrap();
    let td2 = doc.create_element("td");
    doc.append_child(row, td2).unwrap();

    // <tr> with colspan=3 cell
    let row2 = doc.create_element("tr");
    doc.append_child(table, row2).unwrap();
    let td_span = doc.create_element("td");
    doc.set_attribute(td_span, "colspan", "3");
    doc.append_child(row2, td_span).unwrap();

    let mut styles = HashMap::new();

    // col1 visibility:collapse
    let mut c1 = ComputedStyle::default();
    c1.display = DisplayValue::TableColumn;
    c1.visibility = VisibilityValue::Collapse;
    styles.insert(col1, c1);

    let mut c0 = ComputedStyle::default();
    c0.display = DisplayValue::TableColumn;
    styles.insert(col0, c0);

    let mut c2 = ComputedStyle::default();
    c2.display = DisplayValue::TableColumn;
    styles.insert(col2, c2);

    // table display
    let mut tbl = ComputedStyle::default();
    tbl.display = DisplayValue::Table;
    styles.insert(table, tbl);

    let mut tr = ComputedStyle::default();
    tr.display = DisplayValue::TableRow;
    styles.insert(row, tr.clone());
    styles.insert(row2, tr);

    let mut cell = ComputedStyle::default();
    cell.display = DisplayValue::TableCell;
    styles.insert(td1, cell.clone());
    styles.insert(td_span, cell.clone());

    // firstCol width:65px
    let mut first = ComputedStyle::default();
    first.display = DisplayValue::TableCell;
    first.width = LengthValue::Px(65.0);
    styles.insert(td0, first.clone());

    // thirdCol width:160px
    let mut third = ComputedStyle::default();
    third.display = DisplayValue::TableCell;
    third.width = LengthValue::Px(160.0);
    styles.insert(td2, third);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let td0_box = find_child_by_node_id(&result.root, td0).expect("td0 found");
    let td2_box = find_child_by_node_id(&result.root, td2).expect("td2 found");
    let td1_box = find_child_by_node_id(&result.root, td1).expect("td1 found");
    let td_span_box = find_child_by_node_id(&result.root, td_span).expect("colspan cell found");

    // 折叠列单元格宽度应为 0
    assert!(
        td1_box.width.abs() < 0.5,
        "collapsed column cell width should be 0, got {}",
        td1_box.width
    );

    // 非折叠列应保持显式宽度，不被 colspan 长内容撑开
    assert!(
        (td0_box.width - 65.0).abs() < 1.5,
        "firstCol width should be ~65 (explicit, not inflated by colspan), got {}",
        td0_box.width
    );
    assert!(
        (td2_box.width - 160.0).abs() < 1.5,
        "thirdCol width should be ~160 (explicit, not inflated by colspan), got {}",
        td2_box.width
    );

    // colspan 单元格宽度 = 65 + 0 + 160 = 225（仅非折叠列）
    assert!(
        (td_span_box.width - 225.0).abs() < 3.0,
        "colspan-3 cell spanning collapsed col should be ~225 (sum of non-collapsed cols), got {}",
        td_span_box.width
    );

    // colspan 单元格应设置 overflow_x:Hidden 以裁剪溢出内容
    assert_eq!(
        td_span_box.overflow_x,
        crate::types::OverflowClip::Hidden,
        "colspan cell spanning collapsed column must clip overflow"
    );
}

/// 测试：无 border/padding 的容器中，第一个流内子元素的 margin-top 与父容器折叠后，
/// 后续 float 的定位不应把该 margin-top 双重计入（CSS §8.3.1 margin 与父折叠）。
/// 复现 inline-formatting-context-002/003：`<p>`(mt=16) 后跟 float，float 应位于
/// p.border_bottom + p.margin_bottom，而非额外加上 p.margin_top。
#[test]
fn test_float_after_first_child_margin_collapses_with_parent() {
    let (mut doc, body) = make_doc_with_body();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();
    let float_div = doc.create_element("div");
    doc.append_child(body, float_div).unwrap();

    let mut styles = HashMap::new();
    // body：无 border/padding（默认），使首个子元素 margin-top 与之折叠
    let mut body_style = ComputedStyle::default();
    body_style.display = DisplayValue::Block;
    body_style.margin_top = LengthValue::Px(16.0);
    body_style.margin_bottom = LengthValue::Px(8.0);
    styles.insert(body, body_style);

    // 第一个流内子元素 <p>：margin-top=16（与 body 折叠），height=19，margin-bottom=16
    let mut ps = ComputedStyle::default();
    ps.display = DisplayValue::Block;
    ps.margin_top = LengthValue::Px(16.0);
    ps.height = LengthValue::Px(19.0);
    ps.margin_bottom = LengthValue::Px(16.0);
    styles.insert(p, ps);

    // float：高度 19.2
    let mut fs = ComputedStyle::default();
    fs.display = DisplayValue::Block;
    fs.float = FloatValue::Left;
    fs.height = LengthValue::Px(19.2);
    styles.insert(float_div, fs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let p_box = find_child_by_node_id(&result.root, p).expect("p found");
    let f_box = find_child_by_node_id(&result.root, float_div).expect("float found");

    // float 与 p 底边的间距应等于 p.margin_bottom（16），而非 16+16（双重计入 p.margin_top）。
    let gap = f_box.y - (p_box.y + p_box.height);
    assert!(
        (gap - 16.0).abs() < 1.0,
        "float 应位于 p.border_bottom + p.margin_bottom（间距≈16），实际 gap={}（p.y={} h={} float.y={}）",
        gap,
        p_box.y,
        p_box.height,
        f_box.y
    );
}
