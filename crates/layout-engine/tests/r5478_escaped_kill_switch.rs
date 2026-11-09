//! R5478 kill-switch 回退锚（独立测试进程——`ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED=0`
//! 时 escaped floats 不并入候选域，非 BFC 根块内 float 不影响后继 BFC 根兄弟定位；
//! 进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测）。
//!
//! 谱系：V2 旁置形（chromium oracle (50,0)）；旧径证据＝R5477 直子形切片下 #flowroot
//! 停流内原点 (0, 0) 与 float 带盖绘（escaped 形行为失败的旧版本状态）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

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

#[test]
fn kill_switch_keeps_pre_displace_flow_origin() {
    unsafe { std::env::set_var("ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 50px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; width: 300px; }
</style></head>
<body>
<div id="host">
  <div id="fakegroup">
    <div class="f" style="background: cyan;"></div>
  </div>
  <div id="flowroot"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(500.0, 220.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(500.0, 220.0);
    let result = engine.compute(&doc, &styles);

    let nid = doc.get_element_by_id("flowroot").expect("#flowroot 存在");
    let b = find_box(&result.root, nid).expect("#flowroot 布局盒存在");
    assert!(
        (b.x - 0.0).abs() <= 0.5 && (b.y - 0.0).abs() <= 0.5,
        "开关关闭：#flowroot 维持流内原点 (0, 0)（与 float 带盖绘＝escaped 形行为失败的旧版本状态），实际 ({}, {})",
        b.x,
        b.y
    );
}
