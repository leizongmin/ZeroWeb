//! R5478/R5479：escaped-float 臂常驻回归（CSS2 §9.5 escaped 形——非 BFC 根块内
//! float 逃逸影响后继 BFC 根兄弟）。chromium oracle 形态定谳（google-chrome
//! headless 截图逐行扫描）：V2 旁置 (50,0)、V3 下绕 (0,50) 与 chromium 精确一致；
//! R5479 起 beside＋width:auto 收缩子形并入终末臂（V1 chromium (100,0) 宽 350
//! 精确一致；E4 右 float (0,0) 宽 350、E3 双侧 (100,0) 宽 250、E1 tight 下绕见
//! r5479 回归）；V4 按 ZW 自身几何下绕 (0,50)（margin-collapse 顶推差值＝上游
//! 既有差异另记）。kill-switch 锚见 tests/r5478_escaped_kill_switch.rs（独立
//! 进程，R3858 纪律）。

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

const HEAD: &str = r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 50px; height: 50px; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px;"#;

#[test]
fn r5478_escaped_arm_four_shapes() {
    // V1：两 float 50×50 逃逸，flow-root width:auto。chromium oracle＝旁置收缩
    // (100, 0) border-box 宽 350（＝容器内容右缘 − 左 float margin-box 右缘；
    // R5479 auto 宽 beside 收缩形定谳，原 position-only 保守下绕 (0,50) 残差销账）。
    probe(
        &format!(
            "{HEAD} }}</style></head><body><div id=\"host\"><div id=\"fakegroup\"><div class=\"f\" style=\"background:cyan\"></div><div class=\"f\" style=\"background:hotpink\"></div></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "V1",
        (100.0, 0.0, 350.0),
    );

    // V2：单 float，显式宽 300 旁置装得下 → x=50（declared-width 维持原判据，宽不变）。
    probe(
        &format!(
            "{HEAD} width: 300px; }}</style></head><body><div id=\"host\"><div id=\"fakegroup\"><div class=\"f\" style=\"background:cyan\"></div></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "V2",
        (50.0, 0.0, 330.0),
    );

    // V3：两 float 100×50（右缘 200），显式宽 400 装不下 → 下绕 float 底 y=50。
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
        (0.0, 50.0, 430.0),
    );

    // V4：中间普通块 margin 100/20 嵌套偏移，flowroot 显式宽 300（border-box 330
    // beside 判据 150+330>450 不可行）。chromium＝collapse 顶推下绕 y=70；ZW 按
    // 自身几何（host.y=0＋mt=20 未视觉应用＝上游 margin-collapse 既有差异另记）
    // 下绕 (0, 50)——收集链 content_x/y 累加自洽验证。
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
        (0.0, 50.0, 330.0),
    );
}
