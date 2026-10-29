//! R5332：flex 容器 box-sizing:border-box 时 transferred/stretch 目标按 content box
//! cross 计算（css-flexbox-1 §4.5 transferred size suggestion、§9.4 align-self:stretch）。
//!
//! 根因（两处同根）：容器 CSS 尺寸在 border-box 下含 frame，原值直取会把 item 建在
//! 容器 outer cross 上——①tree.rs transferred-suggestion 臂把容器 CSS height 当容器
//! cross → min_size 建在 200（flex-cross-size-border-box-001：容器 400×200 border-box
//! + border 10，img 固有 1×1 被 min 钳到 200×200，应 stretch 180×180）；②sizing.rs
//! R1364 写回臂把 `st.size.height` 写成容器 CSS 原值 → 第二趟 img cross 变 definite
//! 不再 stretch。修复 = 两处都按容器 box_sizing=BorderBox 扣 frame（content-box 原值
//! 即 content，零变化）。kill-switch = `ZW_FLEX_TRANSFERRED_CONTENT_CROSS=0`
//!（default-on，两处同一开关）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use std::sync::Arc;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::StyleSystem;

/// 解析 + 布局（带 1×1 img 解码固有尺寸注册，复刻 runner 口径）。
fn compute_with_tiny_img(html: &str) -> (Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    // 先跑一遍拿 img NodeId（runner 对已解码 img 注册固有尺寸后再重跑）
    let mut engine0 = LayoutEngine::new(800.0, 600.0);
    let root0 = Arc::try_unwrap(engine0.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    let img_id = find_node_id(&doc, &root0, "img").expect("img node");
    let mut sizes = HashMap::new();
    sizes.insert(img_id, (1.0, 1.0));
    let mut ratios = HashMap::new();
    ratios.insert(img_id, 1.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute_with_img_sizes(&doc, &styles, sizes, ratios).root)
        .unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

/// 找指定 id 的 LayoutBox（深度优先）。
fn find_id_box<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id_box(c, doc, id))
}

/// 找指定 tag 名（首个）的 NodeId。
fn find_node_id(doc: &Document, b: &LayoutBox, want_tag: &str) -> Option<NodeId> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.local_name().eq_ignore_ascii_case(want_tag)
    {
        return Some(nid);
    }
    b.children.iter().find_map(|c| find_node_id(doc, c, want_tag))
}

/// 驱动腿（flex-cross-size-border-box-001 形）：border-box 容器 400×200 + border 10 →
/// content 180 高，img（固有 1×1，ratio 1）stretch 应 180×180——transferred min 与
/// R1364 写回都不落在容器 outer cross 200 上。
#[test]
fn r5332_border_box_row_img_stretches_to_content_cross() {
    let html = r##"<html><body>
<div id="flex" style="display:flex;width:400px;height:200px;box-sizing:border-box;border:10px solid transparent;">
  <img src="support/1x1-green.png">
</div>
</body></html>"##;
    let (doc, root) = compute_with_tiny_img(html);
    let flex = find_id_box(&root, &doc, "flex").expect("flex");
    let img = flex.children.first().expect("img item");
    assert!(
        (flex.content_height - 180.0).abs() < 1.0,
        "容器 content 高 h={:.1} 应 180 = 200 − border 2×10（box-sizing:border-box）",
        flex.content_height
    );
    assert!(
        (img.width - 180.0).abs() < 1.0 && (img.height - 180.0).abs() < 1.0,
        "img {}×{} 应 stretch 180×180 = 容器 content cross × ratio（css-flexbox-1 §4.5/§9.4），非 outer cross 200（R5332）",
        img.width,
        img.height
    );
}

/// 对照腿：content-box 容器 CSS 尺寸即 content，border 不占额——img 应 200×200
///（R5332 修复对 content-box 容器零变化）。
#[test]
fn r5332_content_box_container_unaffected() {
    let html = r##"<html><body>
<div id="flex" style="display:flex;width:400px;height:200px;border:10px solid transparent;">
  <img src="support/1x1-green.png">
</div>
</body></html>"##;
    let (doc, root) = compute_with_tiny_img(html);
    let flex = find_id_box(&root, &doc, "flex").expect("flex");
    let img = flex.children.first().expect("img item");
    assert!(
        (img.width - 200.0).abs() < 1.0 && (img.height - 200.0).abs() < 1.0,
        "content-box 容器 img {}×{} 应 200×200 = CSS cross 原值（content box 即 content）",
        img.width,
        img.height
    );
}

/// column 腿：border-box 容器 width:400 + border 10 → content 宽 380，img（ratio 1）
/// cross 380、main = 380×1 = 380——column 臂走 frame_width 扣减路径。
#[test]
fn r5332_border_box_column_img_stretches_to_content_cross() {
    let html = r##"<html><body>
<div id="flex" style="display:flex;flex-direction:column;width:400px;box-sizing:border-box;border:10px solid transparent;">
  <img src="support/1x1-green.png">
</div>
</body></html>"##;
    let (doc, root) = compute_with_tiny_img(html);
    let flex = find_id_box(&root, &doc, "flex").expect("flex");
    let img = flex.children.first().expect("img item");
    assert!(
        (img.width - 380.0).abs() < 1.0 && (img.height - 380.0).abs() < 1.0,
        "column border-box 容器 img {}×{} 应 380×380 = content 宽 400−2×10 × ratio（css-flexbox-1 §4.5/§9.4）",
        img.width,
        img.height
    );
}
