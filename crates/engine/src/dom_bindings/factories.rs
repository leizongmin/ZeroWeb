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

use zero_dom::NodeId;

use super::gc::{with_dom, with_dom_mut};
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
pub(super) fn native_create_element_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let mut tag = string_arg(scope, &args, 0);
    if tag.trim().is_empty() {
        tag = "div".to_string();
    }
    // create（borrow_mut 释放后）→ 包 native 对象（get_or_create_native_element 内含 stale 校验）。
    let Some(id) = with_dom_mut(|d| d.create_element(tag.trim())) else {
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
/// 的 textContent；无 `<title>` → 空串。
pub(super) fn native_get_document_title_invoke(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let title = with_dom(|d| {
        d.get_elements_by_tag_name("title")
            .into_iter()
            .next()
            .and_then(|id| d.text_content(id))
    })
    .flatten()
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &title) {
        rv.set(s.into());
    }
}

/// `__zw_native_set_document_title(str)`：spec `dom-document-title` setter——
/// ① 存在 `<title>` → 改其 textContent（清子 + 加文本节点，镜像 Node textContent setter）；
/// ② 不存在 → 在 `<head>` 建 `<title>` 设文本（无 `<head>` 不创建，best-effort——html5ever 总归一化有 head）。
pub(super) fn native_set_document_title_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let val = string_arg(scope, &args, 0);
    with_dom_mut(|d| {
        let title_id = d.get_elements_by_tag_name("title").into_iter().next();
        if let Some(tid) = title_id {
            // 存在 → 改 textContent（清子 + 加文本节点）。
            let children = d.child_nodes(tid);
            for c in children {
                let _ = d.remove_child(tid, c);
            }
            if !val.is_empty() {
                let text_id = d.create_text_node(&val);
                let _ = d.append_child(tid, text_id);
            }
        } else {
            // 不存在 → 在 <head> 建 <title>（无 head 不创建）。
            let head_id = d.get_elements_by_tag_name("head").into_iter().next();
            if let Some(hid) = head_id {
                let title = d.create_element("title");
                if !val.is_empty() {
                    let text_id = d.create_text_node(&val);
                    let _ = d.append_child(title, text_id);
                }
                let _ = d.append_child(hid, title);
            }
        }
    });
}
