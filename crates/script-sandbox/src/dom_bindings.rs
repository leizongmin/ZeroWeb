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
}
