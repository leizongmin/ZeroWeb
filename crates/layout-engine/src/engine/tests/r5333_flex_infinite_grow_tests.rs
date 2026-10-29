//! R5333：无穷 flex 因子语义（css-values-4 math functions × css-flexbox-1
//! §resolve-flexible-lengths）。
//!
//! 根因（两段接力）：①style 层 flex-grow 长手解析 `parse::<f64>()` 对
//! `calc(infinity)` 字符串直接失败整条拒收（R3345 旧定性「Infinity 非法」证伪——
//! 值域 `<number [0,∞]>` 上界含 ∞），flex-grow-009 的 infinite item 回退 grow 0；
//! ②taffy 比例分配 `free × (inf/inf)` 在 IEEE 下为 NaN → 全员 target NaN→0，
//! 100×100 容器全红（恰 10000px = 2.08%）。修复 = apply.rs 两臂加 math 求值
//!（grow 放行 +∞、shrink 拒 ∞——taffy shrink 臂无 ∞ 语义）+ taffy-local 分配臂
//! ∞ 因子均分全部 free space（css-values-4 ∞ 相等比较，有限因子份额 0）。
//! kill-switch = `ZW_FLEX_INFINITE_GROW=0`（default-on，taffy 侧）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::{Document, NodeKind};

fn compute(html: &str) -> (Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

/// 找指定 id 的 LayoutBox（深度优先）。
fn find_id_box<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id_box(c, doc, id))
}

/// 驱动腿（flex-grow-009 形）：`flex: calc(infinity) 0 0px` 吸收全部 free space
/// 100px，`flex: 1 0 0px` 有限因子份额 0——100×100 全绿非全红。
#[test]
fn r5333_infinite_grow_takes_all_free_space() {
    let html = r#"<html><body>
<div style="display:flex;width:100px;height:100px;">
  <div id="inf" style="flex: calc(infinity) 0 0px;"></div>
  <div id="normal" style="flex: 1 0 0px;"></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    let inf = find_id_box(&root, &doc, "inf").expect("inf item");
    let normal = find_id_box(&root, &doc, "normal").expect("normal item");
    assert!(
        (inf.width - 100.0).abs() < 1.0,
        "无穷 flex-grow item w={:.1} 应吸收全部 free space 100px（css-values-4 ∞ × §resolve-flexible-lengths；flex-grow-009）",
        inf.width
    );
    assert!(
        normal.width < 0.5,
        "有限因子 item w={:.1} 应得 0（份额 free×(f/∞)=0）",
        normal.width
    );
}

/// 对照腿：普通 math 表达式同样求值（非 infinity 特例）——`flex-grow: calc(1 + 3)`
/// = 4 与 flex:1 的 1 按 4:1 分 100px → 80/20。
#[test]
fn r5333_plain_math_grow_is_evaluated() {
    let html = r#"<html><body>
<div style="display:flex;width:100px;height:100px;">
  <div id="math" style="flex-grow: calc(1 + 3);"></div>
  <div id="one" style="flex-grow: 1;"></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    let math = find_id_box(&root, &doc, "math").expect("math item");
    let one = find_id_box(&root, &doc, "one").expect("one item");
    assert!(
        (math.width - 80.0).abs() < 1.0 && (one.width - 20.0).abs() < 1.0,
        "math 求值 grow 4:1 分配应 80/20，实 {:.1}/{:.1}（css-values-4 math 长手求值）",
        math.width,
        one.width
    );
}

/// 守卫腿：flex-shrink 值域同含 ∞ 但 taffy shrink 分配臂无 ∞ 语义（scaled factor
/// ∞ 比例 NaN）——维持拒收回退初始 1（挂账：taffy shrink ∞ 臂）。
#[test]
fn r5333_infinite_shrink_still_rejected() {
    let html = r#"<html><body>
<div style="display:flex;width:100px;height:100px;">
  <div id="wide" style="width:200px;flex-shrink: calc(infinity);"></div>
  <div id="peer" style="width:200px;"></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    let wide = find_id_box(&root, &doc, "wide").expect("wide item");
    let peer = find_id_box(&root, &doc, "peer").expect("peer item");
    // 容器 100 装 400：peer shrink=1、wide 回退 shrink=1 → scaled factor 相等各缩一半到 50
    assert!(
        (wide.width - 50.0).abs() < 1.0 && (peer.width - 50.0).abs() < 1.0,
        "∞ shrink 被拒回退 1 后等比收缩应 50/50，实 {:.1}/{:.1}（taffy shrink 臂无 ∞ 语义守卫）",
        wide.width,
        peer.width
    );
}
