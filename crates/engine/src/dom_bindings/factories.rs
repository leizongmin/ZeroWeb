//! 全局工厂回调——拆自 mod.rs（RFC §3.2 子模块化 stage 3，本轮 R3118；createText/Comment/Fragment R3131；
//! documentElement/body/head R3136；getElementsByTagName R3137；getElementsByClassName R3138；
//! document.title getter/setter R3139）。
//!
//! 14 个**全局**工厂（注册于 `ctx.global`，非 Element 模板成员）：
//! - `__zw_native_element_for_id(idStr)`：`get_element_by_id` → native 元素；
//! - `__zw_native_query_selector(sel)` / `__zw_native_query_selector_all(sel)`：文档根下
//!   全量选择器引擎匹配 → native 元素 / V8 Array（spec `dom-parentnode-queryselector(-all)`）；
//! - `__zw_native_create_element(tag)`：`Document::create_element` → native 元素（未挂载）；
//! - `__zw_native_create_text_node(text)` / `__zw_native_create_comment(text)` /
//!   `__zw_native_create_document_fragment()`（R3131）：造 Text(3)/Comment(8)/Fragment(11) 节点 →
//!   native 对象（未挂载），闭合 native 树构建集（createElement + createText/Comment/Fragment + appendChild）；
//! - `__zw_native_get_document_element()` / `__zw_native_get_body()` / `__zw_native_get_head()`
//!   （R3136）：文档级只读属性 → native 元素或 `null`（spec `dom-document-(documentelement|body|head)`）；
//! - `__zw_native_get_elements_by_tag_name(name)`（R3137）：文档根下按标签名（`*` 通配）收集 →
//!   V8 Array of native 对象（spec `dom-document-getelementsbytagname`）；
//! - `__zw_native_get_elements_by_class_name(name)`（R3138）：空格分隔类名列表（含全部类）收集 →
//!   V8 Array of native 对象（spec `dom-document-getelementsbyclassname`，多类 spec 合规）；
//! - `__zw_native_get_document_title()` / `__zw_native_set_document_title(str)`（R3139）：读/写
//!   首个 `<title>` 元素 textContent（不存在时 setter 在 `<head>` 建 `<title>`；spec `dom-document-title`）。
//!
//! 区别于 mod.rs `native_element_query_selector(-all)_invoke`（**元素子树作用域**，注册于 Element
//! 模板，root = `args.this()` 元素 + 排除自身）——本模块的是**文档级**（root = 文档根）。
//!
//! 可见性：14 个 invoke 为 `pub(super)`（mod.rs `install_dom_bindings` 注册经 `factories::` 调）。
//! 读 `super::string_arg` / `super::get_or_create_native_element`（mod.rs 私有——Rust 规则：私有项
//! 对后代模块可见）+ `super::gc::{with_dom, with_dom_mut}`。

use v8;

use zero_dom::{Document, NodeId};

use super::gc::{active_element, clear_upgrade_node_id, set_upgrade_node_id, with_dom, with_dom_mut};
use super::{get_or_create_native_element, string_arg};

/// 工厂回调：`__zw_native_element_for_id(idStr)` → 解析 `get_element_by_id` →
/// NodeId → 创建/查找 native element 对象（NodeId↔对象身份映射 + stale 重建）。
///
/// 未找到 id → `null`。NodeId 编码进 internal slot[0]（`v8::External` ptr 值）。
pub(super) fn native_element_factory_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id_str = args
        .get(0)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default();
    let Some(node_id) = with_dom(|d| d.get_element_by_id(&id_str)).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, node_id) {
        rv.set(obj.into());
    }
    // 无 Element 模板（未安装）→ undefined（防御，正常路径模板已 set）。
}

/// `__zw_native_query_selector(sel)`：spec `dom-parentnode-queryselector`——
/// 文档根下按**全量选择器引擎**（[`zero_dom::Document::query_selector`]，消费 tag/`*`/
/// `#id`/`.class`/`[attr]`+运算符/伪类/组合器）找首个匹配元素 → native 对象。
///
/// 无匹配 / 空 / 非法选择器 → `null`（`parse_selector_chain` 失败返 `None`，无 panic）。
pub(super) fn native_query_selector_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let Some(id) = with_dom(|d| d.query_selector(d.root(), sel.trim())).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_query_selector_all(sel)`：spec `dom-parentnode-queryselectorall`——
/// 文档根下按全量选择器引擎（[`zero_dom::Document::query_selector_all`]）收集全部匹配
/// 元素 → V8 `Array` of native 对象（文档序）。空 / 非法选择器 → 空 `Array`。
pub(super) fn native_query_selector_all_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(d.root(), sel.trim())).unwrap_or_default();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `__zw_native_create_element(tag)`：spec `dom-document-createelement`——
/// `Document::create_element(tag)` 造新 Element NodeId → native 对象（**未挂载**，需 appendChild）。
/// 空/缺省 tag → `div`（与 polyfill create_element 一致，spec 实际应抛，本切片 best-effort）。
///
/// **S5b upgrade 分支**（R3265）：tag 命中 polyfill customElements registry（经全局 JS
/// `__zw_native_ce_lookup(tag)` 反查，返 registered ctor 或 null）→ 设 [`gc::upgrade_node_id`]（host
/// 已建元素 NodeId）→ 调 registered ctor `new_instance`（super() → [`native_html_element_ctor_invoke`]
/// 读 thread-local NodeId 填 slot[0]）→ 清 thread-local → 返 native custom 实例。未命中 registry → 既有
/// generic Element 模板路径。registry 反查失败 / ctor 不可调用 / `new_instance` 失败 → 回退 generic 路径
///（不抛，best-effort 保 createElement 永不失败）。
pub(super) fn native_create_element_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let mut tag = string_arg(scope, &args, 0);
    if tag.trim().is_empty() {
        tag = "div".to_string();
    }
    let tag_trim = tag.trim().to_string();
    // create（borrow_mut 释放后）。host 先建元素得 NodeId（custom 元素的 tag = 'my-el' 原样保留）。
    let Some(id) = with_dom_mut(|d| d.create_element(&tag_trim)) else {
        return;
    };
    // S5b：tag 命中 customElements registry → upgrade（Reflect.construct registered ctor 经 super() 复用
    // 本 NodeId）。registry 反查 / ctor 调用任何失败 → 回退 generic Element 路径（best-effort 不抛）。
    if let Some(custom) = try_upgrade_custom_element(scope, id, &tag_trim) {
        rv.set(custom.into());
        return;
    }
    // 包 native 对象（get_or_create_native_element 内含 stale 校验）。
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// S5b（R3265）custom element upgrade：反查 polyfill `_ce_registry` 命中则调 registered ctor
/// `new_instance`，super() 经 [`html_element::native_html_element_ctor_invoke`] 读 thread-local
/// [`gc::upgrade_node_id`] 复用 host NodeId。失败返 `None`（调用方回退 generic Element 路径）。
fn try_upgrade_custom_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    id: NodeId,
    tag: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    // 反查 polyfill registry：全局 __zw_native_ce_lookup(tag) → ctor 或 null。
    let context = scope.get_current_context();
    let global = context.global(scope);
    let lookup_key = v8::String::new(scope, "__zw_native_ce_lookup")?;
    let lookup_val = global.get(scope, lookup_key.into())?;
    let Ok(lookup) = v8::Local::<v8::Function>::try_from(lookup_val) else {
        return None; // polyfill 未注册 lookup（native_dom 模式 shim 未加载等）→ 无 upgrade。
    };
    let tag_str = v8::String::new(scope, tag)?;
    let ctor_val = lookup.call(scope, global.into(), &[tag_str.into()]);
    // null/undefined/缺失 → 未注册（普通元素）。
    let ctor_val = ctor_val.filter(|v| !v.is_null_or_undefined())?;
    let Ok(ctor) = v8::Local::<v8::Function>::try_from(ctor_val) else {
        return None;
    };
    // 设 upgrade 注入槽 → super() 复用本 NodeId。失败必须清（防泄漏到后续 new HTMLElement）。
    set_upgrade_node_id(Some(id));
    let instance = ctor.new_instance(scope, &[]);
    clear_upgrade_node_id();
    instance
}

/// `document.createElementNS(ns, qualifiedName)`（spec `dom-document-createelementns`）：造带命名空间
/// 的新 Element（SVG/MathML 编程创建高频）→ native 元素（**未挂载**）。两参 ToString 后调
/// [`Document::create_element_ns`]（解析 prefix:local + 建 QualName）。空 ns/qualifiedName best-effort。
pub(super) fn native_create_element_ns_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let ns = string_arg(scope, &args, 0);
    let qualified = string_arg(scope, &args, 1);
    let Some(id) = with_dom_mut(|d| d.create_element_ns(ns.trim(), qualified.trim())) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_text_node(text)`：spec `dom-document-createtextnode`——
/// `Document::create_text_node` 造 Text NodeId → native 对象（nodeType=3，**未挂载**）。Text/Comment/
/// Fragment 共用 Element 包装模板（nodeType getter 经 NodeKind 读 DOM → 3/8/11，非 Element 亦正确）。
pub(super) fn native_create_text_node_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let text = string_arg(scope, &args, 0);
    let Some(id) = with_dom_mut(|d| d.create_text_node(&text)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_comment(text)`：spec `dom-document-createcomment`——
/// `Document::create_comment` 造 Comment NodeId → native 对象（nodeType=8，**未挂载**）。
pub(super) fn native_create_comment_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let text = string_arg(scope, &args, 0);
    let Some(id) = with_dom_mut(|d| d.create_comment(&text)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_create_document_fragment()`：spec `dom-document-createdocumentfragment`——
/// `Document::create_document_fragment` 造 DocumentFragment NodeId → native 对象（nodeType=11，**未挂载**）。
/// appendChild/insertBefore fragment 经 Document 自动 flatten 子节点（spec：insert fragment 等价插其子）。
pub(super) fn native_create_document_fragment_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let Some(id) = with_dom_mut(|d| d.create_document_fragment()) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `document.importNode(node, deep)`（spec `dom-document-importnode`）：导入（克隆）外部节点。
/// headless 单文档语义 ≈ [`clone_node`]——克隆节点（deep=true 递归克隆子树），返新 native 元素
///（未挂载，需 appendChild）。**模板实例化高频**：`document.importNode(template.content.firstChild, true)`
/// 是 `<template>` 内容实例化标准手段。复用 [`Document::clone_node`]（同 cloneNode 底层）。
/// 非节点参（null/undefined/非 native）→ `null`（best-effort，spec 应抛 NotSupportedError）。
pub(super) fn native_import_node_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let Some(id) = super::node_id_from_value(scope, args.get(0)) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let deep = args.get(1).boolean_value(scope);
    let Some(new_id) = with_dom_mut(|d| d.clone_node(id, deep)) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, new_id) {
        rv.set(obj.into());
    }
}

/// `document.adoptNode(node)`（spec `dom-document-adoptnode`）：headless 单文档语义 = identity——
/// 节点已在本文档（headless 唯一 document），adopt 为 no-op 返原节点（同 NodeId → 同对象身份）。
/// 跨文档 adopt（多 document）在 headless 单文档不显现。非节点参 → `null`。
pub(super) fn native_adopt_node_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let Some(id) = super::node_id_from_value(scope, args.get(0)) else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

// ── R3136 文档级只读属性工厂（documentElement / body / head）──

/// `__zw_native_get_document_element()`：spec `dom-document-documentelement`——文档根元素
///（Document 节点的首个 Element 子节点；HTML 文档为 <html>）。返 native 元素或 `null`（无根元素）。
pub(super) fn native_get_document_element_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    // spec 定义：root element = Document 节点（`d.root()`）的首个 Element 子节点。
    let id = with_dom(|d| d.child_nodes(d.root()).into_iter().find(|&c| d.node_type(c) == Some(1))).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `__zw_native_get_body()`：spec `dom-document-body`——文档的 `<body>` 元素（首个 body 元素）。
/// 返 native 元素或 `null`（无 body）。
pub(super) fn native_get_body_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("body").into_iter().next()).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `__zw_native_get_head()`：spec `dom-document-head`——文档的 `<head>` 元素（首个 head 元素）。
/// 返 native 元素或 `null`（无 head）。
pub(super) fn native_get_head_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let id = with_dom(|d| d.get_elements_by_tag_name("head").into_iter().next()).flatten();
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `__zw_native_get_elements_by_tag_name(name)`：spec `dom-document-getelementsbytagname`——
/// 文档根下按标签名（**大小写不敏感**，HTML 元素）收集全部匹配元素 → V8 `Array` of native 对象
///（文档序 / DFS pre-order）。`name="*"` 匹配**全部元素**（spec 通配）；空 / 无匹配 → 空 `Array`。
///
/// 经 `get_elements_by_tag_name_ns(None, name)`——namespace=None 匹配任意命名空间 + 内置 `*` 通配，
/// 统一处理 `*` 与具体 tag（无需分支；HTML 元素均同 namespace）。
pub(super) fn native_get_elements_by_tag_name_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let name = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.get_elements_by_tag_name_ns(None, name.trim())).unwrap_or_default();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `__zw_native_get_elements_by_class_name(name)`：spec `dom-document-getelementsbyclassname`——
/// `name` 为**空格分隔类名列表**，返文档根下含【全部】指定类的元素 → V8 `Array` of native 对象（文档序）。
/// 空串 / 全空白 → 空 `Array`。
///
/// **多类 spec 合规**：dom `get_elements_by_class_name` 仅做单 token 精确匹配（whole-string 当单类名），
/// 不支持 spec「空格分隔多类 = 含全部」。本工厂 split_ascii_whitespace 取 tokens：候选 = 含首 token 的
/// 元素（复用 `get_elements_by_class_name`，root-scoped 文档序），多 token 时按剩余 token 过滤（读
/// `class` 属性 split 比对）。单 token（常见）无过滤开销。
pub(super) fn native_get_elements_by_class_name_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let name = string_arg(scope, &args, 0);
    let tokens: Vec<&str> = name.split_ascii_whitespace().collect();
    let ids: Vec<NodeId> = with_dom(|d| {
        if tokens.is_empty() {
            return Vec::new();
        }
        // 候选 = 含首 token 的元素（get_elements_by_class_name 精确 token 匹配，root-scoped 文档序）。
        let mut candidates = d.get_elements_by_class_name(tokens[0]);
        // 多 token：保留含【全部】剩余 token 的候选（读 class 属性 split 比对）。
        if tokens.len() > 1 {
            let rest = &tokens[1..];
            candidates.retain(|&id| {
                let classes = d.get_attribute(id, "class").unwrap_or_default();
                let set: Vec<&str> = classes.split_ascii_whitespace().collect();
                rest.iter().all(|t| set.contains(t))
            });
        }
        candidates
    })
    .unwrap_or_default();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

// ── R3139 document.title getter/setter ──

/// `__zw_native_get_document_title()`：spec `dom-document-title` getter——读首个 `<title>` 元素
/// 的 textContent；无 `<title>` → 空串。经共享 [`read_document_title`]（R3160 抽出，`document.title`
/// getter 共用）。
pub(super) fn native_get_document_title_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let title = with_dom(read_document_title).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &title) {
        rv.set(s.into());
    }
}

/// 读文档 title：首个 `<title>` 元素的 textContent；无 `<title>` → 空串。[`with_dom`] 闭包用，
/// 工厂 getter 与 `document.title` getter（document 子模块）共用（R3160 抽出，DRY）。
pub(super) fn read_document_title(d: &Document) -> String {
    d.get_elements_by_tag_name("title")
        .into_iter()
        .next()
        .and_then(|id| d.text_content(id))
        .unwrap_or_default()
}

/// `__zw_native_set_document_title(str)`：spec `dom-document-title` setter——经共享
/// [`write_document_title`]（R3160 抽出，`document.title` setter 共用）。
pub(super) fn native_set_document_title_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let val = string_arg(scope, &args, 0);
    with_dom_mut(|d| write_document_title(d, &val));
}

/// 写文档 title：① 存在 `<title>` → 改其 textContent（清子 + 加文本节点，镜像 Node textContent setter）；
/// ② 不存在 → 在 `<head>` 建 `<title>` 设文本（无 `<head>` 不创建，best-effort——html5ever 总归一化有 head）。
/// [`with_dom_mut`] 闭包用，工厂 setter 与 `document.title` setter（document 子模块）共用（R3160 抽出，DRY）。
pub(super) fn write_document_title(d: &mut Document, val: &str) {
    let title_id = d.get_elements_by_tag_name("title").into_iter().next();
    if let Some(tid) = title_id {
        // 存在 → 改 textContent（清子 + 加文本节点）。
        let children = d.child_nodes(tid);
        for c in children {
            let _ = d.remove_child(tid, c);
        }
        if !val.is_empty() {
            let text_id = d.create_text_node(val);
            let _ = d.append_child(tid, text_id);
        }
    } else {
        // 不存在 → 在 <head> 建 <title>（无 head 不创建）。
        let head_id = d.get_elements_by_tag_name("head").into_iter().next();
        if let Some(hid) = head_id {
            let title = d.create_element("title");
            if !val.is_empty() {
                let text_id = d.create_text_node(val);
                let _ = d.append_child(title, text_id);
            }
            let _ = d.append_child(hid, title);
        }
    }
}

// ── R3148 document.activeElement 工厂 ──

/// `__zw_native_get_active_element()`：spec `dom-document-activeelement`——当前焦点元素（`element.focus()`
/// 设、`element.blur()` 清；gc.rs `ACTIVE_ELEMENT` 线程局部）。返 native 元素或 `null`（无焦点）。
/// **已知限制**：spec 无焦点时返 `<body>`（或根），本切片返 `null`（headless 简化，同 polyfill）。
/// 工厂暴露（非 native `document` 对象 getter——native document 对象为后续切片；shim 可经此读 active）。
pub(super) fn native_get_active_element_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    match active_element() {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}
