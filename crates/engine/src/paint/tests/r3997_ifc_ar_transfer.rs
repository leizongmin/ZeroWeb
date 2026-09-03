//! R3997：IFC 原子行内盒的 CSS aspect-ratio 传递（css-sizing-4 §4.1/§4.2 transferred size）。
//!
//! inline `<svg>`（display:inline，R3987 原子化路径）+ CSS width + aspect-ratio（无 height）
//! 时，auto 侧由显式侧 ×/÷ ratio 推导。driving：css-sizing replaced-element-007/008/015/016
//! 簇——修复前 svg 塌 6×24.6（原子分支 w=100 但 h=auto→0，双侧无效降级零宽）。
//! `auto <ratio>` 时 replaced 元素固有比优先：inline svg 的固有比直接从 viewBox 解析
//!（replaced-element-015：`auto 5/1` + viewBox 1:1 → 1:1 生效）。
//!
//! R4010：断言值从「transferred + ~6 extras」迁到「精确 transferred」——R4007
//! extract_inline_visual_metrics 的幻影 border（computed medium 3px×2，style=none 未
//! 抑制）修复后 sync 盒高不再含 6px extras，旧范围（25.5..26.5 / 105.5..106.5）锚定
//! 的是修复前的 bug 行为。
use zero_layout_engine::{LayoutBox, LayoutEngine};

fn svg_box_of(stylesheet: &str, body_inner: &str) -> Option<(f32, f32)> {
    let html =
        format!(r#"<html><head><style>{stylesheet}</style></head><body style="margin:0">{body_inner}</body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next()?;
    let sheet = zero_css_parser::Parser::parse_stylesheet(stylesheet);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find(id: zero_dom::NodeId, b: &LayoutBox) -> Option<&LayoutBox> {
        if b.node_id == Some(id) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, c))
    }
    find(svg_id, &result.root).map(|b| (b.width, b.height))
}

/// 显式 `aspect-ratio: 5/1` + width:100px → transferred height = 100/5 = 20。
#[test]
fn r3997_ifc_atomic_svg_explicit_ratio_transfer() {
    let box_ = svg_box_of(
        "svg { width: 100px; aspect-ratio: 5/1; }",
        r#"<span><svg viewBox="0 0 1 1"></svg></span>"#,
    );
    let (w, h) = box_.expect("svg box");
    assert!(w >= 100.0, "CSS width 应用（盒宽含边距 extras）：{w}");
    assert!(
        (19.5..20.5).contains(&h),
        "transferred height = 100/5 = 20（R4007 幻影 border 修复后盒高=精确 transferred 值，无 extras）：{h}"
    );
}

/// `aspect-ratio: auto 5/1` + viewBox 1:1：`auto` 优先 replaced 固有比（viewBox 1:1）
/// 而非显式 5/1（css-sizing-4 §aspect-ratio；R2440 同语义）。
#[test]
fn r3997_ifc_atomic_svg_auto_ratio_prefers_viewbox_intrinsic() {
    let box_ = svg_box_of(
        "svg { width: 100px; aspect-ratio: auto 5/1; }",
        r#"<span><svg viewBox="0 0 1 1"></svg></span>"#,
    );
    let (w, h) = box_.expect("svg box");
    assert!(w >= 100.0, "CSS width 应用（盒宽含边距 extras）：{w}");
    assert!(
        (99.5..100.5).contains(&h),
        "auto <ratio> 用 viewBox 1:1 固有比 → 高 100（R4007 幻影 border 修复后盒高=精确值，无 extras）：{h}"
    );
}
