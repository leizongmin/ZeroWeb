//! R5492 常驻锚（缺陷 X）：普通块零行盒 auto 高 strut/0 二分（CSS2 §9.4.2 ×
//! §10.6.3 × §10.8.1）。text_height=0 时行盒存在性由行内流内容定——含非零水平
//! mpb 行内部分 → 容器 strut 计入块高；全零 mpb 且无文本 ＝零高行盒 → 块高 0。
//! 两臂 float 均不计入（§10.6.3 普通块 auto 高＝行盒底，float 溢出不撑高）。
//!
//! chromium oracle 定谳（/tmp/r5490-oracle multi 族 + R5490 复核）：floats-149
//! 形 `.inline` ml40 → blk=18（strut）；同构无 ml → blk=0（零高行盒）；float
//! extent（36）均不计。修复前 ZW 无条件回退 float_bottom 近似（p2/p3 红态 36），
//! 仅 strut 值碰巧等值时绿（p1=18）。

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

fn assert_blk_height(html: &str, want: f32) {
    let result = layout(html);
    let doc = zero_dom::parse_html(html);
    let id = doc.get_element_by_id("blk").expect("块容器存在");
    let (_, _, _, h) = abs_rect(&result.root, id);
    assert!(
        (h - want).abs() <= 0.5,
        "blk 高应 {want}（零行盒 strut/0 二分），实际 {h}——float_bottom 近似回潮或信号缺失",
    );
}

/// 主锚（floats-149 原形）：`.inline` ml40＝非零 mpb 行内部分 → strut（18）计入，
/// float extent 不计入（§10.6.3）。修复前 18 经 float_bottom 碰巧等值，本锚改由
/// strut 臂供给。
#[test]
fn r5492_nonzero_mpb_inline_part_counts_strut() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: block; background: green; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; background: white; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span>Ahem</span><span></span></div></div>
</body></html>"#;
    assert_blk_height(html, 18.0);
}

/// 零高行盒锚：行内流全零 mpb 且无文本 → 块高 0（§9.4.2），float 36 不计入
///（§10.6.3）。修复前 float_bottom 近似红态 36（chromium 0）。
#[test]
fn r5492_zero_height_line_box_yields_zero_block_height() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: block; background: green; }
.inline { display: inline; }
span { float: left; width: 50%; background: cyan; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span>AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD EEEEEEEEEE FFFFFFFFFF GGGGGGGGGG HHHHHHHHHH</span></div></div>
</body></html>"#;
    assert_blk_height(html, 0.0);
}

/// strut 锚（multi-ml40）：同 p2 形加 ml40 → strut（18）计入，float extent 36
/// 不计入。修复前 float_bottom 近似红态 36（chromium 18）。
#[test]
fn r5492_strut_replaces_float_bottom_approximation() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: block; background: green; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; background: cyan; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span>AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD EEEEEEEEEE FFFFFFFFFF GGGGGGGGGG HHHHHHHHHH</span></div></div>
</body></html>"#;
    assert_blk_height(html, 18.0);
}
