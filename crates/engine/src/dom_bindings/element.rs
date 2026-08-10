//! Element 特有原生绑定——拆自 mod.rs（RFC §3.2 子模块化 stage 4b，本轮 R3120；§3.2 Element bulk 闭合）。
//!
//! DOM `Element` 接口特有面（spec DOM Standard `Element`）：tagName / id·className getter+setter
//!（reflected attribute，+ 私有反射助手 read_reflected_attr/write_reflected_attr）/ getAttribute·
//! hasAttribute·setAttribute·removeAttribute / children / element 子树作用域 querySelector·querySelectorAll
//! / innerHTML·outerHTML getter。Node 基类面（nodeType/nodeName/nodeValue/textContent/childNodes/
//! 导航/树 mutation/cloneNode/contains）在 node 子模块（R3119）。
//!
//! 可见性：注册于 Element 模板的 getter/invoke 为 `pub(super)`（mod.rs `install_dom_bindings`
//! 注册经 `element::` 调）；`read_reflected_attr` / `write_reflected_attr` 为本模块私有助手。
//! 读 `super::read_node_id` / `super::string_arg` / `super::get_or_create_native_element` /
//! `super::local_value_to_string`（mod.rs 私有共享——Rust 规则：私有项对后代模块可见）+
//! `super::gc::{with_dom, with_dom_mut}`。

use v8;

use zero_dom::{NodeId, NodeKind};

use super::gc::{with_dom, with_dom_mut};
use super::{get_or_create_native_element, local_value_to_string, read_node_id, string_arg};

// ── accessor getter（ZST fn；状态经 gc.rs 线程局部）─────────────────

/// `tagName` getter：读 internal slot[0] NodeId → Element `local_name` → 大写 → `v8::String`。
///
/// 仅 Element 有 tagName（HTML 大写，spec `dom-element-tagname`）；非 Element / stale →
/// undefined。
pub(super) fn native_tag_name_getter(
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

/// `namespaceURI` getter（spec `dom-node-namespaceuri`）：元素命名空间 URI 字符串，
/// 空 namespace → null；非 Element / stale → undefined（与 tagName getter 一致）。
///
/// 闭合 R3163 限制②（namespace 经 native 可读）。`createElement` 元素为 XHTML 命名空间
///（`http://www.w3.org/1999/xhtml`），`createElementNS` 元素为指定命名空间（SVG/MathML 等），
/// 使 namespace 检查（如 `el.namespaceURI === 'http://www.w3.org/2000/svg'`）经原生可达。
pub(super) fn native_namespace_uri_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    // with_dom 返 Option<Option<String>>：外层 None = 无 DOM 源；内层 Some(uri) = 元素命名空间，
    // 内层 None = 节点不存在 / 非 Element / 空 namespace（spec namespaceURI 无命名空间 → null）。
    let ns: Option<Option<String>> = with_dom(|d| {
        d.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(e) => {
                let ns = e.namespace();
                if ns.is_empty() { None } else { Some(ns.to_string()) }
            }
            _ => None,
        })
    });
    match ns {
        Some(Some(uri)) => {
            if let Some(s) = v8::String::new(scope, &uri) {
                rv.set(s.into());
            }
        }
        // 空命名空间 → null（spec `Node.namespaceURI` 无命名空间返 null，非 undefined）。
        Some(None) => rv.set(v8::null(scope).into()),
        None => {} // 无 DOM 源 → undefined（留默认）。
    }
}

/// `id` getter（reflected attribute，spec `dom-id`）：`get_attribute('id')`，缺省 `""`。
pub(super) fn native_id_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    read_reflected_attr(scope, &args, "id", "", &mut rv);
}

/// `className` getter（reflected attribute，spec `dom-classname`）：`get_attribute('class')`，缺省 `""`。
pub(super) fn native_class_name_getter(
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
pub(super) fn native_id_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    write_reflected_attr(scope, &args, "id", value);
}

/// `className` setter（reflected，spec `dom-classname`）：值 ToString 后 `set_attribute('class', val)`。
pub(super) fn native_class_name_setter(
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

// ── R3155 tabIndex 反射 long 属性（spec HTML `tabindex` content ↔ `tabIndex` IDL long）──
//
// 与 string 反射（id/className/aria*）不同——`tabIndex` 反射 `long`（i32）类型：
// - **getter**：解析 `tabindex` content 属性为整数（HTML「rules for parsing integers」，缺省/非法 → -1）。
//   headless 简化：spec 当元素为原生可聚焦候选时默认返 0、否则 -1，此处统一退化 -1（无原生可聚焦性判定，
//   匹配 `<div>`/`<span>` 等通用元素的实际行为——`document.createElement('div').tabIndex === -1`）。
// - **setter**：值经 V8 ToInt32 强转（spec long setter 经 ToInt32：NaN→0、3.7→3、"12"→12）→
//   写 `tabindex` content 属性为整数字符串。
//
// **a11y 相关**：补充 R3148 焦点工作（focus/blur + activeElement）——`tabIndex` 是程序化控制焦点序的
// 核心属性（FocusManager Tab 导航排序依赖 tabindex 值；0=自然序、正值=显式序、-1=可聚焦但不在 Tab 序）。

/// HTML「rules for parsing integers」简化版：去首尾 ASCII 空白 + 可选 `+`/`-` + 前导 ASCII 数字 → i32。
/// 无数字 / 仅符号 → None；溢出 clamp 至 i32 边界（spec：超 ±2147483647 截断至边界）。遇首个非数字停
/// （spec「collect sequence of ASCII digits」），故 `"12abc"` → `12`。
fn parse_html_integer(s: &str) -> Option<i32> {
    let s = s.trim();
    let bytes = s.as_bytes();
    let (sign, rest) = match bytes.first() {
        Some(b'+') => (1i64, &bytes[1..]),
        Some(b'-') => (-1i64, &bytes[1..]),
        _ => (1i64, bytes),
    };
    if rest.is_empty() || !rest[0].is_ascii_digit() {
        return None;
    }
    let end = rest.iter().position(|b| !b.is_ascii_digit()).unwrap_or(rest.len());
    let num_str = std::str::from_utf8(&rest[..end]).ok()?;
    // i64 解析（避免 usize 前导零 OK）后 clamp 至 i32（spec 溢出截断至 ±2147483647）。
    let n: i64 = num_str.parse().ok()?;
    let n = sign.saturating_mul(n);
    Some(n.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
}

/// `tabIndex` getter（spec HTML `tabIndex`，`tabindex` content 反射 long）：解析 content 属性为整数
/// （HTML 整数解析）；缺省 / 非法 → -1（headless 简化：无原生可聚焦性判定，统一默认 -1）。
pub(super) fn native_tab_index_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let idx = with_dom(|d| d.get_attribute(id, "tabindex"))
        .flatten()
        .and_then(|s| parse_html_integer(&s))
        .unwrap_or(-1);
    rv.set(v8::Integer::new(scope, idx).into());
}

/// `tabIndex` setter：值经 V8 ToInt32 强转（spec long setter）→ 写 `tabindex` content 属性为整数字符串。
/// ToInt32：NaN→0、布尔/字符串→数值（经 JS Number 强转）、3.7→3。
pub(super) fn native_tab_index_setter(
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
    // ToInt32 强转；NaN/undefined → 0（int32_value 返 0）。
    let n = value.int32_value(scope).unwrap_or(0);
    with_dom_mut(|d| d.set_attribute(id, "tabindex", &n.to_string()));
}

// ── R3156 hidden 反射 boolean 属性（spec HTML `hidden` content ↔ `hidden` IDL boolean）──
//
// 第三种反射子类型——`hidden` 反射 `boolean`（与 string id/className/aria*、long tabIndex 均不同）：
// - **getter**：`hidden` content 属性在且值非 `"until-found"` → true（spec：boolean 属性在 = true；
//   `"until-found"` 为独立「hidden until found」状态，IDL getter 返 false）。缺省 → false。
// - **setter**：值经 V8 ToBoolean 强转（spec boolean setter）→ true 设 `hidden` content 属性为 `""`
//   （boolean content 属性空串 = 存在）、false 移除属性。
//
// `el.hidden = true/false` 是现代代码高频（条件显隐组件），且 `hidden` content 属性经 UA 样式表映射
// `display: none`（layout/渲染相关）——native 反射补 spec 合规 boolean 属性面。

/// `hidden` getter（spec HTML `hidden`，content 反射 boolean）：`hidden` 属性在且值非 `"until-found"`
/// → true；缺省 → false。
pub(super) fn native_hidden_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let hidden = with_dom(|d| d.get_attribute(id, "hidden"))
        .flatten()
        .is_some_and(|v| v != "until-found");
    rv.set(v8::Boolean::new(scope, hidden).into());
}

/// `hidden` setter：值经 V8 ToBoolean 强转（spec boolean setter）→ true 设 `hidden` 属性为 `""`、
/// false 移除属性。ToBoolean：`""`/0/NaN/null/undefined → false，余真值 → true。
pub(super) fn native_hidden_setter(
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
    let v = value.boolean_value(scope);
    with_dom_mut(|d| {
        if v {
            d.set_attribute(id, "hidden", ""); // boolean content 属性空串 = 存在
        } else {
            d.remove_attribute(id, "hidden");
        }
    });
}

// ── R3157 通用字符串反射属性（spec HTML title/lang/dir/accessKey 等，IDL 名 = content 名小写）──
//
// title/lang/dir/accessKey 等「IDL 名 = content 属性名（小写）」的字符串反射属性共用一对 getter/setter
//（镜像 R3153 aria* name-dispatch 模式，但映射为 to_ascii_lowercase——accessKey→accesskey，余 identity）。
// 复用既有 [`read_reflected_attr`] / [`write_reflected_attr`]（缺省 ""），零 per-attr 逻辑。

/// `title`/`lang`/`dir`/`accessKey` 等字符串反射 getter：accessor name（IDL 名）经 to_ascii_lowercase →
/// content 属性名 → [`read_reflected_attr`]（缺省 ""）。
pub(super) fn native_string_reflected_getter(
    scope: &mut v8::PinScope,
    name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let attr = name_to_content_attr(scope, name);
    read_reflected_attr(scope, &args, &attr, "", &mut rv);
}

/// 字符串反射 setter：accessor name → 小写 content 名 → [`write_reflected_attr`]（值 ToString）。
pub(super) fn native_string_reflected_setter(
    scope: &mut v8::PinScope,
    name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let attr = name_to_content_attr(scope, name);
    write_reflected_attr(scope, &args, &attr, value);
}

/// accessor name（IDL 名）→ content 属性名（HTML content 属性小写约定，故 to_ascii_lowercase：
/// accessKey→accesskey，title/lang/dir identity）。非 string 名 → 空串（不应发生）。
fn name_to_content_attr(scope: &mut v8::PinScope, name: v8::Local<v8::Name>) -> String {
    if let Ok(s) = v8::Local::<v8::String>::try_from(name) {
        s.to_rust_string_lossy(scope).to_ascii_lowercase()
    } else {
        String::new()
    }
}

// ── R3157 inert 反射 boolean 属性（spec HTML `inert` content ↔ `inert` IDL boolean）──
//
// `inert` 为纯 boolean content 属性（区别 hidden：无 until-found 状态）——getter 存在性判定（属性在 →
// true）、setter ToBoolean 强转 set ""/remove。`el.inert = true` 使整个子树非交互（焦点/编辑/-selection
// 禁用，UA 样式 `pointer-events: none` 等价），a11y/交互高频（modal 对话框背景 inert）。

/// `inert` getter（spec HTML `inert`，content 反射 boolean）：`inert` content 属性在 → true；缺省 → false。
/// 区别 [`native_hidden_getter`]：inert 无 until-found 状态，纯存在性判定。
pub(super) fn native_inert_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let inert = with_dom(|d| d.get_attribute(id, "inert")).flatten().is_some();
    rv.set(v8::Boolean::new(scope, inert).into());
}

/// `inert` setter：值经 V8 ToBoolean 强转（spec boolean setter）→ true 设 `inert` 属性为 `""`、
/// false 移除属性。
pub(super) fn native_inert_setter(
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
    let v = value.boolean_value(scope);
    with_dom_mut(|d| {
        if v {
            d.set_attribute(id, "inert", "");
        } else {
            d.remove_attribute(id, "inert");
        }
    });
}

// ── R3158 draggable 反射 enumerated-boolean 属性（spec HTML `draggable`）──
//
// 第四种反射子类型——`draggable` 为 enumerated content 属性（关键字 `"true"`/`"false"`），IDL 反射为
// `boolean`。区别 pure-boolean（hidden/inert，content 属性为空串存在性）：enumerated content 属性取字面
// `"true"`/`"false"` 值。
// - **getter**：content 属性值 == `"true"` → true；余（`"false"` / 缺省 / 非法）→ false。
//   headless 简化：spec 缺省默认对 `<img>`/`<a·href>` 为 true、余为 false，本实现统一 false（匹配通用
//   `<div>` 等多数元素，同 tabIndex 焦点默认简化）。
// - **setter**：值经 V8 ToBoolean 强转 → 写 content 属性为 `"true"`/`"false"` **字面串**（区别 pure-boolean
//   写空串）——enumerated 属性值是关键字字符串而非存在性。
//
// `el.draggable = true/false` 是 DnD（drag-and-drop）高频；native 反射补 enumerated→boolean 反射类型。

/// `draggable` getter（spec HTML `draggable`，enumerated content 反射 boolean）：content 属性值 == `"true"`
/// → true；余（`"false"` / 缺省 / 非法）→ false。
pub(super) fn native_draggable_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let draggable = with_dom(|d| d.get_attribute(id, "draggable"))
        .flatten()
        .is_some_and(|v| v == "true");
    rv.set(v8::Boolean::new(scope, draggable).into());
}

/// `draggable` setter：值经 V8 ToBoolean 强转 → 写 content 属性为 `"true"`/`"false"` 字面串
///（区别 pure-boolean 写空串——enumerated 属性值为关键字字符串）。
pub(super) fn native_draggable_setter(
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
    let lit = if value.boolean_value(scope) { "true" } else { "false" };
    with_dom_mut(|d| d.set_attribute(id, "draggable", lit));
}

// ── R3153 aria* / role 反射属性（spec WAI-ARIA IDL reflection）──
//
// aria* IDL 属性 ↔ content 属性反射：`el.ariaLabel` ↔ `aria-label`、`el.ariaLabelledBy` ↔
// `aria-labelledby`、`el.role` ↔ `role`。与 camelCase→kebab **不同**：aria 前缀后整体小写单 hyphen
//（`ariaLabelledBy`→`aria-labelledby` 非 `aria-labelled-by`），因 ARIA content 属性名为预定义集合。
// **共用一对 getter/setter**——accessor `name` 参即注册名（ariaLabel/role/...），经 [`idl_to_attr`]
// 转 content 属性名后复用 [`read_reflected_attr`] / [`write_reflected_attr`]（零 per-attr 逻辑重复）。

/// aria*/role IDL 名 → content 属性名：`ariaLabel`→`aria-label`、`ariaLabelledBy`→`aria-labelledby`、
/// `role`→`role`。aria 前缀（4 字符）后跟大写 → `aria-` + 小写(余)；否则原样（role）。
fn idl_to_attr(idl: &str) -> String {
    if idl == "role" {
        return "role".into();
    }
    if let Some(rest) = idl.strip_prefix("aria")
        && rest.chars().next().is_some_and(|c| c.is_ascii_uppercase())
    {
        return format!("aria-{}", rest.to_ascii_lowercase());
    }
    idl.into()
}

/// aria*/role 反射 getter（spec WAI-ARIA IDL reflection）：name 参即 IDL 名（ariaLabel/role/...）→
/// [`idl_to_attr`] 转 content 属性名 → [`read_reflected_attr`]（缺省 `""`）。
pub(super) fn native_aria_reflected_getter(
    scope: &mut v8::PinScope,
    name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let attr = name_to_attr(scope, name);
    read_reflected_attr(scope, &args, &attr, "", &mut rv);
}

/// aria*/role 反射 setter：name 参即 IDL 名 → content 属性名 → [`write_reflected_attr`]（值 ToString）。
pub(super) fn native_aria_reflected_setter(
    scope: &mut v8::PinScope,
    name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let attr = name_to_attr(scope, name);
    write_reflected_attr(scope, &args, &attr, value);
}

/// accessor `name` 参 → IDL 名 → [`idl_to_attr`] content 属性名。非 string 名 → 原样（不应发生）。
fn name_to_attr(scope: &mut v8::PinScope, name: v8::Local<v8::Name>) -> String {
    if let Ok(s) = v8::Local::<v8::String>::try_from(name) {
        idl_to_attr(&s.to_rust_string_lossy(scope))
    } else {
        String::new()
    }
}

// ── 方法回调（Element 上：getAttribute / hasAttribute / setAttribute / removeAttribute）──

/// `getAttribute(name)`：读 internal slot NodeId → `Document::get_attribute`。
/// spec `dom-element-getattribute`：缺省/非 Element → `null`。
pub(super) fn native_get_attribute_invoke(
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
pub(super) fn native_has_attribute_invoke(
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
pub(super) fn native_set_attribute_invoke(
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
pub(super) fn native_remove_attribute_invoke(
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

/// `toggleAttribute(name, force?)`：spec `dom-element-toggleattribute`——切换属性存在性，返切换后是否在。
/// force 缺省：在→移除返 false、不在→设空串 `""` 返 true（toggle 语义）；force 给定：force=true →
/// 确保在（不在则设 `""`）返 true、force=false → 确保移除（在则移除）返 false。
/// 经 [`with_dom_mut`] `has_attribute` + `set_attribute("")` / `remove_attribute`；幂等（已目标态不冗余写）。
/// 注：toggleAttribute 添加属性时值为 `""`（spec），非 `"true"` 或属性名（区别于旧 HTML boolean attr）。
pub(super) fn native_toggle_attribute_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let name = string_arg(scope, &args, 0);
    let force_defined = !args.get(1).is_undefined();
    let force = force_defined && args.get(1).boolean_value(scope);
    let now_present = with_dom_mut(|d| {
        let has = d.has_attribute(id, &name);
        // force 缺省 → toggle（want = !has）；force 给定 → want = force（ensure present/absent）。
        let want = if force_defined { force } else { !has };
        if want {
            if !has {
                d.set_attribute(id, &name, ""); // 添加时值为空串（spec）
            }
            true
        } else {
            if has {
                d.remove_attribute(id, &name);
            }
            false
        }
    })
    .unwrap_or(false);
    rv.set(v8::Boolean::new(scope, now_present).into());
}

/// `hasAttributes()`（spec `dom-element-hasattributes`）：元素是否有任意属性（经
/// `Document::attribute_names` 非空判定）→ `v8::Boolean`。
pub(super) fn native_has_attributes_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let has = with_dom(|d| !d.attribute_names(id).is_empty()).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, has).into());
}

/// `getAttributeNames()`（spec `dom-element-getattributenames`）：元素全部属性名（文档序）→
/// V8 Array of 字符串（空属性集 → 空 Array）。经 `Document::attribute_names`。
pub(super) fn native_get_attribute_names_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        return;
    };
    let names: Vec<String> = with_dom(|d| d.attribute_names(id)).unwrap_or_default();
    let arr = v8::Array::new(scope, names.len() as i32);
    for (i, name) in names.into_iter().enumerate() {
        if let Some(s) = v8::String::new(scope, &name) {
            let _ = arr.set_index(scope, i as u32, s.into());
        }
    }
    rv.set(arr.into());
}

// ── 元素子树作用域查询（element.querySelector(-all)，注册于 Element 模板）──
// 区别于 global `__zw_native_query_selector(-all)`（factories 子模块，R3118）。

/// `element.querySelector(sel)`：spec `dom-parentnode-queryselector`（**元素子树作用域**）——
/// 元素**后代**中首个匹配 → native 对象。区别于文档级 [`factories::native_query_selector_invoke`]：
/// root = `args.this()` 元素 NodeId，且**排除元素自身**（dom `query_selector_all` 含 root 候选，
/// spec descendants-only，镜像 polyfill `query_match_in_subtree` 的 `.filter(|n| *n != root)`）。
///
/// 经 `query_selector_all` + filter + first 取首个后代（比 polyfill `query_selector` + filter 更
/// 正确：后者若元素自身匹配则返 None，本实现继续找首个后代）。
/// OPTIMIZATION: 当前 collect-all-then-first；超大子树可短路（find_first_matching 跳 root）。
/// 无匹配 / 空 / 非法 → `null`；非 native element `this` → `undefined`（getter 一致）。
pub(super) fn native_element_query_selector_invoke(
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
pub(super) fn native_element_query_selector_all_invoke(
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

/// `element.matches(selector)`（spec `dom-element-matches`）：本元素是否匹配选择器 → bool。
/// 复合选择器（tag/`*`/`#id`/`.class`/`[attr]`+运算符/伪类）；组合器不支持（best-effort，
/// 见 `Document::matches`）。非 native element `this` → false（spec matches 仅 Element）。
pub(super) fn native_element_matches_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let m = with_dom(|d| d.matches(id, sel.trim())).unwrap_or(false);
    rv.set(v8::Boolean::new(scope, m).into());
}

/// `element.closest(selector)`（spec `dom-element-closest`）：本元素 + 祖先链首个匹配选择器的节点
/// → native 元素或 `null`。复合选择器（同 [`native_element_matches_invoke`]）；无匹配 / 非法 → `null`。
pub(super) fn native_element_closest_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(id) = read_node_id(scope, &this) else {
        rv.set(v8::null(scope).into());
        return;
    };
    let sel = string_arg(scope, &args, 0);
    let found = with_dom(|d| d.closest(id, sel.trim())).flatten();
    match found {
        Some(fid) => {
            if let Some(obj) = get_or_create_native_element(scope, fid) {
                rv.set(obj.into());
            }
        }
        None => rv.set(v8::null(scope).into()),
    }
}

/// `children` getter（spec `dom-parentnode-children`）：元素**子元素**（跳过文本/注释）
/// → V8 Array of native 对象（文档序）。非 Element 子节点不返（native 仅 Element 对象；
/// `childNodes` 含文本/注释需 native 非 Element 节点，后续切片）。
pub(super) fn native_children_getter(
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

// ── innerHTML / outerHTML 序列化 getter（spec `dom-element-innerhtml` / `-outerhtml`）──

/// `innerHTML` getter（spec `dom-element-innerhtml`）：子节点 `outer_html` 拼接（markup 序列化）。
pub(super) fn native_inner_html_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = with_dom(|d| {
        let mut s = String::new();
        for child in d.child_nodes(id) {
            s.push_str(&d.outer_html(child));
        }
        s
    })
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &html) {
        rv.set(s.into());
    }
}

/// `outerHTML` getter（spec `dom-element-outerhtml`）：本元素 `outer_html`（含自身 tag）。
pub(super) fn native_outer_html_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let html = with_dom(|d| d.outer_html(id)).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &html) {
        rv.set(s.into());
    }
}

// ── innerHTML / outerHTML setter（spec `dom-element-innerhtml` / `-outerhtml`，R3123）──
//
// 闭合 R3113 getter-only 限制：复用 polyfill 路径既有 fragment parse + 子树深拷贝（
// `crate::js_dom_bridge::replace_inner_html` / `replace_outer_html_node`），经 [`with_dom_mut`]
// 写真实 live Document。native 写触发重渲染由 webview `sync_render_after_native_dom`（R3108）
// 拾取——live Document 序列化后与 cached_html 比，变则重绘。
//
// 错误：outerHTML setter 对无父元素（文档根）抛 DOMException 是 spec 行为，但 native 路径尚无
// 异常传递（RFC §3.4 后续）；当前失败记 tracing::warn + no-op（不静默吞，便于排障）。innerHTML
// 极少失败（仅 remove/append 节点级错误）。

/// `innerHTML` setter（spec `dom-element-innerhtml`）：值 ToString 后清空元素现有子节点，
/// 解析 HTML 片段，深拷贝顶层节点追加。复用 [`crate::js_dom_bridge::replace_inner_html`]。
pub(super) fn native_inner_html_setter(
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
    let html = local_value_to_string(scope, value);
    let res = with_dom_mut(|d| crate::js_dom_bridge::replace_inner_html(d, id, &html));
    if let Some(Err(e)) = res {
        tracing::warn!(error = %e, "native innerHTML setter failed");
    }
}

/// `outerHTML` setter（spec `dom-element-outerhtml`）：值 ToString 后把元素整体替换为解析的
/// 片段顶层节点（在父节点中、目标位置前逐个插入，再移除目标自身）。需父节点（文档根无父 →
/// 失败）。复用 [`crate::js_dom_bridge::replace_outer_html_node`]。
pub(super) fn native_outer_html_setter(
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
    let html = local_value_to_string(scope, value);
    let res = with_dom_mut(|d| crate::js_dom_bridge::replace_outer_html_node(d, id, &html));
    if let Some(Err(e)) = res {
        tracing::warn!(error = %e, "native outerHTML setter failed");
    }
}
