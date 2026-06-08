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
