//! R5300（CSS2 §10.3.7 shrink-to-fit 链）：嵌套 abs-pos 链的 CB 宽传播锚。
//!
//! 案 text-decoration-subelements-002（✗1.07%）ref 侧分诊：`abs > abs(overflow:hidden)
//! > abs` 链中，内层 auto 宽 abs 的 shrink-to-fit 被**过期 CB 宽**钳死——
//! `apply_abspos_shrink_to_fit_width` 单趟 walk 里子代 cb_width 取 taffy 首趟布局值
//! （父代自身 unresolved 塌缩宽：最宽元素子或 0），`target = min(measured, cb)` 写不进
//! → overflow:hidden 盒 13px 裁掉全部内容。修复 = pass 内写定宽度即时回写子代栈槽
//! （`propagate_resolved_cb_width`，kill-switch `ZW_ABSPOS_STF_CB_PROP=0`）。

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

/// 主锚（branch 形态，R5236 D2 域）：嵌套内层 `<b>bold</b> tail` auto 宽 abs 应与
/// 顶层同内容参照盒同宽单行——旧行为内层塌 13（= 最宽元素子 <b> 宽），IFC 文本
/// 「 tail」不入 taffy content_size 且过期 CB 钳死补测。
#[test]
fn r5300_nested_abspos_branch_text_shrinks_like_top_level() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px;">
<div style="position: absolute; top: 200px;"><b>bold</b> tail</div>
<div style="position: absolute; top: 300px; overflow: hidden;">
<div style="position: absolute; top: 0;"><b>bold</b> tail</div>
</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let top_ref = *divs.get(divs.len() - 3).expect("top-level ref div");
    let outer = *divs.get(divs.len() - 2).expect("outer abs div");
    let inner = *divs.last().expect("nested inner abs div");
    let (_x, _y, w_ref, h_ref) = find_box(&result.root, top_ref).expect("ref box");
    let (_x, _y, w_outer, _h) = find_box(&result.root, outer).expect("outer box");
    let (_x, _y, w_inner, h_inner) = find_box(&result.root, inner).expect("inner box");
    assert!(
        (w_inner - w_ref).abs() < 2.0,
        "R5300: 嵌套 branch 内层应与顶层参照同宽 shrink-to-fit（ref {w_ref}），实际 {w_inner}（旧塌最宽元素子宽）"
    );
    assert!(
        h_inner <= h_ref + 1.0,
        "R5300: 内层应单行放下（ref 高 {h_ref}），实际 {h_inner}"
    );
    assert!(
        w_outer >= w_inner - 1.0,
        "R5300: overflow:hidden 外层 shrink-wrap 应 ≥ 内层宽（inner {w_inner}），实际 {w_outer}"
    );
}

/// 链式锚（ref 页全形态，三层链）：wrapper abs（仅 abs 子）→ overflow:hidden abs
/// （in-flow 文本 + <i> 元素子）→ abs top:0 覆盖层——旧链 wrapper 塌 0、中层塌最宽
/// 元素子宽且 CB=0 钳死补测；修复后中层 = in-flow 文本 shrink-to-fit，与顶层参照同宽。
#[test]
fn r5300_three_level_abspos_chain_propagates_cb_width() {
    let html = r#"<html><body style="margin:0">
<div style="position: relative; width: 774px; font-size: 32px;">
<div style="position: absolute; top: 500px;">Before gen abs text <i>e = mc<sup>2</sup></i>.</div>
<div style="position: absolute; top: 600px;">
<div style="position: absolute; overflow: hidden;">Before gen abs text <i>e = mc<sup>2</sup></i>.
<div style="position: absolute; top: 0;">
<span>Einstein said that e = mc2......</span>
</div>
</div>
</div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let top_ref = *divs.get(divs.len() - 4).expect("top-level ref div");
    let outer = *divs.get(divs.len() - 3).expect("wrapper abs div");
    let clip = *divs.get(divs.len() - 2).expect("overflow:hidden div");
    let (_x, _y, w_ref, _h) = find_box(&result.root, top_ref).expect("ref box");
    let (w_wrapper, w_clip) = {
        let (_x, _y, w, _h) = find_box(&result.root, outer).expect("wrapper box");
        (w, {
            let (_x, _y, w, _h) = find_box(&result.root, clip).expect("clip box");
            w
        })
    };
    assert!(
        w_wrapper >= w_clip - 1.0,
        "R5300: wrapper（仅 abs 子）塌 0 补测后应 ≥ 中层宽（clip {w_clip}），实际 {w_wrapper}（旧塌 0）"
    );
    assert!(
        (w_clip - w_ref).abs() < 2.0,
        "R5300: overflow:hidden 中层应 = in-flow 文本 shrink-to-fit（ref {w_ref}），实际 {w_clip}（旧塌最宽元素子宽）"
    );
}
