//! CSS 选择器匹配。
//!
//! 实现选择器与 DOM 元素的匹配逻辑，从右到左遍历选择器部分，
//! 检查标签名、ID、类、属性和伪类。

/// 匹配声明结果类型：(属性名, 属性值, 是否important, 特异性, 层索引)
type MatchingDecl = (String, String, bool, (u32, u32, u32), Option<usize>);

use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, PseudoClassSelector,
    Selector, SubclassSelector, TypeSelector,
};
use zero_dom::{Document, NodeId, NodeKind};

/// 检查选择器是否匹配指定 DOM 元素。
///
/// 从右到左遍历选择器的复合选择器链，逐个验证匹配条件。
pub fn matches_selector(doc: &Document, element: NodeId, selector: &Selector) -> bool {
    let parts = &selector.complex.parts;
    if parts.is_empty() {
        return false;
    }

    // 从最后一个复合选择器开始（最右边是目标元素）
    let last_idx = parts.len() - 1;
    let (compound, _) = &parts[last_idx];

    // 首先检查目标元素是否匹配最后一个复合选择器
    if !matches_compound(doc, element, compound) {
        return false;
    }

    // 如果只有一个复合选择器，匹配成功
    if parts.len() == 1 {
        return true;
    }

    // 递归检查前面的复合选择器
    matches_selector_recursive(doc, element, parts, last_idx)
}

/// 递归检查选择器链的其余部分。
///
/// 从目标元素开始，沿 DOM 树向上查找匹配的祖先或兄弟。
fn matches_selector_recursive(
    doc: &Document,
    current: NodeId,
    parts: &[(CompoundSelector, Option<Combinator>)],
    part_idx: usize,
) -> bool {
    if part_idx == 0 {
        return true;
    }

    let (prev_compound, combinator) = &parts[part_idx - 1];

    match combinator {
        Some(Combinator::Descendant) => {
            // 后代组合器：在任何祖先中查找匹配
            let mut ancestor = doc.parent_node(current);
            while let Some(ancestor_id) = ancestor {
                if matches_compound(doc, ancestor_id, prev_compound)
                    && matches_selector_recursive(doc, ancestor_id, parts, part_idx - 1)
                {
                    return true;
                }
                ancestor = doc.parent_node(ancestor_id);
            }
            false
        }
        Some(Combinator::Child) => {
            // 子组合器：只在直接父元素中查找
            if let Some(parent) = doc.parent_node(current)
                && matches_compound(doc, parent, prev_compound)
            {
                return matches_selector_recursive(doc, parent, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::NextSibling) => {
            // 相邻兄弟组合器：只检查前一个兄弟
            if let Some(prev) = doc.previous_sibling(current)
                && is_element(doc, prev)
                && matches_compound(doc, prev, prev_compound)
            {
                return matches_selector_recursive(doc, prev, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::SubsequentSibling) => {
            // 通用兄弟组合器：检查所有前面的兄弟
            let mut sibling = doc.previous_sibling(current);
            while let Some(sibling_id) = sibling {
                if is_element(doc, sibling_id)
                    && matches_compound(doc, sibling_id, prev_compound)
                    && matches_selector_recursive(doc, sibling_id, parts, part_idx - 1)
                {
                    return true;
                }
                sibling = doc.previous_sibling(sibling_id);
            }
            false
        }
        None => {
            // 没有组合器（不应该发生），尝试继续
            matches_selector_recursive(doc, current, parts, part_idx - 1)
        }
    }
}

/// 检查元素是否匹配复合选择器。
fn matches_compound(doc: &Document, element: NodeId, compound: &CompoundSelector) -> bool {
    // 检查类型选择器
    if let Some(type_sel) = &compound.type_selector
        && !matches_type(doc, element, type_sel)
    {
        return false;
    }

    // 检查所有子类选择器
    for sub in &compound.subclass_selectors {
        if !matches_subclass(doc, element, sub) {
            return false;
        }
    }

    true
}

/// 检查类型选择器是否匹配。
fn matches_type(doc: &Document, element: NodeId, type_sel: &TypeSelector) -> bool {
    let node = match doc.get(element) {
        Some(n) => n,
        None => return false,
    };

    match &node.kind {
        NodeKind::Element(elem) => match type_sel {
            TypeSelector::Universal => true,
            TypeSelector::Tag(tag) => elem.local_name().eq_ignore_ascii_case(tag),
        },
        _ => false,
    }
}

/// 检查子类选择器是否匹配。
fn matches_subclass(doc: &Document, element: NodeId, sub: &SubclassSelector) -> bool {
    match sub {
        SubclassSelector::Id(id) => matches_id(doc, element, id),
        SubclassSelector::Class(cls) => matches_class(doc, element, cls),
        SubclassSelector::Attribute(attr) => matches_attribute(doc, element, attr),
        SubclassSelector::PseudoClass(pc) => matches_pseudo_class(doc, element, pc),
        SubclassSelector::PseudoElement(_) => {
            // 伪元素不匹配 DOM 元素
            false
        }
    }
}

/// 检查 ID 选择器是否匹配。
fn matches_id(doc: &Document, element: NodeId, id: &str) -> bool {
    doc.get_attribute(element, "id").is_some_and(|v| v == id)
}

/// 检查类选择器是否匹配。
fn matches_class(doc: &Document, element: NodeId, cls: &str) -> bool {
    let node = match doc.get(element) {
        Some(n) => n,
        None => return false,
    };

    match &node.kind {
        NodeKind::Element(elem) => elem.class_list.iter().any(|c| c == cls),
        _ => false,
    }
}

/// 检查属性选择器是否匹配。
fn matches_attribute(doc: &Document, element: NodeId, attr: &AttributeSelector) -> bool {
    let value = match doc.get_attribute(element, &attr.name) {
        Some(v) => v,
        None => return false,
    };

    match &attr.matcher {
        AttributeMatcher::Exists => true,
        AttributeMatcher::Exact(v) => &value == v,
        AttributeMatcher::Includes(v) => value.split_whitespace().any(|part| part == v),
        AttributeMatcher::DashMatch(v) => value == *v || value.starts_with(&format!("{v}-")),
        AttributeMatcher::Prefix(v) => value.starts_with(v),
        AttributeMatcher::Suffix(v) => value.ends_with(v),
        AttributeMatcher::Substring(v) => value.contains(v),
    }
}

/// 检查伪类选择器是否匹配。
///
/// 支持有限集：`:first-child`, `:last-child`, `:root`, `:empty`, `:nth-child()`。
fn matches_pseudo_class(doc: &Document, element: NodeId, pc: &PseudoClassSelector) -> bool {
    match pc {
        PseudoClassSelector::Simple(name) => match name.as_str() {
            "first-child" => is_first_child(doc, element),
            "last-child" => is_last_child(doc, element),
            "root" => is_root_element(doc, element),
            "empty" => is_empty_element(doc, element),
            "only-child" => is_first_child(doc, element) && is_last_child(doc, element),
            "first-of-type" => is_first_of_type(doc, element),
            "last-of-type" => is_last_of_type(doc, element),
            "only-of-type" => is_first_of_type(doc, element) && is_last_of_type(doc, element),
            _ => false, // 不支持的伪类
        },
        PseudoClassSelector::Not(selectors) => {
            // :not() 匹配不满足任一选择器的元素
            !selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::Is(selectors) => {
            // :is() 匹配满足任一选择器的元素
            selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::Where(selectors) => {
            // :where() 匹配逻辑同 :is()
            selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::Has(selectors) => {
            // :has() 匹配拥有满足条件的后代/子元素的元素
            selectors.iter().any(|s| matches_has_inner(doc, element, s))
        }
        PseudoClassSelector::NthChild(pattern) => matches_nth_child(doc, element, pattern),
        PseudoClassSelector::NthLastChild(pattern) => matches_nth_last_child(doc, element, pattern),
        PseudoClassSelector::NthOfType(pattern) => matches_nth_of_type(doc, element, pattern),
        PseudoClassSelector::NthLastOfType(pattern) => matches_nth_last_of_type(doc, element, pattern),
        PseudoClassSelector::Lang(_) => false, // :lang() 需要语言上下文，暂不支持
    }
}

/// 检查元素是否为第一个子元素。
fn is_first_child(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);
    // 找到第一个元素子节点
    for &child in &children {
        if is_element(doc, child) {
            return child == element;
        }
    }
    false
}

/// 检查元素是否为最后一个子元素。
fn is_last_child(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);
    // 从后往前找到最后一个元素子节点
    for &child in children.iter().rev() {
        if is_element(doc, child) {
            return child == element;
        }
    }
    false
}

/// 检查元素是否为文档根元素（html）。
fn is_root_element(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    // 父节点是文档根节点
    if let Some(node) = doc.get(parent) {
        matches!(node.kind, NodeKind::Document(_))
    } else {
        false
    }
}

/// 检查元素是否为空（没有子元素或文本节点）。
fn is_empty_element(doc: &Document, element: NodeId) -> bool {
    let children = doc.child_nodes(element);
    if children.is_empty() {
        return true;
    }
    // 检查是否只有空文本节点
    for &child in &children {
        if let Some(node) = doc.get(child) {
            match &node.kind {
                NodeKind::Element(_) => return false,
                NodeKind::Text(data) if !data.content.trim().is_empty() => {
                    return false;
                }
                _ => {}
            }
        }
    }
    true
}

/// 检查元素是否匹配 nth-child 模式。
fn matches_nth_child(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 计算元素在兄弟中的位置（1-indexed，只计算元素节点）
    let mut index = 0;
    for &child in &children {
        if is_element(doc, child) {
            index += 1;
            if child == element {
                return matches_nth_pattern(index, pattern);
            }
        }
    }
    false
}

/// 检查元素是否匹配 nth-last-child 模式（从末尾计数）。
fn matches_nth_last_child(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 收集所有元素子节点
    let element_children: Vec<NodeId> =
        children.iter().copied().filter(|&c| is_element(doc, c)).collect();

    // 从末尾计数（1-indexed）
    for (i, &child) in element_children.iter().rev().enumerate() {
        if child == element {
            return matches_nth_pattern((i + 1) as i32, pattern);
        }
    }
    false
}

/// 获取元素的标签名（小写）。
fn element_tag_name(doc: &Document, element: NodeId) -> Option<String> {
    let node = doc.get(element)?;
    match &node.kind {
        zero_dom::NodeKind::Element(e) => Some(e.local_name().to_ascii_lowercase()),
        _ => None,
    }
}

/// 检查元素是否是同类型中的第一个。
fn is_first_of_type(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 找到第一个同类型的兄弟
    for &child in &children {
        if is_element(doc, child)
            && let Some(child_tag) = element_tag_name(doc, child)
            && child_tag == tag
        {
            return child == element;
        }
    }
    false
}

/// 检查元素是否是同类型中的最后一个。
fn is_last_of_type(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 找到最后一个同类型的兄弟
    for &child in children.iter().rev() {
        if is_element(doc, child)
            && let Some(child_tag) = element_tag_name(doc, child)
            && child_tag == tag
        {
            return child == element;
        }
    }
    false
}

/// 检查元素是否匹配 nth-of-type 模式。
fn matches_nth_of_type(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 计算同类型兄弟中的位置（1-indexed）
    let mut index = 0;
    for &child in &children {
        if is_element(doc, child)
            && let Some(child_tag) = element_tag_name(doc, child)
            && child_tag == tag
        {
            index += 1;
            if child == element {
                return matches_nth_pattern(index, pattern);
            }
        }
    }
    false
}

/// 检查元素是否匹配 nth-last-of-type 模式（从末尾计数）。
fn matches_nth_last_of_type(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 收集同类型的元素子节点
    let same_type: Vec<NodeId> = children
        .iter()
        .copied()
        .filter(|&c| {
            is_element(doc, c) && element_tag_name(doc, c).as_deref() == Some(tag.as_str())
        })
        .collect();

    // 从末尾计数（1-indexed）
    for (i, &child) in same_type.iter().rev().enumerate() {
        if child == element {
            return matches_nth_pattern((i + 1) as i32, pattern);
        }
    }
    false
}

/// 检查 `:has()` 内部选择器是否匹配当前元素的某个后代或子元素。
///
/// 选择器的最后一个复合选择器是匹配目标。我们遍历元素的后代，
/// 找到匹配最后一个复合选择器的节点，然后验证组合器链。
fn matches_has_inner(doc: &Document, element: NodeId, selector: &Selector) -> bool {
    let parts = &selector.complex.parts;
    if parts.is_empty() {
        return false;
    }

    let last_idx = parts.len() - 1;
    let (last_compound, _) = &parts[last_idx];

    // 确定搜索范围：如果倒数第二个组合器是 Child，只搜索直接子元素
    let search_direct_children_only = if parts.len() >= 2 {
        let (_, combinator) = &parts[last_idx - 1];
        matches!(combinator, Some(Combinator::Child))
    } else {
        false
    };

    let mut search_candidates = Vec::new();

    if search_direct_children_only {
        // 只检查直接子元素
        for &child in &doc.child_nodes(element) {
            if is_element(doc, child) {
                search_candidates.push(child);
            }
        }
    } else {
        // 递归收集所有后代元素
        collect_descendants(doc, element, &mut search_candidates);
    }

    // 检查每个候选元素是否匹配完整选择器
    for &candidate in &search_candidates {
        if matches_compound(doc, candidate, last_compound)
            && matches_has_selector_chain(doc, candidate, parts, last_idx)
        {
            return true;
        }
    }

    false
}

/// 递归收集所有后代元素节点。
fn collect_descendants(doc: &Document, element: NodeId, result: &mut Vec<NodeId>) {
    for &child in &doc.child_nodes(element) {
        if is_element(doc, child) {
            result.push(child);
            collect_descendants(doc, child, result);
        }
    }
}

/// 验证 `:has()` 内部选择器的组合器链。
///
/// `candidate` 是匹配最后一个复合选择器的元素。
fn matches_has_selector_chain(
    doc: &Document,
    candidate: NodeId,
    parts: &[(CompoundSelector, Option<Combinator>)],
    part_idx: usize,
) -> bool {
    if part_idx == 0 {
        return true;
    }

    let (prev_compound, combinator) = &parts[part_idx - 1];

    match combinator {
        Some(Combinator::Descendant) => {
            // 后代：candidate 的某个祖先必须匹配 prev_compound
            let mut ancestor = doc.parent_node(candidate);
            while let Some(ancestor_id) = ancestor {
                if matches_compound(doc, ancestor_id, prev_compound)
                    && matches_has_selector_chain(doc, ancestor_id, parts, part_idx - 1)
                {
                    return true;
                }
                ancestor = doc.parent_node(ancestor_id);
            }
            false
        }
        Some(Combinator::Child) => {
            // 子元素：candidate 的直接父元素必须匹配 prev_compound
            if let Some(parent) = doc.parent_node(candidate)
                && is_element(doc, parent)
                && matches_compound(doc, parent, prev_compound)
            {
                return matches_has_selector_chain(doc, parent, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::NextSibling) => {
            if let Some(prev) = doc.previous_sibling(candidate)
                && is_element(doc, prev)
                && matches_compound(doc, prev, prev_compound)
            {
                return matches_has_selector_chain(doc, prev, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::SubsequentSibling) => {
            let mut sibling = doc.previous_sibling(candidate);
            while let Some(sibling_id) = sibling {
                if is_element(doc, sibling_id)
                    && matches_compound(doc, sibling_id, prev_compound)
                    && matches_has_selector_chain(doc, sibling_id, parts, part_idx - 1)
                {
                    return true;
                }
                sibling = doc.previous_sibling(sibling_id);
            }
            false
        }
        None => matches_has_selector_chain(doc, candidate, parts, part_idx - 1),
    }
}

/// 检查位置是否匹配 an+b 模式。
fn matches_nth_pattern(index: i32, pattern: &zero_css_parser::ast::NthPattern) -> bool {
    let a = pattern.a;
    let b = pattern.b;

    if a == 0 {
        // 只有 b：精确匹配
        index == b
    } else {
        // an+b：检查 (index - b) 是否能被 a 整除且结果 >= 0
        let diff = index - b;
        if a > 0 {
            diff >= 0 && diff % a == 0
        } else {
            diff <= 0 && diff % a == 0
        }
    }
}

/// 检查节点是否为元素节点。
fn is_element(doc: &Document, node: NodeId) -> bool {
    doc.get(node)
        .map(|n| matches!(n.kind, NodeKind::Element(_)))
        .unwrap_or(false)
}

/// 容器查询评估上下文。
///
/// 包含查询所需的容器尺寸信息。
/// 在完整实现中，此上下文由布局引擎提供。
#[derive(Debug, Clone)]
pub struct ContainerContext {
    /// 容器宽度（px）。
    pub container_width: Option<f64>,
    /// 容器高度（px）。
    pub container_height: Option<f64>,
}

impl ContainerContext {
    /// 创建空的容器上下文。
    pub fn new() -> Self {
        Self {
            container_width: None,
            container_height: None,
        }
    }

    /// 创建带尺寸的容器上下文。
    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            container_width: Some(width),
            container_height: Some(height),
        }
    }
}

impl Default for ContainerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 评估 @container 条件。
///
/// 基于 ContainerContext 中的容器尺寸评估容器查询条件。
/// 没有容器上下文时，@container 规则不应用。
fn evaluate_container_condition(
    container_rule: &zero_css_parser::ast::ContainerRule,
    container_ctx: Option<&ContainerContext>,
) -> bool {
    use zero_css_parser::values::parse_length;

    let Some(ctx) = container_ctx else {
        // 无容器上下文，不应用 @container 规则
        return false;
    };

    let condition = &container_rule.condition;
    let size_cond = match condition {
        zero_css_parser::ast::ContainerCondition::Size(s)
        | zero_css_parser::ast::ContainerCondition::InlineSize(s) => s,
    };

    let feature = size_cond.feature.to_ascii_lowercase();
    let value_str = size_cond.value.trim();

    // 解析条件值为像素
    let Some(cond_px) = parse_length(value_str).map(|l| match l {
        zero_css_parser::values::LengthValue::Px(n) => n,
        _ => 0.0,
    }) else {
        return false;
    };

    // 根据特性名称和比较运算符评估条件
    let container_size = match feature.as_str() {
        "min-width" | "min-inline-size" => {
            // min-width: 容器宽度 >= 条件值
            ctx.container_width.map(|w| w >= cond_px)
        }
        "max-width" | "max-inline-size" => {
            // max-width: 容器宽度 <= 条件值
            ctx.container_width.map(|w| w <= cond_px)
        }
        "width" | "inline-size" => {
            // width: 容器宽度 == 条件值（精确匹配极少使用，按相等判断）
            ctx.container_width.map(|w| (w - cond_px).abs() < f64::EPSILON)
        }
        "min-height" | "min-block-size" => {
            ctx.container_height.map(|h| h >= cond_px)
        }
        "max-height" | "max-block-size" => {
            ctx.container_height.map(|h| h <= cond_px)
        }
        "height" | "block-size" => {
            ctx.container_height.map(|h| (h - cond_px).abs() < f64::EPSILON)
        }
        _ => None,
    };

    container_size.unwrap_or(false)
}

/// 评估 @supports 条件。
///
/// 对于属性值测试 `(property: value)`，检查解析器是否能识别该属性和值。
/// 对于 `selector()`，检查解析器是否能解析该选择器。
/// 对于逻辑组合，递归评估子条件。
fn evaluate_supports_condition(condition: &zero_css_parser::ast::SupportsCondition) -> bool {
    use zero_css_parser::ast::SupportsCondition;

    match condition {
        SupportsCondition::Property(property, value) => {
            is_property_supported(property, value)
        }
        SupportsCondition::Selector(selector_text) => {
            // 尝试解析选择器，能解析即为支持
            let css = format!("{selector_text} {{ }}");
            let stylesheet = zero_css_parser::Parser::parse_stylesheet(&css);
            matches!(
                stylesheet.rules.first(),
                Some(zero_css_parser::ast::Rule::Style(_))
            )
        }
        SupportsCondition::And(conditions) => {
            conditions.iter().all(evaluate_supports_condition)
        }
        SupportsCondition::Or(conditions) => {
            conditions.iter().any(evaluate_supports_condition)
        }
        SupportsCondition::Not(inner) => {
            !evaluate_supports_condition(inner)
        }
    }
}

/// 检查 CSS 属性值对是否受支持。
///
/// 已知属性且值能被解析即为"支持"。
fn is_property_supported(property: &str, value: &str) -> bool {
    use zero_css_parser::values::*;

    let lower = property.to_ascii_lowercase();
    let trimmed = value.trim();

    match lower.as_str() {
        // 布尔特性：有值即为支持
        "display" => parse_display(trimmed).is_some(),
        "position" => parse_position(trimmed).is_some(),
        "overflow" | "overflow-x" | "overflow-y" => parse_overflow(trimmed).is_some(),
        "visibility" => parse_visibility(trimmed).is_some(),
        "box-sizing" => parse_box_sizing(trimmed).is_some(),
        "flex-direction" => parse_flex_direction(trimmed).is_some(),
        "flex-wrap" => parse_flex_wrap(trimmed).is_some(),
        "justify-content" | "align-items" | "align-content" | "align-self"
        | "justify-self" => parse_alignment(trimmed).is_some(),
        "font-weight" => parse_font_weight(trimmed).is_some(),
        "font-style" => parse_font_style(trimmed).is_some(),
        "color" | "background-color" | "border-color" | "border-top-color"
        | "border-right-color" | "border-bottom-color" | "border-left-color" => {
            parse_color(trimmed).is_some()
        }
        "width" | "height" | "min-width" | "max-width" | "min-height" | "max-height"
        | "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
        | "padding" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left"
        | "gap" | "top" | "right" | "bottom" | "left"
        | "border-top-width" | "border-right-width" | "border-bottom-width"
        | "border-left-width"
        | "border-top-left-radius" | "border-top-right-radius"
        | "border-bottom-right-radius" | "border-bottom-left-radius" => {
            parse_length(trimmed).is_some()
        }
        "transform" => parse_transform(trimmed).is_some(),
        "background" | "background-image" => parse_gradient(trimmed).is_some() || parse_color(trimmed).is_some(),
        "scroll-snap-type" => parse_scroll_snap_type(trimmed).is_some(),
        "scroll-snap-align" => parse_scroll_snap_align(trimmed).is_some(),
        "scroll-snap-stop" => parse_scroll_snap_stop(trimmed).is_some(),
        "scroll-margin-top" | "scroll-margin-right" | "scroll-margin-bottom" | "scroll-margin-left" => parse_length(trimmed).is_some(),
        "scroll-padding-top" | "scroll-padding-right" | "scroll-padding-bottom" | "scroll-padding-left" => {
            trimmed.eq_ignore_ascii_case("auto") || parse_length(trimmed).is_some()
        }
        "container-type" => parse_container_type(trimmed).is_some(),
        "container-name" => true, // 任何非空字符串都有效
        // 未知属性：默认不支持（安全保守策略）
        _ => false,
    }
}

/// 从样式表中收集匹配指定元素的声明。
///
/// 遍历样式表中所有规则，检查每个选择器是否匹配元素，
/// 返回所有匹配的声明及其特异性。
///
/// 不评估媒体查询条件——使用 [`collect_matching_declarations_with_media`] 替代。
pub fn collect_matching_declarations(
    doc: &Document,
    element: NodeId,
    stylesheets: &[zero_css_parser::Stylesheet],
) -> Vec<MatchingDecl> {
    collect_matching_declarations_with_media(doc, element, stylesheets, None, None)
}

/// 从样式表中收集匹配指定元素的声明（带媒体查询评估）。
///
/// 遍历样式表中所有规则，检查每个选择器是否匹配元素，
/// 并在遇到 `@media` 规则时根据媒体上下文决定是否进入。
///
/// `@layer` 规则会为内部声明分配级联层索引。
pub fn collect_matching_declarations_with_media(
    doc: &Document,
    element: NodeId,
    stylesheets: &[zero_css_parser::Stylesheet],
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();
    let mut layer_counter: usize = 0;

    for stylesheet in stylesheets {
        collect_from_rules(
            doc,
            element,
            &stylesheet.rules,
            &mut results,
            media_ctx,
            container_ctx,
            None,
            &mut layer_counter,
        );
    }

    results
}

/// 递归从规则中收集匹配的声明。
///
/// `current_layer` 为 `None` 表示未分层的声明，`Some(idx)` 表示当前级联层索引。
/// `layer_counter` 在遇到 `@layer` 规则时递增，用于分配层索引。
#[allow(clippy::too_many_arguments)]
fn collect_from_rules(
    doc: &Document,
    element: NodeId,
    rules: &[zero_css_parser::ast::Rule],
    results: &mut Vec<MatchingDecl>,
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
    current_layer: Option<usize>,
    layer_counter: &mut usize,
) {
    for rule in rules {
        match rule {
            zero_css_parser::ast::Rule::Style(style_rule) => {
                // 检查选择器列表中是否有匹配的选择器
                for selector in &style_rule.selectors {
                    if matches_selector(doc, element, selector) {
                        let spec = zero_css_parser::selector::specificity(selector);
                        for decl in &style_rule.declarations {
                            results.push((
                                decl.property.clone(),
                                decl.value.clone(),
                                decl.important,
                                spec,
                                current_layer,
                            ));
                        }
                        break; // 一个选择器匹配就够了
                    }
                }
            }
            zero_css_parser::ast::Rule::At(at_rule) => {
                if let zero_css_parser::ast::AtRuleBody::Block(inner_rules) = &at_rule.body {
                    if at_rule.name.eq_ignore_ascii_case("media") {
                        // @media 规则：需要评估媒体条件
                        if let Some(ctx) = media_ctx
                            && let Some(query) =
                                zero_css_parser::media_query::parse_media_query(&at_rule.prelude)
                            && zero_css_parser::media_query::evaluate_media_query(&query, ctx)
                        {
                            collect_from_rules(
                                doc,
                                element,
                                inner_rules,
                                results,
                                media_ctx,
                                container_ctx,
                                current_layer,
                                layer_counter,
                            );
                        }
                        // 没有 media_ctx 时，@media 规则不应用（安全默认值）
                    } else {
                        // 非 @media 的 AtRule（如 @supports）无条件递归，保持当前层
                        collect_from_rules(
                            doc,
                            element,
                            inner_rules,
                            results,
                            media_ctx,
                            container_ctx,
                            current_layer,
                            layer_counter,
                        );
                    }
                }
            }
            zero_css_parser::ast::Rule::Keyframes(_) => {
                // @keyframes 规则不参与样式匹配，跳过
            }
            zero_css_parser::ast::Rule::Layer(layer_rule) => {
                // @layer 规则：分配层索引并递归
                let layer_idx = *layer_counter;
                *layer_counter += 1;
                collect_from_rules(
                    doc,
                    element,
                    &layer_rule.rules,
                    results,
                    media_ctx,
                    container_ctx,
                    Some(layer_idx),
                    layer_counter,
                );
            }
            zero_css_parser::ast::Rule::Import(_) => {
                // @import 规则不参与样式匹配，跳过（实际导入由引擎处理）
            }
            zero_css_parser::ast::Rule::Supports(supports_rule) => {
                // @supports 规则：评估条件，条件为真时递归进入
                if evaluate_supports_condition(&supports_rule.condition) {
                    collect_from_rules(
                        doc,
                        element,
                        &supports_rule.rules,
                        results,
                        media_ctx,
                        container_ctx,
                        current_layer,
                        layer_counter,
                    );
                }
            }
            zero_css_parser::ast::Rule::Container(container_rule) => {
                // @container 规则：基于 ContainerContext 评估容器条件
                if evaluate_container_condition(container_rule, container_ctx) {
                    collect_from_rules(
                        doc,
                        element,
                        &container_rule.rules,
                        results,
                        media_ctx,
                        container_ctx,
                        current_layer,
                        layer_counter,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{
        AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
        PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
    };
    use zero_dom::Document;

    // ── 辅助函数 ──

    fn make_tag_selector(tag: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag(tag.to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                )],
            },
        }
    }

    fn make_id_selector(id: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                    },
                    None,
                )],
            },
        }
    }

    fn make_class_selector(cls: &str) -> Selector {
        Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                    },
                    None,
                )],
            },
        }
    }

    /// 创建一个简单的测试 DOM：html > body > div#main.container > p.text
    fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
        let mut doc = Document::new();
        let root = doc.root();

        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();

        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();

        let div = doc.create_element("div");
        doc.set_attribute(div, "id", "main");
        doc.set_attribute(div, "class", "container");
        doc.append_child(body, div).unwrap();

        let p = doc.create_element("p");
        doc.set_attribute(p, "class", "text");
        doc.append_child(div, p).unwrap();

        (doc, html, body, div, p)
    }

    #[test]
    fn test_matches_tag_selector() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = make_tag_selector("div");
        assert!(matches_selector(&doc, div, &sel));

        let sel_p = make_tag_selector("p");
        assert!(!matches_selector(&doc, div, &sel_p));
    }

    #[test]
    fn test_matches_id_selector() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = make_id_selector("main");
        assert!(matches_selector(&doc, div, &sel));

        let sel_not_found = make_id_selector("other");
        assert!(!matches_selector(&doc, div, &sel_not_found));
    }

    #[test]
    fn test_matches_class_selector() {
        let (doc, _html, _body, div, p) = make_test_dom();
        let sel = make_class_selector("container");
        assert!(matches_selector(&doc, div, &sel));

        let sel_text = make_class_selector("text");
        assert!(matches_selector(&doc, p, &sel_text));
        assert!(!matches_selector(&doc, div, &sel_text));
    }

    #[test]
    fn test_matches_universal_selector() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel));
    }

    #[test]
    fn test_matches_descendant_combinator() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        // div p
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::Descendant),
                    ),
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    ),
                ],
            },
        };
        assert!(matches_selector(&doc, p, &sel));
    }

    #[test]
    fn test_matches_child_combinator() {
        let (doc, _html, _body, _div, p) = make_test_dom();
        // div > p
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::Child),
                    ),
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    ),
                ],
            },
        };
        assert!(matches_selector(&doc, p, &sel));

        // body > p 不应该匹配（p 是 div 的子元素，不是 body 的直接子元素）
        let sel2 = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("body".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::Child),
                    ),
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    ),
                ],
            },
        };
        assert!(!matches_selector(&doc, p, &sel2));
    }

    #[test]
    fn test_matches_attribute_selector() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        // [id]
        let sel_exists = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "id".to_string(),
                            matcher: AttributeMatcher::Exists,
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel_exists));

        // [id=main]
        let sel_exact = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "id".to_string(),
                            matcher: AttributeMatcher::Exact("main".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel_exact));
    }

    #[test]
    fn test_matches_pseudo_first_child() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("first-child".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        // div 是 body 的第一个子元素
        assert!(matches_selector(&doc, div, &sel));
    }

    #[test]
    fn test_matches_pseudo_root() {
        let (doc, html, _body, _div, _p) = make_test_dom();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("root".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        // html 是文档根元素
        assert!(matches_selector(&doc, html, &sel));
    }

    #[test]
    fn test_matches_pseudo_empty() {
        let mut doc = Document::new();
        let root = doc.root();
        let div = doc.create_element("div");
        doc.append_child(root, div).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("empty".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel));

        // 添加文本节点后不再是 empty
        let text = doc.create_text_node("hello");
        doc.append_child(div, text).unwrap();
        assert!(!matches_selector(&doc, div, &sel));
    }

    #[test]
    fn test_matches_not_pseudo() {
        let (doc, _html, _body, div, p) = make_test_dom();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_id_selector("main")]),
                        )],
                    },
                    None,
                )],
            },
        };
        // div#main 不匹配 :not(#main)
        assert!(!matches_selector(&doc, div, &sel));
        // p 匹配 :not(#main)
        assert!(matches_selector(&doc, p, &sel));
    }

    #[test]
    fn test_matches_is_pseudo() {
        let (doc, _html, _body, div, p) = make_test_dom();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Is(vec![
                                make_tag_selector("div"),
                                make_tag_selector("span"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel));
        assert!(!matches_selector(&doc, p, &sel));
    }

    #[test]
    fn test_collect_matching_declarations() {
        use zero_css_parser::ast::{Declaration, Rule, StyleRule, Stylesheet};

        let (doc, _html, _body, div, _p) = make_test_dom();

        let stylesheets = vec![Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        }];

        let results = collect_matching_declarations(&doc, div, &stylesheets);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "color");
        assert_eq!(results[0].1, "red");
    }

    #[test]
    fn test_next_sibling_combinator() {
        let (mut doc, _html, _body, div, _p) = make_test_dom();
        // 创建 p 的兄弟
        let span = doc.create_element("span");
        doc.append_child(div, span).unwrap();

        // p + span
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::NextSibling),
                    ),
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("span".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    ),
                ],
            },
        };
        assert!(matches_selector(&doc, span, &sel));
    }

    // ── 补充边界条件测试 ──

    /// 测试通用兄弟组合器（~）：div ~ span 匹配 div 后面的 span 兄弟。
    #[test]
    fn test_subsequent_sibling_combinator() {
        let (mut doc, _html, _body, div, _p) = make_test_dom();
        // 在 div 后添加 span
        let body = doc.parent_node(div).unwrap();
        let span1 = doc.create_element("span");
        doc.append_child(body, span1).unwrap();
        let span2 = doc.create_element("span");
        doc.append_child(body, span2).unwrap();

        // div ~ span 应匹配 span1 和 span2
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::SubsequentSibling),
                    ),
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("span".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    ),
                ],
            },
        };
        assert!(
            matches_selector(&doc, span1, &sel),
            "span1 should match div ~ span"
        );
        assert!(
            matches_selector(&doc, span2, &sel),
            "span2 should match div ~ span"
        );
        // div 本身不应匹配
        assert!(
            !matches_selector(&doc, div, &sel),
            "div should not match div ~ span"
        );
    }

    /// 测试 :last-child 伪类。
    #[test]
    fn test_matches_pseudo_last_child() {
        let (mut doc, _html, _body, div, _p) = make_test_dom();
        let body = doc.parent_node(div).unwrap();
        let span = doc.create_element("span");
        doc.append_child(body, span).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("last-child".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, span, &sel),
            "span (last child) should match :last-child"
        );
        assert!(
            !matches_selector(&doc, div, &sel),
            "div (not last child) should not match :last-child"
        );
    }

    /// 测试 :nth-child(2n) 匹配偶数位置。
    #[test]
    fn test_matches_nth_child_even() {
        let mut doc = Document::new();
        let root = doc.root();
        let body = doc.create_element("body");
        doc.append_child(root, body).unwrap();

        let items: Vec<NodeId> = (0..5)
            .map(|_| {
                let li = doc.create_element("li");
                doc.append_child(body, li).unwrap();
                li
            })
            .collect();

        // :nth-child(2n) 匹配第 2、4 个
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthChild(zero_css_parser::ast::NthPattern {
                                a: 2,
                                b: 0,
                            }),
                        )],
                    },
                    None,
                )],
            },
        };

        assert!(
            !matches_selector(&doc, items[0], &sel),
            "1st child should not match 2n"
        );
        assert!(
            matches_selector(&doc, items[1], &sel),
            "2nd child should match 2n"
        );
        assert!(
            !matches_selector(&doc, items[2], &sel),
            "3rd child should not match 2n"
        );
        assert!(
            matches_selector(&doc, items[3], &sel),
            "4th child should match 2n"
        );
    }

    /// 测试 :where() 伪类匹配。
    #[test]
    fn test_matches_where_pseudo() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Where(vec![
                                make_class_selector("container"),
                                make_class_selector("other"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, div, &sel),
            "div.container should match :where(.container, .other)"
        );
    }

    /// 测试属性选择器 DashMatch（lang 属性）。
    #[test]
    fn test_attribute_dash_match() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "lang", "en-US");
        let root = doc.root();
        doc.append_child(root, elem).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "lang".to_string(),
                            matcher: AttributeMatcher::DashMatch("en".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, elem, &sel),
            "lang='en-US' should match [lang|=en]"
        );
    }

    /// 测试属性选择器 Prefix。
    #[test]
    fn test_attribute_prefix_match() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "data-type", "button-primary");
        let root = doc.root();
        doc.append_child(root, elem).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "data-type".to_string(),
                            matcher: AttributeMatcher::Prefix("button".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, elem, &sel),
            "data-type='button-primary' should match [data-type^=button]"
        );
    }

    /// 测试属性选择器 Suffix。
    #[test]
    fn test_attribute_suffix_match() {
        let mut doc = Document::new();
        let elem = doc.create_element("a");
        doc.set_attribute(elem, "href", "https://example.com/page");
        let root = doc.root();
        doc.append_child(root, elem).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "href".to_string(),
                            matcher: AttributeMatcher::Suffix("/page".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, elem, &sel),
            "href ending with '/page' should match [href$='/page']"
        );
    }

    /// 测试属性选择器 Substring。
    #[test]
    fn test_attribute_substring_match() {
        let mut doc = Document::new();
        let elem = doc.create_element("a");
        doc.set_attribute(elem, "href", "https://example.com/docs/api");
        let root = doc.root();
        doc.append_child(root, elem).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "href".to_string(),
                            matcher: AttributeMatcher::Substring("example".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, elem, &sel),
            "href containing 'example' should match [href*=example]"
        );
    }

    /// 测试类型选择器大小写不敏感。
    #[test]
    fn test_tag_selector_case_insensitive() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = make_tag_selector("DIV");
        assert!(
            matches_selector(&doc, div, &sel),
            "DIV should match div (case insensitive)"
        );
    }

    /// 测试空选择器不匹配任何元素。
    #[test]
    fn test_empty_selector_no_match() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = Selector {
            complex: ComplexSelector { parts: vec![] },
        };
        assert!(
            !matches_selector(&doc, div, &sel),
            "empty selector should not match"
        );
    }

    /// 测试 :not() 排除匹配。
    #[test]
    fn test_not_excludes_matching() {
        let (doc, _html, _body, _div, p) = make_test_dom();

        // p:not(.container) — p 没有 container 类，应匹配
        let sel_not_container = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_class_selector("container")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, p, &sel_not_container),
            "p without .container should match :not(.container)"
        );

        // p:not(.text) — p 有 text 类，不应匹配
        let sel_not_text = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_class_selector("text")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            !matches_selector(&doc, p, &sel_not_text),
            "p.text should not match :not(.text)"
        );
    }

    /// 测试伪元素选择器不匹配任何元素。
    #[test]
    fn test_pseudo_element_never_matches() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoElement(
                            zero_css_parser::ast::PseudoElementSelector::Standard(
                                "before".to_string(),
                            ),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            !matches_selector(&doc, div, &sel),
            "pseudo-element should never match DOM elements"
        );
    }

    // ── :has() 伪类匹配测试 ──

    /// 测试 :has(.child) 匹配拥有 .child 后代的父元素。
    #[test]
    fn test_has_descendant_match() {
        let mut doc = Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        doc.append_child(root, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "child");
        doc.append_child(parent, child).unwrap();

        // div:has(.child)
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Has(vec![make_class_selector("child")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, parent, &sel),
            "div with .child descendant should match :has(.child)"
        );
    }

    /// 测试 :has(> .direct) 匹配拥有 .direct 直接子元素的父元素。
    #[test]
    fn test_has_direct_child_match() {
        let mut doc = Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        doc.append_child(root, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "direct");
        doc.append_child(parent, child).unwrap();

        // div:has(> .direct) — parsed as * > .direct
        let inner_sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Universal),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::Child),
                    ),
                    (
                        CompoundSelector {
                            type_selector: None,
                            subclass_selectors: vec![SubclassSelector::Class("direct".to_string())],
                        },
                        None,
                    ),
                ],
            },
        };
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Has(vec![inner_sel]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, parent, &sel),
            "div with .direct child should match :has(> .direct)"
        );
    }

    /// 测试 :has(.absent) 不匹配没有 .absent 后代的父元素。
    #[test]
    fn test_has_no_match() {
        let mut doc = Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        doc.append_child(root, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "other");
        doc.append_child(parent, child).unwrap();

        // div:has(.absent)
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Has(vec![make_class_selector("absent")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            !matches_selector(&doc, parent, &sel),
            "div without .absent descendant should not match :has(.absent)"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // 扩展测试 — 新增伪类选择器匹配
    // ═══════════════════════════════════════════════════════════════════

    /// 测试 :only-child 匹配。
    #[test]
    fn test_only_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let only = doc.create_element("span");
        let _ = doc.append_child(parent, only);

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("only-child".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, only, &sel));

        // 添加第二个子元素后不再匹配
        let sibling = doc.create_element("p");
        let _ = doc.append_child(parent, sibling);
        assert!(!matches_selector(&doc, only, &sel));
    }

    /// 测试 :first-of-type 匹配。
    #[test]
    fn test_first_of_type() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let first = doc.create_element("span");
        let second = doc.create_element("span");
        let p = doc.create_element("p");
        let _ = doc.append_child(parent, first);
        let _ = doc.append_child(parent, p);
        let _ = doc.append_child(parent, second);

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("first-of-type".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, first, &sel));
        assert!(!matches_selector(&doc, second, &sel));
    }

    /// 测试 :last-of-type 匹配。
    #[test]
    fn test_last_of_type() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let first = doc.create_element("span");
        let second = doc.create_element("span");
        let p = doc.create_element("p");
        let _ = doc.append_child(parent, first);
        let _ = doc.append_child(parent, second);
        let _ = doc.append_child(parent, p);

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("last-of-type".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, first, &sel));
        assert!(matches_selector(&doc, second, &sel));
    }

    /// 测试 :only-of-type 匹配。
    #[test]
    fn test_only_of_type() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let only_p = doc.create_element("p");
        let span = doc.create_element("span");
        let _ = doc.append_child(parent, only_p);
        let _ = doc.append_child(parent, span);

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("only-of-type".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, only_p, &sel));

        // 添加第二个 p 后不再匹配
        let second_p = doc.create_element("p");
        let _ = doc.append_child(parent, second_p);
        assert!(!matches_selector(&doc, only_p, &sel));
    }

    /// 测试 :nth-last-child() 匹配（从末尾计数）。
    #[test]
    fn test_nth_last_child() {
        use zero_css_parser::ast::NthPattern;
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let c1 = doc.create_element("span");
        let c2 = doc.create_element("span");
        let c3 = doc.create_element("span");
        let _ = doc.append_child(parent, c1);
        let _ = doc.append_child(parent, c2);
        let _ = doc.append_child(parent, c3);

        // :nth-last-child(1) 应匹配最后一个
        let sel_last = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthLastChild(NthPattern {
                                a: 0,
                                b: 1,
                            }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, c1, &sel_last));
        assert!(!matches_selector(&doc, c2, &sel_last));
        assert!(matches_selector(&doc, c3, &sel_last));

        // :nth-last-child(2) 应匹配倒数第二个
        let sel_second_last = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthLastChild(NthPattern {
                                a: 0,
                                b: 2,
                            }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, c1, &sel_second_last));
        assert!(matches_selector(&doc, c2, &sel_second_last));
        assert!(!matches_selector(&doc, c3, &sel_second_last));
    }

    /// 测试 :nth-of-type() 匹配（按类型计数）。
    #[test]
    fn test_nth_of_type() {
        use zero_css_parser::ast::NthPattern;
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let s1 = doc.create_element("span");
        let p1 = doc.create_element("p");
        let s2 = doc.create_element("span");
        let p2 = doc.create_element("p");
        let _ = doc.append_child(parent, s1);
        let _ = doc.append_child(parent, p1);
        let _ = doc.append_child(parent, s2);
        let _ = doc.append_child(parent, p2);

        // :nth-of-type(2) 在 span 中应匹配 s2（第二个 span）
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthOfType(NthPattern { a: 0, b: 2 }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, s1, &sel));
        assert!(matches_selector(&doc, s2, &sel));

        // :nth-of-type(1) 在 p 中应匹配 p1
        let sel_p = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthOfType(NthPattern { a: 0, b: 1 }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, p1, &sel_p));
        assert!(!matches_selector(&doc, p2, &sel_p));
    }

    /// 测试 :nth-last-of-type() 匹配（从末尾按类型计数）。
    #[test]
    fn test_nth_last_of_type() {
        use zero_css_parser::ast::NthPattern;
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let s1 = doc.create_element("span");
        let p1 = doc.create_element("p");
        let s2 = doc.create_element("span");
        let _ = doc.append_child(parent, s1);
        let _ = doc.append_child(parent, p1);
        let _ = doc.append_child(parent, s2);

        // :nth-last-of-type(1) 在 span 中应匹配 s2（最后一个 span）
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthLastOfType(NthPattern { a: 0, b: 1 }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, s1, &sel));
        assert!(matches_selector(&doc, s2, &sel));
    }

    /// 测试 :nth-of-type(odd) 匹配奇数位置的同类型元素。
    #[test]
    fn test_nth_of_type_odd() {
        use zero_css_parser::ast::NthPattern;
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        let s1 = doc.create_element("span");
        let s2 = doc.create_element("span");
        let s3 = doc.create_element("span");
        let s4 = doc.create_element("span");
        let _ = doc.append_child(parent, s1);
        let _ = doc.append_child(parent, s2);
        let _ = doc.append_child(parent, s3);
        let _ = doc.append_child(parent, s4);

        // odd = 2n+1
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthOfType(NthPattern { a: 2, b: 1 }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, s1, &sel));  // 1st
        assert!(!matches_selector(&doc, s2, &sel)); // 2nd
        assert!(matches_selector(&doc, s3, &sel));  // 3rd
        assert!(!matches_selector(&doc, s4, &sel)); // 4th
    }

    // ═══════════════════════════════════════════════════════════════════
    // ContainerContext 和 @container 规则匹配测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_container_context_creation() {
        let ctx = ContainerContext::new();
        assert_eq!(ctx.container_width, None);
        assert_eq!(ctx.container_height, None);

        let ctx = ContainerContext::with_size(400.0, 600.0);
        assert_eq!(ctx.container_width, Some(400.0));
        assert_eq!(ctx.container_height, Some(600.0));
    }

    #[test]
    fn test_container_context_default() {
        let ctx = ContainerContext::default();
        assert_eq!(ctx.container_width, None);
        assert_eq!(ctx.container_height, None);
    }

    #[test]
    fn test_container_rule_collects_declarations() {
        use zero_css_parser::ast::{ContainerCondition, ContainerRule, ContainerSizeCondition, Declaration, StyleRule};

        let (doc, _html, _body, _div, p) = make_test_dom();

        let container_rule = ContainerRule {
            name: None,
            condition: ContainerCondition::Size(ContainerSizeCondition {
                feature: "min-width".to_string(),
                value: "400px".to_string(),
            }),
            rules: vec![zero_css_parser::ast::Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("p")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        };

        let stylesheets = vec![zero_css_parser::Stylesheet {
            rules: vec![zero_css_parser::ast::Rule::Container(container_rule)],
        }];

        // 无容器上下文时，@container 规则不应用
        let results = collect_matching_declarations(&doc, p, &stylesheets);
        assert_eq!(results.len(), 0, "@container should not apply without context");

        // 容器宽度 >= 400px 时，规则应用
        let ctx = ContainerContext::with_size(500.0, 600.0);
        let results = collect_matching_declarations_with_media(
            &doc, p, &stylesheets, None, Some(&ctx),
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "color");
        assert_eq!(results[0].1, "red");

        // 容器宽度 < 400px 时，规则不应用
        let ctx_small = ContainerContext::with_size(300.0, 600.0);
        let results = collect_matching_declarations_with_media(
            &doc, p, &stylesheets, None, Some(&ctx_small),
        );
        assert_eq!(results.len(), 0, "@container min-width:400px should not apply at 300px");
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增匹配器边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// :has() 带后代组合器
    fn test_has_with_descendant_combinator() {
        let mut doc = Document::new();
        let root = doc.root();
        let grandparent = doc.create_element("section");
        doc.append_child(root, grandparent).unwrap();
        let parent = doc.create_element("div");
        doc.append_child(grandparent, parent).unwrap();
        let child = doc.create_element("span");
        doc.set_attribute(child, "class", "target");
        doc.append_child(parent, child).unwrap();

        // section:has(div .target)
        let inner_sel = Selector {
            complex: ComplexSelector {
                parts: vec![
                    (
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        Some(Combinator::Descendant),
                    ),
                    (
                        CompoundSelector {
                            type_selector: None,
                            subclass_selectors: vec![SubclassSelector::Class("target".to_string())],
                        },
                        None,
                    ),
                ],
            },
        };
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("section".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Has(vec![inner_sel]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(
            matches_selector(&doc, grandparent, &sel),
            "section containing div .target should match :has(div .target)"
        );
    }

    #[test]
    /// :not() 带多个选择器
    fn test_not_with_multiple_selectors() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        // :not(div, span)
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![
                                make_tag_selector("div"),
                                make_tag_selector("span"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };

        // div 不匹配 :not(div, span)
        assert!(!matches_selector(&doc, div, &sel));
    }

    #[test]
    /// :is() 匹配 vs :where() 匹配（两者匹配逻辑相同，区别在特异性）
    fn test_is_and_where_matching_logic() {
        let (doc, _html, _body, div, _p) = make_test_dom();

        let is_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Is(vec![
                                make_tag_selector("div"),
                                make_tag_selector("span"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };

        let where_sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Where(vec![
                                make_tag_selector("div"),
                                make_tag_selector("span"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };

        // 两者的匹配逻辑相同
        assert!(matches_selector(&doc, div, &is_sel));
        assert!(matches_selector(&doc, div, &where_sel));
    }

    #[test]
    /// 通用选择器匹配所有元素
    fn test_universal_selector_matches_all() {
        let (doc, _html, _body, div, p) = make_test_dom();
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel));
        assert!(matches_selector(&doc, p, &sel));
    }

    #[test]
    /// 属性 Includes matcher 匹配空格分隔值
    fn test_attribute_includes_match() {
        let mut doc = Document::new();
        let elem = doc.create_element("div");
        doc.set_attribute(elem, "class", "foo bar baz");
        let root = doc.root();
        doc.append_child(root, elem).unwrap();

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                            name: "class".to_string(),
                            matcher: AttributeMatcher::Includes("bar".to_string()),
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, elem, &sel));
    }

    #[test]
    /// :nth-child(1) 匹配第一个子元素
    fn test_nth_child_first() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let _ = doc.append_child(parent, li1);
        let _ = doc.append_child(parent, li2);

        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("li".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthChild(zero_css_parser::ast::NthPattern {
                                a: 0,
                                b: 1,
                            }),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, li1, &sel));
        assert!(!matches_selector(&doc, li2, &sel));
    }
}
