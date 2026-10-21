//! R5212（CSS2 §9.5.2 forced clearance × §8.3.1）：direct-float 邻接吸收。
//! clear 盒为本帧首流内子时，其邻接 top-margin 链经帧顶上折、在帧上方物化——
//! 帧内 float 连同链顶盒被整体下拉（链与 float adjoining）——引入 forced
//! clearance：float 保持原位，clear 盒 border edge 贴 float 底，整条邻接链
//! 吸收（Chromium adjoining-float-* 测试族互操作语义：「No matter how large
//! the margin is, it should still be just below the float」）。非首流内子
//! （链与前驱兄弟折叠，float 置于自身 DOM 点不被拉动）与帧顶 border/padding
//! （链不出帧）不吸收——由 tests_10 empty-cleared 与 R5211 separation ② 锚定。
//! 吸收后首流内子的字段 mt 已随整链上折，本级重放抑制（R5210 链值折叠语义）。
//! kill-switch `ZW_FORCED_CLEAR_ABSORB=0` 整体回退。
//! corpus 定谳：980 挂（HEAD 982），FIXED=2 NEW=0——
//! adjoining-float-nested-forced-clearance（2.08%）、adjoining-float-new-fc
//!（6.25%）两案 0.00% 像素级翻绿。

use crate::engine::LayoutEngine;
use crate::types::{ClearValue, LayoutBox};
use zero_css_parser::values::FloatValue;
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

/// 按谓词递归找盒（本组每树唯一命中）。
fn find_box<'a>(root: &'a LayoutBox, pred: &impl Fn(&LayoutBox) -> bool) -> Option<&'a LayoutBox> {
    if pred(root) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_box(c, pred))
}

/// adjoining-float-new-fc（Morten Stenshorne）：BFC clear 盒 mt:300 邻接吸收。
/// 容器(overflow:hidden 红) > div > [F1 float 100×50 绿；B mt:300 clear:left
/// overflow:hidden 100×50 绿]。修前 taffy 槽解析把 B 留在 y=0（BFC 盒走
/// find_content_slot，全宽 float 之下才有自由空间但槽仍报 y=0 全宽），下游
/// else 臂再以字段 mt:300 推到 300——红盒 50..300 外露。修后 forced clearance
/// 吸收：B 贴 float 底 50，红盒无缺口。
#[test]
fn r5212_new_fc_bfc_clear_absorbed_at_float_bottom() {
    let result = layout(
        "<div style=\"width:100px; overflow:hidden; background:red;\">\
           <div>\
             <div style=\"float:left; width:100px; height:50px; background:green;\"></div>\
             <div style=\"margin-top:300px; clear:left; overflow:hidden; width:100px; height:50px; background:green;\"></div>\
           </div>\
         </div>",
    );
    let b = find_box(&result.root, &|c| c.clear == ClearValue::Left).expect("B 未找到");
    assert!(
        (b.y - 50.0).abs() < 0.5,
        "clear 盒贴 float 底 50（mt:300 邻接吸收），实得 y={}",
        b.y
    );
    assert!(
        (b.height - 50.0).abs() < 0.5,
        "clear 盒高 50 覆盖红盒下半，实得 h={}",
        b.height
    );
    let f1 = find_box(&result.root, &|c| {
        c.float != FloatValue::None && (c.height - 50.0).abs() < 0.5
    })
    .expect("F1 未找到");
    assert!(
        (f1.y - 0.0).abs() < 0.5,
        "float 保持原位 0..50（forced clearance 不拉动 float），实得 y={}",
        f1.y
    );
}

/// adjoining-float-nested-forced-clearance：clear 盒嵌套链值吸收 + 首子重放抑制。
/// abspos 容器 > div > [F1 float 100×50 绿；B clear:both > [F2 float 10×10 无色；
/// G mt:100 h:50 绿]]。B 的邻接链（塌穿链 + G 的 mt:100 经 R5210 折入 B 字段）
/// 与 F1 adjoining：B 贴 float 底 50（字段 mt=100 > dmt=0 折叠签名），G 的本级
/// 重放抑制归零（修前 G 在 B 内重放 100，B/G 双重下移，仅 2.08% 变体过）。
#[test]
fn r5212_nested_forced_clearance_absorbs_chain_and_suppresses_replay() {
    let result = layout(
        "<div style=\"position: absolute; width: 100px; height: 100px; background: red;\"></div>\
         <div style=\"position: absolute; width: 100px;\">\
           <div>\
             <div style=\"float:left; width:100px; height:50px; background: green;\"></div>\
             <div style=\"clear:both;\">\
               <div style=\"float:left; width:10px; height:10px;\"></div>\
               <div style=\"margin-top:100px; height:50px; background: green;\"></div>\
             </div>\
           </div>\
         </div>",
    );
    let b = find_box(&result.root, &|c| c.clear == ClearValue::Both).expect("B 未找到");
    assert!(
        (b.y - 50.0).abs() < 0.5,
        "clear 盒贴 float 底 50（嵌套链值吸收），实得 y={}",
        b.y
    );
    assert!(
        b.margin_top > b.declared_margin_top + 0.01,
        "字段 mt=100 > dmt=0 折叠签名（链值吸收证据），实得 mt={} dmt={}",
        b.margin_top,
        b.declared_margin_top
    );
    let g = b.children.last().expect("G 存在");
    assert!(
        (g.y - 0.0).abs() < 0.5 && (g.height - 50.0).abs() < 0.5,
        "G 本级重放抑制归零（rel y=0 h=50，随 B 覆盖下半），实得 y={} h={}",
        g.y,
        g.height
    );
}

/// negative-clearance-after-adjoining-float（corpus 已绿对照，锁死）：CL mt:200
/// clear:left 空盒直邻 float——forced clearance 净效果为负（clearance = 50−200
/// = −150），CL 贴 float 底 50、整链吸收，容器高不被 200 margin 撑开。
#[test]
fn r5212_negative_forced_clearance_lands_at_float_bottom() {
    let result = layout(
        "<div style=\"width:100px;\">\
           <div style=\"float:left; width:100px; height:50px; background:green;\"></div>\
           <div style=\"clear:left; margin-top:200px;\"></div>\
         </div>\
         <div style=\"width:100px; height:50px; background:green;\"></div>",
    );
    let cl = find_box(&result.root, &|c| c.clear == ClearValue::Left).expect("CL 未找到");
    assert!(
        (cl.y - 50.0).abs() < 0.5 && cl.height.abs() < 0.5,
        "CL 贴 float 底 50（负 clearance，mt:200 吸收），实得 y={} h={}",
        cl.y,
        cl.height
    );
}
