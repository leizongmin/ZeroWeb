//! JS → DOM 桥接 — 将 V8 shim 回调记录的变更应用到 `zero_dom::Document`。
//!
//! 与 `zero_script_sandbox::V8Sandbox::register_callback` 配合：JS 侧 shim 把
//! DOM 操作翻译为 `__zw_*` 扁平回调，宿主记录 [`DomMutation`] 后调用
//! [`apply_dom_mutations`] 并序列化回 HTML。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use zero_dom::{Document, NodeId, NodeKind, parse_html};
use zero_script_sandbox::Sandbox;

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
    /// 按句柄移除节点。
    RemoveHandle {
        /// 节点句柄。
        handle: String,
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

/// 将变更列表应用到文档（按顺序）。
pub fn apply_dom_mutations(doc: &mut Document, mutations: &[DomMutation]) -> Result<(), String> {
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
            DomMutation::RemoveHandle { handle } => {
                if let Some(node) = handles.get(handle).copied()
                    && let Some(parent) = doc.get(node).and_then(|n| n.parent)
                {
                    doc.remove_child(parent, node).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    Ok(())
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

/// 解析 HTML、应用变更并返回序列化后的 HTML。
pub fn apply_mutations_to_html(html: &str, mutations: &[DomMutation]) -> Result<String, String> {
    let mut doc = parse_html(html);
    apply_dom_mutations(&mut doc, mutations)?;
    Ok(doc.outer_html(doc.root()))
}

/// 从 HTML 快照查询首个匹配的稳定选择器（供 `__zw_query_match`）。
pub fn query_match_selector(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .and_then(|n| stable_selector_for_node(&doc, n))
        .unwrap_or_default()
}

/// 从 HTML 快照查询全部匹配的稳定选择器（`|` 分隔，供 `__zw_query_all`）。
pub fn query_all_selector_list(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_all_selectors(&doc, selector).join("|")
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

/// 从当前 HTML 快照查询 innerHTML。
pub fn query_inner_html_from_html(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    find_by_selector(&doc, selector)
        .map(|n| doc.inner_html(n))
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
mod tests {
    use super::*;

    #[test]
    fn test_apply_set_attr_src() {
        let html = "<html><body><img id=\"i\" src=\"old.png\"></body></html>";
        let mutations = vec![DomMutation::SetAttr {
            selector: "#i".into(),
            name: "src".into(),
            value: "new.png".into(),
        }];
        let out = apply_mutations_to_html(html, &mutations).unwrap();
        assert!(out.contains("src=\"new.png\""));
    }

    #[test]
    fn test_apply_set_style() {
        let html = "<html><body><div id=\"d\"></div></body></html>";
        let mutations = vec![DomMutation::SetStyle {
            selector: "#d".into(),
            property: "color".into(),
            value: "red".into(),
        }];
        let out = apply_mutations_to_html(html, &mutations).unwrap();
        assert!(out.contains("color: red") || out.contains("color:red"));
    }

    #[test]
    fn test_apply_class_name_via_set_attr() {
        let html = "<html><body><div id=\"d\"></div></body></html>";
        let mutations = vec![DomMutation::SetAttr {
            selector: "#d".into(),
            name: "class".into(),
            value: "active".into(),
        }];
        let out = apply_mutations_to_html(html, &mutations).unwrap();
        assert!(out.contains("class=\"active\""));
    }

    #[test]
    fn test_apply_create_and_append() {
        let html = "<html><body id=\"b\"></body></html>";
        let mutations = vec![
            DomMutation::CreateElement {
                handle: "__n1".into(),
                tag: "p".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__n1".into(),
                name: "id".into(),
                value: "p1".into(),
            },
            DomMutation::CreateTextNode {
                handle: "__n2".into(),
                text: "hello".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#b".into(),
                child_handle: "__n1".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#p1".into(),
                child_handle: "__n2".into(),
            },
        ];
        let out = apply_mutations_to_html(html, &mutations).unwrap();
        assert!(out.contains("<p id=\"p1\">hello</p>"));
    }

    #[test]
    fn test_find_all_selectors() {
        let html = "<html><body><p class=\"x\"></p><p class=\"x\"></p></body></html>";
        let doc = parse_html(html);
        let sels = find_all_selectors(&doc, "p.x");
        assert_eq!(sels.len(), 2);
    }

    #[test]
    fn test_collect_element_ids_dedup_preserve_order() {
        let html = "<html><body>\
                    <div id=\"container\"></div>\
                    <span id=\"target\"></span>\
                    <p id=\"container\"></p>\
                    <b></div>\
                    </body></html>";
        let ids = collect_element_ids(html);
        // 去重（首个 container 保留），保序，跳过无 id 元素。
        assert_eq!(ids, "container|target");
    }

    #[test]
    fn test_collect_element_ids_empty() {
        let html = "<html><body><div></div><p class=\"x\"></p></body></html>";
        assert_eq!(collect_element_ids(html), "");
    }

    #[test]
    fn test_apply_inner_html() {
        let html = "<html><body><div id=\"d\">old</div></body></html>";
        let mutations = vec![DomMutation::SetInnerHtml {
            selector: "#d".into(),
            html: "<b>new</b>".into(),
        }];
        let out = apply_mutations_to_html(html, &mutations).unwrap();
        assert!(out.contains("<b>new</b>"));
    }

    #[test]
    fn test_shim_not_empty() {
        assert!(generate_js_dom_shim().contains("__zw_set_attr"));
        assert!(generate_js_dom_shim().contains("addEventListener"));
    }

    #[test]
    fn test_shim_async_resolve_callback_e2e() {
        // P1b S1（方案 A）端到端：注入**生产** DOM shim（含 __zwResolveCallback + pending 表），
        // 验证 V8Sandbox::resolve_async_callback 经 shim 的 JS 契约真实 resolve Promise。
        // 宿主回调同步返「回调 ID」，JS 建 pending Promise；Rust resolve 触发 .then。
        use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
        let config = SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        // 注入生产 shim（tab_js_worker.rs / js_worker.rs 同款）。
        sandbox.execute(generate_js_dom_shim()).unwrap();
        sandbox.register_callback("__zw_start_async", Box::new(|args| format!("aid:{}", args[0])));
        sandbox
            .execute(
                "var id = __zw_start_async('99');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
            )
            .unwrap();
        // resolve 前：Promise pending。
        let before = sandbox.execute("typeof globalThis.__result").unwrap();
        assert_eq!(before.value, "undefined");
        // Rust 异步完成 → resolve（shim 的 __zwResolveCallback 触发 + microtask drain）。
        sandbox.resolve_async_callback("aid:99", "resolved!");
        let after = sandbox.execute("globalThis.__result").unwrap();
        assert_eq!(after.value, "resolved!");
    }

    #[test]
    fn test_shim_includes_runtime_stubs() {
        let shim = generate_js_dom_shim();
        assert!(shim.contains("globalThis.setTimeout"));
        assert!(shim.contains("globalThis.navigator"));
        assert!(shim.contains("attachEvent"));
        assert!(shim.contains("__zw_get_page_url"));
        assert!(shim.contains("globalThis.screen"));
        assert!(shim.contains("parentNode"));
        // P1b S1（方案 A）异步回调 resolve 通道 JS 侧契约。
        assert!(shim.contains("globalThis.__zwResolveCallback"));
        assert!(shim.contains("globalThis.__zw_pending"));
    }

    #[test]
    fn test_shim_includes_modern_reftest_stubs() {
        // 现代动态 reftest 的 `requestAnimationFrame(() => …; takeScreenshot())` 模式
        // 要求这两个全局存在，否则 setup mutation 永不执行（R917 未捕获的 yield gap）。
        let shim = generate_js_dom_shim();
        assert!(shim.contains("globalThis.requestAnimationFrame"));
        assert!(shim.contains("globalThis.cancelAnimationFrame"));
        assert!(shim.contains("globalThis.takeScreenshot"));
        // `Element.append(...nodesOrStrings)` 现代 API（区别于 appendChild）。
        assert!(shim.contains("if (prop === 'append')"));
        // `getBoundingClientRect()` 方法必须返回零 DOMRect，否则调用抛 TypeError
        // 中断脚本，使其后的 mutation 丢失（120 reftest 文件用作 reflow 触发器）。
        assert!(shim.contains("if (prop === 'getBoundingClientRect')"));
        // HTML 规范 named access on window（`id="x"` → 全局 `x`，257 reftest 文件）。
        assert!(shim.contains("_installNamedAccess"));
        assert!(shim.contains("__zw_collect_ids"));
        // `createElementNS`（XHTML 命名空间 alias createElement；SVG OOS 不渲染但不中断）。
        assert!(shim.contains("createElementNS:"));
        // `getComputedStyle`：动态 reftest 常作「强制 reflow」触发器调用，缺失则抛
        // ReferenceError 中断脚本丢失后续 mutation。返空 CSSStyleDeclaration 桩不抛。
        assert!(shim.contains("globalThis.getComputedStyle"));
        assert!(shim.contains("getPropertyValue"));
    }

    #[test]
    fn test_merge_style_property() {
        let merged = merge_style_property("color: blue", "width", "10px");
        assert!(merged.contains("color: blue"));
        assert!(merged.contains("width: 10px"));
        let replaced = merge_style_property(&merged, "color", "red");
        assert!(!replaced.contains("blue"));
        assert!(replaced.contains("color: red"));
    }

    #[test]
    fn test_enclosing_form_selector() {
        // P1a form submit：input 在 form 内 → 返 form 的 stable selector。
        let html = "<html><body><form id='f'><input id='i'></form></body></html>";
        assert_eq!(enclosing_form_selector(html, "#i").as_deref(), Some("#f"));
        // input 无 enclosing form → None。
        let no_form = "<html><body><div><input id='i'></div></body></html>";
        assert_eq!(enclosing_form_selector(no_form, "#i"), None);
        // 嵌套：input 在 form 内的 div 内 → 仍解析到 form。
        let nested = "<html><body><form id='outer'><div><input id='deep'></div></form></body></html>";
        assert_eq!(enclosing_form_selector(nested, "#deep").as_deref(), Some("#outer"));
        // 未命中 selector → None。
        assert_eq!(enclosing_form_selector(html, "#missing"), None);
    }

    #[test]
    fn test_is_submit_button() {
        // P1a form submit：submit-button 判定。
        assert!(is_submit_button(
            "<html><body><form><input id='b' type='submit'></form></body></html>",
            "#b",
        ));
        assert!(is_submit_button(
            "<html><body><form><input id='i' type='image'></form></body></html>",
            "#i",
        ));
        // button 默认 type=submit → 提交。
        assert!(is_submit_button(
            "<html><body><form><button id='btn'>Go</button></form></body></html>",
            "#btn",
        ));
        assert!(is_submit_button(
            "<html><body><form><button id='s' type='submit'>Go</button></form></body></html>",
            "#s",
        ));
        // 非提交：
        assert!(!is_submit_button(
            "<html><body><form><input id='t' type='text'></form></body></html>",
            "#t",
        ));
        assert!(!is_submit_button(
            "<html><body><form><button id='nb' type='button'>Go</button></form></body></html>",
            "#nb",
        ));
        assert!(!is_submit_button(
            "<html><body><form><div id='d'>x</div></form></body></html>",
            "#d",
        ));
    }

    #[test]
    fn test_remove_attr_and_has_attribute() {
        // P1a checkbox：RemoveAttr 真正移除属性；has_attribute 判存在性。
        let html = "<html><body><input id='c' type='checkbox' checked></body></html>";
        assert!(has_attribute(html, "#c", "checked"));
        let out = apply_mutations_to_html(
            html,
            &[DomMutation::RemoveAttr {
                selector: "#c".into(),
                name: "checked".into(),
            }],
        )
        .unwrap();
        assert!(!out.contains("checked"));
        assert!(!has_attribute(&out, "#c", "checked"));
        // 无该属性 → has_attribute false。
        assert!(!has_attribute(
            "<html><body><input id='n' type='checkbox'></body></html>",
            "#n",
            "checked",
        ));
    }

    #[test]
    fn test_is_checkbox() {
        assert!(is_checkbox(
            "<html><body><input id='c' type='checkbox'></body></html>",
            "#c",
        ));
        assert!(!is_checkbox(
            "<html><body><input id='t' type='text'></body></html>",
            "#t",
        ));
        assert!(!is_checkbox("<html><body><div id='d'></div></body></html>", "#d",));
    }

    #[test]
    fn test_toggle_radio_html() {
        // P1a radio：toggle target → set checked + 同 name 组兄弟 unset（直接 doc 操作）。
        let html = "<html><body><form>\
            <input id='a' type='radio' name='g' checked>\
            <input id='b' type='radio' name='g'>\
            <input id='c' type='checkbox' checked>\
            </form></body></html>";
        // toggle #b → #b checked、#a unchecked（同 name 组）；#c checkbox 不受影响。
        let out = toggle_radio_html(html, "#b").unwrap();
        assert!(has_attribute(&out, "#b", "checked"));
        assert!(!has_attribute(&out, "#a", "checked"));
        assert!(has_attribute(&out, "#c", "checked"));
        // 非 radio → None。
        assert_eq!(toggle_radio_html(html, "#c"), None);
    }

    #[test]
    fn test_is_text_input() {
        // P1a change-on-blur：文本输入判定（textarea + input 文本类；排除 action 类型）。
        assert!(is_text_input(
            "<html><body><input id='t' type='text'></body></html>",
            "#t",
        ));
        assert!(is_text_input(
            "<html><body><input id='e' type='email'></body></html>",
            "#e",
        ));
        assert!(is_text_input(
            "<html><body><textarea id='ta'></textarea></body></html>",
            "#ta",
        ));
        // input 无 type → 默认 text。
        assert!(is_text_input("<html><body><input id='n'></body></html>", "#n"));
        // action 类型排除（change 在 click 派发）。
        assert!(!is_text_input(
            "<html><body><input id='cb' type='checkbox'></body></html>",
            "#cb",
        ));
        assert!(!is_text_input(
            "<html><body><input id='s' type='submit'></body></html>",
            "#s",
        ));
        assert!(!is_text_input("<html><body><div id='d'></div></body></html>", "#d",));
    }
}
