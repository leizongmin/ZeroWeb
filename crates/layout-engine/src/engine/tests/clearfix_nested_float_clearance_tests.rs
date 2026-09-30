//! clearfix + 嵌套浮动 clearance 回归：容器内 **float** 子的 clear 不得置
//! `clearance_active`（R1323 语义收窄），R1392 嵌套浮动底边须与 active 追踪同帧
//!（content-rel 不再虚减 content_y_offset，含中间容器自身 border/padding 分量——
//! R1392 余项）。
//!
//! 背景（baidu hotsearch 叠字，wrapper::after clearfix + li.odd float+clear）：
//! 1. UL 的 clearance_active 被 float 子 li.odd 的 clear 误置 → R1319 sibling-shift
//!    把 wrapper::after 伪元素的合法 clearance（落 float 底 114）当「泄漏」拉回
//!    UL 底（42），wrapper 高度塌回（135→84），后续兄弟整体叠压热榜行。
//! 2. wrapper 带 padding-top:21 时，R1392 嵌套浮动底边按旧 border-rel 约定多减
//!    content_y_offset（93 应 114），clearance 不足，::after 停在 float 底边之上。
//!
//! 规范：CSS2 §9.5.1（float 自身 clear 由 float 摆位解决，不产生流内 clearance）、
//! §9.5.2（clearance）、§10.6.3（auto 高度只计 in-flow 子）。

use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue, PositionValue};
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::BorderStyleValue;

/// 嵌套浮动 + 流内 clear 兄弟核心结构：
/// outer > [wrapper(non-BFC) > float, cleared(clear:both)]。
fn build_float_clear_then_cleared(
    pt: f64,
) -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(outer, wrapper).unwrap();
    let fl = doc.create_element("div");
    doc.append_child(wrapper, fl).unwrap();
    let cleared = doc.create_element("div");
    doc.append_child(outer, cleared).unwrap();

    let mut styles = HashMap::new();
    let mut o = ComputedStyle::default();
    o.display = DisplayValue::Block;
    o.width = LengthValue::Px(400.0);
    styles.insert(outer, o);

    // wrapper：非 BFC 普通块；pt 参数注入 padding-top（R1392 帧修复前提）。
    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Block;
    w.padding_top = LengthValue::Px(pt);
    styles.insert(wrapper, w);

    // float 子**带 clear**（毒药形状：R1323 旧实现据此误置 clearance_active）。
    let mut f = ComputedStyle::default();
    f.display = DisplayValue::Block;
    f.float = FloatValue::Left;
    f.clear = ClearValue::Both;
    f.width = LengthValue::Px(100.0);
    f.height = LengthValue::Px(36.0);
    styles.insert(fl, f);

    // 流内 cleared 兄弟（wrapper 的兄弟、outer 的子）。
    let mut c = ComputedStyle::default();
    c.display = DisplayValue::Block;
    c.clear = ClearValue::Both;
    c.height = LengthValue::Px(10.0);
    styles.insert(cleared, c);

    (doc, styles, cleared)
}

/// float 子的 clear 不得让 wrapper 被标 clearance_active——流内 cleared 兄弟的
/// clearance（落嵌套 float 底 36）不得被 R1319 sibling-shift 当「泄漏」撤销。
#[test]
fn float_child_clear_does_not_undo_sibling_clearance() {
    let (doc, styles, cleared) = build_float_clear_then_cleared(0.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cleared_box = find_child_by_node_id(&result.root, cleared).expect("cleared found");
    // float 底 = 36（wrapper 无 padding，content-rel 同帧）。塌缩回归形 = y 0 附近。
    assert!(
        cleared_box.y > 30.0,
        "float 子的 clear 不应触发 sibling-shift 撤销兄弟 clearance（应 y≈36），实际 y={}",
        cleared_box.y
    );
}

/// R1392 帧修复 + 余项修复：wrapper 带 padding-top:21 时嵌套浮动底边须按
/// content-rel 完整收集——外层 content_y_offset（slice7 帧参）与中间容器自身
/// border/padding 分量（余项）都不得虚减，cleared 兄弟落规范终值 57
///（CSS2 §9.5.1/§9.5.2：float 占 21..57，clear 须让位于其底边）。
///
/// 判别力口径（PR #42 二次复核逐项变异实测）：撤 R1323 producer 收窄 → 本断言红
///（y=21，毒药经 padding 变体显形，与单测#1 同源）；撤 R1392 帧参或撤余项中间
/// frame 分量 → 均红（y=36）。多臂矩阵见
/// `nested_float_bottom_includes_middle_frame_multi_arm`。
#[test]
fn nested_float_bottom_respects_content_frame() {
    let (doc, styles, cleared) = build_float_clear_then_cleared(21.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cleared_box = find_child_by_node_id(&result.root, cleared).expect("cleared found");
    // 撤 R1323 producer = 21（红）；撤 R1392 任一分量 = 36（红）；规范终值 = 57。
    assert!(
        (cleared_box.y - 57.0).abs() < 1.0,
        "嵌套浮动底边 content-rel 收集须含中间容器 frame（pt:21 + float 36 = 57），实际 y={}",
        cleared_box.y
    );
}

/// R1392 余项多臂矩阵：外层 padding-top {0,30} × 中间容器 frame {无, border-top:21,
/// padding-top:21}，证明中间 frame 分量修复不是只对原始样例（pt:21）有效，也不依赖
/// 外层是否带 padding。期望：中间无 frame → 底边 = float 高 36（修复加 0，不回归）；
/// 中间有任一 frame 分量 → 底边 = 21+36 = 57。
///
/// 本组浮子**不带 clear**（隔离 R1392：clearance_active 不被置位，R1323 由上面
/// 两个毒药形状测试单独守护）。
#[test]
fn nested_float_bottom_includes_middle_frame_multi_arm() {
    // 臂 = (outer_pt, middle_border_top, middle_padding_top, 期望 cleared y)。
    let arms = [
        (0.0, 0.0, 0.0, 36.0),
        (0.0, 21.0, 0.0, 57.0),
        (0.0, 0.0, 21.0, 57.0),
        (30.0, 0.0, 0.0, 36.0),
        (30.0, 0.0, 21.0, 57.0),
    ];
    for (outer_pt, middle_bt, middle_pt, expect) in arms {
        let (mut doc, body) = make_doc_with_body();
        let outer = doc.create_element("div");
        doc.append_child(body, outer).unwrap();
        let wrapper = doc.create_element("div");
        doc.append_child(outer, wrapper).unwrap();
        let fl = doc.create_element("div");
        doc.append_child(wrapper, fl).unwrap();
        let cleared = doc.create_element("div");
        doc.append_child(outer, cleared).unwrap();

        let mut styles = HashMap::new();
        let mut o = ComputedStyle::default();
        o.display = DisplayValue::Block;
        o.width = LengthValue::Px(400.0);
        o.padding_top = LengthValue::Px(outer_pt);
        styles.insert(outer, o);

        let mut w = ComputedStyle::default();
        w.display = DisplayValue::Block;
        w.border_top_width = LengthValue::Px(middle_bt);
        w.border_top_style = BorderStyleValue::Solid;
        w.padding_top = LengthValue::Px(middle_pt);
        styles.insert(wrapper, w);

        let mut f = ComputedStyle::default();
        f.display = DisplayValue::Block;
        f.float = FloatValue::Left;
        f.width = LengthValue::Px(100.0);
        f.height = LengthValue::Px(36.0);
        styles.insert(fl, f);

        let mut c = ComputedStyle::default();
        c.display = DisplayValue::Block;
        c.clear = ClearValue::Both;
        c.height = LengthValue::Px(10.0);
        styles.insert(cleared, c);

        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let cleared_box = find_child_by_node_id(&result.root, cleared)
            .unwrap_or_else(|| panic!("{outer_pt}/{middle_bt}/{middle_pt}: cleared found"));
        assert!(
            (cleared_box.y - expect).abs() < 1.0,
            "outer_pt={outer_pt} middle_bt={middle_bt} middle_pt={middle_pt}: 嵌套浮动底边应 \
             {expect}（float 36 + 中间 frame {sum}），实际 y={y}",
            sum = middle_bt + middle_pt,
            y = cleared_box.y
        );
    }
}

/// R1392 余项两级下降：中间容器嵌套两层非 BFC frame（wrapper pt:8 > inner pt:8 >
/// float 36h），递归每层下降都须累加该层 border/padding 分量，底边 = 8+8+36 = 52。
#[test]
fn nested_float_bottom_descends_through_two_middle_frames() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(outer, wrapper).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(wrapper, inner).unwrap();
    let fl = doc.create_element("div");
    doc.append_child(inner, fl).unwrap();
    let cleared = doc.create_element("div");
    doc.append_child(outer, cleared).unwrap();

    let mut styles = HashMap::new();
    let mut o = ComputedStyle::default();
    o.display = DisplayValue::Block;
    o.width = LengthValue::Px(400.0);
    styles.insert(outer, o);

    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Block;
    w.padding_top = LengthValue::Px(8.0);
    styles.insert(wrapper, w);

    let mut i = ComputedStyle::default();
    i.display = DisplayValue::Block;
    i.padding_top = LengthValue::Px(8.0);
    styles.insert(inner, i);

    let mut f = ComputedStyle::default();
    f.display = DisplayValue::Block;
    f.float = FloatValue::Left;
    f.width = LengthValue::Px(100.0);
    f.height = LengthValue::Px(36.0);
    styles.insert(fl, f);

    let mut c = ComputedStyle::default();
    c.display = DisplayValue::Block;
    c.clear = ClearValue::Both;
    c.height = LengthValue::Px(10.0);
    styles.insert(cleared, c);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cleared_box = find_child_by_node_id(&result.root, cleared).expect("cleared found");
    assert!(
        (cleared_box.y - 52.0).abs() < 1.0,
        "两级中间 frame（8+8）+ float 36 应落 52，实际 y={}",
        cleared_box.y
    );
}

/// R1323 收窄排除臂钉住（PR #42 双审查发现 3）：wrapper 内 float 子与**不应计入
/// `has_clear_child` producer 的子**（absolute/fixed 定位、非 block-level）同带
/// clear:both 时，wrapper 不得被标 clearance_active，流内 cleared 兄弟的
/// clearance（落 float 底 36）不得被 R1319 sibling-shift 撤销。
///
/// 判别力口径（PR #42 二次复核逐臂变异实测）：absolute/fixed 臂几何显形——臂被
/// 误计入 producer → sibling-shift 把 cleared 塌回 0，`y > 30` 变红；inline-block
/// 臂几何无症状（wrapper 被浮体撑高后 sibling-shift 拉回目标恰等于 clearance
/// 目标，y 恒 36）——以 `clearance_active` flag 直钉判别（臂被误计入 → flag=true
/// 变红）。
#[test]
fn excluded_child_clear_does_not_mark_clearance_active() {
    for arm in ["absolute", "fixed", "inline-block"] {
        let (mut doc, body) = make_doc_with_body();
        let outer = doc.create_element("div");
        doc.append_child(body, outer).unwrap();
        let wrapper = doc.create_element("div");
        doc.append_child(outer, wrapper).unwrap();
        let fl = doc.create_element("div");
        doc.append_child(wrapper, fl).unwrap();
        let noise = doc.create_element("div");
        doc.append_child(wrapper, noise).unwrap();
        let cleared = doc.create_element("div");
        doc.append_child(outer, cleared).unwrap();

        let mut styles = HashMap::new();
        let mut o = ComputedStyle::default();
        o.display = DisplayValue::Block;
        o.width = LengthValue::Px(400.0);
        styles.insert(outer, o);

        let mut w = ComputedStyle::default();
        w.display = DisplayValue::Block;
        styles.insert(wrapper, w);

        let mut f = ComputedStyle::default();
        f.display = DisplayValue::Block;
        f.float = FloatValue::Left;
        f.clear = ClearValue::Both;
        f.width = LengthValue::Px(100.0);
        f.height = LengthValue::Px(36.0);
        styles.insert(fl, f);

        // 排除臂子：带 clear:both，但按 R1323 不计入 has_clear_child。
        let mut n = ComputedStyle::default();
        n.display = DisplayValue::Block;
        n.clear = ClearValue::Both;
        match arm {
            "absolute" => n.position = PositionValue::Absolute,
            "fixed" => n.position = PositionValue::Fixed,
            _ => n.display = DisplayValue::InlineBlock,
        }
        styles.insert(noise, n);

        let mut c = ComputedStyle::default();
        c.display = DisplayValue::Block;
        c.clear = ClearValue::Both;
        c.height = LengthValue::Px(10.0);
        styles.insert(cleared, c);

        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // flag 直钉（inline-block 臂唯一判别通道，见函数注释判别力口径）。
        let wrapper_box =
            find_child_by_node_id(&result.root, wrapper).unwrap_or_else(|| panic!("{arm}: wrapper found"));
        assert!(
            !wrapper_box.clearance_active,
            "{arm} 子的 clear 不应把 wrapper 标为 clearance_active"
        );

        let cleared_box =
            find_child_by_node_id(&result.root, cleared).unwrap_or_else(|| panic!("{arm}: cleared found"));
        assert!(
            cleared_box.y > 30.0,
            "{arm} 子的 clear 不应触发 sibling-shift 撤销兄弟 clearance（应 y≈36），实际 y={}",
            cleared_box.y
        );
    }
}
