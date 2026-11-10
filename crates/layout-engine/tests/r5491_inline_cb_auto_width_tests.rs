//! R5491 常驻锚（缺陷 Y）：inline 中间层子百分比宽的块容器 `width:auto` fill 上溯
//! 解算（CSS2 §10.3.3 × §10.1/§10.2）。R5484 臂旧径只认显式 Px 宽，auto 容器链断
//! → taffy percent 相对物化层（blk content − inline margin）解算——floats-149 形
//! `.block`（无宽）内 span 50% 得 380（分母 760 = 800 − inline ml 40），chromium
//! 相对块容器 800 解 400 → nested 级联 fit 判定 `380<=380` 边界翻转并排重叠
//!（s2 (380,0)）。修复后 w=400 → band0 降带 (0,0)/(0,18)、f3 band1 边界 fit
//! 并排 (400,18)、blk h=36（R5493 fresh chromium oracle 定谳；R5490 旧 oracle
//! multi-ml40 记录勘误：该页实为单 span Ahem 文本折 3 行的 float，非 3 float
//! 逐行，green=18 非 54——float y 序列与 blk 值以 R5493 fresh oracle 为准，
//! 宽度分母断言不受影响）。
//!
//! 空探针 span（无文本 + 显式 height）聚焦宽度分母与级联位，避开 strut 语义
//!（缺陷 X 域，另行归因修复）。

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

/// 绝对几何（沿父链累加 border/padding 原点，口径对齐 chromium gBCR）。
fn abs_rect(root: &LayoutBox, id: zero_dom::NodeId) -> (f32, f32, f32, f32) {
    fn walk(b: &LayoutBox, id: zero_dom::NodeId, x: f32, y: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = x + b.x + b.border_left + b.padding_left;
        let ay = y + b.y + b.border_top + b.padding_top;
        if b.node_id == Some(id) {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(r) = walk(c, id, ax, ay) {
                return Some(r);
            }
        }
        None
    }
    walk(root, id, 0.0, 0.0).expect("目标盒存在")
}

fn layout(html: &str) -> zero_layout_engine::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn assert_geometry(
    doc: &zero_dom::Document,
    root: &LayoutBox,
    blk: &str,
    spans: [&str; 3],
    want: [(f32, f32, f32, f32); 3],
    blk_w: f32,
    blk_h: f32,
) {
    for (i, name) in spans.iter().enumerate() {
        let id = doc.get_element_by_id(name).expect("span 存在");
        let (x, y, w, h) = abs_rect(root, id);
        assert!(
            (w - want[i].2).abs() <= 0.5,
            "{name} 宽应 {}（50% 相对块容器分母），实际 {w}——旧径物化层分母漂移",
            want[i].2
        );
        assert!((h - want[i].3).abs() <= 0.5, "{name} 高应 {}，实际 {h}", want[i].3);
        assert!(
            (x - want[i].0).abs() <= 0.5 && (y - want[i].1).abs() <= 0.5,
            "{name} 位应 ({}, {})，实际 ({x},{y})——级联 fit 判定边界翻转",
            want[i].0,
            want[i].1
        );
    }
    let blk_id = doc.get_element_by_id(blk).expect("块容器存在");
    let (_, _, bw, bh) = abs_rect(root, blk_id);
    assert!((bw - blk_w).abs() <= 0.5, "块容器宽应 {blk_w}，实际 {bw}");
    assert!((bh - blk_h).abs() <= 0.5, "块容器高应 {blk_h}，实际 {bh}");
}

/// 主锚：blk/body/html 全 auto 宽（fill 上溯至文档根＝viewport 800）→ span 50%
/// ＝400（chromium 对齐；旧径物化层分母 760 解 380）→ f1/f2 band0/降带、f3
/// band1 cursor 400 边界 fit 并排、blk h=36（float 带底，R5493）。
#[test]
fn r5491_auto_width_block_pct_resolves_against_block_container() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div>
</body></html>"#;
    let result = layout(html);
    let doc = zero_dom::parse_html(html);
    assert_geometry(
        &doc,
        &result.root,
        "blk",
        ["s1", "s2", "s3"],
        [
            (0.0, 0.0, 400.0, 18.0),
            (0.0, 18.0, 400.0, 18.0),
            (400.0, 18.0, 400.0, 18.0),
        ],
        800.0,
        36.0,
    );
}

/// 对照形：同构 `.block { width: 800px }` 显式宽（R5484 显式臂）——两臂等值守护。
#[test]
fn r5491_explicit_width_block_matches_auto_resolution() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 800px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div>
</body></html>"#;
    let result = layout(html);
    let doc = zero_dom::parse_html(html);
    assert_geometry(
        &doc,
        &result.root,
        "blk",
        ["s1", "s2", "s3"],
        [
            (0.0, 0.0, 400.0, 18.0),
            (0.0, 18.0, 400.0, 18.0),
            (400.0, 18.0, 400.0, 18.0),
        ],
        800.0,
        36.0,
    );
}

/// fill 上溯中段：auto 宽祖先自身 margin 参与差式——wrap ml 100 → blk content 700
/// → span 50%＝350（旧径物化层 660 解 330）；s2 band0 cursor 490（350＋链偏移
/// 140）不 fit → 降带；s3 band1 cursor 350 边界 fit 并排 (450,18)。
#[test]
fn r5491_auto_width_chain_subtracts_ancestor_margin() {
    let html = r#"<html><head><style>
body { margin: 0; }
.wrap { display: block; margin-left: 100px; }
.block { display: flow-root; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 18px; }
</style></head><body>
<div class="wrap"><div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div></div>
</body></html>"#;
    let result = layout(html);
    let doc = zero_dom::parse_html(html);
    assert_geometry(
        &doc,
        &result.root,
        "blk",
        ["s1", "s2", "s3"],
        [
            (100.0, 0.0, 350.0, 18.0),
            (100.0, 18.0, 350.0, 18.0),
            (450.0, 18.0, 350.0, 18.0),
        ],
        700.0,
        36.0,
    );
}

/// 自身 margin 参与差式：blk ml 60 → content 740 → span 50%＝370（旧径物化层
/// 700 解 350）；s2 band0 cursor 410 不 fit → 降带；s3 band1 cursor 370 边界
/// fit 并排 (430,18)。
#[test]
fn r5491_auto_width_block_subtracts_own_margin() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; margin-left: 60px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div>
</body></html>"#;
    let result = layout(html);
    let doc = zero_dom::parse_html(html);
    assert_geometry(
        &doc,
        &result.root,
        "blk",
        ["s1", "s2", "s3"],
        [
            (60.0, 0.0, 370.0, 18.0),
            (60.0, 18.0, 370.0, 18.0),
            (430.0, 18.0, 370.0, 18.0),
        ],
        740.0,
        36.0,
    );
}
