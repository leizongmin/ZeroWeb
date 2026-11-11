//! R5506（CSS2 §10.3.9/§10.4 used 值约束）：inline-block shrink pass grow 补腿的
//! 自身 definite max-width 钳。
//!
//! 缺陷机制：`shrink_inline_blocks_to_content` horizontal 臂 R5348 grow 腿取流内子
//! 已布局 margin-box 最大者为 grow 目标后直接回写——taffy 初布局已把盒钳在
//! max-width，越钳回写把盒撑出钳外（inline-block 父 max-width:120px + 定宽
//! inline-block 原子子 300px → used 宽 300 应 120；块级子同形）。修复＝grow 目标
//! 按自身 definite max-width 钳（Calc 混合式排除沿 R5368
//! `clamp_box_max_contribution` 同款，% 项伪值解析）。锚全用定宽子盒，字体无关。
//!
//! 谱系：R5348（grow 补腿本体）、R5310/R5368（intrinsic 贡献侧同款钳）、
//! R5505（box_content_max_width 子递归贡献钳——测量域）；本钉收口消费端
//! used-width 域漏钳（同一 max-width 语义的双路径第三面）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.width);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 主判例（原子子形）：body 直下 inline-block span max-width:120px，内含定宽
/// inline-block 原子子 300px——taffy 初布局钳 120，grow 腿钳后目标 120 不越钳回写。
#[test]
fn r5506_atomic_child_own_maxwidth_caps_ib_width() {
    let html = r#"<html><body style="margin:0">
<span id="t" style="display:inline-block;max-width:120px;background:green">
  <span style="display:inline-block;width:300px;height:10px"></span>
</span>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let outer_id = *spans.first().expect("target outer span");
    let inner_id = *spans.last().expect("target inner span");
    let w = find_box(&result.root, outer_id).expect("outer box");
    let iw = find_box(&result.root, inner_id).expect("inner box");
    assert!(
        (w - 120.0).abs() < 0.5,
        "R5506: 自身 max-width 120 应钳住 grow 腿，实际 {w}"
    );
    assert!(
        (iw - 300.0).abs() < 0.5,
        "定宽原子子保持 300（溢出父盒，不得被同步），实际 {iw}"
    );
}

/// 窄容器变体：25px 容器内同形（taffy 初布局 25 < 钳 120）——grow 腿触发但目标钳到
/// 120，写 120（修复前写 300；kill-switch 关 grow 腿则滞留 25）。
#[test]
fn r5506_narrow_container_grow_leg_writes_capped_target() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <span style="display:inline-block;max-width:120px;background:green">
    <span style="display:inline-block;width:300px;height:10px"></span>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let outer_id = *spans.first().expect("target outer span");
    let w = find_box(&result.root, outer_id).expect("outer box");
    assert!(
        (w - 120.0).abs() < 0.5,
        "R5506: 窄容器 grow 腿应写钳后目标 120（修复前 300），实际 {w}"
    );
}

/// 块级子变体：定宽 block div 300px 同形钳制——div 自身 width 不被同步腿改写
///（definite 宽排除，R4023 同款）。
#[test]
fn r5506_block_child_own_maxwidth_caps_ib_width() {
    let html = r#"<html><body style="margin:0">
<span id="t" style="display:inline-block;max-width:120px;background:green">
  <div style="width:300px;height:10px"></div>
</span>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let outer_id = *spans.first().expect("target outer span");
    let divs = doc.get_elements_by_tag_name("div");
    let inner_id = *divs.last().expect("target inner div");
    let w = find_box(&result.root, outer_id).expect("outer box");
    let iw = find_box(&result.root, inner_id).expect("inner box");
    assert!((w - 120.0).abs() < 0.5, "R5506: 块级子同形应钳在 120，实际 {w}");
    assert!(
        (iw - 300.0).abs() < 0.5,
        "definite 宽块级子保持 300（同步腿排除），实际 {iw}"
    );
}

/// R5348 基线保持：无 max-width 时 grow 腿仍写到子外宽（窄容器 25 → 300 溢出），
/// 钳只作用于有 definite max-width 的盒。
#[test]
fn r5506_grow_baseline_unclamped_without_maxwidth() {
    let html = r#"<html><body style="margin:0">
<div style="width:25px;height:50px">
  <span style="display:inline-block;background:green">
    <span style="display:inline-block;width:300px;height:10px"></span>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let outer_id = *spans.first().expect("target outer span");
    let w = find_box(&result.root, outer_id).expect("outer box");
    assert!(
        (w - 300.0).abs() < 0.5,
        "R5348 基线：无 max-width 时 grow 到子外宽 300，实际 {w}"
    );
}

/// min-width 地板保持：min-width:200 + max-width:120 冲突时 min 胜（CSS2 §10.4），
/// 钳后 grow 目标 120 < 当前 200 不回写，盒维持 200。
#[test]
fn r5506_min_width_floor_survives_cap_conflict() {
    let html = r#"<html><body style="margin:0">
<span style="display:inline-block;min-width:200px;max-width:120px;background:green">
  <span style="display:inline-block;width:300px;height:10px"></span>
</span>
</body></html>"#;
    let (doc, result) = layout(html);
    let spans = doc.get_elements_by_tag_name("span");
    let outer_id = *spans.first().expect("target outer span");
    let w = find_box(&result.root, outer_id).expect("outer box");
    assert!(
        (w - 200.0).abs() < 0.5,
        "min-width 200 地板应保持（min 胜 max），实际 {w}"
    );
}
