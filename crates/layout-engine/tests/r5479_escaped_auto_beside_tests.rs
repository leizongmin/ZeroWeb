//! R5479：auto 宽 BFC 根 escaped beside 收缩形常驻回归（CSS2 §9.5 × §10.3.3）。
//! chromium oracle 定谳（google-chrome headless 截图逐行扫描，escaped 形与直接子
//! float 同模型）：beside＋width:auto 子形收缩到 float 旁可用带宽旁置（左 float
//! (100,0) 宽 350、右 float (0,0) 宽 350、双侧夹 (100,0) 宽 250）；收缩带宽 <
//! 框架和（border+padding 30）时下绕 float 底全 CB（tight 门，左/右镜像 (0,50)
//! 宽 450）。修复在 R5478 终末位移 pass（主链排斥循环先于递归定谳拿不到权威
//! escaped 几何，时序不可行）。直接子 float 域的 tight 门与双侧 coord 缺口独立
//! 切片（oracle 已定谳 chromium (0,50) 宽 450 / (100,0) 宽 250，ZW 现状
//! (440,0) 宽 10 / (0,50) 宽 350＝归因在册）。kill-switch
//! `ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED=0` 全回退（锚 r5478_escaped_kill_switch.rs）。

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

/// want = (x, y, w)：border-box 锚（chromium PIL 量测同口径）。
fn probe(html: &str, tag: &str, want: (f32, f32, f32)) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(460.0, 220.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(460.0, 220.0);
    let result = engine.compute(&doc, &styles);
    let nid = doc.get_element_by_id("flowroot").expect("flowroot 存在");
    let b = find_box(&result.root, nid).expect("flowroot 布局盒存在");
    assert!(
        (b.x - want.0).abs() <= 0.5 && (b.y - want.1).abs() <= 0.5 && (b.width - want.2).abs() <= 0.5,
        "{tag}: flowroot 应在 ({}, {}) 宽 {}（chromium oracle 定谳），实际 ({}, {}) 宽 {}",
        want.0,
        want.1,
        want.2,
        b.x,
        b.y,
        b.width
    );
}

#[test]
fn r5479_escaped_auto_beside_shapes() {
    // 左 float beside 收缩：float 100×50 逃逸 → (100, 0) 宽 350（oracle vd-direct 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.f{float:left;width:100px;height:50px;background:cyan}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="f"></div></div><div id="flowroot"></div></div></body></html>"#,
        "left-beside",
        (100.0, 0.0, 350.0),
    );
    // 右 float beside 收缩：float right 100 → (0, 0) 宽 350（oracle e2/e4 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.f{float:right;width:100px;height:50px;background:cyan}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="f"></div></div><div id="flowroot"></div></div></body></html>"#,
        "right-beside",
        (0.0, 0.0, 350.0),
    );
    // 左 float tight 下绕：float 440（可用宽 10 < frame 30）→ (0, 50) 宽 450（oracle e1/e6 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.f{float:left;width:440px;height:50px;background:cyan}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="f"></div></div><div id="flowroot"></div></div></body></html>"#,
        "left-tight-wrap",
        (0.0, 50.0, 450.0),
    );
    // 右 float tight 下绕镜像（oracle e5/e7 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.f{float:right;width:440px;height:50px;background:cyan}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="f"></div></div><div id="flowroot"></div></div></body></html>"#,
        "right-tight-wrap",
        (0.0, 50.0, 450.0),
    );
    // 双侧夹 beside 收缩：left 100 + right 100 逃逸 → (100, 0) 宽 250（oracle e3/e8 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.fl{float:left;width:100px;height:50px;background:cyan}.fr{float:right;width:100px;height:50px;background:hotpink}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="fl"></div><div class="fr"></div></div><div id="flowroot"></div></div></body></html>"#,
        "both-sides-clamp",
        (100.0, 0.0, 250.0),
    );
    // tight 分界恰等侧：float 350（可用宽 100 ≥ frame 30）→ 旁置收缩 (350, 0) 宽 100（oracle vd-tight 同形）。
    probe(
        r#"<html><head><style>body{margin:0}#host{width:450px}.f{float:left;width:350px;height:50px;background:cyan}#flowroot{display:flow-root;border:15px solid black;height:80px}</style></head><body><div id="host"><div id="fakegroup"><div class="f"></div></div><div id="flowroot"></div></div></body></html>"#,
        "tight-boundary-beside",
        (350.0, 0.0, 100.0),
    );
}
