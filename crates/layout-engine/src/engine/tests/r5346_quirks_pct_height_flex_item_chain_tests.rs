//! R5346：quirks 模式 % 高 quirk 的 flex/grid **item 链**语义——
//! ① item 自身 % 高在 CB 明确时入链解析并落账 `my_definite`（R5346 sizing.rs 入口门收窄），
//!   后代（含替换元素）最近 definite 祖先解析不被 item 断链；
//! ② flex/grid 子树内**非 item** 后代：链内有最近 definite 祖先时 % 高照常解析
//!   （R2170 整门收窄；R4129 替换元素臂放行），driving case
//!   `intrinsic-percent-replaced-004.html`（✗1.88%→✓0.00%）；
//! ③ flex/grid **直接子（item）**自身在 CB 不明确时 % 高 quirk 仍不适用
//!   （quirks.spec #the-percentage-height-calculation-quirk 对 item 不适用；
//!   driving case `grid-item-percentage-quirk-001.html`——canvas h:200% 作 grid item
//!   不得对 100px 祖先解 200，须滞留 attr 固有 10×10）。
//! taffy 侧 owner 是 engine/sizing.rs R2016 quirks pass（R4129 臂 + R5346 双门）；
//! IFC 行高侧见 inline/collect_items R5346 臂（kill-switch `ZW_QUIRK_PCT_HEIGHT=0`
//! 特性总闸，OnceLock 进程级单读——kill-switch 腿由 env A/B 取证，不做进程内测试）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone()))
}

/// 按元素名收集盒（先序遍历），返回 (width, height)。
fn collect_boxes_by_tag(doc: &zero_dom::Document, root: &LayoutBox, tag: &str) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        let hit = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == tag))
            .unwrap_or(false);
        if hit {
            out.push((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    out
}

/// 主驱（intrinsic-percent-replaced-004 同构）：quirks 下 flex h:200 > float item
/// h:50% > 块 div > canvas attr 10×10 + CSS h:100%。float item 的 50% 对 CB 200
/// 先定 100 并落账，canvas h:100% 对最近 definite 祖先（float 100）解析 →
/// 100×100（旧实现：item 断链 nearest 停 200 或整门挡入 attr 固有 10×10 窄条）。
#[test]
fn r5346_quirks_pct_height_through_flex_item_chain() {
    // 无 DOCTYPE → quirks mode。
    let html = r#"<div style="display:flex; height:200px;">
  <div style="float: left; height: 50%; background: green;">
    <div><canvas width="10" height="10" style="height: 100%;"></canvas></div>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let canvases = collect_boxes_by_tag(&doc, &root, "canvas");
    assert_eq!(canvases.len(), 1, "一个 canvas");
    let (w, h) = canvases[0];
    assert!(
        (w - 100.0).abs() < 0.5 && (h - 100.0).abs() < 0.5,
        "canvas 应按 float item 链定高 100、比回传宽 100，实际 {w}x{h}"
    );
    // float item 自身：50% × 200 = 100（taffy 原生同值；此处守 item 断链修复）。
    let divs = collect_boxes_by_tag(&doc, &root, "div");
    assert!(
        divs.iter()
            .any(|&(dw, dh)| (dw - 100.0).abs() < 0.5 && (dh - 100.0).abs() < 0.5),
        "float item 应为 100×100（50% × 200 定高 + 比回传内容宽），divs={divs:?}"
    );
}

/// flex/grid 直接子（item）在 CB 不明确时 % 高 quirk 不适用
///（grid-item-percentage-quirk-001 同构）：quirks 下 definite 100 容器 > grid > canvas
/// h:200%——canvas 不得对 100px 祖先解 200 红透，滞留 attr 固有 10×10。
#[test]
fn r5346_grid_item_pct_height_quirk_not_applied() {
    let html = r#"<div style="width: 100px; height: 100px; overflow: hidden; background: green;">
  <div style="display: grid;">
    <canvas width="10" height="10" style="height: 200%; background: red;"></canvas>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let canvases = collect_boxes_by_tag(&doc, &root, "canvas");
    assert_eq!(canvases.len(), 1, "一个 canvas");
    let (w, h) = canvases[0];
    assert!(
        (w - 10.0).abs() < 0.5 && (h - 10.0).abs() < 0.5,
        "grid item canvas h:200% 不得 quirks 解析（滞留 attr 固有 10×10），实际 {w}x{h}"
    );
}

/// R2170 守（近邻 r2170 文件主驱的替换元素变体）：flex 子树内**无 definite 祖先**时
/// 替换元素 h:100% 仍不得按 viewport 解析（ICB 兜底对 item/后代一致禁用），
/// 滞留 attr 固有——R5346 放行仅限链内存在最近 definite 祖先。
#[test]
fn r5346_replaced_without_definite_ancestor_stays_attr_intrinsic() {
    let html = r#"<div style="display:flex;">
  <div>
    <canvas width="10" height="10" style="height: 100%;"></canvas>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let canvases = collect_boxes_by_tag(&doc, &root, "canvas");
    assert_eq!(canvases.len(), 1, "一个 canvas");
    let (w, h) = canvases[0];
    assert!(
        (w - 10.0).abs() < 0.5 && (h - 10.0).abs() < 0.5,
        "无 definite 祖先的 canvas h:100% 不得按 viewport(600) 解析，滞留 attr 10×10，实际 {w}x{h}"
    );
}
