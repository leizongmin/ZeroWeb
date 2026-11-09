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

// ===== R5427：本盒直系行内内容（文本行盒）溢出计入 extent =====
//
// 行内内容不是子 LayoutBox，R5365 的子盒遍历不可见；纯文本滚动容器的行溢出
// （WPT background-attachment-353：nbsp 长行 200×200 溢出 100×100 盒）此前恒
// 缩回 content 盒 → local 定位区回退 padding 盒、脚本滚动钳 0。

use zero_layout_engine::types::{InlineLayoutFragment, InlineLayoutLine};

/// 构造一段 stored 行内布局（content-box 系）。
fn stored_lines(specs: &[(f32, f32, f32, f32)]) -> Vec<InlineLayoutLine> {
    specs
        .iter()
        .map(|&(fx, fy, fw, fh)| InlineLayoutLine {
            y: fy,
            height: fh,
            baseline_y: fh,
            ascent: fh,
            descent: 0.0,
            fragments: vec![InlineLayoutFragment {
                x: fx,
                y: fy,
                width: fw,
                height: fh,
                font_size: fh,
                is_ahem: true,
                is_ahem_font: true,
                text: "nbsp-run".to_string(),
                source: None,
                letter_spacing: 0.0,
                node_id: None,
                baseline_y: fh,
            }],
        })
        .collect()
}

/// ★纯文本滚动容器：stored 行盒溢出（200×200 溢出 100×100 content 盒）计入
/// extent——旧行为（纯子盒遍历）无子盒时恒 (0,0,0,0)。353 驱动案形。
#[test]
fn r5427_text_only_scroller_inline_overflow_extent() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_x = OverflowClip::Hidden;
    target.overflow_y = OverflowClip::Hidden;
    target.content_width = 100.0;
    target.content_height = 100.0;
    target.inline_layout_width = 100.0;
    // 两行 nbsp 长行：行宽 200（不可断行）、行高 100 → 内容 span 200×200。
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 200.0, 100.0), (0.0, 100.0, 200.0, 100.0)]));

    let styles = HashMap::new();
    let (min_x, min_y, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (200.0, 200.0), "行盒溢出须计入 extent 跨度");
    assert_eq!((min_x, min_y), (0.0, 0.0), "正向流行盒 min 侧为原点");
}

/// stored IFC 宽度失配（后处理改宽，text.rs use_stored 同款校验）→ 不消费存储行。
#[test]
fn r5427_stale_inline_layout_not_consumed() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_x = OverflowClip::Scroll;
    target.content_width = 100.0;
    target.inline_layout_width = 150.0;
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 300.0, 100.0)]));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!(
        (ex_w, ex_h),
        (0.0, 0.0),
        "宽度失配的存储行不得入 extent（消费方回退重跑 IFC，几何不可信）"
    );
}

/// 非滚动容器（overflow:visible）不启用行内溢出臂——local 定位区等消费方在
/// 非滚动语境维持 padding 盒口径（R4354 门同源）。
#[test]
fn r5427_visible_overflow_not_gated_in() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.content_width = 100.0;
    target.inline_layout_width = 100.0;
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 200.0, 100.0)]));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (0.0, 0.0), "visible 容器不消费行内溢出臂");
}

/// multicol 容器跳过（列流存储行坐标已重映射，原样入 extent 会双重计入）。
#[test]
fn r5427_multicol_lines_skipped() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_y = OverflowClip::Scroll;
    target.is_multicol = true;
    target.content_width = 100.0;
    target.inline_layout_width = 100.0;
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 200.0, 100.0)]));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (0.0, 0.0), "multicol 存储行不入 extent");
}

/// 行内臂与子盒遍历取并集（同 acc 累积，互不覆盖）。
#[test]
fn r5427_inline_arm_unions_with_child_walk() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");
    let c_id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_x = OverflowClip::Hidden;
    target.content_width = 100.0;
    target.inline_layout_width = 100.0;
    // 行盒纵向溢出到 200；子盒横向溢出到 300。
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 100.0, 100.0), (0.0, 100.0, 100.0, 100.0)]));
    let child = make_box(Some(c_id), 100.0, 0.0, 200.0, 50.0);
    target.children.push(child);

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (300.0, 200.0), "行盒与子盒 span 取并集");
}

/// ★非 stored 容器（inline_layout = None，350 默认字体案形）：回退消费 layout 期
/// 聚合的标量 span `inline_overflow_extent`——旧行为（无此字段）恒 (0,0,0,0)。
#[test]
fn r5427_inline_overflow_extent_fallback_non_stored() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_y = OverflowClip::Hidden;
    target.content_width = 100.0;
    target.content_height = 100.0;
    // layout 期聚合：nbsp 长行 200 宽 / 行底 200（background-attachment-350 形）。
    target.inline_overflow_extent = Some((200.0, 200.0));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (200.0, 200.0), "非 stored 容器须消费标量 span 回退");
}

/// stored 行有效时优先取全 span，标量 span 不再叠加（350 形 + stored 同盒不双计）。
#[test]
fn r5427_stored_lines_take_precedence_over_fallback_span() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_x = OverflowClip::Hidden;
    target.content_width = 100.0;
    target.inline_layout_width = 100.0;
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 150.0, 100.0)]));
    // 陈旧/夸张的标量 span 不得穿透有效存储行。
    target.inline_overflow_extent = Some((999.0, 999.0));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (150.0, 100.0), "stored 行优先，标量 span 不叠加");
}

/// 标量 span 存在但宽度失配（stored 行陈旧）时仍回退标量——IFC 行几何失效
/// 不连带作废 layout 期聚合（同一次 IFC 运行的产物，表/multicol 改宽前聚合）。
#[test]
fn r5427_stale_stored_lines_fall_through_to_extent_span() {
    let mut doc = zero_dom::Document::new();
    let id = doc.create_element("div");

    let mut target = make_box(Some(id), 0.0, 0.0, 100.0, 100.0);
    target.overflow_x = OverflowClip::Scroll;
    target.content_width = 100.0;
    target.inline_layout_width = 150.0;
    target.inline_layout = Some(stored_lines(&[(0.0, 0.0, 300.0, 100.0)]));
    target.inline_overflow_extent = Some((200.0, 150.0));

    let styles = HashMap::new();
    let (_, _, ex_w, ex_h) = scrollable_overflow_extent(&target, Some(&styles));
    assert_eq!((ex_w, ex_h), (200.0, 150.0), "存储行失配时回退标量 span（不再弃整臂）");
}
