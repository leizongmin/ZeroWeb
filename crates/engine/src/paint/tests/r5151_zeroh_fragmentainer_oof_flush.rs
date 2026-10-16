//! R5151（css-multicol-1 §abspos-position / css-position-3）：零空间 fragmentainer
//!（R4515 skip）仅豁免 in-flow 子树——列子的 positioned 后代不参与碎片化，照常
//! flush 绘制（contain-size-monolithic-002：0 高 #container 内 abs 绿块 chromium
//! 全绘制、无列裁剪）。修复前 R4515 `continue` 连带跳过 R1080 本地 positioned
//! flush → abs 后代整块丢失。
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::StyleSystem;

use super::super::painter::Painter;

const CSS: &str = r#"
div#multicol { column-count: 3; width: 300px; margin: 0; }
div#container { position: relative; margin: 0; }
div#abs { position: absolute; contain: size; width: 100px; height: 100px; background: green; }
"#;

fn paint_page(body_inner: &str) -> (RenderPrimitives, LayoutBox) {
    let html = format!(r#"<html><body style="margin:0">{body_inner}</body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(CSS);
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

/// 找 multicol 容器（column 子）盒。
fn find_multicol(b: &LayoutBox) -> Option<&LayoutBox> {
    if b.is_multicol {
        return Some(b);
    }
    b.children.iter().find_map(find_multicol)
}

/// 零空间 fragmentainer 中 R4515 跳过的列子，其 abs positioned 后代照常绘制——
/// 100×100 绿块（#008000）完整入图元（含尺寸守卫：无列窗口裁剪）。
#[test]
fn r5151_abs_descendant_painted_in_zero_space_fragmentainer() {
    let (prims, root) = paint_page(r#"<div id="multicol"><div id="container"><div id="abs"></div></div></div>"#);
    // 前置守卫：确入 R4515 域——容器唯一列子无 in-flow 内容（abs 不占流）塌 0，
    // multicol 平衡列高 0。
    let mc = find_multicol(&root).expect("multicol 容器");
    assert!(
        mc.content_height < 0.5,
        "前置：multicol 容器平衡列高应为 0（列子无 in-flow 内容），实际 ch={}",
        mc.content_height
    );
    // chromium oracle（css_css-contain_contain-size-monolithic-002_html.png）：
    // 100×100 绿块全绘制、无列裁剪。green = #008000。
    let green_square = prims.fills.iter().any(|f| {
        f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.rect.size.width >= 99.0 && f.rect.size.height >= 99.0
    });
    assert!(
        green_square,
        "R4515 跳过的列子的 abs 后代（100×100 绿块）应照常 flush 绘制（R5151），实际 fills 中无完整绿块"
    );
}
