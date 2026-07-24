//! CSS 选择器匹配。
//!
//! 实现选择器与 DOM 元素的匹配逻辑，从右到左遍历选择器部分，
//! 检查标签名、ID、类、属性和伪类。

/// 匹配声明结果类型：(属性名, 属性值, 是否important, 特异性, 层索引)
type MatchingDecl = (String, String, bool, (u32, u32, u32), Option<usize>);

use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, PseudoClassSelector, Selector, SubclassSelector,
    TypeSelector,
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
            // 相邻兄弟组合器：检查前一个元素兄弟（跳过文本节点）
            let mut sibling = doc.previous_sibling(current);
            while let Some(sib) = sibling {
                if is_element(doc, sib) {
                    if matches_compound(doc, sib, prev_compound) {
                        return matches_selector_recursive(doc, sib, parts, part_idx - 1);
                    }
                    // 找到了元素兄弟但不匹配，+ 组合器要求紧邻的前一个元素兄弟
                    return false;
                }
                sibling = doc.previous_sibling(sib);
            }
            false
        }
        Some(Combinator::SubsequentSibling) => {
            // 通用兄弟组合器：检查所有前面的元素兄弟（跳过文本节点）
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

    // CSS 属性值选择器在 HTML 中对 ASCII 大小写不敏感（CSS-Selectors §6.3：HTML 文档属性
    // 值匹配 ASCII case-insensitive；`[attr="val" i]` 显式标记仅对 Level 4 语法生效，但
    // HTML 默认即不敏感）。WPT attribute-value-selector-007 assert：`[lang="es"]` 应匹配
    // `lang="ES"`。对全匹配器统一 to_ascii_lowercase 后比较（XML 文档应大小写敏感，但 ZW
    // reftest corpus 全 HTML；若未来接 XML 须按文档类型分发）。
    let value_lower = value.to_ascii_lowercase();
    match &attr.matcher {
        AttributeMatcher::Exists => true,
        AttributeMatcher::Exact(v) => value_lower == v.to_ascii_lowercase(),
        AttributeMatcher::Includes(v) => {
            let vl = v.to_ascii_lowercase();
            value.split_whitespace().any(|part| part.to_ascii_lowercase() == vl)
        }
        AttributeMatcher::DashMatch(v) => {
            let vl = v.to_ascii_lowercase();
            value_lower == vl || value_lower.starts_with(&format!("{vl}-"))
        }
        AttributeMatcher::Prefix(v) => value_lower.starts_with(&v.to_ascii_lowercase()),
        AttributeMatcher::Suffix(v) => value_lower.ends_with(&v.to_ascii_lowercase()),
        AttributeMatcher::Substring(v) => value_lower.contains(&v.to_ascii_lowercase()),
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
        PseudoClassSelector::Lang(range) => matches_lang(doc, element, range),
    }
}

/// `:lang(<range>)` 匹配（CSS 2.1 §5.11.4）。
///
/// 元素语言 = 自身或最近祖先的 `xml:lang`/`lang` 属性（向上查找首个）。匹配当且仅当
/// 元素语言 == range，或以 `range-` 开头（连字符前缀边界），**大小写不敏感**。
/// 例：`:lang(es)` 匹配 `lang="es"`/`"es-MX"`/`"ES"`，不匹配 `"MX-es"`/`"en"`。
fn matches_lang(doc: &Document, element: NodeId, range: &str) -> bool {
    let range_lower = range.to_ascii_lowercase();
    let mut node = Some(element);
    while let Some(n) = node {
        // xml:lang 优先于 lang（XML 规范），HTML 仅 lang。
        if let Some(lang) = doc
            .get_attribute(n, "xml:lang")
            .or_else(|| doc.get_attribute(n, "lang"))
        {
            let lang_lower = lang.to_ascii_lowercase();
            return lang_lower == range_lower || lang_lower.starts_with(&format!("{range_lower}-"));
        }
        node = doc.parent_node(n);
    }
    false
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
fn matches_nth_child(doc: &Document, element: NodeId, pattern: &zero_css_parser::ast::NthPattern) -> bool {
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
fn matches_nth_last_child(doc: &Document, element: NodeId, pattern: &zero_css_parser::ast::NthPattern) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 收集所有元素子节点
    let element_children: Vec<NodeId> = children.iter().copied().filter(|&c| is_element(doc, c)).collect();

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
fn matches_nth_of_type(doc: &Document, element: NodeId, pattern: &zero_css_parser::ast::NthPattern) -> bool {
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
fn matches_nth_last_of_type(doc: &Document, element: NodeId, pattern: &zero_css_parser::ast::NthPattern) -> bool {
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
        .filter(|&c| is_element(doc, c) && element_tag_name(doc, c).as_deref() == Some(tag.as_str()))
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
            // 相邻兄弟组合器：检查前一个元素兄弟（跳过文本节点）
            let mut sibling = doc.previous_sibling(candidate);
            while let Some(sib) = sibling {
                if is_element(doc, sib) {
                    if matches_compound(doc, sib, prev_compound) {
                        return matches_has_selector_chain(doc, sib, parts, part_idx - 1);
                    }
                    return false;
                }
                sibling = doc.previous_sibling(sib);
            }
            false
        }
        Some(Combinator::SubsequentSibling) => {
            // 通用兄弟组合器：检查所有前面的元素兄弟（跳过文本节点）
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

/// 解析长度字符串为像素值。
///
/// 辅助函数，将 parse_length 结果提取为 f64 像素值。
fn length_to_px(value_str: &str) -> Option<f64> {
    use zero_css_parser::values::parse_length;
    parse_length(value_str.trim()).map(|l| match l {
        zero_css_parser::values::LengthValue::Px(n) => n,
        _ => 0.0,
    })
}

/// 根据特性名获取用于比较的容器尺寸（忽略 min-/max- 前缀）。
fn get_axis_size(ctx: &ContainerContext, feature: &str) -> Option<f64> {
    let base = feature.trim_start_matches("min-").trim_start_matches("max-");
    match base {
        "width" | "inline-size" => ctx.container_width,
        "height" | "block-size" => ctx.container_height,
        _ => None,
    }
}

/// 评估 @container 条件。
///
/// 基于 ContainerContext 中的容器尺寸评估容器查询条件。
/// 支持冒号语法 `(min-width: 400px)`、比较运算符 `(width > 300px)`、
/// 范围语法 `(200px <= width <= 500px)`。
/// 没有容器上下文时，@container 规则不应用。
fn evaluate_container_condition(
    container_rule: &zero_css_parser::ast::ContainerRule,
    container_ctx: Option<&ContainerContext>,
) -> bool {
    let Some(ctx) = container_ctx else {
        // 无容器上下文，不应用 @container 规则
        return false;
    };

    let condition = &container_rule.condition;
    let size_cond = match condition {
        zero_css_parser::ast::ContainerCondition::Size(s) | zero_css_parser::ast::ContainerCondition::InlineSize(s) => {
            s
        }
    };

    let feature = size_cond.feature.to_ascii_lowercase();

    // 范围语法：200px <= width <= 500px
    if let (Some(min_str), Some(max_str)) = (&size_cond.range_min, &size_cond.range_max) {
        let min_px = match length_to_px(min_str) {
            Some(v) => v,
            None => return false,
        };
        let max_px = match length_to_px(max_str) {
            Some(v) => v,
            None => return false,
        };
        let actual = match get_axis_size(ctx, &feature) {
            Some(v) => v,
            None => return false,
        };
        return actual >= min_px && actual <= max_px;
    }

    // 比较运算符语法：width > 300px
    if let Some(ref op) = size_cond.operator {
        let cond_px = match length_to_px(&size_cond.value) {
            Some(v) => v,
            None => return false,
        };
        let actual = match get_axis_size(ctx, &feature) {
            Some(v) => v,
            None => return false,
        };
        return match op.as_str() {
            ">" => actual > cond_px,
            ">=" => actual >= cond_px,
            "<" => actual < cond_px,
            "<=" => actual <= cond_px,
            _ => false,
        };
    }

    // 冒号语法：min-width: 400px
    let cond_px = match length_to_px(&size_cond.value) {
        Some(v) => v,
        None => return false,
    };

    // 根据特性名称评估条件
    let result = match feature.as_str() {
        "min-width" | "min-inline-size" => ctx.container_width.map(|w| w >= cond_px),
        "max-width" | "max-inline-size" => ctx.container_width.map(|w| w <= cond_px),
        "width" | "inline-size" => ctx.container_width.map(|w| (w - cond_px).abs() < f64::EPSILON),
        "min-height" | "min-block-size" => ctx.container_height.map(|h| h >= cond_px),
        "max-height" | "max-block-size" => ctx.container_height.map(|h| h <= cond_px),
        "height" | "block-size" => ctx.container_height.map(|h| (h - cond_px).abs() < f64::EPSILON),
        _ => None,
    };

    result.unwrap_or(false)
}

/// 评估 @supports 条件。
///
/// 对于属性值测试 `(property: value)`，检查解析器是否能识别该属性和值。
/// 对于 `selector()`，检查解析器是否能解析该选择器。
/// 对于逻辑组合，递归评估子条件。
fn evaluate_supports_condition(condition: &zero_css_parser::ast::SupportsCondition) -> bool {
    use zero_css_parser::ast::SupportsCondition;

    match condition {
        SupportsCondition::Property(property, value) => is_property_supported(property, value),
        SupportsCondition::Selector(selector_text) => {
            // 尝试解析选择器，能解析即为支持
            let css = format!("{selector_text} {{ }}");
            let stylesheet = zero_css_parser::Parser::parse_stylesheet(&css);
            if let Some(zero_css_parser::ast::Rule::Style(style_rule)) = stylesheet.rules.first() {
                // 额外验证：检查解析结果没有因为容错解析产生无效结构
                is_valid_selector_parse(selector_text, &style_rule.selectors)
            } else {
                false
            }
        }
        SupportsCondition::And(conditions) => conditions.iter().all(evaluate_supports_condition),
        SupportsCondition::Or(conditions) => conditions.iter().any(evaluate_supports_condition),
        SupportsCondition::Not(inner) => !evaluate_supports_condition(inner),
    }
}

/// 验证解析后的选择器是否忠实于输入文本。
///
/// 容错解析器可能将无效选择器（如 `>>>invalid`）解析为合法选择器链，
/// 因为 `>` 被当作子组合器处理。此函数检测此类无效解析结果。
fn is_valid_selector_parse(input: &str, selectors: &[zero_css_parser::ast::Selector]) -> bool {
    if selectors.is_empty() {
        return false;
    }

    // 检查输入中是否存在连续的组合器序列（如 `>>>`、`> +`、`~ >` 等）
    // 连续组合器不是有效的 CSS 选择器语法
    let trimmed = input.trim();

    // 遍历输入中所有非括号内的字符，检查组合器合法性
    let mut depth = 0i32;
    let mut chars_iter = trimmed.chars().peekable();
    while let Some(ch) = chars_iter.next() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 {
            // 检查连续的组合器：> + ~ 后面紧跟空白再跟另一个组合器
            if ch == '>' || ch == '+' || ch == '~' {
                // 跳过空白
                while chars_iter.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
                    chars_iter.next();
                }
                // 如果紧接着又是一个组合器，这是无效的
                if chars_iter.peek().is_some_and(|c| *c == '>' || *c == '+' || *c == '~') {
                    return false;
                }
            }
        }
    }

    // 检查选择器是否以组合器开头（不允许，除了在 :has() 等上下文中）
    if trimmed.starts_with('>') || trimmed.starts_with('+') || trimmed.starts_with('~') {
        return false;
    }

    true
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
        "justify-content" | "align-items" | "align-content" | "align-self" | "justify-self" => {
            parse_alignment(trimmed).is_some()
        }
        "font-weight" => parse_font_weight(trimmed).is_some(),
        "font-style" => parse_font_style(trimmed).is_some(),
        "color"
        | "background-color"
        | "border-color"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => parse_color(trimmed).is_some(),
        "width"
        | "height"
        | "min-width"
        | "max-width"
        | "min-height"
        | "max-height"
        | "margin"
        | "margin-top"
        | "margin-right"
        | "margin-bottom"
        | "margin-left"
        | "padding"
        | "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "gap"
        | "top"
        | "right"
        | "bottom"
        | "left"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-right-radius"
        | "border-bottom-left-radius" => parse_length(trimmed).is_some(),
        "transform" => parse_transform(trimmed).is_some(),
        "background" | "background-image" => parse_gradient(trimmed).is_some() || parse_color(trimmed).is_some(),
        "scroll-snap-type" => parse_scroll_snap_type(trimmed).is_some(),
        "scroll-snap-align" => parse_scroll_snap_align(trimmed).is_some(),
        "scroll-snap-stop" => parse_scroll_snap_stop(trimmed).is_some(),
        "scroll-margin-top" | "scroll-margin-right" | "scroll-margin-bottom" | "scroll-margin-left" => {
            parse_length(trimmed).is_some()
        }
        "scroll-padding-top" | "scroll-padding-right" | "scroll-padding-bottom" | "scroll-padding-left" => {
            trimmed.eq_ignore_ascii_case("auto") || parse_length(trimmed).is_some()
        }
        "container-type" => parse_container_type(trimmed).is_some(),
        "container-name" => true, // 任何非空字符串都有效
        // 字体属性：auto/normal/none 为合法值
        "font-kerning" => {
            let v = trimmed.to_ascii_lowercase();
            v == "auto" || v == "normal" || v == "none"
        }
        "font-variant-numeric" | "font-feature-settings" | "font-variation-settings" => true,
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
            None,
        );
    }

    results
}

/// 从样式表中收集匹配元素指定伪元素（`::before`/`::after`）的声明。
///
/// 与 [`collect_matching_declarations_with_media`] 相同，但只收集选择器尾部
/// 伪元素等于 `pseudo_name` 的规则，且元素需匹配「去除尾部伪元素后的选择器主体」。
/// 特异性使用原选择器（含伪元素贡献）。用于伪元素级联（生成 `content` 文本等）。
pub fn collect_pseudo_declarations_with_media(
    doc: &Document,
    element: NodeId,
    stylesheets: &[zero_css_parser::Stylesheet],
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
    pseudo_name: &str,
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
            Some(pseudo_name),
        );
    }

    results
}

/// 返回选择器的尾部伪元素名（如 `"before"`/`"after"`），若无则 `None`。
///
/// 按 CSS 语法，伪元素总是最后一个复合选择器的最后一个子类选择器。
pub fn selector_pseudo_element(selector: &Selector) -> Option<&str> {
    use zero_css_parser::ast::PseudoElementSelector;
    let (compound, _) = selector.complex.parts.last()?;
    match compound.subclass_selectors.last()? {
        SubclassSelector::PseudoElement(PseudoElementSelector::Standard(name)) => Some(name.as_str()),
        _ => None,
    }
}

/// 检查元素是否匹配「以指定伪元素结尾」的选择器：尾部伪元素必须等于 `pseudo_name`，
/// 且元素匹配去除该尾部伪元素后的选择器主体。
fn matches_selector_for_pseudo(doc: &Document, element: NodeId, selector: &Selector, pseudo_name: &str) -> bool {
    if selector_pseudo_element(selector) != Some(pseudo_name) {
        return false;
    }
    // 克隆并移除尾部伪元素子类，得到「主体」选择器，复用常规匹配。
    let mut stripped = selector.clone();
    let last_compound = match stripped.complex.parts.last_mut() {
        Some(c) => c,
        None => return false,
    };
    last_compound.0.subclass_selectors.pop();
    matches_selector(doc, element, &stripped)
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
    pseudo: Option<&str>,
) {
    for rule in rules {
        match rule {
            zero_css_parser::ast::Rule::Style(style_rule) => {
                // 检查选择器列表中是否有匹配的选择器
                for selector in &style_rule.selectors {
                    // pseudo=None: 常规元素匹配；
                    // pseudo=Some(name): 仅收集尾部伪元素 == name 的选择器（伪元素声明）。
                    let matched = match pseudo {
                        None => matches_selector(doc, element, selector),
                        Some(name) => matches_selector_for_pseudo(doc, element, selector, name),
                    };
                    if matched {
                        let spec = zero_css_parser::selector::specificity(selector);
                        for decl in &style_rule.declarations {
                            // CSS Text 3 §7.1：`text-align: justify-all` = justify +
                            // text-align-last: justify（末行也两端对齐）。在 declaration 收集层
                            // 展开为两个 author declaration，使 cascade 把两者都当 author declaration。
                            // apply 层单点特判会被 cascade「text-align-last 无 author declaration →
                            // 继承 parent Auto」覆盖（R956 根因）；R955 已让存储路径消费 text-align-last。
                            if decl.property.eq_ignore_ascii_case("text-align")
                                && decl.value.trim().eq_ignore_ascii_case("justify-all")
                            {
                                results.push((
                                    "text-align".to_string(),
                                    "justify".to_string(),
                                    decl.important,
                                    spec,
                                    current_layer,
                                ));
                                results.push((
                                    "text-align-last".to_string(),
                                    "justify".to_string(),
                                    decl.important,
                                    spec,
                                    current_layer,
                                ));
                            } else {
                                results.push((
                                    decl.property.clone(),
                                    decl.value.clone(),
                                    decl.important,
                                    spec,
                                    current_layer,
                                ));
                            }
                        }
                        break; // 一个选择器匹配就够了
                    }
                }
            }
            zero_css_parser::ast::Rule::At(at_rule) => {
                if let zero_css_parser::ast::AtRuleBody::Block(inner_rules) = &at_rule.body {
                    if at_rule.name.eq_ignore_ascii_case("media") {
                        // @media 规则：需要评估媒体条件
                        // 逗号分隔的查询表示 OR 关系——任一匹配即通过
                        if let Some(ctx) = media_ctx
                            && let Some(queries) = zero_css_parser::media_query::parse_media_query(&at_rule.prelude)
                            && queries
                                .iter()
                                .any(|q| zero_css_parser::media_query::evaluate_media_query(q, ctx))
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
                                pseudo,
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
                            pseudo,
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
                    pseudo,
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
                        pseudo,
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
                        pseudo,
                    );
                }
            }
            zero_css_parser::ast::Rule::FontFace(_) => {
                // @font-face 规则不参与样式匹配（自定义字体加载由 reftest/webview
                // 调用方在渲染前从 CSS 提取并注入 FontLoader），跳过。
            }
            zero_css_parser::ast::Rule::Page(_) => {
                // @page 规则不参与元素级样式匹配（页尺寸为文档级，由 render pipeline
                // 从 CSS 提取并注入 print 分页），跳过。
            }
        }
    }
}

#[cfg(test)]
mod tests;
