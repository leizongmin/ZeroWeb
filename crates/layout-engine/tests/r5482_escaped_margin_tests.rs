//! R5482：终末臂 escaped＋margin≠0 混合形常驻回归（CSS2 §9.5×§10.3.3 margin-aware
//! 钉位解原样适用 escaped 域——收缩右界夹 margin-right：right = min(CB − mr,
//! band_right)、w = right − cx；mr=0 时与 R5479 九形 margin:0 域公式同值零变化）。
//!
//! chromium headless oracle 三形定谳（google-chrome 截图 PIL 行级黑框扫描，会话
//! 临时探针页，fakegroup 普通块内 float 逃逸＋flowroot margin≠0）：M1 escaped
//! 左 float 440＋mr=10 → 下绕 float 底 (0,50) 宽 CB−margin_h=440、M2 escaped 左
//! float 100＋mr=10 → beside 收缩 (100,0) 宽 band−mr=340、M3 escaped 左 float
//! 100＋ml=10 → (100,0) 宽 350（面向侧 margin 被钉位吃掉不缩宽）。ZW pre-fix
//! 现状＝M2 (100,0) 宽 350（收缩右界未夹 mr＝本轮唯一缺口；M1 taffy 全 CB 解
//! 已减 margin、wrap 层不写宽恰好精确；M3 精确）。
//!
//! 与 R5481 主链 margin-aware 域（float_positioning solve_auto_bfc_margin_beside）
//! 互为镜像：本文件锚终末 pass escaped 域（bfc_root_displace），R5481 锚直接子域。
//! kill-switch：`ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED=0`（R5478 臂整体，宽度写同
//! 回退）与 `ZW_BFC_AUTO_BESIDE_MODEL=0`（R5479 宽度写独立回退）覆盖本域。

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
fn r5482_escaped_margin_aware_shapes() {
    // M1 缺口哨：escaped 左 float 440×50（带 [0,440]）＋ mr=10 → 下绕 float 底
    // (0, 50) 宽 440（chromium oracle；wrap 层 w = CB − mr − cx）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 440px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; margin-right: 10px; }
</style></head>
<body>
<div id="host">
  <div id="fakegroup"><div class="f"></div></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "escaped-left-margin-tight",
        (0.0, 50.0, 440.0),
    );

    // M2 缺口锚：escaped 左 float 100×50（带 [0,100] beside=100）＋ mr=10 →
    // beside 收缩 (100, 0) 宽 band−mr=340（chromium oracle；pre-fix 宽 350）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 100px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; margin-right: 10px; }
</style></head>
<body>
<div id="host">
  <div id="fakegroup"><div class="f"></div></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "escaped-left-margin-fit",
        (100.0, 0.0, 340.0),
    );

    // M3 哨：escaped 左 float 100×50 ＋ ml=10 → (100, 0) 宽 350（面向侧 margin
    // 被钉位吃掉不缩宽，chromium oracle；pre-fix 已精确＝零变化哨）。
    probe(
        r#"<html><head><style>
body { margin: 0; }
#host { width: 450px; }
.f { float: left; width: 100px; height: 50px; background: cyan; }
#flowroot { display: flow-root; border: 15px solid black; height: 80px; margin-left: 10px; }
</style></head>
<body>
<div id="host">
  <div id="fakegroup"><div class="f"></div></div>
  <div id="flowroot"></div>
</div>
</body></html>"#,
        "escaped-left-marginleft-fit",
        (100.0, 0.0, 350.0),
    );
}
