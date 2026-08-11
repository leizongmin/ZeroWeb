//! P1b 原生 DOM 绑定的 GC + 状态层。
//!
//! 职责（RFC `docs/specs/p1b-v8-native-bindings-rfc.md` §3.1/§3.2 gc.rs）：
//! - **DOM 源**：线程局部持有 `Rc<RefCell<Document>>`（V8 Isolate 单线程，与执行线程
//!   绑定；`Rc<RefCell>` 无 `Send` 要求，贴合单线程访问语义）。
//! - **NodeId ↔ V8 对象映射**：`NodeId`(ffi u64) → `v8::Global<v8::Object>`，保证 JS
//!   对象身份（同 `NodeId` 返回同一对象，spec identity）。
//! - **Element 模板**：线程局部持 `Global<ObjectTemplate>`（含 `nodeType`/`tagName`
//!   accessor + internal slot[0] = NodeId），供工厂实例化。
//! - **stale 校验**：getter 读 NodeId 前校验节点仍在 DOM（移除节点 → JS 对象变 stale，
//!   spec detached 行为：getter 返 undefined）。
//!
//! **线程局部为何安全**：V8 Isolate 非线程安全，与执行线程绑定（`v8_runtime.rs` 文档
//! 注明）；所有 getter / 工厂回调经 V8 在同一线程派发，线程局部状态无跨线程访问。
//! 镜像 `script-sandbox::v8_runtime::HOST_CALLBACKS` 模式（ZST 回调 + 线程局部状态）。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use slotmap::{Key, KeyData};
use zero_dom::{Document, NodeId};

// 原生绑定使用的 DOM 源（`install_dom_bindings` 注入；getter 经 `with_dom` 读）。
// （doc comment 不附着 thread_local! 宏，用普通注释。）
thread_local! {
    static DOM_SOURCE: RefCell<Option<Rc<RefCell<Document>>>> = const { RefCell::new(None) };
    /// NodeId(ffi u64) → V8 对象 **Weak**（JS 对象身份：同 NodeId 返同对象）。
    /// R3133：存 weak 句柄（非 strong Global）——JS 丢弃包装器后 V8 可回收，终结器随之清 LISTENERS
    ///（闭合 R3109 节点 detach 泄漏）。weak 仍活时缓存命中（保身份）；weak 死则 [`get_or_create`]
    /// 重建。**重新附加语义**：节点 removeChild 仍留 arena（detached 但 `get` 可达），不清 LISTENERS
    /// ——监听器跨 detach/重附加保留（spec），仅在包装器真被 GC（JS 也丢）时由终结器回收。
    static NODE_OBJECTS: RefCell<HashMap<u64, v8::Weak<v8::Object>>> = RefCell::new(HashMap::new());
    /// Element ObjectTemplate（含 nodeType/tagName accessor + internal slot[0]）。
    static ELEMENT_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> = const { RefCell::new(None) };
    /// S4 EventTarget 原生（RFC §4 S4）：事件监听器——`(NodeId ffi, 事件类型, capture) → Global<Value>` 列表
    ///（存 Value 句柄，调用时 try_from 降 Function；存 Value 避 Local<Function>→Local<Value> upcast）。
    /// `capture` 标志区分 capture（祖先倒序、CAPTURING_PHASE）vs bubble（祖先正序、BUBBLING_PHASE）监听器
    ///（R3128 useCapture）。addEventListener 把 JS 回调存为 Global 强引用（跨 scope 持久）；dispatchEvent
    /// 在当前 scope 经 Local::new 复活后调用。R3133：节点包装器 weak 化后，包装器被 GC 时终结器调
    /// [`remove_node_listeners`] 清本节点全部监听器（闭合 R3109 泄漏）；removeChild 不清（保重新附加语义，
    /// detached 节点仍 arena 可达），仅 JS 也丢包装器 → GC → 终结器回收。reset 仍兜底清空。
    /// R3135：键由 `(ffi, type, capture)` 改 `(ffi, type)`，值 `Vec<(capture 标志, Global)>`——单列表保
    /// **全局注册序**（capture/bubble 监听器交错按 addEventListener 序），闭合 R3128 限制①（target 阶段
    /// 须按注册序触发全部监听器，不论 capture 标志；dispatch 按 phase 过滤）。
    static LISTENERS: RefCell<HashMap<(u64, String), Vec<(bool, v8::Global<v8::Value>)>>> =
        RefCell::new(HashMap::new());
    /// R3112 NamedNodeMap（element.attributes）：owner element NodeId(ffi) → **Weak**<Object>，
    /// 保 spec 身份（`el.attributes === el.attributes` 同对象）。R3134：weak 化（同 R3133 NODE_OBJECTS）——
    /// JS 丢 NNM 引用即可 GC（NNM 无辅助状态，无需终结器；rebuild 时 insert-overwrite 清死 Weak）。
    static NAMEDNODEMAP_OBJECTS: RefCell<HashMap<u64, v8::Weak<v8::Object>>> =
        RefCell::new(HashMap::new());
    /// R3112 NamedNodeMap ObjectTemplate（length getter + item/getNamedItem/... + internal slot[0]
    /// = owner element NodeId）。`install_dom_bindings` 建 + 缓存，`attributes` getter 实例化。
    static NAMEDNODEMAP_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> =
        const { RefCell::new(None) };
    /// R3122 Attr 节点属性名 arena（idx → name）。Attr 对象 internal slot[1] 存 idx，getter 经此复原名。
    /// 无 dedup——身份缓存 [`ATTR_OBJECTS`] 按 (owner ffi, name) 去重（R3134 已 weak 化，JS 丢 Attr 即 GC）；
    /// arena 仅供对象读回名，**仍 append-only（仅 reset 清）——残余小泄漏**（attr 名字符串，按 distinct attr
    /// 数 bounded；与 OBJECTS 缓存分离，后续可接 free-list / generation 回收）。
    static ATTR_NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// R3122 Attr 节点身份缓存：(owner ffi, attr name) → **Weak**<Object>（同 attr 返同对象，spec identity）。
    /// R3134：weak 化（同 R3133 NODE_OBJECTS / R3112 NNM）——JS 丢 Attr 引用即可 GC（Attr 无辅助状态，
    /// 无需终结器；rebuild 时 insert-overwrite 清死 Weak）。
    static ATTR_OBJECTS: RefCell<HashMap<(u64, String), v8::Weak<v8::Object>>> =
        RefCell::new(HashMap::new());
    /// R3122 Attr ObjectTemplate（nodeType=2 / name / nodeName / value(+setter) / nodeValue / textContent /
    /// ownerElement + internal slot[0]=owner ffi、slot[1]=attr 名 arena idx）。`install_dom_bindings` 建 + 缓存。
    static ATTR_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> = const { RefCell::new(None) };
    /// R3145 DOMTokenList（`element.classList`）：owner element NodeId(ffi) → **Weak**<Object>，
    /// 保 spec 身份（`el.classList === el.classList` 同对象——polyfill 旧每调新建，native 修正为 spec 合规）。
    /// R3134 同模式 weak 化（同 NNM/ATTR）——JS 丢 classList 引用即可 GC（无辅助状态，无需终结器；
    /// rebuild 时 insert-overwrite 清死 Weak）。
    static DOMTOKENLIST_OBJECTS: RefCell<HashMap<u64, v8::Weak<v8::Object>>> =
        RefCell::new(HashMap::new());
    /// R3145 DOMTokenList ObjectTemplate（length/value(+setter) + item/contains/add/remove/toggle/replace/
    /// toString + internal slot[0] = owner element NodeId）。`install_dom_bindings` 建 + 缓存，`classList`
    /// getter 实例化。
    static DOMTOKENLIST_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> =
        const { RefCell::new(None) };
    /// R3151 CSSStyleDeclaration（`element.style`）：owner element NodeId(ffi) → **Weak**<Object>，
    /// 保 spec 身份（`el.style === el.style` 同对象）。R3134/R3145 同模式 weak 化——JS 丢 style 引用即可 GC。
    static STYLE_OBJECTS: RefCell<HashMap<u64, v8::Weak<v8::Object>>> = RefCell::new(HashMap::new());
    /// R3151 CSSStyleDeclaration ObjectTemplate（cssText(+setter)/length/item + getPropertyValue/
    /// setProperty/removeProperty + named-property-handler 拦 camelCase 动态属性 + internal slot[0]
    /// = owner element NodeId）。`install_dom_bindings` 建 + 缓存，`style` getter 实例化。
    static STYLE_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> =
        const { RefCell::new(None) };
    /// R3152 DOMStringMap（`element.dataset`）：owner element NodeId(ffi) → **Weak**<Object>，
    /// 保 spec 身份（`el.dataset === el.dataset` 同对象）。R3134/R3145/R3151 同模式 weak 化。
    static DATASET_OBJECTS: RefCell<HashMap<u64, v8::Weak<v8::Object>>> = RefCell::new(HashMap::new());
    /// R3152 DOMStringMap ObjectTemplate（named-property-handler 拦 camelCase 动态属性 ↔ `data-*` 属性
    /// + internal slot[0] = owner element NodeId）。`install_dom_bindings` 建 + 缓存，`dataset` getter 实例化。
    static DATASET_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> =
        const { RefCell::new(None) };
    /// R3148 当前焦点元素（`document.activeElement` 对）：`element.focus()` 设、`element.blur()` 清。
    /// 线程局部 NodeId（无 ffi 包装——focus 仅 Rust 侧读写，不经 V8 句柄）。polyfill 旧 `_activeElKey`
    /// 纯 JS 状态（不派发 focus/blur 事件）；native 经此追踪 + 真实派发 focus/blur（闭合 polyfill 限制②）。
    static ACTIVE_ELEMENT: RefCell<Option<NodeId>> = const { RefCell::new(None) };
    /// R3265 S5b custom element upgrade 注入栈：`document.createElement('my-el')` 命中 polyfill
    /// `_ce_registry` 时，host 已先建元素得 NodeId（`native_create_element_invoke`），需把该 NodeId
    /// 注入到 custom ctor 的 `super()` → `native_html_element_ctor_invoke` 链中（否则 ctor 会建新
    /// detached div，与 host 建的元素脱节——两个 NodeId）。镜像 `ACTIVE_ELEMENT` 线程局部模式：
    /// `native_create_element_invoke` push → JS ctor `new_instance`（super() 读栈顶填 slot[0]）→ pop。
    /// **R3272 栈化**：原单 `Option<NodeId>` 在嵌套 upgrade（ctor body 内 `createElement` 另一个 custom
    /// 元素）时内层 set 覆盖外层 → 外层 super() 读到内层 NodeId 或 None（身份错乱）。改 `Vec<NodeId>` 栈：
    /// 内层 push/pop 不影响外层（栈顶隔离），正确处理嵌套 upgrade。
    static UPGRADE_NODE_ID: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    /// R3266 S5c custom element 连接态追踪：已连入 document 的 custom 元素 NodeId(ffi) 集合。
    /// native_dom 路径 appendChild/insertBefore/removeChild 经 Rust 直接改 DOM，绕过 polyfill 的
    /// `_ceApplyConn`（基于 sel/handle），故连接态由 Rust 权威追踪。变更（未连→连 / 已连→断）时，
    /// [`custom_elements::notify_connect_change`] 桥接 JS 派发 connectedCallback/disconnectedCallback
    ///（以 native 实例作 `this`，复用 polyfill `_ce_registry` + ctor.prototype）。
    static CONNECTED_CUSTOM: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    /// R3159 `document` 对象（单例，无 NodeId 键——synthetic 命名空间对象）：弱缓存保 spec 身份
    ///（`__zw_native_get_document() === __zw_native_get_document()` 同对象）。JS 丢 `document` 引用即可 GC。
    static DOCUMENT_OBJECT: RefCell<Option<v8::Weak<v8::Object>>> = const { RefCell::new(None) };
    /// R3159 `document` ObjectTemplate（方法复用 factories 子模块 + getter 读 live Document +
    /// title get/set）。`install_dom_bindings` 建 + 缓存，`__zw_native_get_document` 工厂实例化。
    static DOCUMENT_TEMPLATE: RefCell<Option<v8::Global<v8::ObjectTemplate>>> =
        const { RefCell::new(None) };
}

/// 注入 DOM 源 + Element 模板（`install_dom_bindings` 调用）。
pub(crate) fn set_dom_source(dom: Rc<RefCell<Document>>) {
    DOM_SOURCE.with(|c| *c.borrow_mut() = Some(dom));
}

/// 缓存 Element ObjectTemplate（供工厂 [`create_native_element`] 实例化）。
pub(crate) fn set_element_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    ELEMENT_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 Element ObjectTemplate 的 Local（工厂实例化用）。
pub(crate) fn element_template_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    ELEMENT_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 清空全部绑定状态（reset_context / 导航重建时调用；下一切片接线时接入）。
#[allow(dead_code)] // 生产入口：本切片仅测试用；run_page_scripts 接线（下一切片）接入导航/重载重置
pub(crate) fn reset() {
    DOM_SOURCE.with(|c| *c.borrow_mut() = None);
    NODE_OBJECTS.with(|c| c.borrow_mut().clear());
    ELEMENT_TEMPLATE.with(|c| *c.borrow_mut() = None);
    LISTENERS.with(|c| c.borrow_mut().clear());
    NAMEDNODEMAP_OBJECTS.with(|c| c.borrow_mut().clear());
    NAMEDNODEMAP_TEMPLATE.with(|c| *c.borrow_mut() = None);
    ATTR_NAMES.with(|c| c.borrow_mut().clear());
    ATTR_OBJECTS.with(|c| c.borrow_mut().clear());
    ATTR_TEMPLATE.with(|c| *c.borrow_mut() = None);
    DOMTOKENLIST_OBJECTS.with(|c| c.borrow_mut().clear());
    DOMTOKENLIST_TEMPLATE.with(|c| *c.borrow_mut() = None);
    STYLE_OBJECTS.with(|c| c.borrow_mut().clear());
    STYLE_TEMPLATE.with(|c| *c.borrow_mut() = None);
    DATASET_OBJECTS.with(|c| c.borrow_mut().clear());
    DATASET_TEMPLATE.with(|c| *c.borrow_mut() = None);
    ACTIVE_ELEMENT.with(|c| *c.borrow_mut() = None);
    UPGRADE_NODE_ID.with(|c| c.borrow_mut().clear());
    CONNECTED_CUSTOM.with(|c| c.borrow_mut().clear());
    DOCUMENT_OBJECT.with(|c| *c.borrow_mut() = None);
    DOCUMENT_TEMPLATE.with(|c| *c.borrow_mut() = None);
}

/// 缓存 NamedNodeMap ObjectTemplate（`install_dom_bindings` 建 `attributes` 集合模板）。
pub(crate) fn set_namednodemap_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    NAMEDNODEMAP_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 NamedNodeMap ObjectTemplate 的 Local（`attributes` getter 实例化用）。
pub(crate) fn namednodemap_template_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    NAMEDNODEMAP_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 NamedNodeMap 对象（同 owner element 返同对象，spec identity）；weak 死（NNM 被 GC）→ `None`
/// （调用方 rebuild）。R3134：weak 缓存，JS 丢引用即可 GC（闭合属性集合同 pattern 泄漏）。
pub(crate) fn cached_namednodemap<'s>(scope: &mut v8::PinScope<'s, '_>, ffi: u64) -> Option<v8::Local<'s, v8::Object>> {
    NAMEDNODEMAP_OBJECTS.with(|c| c.borrow().get(&ffi).and_then(|w| w.to_local(scope)))
}

/// 缓存 NamedNodeMap 对象（owner element ffi → **Weak**）。R3134：weak 句柄不阻止 GC——JS 丢引用后
/// NNM 可回收，rebuild 时 insert-overwrite 清死 Weak 条目（NNM 无辅助状态，无需终结器）。
pub(crate) fn cache_namednodemap(scope: &mut v8::PinScope, ffi: u64, obj: v8::Local<v8::Object>) {
    NAMEDNODEMAP_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, v8::Weak::new(scope, obj));
    });
}

// ── R3145 DOMTokenList（element.classList）状态 ────────────────────

/// 缓存 DOMTokenList ObjectTemplate（`install_dom_bindings` 建 `classList` 集合模板）。
pub(crate) fn set_domtokenlist_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    DOMTOKENLIST_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 DOMTokenList ObjectTemplate 的 Local（`classList` getter 实例化用）。
pub(crate) fn domtokenlist_template_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    DOMTOKENLIST_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 DOMTokenList 对象（同 owner element 返同对象，spec identity）；weak 死（DTL 被 GC）→ `None`
/// （调用方 rebuild）。R3145：weak 缓存（同 NNM / ATTR），JS 丢引用即可 GC。
pub(crate) fn cached_domtokenlist<'s>(scope: &mut v8::PinScope<'s, '_>, ffi: u64) -> Option<v8::Local<'s, v8::Object>> {
    DOMTOKENLIST_OBJECTS.with(|c| c.borrow().get(&ffi).and_then(|w| w.to_local(scope)))
}

/// 缓存 DOMTokenList 对象（owner element ffi → **Weak**）。R3145：weak 句柄不阻止 GC——rebuild 时
/// insert-overwrite 清死 Weak（DTL 无辅助状态，无需终结器）。
pub(crate) fn cache_domtokenlist(scope: &mut v8::PinScope, ffi: u64, obj: v8::Local<v8::Object>) {
    DOMTOKENLIST_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, v8::Weak::new(scope, obj));
    });
}

// ── R3151 CSSStyleDeclaration（element.style）状态 ────────────────────

/// 缓存 CSSStyleDeclaration ObjectTemplate（`install_dom_bindings` 建 `style` 集合模板）。
pub(crate) fn set_style_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    STYLE_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 CSSStyleDeclaration ObjectTemplate 的 Local（`style` getter 实例化用）。
pub(crate) fn style_template_local<'s>(scope: &mut v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    STYLE_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 CSSStyleDeclaration 对象（同 owner element 返同对象，spec identity）；weak 死 → `None`。
pub(crate) fn cached_style<'s>(scope: &mut v8::PinScope<'s, '_>, ffi: u64) -> Option<v8::Local<'s, v8::Object>> {
    STYLE_OBJECTS.with(|c| c.borrow().get(&ffi).and_then(|w| w.to_local(scope)))
}

/// 缓存 CSSStyleDeclaration 对象（owner element ffi → **Weak**）。
pub(crate) fn cache_style(scope: &mut v8::PinScope, ffi: u64, obj: v8::Local<v8::Object>) {
    STYLE_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, v8::Weak::new(scope, obj));
    });
}

// ── R3152 DOMStringMap（element.dataset）状态 ────────────────────────

/// 缓存 DOMStringMap ObjectTemplate（`install_dom_bindings` 建 `dataset` 集合模板）。
pub(crate) fn set_dataset_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    DATASET_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 DOMStringMap ObjectTemplate 的 Local（`dataset` getter 实例化用）。
pub(crate) fn dataset_template_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    DATASET_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 DOMStringMap 对象（同 owner element 返同对象，spec identity）；weak 死 → `None`。
pub(crate) fn cached_dataset<'s>(scope: &mut v8::PinScope<'s, '_>, ffi: u64) -> Option<v8::Local<'s, v8::Object>> {
    DATASET_OBJECTS.with(|c| c.borrow().get(&ffi).and_then(|w| w.to_local(scope)))
}

/// 缓存 DOMStringMap 对象（owner element ffi → **Weak**）。
pub(crate) fn cache_dataset(scope: &mut v8::PinScope, ffi: u64, obj: v8::Local<v8::Object>) {
    DATASET_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, v8::Weak::new(scope, obj));
    });
}

// ── R3159 `document` 对象（单例）状态 ────────────────────────────────

/// 缓存 `document` ObjectTemplate（`install_dom_bindings` 建）。
pub(crate) fn set_document_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    DOCUMENT_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 `document` ObjectTemplate 的 Local（`__zw_native_get_document` 工厂实例化用）。
pub(crate) fn document_template_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    DOCUMENT_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 `document` 对象（单例 identity，spec `document === document`）；weak 死 → `None`。
pub(crate) fn cached_document<'s>(scope: &mut v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Object>> {
    DOCUMENT_OBJECT.with(|c| c.borrow().as_ref().and_then(|w| w.to_local(scope)))
}

/// 缓存 `document` 对象（单例 **Weak**——JS 丢引用即可 GC，下次取重建）。
pub(crate) fn cache_document(scope: &mut v8::PinScope, obj: v8::Local<v8::Object>) {
    DOCUMENT_OBJECT.with(|c| *c.borrow_mut() = Some(v8::Weak::new(scope, obj)));
}

// ── R3122 Attr 节点状态 ───────────────────────────────────────────

/// 追加属性名到 arena，返 idx（Attr 对象 internal slot[1] 存 `idx+1`，getter 经此复原名）。
pub(crate) fn add_attr_name(name: String) -> u32 {
    ATTR_NAMES.with(|c| {
        let mut v = c.borrow_mut();
        let idx = v.len() as u32;
        v.push(name);
        idx
    })
}

/// 取 arena idx 处的属性名（Attr getter 经 slot[1] idx 复原名）。越界 / 空 → `None`。
pub(crate) fn attr_name(idx: u32) -> Option<String> {
    ATTR_NAMES.with(|c| c.borrow().get(idx as usize).cloned())
}

/// 缓存 Attr ObjectTemplate（`install_dom_bindings` 建）。
pub(crate) fn set_attr_template(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    ATTR_TEMPLATE.with(|c| *c.borrow_mut() = Some(v8::Global::new(scope, tmpl)));
}

/// 取 Attr ObjectTemplate 的 Local（getNamedItem/item 实例化用）。
pub(crate) fn attr_template_local<'s>(scope: &mut v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::ObjectTemplate>> {
    ATTR_TEMPLATE.with(|c| c.borrow().as_ref().map(|g| v8::Local::new(scope, g)))
}

/// 取已缓存的 Attr 对象（同 (owner, name) 返同对象，spec identity）；weak 死（Attr 被 GC）→ `None`
/// （调用方 rebuild）。R3134：weak 缓存（同 NNM / NODE_OBJECTS）。
pub(crate) fn cached_attr<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner_ffi: u64,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    ATTR_OBJECTS.with(|c| {
        c.borrow()
            .get(&(owner_ffi, name.to_string()))
            .and_then(|w| w.to_local(scope))
    })
}

/// 缓存 Attr 对象（(owner ffi, name) → **Weak**）。R3134：weak 句柄不阻止 GC——JS 丢引用后 Attr 可回收，
/// rebuild 时 insert-overwrite 清死 Weak 条目（Attr 无辅助状态，无需终结器）。
pub(crate) fn cache_attr(scope: &mut v8::PinScope, owner_ffi: u64, name: &str, obj: v8::Local<v8::Object>) {
    ATTR_OBJECTS.with(|c| {
        c.borrow_mut()
            .insert((owner_ffi, name.to_string()), v8::Weak::new(scope, obj));
    });
}

/// 在当前 DOM 源上执行只读操作；无 DOM 源时返 `None`。
///
/// 先克隆 `Rc` 出线程局部（释放 borrow）再 `RefCell::borrow`，避免 borrow 跨调用边界。
pub(crate) fn with_dom<R>(f: impl FnOnce(&Document) -> R) -> Option<R> {
    let rc = DOM_SOURCE.with(|c| c.borrow().clone())?;
    let doc = rc.borrow();
    // 仅 `&self` 读操作（node_type / get / get_element_by_id）；无嵌套 borrow_mut，
    // 无 JS 回调再入，RefCell borrow 安全。
    Some(f(&doc))
}

/// 在当前 DOM 源上执行**可变**操作（setAttribute / removeAttribute 等）；无 DOM 源时返 `None`。
///
/// 镜像 [`with_dom`] 但 `borrow_mut`。安全前提：闭包内 `&mut Document` 操作（set_attribute 等）
/// 不触发 JS 回调再入（纯 Rust mutation + record_mutation 推 pending_mutations，无 observer 回调），
/// 故无嵌套 `borrow_mut`；V8 回调顶层单次持 borrow。
pub(crate) fn with_dom_mut<R>(f: impl FnOnce(&mut Document) -> R) -> Option<R> {
    let rc = DOM_SOURCE.with(|c| c.borrow().clone())?;
    let mut doc = rc.borrow_mut();
    Some(f(&mut doc))
}

// ── NodeId ↔ u64(ffi) 编解码 ──────────────────────────────────────

/// `NodeId` → u64（slotmap `KeyData::as_ffi`）。internal slot 经 `v8::External`
/// 存 ptr 值（无堆分配，镜像 S0 PoC `poc_internal_field_round_trip`）。
pub(crate) fn encode_node_id(id: NodeId) -> u64 {
    id.data().as_ffi()
}

/// u64(ffi) → `NodeId`（slotmap `KeyData::from_ffi`，`new_key_type!` 生成 `From<KeyData>`）。
pub(crate) fn decode_node_id(ffi: u64) -> NodeId {
    NodeId::from(KeyData::from_ffi(ffi))
}

// ── R3148 当前焦点元素（document.activeElement 对）──────────────────

/// 取当前焦点元素 NodeId（`document.activeElement` 对）；无焦点 → `None`。
pub(crate) fn active_element() -> Option<NodeId> {
    ACTIVE_ELEMENT.with(|c| *c.borrow())
}

/// 设当前焦点元素（`element.focus()` 设 Some、`element.blur()` 设 None）。
pub(crate) fn set_active_element(id: Option<NodeId>) {
    ACTIVE_ELEMENT.with(|c| *c.borrow_mut() = id);
}

// ── R3265 S5b custom element upgrade 注入栈（createElement('my-el') → super() 链，R3272 栈化）──

/// 取当前 upgrade 注入 NodeId（栈顶）。`native_create_element_invoke` 命中 registry 时 push，
/// `native_html_element_ctor_invoke` super() 读栈顶。无 upgrade 在途 → `None`（S5a 直接 new 行为）。
pub(crate) fn upgrade_node_id() -> Option<NodeId> {
    UPGRADE_NODE_ID.with(|c| c.borrow().last().copied())
}

/// push upgrade 注入 NodeId（`native_create_element_invoke` 调 JS ctor 前调，填 ctor super() 链）。
/// R3272 栈语义：嵌套 upgrade（ctor 内建另一个 custom 元素）内层 push 不覆盖外层（栈顶隔离）。
/// `id=None` 兼容旧调用（清空整个栈——实际无调用方传 None，保签名兼容）。
pub(crate) fn set_upgrade_node_id(id: Option<NodeId>) {
    UPGRADE_NODE_ID.with(|c| {
        if let Some(node_id) = id {
            c.borrow_mut().push(node_id);
        } else {
            c.borrow_mut().clear();
        }
    });
}

/// pop upgrade 注入 NodeId（`native_create_element_invoke` 调 JS ctor 后调，防泄漏到后续 new HTMLElement）。
/// R3272 栈语义：pop 栈顶（嵌套 upgrade 内层 pop 不影响外层）。栈空则 no-op。
pub(crate) fn clear_upgrade_node_id() {
    UPGRADE_NODE_ID.with(|c| {
        c.borrow_mut().pop();
    });
}

// ── R3266 S5c custom element 连接态追踪（connectedCallback/disconnectedCallback 派发门控）──

/// custom 元素 NodeId(ffi) 是否已连入 document（在 `CONNECTED_CUSTOM` 集合中）。
pub(crate) fn is_custom_connected(ffi: u64) -> bool {
    CONNECTED_CUSTOM.with(|c| c.borrow().contains(&ffi))
}

/// 标记 custom 元素已连入 document（返 true = 状态真转 未连→连，应派 connectedCallback）。
pub(crate) fn mark_custom_connected(ffi: u64) -> bool {
    CONNECTED_CUSTOM.with(|c| c.borrow_mut().insert(ffi))
}

/// 标记 custom 元素已断开 document（返 true = 状态真转 已连→断，应派 disconnectedCallback）。
pub(crate) fn unmark_custom_connected(ffi: u64) -> bool {
    CONNECTED_CUSTOM.with(|c| c.borrow_mut().remove(&ffi))
}

// ── NodeId ↔ V8 对象身份映射 ──────────────────────────────────────

/// 取已缓存的 native element 对象（同 NodeId 返同对象）；weak 已死（包装器被 GC）→ `None`
/// （调用方重建）。**不含 stale 校验**（节点是否仍在 arena 由调用方 [`node_exists`] 决定）。
pub(crate) fn cached_native_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    ffi: u64,
) -> Option<v8::Local<'s, v8::Object>> {
    NODE_OBJECTS.with(|c| c.borrow().get(&ffi).and_then(|w| w.to_local(scope)))
}

/// 缓存新创建的 native element 对象（NodeId ffi → **Weak** + 终结器）。R3133：weak 句柄不阻止 GC，
/// 终结器（guaranteed）在包装器被 GC 时清本节点 LISTENERS（闭合 R3109 泄漏）。Weak 须留存缓存
/// （终结器生命周期绑 Weak；Weak 先死则终结器不触发）——empty Weak 由 [`drop_cached_native_element`]
/// 在重建时惰性清理。
pub(crate) fn cache_native_element(scope: &mut v8::PinScope, ffi: u64, obj: v8::Local<v8::Object>) {
    // guaranteed 终结器：GC 时或 isolate 销毁前必触发（FnOnce()，无需 Isolate 句柄）。
    // 闭包捕获 ffi，清本节点全部监听器（线程局部 LISTENERS，终结器在 isolate 线程跑，可安全访问）。
    let weak = v8::Weak::with_guaranteed_finalizer(scope, obj, Box::new(move || remove_node_listeners(ffi)));
    NODE_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, weak);
    });
}

/// 移除缓存的 weak 句柄（节点离场 arena / 重建前清残留 empty Weak）。Weak drop 时若对象仍活，
/// 终结器取消（此时节点已不可达，监听器亦不可达，reset 兜底清；非回归——旧 strong 缓存同样残留）。
pub(crate) fn drop_cached_native_element(ffi: u64) {
    NODE_OBJECTS.with(|c| {
        c.borrow_mut().remove(&ffi);
    });
}

/// stale 校验：节点是否仍在 DOM 中（getter / 工厂重建前调）。
pub(crate) fn node_exists(id: NodeId) -> bool {
    with_dom(|d| d.get(id).is_some()).unwrap_or(false)
}

// ── S4 EventTarget 监听器存储 ─────────────────────────────────────

/// 追加监听器（`(NodeId ffi, 事件类型)` → `(capture, Global<Value>)`）。R3135：单列表保**全局注册序**
///（capture/bubble 监听器交错按 addEventListener 序），闭合 R3128 限制①（target 阶段按注册序触发全部）。
/// `capture` 标志记录每条监听器的阶段归属（dispatch 按 phase 过滤）。
pub(crate) fn add_listener(ffi: u64, event_type: String, capture: bool, f: v8::Global<v8::Value>) {
    LISTENERS.with(|c| c.borrow_mut().entry((ffi, event_type)).or_default().push((capture, f)));
}

/// 取监听器在**当前 scope** 复活的 `(capture, Local<Value>)` 列表（dispatchEvent 用）——**全部**条目按
/// 注册序（dispatch 按 phase 过滤 capture/bubble）。R3135：不再分桶取，返完整列表含 capture 标志。
///
/// 刻意不持 LISTENERS borrow 跨 JS 回调——复活为 Local 后释放 borrow，回调内 addEventListener /
/// removeEventListener 再入不会 panic（新增的监听器不在本快照内，符合 spec「派发期间新增不触发」）。
pub(crate) fn listeners_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    ffi: u64,
    event_type: &str,
) -> Vec<(bool, v8::Local<'s, v8::Value>)> {
    LISTENERS.with(|c| {
        c.borrow()
            .get(&(ffi, event_type.to_string()))
            .map(|vec| vec.iter().map(|(cap, g)| (*cap, v8::Local::new(scope, g))).collect())
            .unwrap_or_default()
    })
}

/// 检查 `(ffi, event_type, capture, listener 身份)` 是否仍在 LISTENERS 中（dispatchEvent 派发期间用）。
///
/// spec DOM「inner invoke」：派发期间被 `removeEventListener` 的监听器（快照仍含其 Local）须 **skip**
///（spec 检 `listener.removed` 标志）。[`listeners_local`] 返回快照不复检存活，故 dispatch 循环对本条目
/// 调此 helper——同 `capture` 且 `strict_equals` 身份仍存在于 map 则存活，否则 skip。典型监听器数小，O(n) 查可接受。
pub(crate) fn listener_present(
    scope: &mut v8::PinScope,
    ffi: u64,
    event_type: &str,
    capture: bool,
    target: v8::Local<v8::Value>,
) -> bool {
    LISTENERS.with(|c| {
        c.borrow().get(&(ffi, event_type.to_string())).is_some_and(|vec| {
            vec.iter()
                .any(|(cap, g)| *cap == capture && v8::Local::new(scope, g).strict_equals(target))
        })
    })
}

/// 移除与 `target`（Local）同身份且 `capture` 匹配的监听器；返移除数（removeEventListener 用）。
/// spec：capture/bubble 监听器独立（removeEventListener 须匹配 capture 标志），故仅删 `capture` 匹配
/// 且身份相同的条目。持 LISTENERS borrow_mut 期间仅做 `Local::new` + `strict_equals`（非 JS 回调，无再入），安全。
pub(crate) fn remove_listener(
    scope: &mut v8::PinScope,
    ffi: u64,
    event_type: &str,
    capture: bool,
    target: v8::Local<v8::Value>,
) -> usize {
    LISTENERS.with(|c| {
        let mut map = c.borrow_mut();
        let Some(vec) = map.get_mut(&(ffi, event_type.to_string())) else {
            return 0;
        };
        let before = vec.len();
        vec.retain(|(cap, g)| {
            !(*cap == capture && {
                let local = v8::Local::new(scope, g);
                local.strict_equals(target)
            })
        });
        before - vec.len()
    })
}

/// R3133 节点包装器终结器：清本节点（ffi）**全部**监听器（所有事件类型 × capture/bubble 桶）。
/// 包装器被 GC 时由 [`cache_native_element`] 注册的 guaranteed 终结器调用（isolate 线程，线程局部
/// LISTENERS 可安全访问）。removeChild 不调本函数（保重新附加语义：detached 节点监听器跨 detach 保留）。
///
/// R3272：同时清 `CONNECTED_CUSTOM`——元素 GC 后 NodeId 可能被 slotmap 复用（新元素得同 ffi），若不清，
/// 新元素 `is_custom_connected` 误判已连 → connectedCallback 不触发（状态污染）。包装器 GC = JS 丢引用 +
/// 节点离场，连接态应随之清。
pub(crate) fn remove_node_listeners(ffi: u64) {
    LISTENERS.with(|c| {
        c.borrow_mut().retain(|(f, _), _| *f != ffi);
    });
    unmark_custom_connected(ffi);
}

#[cfg(test)]
pub(crate) mod test_helpers {
    //! 测试辅助：注入 DOM 源后断言线程局部状态。仅 `#[cfg(test)]` 暴露。

    use super::*;

    /// 注入 DOM 源（测试用，绕过 kill-switch）。
    pub fn inject_dom_for_test(dom: Rc<RefCell<Document>>) {
        set_dom_source(dom);
    }

    /// 清空全部绑定状态（测试间隔离）。
    pub fn reset_for_test() {
        reset();
    }

    /// R3133：本节点（ffi）的 LISTENERS 条目数（事件类型计数；R3135 后 capture 合入单列表，每事件类型 1 条目）。
    /// 终结器测试用——包装器被 GC 后断言本节点监听器已清。
    pub fn listener_keys_for(ffi: u64) -> usize {
        LISTENERS.with(|c| c.borrow().keys().filter(|(f, _)| *f == ffi).count())
    }

    /// R3133：LISTENERS 全部条目数（终结器测试用）。
    pub fn listener_total_entries() -> usize {
        LISTENERS.with(|c| c.borrow().len())
    }

    /// R3134：本 owner 元素的 NNM weak 句柄是否仍活（NNM 未被 GC）。weak-reclaim 测试用。
    pub fn nnm_cache_alive(ffi: u64) -> bool {
        NAMEDNODEMAP_OBJECTS.with(|c| c.borrow().get(&ffi).is_some_and(|w| !w.is_empty()))
    }

    /// R3134：本 (owner, name) 的 Attr weak 句柄是否仍活（Attr 未被 GC）。weak-reclaim 测试用。
    pub fn attr_cache_alive(owner_ffi: u64, name: &str) -> bool {
        ATTR_OBJECTS.with(|c| {
            c.borrow()
                .get(&(owner_ffi, name.to_string()))
                .is_some_and(|w| !w.is_empty())
        })
    }

    /// R3145：本 owner 元素的 DOMTokenList weak 句柄是否仍活（classList 未被 GC）。weak-reclaim 测试用。
    pub fn dtl_cache_alive(ffi: u64) -> bool {
        DOMTOKENLIST_OBJECTS.with(|c| c.borrow().get(&ffi).is_some_and(|w| !w.is_empty()))
    }
}
