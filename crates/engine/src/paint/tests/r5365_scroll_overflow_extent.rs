//! R5365 scrollable overflow 传递性遍历与反向滚动轴常驻 pin。
//!
//! 覆盖三块语义：
//! 1. css-overflow-3 §2.2 scrollable overflow 传递性——static 裁剪盒（overflow ≠
//!    visible）不裁其 abspos 后代（CSS2 §11.1.1：CB 链不过 static 盒），这些后代
//!    仍计入外层滚动容器的 scrollable overflow（驱动：WPT
//!    position-absolute-under-non-containing-stacking-context，crbug.com/1273431）；
//! 2. CB 边界（positioned / contain:layout / transform）阻断逃逸；
//! 3. cssom-view-1 §11.2 反向滚动容器轴约定（rtl ^ row-reverse 内联轴、
//!    column-reverse 块轴）。

use std::collections::HashMap;

use zero_css_parser::values::{PositionValue, TransformValue};
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::property::types::{DirectionValue, DisplayValue, FlexDirectionValue};
use zero_style_system::{ComputedStyle, ContainComputedValue};

use super::super::painter::{reversed_scroll_axes, scrollable_overflow_extent};

/// 辅助函数：创建最小 LayoutBox（坐标为相对父元素内容区域）。
fn make_box(node_id: Option<NodeId>, x: f32, y: f32, width: f32, height: f32) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width,
        height,
        ..Default::default()
    }
}

/// static 裁剪盒（overflow:hidden）内的 abspos 后代逃逸裁剪、仍计入外层
/// 滚动容器 scrollable overflow（extent 250 → 350）。
#[test]
fn abspos_escapes_static_clipper() {
    let mut doc = zero_dom::Document::new();
    let scroller_id = doc.create_element("div");
    let a_id = doc.create_element("div");
    let green_id = doc.create_element("div");

    let mut scroller = make_box(Some(scroller_id), 0.0, 0.0, 200.0, 200.0);
    scroller.overflow_x = OverflowClip::Scroll;
    scroller.overflow_y = OverflowClip::Scroll;
    // A：margin 250 后 h=0，overflow:hidden 裁剪中间盒（static）
    let mut a = make_box(Some(a_id), 250.0, 250.0, 0.0, 0.0);
    a.overflow_x = OverflowClip::Hidden;
    a.overflow_y = OverflowClip::Hidden;
    // green：abspos 静态位置（CB = scroller，跳过 static 的 A）
    let mut green = make_box(Some(green_id), 0.0, 0.0, 100.0, 100.0);
    green.is_absolute = true;
    a.children.push(green);
    scroller.children.push(a);

    let styles = HashMap::new();
    let (min_x, min_y, ex_w, ex_h) = scrollable_overflow_extent(&scroller, Some(&styles));
    // green（250..350）计入 → 跨度 350；A 自身 (250,250,0,0) 亦计入
    assert_eq!(ex_h, 350.0, "abspos 后代须计入 scrollable overflow 高度跨度");
    assert_eq!(ex_w, 350.0);
    assert_eq!(min_x, 250.0);
    assert_eq!(min_y, 250.0);
}

/// positioned（relative）裁剪盒自身为其 abspos 后代的 CB → 后代不逃逸。
#[test]
fn positioned_clipper_blocks_escape() {
    let mut doc = zero_dom::Document::new();
    let scroller_id = doc.create_element("div");
    let a_id = doc.create_element("div");
    let green_id = doc.create_element("div");

    let mut scroller = make_box(Some(scroller_id), 0.0, 0.0, 200.0, 200.0);
    scroller.overflow_x = OverflowClip::Scroll;
    scroller.overflow_y = OverflowClip::Scroll;
    let mut a = make_box(Some(a_id), 250.0, 250.0, 0.0, 0.0);
    a.overflow_x = OverflowClip::Hidden;
    a.overflow_y = OverflowClip::Hidden;
    let mut green = make_box(Some(green_id), 0.0, 0.0, 100.0, 100.0);
    green.is_absolute = true;
    a.children.push(green);
    scroller.children.push(a);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Relative;
    styles.insert(a_id, style);

    let (_, _, _, ex_h) = scrollable_overflow_extent(&scroller, Some(&styles));
    assert_eq!(ex_h, 250.0, "positioned 裁剪盒的 abspos 后代不逃逸（CB = 裁剪盒自身）");
}

/// contain:layout 中间盒为 CB 边界，子树 overflow 不外传（019/020 域）。
#[test]
fn contain_layout_clipper_blocks_escape() {
    let mut doc = zero_dom::Document::new();
    let scroller_id = doc.create_element("div");
    let a_id = doc.create_element("div");
    let green_id = doc.create_element("div");

    let mut scroller = make_box(Some(scroller_id), 0.0, 0.0, 200.0, 200.0);
    scroller.overflow_x = OverflowClip::Scroll;
    scroller.overflow_y = OverflowClip::Scroll;
    let mut a = make_box(Some(a_id), 250.0, 250.0, 0.0, 0.0);
    a.overflow_x = OverflowClip::Hidden;
    a.overflow_y = OverflowClip::Hidden;
    let mut green = make_box(Some(green_id), 0.0, 0.0, 100.0, 100.0);
    green.is_absolute = true;
    a.children.push(green);
    scroller.children.push(a);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.contain = ContainComputedValue::Layout;
    styles.insert(a_id, style);

    let (_, _, _, ex_h) = scrollable_overflow_extent(&scroller, Some(&styles));
    assert_eq!(ex_h, 250.0, "contain:layout 边界阻断子树 overflow 外传");
}

/// transform 中间盒为 CB 边界，同 positioned/contain 阻断。
#[test]
fn transform_clipper_blocks_escape() {
    let mut doc = zero_dom::Document::new();
    let scroller_id = doc.create_element("div");
    let a_id = doc.create_element("div");
    let green_id = doc.create_element("div");

    let mut scroller = make_box(Some(scroller_id), 0.0, 0.0, 200.0, 200.0);
    scroller.overflow_x = OverflowClip::Scroll;
    scroller.overflow_y = OverflowClip::Scroll;
    let mut a = make_box(Some(a_id), 250.0, 250.0, 0.0, 0.0);
    a.overflow_x = OverflowClip::Hidden;
    a.overflow_y = OverflowClip::Hidden;
    let mut green = make_box(Some(green_id), 0.0, 0.0, 100.0, 100.0);
    green.is_absolute = true;
    a.children.push(green);
    scroller.children.push(a);

    let mut styles = HashMap::new();
    let mut style = ComputedStyle::default();
    style.transform = TransformValue::List(vec![zero_css_parser::values::TransformFunction::Translate(1.0, 1.0)]);
    styles.insert(a_id, style);

    let (_, _, _, ex_h) = scrollable_overflow_extent(&scroller, Some(&styles));
    assert_eq!(ex_h, 250.0, "transform 中间盒建立 CB，阻断子树逃逸");
}

/// abspos 盒自身不递归：其 in-flow 后代（CB = abspos 盒）不入外层滚动容器。
#[test]
fn abspos_box_subtree_not_recursed() {
    let mut doc = zero_dom::Document::new();
    let scroller_id = doc.create_element("div");
    let a_id = doc.create_element("div");
    let p_id = doc.create_element("div");
    let c_id = doc.create_element("div");

    let mut scroller = make_box(Some(scroller_id), 0.0, 0.0, 200.0, 200.0);
    scroller.overflow_x = OverflowClip::Scroll;
    scroller.overflow_y = OverflowClip::Scroll;
    let mut a = make_box(Some(a_id), 250.0, 250.0, 0.0, 0.0);
    a.overflow_x = OverflowClip::Hidden;
    a.overflow_y = OverflowClip::Hidden;
    // P：abspos 50 高（计入）；C：P 的 in-flow 子（CB = P，不计入）
    let mut p = make_box(Some(p_id), 0.0, 0.0, 100.0, 50.0);
    p.is_absolute = true;
    let c = make_box(Some(c_id), 10.0, 40.0, 30.0, 40.0);
    p.children.push(c);
    a.children.push(p);
    scroller.children.push(a);

    let styles = HashMap::new();
    let (_, _, _, ex_h) = scrollable_overflow_extent(&scroller, Some(&styles));
    assert_eq!(
        ex_h, 300.0,
        "abspos 盒自身计入（250+50），其子树不计入（若误入则为 340）"
    );
}

/// 反向滚动轴真值表：内联轴 = rtl ^ row-reverse（非 flex 容器 row_reverse 恒
/// false），块轴 = flex column-reverse（cssom-view-1 §11.2）。
#[test]
fn reversed_scroll_axes_truth_table() {
    let style_with = |display: DisplayValue, dir: DirectionValue, fd: FlexDirectionValue| {
        let mut s = ComputedStyle::default();
        s.display = display;
        s.direction = dir;
        s.flex_direction = fd;
        s
    };
    let cases: &[(&str, DisplayValue, DirectionValue, FlexDirectionValue, bool, bool)] = &[
        (
            "block-ltr",
            DisplayValue::Block,
            DirectionValue::Ltr,
            FlexDirectionValue::Row,
            false,
            false,
        ),
        (
            "block-rtl",
            DisplayValue::Block,
            DirectionValue::Rtl,
            FlexDirectionValue::Row,
            true,
            false,
        ),
        (
            "flex-row-rtl",
            DisplayValue::Flex,
            DirectionValue::Rtl,
            FlexDirectionValue::Row,
            true,
            false,
        ),
        (
            "flex-row-reverse-ltr",
            DisplayValue::Flex,
            DirectionValue::Ltr,
            FlexDirectionValue::RowReverse,
            true,
            false,
        ),
        (
            "flex-row-reverse-rtl",
            DisplayValue::Flex,
            DirectionValue::Rtl,
            FlexDirectionValue::RowReverse,
            false,
            false,
        ),
        (
            "flex-column-reverse-ltr",
            DisplayValue::Flex,
            DirectionValue::Ltr,
            FlexDirectionValue::ColumnReverse,
            false,
            true,
        ),
        (
            "flex-column-reverse-rtl",
            DisplayValue::Flex,
            DirectionValue::Rtl,
            FlexDirectionValue::ColumnReverse,
            true,
            true,
        ),
        (
            "flex-column-ltr",
            DisplayValue::Flex,
            DirectionValue::Ltr,
            FlexDirectionValue::Column,
            false,
            false,
        ),
    ];
    for (name, display, dir, fd, want_x, want_y) in cases {
        let (rev_x, rev_y) = reversed_scroll_axes(&style_with(display.clone(), dir.clone(), fd.clone()));
        assert_eq!(rev_x, *want_x, "{name}: 内联轴反向判定不符");
        assert_eq!(rev_y, *want_y, "{name}: 块轴反向判定不符");
    }
}
