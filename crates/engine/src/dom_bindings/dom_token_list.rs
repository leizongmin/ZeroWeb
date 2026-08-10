//! R3145 DOMTokenList（`element.classList` 集合）原生绑定——拆自 mod.rs（RFC §3.2 子模块化）。
//!
//! 内聚 class 集合面：`classList` getter（Element 模板注册）+ DOMTokenList 模板（length /
//! value(+setter) / item / contains / add / remove / toggle(force?) / replace / toString）。
//! internal slot[0] = owner element NodeId；**live**——每次读经 owner 元素当前 `class` 属性
//! `split_whitespace`，mutation 经 `set_attribute("class", joined)` 写回（dom crate node.class_list
//! 自动同步）。身份缓存 DOMTOKENLIST_OBJECTS（同元素返同对象，spec identity——polyfill 旧每调新建，
//! native 修正为 spec 合规）。
//!
//! 镜像 NNM/Attr（R3112/R3122/R3134）live-collection-tied-to-element 模式：weak 身份缓存（R3134 同 pattern，
//! JS 丢引用即 GC，rebuild insert-overwrite 清死 Weak）。
//!
//! 可见性：`native_class_list_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs 的
//! Element 模板注册 + install_dom_bindings 调）；余私有（本模块内部用）。读 `super::read_node_id` /
//! `super::string_arg` / `super::local_value_to_string`（mod.rs 私有——Rust 规则：私有项对后代模块可见）+
//! `super::gc::{...}`。

use v8;

use zero_dom::NodeId;

use super::gc::{
    cache_domtokenlist, cached_domtokenlist, decode_node_id, domtokenlist_template_local, encode_node_id,
    set_domtokenlist_template, with_dom, with_dom_mut,
};
use super::{local_value_to_string, read_node_id, string_arg};

/// 从 DOMTokenList 对象 internal slot[0] 读 owner element NodeId。slot[0]=owner ffi（External ptr 值）。
/// 非 DTL 对象 / stale → `None`。
fn dtl_owner(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<NodeId> {
    let ffi = obj.get_internal_field(scope, 0)?.cast::<v8::External>().value() as usize as u64;
    Some(decode_node_id(ffi))
}

/// 读 owner 元素当前 class tokens（live——经 `class` 属性 `split_whitespace`，与 dom crate node.class_list
/// 解析一致）。无 class 属性 → 空 Vec。
fn current_tokens(doc: &zero_dom::Document, id: NodeId) -> Vec<String> {
    doc.get_attribute(id, "class")
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default()
}

/// 把 tokens 用单空格 join 写回 owner 元素 `class` 属性（空 list → `class=""`，spec：移除全部 token 后
/// 属性为空串，非删除属性；dom crate set_attribute 自动重解析 node.class_list）。
fn write_tokens(doc: &mut zero_dom::Document, id: NodeId, tokens: &[String]) {
    let joined = tokens.join(" ");
    doc.set_attribute(id, "class", &joined);
}

/// spec DOMTokenList token 校验：token 须非空且不含空白。非法 → 抛 TypeError（headless 无 DOMException，
/// 同 polyfill 取最接近可获取异常类型）并返 `false`；合法返 `true`（调用方早退）。
fn require_valid_token(scope: &mut v8::PinScope, token: &str) -> bool {
    if token.is_empty() || token.chars().any(|c| c.is_whitespace()) {
        if let Some(msg) = v8::String::new(
            scope,
            "An invalid or illegal string was specified (token must not be empty or contain whitespace).",
        ) {
            let exc = v8::Exception::type_error(scope, msg);
            scope.throw_exception(exc);
        }
        false
    } else {
        true
    }
}

/// `classList` getter（spec `dom-element-classlist`）：返 owner 元素的 DOMTokenList（缓存保身份）。
pub(super) fn native_class_list_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let ffi = encode_node_id(id);
    // 缓存命中 → 同对象（spec `el.classList === el.classList`）。
    if let Some(cached) = cached_domtokenlist(scope, ffi) {
        rv.set(cached.into());
        return;
    }
    // 实例化 DOMTokenList 模板 + internal slot[0] = owner element NodeId。
    let Some(tmpl) = domtokenlist_template_local(scope) else {
        return;
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_domtokenlist(scope, ffi, obj);
    rv.set(obj.into());
}

/// DOMTokenList `length` getter（spec `dom-domtokenlist-length`）：owner 元素当前 class token 数。
fn native_dtl_length_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = dtl_owner(scope, &holder) else {
        rv.set(v8::Integer::new(scope, 0).into());
        return;
    };
    let len = with_dom(|d| current_tokens(d, id).len()).unwrap_or(0);
    rv.set(v8::Integer::new(scope, len as i32).into());
}

/// DOMTokenList `value` getter（spec `dom-domtokenlist-value`）：owner 元素当前 `class` 属性串（live）。
fn native_dtl_value_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = dtl_owner(scope, &holder) else {
        return;
    };
    let val = with_dom(|d| d.get_attribute(id, "class")).flatten().unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// DOMTokenList `value` setter（spec `dom-domtokenlist-value`）：值 ToString → `set_attribute("class", val)`
///（整体替换，无 token 校验——value 可为任意串）。经 [`with_dom_mut`] 写真实 DOM。
fn native_dtl_value_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(id) = dtl_owner(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(id, "class", &val));
}

/// DOMTokenList `item(index)`（spec `dom-domtokenlist-item`）：index 处 token → 字符串或 null（越界）。
fn native_dtl_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let idx = args.get(0).integer_value(scope).unwrap_or(-1);
    let tokens = with_dom(|d| current_tokens(d, id)).unwrap_or_default();
    match tokens.get(idx as usize) {
        Some(t) => {
            if let Some(s) = v8::String::new(scope, t) {
                rv.set(s.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// DOMTokenList `contains(token)`（spec `dom-domtokenlist-contain`）：token 非法 → 抛；否则返是否含。
fn native_dtl_contains_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let token = string_arg(scope, &args, 0);
    if !require_valid_token(scope, &token) {
        return;
    }
    let has = with_dom(|d| current_tokens(d, id).iter().any(|t| t == &token)).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, has).into());
}

/// DOMTokenList `add(...tokens)`（spec `dom-domtokenlist-add`，variadic）：逐 token 校验 → 去重追加 → 写回。
/// 任一 token 非法 → 抛（已校验通过的 token 不写入，spec 原子性：校验全部先于 mutation）。
fn native_dtl_add_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        return;
    };
    let n = args.length();
    // spec：先校验全部 token（任一非法即抛，不部分写入），再 mutation。
    let mut to_add: Vec<String> = Vec::with_capacity(n.max(0) as usize);
    for i in 0..n {
        let t = string_arg(scope, &args, i);
        if !require_valid_token(scope, &t) {
            return;
        }
        to_add.push(t);
    }
    with_dom_mut(|d| {
        let mut tokens = current_tokens(d, id);
        for t in &to_add {
            if !tokens.iter().any(|x| x == t) {
                tokens.push(t.clone());
            }
        }
        write_tokens(d, id, &tokens);
    });
}

/// DOMTokenList `remove(...tokens)`（spec `dom-domtokenlist-remove`，variadic）：逐 token 校验 → 全部移除 → 写回。
fn native_dtl_remove_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        return;
    };
    let n = args.length();
    let mut to_remove: Vec<String> = Vec::with_capacity(n.max(0) as usize);
    for i in 0..n {
        let t = string_arg(scope, &args, i);
        if !require_valid_token(scope, &t) {
            return;
        }
        to_remove.push(t);
    }
    with_dom_mut(|d| {
        let tokens: Vec<String> = current_tokens(d, id)
            .into_iter()
            .filter(|t| !to_remove.iter().any(|r| r == t))
            .collect();
        write_tokens(d, id, &tokens);
    });
}

/// DOMTokenList `toggle(token, force?)`（spec `dom-domtokenlist-toggle`）：token 非法 → 抛。
/// force≠undefined：force true→加、false→移除（不切换，spec）；force undefined→切换。返最终是否含。
fn native_dtl_toggle_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let token = string_arg(scope, &args, 0);
    if !require_valid_token(scope, &token) {
        return;
    }
    let force_defined = !args.get(1).is_undefined();
    let force = force_defined && args.get(1).boolean_value(scope);
    let on = with_dom_mut(|d| {
        let mut tokens = current_tokens(d, id);
        let i = tokens.iter().position(|t| t == &token);
        let result = if force_defined {
            // force 模式：不切换，按 force 加/移除。
            match (force, i) {
                (true, None) => {
                    tokens.push(token.clone());
                    true
                }
                (false, Some(pos)) => {
                    tokens.remove(pos);
                    false
                }
                _ => force, // 已在且 force=true / 不在且 force=false → 不变
            }
        } else {
            // 切换模式：在→移除返 false；不在→加返 true。
            match i {
                Some(pos) => {
                    tokens.remove(pos);
                    false
                }
                None => {
                    tokens.push(token.clone());
                    true
                }
            }
        };
        write_tokens(d, id, &tokens);
        result
    })
    .unwrap_or(false);
    rv.set(v8::Boolean::new(scope, on).into());
}

/// DOMTokenList `replace(oldT, newT)`（spec `dom-domtokenlist-replace`）：token 非法 → 抛。oldT==newT →
/// 返 contains(oldT)；oldT 不在 → false（不写）；否则原位移除 oldT、插入 newT（dedupe），返 true。
fn native_dtl_replace_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let old_t = string_arg(scope, &args, 0);
    if !require_valid_token(scope, &old_t) {
        return;
    }
    let new_t = string_arg(scope, &args, 1);
    if !require_valid_token(scope, &new_t) {
        return;
    }
    let res = with_dom_mut(|d| {
        let mut tokens = current_tokens(d, id);
        // oldT==newT：返是否含（spec：无操作）。
        if old_t == new_t {
            return tokens.iter().any(|t| t == &old_t);
        }
        let Some(pos) = tokens.iter().position(|t| t == &old_t) else {
            return false; // oldT 不在 → false（不写）
        };
        tokens.remove(pos); // 先移除 oldT
        // newT 已存在则不重复插入（dedupe，spec 结果）；否则原位插 newT 保序。
        if !tokens.iter().any(|t| t == &new_t) {
            tokens.insert(pos.min(tokens.len()), new_t.clone());
        }
        write_tokens(d, id, &tokens);
        true
    })
    .unwrap_or(false);
    rv.set(v8::Boolean::new(scope, res).into());
}

/// DOMTokenList `toString()`（spec）：当前 `class` 属性串（= value getter）。
fn native_dtl_to_string_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = dtl_owner(scope, &this) else {
        return;
    };
    let val = with_dom(|d| d.get_attribute(id, "class")).flatten().unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// 建 DOMTokenList ObjectTemplate（length/value(+setter) + item/contains/add/remove/toggle/replace/toString）
/// 并缓存（`set_domtokenlist_template`）。`install_dom_bindings` 调（Element 模板建之后）。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    let dtl = v8::ObjectTemplate::new(scope);
    dtl.set_internal_field_count(1);
    if let Some(k) = v8::String::new(scope, "length") {
        dtl.set_accessor(k.into(), native_dtl_length_getter);
    }
    if let Some(k) = v8::String::new(scope, "value") {
        dtl.set_accessor_with_setter(k.into(), native_dtl_value_getter, native_dtl_value_setter);
    }
    let item = v8::FunctionTemplate::builder(native_dtl_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "item") {
        dtl.set(k.into(), item.into());
    }
    let contains = v8::FunctionTemplate::builder(native_dtl_contains_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "contains") {
        dtl.set(k.into(), contains.into());
    }
    let add = v8::FunctionTemplate::builder(native_dtl_add_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "add") {
        dtl.set(k.into(), add.into());
    }
    let remove = v8::FunctionTemplate::builder(native_dtl_remove_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "remove") {
        dtl.set(k.into(), remove.into());
    }
    let toggle = v8::FunctionTemplate::builder(native_dtl_toggle_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "toggle") {
        dtl.set(k.into(), toggle.into());
    }
    let replace = v8::FunctionTemplate::builder(native_dtl_replace_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "replace") {
        dtl.set(k.into(), replace.into());
    }
    let to_string = v8::FunctionTemplate::builder(native_dtl_to_string_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "toString") {
        dtl.set(k.into(), to_string.into());
    }
    set_domtokenlist_template(scope, dtl);
}
