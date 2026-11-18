//! R3151 CSSStyleDeclaration（`element.style`）原生绑定——拆自 mod.rs（RFC §3.2 子模块化）。
//!
//! 内聚内联样式面：`style` getter（Element 模板注册）+ CSSStyleDeclaration 模板（cssText(+setter) /
//! length / item / getPropertyValue / setProperty / removeProperty + **named-property-handler** 拦
//! camelCase 动态属性 `el.style.color` / `el.style.backgroundColor`）。internal slot[0] = owner element
//! NodeId；**live**——读经 owner 元素当前 `style` 属性 `parse_style`，mutation 经 `set_attribute("style",`
//! `serialize)` 写回（与 polyfill CSSStyleDeclaration 一致）。
//!
//! named-property-handler（V8 `ObjectTemplate::set_named_property_handler`）：getter/setter/deleter
//! 拦**任意**属性名——camelCase→kebab 转 CSS 属性名（`backgroundColor`→`background-color`），读/写/删
//! owner `style` 属性。cssText/length/item/getPropertyValue/setProperty/removeProperty 等**保留名**返
//! `Intercepted::kNo`（fallthrough 到模板上注册的方法/accessor），避免被动态属性拦截器吞掉。
//!
//! 身份缓存 STYLE_OBJECTS（同元素返同对象，spec identity——polyfill 旧每调新建，native 修正为 spec 合规）；
//! R3134/R3145 同 pattern weak 化（JS 丢引用即 GC）。
//!
//! 可见性：`native_style_getter` / [`build_and_cache_template`] 为 `pub(super)`（mod.rs 的 Element 模板
//! 注册 + install_dom_bindings 调）；余私有。读 `super::read_node_id` / `super::string_arg` /
//! `super::local_value_to_string` + `super::gc::{...}`。

use v8;

use zero_dom::NodeId;

use super::gc::{
    cache_style, cached_style, decode_node_id, encode_node_id, set_style_template, style_template_local, with_dom,
    with_dom_mut,
};
use super::{local_value_to_string, read_node_id, string_arg};

/// named handler 对这些名返 `kNo`（fallthrough）：① CSSStyleDeclaration 模板方法/accessor
///（cssText/length/item/getPropertyValue/setProperty/removeProperty——避免被动态拦截器吞）；② Object/
/// Promise 协议名（constructor/toString/valueOf/then 等——让原型提供真值，避免 `el.style.constructor` 返
/// 空串破坏对象互操作）。其余任意 string 名按 CSS 属性处理（camelCase→kebab，返值或空串）。
const RESERVED: &[&str] = &[
    "cssText",
    "length",
    "item",
    "getPropertyValue",
    "getPropertyPriority",
    "setProperty",
    "removeProperty",
    // Object / Promise 协议名（fallthrough 到原型，保互操作正确）。
    "constructor",
    "toString",
    "toLocaleString",
    "valueOf",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "__proto__",
    "then",
];

/// 从 CSSStyleDeclaration 对象 internal slot[0] 读 owner element NodeId。slot[0]=owner ffi（External 值）。
/// 非 CSD 对象 / stale → `None`。
fn csd_owner(scope: &mut v8::PinScope, obj: &v8::Local<v8::Object>) -> Option<NodeId> {
    let ffi = obj.get_internal_field(scope, 0)?.cast::<v8::External>().value() as usize as u64;
    Some(decode_node_id(ffi))
}

// ── style 属性 parse / serialize（CSSOM § 声明序列化简化）──────────────

/// 一条 CSS 声明（属性名 kebab / 值 / `!important` 标志）。R3171：priority 经 parse→serialize 全程
/// 跟踪——`!important` 从值剥离（CSSOM `getPropertyValue` 返不含 priority 的值，`getPropertyPriority`
/// 单独返 "important"/""），serialize 重新附加。
struct StyleDecl {
    prop: String,
    value: String,
    important: bool,
}

/// 从值串剥离 `!important` priority（`red !important`→(`red`, true)）。`!` 后跟 `important`
///（大小写不敏感，去空白）才视为 priority；否则保留原值（如 content `"a!b"` 不误剥）。
fn split_priority(v: &str) -> (String, bool) {
    if let Some(idx) = v.find('!') {
        let after = v[idx + 1..].trim().to_ascii_lowercase();
        if after == "important" {
            return (v[..idx].trim().to_string(), true);
        }
    }
    (v.trim().to_string(), false)
}

/// 解析 `style` 属性串为有序 `StyleDecl` 列表（去重，**首次位置 + 末次值胜**）。格式 `prop: value; prop2: val2`。
/// 空段 / 无冒号段 / **空值段跳过**；重复 prop **原位更新**值+important（保首次出现位置、末次值胜）。
/// prop/value trim，`!important` 剥离入 `important`。
fn parse_style(s: &str) -> Vec<StyleDecl> {
    let mut out: Vec<StyleDecl> = Vec::new();
    for seg in s.split(';') {
        let Some((k, v)) = seg.split_once(':') else { continue };
        let prop = k.trim().to_ascii_lowercase();
        if prop.is_empty() {
            continue;
        }
        let (value, important) = split_priority(v);
        // R3212：空值声明丢弃（spec「parse a list of declarations」：无值声明 invalid 不入 declarations）。
        // `width:` / `width:  ` 无效——与 R3211 空值 setter 移除对称（set 与 parse 两路均丢空值，行为一致）。
        // 旧保留致 cssText 序列化 dangling `width: ` + length/item 计空值声明（spec 应不计）。
        if value.is_empty() {
            continue;
        }
        // R3213：重复 prop **原位更新**（spec「set a CSS declaration」in-place replace——保首次出现位置、
        // 末次值胜）。旧「去重保首」丢末次值，致 getPropertyValue/cssText 对 duplicate 返首值（spec 应末值）。
        // 实测 `color:red;margin:5px;color:blue` 旧 cssText=`color: red; margin: 5px`（红），spec=`color: blue;
        // margin: 5px`（蓝）。
        if let Some(existing) = out.iter_mut().find(|d| d.prop == prop) {
            existing.value = value;
            existing.important = important;
        } else {
            out.push(StyleDecl { prop, value, important });
        }
    }
    out
}

/// 序列化 `StyleDecl` 列表为 `style` 属性串（`prop: value; prop2: val2`，important 附 ` !important`）。
fn serialize_style(props: &[StyleDecl]) -> String {
    props
        .iter()
        .map(|d| {
            if d.important {
                format!("{}: {} !important", d.prop, d.value)
            } else {
                format!("{}: {}", d.prop, d.value)
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// 读 owner 元素当前 style 声明（live——经 `style` 属性 `parse_style`）。
fn current_props(doc: &zero_dom::Document, id: NodeId) -> Vec<StyleDecl> {
    doc.get_attribute(id, "style")
        .map(|s| parse_style(&s))
        .unwrap_or_default()
}

/// 写回 owner 元素 `style` 属性（空列表 → 移除属性，spec：无声明时 style 属性为空串；dom set_attribute 写空串）。
fn write_props(doc: &mut zero_dom::Document, id: NodeId, props: &[StyleDecl]) {
    let joined = serialize_style(props);
    doc.set_attribute(id, "style", &joined);
}

// ── camelCase ↔ kebab-case 转换（CSSOM 属性名映射简化）────────────────

/// camelCase → kebab-case CSS 属性名（`backgroundColor`→`background-color`）。标准转换：大写前插 '-' + 转小写。
/// 特例 `cssFloat`/`styleFloat` → `float`（spec `float` 暴露为 cssFloat）。
/// **已知限制**：厂商前缀（`webkitTransform` 浏览器→`-webkit-transform`，本实现→`webkit-transform`）不特殊处理
///（headless 现代代码极少用厂商前缀，documented）。
fn camel_to_kebab(name: &str) -> String {
    if name == "cssFloat" || name == "styleFloat" {
        return "float".into();
    }
    let mut out = String::with_capacity(name.len() + 2);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ── css-gaps 刀 B：gap-rule 简写族 inline 通道（css-gaps-1 §rule-shorthand-properties）──
// 写面：简写展开落 longhand（style-system `expand_shorthands` 单源，简写名本身
// 不落盘——Chromium cssText 同形态）；读面：无本名声明时从名单 longhand 重组
// specified 串（段结构对齐 + 初始分量省略）。gCS computed 重组在 js_dom_bridge
// computed_style.rs（used px/rgb 解析域），本块为 specified 原文域。

/// 简写 → 覆盖的 longhand 名单（`rule` 双维度全 6）。shim inline 通道
///（[`gap_rule_shorthand_expand_json`]）同源复用。
fn gap_rule_shorthand_longhands(prop: &str) -> Option<&'static [&'static str]> {
    match prop {
        "column-rule" => Some(&["column-rule-width", "column-rule-style", "column-rule-color"]),
        "row-rule" => Some(&["row-rule-width", "row-rule-style", "row-rule-color"]),
        "rule-width" => Some(&["column-rule-width", "row-rule-width"]),
        "rule-style" => Some(&["column-rule-style", "row-rule-style"]),
        "rule-color" => Some(&["column-rule-color", "row-rule-color"]),
        "rule" => Some(&[
            "column-rule-width",
            "column-rule-style",
            "column-rule-color",
            "row-rule-width",
            "row-rule-style",
            "row-rule-color",
        ]),
        // rule-inset 族（css-gaps-1 §inset，15 简写）：槽位投影展开目标表
        //（style-system expand_rule_inset 同映射，写面名单/读面重组共用）。
        // https://drafts.csswg.org/css-gaps-1/#inset
        "rule-inset" => Some(&[
            "column-rule-inset-cap-start",
            "column-rule-inset-cap-end",
            "column-rule-inset-junction-start",
            "column-rule-inset-junction-end",
            "row-rule-inset-cap-start",
            "row-rule-inset-cap-end",
            "row-rule-inset-junction-start",
            "row-rule-inset-junction-end",
        ]),
        "column-rule-inset" => Some(&[
            "column-rule-inset-cap-start",
            "column-rule-inset-cap-end",
            "column-rule-inset-junction-start",
            "column-rule-inset-junction-end",
        ]),
        "row-rule-inset" => Some(&[
            "row-rule-inset-cap-start",
            "row-rule-inset-cap-end",
            "row-rule-inset-junction-start",
            "row-rule-inset-junction-end",
        ]),
        "rule-inset-cap" => Some(&[
            "column-rule-inset-cap-start",
            "column-rule-inset-cap-end",
            "row-rule-inset-cap-start",
            "row-rule-inset-cap-end",
        ]),
        "column-rule-inset-cap" => Some(&["column-rule-inset-cap-start", "column-rule-inset-cap-end"]),
        "row-rule-inset-cap" => Some(&["row-rule-inset-cap-start", "row-rule-inset-cap-end"]),
        "rule-inset-junction" => Some(&[
            "column-rule-inset-junction-start",
            "column-rule-inset-junction-end",
            "row-rule-inset-junction-start",
            "row-rule-inset-junction-end",
        ]),
        "column-rule-inset-junction" => Some(&["column-rule-inset-junction-start", "column-rule-inset-junction-end"]),
        "row-rule-inset-junction" => Some(&["row-rule-inset-junction-start", "row-rule-inset-junction-end"]),
        "rule-inset-start" => Some(&[
            "column-rule-inset-cap-start",
            "column-rule-inset-junction-start",
            "row-rule-inset-cap-start",
            "row-rule-inset-junction-start",
        ]),
        "column-rule-inset-start" => Some(&["column-rule-inset-cap-start", "column-rule-inset-junction-start"]),
        "row-rule-inset-start" => Some(&["row-rule-inset-cap-start", "row-rule-inset-junction-start"]),
        "rule-inset-end" => Some(&[
            "column-rule-inset-cap-end",
            "column-rule-inset-junction-end",
            "row-rule-inset-cap-end",
            "row-rule-inset-junction-end",
        ]),
        "column-rule-inset-end" => Some(&["column-rule-inset-cap-end", "column-rule-inset-junction-end"]),
        "row-rule-inset-end" => Some(&["row-rule-inset-cap-end", "row-rule-inset-junction-end"]),
        _ => None,
    }
}

/// 简写展开为 inline longhand 声明（非法值 → None——调用方声明忽略；非简写名
/// → None 透传）。shim inline 通道（[`gap_rule_shorthand_expand_json`]）同源复用。
fn gap_rule_expand_decls(prop: &str, val: &str, important: bool) -> Option<Vec<StyleDecl>> {
    gap_rule_shorthand_longhands(prop)?;
    let expanded = zero_style_system::expand_shorthands(&[(prop.to_string(), val.to_string(), important, (0, 0, 0))]);
    if expanded.is_empty() {
        return None;
    }
    Some(
        expanded
            .into_iter()
            .map(|(p, v, imp, _)| StyleDecl {
                prop: p,
                value: v,
                important: imp,
            })
            .collect(),
    )
}

/// rule-inset 族简写名判定（读面重组分流用）。
fn is_rule_inset_shorthand(prop: &str) -> bool {
    matches!(
        prop,
        "rule-inset"
            | "column-rule-inset"
            | "row-rule-inset"
            | "rule-inset-cap"
            | "column-rule-inset-cap"
            | "row-rule-inset-cap"
            | "rule-inset-junction"
            | "column-rule-inset-junction"
            | "row-rule-inset-junction"
            | "rule-inset-start"
            | "column-rule-inset-start"
            | "row-rule-inset-start"
            | "rule-inset-end"
            | "column-rule-inset-end"
            | "row-rule-inset-end"
    )
}

/// 简写读重组：从名单 longhand 声明重组 specified 串。缺任一 longhand / 三维
/// 段结构失配 → None（调用方落空串——Chrome shorthand serialization 同口径）。
/// shim inline 通道（[`gap_rule_shorthand_reconstruct_from_style`]）同源复用。
fn gap_rule_reconstruct(props: &[StyleDecl], prop: &str) -> Option<String> {
    let get = |name: &str| -> Option<String> { props.iter().find(|d| d.prop == name).map(|d| d.value.clone()) };
    let longhands = gap_rule_shorthand_longhands(prop)?;
    // rule-inset 族重组（css-gaps-1 §inset canonical 形，语料 valid 页钉）：
    // 四槽全等 → 单值；否则恒 `cap-start cap-end / junction-start junction-end`
    //（侧内等值不折叠——`10px / 20px` → `10px 10px / 20px 20px`）。槽对简写
    //（cap/junction/start/end）：轴内对等值塌缩后双轴全等 → 该值，否则 None
    //（与 quad 侧内不折叠不同源——轴间失配无可重组形，落空串 Chrome 同口径）。
    // https://drafts.csswg.org/css-gaps-1/#inset
    if is_rule_inset_shorthand(prop) {
        let vals: Vec<String> = longhands.iter().map(|n| get(n)).collect::<Option<Vec<_>>>()?;
        // 四槽重组（全值形）：全等塌缩单值，否则恒全形（侧内等值不折叠）。
        let quad = |cs: &str, ce: &str, js: &str, je: &str| -> String {
            if cs == ce && ce == js && js == je {
                cs.to_string()
            } else {
                format!("{cs} {ce} / {js} {je}")
            }
        };
        return match vals.len() {
            4 => {
                if prop.starts_with("rule-inset-") {
                    // 轴对简写：cap/junction/start/end 双轴——轴内对 (0,1) 与 (2,3)。
                    let pair = |a: &str, b: &str| -> String { if a == b { a.to_string() } else { format!("{a} {b}") } };
                    let c = pair(&vals[0], &vals[1]);
                    let r = pair(&vals[2], &vals[3]);
                    if c == r { Some(c) } else { None }
                } else {
                    // 全值简写：四槽 canonical（全等塌缩 / 恒全形）。
                    Some(quad(&vals[0], &vals[1], &vals[2], &vals[3]))
                }
            }
            8 => {
                // 全值简写 rule-inset（8 longhand 双轴四槽，名单序列四槽+行四槽）：
                // 双轴各四槽重组后全等 → 该值，否则 None（gCS rule-inset 臂跨轴
                // 等值模型同口径，bidirectional-shorthands 页 oracle）。
                let c = quad(&vals[0], &vals[1], &vals[2], &vals[3]);
                let r = quad(&vals[4], &vals[5], &vals[6], &vals[7]);
                if c == r { Some(c) } else { None }
            }
            2 => {
                let (a, b) = (&vals[0], &vals[1]);
                Some(if a == b { a.clone() } else { format!("{a} {b}") })
            }
            _ => None,
        };
    }
    // 维度简写（rule-width/rule-style/rule-color）：双列全等 → 该值。
    if longhands.len() == 2 {
        let (a, b) = (get(longhands[0])?, get(longhands[1])?);
        return if a == b { Some(a) } else { None };
    }
    // 三元组（column-rule/row-rule）或 `rule`（双三元组全等）。
    let triple = |base: &str| -> Option<String> {
        let (w, s, c) = (
            get(&format!("{base}-width"))?,
            get(&format!("{base}-style"))?,
            get(&format!("{base}-color"))?,
        );
        gap_rule_triple_reconstruct(&w, &s, &c)
    };
    if prop == "rule" {
        let r = triple("row-rule")?;
        let c = triple("column-rule")?;
        return if r == c { Some(r) } else { None };
    }
    triple(prop)
}

/// shim inline 通道展开（cssom-1 简写语义，T50/M face 双通道单源——grid
/// `__zw_grid_normalize_decl` 同型）：gap-rule/inset 简写名 → 名单 longhand
/// canonical `(名, 值)` 列表 JSON（裸零折 `0px`、单侧复制、`/` 全形；css-wide
/// 关键字逐 longhand 透传）。空值 → `(名, "")` 列表（shim 侧逐名 remove 语义
/// ——R3211 简写空值 = 名单 longhand 一并清）。非家族简写名 / 展开失败 →
/// 空串（shim 判空回落通用写路径零漂移——position-try split 臂同约定）。
/// priority 由 shim 按 `!important` 后缀逐名回拼，本函数恒返非 important 形。
/// driving: css-gaps 语料 rule-inset-shorthand / gap-decorations-rule-shorthand
/// inline roundtrip 页（wptfyi-css 刀 D inline 站）。
/// https://drafts.csswg.org/cssom-1/#serialize-a-css-declaration
pub(crate) fn gap_rule_shorthand_expand_json(prop: &str, value: &str) -> String {
    let value = value.trim();
    // 刀 TS：transition 家族共用本站（shim T50 写面 miss 兜底链任意名直达，
    // 本站内分派——font 臂回调内分派先例同型，免共享文件 callbacks.rs 编辑）。
    if prop.eq_ignore_ascii_case("transition") {
        return transition_shorthand_expand_json(value);
    }
    if let Some(longhands) = gap_rule_shorthand_longhands(prop) {
        if value.is_empty() {
            // remove 语义：名单 longhand 逐名清（值空）。
            let pairs: Vec<(String, &str)> = longhands.iter().map(|n| ((*n).to_string(), "")).collect();
            return serde_json::to_string(&pairs).unwrap_or_default();
        }
        if let Some(decls) = gap_rule_expand_decls(prop, value, false) {
            let pairs: Vec<(String, String)> = decls.into_iter().map(|d| (d.prop, d.value)).collect();
            return serde_json::to_string(&pairs).unwrap_or_default();
        }
    }
    String::new()
}

/// shim inline 通道读重组（[`gap_rule_shorthand_expand_json`] 对偶面）：从
/// style 声明串（`n: v; n2: v2` 形，attribute 原文域）重组家族简写 specified
/// 串。名比对 ASCII case-insensitive（attr 源名未归一形兜底）；值串剥
/// `!important`（重组为值等值模型，priority 不参与比对）。非家族简写名 /
/// 名单 longhand 缺任一 / 不可重组 → 空串（shim 判空回落空值语义）。
/// style attribute 原文域（`n: v; n2: v2` 形）→ 声明列表（名 ASCII lowercase、
/// 值剥 `!important`——gap/transition 两族读重组站共享解析）。
fn style_attr_decls(raw: &str) -> Vec<StyleDecl> {
    raw.split(';')
        .filter_map(|seg| {
            let seg = seg.trim();
            if seg.is_empty() {
                return None;
            }
            let (name, val) = seg.split_once(':')?;
            let name = name.trim();
            if name.is_empty() {
                return None;
            }
            let (core, _important) = split_priority(val.trim());
            Some(StyleDecl {
                prop: name.to_ascii_lowercase(),
                value: core,
                important: false,
            })
        })
        .collect()
}

pub(crate) fn gap_rule_shorthand_reconstruct_from_style(raw: &str, prop: &str) -> String {
    // 刀 TS：transition 家族读重组共用本站（shim readProp 通用 miss 兜底任意名
    // 直达——raw attr `n: v; ...` 域解析与 gap 臂同源）。家族名简写 raw 存储
    // 形（attr 车道原文）与展开存储形（CSSOM 写面展开落库）双兼容，
    // [`ts_family_value`] 单源裁决。
    if prop.eq_ignore_ascii_case("transition") {
        return ts_family_value(&style_attr_decls(raw), "transition").unwrap_or_default();
    }
    if gap_rule_shorthand_longhands(prop).is_none() {
        return String::new();
    }
    let decls: Vec<StyleDecl> = style_attr_decls(raw);
    gap_rule_reconstruct(&decls, prop).unwrap_or_default()
}

/// 刀 N 三期：font 简写 inline specified 读重组（css-fonts-4 §font-prop
/// serialization 面——shorthand getter 串化）。写面原文落库单点不变（
/// raw `font:` 声明 + 旁路 longhand 覆写声明共存），本函数从 style
/// attribute 原文域整体裁决：
/// - system font 关键字（caption 族）与整条 CSS-wide keyword 声明原样回显
///   （specified 面保留关键字形态，Blink 同形）。
/// - 位解析走 style-system [`zero_style_system::parse_font_shorthand_parts`]
///   文法单源（刀 N 三期抽出——与 expand_font 展开判定同源）。
/// - 旁路 longhand 覆写合并（core 位替换；`font-variant` 简写覆写按位
///   解释）；CSS-wide keyword 落在任一 core 覆写 → 空串（specified 面防逐出）。
/// - 防逐出门（简写不可表达 → 空串）：reset 长手非初始（size-adjust/
///   kerning/ligatures/position/numeric/alternates/east-asian/feature-settings/
///   variation-settings/language-override/optical-sizing/emoji/synthesis）、
///   variant 位超 CSS2 子集（仅 normal|small-caps 可串化）、stretch 位
///   非关键字（% 形简写位不收——css-fonts-4 §font-width-prop）。
/// - canonical 序 = style variant-caps weight stretch size [/ lh] family
///   （normal 位省略；非 normal lh 用 ` / ` 间隔——font-valid.html canonical
///   面驱动；序列化顺序与 [font_shorthand_to_css] computed 面一致）。
///
/// 非 font 名由调用方分派；无 `font` 声明 / 文法 invalid / 防逐出命中 →
/// 空串（shim 判空按 spec 空值语义直返）。
/// <https://drafts.csswg.org/css-fonts-4/#font-prop>
pub(crate) fn font_shorthand_reconstruct_from_style(raw: &str) -> String {
    // 1. 声明串解析（`n: v; n2: v2` attr 原文域；名 lowercase 兜底未归一源形；
    //    值剥 `!important`（priority 不参与位模型）；last-wins（R3213 同口径）。
    //    刀 N 三期 b2：段序仲裁——attr 串序即声明序，font 之后的覆写才生效
    //    （CSSOM 简写 set 复位子属性：font 前的旧长手声明被简写覆写屏蔽；
    //    font 之后的长手覆写合法获胜）。驱动：test_font_family_parsing.html
    //    102F——fontFamily setter 先行落库的旧 font-family 覆盖简写位。
    let mut font_value: Option<String> = None;
    let mut font_idx: usize = usize::MAX;
    let mut overrides: Vec<(String, String, usize)> = Vec::new();
    for (idx, seg) in raw.split(';').enumerate() {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        let Some((name, val)) = seg.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        let (core, _) = split_priority(val.trim());
        if name == "font" {
            font_value = Some(core);
            font_idx = idx;
        } else {
            overrides.push((name, core, idx));
        }
    }
    let Some(font_value) = font_value else {
        return String::new();
    };
    let lower = font_value.to_ascii_lowercase();
    // system font 关键字：原样回显。
    const SYSTEM_FONTS: [&str; 6] = ["caption", "icon", "menu", "message-box", "small-caption", "status-bar"];
    if SYSTEM_FONTS.contains(&lower.as_str()) {
        return font_value;
    }
    // 整条 CSS-wide keyword 声明：原样回显（cssom-1 §serialize 惯例）。
    if matches!(
        lower.as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
    ) {
        return font_value;
    }
    // 2. 位解析（style-system 文法单源）。
    let mut bits = match zero_style_system::parse_font_shorthand_parts(&font_value) {
        Some(bits) => bits,
        None => return String::new(),
    };
    // 3. 旁路覆写合并 + 防逐出门（仅 font 声明之后的段参与；段序仲裁见 1）。
    let mut ligatures_nonnormal = false;
    for (name, value, idx) in &overrides {
        if *idx <= font_idx {
            continue;
        }
        let vl = value.to_ascii_lowercase();
        // CSS-wide keyword 落任一覆写 → specified 面不可串化（css-cascade-5
        // §css-wide-keywords：shorthand getter 无关键字整体形可表达）。
        if matches!(
            vl.as_str(),
            "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
        ) {
            return String::new();
        }
        match name.as_str() {
            // core 位替换（verbatim declared 文本）。
            "font-style" => bits.style = value.clone(),
            "font-weight" => bits.weight = value.clone(),
            "font-size" => bits.size = value.clone(),
            "line-height" => bits.line_height = value.clone(),
            "font-family" => bits.family = value.clone(),
            "font-variant-caps" => bits.variant_caps = Some(value.clone()),
            "font-stretch" => bits.stretch = Some(value.clone()),
            // `font-variant` 简写覆写按位解释（css-fonts-4 §font-variant-prop）：
            // normal = 全 normal；none = ligatures none；单 token caps 位 →
            // variant 位；其余复合形态简写不可表达 → 空串。
            "font-variant" => {
                if vl == "none" {
                    ligatures_nonnormal = true;
                } else if vl != "normal" {
                    let caps_table = [
                        "small-caps",
                        "all-small-caps",
                        "petite-caps",
                        "all-petite-caps",
                        "unicase",
                        "titling-caps",
                    ];
                    if !caps_table.contains(&vl.as_str()) {
                        return String::new();
                    }
                    bits.variant_caps = Some(value.clone());
                }
            }
            // reset 长手非初始 → 防逐出（css-fonts-4 §font-prop serialization：
            // 简写不可表达的长手值阻止 shorthand getter 串化）。
            "font-size-adjust" if vl != "none" => return String::new(),
            "font-kerning" if vl != "auto" => return String::new(),
            "font-variant-ligatures" if vl != "normal" => return String::new(),
            "font-variant-position"
            | "font-variant-numeric"
            | "font-variant-alternates"
            | "font-variant-east-asian"
            | "font-variant-emoji"
            | "font-feature-settings"
            | "font-variation-settings"
            | "font-language-override"
            | "font-synthesis"
            | "font-synthesis-weight"
            | "font-synthesis-style"
            | "font-synthesis-small-caps"
            | "font-synthesis-position"
                if vl != "normal" && vl != "auto" =>
            {
                return String::new();
            }
            "font-optical-sizing" if vl != "auto" => return String::new(),
            _ => {}
        }
    }
    if ligatures_nonnormal {
        return String::new();
    }
    // 4. 位门：variant 位 CSS2 子集（css-fonts-4 §font-prop <font-variant-css21>）
    //    与 stretch 位关键字（九关键字；% 形不可串化）。
    if let Some(caps) = &bits.variant_caps
        && !matches!(caps.to_ascii_lowercase().as_str(), "normal" | "small-caps")
    {
        return String::new();
    }
    if let Some(stretch) = &bits.stretch
        && !matches!(
            stretch.to_ascii_lowercase().as_str(),
            "normal"
                | "ultra-condensed"
                | "extra-condensed"
                | "condensed"
                | "semi-condensed"
                | "semi-expanded"
                | "expanded"
                | "extra-expanded"
                | "ultra-expanded"
        )
    {
        return String::new();
    }
    // 5. canonical 组合（normal 位省略；declared 原文 verbatim）。
    let mut parts: Vec<String> = Vec::new();
    if bits.style != "normal" {
        parts.push(bits.style);
    }
    if let Some(caps) = &bits.variant_caps
        && caps != "normal"
    {
        parts.push(caps.clone());
    }
    if bits.weight != "normal" {
        parts.push(bits.weight);
    }
    if let Some(stretch) = &bits.stretch
        && stretch != "normal"
    {
        parts.push(stretch.clone());
    }
    if bits.line_height != "normal" {
        parts.push(format!("{} / {}", bits.size, bits.line_height));
    } else {
        parts.push(bits.size);
    }
    parts.push(bits.family);
    parts.join(" ")
}

/// 三维段联合重组：逐维 top-level 逗号拆段、段数全等才联合；段结构（Plain /
/// repeat head+循环长）逐段一致；单段形初始分量省略（style none / color
/// currentcolor / width medium——其余全省时保 width 锚，语料 valid 页钉）。
fn gap_rule_triple_reconstruct(w: &str, s: &str, c: &str) -> Option<String> {
    let ws = zero_css_parser::values::split_top_level_commas(w)?;
    let ss = zero_css_parser::values::split_top_level_commas(s)?;
    let cs = zero_css_parser::values::split_top_level_commas(c)?;
    if ws.len() != ss.len() || ws.len() != cs.len() {
        return None;
    }
    // 文法校验（style attribute 长手声明未过写面校验门——垃圾联合拒收）。
    if zero_css_parser::values::parse_rule_width_list(w).is_none()
        || zero_css_parser::values::parse_rule_style_list(s).is_none()
        || zero_css_parser::values::parse_rule_color_list(c).is_none()
    {
        return None;
    }
    let items: Option<Vec<String>> = (0..ws.len())
        .map(|i| gap_rule_joint_item(ws[i], ss[i], cs[i]))
        .collect();
    Some(items?.join(", "))
}

/// repeat 段拆解（head 小写 + 循环体 top-level 逗号列表）；非 repeat 形 → None。
fn gap_rule_repeat_parts(t: &str) -> Option<(String, Vec<&str>)> {
    let lower = t.to_ascii_lowercase();
    if !(lower.starts_with("repeat(") && lower.ends_with(')')) {
        return None;
    }
    let inner = &t[7..t.len() - 1];
    let (head, body) = inner.split_once(',')?;
    Some((
        head.trim().to_ascii_lowercase(),
        zero_css_parser::values::split_top_level_commas(body)?,
    ))
}

/// 段联合：repeat 段三维 head/循环长一致才逐位联合；Plain 段单点联合。
fn gap_rule_joint_item(w: &str, s: &str, c: &str) -> Option<String> {
    if gap_rule_repeat_parts(w).is_some() || gap_rule_repeat_parts(s).is_some() || gap_rule_repeat_parts(c).is_some() {
        let (wh, wb) = gap_rule_repeat_parts(w)?;
        let (sh, sb) = gap_rule_repeat_parts(s)?;
        let (ch, cb) = gap_rule_repeat_parts(c)?;
        if wh != sh || wh != ch || wb.len() != sb.len() || wb.len() != cb.len() {
            return None;
        }
        let items: Option<Vec<String>> = (0..wb.len())
            .map(|k| gap_rule_joint_tokens(wb[k], sb[k], cb[k]))
            .collect();
        return Some(format!("repeat({}, {})", wh, items?.join(", ")));
    }
    gap_rule_joint_tokens(w.trim(), s.trim(), c.trim())
}

/// 单点联合（初始分量省略；全省保 width 锚——`currentcolor none medium`→
/// `medium`，语料 valid 页钉）。
fn gap_rule_joint_tokens(w: &str, s: &str, c: &str) -> Option<String> {
    let style_initial = s.eq_ignore_ascii_case("none");
    let color_initial = c.eq_ignore_ascii_case("currentcolor");
    let width_initial = w.eq_ignore_ascii_case("medium");
    let mut tokens: Vec<&str> = Vec::with_capacity(3);
    if !width_initial || (style_initial && color_initial) {
        tokens.push(w);
    }
    if !style_initial {
        tokens.push(s);
    }
    if !color_initial {
        tokens.push(c);
    }
    if tokens.is_empty() {
        return None;
    }
    Some(tokens.join(" "))
}

// ── `style` getter：返 owner 元素的 CSSStyleDeclaration（缓存保身份）──

/// `style` getter（spec `dom-element-style`）：返 owner 元素的 CSSStyleDeclaration（缓存保身份）。
pub(super) fn native_style_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(id) = read_node_id(scope, &holder) else {
        return;
    };
    let ffi = encode_node_id(id);
    // 缓存命中 → 同对象（spec `el.style === el.style`）。
    if let Some(cached) = cached_style(scope, ffi) {
        rv.set(cached.into());
        return;
    }
    let Some(tmpl) = style_template_local(scope) else {
        return;
    };
    let Some(obj) = tmpl.new_instance(scope) else {
        return;
    };
    let ptr = ffi as usize as *mut std::ffi::c_void;
    let external = v8::External::new(scope, ptr);
    let _ = obj.set_internal_field(0, external.into());
    cache_style(scope, ffi, obj);
    rv.set(obj.into());
}

// ── 刀 TS：transition 简写族 CSSOM 车道（写面展开落 longhand + 读面 canonical 重组）──

/// transition 简写所控 longhand 名单（css-transitions-1 §transition-property-property /
/// §transition-duration-property / §transition-timing-function-property /
/// §transition-delay-property + css-transitions-2 §transition-behavior-property）。
const TRANSITION_LONGHANDS: [&str; 5] = [
    "transition-property",
    "transition-duration",
    "transition-timing-function",
    "transition-delay",
    "transition-behavior",
];

/// css-wide keyword 判定（CSS Cascading-4 §7.3；本文件局部面用）。
fn ts_is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

/// transition 简写展开落 longhand（style-system `expand_shorthands` 单源，同
/// [`gap_rule_expand_decls`] 形态）。展开空（`asdf display` 类非法形 / 重复
/// 组件）→ `None`，调用方声明忽略（spec setProperty 语义）。
fn ts_expand_decls(val: &str, important: bool) -> Option<Vec<StyleDecl>> {
    let expanded =
        zero_style_system::expand_shorthands(&[("transition".to_string(), val.to_string(), important, (0, 0, 0))]);
    if expanded.is_empty() {
        return None;
    }
    Some(
        expanded
            .into_iter()
            .map(|(p, v, imp, _)| StyleDecl {
                prop: p,
                value: v,
                important: imp,
            })
            .collect(),
    )
}

/// 顶层逗号切分（paren-aware：cubic-bezier(...)/steps(...) 内部逗号保持一体；
/// transition.rs 同款本地面）。
fn ts_split_layers(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    parts.push(current.trim().to_string());
    parts
}

fn ts_gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { ts_gcd(b, a % b) }
}

/// transition 简写 canonical 重组（css-transitions-1 §serializing-shorthand-values）：
/// 每层 `<property> <duration> <timing> <delay> <behavior>`，等于初值的组件省略；
/// 例外——delay 非零时 duration 即使 0s 也强制输出（消歧两个 `<time>` 位）；
/// property 初值 `all` 同样省略，但当层全初值时保留（重组非空底座，"all"
/// /"all, all" 形不减——WPT transition-valid 期望串锚定："1s"→"1s" 非
/// "all 1s"、"all 1s"→"1s"）。
/// 五 longhand 全等 css-wide keyword（简写 inherit 等展开存储形）→ 原样回。
fn ts_serialize_from_longhands(props: &[StyleDecl]) -> Option<String> {
    let get = |name: &str| {
        props
            .iter()
            .find(|d| d.prop == name)
            .map(|d| d.value.trim().to_string())
    };
    if TRANSITION_LONGHANDS.iter().all(|n| get(n).is_none()) {
        return None;
    }
    let p = get("transition-property").unwrap_or_else(|| "all".into());
    let d = get("transition-duration").unwrap_or_else(|| "0s".into());
    let t = get("transition-timing-function").unwrap_or_else(|| "ease".into());
    let de = get("transition-delay").unwrap_or_else(|| "0s".into());
    let b = get("transition-behavior").unwrap_or_else(|| "normal".into());
    let first = p.clone();
    if [d.as_str(), t.as_str(), de.as_str(), b.as_str()]
        .iter()
        .all(|v| *v == first.as_str())
        && ts_is_css_wide_keyword(&first)
    {
        return Some(first);
    }
    let (pl, dl, tl, del, bl) = (
        ts_split_layers(&p),
        ts_split_layers(&d),
        ts_split_layers(&t),
        ts_split_layers(&de),
        ts_split_layers(&b),
    );
    // 逐层序列化：下标按各名单长度取模环绕（spec 列表 LCM 展开的最小实现——
    // 各名单等长（简写展开形）时即逐层直取）。
    let mut n = 1usize;
    for len in [pl.len(), dl.len(), tl.len(), del.len(), bl.len()] {
        n = n / ts_gcd(n, len) * len;
    }
    let layers = (0..n)
        .map(|i| {
            let pi = &pl[i % pl.len()];
            let di = &dl[i % dl.len()];
            let ti = &tl[i % tl.len()];
            let dei = &del[i % del.len()];
            let bi = &bl[i % bl.len()];
            let pi_init = pi.eq_ignore_ascii_case("all");
            let di_init = di.eq_ignore_ascii_case("0s");
            let ti_init = ti.eq_ignore_ascii_case("ease");
            let dei_init = dei.eq_ignore_ascii_case("0s");
            let bi_init = bi.eq_ignore_ascii_case("normal");
            // 初值组件一律省略（css-transitions-1 §serializing-shorthand-values；
            // WPT transition-valid 实测 "all 1s"→"1s"、"1s -3s"→"1s -3s"）：property
            // 初值 all 同省，唯全层皆初值时保留 all——空串不是合法 <single-transition>。
            if pi_init && di_init && ti_init && dei_init && bi_init {
                return pi.clone();
            }
            let mut parts: Vec<&str> = Vec::with_capacity(5);
            if !pi_init {
                parts.push(pi.as_str());
            }
            if !di_init || !dei_init {
                // duration 非初值，或 delay 非初值时强制输出（两位 <time> 消歧）
                parts.push(di.as_str());
            }
            if !ti_init {
                parts.push(ti.as_str());
            }
            if !dei_init {
                parts.push(dei.as_str());
            }
            if !bi_init {
                parts.push(bi.as_str());
            }
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(layers)
}

/// transition 简写族读面派生：简写名 → canonical 重组（raw 存储形优先展开同形，
/// css-wide keyword 原样）；longhand 名 → 直查优先，本名落空且简写声明在时按
/// 层序展开提取（spec：简写置后遮蔽 longhand，longhand 置后直查命中）。非本族
/// 名 → `None` 调用方走通用臂。
fn ts_family_value(props: &[StyleDecl], prop: &str) -> Option<String> {
    let raw = props.iter().find(|d| d.prop == "transition");
    if prop == "transition" {
        if let Some(raw) = raw {
            if ts_is_css_wide_keyword(raw.value.trim()) {
                return Some(raw.value.clone());
            }
            // inline attr 车道（parse_style）raw 简写存储形 → 展开 canonical 重组
            //（与 CSSOM 展开存储车道读回同形；attr 车道引擎侧级联展开不受此影响）。
            let expanded = ts_expand_decls(raw.value.trim(), false)?;
            return ts_serialize_from_longhands(&expanded);
        }
        return ts_serialize_from_longhands(props);
    }
    let idx = TRANSITION_LONGHANDS.iter().position(|&n| n == prop)?;
    if let Some(found) = props.iter().find(|d| d.prop == prop) {
        return Some(found.value.clone());
    }
    // longhand 直查落空 → raw 简写声明遮蔽展开按层提取。
    let raw = raw?;
    let expanded = ts_expand_decls(raw.value.trim(), false)?;
    expanded.get(idx).map(|d| d.value.clone())
}

/// transition 简写家族 shim inline 通道展开（cssom-1 §dom-cssstyledeclaration-
/// setproperty 简写语义 + css-transitions-1 §transition-shorthand-property，
/// [`gap_rule_shorthand_expand_json`] 站同约定：空串 = 非家族回落通用写路径）。
/// - 空值 → 名单 longhand 逐名清声明（R3211 remove 语义，gap 站同形 JSON）；
/// - 非法形（[`ts_expand_decls`] 空，"asdf display" 类）→ `[["transition",""]]`
///   自清形——简写键本车道不落声明位（展开臂先清后落），清键即无残留，净效果
///   ＝ spec 声明忽略（invalid set 留原声明不动：单声明位净空两形态等价）；
/// - 合法 → (名, canonical 值) 名单 longhand JSON（shim 逐名落库，简写键清除）。
///
/// 宿主回调 `__zw_style_shorthand_expand`（callbacks.rs 站）经
/// [`gap_rule_shorthand_expand_json`] 分派消费。
pub(crate) fn transition_shorthand_expand_json(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        let pairs: Vec<(String, &str)> = TRANSITION_LONGHANDS.iter().map(|n| ((*n).to_string(), "")).collect();
        return serde_json::to_string(&pairs).unwrap_or_default();
    }
    if let Some(decls) = ts_expand_decls(value, false) {
        let pairs: Vec<(String, String)> = decls.into_iter().map(|d| (d.prop, d.value)).collect();
        return serde_json::to_string(&pairs).unwrap_or_default();
    }
    serde_json::to_string(&[("transition".to_string(), "")]).unwrap_or_default()
}

// ── named-property-handler：拦 camelCase 动态属性 ──────────────────────

/// reserved 名（模板方法/accessor）→ `kNo`（fallthrough）；非 string 名 → `kNo`。否则返该 CSS 属性值
///（camelCase→kebab，读 owner `style`，未设返空串）。**恒拦截**（CSSStyleDeclaration 对任意名返值）。
fn native_style_named_getter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo; // Symbol 等非 string → fallthrough
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo; // 模板方法/accessor → fallthrough
    }
    let prop = camel_to_kebab(&name);
    let val = with_dom(|d| {
        let props = current_props(d, owner);
        // 刀 TS：transition 简写族读面（简写 canonical 重组 / longhand 遮蔽派生）。
        if let Some(v) = ts_family_value(&props, &prop) {
            return Some(v);
        }
        if let Some(found) = props.iter().find(|d| d.prop == prop) {
            return Some(found.value.clone());
        }
        // css-gaps 刀 B：简写名无本名声明时从名单 longhand 重组 specified 串。
        gap_rule_shorthand_longhands(&prop).and_then(|_| gap_rule_reconstruct(&props, &prop))
    })
    .flatten()
    .unwrap_or_default();
    if let Some(vs) = v8::String::new(scope, &val) {
        rv.set(vs.into());
    }
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`；否则 camelCase→kebab 写 owner `style` 属性（upsert + 重新序列化）。
fn native_style_named_setter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let prop = camel_to_kebab(&name);
    let val = local_value_to_string(scope, value);
    // R3211：空值语义 = 移除声明（spec IDL setter ≡ setProperty(prop, value) 无 priority；value 空串 →
    // removeProperty）。`el.style.display = ''` 是 reset inline 样式事实标准高频用法——旧实现 upsert 空值
    // 致 dangling `prop: `（与 setProperty / polyfill merge_style_property 三处同病，本片对称闭合）。
    // css-anchor-1 canonical serialization 与 setProperty 同款（自定义属性原样保留）。
    // grid 家族声明值 CSSOM 校验 + canonical 序列化（grid_cssom 模块头注）。
    let val = if prop.starts_with("--") || val.trim().is_empty() {
        val
    } else if !zero_style_system::validate_decl_value(&prop, val.trim()) {
        // T54（cssom-1 §CSSStyleDeclaration 属性写 + css-values-4 文法校验）：白名单
        // 属性（transform 族/数值槽——[`zero_style_system::validate_decl_value`] 裁决，
        // 表外属性/shorthand/css-wide keyword 放行）非法值整体拒绝——赋值忽略、已有
        // 声明保留（spec setProperty 语义，同下方 grid 非法值臂同形态）。WPT css-values
        // acos-asin-atan-atan2-invalid 63 子测试以 `rotate(asin())` 形态驱动（wptfyi-css
        // DC-2 values trig 桶）。
        return v8::Intercepted::kYes;
    } else if gap_rule_shorthand_longhands(&prop).is_some() {
        // css-gaps 刀 B：简写展开落 longhand（非法值已被 validate 臂拒；简写名
        // 本身与名单旧声明一并让位——upsert 语义，Chromium cssText 同形态）。
        let Some(expanded) = gap_rule_expand_decls(&prop, val.trim(), false) else {
            return v8::Intercepted::kYes;
        };
        with_dom_mut(|d| {
            let mut props = current_props(d, owner);
            props.retain(|d| d.prop != prop);
            if let Some(longhands) = gap_rule_shorthand_longhands(&prop) {
                props.retain(|d| !longhands.contains(&d.prop.as_str()));
            }
            props.extend(expanded);
            write_props(d, owner, &props);
        });
        return v8::Intercepted::kYes;
    } else if prop == "transition" {
        // 刀 TS：transition 简写展开落 longhand（[`ts_expand_decls`] 单源，同
        // css-gaps 刀 B 形态——简写名与其所控 longhand 旧声明一并让位）；非法值
        //（"asdf display" 类展开空）→ 声明忽略（spec setProperty 语义）。
        let Some(expanded) = ts_expand_decls(val.trim(), false) else {
            return v8::Intercepted::kYes;
        };
        with_dom_mut(|d| {
            let mut props = current_props(d, owner);
            props.retain(|d| d.prop != prop);
            props.retain(|d| !TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
            props.extend(expanded);
            write_props(d, owner, &props);
        });
        return v8::Intercepted::kYes;
    } else if super::grid_cssom::is_grid_family_prop(&prop) {
        match super::grid_cssom::canonicalize_grid_decl(&prop, val.trim()) {
            Some(v) => v,
            // 非法 grid 值 → 声明忽略（spec setProperty 语义），已有声明保留。
            None => return v8::Intercepted::kYes,
        }
    } else if prop == "position-area" {
        match super::position_area::canonicalize_decl(val.trim()) {
            Some(v) => v,
            // 非法 position-area → 声明忽略（css-anchor-position-1，语料
            // position-area-parsing invalid 矩阵），已有声明保留。
            None => return v8::Intercepted::kYes,
        }
    } else if let Some(folded) = zero_css_parser::values::fold_decl_specified(&prop, val.trim()) {
        // r2320（CSS Values 4 §10.11）：数学函数 specified 值常量折叠——
        // `rotate(acos(1))` 存为 `rotate(calc(0deg))`（WPT css-values
        // serialize 桶：sin-cos-tan-serialize / acos-asin-atan-atan2-serialize
        // 以 specified 读回等价断言驱动；无折叠点返 None 走原值臂）。
        // g41：background-image 渐变 canonical 串化在 fold_decl_specified
        // 分派内单点接线（写面四站 + mutation replay 三站共享）。
        folded
    } else {
        zero_css_parser::values::canonicalize_anchor_functions(val.trim())
    };
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        if val.trim().is_empty() {
            props.retain(|d| d.prop != prop);
            // css-gaps 刀 B：简写空值 = 名单 longhand 一并复位（spec 简写 removeProperty
            // 覆盖其所控 longhand）。
            if let Some(longhands) = gap_rule_shorthand_longhands(&prop) {
                props.retain(|d| !longhands.contains(&d.prop.as_str()));
            }
            // 刀 TS：transition 简写空值 = 所控 longhand 一并复位（同上形态）。
            if prop == "transition" {
                props.retain(|d| !TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
            }
        } else if let Some(decl) = props.iter_mut().find(|d| d.prop == prop) {
            // upsert（已存在→更新值）；named setter 等价 setProperty(prop, value) 无 priority → 重置 important。
            decl.value = val.clone();
            decl.important = false;
        } else {
            props.push(StyleDecl {
                prop,
                value: val,
                important: false,
            });
        }
        write_props(d, owner, &props);
    });
    v8::Intercepted::kYes
}

/// reserved / 非 string → `kNo`；否则 camelCase→kebab 移除 owner `style` 该属性（重序列化）。返 true（已删）。
fn native_style_named_deleter(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Boolean>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let prop = camel_to_kebab(&name);
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        props.retain(|d| d.prop != prop);
        write_props(d, owner, &props);
    });
    rv.set_bool(true);
    v8::Intercepted::kYes
}

/// `in` 动态检查（spec supported property names）：引擎支持面内属性名返
/// `kYes` + 无 attribute（`'grid-template-columns' in el.style` → true）；
/// reserved 名与非 CSD fallthrough（`in` false）。缺 query 回调时 V8 对
/// handler 名恒答 false——grid-layout-properties 等语料
/// `assert_true(prop in style)` 全挂的主因。
/// 支持面清单见 grid_cssom::SUPPORTED_STYLE_PROPS（枚举器/length 面另账）。
fn native_style_named_query(
    scope: &mut v8::PinScope,
    key: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Integer>,
) -> v8::Intercepted {
    let holder = args.holder();
    let Some(_) = csd_owner(scope, &holder) else {
        return v8::Intercepted::kNo;
    };
    let Ok(s) = v8::Local::<v8::String>::try_from(key) else {
        return v8::Intercepted::kNo;
    };
    let name = s.to_rust_string_lossy(scope);
    if RESERVED.contains(&name.as_str()) {
        return v8::Intercepted::kNo;
    }
    let prop = camel_to_kebab(&name);
    let supported = prop.starts_with("--") || super::grid_cssom::SUPPORTED_STYLE_PROPS.contains(&prop.as_str());
    if supported {
        rv.set(v8::Integer::new(scope, 0));
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

// ── cssText / length / item / getPropertyValue / setProperty / removeProperty ──

/// `cssText` getter（spec `dom-cssstyledeclaration-csstext`）：owner `style` 属性的规范化序列化（parse→serialize）。
fn native_style_css_text_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return;
    };
    let text = with_dom(|d| serialize_style(&current_props(d, owner))).unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &text) {
        rv.set(s.into());
    }
}

/// `cssText` setter（spec）：值 ToString → `set_attribute("style", val)`（整体替换，无解析校验）。
fn native_style_css_text_setter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    value: v8::Local<v8::Value>,
    args: v8::PropertyCallbackArguments,
    _rv: v8::ReturnValue<()>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        return;
    };
    let val = local_value_to_string(scope, value);
    with_dom_mut(|d| d.set_attribute(owner, "style", &val));
}

/// `length` getter（spec）：owner style 声明数（去重后唯一属性数）。
fn native_style_length_getter(
    scope: &mut v8::PinScope,
    _name: v8::Local<v8::Name>,
    args: v8::PropertyCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let holder = args.holder();
    let Some(owner) = csd_owner(scope, &holder) else {
        rv.set(v8::Integer::new(scope, 0).into());
        return;
    };
    let len = with_dom(|d| current_props(d, owner).len()).unwrap_or(0);
    rv.set(v8::Integer::new(scope, len as i32).into());
}

/// `item(index)`（spec `dom-cssstyledeclaration-item`）：index 处属性名（kebab），越界 → 空串。
fn native_style_item_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let idx = args.get(0).integer_value(scope).unwrap_or(-1);
    let name = with_dom(|d| current_props(d, owner).get(idx as usize).map(|d| d.prop.clone())).flatten();
    let name = name.unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &name) {
        rv.set(s.into());
    }
}

/// `getPropertyValue(prop)`（spec）：prop（kebab）值，未设 → 空串。
fn native_style_get_property_value_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let val = with_dom(|d| {
        let props = current_props(d, owner);
        // 刀 TS：transition 简写族读面（同 named getter 钩）。
        if let Some(v) = ts_family_value(&props, &prop) {
            return Some(v);
        }
        if let Some(found) = props.iter().find(|d| d.prop == prop) {
            return Some(found.value.clone());
        }
        // css-gaps 刀 B：简写名无本名声明时从名单 longhand 重组 specified 串。
        gap_rule_shorthand_longhands(&prop).and_then(|_| gap_rule_reconstruct(&props, &prop))
    })
    .flatten()
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &val) {
        rv.set(s.into());
    }
}

/// `getPropertyPriority(prop)`（spec `dom-cssstyledeclaration-getpropertypriority`）：prop 的 priority
///（"important" 若 `!important`，否则 ""）。
fn native_style_get_property_priority_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let important = with_dom(|d| {
        current_props(d, owner)
            .into_iter()
            .find(|d| d.prop == prop)
            .is_some_and(|d| d.important)
    })
    .unwrap_or(false);
    let s = if important { "important" } else { "" };
    if let Some(v) = v8::String::new(scope, s) {
        rv.set(v.into());
    }
}

/// `setProperty(prop, value, priority?)`（spec）：prop（kebab）upsert + 重序列化写回。priority 非空
/// 且 ASCII 大小写不敏感 "important" → 设 important，否则非 important（priority 空/其它 → 非 important）。
/// 返 undefined（spec）。
fn native_style_set_property_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    // WebIDL nullable `CSSOMString? value = ""`：JS null 归空串（cssom-1 `dom-cssstyledeclaration-
    // setproperty` 空值臂 ≡ removeProperty）。string_arg 的 ToString(null)="null" 会把 null 写成
    // 4 字符串毒化空值臂——清理面 `setProperty(prop, null)` 恒被值校验拒 → 既有声明永移除不掉。
    // https://drafts.csswg.org/cssom-1/#dom-cssstyledeclaration-setproperty
    let val = super::local_value_to_string_null_as_empty(scope, args.get(1));
    if prop.is_empty() {
        return;
    }
    // priority 第 3 参：spec 仅 "important" 生效（空/其它 → 非 important）。
    let important = string_arg(scope, &args, 2).eq_ignore_ascii_case("important");
    // R3211：空值语义 = 移除声明（spec `dom-cssstyledeclaration-setproperty`：value 空串 → removeProperty，
    // 不论 priority）。`el.style.setProperty('x','')` 是 reset inline 样式高频用法——旧实现 upsert 空值
    // 致 dangling `prop: `（同 named setter / polyfill merge_style_property 三处同病）。
    if val.trim().is_empty() {
        with_dom_mut(|d| {
            let mut props = current_props(d, owner);
            props.retain(|d| d.prop != prop);
            // css-gaps 刀 B：简写空值 = 名单 longhand 一并复位（spec 简写
            // removeProperty 覆盖其所控 longhand）。
            if let Some(longhands) = gap_rule_shorthand_longhands(&prop) {
                props.retain(|d| !longhands.contains(&d.prop.as_str()));
            }
            // 刀 TS：transition 简写空值 = 所控 longhand 一并复位（同上形态）。
            if prop == "transition" {
                props.retain(|d| !TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
            }
            write_props(d, owner, &props);
        });
        return;
    }
    // css-anchor-1 canonical serialization：anchor()/anchor-size() 参数序归一（锚元素
    // dashed-ident 固定在前）；自定义属性值 spec 语义原样保留（values::anchor 模块头注）。
    // grid 家族声明值 CSSOM 校验 + canonical 序列化（grid_cssom 模块头注）。
    let val = if prop.starts_with("--") {
        val
    } else if !zero_style_system::validate_decl_value(&prop, val.trim()) {
        // T54：白名单属性非法值整体拒绝（同 [`native_style_named_setter`] T54 臂——
        // setProperty 与 IDL setter 同语义）。
        return;
    } else if gap_rule_shorthand_longhands(&prop).is_some() {
        // css-gaps 刀 B：简写展开落 longhand（同 [`native_style_named_setter`] 展开臂，
        // priority 经展开声明透传）。
        let Some(expanded) = gap_rule_expand_decls(&prop, val.trim(), important) else {
            return;
        };
        with_dom_mut(|d| {
            let mut props = current_props(d, owner);
            props.retain(|d| d.prop != prop);
            if let Some(longhands) = gap_rule_shorthand_longhands(&prop) {
                props.retain(|d| !longhands.contains(&d.prop.as_str()));
            }
            props.extend(expanded);
            write_props(d, owner, &props);
        });
        return;
    } else if prop == "transition" {
        // 刀 TS：transition 简写展开落 longhand（同 [`native_style_named_setter`]
        // 展开臂，priority 经展开声明透传）。
        let Some(expanded) = ts_expand_decls(val.trim(), important) else {
            return;
        };
        with_dom_mut(|d| {
            let mut props = current_props(d, owner);
            props.retain(|d| d.prop != prop);
            props.retain(|d| !TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
            props.extend(expanded);
            write_props(d, owner, &props);
        });
        return;
    } else if super::grid_cssom::is_grid_family_prop(&prop) {
        match super::grid_cssom::canonicalize_grid_decl(&prop, val.trim()) {
            Some(v) => v,
            // 非法 grid 值 → 声明忽略（spec setProperty 语义）。
            None => return,
        }
    } else if prop == "position-area" {
        match super::position_area::canonicalize_decl(val.trim()) {
            Some(v) => v,
            // 非法 position-area → 声明忽略（同 named setter 臂语义）。
            None => return,
        }
    } else if let Some(folded) = zero_css_parser::values::fold_decl_specified(&prop, val.trim()) {
        // r2320（CSS Values 4 §10.11）：数学函数 specified 值常量折叠（同
        // [`native_style_named_setter`] r2320 臂）。
        folded
    } else {
        zero_css_parser::values::canonicalize_anchor_functions(val.trim())
    };
    with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        if let Some(decl) = props.iter_mut().find(|d| d.prop == prop) {
            decl.value = val.clone();
            decl.important = important;
        } else {
            props.push(StyleDecl {
                prop,
                value: val,
                important,
            });
        }
        write_props(d, owner, &props);
    });
}

/// `removeProperty(prop)`（spec）：移除 prop（kebab），返旧值（未设 → 空串）。重序列化写回。
fn native_style_remove_property_invoke(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue<v8::Value>,
) {
    let this = args.this();
    let Some(owner) = csd_owner(scope, &this) else {
        rv.set(v8::String::new(scope, "").unwrap().into());
        return;
    };
    let prop = string_arg(scope, &args, 0).to_ascii_lowercase();
    let old = with_dom_mut(|d| {
        let mut props = current_props(d, owner);
        let old = props
            .iter()
            .find(|d| d.prop == prop)
            .map(|d| d.value.clone())
            .or_else(|| {
                // 刀 TS：transition 简写族——本名无声明时旧值取读面同款重组形。
                (prop == "transition")
                    .then(|| ts_family_value(&props, "transition"))
                    .flatten()
            })
            .unwrap_or_default();
        let had_longhands =
            prop == "transition" && props.iter().any(|d| TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
        if old.is_empty() && !had_longhands {
            return old;
        }
        props.retain(|d| d.prop != prop);
        // 刀 TS：简写 removeProperty 覆盖其所控 longhand（spec 简写 remove 语义，
        // 同 css-gaps 刀 B 空值臂形态）。
        if prop == "transition" {
            props.retain(|d| !TRANSITION_LONGHANDS.contains(&d.prop.as_str()));
        }
        write_props(d, owner, &props);
        old
    })
    .unwrap_or_default();
    if let Some(s) = v8::String::new(scope, &old) {
        rv.set(s.into());
    }
}

// ── 模板构建 ──────────────────────────────────────────────────────────

/// 建 CSSStyleDeclaration ObjectTemplate（cssText(+setter) / length / item / getPropertyValue /
/// setProperty / removeProperty + named-property-handler 拦 camelCase 动态属性）并缓存（`set_style_template`）。
/// `install_dom_bindings` 调（Element 模板建之后）。
pub(super) fn build_and_cache_template(scope: &mut v8::PinScope) {
    let csd = v8::ObjectTemplate::new(scope);
    csd.set_internal_field_count(1);
    // cssText accessor（getter 规范化序列化 / setter 整体替换）。
    if let Some(k) = v8::String::new(scope, "cssText") {
        csd.set_accessor_with_setter(k.into(), native_style_css_text_getter, native_style_css_text_setter);
    }
    // length getter。
    if let Some(k) = v8::String::new(scope, "length") {
        csd.set_accessor(k.into(), native_style_length_getter);
    }
    let item = v8::FunctionTemplate::builder(native_style_item_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "item") {
        csd.set(k.into(), item.into());
    }
    let gpv = v8::FunctionTemplate::builder(native_style_get_property_value_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getPropertyValue") {
        csd.set(k.into(), gpv.into());
    }
    // R3171 getPropertyPriority（spec priority 单独读：!"important"/""）。
    let gpp = v8::FunctionTemplate::builder(native_style_get_property_priority_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "getPropertyPriority") {
        csd.set(k.into(), gpp.into());
    }
    let sp = v8::FunctionTemplate::builder(native_style_set_property_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "setProperty") {
        csd.set(k.into(), sp.into());
    }
    let rp = v8::FunctionTemplate::builder(native_style_remove_property_invoke).build(scope);
    if let Some(k) = v8::String::new(scope, "removeProperty") {
        csd.set(k.into(), rp.into());
    }
    // named-property-handler：拦 camelCase 动态属性（reserved 名 fallthrough 到上面模板方法/accessor）。
    csd.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(native_style_named_getter)
            .setter(native_style_named_setter)
            .query(native_style_named_query)
            .deleter(native_style_named_deleter),
    );
    set_style_template(scope, csd);
}

#[cfg(test)]
mod gap_rule_cssom_tests {
    use super::*;

    /// 单点联合初始分量省略（语料 valid 页钉：`currentcolor hidden medium`→
    /// `hidden`、`currentcolor none medium`→`medium` 保 width 锚）。
    #[test]
    fn test_joint_tokens_initial_omission() {
        assert_eq!(
            gap_rule_joint_tokens("medium", "hidden", "currentcolor"),
            Some("hidden".into())
        );
        assert_eq!(
            gap_rule_joint_tokens("medium", "none", "currentcolor"),
            Some("medium".into())
        );
        assert_eq!(gap_rule_joint_tokens("6px", "none", "blue"), Some("6px blue".into()));
        assert_eq!(
            gap_rule_joint_tokens("5px", "solid", "red"),
            Some("5px solid red".into())
        );
        assert_eq!(
            gap_rule_joint_tokens("medium", "double", "currentcolor"),
            Some("double".into())
        );
    }

    /// 三维段联合：单段 + repeat 段逐位（语料 valid/computed 页混合列表形）。
    #[test]
    fn test_triple_reconstruct_mixed_segments() {
        let r = gap_rule_triple_reconstruct(
            "5px, repeat(4, 5px)",
            "double, repeat(4, ridge)",
            "rgb(58, 58, 16), repeat(4, rgb(18, 18, 18))",
        );
        assert_eq!(
            r.as_deref(),
            Some("5px double rgb(58, 58, 16), repeat(4, 5px ridge rgb(18, 18, 18))")
        );
        // 多位循环体逐位（valid 页 `repeat(3, 16px, thin dashed purple, 10px dotted)` 形）。
        let r = gap_rule_triple_reconstruct(
            "repeat(3, 16px, thin, 10px)",
            "repeat(3, none, dashed, dotted)",
            "repeat(3, currentcolor, purple, currentcolor)",
        );
        assert_eq!(r.as_deref(), Some("repeat(3, 16px, thin dashed purple, 10px dotted)"));
    }

    /// 段数失配 → None（语料 from-longhands 页 misaligned 钉）。
    #[test]
    fn test_triple_reconstruct_misaligned_none() {
        assert!(gap_rule_triple_reconstruct("15px, 25px, 35px", "solid, dotted", "green").is_none());
        assert!(gap_rule_triple_reconstruct("repeat(auto, 5px)", "solid, double", "repeat(7, red)").is_none());
        // 同计数同循环长但 auto 多位循环体失配形（from-longhands 页钉）。
        assert!(gap_rule_triple_reconstruct("repeat(auto, 5px)", "repeat(auto, solid)", "repeat(auto, red)").is_some());
    }

    /// 写面展开 → 读面重组全链 roundtrip（语料 valid 页 canonical 序 + 简写页
    /// 展开值）。`solid 10px rgb(0, 0, 255)` → `10px solid rgb(0, 0, 255)`。
    #[test]
    fn test_expand_then_reconstruct_roundtrip() {
        for (input, expected) in [
            ("solid 10px rgb(0, 0, 255)", "10px solid rgb(0, 0, 255)"),
            ("5px solid red", "5px solid red"),
            ("double", "double"),
            ("blue 6px", "6px blue"),
            ("currentcolor hidden medium", "hidden"),
            ("currentcolor none medium", "medium"),
            ("repeat(4, 15px dotted pink)", "repeat(4, 15px dotted pink)"),
            ("thick, dashed, hotpink", "thick, dashed, hotpink"),
        ] {
            let decls = gap_rule_expand_decls("column-rule", input, false)
                .unwrap_or_else(|| panic!("valid form rejected: {input}"));
            assert_eq!(decls.len(), 3, "expansion count for {input}");
            assert_eq!(
                gap_rule_reconstruct(&decls, "column-rule").as_deref(),
                Some(expected),
                "{input}"
            );
        }
    }

    /// 非法值展开拒收（语料 invalid 页钉——validate 门已拒，展开层防御性 None）。
    #[test]
    fn test_expand_invalid_rejected() {
        assert!(gap_rule_expand_decls("column-rule", "red 5px solid red", false).is_none());
        assert!(gap_rule_expand_decls("column-rule", "repeat(0, 5px red)", false).is_none());
        assert!(gap_rule_expand_decls("column-rule", "auto", false).is_none());
    }

    /// `rule` 双维度全等重组 / 失配空串（gCS rule-color 简写臂同口径）。
    #[test]
    fn test_rule_shorthand_pair_equality() {
        let mut decls = gap_rule_expand_decls("rule", "5px solid red", false).unwrap();
        assert_eq!(decls.len(), 6);
        assert_eq!(gap_rule_reconstruct(&decls, "rule").as_deref(), Some("5px solid red"));
        // column 侧改值 → 失配 None。
        for d in decls.iter_mut() {
            if d.prop == "column-rule-color" {
                d.value = "blue".into();
            }
        }
        assert_eq!(gap_rule_reconstruct(&decls, "rule"), None);
    }

    /// shim inline 通道展开 JSON（刀 D inline 站钉面）：canonical 槽投影、
    /// 裸零折、css-wide 透传、空值 remove 列表、非家族名空串。
    #[test]
    fn test_shorthand_expand_json_shapes() {
        // inset 简写 canonical 展开（裸零折 0px、四槽全形）。
        assert_eq!(
            gap_rule_shorthand_expand_json("column-rule-inset", "0"),
            r#"[["column-rule-inset-cap-start","0px"],["column-rule-inset-cap-end","0px"],["column-rule-inset-junction-start","0px"],["column-rule-inset-junction-end","0px"]]"#
        );
        assert_eq!(
            gap_rule_shorthand_expand_json("rule-inset", "10px 20px"),
            r#"[["column-rule-inset-cap-start","10px"],["column-rule-inset-cap-end","20px"],["column-rule-inset-junction-start","10px"],["column-rule-inset-junction-end","20px"],["row-rule-inset-cap-start","10px"],["row-rule-inset-cap-end","20px"],["row-rule-inset-junction-start","10px"],["row-rule-inset-junction-end","20px"]]"#
        );
        // css-wide 关键字逐 longhand 透传。
        assert_eq!(
            gap_rule_shorthand_expand_json("column-rule-inset", "inherit"),
            r#"[["column-rule-inset-cap-start","inherit"],["column-rule-inset-cap-end","inherit"],["column-rule-inset-junction-start","inherit"],["column-rule-inset-junction-end","inherit"]]"#
        );
        // 空值 = 名单 remove 列表。
        assert_eq!(
            gap_rule_shorthand_expand_json("column-rule-inset", ""),
            r#"[["column-rule-inset-cap-start",""],["column-rule-inset-cap-end",""],["column-rule-inset-junction-start",""],["column-rule-inset-junction-end",""]]"#
        );
        // 非法值 / 非家族名 → 空串（回落通用写路径）。
        assert_eq!(gap_rule_shorthand_expand_json("column-rule-inset", "auto"), "");
        assert_eq!(gap_rule_shorthand_expand_json("color", "red"), "");
        // gap-rule 三维简写同通道（`solid 10px red` → canonical 序）。
        let j = gap_rule_shorthand_expand_json("column-rule", "solid 10px red");
        assert_eq!(
            j,
            r#"[["column-rule-width","10px"],["column-rule-style","solid"],["column-rule-color","red"]]"#
        );
    }

    /// shim inline 通道读重组（style 声明串域）：longhand 集重组 / 跨轴失配
    /// 空串 / attr 源大小写兜底 / priority 剥离。
    #[test]
    fn test_shorthand_reconstruct_from_style() {
        // inset 简写：4 长手声明串 → canonical 全形。
        let raw = "column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 20px; \
                   column-rule-inset-junction-start: 10px; column-rule-inset-junction-end: 20px";
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(raw, "column-rule-inset"),
            "10px 20px / 10px 20px"
        );
        // 全等塌缩单值。
        let raw = "column-rule-inset-cap-start: 0px; column-rule-inset-cap-end: 0px; \
                   column-rule-inset-junction-start: 0px; column-rule-inset-junction-end: 0px";
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(raw, "column-rule-inset"),
            "0px"
        );
        // 缺任一 longhand → 空串。
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style("column-rule-inset-cap-start: 10px", "column-rule-inset"),
            ""
        );
        // 非 family 简写名（长手名直接查）→ 空串。
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(raw, "column-rule-inset-cap-start"),
            ""
        );
        // gap-rule 三维：`solid 10px red` 展开 → `10px solid red` canonical 序；
        // priority 后缀不参与比对。
        let raw = "column-rule-width: 10px !important; column-rule-style: solid; \
                   column-rule-color: red";
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(raw, "column-rule"),
            "10px solid red"
        );
    }
}

#[cfg(test)]
mod knife_ts_cssom_tests {
    use super::*;

    fn decl(prop: &str, value: &str) -> StyleDecl {
        StyleDecl {
            prop: prop.into(),
            value: value.into(),
            important: false,
        }
    }

    /// 刀 TS 钉测：canonical 重组——缺省省略 + delay 非零 duration 强制 +
    /// 多层环绕 + css-wide keyword 原样（WPT css-transitions/parsing/
    /// transition-behavior.html 期望串逐形锚定，DOA 预核后落库）。
    #[test]
    fn knife_ts_shorthand_serialize_verdicts() {
        let s = |props: &[StyleDecl]| ts_serialize_from_longhands(props);
        assert_eq!(
            s(&[
                decl("transition-property", "display"),
                decl("transition-duration", "3s")
            ])
            .as_deref(),
            Some("display 3s")
        );
        // duration 0s + delay 1s：duration 强制输出（两位 <time> 消歧）
        assert_eq!(
            s(&[decl("transition-property", "display"), decl("transition-delay", "1s")]).as_deref(),
            Some("display 0s 1s")
        );
        assert_eq!(
            s(&[
                decl("transition-property", "display"),
                decl("transition-behavior", "allow-discrete")
            ])
            .as_deref(),
            Some("display allow-discrete")
        );
        assert_eq!(
            s(&[
                decl("transition-property", "display"),
                decl("transition-duration", "3s"),
                decl("transition-timing-function", "ease-in-out"),
                decl("transition-delay", "1s"),
                decl("transition-behavior", "allow-discrete"),
            ])
            .as_deref(),
            Some("display 3s ease-in-out 1s allow-discrete")
        );
        // 多层：behavior 仅首层非初值；等长名单逐层直取
        assert_eq!(
            s(&[
                decl("transition-property", "display, opacity, color"),
                decl("transition-behavior", "allow-discrete, normal, normal"),
            ])
            .as_deref(),
            Some("display allow-discrete, opacity, color")
        );
        // 五 longhand 全等 css-wide keyword → 原样回
        let inh: Vec<StyleDecl> = TRANSITION_LONGHANDS.iter().map(|n| decl(n, "inherit")).collect();
        assert_eq!(s(&inh).as_deref(), Some("inherit"));
        // 全缺 → None
        assert_eq!(s(&[]), None);
        // P2F 回归钉（电池 v2 实测）：property=初值 all 省略——部分组件形不回显 all
        assert_eq!(s(&[decl("transition-duration", "1s")]).as_deref(), Some("1s"));
        assert_eq!(
            s(&[decl("transition-timing-function", "cubic-bezier(0, -2, 1, 3)")]).as_deref(),
            Some("cubic-bezier(0, -2, 1, 3)")
        );
        assert_eq!(
            s(&[decl("transition-duration", "1s"), decl("transition-delay", "-3s")]).as_deref(),
            Some("1s -3s")
        );
        // 全层初值：保留 all（空串不是合法 <single-transition>）
        assert_eq!(s(&[decl("transition-property", "all")]).as_deref(), Some("all"));
        // 混层：首层全初值保 all、次层省略 all（LCM 取模两证）
        assert_eq!(
            s(&[
                decl("transition-property", "all, top"),
                decl("transition-duration", "0s, 2s"),
            ])
            .as_deref(),
            Some("all, top 2s")
        );
    }

    /// 刀 TS 钉测：读面派生——raw 展开重组 canonical / 层提取 / longhand 直查
    /// 优先 / 非法形拒（WPT css-transitions/parsing/transition-shorthand.html
    /// 形面锚定，DOA 预核后落库）。
    #[test]
    fn knife_ts_family_value_verdicts() {
        let raw = |v: &str| vec![decl("transition", v)];
        assert_eq!(
            ts_family_value(&raw("allow-discrete display"), "transition").as_deref(),
            Some("display allow-discrete")
        );
        // 任意序合法形 → canonical 同串（WPT behavior.html 期望串）
        assert_eq!(
            ts_family_value(&raw("display allow-discrete 3s ease-in-out 1s"), "transition").as_deref(),
            Some("display 3s ease-in-out 1s allow-discrete")
        );
        // 非法形展开空 → None（setter 臂声明忽略的读面对偶）
        assert_eq!(ts_family_value(&raw("asdf display"), "transition"), None);
        // css-wide keyword raw 原样
        assert_eq!(
            ts_family_value(&raw("inherit"), "transition").as_deref(),
            Some("inherit")
        );
        // 层提取：raw "1s -3s cubic-bezier(0, -2, 1, 3) top"（WPT shorthand.html 形）
        let props = raw("1s -3s cubic-bezier(0, -2, 1, 3) top");
        assert_eq!(ts_family_value(&props, "transition-property").as_deref(), Some("top"));
        assert_eq!(ts_family_value(&props, "transition-duration").as_deref(), Some("1s"));
        assert_eq!(ts_family_value(&props, "transition-delay").as_deref(), Some("-3s"));
        assert_eq!(
            ts_family_value(&props, "transition-timing-function").as_deref(),
            Some("cubic-bezier(0, -2, 1, 3)")
        );
        assert_eq!(
            ts_family_value(&props, "transition-behavior").as_deref(),
            Some("normal")
        );
        // longhand 直查优先于 raw 遮蔽
        let mixed = vec![decl("transition", "width 2s"), decl("transition-property", "color")];
        assert_eq!(ts_family_value(&mixed, "transition-property").as_deref(), Some("color"));
    }

    /// 刀 TS 钉测：shim inline 通道展开站（`transition_shorthand_expand_json`，
    /// WPT behavior.html/shorthand.html 形面锚定——写面 F2P 直通路）。
    #[test]
    fn knife_ts_expand_json_verdicts() {
        // 合法形 → 名单 longhand canonical JSON（WPT shorthand.html 层期望串）
        let j = transition_shorthand_expand_json("1s -3s cubic-bezier(0, -2, 1, 3) top");
        let pairs: Vec<(String, String)> = serde_json::from_str(&j).expect("json");
        assert_eq!(pairs.len(), 5);
        let get = |n: &str| pairs.iter().find(|(p, _)| p == n).map(|(_, v)| v.as_str());
        assert_eq!(get("transition-property"), Some("top"));
        assert_eq!(get("transition-duration"), Some("1s"));
        assert_eq!(get("transition-timing-function"), Some("cubic-bezier(0, -2, 1, 3)"));
        assert_eq!(get("transition-delay"), Some("-3s"));
        assert_eq!(get("transition-behavior"), Some("normal"));
        // 任意序 behavior 形（WPT behavior.html）
        let j2 = transition_shorthand_expand_json("allow-discrete display 3s ease-in-out 1s");
        let pairs2: Vec<(String, String)> = serde_json::from_str(&j2).expect("json2");
        let get2 = |n: &str| pairs2.iter().find(|(p, _)| p == n).map(|(_, v)| v.as_str());
        assert_eq!(get2("transition-property"), Some("display"));
        assert_eq!(get2("transition-duration"), Some("3s"));
        assert_eq!(get2("transition-timing-function"), Some("ease-in-out"));
        assert_eq!(get2("transition-delay"), Some("1s"));
        assert_eq!(get2("transition-behavior"), Some("allow-discrete"));
        // 空值 → 名单逐名清（R3211 remove 语义）
        let je = transition_shorthand_expand_json("");
        let pe: Vec<(String, String)> = serde_json::from_str(&je).expect("json-empty");
        assert_eq!(pe.len(), 5);
        assert!(pe.iter().all(|(_, v)| v.is_empty()));
        // 非法形 → 自清形（＝声明忽略净效果，expand 车道无 "[]" shim 契约改）
        assert_eq!(
            transition_shorthand_expand_json("asdf display"),
            r#"[["transition",""]]"#
        );
        assert_eq!(
            transition_shorthand_expand_json("display asdf"),
            r#"[["transition",""]]"#
        );
        // css-wide keyword 仅整值形合法（WPT invalid 页 "initial 1s" should-not-set）：
        // 混形组件展开 → 自清形（声明忽略），整值形原路 expand
        assert_eq!(transition_shorthand_expand_json("initial 1s"), r#"[["transition",""]]"#);
        assert_eq!(
            transition_shorthand_expand_json("inherit top"),
            r#"[["transition",""]]"#
        );
    }

    /// 刀 TS 钉测：inline 读重组站 transition 臂（`gap_rule_shorthand_
    /// reconstruct_from_style` 分派——raw attr 原文域双存储形兼容）。
    #[test]
    fn knife_ts_reconstruct_from_style_verdicts() {
        // 展开存储形（CSSOM 写面落库后的 attr 原文）
        let raw = "transition-property: top; transition-duration: 1s; transition-timing-function: \
                   cubic-bezier(0, -2, 1, 3); transition-delay: -3s; transition-behavior: normal";
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(raw, "transition"),
            "top 1s cubic-bezier(0, -2, 1, 3) -3s"
        );
        // 简写 raw 存储形（attr 车道原文落库）→ 展开 canonical 重组
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style("transition: allow-discrete display", "transition"),
            "display allow-discrete"
        );
        // 全缺 → 空串（无简写语义）
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style("color: red", "transition"),
            ""
        );
        // 非本族名 → 空串（gap/font 臂不受扰）
        assert_eq!(gap_rule_shorthand_reconstruct_from_style("color: red", "color"), "");
        // omit-initial：property 初值 all 与其余初值组件一并省略（WPT valid 页
        // "1s"→"1s"、"all 1s"→"1s" 显式期望串）；全初值时保留 all（空串非法）
        let all_d = "transition-property: all; transition-duration: 1s; \
                     transition-timing-function: ease; transition-delay: 0s; \
                     transition-behavior: normal";
        assert_eq!(gap_rule_shorthand_reconstruct_from_style(all_d, "transition"), "1s");
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style("transition: all 1s", "transition"),
            "1s"
        );
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style("transition: all", "transition"),
            "all"
        );
        // 两位 <time>：duration+delay 均非初值全保留（WPT "1s -3s"→"1s -3s"）
        let both_t = "transition-property: all; transition-duration: 1s; \
                      transition-timing-function: ease; transition-delay: -3s; \
                      transition-behavior: normal";
        assert_eq!(
            gap_rule_shorthand_reconstruct_from_style(both_t, "transition"),
            "1s -3s"
        );
    }
}
