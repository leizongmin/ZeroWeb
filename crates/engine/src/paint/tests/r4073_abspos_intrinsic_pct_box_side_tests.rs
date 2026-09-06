//! R4073 回归：abspos content-based sizing 下子树百分比 margin/padding 相对零解析
//!（css-position-3 §abspos-auto-size；mozilla bug 1936450）。
//!
//! abspos 元素 width 为 auto（shrink-to-fit = fit-content 语义）或 intrinsic 关键字时，
//! 其子的百分比 margin/padding 在度量中不可解析（indefinite containing block）。taffy
//! 对 abspos 容器的 auto 宽用 CB 宽解析子百分比（-50% → -50px 参与 shrink），且
//! resolve_percentage_padding pass 会把子百分比 padding 预解析为 px——两处均需豁免。
//!
//! driving: css-sizing/abspos-auto-sizing-fit-content-percentage-001..004
//!（001/002 margin-left:-50%、003/004 padding-left:50%，abs w=50/150 应 100）。

use crate::pipeline::RenderPipeline;

fn abs_geometry(extra_child_css: &str) -> (f32, f32, f32) {
    // 返回（abs 盒 w, child 盒 w, child x）
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let css = format!(
        "body{{margin:0}} .container{{position:relative;width:100px;height:100px;background:red}} \
.abs{{position:absolute;background:green}} .child{{width:100px;height:100px;{}}}",
        extra_child_css
    );
    let html = "<html><head><style></style></head><body>\
<div class=\"container\"><div class=\"abs\"><div class=\"child\"></div></div></div></body></html>";
    let result = pipeline.render_html(html, &css);
    let container = result
        .layout
        .root
        .children
        .iter()
        .find(|c| c.height > 0.0)
        .and_then(|b| b.children.first())
        .expect("container");
    let abs = container.children.first().expect("abs");
    let child = abs.children.first().expect("child");
    (abs.width, child.width, child.x)
}

/// margin-left:-50% 不解析：abs shrink-to-fit = child 100（应 100 非 50）。
#[test]
fn r4073_pct_margin_not_resolved_in_abs_shrink() {
    let (abs_w, child_w, child_x) = abs_geometry("margin-left:-50%");
    assert!(
        (abs_w - 100.0).abs() < 0.5,
        "R4073: abs fit-content 宽下子百分比 margin 应按零解析，abs w 应 100，got {abs_w}"
    );
    assert!((child_w - 100.0).abs() < 0.5 && child_x.abs() < 0.5);
}

/// padding-left:50% 不解析：abs = 100（应 100 非 150）。
#[test]
fn r4073_pct_padding_not_resolved_in_abs_shrink() {
    let (abs_w, child_w, _child_x) = abs_geometry("padding-left:50%");
    assert!(
        (abs_w - 100.0).abs() < 0.5,
        "R4073: abs fit-content 宽下子百分比 padding 应按零解析，abs w 应 100，got {abs_w}"
    );
    assert!((child_w - 100.0).abs() < 0.5);
}

/// 对照：px margin 正常参与 shrink（-50px → abs = 50）。
#[test]
fn r4073_px_margin_still_resolved() {
    let (abs_w, _child_w, _child_x) = abs_geometry("margin-left:-50px");
    assert!(
        (abs_w - 50.0).abs() < 0.5,
        "R4073 对照: px margin 照常参与收缩，abs w 应 50，got {abs_w}"
    );
}
