//! R5387：单行 flex 容器 auto cross size 的 definite 传导（css-flexbox-1 §9.8）。
//!
//! flexbox-definite-sizes-003/004 勘探链：taffy 输出正确（嵌套 flex 的
//! max-height:100% 链 block=100），taffy 后 R1743 回填（shift_siblings_after_ifc_grow）
//! 把 item 撑回子底 9999，12.6 clamp_percentage_max_height 收紧时 cb 解析断链
//!（innerFlex height:auto → my_definite_content_height None）放过 → 终态 9999。
//! 修复两臂：
//! 1. cb_for_children 补「单行 flex 容器 + height:auto + 布局 content 高为正 →
//!    布局高作 definite cb」（§9.8 Cross-Axis Determination：definite main size 的
//!    单行 flex 容器 cross size definite）；
//! 2. clamp 收缩负 delta 退账不得低于剩余 in-flow 子 extent——flex 父高 = taffy
//!    原值（未被 R1743 撑大），裸退账把 innerFlex 100 打到 0。
//!
//! 诊断链记档：docs/goal/rendering-compat/evidence/r5387-flex-definite-cross-2026-11-02.md
//!
//! https://drafts.csswg.org/css-flexbox-1/#algo-cross-container

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_layout_engine::{LayoutEngine, types::LayoutResult};
use zero_style_system::ComputedStyle;

/// 嵌套 flex + max-height:100% 收紧链驱动案（flexbox-definite-sizes-003/004 形）。
///
/// `inner_max_height`：003 形 None（max-height 在 outer）、004 形 Some(100px)。
fn build(
    inner_max_height: Option<LengthValue>,
) -> (
    zero_dom::Document,
    HashMap<NodeId, ComputedStyle>,
    NodeId,
    NodeId,
    NodeId,
) {
    use zero_dom::Document;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(outer, inner).unwrap();
    let block = doc.create_element("div");
    doc.append_child(inner, block).unwrap();
    let leaf = doc.create_element("div");
    doc.append_child(block, leaf).unwrap();

    let flex = |max_height: Option<LengthValue>| ComputedStyle {
        display: DisplayValue::Flex,
        width: LengthValue::Px(100.0),
        max_height: max_height.unwrap_or(LengthValue::Auto),
        ..Default::default()
    };
    let mut styles = HashMap::new();
    // outer：max-height 定尺链源头（003 = 100px，004 = auto）
    styles.insert(
        outer,
        flex(if inner_max_height.is_none() {
            Some(LengthValue::Px(100.0))
        } else {
            None
        }),
    );
    // inner：单行 flex 容器（flex-wrap 默认 Nowrap），definite main size（width:100px），
    // cross（height）auto——definite 传导臂的目标盒
    styles.insert(inner, flex(inner_max_height));
    // block：flex item + max-height:100%（百分比相对 inner 的 definite cross size 解析）
    styles.insert(
        block,
        ComputedStyle {
            display: DisplayValue::Block,
            max_height: LengthValue::Percentage(100.0),
            ..Default::default()
        },
    );
    // leaf：超高内容（R1743 回填把 block 撑回其底的源头）
    styles.insert(
        leaf,
        ComputedStyle {
            display: DisplayValue::Block,
            height: LengthValue::Px(9999.0),
            ..Default::default()
        },
    );
    (doc, styles, outer, inner, block)
}

/// 在布局树中递归查找节点对应 LayoutBox。
fn find_box(tree: &zero_layout_engine::LayoutBox, id: NodeId) -> Option<&zero_layout_engine::LayoutBox> {
    if tree.node_id == Some(id) {
        return Some(tree);
    }
    tree.children.iter().find_map(|c| find_box(c, id))
}

fn compute(doc: &zero_dom::Document, styles: &HashMap<NodeId, ComputedStyle>) -> LayoutResult {
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(doc, styles)
}

/// 003 形：max-height:100px 在 outer flex。R1743 回填把 block 撑到子底 9999 后，
/// 12.6 须经 inner 的 definite cross（布局高 100）收紧回 100；inner 自身不被
/// 负 delta 退账打到 0。
#[test]
fn outer_maxh_chain_clamps_block_to_inner_definite_cross() {
    let (doc, styles, outer, inner, block) = build(None);
    let result = compute(&doc, &styles);
    let block_box = find_box(&result.root, block).expect("block 布局盒存在");
    let inner_box = find_box(&result.root, inner).expect("inner 布局盒存在");
    let outer_box = find_box(&result.root, outer).expect("outer 布局盒存在");
    assert_eq!(
        (block_box.height, block_box.content_height),
        (100.0, 100.0),
        "block 的 max-height:100% 相对 inner definite cross（100）收紧（修复前 9999）"
    );
    assert_eq!(
        inner_box.height, 100.0,
        "inner 不被 clamp 收缩负 delta 退账打到 0（修复前裸退账→0）"
    );
    assert_eq!(outer_box.height, 100.0, "outer max-height:100px 定尺链不变");
}

/// 004 形：max-height:100px 移到 inner flex 自身——definite cross 同样传导，
/// block 收紧回 100。
#[test]
fn inner_maxh_chain_clamps_block_to_inner_definite_cross() {
    let (doc, styles, _outer, inner, block) = build(Some(LengthValue::Px(100.0)));
    let result = compute(&doc, &styles);
    let block_box = find_box(&result.root, block).expect("block 布局盒存在");
    let inner_box = find_box(&result.root, inner).expect("inner 布局盒存在");
    assert_eq!(
        (block_box.height, block_box.content_height),
        (100.0, 100.0),
        "block 的 max-height:100% 相对 inner definite cross（100）收紧（修复前 9999）"
    );
    assert_eq!(inner_box.height, 100.0, "inner max-height:100px 定尺不变");
}

/// 非 flex 守卫：普通 block（height:auto）父的百分比 max-height 不解析（CSS2
/// §10.5/10.7：auto 高包含块百分比不解析）——cb 传导臂不得把 definite 语义误扩
/// 到普通 block 谱系（min-height-percentage 族锚定 auto 不解析语义）。
#[test]
fn plain_block_auto_height_parent_still_indefinite() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let block = doc.create_element("div");
    doc.append_child(parent, block).unwrap();
    let leaf = doc.create_element("div");
    doc.append_child(block, leaf).unwrap();

    let mut styles = HashMap::new();
    styles.insert(
        parent,
        ComputedStyle {
            display: DisplayValue::Block,
            ..Default::default()
        },
    );
    styles.insert(
        block,
        ComputedStyle {
            display: DisplayValue::Block,
            max_height: LengthValue::Percentage(100.0),
            ..Default::default()
        },
    );
    styles.insert(
        leaf,
        ComputedStyle {
            display: DisplayValue::Block,
            height: LengthValue::Px(9999.0),
            ..Default::default()
        },
    );

    let result = compute(&doc, &styles);
    let block_box = find_box(&result.root, block).expect("block 布局盒存在");
    assert_eq!(
        block_box.height, 9999.0,
        "普通 block 父（auto 高）不构成 definite cb：百分比 max-height 不解析，内容高保持"
    );
}
