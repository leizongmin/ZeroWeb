//! R5215（CSS2 §9.2.1.1）原子 inline-level 子是块容器流参与者——锚定测试。
//!
//! 覆盖三个协作面：
//! ① `adjust_float_positions` 第二阶段把原子 inline-level 子（inline-block 族 /
//!    replaced）按匿名块盒占位推进 flow_bottom（float_positioning.rs）；
//! ② float 容器 IFC 重测（remeasure_text_with_float_exclusions）对块级子 + 原子
//!    inline 子混排容器跳过单一连续行流同步（inline_finalization.rs）；
//! ③ `adjust_inline_block_positions` 的 R5011 锚定按 IFC 伪行分组，跨行各取
//!    taffy 流位（engine/postprocess.rs）。
//!
//! 基线 = chromium 151 CDP getBoundingClientRect（r5215 会话探针实测）。
//! kill-switch `ZW_R5215_ATOMIC_FLOW=0` 回退旧行为（本测试不覆盖 env 切换）。

use super::*;

/// 加载 WPT floats 域页面并布局，返回 (根盒, styles)。
fn layout_wpt_floats_page(name: &str) -> (std::sync::Arc<LayoutBox>, HashMap<NodeId, ComputedStyle>) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/wpt-runner/wpt-data/css/CSS2/floats/"
    )
    .to_string()
        + name;
    let src = std::fs::read_to_string(&path).expect("WPT floats 页面应存在");
    let doc = zero_dom::parse_html(&src);
    // 手抠 <style> 文本（zero-engine collect_stylesheets 不在依赖内）。
    let css = src
        .split("<style type=\"text/css\">")
        .nth(1)
        .and_then(|s| s.split("</style>").next())
        .unwrap_or("");
    let sheets = vec![zero_css_parser::Parser::parse_stylesheet(css)];
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (result.root, styles)
}

/// root(html) 的首个元素子 = body。
fn body_of(root: &LayoutBox) -> &LayoutBox {
    root.children
        .iter()
        .find(|c| c.node_id.is_some())
        .expect("html 下应有 body")
}

fn assert_box(b: &LayoutBox, what: &str, x: f32, y: f32, w: f32, h: f32) {
    assert!(
        (b.x - x).abs() < 0.5 && (b.y - y).abs() < 0.5 && (b.width - w).abs() < 0.5 && (b.height - h).abs() < 0.5,
        "{what}: 期望 (x={x}, y={y}, w={w}, h={h})，实际 (x={}, y={}, w={}, h={})",
        b.x,
        b.y,
        b.width,
        b.height
    );
}

/// floats-wrap-top-below-bfc-002l：float 容器 + 块级子（span display:block）。
/// chromium：float L (0,0) 150×75；float R earliest-fit 下推 (100,75) 300×75；
/// span1 (150,0)；span2 (0,150)；body 406×206。
#[test]
fn r5215_bfc_002l_block_children_flow_positions() {
    let (root, _styles) = layout_wpt_floats_page("floats-wrap-top-below-bfc-002l.xht");
    let body = body_of(&root);
    assert_box(body, "body", 8.0, 8.0, 406.0, 206.0);
    let kids = &body.children;
    assert_eq!(kids.len(), 4, "body 应有 4 个元素子");
    assert_box(&kids[0], "float L", 0.0, 0.0, 150.0, 75.0);
    assert_box(&kids[1], "float R", 100.0, 75.0, 300.0, 75.0);
    assert_box(&kids[2], "span1", 150.0, 0.0, 200.0, 50.0);
    assert_box(&kids[3], "span2", 0.0, 150.0, 200.0, 50.0);
}

/// floats-wrap-top-below-002l-ref：float 容器 + 原子 inline-block 子与块级 div 混排。
/// chromium：float (0,0) 150×25；span1 (150,0)；div (0,50) 400×100；span2 (0,150)；
/// body 406×206。R5215 前 span2 被同步到 IFC 伪行 y=50、body 被重叠回推 +100 至 306。
#[test]
fn r5215_002l_ref_mixed_atomic_flow_positions() {
    let (root, _styles) = layout_wpt_floats_page("floats-wrap-top-below-002l-ref.xht");
    let body = body_of(&root);
    assert_box(body, "body", 8.0, 8.0, 406.0, 206.0);
    let kids = &body.children;
    assert_eq!(kids.len(), 4, "body 应有 4 个元素子");
    assert_box(&kids[0], "float", 0.0, 0.0, 150.0, 25.0);
    assert_box(&kids[1], "span1", 150.0, 0.0, 200.0, 50.0);
    assert_box(&kids[2], "div", 0.0, 50.0, 400.0, 100.0);
    assert_box(&kids[3], "span2", 0.0, 150.0, 200.0, 50.0);
}
