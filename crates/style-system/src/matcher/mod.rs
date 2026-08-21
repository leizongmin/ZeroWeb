//! CSS 选择器匹配。
//!
//! 实现选择器与 DOM 元素的匹配逻辑，从右到左遍历选择器部分，
//! 检查标签名、ID、类、属性和伪类。

/// 匹配声明结果类型：(属性名, 属性值, 是否important, 特异性, 层索引)
type MatchingDecl = (String, String, bool, (u32, u32, u32), Option<usize>);

use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, PseudoClassSelector, Selector,
    SubclassSelector, TypeSelector,
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
        // `&` 嵌套选择器：编译后已被替换为父级化合物，正常路径不应到达此处。
        // 兜底返回 true（匹配任意）——仅未编译残余会命中，不影响正确编译的规则。
        SubclassSelector::Nesting => true,
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

    // CSS Selectors Level 4 §6.3：大小写修饰符三态。
    //   `i` → 强制 ASCII 大小写不敏感（覆盖文档语言默认，无论 HTML/XML）。
    //   `s` → 强制大小写敏感（覆盖文档语言默认）。
    //   缺省 → 按文档语言默认（HTML 不敏感、XML/XHTML 敏感）。
    match attr.case {
        AttrCaseModifier::Insensitive => {
            let value_lower = value.to_ascii_lowercase();
            return match &attr.matcher {
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
            };
        }
        AttrCaseModifier::Sensitive => {
            return match_attr_value_exact(&attr.matcher, &value);
        }
        AttrCaseModifier::Default => {}
    }

    // CSS Selectors §6.3「case-sensitivity depends on the document language」：
    // - HTML 文档：属性名与属性值匹配 ASCII 大小写不敏感（WPT attribute-value-selector-007
    //   assert `[lang="es"]` 应匹配 `lang="ES"`）。
    // - XML/XHTML 文档：大小写敏感（WPT attribute-value-selector-008/009 assert `[title="es"]`
    //   不应匹配 `title="ES"`；meta `nonHTML` flag）。
    // ZW 用 html5ever 统一按 HTML 解析，但 parser 检测 DOCTYPE public_id 含 "XHTML" 时置位
    // `content_is_xml`，此处据此分发大小写语义。
    if doc.content_is_xml() {
        return match_attr_value_exact(&attr.matcher, &value);
    }

    // HTML：属性值匹配 ASCII 大小写不敏感（to_ascii_lowercase 后比较）。
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

/// 大小写敏感地比较属性值（用于 XML/XHTML 默认 + `s` 修饰符强制敏感）。
fn match_attr_value_exact(matcher: &AttributeMatcher, value: &str) -> bool {
    match matcher {
        AttributeMatcher::Exists => true,
        AttributeMatcher::Exact(v) => value == v.as_str(),
        AttributeMatcher::Includes(v) => value.split_whitespace().any(|part| part == v.as_str()),
        AttributeMatcher::DashMatch(v) => value == v.as_str() || value.starts_with(&format!("{v}-")),
        AttributeMatcher::Prefix(v) => value.starts_with(v.as_str()),
        AttributeMatcher::Suffix(v) => value.ends_with(v.as_str()),
        AttributeMatcher::Substring(v) => value.contains(v.as_str()),
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
            // 表单状态伪类：静态 HTML 下 checkedness/selectedness = 布尔属性存在性（无 JS 交互）
            "checked" => is_checked(doc, element),
            "disabled" => is_disabled(doc, element),
            "enabled" => is_enabled(doc, element),
            "required" => is_required(doc, element),
            "optional" => is_optional(doc, element),
            "read-only" => is_read_only(doc, element),
            "read-write" => is_read_write(doc, element),
            "placeholder-shown" => is_placeholder_shown(doc, element),
            // `:blank`——值空或纯空白的文本输入控件（CSS UI L4 / Selectors L4 §12）。
            // R3300 DOM/CSS 同源：委派 Document::is_blank_element（与 :placeholder-shown 空值检测同源，
            // 但不要求 placeholder 属性）。
            "blank" => doc.is_blank_element(element),
            // :default / :indeterminate：HTML 静态语义（无 JS 交互状态）
            "default" => is_default(doc, element),
            "indeterminate" => is_indeterminate(doc, element),
            // :any-link / :link：超链接（a/area/link 带 href）。
            // :link 静态下等价 :any-link（全当未访问，隐私安全）；:visited 永不匹配（走 _ => false）。
            "any-link" => is_any_link(doc, element),
            "link" => is_any_link(doc, element),
            // :scope：文档样式表中等价 :root（匹配文档根元素）。
            "scope" => is_root_element(doc, element),
            // :target：当前文档 URL fragment 指向的唯一元素（CSS Selectors L3 §6.6.2）。
            // 委派 Document 权威方法（R3283 与 DOM 选择器同源，逻辑在 dom/document/target.rs）。
            "target" => doc.is_target_element(element),
            // 约束校验伪类（HTML §4.10.20 + CSS Selectors L4）：候选校验元素的约束状态。
            // 委派 Document 权威方法（R3284 与 DOM 选择器同源，逻辑在 dom/document/validation.rs）。
            "valid" => doc.is_valid_element(element),
            "invalid" => doc.is_invalid_element(element),
            "in-range" => doc.is_in_range_element(element),
            "out-of-range" => doc.is_out_of_range_element(element),
            // `:defined`——HTML §3.1.3：内置元素或已升级 custom element 匹配；未升级（合法 CE 名）不匹配。
            // R3299 DOM/CSS 同源：复用 dom `is_valid_custom_element_name` 静态近似（合法 CE 名 → 未升级 → 不匹配）。
            "defined" => match element_tag_name(doc, element) {
                Some(tag) => !zero_dom::is_valid_custom_element_name(&tag),
                None => false,
            },
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
        PseudoClassSelector::NthChildOf(pattern, of) => matches_nth_child_of(doc, element, pattern, of),
        PseudoClassSelector::NthLastChildOf(pattern, of) => matches_nth_last_child_of(doc, element, pattern, of),
        PseudoClassSelector::NthOfType(pattern) => matches_nth_of_type(doc, element, pattern),
        PseudoClassSelector::NthLastOfType(pattern) => matches_nth_last_of_type(doc, element, pattern),
        // `:lang()`/`:dir()` 委派 Document 权威方法（R3281 与 DOM 选择器同源，逻辑提升见 dom/document/lang_dir.rs）。
        PseudoClassSelector::Lang(range) => doc.matches_lang(element, range),
        PseudoClassSelector::Dir(dir) => doc.matches_dir(element, dir),
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
    // CSS Selectors §:empty：元素无「有内容的子节点」即为空。注释、处理指令、属性、空字符串
    // 文本节点不计入；但**任何非空文本（含纯空白）**都使元素非空（WPT selectors-empty-001.xml
    // line 40 `<test6> </test6>` 在 :not(:empty) 块 → 纯空白文本使元素非空，与 Chromium 一致）。
    for &child in &children {
        if let Some(node) = doc.get(child) {
            match &node.kind {
                NodeKind::Element(_) => return false,
                NodeKind::Text(data) if !data.content.is_empty() => return false,
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

/// 元素是否匹配 `of S` 选择器列表中的任一选择器。
fn matches_any_selector(doc: &Document, element: NodeId, selectors: &[Selector]) -> bool {
    selectors.iter().any(|s| matches_selector(doc, element, s))
}

/// `:nth-child(an+b of S)`（Selectors L4）：元素须匹配 S，且在父元素子代中**仅计匹配 S
/// 的元素**的位置满足 an+b。
fn matches_nth_child_of(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
    of_selectors: &[Selector],
) -> bool {
    if !matches_any_selector(doc, element, of_selectors) {
        return false;
    }
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let mut index = 0;
    for &child in &doc.child_nodes(parent) {
        if is_element(doc, child) && matches_any_selector(doc, child, of_selectors) {
            index += 1;
            if child == element {
                return matches_nth_pattern(index, pattern);
            }
        }
    }
    false
}

/// `:nth-last-child(an+b of S)`（Selectors L4）：从末尾仅计匹配 S 的兄弟。
fn matches_nth_last_child_of(
    doc: &Document,
    element: NodeId,
    pattern: &zero_css_parser::ast::NthPattern,
    of_selectors: &[Selector],
) -> bool {
    if !matches_any_selector(doc, element, of_selectors) {
        return false;
    }
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let matching: Vec<NodeId> = doc
        .child_nodes(parent)
        .iter()
        .copied()
        .filter(|&c| is_element(doc, c) && matches_any_selector(doc, c, of_selectors))
        .collect();
    for (i, &child) in matching.iter().rev().enumerate() {
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

/// `:checked` 匹配（HTML §4.15）：静态 HTML 下 checkedness/selectedness 由布尔属性决定。
/// 匹配 `<input type="checkbox"|"radio" checked>`（默认 type=text 不匹配）与
/// `<option selected>`；其他元素不匹配。
fn is_checked(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    match tag.as_str() {
        "input" => {
            let ty = doc
                .get_attribute(element, "type")
                .unwrap_or_default()
                .to_ascii_lowercase();
            (ty == "checkbox" || ty == "radio") && doc.get_attribute(element, "checked").is_some()
        }
        "option" => doc.get_attribute(element, "selected").is_some(),
        _ => false,
    }
}

/// 可禁用元素（HTML spec `:disabled`/`:enabled` 适用集）。
/// 含 `fieldset`——其自身带 disabled 时匹配 `:disabled`（用于禁用态样式）。
fn is_disableable_tag(tag: &str) -> bool {
    matches!(
        tag,
        "input" | "button" | "select" | "textarea" | "optgroup" | "option" | "fieldset"
    )
}

/// 是否为表单控件（`<fieldset disabled>` 禁用传播的目标；不含 fieldset 自身）。
fn is_form_control_tag(tag: &str) -> bool {
    matches!(tag, "input" | "button" | "select" | "textarea" | "optgroup" | "option")
}

/// `:disabled` 匹配（HTML spec §4.10.18）。
/// - 表单控件：经 [`Document::is_effectively_disabled`]——含 `<fieldset disabled>` 祖先传播
///   （首个 `<legend>` 内除外），与 DOM `:disabled` 选择器一致；
/// - `<fieldset>` 自身：带 `disabled` 属性即匹配（用于禁用态样式，非传播）。
fn is_disabled(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    if is_form_control_tag(&tag) {
        doc.is_effectively_disabled(element)
    } else {
        // fieldset 自身：按 disabled 属性直判。
        is_disableable_tag(&tag) && doc.get_attribute(element, "disabled").is_some()
    }
}

/// `:enabled` 匹配：可禁用元素且非 `:disabled`。
fn is_enabled(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    if is_form_control_tag(&tag) {
        !doc.is_effectively_disabled(element)
    } else {
        is_disableable_tag(&tag) && doc.get_attribute(element, "disabled").is_none()
    }
}

/// 可设 `required` 的元素（HTML spec `:required`/`:optional` 仅限可约束表单控件）。
fn is_requireable_tag(tag: &str) -> bool {
    matches!(tag, "input" | "select" | "textarea")
}

/// `:required` 匹配：可约束元素带 `required` 属性。
fn is_required(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    is_requireable_tag(&tag) && doc.get_attribute(element, "required").is_some()
}

/// `:optional` 匹配：可约束元素无 `required` 属性（`:required` 在可约束元素上的补集）。
fn is_optional(doc: &Document, element: NodeId) -> bool {
    let tag = match element_tag_name(doc, element) {
        Some(t) => t,
        None => return false,
    };
    is_requireable_tag(&tag) && doc.get_attribute(element, "required").is_none()
}

/// `:read-write` 匹配（CSS Basic UI）：可编辑表单文本控件——委派
/// [`Document::is_effectively_read_write`]（含 `<fieldset disabled>` 祖先传播禁用判定，
/// 与 DOM `:read-write` 选择器同源）。
fn is_read_write(doc: &Document, element: NodeId) -> bool {
    doc.is_effectively_read_write(element)
}

/// `:read-only` 匹配：非 `:read-write`（所有不可编辑元素，含 `<p>`/`<div>` 等非表单元素）。
fn is_read_only(doc: &Document, element: NodeId) -> bool {
    !is_read_write(doc, element)
}

/// `:placeholder-shown` 匹配（CSS UI）——委派 [`Document::is_placeholder_shown`]
///（与 DOM `:placeholder-shown` 选择器同源）。
fn is_placeholder_shown(doc: &Document, element: NodeId) -> bool {
    doc.is_placeholder_shown(element)
}

/// `:default` 匹配（HTML §4.15）——委派 [`Document::is_default_form_element`]
///（与 DOM `:default` 选择器同源）。
fn is_default(doc: &Document, element: NodeId) -> bool {
    doc.is_default_form_element(element)
}

/// `:indeterminate` 匹配（HTML §4.15 静态可判定子集）——委派 [`Document::is_indeterminate`]
///（与 DOM `:indeterminate` 选择器同源）。
fn is_indeterminate(doc: &Document, element: NodeId) -> bool {
    doc.is_indeterminate(element)
}

/// `:any-link` / `:link` 匹配（CSS Selectors L4 §18）：超链接元素——`<a>`/`<area>`/`<link>`
/// 带 `href` 属性。`:link` 静态下等价（全当未访问，隐私安全）。
fn is_any_link(doc: &Document, element: NodeId) -> bool {
    let Some(tag) = element_tag_name(doc, element) else {
        return false;
    };
    if !matches!(tag.as_str(), "a" | "area" | "link") {
        return false;
    }
    doc.get_attribute(element, "href").is_some()
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
        // general-enclosed（`(@page)` / `()` 等）恒求值为 false（CSS Conditional §7）。
        SupportsCondition::GeneralEnclosed(_) => false,
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

/// 剥离值末尾的 `!important` / `! important`（CSS 语法 §8.2，`!` 与 `important` 间允许空白）。
/// 用于 @supports 声明支持性求值（`!important` 不影响是否支持）。
fn strip_important(value: &str) -> &str {
    let lower = value.to_ascii_lowercase();
    if let Some(bang) = lower.rfind('!') {
        if lower[bang + 1..].trim_start() == "important" {
            return value[..bang].trim_end();
        }
    }
    value
}

fn supports_rect_values(value: &str, is_valid: fn(&str) -> bool) -> bool {
    let mut count = 0;
    for part in value.split_whitespace() {
        count += 1;
        if count > 4 || !is_valid(part) {
            return false;
        }
    }
    count > 0
}

fn supports_scroll_margin_value(value: &str) -> bool {
    let value = value.trim();
    if matches!(
        value.to_ascii_lowercase().as_str(),
        "auto" | "thin" | "medium" | "thick" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v)) => v.is_finite(),
        Some(zero_css_parser::values::LengthValue::Calc(_)) => true,
        _ => zero_css_parser::values::parse_math_function(value).is_some(),
    }
}

fn supports_scroll_padding_value(value: &str) -> bool {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return true;
    }
    if matches!(
        value.to_ascii_lowercase().as_str(),
        "thin" | "medium" | "thick" | "min-content" | "max-content" | "fit-content"
    ) {
        return false;
    }
    match zero_css_parser::values::parse_length(value) {
        Some(zero_css_parser::values::LengthValue::Px(v))
        | Some(zero_css_parser::values::LengthValue::Em(v))
        | Some(zero_css_parser::values::LengthValue::Ex(v))
        | Some(zero_css_parser::values::LengthValue::Rex(v))
        | Some(zero_css_parser::values::LengthValue::Cap(v))
        | Some(zero_css_parser::values::LengthValue::Rcap(v))
        | Some(zero_css_parser::values::LengthValue::Rem(v))
        | Some(zero_css_parser::values::LengthValue::Vh(v))
        | Some(zero_css_parser::values::LengthValue::Vw(v))
        | Some(zero_css_parser::values::LengthValue::Vmin(v))
        | Some(zero_css_parser::values::LengthValue::Vmax(v))
        | Some(zero_css_parser::values::LengthValue::Ch(v))
        | Some(zero_css_parser::values::LengthValue::Rch(v))
        | Some(zero_css_parser::values::LengthValue::Ic(v))
        | Some(zero_css_parser::values::LengthValue::Ric(v))
        | Some(zero_css_parser::values::LengthValue::Percentage(v)) => v.is_finite() && v >= 0.0,
        _ => false,
    }
}

/// 检查 CSS 属性值对是否受支持。
///
/// 已知属性且值能被解析即为"支持"。
fn is_property_supported(property: &str, value: &str) -> bool {
    use zero_css_parser::values::*;

    let lower = property.to_ascii_lowercase();
    // @supports 声明可带 `!important`（CSS Conditional §7），求值支持性时忽略之。
    // driving: WPT css-supports-004 `(color: green !important)`。
    let trimmed = strip_important(value).trim();
    // CSS 全局关键字（inherit/initial/unset/revert/revert-layer）对所有属性合法——任意属性
    // 都可取全局关键字值，故 `(padding: inherit)` 须判支持（不能因 padding 的长度解析器不
    // 识别 inherit 而判 false）。driving: WPT at-supports-012 `(padding:inherit)` in conjunction。
    let lower_val = trimmed.to_ascii_lowercase();
    if matches!(
        lower_val.as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    ) {
        return true;
    }

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
        "background" | "background-image" => {
            // background 接受 image（gradient / url()）或 color。`url()` 图（含 `<img>` 子资源
            // 路径）ZW 支持（R318 图片贯通 + M7 image 图元），须判支持。driving: WPT at-supports-017。
            parse_gradient(trimmed).is_some()
                || parse_color(trimmed).is_some()
                || trimmed.to_ascii_lowercase().starts_with("url(")
        }
        "scroll-snap-type" => parse_scroll_snap_type(trimmed).is_some(),
        "scroll-snap-align" => parse_scroll_snap_align(trimmed).is_some(),
        "scroll-snap-stop" => parse_scroll_snap_stop(trimmed).is_some(),
        // https://drafts.csswg.org/css-scroll-snap-1/#margin-longhands-physical
        "scroll-margin" => supports_rect_values(trimmed, supports_scroll_margin_value),
        "scroll-margin-top" | "scroll-margin-right" | "scroll-margin-bottom" | "scroll-margin-left" => {
            supports_scroll_margin_value(trimmed)
        }
        // https://drafts.csswg.org/css-scroll-snap-1/#padding-longhands-physical
        "scroll-padding" => supports_rect_values(trimmed, supports_scroll_padding_value),
        "scroll-padding-top" | "scroll-padding-right" | "scroll-padding-bottom" | "scroll-padding-left" => {
            supports_scroll_padding_value(trimmed)
        }
        "container-type" => parse_container_type(trimmed).is_some(),
        "container-name" => true, // 任何非空字符串都有效
        // 字体属性：auto/normal/none 为合法值
        "font-kerning" => {
            let v = trimmed.to_ascii_lowercase();
            v == "auto" || v == "normal" || v == "none"
        }
        "font-variant-numeric"
        | "font-variant-caps"
        | "font-variant-east-asian"
        | "font-variant-position"
        | "font-variant"
        | "font-feature-settings"
        | "font-variation-settings" => true,
        "font-variant-alternates" => parse_font_variant_alternates(trimmed).is_some(),
        "font-stretch" | "font-width" => parse_font_stretch(trimmed).is_some(),
        // https://drafts.csswg.org/css-fonts-4/#font-synthesis
        "font-synthesis" => parse_font_synthesis(trimmed).is_some(),
        "font-synthesis-weight" | "font-synthesis-style" | "font-synthesis-small-caps" | "font-synthesis-position" => {
            let v = trimmed.to_ascii_lowercase();
            v == "auto" || v == "none"
        }
        // font 简写：复用 expand_font 严格校验（须含 font-size + font-family 或系统字体关键字）。
        // driving: WPT css-supports-024 `(font: 16px serif)`。
        "font" => crate::shorthand::font_shorthand_supported(trimmed),
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
    let index = build_stylesheet_index(stylesheets);
    collect_matching_declarations_with_media(doc, element, stylesheets, &index, None, None)
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
    index: &StylesheetIndex,
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();
    let mut layer_counter: usize = 0;

    for (si, stylesheet) in stylesheets.iter().enumerate() {
        collect_from_rules(
            doc,
            element,
            &stylesheet.rules,
            Some(&index.sheets[si]),
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
    index: &StylesheetIndex,
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
    pseudo_name: &str,
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();
    let mut layer_counter: usize = 0;

    for (si, stylesheet) in stylesheets.iter().enumerate() {
        collect_from_rules(
            doc,
            element,
            &stylesheet.rules,
            Some(&index.sheets[si]),
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

/// 单样式表的选择器索引桶：tag（小写）→ 该样式表内 Style 规则的 (规则下标, 选择器下标)。
///
/// 正确性依据：选择器最右复合选择器含 `TypeSelector::Tag(t)` 时，匹配元素 E 的
/// 必要条件即 E 的 tag == t（大小写不敏感）——按 tag 分桶只减少候选、不改变
/// 匹配结果（零语义损失）。无 type selector（`.class`/`#id`/`*` 等）的选择器
/// 可匹配任意 tag，进 `universal` 桶。
struct SheetBuckets {
    /// tag（小写）→ 候选 (规则下标, 选择器下标)，按 (规则, 选择器) 有序。
    tagged: std::collections::HashMap<String, Vec<(usize, usize)>>,
    /// 无 type selector / Universal → 任意 tag 候选。
    universal: Vec<(usize, usize)>,
}

/// 全样式表的选择器索引（compute_styles 每帧构建一次，元素共享）。
pub struct StylesheetIndex {
    sheets: Vec<SheetBuckets>,
}

/// 从选择器最右复合选择器提取 type selector tag（小写）；无/Universal → None。
fn selector_leading_tag(selector: &zero_css_parser::ast::Selector) -> Option<String> {
    use zero_css_parser::ast::{Selector, TypeSelector};
    let Selector { complex } = selector;
    let (compound, _) = complex.parts.last()?;
    match &compound.type_selector {
        Some(TypeSelector::Tag(tag)) => Some(tag.to_ascii_lowercase()),
        Some(TypeSelector::Universal) | None => None,
    }
}

/// 构建全样式表的 tag 分桶索引（O(规则 × 选择器)，每帧一次）。
pub fn build_stylesheet_index(stylesheets: &[zero_css_parser::Stylesheet]) -> StylesheetIndex {
    let mut sheets = Vec::with_capacity(stylesheets.len());
    for stylesheet in stylesheets {
        let mut tagged: std::collections::HashMap<String, Vec<(usize, usize)>> = std::collections::HashMap::new();
        let mut universal: Vec<(usize, usize)> = Vec::new();
        for (ri, rule) in stylesheet.rules.iter().enumerate() {
            let zero_css_parser::ast::Rule::Style(style_rule) = rule else {
                continue; // At 规则不进索引（collect_from_rules 原样遍历）
            };
            for (si, selector) in style_rule.selectors.iter().enumerate() {
                match selector_leading_tag(selector) {
                    Some(tag) => tagged.entry(tag).or_default().push((ri, si)),
                    None => universal.push((ri, si)),
                }
            }
        }
        for bucket in tagged.values_mut() {
            bucket.sort_unstable();
        }
        universal.sort_unstable();
        sheets.push(SheetBuckets { tagged, universal });
    }
    StylesheetIndex { sheets }
}

/// 匹配单个选择器并收集其声明；返回是否匹配。
///
/// 被索引路径（候选选择器）与全量路径（@media 内层）共用，保持收集逻辑单一。
#[allow(clippy::too_many_arguments)]
fn collect_style_rule_decls(
    doc: &Document,
    element: NodeId,
    style_rule: &zero_css_parser::ast::StyleRule,
    selector: &Selector,
    results: &mut Vec<MatchingDecl>,
    current_layer: Option<usize>,
    pseudo: Option<&str>,
) -> bool {
    // pseudo=None: 常规元素匹配；
    // pseudo=Some(name): 仅收集尾部伪元素 == name 的选择器（伪元素声明）。
    let matched = match pseudo {
        None => matches_selector(doc, element, selector),
        Some(name) => matches_selector_for_pseudo(doc, element, selector, name),
    };
    if !matched {
        return false;
    }
    let spec = zero_css_parser::selector::specificity(selector);
    for decl in &style_rule.declarations {
        // CSS Text 3 §7.1：`text-align: justify-all` = justify +
        // text-align-last: justify（末行也两端对齐）。在 declaration 收集层
        // 展开为两个 author declaration，使 cascade 把两者都当 author declaration。
        // apply 层单点特判会被 cascade「text-align-last 无 author declaration →
        // 继承 parent Auto」覆盖（R956 根因）；R955 已让存储路径消费 text-align-last。
        if decl.property.eq_ignore_ascii_case("text-align") && decl.value.trim().eq_ignore_ascii_case("justify-all") {
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
    true
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
    index: Option<&SheetBuckets>,
    results: &mut Vec<MatchingDecl>,
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
    container_ctx: Option<&ContainerContext>,
    current_layer: Option<usize>,
    layer_counter: &mut usize,
    pseudo: Option<&str>,
) {
    // 索引候选：元素 tag 的精确桶 + 通用桶（无 type selector 的选择器）。
    // 同规则多选择器匹配时只收集一次（原语义 break）：候选按 (规则, 选择器)
    // 有序，`last_matched_rule` 跳过已匹配规则的其余选择器。
    match index {
        Some(idx) => {
            let elem_tag = element_tag_name(doc, element).unwrap_or_default();
            let tagged = idx.tagged.get(&elem_tag).map(Vec::as_slice).unwrap_or(&[]);
            let universal = idx.universal.as_slice();
            let mut last_matched_rule = usize::MAX;
            for (ri, si) in tagged.iter().chain(universal.iter()) {
                if *ri == last_matched_rule {
                    continue;
                }
                let zero_css_parser::ast::Rule::Style(style_rule) = &rules[*ri] else {
                    continue; // 索引只含 Style 规则
                };
                let selector = &style_rule.selectors[*si];
                if collect_style_rule_decls(doc, element, style_rule, selector, results, current_layer, pseudo) {
                    last_matched_rule = *ri;
                }
            }
        }
        None => {
            // 无索引（@media 内层规则）：全量遍历（旧行为）
            for rule in rules {
                if let zero_css_parser::ast::Rule::Style(style_rule) = rule {
                    for selector in &style_rule.selectors {
                        if collect_style_rule_decls(doc, element, style_rule, selector, results, current_layer, pseudo)
                        {
                            break; // 一个选择器匹配就够了
                        }
                    }
                }
            }
        }
    }
    // At 规则（@media 等）：不进索引，原样遍历（数量远少于 Style 规则）
    for rule in rules {
        match rule {
            zero_css_parser::ast::Rule::Style(_) => {}
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
                                None, // @media 内层规则未进索引，全量匹配
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
                        // 非 @media 的通用 AtRule（@charset/@foo/@unknown 等）：未知 at-rule
                        // 的 body **不得**作为样式应用（CSS：未知 at-rule 整体忽略，body 不
                        // 参与 cascade）。@supports/@container/@layer 等已知条件 at-rule 各有
                        // 专属 Rule 变体与处理分支（见下方 match），不进此通用 At 分支。
                        // 旧实现对未知 at-rule body 无条件递归→body 内规则泄漏应用
                        //（driving: at-rule-013 `@foo { #block { background: red; } }`）。
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
                    None, // @layer 内层规则未进索引，全量匹配
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
                        None, // @supports 内层规则未进索引，全量匹配
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
                        None, // @container 内层规则未进索引，全量匹配
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
            zero_css_parser::ast::Rule::FontFeatureValues(_) => {
                // 文档级 font feature alias 由 StyleSystem 预扫描解析，不参与选择器匹配。
            }
            zero_css_parser::ast::Rule::Page(_) => {
                // @page 规则不参与元素级样式匹配（页尺寸为文档级，由 render pipeline
                // 从 CSS 提取并注入 print 分页），跳过。
            }
            zero_css_parser::ast::Rule::Property(_) => {
                // @property 规则不参与元素级选择器匹配（注册的自定义属性初值由
                // `compute_styles` 预扫描注入 `registered_properties`，在 var() 解析时
                // 作兜底默认值），跳过。
            }
            zero_css_parser::ast::Rule::CounterStyle(_) => {
                // @counter-style 规则不参与元素级选择器匹配（计数系统由 list-style
                // 消费层从 CSS 提取并注入），跳过。
            }
        }
    }
}

#[cfg(test)]
mod tests;
