//! t-p116 回归：原子行内盒定位 IFC（`adjust_inline_block_positions` 自建）与
//! intrinsic 测量的文本 advance 同源（CSS2 §9.2.1.1 + §10.8）。
//!
//! 缺陷机制：intrinsic（taffy measure，带注入 advance source）按真实 face 测得
//! shrink-to-fit 容器宽；定位 pass 的 IFC 此前裸构造（无字体上下文）走
//! `estimate_char_width` 启发式（字母 0.55em）。face advance < 启发式时（探针
//! 实证：monospace face 0.5em/char vs 启发式 11px），exact-fit 容器内原子盒被
//! 误折到第二行（浏览器实测 div=max-content 70、span y=20 x=0，应单行）。
//! face advance > 启发式时反向：原子盒 x 偏小（位置漂移）。
//! 修前单元复现：注入 0.45em face → 容器 63（真实 18+45），启发式 22+45=67>63 折行。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::rc::Rc;
use zero_dom::NodeId;

/// 注入 face：每字符固定 `ratio`×font_size（等宽语义，空格同宽），
/// 故意 ≠ 启发式的字母 0.55em/空格 0.25em，放大双路径分歧。
struct FaceAdvance {
    ratio: f32,
}

impl crate::inline::AdvanceSource for FaceAdvance {
    fn measure(&self, _ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        if is_ahem {
            return font_size;
        }
        match font_id {
            Some(1) => font_size * self.ratio,
            _ => font_size * 0.5,
        }
    }
}

fn layout_with_face(ratio: f32, html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "body { margin: 0; font: 20px/1 monospace } div { margin: 0 } .ib { display: inline-block; background: #0f0 }",
    );
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let mut resolver = std::collections::HashMap::new();
    resolver.insert("monospace".to_string(), 1u32);
    engine.set_font_resolver(resolver);
    engine.set_advance_source(Rc::new(FaceAdvance { ratio }));
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// face advance（0.45em=9px）< 启发式（字母 11px）：收缩容器按真实 face 测宽，
/// 原子盒不得因启发式 advance 被误折行。修前 span y=20（第二行）、x=0。
#[test]
fn p116_advance_source_narrower_than_estimate_no_wrap() {
    let (doc, result) = layout_with_face(
        0.45,
        "<html><body><div style=\"width:max-content\">hi<span class=ib>ab cd</span></div></body></html>",
    );
    let div_id = doc.get_elements_by_tag_name("div").into_iter().next().expect("div");
    let span_id = doc.get_elements_by_tag_name("span").into_iter().next().expect("span");
    let (_dx, _dy, dw, _dh) = find_box(&result.root, div_id).expect("div box");
    let (x, y, _w, _h) = find_box(&result.root, span_id).expect("span box");
    // 容器宽 = 真实 face 口径：18（hi）+ 45（ab cd，空格同宽）= 63。
    assert!((dw - 63.0).abs() < 1.0, "容器宽应按注入 face 测得 63，got {dw}");
    let text_advance = 2.0 * 20.0 * 0.45; // = 18
    assert!(
        y < 18.0,
        "原子盒应落首行带内（<18），got {y}（≥行高 = 启发式 advance 误折行，t-p116 缺陷）"
    );
    assert!(
        (x - text_advance).abs() < 1.0,
        "原子盒 x 应等于 face 文本推进 {text_advance}，got {x}"
    );
}

/// face advance（0.65em=13px）> 启发式（字母 11px）：反向漂移——原子盒 x 须按
/// face 推进（26），修前启发式给出 22（偏小 4px）。
#[test]
fn p116_advance_source_wider_than_estimate_position() {
    let (doc, result) = layout_with_face(
        0.65,
        "<html><body><div style=\"width:max-content\">hi<span class=ib>ab</span></div></body></html>",
    );
    let span_id = doc.get_elements_by_tag_name("span").into_iter().next().expect("span");
    let (x, y, _w, _h) = find_box(&result.root, span_id).expect("span box");
    let text_advance = 2.0 * 20.0 * 0.65; // = 26
    assert!(y < 18.0, "原子盒应落首行带内，got {y}（容器 26+52=78 自洽，不该折行）");
    assert!(
        (x - text_advance).abs() < 1.0,
        "原子盒 x 应按 face 推进 {text_advance}，got {x}（启发式 22 = 位置漂移，t-p116 缺陷）"
    );
}
