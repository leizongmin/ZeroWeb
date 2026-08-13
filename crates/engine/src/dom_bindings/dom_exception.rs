//! `DOMException`——Web IDL 异常类型（spec `webidl#idl-DOMException`）。
//!
//! 众多 DOM API 校验失败时抛（`classList.add("")`→SyntaxError、`classList.add("a b")`→
//! InvalidCharacterError、`appendChild` 闭环→HierarchyRequestError 等）。WPT `assert_throws_dom`
//! 按 **name**（+ code + `constructor === DOMException`）判定，故原生路径须抛真正的
//! DOMException 实例（非 `v8::Exception::type_error`）。
//!
//! 设计：DOMException 是纯数据对象（`name`/`message`/`code`/`stack`），无 internal slot
//! 指向 Rust——故用 `ObjectTemplate`（name/message/code 为自有属性，构造时 set）而非
//! NodeId 模板。构造器经 [`build_and_register`] 注册全局；Rust 侧校验失败经
//! [`throw_dom_exception`] 构造实例并抛。
//!
//! spec：DOMException name↔code 表 https://webidl.spec.whatwg.org/#dfn-error-names-table

/// DOMException name → legacy code（spec error-names-table；0 = 无 legacy code）。
///
/// 仅列 dom_bindings 当前会用到的 name；未列入的 name 返 0（spec 允许新 name 无 legacy code）。
fn code_for_name(name: &str) -> u32 {
    match name {
        "IndexSizeError" => 1,
        "HierarchyRequestError" => 3,
        "WrongDocumentError" => 4,
        "InvalidCharacterError" => 5,
        "NoModificationAllowedError" => 7,
        "NotFoundError" => 8,
        "NotSupportedError" => 9,
        "InUseAttributeError" => 10,
        "InvalidStateError" => 11,
        "SyntaxError" => 12,
        "InvalidModificationError" => 13,
        "NamespaceError" => 14,
        "InvalidAccessError" => 15,
        "SecurityError" => 18,
        "NetworkError" => 19,
        "AbortError" => 20,
        "URLMismatchError" => 21,
        "TimeoutError" => 23,
        "InvalidNodeTypeError" => 24,
        "DataCloneError" => 25,
        _ => 0,
    }
}

/// 构造一个 DOMException 实例（`name`/`message`/`code`/`stack` 自有属性）。
///
/// 供构造器 invoke 与 [`throw_dom_exception`] 共用。`name` 缺省 `"Error"`（spec：
/// `new DOMException(msg)` 无 name → name="Error", code=0）。
fn new_instance<'s>(scope: &mut v8::PinScope<'s, '_>, message: &str, name: &str) -> Option<v8::Local<'s, v8::Object>> {
    let obj = v8::Object::new(scope);
    let set_str = |key: &str, val: &str| {
        if let (Some(k), Some(v)) = (v8::String::new(scope, key), v8::String::new(scope, val)) {
            let _ = obj.set(scope, k.into(), v.into());
        }
    };
    let set_int = |key: &str, val: u32| {
        if let Some(k) = v8::String::new(scope, key) {
            let _ = obj.set(scope, k.into(), v8::Integer::new(scope, val as i32).into());
        }
    };
    set_str("message", message);
    set_str("name", name);
    set_int("code", code_for_name(name));
    // stack：headless 无真调用栈，给空串（libs 读 .stack 容忍空）。
    set_str("stack", "");
    Some(obj)
}

/// `new DOMException(message [, name])` 构造器 invoke（spec `webidl#dom-domexception-domexception`）。
///
/// `message` 缺省 ""、`name` 缺省 "Error"。返 DOMException 实例（非 `new` 调用亦返实例，
/// 镜像 polyfill part01b.js `Object.create(DOMException.prototype)` 容错）。
pub(super) fn native_dom_exception_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let message = super::string_arg(scope, &args, 0);
    let name = {
        let n = super::string_arg(scope, &args, 1);
        if n.is_empty() { "Error".to_string() } else { n }
    };
    if let Some(obj) = new_instance(scope, &message, &name) {
        rv.set(obj.into());
    }
}

/// 注册全局 `DOMException` 构造器（`install_dom_bindings` 调）。
///
/// 原型挂 `toString`（`"name: message"`，spec）+ legacy code 常量（`INDEX_SIZE_ERR` 等，
/// 兼容旧代码读 `DOMException.SYNTAX_ERR`）。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    let tmpl = v8::FunctionTemplate::builder(native_dom_exception_constructor_invoke).build(scope);
    // 原型 toString：name + ": " + message（spec DOMException.prototype.toString）。
    let proto = tmpl.prototype_template(scope);
    let to_string = v8::FunctionTemplate::builder(native_dom_exception_to_string_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "toString") {
        proto.set(k.into(), to_string.into());
    }
    // legacy code 常量（spec：DOMException.SYNTAX_ERR=12 等，挂构造器函数自身）。
    let register_const = |name: &str, code: u32| {
        if let (Some(f), Some(k)) = (tmpl.get_function(scope), v8::String::new(scope, name)) {
            let _ = f.set(scope, k.into(), v8::Integer::new(scope, code as i32).into());
        }
    };
    register_const("INDEX_SIZE_ERR", 1);
    register_const("HIERARCHY_REQUEST_ERR", 3);
    register_const("WRONG_DOCUMENT_ERR", 4);
    register_const("INVALID_CHARACTER_ERR", 5);
    register_const("NO_MODIFICATION_ALLOWED_ERR", 7);
    register_const("NOT_FOUND_ERR", 8);
    register_const("NOT_SUPPORTED_ERR", 9);
    register_const("INUSE_ATTRIBUTE_ERR", 10);
    register_const("INVALID_STATE_ERR", 11);
    register_const("SYNTAX_ERR", 12);
    register_const("INVALID_MODIFICATION_ERR", 13);
    register_const("NAMESPACE_ERR", 14);
    register_const("INVALID_ACCESS_ERR", 15);
    register_const("SECURITY_ERR", 18);
    register_const("NETWORK_ERR", 19);
    register_const("ABORT_ERR", 20);
    register_const("URL_MISMATCH_ERR", 21);
    register_const("TIMEOUT_ERR", 23);
    register_const("INVALID_NODE_TYPE_ERR", 24);
    register_const("DATA_CLONE_ERR", 25);

    if let (Some(f), Some(key)) = (tmpl.get_function(scope), v8::String::new(scope, "DOMException")) {
        let _ = global.set(scope, key.into(), f.into());
    }
}

/// `DOMException.prototype.toString()` → `"name: message"`（spec）。
fn native_dom_exception_to_string_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    // 读 this 自有属性字符串值（缺省返 fallback）。
    let prop_str = |fallback: &str| -> String {
        let Some(key) = v8::String::new(scope, fallback) else {
            return String::new();
        };
        this.get(scope, key.into())
            .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
            .unwrap_or_default()
    };
    let name = prop_str("name");
    let message = prop_str("message");
    let s = if message.is_empty() {
        name
    } else {
        format!("{name}: {message}")
    };
    if let Some(v) = v8::String::new(scope, &s) {
        rv.set(v.into());
    }
}

/// 校验失败抛 DOMException（供 dom_bindings 各校验点调，如 [`dom_token_list`] token 校验）。
///
/// 构造 `new DOMException(msg, name)` 实例（带 name/code/message）并 `throw_exception`。
/// 与 polyfill part01b.js `throw new DOMException(msg, name)` 行为对齐（A/B 等价）。
///
/// [`dom_token_list`]: super::dom_token_list
pub(super) fn throw_dom_exception(scope: &mut v8::PinScope, name: &str, message: &str) {
    if let Some(obj) = new_instance(scope, message, name) {
        scope.throw_exception(obj.into());
    }
}
