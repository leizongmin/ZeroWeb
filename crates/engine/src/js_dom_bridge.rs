//! JS → DOM 桥接 — 将 V8 shim 回调记录的变更应用到 `zero_dom::Document`。
//!
//! 与 `zero_script_sandbox::V8Sandbox::register_callback` 配合：JS 侧 shim 把
//! DOM 操作翻译为 `__zw_*` 扁平回调，宿主记录 [`DomMutation`] 后调用
//! [`apply_dom_mutations`] 并序列化回 HTML。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use zero_style_system::ComputedStyle;

use zero_dom::{Document, FocusManager, NodeId, NodeKind, parse_html};
use zero_script_sandbox::Sandbox;

// getComputedStyle 计算与序列化（R2709 从本文件拆出，控制主文件行数）。
mod computed_style;
pub use computed_style::*;

/// 一条 DOM 变更记录（由 JS shim 经 `__zw_*` 回调产生）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomMutation {
    /// `element.setAttribute` / 属性 setter（`src`、`class` 等）。
    SetAttr {
        /// CSS 选择器句柄。
        selector: String,
        /// 属性名。
        name: String,
        /// 属性值。
        value: String,
    },
    /// `element.removeAttribute(...)`——真正移除属性（区别于 `SetAttr` 空值；
    /// 布尔属性如 `checked`/`disabled` 需移除才能 unset，P1a checkbox toggle）。
    RemoveAttr {
        /// CSS 选择器句柄。
        selector: String,
        /// 属性名。
        name: String,
    },
    /// `element.toggleAttribute(name, force?)`——切换属性存在性。**决策在 apply 时**读当前存在性
    ///（force 覆盖），区别于 shim 读 stale snapshot 决定（连续 toggle 会都 add）。P1a attribute API。
    ToggleAttribute {
        /// CSS 选择器句柄。
        selector: String,
        /// 属性名。
        name: String,
        /// `Some(true)` 强制加、`Some(false)` 强制移除、`None` 切换（存在→移除/不存在→加）。
        force: Option<bool>,
    },
    /// `element.textContent = ...`
    SetText {
        /// CSS 选择器句柄。
        selector: String,
        /// 文本内容。
        text: String,
    },
    /// `element.innerHTML = ...`（解析 HTML 子树）。
    SetInnerHtml {
        /// CSS 选择器句柄。
        selector: String,
        /// HTML 片段。
        html: String,
    },
    /// `element.style.prop = ...`
    SetStyle {
        /// CSS 选择器句柄。
        selector: String,
        /// CSS 属性名（camelCase 或 kebab-case）。
        property: String,
        /// CSS 属性值。
        value: String,
    },
    /// `element.style.removeProperty(prop)` / `style.prop = ''` 真移除声明（区别于 [`SetStyle`] 空值
    /// 仍 push `prop: `——同 [`RemoveAttr`] 对布尔属性的修正）。P1a style 代理 API。
    RemoveStyle {
        /// CSS 选择器句柄。
        selector: String,
        /// CSS 属性名。
        property: String,
    },
    /// `element.remove()`
    Remove {
        /// CSS 选择器句柄。
        selector: String,
    },
    /// `document.createElement(tag)`
    CreateElement {
        /// 稳定句柄（`__n1` 等）。
        handle: String,
        /// 标签名。
        tag: String,
    },
    /// `document.createTextNode(text)`
    CreateTextNode {
        /// 稳定句柄。
        handle: String,
        /// 文本内容。
        text: String,
    },
    /// `parent.appendChild(child)` — 子节点用 create 时返回的句柄。
    AppendChild {
        /// 父节点选择器。
        parent_selector: String,
        /// 子节点句柄。
        child_handle: String,
    },
    /// `parentHandle.appendChild(child)` — 父节点亦为 create 句柄。
    AppendChildByHandle {
        /// 父节点句柄。
        parent_handle: String,
        /// 子节点句柄。
        child_handle: String,
    },
    /// `parent.insertBefore(child, ref)` — 父节点为选择器，参考节点为选择器。
    InsertBefore {
        /// 父节点选择器。
        parent_selector: String,
        /// 子节点句柄。
        child_handle: String,
        /// 参考节点选择器（新节点插入到它之前）。
        ref_selector: String,
    },
    /// `parentHandle.insertBefore(child, ref)` — 父节点为 create 句柄。
    InsertBeforeByHandle {
        /// 父节点句柄。
        parent_handle: String,
        /// 子节点句柄。
        child_handle: String,
        /// 参考节点选择器。
        ref_selector: String,
    },
    /// 对 create 句柄设置属性（append 前 `el.id = ...` 等）。
    SetAttrOnHandle {
        /// 节点句柄。
        handle: String,
        /// 属性名。
        name: String,
        /// 属性值。
        value: String,
    },
    /// 对 create 句柄设置 textContent。
    SetTextOnHandle {
        /// 节点句柄。
        handle: String,
        /// 文本内容。
        text: String,
    },
    /// 对 create 句柄设置 innerHTML。
    SetInnerHtmlOnHandle {
        /// 节点句柄。
        handle: String,
        /// HTML 片段。
        html: String,
    },
    /// 对 create 句柄设置 style 属性。
    SetStyleOnHandle {
        /// 节点句柄。
        handle: String,
        /// CSS 属性名。
        property: String,
        /// CSS 属性值。
        value: String,
    },
    /// 对 create 句柄真移除 style 声明（[`RemoveStyle`] 的 handle 版）。
    RemoveStyleOnHandle {
        /// 节点句柄。
        handle: String,
        /// CSS 属性名。
        property: String,
    },
    /// 按句柄移除节点。
    RemoveHandle {
        /// 节点句柄。
        handle: String,
    },
    /// `select.value = value`（P1a select）——编程选中匹配 value 的 option，deselect 同 select
    /// 内其他 option。区别于 click 触发的 change（需 UI），编程设值**不**自动派 change（匹配浏览器）。
    SelectOption {
        /// `<select>` 的选择器。
        selector: String,
        /// 要选中的 option 的 value（option 的 value 属性，无则 text content）。
        value: String,
    },
    /// `element.insertAdjacentHTML(position, text)`（P1a）——解析 HTML 片段并按 position
    /// 插入：`beforeend`（末子）/`afterbegin`（首子）/`beforebegin`（前兄弟）/`afterend`
    ///（后兄弟）。服务端原子完成（fragment parse + `copy_subtree_from` + parent 遍历），
    /// 区别于 [`Self::SetInnerHtml`]（整体替换子树）。
    InsertAdjacentHtml {
        /// 目标元素选择器。
        selector: String,
        /// 位置关键字（不区分大小写：beforebegin/afterbegin/beforeend/afterend）。
        position: String,
        /// 待解析的 HTML 片段。
        html: String,
    },
    /// `element.insertAdjacentText(position, text)`（P1a）——把文本作为**字面 Text 节点**
    ///（不解析 HTML）按 position 插入。区别于 [`Self::InsertAdjacentHtml`]（解析片段）。
    InsertAdjacentText {
        /// 目标元素选择器。
        selector: String,
        /// 位置关键字（不区分大小写：beforebegin/afterbegin/beforeend/afterend）。
        position: String,
        /// 文本内容（原样插入，不解析 `<` 等）。
        text: String,
    },
    /// `element.insertAdjacentElement(position, element)`（P1a）——把既有节点（create 句柄或
    /// 已挂载元素）按 position 移动插入。复用 [`Document::append_child`] 自动 reparent
    ///（移除旧 parent）实现「移动」语义。
    InsertAdjacentElement {
        /// 目标元素选择器。
        selector: String,
        /// 位置关键字（不区分大小写：beforebegin/afterbegin/beforeend/afterend）。
        position: String,
        /// 待插入节点的 create 句柄（`__n1` 等）。
        child_handle: String,
    },
    /// `element.outerHTML = html`（P1a，setter）——解析 HTML 片段，把目标元素**整体替换**为
    /// 片段顶层节点（插入到目标的父节点中目标位置，再移除目标自身）。区别于
    /// [`Self::SetInnerHtml`]（替换子树，保留目标自身）。
    SetOuterHtml {
        /// 目标元素选择器。
        selector: String,
        /// 替换用的 HTML 片段。
        html: String,
    },
    /// `document.createDocumentFragment()`（P1a）——创建 DocumentFragment 节点（nodeType 11，
    /// 轻量容器）。供批量构建后一次性插入（append 时 flatten 子节点到目标）。
    CreateDocumentFragment {
        /// 稳定句柄（`__n1` 等）。
        handle: String,
    },
    /// `parent.appendChild(fragment)`（P1a）——把 fragment 的**子节点移动**到 parent（flatten，
    /// fragment 自身不入树）。区别于 [`Self::AppendChild`]（append 节点自身）。父为选择器。
    AppendFragmentChildren {
        /// 父节点选择器。
        parent_selector: String,
        /// DocumentFragment 句柄。
        fragment_handle: String,
    },
    /// `parentHandle.appendChild(fragment)`——父为 create 句柄（detached）。
    AppendFragmentChildrenByHandle {
        /// 父节点句柄。
        parent_handle: String,
        /// DocumentFragment 句柄。
        fragment_handle: String,
    },
    /// `parent.insertBefore(fragment, ref)`（P1a）——fragment 子节点 flatten 到 parent 的
    /// ref 位置之前（区别于 [`Self::InsertBefore`] 插节点自身——会留 wrapper 且 fragment 不清空）。
    InsertFragmentBefore {
        /// 父节点选择器。
        parent_selector: String,
        /// DocumentFragment 句柄。
        fragment_handle: String,
        /// 参考节点选择器（fragment 子插到它之前）。
        ref_selector: String,
    },
    /// `parentHandle.insertBefore(fragment, ref)`——父为 create 句柄。
    InsertFragmentBeforeByHandle {
        /// 父节点句柄。
        parent_handle: String,
        /// DocumentFragment 句柄。
        fragment_handle: String,
        /// 参考节点选择器。
        ref_selector: String,
    },
}

/// 在文档根下按简单选择器查找第一个匹配元素。
pub fn find_by_selector(doc: &Document, selector: &str) -> Option<NodeId> {
    let root = doc.root();
    doc.query_selector(root, selector.trim())
}

/// 在文档根下查找所有匹配元素并生成稳定选择器列表。
pub fn find_all_selectors(doc: &Document, selector: &str) -> Vec<String> {
    let root = doc.root();
    doc.query_selector_all(root, selector.trim())
        .into_iter()
        .filter_map(|id| stable_selector_for_node(doc, id))
        .collect()
}

/// 为节点生成用于后续变更的稳定选择器（优先 `#id`）。
pub fn stable_selector_for_node(doc: &Document, node: NodeId) -> Option<String> {
    if let Some(id) = doc.get_attribute(node, "id") {
        let id = id.trim();
        if !id.is_empty() {
            return Some(format!("#{}", id));
        }
    }
    let tag = doc.get(node).and_then(|n| match &n.kind {
        NodeKind::Element(e) => Some(e.local_name().to_string()),
        _ => None,
    })?;
    if let Some(class) = doc.get_attribute(node, "class") {
        let first = class.split_whitespace().find(|c| !c.is_empty());
        if let Some(c) = first {
            return Some(format!("{}.{}", tag, c));
        }
    }
    Some(tag)
}

/// 为节点生成在文档中**唯一**的选择器（P1a gBCR path A handle-identity 用）。
///
/// 优先用 [`stable_selector_for_node`]（`#id` / `tag.class` / `tag`）——在文档中**唯一匹配**
/// 时返回（最短、最稳）。歧义时（多同 tag 无 id/class）回落到 **nth-child 结构路径**
/// （`html > body > div:nth-child(2) > …`，按 sibling 位置唯一定位，`:nth-child` 经
/// dom 选择器引擎解析+匹配）。两种形态都保证 `find_by_selector` 返回该节点本身——宁可结构路径
/// 冗长不错值。文本节点无 tag → `None`。
fn unique_selector_for_node(doc: &Document, node: NodeId) -> Option<String> {
    if let Some(sel) = stable_selector_for_node(doc, node) {
        let matches = doc.query_selector_all(doc.root(), sel.trim());
        if matches.len() == 1 {
            return Some(sel);
        }
    }
    structural_path_selector(doc, node)
}

/// 节点的元素父（跳过文本等非元素节点）。根元素（`<html>`）→ `None`。
fn element_parent(doc: &Document, node: NodeId) -> Option<NodeId> {
    let mut cur = doc.get(node)?.parent?;
    loop {
        let is_elem = doc.get(cur).is_some_and(|n| matches!(n.kind, NodeKind::Element(_)));
        if is_elem {
            return Some(cur);
        }
        cur = doc.get(cur)?.parent?;
    }
}

/// 节点在其元素父的**元素子**中的 1-based 序号（nth-child 位置）。
fn element_child_index(doc: &Document, parent: NodeId, node: NodeId) -> Option<usize> {
    let children = doc.get(parent)?.children.clone();
    let mut idx = 0usize;
    for sib in &children {
        if doc.get(*sib).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))) {
            idx += 1;
            if *sib == node {
                return Some(idx);
            }
        }
    }
    None
}

/// 构造 nth-child 结构路径选择器：node→root 每层 `tag:nth-child(pos)`，用 `>` 连接。
/// 始终唯一（每层 pin 一个具体 sibling 位置）。根元素（`<html>`）用 `:nth-child(1)`
/// （与 `compute_element_position` 对根置 child_index=1 一致）。
fn structural_path_selector(doc: &Document, node: NodeId) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut cur = node;
    loop {
        let nd = doc.get(cur)?;
        let tag = match &nd.kind {
            NodeKind::Element(e) => e.local_name().to_string(),
            _ => return None, // 文本节点无结构路径
        };
        match element_parent(doc, cur) {
            Some(parent) => {
                let pos = element_child_index(doc, parent, cur)?;
                segments.push(format!("{tag}:nth-child({pos})"));
                cur = parent;
            }
            None => {
                // 根元素（无元素父 = <html>）。
                segments.push(format!("{tag}:nth-child(1)"));
                break;
            }
        }
    }
    segments.reverse();
    Some(segments.join(" > "))
}

/// 将变更列表应用到文档（按顺序）。
///
/// 返回 `handle → 唯一稳定选择器` 映射（P1a gBCR path A handle-identity 基建）：遍历本次
/// 变更中 `CreateElement`/`CreateTextNode` 建立的 ephemeral handles，为每个元素算
/// [`unique_selector_for_node`]（歧义选择器跳过 → 不入 map → 该 handle 回落零 rect）。
/// 选择器反映**变更后**的文档状态（同 batch 内后置 SetAttrOnHandle 设的 id/class 已生效）。
pub fn apply_dom_mutations(doc: &mut Document, mutations: &[DomMutation]) -> Result<HashMap<String, String>, String> {
    let mut handles: HashMap<String, NodeId> = HashMap::new();

    for mutation in mutations {
        match mutation {
            DomMutation::SetAttr { selector, name, value } => {
                let node =
                    find_by_selector(doc, selector).ok_or_else(|| format!("set_attr: no match for {selector}"))?;
                doc.set_attribute(node, name, value);
            }
            DomMutation::RemoveAttr { selector, name } => {
                let node =
                    find_by_selector(doc, selector).ok_or_else(|| format!("remove_attr: no match for {selector}"))?;
                doc.remove_attribute(node, name);
            }
            DomMutation::ToggleAttribute { selector, name, force } => {
                // 决策在 apply 时（读当前存在性）——连续 toggle 复合正确（每次读 evolving state），
                // 不受脚本内 stale snapshot 影响（朴素 shim 实现连续 toggle 都 add 的 bug）。
                let node = find_by_selector(doc, selector)
                    .ok_or_else(|| format!("toggle_attribute: no match for {selector}"))?;
                let has = doc.get_attribute(node, name).is_some();
                let want = force.unwrap_or(!has);
                if want && !has {
                    doc.set_attribute(node, name, "");
                } else if !want && has {
                    doc.remove_attribute(node, name);
                }
            }
            DomMutation::SetText { selector, text } => {
                let node =
                    find_by_selector(doc, selector).ok_or_else(|| format!("set_text: no match for {selector}"))?;
                doc.set_text_content(node, text);
            }
            DomMutation::SetInnerHtml { selector, html } => {
                let node = find_by_selector(doc, selector)
                    .ok_or_else(|| format!("set_inner_html: no match for {selector}"))?;
                replace_inner_html(doc, node, html)?;
            }
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => {
                let node =
                    find_by_selector(doc, selector).ok_or_else(|| format!("set_style: no match for {selector}"))?;
                apply_style_property(doc, node, property, value);
            }
            DomMutation::RemoveStyle { selector, property } => {
                let node =
                    find_by_selector(doc, selector).ok_or_else(|| format!("remove_style: no match for {selector}"))?;
                apply_remove_style(doc, node, property);
            }
            DomMutation::Remove { selector } => {
                if let Some(node) = find_by_selector(doc, selector)
                    && let Some(parent) = doc.get(node).and_then(|n| n.parent)
                {
                    doc.remove_child(parent, node).map_err(|e| e.to_string())?;
                }
            }
            DomMutation::CreateElement { handle, tag } => {
                let id = doc.create_element(tag);
                handles.insert(handle.clone(), id);
            }
            DomMutation::CreateTextNode { handle, text } => {
                let id = doc.create_text_node(text);
                handles.insert(handle.clone(), id);
            }
            DomMutation::AppendChild {
                parent_selector,
                child_handle,
            } => {
                let parent = find_by_selector(doc, parent_selector)
                    .ok_or_else(|| format!("append_child: no parent match for {parent_selector}"))?;
                let child = handles
                    .get(child_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown child handle {child_handle}"))?;
                doc.append_child(parent, child).map_err(|e| e.to_string())?;
            }
            DomMutation::AppendChildByHandle {
                parent_handle,
                child_handle,
            } => {
                let parent = handles
                    .get(parent_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown parent handle {parent_handle}"))?;
                let child = handles
                    .get(child_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown child handle {child_handle}"))?;
                doc.append_child(parent, child).map_err(|e| e.to_string())?;
            }
            DomMutation::InsertBefore {
                parent_selector,
                child_handle,
                ref_selector,
            } => {
                let parent = find_by_selector(doc, parent_selector)
                    .ok_or_else(|| format!("insert_before: no parent match for {parent_selector}"))?;
                let child = handles
                    .get(child_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown child handle {child_handle}"))?;
                let ref_node = find_by_selector(doc, ref_selector)
                    .ok_or_else(|| format!("insert_before: no ref match for {ref_selector}"))?;
                doc.insert_before(parent, child, ref_node).map_err(|e| e.to_string())?;
            }
            DomMutation::InsertBeforeByHandle {
                parent_handle,
                child_handle,
                ref_selector,
            } => {
                let parent = handles
                    .get(parent_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown parent handle {parent_handle}"))?;
                let child = handles
                    .get(child_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown child handle {child_handle}"))?;
                let ref_node = find_by_selector(doc, ref_selector)
                    .ok_or_else(|| format!("insert_before: no ref match for {ref_selector}"))?;
                doc.insert_before(parent, child, ref_node).map_err(|e| e.to_string())?;
            }
            DomMutation::SetAttrOnHandle { handle, name, value } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                doc.set_attribute(node, name, value);
            }
            DomMutation::SetTextOnHandle { handle, text } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                doc.set_text_content(node, text);
            }
            DomMutation::SetInnerHtmlOnHandle { handle, html } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                replace_inner_html(doc, node, html)?;
            }
            DomMutation::SetStyleOnHandle {
                handle,
                property,
                value,
            } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                apply_style_property(doc, node, property, value);
            }
            DomMutation::RemoveStyleOnHandle { handle, property } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                apply_remove_style(doc, node, property);
            }
            DomMutation::RemoveHandle { handle } => {
                if let Some(node) = handles.get(handle).copied()
                    && let Some(parent) = doc.get(node).and_then(|n| n.parent)
                {
                    doc.remove_child(parent, node).map_err(|e| e.to_string())?;
                }
            }
            DomMutation::SelectOption { selector, value } => {
                // P1a select：编程设 select.value——mark 匹配 option selected，deselect 兄弟。
                let sel =
                    find_by_selector(doc, selector).ok_or_else(|| format!("select_option: no match for {selector}"))?;
                let options = doc.query_selector_all(sel, "option");
                let target = options
                    .iter()
                    .copied()
                    .find(|opt| option_value(doc, *opt).as_str() == value.as_str())
                    .ok_or_else(|| format!("select_option: no option with value {value}"))?;
                for opt in options {
                    if opt == target {
                        doc.set_attribute(opt, "selected", "");
                    } else {
                        doc.remove_attribute(opt, "selected");
                    }
                }
            }
            DomMutation::InsertAdjacentHtml {
                selector,
                position,
                html,
            } => {
                let node = find_by_selector(doc, selector)
                    .ok_or_else(|| format!("insert_adjacent_html: no match for {selector}"))?;
                insert_adjacent_html(doc, node, position, html)?;
            }
            DomMutation::InsertAdjacentText {
                selector,
                position,
                text,
            } => {
                let node = find_by_selector(doc, selector)
                    .ok_or_else(|| format!("insert_adjacent_text: no match for {selector}"))?;
                // 字面 Text 节点（不解析 HTML）。
                let tn = doc.create_text_node(text.as_str());
                insert_nodes_at_position(doc, &[tn], node, position)?;
            }
            DomMutation::InsertAdjacentElement {
                selector,
                position,
                child_handle,
            } => {
                let node = find_by_selector(doc, selector)
                    .ok_or_else(|| format!("insert_adjacent_element: no match for {selector}"))?;
                let child = handles
                    .get(child_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown child handle {child_handle}"))?;
                // 复用 append_child 的自动 reparent（child 已挂载则从旧 parent 移除 → 移动语义）。
                insert_nodes_at_position(doc, &[child], node, position)?;
            }
            DomMutation::SetOuterHtml { selector, html } => {
                replace_outer_html(doc, selector, html)?;
            }
            DomMutation::CreateDocumentFragment { handle } => {
                let id = doc.create_document_fragment();
                handles.insert(handle.clone(), id);
            }
            DomMutation::AppendFragmentChildren {
                parent_selector,
                fragment_handle,
            } => {
                let parent = find_by_selector(doc, parent_selector)
                    .ok_or_else(|| format!("append_fragment_children: no parent match for {parent_selector}"))?;
                move_fragment_children(doc, parent, fragment_handle, None, &handles)?;
            }
            DomMutation::AppendFragmentChildrenByHandle {
                parent_handle,
                fragment_handle,
            } => {
                let parent = handles
                    .get(parent_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown parent handle {parent_handle}"))?;
                move_fragment_children(doc, parent, fragment_handle, None, &handles)?;
            }
            DomMutation::InsertFragmentBefore {
                parent_selector,
                fragment_handle,
                ref_selector,
            } => {
                let parent = find_by_selector(doc, parent_selector)
                    .ok_or_else(|| format!("insert_fragment_before: no parent match for {parent_selector}"))?;
                let ref_node = find_by_selector(doc, ref_selector)
                    .ok_or_else(|| format!("insert_fragment_before: no ref match for {ref_selector}"))?;
                move_fragment_children(doc, parent, fragment_handle, Some(ref_node), &handles)?;
            }
            DomMutation::InsertFragmentBeforeByHandle {
                parent_handle,
                fragment_handle,
                ref_selector,
            } => {
                let parent = handles
                    .get(parent_handle)
                    .copied()
                    .ok_or_else(|| format!("unknown parent handle {parent_handle}"))?;
                let ref_node = find_by_selector(doc, ref_selector)
                    .ok_or_else(|| format!("insert_fragment_before: no ref match for {ref_selector}"))?;
                move_fragment_children(doc, parent, fragment_handle, Some(ref_node), &handles)?;
            }
        }
    }

    // P1a gBCR path A：为本次 batch 创建的每个 handle 算唯一稳定选择器（歧义者跳过）。
    // 供 RectBridge handler 解析 handle-identity（`__n{n}`）→ selector → NodeId → rect。
    let mut handle_selectors: HashMap<String, String> = HashMap::new();
    for (handle, node) in &handles {
        if let Some(sel) = unique_selector_for_node(doc, *node) {
            handle_selectors.insert(handle.clone(), sel);
        }
    }

    Ok(handle_selectors)
}

fn apply_style_property(doc: &mut Document, node: NodeId, property: &str, value: &str) {
    let current = doc.get_attribute(node, "style").unwrap_or_default();
    let merged = merge_style_property(&current, property, value);
    doc.set_attribute(node, "style", &merged);
}

fn merge_style_property(style: &str, property: &str, value: &str) -> String {
    let prop_key = property.trim().to_ascii_lowercase();
    let mut parts: Vec<String> = style
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    parts.retain(|p| {
        p.split(':')
            .next()
            .map(|k| k.trim().to_ascii_lowercase())
            .unwrap_or_default()
            != prop_key
    });
    parts.push(format!("{}: {}", property.trim(), value.trim()));
    parts.join("; ")
}

/// 真移除一条 style 声明（`removeProperty`）——滤除匹配属性名的段，不 push 空值
///（区别于 [`merge_style_property`] 对空值仍 push `prop: `）。
fn remove_style_property(style: &str, property: &str) -> String {
    let prop_key = property.trim().to_ascii_lowercase();
    style
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter(|p| {
            p.split(':')
                .next()
                .map(|k| k.trim().to_ascii_lowercase())
                .unwrap_or_default()
                != prop_key
        })
        .collect::<Vec<&str>>()
        .join("; ")
}

fn apply_remove_style(doc: &mut Document, node: NodeId, property: &str) {
    let current = doc.get_attribute(node, "style").unwrap_or_default();
    let removed = remove_style_property(&current, property);
    doc.set_attribute(node, "style", &removed);
}

fn replace_inner_html(doc: &mut Document, parent: NodeId, html: &str) -> Result<(), String> {
    let children: Vec<NodeId> = doc.get(parent).map(|n| n.children.clone()).unwrap_or_default();
    for child in children {
        doc.remove_child(parent, child).map_err(|e| e.to_string())?;
    }
    let trimmed = html.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if !trimmed.contains('<') {
        let text = doc.create_text_node(trimmed);
        doc.append_child(parent, text).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let frag_doc = parse_html(&format!("<!DOCTYPE html><html><body>{trimmed}</body></html>"));
    let body = find_by_selector(&frag_doc, "body").ok_or("innerHTML fragment parse failed")?;
    let frag_children: Vec<NodeId> = frag_doc.get(body).map(|n| n.children.clone()).unwrap_or_default();
    for frag_child in frag_children {
        let copied = copy_subtree_from(doc, &frag_doc, frag_child);
        doc.append_child(parent, copied).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn copy_subtree_from(doc: &mut Document, src_doc: &Document, src_id: NodeId) -> NodeId {
    let src_node = match src_doc.get(src_id) {
        Some(n) => n,
        None => return doc.create_text_node(""),
    };
    match &src_node.kind {
        NodeKind::Text(t) => doc.create_text_node(&t.content),
        NodeKind::Comment(c) => doc.create_comment(&c.content),
        NodeKind::Element(_) => {
            let tag = match src_doc.get(src_id) {
                Some(n) => match &n.kind {
                    NodeKind::Element(e) => e.local_name().to_string(),
                    _ => "span".to_string(),
                },
                None => "span".to_string(),
            };
            let new_id = doc.create_element(&tag);
            if let Some(NodeKind::Element(elem)) = src_doc.get(src_id).map(|n| &n.kind) {
                for attr in &elem.attributes {
                    let name = attr.name.local.to_string();
                    doc.set_attribute(new_id, &name, &attr.value);
                }
            }
            for &child in &src_node.children {
                let copied = copy_subtree_from(doc, src_doc, child);
                doc.append_child(new_id, copied).ok();
            }
            new_id
        }
        _ => doc.create_text_node(""),
    }
}

/// 把一组节点按 position 插入到目标元素 `target` 的相对位置（`beforeend`/`afterbegin`/
/// `beforebegin`/`afterend`）。供 [`insert_adjacent_html`]（fragment 节点）、
/// [`DomMutation::InsertAdjacentText`]（单 text 节点）、[`DomMutation::InsertAdjacentElement`]
///（单既有节点，复用 [`Document::append_child`] 自动 reparent 移动）共用。
///
/// 多节点保持顺序：beforeend/afterbegin 固定参考子（首子）一次、依次插入；beforebegin 固定
/// target 为 ref；afterend 固定 target 下一兄弟为 ref。`beforebegin`/`afterend` 需父节点；
/// 元素无父（文档根）返错（匹配 spec 对 detached/root 元素抛错）。
fn insert_nodes_at_position(
    doc: &mut Document,
    nodes: &[NodeId],
    target: NodeId,
    position: &str,
) -> Result<(), String> {
    // 展开 DocumentFragment → 其子节点（flatten，匹配 spec：insert fragment 等价 insert 其子并清空
    // fragment）。非 fragment 原样。这样 insertAdjacentElement 接 fragment（经 prepend/before/after）
    // 自动正确，不留 wrapper、fragment 清空。子节点在后续 append_child/insert_before 移动时自动
    // 从 fragment detach → fragment 变空。
    let mut flat: Vec<NodeId> = Vec::with_capacity(nodes.len());
    for &n in nodes {
        let is_frag = doc
            .get(n)
            .is_some_and(|nd| matches!(nd.kind, NodeKind::DocumentFragment));
        if is_frag {
            flat.extend(doc.get(n).map(|nd| nd.children.clone()).unwrap_or_default());
        } else {
            flat.push(n);
        }
    }
    if flat.is_empty() {
        return Ok(());
    }
    let nodes = &flat[..];
    let pos = position.trim().to_ascii_lowercase();
    match pos.as_str() {
        "beforeend" => {
            for &child in nodes {
                doc.append_child(target, child).map_err(|e| e.to_string())?;
            }
        }
        "afterbegin" => {
            // 插到 target 现有首子之前（保持插入序：依次插到固定首子前）；无子则 append。
            let first = doc.get(target).and_then(|n| n.children.first().copied());
            match first {
                Some(ref_node) => {
                    for &child in nodes {
                        doc.insert_before(target, child, ref_node).map_err(|e| e.to_string())?;
                    }
                }
                None => {
                    for &child in nodes {
                        doc.append_child(target, child).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        "beforebegin" => {
            let parent = doc
                .get(target)
                .and_then(|n| n.parent)
                .ok_or_else(|| format!("insertAdjacent {pos}: element has no parent"))?;
            for &child in nodes {
                doc.insert_before(parent, child, target).map_err(|e| e.to_string())?;
            }
        }
        "afterend" => {
            let parent = doc
                .get(target)
                .and_then(|n| n.parent)
                .ok_or_else(|| format!("insertAdjacent {pos}: element has no parent"))?;
            // 插到 target 下一兄弟之前（固定 ref，保持插入序）；末位无下一兄弟则 append。
            let parent_kids: Vec<NodeId> = doc.get(parent).map(|n| n.children.clone()).unwrap_or_default();
            let next = parent_kids
                .iter()
                .position(|c| *c == target)
                .and_then(|i| parent_kids.get(i + 1).copied());
            match next {
                Some(ref_node) => {
                    for &child in nodes {
                        doc.insert_before(parent, child, ref_node).map_err(|e| e.to_string())?;
                    }
                }
                None => {
                    for &child in nodes {
                        doc.append_child(parent, child).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        _ => return Err(format!("insertAdjacent: invalid position {position}")),
    }
    Ok(())
}

/// `element.insertAdjacentHTML(position, html)`——解析 HTML 片段，深拷贝其顶层节点，
/// 按 position 插入到目标元素相对位置（`beforeend`/`afterbegin`/`beforebegin`/`afterend`）。
/// 复用 [`replace_inner_html`] 的 fragment parse 思路 + [`copy_subtree_from`]，区别在于
/// **不替换**现有子树、仅增量插入（保持既有子节点身份不变）。供
/// [`DomMutation::InsertAdjacentHtml`] 应用。
///
/// `beforebegin`/`afterend` 需父节点；元素无父（文档根）返错（匹配 spec 对 detached/root
/// 元素抛错）。插入顺序保持 fragment 顶层节点顺序。
fn insert_adjacent_html(doc: &mut Document, node: NodeId, position: &str, html: &str) -> Result<(), String> {
    let trimmed = html.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    // 解析顶层 fragment 节点：包入 <body> 解析取 body 子（与 replace_inner_html 同源）。
    // 先全部 copy 收集，再按 position 插入（原子化，单一出口）。
    let frag_nodes: Vec<NodeId> = if !trimmed.contains('<') {
        vec![doc.create_text_node(trimmed)]
    } else {
        let frag_doc = parse_html(&format!("<!DOCTYPE html><html><body>{trimmed}</body></html>"));
        let body = find_by_selector(&frag_doc, "body").ok_or("insertAdjacentHTML fragment parse failed")?;
        let kids: Vec<NodeId> = frag_doc.get(body).map(|n| n.children.clone()).unwrap_or_default();
        kids.into_iter().map(|k| copy_subtree_from(doc, &frag_doc, k)).collect()
    };
    insert_nodes_at_position(doc, &frag_nodes, node, position)
}

/// `element.outerHTML = html`（setter）——解析 HTML 片段，把目标元素**整体替换**为片段
/// 顶层节点：在目标的父节点中、目标位置之前逐个插入片段节点，再移除目标自身。供
/// [`DomMutation::SetOuterHtml`] 应用。复用 [`replace_inner_html`] 的 fragment parse 思路 +
/// [`copy_subtree_from`]。
///
/// 目标需有父节点（文档根无父 → 返错，匹配 spec 对根元素 outerHTML 赋值抛错）。空片段 →
/// 仅移除目标（spec：`el.outerHTML = ''` 移除元素）。
fn replace_outer_html(doc: &mut Document, selector: &str, html: &str) -> Result<(), String> {
    let node = find_by_selector(doc, selector).ok_or_else(|| format!("set_outer_html: no match for {selector}"))?;
    let parent = doc
        .get(node)
        .and_then(|n| n.parent)
        .ok_or_else(|| format!("set_outer_html: element {selector} has no parent"))?;
    // 解析顶层 fragment 节点（与 replace_inner_html 同源），逐个插到目标之前。
    let trimmed = html.trim();
    if !trimmed.is_empty() {
        if !trimmed.contains('<') {
            let t = doc.create_text_node(trimmed);
            doc.insert_before(parent, t, node).map_err(|e| e.to_string())?;
        } else {
            let frag_doc = parse_html(&format!("<!DOCTYPE html><html><body>{trimmed}</body></html>"));
            let body = find_by_selector(&frag_doc, "body").ok_or("outerHTML fragment parse failed")?;
            let kids: Vec<NodeId> = frag_doc.get(body).map(|n| n.children.clone()).unwrap_or_default();
            for k in kids {
                let copied = copy_subtree_from(doc, &frag_doc, k);
                doc.insert_before(parent, copied, node).map_err(|e| e.to_string())?;
            }
        }
    }
    // 移除目标自身（整体替换）。
    doc.remove_child(parent, node).map_err(|e| e.to_string())?;
    Ok(())
}

/// DocumentFragment 的 flatten 语义：把 fragment 的**子节点逐个移动**到 parent——
/// `ref == None` 时 append（末子），`ref == Some(r)` 时 insert-before r。
/// fragment 自身不入树（`append_child`/`insert_before` 自动从 fragment detach 子节点 → fragment 变空，
/// 匹配 spec「insert 后 fragment 清空」）。供 [`DomMutation::AppendFragmentChildren`] /
/// [`DomMutation::InsertFragmentBefore`] 等应用。
fn move_fragment_children(
    doc: &mut Document,
    parent: NodeId,
    fragment_handle: &str,
    ref_node: Option<NodeId>,
    handles: &HashMap<String, NodeId>,
) -> Result<(), String> {
    let fragment = handles
        .get(fragment_handle)
        .copied()
        .ok_or_else(|| format!("unknown fragment handle {fragment_handle}"))?;
    // 快照 fragment 子列表（移动过程中会从 fragment 移除，故先收集）。
    let kids: Vec<NodeId> = doc.get(fragment).map(|n| n.children.clone()).unwrap_or_default();
    for child in kids {
        match ref_node {
            Some(r) => doc.insert_before(parent, child, r).map_err(|e| e.to_string())?,
            None => doc.append_child(parent, child).map_err(|e| e.to_string())?,
        };
    }
    Ok(())
}

/// 解析 HTML、应用变更并返回序列化后的 HTML。
///
/// 丢弃 `apply_dom_mutations` 产出的 handle→selector 映射（无 handle-identity 需求的路径用）。
/// 需要 handle map 的生产路径（renderer/browser 应用变更后回填 RectBridge）用
/// [`apply_mutations_to_html_with_handles`]。
pub fn apply_mutations_to_html(html: &str, mutations: &[DomMutation]) -> Result<String, String> {
    let mut doc = parse_html(html);
    let _handle_selectors = apply_dom_mutations(&mut doc, mutations)?;
    Ok(doc.outer_html(doc.root()))
}

/// 同 [`apply_mutations_to_html`]，但额外返回 handle→唯一稳定选择器映射（P1a gBCR path A）。
///
/// 生产 apply 路径（renderer `apply_recorded_mutations`、browser mirror）调本函数，把返回的
/// map merge 进 worker 持久的 [`crate::rect_bridge::HandleSelectorMap`]，供 RectBridge handler
/// 解析 handle-identity。reftest/测试路径无此需求，继续用 [`apply_mutations_to_html`]。
pub fn apply_mutations_to_html_with_handles(
    html: &str,
    mutations: &[DomMutation],
) -> Result<(String, HashMap<String, String>), String> {
    let mut doc = parse_html(html);
    let handle_selectors = apply_dom_mutations(&mut doc, mutations)?;
    Ok((doc.outer_html(doc.root()), handle_selectors))
}

/// 从 HTML 快照查询首个匹配元素的**唯一**选择器（供 `__zw_query_match`→querySelector）。
///
/// 用 [`unique_selector_for_node`]（`#id`/`tag.class`/`tag` 唯一时返回；歧义时 nth-child 结构
/// 路径回落）——保证返回的 selector 在 dom_html 中**唯一定位**该元素。querySelector 对无 id/class
/// 的歧义元素（`<option>`/`<li>` 等）此前返回 `stable_selector`（如 "option"，多 option 时指向首个），
/// 导致后续 `el.selected`/`el.value` 读错元素；唯一选择器修复之。同一 dom_html 上与旧实现解析到同一元素。
pub fn query_match_selector(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .and_then(|n| unique_selector_for_node(&doc, n))
        .unwrap_or_default()
}

/// 从 HTML 快照查询全部匹配元素的**唯一**选择器（`|` 分隔，供 `__zw_query_all`→querySelectorAll）。
///
/// 用 [`unique_selector_for_node`]（`#id`/`tag.class`/`tag` 唯一时返回；歧义时 nth-child 结构路径
/// 回落）——每个元素返在 dom_html 中**唯一定位**它的选择器。此前用 [`find_all_selectors`]（返
/// `stable_selector`），对无 id/class 的歧义集合（`querySelectorAll('option')`/`'li'` 等）每个元素
/// 都返如 "option"，N 个 proxy 全指向首个 option → 读全错；唯一选择器修复之（与 R2663 的
/// `query_match_selector` 单查询对称）。
pub fn query_all_selector_list(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let root = doc.root();
    doc.query_selector_all(root, selector.trim())
        .into_iter()
        .filter_map(|id| unique_selector_for_node(&doc, id))
        .collect::<Vec<_>>()
        .join("|")
}

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
        .filter_map(|id| unique_selector_for_node(&doc, id))
        .collect::<Vec<_>>()
        .join("|")
}

/// 元素的**元素子**（跳过文本/注释）唯一选择器，`|` 分隔；无返空串。供 `__zw_element_children`
/// 回调 → shim `el.children` / `firstElementChild` / `lastElementChild` / `childElementCount`。
pub fn element_children_selectors(html: &str, elem_sel: &str) -> String {
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, elem_sel) else {
        return String::new();
    };
    doc.child_nodes(node)
        .into_iter()
        .filter(|c| doc.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
        .filter_map(|c| unique_selector_for_node(&doc, c))
        .collect::<Vec<_>>()
        .join("|")
}

/// 元素的**前/后元素兄弟**唯一选择器，`prev|next` 格式（空字段 = 无该方向兄弟）；元素无父或
/// elem_sel 不解析返 `|`（两空）。供 `__zw_element_siblings` 回调 → shim `previousElementSibling`
/// / `nextElementSibling`。
pub fn element_sibling_selectors(html: &str, elem_sel: &str) -> String {
    let empty = || String::from("|");
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, elem_sel) else {
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
        unique_selector_for_node(&doc, sibs[idx - 1]).unwrap_or_default()
    } else {
        String::new()
    };
    let next = if idx + 1 < sibs.len() {
        unique_selector_for_node(&doc, sibs[idx + 1]).unwrap_or_default()
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
    let Some(node) = find_by_selector(&doc, elem_sel) else {
        return String::new();
    };
    element_parent(&doc, node)
        .and_then(|p| unique_selector_for_node(&doc, p))
        .unwrap_or_default()
}

/// JSON 字符串字面量（转义 `"`、`\`、控制字符）。供 [`child_nodes_json`] /
/// [`sibling_nodes_json`] 序列化文本/注释节点内容（文本可含任意字符，`|` 分隔不安全，故用 JSON）。
fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 单个节点的 JSON 条目：元素 `{"k":"E","s":<selector>}`、文本 `{"k":"T","v":<text>}`、
/// 注释 `{"k":"C","v":<text>}`；其他类型（Doctype 等）跳过返 None。
fn node_entry_json(doc: &Document, id: NodeId) -> Option<String> {
    let node = doc.get(id)?;
    Some(match &node.kind {
        NodeKind::Element(_) => {
            let sel = unique_selector_for_node(doc, id)?;
            format!("{{\"k\":\"E\",\"s\":{}}}", json_str(&sel))
        }
        NodeKind::Text(t) => format!("{{\"k\":\"T\",\"v\":{}}}", json_str(&t.content)),
        NodeKind::Comment(c) => format!("{{\"k\":\"C\",\"v\":{}}}", json_str(&c.content)),
        _ => return None,
    })
}

/// 元素的**全部子节点**（含文本/注释，区别于 [`element_children_selectors`] 仅元素子），JSON 数组。
/// 供 `__zw_child_nodes` 回调 → shim `el.childNodes` / `firstChild` / `lastChild`。
pub fn child_nodes_json(html: &str, elem_sel: &str) -> String {
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, elem_sel) else {
        return "[]".to_string();
    };
    let entries: Vec<String> = doc
        .child_nodes(node)
        .into_iter()
        .filter_map(|c| node_entry_json(&doc, c))
        .collect();
    format!("[{}]", entries.join(","))
}

/// 元素的**前/后节点兄弟**（含文本/注释，区别于 [`element_sibling_selectors`] 仅元素兄弟），
/// JSON `{"p":<entry|null>,"n":<entry|null>}`。供 `__zw_sibling_nodes` 回调 → shim
/// `previousSibling` / `nextSibling`。
pub fn sibling_nodes_json(html: &str, elem_sel: &str) -> String {
    let empty = "{\"p\":null,\"n\":null}".to_string();
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, elem_sel) else {
        return empty;
    };
    let Some(parent) = doc.parent_node(node) else {
        return empty;
    };
    let sibs: Vec<NodeId> = doc.child_nodes(parent);
    let Some(idx) = sibs.iter().position(|s| *s == node) else {
        return empty;
    };
    let prev = if idx > 0 {
        node_entry_json(&doc, sibs[idx - 1])
    } else {
        None
    };
    let next = if idx + 1 < sibs.len() {
        node_entry_json(&doc, sibs[idx + 1])
    } else {
        None
    };
    format!(
        "{{\"p\":{},\"n\":{}}}",
        prev.unwrap_or_else(|| "null".into()),
        next.unwrap_or_else(|| "null".into())
    )
}

/// `container.contains(other)`——other 是 container 的后代或 container 自身（沿 other 的
/// parent_node 链）。供 `__zw_contains` 回调 → shim `el.contains(other)`。
pub fn element_contains(html: &str, container_sel: &str, other_sel: &str) -> bool {
    let doc = parse_html(html);
    let Some(container) = find_by_selector(&doc, container_sel) else {
        return false;
    };
    let Some(mut cur) = find_by_selector(&doc, other_sel) else {
        return false;
    };
    loop {
        if cur == container {
            return true;
        }
        match doc.parent_node(cur) {
            Some(p) if p != cur => cur = p,
            _ => return false,
        }
    }
}

/// 收集文档中所有元素的 `id` 属性值（去重、保序，首次出现优先——与
/// `getElementById` 取首个匹配语义一致）。供 `__zw_collect_ids` 回调实现
/// HTML 规范「Window 上的命名属性访问」（`<div id="x">` → 全局 `x`）。
pub fn collect_element_ids(html: &str) -> String {
    let doc = parse_html(html);
    let root = doc.root();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for node in doc.query_selector_all(root, "[id]") {
        if let Some(val) = doc.get_attribute(node, "id") {
            let v = val.trim();
            if !v.is_empty() && seen.insert(v.to_string()) {
                out.push(v.to_string());
            }
        }
    }
    out.join("|")
}

/// 从当前 HTML 快照查询属性（供 `__zw_get_attr` 回调只读使用）。
pub fn query_attr_from_html(html: &str, selector: &str, name: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .and_then(|n| doc.get_attribute(n, name))
        .unwrap_or_default()
}

/// 从当前 HTML 快照查询元素 tag 名（供 `__zw_get_tag` 回调；shim `_tagFromSel` 对 id-only
/// 选择器等只能启发式猜测，P1a form input 需真实 tag 判 INPUT/TEXTAREA）。
pub fn query_tag_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .and_then(|n| {
            doc.get(n).and_then(|node| match &node.kind {
                NodeKind::Element(e) => Some(e.local_name().to_string()),
                _ => None,
            })
        })
        .unwrap_or_default()
}

/// P1a form submit：从元素 selector 沿 DOM 父链（`parent_node`）找 enclosing `<form>` 的
/// stable selector（无 enclosing form → None）。供 Enter-in-input / submit-button 的 submit 派发。
pub fn enclosing_form_selector(html: &str, elem_sel: &str) -> Option<String> {
    let doc = parse_html(html);
    let mut node = find_by_selector(&doc, elem_sel)?;
    loop {
        if let Some(nd) = doc.get(node)
            && let NodeKind::Element(e) = &nd.kind
            && e.local_name().eq_ignore_ascii_case("form")
        {
            return stable_selector_for_node(&doc, node);
        }
        node = doc.parent_node(node)?;
    }
}

/// P1a form submit：判定元素是否为 submit button（点击会提交 enclosing form）。
/// `<input type=submit>` / `<input type=image>` / `<button>`（type 非 "button"——默认 submit）。
pub fn is_submit_button(html: &str, elem_sel: &str) -> bool {
    let tag = query_tag_from_html(html, elem_sel);
    let ty = query_attr_from_html(html, elem_sel, "type").to_ascii_lowercase();
    if tag.eq_ignore_ascii_case("input") {
        return ty == "submit" || ty == "image";
    }
    if tag.eq_ignore_ascii_case("button") {
        // type="button" 不提交；type=submit/空/missing → 提交（button 默认 type=submit）。
        return ty != "button";
    }
    false
}

/// P1a form control：判定元素是否有某属性（boolean 属性如 `checked`/`disabled` 靠存在性，
/// `getAttribute` 返空串无法区分存在与空值，故供 `__zw_has_attr` / checkbox toggle）。
pub fn has_attribute(html: &str, selector: &str, name: &str) -> bool {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.get_attribute(n, name).is_some())
        .unwrap_or(false)
}

/// 元素的全部属性本地名（`|` 分隔，文档顺序）；无属性或元素不解析返空串。供 `__zw_attr_names`
/// 回调 → shim `el.dataset` 枚举（ownKeys）等需要遍历属性名的场景。
pub fn element_attribute_names(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, selector) else {
        return String::new();
    };
    doc.get(node)
        .and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.attribute_names().join("|")),
            _ => None,
        })
        .unwrap_or_default()
}

/// P1a checkbox：判定元素是否为 `<input type=checkbox>`。
pub fn is_checkbox(html: &str, selector: &str) -> bool {
    query_tag_from_html(html, selector).eq_ignore_ascii_case("input")
        && query_attr_from_html(html, selector, "type").eq_ignore_ascii_case("checkbox")
}

/// P1a radio：判定元素是否为 `<input type=radio>`。
pub fn is_radio(html: &str, selector: &str) -> bool {
    query_tag_from_html(html, selector).eq_ignore_ascii_case("input")
        && query_attr_from_html(html, selector, "type").eq_ignore_ascii_case("radio")
}

/// P1a select：判定元素是否为 `<select>`。
pub fn is_select(html: &str, selector: &str) -> bool {
    query_tag_from_html(html, selector).eq_ignore_ascii_case("select")
}

/// P1a select：读 `<select>` 当前选中 option 的 value（HTML spec 语义）。
///
/// 选中 option = 首个（树序）带 `selected` 属性的 `<option>`；无则首个 option（默认选中）。
/// option value = `value` 属性，无则其 text content（trim）。无 option → 空串。
pub fn select_value_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let Some(sel) = find_by_selector(&doc, selector) else {
        return String::new();
    };
    selected_option(&doc, sel)
        .map(|(opt, _)| option_value(&doc, opt))
        .unwrap_or_default()
}

/// P1a select：读 `<select>` 选中 option 的索引（首个 `selected` option；无则 0；无 option → -1）。
pub fn select_index_from_html(html: &str, selector: &str) -> i32 {
    let doc = parse_html(html);
    let Some(sel) = find_by_selector(&doc, selector) else {
        return -1;
    };
    selected_option(&doc, sel).map(|(_, idx)| idx as i32).unwrap_or(-1)
}

/// 返回 select 的选中 option（节点, 0-based 索引）：首个带 `selected` 属性者，无则首个 option。
fn selected_option(doc: &Document, select: NodeId) -> Option<(NodeId, usize)> {
    let options = doc.query_selector_all(select, "option");
    let first = options.first().copied();
    for (idx, opt) in options.iter().enumerate() {
        if doc.get_attribute(*opt, "selected").is_some() {
            return Some((*opt, idx));
        }
    }
    first.map(|o| (o, 0))
}

/// option 的 value：`value` 属性，无则 text content（trim）。
fn option_value(doc: &Document, opt: NodeId) -> String {
    if let Some(v) = doc.get_attribute(opt, "value") {
        return v;
    }
    doc.text_content(opt).map(|t| t.trim().to_string()).unwrap_or_default()
}

/// P1a select：编程设 `select.value = value`——mark 匹配 value 的 option 为 `selected`，
/// deselect 同 select 内其他 option（HTML spec：单选 select 仅一 option 可选中）。
/// 匹配规则：option 的 [`option_value`] == `value`（value 属性优先，无则 text content）。
/// 无匹配 option → 不改（保持当前选中）。返回新 HTML（非 select / 未命中 → None）。
pub fn set_selected_option_html(html: &str, selector: &str, value: &str) -> Option<String> {
    let mut doc = parse_html(html);
    let sel = find_by_selector(&doc, selector)?;
    let is_sel = doc
        .get(sel)
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("select")));
    if !is_sel {
        return None;
    }
    let options = doc.query_selector_all(sel, "option");
    let mut matched: Option<NodeId> = None;
    for opt in &options {
        if option_value(&doc, *opt) == value {
            matched = Some(*opt);
            break;
        }
    }
    let target = matched?;
    for opt in &options {
        if *opt == target {
            doc.set_attribute(*opt, "selected", "");
        } else {
            doc.remove_attribute(*opt, "selected");
        }
    }
    Some(doc.outer_html(doc.root()))
}

/// P1a radio：toggle `<input type=radio>`——set `checked` on target + `remove_attribute(checked)`
/// on 同 `name` 组 radio 兄弟（直接操作 Document by NodeId，避免兄弟缺 id 时 selector 歧义）。
/// 无 `name` 属性 → 仅 set target（无组）。返回新 HTML（非 radio / 未命中 → None）。
pub fn toggle_radio_html(html: &str, selector: &str) -> Option<String> {
    let mut doc = parse_html(html);
    let target = find_by_selector(&doc, selector)?;
    let is_rd = doc
        .get_attribute(target, "type")
        .map(|t| t.eq_ignore_ascii_case("radio"))
        .unwrap_or(false)
        && doc
            .get(target)
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("input")))
            .unwrap_or(false);
    if !is_rd {
        return None;
    }
    doc.set_attribute(target, "checked", "");
    if let Some(name) = doc.get_attribute(target, "name") {
        let root = doc.root();
        for n in doc.query_selector_all(root, "input[type=radio]") {
            if n != target && doc.get_attribute(n, "name").as_deref() == Some(name.as_str()) {
                doc.remove_attribute(n, "checked");
            }
        }
    }
    Some(doc.outer_html(doc.root()))
}

/// P1a change-on-blur：判定元素是否为「失焦派发 change」的文本输入（`<textarea>` 或
/// `<input>` 非 action 类型——text/email/password/search/number 等文本类；checkbox/radio/
/// button/submit/image/reset 的 change 在 click 时派发，不在此列）。
pub fn is_text_input(html: &str, selector: &str) -> bool {
    let tag = query_tag_from_html(html, selector).to_ascii_lowercase();
    if tag == "textarea" {
        return true;
    }
    if tag == "input" {
        let ty = query_attr_from_html(html, selector, "type").to_ascii_lowercase();
        return !matches!(
            ty.as_str(),
            "checkbox" | "radio" | "button" | "submit" | "image" | "reset"
        );
    }
    false
}

/// P1a Tab 焦点导航：经 dom `FocusManager`（tabindex 排序：正值升序在前，0/默认在文档序在后）
/// 算下一/上一可聚焦元素的 stable selector。`current_sel` 为当前焦点（None → 首/末）。无 focusable → None。
pub fn next_focus_selector(html: &str, current_sel: Option<&str>, forward: bool) -> Option<String> {
    let doc = parse_html(html);
    let mut fm = FocusManager::new();
    fm.scan(&doc);
    if let Some(sel) = current_sel
        && let Some(n) = find_by_selector(&doc, sel)
    {
        fm.set_focus(Some(n));
    }
    let next = if forward { fm.focus_next() } else { fm.focus_previous() }?;
    stable_selector_for_node(&doc, next)
}

/// 从当前 HTML 快照查询 innerHTML。
pub fn query_inner_html_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.inner_html(n))
        .unwrap_or_default()
}

/// 从 HTML 快照查询元素的 outerHTML（含自身 tag/属性 + 子树序列化）。供 `__zw_get_outer_html`
/// 回调 → shim `el.outerHTML`（getter）。
pub fn query_outer_html_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.outer_html(n))
        .unwrap_or_default()
}

/// 从已记录变更中查询 create 句柄上的 innerHTML。
pub fn query_inner_html_from_mutations(mutations: &[DomMutation], handle: &str) -> String {
    for m in mutations.iter().rev() {
        if let DomMutation::SetInnerHtmlOnHandle { handle: h, html } = m
            && h == handle
        {
            return html.clone();
        }
        if let DomMutation::CreateTextNode { handle: h, text } = m
            && h == handle
        {
            return text.clone();
        }
        if let DomMutation::SetTextOnHandle { handle: h, text } = m
            && h == handle
        {
            return text.clone();
        }
    }
    String::new()
}

/// 键盘等 DOM 事件的附加字段（传给 JS `KeyboardEvent`）。
#[derive(Debug, Clone, Default)]
pub struct DomEventDetail {
    /// `KeyboardEvent.key`
    pub key: Option<String>,
    /// `KeyboardEvent.code`
    pub code: Option<String>,
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// 生成在 V8 中派发 DOM 事件的脚本片段。
pub fn script_dispatch_dom_event(selector: &str, event_type: &str, detail: Option<&DomEventDetail>) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ty = escape_js_string(event_type);
    let detail_json = match detail {
        None => "null".to_string(),
        Some(d) => {
            let key = d
                .key
                .as_deref()
                .map(|k| format!("'{}'", escape_js_string(k)))
                .unwrap_or_else(|| "null".to_string());
            let code = d
                .code
                .as_deref()
                .map(|c| format!("'{}'", escape_js_string(c)))
                .unwrap_or_else(|| "null".to_string());
            format!("{{key:{key},code:{code}}}")
        }
    };
    format!("__zw_dispatch_event('{esc_sel}', '{esc_ty}', {detail_json})")
}

/// 构造「向焦点 input/textarea 注入一个文本字符」的 shim 脚本（P1a form input）。
/// 宿主在 keydown 可打印字符时执行：shim `__zw_text_input(sel, ch)` 把字符 append 到 value
/// （`.value` set 更新缓存 + 记 value 属性 mutation）并派发 'input' 事件。非 input/textarea → no-op。
pub fn script_text_input(selector: &str, key: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ch = escape_js_string(key);
    format!("__zw_text_input('{esc_sel}', '{esc_ch}')")
}

/// 构造「Backspace 删末字符」的 shim 脚本（P1a form input 编辑互补）。宿主在 keydown
/// Backspace 时执行：shim `__zw_text_delete(sel)` 删 value 末字符并派发 'input' 事件。
pub fn script_text_delete(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_text_delete('{esc_sel}')")
}

/// 从当前 HTML 快照查询 textContent。
pub fn query_text_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.inner_html(n))
        .unwrap_or_default()
}

/// 从已记录变更中查询 create 句柄上的属性（脚本执行期间只读）。
pub fn query_attr_from_mutations(mutations: &[DomMutation], handle: &str, name: &str) -> String {
    for m in mutations.iter().rev() {
        if let DomMutation::SetAttrOnHandle {
            handle: h,
            name: n,
            value: v,
        } = m
            && h == handle
            && n == name
        {
            return v.clone();
        }
    }
    String::new()
}

/// 从已记录变更中查询 create 句柄上的 textContent。
pub fn query_text_from_mutations(mutations: &[DomMutation], handle: &str) -> String {
    for m in mutations.iter().rev() {
        if let DomMutation::CreateTextNode { handle: h, text } = m
            && h == handle
        {
            return text.clone();
        }
        if let DomMutation::SetTextOnHandle { handle: h, text } = m
            && h == handle
        {
            return text.clone();
        }
    }
    String::new()
}

/// 从已记录变更中查询 create 句柄元素的 tag 名（供 `__zw_get_tag_handle` 回调）。
///
/// detached `createElement(tag)` 元素无 selector，shim `_tagFromSel` 无法猜其真实 tag
/// （恒返 `DIV`）。本函数从 [`DomMutation::CreateElement`] 记录的 `tag` 取真实值，
/// 使 `document.createElement('span').tagName` === `'SPAN'`。句柄未被 createElement
/// 记录（如 DocumentFragment）→ 空串（shim fallback）。
pub fn query_tag_from_mutations(mutations: &[DomMutation], handle: &str) -> String {
    for m in mutations.iter().rev() {
        if let DomMutation::CreateElement { handle: h, tag } = m
            && h == handle
        {
            return tag.clone();
        }
    }
    String::new()
}

/// 向 V8 sandbox 注册全部 `__zw_*` DOM 桥接回调。
///
/// 将 [`generate_js_dom_shim`] 产生的 JS shim 与宿主侧 [`DomMutation`] 收集器连接：
/// JS 侧 `document.querySelector`/`setAttribute`/`createElement` 等操作经
/// `__zw_*` 扁平回调翻译为 `DomMutation`，推入共享 `mutations` 向量；查询类回调
/// （`__zw_get_attr`/`__zw_get_text`/`__zw_query_*`）则从 `dom_html` 快照读取。
///
/// `dom_html` / `page_url` 用 `Arc<Mutex<String>>` 共享，使宿主能在脚本执行前
/// 经 [`V8Sandbox::execute`] 切换快照（与 browser/renderer/reftest 三处共用一致语义）。
///
/// 该函数从 renderer/browser 两个 JS worker 中抽取为共享实现，避免第三份拷贝
/// （reftest harness 也复用，见 `tests/wpt-runner`）。
pub fn register_dom_callbacks(
    sandbox: &mut dyn Sandbox,
    mutations: &Arc<std::sync::Mutex<Vec<DomMutation>>>,
    dom_html: &Arc<std::sync::Mutex<String>>,
    page_url: &Arc<std::sync::Mutex<String>>,
) {
    let counter = Arc::new(AtomicU64::new(0));

    let url = Arc::clone(page_url);
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(move |_args| url.lock().unwrap_or_else(|e| e.into_inner()).clone()),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_match",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_match_selector(&snap, &sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_all",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_all_selector_list(&snap, &sel)
        }),
    );

    // `element.matches(selector)` / `element.closest(selector)`——元素查询 API（直接消费选择器引擎，
    // 含组合器）。elem_sel = 元素唯一选择器（proxy 持有），test_sel = 待测选择器。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_matches",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let test_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if element_matches_test_selector(&snap, &elem_sel, &test_sel) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_closest",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let test_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            closest_matching_selector(&snap, &elem_sel, &test_sel)
        }),
    );

    // `element.querySelector(selector)` / `element.querySelectorAll(selector)`——元素**子树**作用域
    // （spec：仅后代，不含元素自身）。elem_sel = 元素唯一选择器，区别于文档作用域的 query_match/all。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_match_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_match_in_subtree(&snap, &elem_sel, &sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_all_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_all_in_subtree(&snap, &elem_sel, &sel)
        }),
    );

    // 元素遍历/导航 API：children/firstElementChild/lastElementChild/childElementCount（子列表）、
    // previousElementSibling/nextElementSibling（兄弟对）、contains（后代判定）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_element_children",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_children_selectors(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_element_siblings",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_sibling_selectors(&snap, &elem_sel)
        }),
    );

    // 节点级遍历 API（含文本/注释节点）：childNodes/firstChild/lastChild（子列表）、
    // previousSibling/nextSibling（兄弟对）。JSON 序列化（文本内容含任意字符）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_child_nodes",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            child_nodes_json(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_sibling_nodes",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            sibling_nodes_json(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_contains",
        Box::new(move |args| {
            let container_sel = args.first().map(String::from).unwrap_or_default();
            let other_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if element_contains(&snap, &container_sel, &other_sel) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // `element.parentNode` / `parentElement`——元素父唯一选择器（修正旧 stub 恒返 body）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_parent",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            parent_selector_for(&snap, &elem_sel)
        }),
    );

    // HTML 规范「Window 上的命名属性访问」：所有带 id 的元素作为全局变量可访问
    // （`<div id="container">` → JS 裸标识符 `container`）。shim 据此在脚本执行前
    // 安装 `globalThis[id] = getElementById(id)`（仅合法标识符、不覆盖已存在全局）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_collect_ids",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            collect_element_ids(&snap)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_attr_from_html(&snap, &args[0], &args[1])
        }),
    );

    // P1a form input：真实 tag 名查询（shim `_tagFromSel` 对 id-only 选择器等仅启发式猜测，
    // `__zw_text_input` 需真实 tag 判 INPUT/TEXTAREA）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_tag",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_tag_from_html(&snap, &sel)
        }),
    );

    // `getComputedStyle(el).getPropertyValue(prop)`——计算样式（display/position/visibility/
    // opacity + 颜色族）。**per-snapshot 缓存**：html_key → (selector → ComputedStyle)。Document 非
    // Send（含 observer/listener 闭包 + html5ever tendril `Cell`），不能入 `Send + Sync` 闭包；故只
    // 缓存 `ComputedStyle`（纯值类型，Send）。同 html 同 selector 命中 → 仅 serialize（O(1)）；新
    // selector → parse+cascade 一次并存入——同一元素的多属性查询（`cs.display;cs.color;cs.visibility`）
    // 由 3 次全 cascade 摊销为 1 次。html 变（新 snapshot）→ 清空 per-selector 缓存。
    let html = Arc::clone(dom_html);
    let cs_cache: Arc<Mutex<Option<(String, HashMap<String, ComputedStyle>)>>> = Arc::new(Mutex::new(None));
    sandbox.register_callback(
        "__zw_get_computed_style",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let sel = &args[0];
            let prop = &args[1];
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            let mut cache = cs_cache.lock().unwrap_or_else(|e| e.into_inner());
            // html 变 → 清空 per-selector 缓存，重置 key。
            let need_reset = cache.as_ref().is_none_or(|(h, _)| h != &*snap);
            if need_reset {
                *cache = Some(((*snap).clone(), HashMap::new()));
            }
            let (_, map) = cache.as_mut().expect("cs cache populated");
            // 同 selector 命中 → 直接 serialize（O(1)）。
            if let Some(style) = map.get(sel) {
                return serialize_computed_property(style, prop);
            }
            // 未命中：parse + cascade，提取该 selector 的 ComputedStyle 并缓存，再 serialize。
            let (doc, styles) = compute_document_styles(&snap);
            let Some(node) = find_by_selector(&doc, sel) else {
                return String::new();
            };
            let Some(style) = styles.get(&node) else {
                return String::new();
            };
            let value = serialize_computed_property(style, prop);
            map.insert((*sel).clone(), style.clone());
            value
        }),
    );

    // P1a checkbox：属性存在性查询（boolean 属性 checked/disabled 靠存在性；getAttribute 返空串
    // 无法区分存在与空值，故 `el.checked` getter / toggle 判定用本回调）。返 "1"/"0"。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_has_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".to_string();
            }
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if has_attribute(&snap, &args[0], &args[1]) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // 元素全部属性名（`|` 分隔）→ shim `el.dataset` 枚举（ownKeys：data-* 属性 → camelCase 键）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_attr_names",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_attribute_names(&snap, &sel)
        }),
    );

    // P1a select：读 `<select>` 当前选中 option 的 value（HTML spec 语义：首个 selected option，
    // 无则首 option）。shim `select.value` getter 对 tag=SELECT 调此（非 value 属性）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_select_value",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_value_from_html(&snap, &sel)
        }),
    );

    // P1a select：读选中 option 的索引（shim `select.selectedIndex` getter）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_select_index",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_index_from_html(&snap, &sel).to_string()
        }),
    );

    // P1a select：编程设 `select.value = value`——记录 SelectOption mutation（apply 时 mark
    // 匹配 option selected + deselect 兄弟）。匹配浏览器语义：编程设值不自动派 change。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_select_option",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SelectOption {
                        selector: args[0].clone(),
                        value: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_text",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_text_from_html(&snap, &sel)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_attr_handle",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_attr_from_mutations(&list, &args[0], &args[1])
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_text_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_text_from_mutations(&list, &handle)
        }),
    );

    // detached createElement 句柄元素的真实 tag 名（shim `tagName`/`nodeName` 对 handle-only
    // 元素原走 `_tagFromSel` 恒猜 DIV；本回调从 CreateElement 记录取真实 tag）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_tag_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_tag_from_mutations(&list, &handle)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_attr",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetAttr {
                    selector: args[0].clone(),
                    name: args[1].clone(),
                    value: args[2].clone(),
                });
            }
            "ok".into()
        }),
    );

    // `element.removeAttribute(name)` / `delete el.dataset.x` —— 真移除属性（区别于 SetAttr 空值；
    // 布尔/存在性属性须移除才 unset）。记 `DomMutation::RemoveAttr`。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_attr",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveAttr {
                        selector: args[0].clone(),
                        name: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // `element.toggleAttribute(name, force?)`——server-side 决策（apply 时读存在性），连续 toggle
    // 正确复合。第三参 force：`"1"` 强加、`"0"` 强移、缺省切换。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_toggle_attribute",
        Box::new(move |args| {
            if args.len() >= 2 {
                let force = if args.len() >= 3 {
                    match args[2].as_str() {
                        "1" => Some(true),
                        "0" => Some(false),
                        _ => None,
                    }
                } else {
                    None
                };
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::ToggleAttribute {
                        selector: args[0].clone(),
                        name: args[1].clone(),
                        force,
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_style",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetStyle {
                    selector: args[0].clone(),
                    property: args[1].clone(),
                    value: args[2].clone(),
                });
            }
            "ok".into()
        }),
    );

    // `el.style.removeProperty(prop)` — 真移除 style 声明（SetStyle 空值仍 push，不移除）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_style",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveStyle {
                        selector: args[0].clone(),
                        property: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_text",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetText {
                    selector: args[0].clone(),
                    text: args[1].clone(),
                });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove",
        Box::new(move |args| {
            if let Some(sel) = args.first() {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::Remove { selector: sel.clone() });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_element",
        Box::new(move |args| {
            let tag = args.first().map(String::from).unwrap_or_else(|| "div".into());
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateElement {
                    handle: handle.clone(),
                    tag,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_text",
        Box::new(move |args| {
            let text = args.first().map(String::from).unwrap_or_default();
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateTextNode {
                    handle: handle.clone(),
                    text,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_document_fragment",
        Box::new(move |_args| {
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateDocumentFragment { handle: handle.clone() });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_fragment_children",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendFragmentChildren {
                        parent_selector: args[0].clone(),
                        fragment_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_fragment_children_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendFragmentChildrenByHandle {
                        parent_handle: args[0].clone(),
                        fragment_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_fragment_before",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertFragmentBefore {
                        parent_selector: args[0].clone(),
                        fragment_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_fragment_before_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertFragmentBeforeByHandle {
                        parent_handle: args[0].clone(),
                        fragment_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChild {
                        parent_selector: args[0].clone(),
                        child_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChildByHandle {
                        parent_handle: args[0].clone(),
                        child_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_before",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertBefore {
                        parent_selector: args[0].clone(),
                        child_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_before_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertBeforeByHandle {
                        parent_handle: args[0].clone(),
                        child_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_attr_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetAttrOnHandle {
                        handle: args[0].clone(),
                        name: args[1].clone(),
                        value: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_style_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetStyleOnHandle {
                        handle: args[0].clone(),
                        property: args[1].clone(),
                        value: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // `el.style.removeProperty(prop)` 的 handle 版（detached createElement 元素）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_style_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveStyleOnHandle {
                        handle: args[0].clone(),
                        property: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_text_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetTextOnHandle {
                        handle: args[0].clone(),
                        text: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_inner_html",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_inner_html_from_html(&snap, &sel)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_inner_html",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetInnerHtml {
                        selector: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_outer_html",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_outer_html_from_html(&snap, &sel)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_outer_html",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetOuterHtml {
                        selector: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_inner_html_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_inner_html_from_mutations(&list, &handle)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_inner_html_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetInnerHtmlOnHandle {
                        handle: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_html",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentHtml {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        html: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_text",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentText {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        text: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_element",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentElement {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        child_handle: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_handle",
        Box::new(move |args| {
            if let Some(handle) = args.first() {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveHandle { handle: handle.clone() });
            }
            "ok".into()
        }),
    );
}

/// 注入到 V8 的 DOM shim（与 `__zw_*` 回调配套）。
pub fn generate_js_dom_shim() -> &'static str {
    include_str!("js_dom_shim.js")
}

#[cfg(test)]
#[path = "js_dom_bridge_tests.rs"]
mod tests;
