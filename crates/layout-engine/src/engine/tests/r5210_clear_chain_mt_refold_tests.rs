//! R5210（CSS2 §8.3.1 × §9.5.2）：float 上下文重定位的链值 mt 归位。
//! taffy 把「父↔首流内子」可折叠对的整条下行链值折进链顶盒字段 margin_top
//!（内层成员 y 不含各自 mt）；Phase 2 重定位旧实现直接用字段 mt，把已上折的
//! margin 在内层成员本级重放——逐级堆叠泄漏（三层嵌套各 +16px、容器 132 应
//! 100），且祖先帧下移令递归透传的 float 底边在嵌套帧内失真，正 clearance
//! 被判成负。修：`chain_clear_break_fold` 检出下行链中的 clear 阻断点时按
//! 「阻断点以上 declared mt 折叠值」归位本级链值（字段复位 + extent 缩减记录），
//! 阻断点自身 mt 留本级参与 §9.5.2 C1/C2 假设位对比。
//! corpus 定谳：clear-on-child-with-margins-2（1.12%）翻绿，999→998 净 −1。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(body: &str) -> crate::engine::LayoutResult {
    let html = format!("<html><body style=\"margin:0\">{body}</body></html>");
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn find_by_depth<'a>(root: &'a LayoutBox, path: &[usize]) -> &'a LayoutBox {
    let mut cur = root;
    for &i in path {
        cur = &cur.children[i];
    }
    cur
}

/// clear-on-child-with-margins-2 形态：BFC 容器内 [FR 浮动][B1[B2[CL{clear,
/// mt:16}]]]。taffy 把 B1(0)+B2(0)+CL(16) 链值折进 B1；旧实现重放后 B1/B2/CL
/// 逐级 +16（CL 边界 48 应 20、容器 132 应 100），且 B1 帧下移令透传的右浮动
/// 底边（20）在 B2 帧内失准为 0——正 clearance 4px 被判负。修后：B1/B2 链值
/// 归位 0，CL mt 留本级 + 正 clearance 推到右浮动底，容器缩回 100。
#[test]
fn r5210_cleared_chain_margin_refolds_to_break_point() {
    let result = layout(
        "<div style=\"width: 100px;background: red;overflow: hidden;\">\
           <div style=\"float: right; height: 20px; width: 50%; background: green;\"></div>\
           <div>\
             <div style=\"float: left; height:100px; width: 50%; background: green;\"></div>\
             <div>\
               <div style=\"clear: right; height: 80px; margin-top: 16px; background: green;\"></div>\
             </div>\
           </div>\
         </div>",
    );
    // 树形：root[1]=html body 载体 → 容器；容器子序 [FR 浮动, B1]。
    let container = find_by_depth(&result.root, &[1, 0]);
    assert!(
        (container.height - 100.0).abs() < 0.5,
        "容器应缩回 100（修前泄漏 132），实得 {}",
        container.height
    );
    let b1 = &container.children[1];
    assert!(
        b1.y.abs() < 0.5 && (b1.height - 100.0).abs() < 0.5,
        "B1 链值应归位 y=0 h=100（修前 y=16 h=116），实得 y={} h={}",
        b1.y,
        b1.height
    );
    let b2 = &b1.children[1];
    assert!(
        b2.y.abs() < 0.5,
        "B2 链值应归位 y=0（修前本级重放 mt y=16），实得 y={}",
        b2.y
    );
    let cl = &b2.children[0];
    assert!(
        (cl.y - 20.0).abs() < 0.5,
        "CL 边界顶 = B2 顶 0 + 自身 mt 16 + 正 clearance 4 = 20（修前泄漏 16，帧失真下 16），实得 y={}",
        cl.y
    );
    assert!(
        (cl.margin_top - 16.0).abs() < 0.5,
        "clear 阻断点 mt 留本级（§8.3.1 the child has no clearance），实得 {}",
        cl.margin_top
    );
}

/// 无 clear 阻断点的链不激活重解析（零行为变化面）：同构三层嵌套 + mt:16 但
/// 末层无 clear——链值维持既有字段语义（首层 16、内层重放既有行为），防止
/// 击发门误开波及无 float 上下文以外的普通折叠树。
#[test]
fn r5210_chain_without_clear_break_keeps_legacy_semantics() {
    let result = layout(
        "<div style=\"width: 100px;background: red;overflow: hidden;\">\
           <div style=\"float: right; height: 20px; width: 50%; background: green;\"></div>\
           <div>\
             <div style=\"float: left; height:100px; width: 50%; background: green;\"></div>\
             <div>\
               <div style=\"height: 80px; margin-top: 16px; background: green;\"></div>\
             </div>\
           </div>\
         </div>",
    );
    let container = find_by_depth(&result.root, &[1, 0]);
    let b1 = &container.children[1];
    assert!(
        (b1.y - 16.0).abs() < 0.5,
        "无 clear 阻断点：链值 16 留链顶（既有行为，不重解析），实得 y={}",
        b1.y
    );
}
