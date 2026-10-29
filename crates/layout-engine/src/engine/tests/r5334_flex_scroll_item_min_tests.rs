//! R5334：scroll container flex item 自动最小尺寸 = 0（css-flexbox-1 §4.5 +
//! css-overflow-3 §3.1）。
//!
//! 根因：column flex item（`min-height:auto` + `overflow-y:scroll`）的自动最小尺寸
//! 依 §4.5 应为 0（scroll container 无 content-based minimum），taffy 侧已按
//! `Overflow::maybe_into_automatic_min_size` → 0 定尺；但 R1743 父高回填把
//! height:auto 的 item 当「内容决定型 auto 高容器」撑回子 max-bottom（R5330 豁免
//! 仅覆盖指定 min-height 非 auto 的 item），fieldset-as-item-overflow：flex 分配
//! 100px 被撑回 200px 溢出 100×100 容器。修复 = R5330 豁免谓词扩展 scroll
//! container 半区（computed overflow 经 §3.1 传播后任一轴非 visible/clip）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::{Document, NodeKind};

fn compute(html: &str) -> (Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

/// 找指定标签首个元素盒（深度优先）。
fn find_tag_box<'a>(b: &'a LayoutBox, doc: &'a Document, tag: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.local_name() == tag
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_tag_box(c, doc, tag))
}

/// 驱动腿（fieldset-as-item-overflow 形）：column 容器 100×100 + border，fieldset
/// item（margin/border/padding 0 + overflow-y:scroll + flex:1，min-height:auto）
/// 含 200px 高子块——scroll container 自动最小 = 0，item 保持 flex 分配高 100，
/// 不被 R1743 回填撑回子底 200 溢出容器（chromium bug 1149714）。
#[test]
fn r5334_scroll_container_item_keeps_flexed_height() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column;width:100px;height:100px;border:solid;">
  <fieldset style="margin:0;border:none;padding:0;overflow-y:scroll;overflow-x:hidden;flex:1;">
    <div style="height:200px"></div>
  </fieldset>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    let item = find_tag_box(&root, &doc, "fieldset").expect("fieldset item");
    assert!(
        (item.height - 100.0).abs() < 1.0,
        "scroll container flex item 应保持 flex 分配高 100（§4.5 automatic min = 0），实 {:.1}（回填撑回内容高 200 = 缺陷）",
        item.height
    );
}

/// 对照腿：overflow visible 的 column item（min-height:auto）automatic minimum =
/// min-content 200——R1743 回填保留（R5330 短板补偿位），豁免不得扩到 visible 域。
#[test]
fn r5334_visible_overflow_item_content_min_preserved() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:column;width:100px;height:100px;">
  <div style="margin:0;padding:0;flex:1;">
    <div style="height:200px"></div>
  </div>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    // 首个 div = 容器，取其子 item：容器深搜首个匹配的 item div（第二个 div）。
    let item = find_tag_box(&root, &doc, "div")
        .and_then(|container| container.children.first())
        .expect("flex item");
    assert!(
        (item.height - 200.0).abs() < 1.0,
        "overflow visible item 自动最小 = min-content 200 应保留，实 {:.1}",
        item.height
    );
}

/// 守卫腿：row 容器 item 的 height 是 cross 轴——overflow-x:scroll item 宽保持 flex
/// 分配（R4265 min-width:auto 禁用面既有行为），豁免域不扩 row。
#[test]
fn r5334_row_axis_cross_domain_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex;flex-direction:row;width:100px;height:100px;border:solid;">
  <div style="margin:0;border:none;padding:0;overflow-x:scroll;overflow-y:hidden;flex:1;">
    <div style="width:200px;height:20px"></div>
  </div>
</div>
</body></html>"#;
    let (doc, root) = compute(html);
    let item = find_tag_box(&root, &doc, "div")
        .and_then(|container| container.children.first())
        .expect("flex item");
    assert!(
        (item.width - 100.0).abs() < 1.0,
        "row item 主轴宽应保持 flex 分配 100（automatic min 禁用既有行为），实 {:.1}",
        item.width
    );
}
