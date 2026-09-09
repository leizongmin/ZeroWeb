//! R2248：CSS `margin-trim`（css-box-4 §margin-trim）块容器块轴裁剪落地测试。
//!
//! `margin-trim` 在块容器声明 `block` / `block-start` / `block-end` 时，归零首子
//! block-start（margin-top）与/或末子 block-end（margin-bottom）。build_subtree 在
//! taffy 布局前修改 taffy_style.margin（trim 到 0 即移除参与折叠的 margin，对 collapsing /
//! non-collapsing 案均正确）。bounded scope：仅水平书写模式；inline 轴 trim 对块级子无效
//! （block-container-inline-001 实证）；自折叠/嵌套深案 defer。kill-switch `ZW_MARGIN_TRIM=0`。
//!
//! driving：css/css-box/margin-trim/block-container-block-001（margin-trim:block 单子，
//! 首末 margin 均裁剪）、block-start-001（仅首）、block-end-001（仅末）、inline-001
//! （margin-trim:inline 不裁剪块级子 inline 边距）。
use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 按 id 属性递归查找 LayoutBox。
fn find_id<'a>(r: &'a LayoutBox, d: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    let hit = r.node_id.is_some_and(|nid| {
        d.get(nid).is_some_and(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => e.get_attribute("id").is_some_and(|v| v == id),
            _ => false,
        })
    });
    if hit {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_id(c, d, id) {
            return Some(b);
        }
    }
    None
}

/// 容器 `overflow:hidden` 建立 BFC：单子无兄弟折叠、子 margin 不向祖先穿透，
/// 故未裁剪 margin 恒为声明值（50px），裁剪 margin 恒为 0。容差 0.5 抗 float 抖动。
fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// 构建布局并返回 root（统一脚手架）。
fn compute(html: &str) -> (zero_dom::Document, LayoutEngine, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    (doc, eng, Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone()))
}

/// margin-trim:block 单子 → 首子 block-start（margin-top）与末子 block-end（margin-bottom）
/// 均裁剪为 0（单子同时为首末子）。driving: block-container-block-001。
#[test]
fn test_margin_trim_block_single_child_trims_both() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="margin-trim:block; overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find child #a");
    assert!(
        approx(a.margin_top, 0.0),
        "first child block-start trimmed; got {}",
        a.margin_top
    );
    assert!(
        approx(a.margin_bottom, 0.0),
        "last child block-end trimmed; got {}",
        a.margin_bottom
    );
}

/// margin-trim:block-start 单子 → 仅首子 block-start（margin-top）裁剪；block-end 保留。
/// driving: block-container-block-start-001。
#[test]
fn test_margin_trim_block_start_trims_only_top() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="margin-trim:block-start; overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find child #a");
    assert!(
        approx(a.margin_top, 0.0),
        "block-start trims first child margin-top; got {}",
        a.margin_top
    );
    assert!(
        approx(a.margin_bottom, 50.0),
        "block-start must NOT trim block-end; got {}",
        a.margin_bottom
    );
}

/// margin-trim:block-end 单子 → 仅末子 block-end（margin-bottom）裁剪；block-start 保留。
/// driving: block-container-block-end-001。
#[test]
fn test_margin_trim_block_end_trims_only_bottom() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="margin-trim:block-end; overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find child #a");
    assert!(
        approx(a.margin_top, 50.0),
        "block-end must NOT trim block-start; got {}",
        a.margin_top
    );
    assert!(
        approx(a.margin_bottom, 0.0),
        "block-end trims last child margin-bottom; got {}",
        a.margin_bottom
    );
}

/// margin-trim:inline → 不裁剪块级子的边距（inline 轴 trim 仅作用于行内内容，IFC defer）。
/// driving: block-container-inline-001（assert "block should not trim inline margins for
/// block-level children"；块级子的 block-axis margin 同理不受 inline trim 影响）。
#[test]
fn test_margin_trim_inline_does_not_trim_block_child() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="margin-trim:inline; overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find child #a");
    assert!(
        approx(a.margin_top, 50.0),
        "inline trim must not affect block child margin-top; got {}",
        a.margin_top
    );
    assert!(
        approx(a.margin_bottom, 50.0),
        "inline trim must not affect block child margin-bottom; got {}",
        a.margin_bottom
    );
}

/// 无 margin-trim → 子 margin 完整保留（回归守卫，证裁剪逻辑仅在声明 margin-trim 时生效）。
#[test]
fn test_no_margin_trim_preserves_margins() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find child #a");
    assert!(
        approx(a.margin_top, 50.0),
        "no trim: margin-top preserved; got {}",
        a.margin_top
    );
    assert!(
        approx(a.margin_bottom, 50.0),
        "no trim: margin-bottom preserved; got {}",
        a.margin_bottom
    );
}

/// margin-trim:block 多子 → 仅首子 block-start 与末子 block-end 裁剪（首末子检测在多子下正确）。
/// 裁剪边恒为 0（trim 到 0 移除参与折叠的 margin，与折叠无关）；中间子不在首末位故不裁剪。
#[test]
fn test_margin_trim_block_multi_child_first_last_only() {
    let html = r#"<html><body style="margin:0">
<div id="ctr" style="margin-trim:block; overflow:hidden; width:100px">
  <div id="a" style="margin:50px 0; height:20px"></div>
  <div id="b" style="margin:50px 0; height:20px"></div>
  <div id="c" style="margin:50px 0; height:20px"></div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let a = find_id(&root, &doc, "a").expect("find first child #a");
    let c = find_id(&root, &doc, "c").expect("find last child #c");
    // 首子 block-start 裁剪（恒 0，折叠无关）。
    assert!(
        approx(a.margin_top, 0.0),
        "first child block-start trimmed; got {}",
        a.margin_top
    );
    // 末子 block-end 裁剪（恒 0，折叠无关）。
    assert!(
        approx(c.margin_bottom, 0.0),
        "last child block-end trimmed; got {}",
        c.margin_bottom
    );
}

/// R4191（css-box-4 §margin-trim × CSS2 §8.3.1）：height:auto 空元素自折叠臂——
/// 末子为无 border/padding 的空 div（outline 不阻折叠）时，其 mt/mb 自折叠穿透，
/// block-end trim 须两侧同归零。driving: block-container-block-end-self-collapsing-
/// and-border（旧实现仅 height:Px(0) 臂命中，auto 空元素漏裁 → 容器被撑高）。
#[test]
fn test_margin_trim_block_end_auto_empty_self_collapsing() {
    let html = r#"<html><body style="margin:0">
<div id="outer" style="width:100px; height:100px; background:red; overflow:hidden">
  <div id="ctr" style="margin-trim:block-end; border:solid green; background:red">
    <div id="a" style="height:94px; background:green;"></div>
    <div id="b" style="margin-top:222px; outline:solid green;"></div>
  </div>
</div>
</body></html>"#;
    let (doc, _eng, root) = compute(html);
    let ctr = find_id(&root, &doc, "ctr").expect("ctr box");
    // 末子空元素自折叠：mt/mb 222 均被 block-end trim 连同归零 → 容器不被 222px 撑高。
    // 旧实现漏裁 → 容器高 ≥ 94+2+222（穿透 mb）。精确终值含折叠交互（96..100 带），
    // 断言锚定「未撑高」即可区分新旧实现。
    assert!(
        ctr.height < 150.0,
        "R4191: 自折叠末子穿透 margin 应被 trim 归零（容器不被 222px 撑高），实际 {}",
        ctr.height
    );
    assert!(
        approx(ctr.height, 100.0),
        "R4191: 容器高应与 reftest 一致（~100，含折叠交互），实际 {}",
        ctr.height
    );
}
