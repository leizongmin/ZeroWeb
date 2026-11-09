//! R5477 kill-switch 回退锚（独立测试进程——`ZW_BFC_ROOT_FLOAT_DISPLACE=0` 时
//! 块级 BFC 根不再做 §9.5 float 位移终末校正，既有旁置机制产物原样保留；
//! 进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 `src/engine/tests/r5477_bfc_root_displace_tests.rs` driving 形
//!（css-contain/contain-inline-size-bfc-floats-001）。旧径证据：#contain 停在
//! 旁置钉位 x=250 / y=float1 底 100（与 float3 带 [200,300] 全盒高重叠＝目标
//! 行为失败的旧版本状态），容器高维持旧值 270。

use zero_dom::NodeId;
use zero_layout_engine::{LayoutBox, LayoutEngine};
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

fn find_box(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
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

#[test]
fn kill_switch_keeps_pre_displace_beside_position() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；首读前生效。
    unsafe { std::env::set_var("ZW_BFC_ROOT_FLOAT_DISPLACE", "0") };

    let doc = zero_dom::parse_html(BFC_FLOATS_001_HTML);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(BFC_FLOATS_001_HTML));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let outer_id = doc.get_element_by_id("outer").expect("#outer 存在");
    let contain_id = doc.get_element_by_id("contain").expect("#contain 存在");

    let outer = find_box(&result.root, outer_id).expect("#outer 布局盒存在");
    assert!(
        (outer.height - 270.0).abs() <= 0.5,
        "开关关闭：容器高维持旧径 270（float 域底 300 不入 auto 高），实际 {}",
        outer.height
    );

    let contain = find_box(&result.root, contain_id).expect("#contain 布局盒存在");
    assert!(
        (contain.x - 250.0).abs() <= 0.5 && (contain.y - 100.0).abs() <= 0.5,
        "开关关闭：#contain 维持旧径旁置钉位 (250, 100)（仍与 float3 带重叠＝旧版本行为失败证据），实际 ({}, {})",
        contain.x,
        contain.y
    );
}
