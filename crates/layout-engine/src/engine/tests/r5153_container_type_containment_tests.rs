//! R5153（css-contain-5 §3.1 × css-sizing-4 §4.1）：① container-type 蕴含
//! containment 的布局消费接入——`container-type: size` 施加 size containment、
//! `inline-size` 施加 inline-size containment（contain-size-014：contained 盒
//! 不被 img height=200 内容撑高）；② containment 下元素自身 aspect-ratio 非
//! content 派生，definite 宽下转移出高（contain-size-014 应 100×100 非塌 0）；
//! ③ canvas dimension attributes 只映射 aspect ratio（canvas-contain-size：
//! 20×20 attrs + width:100px + contain:size 应 100×100）。
//! kill-switch：`ZW_CONTAINER_TYPE_CONTAIN=0`（①）、`ZW_CONTAIN_AR_TRANSFER=0`（②）、
//! `ZW_CONTAIN_ATTR_RATIO=0`（③）。

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

/// ① `container-type: size` 蕴含 size containment——auto 高按无内容尺寸化，
/// 50×200 子不参与（旧版 container-type 仅解析不消费 → 高 200）。
#[test]
fn r5153_container_type_size_applies_size_containment() {
    let (_x, _y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="t"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t { container-type: size; width: 100px; background: red; }
.inner { width: 50px; height: 200px; }"#,
        "t",
    );
    assert!((w - 100.0).abs() < 0.5, "显式 width:100px 应保持，实际 {w}");
    assert!(
        h.abs() < 0.5,
        "container-type:size 应蕴含 size containment（auto 高塌 0，内容 200px 不参与），实际 {h}"
    );
}

/// ①' `container-type: inline-size` 蕴含 inline-size containment——内联轴
/// （水平 = 宽）CIS-or-0，block 轴（高）照常内容求解。
#[test]
fn r5153_container_type_inline_size_applies_inline_size_containment() {
    let (_x, _y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="t"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t { container-type: inline-size; width: fit-content; background: red; }
.inner { width: 300px; height: 50px; }"#,
        "t",
    );
    assert!(
        w.abs() < 0.5,
        "container-type:inline-size 应蕴含 inline-size containment（fit-content 宽塌 0），实际 {w}"
    );
    assert!(
        (h - 50.0).abs() < 0.5,
        "inline-size containment 不抑制 block 轴——高应照常 50，实际 {h}"
    );
}

/// ② containment 下 CSS aspect-ratio 转移高——auto 高非 content 求解，由
/// definite 宽 × 声明比得出（100/1=100），不塌 0、不被内容拉回。
#[test]
fn r5153_contained_css_aspect_ratio_transfers_height() {
    let (_x, _y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="t"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t { contain: size; aspect-ratio: 1; width: 100px; background: red; }
.inner { width: 50px; height: 200px; }"#,
        "t",
    );
    assert!((w - 100.0).abs() < 0.5, "definite width:100px 应保持，实际 {w}");
    assert!(
        (h - 100.0).abs() < 0.5,
        "contain:size + aspect-ratio:1 + width:100px 应转移出高 100（内容不参与），实际 {h}"
    );
}

/// ②' CIS 高声明优先于 aspect-ratio 转移（R2256 contain-intrinsic-size 语义不变）。
#[test]
fn r5153_contained_cis_height_beats_aspect_ratio_transfer() {
    let (_x, _y, _w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="t"><div class="inner">inner</div></div></body></html>"#,
        r#"div { margin: 0; }
#t { contain: size; aspect-ratio: 1; width: 100px; contain-intrinsic-height: 50px; }
.inner { width: 50px; height: 200px; }"#,
        "t",
    );
    assert!(
        (h - 50.0).abs() < 0.5,
        "contain-intrinsic-height:50px 应优先于 aspect-ratio 转移，实际 {h}"
    );
}

/// ①+② 合流（contain-size-014 形态）：container-type:size + aspect-ratio:1 +
/// width:100px + img height=200 → 100×100（内容忽略、比值转移），非 100×200
/// （旧版无 containment）也非 100×0（塌 0 红底不可见虽蒙混 ref，几何非 spec）。
#[test]
fn r5153_container_type_size_aspect_ratio_ignores_child() {
    let (x, y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div id="outer"><div id="t"><img height="200"></div></div></body></html>"#,
        r#"div { margin: 0; }
#outer { position: relative; }
#t { container-type: size; aspect-ratio: 1; background: red; width: 100px; }"#,
        "t",
    );
    assert!(
        (w - 100.0).abs() < 0.5 && (h - 100.0).abs() < 0.5,
        "container-type:size + aspect-ratio:1 + width:100px 应 100×100，实际 {w}×{h}"
    );
    let _ = (x, y);
}

/// ③ canvas dimension attributes 只映射 aspect ratio（canvas-contain-size 形态）：
/// contain:size + width:100px + 20×20 attrs → 100×100（attr 比转移，内容/固有
/// 尺寸不参与；旧版无 attr 臂 → 塌 0）。
#[test]
fn r5153_contained_canvas_attr_ratio_transfers_height() {
    let (_x, _y, w, h) = layout_box_by_id(
        r#"<html><body style="margin:0"><div style="display:flex"><canvas id="t" width="20" height="20" style="contain: size; background: green; width: 100px;"></canvas></div></body></html>"#,
        "",
        "t",
    );
    assert!(
        (w - 100.0).abs() < 0.5 && (h - 100.0).abs() < 0.5,
        "contained canvas 20×20 attrs + width:100px 应 100×100（attr 比转移），实际 {w}×{h}"
    );
}
