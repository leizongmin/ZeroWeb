//! t-fup-s24-flip-red 返修回归（s24 flip 布局域缺口二则，双审查 D1/D2 发现）：
//! ① UA 表 canvas 默认已翻 display:inline——fallback 抑制面（HTML §4.8.10：fallback
//!   仅在元素不支持时显示，任意 display 下不参与布局）在三条路径漏 Inline：
//!   `is_replaced_with_fallback` display 门、`is_inline_r109` 拆分宿主判据（直读
//!   DOM）、R5251 纯块载体升格（直读 DOM）。泄漏形态：fallback `<p>` 建盒 + canvas
//!   整盒消失。本测钉「任意 display 下 fallback 不建盒 + canvas 盒保持 attr 固有
//!   尺寸」。
//! ② `vertical_block_flow.rs` 两处 block-level 子判据（vrl 根右缘重锚 + vertical 容器
//!   块轴堆叠）漏 ListItem——UA 默认 li 翻 ListItem 后（converter 映射 taffy Block，
//!   engine.rs is_block_level 同列）缺席堆叠 pass，vertical 容器下 ≥2 个 li 同位
//!   塌叠。本测钉「vrl 容器下 UA li 沿块轴（物理 x）右起堆叠不塌叠」。

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

/// 按元素名收集盒几何（先序遍历，DOM 序），返回 (x, y, width, height)。
fn collect_geoms_by_tag(doc: &zero_dom::Document, root: &LayoutBox, tag: &str) -> Vec<(f32, f32, f32, f32)> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        let hit = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == tag))
            .unwrap_or(false);
        if hit {
            out.push((b.x, b.y, b.width, b.height));
        }
        stack.extend(b.children.iter().rev());
    }
    out
}

/// D1：UA 默认 display:inline 的 canvas（无 CSS display，走翻转后 UA 表）带元素
/// fallback 子——canvas 盒保持 attr 固有 100×50，fallback `<p>` 不建盒。
#[test]
fn s24_flip_inline_canvas_fallback_children_suppressed() {
    let html = r#"<canvas width="100" height="50"><p class="fallback">FAIL (fallback content)</p></canvas>"#;
    let (doc, root) = layout(html);
    let canvases = collect_geoms_by_tag(&doc, &root, "canvas");
    assert_eq!(canvases.len(), 1, "inline canvas 仍须持有 LayoutBox");
    let (x, y, w, h) = canvases[0];
    assert!(
        (w - 100.0).abs() < 0.5 && (h - 50.0).abs() < 0.5,
        "canvas 应保持 attr 固有 100×50（fallback 子不得撑高——margin-collapse 顶移 \
         body 即此泄漏），实际 x={x} y={y} {w}x{h}"
    );
    let ps = collect_geoms_by_tag(&doc, &root, "p");
    assert!(
        ps.is_empty(),
        "受支持 canvas 的 fallback 子（<p>）不得建盒（HTML §4.8.10 任意 display 下 \
         排除；ghost 盒会泄入 gBCR/hit-testing），实际 {ps:?}"
    );
}

/// D2：vertical-rl 容器下 2 个 UA 默认 li（display:list-item）沿块轴（物理 x）
/// 右起堆叠——不得因 block-level 判据漏 ListItem 而同位塌叠。
#[test]
fn s24_flip_vrl_ua_li_children_stack_along_block_axis() {
    let html = r#"<div style="writing-mode: vertical-rl; height: 400px;"><li>alpha</li><li>beta</li></div>"#;
    let (doc, root) = layout(html);
    let lis = collect_geoms_by_tag(&doc, &root, "li");
    assert_eq!(lis.len(), 2, "两个 li 都须持有 LayoutBox，实际 {lis:?}");
    let (x0, _, w0, h0) = lis[0];
    let (x1, _, w1, h1) = lis[1];
    assert!(
        w0 > 0.5 && h0 > 0.5 && w1 > 0.5 && h1 > 0.5,
        "li 盒不得塌 0，实际 {lis:?}"
    );
    assert!(
        (x0 - x1).abs() > 1.0,
        "vrl 块轴（物理 x）上两个 li 须各占一条（右起堆叠），不得同位塌叠，实际 {lis:?}"
    );
    assert!(x0 > x1, "vertical-rl 首个 li 应在最右（块流右缘起排），实际 {lis:?}");
}
