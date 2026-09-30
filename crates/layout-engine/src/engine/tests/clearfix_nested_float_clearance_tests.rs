//! clearfix + 嵌套浮动 clearance 回归：容器内 **float** 子的 clear 不得置
//! `clearance_active`（R1323 语义收窄），R1392 嵌套浮动底边须与 active 追踪同帧
//!（content-rel 不再虚减 content_y_offset）。
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

/// R1392 帧修复：wrapper 带 padding-top:21 时嵌套浮动底边须按 content-rel 收集
/// （36），不得虚减 content_y_offset。
///
/// 判别力口径（PR #42 二次复核逐项变异实测）：撤 R1323 producer 收窄 → 本断言红
///（y=21，毒药经 padding 变体显形，与单测#1 同源）；**仅撤 R1392 帧参在本形状被
/// 掩蔽（仍 36、绿）**——本断言不作为帧参单独回退的守护。
///
/// FIXME(R1392 余项，已记账后续切片)：36 是**当前实现**钉值，不是规范终值。按
/// CSS2 §9.5.1/§9.5.2，float 在页面坐标系占 21..57（wrapper pt:21 + float 36h），
/// 流内 cleared 兄弟应落 **57**；当前 `nested_float_bottoms` 累加嵌套浮动底边漏加
/// 中间容器自身 border+padding 分量（本例 21），cleared 落 36 与 float 叠压 21px。
/// 旁证：同几何下 wrapper 改 overflow:hidden（BFC）后兄弟落 57（双审查实测；
/// 二次复核经余项修复探针直接实证 57）。baidu 实际形状中间 UL 无 border/padding，
/// 不受此余项影响。修复该余项时必须把本断言更新为 57 并删除本 FIXME。
#[test]
fn nested_float_bottom_respects_content_frame() {
    let (doc, styles, cleared) = build_float_clear_then_cleared(21.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let cleared_box = find_child_by_node_id(&result.root, cleared).expect("cleared found");
    // 撤 R1323 producer = 21（红）；当前实现（含已知余项）= 36；规范终值 = 57。
    assert!(
        (cleared_box.y - 36.0).abs() < 1.0,
        "嵌套浮动底边 content-rel 收集（R1392 帧修复钉值，规范终值 57 见 FIXME），实际 y={}",
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
