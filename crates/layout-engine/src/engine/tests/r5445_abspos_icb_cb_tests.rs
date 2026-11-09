//! R5445：abs-auto 盒 spec CB ≠ 直接父时按 spec CB available 双向重测（CSS2 §10.1）。
//!
//! taffy 无「最近 positioned 祖先 / ICB」包含块语义，abs-auto 叶一律按直接父
//! content width 钳制；apply_abspos_shrink_to_fit_width 旧径只扩不缩＋branch-text
//! 窄门使钳制存活。探针定谳（chrome oracle 双侧对照）：非 positioned 直父
//! （spec CB = ICB）ZW 96 vs chrome 796；positioned 祖先隔非 positioned 中间层
//! （spec CB = 祖先 800）ZW 96 vs chrome 796。driving：
//! css/CSS2/css21-errata/s-11-1-1b-005 残差 0.46%（abs p 按 cell 宽折 5 行 vs
//! chrome ICB 单行）→ ✓0.00%。`ZW_ABSPOS_ICB_CB=0` 回退旧行为。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn compute(html: &str, ids: &[&'static str]) -> (crate::types::LayoutResult, Vec<(&'static str, NodeId)>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let pairs: Vec<(&'static str, NodeId)> = ids
        .iter()
        .map(|n| (*n, doc.get_element_by_id(n).expect("id")))
        .collect();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    (engine.compute(&doc, &styles), pairs)
}

const LONG_TEXT: &str = "Test passes if the text stays on a single wide line instead of wrapping many times.";

/// 无 positioned 祖先（spec CB = ICB 800）：abs-auto 叶按 ICB available 重测，
/// taffy 直接父钳制（100）不再存活。旧缺陷签名：p 宽 ≤ 100 折多行。
#[test]
fn r5445_abs_leaf_sizes_against_icb_without_positioned_ancestor() {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="wrap" style="width:100px; height:40px; background:green;">
  <p id="tgt" style="position:absolute; top:0; background:red">{}</p>
</div>
</body></html>"#,
        LONG_TEXT
    );
    let (result, ids) = compute(&html, &["tgt"]);
    let tgt = find(&result.root, ids[0].1).expect("tgt box");
    assert!(
        (tgt.width - 800.0).abs() < 2.0 || tgt.width > 400.0,
        "abs 叶应按 ICB available 重测（≈800 或内容 max-content），实际 {}",
        tgt.width
    );
}

/// positioned 祖先隔非 positioned 中间层（spec CB = 祖先宽）：abs-auto 叶按祖先
/// 宽重测，非中间窄 wrapper 宽。探针 P5 形（chrome 796）。
#[test]
fn r5445_abs_leaf_sizes_against_positioned_ancestor_over_nonpositioned_wrapper() {
    let html = format!(
        r#"<html><body style="margin:0">
<section id="sec" style="position:relative; width:800px; height:40px;">
  <div id="wrap" style="width:100px; height:40px; background:green;">
    <p id="tgt" style="position:absolute; top:0; background:red">{}</p>
  </div>
</section>
</body></html>"#,
        LONG_TEXT
    );
    let (result, ids) = compute(&html, &["tgt"]);
    let tgt = find(&result.root, ids[0].1).expect("tgt box");
    assert!(
        tgt.width > 400.0,
        "abs 叶应按 positioned 祖先（800）available 重测，实际 {}",
        tgt.width
    );
}

/// 直接父即 spec CB（positioned 直父）：维持 taffy 直父钳制口径不扩臂（回归卫）。
/// 探针 P2 形（chrome 与 ZW 同钳 100 带）。
#[test]
fn r5445_positioned_direct_parent_keeps_parent_clamp() {
    let html = format!(
        r#"<html><body style="margin:0">
<div id="wrap" style="position:relative; width:100px; height:40px; background:green;">
  <p id="tgt" style="position:absolute; top:0; background:red">{}</p>
</div>
</body></html>"#,
        LONG_TEXT
    );
    let (result, ids) = compute(&html, &["tgt"]);
    let tgt = find(&result.root, ids[0].1).expect("tgt box");
    assert!(
        tgt.width <= 101.0,
        "spec CB = positioned 直父时应维持直父钳制（≈100），实际 {}",
        tgt.width
    );
}
