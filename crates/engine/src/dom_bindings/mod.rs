//! P1b 原生 DOM 绑定——替换 polyfill 字符串桥（S1 生产化）。
//!
//! 把 Rust DOM 的 `NodeId` 包装为 V8 对象（`ObjectTemplate` + internal slot[0] 存
//! NodeId），`nodeType`/`tagName` 经原生 accessor getter 直接读 Rust DOM，**不经 shim
//! 字符串桥**（值传递：`v8::Integer`/`v8::String` 原生返回）。
//!
//! - RFC：`docs/specs/p1b-v8-native-bindings-rfc.md`（方案 C 混合 DOM-Node）。
//! - 架构决策：绑定置于 `engine`（拥有 DOM），engine 加 feature-gated `v8` dep；getter
//!   经线程局部 DOM 源（`gc.rs`）读真实 DOM。
//! - 接线：[`install_dom_bindings_if_enabled`] kill-switch 门控（默认关 → 零回归）；
//!   run_page_scripts 生产接线（live Document 共享 + V8Sandbox 上下文安装）为下一切片。
//!
//! spec：`nodeType` https://dom.spec.whatwg.org/#dom-node-nodetype（Element=1）；
//! `tagName` https://dom.spec.whatwg.org/#dom-element-tagname（HTML 大写）。

mod gc;

use std::cell::RefCell;
use std::rc::Rc;

use zero_dom::{Document, NodeId, NodeKind};

// gc 的线程局部访问器（crate-private）。
use gc::{
    cache_native_element, cached_native_element, decode_node_id, drop_cached_native_element, element_template_local,
    encode_node_id, node_exists, set_dom_source, set_element_template, with_dom,
};

/// P1b 原生 DOM 绑定 kill-switch 环境变量名（默认关）。
pub const ZW_NATIVE_DOM_ENV: &str = "ZW_NATIVE_DOM";

/// kill-switch 是否开启（env `ZW_NATIVE_DOM=1|true`）。默认关 → 零回归。
///
/// 进程级静态缓存（`OnceLock`）避免每次安装读 env；env 在进程启动时确定。
pub fn native_dom_enabled() -> bool {
    use std::sync::OnceLock;
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| {
        std::env::var(ZW_NATIVE_DOM_ENV)
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// 生产入口：kill-switch 开启时安装原生 DOM 绑定；关闭时 no-op（零回归）。
///
/// 返回是否实际安装（开启且管线就绪）。bench / 单测直接调 [`install_dom_bindings`]
/// 验证管线（不经 kill-switch）。
pub fn install_dom_bindings_if_enabled(
    scope: &mut v8::PinScope,
    ctx: v8::Local<v8::Context>,
    dom: Rc<RefCell<Document>>,
) -> bool {
    if !native_dom_enabled() {
        return false;
    }
    install_dom_bindings(scope, ctx, dom);
    true
}

/// 安装原生 DOM 绑定到指定 V8 上下文。
///
/// - 注入 DOM 源（线程局部，供 getter 读）。
/// - 创建 Element `ObjectTemplate`（internal_field_count=1 + `nodeType`/`tagName`
///   accessor getter），缓存供工厂实例化。
/// - 注册全局工厂 `__zw_native_element_for_id(idStr)`：`get_element_by_id` 解析 →
///   NodeId → 创建/查找 native element 对象（NodeId↔对象身份映射）。
///
/// 幂等：重复调用重置 DOM 源 + 模板（导航/重载场景）。
///
/// **单文件 ≤ 2000 行**：本模块仅首组 getter（nodeType/tagName），后续属性族按 RFC §4
/// S1 只读 / S2 写入拆分扩展（element.rs / document.rs 等子模块）。
pub fn install_dom_bindings(scope: &mut v8::PinScope, ctx: v8::Local<v8::Context>, dom: Rc<RefCell<Document>>) {
    // 1. DOM 源注入（getter 经线程局部读真实 DOM，不经序列化 HTML 串）。
    set_dom_source(dom);

    // 2. Element ObjectTemplate：internal slot[0] 存 NodeId + nodeType/tagName accessor。
    let tmpl = v8::ObjectTemplate::new(scope);
    tmpl.set_internal_field_count(1);
    // spec: nodeType / tagName（Element 上只读属性）。accessor getter 为 ZST fn
    //（UnitType），状态经 gc.rs 线程局部（镜像 HOST_CALLBACKS 模式）。
    if let Some(k) = v8::String::new(scope, "nodeType") {
        tmpl.set_accessor(k.into(), native_node_type_getter);
    }
    if let Some(k) = v8::String::new(scope, "tagName") {
        tmpl.set_accessor(k.into(), native_tag_name_getter);
    }
    set_element_template(scope, tmpl);

    // 3. 全局工厂 __zw_native_element_for_id(idStr) → native element 对象。
    let global = ctx.global(scope);
    let factory = v8::FunctionTemplate::builder(native_element_factory_invoke).build(scope);
    let Some(f) = factory.get_function(scope) else {
        return;
    };
    let Some(key) = v8::String::new(scope, "__zw_native_element_for_id") else {
        return;
    };
    let _ = global.set(scope, key.into(), f.into());
}

// ── accessor getter（ZST fn；状态经 gc.rs 线程局部）─────────────────

/// `nodeType` getter：读 internal slot[0] NodeId → `Document::node_type` → `v8::Integer`。
///
/// stale（节点移除）或无 NodeId → 留 undefined（spec detached 行为）。
fn native_node_type_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    // with_dom 返 Option<Option<u8>>（外层=无 DOM 源，内层=节点无 node_type）。
    let nt: Option<u8> = with_dom(|d| d.node_type(id)).flatten();
    if let Some(nt) = nt {
        rv.set(v8::Integer::new(scope, i32::from(nt)).into());
    }
}

/// `tagName` getter：读 internal slot[0] NodeId → Element `local_name` → 大写 → `v8::String`。
///
/// 仅 Element 有 tagName（HTML 大写，spec `dom-element-tagname`）；非 Element / stale →
/// undefined。
fn native_tag_name_getter(
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

// ── 工厂回调（global __zw_native_element_for_id）───────────────────

/// 工厂回调：`__zw_native_element_for_id(idStr)` → 解析 `get_element_by_id` →
/// NodeId → 创建/查找 native element 对象（NodeId↔对象身份映射 + stale 重建）。
///
/// 未找到 id → `null`。NodeId 编码进 internal slot[0]（`v8::External` ptr 值）。
fn native_element_factory_invoke(
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

// ── NodeId ↔ internal slot 读写 + 对象身份映射 ────────────────────

/// 从对象 internal slot[0] 读 NodeId（`v8::External` ptr 值 → u64 → NodeId）。
///
/// 无 slot 值（非 native element 对象误用 accessor）→ `None`（getter 留 undefined）。
fn read_node_id(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<NodeId> {
    let data = obj.get_internal_field(scope, 0)?;
    let ext = data.cast::<v8::External>();
    Some(decode_node_id(ext.value() as usize as u64))
}

/// 创建或复用 native element 对象（NodeId↔对象身份映射 + stale 重建）。
///
/// - 缓存命中且节点仍存在 → 返同一对象（spec identity）。
/// - 缓存命中但 stale（节点移除）→ 移除缓存 + 重建。
/// - 未命中 → 实例化 Element 模板 + 存 NodeId 进 internal slot[0] + 缓存。
fn get_or_create_native_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    node_id: NodeId,
) -> Option<v8::Local<'s, v8::Object>> {
    let ffi = encode_node_id(node_id);
    // 缓存命中：stale 校验决定复用 / 重建。
    if let Some(cached) = cached_native_element(scope, ffi) {
        if node_exists(node_id) {
            return Some(cached);
        }
        drop_cached_native_element(ffi);
    }
    // 未命中 / stale 重建：实例化 Element 模板 + 存 NodeId。
    let tmpl = element_template_local(scope)?;
    let obj = tmpl.new_instance(scope)?;
    // NodeId 经 External ptr 值存 internal slot[0]（无堆分配，镜像 S0 PoC）。
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_native_element(scope, ffi, obj);
    Some(obj)
}

#[cfg(test)]
mod tests;
