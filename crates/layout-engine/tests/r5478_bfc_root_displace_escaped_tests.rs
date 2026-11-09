//! R5478：escaped-float 臂常驻回归（CSS2 §9.5 escaped 形——非 BFC 根块内 float
//! 逃逸影响后继 BFC 根兄弟）。chromium oracle V1–V4 形态定谳（google-chrome
//! headless 截图）：V2 旁置 (50,0)、V3 下绕 (0,50) 与 chromium 精确一致；V1
//! beside＋width:auto 子形 chromium 收缩到 (100,0) 宽 350，本 pass position-only
//! 保守下绕 (0,50)（宽度收缩域待独立切片）；V4 按 ZW 自身几何下绕 (0,50)
//! （margin-collapse 顶推差值＝上游既有差异另记）。kill-switch 锚见
//! tests/r5478_escaped_kill_switch.rs（独立进程，R3858 纪律）。

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

fn probe(html: &str, tag: &str, want: (f32, f32)) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(500.0, 220.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(500.0, 220.0);
    let result = engine.compute(&doc, &styles);
    let nid = doc.get_element_by_id("flowroot").expect("flowroot 存在");
    let b = find_box(&result.root, nid).expect("flowroot 布局盒存在");
    println!(
        "{tag}: flowroot x={} y={} w={} h={} (want ({}, {}))",
        b.x, b.y, b.width, b.height, want.0, want.1
    );
    assert!(
        (b.x - want.0).abs() <= 0.5 && (b.y - want.1).abs() <= 0.5,
        "{tag}: flowroot 应在 ({}, {})（chromium oracle 定谳），实际 ({}, {})",
        want.0,
        want.1,
        b.x,
        b.y
    );
}

const HEAD: &str = r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 50px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px;"#;

#[test]
fn r5478_escaped_arm_four_shapes() {
    // V1：两 float 50×50 逃逸，flow-root width:auto。chromium oracle＝旁置＋auto 宽
    // 收缩到 float 旁可用空间 (100, 0) 宽 350；本臂 position-only（无宽度写），旁置
    // 判据用未收缩宽（100+480>450 不装）→ 保守下绕 (0, 50)。**已知残差**：beside
    // ＋auto 宽收缩子形（显式宽不收缩——V3/V4 实证），修复须 auto 宽解算域，非
    // 本测量面。
    probe(
        &format!(
            "{HEAD} }}</style></head><body><div id=\"host\"><div id=\"fakegroup\"><div class=\"f\" style=\"background:cyan\"></div><div class=\"f\" style=\"background:hotpink\"></div></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "V1",
        (0.0, 50.0),
    );

    // V2：单 float，300 宽旁置装得下 → x=50
    probe(
        &format!(
            "{HEAD} width: 300px; }}</style></head><body><div id=\"host\"><div id=\"fakegroup\"><div class=\"f\" style=\"background:cyan\"></div></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "V2",
        (50.0, 0.0),
    );

    // V3：两 float 100×50（右缘 200），400 宽装不下 → 下绕 float 底 y=50
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 100px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; width: 400px; }
</style></head>
<body>
<div id="host">
  <div id="fakegroup">
    <div class="f" style="background: cyan;"></div>
    <div class="f" style="background: hotpink;"></div>
  </div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "V3",
        (0.0, 50.0),
    );

    // V4：中间普通块 margin 100/20 嵌套偏移。chromium＝collapse 顶推 host 到 y=20、
    // float 带几何 (100..150, 20..70)、下绕 y=70。ZW 侧 host.y=0（collapse 差值未
    // 视觉应用＝上游 margin-collapse 既有差异，另行记录），float 带 (100..150,
    // 0..50)——本臂按 ZW 自身几何下绕 y=50（收集链 content_x/y 累加自洽验证）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
#mid { margin-left: 100px; margin-top: 20px; width: 350px; }
.f { float: left; width: 50px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; width: 300px; }
</style></head>
<body>
<div id="host">
  <div id="mid">
    <div id="fakegroup">
      <div class="f" style="background: cyan;"></div>
    </div>
  </div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "V4",
        (0.0, 50.0),
    );
}
