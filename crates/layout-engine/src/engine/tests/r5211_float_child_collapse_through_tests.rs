//! R5211（CSS2 §8.3.1 × §9.5.1/§9.5.2）：float 子不阻断父盒 collapse-through
//! + 塌穿链分离点断链门。float 属 out of flow，不算父盒的 in-flow 内容——
//! 仅含 float 子的父盒上下 margin 相邻（self-collapsing），塌穿照常；塌穿链
//! 上的首个非自塌项若恰逢「分离点」（clear 引入 clearance，或非 same-BFC 盒
//! 被 float 推开），其 top margin 自分离处起不再折入 first-child 链逃逸。
//! kill-switch `ZW_FLOAT_CHILD_COLLAPSE_THROUGH=0` 整体回退。
//! corpus 定谳：992→982 净 −10（margin-collapse-114 等 FIXED=10、NEW=0）。

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

/// margin-collapse-114 第二 cell 形态：[.a{mb:50,h:50}][.b{mt:100}⤳float][.c{mt:150}]。
/// float-only .b 自塌（§8.3.1）：.a(50)/.b(100)/.c(150) 三段 margin 折叠为一段
/// 150——.b 顶 150、.c 顶 = 50+150 = 200；float 不随塌穿重放 mt（修前拆两段
/// 重放，.c 高一段至 350），置于 .b 假想内容位 + 自身 mt 100 = 250。
#[test]
fn r5211_float_only_child_self_collapses() {
    let result = layout(
        "<div style=\"width:200px\">\
           <div style=\"margin:0 0 50px 0; height:50px; background:yellow\"></div>\
           <div style=\"margin:100px 0 0 0\">\
             <div style=\"float:left; margin:100px 0 50px 0; height:50px; width:100%; background:aqua\"></div>\
           </div>\
           <div style=\"margin:150px 0 0 0; height:50px; background:lime\"></div>\
         </div>",
    );
    let wrapper = find_by_depth(&result.root, &[1, 0]);
    let b = &wrapper.children[1];
    assert!(
        (b.y - 150.0).abs() < 0.5 && b.height.abs() < 0.5,
        "float-only .b 自塌 y=150 h=0，实得 y={} h={}",
        b.y,
        b.height
    );
    let c = &wrapper.children[2];
    assert!(
        (c.y - 200.0).abs() < 0.5,
        ".c 顶 = .a 底 50 + 三段折叠 margin 150 = 200（修前重放多一段 350），实得 y={}",
        c.y
    );
    let float_box = &b.children[0];
    assert!(
        (float_box.y - 100.0).abs() < 0.5,
        "float 置于 .b 假想内容位 + 自身 mt 100（rel 100 / abs 250），实得 y={}",
        float_box.y
    );
}

/// 分离门两支分治：塌穿链上的首个非自塌项逢分离点断链。
/// ① adjoining-float-before-clearance：float 在链内（红盒子树所置），clear 盒
///    mt:400 引入 clearance——margin 不逃逸拖 float 下沉，盒顶贴 float 底 50，
///    红盒无缺口 h=100（修前逃逸出 400 缺口）。
/// ② no-clearance-due-to-large-margin：float 在链外（body 层前兄），150 margin
///    被 padding 容器拦停后真实下推红盒过 float 底——规范不引入 clearance，
///    白盒盖满红盒（修前误断链致红盒暴露）。
#[test]
fn r5211_separation_gate_breaks_only_chain_internal_floats() {
    // ① clear 支击发：float 在塌穿链内。
    let adjoining = layout(
        "<div style=\"width: 100px; background: red;\">\
           <div>\
             <div style=\"float: left; width: 100px; height: 50px; background: green;\"></div>\
           </div>\
           <div style=\"margin-top: 400px; clear: left; height: 50px; background: green;\"></div>\
         </div>",
    );
    let red_box = find_by_depth(&adjoining.root, &[1, 0]);
    assert!(
        (red_box.height - 100.0).abs() < 0.5,
        "clear 盒贴 float 底后红盒 h=100（修前 margin 逃逸缺口 450），实得 h={}",
        red_box.height
    );
    let cleared = &red_box.children[1];
    assert!(
        (cleared.y - 50.0).abs() < 0.5,
        "clear 盒顶 = float 底 50（§9.5.2 Example 1 clearance 净效果），实得 y={}",
        cleared.y
    );
    // ② clear 支不击发：float 在链外（前兄子树所置）。
    let no_clearance = layout(
        "<div style=\"float:left; width:100px; height:100px; background:green;\"></div>\
         <div style=\"padding-top:1px;\">\
           <div style=\"background:red;\">\
             <div style=\"clear:left; background:white; margin-top:150px;\"><br></div>\
           </div>\
         </div>",
    );
    let container = find_by_depth(&no_clearance.root, &[1, 1]);
    let red_box = &container.children[0];
    let white = &red_box.children[0];
    assert!(
        (red_box.y - 150.0).abs() < 0.5,
        "150 margin 真实下推红盒至 150（float 底 100 之下，规范不引入 clearance），实得 y={}",
        red_box.y
    );
    assert!(
        white.y.abs() < 0.5 && (white.height - red_box.height).abs() < 0.5,
        "白盒盖满红盒（rel y=0 贴红盒顶且高相等），实得白 y={} h={} 红 y={} h={}",
        white.y,
        white.height,
        red_box.y,
        red_box.height
    );
}
