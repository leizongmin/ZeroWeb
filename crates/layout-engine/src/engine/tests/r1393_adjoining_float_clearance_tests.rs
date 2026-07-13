//! R1393 回归测试：adjoining-float clearance——clear 的 margin 经非 BFC wrapper 与嵌套
//! 浮动 adjoining 时（§8.3.1+§9.5.2），clearance 须把 clear 定到浮动下方并吸收 margin。
//!
//! 背景（adjoining-float-before-clearance）：outer > [wrapper(non-BFC) > float(left)],
//! clear(left, margin-top:400, height:50)。float 嵌在 wrapper 内，outer 无直接 float 子
//! → 旧 `has_active_float_context=false`，clear 走 R1389 else 分支看不到嵌套浮动 →
//! clear 的 400px margin-top 被当实空间涂，outer 高度被撑到 ~450px 露红。
//!
//! 修复（三层）：
//! 1. `has_active_float_context` 扩展：容器有 clear 子 + 非 BFC 后代嵌套 float 时也走主
//!    clearance 路径（窄 gate，仅此签名）。
//! 2. `nested_float_bottoms`：递归收集非 BFC 后代浮动底边并入 `active_*_float_bottom`，
//!    使 clear 看到嵌套浮动（R1392）。
//! 3. adjoining 吸收：clear 的 hypothetical_y（含 margin）> clear_bottom 但 clear 清除的
//!    是嵌套浮动（nested_for_side 决定 clear_bottom）时，把 clear 定到 clear_bottom 并吸收
//!    margin + 标记 ran_adjoining_clearance 跑 containment 收缩容器高度。
//!
//! 期望：clear 落在 float 下方（y ≈ float 底），margin-top 被吸收，outer 高度 ≈ float+clear。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 构造 adjoining-float-before-clearance 核心结构：outer > [wrapper > float, clear]。
/// 返回 (doc, styles, outer_id, clear_id)。
fn build_adjoining_float() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let wrapper = doc.create_element("div");
    doc.append_child(outer, wrapper).unwrap();
    let float_div = doc.create_element("div");
    doc.append_child(wrapper, float_div).unwrap();
    let clear_div = doc.create_element("div");
    doc.append_child(outer, clear_div).unwrap();

    let mut styles = HashMap::new();
    let mut o = ComputedStyle::default();
    o.display = DisplayValue::Block;
    o.width = LengthValue::Px(100.0);
    styles.insert(outer, o);

    // wrapper：普通 block（非 BFC），无 border/padding → margin 可与子/兄弟折叠。
    let mut w = ComputedStyle::default();
    w.display = DisplayValue::Block;
    styles.insert(wrapper, w);

    let mut f = ComputedStyle::default();
    f.display = DisplayValue::Block;
    f.float = FloatValue::Left;
    f.width = LengthValue::Px(100.0);
    f.height = LengthValue::Px(50.0);
    styles.insert(float_div, f);

    // clear：margin-top 极大（400），clear:left—— adjoining 须吸收 margin。
    let mut c = ComputedStyle::default();
    c.display = DisplayValue::Block;
    c.clear = zero_css_parser::values::ClearValue::Left;
    c.margin_top = LengthValue::Px(400.0);
    c.height = LengthValue::Px(50.0);
    styles.insert(clear_div, c);

    (doc, styles, outer, clear_div)
}

/// R1393：clear 的 margin-top:400 经 wrapper 与嵌套 float adjoining → clearance 吸收 margin，
/// clear 落在 float 下方（y ≈ 50），而非 margin 推到的 450。
#[test]
fn r1393_adjoining_float_absorbs_clear_margin() {
    let (doc, styles, _outer, clear_div) = build_adjoining_float();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let clear_box = find_child_by_node_id(&result.root, clear_div).expect("clear found");
    // float 底 = 50。adjoining 吸收后 clear 底贴 float（容差含 body/html 偏移）。
    // clear_box.y 是相对 outer content；outer 无 border/padding，故 clear.y ≈ 50。
    assert!(
        clear_box.y < 80.0,
        "adjoining-float: clear should be just below the float (y≈50), not pushed down by margin-top:400 (got y={})",
        clear_box.y
    );
}
