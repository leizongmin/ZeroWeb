//! R5216（CSS2 §9.5 rule 8「shortened line box」）行盒避 float 下绕——锚定测试。
//!
//! fit 门断行产生的**新空行** y 带仍被 float 缩到放不下时，行位须按 R4234 同径
//! 下移到重叠 float 带底重算（inline/break_lines.rs `shift_new_line_below_floats`
//! 断行后二次应用；kill-switch `ZW_ATOMIC_POSTBREAK_FLOAT_FIT=0` 回退旧行为）。
//!
//! 基线 = 上游 WPT 参考页（floats-wrap-top-below-002l/002r-ref.xht，
//! reftest-upstream 0.00% 对齐；family sweep ON/OFF 见
//! docs/goal/rendering-compat/evidence/r5216-*.md）。

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

/// floats-wrap-top-below-inline-002l：float L (0,0) 150×75 + float R (100,75)
/// 300×75 夹击行 2 带 50..100（avail=0），span2 须下绕到 float 底 y=150 行首。
/// chromium/upstream ref：span1 (150,0)；span2 (0,150)；body 406×206。
/// 修复前 span2 滞留 (150,50) 与 float 带重叠（reftest 4.60%）。
#[test]
fn r5216_inline_002l_line2_drops_below_floats() {
    let (root, _styles) = layout_wpt_floats_page("floats-wrap-top-below-inline-002l.xht");
    let body = body_of(&root);
    assert_box(body, "body", 8.0, 8.0, 406.0, 206.0);
    let kids = &body.children;
    assert_eq!(kids.len(), 4, "body 应有 4 个元素子");
    assert_box(&kids[0], "float L", 0.0, 0.0, 150.0, 75.0);
    assert_box(&kids[1], "float R", 100.0, 75.0, 300.0, 75.0);
    assert_box(&kids[2], "span1", 150.0, 0.0, 200.0, 50.0);
    assert_box(&kids[3], "span2", 0.0, 150.0, 200.0, 50.0);
}

/// floats-wrap-top-below-inline-002r：镜像 float——float R (250,0) 150×75 +
/// float L (0,75) 300×75，行 2 带同样被夹击 avail=0。chromium/upstream ref：
/// span1 (0,0)；span2 下绕 (0,150)；body 406×206。修复前 span2 滞留 (300,50)。
#[test]
fn r5216_inline_002r_line2_drops_below_floats() {
    let (root, _styles) = layout_wpt_floats_page("floats-wrap-top-below-inline-002r.xht");
    let body = body_of(&root);
    assert_box(body, "body", 8.0, 8.0, 406.0, 206.0);
    let kids = &body.children;
    assert_eq!(kids.len(), 4, "body 应有 4 个元素子");
    assert_box(&kids[0], "float R", 250.0, 0.0, 150.0, 75.0);
    assert_box(&kids[1], "float L", 0.0, 75.0, 300.0, 75.0);
    assert_box(&kids[2], "span1", 0.0, 0.0, 200.0, 50.0);
    assert_box(&kids[3], "span2", 0.0, 150.0, 200.0, 50.0);
}
