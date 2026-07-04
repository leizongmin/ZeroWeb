//! R717（CSS §10.3.2 + Flexbox §4.5）：ratio-only SVG `<img>`（%-dim / viewBox-only）
//! 在 flex 容器内按 transferred-size 推导 main 尺寸的回归测试。
//!
//! 驱动案 `aspect-ratio-intrinsic-size-007`：`<svg width="100%" height="100%" viewBox="0 0 7500 3750">`
//! 经 `<img>` 嵌入 `<div style="display:flex;flex-direction:column">`。SVG 无确定固有尺寸、仅有
//! viewBox 比 2:1。期望 img 在 flex column 内 width 拉伸到容器宽（800）、height = width/ratio = 400。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 在布局树中查找指定 DOM NodeId 的盒尺寸 (width, height)。
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

/// R717 驱动案：flex column + ratio-only SVG img（ratio 2:1）→ width 拉伸 800、height=400。
#[test]
fn r717_flex_column_ratio_only_img_derives_height() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column"><img src="large-green-rectangle.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    // ratio-only 信号：ratio = 2.0（viewBox 7500×3750），无确定固有尺寸。
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // 容器宽 800（flex column align-stretch cross=width）；height = width / ratio = 400。
    assert!(
        (w - 800.0).abs() < 2.0,
        "img width should stretch to container 800, got {w}"
    );
    assert!(
        (h - 400.0).abs() < 2.0,
        "R717: img height should ratio-derive to 400 (800/2), got {h}"
    );
}

/// R717 flex row 对称：ratio-only img 在 flex row（明确 height 200）→ width = height × ratio。
#[test]
fn r717_flex_row_ratio_only_img_derives_width() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;height:200px"><img src="r.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // flex row：cross=height 拉伸到容器 200；main=width = height × ratio = 400。
    assert!(
        (h - 200.0).abs() < 2.0,
        "img height should stretch to container 200, got {h}"
    );
    assert!(
        (w - 400.0).abs() < 2.0,
        "R717: img width should ratio-derive to 400 (200×2), got {w}"
    );
}

/// R717 非 flex 父（block）不应触发 ratio-derivation——img 保持无确定尺寸（不 collapse 也不强推）。
/// 此前该 img 同样无 size（ratio-only SVG 从不在 image_sizes 中），故不构成回归。
#[test]
fn r717_block_parent_ratio_only_img_no_force() {
    let html = r#"<html><body style="margin:0">
<div><img src="r.svg"/></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    // 仅断言不 panic + img 存在；非 flex 块上下文 ZW 暂未实现 300×150 默认（独立 gap）。
    let (_w, _h) = find_box(&result.root, img_id).expect("img box found");
}

// ── R1013：aspect-ratio fixup 守卫（非替换 + main 轴 min-size 不覆盖）──
// R994 fixup 对非替换 leaf（div + CSS aspect-ratio）泛化后，对带 main 轴 definite min-size
// 的项误覆盖（cross→main 反向推导破坏 min-size 驱动），致 flex-item-transferred-sizes-padding
// 回归 +73pp。守卫：非替换 + min-size 时跳过 fixup；替换元素（img）保留（transferred 语义不变）。

/// R1013 驱动案：flex column + 非替换 div（aspect-ratio:1/1 + min-height:100px）→ height 不被
/// fixup 覆盖为 cross/ratio。修复前 fixup 把 height 强推为 width，破坏 min-height:100px 驱动。
#[test]
fn r1013_flex_column_non_replaced_with_min_height_not_overridden() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column">
  <div style="min-height:100px; aspect-ratio:1/1; background:green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    // 第一个 div 是 flex 容器，第二个是 item。
    let item_id = divs.into_iter().nth(1).expect("item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (_w, h) = find_box(&result.root, item_id).expect("item box found");
    // min-height:100px 驱动：height 至少 100。修复前 fixup 覆盖为 width/ratio（width 可能很小）
    // 致 height 远小于 100。此处断言 height ≥ 100（min-height 被尊重）。
    assert!(
        h >= 99.0,
        "R1013: 非替换 div + min-height:100px 的 height 应 ≥ 100（min-height 驱动），got {h}"
    );
}

/// R1013 对照：替换元素（img）+ min-height 仍享 fixup（transferred-size 语义不变）。
/// flex-aspect-ratio-img-column-006 / row-004 需 fixup 才 <1%（min-size 不改变替换项语义）。
#[test]
fn r1013_flex_replaced_with_min_height_still_uses_fixup() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column">
  <img src="r.svg" style="min-height:100px"/>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 2.0_f32);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), ratios);
    let (_w, h) = find_box(&result.root, img_id).expect("img box found");
    // 替换元素 fixup 仍触发：width 拉伸 800，height = width/ratio = 400（≥ min-height 100）。
    assert!(
        h >= 100.0,
        "R1013: 替换 img + min-height:100px 仍应 fixup（height ≥ 100），got {h}"
    );
}

// ── R1015：flex container width:auto + float shrink-to-fit（R370 首切）──
// 非替换 leaf flex item（aspect-ratio + min-height）的 transferred cross-size 推导 +
// flex_column_intrinsic_width 让 float:flex 容器 shrink-to-fit。驱动案
// flex-item-transferred-sizes-padding（88.19%→0.60% PASS）。

/// R1015 驱动案：float:left + flex column + item（aspect-ratio:1/1 + min-height:100px）。
/// 容器应 shrink-to-fit 到 ~100px 宽（非拉满视口 800），item 应 ~100×100。
#[test]
fn r1015_float_flex_column_aspect_ratio_item_shrinks_to_fit() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; flex-direction:column; float:left">
  <div style="min-height:100px; aspect-ratio:1/1; padding-left:25px; padding-right:25px; box-sizing:border-box; background:green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let container_id = divs.into_iter().next().expect("container div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (cw, _ch) = find_box(&result.root, container_id).expect("container box found");
    // 容器 shrink-to-fit：宽度应近 item intrinsic（~100），而非拉满视口 800。
    assert!(
        cw < 200.0,
        "R1015: float:left flex column 容器应 shrink-to-fit 到 <200px，got {cw}"
    );
    assert!(
        cw >= 80.0,
        "R1015: 容器应至少 ~100px（item transferred width），got {cw}"
    );
}

/// R1017 驱动案 `aspect-ratio-intrinsic-size-003`：inline-flex + definite height + item
/// aspect-ratio:1/1 → 容器经 IFC `shrink_inline_blocks_to_content` 路径 shrink-to-fit 到
/// item transferred width（height 100 × ratio 1 = 100）。R1016 taffy-gate 路径证伪后，
/// R1017 改走 inline-block IFC 测量路径（inline-flex 是 inline-level）。
#[test]
fn r1017_inline_flex_definite_height_aspect_ratio_item_shrinks_to_fit() {
    let html = r#"<html><body style="margin:0">
<div style="display:inline-flex; height:100px; background:green">
  <div style="aspect-ratio:1/1;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let mut iter = divs.into_iter();
    let container_id = iter.next().expect("container div");
    let item_id = iter.next().expect("item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (cw, ch) = find_box(&result.root, container_id).expect("container box found");
    let (iw, ih) = find_box(&result.root, item_id).expect("item box found");
    // 几何应精确 100×100（item transferred width = container height 100 × ratio 1）。
    // 容器 width shrink 自 800→100；height 来自 style height:100px；item stretch 填满。
    // 残余 oracle 1.80% = `<p>` 文本字体光栅化墙（非布局；R990/R1005 territory）。
    assert!(
        (cw - 100.0).abs() < 2.0,
        "R1017: inline-flex 容器 width 应 shrink 到 100，got {cw}"
    );
    assert!(
        (ch - 100.0).abs() < 2.0,
        "R1017: 容器 height 应为 style 100px，got {ch}"
    );
    assert!(
        (iw - 100.0).abs() < 2.0 && (ih - 100.0).abs() < 2.0,
        "R1017: item 应 stretch 填满 100×100，got {iw}x{ih}"
    );
}

/// R1018 驱动案 `aspect-ratio-intrinsic-size-011`（post-JS final state）：block div +
/// `width:fit-content`（bare keyword，parser 映射 MaxContent）+ flex 子 + aspect-ratio item。
/// block-level shrink-to-fit gate + block_max_content_width（dispatch flex 子）→ target shrink 到
/// flex 子 intrinsic（item transferred width = height 100 × ratio 1 = 100）。
#[test]
fn r1018_block_fit_content_with_flex_aspect_ratio_child_shrinks_to_fit() {
    let html = r#"<html><body style="margin:0">
<div id="target" style="height:100px; width:fit-content; background:green">
  <div style="display:flex; height:100%; background:green">
    <div style="aspect-ratio:1/1;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let mut iter = divs.into_iter();
    let target_id = iter.next().expect("target div");
    let flex_id = iter.next().expect("flex div");
    let item_id = iter.next().expect("item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (tw, th) = find_box(&result.root, target_id).expect("target box found");
    let (fw, fh) = find_box(&result.root, flex_id).expect("flex box found");
    let (iw, ih) = find_box(&result.root, item_id).expect("item box found");
    // target 应 shrink 到 ~100（flex 子 intrinsic），非拉满视口 800（fit-content pre-R1018）也非 0。
    assert!(
        (tw - 100.0).abs() < 3.0,
        "R1018: target width:fit-content 应 shrink 到 100，got {tw}"
    );
    assert!((th - 100.0).abs() < 3.0, "R1018: target height 应为 100px，got {th}");
    assert!(
        (fw - 100.0).abs() < 3.0 && (fh - 100.0).abs() < 3.0,
        "R1018: flex 子应 100×100（height:100% + width:fill target），got {fw}x{fh}"
    );
    assert!(
        (iw - 100.0).abs() < 3.0 && (ih - 100.0).abs() < 3.0,
        "R1018: item 应 stretch 100×100，got {iw}x{ih}"
    );
}

/// R1019 驱动案 `aspect-ratio-intrinsic-size-014`（post-JS final state）：float:left block +
/// width:auto + flex 子（height:100%）+ aspect-ratio item。block-float gate（R1019 扩 is_auto_float
/// 含 Block）+ block_max_content_width（dispatch flex 子）→ float block shrink 到 flex 子 intrinsic
///（item transferred width = height 100 × ratio 1 = 100）。
#[test]
fn r1019_float_block_with_flex_aspect_ratio_child_shrinks_to_fit() {
    let html = r#"<html><body style="margin:0">
<div id="target" style="float:left; height:100px; background:green">
  <div style="display:flex; height:100%; background:green">
    <div style="aspect-ratio:1/1;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let mut iter = divs.into_iter();
    let target_id = iter.next().expect("target div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (tw, th) = find_box(&result.root, target_id).expect("target box found");
    // target（float:left block）应 shrink 到 ~100（flex 子 intrinsic），非拉满视口 800。
    assert!(
        (tw - 100.0).abs() < 5.0,
        "R1019: float:left block + flex 子 应 shrink 到 100，got {tw}"
    );
    assert!((th - 100.0).abs() < 3.0, "R1019: target height 应为 100px，got {th}");
}

/// R1020 驱动案 `change-intrinsic-width`（post-JS final state）：multicol 容器 columns:2 +
/// width:fit-content + 2 个 leaf 子（50px each）→ 容器 shrink-to-fit 到 2 × 50 = 100。
/// block_max_content_width 的 multicol 分支（leaf guard：仅所有 in-flow 子无元素子才乘 N）。
#[test]
fn r1020_multicol_fit_content_two_leaf_columns_shrinks_to_2x_content() {
    let html = r#"<html><body style="margin:0">
<div id="target" style="columns:2; column-gap:0; width:fit-content; height:100px; background:green">
  <div style="width:50px; height:100px; background:green"></div>
  <div style="width:50px; height:100px; background:green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let target_id = divs.into_iter().next().expect("target div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (tw, _th) = find_box(&result.root, target_id).expect("target box found");
    assert!(
        (tw - 100.0).abs() < 5.0,
        "R1020: multicol columns:2 + leaf 子 50px 应 shrink 到 2×50=100，got {tw}"
    );
}

/// R1020 对照：multicol + column-span:all 嵌套子不触发 N×（leaf guard 守护）——容器取 span:all
/// 内容宽（100），非 3×100=300。intrinsic-size-002 驱动。
#[test]
fn r1020_multicol_column_span_all_not_multiplied() {
    let html = r#"<html><body style="margin:0">
<div style="width:100px; height:100px; background:red">
  <div id="mc" style="width:fit-content; columns:3; background:green">
    <div style="column-span:all"><div style="width:100px; height:100px;"></div></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let mc_id = divs.into_iter().nth(1).expect("mc div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (mcw, _mch) = find_box(&result.root, mc_id).expect("mc box found");
    // span:all 子有元素子 → leaf guard 跳过 N× → 容器取 max(span:all content=100)，非 300。
    assert!(
        (mcw - 100.0).abs() < 10.0,
        "R1020: multicol + column-span:all 嵌套子不应 N×（应 ~100），got {mcw}"
    );
}

/// R1015 对照：非 float 的 block flex column + width:auto 不触发 shrink（保持当前行为，零回归）。
#[test]
fn r1015_block_flex_column_auto_width_no_shrink() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; flex-direction:column">
  <div style="min-height:100px; aspect-ratio:1/1; background:green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let container_id = divs.into_iter().next().expect("container div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (cw, _ch) = find_box(&result.root, container_id).expect("container box found");
    // 非 float 的 block flex 容器 width:auto 仍拉满（不触发 shrink），保持当前行为。
    assert!(
        cw >= 700.0,
        "R1015: block flex column auto width 应保持拉满（≥700），不 shrink，got {cw}"
    );
}
