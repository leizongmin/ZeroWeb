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

/// 在给定对象（`new DOMException(...)` 的 `This` 或经构造器 new 的实例）上 set spec 自有属性。
///
/// 实例须经 DOMException 构造器 new（prototype = DOMException.prototype → `e.constructor
/// === DOMException`，WPT `assert_throws_dom` 最后一步要求）。此前用 `Object::new` 建裸对象
/// 导致 constructor === Object，native 路径所有 assert_throws_dom 失败（R5 修正）。
fn fill_instance(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>, message: &str, name: &str) {
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
}

/// `new DOMException(message [, name])` 构造器 invoke（spec `webidl#dom-domexception-domexception`）。
///
/// `message` 缺省 ""、`name` 缺省 "Error"。在 `args.this()`（new 调用时 prototype 已是
/// DOMException.prototype）上 set 属性——保证 `instance.constructor === DOMException`。
/// 非 `new` 调用（`DOMException(msg)` 无 new）：This 无正确 prototype，回退取全局构造器 new。
pub(super) fn native_dom_exception_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    // spec `webidl#dom-domexception-domexception`：message 缺省 ""、name 缺省 "Error"。
    // 注意 `string_arg` 对缺省/undefined 参数返 "undefined"（JS ToString），故须先判 undefined。
    // `new DOMException()`（无参）→ message="" name="Error"；`new DOMException('m')` → name="Error"。
    let message = if args.get(0).is_undefined() {
        String::new()
    } else {
        super::string_arg(scope, &args, 0)
    };
    let name = if args.get(1).is_undefined() {
        "Error".to_string()
    } else {
        let n = super::string_arg(scope, &args, 1);
        // spec：name 显式传空串仍按 "Error"（error-names-table 无空名）。非空用原值。
        if n.is_empty() { "Error".to_string() } else { n }
    };
    let this = args.this();
    // new 调用：This 的 prototype 是 DOMException.prototype → 直接 set。判据：This 的 constructor
    // name 为 DOMException（FunctionTemplate new_instance 的 This 满足）。简化：new 调用恒用 This。
    fill_instance(scope, &this, &message, &name);
    rv.set(this.into());
}

/// 注册全局 `DOMException` 构造器（`install_dom_bindings` 调）。
///
/// 原型挂 `toString`（`"name: message"`，spec）+ legacy code 常量（`INDEX_SIZE_ERR` 等，
/// 兼容旧代码读 `DOMException.SYNTAX_ERR`）。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    // 幂等：install_dom_bindings 在 webview native_dom=true 路径被多次调（run_page_scripts +
    // execute_script 各一次），每次建新 FunctionTemplate 会覆盖全局 DOMException → 抛出的异常
    // 持有旧构造器、全局是新构造器，`e.constructor === DOMException` 失败（"wrong global"）。
    // 若全局已有 DOMException（首次 install 装的），跳过重建，复用同一构造器。
    let has_dom_exception = v8::String::new(scope, "DOMException")
        .and_then(|key| global.get(scope, key.into()))
        .is_some_and(|v| !v.is_undefined() && !v.is_null());
    if has_dom_exception {
        return;
    }
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
        // prototype.constructor → DOMException function（spec：保证 `instance.constructor
        // === DOMException`，WPT assert_throws_dom 最后一步 `e.constructor === self.DOMException`
        // 要求）。FunctionTemplate prototype template 的 set 不接受 Local<Function>（V8 Fatal），
        // 亦不接受 tmpl 自身（循环 CHECK），故改为在 prototype **对象**（构造器 function 的
        // `prototype` 属性，FunctionTemplate 实例化时自动建）上 set constructor——对象 set
        // 接受任意 Value。R6 修复 R5 的 "wrong global"。
        let proto_obj = v8::String::new(scope, "prototype")
            .and_then(|pk| f.get(scope, pk.into()))
            .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok());
        if let (Some(proto), Some(ck)) = (proto_obj, v8::String::new(scope, "constructor")) {
            let _ = proto.set(scope, ck.into(), f.into());
        }
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
/// 经全局 `DOMException` 构造器 new 实例（prototype = DOMException.prototype →
/// `instance.constructor === DOMException`，WPT `assert_throws_dom` 最后一步要求）并
/// `throw_exception`。与 polyfill part01b.js `throw new DOMException(msg, name)` 行为对齐（A/B 等价）。
/// 此前用 `Object::new` 建裸对象导致 constructor===Object（R5 修正）。
///
/// [`dom_token_list`]: super::dom_token_list
pub(super) fn throw_dom_exception(scope: &mut v8::PinScope, name: &str, message: &str) {
    let context = scope.get_current_context();
    let global = context.global(scope);
    let Some(key) = v8::String::new(scope, "DOMException") else {
        return;
    };
    let Some(ctor_val) = global.get(scope, key.into()) else {
        return;
    };
    let Ok(ctor) = v8::Local::<v8::Function>::try_from(ctor_val) else {
        return;
    };
    let args: Vec<v8::Local<v8::Value>> = match (v8::String::new(scope, message), v8::String::new(scope, name)) {
        (Some(m), Some(n)) => vec![m.into(), n.into()],
        (Some(m), None) => vec![m.into()],
        _ => vec![],
    };
    if let Some(obj) = ctor.new_instance(scope, &args) {
        scope.throw_exception(obj.into());
    }
}
