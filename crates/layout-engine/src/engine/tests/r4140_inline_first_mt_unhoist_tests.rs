//! R4140：inline-level 首子 margin-top 折叠抬升修正回归测试。
//!
//! CSS §8.3.1：inline-level 盒的 margin 不与任何东西折叠。taffy 把 inline-block 原子
//! 映射为 Block 子，容器无 border/padding-top 时其 mt 被折叠抬升进容器 margin-top
//!（容器整体偏低），而原子的行内定位（IFC run.y）又应用一次 mt → 双计。
//! 修正（float_positioning.rs）：容器 mt 膨胀且 == 首个流内 inline-level 子的干净 mt
//! 时，把多折叠量从容器 y 扣除并恢复 declared margin_top。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_by_width(root: &LayoutBox, w: f32) -> Vec<&LayoutBox> {
    let mut hits = Vec::new();
    if (root.width - w).abs() < 0.5 {
        hits.push(root);
    }
    for c in &root.children {
        hits.extend(find_by_width(c, w));
    }
    hits
}

/// body(margin:8) 首子 inline-block(mt:32)：body y 应 = 8（自身声明 margin，不被
/// 原子 mt 折叠抬升到 32）；原子相对 body 的 y = 32（IFC run.y）。
/// 旧实现：body y=32（taffy 折叠）+ 原子 y=32 → 绝对 y=64（应 40）。
#[test]
fn test_r4140_body_y_not_hoisted_by_inline_atom_margin() {
    let html = r#"<html><body style="margin:8px"><div style="display:inline-block;vertical-align:top;margin-top:32px;width:20px;height:20px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // body：804px 宽（800 - 2×8 margin − … 边界盒按 viewport 减 margin），用 y 判定
    let bodies: Vec<&LayoutBox> = find_by_width(&result.root, 784.0);
    assert!(!bodies.is_empty(), "应找到 784px 宽的 body 盒（800 − 左右 margin 8×2）");
    let body = bodies[0];
    assert_eq!(
        body.y, 8.0,
        "body y 应为 8（声明 margin-top；taffy 折叠抬升须被修正回）"
    );
    assert_eq!(body.margin_top, 8.0, "body margin_top 应回到声明值 8");
    // 原子相对 body y=32（mt），绝对 y=40（chromium 一致）
    let atom = body
        .children
        .iter()
        .find(|c| (c.width - 20.0).abs() < 0.5)
        .expect("body 下应有 20px 宽原子");
    assert_eq!(atom.y, 32.0, "原子相对 body y = mt（IFC run.y 摆位）");
}

/// 对照：block 首子 margin 与 body 折叠语义不受影响（max(8,32)=32，CSS §8.3.1）。
#[test]
fn test_r4140_block_first_child_margin_still_collapses() {
    let html = r#"<html><body style="margin:8px"><div style="display:block;margin-top:32px;width:20px;height:20px"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let bodies: Vec<&LayoutBox> = find_by_width(&result.root, 784.0);
    let body = bodies.first().expect("应找到 body 盒");
    // block 首子：taffy 折叠正确，body y=32、子 y=0（折叠后 margin 消费于容器外）
    assert_eq!(
        body.y, 32.0,
        "block 首子 mt:32 应与 body mt:8 折叠为 32（回归守卫：修正不得波及 block 子）"
    );
}
