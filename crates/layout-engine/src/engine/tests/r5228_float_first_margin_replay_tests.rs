//! R5228（CSS2 §8.3.1 × §9.5.1）：float 前置容器的首流内子重放不重放已折叠 margin。
//! taffy 把前序 float 当 block 排列 → 后继流内子 taffy y 含 float 高度，Phase 2
//! 重放臂（float_y_offset>0）以 `flow_bottom + collapse(0, 字段 mt)` 归位——但首流
//! 内子的 mt 已被 engine 折叠 pass 折进容器 margin（膨胀签名 box_node.margin_top >
//! declared，§8.3.1 parent↔child 折叠），本级重放 = 双计：border-right-width-095-ref
//! [float,p,float] 实测 p +16、flow_bottom 快照污染 → 后继 float floor 36→52
//!（square abs 52→68，chromium/ref 作者口径 52）。修：膨胀签名成立时 replay_mt=0
//!（R5212 同款签名；签名不成立时链顶值合法留本级——R5210 BFC 容器语义不扰动）。

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

/// border-right-width-095-ref 同构：[float 96 高][p mt:16 line:20][float]。
/// p 的 mt 折进容器 margin → p 位于 content 原点 y=0；后继 float 外顶 =
/// p margin-box 底 = 20+16 = 36（§9.5.1 前驱块 margin 边缘，chromium 探针
/// float-after-block = block margin-box bottom 同式）。修前 p y=16、float y=52。
#[test]
fn r5228_float_first_container_first_block_mt_not_replayed() {
    let result = layout(
        "<div style=\"width: 400px;\">\
           <div style=\"float: right; height: 96px; width: 96px;\"></div>\
           <p style=\"margin: 16px 0; font-size: 16px; line-height: 20px;\">hint</p>\
           <div style=\"float: right; height: 50px; width: 50px;\"></div>\
         </div>",
    );
    // 树形：root[1]=body → 容器；容器子序 [float, p, float]。
    let container = find_by_depth(&result.root, &[1, 0]);
    assert!(
        container.margin_top > container.declared_margin_top + 0.01,
        "前置形态：容器 margin 应吸收首流内子链顶 mt（膨胀签名），实得 {} vs declared {}",
        container.margin_top,
        container.declared_margin_top
    );
    let p = &container.children[1];
    assert!(
        p.y.abs() < 0.5,
        "首流内子 p 应位于 content 原点（mt 已折进容器，不本级重放），实得 y={}",
        p.y
    );
    let flt = &container.children[2];
    assert!(
        (flt.y - 36.0).abs() < 0.5,
        "后继 float 外顶 = p margin-box 底 20+16=36（修前 floor 污染 52），实得 y={}",
        flt.y
    );
}

/// 无 float 前置守卫：同构容器去掉首个 float——taffy 原生父↔子折叠、无重放，
/// p y=0、无后继 float 可钳（流内尾块不受影响）。守护门不误伤普通折叠树。
#[test]
fn r5228_plain_container_unchanged() {
    let result = layout(
        "<div style=\"width: 400px;\">\
           <p style=\"margin: 16px 0; font-size: 16px; line-height: 20px;\">hint</p>\
         </div>",
    );
    let container = find_by_depth(&result.root, &[1, 0]);
    let p = &container.children[0];
    assert!(
        p.y.abs() < 0.5,
        "无 float 前置：p 位于 content 原点（既有折叠语义），实得 y={}",
        p.y
    );
}
