//! R5503（css-writing-modes-3 §2 direction × vertical writing）常驻锚：竖排 rtl
//! 行内轴推进——基方向 rtl 时行内起点 = 行内终点缘（列底），整行 item 序反转
//!（原子盒与文本 run 同为反转单元；run 内字形序不变）。V1-V7 为 chromium oracle
//! 实测结构对表（800×600、body margin 8 → 内容区 y 8..608；/tmp/r5502-vrtl-oracle
//! 留档，evidence/r5502-vertical-rtl-inline-order-attribution-2026-11-11.md §一）。
//! 文本锚以 Ahem 定宽（advance = font_size）消除字体差、钉 ZW 算术；原子盒锚走
//! LayoutBox 定位同步链（paint 真值）。kill-switch 反向锚 =
//! r5503_vertical_rtl_inline_order_kill_switch.rs（独立进程）。

use std::sync::Arc;

use zero_layout_engine::{LayoutEngine, types::LayoutBox};
use zero_style_system::StyleSystem;

fn compute(css: &str, body_inner: &str) -> (Arc<LayoutBox>, zero_dom::Document) {
    let html =
        format!(r#"<html><head><style>body {{ margin: 8px; }} {css}</style></head><body>{body_inner}</body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(f64::from(800.0_f32), f64::from(600.0_f32));
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = engine.compute(&doc, &styles).root;
    (root, doc)
}

/// 按 id 属性定位盒，返回绝对坐标 (x, y, w, h)（paint 约定 abs = parent_abs +
/// padding + border + child.x/y，R5501 同式）。
fn find_by_id(root: &LayoutBox, doc: &zero_dom::Document, id: &str) -> (f32, f32, f32, f32) {
    fn walk(b: &LayoutBox, doc: &zero_dom::Document, id: &str, px: f32, py: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = px + b.padding_left + b.border_left + b.x;
        let ay = py + b.padding_top + b.border_top + b.y;
        let hit = b
            .node_id
            .and_then(|nid| doc.get(nid))
            .and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.get_attribute("id").as_deref() == Some(id)),
                _ => None,
            })
            .unwrap_or(false);
        if hit {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(f) = walk(c, doc, id, ax, ay) {
                return Some(f);
            }
        }
        None
    }
    walk(root, doc, id, 0.0, 0.0).unwrap_or_else(|| panic!("#{id} 盒不在树"))
}

/// 按 id 定位盒并返回存储 inline_layout（竖排：line.y=块轴列 x、fragment.y=行内
/// 深度位；存储门 is_pure_ahem，文本锚须 Ahem）。
fn inline_lines_of(
    root: &LayoutBox,
    doc: &zero_dom::Document,
    id: &str,
) -> Vec<zero_layout_engine::types::InlineLayoutLine> {
    fn walk<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
        let hit = b
            .node_id
            .and_then(|nid| doc.get(nid))
            .and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.get_attribute("id").as_deref() == Some(id)),
                _ => None,
            })
            .unwrap_or(false);
        if hit {
            return Some(b);
        }
        for c in &b.children {
            if let Some(f) = walk(c, doc, id) {
                return Some(f);
            }
        }
        None
    }
    walk(root, doc, id)
        .and_then(|b| b.inline_layout.clone())
        .unwrap_or_else(|| panic!("#{id} 无存储 inline_layout（存储门未放行？）"))
}

const VBOXES: &str = r#"<div id="d"><span id="ga" style="display:inline-block;width:50px;height:100px;background:#0f0"></span><span id="gb" style="display:inline-block;width:50px;height:200px;background:#f00"></span></div>"#;

/// V1（chromium P1）：vrl+rtl definite 600 双原子盒——ga（DOM 首）锚行内终点缘
///（底）508..608、gb 308..508，整行 DOM 反序。镜像算术：ga y' = 600−0−100 = 500
///（abs 508）、gb y' = 600−100−200 = 300（abs 308）。修复前 ZW ga 8/gb 108（DOM 序）。
#[test]
fn v1_vrl_rtl_atomic_boxes_bottom_anchored_reversed() {
    let (root, doc) = compute(
        "div { writing-mode: vertical-rl; direction: rtl; height: 600px; width: 108px; }",
        VBOXES,
    );
    let (_, gy, _, gh) = find_by_id(&root, &doc, "ga");
    let (_, by, _, bh) = find_by_id(&root, &doc, "gb");
    assert_eq!(gh, 100.0);
    assert_eq!(bh, 200.0);
    assert_eq!(
        gy, 508.0,
        "DOM 首盒须锚行内终点缘（竖排 rtl 行内起点=底，css-writing-modes-3 §2）"
    );
    assert_eq!(by, 308.0);
}

/// V2（chromium P2）：vrl+ltr 对照——ga 8..108、gb 108..308（DOM 序），基 LTR
/// 零扰动（direction 缺省不触发镜像）。
#[test]
fn v2_vrl_ltr_control_order_unchanged() {
    let (root, doc) = compute(
        "div { writing-mode: vertical-rl; height: 600px; width: 108px; }",
        VBOXES,
    );
    let (_, gy, _, _) = find_by_id(&root, &doc, "ga");
    let (_, by, _, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gy, 8.0);
    assert_eq!(by, 108.0);
}

/// V3（chromium P3）：vlr+rtl——行内轴语义与 vrl 同（ga 508/gb 308）；块轴列推进
/// 不受 direction 影响（vlr 首列贴左，两盒 x=8）。
#[test]
fn v3_vlr_rtl_inline_mirror_block_axis_unchanged() {
    let (root, doc) = compute(
        "div { writing-mode: vertical-lr; direction: rtl; height: 600px; width: 108px; }",
        VBOXES,
    );
    let (gx, gy, _, _) = find_by_id(&root, &doc, "ga");
    let (bx, by, _, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gy, 508.0, "vlr 行内轴镜像与 vrl 同构");
    assert_eq!(by, 308.0);
    assert_eq!(gx, 8.0, "块轴列推进不受 direction 影响（vlr 首列贴左）");
    assert_eq!(bx, 8.0);
}

/// V4（chromium P4）：vrl+rtl 纯文本 HAMBURG（Ahem 24，行内深度 7×24=168）——
/// 单 run 行镜像恒等反转，带锚行内终点缘：fragment y' = 600−0−168 = 432，
/// 带 [432, 600] 底贴 600。修复前 ZW 带 [0, 168]（自顶起）。
#[test]
fn v4_vrl_rtl_text_band_flush_inline_end() {
    let html = r#"<div id="d" style="writing-mode: vertical-rl; direction: rtl; height: 600px; font-family: Ahem; font-size: 24px;">HAMBURG</div>"#;
    let (root, doc) = compute("", html);
    let lines = inline_lines_of(&root, &doc, "d");
    assert_eq!(lines.len(), 1);
    let frags = &lines[0].fragments;
    assert_eq!(frags.len(), 1, "单 run 单词 = 单 segment");
    assert_eq!(frags[0].y, 432.0, "竖排 rtl 文本带须底贴行内终点缘 600");
    assert_eq!(frags[0].y + frags[0].height, 600.0);
}

/// V5（chromium P5）：vrl+ltr 文本对照——带 [0, 168] 自顶起（零扰动）。
#[test]
fn v5_vrl_ltr_text_control_top_anchored() {
    let html = r#"<div id="d" style="writing-mode: vertical-rl; height: 600px; font-family: Ahem; font-size: 24px;">HAMBURG</div>"#;
    let (root, doc) = compute("", html);
    let lines = inline_lines_of(&root, &doc, "d");
    let frags = &lines[0].fragments;
    assert_eq!(frags[0].y, 0.0);
    assert_eq!(frags[0].y + frags[0].height, 168.0);
}

/// V6（chromium P6）：vrl+rtl 混排 AB[box]CD（Ahem 24，box 100×100）——整行 item
/// 序反转：AB 段 [552, 600]（底）、box [452, 552]、CD 段 [404, 452]（顶），DOM 序
/// 修复前 AB 顶/CD 底。镜像算术 y' = 600 − y − h。
#[test]
fn v6_vrl_rtl_mixed_reverses_whole_item_sequence() {
    let html = r#"<div id="d" style="writing-mode: vertical-rl; direction: rtl; height: 600px; width: 108px; font-family: Ahem; font-size: 24px;">AB<span id="gm" style="display:inline-block;width:100px;height:100px;background:#0f0"></span>CD</div>"#;
    let (root, doc) = compute("", html);
    let (_, my, _, _) = find_by_id(&root, &doc, "gm");
    assert_eq!(my, 460.0, "原子盒 y' = 600−48−100 = 452（abs 460），居行中反序位");
    let lines = inline_lines_of(&root, &doc, "d");
    let frags = &lines[0].fragments;
    let ab = frags.iter().find(|f| f.text.contains('A')).expect("AB 段");
    let cd = frags.iter().find(|f| f.text.contains('C')).expect("CD 段");
    assert_eq!(ab.y, 552.0, "DOM 首文本段 AB 锚底 [552, 600]");
    assert_eq!(cd.y, 404.0, "DOM 尾文本段 CD 居顶 [404, 452]");
    assert!(ab.y > my - 8.0 && cd.y < my - 8.0, "视觉序 CD→box→AB（DOM 反序）");
}

/// V7（chromium P8）：vrl+rtl 多列——每列独立底锚（内容贴行内终点缘）；列尾悬挂
/// 空格（R4433）越缘外悬：Ahem 24 词体 72 + 尾空格 24，definite 300 → 列 0/1 各
/// 3 词（深度 288）、列 2 两词。镜像常数 c = max(300 + 24, 288) = 324：列 0 首词
/// AAA␣ [228, 324]（字形贴 300、空格悬 300..324）；列 2 末词 HHH 无尾空格
/// c = 300。每列 max(y+h) ≤ 324 恒无负 y。
#[test]
fn v7_vrl_rtl_multicol_per_column_bottom_anchor() {
    let html = r#"<div id="d" style="writing-mode: vertical-rl; direction: rtl; height: 300px; width: 200px; font-family: Ahem; font-size: 24px;">AAA BBB CCC DDD EEE FFF GGG HHH</div>"#;
    let (root, doc) = compute("", html);
    let lines = inline_lines_of(&root, &doc, "d");
    assert_eq!(lines.len(), 3, "Ahem 24/深度 300 → 3+3+2 词三列");
    let col0_ys: Vec<f32> = lines[0].fragments.iter().map(|f| f.y).collect();
    assert_eq!(col0_ys, vec![228.0, 132.0, 36.0], "列 0 镜像位（AAA␣ 贴底、CCC␣ 居顶）");
    let col0_ends: Vec<f32> = lines[0].fragments.iter().map(|f| f.y + f.height).collect();
    assert_eq!(col0_ends, vec![324.0, 228.0, 132.0], "列尾悬挂空格越 300 缘外悬");
    let col2_ys: Vec<f32> = lines[2].fragments.iter().map(|f| f.y).collect();
    assert_eq!(col2_ys, vec![204.0, 132.0], "列 2 无列尾空格 → c = 300 纯镜像");
    for (i, line) in lines.iter().enumerate() {
        for f in &line.fragments {
            assert!(f.y >= 0.0, "列 {i} 片段负 y（镜像溢出未钳）");
        }
    }
}
