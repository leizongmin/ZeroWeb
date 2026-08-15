//! 选择器匹配 / 子树查询 host 实现。从 js_dom_bridge.rs 拆出（R2975，文件大小治理 slice 3）。
//! `element.matches()` / `closest()` / 元素子树 querySelector(All) / children / sibling / parent
//! 选择器（经 `__zw_matches` / `__zw_closest` / `__zw_query_*_sub` / `__zw_children` 等回调）。
//! `use super::*` 复用父模块 find_by_selector / unique_selector_for_node / element_parent /
//! json_str（子模块可访祖先私有项）+ zero_dom::parse_html。pub 函数经 `pub use selector_match::*`
//! 重导出，register_dom_callbacks 调用点零改动。

use super::*;

/// `element.matches(selector)`——元素是否匹配选择器（含组合器，querySelectorAll 全匹配语义）。
/// 求全匹配集，判 elem 是否在集中。供 `__zw_matches` 回调 → shim `el.matches()`。
pub fn element_matches_test_selector(html: &str, elem_sel: &str, test_sel: &str) -> bool {
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, elem_sel) else {
        return false;
    };
    let root = doc.root();
    doc.query_selector_all(root, test_sel.trim())
        .into_iter()
        .any(|n| n == node)
}

/// `element.closest(selector)`——自身或最近祖先中首个匹配 test_sel 的元素，返其唯一选择器；
/// 无匹配返空串。沿 parent_node 链（含自身）逐层判全匹配集（含组合器）。供 `__zw_closest` 回调
/// → shim `el.closest()`（返元素 proxy 或 null）。
pub fn closest_matching_selector(html: &str, elem_sel: &str, test_sel: &str) -> String {
    let doc = parse_html(html);
    let Some(start) = find_by_selector(&doc, elem_sel) else {
        return String::new();
    };
    let root = doc.root();
    let matched: std::collections::HashSet<NodeId> =
        doc.query_selector_all(root, test_sel.trim()).into_iter().collect();
    let mut cur = Some(start);
    while let Some(n) = cur {
        if n == root {
            break; // 文档根非元素，不可匹配选择器。
        }
        if matched.contains(&n) {
            return unique_selector_for_node(&doc, n).unwrap_or_default();
        }
        cur = doc.parent_node(n);
    }
    String::new()
}

/// `element.querySelector(selector)`——元素**子树**内首个匹配元素（spec：仅后代，不含元素自身），
/// 返其唯一选择器；无匹配返空串。区别于文档作用域的 [`query_match_selector`]。供
/// `__zw_query_match_sub` 回调 → shim 元素 `el.querySelector()`。
pub fn query_match_in_subtree(html: &str, elem_sel: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let Some(root) = find_by_selector(&doc, elem_sel) else {
        return String::new();
    };
    doc.query_selector(root, selector.trim())
        // 子树作用域：排除元素自身（dom crate query_selector 含 root，spec descendants-only）。
        .filter(|n| *n != root)
        .and_then(|n| unique_selector_for_node(&doc, n))
        .unwrap_or_default()
}

/// `element.querySelectorAll(selector)`——元素**子树**内全部匹配元素（spec：仅后代），
/// 返 `|` 分隔的唯一选择器串；无匹配返空串。区别于文档作用域的 [`query_all_selector_list`]。
/// 供 `__zw_query_all_sub` 回调 → shim 元素 `el.querySelectorAll()`。
pub fn query_all_in_subtree(html: &str, elem_sel: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let Some(root) = find_by_selector(&doc, elem_sel) else {
        return String::new();
    };
    doc.query_selector_all(root, selector.trim())
        .into_iter()
        // 子树作用域：排除元素自身（dom crate collect_matching 含 root，spec descendants-only）。
        .filter(|id| *id != root)
        .filter_map(|id| unique_selector_for_node(&doc, id))
        .collect::<Vec<_>>()
        .join("|")
}

/// 元素的**元素子**（跳过文本/注释）唯一选择器，`|` 分隔；无返空串。供 `__zw_element_children`
/// 回调 → shim `el.children` / `firstElementChild` / `lastElementChild` / `childElementCount`。
pub fn element_children_selectors(html: &str, elem_sel: &str) -> String {
    let doc = parse_html(html);
    element_children_selectors_doc(&doc, elem_sel)
}

/// [`element_children_selectors`] 的 doc 版（`with_query_doc` 缓存——js-dom R52 per-op 成本：
/// DOM 遍历族回调旧实现每次全文档 re-parse，`body.firstChild` 实测 1.2ms/次）。
pub fn element_children_selectors_doc(doc: &Document, elem_sel: &str) -> String {
    let Some(node) = find_by_selector(doc, elem_sel) else {
        return String::new();
    };
    doc.child_nodes(node)
        .into_iter()
        .filter(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
        .filter_map(|c| unique_selector_for_node(doc, c))
        .collect::<Vec<_>>()
        .join("|")
}

/// 元素的**前/后元素兄弟**唯一选择器，`prev|next` 格式（空字段 = 无该方向兄弟）；元素无父或
/// elem_sel 不解析返 `|`（两空）。供 `__zw_element_siblings` 回调 → shim `previousElementSibling`
/// / `nextElementSibling`。
pub fn element_sibling_selectors(html: &str, elem_sel: &str) -> String {
    let doc = parse_html(html);
    element_sibling_selectors_doc(&doc, elem_sel)
}

/// [`element_sibling_selectors`] 的 doc 版（`with_query_doc` 缓存——js-dom R52）。
pub fn element_sibling_selectors_doc(doc: &Document, elem_sel: &str) -> String {
    let empty = || String::from("|");
    let Some(node) = find_by_selector(doc, elem_sel) else {
        return empty();
    };
    let Some(parent) = doc.parent_node(node) else {
        return empty();
    };
    let sibs: Vec<NodeId> = doc
        .child_nodes(parent)
        .into_iter()
        .filter(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
        .collect();
    let Some(idx) = sibs.iter().position(|s| *s == node) else {
        return empty();
    };
    let prev = if idx > 0 {
        unique_selector_for_node(doc, sibs[idx - 1]).unwrap_or_default()
    } else {
        String::new()
    };
    let next = if idx + 1 < sibs.len() {
        unique_selector_for_node(doc, sibs[idx + 1]).unwrap_or_default()
    } else {
        String::new()
    };
    format!("{prev}|{next}")
}

/// 元素的**元素父**唯一选择器（`#outer > #inner` 的 inner → `#outer`）；无元素父（根 `<html>` /
/// elem_sel 不解析）返空串。供 `__zw_parent` 回调 → shim `el.parentNode` / `el.parentElement`
///（修正旧 stub 对嵌套元素恒返 body 的 bug）。复用 [`element_parent`]。
pub fn parent_selector_for(html: &str, elem_sel: &str) -> String {
    let doc = parse_html(html);
    parent_selector_for_doc(&doc, elem_sel)
}

/// [`parent_selector_for`] 的 doc 版（`with_query_doc` 缓存——js-dom R52）。
pub fn parent_selector_for_doc(doc: &Document, elem_sel: &str) -> String {
    let Some(node) = find_by_selector(doc, elem_sel) else {
        return String::new();
    };
    element_parent(doc, node)
        .and_then(|p| unique_selector_for_node(doc, p))
        .unwrap_or_default()
}
