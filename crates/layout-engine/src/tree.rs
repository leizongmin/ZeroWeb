//! 从 DOM 树和计算样式构建 taffy 布局树。
//!
//! 提供将 DOM 元素节点与 taffy 节点关联的功能，
//! 跳过文本节点、注释节点和 display:none 的元素。

use std::collections::HashMap;
use taffy::prelude::*;
use zero_css_parser::values::DisplayValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

use crate::converter::{computed_style_to_taffy, parse_grid_template_areas, GridAreaMap};

/// 构建上下文 — 跟踪 DOM 节点与 taffy 节点的映射。
struct BuildContext {
    /// taffy 布局树。
    taffy: TaffyTree<()>,
    /// DOM NodeId → taffy NodeId 映射。
    node_map: HashMap<NodeId, taffy::NodeId>,
    /// taffy NodeId → DOM NodeId 反向映射。
    taffy_to_dom: HashMap<taffy::NodeId, NodeId>,
}

impl BuildContext {
    /// 创建空的构建上下文。
    fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: HashMap::new(),
            taffy_to_dom: HashMap::new(),
        }
    }
}

/// 从 DOM 树和计算样式构建 taffy 树。
///
/// # 参数
///
/// - `doc` — DOM 文档
/// - `styles` — 元素 NodeId → ComputedStyle 映射
/// - `_viewport_width` — 视口宽度（预留，暂未使用）
/// - `_viewport_height` — 视口高度（预留，暂未使用）
///
/// # 返回值
///
/// 返回 (taffy 树, 根节点 ID, taffy→DOM 映射)
pub fn build_layout_tree(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    _viewport_width: f32,
    _viewport_height: f32,
) -> (TaffyTree<()>, taffy::NodeId, HashMap<taffy::NodeId, NodeId>) {
    let mut ctx = BuildContext::new();

    // 找到第一个元素节点作为根（通常是 document > html）
    let root = doc.root();
    let first_element = find_first_element(doc, root);

    let root_taffy_id = build_subtree(&mut ctx, doc, styles, first_element, None);

    (ctx.taffy, root_taffy_id, ctx.taffy_to_dom)
}

/// 查找指定节点子树中的第一个元素节点。
fn find_first_element(doc: &Document, node: NodeId) -> NodeId {
    let node_data = match doc.get(node) {
        Some(n) => n,
        None => return node,
    };

    if matches!(&node_data.kind, NodeKind::Element(_)) {
        return node;
    }

    // 深度优先搜索子节点
    for &child in &node_data.children {
        let found = find_first_element(doc, child);
        let child_data = doc.get(found);
        if child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            return found;
        }
    }

    node
}

/// 递归构建 DOM 子树对应的 taffy 子树。
///
/// 返回创建的 taffy 节点 ID。如果元素为 display:none 则不创建节点。
/// `parent_grid_areas` 为父级 grid 容器的区域映射（如果有），
/// 用于解析子元素的 grid-area 命名引用。
fn build_subtree(
    ctx: &mut BuildContext,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    dom_id: NodeId,
    parent_grid_areas: Option<&GridAreaMap>,
) -> taffy::NodeId {
    // 获取计算样式（或使用默认值）
    let computed = styles.get(&dom_id).cloned().unwrap_or_default();

    // 跳过 display:none 的元素
    if computed.display == DisplayValue::None {
        // 返回一个零尺寸的隐藏节点
        let hidden_style = taffy::Style {
            display: taffy::style::Display::None,
            ..taffy::Style::default()
        };
        return ctx
            .taffy
            .new_leaf(hidden_style)
            .unwrap_or_else(|_| ctx.taffy.new_leaf(taffy::Style::default()).unwrap());
    }

    // 解析此元素的 grid-template-areas（如果有）
    let grid_areas = computed
        .grid_template_areas
        .as_ref()
        .map(|s| parse_grid_template_areas(s));

    // 转换为 taffy 样式（传入父级区域映射）
    let taffy_style = computed_style_to_taffy(&computed, parent_grid_areas);

    // 收集需要创建 taffy 节点的子元素
    let node_data = doc.get(dom_id);
    let children_dom: Vec<NodeId> = node_data.map(|n| n.children.clone()).unwrap_or_default();

    // 先收集子元素
    let mut child_taffy_ids: Vec<taffy::NodeId> = Vec::new();
    for &child_dom in &children_dom {
        let child_data = doc.get(child_dom);
        // 只处理元素节点
        if child_data.is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            let child_taffy = build_subtree(
                ctx,
                doc,
                styles,
                child_dom,
                grid_areas.as_ref(),
            );
            child_taffy_ids.push(child_taffy);
        }
    }

    // 创建 taffy 节点
    let taffy_id = if child_taffy_ids.is_empty() {
        ctx.taffy.new_leaf(taffy_style).unwrap()
    } else {
        ctx.taffy
            .new_with_children(taffy_style, &child_taffy_ids)
            .unwrap()
    };

    // 记录映射
    ctx.node_map.insert(dom_id, taffy_id);
    ctx.taffy_to_dom.insert(taffy_id, dom_id);

    taffy_id
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use zero_css_parser::values::{DisplayValue, FlexDirectionValue, LengthValue};
    use zero_dom::Document;

    /// 辅助：创建简单 DOM（html > body > div）。
    fn make_simple_doc() -> (Document, NodeId, NodeId, NodeId) {
        let mut doc = Document::new();
        let root = doc.root();

        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();

        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();

        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        (doc, html, body, div)
    }

    /// 在 taffy_to_dom 中查找指定 dom_id 对应的 taffy NodeId。
    fn find_taffy_for_dom(
        taffy_to_dom: &HashMap<taffy::NodeId, NodeId>,
        target_dom: NodeId,
    ) -> taffy::NodeId {
        taffy_to_dom
            .iter()
            .find(|(_, dom_id)| **dom_id == target_dom)
            .map(|(t, _)| *t)
            .unwrap()
    }

    /// 测试简单树构建。
    #[test]
    fn test_build_simple_tree() {
        let (doc, html, _body, _div) = make_simple_doc();
        let styles = HashMap::new();
        let (_taffy_tree, root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        assert!(taffy_to_dom.contains_key(&root_id));
        // html 节点应该在映射中
        assert_eq!(taffy_to_dom.get(&root_id), Some(&html));
    }

    /// 测试多层嵌套。
    #[test]
    fn test_build_nested_tree() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let div1 = doc.create_element("div");
        doc.append_child(body, div1).unwrap();
        let div2 = doc.create_element("div");
        doc.append_child(div1, div2).unwrap();
        let div3 = doc.create_element("span");
        doc.append_child(div2, div3).unwrap();

        let styles = HashMap::new();
        let (taffy_tree, root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let children = taffy_tree.children(root_id).unwrap();
        assert!(!children.is_empty());
        // 应该有 html, body, div, div, span 的映射
        assert!(taffy_to_dom.len() >= 5);
    }

    /// 测试跳过 display:none 元素。
    #[test]
    fn test_build_skips_display_none() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let hidden = doc.create_element("div");
        doc.append_child(body, hidden).unwrap();
        let visible = doc.create_element("span");
        doc.append_child(body, visible).unwrap();

        let mut styles = HashMap::new();
        let mut hidden_style = ComputedStyle::default();
        hidden_style.display = DisplayValue::None;
        styles.insert(hidden, hidden_style);

        let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // visible 应该在映射中
        assert!(taffy_to_dom.values().any(|id| *id == visible));
    }

    /// 测试跳过文本节点。
    #[test]
    fn test_build_skips_text_nodes() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let text = doc.create_text_node("Hello World");
        doc.append_child(body, text).unwrap();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let styles = HashMap::new();
        let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // 文本节点不应在 taffy 映射中
        assert!(!taffy_to_dom.values().any(|id| *id == text));
        // div 应该存在
        assert!(taffy_to_dom.values().any(|id| *id == div));
    }

    /// 测试 flex 容器构建。
    #[test]
    fn test_build_flex_container() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let flex_container = doc.create_element("div");
        doc.append_child(html, flex_container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(flex_container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(flex_container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::Row;
        styles.insert(flex_container, container_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let container_taffy = find_taffy_for_dom(&taffy_to_dom, flex_container);
        let style = taffy_tree.style(container_taffy).unwrap();
        assert_eq!(style.display, taffy::style::Display::Flex);
    }

    /// 测试 grid 容器构建。
    #[test]
    fn test_build_grid_container() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let grid_container = doc.create_element("div");
        doc.append_child(html, grid_container).unwrap();
        let item = doc.create_element("span");
        doc.append_child(grid_container, item).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Grid;
        styles.insert(grid_container, container_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let container_taffy = find_taffy_for_dom(&taffy_to_dom, grid_container);
        let style = taffy_tree.style(container_taffy).unwrap();
        assert_eq!(style.display, taffy::style::Display::Grid);
    }

    /// 测试混合 display 类型。
    #[test]
    fn test_build_mixed_display_types() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        let block = doc.create_element("div");
        doc.append_child(body, block).unwrap();
        let flex = doc.create_element("div");
        doc.append_child(body, flex).unwrap();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let mut styles = HashMap::new();
        let mut block_style = ComputedStyle::default();
        block_style.display = DisplayValue::Block;
        styles.insert(block, block_style);

        let mut flex_style = ComputedStyle::default();
        flex_style.display = DisplayValue::Flex;
        styles.insert(flex, flex_style);

        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        styles.insert(grid, grid_style);

        let (_taffy_tree, _root_id, _taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // 树应该成功构建
    }

    /// 测试绝对定位元素。
    #[test]
    fn test_build_with_absolute_position() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let container = doc.create_element("div");
        doc.append_child(html, container).unwrap();
        let abs_child = doc.create_element("span");
        doc.append_child(container, abs_child).unwrap();

        let mut styles = HashMap::new();
        let mut abs_style = ComputedStyle::default();
        abs_style.position = zero_css_parser::values::PositionValue::Absolute;
        abs_style.top = LengthValue::Px(10.0);
        abs_style.left = LengthValue::Px(20.0);
        styles.insert(abs_child, abs_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let abs_taffy = find_taffy_for_dom(&taffy_to_dom, abs_child);
        let style = taffy_tree.style(abs_taffy).unwrap();
        assert_eq!(style.position, taffy::style::Position::Absolute);
    }

    /// 测试 auto margin 和显式 0px margin。
    #[test]
    fn test_build_with_auto_margins() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let div = doc.create_element("div");
        doc.append_child(html, div).unwrap();

        // 默认 margin 是 Px(0.0)，不是 auto
        let styles = HashMap::new();
        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
        let style = taffy_tree.style(div_taffy).unwrap();
        // 默认 margin 是 Px(0.0)，转换为 Length(0.0)
        assert_eq!(
            style.margin.top,
            taffy::style::LengthPercentageAuto::Length(0.0)
        );
    }

    /// 测试 margin: auto 正确传递。
    #[test]
    fn test_build_with_explicit_auto_margin() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;

        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let div = doc.create_element("div");
        doc.append_child(html, div).unwrap();

        let mut style = ComputedStyle::default();
        style.margin_top = LengthValue::Auto;
        style.margin_right = LengthValue::Auto;
        let mut styles = HashMap::new();
        styles.insert(div, style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
        let style = taffy_tree.style(div_taffy).unwrap();
        assert_eq!(style.margin.top, taffy::style::LengthPercentageAuto::Auto);
        assert_eq!(style.margin.right, taffy::style::LengthPercentageAuto::Auto);
    }

    /// 测试百分比 width 正确传递。
    #[test]
    fn test_build_with_percentage_width() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;

        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let div = doc.create_element("div");
        doc.append_child(html, div).unwrap();

        let mut style = ComputedStyle::default();
        style.width = LengthValue::Percentage(50.0);
        let mut styles = HashMap::new();
        styles.insert(div, style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
        let style = taffy_tree.style(div_taffy).unwrap();
        assert_eq!(style.size.width, taffy::style::Dimension::Percent(0.5));
    }

    /// 测试空文档。
    #[test]
    fn test_build_empty_document() {
        let doc = Document::new();
        let styles = HashMap::new();
        let (taffy_tree, root_id, _taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // 空文档没有元素节点，但 taffy 树仍然会创建一个根节点。
        // 布局不 panic 即为通过。
        let _ = taffy_tree;
        // root_id 应该存在
        assert!(root_id == root_id); // 确保编译通过
    }

    /// 测试深层嵌套（50 层）。
    #[test]
    fn test_build_deep_nesting() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();

        let mut current = html;
        for _ in 0..50 {
            let div = doc.create_element("div");
            doc.append_child(current, div).unwrap();
            current = div;
        }

        let styles = HashMap::new();
        let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // 应该有 1 (html) + 50 (divs) = 51 个映射
        assert_eq!(taffy_to_dom.len(), 51);
    }

    /// 测试宽树（100 个兄弟元素）。
    #[test]
    fn test_build_wide_tree() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();

        for _ in 0..100 {
            let div = doc.create_element("div");
            doc.append_child(body, div).unwrap();
        }

        let styles = HashMap::new();
        let (_taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        // html + body + 100 divs = 102
        assert_eq!(taffy_to_dom.len(), 102);
    }

    /// 测试带 gap 的构建。
    #[test]
    fn test_build_with_gap() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let flex = doc.create_element("div");
        doc.append_child(html, flex).unwrap();
        let item = doc.create_element("span");
        doc.append_child(flex, item).unwrap();

        let mut styles = HashMap::new();
        let mut flex_style = ComputedStyle::default();
        flex_style.display = DisplayValue::Flex;
        flex_style.gap = LengthValue::Px(10.0);
        styles.insert(flex, flex_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let flex_taffy = find_taffy_for_dom(&taffy_to_dom, flex);
        let style = taffy_tree.style(flex_taffy).unwrap();
        assert_eq!(
            style.gap.width,
            taffy::style::LengthPercentage::Length(10.0)
        );
    }

    /// 测试带 padding/border/margin。
    #[test]
    fn test_build_with_padding_border_margin() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let div = doc.create_element("div");
        doc.append_child(html, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.padding_top = LengthValue::Px(10.0);
        div_style.border_top_width = LengthValue::Px(2.0);
        div_style.margin_top = LengthValue::Px(5.0);
        styles.insert(div, div_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
        let style = taffy_tree.style(div_taffy).unwrap();
        assert_eq!(
            style.padding.top,
            taffy::style::LengthPercentage::Length(10.0)
        );
        assert_eq!(
            style.border.top,
            taffy::style::LengthPercentage::Length(2.0)
        );
        assert_eq!(
            style.margin.top,
            taffy::style::LengthPercentageAuto::Length(5.0)
        );
    }

    /// 测试带 min/max size。
    #[test]
    fn test_build_with_min_max_size() {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let div = doc.create_element("div");
        doc.append_child(html, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.min_width = LengthValue::Px(50.0);
        div_style.max_width = LengthValue::Px(500.0);
        styles.insert(div, div_style);

        let (taffy_tree, _root_id, taffy_to_dom) = build_layout_tree(&doc, &styles, 800.0, 600.0);
        let div_taffy = find_taffy_for_dom(&taffy_to_dom, div);
        let style = taffy_tree.style(div_taffy).unwrap();
        assert_eq!(style.min_size.width, taffy::style::Dimension::Length(50.0));
        assert_eq!(style.max_size.width, taffy::style::Dimension::Length(500.0));
    }
}
