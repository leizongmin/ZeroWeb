//! R5514（CSS2 §9.2.1.1 + §10.1）：display:inline 非替换盒不是块容器，永非后代盒级
//! 百分比高度解析的包含块——R695 % 高 walk 的 CB 链对 inline 中间层（split inline 壳）
//! 透明，壳不 reset definite；CB 链经壳时 walk 代写 px（taffy 原生 % 解析基准是 taffy
//! 直属父 = 壳 auto 高，不可达链上块容器）。驱动案 percent-height-replaced-in-percent-
//! cell-004：table-cell(height:50px) > span 壳 > block canvas height:200% 应 100 高、
//! 1:1 ratio 推宽 100（旧：壳 reset None → compute-to-auto + attr 固有 1×1，绿面整缺）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use std::sync::Arc;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::StyleSystem;

fn compute(html: &str) -> (Document, LayoutBox, HashMap<NodeId, zero_style_system::ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root, styles)
}

/// 找指定 id 的 LayoutBox（深度优先）。
fn find_id<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id(c, doc, id))
}

/// WPT 004 案形：cell(height:50px) > span 壳 > block canvas height:200%——canvas 对
/// 链上块容器（cell 50px）解析 100，1:1 ratio 推宽 100（ref 绿方块 100×100）。
#[test]
fn r5514_cell_span_shell_canvas_pct_resolves_through_inline() {
    let html = r#"<!DOCTYPE html><html><body style="margin:0">
<div style="display: table;">
  <div style="display: table-cell; height: 50px;">
    <span><canvas id=c width=1 height=1 style="display: block; height: 200%; background: green;"></canvas></span>
  </div>
  <div style="display: table-cell;">
    <div style="height: 100px;"></div>
  </div>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let c = find_id(&root, &doc, "c").expect("canvas 存在");
    assert!(
        (c.height - 100.0).abs() <= 0.5,
        "canvas 高应 200%×50px=100（CB 链穿 inline 壳对 cell 解析），实际 {}",
        c.height
    );
    assert!(
        (c.width - 100.0).abs() <= 0.5,
        "canvas 宽应按 1:1 ratio 推 100，实际 {}",
        c.width
    );
}

/// plain 树同构（div(50px) > span > block div height:50%）行为不变回归守卫：span 无盒
/// 形（R1044 纯块子）下内层块直连 definite 父，taffy 原生 % 解析 25——透明化不改变
/// 无壳路径（cb_through_inline=false 不代写）。
#[test]
fn r5514_plain_span_block_pct_unchanged() {
    let html = r#"<!DOCTYPE html><html><body style="margin:0">
<div style="height: 50px;">
  <span><div id=inner style="height: 50%; background: green;"></div></span>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let inner = find_id(&root, &doc, "inner").expect("inner div 存在");
    assert!(
        (inner.height - 25.0).abs() <= 0.5,
        "plain 无壳形内层块 50%×50px 应 25 不变，实际 {}",
        inner.height
    );
}

/// inline-block 中间层（块容器，auto 高）不透明：子 height:50% 对 indefinite CB
/// compute-to-auto 塌 0（CSS2 §10.5 既有语义，回归守卫）。
#[test]
fn r5514_inline_block_wrapper_pct_collapses_unchanged() {
    let html = r#"<!DOCTYPE html><html><body style="margin:0">
<div style="height: 50px;">
  <span style="display: inline-block;"><div id=inner style="height: 50%; background: green;"></div></span>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let inner = find_id(&root, &doc, "inner").expect("inner div 存在");
    assert!(
        inner.height <= 0.5,
        "inline-block（auto 高）内 50% 应 compute-to-auto 塌 0 不变，实际 {}",
        inner.height
    );
}
