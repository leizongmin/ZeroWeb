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

// WebCrypto host 实现（R2973 从本文件拆出，控制主文件行数）。纯字节级 crypto，无 DOM/CSS 依赖。
mod crypto;
pub use crypto::*;

// CompressionStream/DecompressionStream host 实现（R2986，gzip/deflate 经 flate2）。复用 crypto byte wire。
mod compress;
pub use compress::*;

// Canvas 2D host 操作派发（R2974 从本文件拆出，控制主文件行数）。纯 zero_canvas 类型，无 DOM/选择器依赖。
mod canvas;
pub use canvas::*;

// 选择器匹配 / 子树查询（R2975 从本文件拆出，控制主文件行数）。matches/closest/subtree/children/sibling/parent。
mod selector_match;
pub use selector_match::*;

// `register_dom_callbacks` —— 全部 `__zw_*` 回调注册（R2976 从本文件拆出，控制主文件行数）。最大单体函数。
mod callbacks;
pub use callbacks::*;

// V8 shim 调用串 / 页面脚本包装生成（R3001 从本文件拆出，控制主文件行数）。纯字符串构造，无 DOM/CSS 依赖。
mod script_gen;
pub use script_gen::*;

// CSS 选择器 / 样式规则 wire 序列化（R3001 从本文件拆出，控制主文件行数）。zero_css_parser AST → CSS 字符串。
mod css_wire;
pub use css_wire::*;

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
    /// `document.createComment(text)`（R2816）——注释节点（nodeType 8），框架 placeholder/anchor 高频。
    /// 镜像 [`Self::CreateTextNode`]（host `doc.create_comment` 已存在）。
    CreateComment {
        /// 稳定句柄。
        handle: String,
        /// 注释内容。
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
    /// 对 create 句柄**真移除**属性（区别于 [`SetAttrOnHandle`] 空值——布尔/存在性属性须移除才 unset；
    /// handle 元素 `removeAttribute` 用，R2993）。apply 时 `doc.remove_attribute`；query 函数 latest-wins 判定。
    RemoveAttrOnHandle {
        /// 节点句柄。
        handle: String,
        /// 属性名。
        name: String,
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
            DomMutation::CreateComment { handle, text } => {
                let id = doc.create_comment(text);
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
            DomMutation::RemoveAttrOnHandle { handle, name } => {
                let node = handles
                    .get(handle)
                    .copied()
                    .ok_or_else(|| format!("unknown handle {handle}"))?;
                doc.remove_attribute(node, name);
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

/// `DOMParser` 元素查询 → JSON 数组（R2790）。解析**任意 HTML 串** + 跑 selector，返匹配元素的
/// 只读快照。与 [`query_match_selector`] 关键不同：解析出的文档**不在 dom_html 快照中**，唯一选择器
/// 无处落地（selector 须在 dom_html 内解析），故这里返**完整元素数据**（tag/id/cls/text/outer/attrs），
/// 供 shim 包成只读 element-proxy（DOMParser 不支持 mutation）。`all=false` 首个 / `all=true` 全部。
/// 供 `__zw_parse_html_query` 回调 → shim `DOMParser.parseFromString`。
///
/// 每个 element 快照 JSON：`{"tag","id","cls","text","outer","attrs":{name:value,...}}`。
/// `text` = `text_content`（含后代文本，与 spec `.textContent` 一致）；`outer` = `outer_html`。
/// shim 据此暴露 `tagName/id/className/textContent/outerHTML/innerHTML(派生)/getAttribute/子树 query`。
pub fn parse_html_element_json(html: &str, selector: &str, all: bool) -> String {
    let doc = parse_html(html);
    let root = doc.root();
    let ids: Vec<NodeId> = if all {
        doc.query_selector_all(root, selector.trim())
    } else {
        doc.query_selector(root, selector.trim()).into_iter().collect()
    };
    let mut items: Vec<String> = Vec::new();
    for id in ids {
        let Some(nd) = doc.get(id) else {
            continue;
        };
        let NodeKind::Element(e) = &nd.kind else {
            continue;
        };
        let tag = e.local_name().to_string();
        let mut attrs_json: Vec<String> = Vec::new();
        for attr in &e.attributes {
            attrs_json.push(format!(
                "{}:{}",
                json_str(attr.name.local.as_ref()),
                json_str(attr.value.as_ref())
            ));
        }
        let text = doc.text_content(id).unwrap_or_default();
        let outer = doc.outer_html(id);
        items.push(format!(
            "{{\"tag\":{},\"id\":{},\"cls\":{},\"text\":{},\"outer\":{},\"attrs\":{{{}}}}}",
            json_str(&tag),
            json_str(&doc.get_attribute(id, "id").unwrap_or_default()),
            json_str(&doc.get_attribute(id, "class").unwrap_or_default()),
            json_str(&text),
            json_str(&outer),
            attrs_json.join(",")
        ));
    }
    format!("[{}]", items.join(","))
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

/// P1a form reset：判定元素是否为 reset 按钮（`<input type=reset>` / `<button type=reset>`）。
/// 供 renderer click 路由调 `apply_reset_on_click`（R3050，闭合 R3048 限制⑤——reset 按钮点击自动 form.reset()）。
pub fn is_reset_button(html: &str, elem_sel: &str) -> bool {
    let tag = query_tag_from_html(html, elem_sel);
    let ty = query_attr_from_html(html, elem_sel, "type").to_ascii_lowercase();
    // input/button 仅当显式 type=reset（区别 submit：button 默认 type=submit 非 reset）。
    (tag.eq_ignore_ascii_case("input") || tag.eq_ignore_ascii_case("button")) && ty == "reset"
}

/// P1a 导航：解析 anchor `<a href>` click 的导航目标 URL（R3052）。供 renderer click 路由判定「点击链接是否导航」。
///
/// 返回 `Some(绝对 URL)` 当：元素为 `<a>` 且有非空 href，且 href 非 `javascript:` / `mailto:` / `tel:` / `sms:` /
/// `data:` / `#fragment`，且非 `target=_blank/_top/_parent`（新窗口/顶层，headless no-op）。相对 href 经
/// [`resolve_document_url`] 按 base 解析为绝对。否则 `None`（不导航）。`javascript:` URL 不 eval（headless 简化）；
/// `#hash` 不滚动到锚（headless 无 viewport）。
pub fn anchor_click_target(html: &str, selector: &str, base_url: &str) -> Option<String> {
    if !query_tag_from_html(html, selector).eq_ignore_ascii_case("a") {
        return None;
    }
    let href = query_attr_from_html(html, selector, "href");
    let href = href.trim();
    if href.is_empty() {
        return None; // 无 href / 空 → 不导航
    }
    let lower = href.to_ascii_lowercase();
    // 非导航 scheme / fragment（real browser：javascript: eval / mailto: 外部 / #hash 同文档滚动 → headless no-op）。
    if lower.starts_with("javascript:")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("sms:")
        || lower.starts_with("data:")
        || lower.starts_with('#')
    {
        return None;
    }
    // target=_blank/_top/_parent → 新窗口/顶层（headless 无多窗口，no-op）。
    let target = query_attr_from_html(html, selector, "target");
    let tl = target.trim().to_ascii_lowercase();
    if tl == "_blank" || tl == "_top" || tl == "_parent" {
        return None;
    }
    Some(crate::resolve_document_url(base_url, href))
}

/// P1a 导航：解析 anchor `<a href="#...">` click 的 hash 目标（R3053，闭合 R3052 限制③）。供 renderer click 路由
/// 判定「点击 hash 链接是否更新 location.hash」。返回 `Some(hash)`（含前导 `#`，如 `#sec` / `#`）当元素为 `<a>` 且
/// href 以 `#` 开头；否则 `None`。renderer 经 `script_call_set_location_hash` 调 shim `location.hash = hash`
///（R3006：更新 hash + history entry + 派 hashchange）。headless 无 viewport → 不滚动到锚，仅 hash/hashchange。
pub fn anchor_hash_target(html: &str, selector: &str) -> Option<String> {
    if !query_tag_from_html(html, selector).eq_ignore_ascii_case("a") {
        return None;
    }
    let href = query_attr_from_html(html, selector, "href");
    let href = href.trim();
    if href.starts_with('#') {
        Some(href.to_string())
    } else {
        None
    }
}

/// P1a form control：判定元素是否有某属性（boolean 属性如 `checked`/`disabled` 靠存在性，
/// `getAttribute` 返空串无法区分存在与空值，故供 `__zw_has_attr` / checkbox toggle）。
pub fn has_attribute(html: &str, selector: &str, name: &str) -> bool {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.get_attribute(n, name).is_some())
        .unwrap_or(false)
}

/// P1a 导航（R3054）：解析 `<form>` **GET** 提交的目标绝对 URL（闭合 click 默认动作族：reset/anchor/hash →
/// form-submit 导航）。供 renderer submit 路由（click submit 按钮 / Enter-in-input）在 submit 事件未
/// preventDefault 时计算导航目标。返回 `Some(绝对 GET URL)` 当 form 的 method 为 GET（默认）且 action
/// 可解析；**POST → None**（headless POST 导航 defer——需 fetch body 路径）；method=dialog → None（关 dialog，
/// headless no-op）；无 form / 解析失败 → None。
///
/// **成功控件收集**（HTML spec「构造表单数据集」实用子集）：遍历 form 子树 input/select/textarea/button，
/// 跳过无 `name` / `disabled` 者；input：checkbox/radio 仅 `checked` 入（值=`value` 属性或 "on"），
/// type=submit/image/reset/button/file 跳过（submitter 单独处理）；select：首个 `selected` option（无则首 option，
/// spec 默认选中 quirk）值=`value` 属性或文本；textarea：文本内容。submitter（type=submit 且有 name）的
/// name=value 入数据集（spec：激活的提交按钮参与提交）。
///
/// **已知限制（defer）**：① `<input type=file>`（无真文件）；② `<input type=image>` 的 name.x/name.y 坐标；
/// ③ `<input type=image>` / button 无 value 时 submitter 值 spec 默认值近似；④ disabled fieldset 的首个
/// `<legend>` 子内控件豁免（spec，罕见）。GET 表单全覆盖；POST 见 [`form_post_submission`]。
/// select multiple（R3056）+ fieldset disabled 联动（R3056）+ option disabled 已实现。
///
/// https://html.spec.whatwg.org/#constructing-the-form-data-set
/// https://url.spec.whatwg.org/#concept-urlencoded-serializer
pub fn form_get_submission_url(
    html: &str,
    form_sel: &str,
    submitter_sel: Option<&str>,
    base_url: &str,
) -> Option<String> {
    let (action_abs, method, pairs) = collect_form_data(html, form_sel, submitter_sel, base_url)?;
    // GET：method 非 post/dialog（缺省/GET/无效值均按 GET，spec）。POST 由 form_post_submission 处理。
    if method == "post" || method == "dialog" {
        return None;
    }
    let mut url = url::Url::parse(&action_abs).ok()?;
    // GET：form 数据集替换 action URL 的 query 段（spec），fragment 保留。
    if pairs.is_empty() {
        // 空数据集 → 清 query（含 action 旧 query），无尾 `?`。
        url.set_query(None);
    } else {
        let mut q = url.query_pairs_mut();
        q.clear();
        for (n, v) in &pairs {
            q.append_pair(n, v);
        }
    }
    Some(url.to_string())
}

/// P1a 导航（R3055）：解析 `<form>` **POST** 提交的目标 URL + `application/x-www-form-urlencoded` body
///（闭合 form 导航 POST 侧，对称 R3054 GET——登录/数据提交表单常用 POST）。返回 `Some((action_url, body))`
/// 当 method=POST 且 action 可解析；GET/dialog → None。body = 成功控件 urlencoded（name=value & ...，
/// 复用 [`collect_form_data`]，控件收集规则与 GET 完全一致）。action_url 不含 query（POST 数据在 body）。
///
/// **已知限制（defer）**：① `enctype=multipart/form-data`（headless 近似 urlencoded，独立切片）；
/// ② 同 GET（file/image 坐标/disabled-legend 豁免）。POST 登录/提交表单（urlencoded enctype，默认）全覆盖。
/// select multiple + fieldset disabled 联动 + option disabled 已实现（R3056）。
pub fn form_post_submission(
    html: &str,
    form_sel: &str,
    submitter_sel: Option<&str>,
    base_url: &str,
) -> Option<(String, String)> {
    let (action_abs, method, pairs) = collect_form_data(html, form_sel, submitter_sel, base_url)?;
    if method != "post" {
        return None;
    }
    // POST body = urlencoded form data（spec application/x-www-form-urlencoded，multipart defer）。
    // 经 throwaway Url 的 query_pairs_mut 复用 url crate 的 form_urlencoded 序列化（无新依赖）。
    let mut body_url = url::Url::parse("https://form.post.invalid/").ok()?;
    {
        let mut q = body_url.query_pairs_mut();
        for (n, v) in &pairs {
            q.append_pair(n, v);
        }
    }
    let body = body_url.query().unwrap_or("").to_string();
    Some((action_abs, body))
}

/// P1a form 导航共享核心：解析 `<form>` → 校验 form 元素 → 读 method（lowercased）+ action（按 base 解析为
/// 绝对，缺省→base_url）→ 收集成功控件 (name, value) 对（文档序）。供 [`form_get_submission_url`]（GET）和
/// [`form_post_submission`]（POST）复用（DRY）。非 form / action 不可解析 → None。
fn collect_form_data(
    html: &str,
    form_sel: &str,
    submitter_sel: Option<&str>,
    base_url: &str,
) -> Option<(String, String, Vec<(String, String)>)> {
    let doc = parse_html(html);
    let form = find_by_selector(&doc, form_sel)?;
    // 仅 <form> 元素。
    let is_form = doc.get(form).and_then(|n| match &n.kind {
        NodeKind::Element(e) => Some(e.local_name().eq_ignore_ascii_case("form")),
        _ => None,
    })?;
    if !is_form {
        return None;
    }
    // method（lowercased，供调用方判 GET/POST/dialog）；action 按 base 解析为绝对。
    let method = doc
        .get_attribute(form, "method")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let action = doc.get_attribute(form, "action").unwrap_or_default();
    let action_abs = if action.trim().is_empty() {
        base_url.to_string()
    } else {
        crate::resolve_document_url(base_url, action.trim())
    };
    // action 须可解析为绝对 URL（调用方据此建 Url / body）。
    url::Url::parse(&action_abs).ok()?;

    // submitter 解析为节点身份（NodeId 比较，避免跨选择器生成路径的字符串不一致）。
    let submitter_node = submitter_sel.and_then(|s| find_by_selector(&doc, s));

    // 收集成功控件（文档序，spec「constructing the form data set」实用子集）。
    let mut controls: Vec<NodeId> = Vec::new();
    collect_form_controls(&doc, form, &mut controls);
    let mut pairs: Vec<(String, String)> = Vec::new();
    for ctrl in controls {
        // 跳过无 name / disabled（spec：disabled 不参与提交；R3056 含 fieldset disabled 联动）。
        let name = doc.get_attribute(ctrl, "name").unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        if is_control_disabled(&doc, ctrl) {
            continue;
        }
        let tag = element_local_name(&doc, ctrl).to_ascii_lowercase();
        let ty = doc.get_attribute(ctrl, "type").unwrap_or_default().to_ascii_lowercase();
        match tag.as_str() {
            "input" => match ty.as_str() {
                // submit/image：仅激活的 submitter 参与（spec）；非 submitter 跳过。image 坐标 defer。
                "submit" | "image" => {
                    if ty == "submit" && Some(ctrl) == submitter_node {
                        pairs.push((name, doc.get_attribute(ctrl, "value").unwrap_or_default()));
                    }
                }
                // reset/button/file：从不提交（file 无真文件，headless defer）。
                "reset" | "button" | "file" => {}
                "checkbox" | "radio" => {
                    // 仅 checked 入；值 = value 属性（缺省时 "on"，spec default）。
                    if doc.get_attribute(ctrl, "checked").is_some() {
                        let val = doc.get_attribute(ctrl, "value").unwrap_or_else(|| "on".to_string());
                        pairs.push((name, val));
                    }
                }
                // text/password/hidden/search/email/url/number/date/color/range/... → value 属性。
                _ => pairs.push((name, doc.get_attribute(ctrl, "value").unwrap_or_default())),
            },
            "select" => {
                // R3056：multiple → 全部 selected 且未 disabled 的 option 各入一项（spec）；无 selected 则不提交
                //（与单选「默认首项」quirk 不同）。单选 → 首个 selected 且未 disabled option（无则首个未 disabled option）。
                let mut opts: Vec<NodeId> = Vec::new();
                collect_form_controls_tag(&doc, ctrl, "option", &mut opts);
                let is_multiple = doc.get_attribute(ctrl, "multiple").is_some();
                if is_multiple {
                    for opt in opts {
                        let enabled = doc.get_attribute(opt, "disabled").is_none();
                        let selected = doc.get_attribute(opt, "selected").is_some();
                        if enabled && selected {
                            let val = doc.get_attribute(opt, "value").unwrap_or_else(|| doc.inner_html(opt));
                            pairs.push((name.clone(), val));
                        }
                    }
                } else {
                    let chosen = opts
                        .iter()
                        .copied()
                        .find(|o| {
                            doc.get_attribute(*o, "disabled").is_none() && doc.get_attribute(*o, "selected").is_some()
                        })
                        .or_else(|| {
                            opts.iter()
                                .copied()
                                .find(|o| doc.get_attribute(*o, "disabled").is_none())
                        });
                    if let Some(opt) = chosen {
                        let val = doc.get_attribute(opt, "value").unwrap_or_else(|| doc.inner_html(opt));
                        pairs.push((name, val));
                    }
                }
            }
            "textarea" => {
                // textarea 值 = 子树文本内容（R2996 模型：value ↔ textContent，inner_html 近似纯文本节点）。
                pairs.push((name, doc.inner_html(ctrl)));
            }
            "button" => {
                // <button>：默认 type=submit；仅 submitter 参与，type=button/reset 跳过。
                let bty = if ty.is_empty() {
                    "submit".to_string()
                } else {
                    ty.clone()
                };
                if bty == "submit" && Some(ctrl) == submitter_node {
                    pairs.push((name, doc.get_attribute(ctrl, "value").unwrap_or_default()));
                }
            }
            _ => {}
        }
    }
    Some((action_abs, method, pairs))
}

/// 递归收集 `root` 子树（含嵌套 fieldset/div 等）内全部 input/select/textarea/button 元素，文档序。
/// 供 [`form_get_submission_url`] 遍历表单成功控件（html5ever 不构造嵌套 form，无重复计风险）。
fn collect_form_controls(doc: &Document, root: NodeId, out: &mut Vec<NodeId>) {
    for child in doc.child_nodes(root) {
        let is_elem = doc.get(child).is_some_and(|n| matches!(n.kind, NodeKind::Element(_)));
        if !is_elem {
            continue;
        }
        if matches!(
            element_local_name(doc, child),
            "input" | "select" | "textarea" | "button"
        ) {
            out.push(child);
        }
        collect_form_controls(doc, child, out);
    }
}

/// 递归收集 `root` 子树内全部指定 tag 元素，文档序。供 select 的 `<option>` 收集。
fn collect_form_controls_tag(doc: &Document, root: NodeId, tag: &str, out: &mut Vec<NodeId>) {
    for child in doc.child_nodes(root) {
        let is_elem = doc.get(child).is_some_and(|n| matches!(n.kind, NodeKind::Element(_)));
        if !is_elem {
            continue;
        }
        if element_local_name(doc, child) == tag {
            out.push(child);
        }
        collect_form_controls_tag(doc, child, tag, out);
    }
}

/// 元素的 local_name（小写归一前），非元素返空串。
fn element_local_name(doc: &Document, node: NodeId) -> &str {
    doc.get(node)
        .and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name()),
            _ => None,
        })
        .unwrap_or("")
}

/// 控件是否禁用（spec「disable 属性」barred-the-second-constraint）：自身 `disabled` 属性 OR 任一祖先
/// `<fieldset>` 有 `disabled` 属性（fieldset disabled 联动禁用全部后代控件，R3056）。供 [`collect_form_data`]
/// 跳过禁用控件。legend 豁免 defer（spec：disabled fieldset 首个 `<legend>` 子内控件不禁用，罕见场景）。
fn is_control_disabled(doc: &Document, ctrl: NodeId) -> bool {
    if doc.get_attribute(ctrl, "disabled").is_some() {
        return true;
    }
    let mut cur = doc.parent_node(ctrl);
    while let Some(p) = cur {
        if element_local_name(doc, p).eq_ignore_ascii_case("fieldset") && doc.get_attribute(p, "disabled").is_some() {
            return true;
        }
        cur = doc.parent_node(p);
    }
    false
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

/// 元素的全部属性本地名（`|` 分隔，文档序），**latest-wins**——在快照基底上正序应用 pending
/// `SetAttr`/`RemoveAttr` mutation（同 `selector`）：SetAttr 加名（去重保序）、RemoveAttr 删名。
/// 供 `__zw_attr_names` 回调 → shim `getAttributeNames`/`hasAttributes`/`dataset` 枚举反映同批
/// setAttribute/removeAttribute / dataset 设删（R3002，闭合 R2995 限制 ③ stale）。
pub fn element_attribute_names_lw(html: &str, mutations: &[DomMutation], selector: &str) -> String {
    let mut names: Vec<String> = element_attribute_names(html, selector)
        .split('|')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    for m in mutations {
        match m {
            DomMutation::SetAttr { selector: s, name, .. } if s == selector => {
                if !names.iter().any(|n| n == name) {
                    names.push(name.clone());
                }
            }
            DomMutation::RemoveAttr { selector: s, name } if s == selector => {
                names.retain(|n| n != name);
            }
            _ => {}
        }
    }
    names.join("|")
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

/// `descendant` 是否为 `ancestor` 的后代（含自身）——经 parent_node 链上行。供 SelectOption 关联
/// option↔其所属 select（R3000）。区别于 `element_contains`（取 html+selector 重解析），本函数在已解析
/// Document 上操作，避免循环内重复 parse。
fn is_descendant_node(doc: &Document, descendant: NodeId, ancestor: NodeId) -> bool {
    let mut cur = descendant;
    loop {
        if cur == ancestor {
            return true;
        }
        match doc.parent_node(cur) {
            Some(p) if p != cur => cur = p,
            _ => return false,
        }
    }
}

/// R3000：读 select 的最新编程选中值（`select.value=` 推的 `SelectOption` mutation）。
/// 逆序扫 mutations，返最新 `SelectOption{selector==select_sel}` 的 value。无 → None（回落快照）。
pub fn latest_select_option_value<'a>(mutations: &'a [DomMutation], select_sel: &str) -> Option<&'a str> {
    for m in mutations.iter().rev() {
        if let DomMutation::SelectOption { selector, value } = m
            && selector == select_sel
        {
            return Some(value.as_str());
        }
    }
    None
}

/// R3000：select 内首个 value==`value` 的 option 索引（0-based）；无匹配 → -1。供 `select.selectedIndex`
/// getter consult `SelectOption`（编程选中后反映正确索引，旧读快照 stale）。复用 `option_value` 匹配规则。
pub fn option_index_for_value(html: &str, select_sel: &str, value: &str) -> i32 {
    let doc = parse_html(html);
    let Some(sel) = find_by_selector(&doc, select_sel) else {
        return -1;
    };
    for (idx, opt) in doc.query_selector_all(sel, "option").iter().enumerate() {
        if option_value(&doc, *opt) == value {
            return idx as i32;
        }
    }
    -1
}

/// R3000：解析 option 的 selected 态——单次逆序扫 mutations，**最新适用 mutation 胜出**（正确处理
/// SetAttr/RemoveAttr 与 SelectOption 交错）：
/// - `SetAttr{selected}` on option_sel → Some(true)；`RemoveAttr{selected}` on option_sel → Some(false)。
/// - `SelectOption` on option 所属 select → Some(option_value == select_option_value)（matched=true/其它=false）。
///
/// 仅当遇到 SelectOption mutation 时才解析 HTML（懒解析——多数 option.selected 读无 SelectOption 待 apply）。
/// 返回 None → 回落快照（option 的 selected 属性存在性）。供 `__zw_option_selected` 回调。
pub fn option_selected_resolved(html: &str, mutations: &[DomMutation], option_sel: &str) -> Option<bool> {
    let needs_html = mutations.iter().any(|m| matches!(m, DomMutation::SelectOption { .. }));
    let doc = if needs_html { Some(parse_html(html)) } else { None };
    for m in mutations.iter().rev() {
        match m {
            DomMutation::SetAttr { selector, name, .. } if selector == option_sel && name == "selected" => {
                return Some(true);
            }
            DomMutation::RemoveAttr { selector, name } if selector == option_sel && name == "selected" => {
                return Some(false);
            }
            DomMutation::SelectOption { selector, value } => {
                if let Some(doc) = &doc
                    && let (Some(opt), Some(seln)) =
                        (find_by_selector(doc, option_sel), find_by_selector(doc, selector))
                    && is_descendant_node(doc, opt, seln)
                {
                    return Some(option_value(doc, opt) == value.as_str());
                }
            }
            _ => {}
        }
    }
    None
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
        if let DomMutation::CreateComment { handle: h, text } = m
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

/// 从当前 HTML 快照查询 textContent。
pub fn query_text_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.inner_html(n))
        .unwrap_or_default()
}

/// 从已记录变更中查询 create 句柄上的属性（脚本执行期间只读）。
pub fn query_attr_from_mutations(mutations: &[DomMutation], handle: &str, name: &str) -> String {
    // 逆序扫描，latest-wins：最近的 SetAttrOnHandle 决定值，最近的 RemoveAttrOnHandle 表 absent（R2993）。
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
        if let DomMutation::RemoveAttrOnHandle { handle: h, name: n } = m
            && h == handle
            && n == name
        {
            return String::new();
        }
    }
    String::new()
}

/// 从已记录变更中判定 create 句柄元素是否设有某属性（脚本执行期间只读）。句柄元素不存在于
/// HTML 快照（由 `createElement` 创建），`has_attribute` 快照查询对其恒 false；本函数逆序扫
/// [`DomMutation::SetAttrOnHandle`] / [`DomMutation::RemoveAttrOnHandle`]，**latest-wins** 判定存在性
///（R2993：remove 后返 false，区别于 set-empty 仍 present；boolean 属性 `selected`/`disabled` 等存在即 true）。
/// 供 `__zw_has_attr_handle` 回调 → `new Option()` 创建的句柄 option 的 `.selected`/`.defaultSelected` 读
/// + handle 元素 `hasAttribute`（R2992+）+ custom element attributeChangedCallback old-value 判定（R2992）。
pub fn has_attr_from_mutations(mutations: &[DomMutation], handle: &str, name: &str) -> bool {
    for m in mutations.iter().rev() {
        match m {
            DomMutation::SetAttrOnHandle { handle: h, name: n, .. } if h == handle && n == name => return true,
            DomMutation::RemoveAttrOnHandle { handle: h, name: n } if h == handle && n == name => return false,
            _ => {}
        }
    }
    false
}

/// sel-based 元素属性的 latest-wins 覆盖判定（R2995）。
///
/// sel-based 元素的 `getAttribute`/`hasAttribute` 旧实现仅读 HTML 快照（[`query_attr_from_html`] /
/// [`has_attribute`]），但快照在脚本执行期间不反映同批 `SetAttr`/`RemoveAttr` 变更（render apply 后才更新）→
/// `removeAttribute` 后 `hasAttribute` 恒 true、`getAttribute` 仍返旧值（R2993 latent gap）。本函数逆序扫
/// selector-keyed [`DomMutation::SetAttr`] / [`DomMutation::RemoveAttr`]，latest-wins 给出覆盖信号：
/// - `Some(Some(value))`：最近 `SetAttr` 命中 → 属性值为 `value`（存在）；
/// - `Some(None)`：最近 `RemoveAttr` 命中 → 属性 absent；
/// - `None`：无覆盖 → 调用方回落快照（`getAttribute` 走 [`query_attr_from_html`]，`hasAttribute` 走 [`has_attribute`]）。
///
/// 与 handle 路径（[`query_attr_from_mutations`] / [`has_attr_from_mutations`]）对称：handle 元素不在快照，
/// 故无回落；sel-based 元素在快照，故无命中时回落。selector 为元素身份选择器（querySelector/getElementById/
/// html·body·head），同元素 setAttribute/getAttribute 传同一 selector 串，字符串相等匹配。
pub fn sel_attr_override(mutations: &[DomMutation], selector: &str, name: &str) -> Option<Option<String>> {
    for m in mutations.iter().rev() {
        match m {
            DomMutation::SetAttr {
                selector: s,
                name: n,
                value: v,
            } if s == selector && n == name => return Some(Some(v.clone())),
            DomMutation::RemoveAttr { selector: s, name: n } if s == selector && n == name => {
                return Some(None);
            }
            _ => {}
        }
    }
    None
}

/// sel-based 元素 textContent 的 latest-wins 覆盖判定（R3028）。
///
/// 镜像 [`sel_attr_override`]：sel-based 元素的 textContent getter 旧实现仅读 HTML 快照
/// （[`query_text_from_html`]），但快照在脚本执行期间不反映同批 [`DomMutation::SetText`] 变更
/// （render apply 后才更新）→ `el.textContent = 'x'` 后再读 `el.textContent` 仍返旧值（stale 快照 latent
/// bug）；MutationObserver `characterDataOldValue` 亦需 mutate 前 latest-wins 读旧文本。本函数逆序扫
/// selector-keyed `SetText`，latest-wins 给出当前逻辑文本：
/// - `Some(text)`：最近 `SetText` 命中 → 文本为 `text`；
/// - `None`：无覆盖 → 调用方回落快照（textContent getter / MO old value）。
///
/// 与 handle 路径（[`query_text_from_mutations`]）对称：handle 元素不在快照故无回落；sel-based 元素
/// 在快照，故无命中时回落。反射类读（output.defaultValue / textarea 初始 value）须稳定读快照，
/// 故仍用纯快照 [`query_text_from_html`]，不走本函数。
pub fn sel_text_override(mutations: &[DomMutation], selector: &str) -> Option<String> {
    for m in mutations.iter().rev() {
        if let DomMutation::SetText { selector: s, text } = m
            && s == selector
        {
            return Some(text.clone());
        }
    }
    None
}

/// 从已记录变更中查询 create 句柄上的 textContent。
pub fn query_text_from_mutations(mutations: &[DomMutation], handle: &str) -> String {
    for m in mutations.iter().rev() {
        if let DomMutation::CreateTextNode { handle: h, text } = m
            && h == handle
        {
            return text.clone();
        }
        if let DomMutation::CreateComment { handle: h, text } = m
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

/// WHATWG URL 解析为 JSON 串（供 JS shim `new URL(url, base)` 经 `__zw_parse_url` 回调消费）。
///
/// 委托 `url` crate（spec-correct：base 解析 / percent-encoding / IDNA / 默认端口归一）。成功返
/// JSON 串（protocol/username/password/hostname/port/host/origin/pathname/search/hash/href）；
/// 失败（含 base 无效）返空串（shim 抛 TypeError，spec 一致）。提取为纯函数便于单测复用。
pub(crate) fn parse_url_to_json(input: &str, base: Option<&str>) -> String {
    let result = match base.filter(|b| !b.is_empty()) {
        Some(base_str) => match url::Url::parse(base_str) {
            Ok(base_url) => url::Url::options().base_url(Some(&base_url)).parse(input),
            Err(_) => return String::new(),
        },
        None => url::Url::parse(input),
    };
    match result {
        Ok(u) => serialize_url(&u),
        Err(_) => String::new(),
    }
}

/// `Url` → JSON 组件串（protocol/username/password/hostname/port/host/origin/pathname/search/hash/href）。
/// `parse_url_to_json` 与 [`set_url_part`] 共用——后者 mutate `Url` 后同样序列化。
fn serialize_url(url: &url::Url) -> String {
    let scheme = url.scheme();
    let hostname = url.host_str().unwrap_or("");
    let port = url.port().map(|p| p.to_string()).unwrap_or_default();
    let host = if port.is_empty() {
        hostname.to_string()
    } else {
        format!("{hostname}:{port}")
    };
    format!(
        r#"{{"protocol":{},"username":{},"password":{},"hostname":{},"port":{},"host":{},"origin":{},"pathname":{},"search":{},"hash":{},"href":{}}}"#,
        json_str(&format!("{scheme}:")),
        json_str(url.username()),
        json_str(url.password().unwrap_or("")),
        json_str(hostname),
        json_str(&port),
        json_str(&host),
        json_str(&url.origin().ascii_serialization()),
        json_str(url.path()),
        json_str(&url.query().map(|q| format!("?{q}")).unwrap_or_default()),
        json_str(&url.fragment().map(|f| format!("#{f}")).unwrap_or_default()),
        json_str(url.as_str()),
    )
}

/// URL 组件 setter（供 JS shim URL 属性 setter 经 `__zw_set_url_part` 回调消费）。
///
/// 委托 `url` crate 的 `Url` setters（spec-correct：set_scheme/set_host/set_path/set_query 等，
/// 含 percent-encoding / IDNA / 默认端口归一）。`prev_href` 为当前 href；`part` 标识组件（href 时
/// 忽略 prev_href，整体重解析 value）；成功返新 JSON 串，失败返空串（shim 抛 TypeError，spec）。
pub(crate) fn set_url_part(prev_href: &str, part: &str, value: &str) -> String {
    let mut url = if part == "href" {
        match url::Url::parse(value) {
            Ok(u) => u,
            Err(_) => return String::new(),
        }
    } else {
        match url::Url::parse(prev_href) {
            Ok(u) => u,
            Err(_) => return String::new(),
        }
    };
    let ok = match part {
        "protocol" => url.set_scheme(value.trim_end_matches(':')).is_ok(),
        "hostname" => url.set_host(Some(value)).is_ok(),
        "host" => set_url_host_and_port(&mut url, value),
        "port" => url.set_port(value.parse::<u16>().ok()).is_ok(),
        "pathname" => {
            url.set_path(value);
            true
        }
        "search" => {
            let q = value.strip_prefix('?').unwrap_or(value);
            url.set_query(if q.is_empty() { None } else { Some(q) });
            true
        }
        "hash" => {
            let f = value.strip_prefix('#').unwrap_or(value);
            url.set_fragment(if f.is_empty() { None } else { Some(f) });
            true
        }
        "username" => url.set_username(value).is_ok(),
        "password" => url
            .set_password(if value.is_empty() { None } else { Some(value) })
            .is_ok(),
        _ => true, // 含 "href"（已整体重解析）
    };
    if ok { serialize_url(&url) } else { String::new() }
}

/// `host` setter 辅助：`host[:port]` 拆分（仅当 `:` 后全数字视为端口），分别 set_host + set_port。
fn set_url_host_and_port(url: &mut url::Url, value: &str) -> bool {
    let (host, port) = match value.rfind(':') {
        Some(i) if !value[i + 1..].is_empty() && value[i + 1..].chars().all(|c| c.is_ascii_digit()) => {
            (&value[..i], Some(&value[i + 1..]))
        }
        _ => (value, None),
    };
    if url.set_host(Some(host)).is_err() {
        return false;
    }
    if let Some(p) = port {
        let _ = url.set_port(p.parse::<u16>().ok());
    }
    true
}

/// 媒体查询求值为 JSON 串（供 JS shim `window.matchMedia(query)` 经 `__zw_match_media` 回调消费）。
///
/// 委托 `zero_css_parser::media_query`（spec-correct：min/max-width/height、orientation、resolution、
/// prefers-color-scheme 等）。逗号分隔的 query list 取**或**（任一 match 即 matches=true）。返回
/// `{"matches":bool,"media":<query>}`。viewport 宽高由 JS 侧传入（innerWidth/innerHeight，生产由
/// host 更新为真 viewport）。提取为纯函数便于单测复用。
pub(crate) fn match_media_to_json(query: &str, width: f64, height: f64) -> String {
    let ctx = zero_css_parser::media_query::MediaContext::new(width, height);
    let matches = zero_css_parser::media_query::parse_media_query(query)
        .map(|mqs| {
            mqs.iter()
                .any(|mq| zero_css_parser::media_query::evaluate_media_query(mq, &ctx))
        })
        .unwrap_or(false);
    format!(r#"{{"matches":{},"media":{}}}"#, matches, json_str(query))
}

/// CSS.supports 特性检测（供 JS shim `CSS.supports(prop,val?)` 经 `__zw_css_supports` 回调消费）。
///
/// 两参形式 `supports(prop, val)`：ZW 是否 apply 该声明（known-property gate + [`apply_property_value_with_quirks`]）。
/// 单参形式 `supports(text)`：supports-condition 求值（`not`/括号/声明 `prop: val`；`and`/`or` 深嵌套 defer）。
/// 语义近似「ZW 能 apply」≈「支持」，对 ZW 解析但部分实现的特性偏乐观（优于假阴性）。
pub(crate) fn css_supports(prop_or_text: &str, value: Option<&str>) -> bool {
    let mut style = zero_style_system::ComputedStyle::default();
    match value {
        Some(v) => decl_supported(&mut style, prop_or_text.trim(), v.trim()),
        None => eval_supports_condition(prop_or_text, &mut style),
    }
}

/// 声明是否被 ZW 支持（已知属性 + 值可 apply）。
fn decl_supported(style: &mut zero_style_system::ComputedStyle, prop: &str, val: &str) -> bool {
    zero_style_system::PropertyRegistry::known_properties().contains(&prop)
        && zero_style_system::apply_property_value_with_quirks(style, prop, val, false, false)
}

/// supports-condition 递归求值。优先用 css-parser 的 [`parse_supports_condition`]（完整处理
/// `and`/`or`/`not`/嵌套/`selector()`，返 [`SupportsCondition`] AST），按 AST 求值；parser 失败时
/// 回退裸声明 `prop: val`（浏览器兼容：`CSS.supports('display:grid')` 无括号形式，parser 要求括号）。
/// https://drafts.csswg.org/css-conditional-3/#supports-condition
fn eval_supports_condition(text: &str, style: &mut zero_style_system::ComputedStyle) -> bool {
    if let Some(cond) = zero_css_parser::parse_supports_condition(text) {
        return eval_supports_cond_ast(&cond, style);
    }
    // 回退：裸声明 `prop: val`（无括号）。
    let t = text.trim();
    if let Some(idx) = t.find(':') {
        let prop = t[..idx].trim();
        let val = t[idx + 1..].trim();
        return decl_supported(style, prop, val);
    }
    false
}

/// 递归求值 [`SupportsCondition`] AST：
/// - [`SupportsCondition::Property`] → [`decl_supported`]（已知属性 + 可 apply）。
/// - [`SupportsCondition::And`] → 全部为真；[`SupportsCondition::Or`] → 任一为真；
///   [`SupportsCondition::Not`] → 取反。
/// - [`SupportsCondition::Selector`] → headless permissive true（ZW 支持选择器，含 :has() 等已实现）。
/// - [`SupportsCondition::GeneralEnclosed`] → false（spec：general-enclosed 恒 false）。
fn eval_supports_cond_ast(
    cond: &zero_css_parser::SupportsCondition,
    style: &mut zero_style_system::ComputedStyle,
) -> bool {
    use zero_css_parser::SupportsCondition;
    match cond {
        SupportsCondition::Property(prop, val) => decl_supported(style, prop, val),
        SupportsCondition::Selector(_) => true,
        SupportsCondition::And(conds) => conds.iter().all(|c| eval_supports_cond_ast(c, style)),
        SupportsCondition::Or(conds) => conds.iter().any(|c| eval_supports_cond_ast(c, style)),
        SupportsCondition::Not(c) => !eval_supports_cond_ast(c, style),
        SupportsCondition::GeneralEnclosed(_) => false,
    }
}

/// 注入到 V8 的 DOM shim（与 `__zw_*` 回调配套）。shim 原为单文件 7610 行（2000 行准则 3.8×），R2963 拆为
/// `js_dom_shim/part01..06.js`（按行边界字节精确切分，各 <1350 行）。首次调用经 `OnceLock` 拼接（泄漏一次，
/// 返 `&'static str`，264 调用点零改动）；拼接字节与原单文件逐字节一致 → 零行为变更。
pub fn generate_js_dom_shim() -> &'static str {
    static SHIM: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SHIM.get_or_init(|| {
        let mut s = String::new();
        s.push_str(include_str!("js_dom_shim/part01.js"));
        s.push_str(include_str!("js_dom_shim/part02.js"));
        s.push_str(include_str!("js_dom_shim/part03.js"));
        s.push_str(include_str!("js_dom_shim/part04.js"));
        s.push_str(include_str!("js_dom_shim/part05.js"));
        s.push_str(include_str!("js_dom_shim/part06.js"));
        s.replace("__ZERO_BUILD_VERSION__", zero_product_version::VERSION)
    })
}

// 修复（2026-08-08）：测试模块整个是 V8 DOM 桥接专属（part01-11 全部无条件
// 引用 zero_script_sandbox::V8Sandbox，quickjs 模式无此类型）——quickjs 矩阵
// clippy 175 个 unresolved import 失败。模块级 cfg(feature = "v8") 跳过。
#[cfg(test)]
#[cfg(feature = "v8")]
#[path = "js_dom_bridge_tests.rs"]
mod tests;
