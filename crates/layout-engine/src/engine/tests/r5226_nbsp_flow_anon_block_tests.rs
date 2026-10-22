//! R5226（CSS2 §9.2.1.1 + CSS Text §3）nbsp-only inline 内容推进块流锚点。
//! U+00A0 是 preserved 内容（永不折叠），块容器混合内容下须生成匿名块盒推进
//! 后继块；空白判定用可折叠白空间集合（排除 nbsp/U+3000），源码缩进/换行等
//! 可折叠白空间维持 collapse-through。LayoutBox.x/y 为父相对坐标——锚点以
//! 祖先链累加断言绝对位。
//!
//! 已知边界（本轮不动，归档挂账）：仅 OOF 子（abspos/float，无 in-flow 块级子）
//! 的容器不触发 §9.2.1.1 拆分（has_block=false），nbsp 行不参与其静态位
//!（between-float-and-text 的 abspos 臂；float 因「y = flow_bottom + margin-top」
//! 模型与 §9.5.1 行顶锚在单行形态巧合同值，见 goal 挂账）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_abs_positions(root: &LayoutBox, node_id: zero_dom::NodeId) -> Vec<(f32, f32)> {
    let mut hits = Vec::new();
    fn walk(b: &LayoutBox, abs_x: f32, abs_y: f32, node_id: zero_dom::NodeId, hits: &mut Vec<(f32, f32)>) {
        let ax = abs_x + b.x;
        let ay = abs_y + b.y;
        if b.node_id == Some(node_id) {
            hits.push((ax, ay));
        }
        for c in &b.children {
            walk(c, ax, ay, node_id, hits);
        }
    }
    walk(root, 0.0, 0.0, node_id, &mut hits);
    hits
}

fn assert_abs_y(positions: &[(f32, f32)], expect_y: f32, label: &str) {
    assert!(!positions.is_empty(), "{label} 盒应存在于布局树");
    for (i, (_, ay)) in positions.iter().enumerate() {
        assert!(
            (ay - expect_y).abs() <= 0.5,
            "{label} 第 {i} 个盒 abs_y 应为 {expect_y}，实际 {ay}"
        );
    }
}

/// nbsp-only 文本后随 in-flow 块级子：nbsp 须生成匿名块盒（行高 20px）推进流，
/// 后继块落在 y=20（chromium 探针同值；旧 trim() 口径判空白跳过 → 后继块 y=0）。
#[test]
fn r5226_nbsp_only_text_advances_block() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  &nbsp;
  <div id=blk style="width:100px; height:10px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let blk = doc.get_element_by_id("blk").expect("div#blk");
    let pos = find_abs_positions(&result.root, blk);
    assert_abs_y(&pos, 20.0, "nbsp 行后的后继块");
}

/// 真实文本后随块级子守卫：mixed 拆分既有推进行为不变（对照锚）。
#[test]
fn r5226_real_text_advances_block() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  hello
  <div id=blk style="width:100px; height:10px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let blk = doc.get_element_by_id("blk").expect("div#blk");
    let pos = find_abs_positions(&result.root, blk);
    assert_abs_y(&pos, 20.0, "真实文本行后的后继块");
}

/// 可折叠白空间（源码缩进/换行）后随块级子守卫：collapse-through 语义不变——
/// 纯可折叠空白片段仍跳过匿名块生成，后继块 y=0。
#[test]
fn r5226_collapsible_ws_only_still_collapses_through() {
    let html = "<html><body style=\"margin:0\">\n<div style=\"line-height:20px;\">\n  \n  <div id=blk style=\"width:100px; height:10px;\"></div>\n</div>\n</body></html>";
    let (doc, result) = layout(html);
    let blk = doc.get_element_by_id("blk").expect("div#blk");
    let pos = find_abs_positions(&result.root, blk);
    assert_abs_y(&pos, 0.0, "可折叠白空间后的后继块（塌穿保留）");
}

/// float 前为 inline 文本：float.y = 行顶 + margin-top（§9.5.1 行顶对齐；
/// chromium 探针 20 同值）——nbsp 修复不得扰动 float 的行顶锚模型。
#[test]
fn r5226_float_after_text_line_top_anchor() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  hello
  <div id=fl style="float:left; margin-top:20px; width:100px; height:50px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let fl = doc.get_element_by_id("fl").expect("div#fl");
    let pos = find_abs_positions(&result.root, fl);
    assert_abs_y(&pos, 20.0, "文本后 float（行顶锚）");
}

/// float 前为块级子：float.y = 前块底 + margin-top（§9.5.1 流游标锚；
/// chromium 探针 50 同值）。
#[test]
fn r5226_float_after_block_cursor_anchor() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  hello
  <div style="height:10px;"></div>
  <div id=fl style="float:left; margin-top:20px; width:100px; height:50px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let fl = doc.get_element_by_id("fl").expect("div#fl");
    let pos = find_abs_positions(&result.root, fl);
    assert_abs_y(&pos, 50.0, "块盒后 float（流游标锚）");
}
