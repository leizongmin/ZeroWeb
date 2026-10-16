//! R5150（CSS Containment 1 §3.1 × css-multicol）：contain:size 列子元素的 monolithic
//! 内容在**零空间 fragmentainer**（平衡列高 0 的 multicol 容器）中仍须绘制——尺寸按
//! 无内容计（平衡列高 0、无 column-rule），但内容照常排版并溢出可见（chromium
//! contain-size-monolithic-001 实证：0 高容器下方 overflow 绘 4 行文本）。
//!
//! 修复前：paint 侧 R4515 零空间 fragmentainer skip（`col_h<0.5 && 容器 content_height<0.5
//! → 子树不可见`）把 contain:size 列子整子树丢弃 → 测试页 multicol 区全空白（ref 页
//! 同位有 4 行文本 → 两侧分化超容差）。
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::StyleSystem;

use super::super::painter::Painter;

const CSS_CONTAINED: &str = r#"
div#multi-column { column-count: 2; column-fill: balance; column-gap: 4ch; column-rule: red solid 4ch; column-width: 2ch; font-family: monospace; font-size: 20px; width: 8ch; margin: 0; }
div#size-contain { contain: size; margin: 0; }
p { margin: 0; }
"#;

/// 搭建 contain-size-monolithic-001 同构页面并走 layout → paint 全管线。
///
/// multicol 容器（column-count:2 + width:8ch + 20px monospace）唯一子是 contain:size
/// 文本块 → 子尺寸塌 0 → 容器平衡列高 0。断言子树文本 glyph 照常发射。
fn paint_page(body_inner: &str) -> (RenderPrimitives, LayoutBox) {
    let html = format!(r#"<html><body style="margin:0">{body_inner}</body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(CSS_CONTAINED);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let root = (*result.root).clone();
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    (painter.into_primitives(), root)
}

fn count_uppercase_glyphs(prims: &RenderPrimitives) -> usize {
    prims
        .glyphs
        .iter()
        .filter(|g| matches!(g.glyph_id, c if (b'A' as u32..=b'Z' as u32).contains(&c)))
        .count()
}

/// 找 contain:size 盒（has_size_containment）及其父盒。
fn find_contained_with_parent(b: &LayoutBox) -> Option<(f32, f32)> {
    for c in &b.children {
        if c.has_size_containment {
            return Some((b.content_height, c.height));
        }
        if let Some(found) = find_contained_with_parent(c) {
            return Some(found);
        }
    }
    None
}

/// contain:size 列子的文本在零空间 fragmentainer 下不丢——glyph 照常发射（≥8 字母形）。
#[test]
fn r5150_contain_size_child_text_painted_in_zero_space_fragmentainer() {
    let (prims, root) = paint_page(
        r#"<p>header</p>
<div id="multi-column"><div id="size-contain">AB CD EF GH</div></div>"#,
    );
    // 前置守卫：结构确实进入 R4515 域——容器平衡列高 0（尺寸含 containment 生效）。
    let (mc_ch, contained_h) = find_contained_with_parent(&root).expect("contain:size 盒");
    assert!(
        contained_h < 0.5,
        "前置：contain:size 子应按无内容塌 0，实际 h={contained_h}"
    );
    assert!(
        mc_ch < 0.5,
        "前置：multicol 容器平衡列高应为 0（唯一列子塌 0），实际 ch={mc_ch}"
    );
    let text_glyphs = count_uppercase_glyphs(&prims);
    assert!(
        text_glyphs >= 8,
        "contain:size 列子（AB CD EF GH，8 字母）在 0 高 multicol 容器内文本应照常绘制（R4515 豁免），实际大写字母 glyph={text_glyphs}"
    );
}

/// 对照组（守卫语义未扩权）：无 contain:size 的普通列子同样有文本 glyph（既有行为基线，
/// 防止豁免条件误写成恒真导致 R4515 失效）。
#[test]
fn r5150_control_without_containment_still_paints() {
    let (prims, _root) = paint_page(
        r#"<p>header</p>
<div id="multi-column"><div>AB CD EF GH</div></div>"#,
    );
    let text_glyphs = count_uppercase_glyphs(&prims);
    assert!(
        text_glyphs >= 8,
        "无 containment 对照组文本照常（既有基线）：{text_glyphs}"
    );
}
