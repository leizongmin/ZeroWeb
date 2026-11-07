//! R5457 vertical-rl 容器孤立正交 float 块流起始（右缘）重锚钉（#33 slice-2 驱动）：
//! float-lft-orthog-htb-in-vrl-002/003 形态——CSS2.1 §9.5.1 + css-writing-modes-3
//! #orthogonal-auto：正交 float 在 vrl 父的块流起始位 = 容器 content 右缘（margin-box
//! 右缘贴该缘，超列宽左溢）。taffy 交换帧按逻辑 y 左起排 → float:left 停留内容左缘
//!（002 test 臂物理 x=784 右出视口实证）；chromium oracle 两方向右缘同为 .vp content
//! 右缘。修复（vertical_block_flow.rs `anchor_orthog_float_rl_start`，step 12.7）。
//!
//! kill-switch 独立进程钉见 r5457_orthog_float_rl_anchor_kill_switch.rs。

use zero_layout_engine::types::LayoutBox;
use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 引擎 compute（800×600 视口；样式经外部 sheet 传入，r5342/r5456 同款）。
fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 002 形态取 .vp 与其 float 子盒（html > body > .vp > float；跳过 node_id=None
/// 匿名盒——html 匿名包裹是否出现随 pass 组合漂移，不可作固定层级）。
fn vp_and_float_child(root: &LayoutBox) -> (&LayoutBox, &LayoutBox) {
    let body = root
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect("body 盒（node_id 在树）");
    let vp = body
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect(".vp 盒（node_id 在树）");
    let float_box = vp
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect(".vp float 子盒（node_id 在树）");
    (vp, float_box)
}

/// 002 test 臂形态：.vp 内文本 + 横排 float:left 子。
const FLOAT_HTML: &str = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;

const FLOAT_CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; margin: 0; }";

/// 003 同构形态：float:right。
const FR_HTML: &str = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated right</div>
</div>
</body></html>"#;

const FR_CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: right; writing-mode: horizontal-tb; margin: 0; }";

/// 主判例（002）：float:left 子 margin-box 右缘贴容器 content 右缘（块流起始）。
/// content 系断言对 html/.vp 塌宽债（#33 挂账另一切片）稳健。
#[test]
fn orthog_float_left_anchors_rl_block_start() {
    let result = compute(FLOAT_HTML, FLOAT_CSS);
    let (vp, float_box) = vp_and_float_child(&result.root);
    assert!(
        (float_box.x + float_box.width - vp.content_width).abs() <= 0.5,
        "float 右缘贴块流起始（content 系：x+w == content_width；旧 taffy 左起排 \
         x≈0 例外），实得 x={} w={} content_width={}",
        float_box.x,
        float_box.width,
        vp.content_width
    );
    // R5456 钉维持：自身取向测量（单行高 40、max-content 宽 262 域）不回归。
    assert_eq!(float_box.height, 40.0, "单行行高");
    assert!(
        (250.0..=280.0).contains(&float_box.width),
        "max-content 单行宽 262 域，实得 {}",
        float_box.width
    );
}

/// 同判例 float:right（003）：块流起始与方向无关（单列容器），右缘同贴 content 右。
#[test]
fn orthog_float_right_anchors_same_block_start() {
    let result = compute(FR_HTML, FR_CSS);
    let (vp, float_box) = vp_and_float_child(&result.root);
    assert!(
        (float_box.x + float_box.width - vp.content_width).abs() <= 0.5,
        "float:right 右缘同贴块流起始，实得 x={} w={} content_width={}",
        float_box.x,
        float_box.width,
        vp.content_width
    );
}

/// gate 排除钉：float + 流内块子混排容器不触本臂（R4428 taffy 几何自洽域维持）——
/// float:left 维持 taffy 原始左起排位置（content 系 x≈0），不被右缘重锚。
#[test]
fn gate_excludes_float_with_inflow_sibling() {
    let html = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
<div class="b">in-flow block</div>
</div>
</body></html>"#;
    let css = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; margin: 0; } \
.b { margin: 0; }";
    let result = compute(html, css);
    let body = result
        .root
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect("body 盒");
    let vp = body.children.iter().find(|c| c.node_id.is_some()).expect(".vp 盒");
    let float_box = vp.children.iter().find(|c| c.node_id.is_some()).expect(".vp float 盒");
    assert!(
        float_box.x.abs() <= 0.5,
        "混排容器 gate 排除：float 维持 taffy 原始几何（x≈0 不右锚），实得 x={}",
        float_box.x
    );
}
