//! R5481：margin-aware 域左 float tight 下绕门常驻回归（CSS2 §9.5×§10.3.3
//! frame-floor 模型——beside＋width:auto BFC 根收缩带宽 < 框架和时 chromium 下绕
//! float 底全 CB 重解（x=ml、w=CB − margin_h），与 margin:0 域（R5479/R5480）、
//! 右 float 镜像门（R5217 既有）同一模型）。
//!
//! chromium headless oracle 四形定谳（google-chrome 截图 PIL 行级黑框扫描，
//! 会话临时探针页）：p1 left-margin-tight (0,50) 宽 440、p2
//! right-margin-tight (10,50) 宽 440、p3 left-margin-fit (100,0) 宽 340、p4
//! right-margin-fit (10,0) 宽 340。ZW pre-fix 现状＝p1 (440,0) 宽 10（R5217 左
//! 分支「max(left+frame) 夹回 band 永不下绕」猜测臂，当年探针形全 fit 域 tight
//! 未实证）、p2/p3/p4 三形已精确。
//!
//! 门受既有 `ZW_BFC_MARGIN_SOLVE` 域内门控（margin-aware 钉位解总开关；关闭态
//! 回退臂由 R5480 margin:0 tight 门兜底同值，无独立 kill-switch——域内判定修正
//! 而非行为臂新增）。

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
#flowroot { display: flow-root; border: 15px solid black; height: 80px;"#;

#[test]
fn r5481_margin_aware_tight_and_fit() {
    // 缺口：左 float 440×50（带 [440,450] 宽 10 < frame 30）＋ mr=10 → tight 门
    // 下绕 float 底全 CB (0, 50) 宽 CB−margin_h=440（chromium p1 oracle；
    // pre-fix 旁置收缩 (440, 0) 宽 10）。
    probe(
        &format!(
            "{HEAD} margin-right: 10px; }}</style></head><body><div id=\"host\"><div class=\"f\" style=\"float:left;width:440px;height:50px;background:cyan\"></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "left-margin-tight",
        (0.0, 50.0, 440.0),
    );

    // 右 float 镜像（带 [0,10] 宽 10 < frame）＋ ml=10 → R5217 既有右门下绕
    // (10, 50) 宽 440（chromium p2 oracle，本臂零变化哨）。
    probe(
        &format!(
            "{HEAD} margin-left: 10px; }}</style></head><body><div id=\"host\"><div class=\"f\" style=\"float:right;width:440px;height:50px;background:cyan\"></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "right-margin-tight",
        (10.0, 50.0, 440.0),
    );

    // fit 哨：左 float 100×50（带 [100,450] 宽 350 ≥ frame）＋ mr=10 → 旁置钉位
    // (100, 0) 宽 340（chromium p3 oracle，面向侧 margin 被钉位吃掉——R5217 实证
    // 域零回归哨）。
    probe(
        &format!(
            "{HEAD} margin-right: 10px; }}</style></head><body><div id=\"host\"><div class=\"f\" style=\"float:left;width:100px;height:50px;background:cyan\"></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "left-margin-fit",
        (100.0, 0.0, 340.0),
    );

    // fit 哨右镜像：右 float 100×50 ＋ ml=10 → 钉右缘 (10, 0) 宽 340（chromium
    // p4 oracle——R5217 实证域零回归哨）。
    probe(
        &format!(
            "{HEAD} margin-left: 10px; }}</style></head><body><div id=\"host\"><div class=\"f\" style=\"float:right;width:100px;height:50px;background:cyan\"></div><div id=\"flowroot\"></div></div></body></html>"
        ),
        "right-margin-fit",
        (10.0, 0.0, 340.0),
    );
}
