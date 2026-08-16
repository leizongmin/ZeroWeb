//! P1b/M6 S0q — QuickJS（rquickjs）原生 DOM 绑定骨架 PoC（js-dom goal DC-7）。
//!
//! 镜像 V8 `dom_bindings`（方案 C 混合 DOM-Node，RFC `docs/specs/p1b-v8-native-bindings-rfc.md`）：
//! 把 Rust DOM 的 `NodeId` 包装为 QuickJS 原生对象，`nodeType`/`tagName`/`nodeName`/`id` 经
//! rquickjs `Accessor` getter/setter 直接读写 Rust DOM，**不经 shim 字符串桥**。
//!
//! 与 V8 版的差异（rquickjs vs rusty_v8 API 形态）：
//! - **NodeId 承载**：V8 用 `ObjectTemplate` internal slot[0]；rquickjs `Object` 无 internal
//!   field，改用**非枚举、非可配置、非可写的隐藏 own property** `__zwNodeFfi`（f64 承载
//!   u64 slotmap `KeyData::as_ffi`——JS Number 对 < 2^53 的整数无损，slotmap ffi 的
//!   version/idx 各 u32 实际远小于 2^21，见单测 `ffi_f64_round_trip`）。不可枚举 →
//!   `JSON.stringify`/`Object.keys` 不可见；不可配置 → 页面脚本无法改写/删除。
//! - **DOM 源**：同 V8 `gc.rs` 模式——线程局部 `Rc<RefCell<Document>>`（QuickJS Runtime
//!   非线程安全，与执行线程绑定，getter 经同线程派发）。
//! - **对象身份**：`NodeId(ffi) → Persistent<Value>` strong 缓存（同 NodeId 返同对象，
//!   spec identity）。S0q PoC 用 strong 引用——weak/finalizer 生命周期验证是 S0q 后续切片
//!   （master.md M6 记录；V8 侧 R3133 的 Weak + guaranteed finalizer 对等物）。
//! - **具名 fn 而非闭包**：rquickjs 闭包类型推断对「`Ctx` 参数 + `'js` 值返回」组合有
//!   HRP（高阶生命周期）困难（闭包两生命周期无法统一且 `Value` 对 `'js` invariant）；
//!   getter/setter/工厂全部用具名 fn（签名 `'js` 显式统一），`this` 经 `This<Object>`
//!   参数接收（`FromParam`，JS 调用侧不占实参位）。
//!
//! 接线：`install_dom_bindings_quickjs(ctx, dom)` 经 `Sandbox::install_native_bindings_quickjs`
//! escape-hatch 进入持久 QuickJS Context 安装（webview `native_dom=true`，kill-switch 仍默认关）。
//!
//! spec：`nodeType` https://dom.spec.whatwg.org/#dom-node-nodetype（Element=1）；
//! `tagName` https://dom.spec.whatwg.org/#dom-element-tagname（HTML 大写）。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rquickjs::function::{Opt, This};
use rquickjs::{Ctx, FromJs, Function, Object, Persistent, Value};
use slotmap::{Key, KeyData};
use zero_dom::{Document, NodeId, NodeKind};

/// 原生对象隐藏 own property 名（承载 NodeId 的 u64 ffi 值，经 f64 Number）。
const NODE_FFI_PROP: &str = "__zwNodeFfi";

// ── 线程局部状态（镜像 V8 dom_bindings/gc.rs；QuickJS Runtime 单线程，getter 同线程派发）──

thread_local! {
    /// DOM 源：安装时注入，getter 经 [`with_dom`] 读真实 DOM（不经序列化 HTML 串）。
    static DOM_SOURCE: RefCell<Option<Rc<RefCell<Document>>>> = const { RefCell::new(None) };
    /// NodeId(ffi) → QuickJS 对象身份缓存（同 NodeId 返同对象，spec identity）。
    /// S0q PoC：strong Persistent（GC/weak/finalizer 生命周期验证是后续切片）。
    static NODE_OBJECTS: RefCell<HashMap<u64, Persistent<Value<'static>>>> =
        RefCell::new(HashMap::new());
    /// S4q EventTarget 监听器存储（镜像 V8 gc.rs LISTENERS 模式）：`(NodeId ffi, 事件类型)`
    /// → `Vec<(capture 标志, Persistent<Value>)>`——单列表保**全局注册序**（spec 派发按
    /// 注册序；capture/bubble 混合按 addEventListener 顺序）。监听回调存 strong
    /// Persistent（跨调用持久；S0q 同款 weak 化延后注记）。
    static LISTENERS: RefCell<HashMap<(u64, String), Vec<(bool, Persistent<Value<'static>>)>>> =
        RefCell::new(HashMap::new());
    /// S5q customElements registry（spec `dom-customelementregistry`）：tag（ASCII 小写键，
    /// spec define 规范化）→ registered ctor（Persistent strong）。`whenDefined` 等待集
    /// 经 `__zw_native_ce_when_defined` 全局 JS Set 承载（promise resolve 属异步域，PoC
    /// 以同步注册后立即解析语义简化——见 install 注册的 wrapper）。
    static CE_REGISTRY: RefCell<HashMap<String, Persistent<Value<'static>>>> =
        RefCell::new(HashMap::new());
    /// S5q 连接态追踪（镜像 V8 CONNECTED_CUSTOM）：已连入 document 的 custom 元素 ffi 集。
    static CONNECTED_CUSTOM: RefCell<std::collections::HashSet<u64>> = RefCell::new(std::collections::HashSet::new());
}

/// 在当前 DOM 源上执行只读操作；无 DOM 源时返 `None`。
fn with_dom<R>(f: impl FnOnce(&Document) -> R) -> Option<R> {
    let rc = DOM_SOURCE.with(|c| c.borrow().clone())?;
    let doc = rc.borrow();
    Some(f(&doc))
}

/// 在当前 DOM 源上执行可变操作；无 DOM 源时返 `None`。
fn with_dom_mut<R>(f: impl FnOnce(&mut Document) -> R) -> Option<R> {
    let rc = DOM_SOURCE.with(|c| c.borrow().clone())?;
    let mut doc = rc.borrow_mut();
    Some(f(&mut doc))
}

/// 清空全部绑定状态（reset_context / 导航重建 / WebView Drop 时调用；镜像 V8 reset_native_state）。
pub fn reset_quickjs_state() {
    DOM_SOURCE.with(|c| *c.borrow_mut() = None);
    NODE_OBJECTS.with(|c| c.borrow_mut().clear());
    LISTENERS.with(|c| c.borrow_mut().clear());
    CE_REGISTRY.with(|c| c.borrow_mut().clear());
    CONNECTED_CUSTOM.with(|c| c.borrow_mut().clear());
    ATTR_MAP_OBJECTS.with(|c| c.borrow_mut().clear());
}

// ── NodeId ↔ u64(ffi) ↔ f64（JS Number）编解码 ──────────────────────

fn encode_node_id(id: NodeId) -> u64 {
    id.data().as_ffi()
}

fn decode_node_id(ffi: u64) -> NodeId {
    NodeId::from(KeyData::from_ffi(ffi))
}

/// stale 校验：节点是否仍在 DOM arena（移除节点 → getter 返 undefined，spec detached）。
fn node_exists(id: NodeId) -> bool {
    with_dom(|d| d.get(id).is_some()).unwrap_or(false)
}

/// Rust 侧 nodeName 计算（spec `dom-node-nodename`；镜像 V8 dom_bindings/node.rs `node_name`）。
fn node_name(doc: &Document, id: NodeId) -> Option<String> {
    let n = doc.get(id)?;
    Some(match &n.kind {
        NodeKind::Element(e) => e.tag_name(),
        NodeKind::Text(_) => "#text".into(),
        NodeKind::Comment(_) => "#comment".into(),
        NodeKind::Document(_) => "#document".into(),
        NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => "#document-fragment".into(),
        NodeKind::ProcessingInstruction(p) => p.target.clone(),
        NodeKind::DocumentType(_) => "#document-type".into(),
    })
}

// ── 原生 getter（具名 fn + `This<Object>` 接收 this；见模块文档）──

/// 从 `this` 读回 NodeId（隐藏 own property `__zwNodeFfi`）；缺失/stale → None。
fn node_id_of(this: &Object) -> Option<NodeId> {
    let num: f64 = this.get(NODE_FFI_PROP).ok()?;
    let id = decode_node_id(num as u64);
    node_exists(id).then_some(id)
}

/// `nodeType` getter（spec `dom-node-nodetype`）。
fn node_type_getter<'js>(this: This<Object<'js>>) -> i32 {
    let Some(id) = node_id_of(&this.0) else {
        return 0;
    };
    with_dom(|d| d.node_type(id)).flatten().map(i32::from).unwrap_or(0)
}

/// `tagName` getter（spec `dom-element-tagname`：HTML-uppercased local name；非元素 → ""）。
fn tag_name_getter<'js>(this: This<Object<'js>>) -> String {
    element_string_of(&this.0, |e| e.tag_name()).unwrap_or_default()
}

/// `nodeName` getter（全节点类型，spec `dom-node-nodename`）。
fn node_name_getter<'js>(this: This<Object<'js>>) -> String {
    let Some(id) = node_id_of(&this.0) else {
        return String::new();
    };
    with_dom(|d| node_name(d, id)).flatten().unwrap_or_default()
}

/// `id` getter（spec `dom-element-id`，content 反射；缺省 ""）。
fn id_getter<'js>(this: This<Object<'js>>) -> String {
    element_string_of(&this.0, |e| e.id.clone().unwrap_or_default()).unwrap_or_default()
}

/// `id` setter（spec `dom-element-id`：ToString 后写 `id` 内容属性）。
fn id_setter<'js>(this: This<Object<'js>>, value: rquickjs::Coerced<String>) {
    let Some(id_node) = node_id_of(&this.0) else {
        return;
    };
    with_dom_mut(|d| d.set_attribute(id_node, "id", &value.0));
}

/// `className` getter（spec `dom-element-classname`，`class` content 反射；缺省 ""）。
/// S1q（镜像 V8 `native_class_name_getter` 的 `read_reflected_attr(scope,…,"class","")`）。
fn class_name_getter<'js>(this: This<Object<'js>>) -> String {
    reflected_attr_string_of(&this.0, "class").unwrap_or_default()
}

/// `className` setter（spec `dom-element-classname`：ToString 后写 `class` 内容属性）。
fn class_name_setter<'js>(this: This<Object<'js>>, value: rquickjs::Coerced<String>) {
    let Some(id_node) = node_id_of(&this.0) else {
        return;
    };
    with_dom_mut(|d| d.set_attribute(id_node, "class", &value.0));
}

/// `namespaceURI` getter（spec `dom-node-namespaceuri`）：元素命名空间 URI 字符串；
/// 空 namespace / 非元素 → JS null（镜像 V8 `native_namespace_uri_getter` 的
/// Some(None)→null 分支——undefined 仅留给无 DOM 源的 stale 场景，此处简化返 null）。
fn namespace_uri_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    let ns = element_ns_of(&this.0);
    match ns {
        Some(uri) => match rquickjs::String::from_str(ctx.clone(), &uri) {
            Ok(s) => s.into_value(),
            Err(_) => Value::new_null(ctx),
        },
        None => Value::new_null(ctx),
    }
}

/// `localName` getter（spec `dom-element-localname`）：ASCII 小写化的 local name
/// 只对 HTML 命名空间（`zero_dom::ElementData::local_name` 解析时已按 ns 处理大小写，
/// 此处直接透传——HTML 元素 parser 产出小写 local，SVG/MathML 保留原样）。
fn local_name_getter<'js>(this: This<Object<'js>>) -> String {
    element_string_of(&this.0, |e| e.local_name().to_string()).unwrap_or_default()
}

/// `textContent` getter（spec `dom-node-textcontent`）：子树文本拼接
/// （`Document::text_content` 递归收集后代 Text 节点）。空子树 → ""。
fn text_content_getter<'js>(this: This<Object<'js>>) -> String {
    let Some(id) = node_id_of(&this.0) else {
        return String::new();
    };
    with_dom(|d| d.text_content(id)).flatten().unwrap_or_default()
}

/// `textContent` setter（spec `dom-node-textcontent`，S2q 写入族；镜像 V8
/// `native_text_content_setter`）：值 ToString 后**清空全部子节点**，非空则追加单
/// Text 节点。**null → 空串**（spec `LegacyNullToEmptyString`——`el.textContent=null`
/// 清子，非写 "null" 文本）。Coerced<String> 的 JS ToString 对 null 产出 "null" 字面
/// 量（不可区分），故 setter 收 `Value` 手动判 null/undefined 再 coerce。
fn text_content_setter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, value: Value<'js>) {
    let Some(id) = node_id_of(&this.0) else {
        return;
    };
    // null/undefined → 空串（清子语义）；其余 → ToString（Coerced 转换，null 已被上面截获）。
    let val: String = if value.is_null() || value.is_undefined() {
        String::new()
    } else {
        match rquickjs::Coerced::<String>::from_js(&ctx, value) {
            Ok(v) => v.0,
            Err(_) => String::new(),
        }
    };
    with_dom_mut(|d| {
        // 移除全部子节点（先收集 NodeId 避免边遍历边改——同 V8 实现注记）。
        let children = d.child_nodes(id);
        for c in children {
            let _ = d.remove_child(id, c);
        }
        // 非空 → 追加单 Text 节点。
        if !val.is_empty() {
            let text_id = d.create_text_node(&val);
            let _ = d.append_child(id, text_id);
        }
    });
}

/// 读元素的反射内容属性（attr_name effective——HTML 小写/SVG 原样由 dom 层处理）；
/// 缺失/非元素/stale → None。
fn reflected_attr_string_of(this: &Object, name: &str) -> Option<String> {
    let id = node_id_of(this)?;
    with_dom(|d| d.get_attribute(id, name)).flatten()
}

// ── S1q 字符串反射族（title/lang/accessKey——V8 侧经 native_string_reflected_*
//    按 accessor name 泛化分发；QuickJS Accessor 无 name 回调，逐属性具名 fn 经共享
//    helper 实现，语义等价：getter 缺省 ""，setter ToString 写 content 属性。
//    IDL 名 → content 名映射：accessKey→accesskey，余 identity（V8 name_to_content_attr
//    的小写化规则在此按静态映射展开）。spec HTML `dom-l10n`/`dom-lang`/`dom-accesskey`。──

/// `title` getter（spec HTML `dom-title`，`title` 反射；缺省 ""）。
fn title_getter<'js>(this: This<Object<'js>>) -> String {
    reflected_attr_string_of(&this.0, "title").unwrap_or_default()
}

/// `title` setter（spec `dom-title`；`[LegacyNullToEmptyString]`——null→""，Coerced 已覆盖）。
fn title_setter<'js>(this: This<Object<'js>>, value: rquickjs::Coerced<String>) {
    set_reflected_attr(&this.0, "title", &value.0);
}

/// `lang` getter（spec HTML `dom-lang`，`lang` 反射；缺省 ""）。
fn lang_getter<'js>(this: This<Object<'js>>) -> String {
    reflected_attr_string_of(&this.0, "lang").unwrap_or_default()
}

/// `lang` setter（spec `dom-lang`；`[LegacyNullToEmptyString]`）。
fn lang_setter<'js>(this: This<Object<'js>>, value: rquickjs::Coerced<String>) {
    set_reflected_attr(&this.0, "lang", &value.0);
}

/// `accessKey` getter（spec HTML `dom-accesskey`，`accesskey` 反射；缺省 ""）。
fn access_key_getter<'js>(this: This<Object<'js>>) -> String {
    reflected_attr_string_of(&this.0, "accesskey").unwrap_or_default()
}

/// `accessKey` setter（IDL accessKey → content `accesskey`；`[LegacyNullToEmptyString]`）。
fn access_key_setter<'js>(this: This<Object<'js>>, value: rquickjs::Coerced<String>) {
    set_reflected_attr(&this.0, "accesskey", &value.0);
}

/// 写元素反射内容属性（stale/非元素 no-op）。
fn set_reflected_attr(this: &Object, name: &str, value: &str) {
    let Some(id) = node_id_of(this) else {
        return;
    };
    with_dom_mut(|d| d.set_attribute(id, name, value));
}

/// 读元素命名空间 URI（空 ns / 非元素 → None）。
fn element_ns_of(this: &Object) -> Option<String> {
    let id = node_id_of(this)?;
    with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => {
                let ns = e.namespace();
                (!ns.is_empty()).then(|| ns.to_string())
            }
            _ => None,
        })
    })
    .flatten()
}

/// 在元素的 ElementData 上计算字符串（stale/非元素 → None）。
fn element_string_of(this: &Object, f: impl FnOnce(&zero_dom::ElementData) -> String) -> Option<String> {
    let id = node_id_of(this)?;
    with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(f(e)),
            _ => None,
        })
    })
    .flatten()
}

// ── 安装入口 ─────────────────────────────────────────────────────────

/// 安装 QuickJS 原生 DOM 绑定到指定 Context（镜像 V8 `dom_bindings::install_dom_bindings`）。
///
/// - 注入 DOM 源（线程局部，供 getter 读）。
/// - 注册全局工厂 `__zw_native_element_for_id(idStr)`：`get_element_by_id` 解析 →
///   NodeId → 创建/查找 native element 对象（NodeId↔对象身份映射）。
/// - 元素对象带 `nodeType`/`tagName`/`nodeName`/`id`(+setter) 原生 getter
///   （rquickjs `Accessor`）+ 隐藏 `__zwNodeFfi`。
///
/// 幂等：重复调用重置 DOM 源 + 身份缓存换代全量清（导航/重载场景，R55 同款语义）。
pub fn install_dom_bindings_quickjs<'js>(ctx: &Ctx<'js>, dom: Rc<RefCell<Document>>) {
    // 1. DOM 源注入 + 身份缓存换代（幂等）。
    DOM_SOURCE.with(|c| *c.borrow_mut() = Some(dom));
    NODE_OBJECTS.with(|c| c.borrow_mut().clear());

    // 2. 全局工厂 `__zw_native_element_for_id(idStr)`（与 V8 版同名同 wire 形态，
    //    A/B 对照门/测试可双引擎复用同一调用脚本）。具名 fn 形态（见模块文档 HRP 注记）；
    //    参数 Opt<Coerced<String>>（可选 + JS ToString 语义）；miss（元素不存在/缺参）→
    //    JS null（与 V8 版 wire 语义一致，区别 Option 直返 None → undefined）。
    if let Ok(f) = Function::new(ctx.clone(), native_element_for_id_entry) {
        let _ = ctx.globals().set("__zw_native_element_for_id", f);
    }

    // 3. 全局工厂 `__zw_native_create_element(tag)`（与 V8 版同名同 wire——A/B 对照门
    //    双引擎复用）。detached 元素入 arena（spec：createElement 产物无父、不在文档，
    //    但 ownerDocument 可查——arena 即承载），返回带全套 getter/方法的 native 对象。
    if let Ok(f) = Function::new(ctx.clone(), native_create_element_entry) {
        let _ = ctx.globals().set("__zw_native_create_element", f);
    }

    // 4. S3q 全局查询工厂 `__zw_native_query_selector(sel)` / `__zw_native_query_selector_all(sel)`
    //    （与 V8 版同名同 wire）。文档根下全量选择器引擎（zero_dom query_selector 族，
    //    消费 tag/`*`/`#id`/.class/[attr]+运算符/伪类/组合器）。
    if let Ok(f) = Function::new(ctx.clone(), native_query_selector_entry) {
        let _ = ctx.globals().set("__zw_native_query_selector", f);
    }
    if let Ok(f) = Function::new(ctx.clone(), native_query_selector_all_entry) {
        let _ = ctx.globals().set("__zw_native_query_selector_all", f);
    }

    // 5. S5q `customElements` 全局对象（五件套；spec `dom-customelementregistry`）。
    //    命名对齐 V8 侧 wire 语义（Rust registry 权威 + JS 薄方法面）。
    if let Ok(ce) = rquickjs::Object::new(ctx.clone()) {
        if let Ok(f) = Function::new(ctx.clone(), ce_define) {
            let _ = ce.set("define", f);
        }
        if let Ok(f) = Function::new(ctx.clone(), ce_get) {
            let _ = ce.set("get", f);
        }
        if let Ok(f) = Function::new(ctx.clone(), ce_get_name) {
            let _ = ce.set("getName", f);
        }
        if let Ok(f) = Function::new(ctx.clone(), ce_when_defined) {
            let _ = ce.set("whenDefined", f);
        }
        if let Ok(f) = Function::new(ctx.clone(), ce_upgrade) {
            let _ = ce.set("upgrade", f);
        }
        let _ = ctx.globals().set("__zw_native_customElements", ce);
    }
}

/// 从 HTML 文本解析 `Document` + 安装 QuickJS 原生绑定（webview 接线封装 parse，
/// 镜像 V8 `install_dom_bindings_from_html`——避免 webview 直接依赖 `zero_dom`）。
/// read-only 快照（re-parse 入参 html；不随页面 mutation 同步，live Document 接线优先）。
pub fn install_dom_bindings_quickjs_from_html(ctx: &Ctx, html: &str) {
    let dom = Rc::new(RefCell::new(zero_dom::parse_html(html)));
    install_dom_bindings_quickjs(ctx, dom);
}

/// 工厂入口（named fn 形态，绕开闭包 HRP；见模块文档）。
fn native_element_for_id_entry<'js>(ctx: Ctx<'js>, id_str: Opt<rquickjs::Coerced<String>>) -> Value<'js> {
    match id_str.0 {
        Some(s) => native_element_for_id_impl(&ctx, &s.0),
        None => Value::new_null(ctx),
    }
}

/// `__zw_native_create_element(tag)` 工厂入口（S2q mutation 族配套——detached 元素
/// 经 appendChild 入树的 child 来源）。
fn native_create_element_entry<'js>(
    ctx: Ctx<'js>,
    tag: Opt<rquickjs::Coerced<String>>,
) -> rquickjs::Result<Value<'js>> {
    let Some(tag) = tag.0 else {
        return Ok(Value::new_null(ctx));
    };
    // S4q 完整化（R66）：非法 tag → InvalidCharacterError（spec `dom-document-createelement`
    // validate；镜像 V8 is_valid_qualified_name 路径——空/首字符非 name-start/含 `<`/`>`/空白）。
    if !is_valid_tag_name(&tag.0) {
        return Err(throw_dom_exception(
            &ctx,
            "InvalidCharacterError",
            "The tag name provided is not a valid name.",
        ));
    }
    let Some(node_id) = with_dom_mut(|d| d.create_element(&tag.0)) else {
        return Ok(Value::new_null(ctx));
    };
    let v = get_or_build_node_value(&ctx, node_id);
    // S5q upgrade（PoC 路径）：命中 registry → 原型挂 ctor.prototype（见模块注释）。
    if let Some(obj) = v.as_object().cloned() {
        apply_ce_prototype(&obj, &ctx, &tag.0);
    }
    Ok(v)
}

/// NodeId → native 对象（身份缓存命中或新建，带全套属性/方法面）。`get_or_create`
/// 的 QuickJS 版共享入口——`element_for_id` 工厂与 `create_element` 工厂统一走此。
fn get_or_build_node_value<'js>(ctx: &Ctx<'js>, node_id: NodeId) -> Value<'js> {
    let ffi = encode_node_id(node_id);
    if let Some(hit) = NODE_OBJECTS.with(|c| c.borrow().get(&ffi).cloned())
        && let Ok(v) = hit.restore(ctx)
    {
        return v;
    }
    match build_element_object(ctx, ffi) {
        Ok(obj) => {
            NODE_OBJECTS.with(|c| {
                let v: Value = obj.clone().into_value();
                c.borrow_mut().insert(ffi, Persistent::save(ctx, v));
            });
            obj.into_value()
        }
        Err(_) => Value::new_null(ctx.clone()),
    }
}

/// 工厂实现：`get_element_by_id` → NodeId → native 对象（身份缓存命中或新建）。
fn native_element_for_id_impl<'js>(ctx: &Ctx<'js>, id_str: &str) -> Value<'js> {
    let Some(node_id) = with_dom(|d| d.get_element_by_id(id_str)).flatten() else {
        return Value::new_null(ctx.clone());
    };
    let ffi = encode_node_id(node_id);
    // 身份缓存命中（同 NodeId 返同对象，spec identity）。Persistent::restore 消耗 self → clone。
    if let Some(hit) = NODE_OBJECTS.with(|c| c.borrow().get(&ffi).cloned())
        && let Ok(v) = hit.restore(ctx)
    {
        return v;
    }
    let obj = match build_element_object(ctx, ffi) {
        Ok(o) => o,
        Err(_) => return Value::new_null(ctx.clone()),
    };
    NODE_OBJECTS.with(|c| {
        let v: Value = obj.clone().into_value();
        c.borrow_mut().insert(ffi, Persistent::save(ctx, v));
    });
    obj.into_value()
}

/// 构建元素原生对象：隐藏 `__zwNodeFfi` + 只读 getter Accessor + `id` setter。
fn build_element_object<'js>(ctx: &Ctx<'js>, ffi: u64) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    // 隐藏 own property 承载 NodeId。Property builder flag 无参形态：默认 writable/enumerable
    // 关，.configurable() 打开 → 三者组合「可配置、不可写、不可枚举」——非可写防改值，
    // 不可枚举防 Object.keys/JSON.stringify 泄露；可配置允许同类 redefine（PoC 宽松面）。
    use rquickjs::object::Property;
    obj.prop(NODE_FFI_PROP, Property::from(ffi as f64).configurable())?;
    use rquickjs::object::Accessor;
    obj.prop("nodeType", Accessor::from(node_type_getter).configurable())?;
    obj.prop("tagName", Accessor::from(tag_name_getter).configurable())?;
    obj.prop("nodeName", Accessor::from(node_name_getter).configurable())?;
    obj.prop(
        "id",
        Accessor::from(id_getter).set(id_setter).configurable().enumerable(),
    )?;
    // S1q 只读属性族（镜像 V8 dom_bindings 既有面）。
    obj.prop(
        "className",
        Accessor::from(class_name_getter)
            .set(class_name_setter)
            .configurable()
            .enumerable(),
    )?;
    obj.prop("namespaceURI", Accessor::from(namespace_uri_getter).configurable())?;
    // S1q 复合对象（R69）：attributes NamedNodeMap 面（缓存保身份）。
    obj.prop("attributes", Accessor::from(attributes_getter).configurable())?;
    obj.prop("localName", Accessor::from(local_name_getter).configurable())?;
    obj.prop(
        "textContent",
        Accessor::from(text_content_getter)
            .set(text_content_setter)
            .configurable(),
    )?;
    // S1q 字符串反射族（title/lang/accessKey）。
    obj.prop(
        "title",
        Accessor::from(title_getter)
            .set(title_setter)
            .configurable()
            .enumerable(),
    )?;
    obj.prop(
        "lang",
        Accessor::from(lang_getter).set(lang_setter).configurable().enumerable(),
    )?;
    obj.prop(
        "accessKey",
        Accessor::from(access_key_getter)
            .set(access_key_setter)
            .configurable()
            .enumerable(),
    )?;
    // S2q 属性方法族（spec `dom-element-getattribute` 家族；镜像 V8 factories 面）。
    // Function prop 非 enumerable（与 V8 ObjectTemplate 方法一致——Object.keys 不受扰）。
    obj.prop("getAttribute", Function::new(ctx.clone(), get_attribute_method)?)?;
    obj.prop("setAttribute", Function::new(ctx.clone(), set_attribute_method)?)?;
    obj.prop("removeAttribute", Function::new(ctx.clone(), remove_attribute_method)?)?;
    obj.prop("hasAttribute", Function::new(ctx.clone(), has_attribute_method)?)?;
    // S2q 子树 mutation 族（非 enumerable）。
    obj.prop("appendChild", Function::new(ctx.clone(), append_child_method)?)?;
    obj.prop("removeChild", Function::new(ctx.clone(), remove_child_method)?)?;
    // S2q 续：树读回 getter（非 enumerable）。
    obj.prop("childNodes", Accessor::from(child_nodes_getter).configurable())?;
    obj.prop("parentNode", Accessor::from(parent_node_getter).configurable())?;
    obj.prop("firstChild", Accessor::from(first_child_getter).configurable())?;
    obj.prop("lastChild", Accessor::from(last_child_getter).configurable())?;
    // S3q 查询族（元素级，非 enumerable）。
    obj.prop(
        "querySelector",
        Function::new(ctx.clone(), element_query_selector_method)?,
    )?;
    obj.prop(
        "querySelectorAll",
        Function::new(ctx.clone(), element_query_selector_all_method)?,
    )?;
    // S4q EventTarget（非 enumerable）。
    obj.prop(
        "addEventListener",
        Function::new(ctx.clone(), add_event_listener_method)?,
    )?;
    obj.prop(
        "removeEventListener",
        Function::new(ctx.clone(), remove_event_listener_method)?,
    )?;
    obj.prop("dispatchEvent", Function::new(ctx.clone(), dispatch_event_method)?)?;
    Ok(obj)
}

// ── S2q 属性方法族（具名 fn + This<Object>；镜像 V8 dom_bindings 方法面）──

/// `getAttribute(name)`（spec `dom-element-getattribute`）：missing → JS null
///（区别空串值；V8 版同语义）。name 经 Coerced ToString；HTML 小写化由 dom 层
/// `attr_name_effective` 处理。
fn get_attribute_method<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, name: rquickjs::Coerced<String>) -> Value<'js> {
    match reflected_attr_string_of(&this.0, &name.0) {
        Some(v) => match rquickjs::String::from_str(ctx.clone(), &v) {
            Ok(s) => s.into_value(),
            Err(_) => Value::new_null(ctx),
        },
        None => Value::new_null(ctx),
    }
}

// ── S1q 复合对象（R69）：`attributes` NamedNodeMap 面 ─────────────────
//
// 镜像 V8 namednodemap.rs（缓存保身份 `el.attributes === el.attributes`）。QuickJS 侧
// 形态：**每次读快照重建 plain object 的 Array-lite**，身份经二级缓存 ATTR_MAP_OBJECTS
//（owner ffi → Persistent）保同对象。条目为 `{name, value}` plain object（Attr 节点
// instanceof 面延后——V8 侧 _zwMakeAttr 同域问题）。方法面：length getter / item(i) /
// getNamedItem(name) / setNamedItem({name,value} 或 Attr 形) / removeNamedItem(name)。

thread_local! {
    /// owner element ffi → attributes 包装对象身份缓存（spec identity：
    /// `el.attributes === el.attributes`；同 NODE_OBJECTS 模式的二级缓存）。
    static ATTR_MAP_OBJECTS: RefCell<HashMap<u64, Persistent<Value<'static>>>> =
        RefCell::new(HashMap::new());
}

/// `attributes` getter（spec `dom-element-attributes`）：返 NamedNodeMap 形包装
///（缓存保身份）。动态 length——经 getter 读 live DOM（快照对象但 length/方法即时读）。
fn attributes_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    let Some(id) = node_id_of(&this.0) else {
        return Value::new_null(ctx);
    };
    let ffi = encode_node_id(id);
    if let Some(hit) = ATTR_MAP_OBJECTS.with(|c| c.borrow().get(&ffi).cloned())
        && let Ok(v) = hit.restore(&ctx)
    {
        return v;
    }
    let Ok(obj) = build_attributes_map_object(&ctx, ffi) else {
        return Value::new_null(ctx);
    };
    let v: Value = obj.into_value();
    ATTR_MAP_OBJECTS.with(|c| {
        c.borrow_mut().insert(ffi, Persistent::save(&ctx, v.clone()));
    });
    v
}

/// 构建 attributes 包装对象（owner ffi 隐藏属性 + length getter + 方法面）。
fn build_attributes_map_object<'js>(ctx: &Ctx<'js>, owner_ffi: u64) -> rquickjs::Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    use rquickjs::object::Property;
    obj.prop(NODE_FFI_PROP, Property::from(owner_ffi as f64).configurable())?;
    use rquickjs::object::Accessor;
    obj.prop("length", Accessor::from(nnm_length_getter).configurable())?;
    obj.prop("item", Function::new(ctx.clone(), nnm_item_method)?)?;
    obj.prop("getNamedItem", Function::new(ctx.clone(), nnm_get_named_item_method)?)?;
    obj.prop("setNamedItem", Function::new(ctx.clone(), nnm_set_named_item_method)?)?;
    obj.prop(
        "removeNamedItem",
        Function::new(ctx.clone(), nnm_remove_named_item_method)?,
    )?;
    Ok(obj)
}

/// 从 attributes 包装对象读 owner NodeId。
fn nnm_owner_of(this: &Object) -> Option<NodeId> {
    let num: f64 = this.get(NODE_FFI_PROP).ok()?;
    let id = decode_node_id(num as u64);
    node_exists(id).then_some(id)
}

/// `attributes.length` getter（spec `dom-namednodemap-length`）：owner 元素属性数。
fn nnm_length_getter<'js>(this: This<Object<'js>>) -> i32 {
    let Some(id) = nnm_owner_of(&this.0) else {
        return 0;
    };
    with_dom(|d| d.attribute_names(id)).map(|v| v.len() as i32).unwrap_or(0)
}

/// 属性条目 → `{name, value}` plain object（Attr 节点形态延后）。
fn nnm_attr_entry<'js>(ctx: &Ctx<'js>, id: NodeId, name: &str) -> Option<Object<'js>> {
    let value = with_dom(|d| d.get_attribute(id, name)).flatten()?;
    let obj = Object::new(ctx.clone()).ok()?;
    let _ = obj.set("name", name.to_string());
    let _ = obj.set("value", value);
    Some(obj)
}

/// `attributes.item(i)`（spec `dom-namednodemap-item`）：越界/非数字 → null。
fn nnm_item_method<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, index: rquickjs::Coerced<f64>) -> Value<'js> {
    let Some(id) = nnm_owner_of(&this.0) else {
        return Value::new_null(ctx);
    };
    let names = with_dom(|d| d.attribute_names(id)).unwrap_or_default();
    let i = index.0 as usize;
    match names.get(i) {
        Some(name) => match nnm_attr_entry(&ctx, id, name) {
            Some(o) => o.into_value(),
            None => Value::new_null(ctx),
        },
        None => Value::new_null(ctx),
    }
}

/// `attributes.getNamedItem(name)`（spec `dom-namednodemap-getnameditem`）：miss → null。
fn nnm_get_named_item_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    name: rquickjs::Coerced<String>,
) -> Value<'js> {
    let Some(id) = nnm_owner_of(&this.0) else {
        return Value::new_null(ctx);
    };
    match nnm_attr_entry(&ctx, id, &name.0) {
        Some(o) => o.into_value(),
        None => Value::new_null(ctx),
    }
}

/// `attributes.setNamedItem(attr)`（spec `dom-namednodemap-setnameditem`）：从入参对象
/// 读 name/value 写 owner（兼容 plain 对象与 Attr 形态——镜像 V8 read_str_prop）。
fn nnm_set_named_item_method<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, attr: Opt<Value<'js>>) {
    let Some(id) = nnm_owner_of(&this.0) else {
        return;
    };
    let Some(obj) = attr.0.and_then(|v| v.into_object()) else {
        return;
    };
    let Ok(name) = obj.get::<_, String>("name") else {
        return;
    };
    let value = obj.get::<_, String>("value").unwrap_or_default();
    let old: Option<String> = with_dom(|d| d.get_attribute(id, &name)).flatten();
    with_dom_mut(|d| d.set_attribute(id, &name, &value));
    dispatch_attribute_changed(&ctx, id, &name, old.as_deref(), Some(&value));
}

/// `attributes.removeNamedItem(name)`（spec `dom-namednodemap-removenameditem`）：
/// 移除并返被移除条目 `{name, value}`；缺失 → null（spec 抛 NotFoundError——错误路径
/// 基建已有，此处 PoC 宽松 null；对齐待 DOMException 构造器域）。
fn nnm_remove_named_item_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    name: rquickjs::Coerced<String>,
) -> Value<'js> {
    let Some(id) = nnm_owner_of(&this.0) else {
        return Value::new_null(ctx);
    };
    // 先捕获 old value（构造返回条目 + 派发共用），再移除。
    let Some(old_value) = with_dom(|d| d.get_attribute(id, &name.0)).flatten() else {
        return Value::new_null(ctx);
    };
    with_dom_mut(|d| d.remove_attribute(id, &name.0));
    dispatch_attribute_changed(&ctx, id, &name.0, Some(old_value.as_str()), None);
    let entry = Object::new(ctx.clone()).ok();
    match entry {
        Some(o) => {
            let _ = o.set("name", name.0.clone());
            let _ = o.set("value", old_value);
            o.into_value()
        }
        None => Value::new_null(ctx),
    }
}

/// `setAttribute(name, value)`（spec `dom-element-setattribute`）。
fn set_attribute_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    name: rquickjs::Coerced<String>,
    value: rquickjs::Coerced<String>,
) {
    // R68：old value 写前捕获（spec attributeChangedCallback(name, old, new)——old 为
    // 变更前值，缺失 → None → JS null）。
    let old: Option<String> = node_id_of(&this.0).and_then(|id| with_dom(|d| d.get_attribute(id, &name.0)).flatten());
    set_reflected_attr(&this.0, &name.0, &value.0);
    // S5q 深化（R65）+ R68 完整化：attributeChangedCallback——custom 元素（registry
    // 命中）经 setAttribute 变更时派发；observedAttributes 过滤 + oldValue 在
    // dispatch_attribute_changed 内做。
    if let Some(id) = node_id_of(&this.0) {
        dispatch_attribute_changed(&ctx, id, &name.0, old.as_deref(), Some(&value.0));
    }
}

/// `removeAttribute(name)`（spec `dom-element-removeattribute`）：真移除
///（区别 set 空串——布尔属性 unset 语义；镜像 V8 RemoveAttr OnHandle 修正）。
fn remove_attribute_method<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, name: rquickjs::Coerced<String>) {
    let Some(id) = node_id_of(&this.0) else {
        return;
    };
    // R68：removeAttribute 也派发 attributeChangedCallback（spec 属性变更含移除，
    // newValue = null）。缺失属性 no-op 不派发（spec 仅已存在属性的移除才是变更）。
    let old: Option<String> = with_dom(|d| d.get_attribute(id, &name.0)).flatten();
    with_dom_mut(|d| d.remove_attribute(id, &name.0));
    if let Some(old) = old {
        dispatch_attribute_changed(&ctx, id, &name.0, Some(old.as_str()), None);
    }
}

/// `hasAttribute(name)`（spec `dom-element-hasattribute`）。
fn has_attribute_method<'js>(this: This<Object<'js>>, name: rquickjs::Coerced<String>) -> bool {
    let Some(id) = node_id_of(&this.0) else {
        return false;
    };
    with_dom(|d| d.has_attribute(id, &name.0)).unwrap_or(false)
}

// ── S4q 完整化（R66）：DOMException 基建 ──────────────────────────────
//
// QuickJS 侧形态：抛**带 name 属性的 Error 对象**（rquickjs `Ctx::throw(value)`——
// JS 侧 catch 得到 e.name/e.message，与 DOMException 的可观测面等价；DOMException
// 全局构造器基建（instanceof 面）延后——V8 侧 R6 的 identity 三重根因教训在案）。
// name 映射镜像 V8 dom_bindings/node.rs `dom_error_exception`。

/// DomError → (DOMException name, message)（镜像 V8 `dom_error_exception`）。
fn dom_error_name(err: &zero_dom::DomError) -> (&'static str, String) {
    match err {
        zero_dom::DomError::WouldCreateCycle | zero_dom::DomError::CannotInsertDocumentRoot => (
            "HierarchyRequestError",
            "The new child is an ancestor of the parent.".into(),
        ),
        zero_dom::DomError::NotAChild { .. } => (
            "NotFoundError",
            "The child to be replaced is not a child of this node.".into(),
        ),
        zero_dom::DomError::NodeNotFound(_) => ("HierarchyRequestError", "The node does not exist.".into()),
        zero_dom::DomError::NotAnElement | zero_dom::DomError::AlreadyHasShadowRoot => {
            ("InvalidStateError", err.to_string())
        }
    }
}

/// 抛 DOMException 形态错误（name + message 属性的 Error 对象经 `Ctx::throw`）。
/// 调用方须在 native fn 内 return 该返回值（Err 状态已装在 ctx，函数返什么都会被
/// 异常取代——返 null 占位）。
fn throw_dom_exception(ctx: &Ctx, name: &str, message: &str) -> rquickjs::Error {
    let err = match rquickjs::Object::new(ctx.clone()) {
        Ok(o) => {
            let _ = o.set("name", name.to_string());
            let _ = o.set("message", message.to_string());
            let _ = o.set("stack", format!("DOMException: {message}"));
            o.into_value()
        }
        Err(e) => return e,
    };
    ctx.throw(err)
}

// ── S2q 子树 mutation 族（appendChild/removeChild；insertBy 等后续切片）──

// ── S3q 查询族（spec `dom-parentnode-queryselector` 家族；镜像 V8 factories.rs）──

/// `__zw_native_query_selector(sel)` 入口：文档根下首个匹配 → native 对象；
/// 无匹配/空/非法选择器 → null（parse 失败返 None 无 panic，V8 版同语义）。
fn native_query_selector_entry<'js>(ctx: Ctx<'js>, sel: Opt<rquickjs::Coerced<String>>) -> Value<'js> {
    let Some(sel) = sel.0 else {
        return Value::new_null(ctx);
    };
    let Some(id) = with_dom(|d| d.query_selector(d.root(), sel.0.trim())).flatten() else {
        return Value::new_null(ctx);
    };
    get_or_build_node_value(&ctx, id)
}

/// `__zw_native_query_selector_all(sel)` 入口：全部匹配 → Array of native 对象
///（文档序）；空/非法 → 空 Array。
fn native_query_selector_all_entry<'js>(
    ctx: Ctx<'js>,
    sel: Opt<rquickjs::Coerced<String>>,
) -> rquickjs::Result<rquickjs::Array<'js>> {
    let ids: Vec<NodeId> = match sel.0 {
        Some(s) => with_dom(|d| d.query_selector_all(d.root(), s.0.trim())).unwrap_or_default(),
        None => Vec::new(),
    };
    let arr = rquickjs::Array::new(ctx.clone())?;
    for (i, id) in ids.into_iter().enumerate() {
        let v = get_or_build_node_value(&ctx, id);
        arr.set(i, v)?;
    }
    Ok(arr)
}

/// 元素级 `querySelector(sel)` 方法（spec `dom-parentnode-queryselector`，元素子树作用域）。
fn element_query_selector_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    sel: rquickjs::Coerced<String>,
) -> Value<'js> {
    let Some(scope_id) = node_id_of(&this.0) else {
        return Value::new_null(ctx);
    };
    let Some(id) = with_dom(|d| d.query_selector(scope_id, sel.0.trim())).flatten() else {
        return Value::new_null(ctx);
    };
    get_or_build_node_value(&ctx, id)
}

/// 元素级 `querySelectorAll(sel)` 方法（元素子树作用域，全部匹配 Array）。
fn element_query_selector_all_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    sel: rquickjs::Coerced<String>,
) -> rquickjs::Result<rquickjs::Array<'js>> {
    let Some(scope_id) = node_id_of(&this.0) else {
        return rquickjs::Array::new(ctx);
    };
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(scope_id, sel.0.trim())).unwrap_or_default();
    let arr = rquickjs::Array::new(ctx.clone())?;
    for (i, id) in ids.into_iter().enumerate() {
        let v = get_or_build_node_value(&ctx, id);
        arr.set(i, v)?;
    }
    Ok(arr)
}
// ── S2q 续：树读回 getter（childNodes/parentNode/firstChild/lastChild）──

/// `childNodes` getter（spec `dom-node-childnodes`）：全子节点（含 Text/Comment）
/// native 对象数组。**Array 返回形态**（非 NodeList——collection 语义的 live 性/
/// indexed props 属 S1q 复合对象域；PoC 快照数组，与 V8 版 tests 断言面一致）。
fn child_nodes_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    let Some(id) = node_id_of(&this.0) else {
        return Value::new_null(ctx);
    };
    let children = with_dom(|d| d.child_nodes(id)).unwrap_or_default();
    match rquickjs::Array::new(ctx.clone()) {
        Ok(arr) => {
            for (i, c) in children.iter().enumerate() {
                let v = get_or_build_node_value(&ctx, *c);
                let _ = arr.set(i, v);
            }
            arr.into_value()
        }
        Err(_) => Value::new_null(ctx),
    }
}

/// `parentNode` getter（spec `dom-node-parentnode`）：无父（detached/根）→ null。
fn parent_node_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    let Some(id) = node_id_of(&this.0) else {
        return Value::new_null(ctx);
    };
    match with_dom(|d| d.parent_node(id)).flatten() {
        Some(p) => get_or_build_node_value(&ctx, p),
        None => Value::new_null(ctx),
    }
}

/// `firstChild` / `lastChild` getter 共用（spec `dom-node-firstchild`/`lastchild`）。
fn first_last_child_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, last: bool) -> Value<'js> {
    let Some(id) = node_id_of(&this.0) else {
        return Value::new_null(ctx);
    };
    let child = with_dom(|d| if last { d.last_child(id) } else { d.first_child(id) }).flatten();
    match child {
        Some(c) => get_or_build_node_value(&ctx, c),
        None => Value::new_null(ctx),
    }
}

/// `firstChild` getter。
fn first_child_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    first_last_child_getter(this, ctx, false)
}

/// `lastChild` getter。
fn last_child_getter<'js>(this: This<Object<'js>>, ctx: Ctx<'js>) -> Value<'js> {
    first_last_child_getter(this, ctx, true)
}

/// 从 Value 读 native 对象的 NodeId（对象须为 `__zw_native_*` 工厂产物——隐藏
/// `__zwNodeFfi` prop 标记）；非本族对象/缺失 → None。
fn node_id_from_value(v: &Value) -> Option<NodeId> {
    let obj: &Object = v.as_object()?;
    node_id_of(obj)
}

/// `appendChild(child)`（spec `dom-node-appendchild`）：child 须为 native 对象
///（`__zw_native_create_element` 产物）；DomError → JS TypeError（QuickJS 无
/// DOMException 构造器基建，PoC 以 TypeError 承载错误路径——V8 侧 DomError→
/// DOMException 映射见 dom_bindings/node.rs，QuickJS 版随 S4q 异常基建对齐）。
/// spec 移动语义：child 已有父时自动 reparent（zero_dom append_child 内建）。
/// 返回 child（spec appendChild 返回追加的节点）。
fn append_child_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    child: Opt<Value<'js>>,
) -> rquickjs::Result<Value<'js>> {
    let (Some(parent_id), Some(child_v)) = (node_id_of(&this.0), child.0) else {
        return Ok(Value::new_null(ctx));
    };
    let Some(child_id) = node_id_from_value(&child_v) else {
        return Ok(Value::new_null(ctx));
    };
    match with_dom_mut(|d| d.append_child(parent_id, child_id)) {
        Some(Ok(())) => {
            // S5q lifecycle：custom 子树连接态真转 → connectedCallback 派发。
            notify_connect_after_insert(&ctx, parent_id, child_id);
            Ok(child_v)
        }
        // S4q 完整化（R66）：DomError → DOMException（镜像 V8 R4；此前 null 吞错）。
        // `Ctx::throw` 返 Err 须**返回给调用方**才生效——忽略返回值 = 异常被丢弃。
        Some(Err(e)) => {
            let (name, msg) = dom_error_name(&e);
            Err(throw_dom_exception(&ctx, name, &msg))
        }
        None => Ok(Value::new_null(ctx)),
    }
}

/// `removeChild(child)`（spec `dom-node-removechild`）：返回被移除的 child
///（spec 返回值）；失配（非子节点/缺失）→ null（PoC 错误路径同 appendChild 注记）。
fn remove_child_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    child: Opt<Value<'js>>,
) -> rquickjs::Result<Value<'js>> {
    let (Some(parent_id), Some(child_v)) = (node_id_of(&this.0), child.0) else {
        return Ok(Value::new_null(ctx));
    };
    let Some(child_id) = node_id_from_value(&child_v) else {
        return Ok(Value::new_null(ctx));
    };
    match with_dom_mut(|d| d.remove_child(parent_id, child_id)) {
        Some(Ok(_)) => {
            // S5q lifecycle：断开子树 custom 元素 → disconnectedCallback 派发。
            notify_disconnect_after_remove(&ctx, child_id);
            Ok(child_v)
        }
        // S4q 完整化（R66）：非子节点 → NotFoundError（spec；此前 null 吞错）。
        Some(Err(e)) => Err(throw_dom_exception(&ctx, dom_error_name(&e).0, &dom_error_name(&e).1)),
        None => Ok(Value::new_null(ctx)),
    }
}

// ── S4q EventTarget（addEventListener/removeEventListener/dispatchEvent；镜像 V8 S4）──
//
// 派发语义（spec `concept-event-dispatch` PoC 子集）：target 站派发（无 capture/bubble
// 链——祖先链派发需 R40 式虚站基建，延后续切片）；按注册序触发全部监听器；
// stopPropagation 未实现（无传播链即无止点）；事件对象为**轻量 plain object**
//（type/target/currentTarget 三字段——Event 构造器族属 S5q CE 域）。

/// `addEventListener(type, callback, capture?)`（spec `dom-eventtarget-addeventlistener`）。
/// callback 存 Persistent 到 LISTENERS（`(ffi, type)` 键，注册序 append）。
/// capture 经第三参 truthiness（Opt<Value> 手动判——options 对象形态延后）。
fn add_event_listener_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    type_: rquickjs::Coerced<String>,
    callback: Value<'js>,
    capture: Opt<Value<'js>>,
) {
    let Some(id) = node_id_of(&this.0) else {
        return;
    };
    if !callback.is_function() {
        return; // spec：callback 非 callable → 忽略（不抛）
    }
    let cap = capture.0.is_some_and(|v| v.as_bool() == Some(true));
    let ffi = encode_node_id(id);
    LISTENERS.with(|l| {
        l.borrow_mut()
            .entry((ffi, type_.0))
            .or_default()
            .push((cap, Persistent::save(&ctx, callback)));
    });
}

/// `removeEventListener(type, callback, capture?)`（spec
/// `dom-eventtarget-removeeventlistener`）：按 `(type, callback identity, capture)`
/// 移除首个匹配（spec 移除「最早注册的等价监听器」；identity 经 JS === 语义近似——
/// Persistent restore 后同对象比较）。
fn remove_event_listener_method<'js>(
    this: This<Object<'js>>,
    ctx: Ctx<'js>,
    type_: rquickjs::Coerced<String>,
    callback: Value<'js>,
    capture: Opt<Value<'js>>,
) {
    let Some(id) = node_id_of(&this.0) else {
        return;
    };
    let cap = capture.0.is_some_and(|v| v.as_bool() == Some(true));
    let ffi = encode_node_id(id);
    LISTENERS.with(|l| {
        let mut map = l.borrow_mut();
        let Some(list) = map.get_mut(&(ffi, type_.0.clone())) else {
            return;
        };
        // 首个 (capture 匹配, 同回调) 条目移除。同回调判定：restore 到当前 ctx 后
        // JS 严格等（值恒等——Persistent 持同一 JS 对象）。
        for i in 0..list.len() {
            if list[i].0 == cap
                && list[i]
                    .1
                    .clone()
                    .restore(&ctx)
                    .is_ok_and(|stored| values_identical(&stored, &callback))
            {
                list.remove(i);
                break;
            }
        }
        if list.is_empty() {
            map.remove(&(ffi, type_.0));
        }
    });
}

/// JS 严格等（===）近似：对象恒等（rquickjs Value PartialEq 即 JS SameValue 语义的
/// 引用等价——同 Persistent 来源恒等成立）。
fn values_identical<'js>(a: &Value<'js>, b: &Value<'js>) -> bool {
    a == b
}

/// `event.stopPropagation()` 注入体（R67，spec `dom-event-stoppropagation`）：置内部
/// `__zw_stop` flag——止后续**节点**（当前节点剩余监听器仍触发，spec「inner invoke」全尽）。
/// 具名 fn 收 `This<Object>` = 事件对象，直接写 this 的 flag 属性（rquickjs 闭包 HRP 见模块注释）。
fn event_stop_propagation<'js>(this: This<Object<'js>>) {
    let _ = this.0.set("__zw_stop", true);
}

/// `event.stopImmediatePropagation()` 注入体（R67，spec `dom-event-stopimmediatepropagation`）：
/// 置 `__zw_stop_immediate`——立即止当前节点剩余监听器 + 后续节点。
fn event_stop_immediate_propagation<'js>(this: This<Object<'js>>) {
    let _ = this.0.set("__zw_stop_immediate", true);
}

/// 事件对象读 bool 属性（缺失/非 bool → false）。
fn event_bool_of(event: &Object, key: &str) -> bool {
    event.get::<_, bool>(key).unwrap_or(false)
}

/// `dispatchEvent(event)`（spec `dom-eventtarget-dispatchevent`，R67 三阶段完整化——
/// 镜像 V8 R3128/R3135 `dispatch_event_impl`）：event 为对象读 `.type`，或直接 type
/// 字符串（包成 `{type}` 对象——R63 PoC 形态兼容）。三阶段派发：capture（祖先 root→parent
/// 倒序，CAPTURING_PHASE=1，仅 capture 监听器）→ target（AT_TARGET=2，全部监听器注册序）
/// → bubble（祖先 parent→root 正序，BUBBLING_PHASE=3，仅 bubble 监听器，仅 `bubbles:true`）。
/// `currentTarget`/`eventPhase` 随传播更新，派发后复位（currentTarget=null/eventPhase=NONE）。
/// stopPropagation/stopImmediatePropagation 缺失时注入（fresh flag 复位，同 event 可重派发）。
/// 返回 `!(cancelable && defaultPrevented)`（spec 返值语义；preventDefault 注入延 Event
/// 构造器域）。监听器执行异常**不中断**派发（spec：回调异常报告后继续——此处吞掉）。
fn dispatch_event_method<'js>(this: This<Object<'js>>, ctx: Ctx<'js>, event: Opt<Value<'js>>) -> bool {
    let Some(id) = node_id_of(&this.0) else {
        return true;
    };
    // 事件对象标准化：对象原样；字符串 → 包 `{type}`（spec dispatchEvent 收 Event，此处
    // 轻量 plain object 形态——Event 构造器族属后续切片）。
    let event_obj = match event.0 {
        Some(v) if v.is_object() => match v.into_object() {
            Some(o) => o,
            None => return true,
        },
        Some(v) if v.is_string() => {
            let s: String = rquickjs::Coerced::<String>::from_js(&ctx, v.clone())
                .map(|c| c.0)
                .unwrap_or_default();
            let Ok(o) = rquickjs::Object::new(ctx.clone()) else {
                return true;
            };
            let _ = o.set("type", s);
            o
        }
        _ => return true,
    };
    // event.type（listener 键）。
    let Ok(type_) = event_obj.get::<_, String>("type") else {
        return true;
    };
    // spec：event.target = 派发目标（固定）；currentTarget 随传播。
    let target_v: Value = this.0.clone().into_value();
    let _ = event_obj.set("target", target_v.clone());
    let _ = event_obj.set("currentTarget", target_v);
    // stop flag 复位（fresh 派发语义，支持同 event 重派发）。
    let _ = event_obj.set("__zw_stop", false);
    let _ = event_obj.set("__zw_stop_immediate", false);
    // stop 方法注入（仅缺失时，不覆盖既有）。
    inject_missing_stop_method(&ctx, &event_obj, "stopPropagation", event_stop_propagation);
    inject_missing_stop_method(
        &ctx,
        &event_obj,
        "stopImmediatePropagation",
        event_stop_immediate_propagation,
    );
    let bubbles = event_bool_of(&event_obj, "bubbles");
    // 沿 parent 链收集 [target, parent, ..., root]（bubble 序）。with_dom 闭包内纯读收集
    // NodeId，释放 borrow 后再逐层派发（派发可能再入 mutation/listener 存储）。
    let chain: Vec<NodeId> = with_dom(|d| {
        let mut chain = vec![id];
        let mut cur = d.parent_node(id);
        while let Some(p) = cur {
            chain.push(p);
            cur = d.parent_node(p);
        }
        chain
    })
    .unwrap_or_default();
    // (node, phase) 访问列表：capture 祖先倒序 → target → bubble 祖先正序。
    let mut visits: Vec<(NodeId, i32)> = Vec::with_capacity(chain.len() * 2);
    for &n in chain[1..].iter().rev() {
        visits.push((n, 1)); // CAPTURING_PHASE
    }
    visits.push((id, 2)); // AT_TARGET
    for &n in chain[1..].iter() {
        visits.push((n, 3)); // BUBBLING_PHASE
    }
    let event_v: Value = event_obj.clone().into_value();
    let mut halted = false;
    for (node_id, phase) in visits {
        if halted {
            break;
        }
        // 非 bubbles 事件：bubble 阶段整体跳过（capture + target 仍派发）。
        if phase == 3 && !bubbles {
            break;
        }
        // 当前层 native 元素（currentTarget + listener this）。get_or_build 对任意 NodeId
        // 返包装（Document 等非 Element 亦得包装，currentTarget 可观测）。
        let curr = get_or_build_node_value(&ctx, node_id);
        let Some(curr_obj) = curr.as_object().cloned() else {
            continue;
        };
        let _ = event_obj.set("currentTarget", curr.clone());
        let _ = event_obj.set("eventPhase", phase);
        let ffi = encode_node_id(node_id);
        // 快照监听器（注册序，含 capture 标志；派发中 remove 的存活检查经 listener_present
        // ——镜像 V8 R3170 spec「inner invoke」步骤 5）。
        let listeners: Vec<(bool, Persistent<Value<'static>>)> = LISTENERS
            .with(|l| l.borrow().get(&(ffi, type_.clone())).cloned())
            .unwrap_or_default();
        for (cap, p) in listeners {
            // phase 过滤（spec invoke）：capture 阶段仅 capture；target 阶段全部；bubble 仅 bubble。
            let invoke = match phase {
                1 => cap,
                2 => true,
                _ => !cap,
            };
            if !invoke {
                continue;
            }
            // 派发期间被 removeEventListener 的监听器 skip。
            if !listener_present(&ctx, ffi, &type_, cap, &p) {
                continue;
            }
            let Ok(cb) = p.restore(&ctx) else {
                continue;
            };
            let Ok(func) = rquickjs::Function::from_value(cb) else {
                continue;
            };
            // this = currentTarget（spec listener 调用 this）。Args::this 显式设 this 后
            // push 事件对象，apply 低层 JS_Call（rquickjs 无 call_with_this 高层封装）。
            let mut args = rquickjs::function::Args::new(ctx.clone(), 1);
            let _ = args.this(curr_obj.clone());
            let _ = args.push_arg(event_v.clone());
            let _: rquickjs::Result<rquickjs::Value> = args.apply(&func);
            // stopImmediatePropagation：立即止（当前节点剩余 + 后续节点）。
            if event_bool_of(&event_obj, "__zw_stop_immediate") {
                halted = true;
                break;
            }
        }
        if halted {
            break;
        }
        // stopPropagation：当前节点监听器全尽，止后续节点（spec：止后续非当前剩余）。
        if event_bool_of(&event_obj, "__zw_stop") {
            halted = true;
        }
    }
    // 派发结束：currentTarget=null、eventPhase=NONE（spec：派发后 currentTarget 为 null）。
    let _ = event_obj.set("currentTarget", Value::new_null(ctx.clone()));
    let _ = event_obj.set("eventPhase", 0);
    // 返值 `!(cancelable && defaultPrevented)`（spec；plain object 无 preventDefault
    // 注入时 defaultPrevented 恒 false → true）。
    let cancelable = event_bool_of(&event_obj, "cancelable");
    let default_prevented = event_bool_of(&event_obj, "defaultPrevented");
    !(cancelable && default_prevented)
}

/// 事件对象缺失 stop 方法注入（不覆盖既有——原生 Event 实例经原型链已有则跳过）。
/// 只注入无参具名 fn（`fn(This<Object>)`）——rquickjs Function::new 对该签名直接可用。
fn inject_missing_stop_method<'js>(ctx: &Ctx<'js>, event: &Object<'js>, name: &str, f: fn(This<Object<'js>>)) {
    let has = event.get::<_, Value>(name).is_ok_and(|v| v.is_function());
    if !has {
        let Ok(func) = Function::new(ctx.clone(), f) else {
            return;
        };
        let _ = event.set(name, func);
    }
}

/// 监听器存活检查（派发期间 removeEventListener 后 skip——spec「inner invoke」步骤 5）。
/// 快照条目与 map 现存条目做 Persistent 值恒等（同 JS 对象）。
fn listener_present<'js>(
    ctx: &Ctx<'js>,
    ffi: u64,
    type_: &str,
    cap: bool,
    stored: &Persistent<Value<'static>>,
) -> bool {
    let Some(stored_v) = stored.clone().restore(ctx).ok() else {
        return false;
    };
    LISTENERS.with(|l| {
        l.borrow().get(&(ffi, type_.to_string())).is_some_and(|list| {
            list.iter()
                .any(|(c, p)| *c == cap && p.clone().restore(ctx).is_ok_and(|v| values_identical(&v, &stored_v)))
        })
    })
}

// ── S5q customElements 五件套 + lifecycle（spec `dom-customelementregistry`；
//    镜像 V8 custom_elements.rs 的「Rust 管树逻辑 / JS 管回调对象」职责分离）──
//
// QuickJS native 域无 polyfill `_ce_registry`，registry 存 Rust（CE_REGISTRY：
// tag → Persistent ctor）。**upgrade 策略**（PoC 简化，镜像 V8 S5b 的替代路径）：
// `__zw_native_create_element(tag)` 命中 registry 时**不 new ctor**，而是建 generic
// native 元素后把 ctor.prototype 挂为其原型（`set_prototype`）——custom class 的
// 字段初始化（constructor body）不执行，但 prototype 上的方法/属性全部可用，
// lifecycle 回调可派发。完整 ctor 执行（super() 注入 NodeId 链）延后续切片
//（rquickjs `construct` + upgrade 栈镜像 V8 UPGRADE_NODE_ID）。
// lifecycle：connected/disconnected 经 append/remove 钩子（下方 notify 函数在
// mutation 方法成功路径调用）；attributeChanged 简化经 setAttribute 变更派发（observedAttributes 过滤延后）。

/// `customElements.define(tag, ctor)`（spec `dom-customelementregistry-define`）：
/// tag ASCII 小写规范化；ctor 须 callable；重复定义静默忽略（spec 抛 NotSupportedError——
/// DOMException 基建延 S4q 完整化，注记同 R60b）。
fn ce_define<'js>(ctx: Ctx<'js>, tag: rquickjs::Coerced<String>, ctor: Value<'js>) -> rquickjs::Result<()> {
    // S4q 完整化（R66）：非 callable ctor → TypeError（spec「callback」型校验）。
    if !ctor.is_function() {
        let e = rquickjs::Exception::from_message(ctx.clone(), "The callback is not a function.")?;
        return Err(e.throw());
    }
    let key = tag.0.to_ascii_lowercase();
    // 重复定义 → NotSupportedError（spec；此前静默覆盖）。
    if CE_REGISTRY.with(|r| r.borrow().contains_key(&key)) {
        return Err(throw_dom_exception(
            &ctx,
            "NotSupportedError",
            "An element definition already exists with this name.",
        ));
    }
    CE_REGISTRY.with(|r| {
        r.borrow_mut().insert(key, Persistent::save(&ctx, ctor));
    });
    Ok(())
}

/// 标签名合法性（spec Name production 简化检查；镜像 V8 `is_valid_qualified_name` 的
/// QuickJS 版：非空 + 首字符 name-start（字母/`_`）+ 不含 `<`/`>`/空白/`/`）。
fn is_valid_tag_name(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }
    let first = tag.chars().next().expect("non-empty checked");
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    !tag.chars()
        .any(|c| c == '<' || c == '>' || c.is_whitespace() || c == '/')
}

/// `customElements.get(tag)`：未注册 → JS null。
fn ce_get<'js>(ctx: Ctx<'js>, tag: rquickjs::Coerced<String>) -> Value<'js> {
    let key = tag.0.to_ascii_lowercase();
    match CE_REGISTRY.with(|r| r.borrow().get(&key).cloned()) {
        Some(p) => p.restore(&ctx).unwrap_or_else(|_| Value::new_null(ctx)),
        None => Value::new_null(ctx),
    }
}

/// `customElements.getName(ctor)`：反向查 tag；未注册 → null。
fn ce_get_name<'js>(ctx: Ctx<'js>, ctor: Value<'js>) -> Value<'js> {
    let found: Option<String> = CE_REGISTRY.with(|r| {
        let map = r.borrow();
        for (k, p) in map.iter() {
            if let Ok(stored) = p.clone().restore(&ctx)
                && values_identical(&stored, &ctor)
            {
                return Some(k.clone());
            }
        }
        None
    });
    match found {
        Some(tag) => match rquickjs::String::from_str(ctx.clone(), &tag) {
            Ok(s) => s.into_value(),
            Err(_) => Value::new_null(ctx),
        },
        None => Value::new_null(ctx),
    }
}

/// `customElements.whenDefined(tag)`（spec）：PoC 同步简化——立即 resolve（真等待
/// 语义需 pending promise 表 + define 时批量 resolve，延异步域切片）。经
/// `Promise::new` + 立即调 resolve 构造已解析 promise。
fn ce_when_defined<'js>(ctx: Ctx<'js>, _tag: rquickjs::Coerced<String>) -> rquickjs::Result<rquickjs::Value<'js>> {
    let (promise, resolve, _) = rquickjs::Promise::new(&ctx)?;
    let _: rquickjs::Result<rquickjs::Value> = resolve.call(());
    Ok(promise.into_value())
}

/// `customElements.upgrade(node)`（spec）：PoC no-op 返回 undefined——upgrade 语义
///（对已在树中的未升级元素跑 ctor）依赖完整 ctor 执行链，延后续切片。
fn ce_upgrade<'js>(_ctx: Ctx<'js>, _node: Value<'js>) {}

/// create_element 命中 registry 的 upgrade 路径：generic 元素原型挂 ctor.prototype
///（见上方模块注释的 PoC 策略）。构造后调用。
fn apply_ce_prototype<'js>(obj: &Object<'js>, ctx: &Ctx<'js>, tag: &str) {
    let key = tag.to_ascii_lowercase();
    let Some(ctor_p) = CE_REGISTRY.with(|r| r.borrow().get(&key).cloned()) else {
        return;
    };
    let Ok(ctor) = ctor_p.restore(ctx) else {
        return;
    };
    let Some(ctor_obj) = ctor.as_object() else {
        return;
    };
    let Ok(proto) = ctor_obj.get::<_, Object>("prototype") else {
        return;
    };
    let _ = obj.set_prototype(Some(&proto));
    // S5q 深化（R65）：完整 ctor 执行——以 native 元素为 this **普通调用** registered
    // ctor（Args::this + apply，非 construct——`JS_CallConstructor*` 遵循 construct 语义
    // 会新建 this 对象，字段初始化落不到 native 元素上；this 绑定的普通调用等价 V8 版
    // super() 注入链的 Rust 侧目标形态）。ctor body 真正执行（this.count=41 等字段
    // 初始化直接落在 native 元素）；返回值忽略；抛异常静默（best-effort：升级失败退回
    // 纯原型挂载形态，方法面仍可用）。
    if let Ok(func) = rquickjs::Function::from_value(ctor) {
        let mut args = rquickjs::function::Args::new(ctx.clone(), 0);
        let _ = args.this(obj.clone());
        let _: rquickjs::Result<rquickjs::Value> = args.apply(&func);
    }
}

/// 节点子树 DFS 收集 custom 元素（tag 含 `-`——CE 名规范 fast-path，镜像 V8 R3271）。
fn collect_custom_subtree(id: NodeId, f: &mut dyn FnMut(NodeId, &str)) {
    let Some((tag, children)) = with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some((e.local_name().to_string(), d.child_nodes(id))),
            _ => None,
        })
    })
    .flatten() else {
        return;
    };
    f(id, &tag);
    for c in children {
        collect_custom_subtree(c, f);
    }
}

/// 节点是否连入 document（parent 链到根 = 文档根；镜像 V8 is_connected_to_document
/// headless 近似——祖先 parent=None 即根）。
fn is_connected_to_document(id: NodeId) -> bool {
    let Some(mut cur) = with_dom(|d| d.parent_node(id)).flatten() else {
        return false;
    };
    loop {
        let Some(next) = with_dom(|d| d.parent_node(cur)).flatten() else {
            return true; // 到根（parent=None）= 连入
        };
        cur = next;
    }
}

/// appendChild 成功后的 lifecycle 派发（镜像 V8 notify_connect_after_insert）：
/// 子树内 custom 元素按连接态真转派发 connectedCallback/disconnectedCallback
///（回调在 ctor.prototype 上，this = native 元素对象）。
fn notify_connect_after_insert<'js>(ctx: &Ctx<'js>, parent_id: NodeId, child_id: NodeId) {
    let parent_connected = is_connected_to_document(parent_id);
    let mut to_connect: Vec<NodeId> = Vec::new();
    let mut to_disconnect: Vec<NodeId> = Vec::new();
    collect_custom_subtree(child_id, &mut |id, tag| {
        if !tag.contains('-') {
            return;
        }
        let ffi = encode_node_id(id);
        let was = CONNECTED_CUSTOM.with(|c| c.borrow().contains(&ffi));
        if parent_connected && !was {
            to_connect.push(id);
        } else if !parent_connected && was {
            to_disconnect.push(id);
        }
    });
    if to_connect.is_empty() && to_disconnect.is_empty() {
        return;
    }
    // 先标记（防派发中再 mutation 状态错乱），再派发。
    for &id in &to_connect {
        CONNECTED_CUSTOM.with(|c| c.borrow_mut().insert(encode_node_id(id)));
    }
    for &id in &to_disconnect {
        CONNECTED_CUSTOM.with(|c| c.borrow_mut().remove(&encode_node_id(id)));
    }
    for id in to_connect {
        dispatch_ce_lifecycle(ctx, id, "connectedCallback");
    }
    for id in to_disconnect {
        dispatch_ce_lifecycle(ctx, id, "disconnectedCallback");
    }
}

/// removeChild 成功后的断开派发：子树 custom 元素（原已连）→ disconnectedCallback。
fn notify_disconnect_after_remove<'js>(ctx: &Ctx<'js>, removed_id: NodeId) {
    let mut to_disconnect: Vec<NodeId> = Vec::new();
    collect_custom_subtree(removed_id, &mut |id, tag| {
        if !tag.contains('-') {
            return;
        }
        let ffi = encode_node_id(id);
        if CONNECTED_CUSTOM.with(|c| c.borrow().contains(&ffi)) {
            to_disconnect.push(id);
        }
    });
    for id in to_disconnect {
        CONNECTED_CUSTOM.with(|c| c.borrow_mut().remove(&encode_node_id(id)));
        dispatch_ce_lifecycle(ctx, id, "disconnectedCallback");
    }
}

/// 对 custom 元素派发 ctor.prototype 上的 lifecycle 回调（this = native 元素；
/// 回调缺失/执行异常静默——best-effort，镜像 V8 桥接语义）。
fn dispatch_ce_lifecycle<'js>(ctx: &Ctx<'js>, id: NodeId, callback: &str) {
    let tag = with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name().to_ascii_lowercase()),
            _ => None,
        })
    })
    .flatten();
    let Some(tag) = tag else {
        return;
    };
    let Some(ctor_p) = CE_REGISTRY.with(|r| r.borrow().get(&tag).cloned()) else {
        return;
    };
    let Ok(ctor) = ctor_p.restore(ctx) else {
        return;
    };
    let Some(ctor_obj) = ctor.as_object() else {
        return;
    };
    let Ok(proto) = ctor_obj.get::<_, Object>("prototype") else {
        return;
    };
    let Ok(cb) = proto.get::<_, rquickjs::Function>(callback) else {
        return; // 未定义回调 → 静默（spec：回调可选）
    };
    let target = get_or_build_node_value(ctx, id);
    let Some(target_obj) = target.as_object().cloned() else {
        return;
    };
    let mut args = rquickjs::function::Args::new(ctx.clone(), 0);
    let _ = args.this(target_obj);
    let _: rquickjs::Result<rquickjs::Value> = args.apply(&cb);
}

/// 属性变更后派发 attributeChangedCallback(name, oldValue, newValue)（spec
/// `dom-customelementregistry` attributeChangedCallback；this = native 元素）。
/// 仅 registry 命中的 custom 元素派发；**observedAttributes 过滤（R68）**——ctor 的
/// `observedAttributes`（数组/getter，一次性求值）不含 name 则不派发（spec：仅 observed
/// 属性触发）；回调缺失静默。`old`/`new_value` 由调用方给（setAttribute 写前捕获 old、
/// removeAttribute 传 old + new=None → newValue null）。
fn dispatch_attribute_changed<'js>(ctx: &Ctx<'js>, id: NodeId, name: &str, old: Option<&str>, new_value: Option<&str>) {
    let tag = with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.local_name().to_ascii_lowercase()),
            _ => None,
        })
    })
    .flatten();
    let Some(tag) = tag else {
        return;
    };
    let Some(ctor_p) = CE_REGISTRY.with(|r| r.borrow().get(&tag).cloned()) else {
        return;
    };
    let Ok(ctor) = ctor_p.restore(ctx) else {
        return;
    };
    let Some(ctor_obj) = ctor.as_object() else {
        return;
    };
    // R68 observedAttributes 过滤（spec `dom-customelementregistry-observedattributes`）：
    // ctor.observedAttributes 求值（静态数组属性或 getter 均可——get 泛型调用），缺失
    // 或非数组 → 不过滤（PoC 宽松：无声明视为全观察，保持 R65 行为兼容）；命中数组则
    // 仅 name ∈ 数组（字符串比较）时派发。
    if let Ok(observed) = ctor_obj.get::<_, rquickjs::Array>("observedAttributes") {
        let mut hit = false;
        for i in 0..observed.len() {
            if let Ok(s) = observed.get::<String>(i)
                && s == name
            {
                hit = true;
                break;
            }
        }
        if !hit {
            return;
        }
    }
    let Ok(proto) = ctor_obj.get::<_, Object>("prototype") else {
        return;
    };
    let Ok(cb) = proto.get::<_, rquickjs::Function>("attributeChangedCallback") else {
        return;
    };
    let target = get_or_build_node_value(ctx, id);
    let Some(target_obj) = target.as_object().cloned() else {
        return;
    };
    let old_v: Value = old
        .and_then(|s| rquickjs::String::from_str(ctx.clone(), s).ok())
        .map(Value::from)
        .unwrap_or_else(|| Value::new_null(ctx.clone()));
    let new_v: Value = new_value
        .and_then(|s| rquickjs::String::from_str(ctx.clone(), s).ok())
        .map(Value::from)
        .unwrap_or_else(|| Value::new_null(ctx.clone()));
    let mut args = rquickjs::function::Args::new(ctx.clone(), 3);
    let _ = args.this(target_obj);
    let _ = args.push_arg(name.to_string());
    let _ = args.push_arg(old_v);
    let _ = args.push_arg(new_v);
    let _: rquickjs::Result<rquickjs::Value> = args.apply(&cb);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// slotmap ffi ↔ f64 Number 往返：`from_ffi` 保证「`as_ffi` 产出值恒可逆」（version
    /// 强制 odd 的 `|1`），但**任意值不自等**（`from_ffi(0).as_ffi() = 2^32`）。测试用
    /// 「真 NodeId 的 as_fi 产出值」作往返域（生产路径唯一实际形态——encode 只对真
    /// NodeId 调）。真 ffi = `(version:u32 odd) << 32 | idx:u32`；f64 无损域 = 低 53 位
    /// 精确 → version < 2^21（两百余万代节点更换，不可达）内恒无损。
    #[test]
    fn ffi_f64_round_trip() {
        for idx in [0u32, 1, 255, 1 << 20] {
            for version in [1u32, 3, (1 << 21) - 1] {
                let ffi = (u64::from(version) << 32) | u64::from(idx);
                let id = decode_node_id(ffi);
                assert_eq!(encode_node_id(id), ffi, "真 NodeId 的 as_ffi 产出值应恒可逆");
                let as_f64 = ffi as f64;
                assert_eq!(as_f64 as u64, ffi, "常规段 ffi 经 f64 承载应无损（version < 2^21）");
            }
        }
    }

    /// S0q PoC 端到端：QuickJS 原生 getter 读 live Document（不经字符串桥）。
    /// 镜像 V8 `dom_bindings` S0 PoC（`poc_internal_field_round_trip` 家族）。
    #[test]
    fn quickjs_native_element_poc() {
        let runtime = rquickjs::Runtime::new().expect("runtime");
        let ctx = rquickjs::Context::full(&runtime).expect("context");
        ctx.with(|ctx| {
            let dom = Rc::new(RefCell::new(zero_dom::parse_html(
                "<html><body><div id='main' class='c'>hi</div></body></html>",
            )));
            install_dom_bindings_quickjs(&ctx, dom);

            let eval_str = |code: &str| -> String {
                ctx.eval::<rquickjs::Coerced<String>, _>(code).map(|v| v.0).unwrap_or_else(|e| {
                    let caught = ctx.catch();
                    let msg = if caught.is_object() {
                        caught
                            .as_object()
                            .and_then(|o| o.get::<_, Option<String>>("message").ok().flatten())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    if msg.is_empty() { format!("__ERR__:{e}") } else { format!("__ERR__:{e}: {msg}") }
                })
            };

            // 1. 工厂命中：id → native 对象，getter 读 Rust DOM。
            assert_eq!(eval_str("typeof __zw_native_element_for_id"), "function");
            assert_eq!(eval_str("__zw_native_element_for_id('main').nodeType"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('main').tagName"), "DIV");
            assert_eq!(eval_str("__zw_native_element_for_id('main').nodeName"), "DIV");
            assert_eq!(eval_str("__zw_native_element_for_id('main').id"), "main");

            // S1q 只读属性族：className/namespaceURI/localName/textContent。
            assert_eq!(eval_str("__zw_native_element_for_id('main').className"), "c");
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').namespaceURI"),
                "http://www.w3.org/1999/xhtml"
            );
            assert_eq!(eval_str("__zw_native_element_for_id('main').localName"), "div");
            assert_eq!(eval_str("__zw_native_element_for_id('main').textContent"), "hi");
            // S2q textContent setter：替换子树 + null 清子（LegacyNull 语义）+ 读回闭环。
            assert_eq!(
                eval_str("(__zw_native_element_for_id('main').textContent = 'new text', 1)"),
                "1"
            );
            assert_eq!(eval_str("__zw_native_element_for_id('main').textContent"), "new text");
            assert_eq!(
                eval_str("(__zw_native_element_for_id('main').textContent = null, 1)"),
                "1"
            );
            assert_eq!(eval_str("__zw_native_element_for_id('main').textContent"), "");
            // className setter 写 live Document + getter 读回（S1q 读写闭环）。
            assert_eq!(
                eval_str("(__zw_native_element_for_id('main').className = 'x y', 1)"),
                "1"
            );
            assert_eq!(eval_str("__zw_native_element_for_id('main').className"), "x y");

            // S1q 字符串反射族：title/lang/accessKey（缺省 "" + setter 读写闭环）。
            assert_eq!(eval_str("__zw_native_element_for_id('main').title"), "");
            assert_eq!(eval_str("(__zw_native_element_for_id('main').title = 'tip', 1)"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('main').title"), "tip");
            assert_eq!(eval_str("(__zw_native_element_for_id('main').lang = 'zh', 1)"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('main').lang"), "zh");
            assert_eq!(eval_str("(__zw_native_element_for_id('main').accessKey = 'k', 1)"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('main').accessKey"), "k");

            // S2q 属性方法族：get/set/remove/hasAttribute（missing → null；remove 真移除）。
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').getAttribute('data-x')"),
                "null"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').setAttribute('data-x', '1'), 1"),
                "1"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').getAttribute('data-x')"),
                "1"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').hasAttribute('data-x')"),
                "true"
            );
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').removeAttribute('data-x'), \
                          __zw_native_element_for_id('main').hasAttribute('data-x')"
                ),
                "false"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').getAttribute('data-x')"),
                "null"
            );

            // 2. 身份缓存：同 NodeId 返同对象（spec identity）。
            assert_eq!(
                eval_str("__zw_native_element_for_id('main') === __zw_native_element_for_id('main')"),
                "true"
            );

            // 3. miss → null。
            assert_eq!(eval_str("__zw_native_element_for_id('nope')"), "null");

            // S2q mutation 族：create（detached）→ append（入树 + textContent 反映）
            // → remove（失配 null / 成功返 child）。
            assert_eq!(
                eval_str("globalThis.__el = __zw_native_create_element('p'); __el.nodeType"),
                "1"
            );
            assert_eq!(eval_str("__el.tagName"), "P", "createElement 产物 tagName（HTML 大写）");
            assert_eq!(
                eval_str("(__el.textContent = 'added', __zw_native_element_for_id('main').appendChild(__el) === __el)"),
                "true",
                "appendChild 返回 child（spec 返回值）"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').textContent"),
                "added",
                "append 后父 textContent 反映新子树（textContent setter 曾设 'new text' 后 null 清空，现子元素文本）"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').removeChild(__el) === __el"),
                "true",
                "removeChild 返回被移除 child"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').textContent"),
                "",
                "remove 后父 textContent 空"
            );

            // S2q 续：树读回 getter——childNodes/parentNode/firstChild/lastChild。
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').appendChild(__el), \
                          __zw_native_element_for_id('main').childNodes.length"
                ),
                "1",
                "append 后 childNodes 反映（Array 形态）"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').childNodes[0] === __el"),
                "true",
                "childNodes[0] 身份 === appendChild 的 child（身份缓存）"
            );
            assert_eq!(
                eval_str("__el.parentNode === __zw_native_element_for_id('main')"),
                "true",
                "child 的 parentNode 指回父（双向一致）"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').firstChild === __el"),
                "true",
                "firstChild"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').lastChild === __el"),
                "true",
                "lastChild（单子时同 firstChild）"
            );
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').removeChild(__el), \
                     __zw_native_element_for_id('main').firstChild === null"
                ),
                "true",
                "remove 后 firstChild → null（空子树）"
            );
            assert_eq!(
                eval_str("__el.parentNode === null"),
                "true",
                "detached 后 parentNode → null"
            );

            // S3q 查询族：全局工厂 + 元素级方法（全量选择器引擎）。
            assert_eq!(eval_str("typeof __zw_native_query_selector"), "function");
            assert_eq!(
                eval_str("__zw_native_query_selector('#main') === __zw_native_element_for_id('main')"),
                "true",
                "全局 qs 命中身份与 element_for_id 一致（共享身份缓存）"
            );
            assert_eq!(
                eval_str("__zw_native_query_selector('.c') === null"),
                "true",
                "class 选择器（'main' 无 class——className setter 已改 'x y'）"
            );
            assert_eq!(
                eval_str("__zw_native_query_selector('div#main') !== null"),
                "true",
                "复合选择器（tag#id）"
            );
            assert_eq!(eval_str("__zw_native_query_selector('#nope')"), "null", "miss → null");
            assert_eq!(
                eval_str("__zw_native_query_selector_all('div').length >= 1"),
                "true",
                "qsa 返回 Array（文档序）"
            );
            assert_eq!(
                eval_str("__zw_native_query_selector_all('nope-x').length"),
                "0",
                "qsa miss → 空 Array"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('main').querySelector('#main') !== null"),
                "true",
                "元素级 qs（自身子树作用域，含自身 id 命中）"
            );

            // S4q EventTarget：add → dispatch（注册序 + this/事件对象）→ remove → 再派发零触发。
            assert_eq!(
                eval_str(
                    "globalThis.__log = [];\
                     globalThis.__fnA = function (e) { __log.push('a:' + e.type + ':' + (this === __zw_native_element_for_id('main'))); };\
                     globalThis.__fnB = function (e) { __log.push('b'); };\
                     __zw_native_element_for_id('main').addEventListener('click', __fnA);\
                     __zw_native_element_for_id('main').addEventListener('click', __fnB);\
                     __zw_native_element_for_id('main').dispatchEvent('click'), 1"
                ),
                "1"
            );
            assert_eq!(
                eval_str("__log.join('|')"),
                "a:click:true|b",
                "注册序派发 + 事件对象 type + this === target"
            );
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').removeEventListener('click', __fnA);\
                     __log.length = 0;\
                     __zw_native_element_for_id('main').dispatchEvent('click'), __log.join('|')"
                ),
                "b",
                "removeEventListener 后仅剩 B（identity 匹配移除）"
            );

            // R67 三阶段派发（镜像 V8 R3128/R3135）：capture 祖先倒序(1，仅 capture
            // 监听器) → target 注册序全监听器(2) → bubble 祖先正序(3，仅 bubble)。
            // 链 = document > html > body > div#main；__p=body（capture 监听器），
            // __p2=html（bubble 监听器）——html 监听器不触发 capture（cap=false），
            // body 监听器不触发 bubble（cap=true），phase 过滤即 spec invoke 语义。
            assert_eq!(
                eval_str(
                    "globalThis.__pLog = [];\
                     var __p = __zw_native_element_for_id('main').parentNode;\
                     var __p2 = __p.parentNode;\
                     __p.addEventListener('ping', function (e) { __pLog.push('p:' + e.eventPhase + ':' + (e.currentTarget===__p ? 'P' : '?')); }, true);\
                     __p2.addEventListener('ping', function (e) { __pLog.push('g:' + e.eventPhase + ':' + (e.currentTarget===__p2 ? 'G' : '?')); });\
                     __zw_native_element_for_id('main').addEventListener('ping', function (e) { __pLog.push('m:' + e.eventPhase + ':' + (e.currentTarget===e.target ? 'T' : '?')); });\
                     __zw_native_element_for_id('main').dispatchEvent({type:'ping', bubbles:true}), __pLog.join('|')"
                ),
                "p:1:P|m:2:T|g:3:G",
                "三阶段：body capture 监听器(1) → target(2) → html bubble 监听器(3)；\
                 capture 标志双向过滤（p 不入 bubble 站、g 不入 capture 站）"
            );
            // 非 bubbles 事件：bubble 阶段跳过（capture + target 仍派发）。
            assert_eq!(
                eval_str(
                    "__pLog.length = 0;\
                     __zw_native_element_for_id('main').dispatchEvent({type:'ping', bubbles:false}), __pLog.join('|')"
                ),
                "p:1:P|m:2:T",
                "bubbles:false 不上溯（capture + target 照派）"
            );
            // stopPropagation：capture 站止后续节点（target/bubble 全不触发）。
            assert_eq!(
                eval_str(
                    "globalThis.__sLog = [];\
                     __p.addEventListener('stop', function (e) { __sLog.push('cap'); e.stopPropagation(); }, true);\
                     __p2.addEventListener('stop', function (e) { __sLog.push('g-bub'); });\
                     __zw_native_element_for_id('main').addEventListener('stop', function (e) { __sLog.push('m'); });\
                     __zw_native_element_for_id('main').dispatchEvent({type:'stop', bubbles:true}), __sLog.join('|')"
                ),
                "cap",
                "capture 站 stopPropagation 止后续节点（spec：当前节点剩余监听器全尽）"
            );
            // 派发后复位：currentTarget=null / eventPhase=NONE(0)；事件对象直传形态。
            assert_eq!(
                eval_str(
                    "globalThis.__after = null;\
                     __zw_native_element_for_id('main').addEventListener('after', function (e) { __after = [e.eventPhase, e.currentTarget === e.target ? 'T' : 'A']; });\
                     globalThis.__evA = {type:'after'};\
                     __zw_native_element_for_id('main').dispatchEvent(__evA);\
                     var __r = __after.join(',') + ';' + __evA.eventPhase + ';' + (__evA.currentTarget === null ? 'null' : 'obj');\
                     globalThis.__evA = null; __r"
                ),
                "2,T;0;null",
                "派发期 eventPhase=2/currentTarget=target；派发后复位 0/null"
            );

            // S5q customElements：define/get/getName + create 命中 upgrade（原型挂载）+
            // lifecycle connected/disconnected（append/remove 到 document 链）。
            assert_eq!(
                eval_str(
                    "globalThis.__ceLog = [];\
                     globalThis.__MyEl = function () {};\
                     __MyEl.prototype.greet = function () { return 'hi-' + this.tagName; };\
                     __MyEl.prototype.connectedCallback = function () { __ceLog.push('conn:' + this.tagName); };\
                     __MyEl.prototype.disconnectedCallback = function () { __ceLog.push('disc:' + this.tagName); };\
                     __zw_native_customElements.define('my-el', __MyEl), 1"
                ),
                "1"
            );
            assert_eq!(
                eval_str("__zw_native_customElements.get('MY-EL') === __MyEl"),
                "true",
                "get（ASCII 小写规范化命中）"
            );
            assert_eq!(
                eval_str("__zw_native_customElements.getName(__MyEl)"),
                "my-el",
                "getName 反查"
            );
            assert_eq!(
                eval_str("__zw_native_customElements.get('nope')"),
                "null",
                "未注册 → null"
            );
            // create 命中 registry → 原型挂载 + 完整 ctor 执行（R65：constructor body
            // 以 native 元素为 this 真正执行——字段初始化生效）。
            assert_eq!(
                eval_str(
                    "globalThis.__MyEl2 = function () { this.count = 41; };\
                     __MyEl2.prototype.greet = function () { return 'hi-' + this.tagName + ':' + (++this.count); };\
                     __zw_native_customElements.define('my-el2', __MyEl2), 1"
                ),
                "1"
            );
            assert_eq!(
                eval_str(
                    "globalThis.__myEl2 = __zw_native_create_element('my-el2');\
                     __myEl2.greet() + '/' + __myEl2.greet()"
                ),
                "hi-MY-EL2:42/hi-MY-EL2:43",
                "完整 ctor 执行：this.count=41 字段初始化生效 + greet ++count 状态保持（ctor body 以 native 元素为 this）"
            );
            assert_eq!(
                eval_str(
                    "globalThis.__myEl = __zw_native_create_element('my-el');\
                     __myEl.greet()"
                ),
                "hi-MY-EL",
                "R64 路径回归：无字段 ctor 的 prototype 方法仍可达"
            );
            // attributeChangedCallback：setAttribute 派发（name/null-old/new 简化）。
            assert_eq!(
                eval_str(
                    "__MyEl.prototype.attributeChangedCallback = function (n, o, v) { __ceLog.push('attr:' + n + ':' + v); };\
                     (__zw_native_element_for_id('main').appendChild(__myEl), 1)"
                ),
                "1",
                "先连入（append）供后续 setAttribute 派发场景"
            );
            assert_eq!(
                eval_str(
                    "__myEl.setAttribute('data-k', 'v1'), __ceLog.join('|')"
                ),
                "conn:MY-EL|attr:data-k:v1",
                "setAttribute → attributeChangedCallback（前缀 conn 来自 append 连入派发——ceLog 序贯）"
            );
            // R68：oldValue 写前捕获（二次 set 传旧值）+ removeAttribute 派发（new=null）
            // + observedAttributes 过滤（不命中不派发）。
            assert_eq!(
                eval_str(
                    "globalThis.__attrR68 = [];\
                     __MyEl.prototype.attributeChangedCallback = function (n, o, v) {\
                         __attrR68.push(n + ':' + (o === null ? 'null' : o) + ':' + (v === null ? 'null' : v));\
                     };\
                     __myEl.setAttribute('data-k', 'v2');\
                     __myEl.setAttribute('data-k', 'v2');\
                     __myEl.removeAttribute('data-k');\
                     __myEl.removeAttribute('data-k'), __attrR68.join('|')"
                ),
                "data-k:v1:v2|data-k:v2:v2|data-k:v2:null",
                "R68：oldValue 写前捕获（v1→v2）+ 同值 set 仍派发（spec 无值变化短路）+ removeAttribute new=null + 缺失 remove 不派发"
            );
            assert_eq!(
                eval_str(
                    "__attrR68.length = 0;\
                     __MyEl.observedAttributes = ['data-obs'];\
                     __myEl.setAttribute('data-skip', 'x');\
                     __myEl.setAttribute('data-obs', 'y'), __attrR68.join('|')"
                ),
                "data-obs:null:y",
                "R68 observedAttributes 过滤：不命中（data-skip）不派发，命中（data-obs）才派发（old=null 首次）"
            );
            // R69 attributes 复合对象：身份缓存 + length/item/getNamedItem/setNamedItem/
            // removeNamedItem 全闭环 + live 跟随。用 detached 新建元素（零初始属性，
            // 断言不与 #main 前序脚本累积的属性耦合）。
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').attributes === __zw_native_element_for_id('main').attributes"
                ),
                "true",
                "R69 身份缓存：el.attributes === el.attributes"
            );
            assert_eq!(
                eval_str(
                    "globalThis.__nnmEl = __zw_native_create_element('section');\
                     globalThis.__nnm = __nnmEl.attributes;\
                     [__nnm.length, __nnm.item(0) === null].join(',')"
                ),
                "0,true",
                "R69 新建元素零属性"
            );
            assert_eq!(
                eval_str(
                    "__nnm.setNamedItem({name:'data-a', value:'1'});\
                     [__nnm.length, __nnm.getNamedItem('data-a').value, __nnm.item(0).name].join(',')"
                ),
                "1,1,data-a",
                "R69 setNamedItem 写入 + length/getNamedItem/item 读回"
            );
            assert_eq!(
                eval_str(
                    "__nnm.getNamedItem('data-missing') === null && __nnm.item(9) === null"
                ),
                "true",
                "R69 miss 语义：getNamedItem/item 越界 → null"
            );
            assert_eq!(
                eval_str(
                    "__nnm.setNamedItem({name:'data-b', value:'2'});\
                     __nnm.removeNamedItem('data-a').value + '/' + __nnm.length + '/' +\
                     (__nnm.removeNamedItem('data-a') === null) + '/' +\
                     __nnmEl.getAttribute('data-b')"
                ),
                "1/1/true/2",
                "R69 removeNamedItem 返被移除条目 + 再移除 null + 元素 getAttribute live 跟随"
            );
            // lifecycle：append 到 document 链 → connectedCallback；remove → disconnected。
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').appendChild(__myEl), __ceLog.join('|')"
                ),
                "conn:MY-EL|attr:data-k:v1",
                "append 到 document 子树 → connectedCallback（attr 前缀来自先行 setAttribute 派发——ceLog 序贯）"
            );
            assert_eq!(
                eval_str(
                    "__zw_native_element_for_id('main').removeChild(__myEl), __ceLog.join('|')"
                ),
                "conn:MY-EL|attr:data-k:v1|disc:MY-EL",
                "remove → disconnectedCallback"
            );

            // 4. id setter 写 live Document + getter 读回（原生读写闭环）。
            assert_eq!(eval_str("(__zw_native_element_for_id('main').id = 'renamed', 1)"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('renamed').tagName"), "DIV");
            assert_eq!(eval_str("__zw_native_element_for_id('main')"), "null");

            // S4q 完整化（R66）：DOMException 错误路径——self-append cycle / 非法 tag /
            // 重复 define（catch 得 e.name）。
            assert_eq!(
                eval_str(
                    "globalThis.__err = null;\
                     try { var __self = __zw_native_element_for_id('renamed'); __self.appendChild(__self); }\
                     catch (e) { __err = (e && e.name) ? e.name : ('raw:' + e); }\
                     __err"
                ),
                "HierarchyRequestError",
                "appendChild 自环 → HierarchyRequestError（镜像 V8 R4）"
            );
            assert_eq!(
                eval_str(
                    "globalThis.__err2 = null;\
                     try { __zw_native_create_element('<bad>'); }\
                     catch (e) { __err2 = e.name; }\
                     __err2"
                ),
                "InvalidCharacterError",
                "createElement 非法 tag → InvalidCharacterError（镜像 V8 R3）"
            );
            assert_eq!(
                eval_str(
                    "globalThis.__err3 = null;\
                     try { __zw_native_customElements.define('my-el', __MyEl); }\
                     catch (e) { __err3 = e.name; }\
                     __err3"
                ),
                "NotSupportedError",
                "重复 define → NotSupportedError（spec；此前静默覆盖）"
            );

            // 5. 隐藏 ffi 不可枚举（Object.keys 只见 enumerable 反射属性——S1q 后
            //    id/className/title/lang/accessKey）。
            assert_eq!(
                eval_str("Object.keys(__zw_native_element_for_id('renamed')).join(',')"),
                "id,className,title,lang,accessKey"
            );

            reset_quickjs_state();
        });
    }

    /// S1q namespaceURI 空命名空间 → null（镜像 V8 `native_namespace_uri_getter` 分支）。
    /// SVG 元素（非 HTML ns）返回其真实命名空间；`create_element_ns("", …)` 空 ns → null。
    #[test]
    fn quickjs_namespace_uri_variants() {
        let runtime = rquickjs::Runtime::new().expect("runtime");
        let ctx = rquickjs::Context::full(&runtime).expect("context");
        ctx.with(|ctx| {
            let dom = Rc::new(RefCell::new(zero_dom::parse_html(
                "<html><body><svg id='s'></svg><div id='d'></div></body></html>",
            )));
            install_dom_bindings_quickjs(&ctx, dom);

            let eval_str = |code: &str| -> String {
                ctx.eval::<rquickjs::Coerced<String>, _>(code)
                    .map(|v| v.0)
                    .unwrap_or_else(|e| format!("__ERR__:{e}"))
            };

            assert_eq!(
                eval_str("__zw_native_element_for_id('s').namespaceURI"),
                "http://www.w3.org/2000/svg"
            );
            assert_eq!(
                eval_str("__zw_native_element_for_id('d').namespaceURI"),
                "http://www.w3.org/1999/xhtml"
            );
            assert_eq!(eval_str("__zw_native_element_for_id('d').localName"), "div");
            assert_eq!(eval_str("__zw_native_element_for_id('s').localName"), "svg");

            reset_quickjs_state();
        });
    }
}
