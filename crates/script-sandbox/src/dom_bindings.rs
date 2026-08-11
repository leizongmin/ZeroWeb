//! P1b S0 PoC — 原生 V8 绑定 TBD 验证（零行为变更，默认不接线；S1+ 生产化）。
//!
//! 验证 rusty_v8 150.2.0 的 internal-field（TBD-1）+ weak-handle GC（TBD-2）API 可用性，
//! 为 P1b 原生 DOM 绑定（NodeId internal slot + 原生 getter，不经 shim 字符串桥）铺路。
//! spec/RFC：`docs/specs/p1b-v8-native-bindings-rfc.md` §S0 / §6 TBD。
//!
//! **本模块不接入生产管线**（S0 纯验证）；S1 起按 RFC §4 在 engine 建 dom_bindings +
//! gc.rs，经 kill-switch 接通原生 getter。engine 现无直接 v8 访问（经 Sandbox trait），
//! S0 PoC 置于 script-sandbox（有 v8）；engine 接线（直接 v8 dep 或 script-sandbox 托管）
//! 是 S1 架构决策。

/// TBD-1 验证：ObjectTemplate internal field 存 NodeId（v8::External 包 `*mut c_void`）+ 读回。
///
/// 证明原生绑定值传递管线可用：NodeId 存入 internal slot[0]，经 External 编码，
/// `get_internal_field` 读回 External → 取 NodeId（不经 shim 字符串桥）。
///
/// PoC 简化：NodeId 直接编码进 External 指针值（`node_id as usize as *mut c_void`，无堆分配）。
/// 真绑定（S1+）存 NodeId 表索引或直接 Integer 存 u32。返回读回的 NodeId（应等于传入）。
#[allow(dead_code)] // S0 PoC 验证函数——经 #[cfg(test)] 测试覆盖；生产不接管线（S1+ 生产化）
pub fn poc_internal_field_round_trip(node_id: u32) -> u32 {
    crate::v8_runtime::ensure_v8_initialized();
    let isolate = &mut v8::Isolate::new(Default::default());
    let mut read_back = 0u32;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        let tmpl = v8::ObjectTemplate::new(scope);
        tmpl.set_internal_field_count(1);
        let obj = tmpl.new_instance(scope).expect("ObjectTemplate::new_instance");

        // 存 NodeId 经 External（ptr 值 = NodeId，PoC 无堆分配）。
        let ptr = node_id as usize as *mut std::ffi::c_void;
        let external = v8::External::new(scope, ptr);
        let _ = obj.set_internal_field(0, external.into());

        // 读回 internal field 0 → External → NodeId。
        if let Some(data) = obj.get_internal_field(scope, 0) {
            let ext = data.cast::<v8::External>();
            read_back = ext.value() as usize as u32;
        }
    }
    read_back
}

/// TBD-2 验证：Weak handle 跟踪对象 liveness——强引用释放 + 强制 GC 后 weak 变 empty。
///
/// 证明原生绑定的 GC 安全机制可用：Rust 持 weak handle（不阻止回收），对象无强引用时
/// 经 GC 回收，weak 反映为 empty（stale 检测基础：getter 读 weak → empty 则对象已回收）。
/// 镜像 rusty_v8 test_api.rs `global_from_into_raw` 的 weak.is_empty() 模式（可靠，
/// 非 with_finalizer 的 best-effort 时序）。finalizer callback（with_finalizer）API 另经
/// RFC §6 记录（with_finalizer/with_guaranteed_finalizer 签名已验证可用）。
#[allow(dead_code)] // S0 PoC 验证函数——经 #[cfg(test)] 测试覆盖；生产不接管线（S1+ gc.rs 生产化）
pub fn poc_weak_handle_becomes_empty_on_gc() -> bool {
    crate::v8_runtime::ensure_v8_initialized();
    let isolate = &mut v8::Isolate::new(Default::default());
    let became_empty;
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);

        // 内层 scope 建 local + Global + Weak；内层结束时 local/Global 释放（强引用断）。
        let weak = {
            v8::scope!(let inner, scope);
            let local = v8::Object::new(inner);
            let global = v8::Global::new(inner, local);
            v8::Weak::new(inner, &global)
        };
        // 无强引用 → low_memory_notification（host GC hint，无需 --expose-gc；
        // request_garbage_collection_for_testing 需 --expose-gc 全局 flag，非零行为变更）→ 对象回收 → weak empty。
        scope.low_memory_notification();
        became_empty = weak.is_empty();
    }
    became_empty
}

/// TBD-1（S5 前置，**已验证可行**）：native FunctionTemplate 构造器被 **JS `class extends`** 子类化时，
/// 子类实例继承 native prototype + **拿到** native 构造器（经 `super()`）填的 internal field。
///
/// **背景**：S5 customElements 需 `class MyEl extends HTMLElement`（HTMLElement = native 构造器）。
/// Event 绑定（R3127/R3129）已验证 **native `FunctionTemplate::inherit`**（一个 native 模板继承另一个），
/// 但 **JS 侧 `class extends NativeCtor`** 是不同机制——子类实例由 JS `[[Construct]]` 分配。RFC §6 TBD-1
/// 标「S5 前专项验证 class 继承」：验证该子类实例是否有 native instance_template 的 internal field slot。
///
/// **结论（rusty_v8 150.2.0）**：✅ **S5 可行**——`super()` 调 native ctor 时，子类实例**继承了 native
/// instance_template 的 internal field 布局**，`set_internal_field(0, ...)` 成功（返 true），native getter
/// 经 instance_template accessor（holder=实例）读到 NodeId=42（subclass_node_type=42，与直接构造一致）。
/// 故 S5 customElements 可直接复用 internal-field NodeId 存储（与既有 native 元素生产路径一致），无需
/// private symbol / WeakMap 替代。
///
/// **关键技术点**（PoC 调试得出）：native NodeId getter 须挂 **instance_template**（非 prototype_template）
/// accessor——前者 holder=实例（有 slot），后者 holder=原型对象（无 slot，get_internal_field 返 None）。
///
/// PoC：建 native `HTMLElement` 构造器（instance_template internal_field_count=1，ctor 填 slot[0]=42，
/// instance_template `nodeType` accessor 读 slot[0]），注册全局；跑 direct `new HTMLElement()` +
/// `class Sub extends HTMLElement` 子类化，读回 5 值对比。
///
/// 返回 `(direct_node_type, subclass_node_type, instanceof_base, instanceof_sub, sub_ctor_ran)`。
/// direct 与 subclass 均应 nodeType=42（S5 可行）。
#[allow(dead_code)] // S5 PoC 验证函数——经 #[cfg(test)] 测试覆盖；生产不接管线（S5 生产化前）
pub fn poc_native_ctor_subclass() -> (u32, u32, bool, bool, bool) {
    crate::v8_runtime::ensure_v8_initialized();
    let isolate = &mut v8::Isolate::new(Default::default());
    let (direct_node_type, subclass_node_type, instanceof_base, instanceof_sub, sub_ctor_ran);
    {
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        let global = context.global(scope);

        // native HTMLElement 构造器模板：instance_template internal_field_count=1（实例有 slot[0]），
        // ctor 回调填 slot[0]=42，prototype `nodeType` getter 读 slot[0]。
        let tmpl = v8::FunctionTemplate::builder(poc_native_html_element_ctor_invoke).build(scope);
        tmpl.instance_template(scope).set_internal_field_count(1);
        // nodeType getter 置 **instance_template**（accessor 持有者 = 实例，有 slot；prototype_template
        // 的 holder = 原型对象无 slot）。直接构造实例 + 子类实例（继承原型）均经此 accessor。
        if let Some(key) = v8::String::new(scope, "nodeType") {
            tmpl.instance_template(scope)
                .set_accessor(key.into(), poc_native_nodetype_getter);
        }
        // 注册全局 HTMLElement。
        if let (Some(f), Some(key)) = (tmpl.get_function(scope), v8::String::new(scope, "HTMLElement")) {
            let _ = global.set(scope, key.into(), f.into());
        }

        // JS 子类化 + 直接构造对比：direct new HTMLElement()（instance_template 产物，有 slot）vs
        // new (class extends HTMLElement)()（JS [[Construct]] 产物）。super() 调 native ctor 填 slot[0]。
        let code = r#"
            globalThis.__direct = new HTMLElement();
            globalThis.Sub = class Sub extends HTMLElement { constructor() { super(); this.__subRan = true; } };
            globalThis.__inst = new globalThis.Sub();
            globalThis.__r_direct_nt = globalThis.__direct.nodeType;
            globalThis.__r_sub_nt = globalThis.__inst.nodeType;
            globalThis.__r_iob = globalThis.__inst instanceof HTMLElement;
            globalThis.__r_ios = globalThis.__inst instanceof globalThis.Sub;
            globalThis.__r_sub = globalThis.__inst.__subRan === true;
        "#;
        let script = v8::Script::compile(scope, v8::String::new(scope, code).unwrap(), None);
        if let Some(script) = script {
            let _ = script.run(scope);
        }
        // 读回结果。
        direct_node_type = read_global_u32(scope, &global, "__r_direct_nt");
        subclass_node_type = read_global_u32(scope, &global, "__r_sub_nt");
        instanceof_base = read_global_bool(scope, &global, "__r_iob");
        instanceof_sub = read_global_bool(scope, &global, "__r_ios");
        sub_ctor_ran = read_global_bool(scope, &global, "__r_sub");
    }
    (
        direct_node_type,
        subclass_node_type,
        instanceof_base,
        instanceof_sub,
        sub_ctor_ran,
    )
}

/// PoC native HTMLElement 构造器回调：`this`（实例）slot[0] 存 NodeId=42（External ptr 值）。
/// 诊断：① 设 JS 属性 `__ctorRan=true`（区分 ctor 是否运行）② 经 static atomic 记录
/// `set_internal_field` 返回值（true=有 slot 可写 / false=无 slot）。
fn poc_native_html_element_ctor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    POC_CTOR_RAN.store(1, std::sync::atomic::Ordering::SeqCst);
    let this = args.this();
    let Some(obj) = this.to_object(scope) else {
        POC_SET_FIELD_OK.store(2, std::sync::atomic::Ordering::SeqCst); // to_object failed
        return;
    };
    // NodeId=42 经 External ptr 值存 slot[0]（镜像 S0/production 模式）。
    let ptr = 42usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let ok = obj.set_internal_field(0, external.into());
    POC_SET_FIELD_OK.store(if ok { 1 } else { 0 }, std::sync::atomic::Ordering::SeqCst);
    // 诊断 JS 属性（确认 ctor 体运行 + this 可写普通属性）。
    if let Some(k) = v8::String::new(scope, "__ctorRan") {
        let _ = obj.set(scope, k.into(), v8::Boolean::new(scope, true).into());
    }
}

/// 诊断 static：ctor 是否运行（0=未跑/未构造，1=跑过）。
static POC_CTOR_RAN: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
/// 诊断 static：set_internal_field 返回（0=false 无 slot，1=true 有 slot 写成，2=to_object 失败）。
static POC_SET_FIELD_OK: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// PoC prototype `nodeType` getter：读 holder slot[0] → External → u32（无 slot 返 0）。
fn poc_native_nodetype_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    POC_GETTER_CALLED.store(1, std::sync::atomic::Ordering::SeqCst);
    // accessor 在 instance_template 上 → holder = 实例（有 slot）。直接构造实例有 slot；
    // 子类实例由 JS [[Construct]] 分配，**可能无 slot**（本 PoC 验证点）。
    let holder = args.holder();
    match holder.get_internal_field(scope, 0) {
        Some(data) => {
            let ext = data.cast::<v8::External>();
            let v = ext.value() as usize as u32;
            POC_GETTER_READ.store(v as u8, std::sync::atomic::Ordering::SeqCst);
            rv.set(v8::Integer::new_from_unsigned(scope, v).into());
        }
        None => POC_GETTER_READ.store(255, std::sync::atomic::Ordering::SeqCst), // slot 读取返 None
    }
}

/// 诊断 static：getter 是否被调（0=未调，1=调过）。
static POC_GETTER_CALLED: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
/// 诊断 static：getter 读到的 slot 值（u8 近似，255=get_internal_field 返 None）。
static POC_GETTER_READ: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn read_global_u32(scope: &mut v8::PinScope, global: &v8::Local<v8::Object>, key: &str) -> u32 {
    let Some(k) = v8::String::new(scope, key) else { return 0 };
    let Some(v) = global.get(scope, k.into()) else { return 0 };
    v.uint32_value(scope).unwrap_or(0)
}

fn read_global_bool(scope: &mut v8::PinScope, global: &v8::Local<v8::Object>, key: &str) -> bool {
    let Some(k) = v8::String::new(scope, key) else {
        return false;
    };
    let Some(v) = global.get(scope, k.into()) else {
        return false;
    };
    v.is_true()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TBD-1：internal field 存 NodeId 经 External round-trip（rusty_v8 150.2.0 API 可用性）。
    #[test]
    fn poc_internal_field_round_trip_round_trips_node_id() {
        assert_eq!(poc_internal_field_round_trip(12345), 12345);
        assert_eq!(poc_internal_field_round_trip(0), 0);
        assert_eq!(poc_internal_field_round_trip(u32::MAX), u32::MAX);
    }

    /// TBD-2：Weak handle 在强引用释放 + GC 后变 empty（GC 安全机制可用）。
    #[test]
    fn tbd2_weak_handle_becomes_empty_on_gc() {
        assert!(
            poc_weak_handle_becomes_empty_on_gc(),
            "Weak handle 应在强引用释放 + GC 后变 empty（对象被回收）"
        );
    }

    /// TBD-1（S5）：native FunctionTemplate 构造器经 JS `class extends` 子类化——**S5 可行**（定型）。
    ///
    /// **发现（rusty_v8 150.2.0，instance_template accessor）**：
    /// - ✅ JS `class extends NativeCtor` **prototype 链继承工作**（子类实例 instanceof 基类 + 子类）。
    /// - ✅ 子类构造器**执行**（`super()` 调 native ctor，`__subRan=true`）。
    /// - ✅ **直接** `new NativeCtor()` 实例**有** internal field slot（nodeType=42）。
    /// - ✅ **JS 子类实例也拿到 internal field slot**——`super()` 调 native ctor 时 `this`（子类实例）**继承了
    ///   native instance_template 的 internal field 布局**，`set_internal_field(0,...)` 成功（返回 true），
    ///   native getter（instance_template accessor，holder=实例）读到 42（subclass_node_type=42）。
    ///
    /// **关键技术点**（PoC 调试得出）：native NodeId getter 必须挂在 **instance_template**（非 prototype_template）
    /// 的 accessor 上——instance_template accessor 的 `holder` = 实例（有 slot）；prototype_template accessor 的
    /// `holder` = 原型对象（无 slot，get_internal_field 返 None）。这是生产 native DOM 绑定的正确 accessor 挂载点
    ///（css_style_declaration/dataset 用 holder 的对象经此模型，但它们 holder 是被包装的元素 proxy，本 PoC 验证
    /// 构造器实例自身的 slot 可达）。
    ///
    /// **对 S5 customElements 的影响**：S5 `class MyEl extends HTMLElement`（HTMLElement = native 构造器）的
    /// 自定义元素实例**可直接复用 internal field 存 NodeId**（与 native-direct 元素一致的生产模式）——无需
    /// private symbol / WeakMap 替代存储。customElements upgrade 在 native ctor（super() 链）中填 slot[0]=NodeId，
    /// native getter 经 instance_template accessor 在子类实例上读到。**S5 可按既有 internal-field 架构推进**。
    #[test]
    fn poc_native_ctor_subclass_tbd1_s5_finding() {
        POC_CTOR_RAN.store(0, std::sync::atomic::Ordering::SeqCst);
        POC_SET_FIELD_OK.store(0, std::sync::atomic::Ordering::SeqCst);
        POC_GETTER_CALLED.store(0, std::sync::atomic::Ordering::SeqCst);
        POC_GETTER_READ.store(0, std::sync::atomic::Ordering::SeqCst);
        let (direct_nt, subclass_nt, instanceof_base, instanceof_sub, sub_ctor_ran) = poc_native_ctor_subclass();
        let ctor_ran = POC_CTOR_RAN.load(std::sync::atomic::Ordering::SeqCst);
        let set_ok = POC_SET_FIELD_OK.load(std::sync::atomic::Ordering::SeqCst);
        let getter_called = POC_GETTER_CALLED.load(std::sync::atomic::Ordering::SeqCst);
        let getter_read = POC_GETTER_READ.load(std::sync::atomic::Ordering::SeqCst);
        eprintln!(
            "DIAG direct_nt={direct_nt} sub_nt={subclass_nt} iob={instanceof_base} ios={instanceof_sub} \
             sub_ran={sub_ctor_ran} ctor_ran={ctor_ran} set_internal_field_ok={set_ok} \
             getter_called={getter_called} getter_read={getter_read}"
        );
        // 直接构造：native instance_template 产物 → 有 slot → nodeType=42。
        assert_eq!(direct_nt, 42, "new NativeCtor() 实例有 internal field slot");
        // JS 子类 prototype 链 + 构造器执行都工作。
        assert!(
            instanceof_base,
            "JS 子类实例 instanceof native 基类（prototype 链继承 ✓）"
        );
        assert!(instanceof_sub, "JS 子类实例 instanceof 自身子类 ✓");
        assert!(sub_ctor_ran, "JS 子类构造器执行（super() 调 native ctor）✓");
        // ✅ S5 可行：JS 子类实例经 super() 拿到 internal field slot（subclass_node_type=42）。
        assert_eq!(
            subclass_nt, 42,
            "JS `class extends NativeCtor` 子类实例拿到 internal field slot（instance_template 布局经 super() 继承）⇒ S5 customElements 可复用 internal-field NodeId 存储"
        );
    }
}
