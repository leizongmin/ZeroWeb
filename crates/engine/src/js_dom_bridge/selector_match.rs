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
    doc.query_selector_all(root, zero_dom::trim_ascii_ws(test_sel))
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
    // R153（js-dom M4）：`:scope` 伪类的 closest 作用域语义——spec selectors-4 §6.4
    // 「:scope = the element itself when evaluated against a scoping root; closest 的
    // scoping root = 调用元素」。全匹配集经文档级 querySelectorAll（`:scope` 由 dom crate
    // `is_scope_element` 判为文档根元素 html）→ `test4.closest(':scope')` 命中 html 而非
    // 自身。此处把 test_sel 中的独立 `:scope` token 文本替换为 start 的唯一选择器（WPT
    // Element-closest 四形态：`:scope`/`select > :scope`/`div > :scope`/`:has(> :scope)`）。
    // https://dom.spec.whatwg.org/#dom-element-closest
    let sel_raw = zero_dom::trim_ascii_ws(test_sel);
    let resolved_sel = if sel_raw.contains(":scope") {
        match unique_selector_for_node(&doc, start) {
            Some(unique) => replace_scope_tokens(sel_raw, &unique),
            None => sel_raw.to_string(),
        }
    } else {
        sel_raw.to_string()
    };
    let root = doc.root();
    let matched: std::collections::HashSet<NodeId> = doc
        .query_selector_all(root, zero_dom::trim_ascii_ws(&resolved_sel))
        .into_iter()
        .collect();
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

/// 把选择器串中的独立 `:scope` 伪类 token 替换为 `scope_sel`（closest 调用元素的唯一
/// 选择器）。只匹配 `:scope` 后随空白/组合器/`,`)`/串尾 的独立 token（`:scope` 是合法
/// 伪类名，无参数形态；`:scope()` 非法保持原样由引擎拒配）——不做裸子串替换，避免误伤
/// 未来的 `:scope-x` 类名伪类（当前引擎无此伪类，防御性）。
fn replace_scope_tokens(selector: &str, scope_sel: &str) -> String {
    let bytes = selector.as_bytes();
    let mut out = String::with_capacity(selector.len() + scope_sel.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b':' && selector[i..].starts_with(":scope") {
            let after = i + ":scope".len();
            let boundary = after >= bytes.len()
                || matches!(
                    bytes[after],
                    b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'+' | b'~' | b',' | b')'
                );
            if boundary {
                out.push_str(scope_sel);
                i = after;
                continue;
            }
        }
        // 非 token 起点：逐字符拷贝（保留原选择器文本）。
        let ch = selector[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `element.querySelector(selector)`——元素**子树**内首个匹配元素（spec：仅后代，不含元素自身），
/// 返其唯一选择器；无匹配返空串。区别于文档作用域的 [`query_match_selector`]。供
/// `__zw_query_match_sub` 回调 → shim 元素 `el.querySelector()`。
pub fn query_match_in_subtree(html: &str, elem_sel: &str, selector: &str) -> String {
    let doc = parse_html(html);
    query_match_in_subtree_doc(&doc, elem_sel, selector)
}

/// [`query_match_in_subtree`] 的 doc 版（js-dom M1 L2 R102：调用方经
/// `with_query_doc_live_aware` 供 doc——live 读 or 快照缓存）。
pub fn query_match_in_subtree_doc(doc: &Document, elem_sel: &str, selector: &str) -> String {
    let Some(root) = find_by_selector(doc, elem_sel) else {
        return String::new();
    };
    doc.query_selector(root, zero_dom::trim_ascii_ws(selector))
        // 子树作用域：排除元素自身（dom crate query_selector 含 root，spec descendants-only）。
        .filter(|n| *n != root)
        .and_then(|n| unique_selector_for_node(doc, n))
        .unwrap_or_default()
}

/// `element.querySelectorAll(selector)`——元素**子树**内全部匹配元素（spec：仅后代），
/// 返 `|` 分隔的唯一选择器串；无匹配返空串。区别于文档作用域的 [`query_all_selector_list`]。
/// 供 `__zw_query_all_sub` 回调 → shim 元素 `el.querySelectorAll()`。
pub fn query_all_in_subtree(html: &str, elem_sel: &str, selector: &str) -> String {
    let doc = parse_html(html);
    query_all_in_subtree_doc(&doc, elem_sel, selector)
}

/// [`query_all_in_subtree`] 的 doc 版（js-dom M1 L2 R102：见 [`query_match_in_subtree_doc`]）。
pub fn query_all_in_subtree_doc(doc: &Document, elem_sel: &str, selector: &str) -> String {
    let Some(root) = find_by_selector(doc, elem_sel) else {
        return String::new();
    };
    doc.query_selector_all(root, zero_dom::trim_ascii_ws(selector))
        .into_iter()
        // 子树作用域：排除元素自身（dom crate collect_matching 含 root，spec descendants-only）。
        .filter(|id| *id != root)
        .filter_map(|id| unique_selector_for_node(doc, id))
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
