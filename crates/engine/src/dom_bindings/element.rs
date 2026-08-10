//! Element 特有原生绑定——拆自 mod.rs（RFC §3.2 子模块化 stage 4b，本轮 R3120；§3.2 Element bulk 闭合）。
//!
//! DOM `Element` 接口特有面（spec DOM Standard `Element`）：tagName / id·className getter+setter
//!（reflected attribute，+ 私有反射助手 read_reflected_attr/write_reflected_attr）/ getAttribute·
//! hasAttribute·setAttribute·removeAttribute / children / element 子树作用域 querySelector·querySelectorAll
//! / innerHTML·outerHTML getter。Node 基类面（nodeType/nodeName/nodeValue/textContent/childNodes/
//! 导航/树 mutation/cloneNode/contains）在 node 子模块（R3119）。
//!
//! 可见性：注册于 Element 模板的 getter/invoke 为 `pub(super)`（mod.rs `install_dom_bindings`
//! 注册经 `element::` 调）；`read_reflected_attr` / `write_reflected_attr` 为本模块私有助手。
//! 读 `super::read_node_id` / `super::string_arg` / `super::get_or_create_native_element` /
//! `super::local_value_to_string`（mod.rs 私有共享——Rust 规则：私有项对后代模块可见）+
//! `super::gc::{with_dom, with_dom_mut}`。

use v8;

use zero_dom::{NodeId, NodeKind};

use super::gc::{with_dom, with_dom_mut};
use super::{get_or_create_native_element, local_value_to_string, read_node_id, string_arg};

// ── accessor getter（ZST fn；状态经 gc.rs 线程局部）─────────────────

/// `tagName` getter：读 internal slot[0] NodeId → Element `local_name` → 大写 → `v8::String`。
///
/// 仅 Element 有 tagName（HTML 大写，spec `dom-element-tagname`）；非 Element / stale →
/// undefined。
pub(super) fn native_tag_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let tag: Option<String> = with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name().to_ascii_uppercase()),
            _ => None,
        })
    })
    .flatten();
    let Some(tag) = tag else {
        return;
    };
    if let Some(s) = v8::String::new(scope, &tag) {
        rv.set(s.into());
    }
    // 非 Element / stale → undefined（留 ReturnValue 默认）。
}

/// `id` getter（reflected attribute，spec `dom-id`）：`get_attribute('id')`，缺省 `""`。
pub(super) fn native_id_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    read_reflected_attr(scope, &args, "id", "", &mut rv);
}

/// `className` getter（reflected attribute，spec `dom-classname`）：`get_attribute('class')`，缺省 `""`。
pub(super) fn native_class_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    read_reflected_attr(scope, &args, "class", "", &mut rv);
}

/// 反射属性 getter 共用：读 internal slot NodeId → `Document::get_attribute(name)`，
/// 缺省 `default`（reflected 属性缺省 `""`）。
fn read_reflected_attr(
    scope: &mut v8::PinScope,
    args: &v8::PropertyCallbackArguments,
    attr: &str,
    default: &str,
    rv: &mut v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = with_dom(|d| d.get_attribute(id, attr))
        .flatten()
        .unwrap_or_else(|| default.to_string());
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// `id` setter（reflected，spec `dom-id`）：值 ToString 后 `set_attribute('id', val)`。
/// 经 [`with_dom_mut`] 写真实 DOM（更新 id_map）。
pub(super) fn native_id_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    write_reflected_attr(scope, &args, "id", value);
}

/// `className` setter（reflected，spec `dom-classname`）：值 ToString 后 `set_attribute('class', val)`。
pub(super) fn native_class_name_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    write_reflected_attr(scope, &args, "class", value);
}

/// 反射属性 setter 共用：读 internal slot NodeId → 值 ToString → `Document::set_attribute(name, val)`。
fn write_reflected_attr(
    scope: &mut v8::PinScope,
    args: &v8::PropertyCallbackArguments,
    attr: &str,
    value: v8::Local<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(id, attr, &val));
}

// ── 方法回调（Element 上：getAttribute / hasAttribute / setAttribute / removeAttribute）──

/// `getAttribute(name)`：读 internal slot NodeId → `Document::get_attribute`。
/// spec `dom-element-getattribute`：缺省/非 Element → `null`。
pub(super) fn native_get_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let val = with_dom(|d| d.get_attribute(id, &name)).flatten();
    match val {
        Some(v) => {
            if let Some(s) = v8::String::new(scope, &v) {
                rv.set(s.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `hasAttribute(name)`：读 internal slot NodeId → `Document::has_attribute` → `v8::Boolean`。
/// spec `dom-element-hasattribute`。
pub(super) fn native_has_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let has = with_dom(|d| d.has_attribute(id, &name)).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, has).into());
}

/// `setAttribute(name, value)`：读 internal slot NodeId → 两参 ToString →
/// `Document::set_attribute`（更新 id_map 当 name=='id'）。spec `dom-element-setattribute`
/// 返 `undefined`（留 ReturnValue 默认）。非 native element `this` → no-op。
pub(super) fn native_set_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let value = string_arg(scope, &args, 1);
    with_dom_mut(|d| d.set_attribute(id, &name, &value));
}

/// `removeAttribute(name)`：读 internal slot NodeId → `Document::remove_attribute`（name=='id'
/// 时清 id_map）。spec `dom-element-removeattribute` 返 `undefined`。
pub(super) fn native_remove_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    with_dom_mut(|d| d.remove_attribute(id, &name));
}

/// `toggleAttribute(name, force?)`：spec `dom-element-toggleattribute`——切换属性存在性，返切换后是否在。
/// force 缺省：在→移除返 false、不在→设空串 `""` 返 true（toggle 语义）；force 给定：force=true →
/// 确保在（不在则设 `""`）返 true、force=false → 确保移除（在则移除）返 false。
/// 经 [`with_dom_mut`] `has_attribute` + `set_attribute("")` / `remove_attribute`；幂等（已目标态不冗余写）。
/// 注：toggleAttribute 添加属性时值为 `""`（spec），非 `"true"` 或属性名（区别于旧 HTML boolean attr）。
pub(super) fn native_toggle_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let force_defined = !args.get(1).is_undefined();
    let force = force_defined && args.get(1).boolean_value(scope);
    let now_present = with_dom_mut(|d| {
        let has = d.has_attribute(id, &name);
        // force 缺省 → toggle（want = !has）；force 给定 → want = force（ensure present/absent）。
        let want = if force_defined { force } else { !has };
        if want {
            if !has {
                d.set_attribute(id, &name, ""); // 添加时值为空串（spec）
            }
            true
        } else {
            if has {
                d.remove_attribute(id, &name);
            }
            false
        }
    })
    .unwrap_or(false);
    rv.set(v8::Boolean::new(scope, now_present).into());
}

/// `hasAttributes()`（spec `dom-element-hasattributes`）：元素是否有任意属性（经
/// `Document::attribute_names` 非空判定）→ `v8::Boolean`。
pub(super) fn native_has_attributes_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let has = with_dom(|d| !d.attribute_names(id).is_empty()).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, has).into());
}

/// `getAttributeNames()`（spec `dom-element-getattributenames`）：元素全部属性名（文档序）→
/// V8 Array of 字符串（空属性集 → 空 Array）。经 `Document::attribute_names`。
pub(super) fn native_get_attribute_names_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let names: Vec<String> = with_dom(|d| d.attribute_names(id)).unwrap_or_default();
    let arr = v8::Array::new(scope, names.len() as i32);
    for (i, name) in names.into_iter().enumerate() {
        if let Some(s) = v8::String::new(scope, &name) {
            let _ = arr.set_index(scope, i as u32, s.into());
        }
    }
    rv.set(arr.into());
}

// ── 元素子树作用域查询（element.querySelector(-all)，注册于 Element 模板）──
// 区别于 global `__zw_native_query_selector(-all)`（factories 子模块，R3118）。

/// `element.querySelector(sel)`：spec `dom-parentnode-queryselector`（**元素子树作用域**）——
/// 元素**后代**中首个匹配 → native 对象。区别于文档级 [`factories::native_query_selector_invoke`]：
/// root = `args.this()` 元素 NodeId，且**排除元素自身**（dom `query_selector_all` 含 root 候选，
/// spec descendants-only，镜像 polyfill `query_match_in_subtree` 的 `.filter(|n| *n != root)`）。
///
/// 经 `query_selector_all` + filter + first 取首个后代（比 polyfill `query_selector` + filter 更
/// 正确：后者若元素自身匹配则返 None，本实现继续找首个后代）。
/// OPTIMIZATION: 当前 collect-all-then-first；超大子树可短路（find_first_matching 跳 root）。
/// 无匹配 / 空 / 非法 → `null`；非 native element `this` → `undefined`（getter 一致）。
pub(super) fn native_element_query_selector_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(root) = read_node_id(scope, &this) else {
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let id: Option<NodeId> = with_dom(|d| d.query_selector_all(root, sel.trim()))
        .unwrap_or_default()
        .into_iter()
        .find(|id| *id != root);
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `element.querySelectorAll(sel)`：spec `dom-parentnode-queryselectorall`（**元素子树作用域**）——
/// 元素**后代**全部匹配 → V8 `Array` of native 对象（文档序，排除元素自身）。非 native element
/// `this` → 空 `Array`（避免 `.length` 访问报错）。
pub(super) fn native_element_query_selector_all_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(root) = read_node_id(scope, &this) else {
        rv.set(v8::Array::new(scope, 0).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(root, sel.trim()))
        .unwrap_or_default()
        .into_iter()
        .filter(|id| *id != root)
        .collect();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `element.matches(selector)`（spec `dom-element-matches`）：本元素是否匹配选择器 → bool。
/// 复合选择器（tag/`*`/`#id`/`.class`/`[attr]`+运算符/伪类）；组合器不支持（best-effort，
/// 见 `Document::matches`）。非 native element `this` → false（spec matches 仅 Element）。
pub(super) fn native_element_matches_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let m = with_dom(|d| d.matches(id, sel.trim())).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, m).into());
}

/// `element.closest(selector)`（spec `dom-element-closest`）：本元素 + 祖先链首个匹配选择器的节点
/// → native 元素或 `null`。复合选择器（同 [`native_element_matches_invoke`]）；无匹配 / 非法 → `null`。
pub(super) fn native_element_closest_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let found = with_dom(|d| d.closest(id, sel.trim())).flatten();
    match found {
        Some(fid) => {
            if let Some(obj) = get_or_create_native_element(scope, fid) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `children` getter（spec `dom-parentnode-children`）：元素**子元素**（跳过文本/注释）
/// → V8 Array of native 对象（文档序）。非 Element 子节点不返（native 仅 Element 对象；
/// `childNodes` 含文本/注释需 native 非 Element 节点，后续切片）。
pub(super) fn native_children_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let child_ids: Vec<NodeId> = with_dom(|d| {
        d.child_nodes(id)
            .into_iter()
            .filter(|c| d.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
            .collect()
    })
    .unwrap_or_default();
    let arr = v8::Array::new(scope, child_ids.len() as i32);
    for (i, cid) in child_ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, cid) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

// ── innerHTML / outerHTML 序列化 getter（spec `dom-element-innerhtml` / `-outerhtml`）──

/// `innerHTML` getter（spec `dom-element-innerhtml`）：子节点 `outer_html` 拼接（markup 序列化）。
pub(super) fn native_inner_html_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = with_dom(|d| {
        let mut s = String::new();
        for child in d.child_nodes(id) {
            s.push_str(&d.outer_html(child));
        }
        s
    })
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &html) {
        rv.set(s.into());
    }
}

/// `outerHTML` getter（spec `dom-element-outerhtml`）：本元素 `outer_html`（含自身 tag）。
pub(super) fn native_outer_html_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = with_dom(|d| d.outer_html(id)).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &html) {
        rv.set(s.into());
    }
}

// ── innerHTML / outerHTML setter（spec `dom-element-innerhtml` / `-outerhtml`，R3123）──
//
// 闭合 R3113 getter-only 限制：复用 polyfill 路径既有 fragment parse + 子树深拷贝（
// `crate::js_dom_bridge::replace_inner_html` / `replace_outer_html_node`），经 [`with_dom_mut`]
// 写真实 live Document。native 写触发重渲染由 webview `sync_render_after_native_dom`（R3108）
// 拾取——live Document 序列化后与 cached_html 比，变则重绘。
//
// 错误：outerHTML setter 对无父元素（文档根）抛 DOMException 是 spec 行为，但 native 路径尚无
// 异常传递（RFC §3.4 后续）；当前失败记 tracing::warn + no-op（不静默吞，便于排障）。innerHTML
// 极少失败（仅 remove/append 节点级错误）。

/// `innerHTML` setter（spec `dom-element-innerhtml`）：值 ToString 后清空元素现有子节点，
/// 解析 HTML 片段，深拷贝顶层节点追加。复用 [`crate::js_dom_bridge::replace_inner_html`]。
pub(super) fn native_inner_html_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = local_value_to_string(scope, value);
    let res = with_dom_mut(|d| crate::js_dom_bridge::replace_inner_html(d, id, &html));
    if let Some(Err(e)) = res {
        tracing::warn!(error = %e, "native innerHTML setter failed");
    }
}

/// `outerHTML` setter（spec `dom-element-outerhtml`）：值 ToString 后把元素整体替换为解析的
/// 片段顶层节点（在父节点中、目标位置前逐个插入，再移除目标自身）。需父节点（文档根无父 →
/// 失败）。复用 [`crate::js_dom_bridge::replace_outer_html_node`]。
pub(super) fn native_outer_html_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = local_value_to_string(scope, value);
    let res = with_dom_mut(|d| crate::js_dom_bridge::replace_outer_html_node(d, id, &html));
    if let Some(Err(e)) = res {
        tracing::warn!(error = %e, "native outerHTML setter failed");
    }
}
