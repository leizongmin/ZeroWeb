//! R5477：块级 BFC 根 float 位移终末校正（CSS2 §9.5 chromium FloatAvoider 语义）。
//!
//! driving 形（css-contain/contain-inline-size-bfc-floats-001）：#outer(400) 内
//! float1(right 200×100) + float2(left 250×100) + float3(right 300×100) +
//! #contain(display:flow-root + width:fit-content + contain:inline-size → w=0,
//! 内容 orange 200×20 + filler 150 → h=170)。既有旁置机制把 #contain 停在
//! x=250/y=float1 底（与 float3 带重叠）；chromium oracle 定谳终位 = 容器相对
//! (0, 200)（逐层 float 底边界 × 候选 x {流内原点, 旁置左 float 右缘} 首配即停，
//! 零宽盒左缘按 float_left ≤ x < float_right 计入），auto 高容器涨高到 370。
//! kill-switch `ZW_BFC_ROOT_FLOAT_DISPLACE=0`（独立进程锚 tests/r5477_bfc_root_
//! displace_kill_switch.rs）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 手抠 <style> 文本（compute_styles 不加载 style 元素，r5167/r5475 同口径）。
fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &LayoutBox, id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

const BFC_FLOATS_001_HTML: &str = r#"<html><head><style>
  .float { float: left; }
  .right { float: right; }
  #outer { width: 400px; }
  #float1 { width: 200px; height: 100px; }
  #float2 { width: 250px; height: 100px; }
  #float3 { width: 300px; height: 100px; }
  #contain { contain: inline-size; display: flow-root; width: fit-content; line-height: 1em; }
  #filler { height: 150px; }
  #orange { display: inline-block; width: 200px; height: 20px; }
</style></head><body style="margin:0">
<div id="outer">
  <div id="float1" class="float right"></div>
  <div id="float2" class="float left"></div>
  <div id="float3" class="float right"></div>
  <div id="contain"><span id="orange"></span><div id="filler"></div></div>
</div>
</body></html>"#;

/// driving 形：#contain 位移到容器相对 (0, 200)（float3 带顶 = float2 底边界层，
/// 流内原点候选），零宽不与 float3 重叠（0 < float3 左缘 100）；容器 auto 高
/// 涨到 370 覆盖新内容底；orange 随子树平移到 (0, 200)（200 宽溢出可见域）。
#[test]
fn r5477_flow_root_displaced_below_float_band() {
    let doc = zero_dom::parse_html(BFC_FLOATS_001_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(BFC_FLOATS_001_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_id = doc.get_element_by_id("outer").expect("#outer 存在");
    let contain_id = doc.get_element_by_id("contain").expect("#contain 存在");
    let orange_id = doc.get_element_by_id("orange").expect("#orange 存在");

    let outer = find_box(&result.root, outer_id).expect("#outer 布局盒存在");
    assert!(
        (outer.height - 370.0).abs() <= 0.5,
        "auto 高容器应涨高到 370 覆盖位移后内容底（chromium oracle 同值），实际 {}",
        outer.height
    );

    let contain = find_box(&result.root, contain_id).expect("#contain 布局盒存在");
    assert!(
        (contain.x - 0.0).abs() <= 0.5 && (contain.y - 200.0).abs() <= 0.5,
        "#contain 应位移到容器相对 (0, 200)（chromium FloatAvoider 定谳），实际 ({}, {})",
        contain.x,
        contain.y
    );
    assert!(
        (contain.width - 0.0).abs() <= 0.5 && (contain.height - 170.0).abs() <= 0.5,
        "contain:inline-size 宽 0 不变（本 pass 只校正位置），盒高 170 不变，实际 {}×{}",
        contain.width,
        contain.height
    );

    let orange = find_box(&result.root, orange_id).expect("#orange 布局盒存在");
    // 布局盒坐标为父相对：orange 在位移后的 #contain 内保持 (0, 0)＝页面 (0, 200)
    //（子树随父平移，pass 不改子内坐标）。
    assert!(
        (orange.x - 0.0).abs() <= 0.5 && (orange.y - 0.0).abs() <= 0.5,
        "orange 保持 #contain 相对 (0, 0)（父相对坐标；页面位随父到 (0, 200)），实际 ({}, {})",
        orange.x,
        orange.y
    );
}

const BESIDE_UNTOUCHED_HTML: &str = r#"<html><head><style>
  #host { width: 300px; }
  #f { float: left; width: 150px; height: 150px; }
  #bfc { overflow: hidden; width: 200px; height: 150px; }
</style></head><body style="margin:0">
<div id="host">
  <div id="f"></div>
  <div id="bfc"></div>
</div>
</body></html>"#;

/// 不触碰不变式：既有机制已落非重叠位的 BFC 根不被本 pass 再动（chrome oracle
/// 定谳：300 容器内 float 150 + 显式宽 200 的 BFC 根旁置空间不足 → 下绕 float 底
/// (0, 150)；该位与 float 带无重叠，本 pass 触发门不亮，原样保留）。
#[test]
fn r5477_beside_bfc_root_untouched() {
    let doc = zero_dom::parse_html(BESIDE_UNTOUCHED_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(BESIDE_UNTOUCHED_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_id = doc.get_element_by_id("bfc").expect("#bfc 存在");
    let bfc = find_box(&result.root, bfc_id).expect("#bfc 布局盒存在");
    assert!(
        (bfc.x - 0.0).abs() <= 0.5 && (bfc.y - 150.0).abs() <= 0.5,
        "旁置空间不足时既有机制下绕 float 底 (0, 150)（chrome oracle 同值），本 pass 不再触碰，实际 ({}, {})",
        bfc.x,
        bfc.y
    );
}

const PUSHED_SIBLING_HEIGHT_HTML: &str = r#"<html><head><style>
  .float { float: left; }
  .right { float: right; }
  #outer { width: 400px; }
  #float1 { width: 200px; height: 100px; }
  #float2 { width: 250px; height: 100px; }
  #float3 { width: 300px; height: 100px; }
  #contain { contain: inline-size; display: flow-root; width: fit-content; line-height: 1em; }
  #filler { height: 150px; }
  #orange { display: inline-block; width: 200px; height: 20px; }
  #tail { height: 80px; }
</style></head><body style="margin:0">
<div id="outer">
  <div id="float1" class="float right"></div>
  <div id="float2" class="float left"></div>
  <div id="float3" class="float right"></div>
  <div id="contain"><span id="orange"></span><div id="filler"></div></div>
  <div id="tail"></div>
</div>
</body></html>"#;

/// 位移＋后继兄弟随推＋auto 高包底：driving form 加尾随普通块 #tail(80 高)。
/// #contain 位移到 (0, 200) 后 #tail 随推 +100 到 y=370；容器 auto 高须涨到
/// 450 = 被推 #tail 底（370+80），不只看位移盒自身底（370）——needed 取
/// 位移盒与被推兄弟 margin-box 底最大值（基线：contain 停 (250,0) 重叠
/// float3 带、tail 170、outer 250）。
#[test]
fn r5477_pushed_sibling_extends_container_auto_height() {
    let doc = zero_dom::parse_html(PUSHED_SIBLING_HEIGHT_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(PUSHED_SIBLING_HEIGHT_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_id = doc.get_element_by_id("outer").expect("#outer 存在");
    let contain_id = doc.get_element_by_id("contain").expect("#contain 存在");
    let tail_id = doc.get_element_by_id("tail").expect("#tail 存在");

    let contain = find_box(&result.root, contain_id).expect("#contain 布局盒存在");
    assert!(
        (contain.x - 0.0).abs() <= 0.5 && (contain.y - 200.0).abs() <= 0.5,
        "#contain 应位移到容器相对 (0, 200)，实际 ({}, {})",
        contain.x,
        contain.y
    );
    let tail = find_box(&result.root, tail_id).expect("#tail 布局盒存在");
    assert!(
        (tail.y - 370.0).abs() <= 0.5,
        "后继流内兄弟应随推 +100 到 y=370，实际 {}",
        tail.y
    );
    let outer = find_box(&result.root, outer_id).expect("#outer 布局盒存在");
    assert!(
        (outer.height - 450.0).abs() <= 0.5,
        "auto 高容器应涨到 450 包住被推兄弟底（370+80），实际 {}",
        outer.height
    );
}
