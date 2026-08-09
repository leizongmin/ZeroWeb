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
    encode_node_id, node_exists, set_dom_source, set_element_template, with_dom, with_dom_mut,
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
        tmpl.set_accessor(k.into(), native_node_type_getter);
    }
    if let Some(k) = v8::String::new(scope, "tagName") {
        tmpl.set_accessor(k.into(), native_tag_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "nodeName") {
        tmpl.set_accessor(k.into(), native_node_name_getter);
    }
    if let Some(k) = v8::String::new(scope, "id") {
        tmpl.set_accessor_with_setter(k.into(), native_id_getter, native_id_setter);
    }
    if let Some(k) = v8::String::new(scope, "className") {
        tmpl.set_accessor_with_setter(k.into(), native_class_name_getter, native_class_name_setter);
    }
    // `children` getter（spec `dom-parentnode-children`）：元素**子元素**（跳过文本/注释）
    // → V8 Array of native 对象（文档序）。
    if let Some(k) = v8::String::new(scope, "children") {
        tmpl.set_accessor(k.into(), native_children_getter);
    }
    // `textContent` getter/setter（spec `dom-node-textcontent`）：读=子树文本拼接；写=清子 + 文本节点。
    if let Some(k) = v8::String::new(scope, "textContent") {
        tmpl.set_accessor_with_setter(k.into(), native_text_content_getter, native_text_content_setter);
    }
    // `childNodes` getter（spec `dom-node-childnodes`）：**全部子节点**（含文本/注释）→ V8 Array of
    // native 对象。区别于 `children`（仅元素）——解锁 R3103 textContent 写的文本节点可见性。
    // 文本/注释节点包同一模板：nodeType(3/8)/nodeName(#text/#comment)/textContent(=data) 经
    // 既有 node-type-aware getter 正确返回。
    if let Some(k) = v8::String::new(scope, "childNodes") {
        tmpl.set_accessor(k.into(), native_child_nodes_getter);
    }
    // `nodeValue` getter（spec `dom-node-nodevalue`）：Text/Comment/PI=data；Element/Document/…=null。
    if let Some(k) = v8::String::new(scope, "nodeValue") {
        tmpl.set_accessor(k.into(), native_node_value_getter);
    }
    // spec 方法（FunctionTemplate，args.this 读 NodeId）：getAttribute / hasAttribute /
    // setAttribute / removeAttribute。ObjectTemplate::set 须传 **Template**（非 Function 实例）——
    // FunctionTemplate 是 Template，实例化时各对象共享，args.this() 取回实例。
    let get_attr_tmpl = v8::FunctionTemplate::builder(native_get_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getAttribute") {
        tmpl.set(k.into(), get_attr_tmpl.into());
    }
    let has_attr_tmpl = v8::FunctionTemplate::builder(native_has_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "hasAttribute") {
        tmpl.set(k.into(), has_attr_tmpl.into());
    }
    let set_attr_tmpl = v8::FunctionTemplate::builder(native_set_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "setAttribute") {
        tmpl.set(k.into(), set_attr_tmpl.into());
    }
    let rm_attr_tmpl = v8::FunctionTemplate::builder(native_remove_attribute_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeAttribute") {
        tmpl.set(k.into(), rm_attr_tmpl.into());
    }
    // element 子树作用域查询（spec `dom-parentnode-queryselector(-all)`）：`args.this()` 取
    // 元素 NodeId 作 root，**仅后代**（排除元素自身，见 [`native_element_query_selector_invoke`]）。
    let eqs_tmpl = v8::FunctionTemplate::builder(native_element_query_selector_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "querySelector") {
        tmpl.set(k.into(), eqs_tmpl.into());
    }
    let eqsa_tmpl = v8::FunctionTemplate::builder(native_element_query_selector_all_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "querySelectorAll") {
        tmpl.set(k.into(), eqsa_tmpl.into());
    }
    // spec 树 mutation 方法（`args.this()` = parent NodeId，参为 native element 对象读 internal slot）：
    // appendChild / insertBefore / removeChild。Document 写经 with_dom_mut。
    let append_tmpl = v8::FunctionTemplate::builder(native_append_child_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "appendChild") {
        tmpl.set(k.into(), append_tmpl.into());
    }
    let insert_before_tmpl = v8::FunctionTemplate::builder(native_insert_before_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "insertBefore") {
        tmpl.set(k.into(), insert_before_tmpl.into());
    }
    let remove_child_tmpl = v8::FunctionTemplate::builder(native_remove_child_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeChild") {
        tmpl.set(k.into(), remove_child_tmpl.into());
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

    // 4. 全局工厂 __zw_native_query_selector(sel) / __zw_native_query_selector_all(sel)
    //    —— spec `dom-parentnode-queryselector(-all)`：文档根下按**全量选择器引擎**
    //    （[`zero_dom::Document::query_selector`] / `query_selector_all`，消费 tag/`*`/
    //    `#id`/`.class`/`[attr]`+6 运算符/伪类/后代·子组合器/逗号列表）匹配 → native 对象
    //    （单）/ V8 Array（复，文档序）。R3098 工厂仅 `get_element_by_id`，本切片把
    //    querySelector 族从 polyfill 字符串桥搬到 native（返 NodeId→对象，无 String 往返）。
    let qs = v8::FunctionTemplate::builder(native_query_selector_invoke).build(scope);
    let qs_fn = qs.get_function(scope);
    let qs_key = v8::String::new(scope, "__zw_native_query_selector");
    if let (Some(f), Some(key)) = (qs_fn, qs_key) {
        let _ = global.set(scope, key.into(), f.into());
    }
    let qsa = v8::FunctionTemplate::builder(native_query_selector_all_invoke).build(scope);
    let qsa_fn = qsa.get_function(scope);
    let qsa_key = v8::String::new(scope, "__zw_native_query_selector_all");
    if let (Some(f), Some(key)) = (qsa_fn, qsa_key) {
        let _ = global.set(scope, key.into(), f.into());
    }

    // 5. 全局工厂 __zw_native_create_element(tag) —— spec `dom-document-createelement`：
    //    `Document::create_element` 造新 Element NodeId → native 对象（未挂载，appendChild 落位）。
    //    解锁原生树构建（createElement + appendChild 全 native，无 polyfill String 桥）。
    let ce = v8::FunctionTemplate::builder(native_create_element_invoke).build(scope);
    let ce_fn = ce.get_function(scope);
    let ce_key = v8::String::new(scope, "__zw_native_create_element");
    if let (Some(f), Some(key)) = (ce_fn, ce_key) {
        let _ = global.set(scope, key.into(), f.into());
    }
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

/// `nodeName` getter：spec `dom-node-nodename`——Element=tagName（HTML 大写），
/// 其他节点类型为固定串（#text/#comment/#document/#document-fragment）。
///
/// native 对象经 `get_element_by_id` 创建，均为 Element，故主路径 nodeName==tagName；
/// 非 Element 分支为 spec 合规防御（PI/DocumentType 的 target/name 近似，元素主导）。
fn native_node_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let nm: Option<String> = with_dom(|d| node_name(d, id)).flatten();
    let Some(nm) = nm else {
        return;
    };
    if let Some(s) = v8::String::new(scope, &nm) {
        rv.set(s.into());
    }
}

/// Rust 侧 nodeName 计算（spec `dom-node-nodename`）。
fn node_name(doc: &Document, id: NodeId) -> Option<String> {
    let n = doc.get(id)?;
    Some(match &n.kind {
        NodeKind::Element(e) => e.local_name().to_ascii_uppercase(),
        NodeKind::Text(_) => "#text".into(),
        NodeKind::Comment(_) => "#comment".into(),
        NodeKind::Document(_) => "#document".into(),
        NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => "#document-fragment".into(),
        // PI 的 nodeName=target、DocumentType=name；native 对象均为 Element，此处近似防御。
        NodeKind::ProcessingInstruction(_) => "#processing-instruction".into(),
        NodeKind::DocumentType(_) => "#document-type".into(),
    })
}

/// `id` getter（reflected attribute，spec `dom-id`）：`get_attribute('id')`，缺省 `""`。
fn native_id_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    read_reflected_attr(scope, &args, "id", "", &mut rv);
}

/// `className` getter（reflected attribute，spec `dom-classname`）：`get_attribute('class')`，缺省 `""`。
fn native_class_name_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    read_reflected_attr(scope, &args, "class", "", &mut rv);
}

/// 反射属性 getter 共用：读 internal slot NodeId → `Document::get_attribute(name)`，
/// 缺省 `default`（reflected 属性缺省 `""`）。
fn read_reflected_attr(
    scope: &mut v8::PinScope,
    args: &v8::PropertyCallbackArguments,
    attr: &str,
    default: &str,
    rv: &mut v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = with_dom(|d| d.get_attribute(id, attr))
        .flatten()
        .unwrap_or_else(|| default.to_string());
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// `id` setter（reflected，spec `dom-id`）：值 ToString 后 `set_attribute('id', val)`。
/// 经 [`with_dom_mut`] 写真实 DOM（更新 id_map）。
fn native_id_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    write_reflected_attr(scope, &args, "id", value);
}

/// `className` setter（reflected，spec `dom-classname`）：值 ToString 后 `set_attribute('class', val)`。
fn native_class_name_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    write_reflected_attr(scope, &args, "class", value);
}

/// 反射属性 setter 共用：读 internal slot NodeId → 值 ToString → `Document::set_attribute(name, val)`。
fn write_reflected_attr(
    scope: &mut v8::PinScope,
    args: &v8::PropertyCallbackArguments,
    attr: &str,
    value: v8::Local<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(id, attr, &val));
}

/// 取任意 `Local<Value>` 经 JS ToString → Rust String（缺省空串）。
/// spec reflected attribute setter 把值强转字符串后存。
fn local_value_to_string(scope: &mut v8::PinScope, value: v8::Local<v8::Value>) -> String {
    value
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default()
}

// ── 方法回调（Element 上：getAttribute / hasAttribute / setAttribute / removeAttribute）──

/// `getAttribute(name)`：读 internal slot NodeId → `Document::get_attribute`。
/// spec `dom-element-getattribute`：缺省/非 Element → `null`。
fn native_get_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let val = with_dom(|d| d.get_attribute(id, &name)).flatten();
    match val {
        Some(v) => {
            if let Some(s) = v8::String::new(scope, &v) {
                rv.set(s.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `hasAttribute(name)`：读 internal slot NodeId → `Document::has_attribute` → `v8::Boolean`。
/// spec `dom-element-hasattribute`。
fn native_has_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let has = with_dom(|d| d.has_attribute(id, &name)).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, has).into());
}

/// `setAttribute(name, value)`：读 internal slot NodeId → 两参 ToString →
/// `Document::set_attribute`（更新 id_map 当 name=='id'）。spec `dom-element-setattribute`
/// 返 `undefined`（留 ReturnValue 默认）。非 native element `this` → no-op。
fn native_set_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let value = string_arg(scope, &args, 1);
    with_dom_mut(|d| d.set_attribute(id, &name, &value));
}

/// `removeAttribute(name)`：读 internal slot NodeId → `Document::remove_attribute`（name=='id'
/// 时清 id_map）。spec `dom-element-removeattribute` 返 `undefined`。
fn native_remove_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    with_dom_mut(|d| d.remove_attribute(id, &name));
}

/// 取 FunctionCallbackArguments 第 `idx` 参为 Rust String（缺省空串）。
fn string_arg(scope: &mut v8::PinScope, args: &v8::FunctionCallbackArguments, idx: i32) -> String {
    args.get(idx)
        .to_string(scope)
        .map(|s| s.to_rust_string_lossy(scope))
        .unwrap_or_default()
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

/// `__zw_native_query_selector(sel)`：spec `dom-parentnode-queryselector`——
/// 文档根下按**全量选择器引擎**（[`zero_dom::Document::query_selector`]，消费 tag/`*`/
/// `#id`/`.class`/`[attr]`+运算符/伪类/组合器）找首个匹配元素 → native 对象。
///
/// 无匹配 / 空 / 非法选择器 → `null`（`parse_selector_chain` 失败返 `None`，无 panic）。
fn native_query_selector_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let Some(id) = with_dom(|d| d.query_selector(d.root(), sel.trim())).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `__zw_native_query_selector_all(sel)`：spec `dom-parentnode-queryselectorall`——
/// 文档根下按全量选择器引擎（[`zero_dom::Document::query_selector_all`]）收集全部匹配
/// 元素 → V8 `Array` of native 对象（文档序）。空 / 非法选择器 → 空 `Array`。
fn native_query_selector_all_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let sel = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(d.root(), sel.trim())).unwrap_or_default();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `element.querySelector(sel)`：spec `dom-parentnode-queryselector`（**元素子树作用域**）——
/// 元素**后代**中首个匹配 → native 对象。区别于文档级 [`native_query_selector_invoke`]：
/// root = `args.this()` 元素 NodeId，且**排除元素自身**（dom `query_selector_all` 含 root 候选，
/// spec descendants-only，镜像 polyfill `query_match_in_subtree` 的 `.filter(|n| *n != root)`）。
///
/// 经 `query_selector_all` + filter + first 取首个后代（比 polyfill `query_selector` + filter 更
/// 正确：后者若元素自身匹配则返 None，本实现继续找首个后代）。
/// OPTIMIZATION: 当前 collect-all-then-first；超大子树可短路（find_first_matching 跳 root）。
/// 无匹配 / 空 / 非法 → `null`；非 native element `this` → `undefined`（getter 一致）。
fn native_element_query_selector_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(root) = read_node_id(scope, &this) else {
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let id: Option<NodeId> = with_dom(|d| d.query_selector_all(root, sel.trim()))
        .unwrap_or_default()
        .into_iter()
        .find(|id| *id != root);
    match id {
        Some(id) => {
            if let Some(obj) = get_or_create_native_element(scope, id) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `element.querySelectorAll(sel)`：spec `dom-parentnode-queryselectorall`（**元素子树作用域**）——
/// 元素**后代**全部匹配 → V8 `Array` of native 对象（文档序，排除元素自身）。非 native element
/// `this` → 空 `Array`（避免 `.length` 访问报错）。
fn native_element_query_selector_all_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(root) = read_node_id(scope, &this) else {
        rv.set(v8::Array::new(scope, 0).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let ids: Vec<NodeId> = with_dom(|d| d.query_selector_all(root, sel.trim()))
        .unwrap_or_default()
        .into_iter()
        .filter(|id| *id != root)
        .collect();
    let arr = v8::Array::new(scope, ids.len() as i32);
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, id) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `__zw_native_create_element(tag)`：spec `dom-document-createelement`——
/// `Document::create_element(tag)` 造新 Element NodeId → native 对象（**未挂载**，需 appendChild）。
/// 空/缺省 tag → `div`（与 polyfill create_element 一致，spec 实际应抛，本切片 best-effort）。
fn native_create_element_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let mut tag = string_arg(scope, &args, 0);
    if tag.trim().is_empty() {
        tag = "div".to_string();
    }
    // create（borrow_mut 释放后）→ 包 native 对象（get_or_create_native_element 内含 stale 校验）。
    let Some(id) = with_dom_mut(|d| d.create_element(tag.trim())) else {
        return;
    };
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
}

/// `children` getter（spec `dom-parentnode-children`）：元素**子元素**（跳过文本/注释）
/// → V8 Array of native 对象（文档序）。非 Element 子节点不返（native 仅 Element 对象；
/// `childNodes` 含文本/注释需 native 非 Element 节点，后续切片）。
fn native_children_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let child_ids: Vec<NodeId> = with_dom(|d| {
        d.child_nodes(id)
            .into_iter()
            .filter(|c| d.get(*c).is_some_and(|n| matches!(n.kind, NodeKind::Element(_))))
            .collect()
    })
    .unwrap_or_default();
    let arr = v8::Array::new(scope, child_ids.len() as i32);
    for (i, cid) in child_ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, cid) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `textContent` getter（spec `dom-node-textcontent`）：子树文本拼接（`Document::text_content`，
/// 递归收集后代 Text 节点 data）。空子树 → `""`。
fn native_text_content_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let text = with_dom(|d| d.text_content(id)).flatten().unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &text) {
        rv.set(s.into());
    }
}

/// `textContent` setter（spec `dom-node-textcontent`）：值 ToString 后**清空全部子节点**，
/// 非空则追加单 Text 节点（`create_text_node` + `append_child`）。空串 → 仅清空（不添空 Text 节点）。
fn native_text_content_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| {
        // 移除全部子节点（先收集 NodeId 避免边遍历边改）。
        let children = d.child_nodes(id);
        for c in children {
            let _ = d.remove_child(id, c);
        }
        // 非空 → 追加文本节点。
        if !val.is_empty() {
            let text_id = d.create_text_node(&val);
            let _ = d.append_child(id, text_id);
        }
    });
}

/// `childNodes` getter（spec `dom-node-childnodes`）：**全部子节点**（含文本/注释）→ V8 Array of
/// native 对象（文档序）。区别于 [`native_children_getter`]（仅元素）——文本/注释节点经同一模板
/// 包后 nodeType(3/8)/nodeName/textContent 正确（node-type-aware getter）。
fn native_child_nodes_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let child_ids: Vec<NodeId> = with_dom(|d| d.child_nodes(id)).unwrap_or_default();
    let arr = v8::Array::new(scope, child_ids.len() as i32);
    for (i, cid) in child_ids.into_iter().enumerate() {
        if let Some(obj) = get_or_create_native_element(scope, cid) {
            let _ = arr.set_index(scope, i as u32, obj.into());
        }
    }
    rv.set(arr.into());
}

/// `nodeValue` getter（spec `dom-node-nodevalue`）：Text/Comment/PI=data；其余（Element/Document/
/// DocumentFragment/ShadowRoot/DocumentType）=null。区别于 `textContent`（Element 返子树文本）。
fn native_node_value_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let Some(val) = with_dom(|d| node_value(d, id)).flatten() else {
        rv.set(v8::null(scope).into());
        return;
    };
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// Rust 侧 nodeValue 计算（spec `dom-node-nodevalue`）。Text/Comment/PI=data；其余 None（→null）。
fn node_value(doc: &Document, id: NodeId) -> Option<String> {
    let n = doc.get(id)?;
    Some(match &n.kind {
        NodeKind::Text(t) => t.content.clone(),
        NodeKind::Comment(c) => c.content.clone(),
        NodeKind::ProcessingInstruction(p) => p.data.clone(),
        // Element/Document/DocumentFragment/ShadowRoot/DocumentType → null（spec）。
        _ => return None,
    })
}

// ── 树 mutation 方法（Element 上：appendChild / insertBefore / removeChild）──

/// `appendChild(child)`：spec `dom-node-appendchild`——`args.this()`=parent，参=child native
/// 对象；`Document::append_child` 移动（含 re-parent、cycle 检测）。成功返 child 对象（spec），
/// Err（cycle/not-found）→ best-effort 留 undefined（不抛，限制记录）。
fn native_append_child_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    let ok = with_dom_mut(|d| d.append_child(parent, child))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        set_native_element(scope, child, &mut rv);
    }
}

/// `insertBefore(newChild, refChild)`：spec `dom-node-insertbefore`——parent=this，参 0=newChild、
/// 参 1=refChild native 对象；`Document::insert_before`。`refChild` 缺省/null → 末尾追加（spec）。
/// 成功返 newChild 对象；Err → best-effort 留 undefined。
fn native_insert_before_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(new_child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    // refChild null/缺省 → 末尾追加（spec：ref 为 null 时同 appendChild）。
    let ref_child = node_id_from_value(scope, args.get(1));
    let ok = with_dom_mut(|d| match ref_child {
        Some(ref_id) => d.insert_before(parent, new_child, ref_id),
        None => d.append_child(parent, new_child),
    })
    .map(|r| r.is_ok())
    .unwrap_or(false);
    if ok {
        set_native_element(scope, new_child, &mut rv);
    }
}

/// `removeChild(child)`：spec `dom-node-removechild`——parent=this，参=child native 对象；
/// `Document::remove_child`。成功返被移除的 child 对象（spec）；Err → best-effort 留 undefined。
fn native_remove_child_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(parent) = read_node_id(scope, &this) else {
        return;
    };
    let Some(child) = node_id_from_value(scope, args.get(0)) else {
        return;
    };
    let ok = with_dom_mut(|d| d.remove_child(parent, child))
        .map(|r| r.is_ok())
        .unwrap_or(false);
    if ok {
        set_native_element(scope, child, &mut rv);
    }
}

/// mutation 方法成功尾共用：把 NodeId 包成 native 对象 set 到 `rv`（appendChild/insertBefore/
/// removeChild 成功返被操作节点对象）。抽离以避 `if ok { if let ... }` 嵌套（MSRV 1.85 无 let-chains）。
fn set_native_element(scope: &mut v8::PinScope, id: NodeId, rv: &mut v8::ReturnValue<v8::Value>) {
    if let Some(obj) = get_or_create_native_element(scope, id) {
        rv.set(obj.into());
    }
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
