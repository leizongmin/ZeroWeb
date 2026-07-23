//! R1964 诊断 + characterization：vertical-rl float 的 height=inf 定位。
//!
//! taffy 零 vertical-mode awareness，ZW 经 apply_vertical_writing_mode 轴交换 emulate vertical
//!（R1963）。本测试实证定位 inf 源头：vertical-rl float 的 **block 子（`<p>`）** 经 taffy
//! 轴交换 block layout 得 height=inf（auto-height vertical block 在 unbounded float 上下文），
//! 传播到 float。horizontal 对照 finite。
//!
//! characterization：当前 vertical `<p>` height **is_infinite=true（bug）**。当 R109 vertical
//! block-height 经 ZW-interface fix 修好后，此断言会失败 → 改为 assert finite（≈文本 inline
//! extent）。保留作 durable 定位数据 + fix 的 success signal。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::FloatValue;
use zero_style_system::StyleSystem;
use zero_style_system::WritingModeValue;

/// 深度优先找到第一个 float:left 的 LayoutBox。
fn find_float_left(root: &LayoutBox) -> Option<&LayoutBox> {
    if matches!(root.float, FloatValue::Left) {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_float_left(child) {
            return Some(f);
        }
    }
    None
}

#[test]
fn r1964_diag_vertical_rl_float_height() {
    let html = r#"<html><body style="margin:0">
<div style="float:left; writing-mode:vertical-rl; background:red;">
  <p>hello world test text vertical</p>
</div>
<div style="width:200px;height:200px"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_div = find_float_left(&result.root).expect("should find a float:left div");
    eprintln!(
        "R1964 VERTICAL-RL float: width={} height={} (is_infinite={}) writing_mode={:?}",
        float_div.width,
        float_div.height,
        float_div.height.is_infinite(),
        float_div.writing_mode
    );
    // 走 float 子树，定位 inf 源头（float 本身 vs 其 block 子如 <p>）。
    let mut stack: Vec<(&LayoutBox, usize)> = float_div.children.iter().map(|c| (c, 1)).collect();
    while let Some((b, depth)) = stack.pop() {
        let tag = b.node_id.and_then(|id| doc.get(id).map(|n| n.kind.clone()));
        eprintln!(
            "R1964   child[{}]: width={} height={} (is_infinite={}) wm={:?} {:?}",
            depth,
            b.width,
            b.height,
            b.height.is_infinite(),
            b.writing_mode,
            tag.map(|k| format!("{:?}", k)).unwrap_or_default()
        );
        for c in &b.children {
            stack.push((c, depth + 1));
        }
    }

    // 对照：horizontal float（同结构无 writing-mode）。
    let html_h = r#"<html><body style="margin:0">
<div style="float:left; background:red;">
  <p>hello world test text horizontal</p>
</div>
<div style="width:200px;height:200px"></div>
</body></html>"#;
    let doc_h = zero_dom::parse_html(html_h);
    let styles_h = sys.compute_styles(&doc_h, &[]);
    let mut engine_h = LayoutEngine::new(800.0, 600.0);
    let result_h = engine_h.compute(&doc_h, &styles_h);
    let float_h = find_float_left(&result_h.root).expect("should find horizontal float:left div");
    eprintln!(
        "R1964 HORIZONTAL float: width={} height={} (is_infinite={}) writing_mode={:?}",
        float_h.width,
        float_h.height,
        float_h.height.is_infinite(),
        float_h.writing_mode
    );

    // 诊断断言：仅报告，不强制（vertical 当前预期 inf = bug，horizontal finite = 正常）。
    let _ = float_div.writing_mode == WritingModeValue::VerticalRl;

    // characterization（R1964）：vertical-rl float 的 block 子（<p>）当前 height=inf（bug）。
    // 定位 inf 源头 = <p> block（非 float 本身）。fix 后此断言失败 → 改 assert finite。
    let p_child = float_div.children.iter().find(|c| {
        c.node_id.is_some_and(|id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(_)))
        })
    });
    if let Some(p) = p_child {
        assert!(
            p.height.is_infinite(),
            "R1964 characterization: vertical-rl <p> block height should currently be inf (bug), got {}",
            p.height
        );
    }
}
