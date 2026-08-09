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

mod element;
mod event;
mod event_target;
mod factories;
mod gc;
mod namednodemap;
mod node;

use std::cell::RefCell;
use std::rc::Rc;

use zero_dom::{Document, NodeId};

// gc 的线程局部访问器（crate-private）。
// R3116/R3117：NNM + EventTarget 相关 gc 助手（cache/cached/namednodemap_template_local/
// set_namednodemap_template + add_listener/listeners_local/remove_listener）迁对应子模块，不再于此导入。
// R3119/R3120：Node/Element 全部 getter/invoke 迁子模块，with_dom/with_dom_mut 用者随之移出，
// 仅 slot 映射助手（cache/cached/encode/decode/node_exists/element_template_local + 模板/DOM 源 setter）留此。
use gc::{
    cache_native_element, cached_native_element, decode_node_id, drop_cached_native_element, element_template_local,
    encode_node_id, node_exists, set_dom_source, set_element_template,
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

/// 从 HTML 文本解析 `Document` + 安装原生绑定（webview 接线封装 parse）。
///
/// 供 webview `run_page_scripts` 经 `Sandbox::install_native_bindings` escape-hatch 调用，
/// 避免 webview 直接依赖 `zero_dom`——Document 创建封装于 engine。read-only 快照
/// （re-parse 入参 html；不随页面 mutation 同步，写入切片后续）。
pub fn install_dom_bindings_from_html(scope: &mut v8::PinScope, ctx: v8::Local<v8::Context>, html: &str) {
    let dom = Rc::new(RefCell::new(zero_dom::parse_html(html)));
    install_dom_bindings(scope, ctx, dom);
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

    // 2. Element ObjectTemplate：internal slot[0] 存 NodeId + 只读属性 accessor + 方法。
    // 注意：accessor getter / FunctionTemplate 回调须为 ZST fn **项**（UnitType，size=0），
    // 不能 cast 成 fn 指针（size=8，触发 v8 UnitValue size_must_be_0），故逐成员注册。
    let tmpl = v8::ObjectTemplate::new(scope);
    tmpl.set_internal_field_count(1);
    // spec 只读属性 accessor（状态经 gc.rs 线程局部，镜像 HOST_CALLBACKS）。
    if let Some(k) = v8::String::new(scope, "nodeType") {
        tmpl.set_accessor(k.into(), node::native_node_type_getter);
    }
    if let Some(k) = v8::String::new(scope, "tagName") {
        tmpl.set_accessor(k.into(), element::native_tag_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "nodeName") {
        tmpl.set_accessor(k.into(), node::native_node_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "id") {
        tmpl.set_accessor_with_setter(k.into(), element::native_id_getter, element::native_id_setter);
    }
    if let Some(k) = v8::String::new(scope, "className") {
        tmpl.set_accessor_with_setter(
            k.into(),
            element::native_class_name_getter,
            element::native_class_name_setter,
        );
    }
    // `children` getter（spec `dom-parentnode-children`）：元素**子元素**（跳过文本/注释）
    // → V8 Array of native 对象（文档序）。
    if let Some(k) = v8::String::new(scope, "children") {
        tmpl.set_accessor(k.into(), element::native_children_getter);
    }
    // `textContent` getter/setter（spec `dom-node-textcontent`）：读=子树文本拼接；写=清子 + 文本节点。
    if let Some(k) = v8::String::new(scope, "textContent") {
        tmpl.set_accessor_with_setter(
            k.into(),
            node::native_text_content_getter,
            node::native_text_content_setter,
        );
    }
    // `childNodes` getter（spec `dom-node-childnodes`）：**全部子节点**（含文本/注释）→ V8 Array of
    // native 对象。区别于 `children`（仅元素）——解锁 R3103 textContent 写的文本节点可见性。
    // 文本/注释节点包同一模板：nodeType(3/8)/nodeName(#text/#comment)/textContent(=data) 经
    // 既有 node-type-aware getter 正确返回。
    if let Some(k) = v8::String::new(scope, "childNodes") {
        tmpl.set_accessor(k.into(), node::native_child_nodes_getter);
    }
    // `nodeValue` getter/setter（spec `dom-node-nodevalue`）：读=Text/Comment/PI=data，其余=null；
    // 写=Text/Comment/PI 改 content/data（`Document::set_node_value`），其余 no-op（spec）。
    if let Some(k) = v8::String::new(scope, "nodeValue") {
        tmpl.set_accessor_with_setter(k.into(), node::native_node_value_getter, node::native_node_value_setter);
    }
    // spec 方法（FunctionTemplate，args.this 读 NodeId）：getAttribute / hasAttribute /
    // setAttribute / removeAttribute。ObjectTemplate::set 须传 **Template**（非 Function 实例）——
    // FunctionTemplate 是 Template，实例化时各对象共享，args.this() 取回实例。
    let get_attr_tmpl = v8::FunctionTemplate::builder(element::native_get_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getAttribute") {
        tmpl.set(k.into(), get_attr_tmpl.into());
    }
    let has_attr_tmpl = v8::FunctionTemplate::builder(element::native_has_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "hasAttribute") {
        tmpl.set(k.into(), has_attr_tmpl.into());
    }
    let set_attr_tmpl = v8::FunctionTemplate::builder(element::native_set_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "setAttribute") {
        tmpl.set(k.into(), set_attr_tmpl.into());
    }
    let rm_attr_tmpl = v8::FunctionTemplate::builder(element::native_remove_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeAttribute") {
        tmpl.set(k.into(), rm_attr_tmpl.into());
    }
    // element 子树作用域查询（spec `dom-parentnode-queryselector(-all)`）：`args.this()` 取
    // 元素 NodeId 作 root，**仅后代**（排除元素自身，见 [`element::native_element_query_selector_invoke`]）。
    let eqs_tmpl = v8::FunctionTemplate::builder(element::native_element_query_selector_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "querySelector") {
        tmpl.set(k.into(), eqs_tmpl.into());
    }
    let eqsa_tmpl = v8::FunctionTemplate::builder(element::native_element_query_selector_all_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "querySelectorAll") {
        tmpl.set(k.into(), eqsa_tmpl.into());
    }
    // spec 树 mutation 方法（`args.this()` = parent NodeId，参为 native element 对象读 internal slot）：
    // appendChild / insertBefore / removeChild。Document 写经 with_dom_mut。
    let append_tmpl = v8::FunctionTemplate::builder(node::native_append_child_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "appendChild") {
        tmpl.set(k.into(), append_tmpl.into());
    }
    let insert_before_tmpl = v8::FunctionTemplate::builder(node::native_insert_before_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "insertBefore") {
        tmpl.set(k.into(), insert_before_tmpl.into());
    }
    let remove_child_tmpl = v8::FunctionTemplate::builder(node::native_remove_child_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeChild") {
        tmpl.set(k.into(), remove_child_tmpl.into());
    }
    // `replaceChild(newChild, oldChild)`（spec `dom-node-replace-child`）——补全树 mutation 集
    //（appendChild/insertBefore/removeChild/replaceChild）。成功返 oldChild（spec）。
    let replace_child_tmpl = v8::FunctionTemplate::builder(node::native_replace_child_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "replaceChild") {
        tmpl.set(k.into(), replace_child_tmpl.into());
    }
    // S4 EventTarget（spec `dom-eventtarget-add-event-listener` 等）：addEventListener /
    // removeEventListener / dispatchEvent 原生——监听器存线程局部（gc.rs LISTENERS，键=(NodeId
    // ffi, 事件类型)），dispatchEvent 在当前 scope 复活 Local 调用（不冒泡，最小切片）。
    let add_evt_tmpl = v8::FunctionTemplate::builder(event_target::native_add_event_listener_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "addEventListener") {
        tmpl.set(k.into(), add_evt_tmpl.into());
    }
    let rm_evt_tmpl = v8::FunctionTemplate::builder(event_target::native_remove_event_listener_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeEventListener") {
        tmpl.set(k.into(), rm_evt_tmpl.into());
    }
    let disp_evt_tmpl = v8::FunctionTemplate::builder(event_target::native_dispatch_event_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "dispatchEvent") {
        tmpl.set(k.into(), disp_evt_tmpl.into());
    }
    // R3110 节点导航 getter（spec `dom-node-parent-node` 等 / `dom-node-has-child-nodes`）：
    // parentNode / firstChild / lastChild / nextSibling / previousSibling → native 节点或 null；
    // hasChildNodes() → bool。读 Document 树关系（`with_dom`），结果包 native 节点对象。
    if let Some(k) = v8::String::new(scope, "parentNode") {
        tmpl.set_accessor(k.into(), node::native_parent_node_getter);
    }
    if let Some(k) = v8::String::new(scope, "firstChild") {
        tmpl.set_accessor(k.into(), node::native_first_child_getter);
    }
    if let Some(k) = v8::String::new(scope, "lastChild") {
        tmpl.set_accessor(k.into(), node::native_last_child_getter);
    }
    if let Some(k) = v8::String::new(scope, "nextSibling") {
        tmpl.set_accessor(k.into(), node::native_next_sibling_getter);
    }
    if let Some(k) = v8::String::new(scope, "previousSibling") {
        tmpl.set_accessor(k.into(), node::native_previous_sibling_getter);
    }
    let hcn_tmpl = v8::FunctionTemplate::builder(node::native_has_child_nodes_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "hasChildNodes") {
        tmpl.set(k.into(), hcn_tmpl.into());
    }
    // R3112 `attributes` getter（spec `dom-element-attributes`）：返元素属性集合 NamedNodeMap
    //（live，同元素返同对象——gc.rs NAMEDNODEMAP_OBJECTS 缓存；internal slot[0] = owner element NodeId）。
    if let Some(k) = v8::String::new(scope, "attributes") {
        tmpl.set_accessor(k.into(), namednodemap::native_attributes_getter);
    }
    // R3113 `innerHTML` / `outerHTML` getter + R3123 setter（spec `dom-element-innerhtml` / `-outerhtml`）：
    // 读=子节点 outer_html 拼接 / 本元素 outer_html；写=解析 HTML 片段清子替换（innerHTML）或整体
    // 替换自身（outerHTML）。setter 复用 `js_dom_bridge` fragment parse，经 with_dom_mut 写 live DOM。
    if let Some(k) = v8::String::new(scope, "innerHTML") {
        tmpl.set_accessor_with_setter(
            k.into(),
            element::native_inner_html_getter,
            element::native_inner_html_setter,
        );
    }
    if let Some(k) = v8::String::new(scope, "outerHTML") {
        tmpl.set_accessor_with_setter(
            k.into(),
            element::native_outer_html_getter,
            element::native_outer_html_setter,
        );
    }
    // R3114 `cloneNode(deep)`（spec `dom-node-clonenode`）：复用 `Document::clone_node`（克隆元素+属性，
    // deep 递归子树）；返新 native 元素（未挂载）。
    let clone_tmpl = v8::FunctionTemplate::builder(node::native_clone_node_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "cloneNode") {
        tmpl.set(k.into(), clone_tmpl.into());
    }
    // R3115 `contains(node)`（spec `dom-node-contains`）：node 是否为本元素或其后代
    //（含自身：`el.contains(el)===true`）；walk parent 链。
    let contains_tmpl = v8::FunctionTemplate::builder(node::native_contains_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "contains") {
        tmpl.set(k.into(), contains_tmpl.into());
    }
    set_element_template(scope, tmpl);

    // R3112 NamedNodeMap ObjectTemplate（element.attributes 集合）——R3116 拆到 namednodemap 子模块。
    namednodemap::build_and_cache_template(scope);

    // 3. 全局工厂 __zw_native_element_for_id(idStr) → native element 对象。
    let global = ctx.global(scope);
    let factory = v8::FunctionTemplate::builder(factories::native_element_factory_invoke).build(scope);
    let Some(f) = factory.get_function(scope) else {
        return;
    };
    let Some(key) = v8::String::new(scope, "__zw_native_element_for_id") else {
        return;
    };
    let _ = global.set(scope, key.into(), f.into());

    // 4. 全局工厂 __zw_native_query_selector(sel) / __zw_native_query_selector_all(sel)
    //    —— spec `dom-parentnode-queryselector(-all)`：文档根下按**全量选择器引擎**
    //    （[`zero_dom::Document::query_selector`] / `query_selector_all`，消费 tag/`*`/
    //    `#id`/`.class`/`[attr]`+6 运算符/伪类/后代·子组合器/逗号列表）匹配 → native 对象
    //    （单）/ V8 Array（复，文档序）。R3098 工厂仅 `get_element_by_id`，本切片把
    //    querySelector 族从 polyfill 字符串桥搬到 native（返 NodeId→对象，无 String 往返）。
    let qs = v8::FunctionTemplate::builder(factories::native_query_selector_invoke).build(scope);
    let qs_fn = qs.get_function(scope);
    let qs_key = v8::String::new(scope, "__zw_native_query_selector");
    if let (Some(f), Some(key)) = (qs_fn, qs_key) {
        let _ = global.set(scope, key.into(), f.into());
    }
    let qsa = v8::FunctionTemplate::builder(factories::native_query_selector_all_invoke).build(scope);
    let qsa_fn = qsa.get_function(scope);
    let qsa_key = v8::String::new(scope, "__zw_native_query_selector_all");
    if let (Some(f), Some(key)) = (qsa_fn, qsa_key) {
        let _ = global.set(scope, key.into(), f.into());
    }

    // 5. 全局工厂 __zw_native_create_element(tag) —— spec `dom-document-createelement`：
    //    `Document::create_element` 造新 Element NodeId → native 对象（未挂载，appendChild 落位）。
    //    解锁原生树构建（createElement + appendChild 全 native，无 polyfill String 桥）。
    let ce = v8::FunctionTemplate::builder(factories::native_create_element_invoke).build(scope);
    let ce_fn = ce.get_function(scope);
    let ce_key = v8::String::new(scope, "__zw_native_create_element");
    if let (Some(f), Some(key)) = (ce_fn, ce_key) {
        let _ = global.set(scope, key.into(), f.into());
    }

    // 6. R3127 全局 Event / CustomEvent 构造器——`new Event(type, opts)` 产标准 event 对象
    //    （instanceof Event 成立），stop/preventDefault 上原型（共享，非每次派发注入）。
    //    闭合 R3124 限制③ + R3126 限制③。详见 `event` 子模块。
    event::build_and_register(scope, global);
}

// ── 共享助手（slot 读写 / 字符串参 / 值转换）─────────────────────
// 供各子模块（element / node / factories / namednodemap / event_target）经 `super::` 引用。
// Element 特有 accessor / 方法 / 子树查询 / innerHTML·outerHTML 拆到 element 子模块（本轮 R3120）；
// Node 基类拆到 node 子模块（R3119）。

/// 取任意 `Local<Value>` 经 JS ToString → Rust String（缺省空串）。
/// spec reflected attribute setter 把值强转字符串后存。
fn local_value_to_string(scope: &mut v8::PinScope, value: v8::Local<v8::Value>) -> String {
    value
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default()
}

/// 取 FunctionCallbackArguments 第 `idx` 参为 Rust String（缺省空串）。
fn string_arg(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32) -> String {
    args.get(idx)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default()
}

/// 从任意 `Value` 取其 internal slot NodeId（若为 native element 对象）；否则 `None`
/// （非 native 对象误作参 → best-effort 忽略）。`args.get(idx)` 取参后经此读 NodeId。
///
/// null/undefined 先短路返 `None`：`to_object` 对 undefined/null 会**抛 JS 异常**
/// （"Cannot convert undefined or null to object"），即便 `?` 给 Rust 返 `None`，挂起异常
/// 仍令脚本失败。insertBefore 缺省 refChild（null → 末尾追加）等合法用法依赖此处静默返 `None`。
fn node_id_from_value(scope: &mut v8::PinScope, value: v8::Local<v8::Value>) -> Option<NodeId> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let obj = value.to_object(scope)?;
    read_node_id(scope, &obj)
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
