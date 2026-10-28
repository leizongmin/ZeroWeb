//! R5315（css-position-3 §static-position）：inline-block OOF 行内流静态位锚。
//!
//! 案 vertical-align-baseline-004a/005a（✗5.70% ×2）：abspos ruler img（UA replaced
//! 组，ZW 判 InlineBlock）被 R5166 锚发射旧门（仅 `display:inline`）排除 → 无锚，
//! taffy 给块游标静态位（容器内容左缘）→ img 盖住前驱 atomic inline-block 左半
//!（ref：img 在 ib 之后 x=ib 宽处）。修复 = 锚发射门扩至 InlineBlock（collect_items）。
//! 断言修复前红：旧门下无锚，OOF 落 x=0。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// author `display:inline-block` 的 abspos 盒随前驱 atomic inline-block 锚在行内
/// 光标位（ib 推进值 50px），非容器内容左缘（修复前 0）。
#[test]
fn r5315_inline_block_oof_anchor_follows_atomic_predecessor() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 400px;">
<span id=ib style="display: inline-block; width: 50px; height: 20px"></span><span id=oof style="position: absolute; display: inline-block; width: 10px; height: 10px">A</span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ib = doc.get_element_by_id("ib").expect("ib");
    let oof = doc.get_element_by_id("oof").expect("oof");
    let (ox, oy) = find_box(&result.root, oof).expect("oof 盒应存在");
    let _ = find_box(&result.root, ib);
    assert!(
        (ox - 50.0).abs() < 0.5,
        "inline-block OOF 应锚在前驱 atomic 推进后的行内位 50，实际 ({ox},{oy})"
    );
}

/// UA 默认分类路径（无 author display）：abspos img（replaced 组 → InlineBlock）
/// 在 inline-block 前驱后锚 x = 前驱宽——004a 形态直钉（definite top，仅 x 走锚）。
#[test]
fn r5315_abspos_img_after_inline_block_gets_inline_anchor() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 400px;">
<span id=ib style="display: inline-block; width: 60px; height: 20px"></span><img id=r style="position: absolute; top: 10px; width: 20px; height: 20px">
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let r = doc.get_element_by_id("r").expect("r");
    let (rx, _ry) = find_box(&result.root, r).expect("img 盒应存在");
    assert!(
        (rx - 60.0).abs() < 0.5,
        "abspos img（InlineBlock 默认分类）应锚在 ib 后 x=60，实际 x={rx}"
    );
}

/// 锚消费累计的流基修正（abspos-inline-008 形态）：relative inline wrapper
///（display:inline，inset 由 taffy 单次施加）内 inline-block OOF——锚 x 为断行期未
/// 偏移坐标，累计扣回 wrapper 已施加 inset 后写入；合成位（wrapper 位移后原点 +
/// child.x）= 偏移前流位 + wrapper 相对偏移。修复前按位移后 x 累计 → inset 反号
/// 二次施加（合成位右偏 |inset|）。
#[test]
fn r5315_anchor_accum_skips_relative_wrapper_inset() {
    // body 原点即页面 0：div(+100) content 原点 100，span(−100) 位移后合成原点 0，
    // OOF 静态位 = span 流内 0 → 合成 x=0（ref：绿盒贴页左缘）。
    let html = r#"<html><body style="margin:0">
<div id=host style="position: relative; left: 100px; width: 100px;">
<span id=wrap style="position: relative; left: -100px;">
<span id=oof style="position: absolute; display: inline-block; width: 100px; height: 20px">G</span>
</span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let host = doc.get_element_by_id("host").expect("host");
    let wrap = doc.get_element_by_id("wrap").expect("wrap");
    let oof = doc.get_element_by_id("oof").expect("oof");
    let (hx, _) = find_box(&result.root, host).expect("host 盒应存在");
    let (wx, _) = find_box(&result.root, wrap).expect("wrap 盒应存在");
    let (ox, _oy) = find_box(&result.root, oof).expect("oof 盒应存在");
    // 三段合成 = 页面流位（body content 原点为 0）：div(+100) + span(−100) + oof.x
    // 应为 0（ref 绿盒贴页左缘）。修复前 oof.x=+100（inset 反号二次施加）→ 合成 100。
    let abs_x = hx + wx + ox;
    assert!(
        abs_x.abs() < 0.5,
        "relative wrapper 内 OOF 合成位应 = 流位 0（host +100 + wrap −100 + oof 0），实际 host.x={hx} wrap.x={wx} oof.x={ox} 合成={abs_x}"
    );
}
