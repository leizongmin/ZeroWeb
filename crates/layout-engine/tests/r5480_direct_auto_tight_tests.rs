//! R5480：主链直接子域 beside＋width:auto BFC 根 tight 下绕门常驻回归（CSS2 §9.5×
//! §10.3.3 收缩带模型——收缩带宽 < 框架和（border+padding LR）时 chromium 不旁置、
//! 下绕 float 底全 CB 重解；chromium headless oracle 定谳 vd-extreme-direct (0,50)
//! 宽 450、e5 rf-tight (0,50) 宽 450、双侧夹 (100,0) 宽 250）。
//!
//! 缺口归因（R5479 轮在册两缺口，float_positioning 排斥循环直接子域）：
//! ①左 float 臂 final else（R1369 shrink-to-fit 臂）无 tight 门 → float 440 形旁置
//! 收缩到宽 10（chromium (0,50) 全 CB 450）；②右 float 臂 R1722 pushbelow 门缺
//! `!declared_width_auto`（左臂 R1728 有此门，右臂漏）→ 左臂收缩后的 auto 宽 BFC 被
//! is_definite_width 代理误判 definite → 下绕 (0,50) 宽 350（chromium 双侧夹带内
//! 收缩旁置 (100,0) 宽 250）。旁置守卫锚（vd-direct (100,0) 宽 350、右 float
//! (0,0) 宽 350）＝R5479 轮已精确域零回归哨。
//!
//! kill-switch：`ZW_BFC_AUTO_TIGHT_WRAP=0`（锚见 tests/r5480_auto_tight_kill_switch.rs，
//! 独立进程纪律 R3858）。

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
    sys.set_viewport(500.0, 220.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(500.0, 220.0);
    let result = engine.compute(&doc, &styles);
    let nid = doc.get_element_by_id("flowroot").expect("flowroot 存在");
    let b = find_box(&result.root, nid).expect("flowroot 布局盒存在");
    println!(
        "{tag}: flowroot x={} y={} w={} h={} (want ({}, {}, {}))",
        b.x, b.y, b.width, b.height, want.0, want.1, want.2
    );
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
fn r5480_direct_auto_tight_and_beside() {
    // 缺口①：左 float 440×50（收缩带宽 10 < frame 30）→ tight 门下绕 float 底
    // 全 CB (0, 50) 宽 450（chromium vd-extreme-direct oracle）。
    probe(
        r#"<html><head><style>
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
</body></html>"#,
        "direct-extreme-tight",
        (0.0, 50.0, 450.0),
    );

    // 缺口①右镜像：右 float 440×50（e5 rf-tight oracle 同模型）→ (0, 50) 宽 450。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: right; width: 440px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
</style></head>
<body>
<div id="host">
  <div class="f"></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "direct-rf-tight",
        (0.0, 50.0, 450.0),
    );

    // 缺口②：双侧 float 100×50（带 [100, 350] 宽 250 ≥ frame）→ 带内收缩旁置
    // (100, 0) 宽 250（chromium 双侧夹 oracle）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.fl { float: left; width: 100px; height: 50px; background: cyan; }
.fr { float: right; width: 100px; height: 50px; background: hotpink; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
</style></head>
<body>
<div id="host">
  <div class="fl"></div>
  <div class="fr"></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "direct-both-sides",
        (100.0, 0.0, 250.0),
    );

    // 旁置守卫：左 float 100×50（带 [100, 450] 宽 350 ≥ frame）收缩旁置
    // (100, 0) 宽 350——R5479 轮已精确域零回归哨。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 100px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
</style></head>
<body>
<div id="host">
  <div class="f"></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "direct-beside-guard",
        (100.0, 0.0, 350.0),
    );

    // 旁置守卫右镜像：右 float 100×50 → (0, 0) 宽 350（vd-right oracle，哨）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: right; width: 100px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; }
</style></head>
<body>
<div id="host">
  <div class="f"></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "direct-right-guard",
        (0.0, 0.0, 350.0),
    );
}
