//! 原生 `HTMLElement` 构造器（P1b S5a，RFC §3.5.1）——注册为全局，使 `class MyEl extends HTMLElement`
//! 经 JS `class extends` 子类化成立（R3262 PoC 验证 internal field 经 super() 继承可行）。
//!
//! **S5a**（R3264）：`new HTMLElement()` / `class extends HTMLElement` 构造器——ctor 建新 detached 元素 +
//! 填 slot[0]=NodeId + 缓存 wrapper；instance_template `nodeType` accessor。
//! **S5b**（R3265）：ctor 优先读 [`gc::upgrade_node_id`]，createElement('my-el') upgrade 复用 host NodeId。
//! **S5d**（R3267）：instance_template 补 attr 族 + tree mutation 族（attributeChangedCallback 端到端可测）。
//! **S5 后续（R3268）**：instance_template 补全 Element/Node 完整接口（[`install_element_interface`]）——
//! custom 实例具备与 generic Element 模板等价的全套 Element/Node API（查询/内容/导航/子元素/反射属性/
//! 复杂对象 getter/Node 方法/现代 mutation/Adjacent/事件/焦点）。复用 element.rs/node.rs/event_target.rs/
//! dom_token_list.rs/css_style_declaration.rs/dataset.rs/namednodemap.rs 的 invoke（holder=custom 实例有 slot，
//! `read_node_id`/`args.this()` 经 slot 取 NodeId，与 generic Element 同款）。
//!
//! spec/设计：`docs/specs/p1b-v8-native-bindings-rfc.md` §3.5.1。

use v8;

use super::gc::{cache_native_element, encode_node_id, upgrade_node_id, with_dom_mut};

/// 构建并注册全局 `HTMLElement` 构造器（`install_dom_bindings` 调）。
///
/// FunctionTemplate 构造器：instance_template internal_field_count=1（实例 slot[0] 存 NodeId），
/// ctor 回调 [`native_html_element_ctor_invoke`] 建新 detached 元素 + 填 slot[0] + 缓存 wrapper。
/// instance_template 经 [`install_element_interface`] 挂完整 Element/Node 接口（custom 实例 Element API
/// 等价 generic Element 模板；holder=实例有 slot，accessor/method 透明可达——R3262 验证 super() 继承 slot）。
pub(super) fn build_and_register(scope: &mut v8::PinScope, global: v8::Local<v8::Object>) {
    let tmpl = v8::FunctionTemplate::builder(native_html_element_ctor_invoke).build(scope);
    let inst = tmpl.instance_template(scope);
    inst.set_internal_field_count(1);
    // 完整 Element/Node 接口挂 instance_template（与 generic Element 模板等价）。
    install_element_interface(scope, inst);
    if let (Some(f), Some(key)) = (tmpl.get_function(scope), v8::String::new(scope, "HTMLElement")) {
        let _ = global.set(scope, key.into(), f.into());
    }
}

/// 在 ObjectTemplate 上注册完整 Element/Node 接口（HTMLElement instance_template 用）。
///
/// 镜像 mod.rs `install_dom_bindings` 的 generic Element 模板注册集——accessor getter/setter + method
/// 复用 element.rs/node.rs/event_target.rs/dom_token_list.rs/css_style_declaration.rs/dataset.rs/
/// namednodemap.rs 的 invoke（共享一套，holder=custom 实例有 slot，与 generic Element 同款读写）。
/// **为何不复用 mod.rs 函数**：mod.rs 注册直接在 `tmpl`（局部变量）上 inline，抽出共享函数需重构核心
/// 文件（回归风险）；本函数自包含 HTMLElement 接口，重复为机械「名→invoke」映射（低风险，可接受）。
fn install_element_interface(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>) {
    use super::css_style_declaration::native_style_getter;
    use super::dataset::native_dataset_getter;
    use super::dom_token_list::native_class_list_getter;
    use super::element as el;
    use super::event_target as et;
    use super::namednodemap::native_attributes_getter;
    use super::node as nd;

    // ── 只读属性 accessor ──
    set_getter(scope, tmpl, "nodeType", nd::native_node_type_getter);
    set_getter(scope, tmpl, "tagName", el::native_tag_name_getter);
    set_getter(scope, tmpl, "namespaceURI", el::native_namespace_uri_getter);
    set_getter(scope, tmpl, "nodeName", nd::native_node_name_getter);

    // ── 反射属性（getter+setter）──
    set_getset(scope, tmpl, "id", el::native_id_getter, el::native_id_setter);
    set_getset(
        scope,
        tmpl,
        "className",
        el::native_class_name_getter,
        el::native_class_name_setter,
    );
    set_getset(
        scope,
        tmpl,
        "tabIndex",
        el::native_tab_index_getter,
        el::native_tab_index_setter,
    );
    set_getset(
        scope,
        tmpl,
        "hidden",
        el::native_hidden_getter,
        el::native_hidden_setter,
    );
    set_getset(
        scope,
        tmpl,
        "contentEditable",
        el::native_content_editable_getter,
        el::native_content_editable_setter,
    );
    set_getter(scope, tmpl, "isContentEditable", el::native_is_content_editable_getter);
    set_getset(
        scope,
        tmpl,
        "spellcheck",
        el::native_spellcheck_getter,
        el::native_spellcheck_setter,
    );
    for prop in ["title", "lang", "accessKey"] {
        set_getset(
            scope,
            tmpl,
            prop,
            el::native_string_reflected_getter,
            el::native_string_reflected_setter,
        );
    }
    set_getset(
        scope,
        tmpl,
        "dir",
        el::native_dir_getter,
        el::native_string_reflected_setter,
    );
    set_getset(scope, tmpl, "inert", el::native_inert_getter, el::native_inert_setter);
    set_getset(
        scope,
        tmpl,
        "draggable",
        el::native_draggable_getter,
        el::native_draggable_setter,
    );
    // ARIA 反射属性（~47，共用 native_aria_reflected_getter/setter + idl_to_attr 转 content 名）。
    for prop in [
        "role",
        "ariaLabel",
        "ariaLabelledBy",
        "ariaDescribedBy",
        "ariaDescription",
        "ariaDetails",
        "ariaHidden",
        "ariaDisabled",
        "ariaCurrent",
        "ariaHasPopup",
        "ariaControls",
        "ariaOwns",
        "ariaFlowTo",
        "ariaLive",
        "ariaAtomic",
        "ariaBusy",
        "ariaRelevant",
        "ariaInvalid",
        "ariaErrorMessage",
        "ariaKeyShortcuts",
        "ariaRoleDescription",
        "ariaBrailleLabel",
        "ariaBrailleRoleDescription",
        "ariaGrabbed",
        "ariaDropEffect",
        "ariaPressed",
        "ariaSelected",
        "ariaChecked",
        "ariaExpanded",
        "ariaModal",
        "ariaMultiLine",
        "ariaMultiSelectable",
        "ariaOrientation",
        "ariaSort",
        "ariaAutoComplete",
        "ariaReadOnly",
        "ariaRequired",
        "ariaPlaceholder",
        "ariaLevel",
        "ariaValueNow",
        "ariaValueMin",
        "ariaValueMax",
        "ariaValueText",
        "ariaSetSize",
        "ariaPosInSet",
        "ariaRowCount",
        "ariaRowIndex",
        "ariaColCount",
        "ariaColIndex",
        "ariaRowSpan",
        "ariaColSpan",
    ] {
        set_getset(
            scope,
            tmpl,
            prop,
            el::native_aria_reflected_getter,
            el::native_aria_reflected_setter,
        );
    }

    // ── 复杂对象 getter（返独立 ObjectTemplate 实例，owner=holder element NodeId）──
    set_getter(scope, tmpl, "classList", native_class_list_getter);
    set_getter(scope, tmpl, "style", native_style_getter);
    set_getter(scope, tmpl, "dataset", native_dataset_getter);
    set_getter(scope, tmpl, "attributes", native_attributes_getter);

    // ── 内容属性 ──
    set_getter(scope, tmpl, "children", el::native_children_getter);
    set_getset(
        scope,
        tmpl,
        "textContent",
        nd::native_text_content_getter,
        nd::native_text_content_setter,
    );
    set_getter(scope, tmpl, "childNodes", nd::native_child_nodes_getter);
    set_getset(
        scope,
        tmpl,
        "nodeValue",
        nd::native_node_value_getter,
        nd::native_node_value_setter,
    );
    set_getset(
        scope,
        tmpl,
        "innerHTML",
        el::native_inner_html_getter,
        el::native_inner_html_setter,
    );
    set_getset(
        scope,
        tmpl,
        "outerHTML",
        el::native_outer_html_getter,
        el::native_outer_html_setter,
    );

    // ── 节点导航（Node）──
    set_getter(scope, tmpl, "parentNode", nd::native_parent_node_getter);
    set_getter(scope, tmpl, "firstChild", nd::native_first_child_getter);
    set_getter(scope, tmpl, "lastChild", nd::native_last_child_getter);
    set_getter(scope, tmpl, "nextSibling", nd::native_next_sibling_getter);
    set_getter(scope, tmpl, "previousSibling", nd::native_previous_sibling_getter);

    // ── Attribute 方法 ──
    set_method(scope, tmpl, "getAttribute", el::native_get_attribute_invoke);
    set_method(scope, tmpl, "hasAttribute", el::native_has_attribute_invoke);
    set_method(scope, tmpl, "setAttribute", el::native_set_attribute_invoke);
    set_method(scope, tmpl, "removeAttribute", el::native_remove_attribute_invoke);
    set_method(scope, tmpl, "toggleAttribute", el::native_toggle_attribute_invoke);
    set_method(scope, tmpl, "hasAttributes", el::native_has_attributes_invoke);
    set_method(scope, tmpl, "getAttributeNames", el::native_get_attribute_names_invoke);

    // ── 查询方法 ──
    set_method(scope, tmpl, "querySelector", el::native_element_query_selector_invoke);
    set_method(
        scope,
        tmpl,
        "querySelectorAll",
        el::native_element_query_selector_all_invoke,
    );
    set_method(scope, tmpl, "matches", el::native_element_matches_invoke);
    set_method(scope, tmpl, "closest", el::native_element_closest_invoke);

    // ── 树 mutation（Node）──
    set_method(scope, tmpl, "appendChild", nd::native_append_child_invoke);
    set_method(scope, tmpl, "insertBefore", nd::native_insert_before_invoke);
    set_method(scope, tmpl, "removeChild", nd::native_remove_child_invoke);
    set_method(scope, tmpl, "replaceChild", nd::native_replace_child_invoke);
    set_method(scope, tmpl, "hasChildNodes", nd::native_has_child_nodes_invoke);
    set_method(scope, tmpl, "cloneNode", nd::native_clone_node_invoke);
    set_method(scope, tmpl, "contains", nd::native_contains_invoke);

    // ── 现代 mutation 族（ChildNode/ParentNode）──
    set_method(scope, tmpl, "remove", nd::native_element_remove_invoke);
    set_method(scope, tmpl, "prepend", nd::native_element_prepend_invoke);
    set_method(scope, tmpl, "append", nd::native_element_append_invoke);
    set_method(scope, tmpl, "before", nd::native_element_before_invoke);
    set_method(scope, tmpl, "after", nd::native_element_after_invoke);
    set_method(scope, tmpl, "replaceWith", nd::native_element_replace_with_invoke);

    // ── Adjacent 族 ──
    set_method(
        scope,
        tmpl,
        "insertAdjacentElement",
        nd::native_element_insert_adjacent_element_invoke,
    );
    set_method(
        scope,
        tmpl,
        "insertAdjacentText",
        nd::native_element_insert_adjacent_text_invoke,
    );

    // ── 事件（EventTarget）──
    set_method(scope, tmpl, "addEventListener", et::native_add_event_listener_invoke);
    set_method(
        scope,
        tmpl,
        "removeEventListener",
        et::native_remove_event_listener_invoke,
    );
    set_method(scope, tmpl, "dispatchEvent", et::native_dispatch_event_invoke);

    // ── 焦点 / 交互 ──
    set_method(scope, tmpl, "click", et::native_element_click_invoke);
    set_method(scope, tmpl, "focus", et::native_element_focus_invoke);
    set_method(scope, tmpl, "blur", et::native_element_blur_invoke);
}

// ── ObjectTemplate 注册 helper ──

/// 注册 method（FunctionTemplate wrapper，ZST fn 项）。ObjectTemplate::set 须传 **FunctionTemplate**
///（非 Function 实例，否则 V8 fatal "must be a Template"）。
fn set_method(
    scope: &mut v8::PinScope,
    tmpl: v8::Local<v8::ObjectTemplate>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let ft = v8::FunctionTemplate::builder(callback).build(scope);
    if let Some(k) = v8::String::new(scope, name) {
        tmpl.set(k.into(), ft.into());
    }
}

/// 注册只读 accessor（getter only）。getter bound 由 `set_accessor` 自身约束（MapFnTo 转换）。
fn set_getter<G>(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>, name: &str, getter: G)
where
    G: v8::MapFnTo<v8::AccessorNameGetterCallback>,
{
    if let Some(k) = v8::String::new(scope, name) {
        tmpl.set_accessor(k.into(), getter);
    }
}

/// 注册 getter+setter accessor（reflected 属性）。bound 由 `set_accessor_with_setter` 自身约束。
fn set_getset<G, S>(scope: &mut v8::PinScope, tmpl: v8::Local<v8::ObjectTemplate>, name: &str, getter: G, setter: S)
where
    G: v8::MapFnTo<v8::AccessorNameGetterCallback>,
    S: v8::MapFnTo<v8::AccessorNameSetterCallback>,
{
    if let Some(k) = v8::String::new(scope, name) {
        tmpl.set_accessor_with_setter(k.into(), getter, setter);
    }
}

/// `new HTMLElement()` / `class X extends HTMLElement` 的 `super()` 构造器回调：建新 detached 元素
///（`Document::create_element('div')`，基类抽象无具体 tag）→ NodeId 进 `this` slot[0]（External ptr 值）
/// → 缓存 wrapper（身份映射，与既有 native 元素一致）。subclass 实例经 super() 调此 ctor，slot 继承（R3262）。
///
/// **S5b upgrade 分支**（R3265）：custom element upgrade 经 `native_create_element_invoke` 在调
/// registered ctor 前设 [`gc::upgrade_node_id`]（host 已建元素得 NodeId，tag=`my-el`），ctor `super()`
/// 调此 ctor 时优先用该 NodeId（**不建新 div**），使 custom 实例与 host 建的元素同 NodeId。
fn native_html_element_ctor_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(obj) = this.to_object(scope) else {
        return;
    };
    // S5b upgrade：customElements upgrade 在途（`native_create_element_invoke` 设）→ 复用 host 建的
    // NodeId（同 tag='my-el' 元素），避免建新 detached div 与 host 元素脱节。无 upgrade 在途 → S5a
    // 直接 new 语义（建新 detached div）。
    let id = upgrade_node_id().or_else(|| with_dom_mut(|d| d.create_element("div")));
    let Some(id) = id else {
        return;
    };
    let ffi = encode_node_id(id);
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    // 缓存 wrapper（身份映射 + GC weak，复用既有 native 元素模式）——同 NodeId 后续 get_or_create 命中。
    cache_native_element(scope, ffi, obj);
}
