//! R5480 kill-switch 回退锚（独立测试进程——`ZW_BFC_AUTO_TIGHT_WRAP=0` 时主链
//! beside＋width:auto BFC 根 tight 下绕门关闭，左臂 shrink-to-fit 旁置收缩、右臂
//! definite 代理判据旧径；进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测）。
//!
//! 回退态证据＝R5480 缺口①旧版本行为：float 440 形旁置收缩 (440, 0) 宽 10
//!（chromium (0, 50) 全 CB 450＝开启态 r5480 direct-extreme-tight 锚）。

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
fn kill_switch_keeps_pre_tight_wrap_shrink_beside() {
    unsafe { std::env::set_var("ZW_BFC_AUTO_TIGHT_WRAP", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 440px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
</style></head>
<body>
<div id="host">
  <div class="f"></div>
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
        (b.x - 440.0).abs() <= 0.5 && (b.y - 0.0).abs() <= 0.5 && (b.width - 10.0).abs() <= 0.5,
        "开关关闭：tight 门回退旧 shrink-to-fit 旁置收缩 (440, 0) 宽 10（R5480 修复前行为），实际 ({}, {}) 宽 {}",
        b.x,
        b.y,
        b.width
    );
}
