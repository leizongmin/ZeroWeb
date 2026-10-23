//! R5236（CSS2 §10.3.8 abspos shrink-to-fit）：含 replaced 原子后代的 auto 宽 abspos
//! 宽度解析锚。
//!
//! R5234 分诊定谳（before-after-positioned-001 D2 域）：`<span style="position:absolute;
//! left:0">1<img 32×32>"Before gen abs</span>` 形态 taffy 解析为 min-content（32px =
//! img 宽，窄列逐词换行），§10.3.8 shrink-to-fit 应 = max-content（单行 ≈160px）。
//! 根因：R2156 slice19 提升原子为 taffy Absolute 子（R4941）后 abs span 成 branch 节点
//!（无 measure 函数），taffy content_size = 子贡献 max（IFC 文本不入）→ 宽 = 最宽原子子。
//! img 0×0 时塌 0 入 R3929 塌 0 补测臂反而正确。修复 = `apply_abspos_shrink_to_fit_width`
//! 扩臂（branch-text 盒保持 width_fix 重测，只扩不缩）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width, b.height));
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

/// D2 主锚：attr 固有尺寸 img（32×32）+ 文本的 abs span auto 宽应收缩适配到单行
/// max-content（文本 16 字符 × 8 + img 32 = 160），高单行；旧塌 32 逐词换行（h≈104）。
#[test]
fn r5236_abs_span_replaced_child_shrinks_to_max_content() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; height: 102px;">
<span style="position: absolute; left: 0;">1<img src="support/square.png" width="32" height="32" />"Before gen abs</span>Inner
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let tid = *spans.first().expect("abs span");
    let (_x, _y, w, h) = find_box(&result.root, tid).expect("abs span box");
    assert!(
        (w - 160.0).abs() < 2.0,
        "R5236 D2: abs span 含原子子 + 文本的 auto 宽应 shrink-to-fit 单行 160，实际 {w}"
    );
    assert!(
        h < 40.0,
        "R5236 D2: 单行放下不应逐词换行（旧 min-content 形态 h≈104；单行含 32px img 的行盒高 ≈32），实际 {h}"
    );
}

/// 对照（既有绿路径）：img 无固有尺寸（0×0）时塌 0 入 R3929 塌 0 补测臂——行为保持。
#[test]
fn r5236_abs_span_zero_img_still_correct() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; height: 102px;">
<span style="position: absolute; left: 0;">1<img src="support/square.png" />"Before gen abs</span>Inner
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let tid = *spans.first().expect("abs span");
    let (_x, _y, w, h) = find_box(&result.root, tid).expect("abs span box");
    assert!(
        (w - 128.0).abs() < 2.0 && h < 30.0,
        "R5236: img 0×0 abs span 应保持 128 单行（既有塌 0 补测路径），实际 {w}×{h}"
    );
}

/// 守卫：definite max-width 排除（扩臂测量不含 max-width 钳，不介入防撑破钳制盒）——
/// max-width:50 的 abs span 维持 taffy 钳制宽（≤50），不得被 max-content（160）覆写。
#[test]
fn r5236_abs_span_max_width_not_grown_past_cap() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; height: 102px;">
<span style="position: absolute; left: 0; max-width: 50px;">1<img src="support/square.png" width="32" height="32" />"Before gen abs</span>Inner
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let tid = *spans.first().expect("abs span");
    let (_x, _y, w, _h) = find_box(&result.root, tid).expect("abs span box");
    assert!(
        w <= 51.0,
        "R5236: max-width:50 的 abs span 不得被扩臂撑破钳制（应 ≤50），实际 {w}"
    );
}

/// 守卫：aspect-ratio transferred 宽排除（R4987 域）——`min-width:0` 关掉 R4987 臂①
/// 的 content floor 后，宽 = 纯传递 高×ratio = 100；扩臂若误触会写成 max-content 161。
#[test]
fn r5236_abs_span_aspect_ratio_transferred_not_overwritten() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; height: 102px;">
<span style="position: absolute; left: 0; aspect-ratio: 1 / 1; height: 100px; min-width: 0;">1<img src="support/square.png" width="32" height="32" />"Before gen abs</span>Inner
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let tid = *spans.first().expect("abs span");
    let (_x, _y, w, h) = find_box(&result.root, tid).expect("abs span box");
    assert!(
        (w - 100.0).abs() < 3.0,
        "R5236: aspect-ratio 纯传递宽应保持 100（扩臂排除），实际 {w}（h={h}）"
    );
}

/// 对照（R3929 既有绿路径）：纯文本 leaf abs div shrink-to-fit 不受扩臂扰动。
#[test]
fn r5236_abs_div_text_only_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; height: 102px;">
<div style="position: absolute; left: 0;">"Before gen abs text</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.last().expect("abs div");
    let (_x, _y, w, h) = find_box(&result.root, tid).expect("abs div box");
    assert!(
        (w - 160.0).abs() < 2.0 && h < 30.0,
        "R5236: 纯文本 leaf abs div 应保持 160 单行（R5104 measure 臂），实际 {w}×{h}"
    );
}
