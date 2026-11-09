//! R5479 auto 宽 beside 收缩 kill-switch 回退锚（独立测试进程——
//! `ZW_BFC_AUTO_BESIDE_MODEL=0` 时 beside＋width:auto 收缩形回退 R5478 旧行为：
//! position-only 下绕 float 底、不写宽；进程内 set_var 与并行单测互踩，R3858
//! 教训——本文件仅此一测）。
//!
//! oracle 几何同 tests/r5479_escaped_auto_beside_tests.rs left-beside 形
//!（chromium (100, 0) 宽 350）。旧径证据＝R5478 保守残差位 (0, 50) 全宽 450。

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
fn auto_beside_kill_switch_wraps_without_width_write() {
    unsafe { std::env::set_var("ZW_BFC_AUTO_BESIDE_MODEL", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 50px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
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
    sys.set_viewport(460.0, 220.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(460.0, 220.0);
    let result = engine.compute(&doc, &styles);

    let nid = doc.get_element_by_id("flowroot").expect("#flowroot 存在");
    let b = find_box(&result.root, nid).expect("#flowroot 布局盒存在");
    assert!(
        (b.x - 0.0).abs() <= 0.5 && (b.y - 50.0).abs() <= 0.5 && (b.width - 450.0).abs() <= 0.5,
        "开关关闭：beside＋width:auto 形回退 R5478 旧径下绕 (0, 50) 全宽 450（chromium (100, 0) 宽 350＝开启态），实际 ({}, {}) 宽 {}",
        b.x,
        b.y,
        b.width
    );
}
