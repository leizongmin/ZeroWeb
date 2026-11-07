//! R5456 正交 htb 叶测量交换帧钉（#33 orthogonal float h=NaN 债驱动）：
//! float-lft-orthog-htb-in-vrl-002/003 形态——vertical-rl 父下 horizontal-tb float
//! 旧路径按父 vertical 模式测量，taffy float MaxContent 约束 INFINITY 下 vertical-rl
//! 列 x = 约束右缘锚定 → measured_width=inf → 物理高 inf → 后处理 inf−inf=NaN 不绘制
//!（solo ✗6.91% 可复现）。修复（inline_finalization.rs R5456 臂）：正交收缩叶以自身
//! 取向横排 IFC 测量，返回按交换帧转置；sizing.rs R5456 域门把正交 float 排除出
//! R5368 stf 钉宽（cb_w 取父块轴 extent 错轴，钉 123.2 拆单行为 3 行）。
//!
//! kill-switch 独立进程钉见 r5456_orthog_htb_float_measure_kill_switch.rs。

use zero_layout_engine::types::LayoutBox;
use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 引擎 compute（800×600 视口；样式经外部 sheet 传入，r5342 同款）。
fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 全树几何有限性走查（NaN/inf 任一即失败）。
fn assert_all_finite(node: &LayoutBox, path: &str) {
    assert!(
        node.width.is_finite() && node.height.is_finite(),
        "{path} 几何须有限（#33 inf−inf=NaN 债），实得 w={} h={}",
        node.width,
        node.height
    );
    for (i, c) in node.children.iter().enumerate() {
        assert_all_finite(c, &format!("{path}/ child #{i}"));
    }
}

/// 002 形态取 .vp 首子盒（html > body > .vp > [.f|.a]；跳过 node_id=None 匿名盒——
/// html 匿名包裹是否出现随 pass 组合漂移，不可作固定层级）。
fn first_vp_child(root: &LayoutBox) -> &LayoutBox {
    let html = root;
    let body = html
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect("body 盒（node_id 在树）");
    let vp = body
        .children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect(".vp 盒（node_id 在树）");
    vp.children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect(".vp 首子盒（node_id 在树）")
}

/// 002 test 臂形态：.vp 内文本 + 横排 float 子。
const FLOAT_HTML: &str = r#"<html><body>
<div class="vp">VRL parent
<div class="f">HTB floated left</div>
</div>
</body></html>"#;

const FLOAT_CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.f { background-color: blue; color: white; float: left; writing-mode: horizontal-tb; margin: 0; }";

/// 002 ref 臂形态：abs 定位横排子（同文本同框）。
const ABS_HTML: &str = r#"<html><body>
<div class="vp">
<div class="a">HTB floated left</div>
<span>VRL parent</span>
</div>
</body></html>"#;

const ABS_CSS: &str = "html { writing-mode: vertical-rl; } \
body { margin: 8px; } \
.vp { border: 8px solid orange; font-size: 32px; line-height: 1.25; margin: 0; } \
.a { background-color: blue; color: white; right: 16px; position: absolute; writing-mode: horizontal-tb; margin: 0; }";

/// 主判例：正交 float 以自身取向测量——单行行高 40、max-content 单行宽（≥200，
/// 旧钉宽 wrapped 123 域外），全树无 NaN/inf。
#[test]
fn orthog_htb_float_measures_own_orientation_single_line() {
    let result = compute(FLOAT_HTML, FLOAT_CSS);
    assert_all_finite(&result.root, "root");
    let float_box = first_vp_child(&result.root);
    assert!(
        !float_box.width.is_nan() && !float_box.height.is_nan(),
        "float 几何非 NaN"
    );
    assert_eq!(
        float_box.height, 40.0,
        "单行行高（32px×1.25），实得 {}",
        float_box.height
    );
    assert!(
        (250.0..=280.0).contains(&float_box.width),
        "max-content 单行宽（262 域；旧 R5368 错轴钉宽 wrapped≈123 / NaN），实得 {}",
        float_box.width
    );
}

/// ref 臂同文本 abs 盒同样以自身取向测量（有限几何；位置/宽度语义属
/// taffy absolute 交换帧域＝R5455 家族后续 slice，此处只钉 NaN 债消除）。
#[test]
fn orthog_htb_abs_measures_finite() {
    let result = compute(ABS_HTML, ABS_CSS);
    assert_all_finite(&result.root, "root");
    let abs_box = first_vp_child(&result.root);
    assert!(
        abs_box.width.is_finite() && abs_box.height.is_finite(),
        "abs 几何有限，实得 w={} h={}",
        abs_box.width,
        abs_box.height
    );
}
