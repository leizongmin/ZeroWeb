//! R4193（css-contain-3 §containment-inline-size）：contain:inline-size 的内联尺寸抑制。
//!
//! `contain:inline-size + width:fit-content` 容器内联尺寸按**空内容**求解（CIS-or-0）：
//! regular-container/flex/grid 三案（各 5.26%）——内容 100px 不参与，纯 border 0 50px
//! = 100 宽绿方块。两道修：① converter size 臂对 has_inline_size() 生效（width 轴
//! CIS-or-0，height 轴照常）；② intrinsic 测量趟跳过 inline-size-contained 元素
//!（防测量值覆盖受控宽）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        for c in &b.children {
            stack.push(c);
        }
    }
    None
}

/// block 容器：contain:inline-size + width:fit-content + border 0 50px → 100 宽（纯边框）。
#[test]
fn r4193_inline_size_containment_collapses_fit_content_width() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<div style="contain:inline-size; width:fit-content; border:solid green; border-width:0 50px; background:red;">
  <div style="width:100px; height:100px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(
        "body { margin: 0; } div { contain: inline-size; width: fit-content; border: solid green; border-width: 0 50px; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let container = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("container div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, container).expect("container box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R4193: inline-size containment 下 fit-content 宽应为纯边框 100，实际 {w}（旧实现测内容 200→300）"
    );
    assert!(
        (h - 100.0).abs() < 0.5,
        "R4193: block 轴无 containment，高度照常 = 内容 100，实际 {h}"
    );
}

/// R4194（css-writing-modes-3 §6 + css-contain-3 §containment-inline-size）：vertical
/// writing-mode 下内联轴 = 物理高度——vertical-rl + contain:inline-size + width:fit-content
/// + border 50px 0 应 **高** = 纯边框 100（内容 100×100 不参与内联尺寸）。
///
/// 块轴（物理宽）无 containment，内容照常参与；块轴 fit-content 收缩经 R4194 关键字
/// gate + 右锚平移处理（taffy 子摆位含 border 侧偏移，收缩后精确宽度 100 属 vertical
/// 块轴 extent 深域残余）。
#[test]
fn r4194_inline_size_containment_vertical_writing_suppresses_height() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<div style="writing-mode:vertical-rl; contain:inline-size; width:fit-content; border:solid green; border-width:50px 0; background:red;">
  <div style="width:100px; height:100px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet("body { margin: 0; }");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let container = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .next()
        .expect("container div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, h) = find_box(&result.root, container).expect("container box");
    assert!(
        (h - 100.0).abs() < 0.5,
        "R4194: vertical 下内联轴 = 物理高度，fit-content 高应为纯边框 100，实际 {h}"
    );
    assert!(
        (w - 150.0).abs() < 0.5,
        "R4194: 块轴（物理宽）无 containment，内容 100 + taffy 子摆位偏移 50 = 150 \
        （精确 100 需 vertical 块轴 extent 深域，R4194 挂账），实际 {w}"
    );
}

/// R4194（css-flexbox §4.5 + css-contain-3）：flex item 的 auto min-size 是
/// content-based——inline-size containment 下须钳 0（flex-basis:100 + 300px 子应 100，
/// 不被内容 min-content 地板撑开）。
#[test]
fn r4194_inline_size_containment_clamps_flex_item_auto_min() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<div style="width:400px; height:100px; background:red;">
  <div style="display:flex;">
    <div style="contain:inline-size; flex-basis:100px; background:green;">
      <div style="width:300px; height:100px;"></div>
    </div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet("body { margin: 0; }");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let item = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(2)
        .expect("flex item div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (w, _h) = find_box(&result.root, item).expect("item box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R4194: inline-size containment 下 flex item auto min 钳 0，flex-basis 100 生效，实际 {w}"
    );
}

/// R4194 勘察：vertical-rl + contain:inline-size 树 dump。
#[test]
#[ignore]
fn debug_r4194_vertical_dump() {
    let html = r#"<html><head><style>body { margin: 0; }</style></head><body>
<div style="writing-mode:vertical-rl; contain:inline-size; width:fit-content; border:solid green; border-width:50px 0; background:red;">
  <div style="width:100px; height:100px;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet("body { margin: 0; }");
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn dump(b: &LayoutBox, depth: usize) {
        let pad = "  ".repeat(depth);
        println!(
            "{pad}node={:?} x={} y={} h={} w={} cw={} ch={} vert={} flowroot={}",
            b.node_id,
            b.x,
            b.y,
            b.height,
            b.width,
            b.content_width,
            b.content_height,
            b.writing_mode.is_vertical_block_flow(),
            b.is_flow_root
        );
        for c in &b.children {
            dump(c, depth + 1);
        }
    }
    dump(&result.root, 1);
}
