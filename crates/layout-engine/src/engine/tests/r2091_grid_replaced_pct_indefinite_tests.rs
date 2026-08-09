//! R2091：grid/flex item 中 replaced 元素 + Percentage height + indefinite CB 修正。
//! driving case：`<canvas width=10 height=10 style="height:200%">` 作 grid item 时
//! 不再 double-resolve 到 400×400，而按 intrinsic 解析（auto+intrinsic 经 R2016 else 分支）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

/// 在布局树中找指定 tag 的第一个 LayoutBox，返回其 (height, width)。
fn find_first(doc: &zero_dom::Document, root: &LayoutBox, tag: &str) -> Option<(f32, f32)> {
    let mut stack: Vec<&LayoutBox> = vec![root];
    while let Some(b) = stack.pop() {
        let is_target = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == tag))
            .unwrap_or(false);
        if is_target {
            return Some((b.height, b.width));
        }
        stack.extend(&b.children);
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone()))
}

/// R2091 主驱：canvas（grid item，height:200%，indefinite grid）→ 应 ~intrinsic（10），
/// 非 400×400（double-resolve bug）。
#[test]
fn r2091_grid_item_replaced_percentage_height_uses_intrinsic() {
    let html = r#"<div style="width: 100px; height: 100px; overflow: hidden; background: green;">
  <div style="display: grid; position: relative; left: -20px;">
    <canvas width=10 height=10 style="height: 200%; background: red;"></canvas>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let (h, w) = find_first(&doc, &root, "canvas").expect("canvas not found");
    // 修复前：h=400 w=400（double-resolve）。修复后：~intrinsic 10（auto+intrinsic）。
    assert!(
        h <= 20.0 && w <= 20.0,
        "canvas should resolve to ~intrinsic (10) not double-resolved 400; got h={h} w={w}"
    );
}

// 注：grid 容器 **definite height**（如 height:100px）+ canvas height:200% 的 definite-CB
// 场景当前**另有 pre-existing bug**（canvas 解析为 h=3136 而非 200，taffy 对 definite-grid +
// replaced-percentage + aspect-ratio 的另一 double-resolve 路径）。R2091 gate 显式排除
// definite CB（cb_definite.is_some() 不触发），不触及该路径——属独立 follow-up，非本 fix 范围。

/// R2091 kill-switch：ZW_GRID_REPLACED_PCT_INDEFINITE=0 时回退旧行为（canvas 大尺寸）。
/// 注：env-var 测试在 cargo test 并行下与主驱测试 race（env 进程全局），故不写成 #[test]，
/// 改在生产 A/B（reftest-oracle）验证 kill-switch 行为；此处仅文档化开关名。
const _R2091_KILL_SWITCH_ENV: &str = "ZW_GRID_REPLACED_PCT_INDEFINITE";

/// 收集函数单元测试：canvas HTML attrs 进 map，img 不收（caller 已填），无 attr 不收。
#[test]
fn r2091_gather_replaced_html_attr_intrinsic_collects_canvas_only() {
    let html = r#"<div style="display:grid">
  <canvas width=10 height=20></canvas>
  <canvas width=30></canvas>
  <embed src="x" width=40 height=50>
  <img src="x.png" width=60 height=70>
  <div width=80 height=90></div>
</div>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let map = LayoutEngine::gather_replaced_html_attr_intrinsic(&doc, &result.root);
    // canvas(10,20) + embed(40,50) 进 map；canvas 缺 height 不进；img 不收；div 非 replaced 不收。
    assert!(
        map.values().any(|v| *v == (10.0, 20.0)),
        "canvas 10x20 should be gathered"
    );
    assert!(
        map.values().any(|v| *v == (40.0, 50.0)),
        "embed 40x50 should be gathered"
    );
    assert!(
        !map.values().any(|v| *v == (60.0, 70.0)),
        "img should NOT be gathered (caller fills decoded dims)"
    );
}
