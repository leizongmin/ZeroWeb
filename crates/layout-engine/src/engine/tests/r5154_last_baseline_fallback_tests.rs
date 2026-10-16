//! R5154（css-align-3 §4.2 baseline fall-back）：grid item `align-self: last
//! baseline` 在基线共享不可行时按 fallback alignment 定位——last baseline →
//! safe self-end。修正 sole participant 贴行首与 Chromium 不符
//!（contain-size-grid-indefinite-height-min-height-flex-row，1.04% 红）。
//! kill-switch：`ZW_GRID_LAST_BASELINE_END=0`（default-on；=0 反证见 evidence）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_by_id<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_by_id(id, doc, c))
}

/// 返回 (x, y, w, h)。
fn layout_box_by_id(html: &str, css: &str, id: &str) -> (f32, f32, f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find_by_id(id, &doc, &result.root).unwrap_or_else(|| panic!("#{id} 无布局盒"));
    (b.x, b.y, b.width, b.height)
}

/// corpus 案面（contain-size-grid-indefinite-height-min-height-flex-row）：
/// contain:size + min-height:100px + rows:1fr 的 grid 中，sole
/// `align-self: last baseline` item 应落行尾（grid 局部 y=50），非与 abspos
/// item 叠在行首（旧版两者均 (0,0)，5000px 像素差）。
#[test]
fn r5154_grid_last_baseline_item_at_row_end() {
    let (x, y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><grid id="grid"><div id="abs" class="item abs"></div><div id="bl" class="item bl"></div></grid></body></html>"#,
        r#"div { margin: 0; }
#grid { display: grid; min-height: 100px; grid-template-rows: 1fr; contain: size; }
#abs { position: absolute; }
#bl { align-self: last baseline; }
.item { width: 100px; height: 50px; background-color: green; }"#,
        "bl",
    );
    let (gx, gy, gw, gh) = layout_box_by_id(
        r#"<html><body style="margin:0"><grid id="grid"><div id="abs" class="item abs"></div><div id="bl" class="item bl"></div></grid></body></html>"#,
        r#"div { margin: 0; }
#grid { display: grid; min-height: 100px; grid-template-rows: 1fr; contain: size; }
#abs { position: absolute; }
#bl { align-self: last baseline; }
.item { width: 100px; height: 50px; background-color: green; }"#,
        "grid",
    );
    assert!(
        (gh - 100.0).abs() < 0.5 && (gw - 800.0).abs() < 0.5,
        "grid 应 800×100（body margin:0 下占满宽；min-height + contain:size 定高），实际 {gw}×{gh}"
    );
    let _ = (gx, gy);
    assert!(
        (x - 0.0).abs() < 0.5 && (y - 50.0).abs() < 0.5 && (w - 100.0).abs() < 0.5 && (h - 50.0).abs() < 0.5,
        "last baseline item 应在行尾 (0,50) 100×50（fallback safe self-end），实际 ({x},{y}) {w}×{h}"
    );
}

/// 同案 abspos item 不受影响：static position 在 grid 内容盒行首（grid 局部
/// (0,0)），与 chromium 实测一致。
#[test]
fn r5154_abspos_sibling_unchanged() {
    let (x, y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><grid id="grid"><div id="abs" class="item abs"></div><div id="bl" class="item bl"></div></grid></body></html>"#,
        r#"div { margin: 0; }
#grid { display: grid; min-height: 100px; grid-template-rows: 1fr; contain: size; }
#abs { position: absolute; }
#bl { align-self: last baseline; }
.item { width: 100px; height: 50px; background-color: green; }"#,
        "abs",
    );
    assert!(
        (x - 0.0).abs() < 0.5 && (y - 0.0).abs() < 0.5 && (w - 100.0).abs() < 0.5 && (h - 50.0).abs() < 0.5,
        "abspos item static position 应在行首 (0,0) 100×50，实际 ({x},{y}) {w}×{h}"
    );
}

/// flex 上下文零行为变化：`align-self: last baseline` 维持 Auto 语义（未实现
/// flex last-baseline 共享）——无定高 item 随 align-items 初始 stretch 拉满行高。
#[test]
fn r5154_flex_last_baseline_stays_auto_semantics() {
    let (_x, _y, _w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="row" style="display:flex; height:100px"><div id="bl"></div></div></body></html>"#,
        r#"div { margin: 0; }
#bl { align-self: last baseline; }"#,
        "bl",
    );
    assert!(
        (h - 100.0).abs() < 0.5,
        "flex item last baseline 应维持 stretch（Auto 语义）高 100，实际 {h}"
    );
}
