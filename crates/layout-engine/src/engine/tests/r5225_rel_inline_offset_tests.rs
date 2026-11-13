//! R5225（css-position-3 §relative-positioning）display:inline 非替换盒 relative inset
//! 单次施加锚点。converter 对 display:inline 保留 taffy inset（R109 split 族依赖），
//! 后处理臂再施加 = 双重计数（abspos-inline-008 span 位移 2×−100px 实证）；本臂只对
//! inline-block（R5054：converter 已清 inset）与替换元素（inline layout 定位、taffy
//! inset 不消费）施加。LayoutBox.x 为父相对坐标——锚点以祖先链累加断言绝对位。

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

fn assert_abs(box_positions: &[(f32, f32)], expect_x: f32, label: &str) {
    assert!(!box_positions.is_empty(), "{label} 盒应存在于布局树");
    for (i, (ax, _)) in box_positions.iter().enumerate() {
        assert!(
            (ax - expect_x).abs() <= 0.5,
            "{label} 第 {i} 个盒 abs_x 应为 {expect_x}（relative 偏移单次施加），实际 {ax}"
        );
    }
}

/// 单盒命中位（abs_x, abs_y）列表。
type AbsHits = Vec<(f32, f32)>;

/// 按 fragment_node_ids 分桶收集同 dom_id 的 R109 split 族盒：shell（None，ZW 内部
/// 簿记）与 Inline 片段（Some，携 IFC 文本、chromium 可见面对应物）。
fn find_abs_positions_split(root: &LayoutBox, node_id: zero_dom::NodeId) -> (AbsHits, AbsHits) {
    let mut shells = Vec::new();
    let mut frags = Vec::new();
    fn walk(
        b: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        node_id: zero_dom::NodeId,
        shells: &mut Vec<(f32, f32)>,
        frags: &mut Vec<(f32, f32)>,
    ) {
        let ax = abs_x + b.x;
        let ay = abs_y + b.y;
        if b.node_id == Some(node_id) {
            if b.fragment_node_ids.is_some() {
                frags.push((ax, ay));
            } else {
                shells.push((ax, ay));
            }
        }
        for c in &b.children {
            walk(c, ax, ay, node_id, shells, frags);
        }
    }
    walk(root, 0.0, 0.0, node_id, &mut shells, &mut frags);
    (shells, frags)
}

/// display:inline 相对定位 + 仅 OOF 块级子（abspos-inline-008 形态）：taffy 单次施加，
/// 臂不重复——span 与 abspos 子（静态位随 CB）abs_x 均 = 100−100 = 0。
#[test]
fn r5225_rel_inline_oof_child_single_offset() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:100px; height:20px;">
  <span id=s style="position:relative; left:-100px;">
    <div id=abs style="position:absolute; width:50px; height:50px;"></div>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let s = doc.get_element_by_id("s").expect("span#s");
    let abs = doc.get_element_by_id("abs").expect("div#abs");
    let spos = find_abs_positions(&result.root, s);
    let apos = find_abs_positions(&result.root, abs);
    assert_abs(&spos, 0.0, "span#s");
    assert_abs(&apos, 0.0, "div#abs（静态位随 CB）");
}

/// R109 split inline 相对定位守卫（R5512 载体晶格）：span 的 relative inset 载体
/// 从 shell 移到各 Inline 匿名块片段（tree.rs Inline 片段臂保留 inset、shell 清零，
/// 防 taffy shell inset 与片段 inset 双载体双重施加）。chromium oracle（同 markup
/// headless 探针）：span 行盒 bbox (0,0)——片段并集即 −100 单次施加；shell 为 ZW
/// 内部簿记盒、chromium 无对应物，断言其位于容器内容原点 100（仅外层 +100 落壳，
/// inset 不落壳）。in-flow block 子随动（chromium 同受 −100 位移）需 CSS2
/// §9.2.1.1 容器级 anonymous block 编织（R4482 混排容器切片），本处不断言。
#[test]
fn r5225_rel_inline_split_offset_preserved() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:200px; height:40px;">
  <span id=s style="position:relative; left:-100px;">text
    <div style="width:50px; height:10px;"></div>
  </span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let s = doc.get_element_by_id("s").expect("span#s");
    let (shells, frags) = find_abs_positions_split(&result.root, s);
    assert_abs(&shells, 100.0, "split inline shell（inset 载体已移片段）");
    assert_abs(&frags, 0.0, "split inline Inline 片段（−100 单次施加）");
}

/// block-level 相对定位守卫：taffy 单次施加路径不变，嵌套块 abs_x = 0。
#[test]
fn r5225_rel_block_nested_offset_preserved() {
    let html = r#"<html><body style="margin:0">
<div style="position:relative; left:100px; width:200px; height:20px;">
  <div id=inner style="position:relative; left:-100px; width:50px; height:10px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let inner = doc.get_element_by_id("inner").expect("div#inner");
    let ipos = find_abs_positions(&result.root, inner);
    assert_abs(&ipos, 0.0, "block-in-block inner");
}
