//! R2171：flex/grid 容器自身 cross 尺寸从 aspect-ratio + Auto-main 推导（taffy 0.12.1 gap）。
//! driving case：`flex-aspect-ratio-cross-size-002.html`（quirks 模式，2.08%→0.00%；与 R2170
//! quirks gate 叠加后全过）。
//! 结构：definite-width 父（200）> flex 容器{aspect-ratio:4; width:auto; height:auto} > 子。
//! taffy 0.12.1 仅在 main **显式** Px 时应用 ar；main 为 Auto（解析到 definite）时不事后应用
//! → 容器 cross 塌缩到 0（standards + quirks 同，实测插桩确认）。chromium 两案都应用 ar（h=50）。
//! R2171 pass：cross 当前为 0（taffy 失败模式）+ ar + main/cross 均 Auto + main 已解析 definite
//! → 推导 cross = main/ratio。kill-switch ZW_AR_CONTAINER_CROSS（default-on）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// 返回深度最大的 div 的 used height（driving case 中 outer flex/grid 容器是深度最大的 div）。
fn deepest_div_height(doc: &zero_dom::Document, root: &LayoutBox) -> Option<f32> {
    let mut best: Option<(i32, f32)> = None;
    let mut stack: Vec<(&LayoutBox, i32)> = vec![(root, 0)];
    while let Some((b, depth)) = stack.pop() {
        let is_div = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "div"))
            .unwrap_or(false);
        if is_div && best.is_none_or(|(bd, _)| depth > bd) {
            best = Some((depth, b.height));
        }
        for c in &b.children {
            stack.push((c, depth + 1));
        }
    }
    best.map(|(_, h)| h)
}

/// R2171 主驱：flex 容器（ar + width:auto + height:auto，definite-width 父）应从 ar 推导
/// cross（height = width/ratio）。修复前 taffy 给 height=0（gap）。
#[test]
fn r2171_flex_container_ar_auto_width_derives_cross() {
    // definite-width 父(200) > flex{aspect-ratio:4}。ar=4 → height=200/4=50。
    let html = r#"<div style="width:200px;">
  <div style="display:flex; aspect-ratio:4;"><div></div></div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    // 修复前：taffy 给 height=0（auto-width 不应用 ar）。修复后：height=200/4=50。
    assert!(
        (h - 50.0).abs() < 5.0,
        "flex container with aspect-ratio + auto-width must derive cross from ar (200/4=~50); got h={h}"
    );
}

/// R2171（grid 变体）：grid 容器同理从 ar 推导 cross。
#[test]
fn r2171_grid_container_ar_auto_width_derives_cross() {
    let html = r#"<div style="width:200px;">
  <div style="display:grid; aspect-ratio:4;"><div></div></div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    assert!(
        (h - 50.0).abs() < 5.0,
        "grid container with aspect-ratio + auto-width must derive cross from ar (~50); got h={h}"
    );
}

/// R2171 非回归守：ar + **显式** width（taffy 已正确应用 ar）不受本 pass 影响（cross 非零，
/// pass 的 cross<0.5 守卫不触发）。-001 谱系。
#[test]
fn r2171_explicit_width_ar_unchanged_by_pass() {
    // 显式 width:200px + ar:4 → taffy 已给 height=50；pass 不应改变。
    let html = r#"<div style="display:flex; width:200px; aspect-ratio:4;"><div></div></div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    assert!(
        (h - 50.0).abs() < 5.0,
        "explicit-width flex+ar must still derive cross (~50); got h={h}"
    );
}
