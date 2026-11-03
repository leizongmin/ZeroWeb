//! R5401：legacy `-webkit-box` 水平语义容器忽略 item `align-self`（css-align-3
//! overflow-values 适用性）。
//!
//! 谱系：WPT `css/css-flexbox/flexbox-safe-overflow-position-006.html`（✗0.00% 基线
//! → R5401 safe 前缀可解析后 0.42% 近似回归）。案面 assert「align-self:safe has no
//! effect on -webkit-box」：chromium legacy box 的 item 对齐只由 `-webkit-box-align`
//! 控制。ZW 把 `-webkit-box` 映射 flex（cascade R3798）+ `-webkit-box-align` 映射
//! align-items；R5401 起 `align-self: safe center` 可解析 → item 被改写为 safe-center
//! 溢出回退 start，脱离容器 align-items。修复：cascade 胜者溯源合成键
//! `-webkit-box-flex-origin` → inheritance 置 `legacy_webkit_box_flex` → tree.rs
//! build_subtree 父上下文抑制子项 align_self（置 None = auto 回落容器 align-items）。
//!
//! kill-switch：`ZW_WEBKIT_BOX_ALIGN_SELF_IGNORE=0`（另文件独立进程锚，R3858 口径）。
//!
//! https://drafts.csswg.org/css-align-3/#overflow-values

use std::collections::HashMap;
use zero_css_parser::values::{AlignmentValue, DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

/// 006 形：legacy box 容器 90×90 + `-webkit-box-align: end`（align_items: End）+
/// item 100×100 `align-self: safe center`。
fn build_006(legacy_flag: bool) -> (zero_dom::Document, HashMap<NodeId, ComputedStyle>, NodeId, NodeId) {
    use zero_dom::Document;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let item = doc.create_element("div");
    doc.append_child(container, item).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        container,
        ComputedStyle {
            display: DisplayValue::Flex,
            width: LengthValue::Px(90.0),
            height: LengthValue::Px(90.0),
            align_items: AlignmentValue::End,
            // cascade 溯源产物：`-webkit-box` 映射 flex 胜出 → true；作者真 flex → false。
            legacy_webkit_box_flex: legacy_flag,
            ..Default::default()
        },
    );
    styles.insert(
        item,
        ComputedStyle {
            display: DisplayValue::Block,
            width: LengthValue::Px(100.0),
            height: LengthValue::Px(100.0),
            align_self: AlignmentValue::SafeCenter,
            ..Default::default()
        },
    );
    (doc, styles, container, item)
}

fn find_box(tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if tree.node_id == Some(id) {
        return Some(tree);
    }
    tree.children.iter().find_map(|c| find_box(c, id))
}

fn compute(doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) -> zero_layout_engine::LayoutResult {
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(doc, styles)
}

/// ★ 006 主形态：legacy box 容器抑制 item align-self——item 回落容器 align-items:end
/// （cross 溢出 10px，end 对齐 → 顶部溢出 y−10），不被 safe-center 回退 start 改写。
#[test]
fn legacy_webkit_box_container_suppresses_item_align_self() {
    let (doc, styles, container, item) = build_006(true);
    let result = compute(&doc, &styles);
    let c = find_box(&result.root, container).expect("container 布局盒存在");
    let i = find_box(&result.root, item).expect("item 布局盒存在");
    let dy = i.y - c.y;
    assert!(
        (dy - (-10.0)).abs() <= 0.5,
        "legacy box: item 必须按容器 align-items:end 落位（y−10）；dy={dy} \
         （align-self 泄漏时 safe-center 溢出回退 start → dy=0）"
    );
}

/// 真 flex 对照守卫：同一几何、无 legacy 标记——safe center 溢出回退 start 正常生效
/// （R5401 解析 + taffy apply_alignment_fallback 能力锚，flexbox-safe-overflow-position-001
/// 语义），不被抑制臂误伤。
#[test]
fn genuine_flex_container_applies_safe_center_fallback() {
    let (doc, styles, container, item) = build_006(false);
    let result = compute(&doc, &styles);
    let c = find_box(&result.root, container).expect("container 布局盒存在");
    let i = find_box(&result.root, item).expect("item 布局盒存在");
    let dy = i.y - c.y;
    assert!(
        dy.abs() <= 0.5,
        "真 flex: safe center 溢出回退 start → dy=0；dy={dy}（抑制臂误伤时 end 对齐 → dy=−10）"
    );
}
