//! 原生 `Event` / `CustomEvent` / `MouseEvent` / `KeyboardEvent` 构造器（R3127 + R3129）——注册为全局，
//! 使 `new Event(type, opts)` 等产出标准 event 对象（`instanceof` 成立），stop / preventDefault 方法上
//! **Event 原型**（子类经 FunctionTemplate::inherit 继承，共享，非每次派发注入）。
//!
//! spec DOM `Event`：构造器读 `(type, eventInitDict?)`；init dict `bubbles`/`cancelable`/`composed`
//!（缺省 false）。派发态属性 `target`/`currentTarget`（null）、`eventPhase`（NONE=0）、
//! `defaultPrevented`/`isTrusted`（false）、`timeStamp`（DOMHighResTimeStamp，R22 单调 perf time）。
//! [`event_target::native_dispatch_event_invoke`] 派发时覆写 `target`/`currentTarget`/`eventPhase`。
//! `MouseEvent`（R3129）加 坐标族/clientX/clientY/button/buttons/修饰键/relatedTarget；`KeyboardEvent`（R3129）
//! 加 key/code/修饰键/repeat/isComposing/keyCode/charCode/location。两子类经 inherit → `instanceof Event`
//! 亦成立 + stop/preventDefault 经原型链可达。
//!
//! **与 R3125/R3126/R3128 集成**：Event（及子类）实例经原型具 `stopPropagation`/`stopImmediatePropagation`/
//! `preventDefault` → [`event_target`] 派发的「缺失时注入」检查（`get(...).is_function()`）命中既有 → 不重复注入
//!（原型方法经 `get` 透明解析）。闭合 R3124 限制③（plain object 非 `instanceof Event`）+ R3126 限制③（stop 方法
//! 注入非原型）+ R3127 限制④（无 MouseEvent/KeyboardEvent 子类）。
//!
//! 可见性：`build_and_register` 为 `pub(super)`（mod.rs `install_dom_bindings` 全局注册调）；构造器/方法
//! 回调为私有。复用 `super::event_target::{native_stop_propagation_invoke, native_stop_immediate_invoke}`
//!（原型 stop 方法）+ `super::string_arg`（构造器读 type）。

use v8;

use super::event_target::{native_stop_immediate_invoke, native_stop_propagation_invoke};

/// `Event.timeStamp` 的单调时钟 origin（js-dom R22）。
///
/// spec DOM `Event.timeStamp` = 创建时刻的 DOMHighResTimeStamp（ms，单调，自 time origin 起的子毫秒精度）。
/// 旧实现恒 0（注释「沙箱无 perf timer，暂 0」）致 WPT `Event-timestamp-safe-resolution.html` 的
/// `do { e2.timeStamp - e1.timeStamp } while (==0)` 死循环（连续两次 `new MouseEvent()` 时间戳相同 → 恒 0 差），
/// 拖垮 native dom/events 全量（>60s 卡死，非真 dispatchEvent hang）。
///
/// 复用 polyfill `__zw_performance_now` 同款 `Instant` 语义：进程级 origin（`OnceLock`，首次构造 Event 时
/// 懒初始化）+ `elapsed()` ms。**不要求与 polyfill perf_origin 完全一致**——spec 仅要求单调 + 连续创建非零差
///（解锁死循环 + spec 合规）。`OnceLock<Instant>` lazy init 线程安全；`Instant::elapsed` 无锁纯读。
// R138（js-dom M4）：改用**共享 origin**（`js_dom_bridge::callbacks::shared_perf_origin`——
// performance.now() 回调与 Event.timeStamp 同一 Instant）。spec DOM 要求两者同 time origin
//（WPT Event-timestamp-high-resolution 断言 `ev.timeStamp >= before = performance.now()`）；
// 自有 origin 起点晚于回调注册 → timeStamp 数值恒小于 performance.now() 断言失败。
fn perf_time_origin() -> std::time::Instant {
    crate::js_dom_bridge::shared_perf_origin()
}

/// 当前 DOMHighResTimeStamp（ms，自 [`perf_time_origin`] 起的单调 elapsed）。供 `Event.timeStamp` 用。
/// R150（js-dom M4）：量化到 5µs（0.005ms）——定时侧信道缓解（WPT
/// Event-timestamp-safe-resolution 千样本 GCD ≥ 5µs；真实浏览器对 Event timeStamp
/// 施加 coarse 粒度。与 shim `_makeEvent` 的 JS 侧量化同语义，两路径一致）。
fn perf_now_ms() -> f64 {
    let t = perf_time_origin().elapsed().as_secs_f64() * 1000.0;
    // ceil 向上取整：任意正 elapsed 量化后恒 ≥ 0.005ms（round 会把 <2.5µs 的早期
    // 值归 0——R22 断言 timeStamp > 0 回归）。ceil 保持 5µs 步进粒度（GCD 性质不变）。
    (t * 200.0).ceil() / 200.0
}
use super::string_arg;

/// 构建并注册 `Event` + 子类（`CustomEvent`/`MouseEvent`/`KeyboardEvent`）全局构造器（mod.rs
/// `install_dom_bindings` 末调）。
///
/// `Event` 基类模板挂原型方法（stop/preventDefault 上原型一次）；子类经 FunctionTemplate::inherit
/// 继承 Event 模板 → 原型链 MouseEvent.prototype → Event.prototype，`new MouseEvent('x') instanceof
/// MouseEvent && instanceof Event` 均成立，且 stop/preventDefault 经继承可达（无需子类重复挂）。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    // Event 基类：原型方法一次挂。
    let event_tmpl = v8::FunctionTemplate::builder(native_event_constructor_invoke).build(scope);
    attach_event_proto_methods(scope, event_tmpl);
    register_ctor(scope, global, "Event", event_tmpl);
    // CustomEvent / MouseEvent / KeyboardEvent extends Event（inherit → 原型链 + instanceof Event）。
    let ce_tmpl = v8::FunctionTemplate::builder(native_custom_event_constructor_invoke).build(scope);
    ce_tmpl.inherit(event_tmpl);
    register_ctor(scope, global, "CustomEvent", ce_tmpl);
    let mouse_tmpl = v8::FunctionTemplate::builder(native_mouse_event_constructor_invoke).build(scope);
    mouse_tmpl.inherit(event_tmpl);
    register_ctor(scope, global, "MouseEvent", mouse_tmpl);
    let kb_tmpl = v8::FunctionTemplate::builder(native_keyboard_event_constructor_invoke).build(scope);
    kb_tmpl.inherit(event_tmpl);
    register_ctor(scope, global, "KeyboardEvent", kb_tmpl);
    // R3141 createEvent 工厂（legacy 事件创建：document.createEvent(type) → new Ctor() + initEvent）。
    let ce = v8::FunctionTemplate::builder(native_create_event_invoke).build(scope);
    if let (Some(f), Some(key)) = (
        ce.get_function(scope),
        v8::String::new(scope, "__zw_native_create_event"),
    ) {
        let _ = global.set(scope, key.into(), f.into());
    }
}

/// 挂 Event 原型方法（stopPropagation/stopImmediatePropagation/preventDefault）到构造器模板原型。
/// 仅 Event 基类调一次；子类经 inherit 自动可达。
fn attach_event_proto_methods(scope: &mut v8::PinScope, tmpl: v8::Local<v8::FunctionTemplate>) {
    let proto = tmpl.prototype_template(scope);
    let stop = v8::FunctionTemplate::builder(native_stop_propagation_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "stopPropagation") {
        proto.set(k.into(), stop.into());
    }
    let stop_imm = v8::FunctionTemplate::builder(native_stop_immediate_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "stopImmediatePropagation") {
        proto.set(k.into(), stop_imm.into());
    }
    let pd = v8::FunctionTemplate::builder(native_prevent_default_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "preventDefault") {
        proto.set(k.into(), pd.into());
    }
    // R3141 initEvent（legacy event 创建配套：createEvent 后 initEvent 设 type/bubbles/cancelable）。
    let init_evt = v8::FunctionTemplate::builder(native_init_event_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "initEvent") {
        proto.set(k.into(), init_evt.into());
    }
}

/// 取构造器 FunctionTemplate 的 function + 注册为全局 `name`。
fn register_ctor(
    scope: &mut v8::PinScope,
    global: v8::Local<v8::Object>,
    name: &str,
    tmpl: v8::Local<v8::FunctionTemplate>,
) {
    let Some(f) = tmpl.get_function(scope) else {
        return;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = global.set(scope, key.into(), f.into());
}

/// 读 eventInitDict 第 `idx` 参的布尔属性（`bubbles`/`cancelable`，缺省 false）。
fn init_bool(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32, name: &str) -> bool {
    let Ok(opts) = v8::Local::<v8::Object>::try_from(args.get(idx)) else {
        return false;
    };
    v8::String::new(scope, name)
        .and_then(|k| opts.get(scope, k.into()))
        .is_some_and(|v| v.is_true())
}

/// 读 eventInitDict 第 `idx` 参的整数属性（经 V8 ToInt32 强转，处理 SMI/Number；缺省 `default`）。
fn init_int(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32, name: &str, default: i32) -> i32 {
    let Ok(opts) = v8::Local::<v8::Object>::try_from(args.get(idx)) else {
        return default;
    };
    v8::String::new(scope, name)
        .and_then(|k| opts.get(scope, k.into()))
        .and_then(|v| v.int32_value(scope))
        .unwrap_or(default)
}

/// 读 eventInitDict 第 `idx` 参的字符串属性（缺省 `default`）。
fn init_string(
    scope: &mut v8::PinScope,
    args: &v8::FunctionCallbackArguments,
    idx: i32,
    name: &str,
    default: &str,
) -> String {
    let Ok(opts) = v8::Local::<v8::Object>::try_from(args.get(idx)) else {
        return default.to_string();
    };
    v8::String::new(scope, name)
        .and_then(|k| opts.get(scope, k.into()))
        .and_then(|v| v.to_string(scope).map(|s| s.to_rust_string_lossy(scope)))
        .unwrap_or_else(|| default.to_string())
}

/// 设 4 个修饰键属性（shiftKey/altKey/ctrlKey/metaKey， MouseEvent/KeyboardEvent 共用）。
/// 从 init dict 读，缺省 false。
fn set_modifier_keys(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, args: &v8::FunctionCallbackArguments) {
    for key in ["shiftKey", "altKey", "ctrlKey", "metaKey"] {
        let val = init_bool(scope, args, 1, key);
        if let Some(k) = v8::String::new(scope, key) {
            let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, val).into());
        }
    }
}

/// 设 UIEvent.`view` 属性（js-dom R25）。spec UIEvent view = WindowProxy 或 null（init dict `view` 字段，
/// 缺省/undefined/null → null）。旧 native MouseEvent/KeyboardEvent 不设 view → WPT Event-subclasses-
/// constructors `assert_props` 父链检查（MouseEvent extends UIEvent）`'view' in event` fail。从 init dict
/// 读任意值（WPT 测 `view: window`），非对象值 → null。
fn set_ui_view(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, args: &v8::FunctionCallbackArguments) {
    let view = match v8::Local::<v8::Object>::try_from(args.get(1)) {
        Ok(opts) => {
            let v = v8::String::new(scope, "view").and_then(|k| opts.get(scope, k.into()));
            match v {
                // 仅对象（window）/非 null/undefined 原样用；余（含 null/undefined）→ null。
                Some(v) if v.is_object() => v,
                _ => v8::null(scope).into(),
            }
        }
        Err(_) => v8::null(scope).into(),
    };
    if let Some(k) = v8::String::new(scope, "view") {
        let _ = obj.set(scope, k.into(), view);
    }
}

/// 设整数属性（name → i32）。
fn set_int(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, name: &str, val: i32) {
    if let Some(k) = v8::String::new(scope, name) {
        let _ = obj.set(scope, k.into(), v8::Integer::new(scope, val).into());
    }
}

/// 设 event 实例的 init 属性（type/bubbles/cancelable/composed + 派发态默认）。
fn set_event_init(
    scope: &mut v8::PinScope,
    obj: v8::Local<v8::Object>,
    event_type: &str,
    bubbles: bool,
    cancelable: bool,
) {
    fn set_str(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, name: &str, val: &str) {
        if let (Some(k), Some(v)) = (v8::String::new(scope, name), v8::String::new(scope, val)) {
            let _ = obj.set(scope, k.into(), v.into());
        }
    }
    fn set_bool(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>, name: &str, val: bool) {
        if let Some(k) = v8::String::new(scope, name) {
            let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, val).into());
        }
    }
    set_str(scope, obj, "type", event_type);
    set_bool(scope, obj, "bubbles", bubbles);
    set_bool(scope, obj, "cancelable", cancelable);
    set_bool(scope, obj, "composed", false);
    // 派发态属性默认（dispatchEvent 派发时覆写 target/currentTarget/eventPhase）。
    if let Some(k) = v8::String::new(scope, "target") {
        let _ = obj.set(scope, k.into(), v8::null(scope).into());
    }
    if let Some(k) = v8::String::new(scope, "currentTarget") {
        let _ = obj.set(scope, k.into(), v8::null(scope).into());
    }
    // srcElement（js-dom M4 R32，spec `dom-event-srcelement`）：Event.target 的 legacy IE 别名
    //（IDL getter 返 target）。native 用 data 属性镜像 target（init null + dispatch 与 target 同步设），
    // 避免 prototype accessor 复杂度（与 target 同生命周期）。
    if let Some(k) = v8::String::new(scope, "srcElement") {
        let _ = obj.set(scope, k.into(), v8::null(scope).into());
    }
    if let Some(k) = v8::String::new(scope, "eventPhase") {
        let _ = obj.set(scope, k.into(), v8::Integer::new(scope, 0).into());
    }
    set_bool(scope, obj, "defaultPrevented", false);
    set_bool(scope, obj, "isTrusted", false);
    // timeStamp（spec `Event.timeStamp` = DOMHighResTimeStamp，js-dom R22）：创建时刻的单调 perf time（ms，
    // 子毫秒）。旧恒 0 致 WPT Event-timestamp-safe-resolution do-while(==0) 死循环。f64（Number）——spec 要求
    // 子毫秒精度（5µs 分辨率断言），Integer 会丢精度。
    if let Some(k) = v8::String::new(scope, "timeStamp") {
        let _ = obj.set(scope, k.into(), v8::Number::new(scope, perf_now_ms()).into());
    }
}

/// `new Event(type, eventInitDict?)` 构造器（spec `dom-event-constructor`）：设 init 属性于 `this`。
/// 不设 ReturnValue（undefined → V8 用 `this`，标准构造器语义）。
fn native_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
}

/// `new CustomEvent(type, eventInitDict?)` 构造器（spec `dom-customevent-constructor`）：Event +
/// `detail`（init dict `detail` 字段，缺省 null）。复用 [`native_event_constructor_invoke`] 的 init 属性。
fn native_custom_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
    // detail：从 init dict 读原值（任意类型），缺省 / undefined → null。Object::get 对缺失属性返
    // Some(undefined)（非 None），故显式判 undefined 回落 null（spec：detail 缺省 null）。
    let detail = match v8::Local::<v8::Object>::try_from(args.get(1)) {
        Ok(opts) => {
            let v = v8::String::new(scope, "detail").and_then(|k| opts.get(scope, k.into()));
            match v {
                Some(v) if !v.is_null() && !v.is_undefined() => v,
                _ => v8::null(scope).into(),
            }
        }
        Err(_) => v8::null(scope).into(),
    };
    if let Some(k) = v8::String::new(scope, "detail") {
        let _ = this.set(scope, k.into(), detail);
    }
}

/// `new MouseEvent(type, eventInitDict?)` 构造器（spec UIEvent/MouseEvent）：Event init 属性 +
/// 鼠标字段（坐标/按钮/修饰键，init dict 读，缺省 0/false）。inherits Event 模板（instanceof MouseEvent/Event）。
fn native_mouse_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
    // UIEvent.detail + 坐标族 + button/buttons（缺省 0）。先取值再设（避 scope 双重 mutable borrow）。
    for name in [
        "detail",
        "screenX",
        "screenY",
        "clientX",
        "clientY",
        "pageX",
        "pageY",
        "movementX",
        "movementY",
        "button",
        "buttons",
    ] {
        let v = init_int(scope, &args, 1, name, 0);
        set_int(scope, this, name, v);
    }
    // UIEvent.view（R25，缺省 null）——MouseEvent extends UIEvent，WPT 父链检查。
    set_ui_view(scope, this, &args);
    // 修饰键（shiftKey/altKey/ctrlKey/metaKey）。
    set_modifier_keys(scope, this, &args);
    // relatedTarget（缺省 / undefined → null）。
    let related = match v8::Local::<v8::Object>::try_from(args.get(1)) {
        Ok(opts) => {
            let v = v8::String::new(scope, "relatedTarget").and_then(|k| opts.get(scope, k.into()));
            match v {
                Some(v) if !v.is_null() && !v.is_undefined() => v,
                _ => v8::null(scope).into(),
            }
        }
        Err(_) => v8::null(scope).into(),
    };
    if let Some(k) = v8::String::new(scope, "relatedTarget") {
        let _ = this.set(scope, k.into(), related);
    }
}

/// `new KeyboardEvent(type, eventInitDict?)` 构造器（spec KeyboardEvent）：Event init 属性 +
/// 键盘字段（key/code/修饰键/repeat/isComposing/keyCode/charCode/location，init dict 读，缺省）。
/// inherits Event 模板（instanceof KeyboardEvent/Event）。
fn native_keyboard_event_constructor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = init_bool(scope, &args, 1, "bubbles");
    let cancelable = init_bool(scope, &args, 1, "cancelable");
    set_event_init(scope, this, &event_type, bubbles, cancelable);
    // key/code（缺省 ""）。先取值再设（避 scope 双重 mutable borrow）。
    let key = init_string(scope, &args, 1, "key", "");
    let code = init_string(scope, &args, 1, "code", "");
    if let (Some(k), Some(v)) = (v8::String::new(scope, "key"), v8::String::new(scope, &key)) {
        let _ = this.set(scope, k.into(), v.into());
    }
    if let (Some(k), Some(v)) = (v8::String::new(scope, "code"), v8::String::new(scope, &code)) {
        let _ = this.set(scope, k.into(), v.into());
    }
    // UIEvent.view（R25，缺省 null）——KeyboardEvent extends UIEvent，WPT 父链检查。
    set_ui_view(scope, this, &args);
    // js-dom M4 R109：UIEvent.detail（缺省 0）——KeyboardEvent extends UIEvent，WPT
    // Event-subclasses-constructors assert_props 递归父链检查 `'detail' in event`。
    let detail = init_int(scope, &args, 1, "detail", 0);
    set_int(scope, this, "detail", detail);
    // 修饰键。
    set_modifier_keys(scope, this, &args);
    // repeat / isComposing（缺省 false）。
    for name in ["repeat", "isComposing"] {
        let val = init_bool(scope, &args, 1, name);
        if let Some(k) = v8::String::new(scope, name) {
            let _ = this.set(scope, k.into(), v8::Boolean::new(scope, val).into());
        }
    }
    // keyCode/charCode/location（缺省 0）。
    let key_code = init_int(scope, &args, 1, "keyCode", 0);
    set_int(scope, this, "keyCode", key_code);
    for name in ["charCode", "location"] {
        let v = init_int(scope, &args, 1, name, 0);
        set_int(scope, this, name, v);
    }
    // which（R25）：KeyboardEvent.which legacy 属性。缺省回退 keyCode（spec：which = keyCode 兼容）。
    let which = init_int(scope, &args, 1, "which", key_code);
    set_int(scope, this, "which", which);
}

/// `event.preventDefault()` 原型方法（spec `dom-event-prevent-default`）：仅当 `cancelable` 时设
/// `defaultPrevented=true`。headless 无浏览器默认行为，故仅记账（供 dispatchEvent 返值语义后续）。
/// R3147：`pub(super)`——[`event_target::dispatch_event_impl`] 对 plain event 对象（如 `element.click()`
/// 构造的合成事件，无 Event 原型）「缺失时注入」preventDefault，使监听器 `e.preventDefault()` 可用 +
/// 派发返值正确反映 defaultPrevented（同 stopPropagation 注入 pattern）。
pub(super) fn native_prevent_default_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let obj = args.this();
    let cancelable = v8::String::new(scope, "cancelable")
        .and_then(|k| obj.get(scope, k.into()))
        .is_some_and(|v| v.is_true());
    if cancelable && let Some(k) = v8::String::new(scope, "defaultPrevented") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}

/// `event.initEvent(type, bubbles, cancelable)` 原型方法（spec `dom-event-initevent`，legacy）：
/// 重置 event 为「已初始化未派发」态——复用 [`set_event_init`] 设 type/bubbles/cancelable + 派发态默认
///（target/currentTarget=null、eventPhase=0、defaultPrevented=false）。dispatchEvent 派发前复位 stop flag，
/// 故 initEvent 不必显式清。子类（MouseEvent 等）经原型链继承可达（spec 各有 initXxxEvent，本切片基类通用）。
fn native_init_event_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let event_type = string_arg(scope, &args, 0);
    let bubbles = args.get(1).is_true();
    let cancelable = args.get(2).is_true();
    set_event_init(scope, this, &event_type, bubbles, cancelable);
}

/// `__zw_native_create_event(type)`：spec `dom-document-createevent`（legacy 事件创建——`document.createEvent`）。
/// 映射 legacy DOM type 字符串 → 构造器名，`new Ctor()`（无参，构造器设默认 init）产 instanceof Event 对象
///（派发态默认 + 原型 stop/preventDefault/initEvent 经原型链可达）；后续 `initEvent` 覆写。
///
/// 映射：`Event`/`Events`/`HTMLEvents`/未知 → `Event`；`CustomEvent` → `CustomEvent`；
/// `MouseEvent`/`MouseEvents` → `MouseEvent`；`KeyboardEvent`/`KeyboardEvents` → `KeyboardEvent`
///（spec 各 type 应抛 NotSupportedError，本切片未知 → Event best-effort）。
pub(super) fn native_create_event_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let type_str = string_arg(scope, &args, 0);
    let ctor_name = match type_str.as_str() {
        "CustomEvent" => "CustomEvent",
        "MouseEvent" | "MouseEvents" => "MouseEvent",
        "KeyboardEvent" | "KeyboardEvents" => "KeyboardEvent",
        // "Event"/"Events"/"HTMLEvents"/未知 → Event（spec 未知应抛，best-effort Event）。
        _ => "Event",
    };
    // 查全局构造器（Event/CustomEvent/MouseEvent/KeyboardEvent 经 build_and_register 注册）。
    let context = scope.get_current_context();
    let global = context.global(scope);
    let Some(ctor_key) = v8::String::new(scope, ctor_name) else {
        return;
    };
    let Some(ctor_val) = global.get(scope, ctor_key.into()) else {
        return;
    };
    let Ok(ctor) = v8::Local::<v8::Function>::try_from(ctor_val) else {
        return;
    };
    // new Ctor("") — 传空串 type（构造器设默认 init：type="" / bubbles=false / cancelable=false），
    // 返 instanceof Ctor 对象。spec createEvent 返「未初始化」event（type="" 待 initEvent 覆写）。
    let new_args: Vec<v8::Local<v8::Value>> = match v8::String::new(scope, "") {
        Some(s) => vec![s.into()],
        None => vec![],
    };
    if let Some(instance) = ctor.new_instance(scope, &new_args) {
        rv.set(instance.into());
    }
}
