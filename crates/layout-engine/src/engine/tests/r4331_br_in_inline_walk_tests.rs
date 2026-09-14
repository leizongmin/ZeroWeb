//! R4331：inline 元素包裹的 `<br>` 强制换行不丢失（collect_flat_inline_children walk 臂）。
//!
//! `<div>abc<span>x<br/>y</span>def</div>`：span 含元素子（br）→ 主 collect 走
//! `collect_flat_inline_children` 扁平化 walk——旧实现把 br 落「空文本 + 零 frame」臂
//! continue 吞掉（`InlineItem::Br` 从未入列），行断消失：`x`/`y` 同行渲染，段落行结构
//! 整体漂移（run-in-breaking-001 ref 页 span 包裹 br 实证：7.60% → 修复后残差 2.29%）。
//! 主路径 br 臂（collect_inline_items Element 分支）不受影响（br 为容器直接子时不走 walk）。
//!
//! load-bearing：default-on 时两行盒 y 差 ≥ 一行高；kill-switch（回退吞 br）时单行。
//! A/B：corpus 14766→14751（净 -15 = line-break-{loose,normal,strict}-011/014/016a/016b/018
//! ×3=15 案「test/ref 同错诚实化暴露」——control 的 span 包裹 br 修复后真断行、test 的
//! 软换行仍受 R1769 字体源域 CJK advance 偏差制约，两错同源一致被打破；R246 先例口径）；
//! run-in-breaking-001 7.60→2.29%、002 8.4→1.21%；零真回归（box-shadow-overlapping-003
//! 为 ~50% 双峰 flake，复跑即绿）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

/// 递归找到首个指定 tag 名的 LayoutBox。
fn find_box_by_tag<'a>(root: &'a LayoutBox, doc: &Document, tag: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = root.node_id
        && doc
            .get(nid)
            .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case(tag)))
    {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_box_by_tag(c, doc, tag) {
            return Some(b);
        }
    }
    None
}

/// R4331：`<div>ab<span>c<br/>d</span>ef</div>` 应产两行（br 在 span 内仍强制换行）。
/// 旧行为（walk 吞 br）单行：容器高度 ≈ 一行。
#[test]
fn test_br_inside_inline_element_forces_line_break() {
    let html = r#"<html><body style="margin:0"><div style="width:600px">ab<span>c<br/>d</span>ef</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");
    // 16px 字号 normal 行高 ≈ 18.6px：两行 > 25px，旧吞 br 单行 < 20px。
    assert!(
        div.height > 25.0,
        "br inside <span> must force a line break (two lines); got div.height={}",
        div.height
    );
}

/// R4331 对照臂：br 为块容器直接子时（主路径 br 臂）本就换行——双臂一致性守卫。
#[test]
fn test_br_direct_child_of_block_still_breaks() {
    let html = r#"<html><body style="margin:0"><div style="width:600px">ab<br/>cd</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_box_by_tag(&result.root, &doc, "div").expect("should find <div>");
    assert!(
        div.height > 25.0,
        "br as direct block child must still force a line break; got div.height={}",
        div.height
    );
}
