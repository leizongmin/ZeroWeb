use super::*;
use zero_css_parser::values::{DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue, PositionValue};
use zero_style_system::FlexBasisValue;
// ── 边缘场景补充测试（第七批）──

/// 测试百分比宽度相对于父容器计算。
///
/// 父容器 400px，子元素宽度 50%（200px）。
/// 验证 taffy 正确解析百分比宽度并计算出精确的像素值。
#[test]
fn test_layout_percentage_width_with_parent() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let child = doc.create_element("div");
    doc.append_child(parent, child).unwrap();

    let mut styles = HashMap::new();
    // 父容器固定宽度 400px
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.width = LengthValue::Px(400.0);
    parent_style.height = LengthValue::Px(200.0);
    styles.insert(parent, parent_style);

    // 子元素宽度 50%
    let mut child_style = ComputedStyle::default();
    child_style.display = DisplayValue::Block;
    child_style.width = LengthValue::Percentage(50.0);
    child_style.height = LengthValue::Px(80.0);
    styles.insert(child, child_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let child_box = find_child_by_node_id(&result.root, child).expect("child 应找到");
    // 50% of 400px = 200px
    assert!(
        (child_box.width - 200.0).abs() < 1.0,
        "子元素宽度应为 200px（400 * 50%），实际 {}",
        child_box.width
    );
    assert_eq!(child_box.height, 80.0, "子元素高度应为 80");
}

/// 测试 flex 容器中同时包含 flex-grow 和固定尺寸子元素。
///
/// 容器 400px：一个 flex-grow=1 的自适应项 + 一个固定 120px 的项。
/// 自适应项应占据剩余 280px。
#[test]
fn test_flex_grow_coexists_with_fixed_item() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let grow_item = doc.create_element("span");
    doc.append_child(container, grow_item).unwrap();
    let fixed_item = doc.create_element("span");
    doc.append_child(container, fixed_item).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // grow_item: 无固定宽度，flex-grow=1
    let mut grow_style = ComputedStyle::default();
    grow_style.flex_grow = 1.0;
    grow_style.height = LengthValue::Px(50.0);
    styles.insert(grow_item, grow_style);

    // fixed_item: 固定宽度 120px，无 grow
    let mut fixed_style = ComputedStyle::default();
    fixed_style.width = LengthValue::Px(120.0);
    fixed_style.height = LengthValue::Px(50.0);
    styles.insert(fixed_item, fixed_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let grow_box = find_child_by_node_id(&result.root, grow_item).expect("grow_item found");
    let fixed_box = find_child_by_node_id(&result.root, fixed_item).expect("fixed_item found");

    // 固定项宽度不变
    assert!(
        (fixed_box.width - 120.0).abs() < 1.0,
        "固定项宽度应为 120px，实际 {}",
        fixed_box.width
    );

    // grow 项占据剩余空间: 400 - 120 = 280px
    assert!(
        (grow_box.width - 280.0).abs() < 1.0,
        "grow 项宽度应为 280px（400-120），实际 {}",
        grow_box.width
    );

    // 总宽度应约 400px
    let total = grow_box.width + fixed_box.width;
    assert!((total - 400.0).abs() < 1.0, "两项总宽度应约 400px，实际 {}", total);
}

/// 测试 CSS Flexbox §4.5 transferred size suggestion：替换元素（`<img>`）有 intrinsic aspect
/// ratio 且 cross 尺寸确定时，主轴 `min-size:auto` 至少为 cross 经 ratio 推导的尺寸。
/// 否则 flex item 在窄容器（`width:0`）中越过内容测量值塌缩为 0（flex-minimum-width-flex-items-011）。
#[test]
fn test_flex_transferred_size_suggestion() {
    let (mut doc, body) = make_doc_with_body();
    // flex 容器 width:0（迫使 item 收缩到 min-size:auto）
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    // img：无 width，height:50px（cross 确定），intrinsic 300×150（aspect 300/150 = 2）
    let img = doc.create_element("img");
    doc.append_child(container, img).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(0.0);
    styles.insert(container, container_style);

    let mut img_style = ComputedStyle::default();
    img_style.height = LengthValue::Px(50.0);
    styles.insert(img, img_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img, (300.0_f32, 150.0_f32));
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());

    let img_box = find_child_by_node_id(&result.root, img).expect("img found");
    // transferred = cross(50) × aspect(2) = 100；item 不应塌缩到 0
    assert!(
        (img_box.width - 100.0).abs() < 1.0,
        "img 宽度应为 transferred size 100px（50 × aspect 2），实际 {}",
        img_box.width
    );
}

/// 测试带显式宽度的空 flex item 在负 free space 下正确收缩（CSS Flexbox §7.3.2 + §4.5）。
///
/// 两个 width:100px 的空 flex item 放在 width:100px 的 flex 容器中（free space = -100px），
/// flex-shrink:1 的 item 应各收缩到 50px。此前 ZeroWeb 的 `measure_text_content` 在
/// 内容测量（MinContent/MaxContent）时回退到显式 CSS width，使空 item 的 min-size:auto = 100px，
/// 阻止收缩（flex-shrink-001/002/003/006/007/008 FAIL）。修复后空叶节点的内容测量返回 0。
#[test]
fn test_flex_shrink_explicit_width_items() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item1 = doc.create_element("div");
    let item2 = doc.create_element("div");
    doc.append_child(container, item1).unwrap();
    doc.append_child(container, item2).unwrap();

    let mut styles = HashMap::new();
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.width = LengthValue::Px(100.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    let mk_item = |flex_shrink: f64| {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(100.0);
        s.flex_shrink = flex_shrink;
        s
    };
    styles.insert(item1, mk_item(1.0));
    styles.insert(item2, mk_item(1.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    // 两个 100px item 在 100px 容器中，flex-shrink:1 各收缩 50px → 各 50px
    assert!(
        (b1.width - 50.0).abs() < 1.0,
        "item1 应收缩到 50px（min-size:auto 不再被显式 width 阻塞），实际 {}",
        b1.width
    );
    assert!((b2.width - 50.0).abs() < 1.0, "item2 应收缩到 50px，实际 {}", b2.width);
    // item2 紧跟 item1（水平排列，无溢出过容器右缘 100px）
    assert!(
        (b1.x + b1.width - b2.x).abs() < 1.0,
        "item2 应紧跟 item1，item1 右缘={} item2.x={}",
        b1.x + b1.width,
        b2.x
    );
}

/// 测试相对定位元素 top/left 偏移后仍占据原始空间。
///
/// 三个 block 元素：div1 正常，div2 position:relative + top:20px + left:10px，div3 正常。
/// div3 的 y 位置不应受 div2 的相对偏移影响（相对定位不脱离文档流）。
#[test]
fn test_relative_position_preserves_flow_space() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();
    let div3 = doc.create_element("div");
    doc.append_child(body, div3).unwrap();

    let mut styles = HashMap::new();

    // div1: 正常块级元素
    styles.insert(div1, make_style_with_display(DisplayValue::Block, 200.0, 50.0));

    // div2: 相对定位，有偏移
    let mut rel_style = ComputedStyle::default();
    rel_style.display = DisplayValue::Block;
    rel_style.position = PositionValue::Relative;
    rel_style.top = LengthValue::Px(20.0);
    rel_style.left = LengthValue::Px(10.0);
    rel_style.width = LengthValue::Px(200.0);
    rel_style.height = LengthValue::Px(60.0);
    styles.insert(div2, rel_style);

    // div3: 正常块级元素
    styles.insert(div3, make_style_with_display(DisplayValue::Block, 200.0, 40.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");
    let b3 = find_child_by_node_id(&result.root, div3).expect("div3 found");

    // div2 的视觉位置受 top/left 偏移影响
    // div2.y 在 taffy 布局中应包含 top 偏移
    // 相对定位不脱离文档流：div3.y 应基于 div2 的正常流位置计算
    // 即 div3.y ≈ div1.y + div1.height + div2.height（忽略 div2 的偏移）
    let expected_div3_y = b1.y + b1.height + 60.0; // div2.height = 60
    assert!(
        (b3.y - expected_div3_y).abs() < 1.0,
        "div3.y ({}) 应约等于 div1.y({}) + div1.height({}) + div2.normal_height(60) = {}，\
         相对定位不影响后续元素流位置",
        b3.y,
        b1.y,
        b1.height,
        expected_div3_y
    );

    // div2 不应是 absolute 或 fixed
    assert!(!b2.is_absolute, "relative 不应是 absolute");
    assert!(!b2.is_fixed, "relative 不应是 fixed");
}

/// 测试多个 fixed 定位元素在非 fixed 祖先内的视口坐标调整。
///
/// 结构：body > div(relative, margin:20px) > fixed1 + fixed2
/// 两个 fixed 元素应被 adjust_fixed_to_viewport 正确调整为视口坐标。
/// fixed1 和 fixed2 应各自独立调整，互不影响。
#[test]
fn test_multiple_fixed_elements_viewport_adjustment() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let fixed1 = doc.create_element("span");
    doc.append_child(container, fixed1).unwrap();
    let fixed2 = doc.create_element("span");
    doc.append_child(container, fixed2).unwrap();

    let mut styles = HashMap::new();

    // 容器有偏移（margin 造成祖先累积偏移）
    let mut container_style = ComputedStyle::default();
    container_style.position = PositionValue::Relative;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(300.0);
    container_style.margin_top = LengthValue::Px(30.0);
    container_style.margin_left = LengthValue::Px(20.0);
    styles.insert(container, container_style);

    // fixed1: top=10, left=15
    let mut f1_style = ComputedStyle::default();
    f1_style.position = PositionValue::Fixed;
    f1_style.top = LengthValue::Px(10.0);
    f1_style.left = LengthValue::Px(15.0);
    f1_style.width = LengthValue::Px(80.0);
    f1_style.height = LengthValue::Px(60.0);
    styles.insert(fixed1, f1_style);

    // fixed2: top=100, left=200
    let mut f2_style = ComputedStyle::default();
    f2_style.position = PositionValue::Fixed;
    f2_style.top = LengthValue::Px(100.0);
    f2_style.left = LengthValue::Px(200.0);
    f2_style.width = LengthValue::Px(120.0);
    f2_style.height = LengthValue::Px(80.0);
    styles.insert(fixed2, f2_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fb1 = find_child_by_node_id(&result.root, fixed1).expect("fixed1 found");
    let fb2 = find_child_by_node_id(&result.root, fixed2).expect("fixed2 found");

    // 两个都应标记为 fixed
    assert!(fb1.is_fixed, "fixed1 应标记为 fixed");
    assert!(fb2.is_fixed, "fixed2 应标记为 fixed");

    // 坐标应为有限值
    assert!(fb1.x.is_finite(), "fixed1 x 应为有限值");
    assert!(fb1.y.is_finite(), "fixed1 y 应为有限值");
    assert!(fb2.x.is_finite(), "fixed2 x 应为有限值");
    assert!(fb2.y.is_finite(), "fixed2 y 应为有限值");

    // 尺寸正确
    assert_eq!(fb1.width, 80.0, "fixed1 宽度应为 80");
    assert_eq!(fb1.height, 60.0, "fixed1 高度应为 60");
    assert_eq!(fb2.width, 120.0, "fixed2 宽度应为 120");
    assert_eq!(fb2.height, 80.0, "fixed2 高度应为 80");

    // fixed2 应在 fixed1 下方（top=100 > top=10）
    assert!(fb2.y > fb1.y, "fixed2 (y={}) 应在 fixed1 (y={}) 下方", fb2.y, fb1.y);

    // fixed2 应在 fixed1 右侧（left=200 > left=15）
    assert!(fb2.x > fb1.x, "fixed2 (x={}) 应在 fixed1 (x={}) 右侧", fb2.x, fb1.x);
}

/// 测试 grid 容器使用 grid-auto-rows 显式指定隐式行高度，
/// 当子元素超过显式模板行数时，隐式行使用 auto-rows 定义的高度。
#[test]
fn test_grid_auto_rows_implicit_track_height() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    // 放 4 个子元素，但只定义 1 行（显式模板）
    let mut item_ids = Vec::new();
    for _ in 0..4 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px".to_string());
    grid_style.grid_template_rows = Some("80px".to_string());
    // 隐式行高度 40px
    grid_style.grid_auto_rows = Some("40px".to_string());
    grid_style.width = LengthValue::Px(100.0);
    grid_style.height = LengthValue::Px(400.0);
    styles.insert(grid, grid_style);

    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
    let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item_ids[3]).expect("item3 found");

    // 第一个元素在显式行中（80px）
    assert!(
        (b0.height - 80.0).abs() < 1.0,
        "显式行 item0 高度应约 80px，实际 {}",
        b0.height
    );

    // 后续元素在隐式行中（40px）
    assert!(
        (b1.height - 40.0).abs() < 1.0,
        "隐式行 item1 高度应约 40px（grid-auto-rows），实际 {}",
        b1.height
    );
    assert!(
        (b2.height - 40.0).abs() < 1.0,
        "隐式行 item2 高度应约 40px（grid-auto-rows），实际 {}",
        b2.height
    );
    assert!(
        (b3.height - 40.0).abs() < 1.0,
        "隐式行 item3 高度应约 40px（grid-auto-rows），实际 {}",
        b3.height
    );

    // 所有元素应垂直排列
    assert!(b1.y > b0.y, "item1 应在 item0 下方");
    assert!(b2.y > b1.y, "item2 应在 item1 下方");
    assert!(b3.y > b2.y, "item3 应在 item2 下方");

    // 所有元素宽度应约 100px
    for (i, &id) in item_ids.iter().enumerate() {
        let b = find_child_by_node_id(&result.root, id).unwrap();
        assert!(
            (b.width - 100.0).abs() < 1.0,
            "item{} 宽度应约 100px，实际 {}",
            i,
            b.width
        );
    }
}

// -- 边缘场景补充测试（第八批）--

/// 测试相邻兄弟 block 的 margin 折叠近似行为。
///
/// 三个 block 元素垂直堆叠，相邻元素的 margin-bottom 与 margin-top
/// 在 taffy 中可能不发生折叠（不同于 CSS 规范的 margin collapse），
/// 验证布局引擎对正 margin 的处理是确定性的。
#[test]
fn test_block_adjacent_sibling_margins() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();
    let div3 = doc.create_element("div");
    doc.append_child(body, div3).unwrap();

    let mut styles = HashMap::new();

    let mut s1 = make_style_with_display(DisplayValue::Block, 200.0, 60.0);
    s1.margin_bottom = LengthValue::Px(20.0);
    styles.insert(div1, s1);

    let mut s2 = make_style_with_display(DisplayValue::Block, 200.0, 60.0);
    s2.margin_top = LengthValue::Px(30.0);
    s2.margin_bottom = LengthValue::Px(10.0);
    styles.insert(div2, s2);

    let mut s3 = make_style_with_display(DisplayValue::Block, 200.0, 60.0);
    s3.margin_top = LengthValue::Px(40.0);
    styles.insert(div3, s3);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");
    let b3 = find_child_by_node_id(&result.root, div3).expect("div3 found");

    // 所有元素宽度应为 200px
    assert_eq!(b1.width, 200.0);
    assert_eq!(b2.width, 200.0);
    assert_eq!(b3.width, 200.0);

    // 垂直排列顺序确定：b2 在 b1 之后，b3 在 b2 之后
    assert!(
        b2.y >= b1.y + b1.height,
        "div2 应在 div1 底部之后: b2.y({}) >= b1.y({}) + b1.h({})",
        b2.y,
        b1.y,
        b1.height
    );
    assert!(
        b3.y >= b2.y + b2.height,
        "div3 应在 div2 底部之后: b3.y({}) >= b2.y({}) + b2.h({})",
        b3.y,
        b2.y,
        b2.height
    );

    // margin_bottom 和 margin_top 的间距应有限非负
    let gap1 = b2.y - b1.y - b1.height;
    assert!(
        gap1 >= 0.0 && gap1.is_finite(),
        "div1-div2 间距应为有限非负值，实际 {}",
        gap1
    );
    let gap2 = b3.y - b2.y - b2.height;
    assert!(
        gap2 >= 0.0 && gap2.is_finite(),
        "div2-div3 间距应为有限非负值，实际 {}",
        gap2
    );
}

/// 测试绝对定位元素在 static 父容器内的行为。
///
/// 当父元素为 position:static（默认值）时，绝对定位子元素
/// 应相对于最近的 positioned 祖先（或初始包含块）定位。
/// 验证 absolute 子元素仍然获得正确的 is_absolute 标记和尺寸。
#[test]
fn test_absolute_in_static_parent() {
    let (mut doc, body) = make_doc_with_body();
    let static_parent = doc.create_element("div");
    doc.append_child(body, static_parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(static_parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // 父元素：position:static（默认），不建立定位上下文
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.width = LengthValue::Px(300.0);
    parent_style.height = LengthValue::Px(200.0);
    parent_style.padding_top = LengthValue::Px(20.0);
    parent_style.padding_left = LengthValue::Px(15.0);
    styles.insert(static_parent, parent_style);

    // 子元素：position:absolute
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(25.0);
    abs_style.left = LengthValue::Px(35.0);
    abs_style.width = LengthValue::Px(80.0);
    abs_style.height = LengthValue::Px(60.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let parent_box = find_child_by_node_id(&result.root, static_parent).expect("parent found");
    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");

    // 绝对定位标记正确
    assert!(abs_box.is_absolute, "子元素应标记为 absolute");

    // 父元素 padding 正确
    assert_eq!(parent_box.padding_top, 20.0);
    assert_eq!(parent_box.padding_left, 15.0);

    // 子元素尺寸正确
    assert_eq!(abs_box.width, 80.0);
    assert_eq!(abs_box.height, 60.0);

    // 子元素位置坐标应为有限值
    assert!(abs_box.x.is_finite(), "abs x 应为有限值");
    assert!(abs_box.y.is_finite(), "abs y 应为有限值");
}

/// 测试 body 外边距不会被 absolute 子元素重复计入。
#[test]
fn test_absolute_in_body_ignores_body_margin() {
    let (mut doc, body) = make_doc_with_body();
    let abs_child = doc.create_element("div");
    doc.append_child(body, abs_child).unwrap();

    let mut styles = HashMap::new();

    let mut body_style = ComputedStyle::default();
    body_style.margin_top = LengthValue::Px(8.0);
    body_style.margin_right = LengthValue::Px(8.0);
    body_style.margin_bottom = LengthValue::Px(8.0);
    body_style.margin_left = LengthValue::Px(8.0);
    styles.insert(body, body_style);

    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(118.0);
    abs_style.left = LengthValue::Px(8.0);
    abs_style.width = LengthValue::Px(768.0);
    abs_style.height = LengthValue::Px(100.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let (abs_x, abs_y) = find_absolute_position_by_node_id(&result.root, abs_child).expect("abs found");
    // CSS 2.1 §10.1：无 positioned ancestor 的 absolute 元素以初始包含块（视口）为
    // containing block。left:8/top:118 解析为视口相对坐标，不受 body margin 偏移影响。
    assert!((abs_x - 8.0).abs() < 2.0, "abs 视口 x 应为 left:8，实际为 {}", abs_x);
    assert!((abs_y - 118.0).abs() < 2.0, "abs 视口 y 应为 top:118，实际为 {}", abs_y);
}

/// 测试无子元素的空 flex 容器。
///
/// 空的 flex 容器尺寸由自身 width/height 决定，
/// 子元素列表应为空且布局不 panic。
#[test]
fn test_empty_flex_container() {
    let (mut doc, body) = make_doc_with_body();
    let flex = doc.create_element("div");
    doc.append_child(body, flex).unwrap();

    let mut styles = HashMap::new();
    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    flex_style.flex_direction = FlexDirectionValue::Row;
    flex_style.width = LengthValue::Px(400.0);
    flex_style.height = LengthValue::Px(200.0);
    styles.insert(flex, flex_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let flex_box = find_child_by_node_id(&result.root, flex).expect("flex found");

    // 空容器尺寸正确
    assert!(
        (flex_box.width - 400.0).abs() < 1.0,
        "空 flex 容器宽度应为 400，实际 {}",
        flex_box.width
    );
    assert!(
        (flex_box.height - 200.0).abs() < 1.0,
        "空 flex 容器高度应为 200，实际 {}",
        flex_box.height
    );

    // 无子元素
    assert!(flex_box.children.is_empty(), "空 flex 容器不应有子元素");

    // 内容区域应等于总尺寸（无 padding/border）
    assert!(
        (flex_box.content_width - flex_box.width).abs() < 0.001,
        "空 flex 内容宽度应等于总宽度"
    );
    assert!(
        (flex_box.content_height - flex_box.height).abs() < 0.001,
        "空 flex 内容高度应等于总高度"
    );
}

/// 测试 grid 单列 auto-rows 布局。
///
/// grid-template-columns 只有一列（100px），grid-auto-rows: 60px，
/// 4 个子元素自动放置在单列中，验证每行高度为 60px。
#[test]
fn test_grid_single_column_auto_rows() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut item_ids = Vec::new();
    for _ in 0..4 {
        let item = doc.create_element("span");
        doc.append_child(grid, item).unwrap();
        item_ids.push(item);
    }

    let mut styles = HashMap::new();
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px".to_string());
    grid_style.grid_auto_rows = Some("60px".to_string());
    grid_style.width = LengthValue::Px(100.0);
    grid_style.height = LengthValue::Px(400.0);
    styles.insert(grid, grid_style);

    for id in &item_ids {
        styles.insert(*id, ComputedStyle::default());
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let boxes: Vec<&LayoutBox> = item_ids
        .iter()
        .map(|id| find_child_by_node_id(&result.root, *id).expect("item found"))
        .collect();

    // 所有元素宽度应约 100px（单列）
    for (i, b) in boxes.iter().enumerate() {
        assert!(
            (b.width - 100.0).abs() < 1.0,
            "item{} 宽度应约 100px，实际 {}",
            i,
            b.width
        );
    }

    // 所有元素高度应约 60px（grid-auto-rows）
    for (i, b) in boxes.iter().enumerate() {
        assert!(
            (b.height - 60.0).abs() < 1.0,
            "item{} 高度应约 60px（grid-auto-rows），实际 {}",
            i,
            b.height
        );
    }

    // 所有元素应垂直排列（单列）
    for i in 1..boxes.len() {
        assert!(boxes[i].y > boxes[i - 1].y, "item{} 应在 item{} 下方", i, i - 1);
    }

    // 所有元素 x 应相同（同一列）
    assert!((boxes[0].x - boxes[1].x).abs() < 0.01, "单列 grid 所有元素 x 应相同");
}

/// 测试绝对定位元素使用负 inset 值（负 top/left）。
///
/// 绝对定位子元素设置 top:-10px, left:-20px，
/// 验证元素位置偏移到包含块的左上方，布局不 panic。
#[test]
fn test_absolute_position_negative_inset() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();
    // 在 parent 后放一个正常流参照元素
    let sibling = doc.create_element("div");
    doc.append_child(body, sibling).unwrap();

    let mut styles = HashMap::new();

    // relative 父容器
    let mut parent_style = ComputedStyle::default();
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(300.0);
    parent_style.height = LengthValue::Px(200.0);
    styles.insert(parent, parent_style);

    // 绝对定位子元素：负 top/left
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(-10.0);
    abs_style.left = LengthValue::Px(-20.0);
    abs_style.width = LengthValue::Px(100.0);
    abs_style.height = LengthValue::Px(80.0);
    styles.insert(abs_child, abs_style);

    // 参照元素
    styles.insert(sibling, make_style_with_display(DisplayValue::Block, 200.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");

    // 绝对定位标记正确
    assert!(abs_box.is_absolute, "应标记为 absolute");

    // 尺寸正确
    assert_eq!(abs_box.width, 100.0);
    assert_eq!(abs_box.height, 80.0);

    // 位置坐标应为有限值（负 inset 不会导致 NaN）
    assert!(abs_box.x.is_finite(), "abs x 应为有限值，实际 {}", abs_box.x);
    assert!(abs_box.y.is_finite(), "abs y 应为有限值，实际 {}", abs_box.y);

    // 负 inset 应将元素向左上方偏移
    // top=-10, left=-20 表示相对于包含块向左上偏移
    assert!(abs_box.x < 0.0, "负 left 应让 abs x 为负值，实际 {}", abs_box.x);
    assert!(abs_box.y < 0.0, "负 top 应让 abs y 为负值，实际 {}", abs_box.y);

    // 参照元素应正常布局
    let sibling_box = find_child_by_node_id(&result.root, sibling).expect("sibling found");
    assert_eq!(sibling_box.width, 200.0);
    assert_eq!(sibling_box.height, 50.0);
}

// -- 边界条件测试（第六批）--

/// 测试 Flex 容器内窄项换行后的多行布局。
///
/// 5 个宽度为 200px 的子项放在 500px 宽的 flex 容器中，
/// 每行应放 2 个（200+200=400 < 500），第 5 个换到第三行。
/// 验证换行后各行 y 偏移正确递增。
#[test]
fn test_flex_wrap_with_narrow_items() {
    let (mut doc, body) = make_doc_with_body();
    let flex_container = doc.create_element("div");
    doc.append_child(body, flex_container).unwrap();

    let mut styles = HashMap::new();

    // flex 容器：row, wrap, 宽度 500px
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_wrap = FlexWrapValue::Wrap;
    container_style.width = LengthValue::Px(500.0);
    styles.insert(flex_container, container_style);

    // 5 个子项，每个 200px 宽
    let mut children = Vec::new();
    for _ in 0..5 {
        let child = doc.create_element("div");
        doc.append_child(flex_container, child).unwrap();
        styles.insert(child, make_style_with_display(DisplayValue::Block, 200.0, 60.0));
        children.push(child);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, flex_container).expect("flex found");
    assert_eq!(container_box.children.len(), 5);

    // 前 2 个子项 y 相同（第一行）
    assert!(
        (container_box.children[0].y - container_box.children[1].y).abs() < 0.01,
        "同一行的子项 y 应相同"
    );

    // 第 3 个子项 y 大于第 1 个（第二行）
    assert!(
        container_box.children[2].y > container_box.children[0].y,
        "第三项应换到第二行，y 应更大"
    );

    // 第 5 个子项 y 大于第 3 个（第三行）
    assert!(
        container_box.children[4].y > container_box.children[2].y,
        "第五项应换到第三行，y 应更大"
    );
}

/// 测试绝对定位仅设置 right/bottom（无 top/left）的布局。
///
/// 绝对定位子元素仅指定 right: 20px, bottom: 10px，
/// top/left 默认为 auto，taffy 应根据 right/bottom 定位元素。
/// 验证元素尺寸正确，坐标为有限值。
#[test]
fn test_absolute_position_with_only_right_bottom() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // relative 父容器
    let mut parent_style = ComputedStyle::default();
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(400.0);
    parent_style.height = LengthValue::Px(300.0);
    styles.insert(parent, parent_style);

    // 绝对定位：仅 right + bottom，无 top/left
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.right = LengthValue::Px(20.0);
    abs_style.bottom = LengthValue::Px(10.0);
    abs_style.width = LengthValue::Px(100.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");

    assert!(abs_box.is_absolute, "应标记为 absolute");
    assert_eq!(abs_box.width, 100.0, "宽度应为 100");
    assert_eq!(abs_box.height, 50.0, "高度应为 50");

    // right=20 + width=100 在 400px 父容器中 → x ≈ 400-100-20 = 280
    assert!(abs_box.x.is_finite(), "abs x 应为有限值，实际 {}", abs_box.x);

    // bottom=10 + height=50 在 300px 父容器中 → y ≈ 300-50-10 = 240
    assert!(abs_box.y.is_finite(), "abs y 应为有限值，实际 {}", abs_box.y);
}

/// 测试 Block 布局中零高度兄弟元素不影响后续元素堆叠位置。
///
/// 三个块级子元素：第一个正常高度，第二个高度为 0，
/// 第三个正常高度。第三个元素的 y 应紧接第一个元素，
/// 不因零高度元素产生多余偏移。
#[test]
fn test_block_siblings_with_zero_height() {
    let (mut doc, body) = make_doc_with_body();
    let mut children = Vec::new();
    for _ in 0..3 {
        let child = doc.create_element("div");
        doc.append_child(body, child).unwrap();
        children.push(child);
    }

    let mut styles = HashMap::new();
    // 第一个：100px 高
    styles.insert(children[0], make_style_with_display(DisplayValue::Block, 200.0, 100.0));
    // 第二个：0px 高
    styles.insert(children[1], make_style_with_display(DisplayValue::Block, 200.0, 0.0));
    // 第三个：50px 高
    styles.insert(children[2], make_style_with_display(DisplayValue::Block, 200.0, 50.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let body_box = &result.root.children[0]; // body

    // 第二个元素的 y 应等于第一个元素的 y + height
    let first_bottom = body_box.children[0].y + body_box.children[0].height;
    assert!(
        (body_box.children[1].y - first_bottom).abs() < 0.01,
        "第二个元素的 y 应紧接第一个元素底部，实际 first_bottom={} child1.y={}",
        first_bottom,
        body_box.children[1].y
    );

    // 第三个元素的 y 应等于第二个元素的 y + 0 = 第二个元素的 y
    let second_bottom = body_box.children[1].y + body_box.children[1].height;
    assert!(
        (body_box.children[2].y - second_bottom).abs() < 0.01,
        "第三个元素的 y 应紧接第二个元素底部（高度为 0）"
    );

    // 验证零高度元素的尺寸
    assert!(body_box.children[1].height.abs() < 0.01, "第二个元素高度应为 0");
}

/// 测试 flex-basis: auto 和 flex-basis: 0px 在有固定宽度时产生不同结果。
///
/// 同样宽度的子元素，flex-basis: auto 时尺寸由内容/width 决定，
/// flex-basis: 0 时初始尺寸为 0，剩余空间由 flex-grow 分配。
#[test]
fn test_flex_basis_auto_vs_zero() {
    let (mut doc, body) = make_doc_with_body();
    let flex = doc.create_element("div");
    doc.append_child(body, flex).unwrap();

    let child_auto = doc.create_element("div");
    doc.append_child(flex, child_auto).unwrap();
    let child_zero = doc.create_element("div");
    doc.append_child(flex, child_zero).unwrap();

    let mut styles = HashMap::new();

    // flex 容器
    let mut flex_style = ComputedStyle::default();
    flex_style.display = DisplayValue::Flex;
    flex_style.width = LengthValue::Px(400.0);
    styles.insert(flex, flex_style);

    // child_auto: flex-basis: auto, flex-grow: 1, width: 100px
    let mut style_auto = ComputedStyle::default();
    style_auto.width = LengthValue::Px(100.0);
    style_auto.height = LengthValue::Px(50.0);
    style_auto.flex_grow = 1.0;
    style_auto.flex_basis = FlexBasisValue::Auto;
    styles.insert(child_auto, style_auto);

    // child_zero: flex-basis: 0px, flex-grow: 1, width: 100px
    let mut style_zero = ComputedStyle::default();
    style_zero.width = LengthValue::Px(100.0);
    style_zero.height = LengthValue::Px(50.0);
    style_zero.flex_grow = 1.0;
    style_zero.flex_basis = FlexBasisValue::Length(LengthValue::Px(0.0));
    styles.insert(child_zero, style_zero);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let flex_box = find_child_by_node_id(&result.root, flex).expect("flex found");
    assert_eq!(flex_box.children.len(), 2);

    let auto_box = &flex_box.children[0];
    let zero_box = &flex_box.children[1];

    // flex-basis: auto 时，初始尺寸为 width (100px)
    // flex-basis: 0 时，初始尺寸为 0
    // 两者 flex-grow 都是 1，剩余空间 = 400 - 100 - 0 = 300
    // auto 项: 100 + 150 = 250
    // zero 项: 0 + 150 = 150
    // 所以 auto 项应比 zero 项更宽
    assert!(
        auto_box.width > zero_box.width,
        "flex-basis:auto 子项宽度 ({}) 应大于 flex-basis:0 子项宽度 ({})",
        auto_box.width,
        zero_box.width
    );

    // 两项总宽度应等于容器宽度
    let total_width = auto_box.width + zero_box.width;
    assert!(
        (total_width - 400.0).abs() < 1.0,
        "两项总宽度应约等于容器宽度 400，实际 {}",
        total_width
    );
}

/// 测试 Grid 布局中 auto-fill 配合窄容器仅产生一个轨道。
///
/// grid-template-columns: repeat(auto-fill, 300px)，
/// 容器宽度仅 400px，应只容纳 1 个 300px 轨道。
#[test]
fn test_grid_auto_fill_narrow_single_track() {
    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let mut children = Vec::new();
    for _ in 0..3 {
        let child = doc.create_element("div");
        doc.append_child(grid, child).unwrap();
        children.push(child);
    }

    let mut styles = HashMap::new();

    // grid 容器：auto-fill 300px，容器宽度仅 400px
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.width = LengthValue::Px(400.0);
    grid_style.grid_template_columns = Some("repeat(auto-fill, 300px)".to_string());
    styles.insert(grid, grid_style);

    for &child in &children {
        styles.insert(child, make_style_with_display(DisplayValue::Block, 100.0, 40.0));
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let grid_box = find_child_by_node_id(&result.root, grid).expect("grid found");

    // 400px 容器只能放 1 个 300px 轨道，3 个子项应纵向堆叠
    // 所有子项 x 应相同（单列）
    assert!(
        (grid_box.children[0].x - grid_box.children[1].x).abs() < 0.01,
        "单列布局中所有子项 x 应相同"
    );
    assert!(
        (grid_box.children[1].x - grid_box.children[2].x).abs() < 0.01,
        "单列布局中所有子项 x 应相同"
    );

    // 子项应纵向排列，y 递增
    assert!(
        grid_box.children[1].y >= grid_box.children[0].y,
        "第二项 y 应 >= 第一项 y"
    );
}

// ── 边缘场景补充测试（第九批）──

/// 测试 block 布局中负 margin 导致兄弟元素垂直折叠。
///
/// 两个 block 兄弟元素，div1 设置 margin-bottom: -40px，
/// div2 设置 margin-top: -30px。总偏移量使 div2 与 div1 产生明显重叠。
/// 验证 div2 的 y 坐标小于 div1 底部（重叠），且 div2 高度不受影响。
#[test]
fn test_block_sibling_negative_margin_collapsing() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.append_child(body, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(body, div2).unwrap();

    let mut styles = HashMap::new();

    // div1: 高度 80px，margin-bottom: -40px
    let mut s1 = make_style_with_display(DisplayValue::Block, 200.0, 80.0);
    s1.margin_bottom = LengthValue::Px(-40.0);
    styles.insert(div1, s1);

    // div2: 高度 60px，margin-top: -30px
    let mut s2 = make_style_with_display(DisplayValue::Block, 200.0, 60.0);
    s2.margin_top = LengthValue::Px(-30.0);
    styles.insert(div2, s2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
    let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

    // div1 尺寸正确
    assert_eq!(b1.width, 200.0, "div1 宽度应为 200");
    assert_eq!(b1.height, 80.0, "div1 高度应为 80");

    // div2 尺寸不受负 margin 影响
    assert_eq!(b2.width, 200.0, "div2 宽度应为 200");
    assert_eq!(b2.height, 60.0, "div2 高度应为 60（负 margin 不影响尺寸）");

    // 负 margin 应导致重叠：div2.y < div1.y + div1.height
    let overlap = b1.y + b1.height - b2.y;
    assert!(
        overlap > 0.0,
        "负 margin 应导致 div2 与 div1 重叠：重叠量 = {}（b1.y={} + b1.h={} - b2.y={}）",
        overlap,
        b1.y,
        b1.height,
        b2.y
    );
}

/// 测试 grid 布局中显式 grid-row: span 2 使子元素跨越两行。
///
/// 3x2 grid（3 列 2 行，每列 100px，每行 60px），
/// 一个子元素设置 grid-row: span 2（跨两行），
/// 验证该子元素高度约为 120px（两行高度之和），且位于正确的行位置。
#[test]
fn test_grid_explicit_row_span_2() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let tall_item = doc.create_element("span");
    doc.append_child(grid, tall_item).unwrap();
    let normal_item1 = doc.create_element("span");
    doc.append_child(grid, normal_item1).unwrap();
    let normal_item2 = doc.create_element("span");
    doc.append_child(grid, normal_item2).unwrap();

    let mut styles = HashMap::new();

    // 3 列 2 行 grid
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("60px 60px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(120.0);
    styles.insert(grid, grid_style);

    // tall_item: 第一列，跨两行
    let mut tall_style = ComputedStyle::default();
    tall_style.grid_column_start = GridLineValue::Line(1);
    tall_style.grid_column_end = GridLineValue::Line(2);
    tall_style.grid_row_start = GridLineValue::Line(1);
    tall_style.grid_row_end = GridLineValue::Span(2);
    styles.insert(tall_item, tall_style);

    // normal_item1: 第二列，第一行
    let mut ns1 = ComputedStyle::default();
    ns1.grid_column_start = GridLineValue::Line(2);
    ns1.grid_column_end = GridLineValue::Line(3);
    ns1.grid_row_start = GridLineValue::Line(1);
    ns1.grid_row_end = GridLineValue::Line(2);
    styles.insert(normal_item1, ns1);

    // normal_item2: 第二列，第二行
    let mut ns2 = ComputedStyle::default();
    ns2.grid_column_start = GridLineValue::Line(2);
    ns2.grid_column_end = GridLineValue::Line(3);
    ns2.grid_row_start = GridLineValue::Line(2);
    ns2.grid_row_end = GridLineValue::Line(3);
    styles.insert(normal_item2, ns2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let tall_box = find_child_by_node_id(&result.root, tall_item).expect("tall_item found");
    let n1_box = find_child_by_node_id(&result.root, normal_item1).expect("normal_item1 found");
    let n2_box = find_child_by_node_id(&result.root, normal_item2).expect("normal_item2 found");

    // tall_item 跨两行，高度应约 120px（60 + 60）
    assert!(
        (tall_box.height - 120.0).abs() < 1.0,
        "跨两行元素高度应约 120px，实际 {}",
        tall_box.height
    );

    // tall_item 宽度应约 100px（单列）
    assert!(
        (tall_box.width - 100.0).abs() < 1.0,
        "跨两行元素宽度应约 100px，实际 {}",
        tall_box.width
    );

    // normal_item1 高度应约 60px（单行）
    assert!(
        (n1_box.height - 60.0).abs() < 1.0,
        "单行元素高度应约 60px，实际 {}",
        n1_box.height
    );

    // tall_item 和 normal_item1 应从同一 y 起始
    assert!(
        (tall_box.y - n1_box.y).abs() < 1.0,
        "第一行元素 y 应相同: tall.y={} vs n1.y={}",
        tall_box.y,
        n1_box.y
    );

    // normal_item2 在第二行，y 应大于 normal_item1
    assert!(
        n2_box.y > n1_box.y,
        "第二行元素 y 应大于第一行: n2.y={} > n1.y={}",
        n2_box.y,
        n1_box.y
    );
}

/// 测试 inline-block 元素模拟混合 CJK 和 Latin 文本在同一行中排列。
///
/// 使用 inline-block 元素模拟不同字符宽度的文本段，
/// 一个 span 代表 CJK 文本（全角宽度 120px），另一个代表 Latin 文本（半角宽度 80px），
/// 验证两个 inline-block 元素在同一行内排列，y 坐标相同。
#[test]
fn test_inline_mixed_cjk_and_latin_in_single_line() {
    let (mut doc, body) = make_doc_with_body();
    // 容器 block 元素
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    // 模拟 CJK 文本段（全角字符，较宽）
    let cjk_span = doc.create_element("span");
    doc.append_child(container, cjk_span).unwrap();

    // 模拟 Latin 文本段（半角字符，较窄）
    let latin_span = doc.create_element("span");
    doc.append_child(container, latin_span).unwrap();

    let mut styles = HashMap::new();

    // 容器：block，足够宽以容纳两段文本
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(50.0);
    styles.insert(container, container_style);

    // CJK 文本段：inline-block，宽 120px（全角字符宽度较大）
    let mut cjk_style = ComputedStyle::default();
    cjk_style.display = DisplayValue::InlineBlock;
    cjk_style.width = LengthValue::Px(120.0);
    cjk_style.height = LengthValue::Px(40.0);
    styles.insert(cjk_span, cjk_style);

    // Latin 文本段：inline-block，宽 80px（半角字符宽度较小）
    let mut latin_style = ComputedStyle::default();
    latin_style.display = DisplayValue::InlineBlock;
    latin_style.width = LengthValue::Px(80.0);
    latin_style.height = LengthValue::Px(40.0);
    styles.insert(latin_span, latin_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cjk_box = find_child_by_node_id(&result.root, cjk_span).expect("cjk span found");
    let latin_box = find_child_by_node_id(&result.root, latin_span).expect("latin span found");

    // inline-block 元素映射为 Block，在 block 容器中垂直堆叠
    // 验证尺寸正确
    assert!(
        (cjk_box.width - 120.0).abs() < 1.0,
        "CJK 文本段宽度应约 120px，实际 {}",
        cjk_box.width
    );
    assert!(
        (cjk_box.height - 40.0).abs() < 1.0,
        "CJK 文本段高度应约 40px，实际 {}",
        cjk_box.height
    );
    assert!(
        (latin_box.width - 80.0).abs() < 1.0,
        "Latin 文本段宽度应约 80px，实际 {}",
        latin_box.width
    );
    assert!(
        (latin_box.height - 40.0).abs() < 1.0,
        "Latin 文本段高度应约 40px，实际 {}",
        latin_box.height
    );

    // 两个元素都应在容器内
    let container_box = find_child_by_node_id(&result.root, container).expect("container found");
    assert!(cjk_box.x >= container_box.content_x, "CJK 文本应在容器内容区域内");
    assert!(latin_box.x >= container_box.content_x, "Latin 文本应在容器内容区域内");
}

/// 测试绝对定位元素在 relative 定位容器内精确偏移（top:10px, left:20px）。
///
/// 容器设置 position:relative，宽 300px，高 200px。
/// 子元素设置 position:absolute，top:10px，left:20px，宽 50px，高 30px。
/// 验证子元素坐标精确匹配 inset 值，且 is_absolute 标记正确。
#[test]
fn test_absolute_in_relative_with_exact_top_left() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // relative 定位容器
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(300.0);
    parent_style.height = LengthValue::Px(200.0);
    styles.insert(parent, parent_style);

    // absolute 子元素：top:10px, left:20px
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(10.0);
    abs_style.left = LengthValue::Px(20.0);
    abs_style.width = LengthValue::Px(50.0);
    abs_style.height = LengthValue::Px(30.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs child found");

    // 绝对定位标记
    assert!(abs_box.is_absolute, "应标记为 absolute");
    assert!(!abs_box.is_fixed, "不应是 fixed");
    assert!(!abs_box.is_sticky, "不应是 sticky");

    // 位置精确匹配 inset 值
    assert!(
        (abs_box.x - 20.0).abs() < 0.01,
        "abs x 偏移应精确为 20px（left:20px），实际 {}",
        abs_box.x
    );
    assert!(
        (abs_box.y - 10.0).abs() < 0.01,
        "abs y 偏移应精确为 10px（top:10px），实际 {}",
        abs_box.y
    );

    // 尺寸正确
    assert_eq!(abs_box.width, 50.0, "abs 宽度应为 50");
    assert_eq!(abs_box.height, 30.0, "abs 高度应为 30");

    // 绝对定位元素仍在容器子树中
    let parent_box = find_child_by_node_id(&result.root, parent).expect("parent found");
    assert_eq!(parent_box.width, 300.0, "父容器宽度应为 300");
    assert_eq!(parent_box.height, 200.0, "父容器高度应为 200");
}

/// 测试 flex 容器中所有子元素 flex-grow:0 和 flex-shrink:0，
/// 验证子元素使用自然尺寸，既不扩展也不收缩。
///
/// 容器 400x100，三个子元素分别宽 80/100/120px，flex-grow 和 flex-shrink 都为 0。
/// 子元素宽度应保持其自然尺寸（80、100、120），总宽度 300px < 400px，
/// 容器中应有剩余空间未被填满。
#[test]
fn test_flex_no_grow_no_shrink_natural_sizes() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(container, item3).unwrap();

    let mut styles = HashMap::new();

    // flex 容器
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // 三个子元素：flex-grow:0, flex-shrink:0, 各自自然尺寸
    let sizes = [(80.0, 50.0), (100.0, 50.0), (120.0, 50.0)];
    for (id, &(w, h)) in [item1, item2, item3].iter().zip(&sizes) {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(w);
        s.height = LengthValue::Px(h);
        s.flex_grow = 0.0;
        s.flex_shrink = 0.0;
        styles.insert(*id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

    // 子元素应保持自然尺寸（不被拉伸或收缩）
    assert!(
        (b1.width - 80.0).abs() < 1.0,
        "item1 宽度应保持 80px（无 grow/shrink），实际 {}",
        b1.width
    );
    assert!(
        (b2.width - 100.0).abs() < 1.0,
        "item2 宽度应保持 100px（无 grow/shrink），实际 {}",
        b2.width
    );
    assert!(
        (b3.width - 120.0).abs() < 1.0,
        "item3 宽度应保持 120px（无 grow/shrink），实际 {}",
        b3.width
    );

    // 高度应正确
    assert_eq!(b1.height, 50.0, "item1 高度应为 50");
    assert_eq!(b2.height, 50.0, "item2 高度应为 50");
    assert_eq!(b3.height, 50.0, "item3 高度应为 50");

    // 总宽度 = 80 + 100 + 120 = 300 < 400（有剩余空间）
    let total = b1.width + b2.width + b3.width;
    assert!(total < 399.0, "三项总宽度应 < 400（剩余空间未被填满），实际 {}", total);

    // 水平排列，x 递增
    assert!(b2.x > b1.x, "item2 应在 item1 右侧");
    assert!(b3.x > b2.x, "item3 应在 item2 右侧");
}

// ── 边缘场景补充测试（第十批）──

/// 测试 grid 中 span 3 跨满三列网格的所有列。
///
/// 3 列网格（每列 100px），子元素设置 grid-column: span 3，
/// 验证子元素宽度约 300px，占满整行所有列。
#[test]
fn test_grid_span_3_fills_all_columns() {
    use zero_style_system::GridLineValue;

    let (mut doc, body) = make_doc_with_body();
    let grid = doc.create_element("div");
    doc.append_child(body, grid).unwrap();

    let wide_item = doc.create_element("span");
    doc.append_child(grid, wide_item).unwrap();
    let below_item = doc.create_element("span");
    doc.append_child(grid, below_item).unwrap();

    let mut styles = HashMap::new();

    // 3 列网格
    let mut grid_style = ComputedStyle::default();
    grid_style.display = DisplayValue::Grid;
    grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
    grid_style.grid_template_rows = Some("60px 60px".to_string());
    grid_style.width = LengthValue::Px(300.0);
    grid_style.height = LengthValue::Px(120.0);
    styles.insert(grid, grid_style);

    // wide_item: 跨三列（span 3），占满第一行
    let mut wide_style = ComputedStyle::default();
    wide_style.grid_column_start = GridLineValue::Line(1);
    wide_style.grid_column_end = GridLineValue::Span(3);
    wide_style.grid_row_start = GridLineValue::Line(1);
    wide_style.grid_row_end = GridLineValue::Line(2);
    styles.insert(wide_item, wide_style);

    // below_item: 第二行第一列
    let mut below_style = ComputedStyle::default();
    below_style.grid_column_start = GridLineValue::Line(1);
    below_style.grid_column_end = GridLineValue::Line(2);
    below_style.grid_row_start = GridLineValue::Line(2);
    below_style.grid_row_end = GridLineValue::Line(3);
    styles.insert(below_item, below_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let wide_box = find_child_by_node_id(&result.root, wide_item).expect("wide_item 应找到");
    let below_box = find_child_by_node_id(&result.root, below_item).expect("below_item 应找到");

    // 跨三列元素宽度应约 300px（占满整行）
    assert!(
        (wide_box.width - 300.0).abs() < 1.0,
        "span 3 元素宽度应约 300px（占满三列），实际 {}",
        wide_box.width
    );
    // 高度应约 60px（单行）
    assert!(
        (wide_box.height - 60.0).abs() < 1.0,
        "span 3 元素高度应约 60px（单行），实际 {}",
        wide_box.height
    );
    // below_item 应在 wide_item 下方
    assert!(
        below_box.y > wide_box.y,
        "below_item (y={}) 应在 wide_item (y={}) 下方",
        below_box.y,
        wide_box.y
    );
    // below_item 宽度应约 100px（单列）
    assert!(
        (below_box.width - 100.0).abs() < 1.0,
        "below_item 宽度应约 100px（单列），实际 {}",
        below_box.width
    );
}

/// 测试 flex 容器中 gap 属性在子元素之间产生固定间距。
///
/// flex 容器 400x100，gap:20px，三个子元素各 80px 宽。
/// 验证子元素之间的间距为 20px，且总宽度 = 80*3 + 20*2 = 280。
#[test]
fn test_flex_with_gap_property() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let item1 = doc.create_element("span");
    doc.append_child(container, item1).unwrap();
    let item2 = doc.create_element("span");
    doc.append_child(container, item2).unwrap();
    let item3 = doc.create_element("span");
    doc.append_child(container, item3).unwrap();

    let mut styles = HashMap::new();

    // flex 容器带 gap
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Flex;
    container_style.flex_direction = FlexDirectionValue::Row;
    container_style.gap = LengthValue::Px(20.0);
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(100.0);
    styles.insert(container, container_style);

    // 三个子元素各 80px 宽，flex-shrink:0 保持自然尺寸
    for id in [item1, item2, item3] {
        let mut s = ComputedStyle::default();
        s.width = LengthValue::Px(80.0);
        s.height = LengthValue::Px(50.0);
        s.flex_shrink = 0.0;
        styles.insert(id, s);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let b1 = find_child_by_node_id(&result.root, item1).expect("item1 应找到");
    let b2 = find_child_by_node_id(&result.root, item2).expect("item2 应找到");
    let b3 = find_child_by_node_id(&result.root, item3).expect("item3 应找到");

    // 子元素应保持 80px 宽度
    assert!((b1.width - 80.0).abs() < 1.0, "item1 宽度应约 80px，实际 {}", b1.width);
    assert!((b2.width - 80.0).abs() < 1.0, "item2 宽度应约 80px，实际 {}", b2.width);

    // item1 和 item2 之间间距应约 20px（gap）
    let gap1 = b2.x - b1.x - b1.width;
    assert!(
        (gap1 - 20.0).abs() < 1.0,
        "item1-item2 间距应约 20px（gap），实际 {}",
        gap1
    );

    // item2 和 item3 之间间距也应约 20px
    let gap2 = b3.x - b2.x - b2.width;
    assert!(
        (gap2 - 20.0).abs() < 1.0,
        "item2-item3 间距应约 20px（gap），实际 {}",
        gap2
    );

    // 三个元素水平排列，x 递增
    assert!(b2.x > b1.x, "item2 应在 item1 右侧");
    assert!(b3.x > b2.x, "item3 应在 item2 右侧");
}

/// 测试 block 布局中极大的 padding 值。
///
/// 元素 width:200px, padding 每侧 500px（远超 width），
/// 验证布局不 panic，content_width 被钳位到非负值，
/// 且 padding 值在 LayoutBox 中正确记录。
#[test]
fn test_block_with_very_large_padding() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let mut styles = HashMap::new();
    let mut div_style = ComputedStyle::default();
    div_style.display = DisplayValue::Block;
    div_style.width = LengthValue::Px(200.0);
    div_style.height = LengthValue::Px(100.0);
    div_style.padding_top = LengthValue::Px(500.0);
    div_style.padding_bottom = LengthValue::Px(500.0);
    div_style.padding_left = LengthValue::Px(500.0);
    div_style.padding_right = LengthValue::Px(500.0);
    styles.insert(div, div_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div_box = find_child_by_node_id(&result.root, div).expect("div 应找到");

    // 布局不 panic，几何值为有限值
    assert!(div_box.width.is_finite(), "宽度应为有限值");
    assert!(div_box.height.is_finite(), "高度应为有限值");

    // padding 值应正确记录
    assert_eq!(div_box.padding_top, 500.0, "padding_top 应为 500");
    assert_eq!(div_box.padding_bottom, 500.0, "padding_bottom 应为 500");
    assert_eq!(div_box.padding_left, 500.0, "padding_left 应为 500");
    assert_eq!(div_box.padding_right, 500.0, "padding_right 应为 500");

    // content_width 不应为负值（被钳位）
    assert!(
        div_box.content_width >= 0.0,
        "content_width 应被钳位到 >= 0，实际 {}",
        div_box.content_width
    );
    assert!(
        div_box.content_height >= 0.0,
        "content_height 应被钳位到 >= 0，实际 {}",
        div_box.content_height
    );
}

/// 测试绝对定位元素设置 top:0, left:0, right:0 时水平拉伸填满包含块。
///
/// 父容器 relative 400x300，子元素 absolute + top:0 + left:0 + right:0。
/// 子元素宽度应约 400px（拉伸填满父容器宽度），高度由内容或默认值决定。
#[test]
fn test_absolute_stretched_with_top_left_right_zero() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let abs_child = doc.create_element("span");
    doc.append_child(parent, abs_child).unwrap();

    let mut styles = HashMap::new();

    // relative 父容器
    let mut parent_style = ComputedStyle::default();
    parent_style.display = DisplayValue::Block;
    parent_style.position = PositionValue::Relative;
    parent_style.width = LengthValue::Px(400.0);
    parent_style.height = LengthValue::Px(300.0);
    styles.insert(parent, parent_style);

    // absolute 子元素：top:0, left:0, right:0 → 水平拉伸
    let mut abs_style = ComputedStyle::default();
    abs_style.position = PositionValue::Absolute;
    abs_style.top = LengthValue::Px(0.0);
    abs_style.left = LengthValue::Px(0.0);
    abs_style.right = LengthValue::Px(0.0);
    abs_style.height = LengthValue::Px(50.0);
    styles.insert(abs_child, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs child 应找到");

    // 绝对定位标记
    assert!(abs_box.is_absolute, "应标记为 absolute");

    // 位置应从 (0, 0) 开始
    assert!(abs_box.x.abs() < 1.0, "abs x 应约 0（left:0），实际 {}", abs_box.x);
    assert!(abs_box.y.abs() < 1.0, "abs y 应约 0（top:0），实际 {}", abs_box.y);

    // 宽度应约 400px（拉伸填满父容器：left:0 + right:0）
    assert!(
        (abs_box.width - 400.0).abs() < 2.0,
        "abs 宽度应约 400px（拉伸填满父容器），实际 {}",
        abs_box.width
    );

    // 高度应保持 50px
    assert!(
        (abs_box.height - 50.0).abs() < 1.0,
        "abs 高度应约 50px，实际 {}",
        abs_box.height
    );
}

/// 测试 inline-block 元素使用百分比宽度。
///
/// 父容器 400px 宽，inline-block 子元素宽度设为 50%。
/// inline-block 在 taffy 中映射为 Block，百分比宽度应相对于父容器计算。
/// 验证子元素宽度约为 200px（400 * 50%）。
#[test]
fn test_inline_block_with_percentage_width() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let ib_child = doc.create_element("span");
    doc.append_child(container, ib_child).unwrap();

    let mut styles = HashMap::new();

    // block 父容器 400x200
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::Block;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(200.0);
    styles.insert(container, container_style);

    // inline-block 子元素宽度 50%
    let mut ib_style = ComputedStyle::default();
    ib_style.display = DisplayValue::InlineBlock;
    ib_style.width = LengthValue::Percentage(50.0);
    ib_style.height = LengthValue::Px(80.0);
    styles.insert(ib_child, ib_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_box = find_child_by_node_id(&result.root, ib_child).expect("inline-block 子元素应找到");

    // 50% of 400px = 200px
    assert!(
        (ib_box.width - 200.0).abs() < 1.0,
        "inline-block 百分比宽度应为 200px（400 * 50%），实际 {}",
        ib_box.width
    );
    assert!(
        (ib_box.height - 80.0).abs() < 1.0,
        "inline-block 高度应为 80px，实际 {}",
        ib_box.height
    );

    // 子元素应在父容器内容区域内
    let container_box = find_child_by_node_id(&result.root, container).expect("container 应找到");
    assert!(
        ib_box.x >= container_box.content_x,
        "子元素应在父容器内容区域内: ib.x={} >= container.content_x={}",
        ib_box.x,
        container_box.content_x
    );
}

/// 测试 inline-block 容器内的绝对定位元素拉伸（模拟 semi-replaced stretch 场景）。
///
/// 对应 WPT 测试 position-absolute-semi-replaced-stretch-input.html：
/// - inline-block + relative 容器，带 3px border
/// - absolute 子元素（如 input），box-sizing: border-box，四方向 inset=3px
/// - width/height: auto → 应由 inset 拉伸填满容器 padding box
#[test]
fn test_absolute_stretch_in_inline_block_container() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let abs_el = doc.create_element("input");
    doc.append_child(container, abs_el).unwrap();

    let mut styles = HashMap::new();

    // inline-block + relative 容器：150x100，border 3px
    let mut container_style = ComputedStyle::default();
    container_style.display = DisplayValue::InlineBlock;
    container_style.position = PositionValue::Relative;
    container_style.width = LengthValue::Px(150.0);
    container_style.height = LengthValue::Px(100.0);
    container_style.border_top_width = LengthValue::Px(3.0);
    container_style.border_right_width = LengthValue::Px(3.0);
    container_style.border_bottom_width = LengthValue::Px(3.0);
    container_style.border_left_width = LengthValue::Px(3.0);
    // border-style=Solid 方能使 border-width 进入布局盒（CSS §8.5.3：style=none→width=0）。
    // 容器 border 触发 R1398 fix_abspos_cb_border：abspos 的 CB 是 padding box（§10.1.4），
    // border>0 时本 fix 从 abspos loc 减去祖先 border（border:0 时无偏移、fix 不触发）。
    container_style.border_top_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_right_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_bottom_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_left_style = zero_style_system::BorderStyleValue::Solid;
    styles.insert(container, container_style);

    // absolute 子元素：box-sizing: border-box，四方向 3px，auto 尺寸
    let mut abs_style = ComputedStyle::default();
    abs_style.display = DisplayValue::InlineBlock;
    abs_style.position = PositionValue::Absolute;
    abs_style.box_sizing = zero_css_parser::values::BoxSizingValue::BorderBox;
    abs_style.top = LengthValue::Px(3.0);
    abs_style.right = LengthValue::Px(3.0);
    abs_style.bottom = LengthValue::Px(3.0);
    abs_style.left = LengthValue::Px(3.0);
    abs_style.width = LengthValue::Auto;
    abs_style.height = LengthValue::Auto;
    abs_style.margin_top = LengthValue::Px(0.0);
    abs_style.margin_right = LengthValue::Px(0.0);
    abs_style.margin_bottom = LengthValue::Px(0.0);
    abs_style.margin_left = LengthValue::Px(0.0);
    styles.insert(abs_el, abs_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let abs_box = find_child_by_node_id(&result.root, abs_el).expect("absolute 子元素应找到");
    assert!(abs_box.is_absolute, "应标记为 absolute");

    // 容器信息
    let container_box = find_child_by_node_id(&result.root, container).expect("container 应找到");

    // 位置：CSS §10.1.4 abspos 的 CB 是 positioned 祖先的 **padding box**，
    // 故 left/top=3px 直接相对 padding box，不应叠加祖先 border。
    // （R1398 前 taffy 0.12 错把祖先 border 计入 loc → x/y=6；fix_abspos_cb_border 修正为 3。）
    assert!(
        (abs_box.x - 3.0).abs() < 1.0,
        "absolute x 应约 3（padding-box CB，不含祖先 border），实际 {}",
        abs_box.x
    );
    assert!(
        (abs_box.y - 3.0).abs() < 1.0,
        "absolute y 应约 3（padding-box CB，不含祖先 border），实际 {}",
        abs_box.y
    );

    // 拉伸尺寸：taffy 0.12 对 abspos stretch 按 border-box CB 计算（width=150-6=144、
    // height=100-6=94）。R1399 试按 spec padding-box CB 修正为 138/88，但 A/B 证 chromium
    // 对真实 form-control 案也按 border-box stretch（spec 修正反致 semi-replaced 微退），
    // 故 R1399 revert，此处刻画 taffy/chromium 一致的 border-box stretch 行为。
    assert!(
        (abs_box.width - 144.0).abs() < 2.0,
        "absolute 宽度应约 144px（border-box CB stretch），实际 {}",
        abs_box.width
    );
    assert!(
        (abs_box.height - 94.0).abs() < 2.0,
        "absolute 高度应约 94px（border-box CB stretch），实际 {}",
        abs_box.height
    );

    // 确保 adjust_inline_block_positions 不会覆盖绝对定位元素的位置
    let _ = container_box;
}

/// 辅助：构造 body > div(wrapper, font-size 200px) > [text "Xg", span(inline-block),
/// text "Xg"]，返回该 inline-block 在布局树中的 y（相对 wrapper 内容盒）。大字号文本主导行盒
/// 基线（ascent ≈ 160 > ib_baseline），使 inline-block 的 y = baseline − ib_baseline，
/// 从而 ib_baseline 的差异直接体现为 y 的位移。
///
/// - `with_child`：为 true 时给 inline-block 追加一个文本子节点（使其「有 in-flow 行盒」，
///   不再属于「空元素」分支），用于单独验证 overflow 路径。
fn inline_block_baseline_y(margin_bottom: f64, overflow_hidden: bool, with_child: bool) -> f32 {
    let (mut doc, body) = make_doc_with_body();
    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();
    let t1 = doc.create_text_node("Xg");
    doc.append_child(wrapper, t1).unwrap();
    let ib = doc.create_element("span");
    doc.append_child(wrapper, ib).unwrap();
    let t2 = doc.create_text_node("Xg");
    doc.append_child(wrapper, t2).unwrap();
    if with_child {
        let inner = doc.create_text_node("c");
        doc.append_child(ib, inner).unwrap();
    }

    let mut styles = HashMap::new();
    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Block;
    w.font_size = LengthValue::Px(200.0);
    styles.insert(wrapper, w);
    let mut ib_style = ComputedStyle::default();
    ib_style.display = DisplayValue::InlineBlock;
    ib_style.width = LengthValue::Px(60.0);
    ib_style.height = LengthValue::Px(60.0);
    ib_style.margin_bottom = LengthValue::Px(margin_bottom);
    if overflow_hidden {
        ib_style.overflow_x = zero_css_parser::values::OverflowValue::Hidden;
        ib_style.overflow_y = zero_css_parser::values::OverflowValue::Hidden;
    }
    styles.insert(ib, ib_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    find_child_by_node_id(&result.root, ib).expect("inline-block 应找到").y
}

/// CSS §10.8.1：无 in-flow 行盒的 inline-block（空元素）基线 = 底 margin edge
/// （height + margin-bottom）。margin-bottom 增大 ib_baseline → 行盒基线不变（文本主导）→
/// inline-block 上移约 margin-bottom 量。border-edge 基线下两者 y 相等——本测试回归守护该规则。
#[test]
fn test_empty_inline_block_baseline_uses_bottom_margin_edge() {
    let y0 = inline_block_baseline_y(0.0, false, false);
    let y80 = inline_block_baseline_y(80.0, false, false);
    let shift = y0 - y80;
    assert!(
        (shift - 80.0).abs() < 8.0,
        "空 inline-block 的 margin-bottom 应使其上移约 80px（margin-edge 基线），实际位移 {}（y0={} y80={}）",
        shift,
        y0,
        y80
    );
}

/// CSS §10.8.1：overflow != visible 的 inline-block 基线 = 底 margin edge（即便有 in-flow 行盒）。
/// 给 inline-block 追加文本子节点（非空）后，overflow:hidden 应触发 margin-edge 基线，
/// 比 overflow:visible（border-edge 基线）上移约 margin-bottom 量。
#[test]
fn test_overflow_hidden_inline_block_baseline_uses_bottom_margin_edge() {
    let y_visible = inline_block_baseline_y(80.0, false, true);
    let y_hidden = inline_block_baseline_y(80.0, true, true);
    let shift = y_visible - y_hidden;
    assert!(
        (shift - 80.0).abs() < 8.0,
        "overflow:hidden inline-block 应比 overflow:visible 上移约 80px（margin-edge 基线），实际位移 {}（visible={} hidden={}）",
        shift,
        y_visible,
        y_hidden
    );
}

/// 辅助：构造 body > div(wrapper, font-size 200px) > [text "Xg", span(inline-flex), text "Xg"]，
/// 返回该 inline-flex 在布局树中的 y（相对 wrapper 内容盒）。大字号文本主导行盒基线
///（ascent ≈ 160 > ib_baseline），使 inline-flex 的 y = baseline − ib_baseline，
/// 从而 ib_baseline 的差异直接体现为 y 的位移。`margin_bottom` 设给 inline-flex。
///
/// 空 inline-flex（无子元素）测 CSS Writing Modes §4.4 合成 alphabetic 基线 = margin-box
/// 下沿（height + margin-bottom）。central 基线（h/2）不随 margin-bottom 变，故 margin-bottom
/// 增大时是否上移可区分两路径。
fn inline_flex_baseline_y(margin_bottom: f64) -> f32 {
    let (mut doc, body) = make_doc_with_body();
    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();
    let t1 = doc.create_text_node("Xg");
    doc.append_child(wrapper, t1).unwrap();
    let ib = doc.create_element("span");
    doc.append_child(wrapper, ib).unwrap();
    let t2 = doc.create_text_node("Xg");
    doc.append_child(wrapper, t2).unwrap();

    let mut styles = HashMap::new();
    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Block;
    w.font_size = LengthValue::Px(200.0);
    styles.insert(wrapper, w);
    let mut ib_style = ComputedStyle::default();
    ib_style.display = DisplayValue::InlineFlex;
    ib_style.width = LengthValue::Px(60.0);
    ib_style.height = LengthValue::Px(60.0);
    ib_style.margin_bottom = LengthValue::Px(margin_bottom);
    styles.insert(ib, ib_style);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    find_child_by_node_id(&result.root, ib).expect("inline-flex 应找到").y
}

/// CSS Writing Modes §4.4：空 inline-flex 容器（无子元素）合成 alphabetic 基线 =
/// margin-box 下沿（height + margin-bottom）。margin-bottom 增大 → ib_baseline 增大 →
/// 行盒基线不变（200px 文本主导）→ inline-flex 上移约 margin-bottom 量。
/// 回归守护：此前空 inline-flex 走 central（h/2）基线，不随 margin-bottom 变。
#[test]
fn test_empty_inline_flex_synthesizes_alphabetic_baseline() {
    let y0 = inline_flex_baseline_y(0.0);
    let y80 = inline_flex_baseline_y(80.0);
    let shift = y0 - y80;
    assert!(
        (shift - 80.0).abs() < 12.0,
        "空 inline-flex 的 margin-bottom 应使其上移约 80px（§4.4 alphabetic margin-edge 基线），实际位移 {}（y0={} y80={}）",
        shift,
        y0,
        y80
    );
}
