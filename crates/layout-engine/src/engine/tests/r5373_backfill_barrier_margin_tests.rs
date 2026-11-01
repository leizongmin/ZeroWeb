//! R5373 回归测试：R1743 父高度回填 extent 计入塌缩屏障（CSS2 §10.6.3 + §8.3.1）。
//!
//! taffy 对含行内元素子的块（p > strong 跨行多 fragment）欠计块高，R1743 回填是
//! 定高位；父有 bottom 框边（border/padding）或 BFC 时 §8.3.1 阻断父子 bottom 折叠，
//! 回填 extent 须取末子 margin-box 底（含 mb），否则父高短 child.mb。
//! driving：WPT background-root-017（body/pb+bb > p/mb16 → 短 16，双层叠 32）。
//! load-bearing：无屏障臂验证折叠泄出侧维持 border-box 底（原语义）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 布局并返回两 id 盒。harness `compute_styles(&doc, &[])` 不吃文档 `<style>` 块，
/// 样式全 inline。
fn layout_two(html: &str, id_a: &str, id_b: &str) -> (LayoutBox, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let node_a = doc.get_element_by_id(id_a).expect("元素存在");
    let node_b = doc.get_element_by_id(id_b).expect("元素存在");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (
        find(&result.root, node_a).unwrap().clone(),
        find(&result.root, node_b).unwrap().clone(),
    )
}

/// 017 驱动文本（足够长以跨行多 fragment）。
const LONG: &str = "This paragraph should be a box that is green with a green border, which should be in a navy box with a bright blue border. The rest of the page should also be navy. The outer sea of blue should extend to the edges of the viewport.";

/// R5373 主案：div(pb16+bb3) > p(mb16) > 文本+strong(padding16 跨行)。taffy 欠计 p 高，
/// 回填 extent 须含 p mb → div content 高 = p margin-box 底。
#[test]
fn r5373_backfill_barrier_includes_child_margin_bottom() {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="par" style="border-bottom:3px solid lime; padding-bottom:16px">
  <p id="kid" style="margin:16px 0">{} <strong style="padding:16px">{}</strong></p>
</div>
</body></html>"#,
        LONG, LONG
    );
    let (par, kid) = layout_two(&html, "par", "kid");
    let margin_box_bottom = kid.y + kid.height + kid.margin_bottom;
    assert!(
        (par.content_height - margin_box_bottom).abs() < 0.5,
        "屏障在场回填 extent 应含子 mb：content h 应 {:.1}，实际 {:.1}",
        margin_box_bottom,
        par.content_height
    );
}

/// BFC 无框边臂（挂账不展开）：overflow:hidden 父按 spec 亦阻断折叠，但 R4068 空子
/// oracle 域 + 泄漏 trailing 链（margin-collapse-102/103、collapse-through-percentage-
/// padding A/B 5 案回退）为现行矿区——extent 维持 border-box 底（回归守卫）。
#[test]
fn r5373_backfill_bfc_without_frame_edge_keeps_border_box_bottom() {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="par" style="overflow:hidden">
  <p id="kid" style="margin:16px 0">{} <strong style="padding:16px">{}</strong></p>
</div>
</body></html>"#,
        LONG, LONG
    );
    let (par, kid) = layout_two(&html, "par", "kid");
    let border_box_bottom = kid.y + kid.height;
    assert!(
        (par.content_height - border_box_bottom).abs() < 0.5,
        "BFC 无框边 extent 应维持 border-box 底 {:.1}，实际 {:.1}",
        border_box_bottom,
        par.content_height
    );
}

/// 无屏障对照：父无 bottom 框边非 BFC → 子 mb 与父 bottom margin 折叠泄出，
/// 回填 extent 维持 border-box 底（R1743 原语义，回归守卫）。
#[test]
fn r5373_backfill_no_barrier_keeps_border_box_bottom() {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="par">
  <p id="kid" style="margin:16px 0">{} <strong style="padding:16px">{}</strong></p>
</div>
</body></html>"#,
        LONG, LONG
    );
    let (par, kid) = layout_two(&html, "par", "kid");
    let border_box_bottom = kid.y + kid.height;
    assert!(
        (par.content_height - border_box_bottom).abs() < 0.5,
        "无屏障回填 extent 应维持 border-box 底 {:.1}，实际 {:.1}",
        border_box_bottom,
        par.content_height
    );
}

/// 017 全形态：html/body 链各含 border+padding，双层回填各含子 mb（缺陷 16+32 像素源）。
#[test]
fn r5373_root_chain_double_barrier() {
    let html = format!(
        r#"<html style="margin:16px; padding:16px; border:3px solid blue">
<body id="body2" style="margin:16px; padding:16px; border:3px solid lime">
<p id="kid" style="margin:16px; padding:16px">{} <strong style="margin:16px; padding:16px">{}</strong></p>
</body></html>"#,
        LONG, LONG
    );
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let kid_node = doc.get_element_by_id("kid").expect("元素存在");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let root = &result.root;
    let body2 = root
        .children
        .iter()
        .find(|b| b.height > 0.0 && b.node_id.is_some())
        .expect("body 盒存在");
    let kid = find(root, kid_node).unwrap();
    let kid_mb_bottom = kid.y + kid.height + kid.margin_bottom;
    assert!(
        (body2.content_height - kid_mb_bottom).abs() < 0.5,
        "body content h 应含 p mb（{:.1}），实际 {:.1}",
        kid_mb_bottom,
        body2.content_height
    );
    let body_mb_bottom = body2.y + body2.height + body2.margin_bottom;
    let html_content = root.content_height;
    assert!(
        (html_content - body_mb_bottom).abs() < 0.5,
        "html content h 应含 body mb（{:.1}），实际 {:.1}",
        body_mb_bottom,
        html_content
    );
}
