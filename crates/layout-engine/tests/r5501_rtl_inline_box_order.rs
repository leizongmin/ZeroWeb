//! R5501（CSS2 §9.4.2 RTL 行内盒序）常驻锚：`direction:rtl` 行内级盒按基方向
//! 右→左排布（行内 item 序反转，原子盒与文本 run 同为反转单元），行装配起点
//! （右贴）与 text-align 域独立。V1-V4 为 chromium oracle 实测值对表
//!（800×600、body margin 8 → 内容区 x 8..792；/tmp/r5500-rtl-oracle 留档，
//! evidence/r5500-bench-drift-and-rtl-box-order-attribution-2026-11-11.md §二）。
//! v6 用 Ahem 定宽消除字体差（宽度不对称防「中位盒两序同位」假绿）。
//! kill-switch 反向锚 = r5501_rtl_inline_box_order_kill_switch.rs（独立进程）。

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
/// padding + border + child.x/y，R5500 探针同式）。
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

const BOXES: &str = r#"<div><span id="ga" style="display:inline-block;width:100px;height:50px;background:#0f0"></span><span id="gb" style="display:inline-block;width:200px;height:50px;background:#f00"></span></div>"#;

/// V1：rtl 双原子盒——chromium oracle 实测 ga（DOM 首盒）贴右 692..792、gb 左邻
/// 492..692（行内容宽 300 = 792−300 起）。修复前 ZW ga 492/gb 592..792（DOM 序）。
#[test]
fn v1_rtl_reverses_atomic_box_order() {
    let (root, doc) = compute("div { direction: rtl; }", BOXES);
    let (gx, _, gw, _) = find_by_id(&root, &doc, "ga");
    let (bx, _, bw, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gw, 100.0);
    assert_eq!(bw, 200.0);
    assert_eq!(gx, 692.0, "DOM 首盒须贴右缘（CSS2 §9.4.2 行内级盒右→左排布）");
    assert_eq!(bx, 492.0);
}

/// V2：rtl 单盒贴右 692..792（chromium oracle 同值）——行装配起点=右，既有行为
/// 回归守卫（单片段行反转恒等）。
#[test]
fn v2_rtl_single_box_start_edge_right() {
    let single =
        r#"<div><span id="ga" style="display:inline-block;width:100px;height:50px;background:#0f0"></span></div>"#;
    let (root, doc) = compute("div { direction: rtl; }", single);
    let (gx, _, gw, _) = find_by_id(&root, &doc, "ga");
    assert_eq!(gw, 100.0);
    assert_eq!(gx, 692.0);
}

/// V3：LTR 对照——ga 8..108、gb 108..308（chromium oracle 同值），基 LTR 零扰动。
#[test]
fn v3_ltr_control_order_unchanged() {
    let (root, doc) = compute("div { }", BOXES);
    let (gx, _, _, _) = find_by_id(&root, &doc, "ga");
    let (bx, _, _, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gx, 8.0);
    assert_eq!(bx, 108.0);
}

/// V4：rtl + text-align:left——chromium oracle 实测 gb 8..208、ga 208..308
///（组贴左缘、盒内序仍反转）——「序」与「对齐」两域分离。
#[test]
fn v4_rtl_left_align_reverses_within_left_group() {
    let (root, doc) = compute("div { direction: rtl; text-align: left; }", BOXES);
    let (gx, _, _, _) = find_by_id(&root, &doc, "ga");
    let (bx, _, _, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gx, 208.0);
    assert_eq!(bx, 8.0);
}

/// v6（Ahem 定宽不对称变体）：rtl 混排「AB <box> CDEF」——片段宽 32+16+100+16+64
/// = 228，右贴行跨 564..792；整行反转后视觉序 CDEF(564..628) ␣ box(644..744) ␣
/// AB(760..792)，box x = 644（DOM 序旧世界为 612，宽度不对称防两序同位假绿）。
/// chromium oracle v6（monospace 实测）钉的是**序**面（DOM 首文本最右、box 居中、
/// 尾文本最左）；本锚以 Ahem 算术钉 ZW 几何。
#[test]
fn v6_mixed_line_reverses_whole_item_sequence() {
    let html = r#"<p style="direction: rtl; font-family: Ahem; font-size: 16px;">AB <span id="vb" style="display:inline-block;width:100px;height:18px;background:#0f0"></span> CDEF</p>"#;
    let (root, doc) = compute("", html);
    let (bx, _, bw, _) = find_by_id(&root, &doc, "vb");
    assert_eq!(bw, 100.0);
    assert_eq!(
        bx, 644.0,
        "混排行整行反转：box 右移至反转位（DOM 首文本 AB 贴右缘 760..792）"
    );
}

/// v5：rtl 纯单文本行「ABC def」——单 run 行反转恒等（行级 L2 单片段 run 不反序）。
/// 以同页 rtl/ltr 双渲染盒几何恒等做 A/B（度量无关）：反转恒等 ⇒ 行宽/行高一致。
/// 纯文本 run 内字形序/文本面由 corpus 803 rtl 案＋bidi 族 reftest 常驻覆盖
///（box tree 无文本片段几何可断言，不在此重复）。
#[test]
fn v5_rtl_single_text_run_identity() {
    let make = |dir: &str| {
        let html = format!(r#"<p id="p" style="{dir}; font-family: Ahem; font-size: 16px;">ABC def</p>"#);
        let (root, doc) = compute("", &html);
        find_by_id(&root, &doc, "p")
    };
    let (_, _, l_w, l_h) = make("direction: ltr");
    let (_, _, r_w, r_h) = make("direction: rtl");
    assert_eq!(l_w, r_w, "单 run 行反转恒等：行宽须一致");
    assert_eq!(l_h, r_h, "单 run 行反转恒等：行高须一致");
}
