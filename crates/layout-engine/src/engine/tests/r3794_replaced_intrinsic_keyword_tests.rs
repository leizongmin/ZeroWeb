//! R3794（css-sizing-4 §4.1 + CSS2 §10.3.2）：replaced 元素 intrinsic 尺寸关键字
//! （min-content/max-content/fit-content）+ definite 对侧 → transferred size。
//!
//! converter 把 intrinsic 关键字映射 `length(0)`（converter:526），旧
//! `apply_replaced_element_sizing` 的 `width_auto` 只认 `Auto` → 关键字落入「两侧都显式」
//! 分支不干预 → img 宽塌缩 0（intrinsic-size-020..025：img `height:100px; width:min-content`
//! 固有 1:1 应 100×100，旧渲 0×100；父 `width:min/max-content` 收缩测 0 回退满宽）。
//!
//! 同族：`box_content_max_width` 的 own_ar transferred gate 旧仅认 `width:Auto`——
//! `width:min-content; height:100px; aspect-ratio:1/1` 子测 0（intrinsic-size-014/015）；
//! 百分比 height 叶盒（`height:100%` 链）CSS 解析 None → 旧测 0（intrinsic-size-006/008）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn compute_with_intrinsic(html: &str, iw: f32, ih: f32) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let imgs = doc.get_elements_by_tag_name("img");
    let mut intrinsic = std::collections::HashMap::new();
    for id in imgs {
        intrinsic.insert(id, (iw, ih));
    }
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, intrinsic, std::collections::HashMap::new());
    (doc, result)
}

/// intrinsic-size-020..025 族：200×200 固有（1:1）绿方块。
fn compute_with_1x1_img(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    compute_with_intrinsic(html, 200.0, 200.0)
}

/// intrinsic-size-020 驱动案：img definite height + width:min-content 应按固有比
/// transferred（100×100 绿方块），min-content 父收缩到 100。
#[test]
fn r3794_img_min_content_width_transfers_from_definite_height() {
    let html = r#"<html><body style="margin:0">
<div style="width: min-content; background: red;">
  <img src="x.png" style="height: 100px; width: min-content">
</div>
</body></html>"#;
    let (doc, result) = compute_with_1x1_img(html);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let (w, h) = find_box(&result.root, img).expect("img box");
    assert!((h - 100.0).abs() < 1.0, "R3794: img 高应 100（CSS 显式），实际 {h}");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794: img width:min-content + 固有 1:1 + height:100 → transferred 100，实际 {w}（旧塌缩 0）"
    );
}

/// intrinsic-size-023 对称：width:max-content 父内 img 同样 transferred（非 Auto 关键字）。
#[test]
fn r3794_img_max_content_width_transfers_in_max_content_parent() {
    let html = r#"<html><body style="margin:0">
<div style="width: max-content; background: red;">
  <img src="x.png" style="height: 100px; width: max-content">
</div>
</body></html>"#;
    let (doc, result) = compute_with_1x1_img(html);
    let outer = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("outer div");
    let (_, w) = find_box(&result.root, outer).expect("outer box");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794: max-content 父应按 img transferred 100 收缩，实际 {w}（旧测 0 回退满宽 784）"
    );
}

/// intrinsic-size-014：`width:min-content; height:100px; aspect-ratio:1/1` 子对
/// min-content 父贡献 100px——own_ar gate 扩 intrinsic 关键字。
#[test]
fn r3794_aspect_ratio_block_min_content_width_transfers() {
    let html = r#"<html><body style="margin:0">
<div style="width: min-content; background: green;">
  <div style="width: min-content; height: 100px; aspect-ratio: 1/1;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("outer div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (_, w) = find_box(&result.root, outer).expect("outer box");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794: width:min-content + aspect-ratio 子应 transferred 100 使父收缩，实际 {w}（旧 gate 仅认 Auto → 子测 0 → 父满宽）"
    );
}

/// intrinsic-size-006：`height:100%` 链（definite 父）+ aspect-ratio 叶盒——百分比 height
/// 经第一趟 taffy 解析后作 transferred main 来源。
#[test]
fn r3794_percent_height_aspect_ratio_leaf_transfers_via_resolved_height() {
    let html = r#"<html><body style="margin:0">
<div style="width: min-content; height: 100px; background: green;">
  <div style="height: 100%;">
    <div style="height: 100%; aspect-ratio: 1/1;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let outer = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("outer div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (_, w) = find_box(&result.root, outer).expect("outer box");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794: height:100% + aspect-ratio 叶盒经解析高 transferred 100 使 min-content 父收缩，实际 {w}（旧 CSS 百分比解析 None → 测 0 → 父满宽）"
    );
}

/// R3794 守卫（replaced-elements-min-height-20）：min 是地板只抬不降——固有 50×25 SVG
/// + min-height:20 保持固有（floor no-op），不得缩到 20×40。
#[test]
fn r3794_min_transfer_is_floor_only_never_lowers() {
    let html = r#"<html><body style="margin:0">
<img src="x.svg" style="min-height: 20px">
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut intrinsic = std::collections::HashMap::new();
    intrinsic.insert(img, (50.0_f32, 25.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, intrinsic, std::collections::HashMap::new());
    let (w, h) = find_box(&result.root, img).expect("img box");
    assert!(
        (w - 50.0).abs() < 1.0 && (h - 25.0).abs() < 1.0,
        "R3794: 固有 50×25 + min-height:20（floor no-op）应保持 50×25，实际 {w}×{h}"
    );
}

/// R3794 守卫（replaced-aspect-ratio-intrinsic-size-001）：1×1 固有 + min-height:100 +
/// width:max-content → 100×100（floor 抬升 + transferred）。
#[test]
fn r3794_min_height_floor_raises_intrinsic_with_transferred_width() {
    let html = r#"<html><body style="margin:0">
<img src="x.png" style="min-height: 100px; width: max-content">
</body></html>"#;
    let (doc, result) = compute_with_intrinsic(html, 1.0, 1.0);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let (w, h) = find_box(&result.root, img).expect("img box");
    assert!(
        (w - 100.0).abs() < 1.5 && (h - 100.0).abs() < 1.5,
        "R3794: 1×1 固有 + min-height:100 + width:max-content 应 100×100，实际 {w}×{h}"
    );
}

/// R3794 守卫（flex-aspect-ratio-img-row-007）：flex item 的 min-width 是 flex base floor
/// 非 base 本身——min transfer 不适用于 flex item（否则 flex:1 grow 越过 floor）。
#[test]
fn r3794_flex_item_min_width_not_used_as_definite_base() {
    let html = r#"<html><body style="margin:0">
<div style="display: flex; width:200px;">
  <img src="x.png" style="min-width: 100px; flex: 1 0 auto;">
  <div style="flex: 1 0 1px;"></div>
</div>
</body></html>"#;
    let (doc, result) = compute_with_intrinsic(html, 1.0, 1.0);
    let img = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let (w, _h) = find_box(&result.root, img).expect("img box");
    assert!(
        (w - 100.0).abs() < 3.0,
        "R3794: flex item min-width:100 + flex:1 → base 不受 min 影响，最终 ~100（min floor），实际 {w}（误 definite base → 150）"
    );
}

/// R3794c（flex-aspect-ratio-013 驱动案）：column inline-flex（height:100px）内 item
/// aspect-ratio:1/1 + height:50px + flex:1——flex 主轴拉伸后 main=100，transferred cross
/// 应 100（item 100×100、容器 shrink-to-fit 100×100 绿方块）。旧 item cross 保持 taffy
/// 第一趟拉伸伪影 784、容器 50。
#[test]
fn r3794c_column_flex_stretched_main_transfers_to_cross() {
    let html = r#"<html><body style="margin:0">
<div style="display: inline-flex; flex-direction: column; flex-wrap: wrap; height: 100px;">
  <div style="background: green; aspect-ratio: 1/1; min-height: 0; height: 50px; flex: 1;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let container = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("container");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, container).expect("container box");
    assert!((h - 100.0).abs() < 1.0, "R3794c: 容器高应 100（CSS 显式），实际 {h}");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794c: item 主轴拉伸 100 × ratio 1 → transferred cross 100，容器 shrink-to-fit 100，实际 {w}（旧 item 784 / 容器 50）"
    );
}

/// R3794c 守卫（flex-item-transferred-sizes-padding-border-sizing）：border-box item
/// （padL/R 25 + min-height:100 + ratio 1）transferred cross = main × ratio（border-box），
/// 不加 frame 双计 padding（150 应 100）。
#[test]
fn r3794c_border_box_transfer_does_not_double_count_padding() {
    // 注：layout-engine 单测不经 WPT harness 的 merge_page_css，`<style>` 块不参与级联——
    // 样式一律用 style="" 属性表达（同 r3792 契约）。
    let html = r#"<html><body style="margin:0">
<div style="display: inline-flex; flex-direction: column; box-sizing: border-box;">
  <div style="min-height: 100px; aspect-ratio: 1/1; padding-left: 25px; padding-right: 25px; box-sizing: border-box;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, item).expect("item box");
    assert!(
        (w - 100.0).abs() < 1.5,
        "R3794c: border-box transferred cross = main × ratio = 100，实际 {w}（+frame 双计 padding → 150）"
    );
}
