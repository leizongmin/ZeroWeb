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
use rquickjs::{Ctx, Function, Object, Persistent, Value};
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
/// S1q 只读版（setter 属 S2q 写入族）。
fn text_content_getter<'js>(this: This<Object<'js>>) -> String {
    let Some(id) = node_id_of(&this.0) else {
        return String::new();
    };
    with_dom(|d| d.text_content(id)).flatten().unwrap_or_default()
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
    obj.prop("localName", Accessor::from(local_name_getter).configurable())?;
    obj.prop("textContent", Accessor::from(text_content_getter).configurable())?;
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
    Ok(obj)
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
                ctx.eval::<rquickjs::Coerced<String>, _>(code)
                    .map(|v| v.0)
                    .unwrap_or_else(|e| format!("__ERR__:{e}"))
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

            // 2. 身份缓存：同 NodeId 返同对象（spec identity）。
            assert_eq!(
                eval_str("__zw_native_element_for_id('main') === __zw_native_element_for_id('main')"),
                "true"
            );

            // 3. miss → null。
            assert_eq!(eval_str("__zw_native_element_for_id('nope')"), "null");

            // 4. id setter 写 live Document + getter 读回（原生读写闭环）。
            assert_eq!(eval_str("(__zw_native_element_for_id('main').id = 'renamed', 1)"), "1");
            assert_eq!(eval_str("__zw_native_element_for_id('renamed').tagName"), "DIV");
            assert_eq!(eval_str("__zw_native_element_for_id('main')"), "null");

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
