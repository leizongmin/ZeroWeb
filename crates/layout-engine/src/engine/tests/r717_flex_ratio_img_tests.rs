//! R717（CSS §10.3.2 + Flexbox §4.5）：ratio-only SVG `<img>`（%-dim / viewBox-only）
//! 在 flex 容器内按 transferred-size 推导 main 尺寸的回归测试。
//!
//! 驱动案 `aspect-ratio-intrinsic-size-007`：`<svg width="100%" height="100%" viewBox="0 0 7500 3750">`
//! 经 `<img>` 嵌入 `<div style="display:flex;flex-direction:column">`。SVG 无确定固有尺寸、仅有
//! viewBox 比 2:1。期望 img 在 flex column 内 width 拉伸到容器宽（800）、height = width/ratio = 400。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
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

/// 在布局树中查找指定 DOM NodeId 的 LayoutBox 引用（供检查 column_span_offsets 等）。
fn find_box_ref(root: &LayoutBox, node_id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
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

/// R1028：`column-span:all` spanner 脱离列流成全宽元素（width = 容器 content_width），
/// 非 narrowed 到列宽。spanner-fragmentation-000~007 / multicol-span-all-001 驱动
/// （+9 oracle pass）。
#[test]
fn r1028_column_span_all_spanner_is_full_width() {
    let html = r#"<html><body style="margin:0">
<div id="mc" style="width:400px; columns:2; column-gap:0; background:green">
  <div id="first" style="height:50px; background:blue"></div>
  <div id="spanner" style="column-span:all; height:30px; background:black"></div>
  <div id="last" style="height:50px; background:red"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let mut iter = divs.into_iter();
    let _mc_id = iter.next().expect("mc");
    let _first_id = iter.next().expect("first");
    let spanner_id = iter.next().expect("spanner");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let (sw, _sh) = find_box(&result.root, spanner_id).expect("spanner box found");
    // spanner 全宽 = 容器 400px（非被 narrow 到列宽 200px）。
    assert!(
        (sw - 400.0).abs() < 10.0,
        "R1028: column-span:all spanner 应全宽 ~400px（容器宽），非列宽 200px，got {sw}"
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

/// R1075：非 spanner balance 多列容器（definite 高度 + 内容超 col_count×列高）走 **inline
/// 列溢出**——列高 cap 在容器高度，超出内容生成额外 column box 溢出到容器右外侧（chromium
/// 实测确认）。columns:2 height:50 width:100 + 单个 200px 子 → 4 列各 50px（2 in-container +
/// 2 右溢出），column_span_offsets 4 片段，col_x 单调递增 0/50/100/150（gap 0, col_w 50）。
/// 纠正旧 balanced 在 col_count 处 break 丢弃 overflow（minimal multicol 同病）。
#[test]
fn r1075_non_spanner_balance_inline_overflow() {
    let html = r#"<html><body style="margin:0">
<div id="mc" style="columns:2; column-gap:0; width:100px; height:50px; background:green">
  <div id="child" style="height:200px; background:yellow"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let child_id = divs.into_iter().nth(1).expect("child div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let child_box = find_box_ref(&result.root, child_id).expect("child box found");
    assert_eq!(
        child_box.column_span_offsets.len(),
        4,
        "R1075: 200px child in 2-col×50px → 4 inline-overflow columns (2 in-container + 2 right)"
    );
    let xs: Vec<f32> = child_box.column_span_offsets.iter().map(|t| t.2).collect();
    assert!(
        (xs[0] - 0.0).abs() < 1.0
            && (xs[1] - 50.0).abs() < 1.0
            && (xs[2] - 100.0).abs() < 1.0
            && (xs[3] - 150.0).abs() < 1.0,
        "R1075: inline-overflow col_x 单调递增 0/50/100/150（向右溢出），got {xs:?}"
    );
}

/// R1075 守卫：monolithic（overflow≠visible）子元素不可分——不应触发 inline 列溢出拆分。
/// overflow-unsplittable-001 谱系：columns:2 height:100 + overflow:scroll 子（含 200px 孙）→
/// monolithic 子保持整体（balanced 路径，不拆），column_span_offsets 仅 1 片段（不跨列拆分）。
#[test]
fn r1075_monolithic_child_not_split_by_inline_overflow() {
    let html = r#"<html><body style="margin:0">
<div style="columns:2; column-gap:0; width:100px; height:100px; background:green">
  <div id="scroll" style="overflow:scroll; width:50px; background:blue">
    <div style="height:200px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let scroll_id = doc.get_element_by_id("scroll").expect("scroll div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let scroll_box = find_box_ref(&result.root, scroll_id).expect("scroll box found");
    // monolithic 子不拆分——column_span_offsets ≤ 1（不跨列），避免 R1075 误拆 overflow:scroll。
    assert!(
        scroll_box.column_span_offsets.len() <= 1,
        "R1075 guard: monolithic (overflow:scroll) 子不应被 inline-overflow 拆分，got {} 片段",
        scroll_box.column_span_offsets.len()
    );
}

/// R1076：column-fill:auto + definite 高度 + 内容超 col_count×列高 → **inline 列溢出**
///（chromium 实测确认，column-wrap:auto 默认）。columns:1 column-fill:auto height:100 +
/// 单个 200px 子 → 2 列各 100px（col0 in-container + col1 右溢出），column_span_offsets
/// 2 片段，col_x 递增 0/100（width 100, gap 0）。
#[test]
fn r1076_sequential_fill_auto_inline_overflow() {
    let html = r#"<html><body style="margin:0">
<div id="mc" style="columns:1; column-fill:auto; column-gap:0; width:100px; height:100px; background:green">
  <div id="child" style="height:200px; background:yellow"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let divs = doc.get_elements_by_tag_name("div");
    let child_id = divs.into_iter().nth(1).expect("child div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let child_box = find_box_ref(&result.root, child_id).expect("child box found");
    assert_eq!(
        child_box.column_span_offsets.len(),
        2,
        "R1076: 200px child in column-fill:auto 1-col×100px → 2 inline columns (1 in-container + 1 right)"
    );
    let xs: Vec<f32> = child_box.column_span_offsets.iter().map(|t| t.2).collect();
    assert!(
        (xs[0] - 0.0).abs() < 1.0 && (xs[1] - 100.0).abs() < 1.0,
        "R1076: inline-overflow col_x 0/100（向右溢出），got {xs:?}"
    );
}

/// R1076 守卫：nested multicol 子元素不触发 sequential inline-overflow（nested fragmentation
/// 须独立模型，同 R1035 守卫）。outer columns:1 column-fill:auto height:100 + inner columns:1
///（nested）+ child 200 → gate 跳过（has_nested_multicol），用 _with_breaking（不 push 溢出列），
/// inner 子的 column_span_offsets 不应 ≥ 2 片段（未 inline 拆分）。
#[test]
fn r1076_nested_multicol_child_guarded_sequential() {
    let html = r#"<html><body style="margin:0">
<div style="columns:1; column-fill:auto; column-gap:0; width:100px; height:100px; background:green">
  <div id="inner" style="columns:1; height:200px; background:yellow">
    <div style="height:200px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let inner_id = doc.get_element_by_id("inner").expect("inner div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, std::collections::HashMap::new(), HashMap::new());
    let inner_box = find_box_ref(&result.root, inner_id).expect("inner box found");
    // nested multicol 子被 guard 排除 → 不走 R1076 inline-overflow（不同 x 的溢出列）。
    // _with_breaking 可能仍**同列**拆分（col_x 都=0），但不应产生 col_x>0 的 inline 溢出列。
    let max_col_x = inner_box
        .column_span_offsets
        .iter()
        .map(|t| t.2)
        .fold(0.0_f32, f32::max);
    assert!(
        max_col_x < 1.0,
        "R1076 guard: nested multicol 子不应 inline-overflow（所有片段 col_x 应=0），got max_col_x={max_col_x}"
    );
}

/// R1363：flex row item（替换 img + 显式 width + aspect-ratio）的 main 被 min-size:auto 钳制时，
/// cross（height）须按**钳制后** main 推导，非按显式（未钳制）width 预推。
/// 驱动案 flex-minimum-width-flex-items-013（9.98%→0.63% FLIP）：
/// `<img style="width:999px">` 固有 300x150，flex width:0 height:50 → min 钳 width 到 100，
/// height 应 = 100/2 = 50（非 999/2 = 500）。tree.rs 预推 height 会设 definite 500 阻止 flex 重推。
#[test]
fn r1363_flex_row_item_aspect_ratio_cross_from_clamped_main() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:0px; height:50px;">
  <img id="img" src="300x150.png" style="width:999px;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut sizes = HashMap::new();
    sizes.insert(img_id, (300.0_f32, 150.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, HashMap::new());
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // width 被 min-size:auto 钳到 100；height 须按钳制后 width / ratio(2) = 50（非 999/2=500）。
    assert!(
        (w - 100.0).abs() < 5.0,
        "R1363: img width 应被 min-size 钳到 ~100，got {w}"
    );
    assert!(
        (h - 50.0).abs() < 5.0,
        "R1363: img height 应按钳制后 width 推导 = 50（非显式 999/2=500），got {h}"
    );
}

/// R1363 对照：vertical-lr flex 容器内的 img 不应触发跳过（主/交叉轴互换，推导不同），
/// 保持旧行为避免 vert-lr 回归（flex-aspect-ratio-img-vert-lr）。
#[test]
fn r1363_vertical_lr_flex_item_not_skipped() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; writing-mode:vertical-lr;">
  <img id="img" src="300x150.png" style="width:999px;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut sizes = HashMap::new();
    sizes.insert(img_id, (300.0_f32, 150.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, HashMap::new());
    // 仅断言不 panic + img 存在（vertical flex 的 aspect-ratio 推导独立 R109 地带，本 fix 不介入）。
    let (_w, _h) = find_box(&result.root, img_id).expect("img box found");
}

/// R1365：flex item 的 flex-basis 为百分比且容器 main 尺寸不明确时，item 的 main-size 属性
/// 不应被当 definite（CSS-Flexbox §9 + §7.1：百分比 flex-basis 对不明确容器回退 content，
/// 显式 main-size 被忽略）。驱动案 flex-basis-010（8.96%→0.63% FLIP）：
/// `flex:0 0 0%` + height:500px，容器 column 无 height → item height 应回退 content(100)，非 500。
#[test]
fn r1365_indefinite_percent_flex_basis_ignores_main_size() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:100px; flex-direction:column;">
  <div id="item" style="flex:0 0 0%; height:500px;">
    <div style="height:100px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item_id = doc.get_element_by_id("item").expect("item");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, HashMap::new(), HashMap::new());
    let (_w, h) = find_box(&result.root, item_id).expect("item box found");
    // height:500 被忽略，item main 回退 content（子 100），非 500。
    assert!(
        (h - 100.0).abs() < 10.0,
        "R1365: indefinite % flex-basis 应让 item height 回退 content ~100（非显式 500），got {h}"
    );
}

/// R1365 对照：容器 main 尺寸**明确**（height:200px）时，fix 不应误触发（百分比 flex-basis
/// 对明确容器正常解析）。守护 gate：仅 indefinite 容器 main 才清 size.main。
#[test]
fn r1365_definite_parent_main_not_affected() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:100px; height:200px; flex-direction:column;">
  <div id="item" style="flex:0 0 0%; height:500px;">
    <div style="height:100px;"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item_id = doc.get_element_by_id("item").expect("item");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, HashMap::new(), HashMap::new());
    // 容器 main 明确 → fix 不触发。仅断言不 panic + item 存在（守护 gate）。
    let (_w, _h) = find_box(&result.root, item_id).expect("item box found");
}

/// R1366：flex item aspect-ratio main 按容器 stretched cross 推导 + 同步设 cross。
/// 驱动案 flex-aspect-ratio-img-row-006（6.78%→0.53% FLIP）：img 固有 200x200 + width/height
/// auto + 容器 150×100 + flex-shrink:0 → main(width)=容器 cross(100)×ratio(1)=100，
/// cross(height)=100。tree.rs 预设 size=固有 200x200 definite 阻挡推导，R1366 fixup 覆盖。
#[test]
fn r1366_flex_item_aspect_ratio_main_from_stretched_cross() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:150px; height:100px;">
  <img id="img" src="200x200-green.png" style="min-width:0px; flex-shrink:0;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut sizes = HashMap::new();
    sizes.insert(img_id, (200.0_f32, 200.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, HashMap::new());
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // main(width) 按容器 cross(100)×ratio(1)=100；cross(height)=100（非固有 200）。
    assert!(
        (w - 100.0).abs() < 8.0,
        "R1366: img width 应按容器 stretched cross 推 = 100，got {w}"
    );
    assert!(
        (h - 100.0).abs() < 8.0,
        "R1366: img height 应 = 100（非固有 200），got {h}"
    );
}

/// R3616：flex transferred auto-min 扣 item cross padding 时也要解析 residual real length。
/// direct `ComputedStyle` 下 `padding-top/bottom:1em;font-size:20px` 会让 100px cross 的
/// content cross 变成 60px；旧逻辑只扣 `Px` padding，误按 100px transferred floor。
#[test]
fn r3616_flex_transferred_min_subtracts_relative_cross_padding() {
    let html = r#"<html><body style="margin:0">
<div id="container" style="display:flex; width:50px; height:100px;">
  <img id="img" src="200x200-green.png" style="min-width:0px;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let mut styles = sys.compute_styles(&doc, &[]);
    let container = doc.get_element_by_id("container").expect("container");
    let img = doc.get_element_by_id("img").expect("img");
    let container_style = styles.get_mut(&container).expect("container style");
    container_style.font_size = LengthValue::Px(20.0);
    let img_style = styles.get_mut(&img).expect("img style");
    img_style.font_size = LengthValue::Px(20.0);
    img_style.padding_top = LengthValue::Em(1.0);
    img_style.padding_bottom = LengthValue::Em(1.0);

    let mut sizes = HashMap::new();
    sizes.insert(img, (200.0_f32, 200.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, HashMap::new());
    let (w, _h) = find_box(&result.root, img).expect("img box found");

    assert!(
        (w - 60.0).abs() < 8.0,
        "R3616: img width 应按 content cross 100-2em@20px 推 = 60，got {w}"
    );
}

/// R1366 对照：item cross 轴**有 padding** 时，cross=parent_cross 不精确（content-box != border-box），
/// 故 cross-fix 跳过（守 padding-001 baseline，避免 R1364 v1 的 flip-fail 回归）。main 推导仍可触发。
#[test]
fn r1366_padded_item_cross_fix_skipped() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:150px; height:100px;">
  <img id="img" src="200x200-green.png" style="min-width:0px; flex-shrink:0; padding-top:10px; padding-bottom:10px;">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut sizes = HashMap::new();
    sizes.insert(img_id, (200.0_f32, 200.0_f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, HashMap::new());
    // 有 padding → cross-fix 跳过（不强制 cross=parent_cross）。仅断言不 panic + img 存在（守护 gate）。
    let (_w, _h) = find_box(&result.root, img_id).expect("img box found");
}

/// R1369：definite-width BFC（如 flex 容器）与 float 垂直重叠且 overflow 容器（不 fit beside）
/// 时，应推到 float 下方（CSS §9.5：BFC border-box 不重叠 float；definite 宽度保持不 shrink）。
/// 驱动案 flexbox_fbfc（14.38%→1.38%，geometry 修正；残余 1.38% = `<p>`/内文 font-wall）：
/// `<div float:left width:150>` + `<div flex width:480>`（容器宽 600，150+480>600 不 fit）。
/// 旧 ZW 把 flex 推到 float 右 x=150 并 shrink-to-fit（错）；taffy 0.12 native float 也推到
/// x=150。R1369 在 ZW 后处理 exclusion 的 `avoidance_x > child.x` **之外**做 fit-check，
/// 把 definite-width overflow BFC 推到 float 下方（y=float_bottom，x 回 margin_left）。
#[test]
fn r1369_definite_bfc_overflow_float_goes_below() {
    let html = r#"<html><body style="margin:0; width:600px;">
<div id="float" style="background:blue; width:150px; float:left; height:40px;"></div>
<div id="flex" style="background:yellow; width:480px; display:flex;">
  <div style="background:pink; height:40px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(600.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let flex_id = doc.get_element_by_id("flex").expect("flex");
    let mut engine = LayoutEngine::new(600.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, HashMap::new(), HashMap::new());
    let (fx, fy, fw) = {
        let mut stack: Vec<&LayoutBox> = vec![&result.root];
        let mut r = None;
        while let Some(b) = stack.pop() {
            if b.node_id == Some(flex_id) {
                r = Some((b.x, b.y, b.width));
                break;
            }
            stack.extend(b.children.iter());
        }
        r.expect("flex box")
    };
    // flex(480) 不 fit beside float(150)（150+480>600）→ 推到 float 下方：x=0（回 margin_left），
    // y=40（float_bottom），width=480 保持（非 shrink 到 450）。
    assert!(
        (fx - 0.0).abs() < 2.0,
        "R1369: definite-width BFC overflow float 应回 x=0（非停留 float 右 x=150），got fx={fx}"
    );
    assert!(
        (fy - 40.0).abs() < 2.0,
        "R1369: BFC 应推到 float 下方 y=40（float_bottom），got fy={fy}"
    );
    assert!(
        (fw - 480.0).abs() < 2.0,
        "R1369: BFC 宽度应保持 480（非 shrink 到 450），got fw={fw}"
    );
}
