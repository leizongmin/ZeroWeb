//! `CSS.registerProperty` 运行时注册表 — css-properties-values-api-1 刀（2026-11-17）。
//!
//! JS shim `CSS.registerProperty(desc)` 经 `__zw_register_property` 回调注入（校验+
//! 存储），消费面两处：
//! - gCS `--` 分支（[`super::computed_style`]）：注册属性的计算值按 syntax 分型
//!   串化（`green`+`<color>` → `rgb(0, 128, 0)`）+ 未声明时 initial-value 兜底；
//! - 动画插值通道（[`crate::animation::interpolate_property_value`] 的 `--` 臂）：
//!   WAAPI/transitions 对注册属性按 syntax 分型 lerp（`100deg`→`200deg` at .5 =
//!   `150deg`）。
//!
//! 生命周期同 [`super::js_anim_registry`]：导航/新 WebView 换代经
//! `register_dom_callbacks` 清表（文档作用域注册，spec §register-a-custom-property）。
//!
//! <https://drafts.css-houdini.org/css-properties-values-api-1/>

use std::cell::RefCell;
use std::collections::HashMap;

/// 注册失败类目（shim 侧映射为 DOMException/TypeError）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegErrorKind {
    Type,
    Syntax,
    InvalidModification,
}

/// syntax 分量类型（§support-syntax `<syntax-component-name>`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeKind {
    Angle,
    Color,
    Integer,
    Length,
    LengthPercentage,
    Number,
    Percentage,
    Resolution,
    Time,
    TransformFunction,
    TransformList,
    /// `<string>`/`<custom-ident>`/`<url>`/`<image>`——不串化、离散插值。
    Plain,
}

impl TypeKind {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "angle" => Self::Angle,
            "color" => Self::Color,
            "integer" => Self::Integer,
            "length" => Self::Length,
            "length-percentage" => Self::LengthPercentage,
            "number" => Self::Number,
            "percentage" => Self::Percentage,
            "resolution" => Self::Resolution,
            "time" => Self::Time,
            "transform-function" => Self::TransformFunction,
            "transform-list" => Self::TransformList,
            "string" | "custom-ident" | "url" | "image" => Self::Plain,
            _ => return None,
        })
    }

    /// `<syntax-component-name>` 名集合——custom-ident 字面量禁用名（§support-syntax：
    /// 与任一 component-name 同名的 ident 字面量非法，`length` 裸写因此 SyntaxError）。
    fn is_component_name(name: &str) -> bool {
        matches!(
            name,
            "angle"
                | "color"
                | "integer"
                | "length"
                | "length-percentage"
                | "number"
                | "percentage"
                | "resolution"
                | "string"
                | "time"
                | "url"
                | "image"
                | "transform-function"
                | "transform-list"
                | "custom-ident"
        )
    }
}

/// 列表重复标记（§support-syntax 乘子：`+` 空格列表 / `#` 逗号列表）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListMarker {
    Single,
    Space,
    Comma,
}

/// 单个 syntax 备选分量：typed 类型或 custom-ident 字面量（`kind: None`）。
#[derive(Debug, Clone)]
struct SyntaxComponent {
    kind: Option<TypeKind>,
    /// ident 字面量原文（kind None 时有效；`|` 组合中两侧字面量相等才可插值）。
    ident: String,
    list: ListMarker,
}

/// 解析后的 syntax 描述符。
#[derive(Debug, Clone)]
enum SyntaxDescriptor {
    /// `*`（universal：任意值，离散动画）。
    Universal,
    /// `normal`（默认：无语法约束，同 universal 离散语义）。
    Normal,
    /// `a | b | ...` 组合（单项也归此形）。
    Combined(Vec<SyntaxComponent>),
}

/// 运行时注册条目。
#[derive(Debug, Clone)]
struct RegisteredProperty {
    syntax: SyntaxDescriptor,
    /// FIXME(regprop-inherit)：inherits:true 祖先链传播 / false 父值截断未接
    /// style-system gather（运行时注册表未入 compute_styles），字段暂存待接。
    #[allow(dead_code)]
    inherits: bool,
    /// `initial-value` 原文；注册时预解析为计算形（`initial_computed`）。
    #[allow(dead_code)]
    initial: Option<String>,
    initial_computed: Option<String>,
}

thread_local! {
    /// 文档作用域注册表（线程亲和 = 渲染线程单文档语义，与 js_anim_registry ENTRIES 同形）。
    static REGISTRY: RefCell<HashMap<String, RegisteredProperty>> = RefCell::new(HashMap::new());
}

/// 注册表换代清空（导航/新 WebView；见 [`super::callbacks`] 装载序列）。
pub(crate) fn clear_all() {
    REGISTRY.with(|r| r.borrow_mut().clear());
}

/// 注册入口（`__zw_register_property` 回调委托；WebIDL 字典转换面已在 shim 完成，
/// 此处只做 spec 校验 + 存储）。
///
/// 校验序（语料 register-property.html 锚定）：name 形 → syntax 文法 →
/// initial/inherits 配对 → 重名 [`RegErrorKind::InvalidModification`]。
pub(crate) fn register(
    name: &str,
    syntax_raw: Option<&str>,
    inherits_present: bool,
    inherits: bool,
    initial_present: bool,
    initial: Option<&str>,
) -> Result<(), (RegErrorKind, &'static str)> {
    // §register-a-custom-property 步骤 1-2：name 须 <custom-property-name>。
    if !valid_custom_property_name(name) {
        return Err((RegErrorKind::Syntax, "Invalid property name"));
    }
    // 步骤 3：syntax 文法（缺省 "normal"）。
    let syntax_raw = syntax_raw.unwrap_or("normal");
    let syntax = match parse_syntax_descriptor(syntax_raw) {
        Some(s) => s,
        None => return Err((RegErrorKind::Syntax, "Invalid syntax descriptor")),
    };
    let typed = matches!(syntax, SyntaxDescriptor::Combined(_));
    let universal = matches!(syntax, SyntaxDescriptor::Universal);
    // 步骤 4-5：typed/universal 形须显式 inherits（语料 --inherit-test-3：typed+
    // initial 缺 inherits → TypeError；normal 形可缺省——WebIDL 字典 required-when-
    // typed 面）。
    if !inherits_present && (typed || universal) {
        return Err((
            RegErrorKind::Type,
            "The inherits dictionary member is required for this syntax",
        ));
    }
    // 步骤 6-8：initial-value 配对——typed 形必有且可按 syntax 解析（§register：
    // 「If syntax is not '*' and initialValue is not present / not parseable →
    // SyntaxError」）；'*' 可省；normal 形禁带（语料无锚、Chrome SyntaxError 口径）。
    let initial_computed = if typed {
        let Some(iv) = initial.filter(|v| !v.trim().is_empty()) else {
            return Err((RegErrorKind::Syntax, "initial-value is required for this syntax"));
        };
        match canonicalize_value(&syntax, iv, 16.0) {
            Some(c) => Some(c),
            None => return Err((RegErrorKind::Syntax, "initial-value does not parse for this syntax")),
        }
    } else if universal {
        match initial {
            Some(iv) if !iv.trim().is_empty() => Some(iv.trim().to_string()),
            _ => None,
        }
    } else if initial_present {
        return Err((RegErrorKind::Syntax, "initial-value is not allowed with normal syntax"));
    } else {
        None
    };
    // 步骤 9：重名注册 InvalidModificationError（语料 --re-register 锚定）。
    REGISTRY.with(|r| {
        let mut map = r.borrow_mut();
        if map.contains_key(name) {
            return Err((RegErrorKind::InvalidModification, "Property is already registered"));
        }
        map.insert(
            name.to_string(),
            RegisteredProperty {
                syntax,
                inherits,
                initial: initial.map(|v| v.trim().to_string()),
                initial_computed,
            },
        );
        Ok(())
    })
}

/// 查注册条目（模块内插值/串化消费）：syntax 描述符 + initial 计算形。
fn registration_of(name: &str) -> Option<(SyntaxDescriptor, Option<String>)> {
    REGISTRY.with(|r| {
        r.borrow()
            .get(name)
            .map(|p| (p.syntax.clone(), p.initial_computed.clone()))
    })
}

/// 名是否已注册（gCS 分支路由判据，避免无条件克隆 syntax）。
pub(crate) fn registration_known(name: &str) -> bool {
    REGISTRY.with(|r| r.borrow().contains_key(name))
}

/// `<custom-property-name>` 校验：`--` 前缀（dashed-ident 的运行时最小面——语料
/// register-property.html 接受 `--name2, no need for escapes` 带空格/逗号形，
/// 仅拒非 `--` 前缀与空串）。
fn valid_custom_property_name(name: &str) -> bool {
    name.starts_with("--") && name.len() > 2
}

/// ident 字面量校验（§support-syntax custom-ident 字面量：CSS ident 字符集子面——
/// 字母/数字/连字符/下划线，首字符非数字；禁 CSS 宽关键字与 component-name 同名）。
fn valid_ident_literal(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '-' => {}
        _ => return false,
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return false;
    }
    // 禁名面（§support-syntax）：CSS 宽关键字 + `default` + component-name 同名。
    // `none` **合法**（语料 `<url> | none` / `<transform-list>|none` 两页 Chrome-green
    // 锚定——tentative 页先行面；首版误禁曾致两页仍死）。
    !matches!(
        s,
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
    ) && !TypeKind::is_component_name(s)
}

/// `<syntax-string>` 文法解析（§support-syntax：`*` / `normal` / `<type>`[+|#]
/// [ `|` `<type>`[+|#] ]*，ident 字面量可作分量；`*` 不可组合；非法返 None）。
fn parse_syntax_descriptor(s: &str) -> Option<SyntaxDescriptor> {
    let t = s.trim();
    if t == "*" {
        return Some(SyntaxDescriptor::Universal);
    }
    if t == "normal" {
        return Some(SyntaxDescriptor::Normal);
    }
    if t.is_empty() || t.contains('*') {
        return None;
    }
    let mut components = Vec::new();
    for alt in t.split('|') {
        let alt = alt.trim();
        if alt.is_empty() {
            return None;
        }
        let (list, core) = if let Some(c) = alt.strip_suffix('+') {
            (ListMarker::Space, c.trim())
        } else if let Some(c) = alt.strip_suffix('#') {
            (ListMarker::Comma, c.trim())
        } else {
            (ListMarker::Single, alt)
        };
        let comp = if let Some(inner) = core.strip_prefix('<').and_then(|c| c.strip_suffix('>')) {
            let kind = TypeKind::from_name(inner.trim())?;
            SyntaxComponent {
                kind: Some(kind),
                ident: String::new(),
                list,
            }
        } else {
            if !valid_ident_literal(core) {
                return None;
            }
            SyntaxComponent {
                kind: None,
                ident: core.to_string(),
                list,
            }
        };
        components.push(comp);
    }
    Some(SyntaxDescriptor::Combined(components))
}

/// 按分量类型串化单值（canonical 计算形；不可解析返 None = guaranteed-invalid 面）。
fn canonicalize_scalar(kind: TypeKind, value: &str, font_size_px: f64) -> Option<String> {
    let v = value.trim();
    match kind {
        TypeKind::Angle => {
            let deg = parse_angle(v)?;
            Some(super::computed_style::format_num(deg, "deg"))
        }
        TypeKind::Time => {
            let s = parse_time(v)?;
            Some(super::computed_style::format_num(s, "s"))
        }
        TypeKind::Resolution => {
            let dppx = parse_resolution(v)?;
            Some(super::computed_style::format_num(dppx, "dppx"))
        }
        TypeKind::Number => Some(super::computed_style::format_num(v.parse::<f64>().ok()?, "")),
        TypeKind::Integer => Some(format!("{}", v.parse::<f64>().ok()?.round() as i64)),
        TypeKind::Percentage => {
            let n = v.strip_suffix('%')?.trim().parse::<f64>().ok()?;
            Some(super::computed_style::format_num(n, "%"))
        }
        TypeKind::Length | TypeKind::LengthPercentage => match parse_lp(v, font_size_px)? {
            (n, "px") => Some(super::computed_style::format_num(n, "px")),
            (n, "%") => Some(super::computed_style::format_num(n, "%")),
            _ => None,
        },
        TypeKind::Color => {
            // css-parser 全色面（命名色 green→rgb(0,128,0) 锚定）→ rgb/rgba
            // canonical（computed_style::color_to_css 同形，alpha 最短十进制）。
            let cv = zero_css_parser::values::parse_color(v)?;
            let c = crate::paint::color_value_to_render(&cv);
            if c.a == 255 {
                Some(format!("rgb({}, {}, {})", c.r, c.g, c.b))
            } else {
                Some(format!(
                    "rgba({}, {}, {}, {})",
                    c.r,
                    c.g,
                    c.b,
                    super::canvas::serialize_alpha(c.a)
                ))
            }
        }
        TypeKind::TransformList | TypeKind::TransformFunction => {
            // 计算值串化取解析形回写（插值走 transform_interp 通道）；空/none 原样。
            Some(v.to_string())
        }
        TypeKind::Plain => Some(v.to_string()),
    }
}

/// `123deg` → 123.0（deg canonical；grad/rad/turn 换算）。
fn parse_angle(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    match unit {
        "deg" => Some(num),
        "grad" => Some(num * 360.0 / 400.0),
        "rad" => Some(num * 180.0 / std::f64::consts::PI),
        "turn" => Some(num * 360.0),
        _ => None,
    }
}

/// time 端点解析（s canonical）。
fn parse_time(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    match unit {
        "s" => Some(num),
        "ms" => Some(num / 1000.0),
        _ => None,
    }
}

/// resolution 端点解析（dppx canonical；1dppx=96dpi、1dpcm=2.54dpi）。
fn parse_resolution(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    match unit {
        "dppx" | "x" => Some(num),
        "dpi" => Some(num / 96.0),
        "dpcm" => Some(num * 2.54 / 96.0),
        _ => None,
    }
}

/// `123deg` → (123.0, "deg") 形拆分。
fn split_num_unit(v: &str) -> Option<(f64, &str)> {
    let idx = v.find(|c: char| c.is_ascii_alphabetic())?;
    let (num, unit) = v.split_at(idx);
    Some((num.trim().parse::<f64>().ok()?, unit))
}

/// length-percentage 端点解析 → (数值, "px"|"%)（em/rem 按元素 font-size 解析 px）。
fn parse_lp(s: &str, font_size_px: f64) -> Option<(f64, &'static str)> {
    match zero_css_parser::values::parse_length(s.trim())? {
        zero_css_parser::values::LengthValue::Px(n) => Some((n, "px")),
        zero_css_parser::values::LengthValue::Percentage(n) => Some((n, "%")),
        zero_css_parser::values::LengthValue::Em(n) | zero_css_parser::values::LengthValue::Rem(n) => {
            Some((n * font_size_px, "px"))
        }
        _ => None,
    }
}

/// 注册属性计算值串化（gCS `--` 分支消费）：
/// - 有声明值：按 syntax 分型解析串化；解析失败 → `Some("")`（§3.2 invalid at
///   computed-value time → guaranteed-invalid，序列化空串）；
/// - 无声明值：initial 兜底（`inherits: false` 元素重置语义近似——FIXME(regprop-inherit)：
///   inherits:true 的祖先链传播与 inherits:false 的父值截断未做，依赖
///   style-system gather 未接运行时注册表）。
///
/// 未注册返 None（调用方走既有未注册 `--` 路径）。
pub(crate) fn computed_value(name: &str, declared: Option<&str>, font_size_px: f64) -> Option<String> {
    let (syntax, initial) = registration_of(name)?;
    Some(match declared {
        Some(v) if !v.trim().is_empty() => canonicalize_value(&syntax, v, font_size_px).unwrap_or_default(),
        _ => initial.unwrap_or_default(),
    })
}

/// 值按 syntax 分型串化（含列表展开；`|` 组合按首个可解析的备选分型）。
fn canonicalize_value(syntax: &SyntaxDescriptor, value: &str, font_size_px: f64) -> Option<String> {
    match syntax {
        SyntaxDescriptor::Universal | SyntaxDescriptor::Normal => Some(value.trim().to_string()),
        SyntaxDescriptor::Combined(components) => components
            .iter()
            .find_map(|comp| canonicalize_component(comp, value, font_size_px)),
    }
}

/// 单分量串化（含 +/# 列表展开）。
fn canonicalize_component(comp: &SyntaxComponent, value: &str, font_size_px: f64) -> Option<String> {
    match comp.kind {
        None => {
            // ident 字面量：相等即原样；不等落下一备选。
            (value.trim() == comp.ident).then(|| value.trim().to_string())
        }
        Some(kind) => match comp.list {
            ListMarker::Single => canonicalize_scalar(kind, value, font_size_px),
            ListMarker::Space => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                parts
                    .iter()
                    .map(|p| canonicalize_scalar(kind, p, font_size_px))
                    .collect::<Option<Vec<_>>>()
                    .map(|v| v.join(" "))
            }
            ListMarker::Comma => {
                let parts: Vec<&str> = value.split(',').collect();
                if parts.is_empty() {
                    return None;
                }
                parts
                    .iter()
                    .map(|p| canonicalize_scalar(kind, p, font_size_px))
                    .collect::<Option<Vec<_>>>()
                    .map(|v| v.join(", "))
            }
        },
    }
}

/// 注册属性分型插值（`interpolate_property_value` `--` 臂消费；None = 不可插值，
/// 调用方回落 css-values-4 §8.5.6 离散翻转）。
///
/// 语义（css-properties-values-api-1 §animation + css-values-4 §8.5.6）：
/// - universal/normal/ident 字面量失配 → 离散（None）；
/// - 同型标量 lerp（angle/color/number/integer/length/percentage/time/resolution）；
/// - px↔% 混合 → calc 混合形（`calc(a% + bpx)` 最短十进制）；
/// - 列表（+/#）逐分量同型 lerp，计数失配 → 离散；
/// - transform-list/function 委托既有 transform_interp 通道；
/// - 组合（`|`）按首个两侧可插值的备选；全不可插值 → 离散。
pub(crate) fn interpolate(name: &str, from: &str, to: &str, t: f64, font_size_px: f64) -> Option<String> {
    let (syntax, _initial) = registration_of(name)?;
    interpolate_syntax(&syntax, from, to, t, font_size_px)
}

/// 按 syntax 分型插值。
fn interpolate_syntax(syntax: &SyntaxDescriptor, from: &str, to: &str, t: f64, font_size_px: f64) -> Option<String> {
    match syntax {
        SyntaxDescriptor::Universal | SyntaxDescriptor::Normal => None,
        SyntaxDescriptor::Combined(components) => components
            .iter()
            .find_map(|comp| interpolate_component(comp, from, to, t, font_size_px)),
    }
}

/// 单分量插值。
fn interpolate_component(comp: &SyntaxComponent, from: &str, to: &str, t: f64, font_size_px: f64) -> Option<String> {
    match comp.kind {
        None => (from.trim() == to.trim() && from.trim() == comp.ident).then(|| comp.ident.clone()),
        Some(kind) => match comp.list {
            ListMarker::Single => interpolate_scalar(kind, from, to, t, font_size_px),
            ListMarker::Space => {
                let f: Vec<&str> = from.split_whitespace().collect();
                let v: Vec<&str> = to.split_whitespace().collect();
                if f.len() != v.len() || f.is_empty() {
                    return None;
                }
                let out: Option<Vec<String>> = f
                    .iter()
                    .zip(v.iter())
                    .map(|(a, b)| interpolate_scalar(kind, a, b, t, font_size_px))
                    .collect();
                out.map(|v| v.join(" "))
            }
            ListMarker::Comma => {
                let f: Vec<&str> = from.split(',').collect();
                let v: Vec<&str> = to.split(',').collect();
                if f.len() != v.len() || f.is_empty() {
                    return None;
                }
                let out: Option<Vec<String>> = f
                    .iter()
                    .zip(v.iter())
                    .map(|(a, b)| interpolate_scalar(kind, a, b, t, font_size_px))
                    .collect();
                out.map(|v| v.join(", "))
            }
        },
    }
}

/// 标量分型插值。
fn interpolate_scalar(kind: TypeKind, from: &str, to: &str, t: f64, font_size_px: f64) -> Option<String> {
    let lerp = |a: f64, b: f64| a + (b - a) * t;
    match kind {
        TypeKind::Angle => Some(super::computed_style::format_num(
            lerp(parse_angle(from.trim())?, parse_angle(to.trim())?),
            "deg",
        )),
        TypeKind::Time => Some(super::computed_style::format_num(
            lerp(parse_time(from.trim())?, parse_time(to.trim())?),
            "s",
        )),
        TypeKind::Resolution => Some(super::computed_style::format_num(
            lerp(parse_resolution(from.trim())?, parse_resolution(to.trim())?),
            "dppx",
        )),
        TypeKind::Number => Some(super::computed_style::format_num(
            lerp(from.trim().parse::<f64>().ok()?, to.trim().parse::<f64>().ok()?),
            "",
        )),
        TypeKind::Integer => Some(format!(
            "{}",
            lerp(from.trim().parse::<f64>().ok()?, to.trim().parse::<f64>().ok()?).round() as i64
        )),
        TypeKind::Percentage => {
            let f = from.trim().strip_suffix('%')?.trim().parse::<f64>().ok()?;
            let v = to.trim().strip_suffix('%')?.trim().parse::<f64>().ok()?;
            Some(super::computed_style::format_num(lerp(f, v), "%"))
        }
        TypeKind::Length => {
            let f = parse_lp(from.trim(), font_size_px)?;
            let v = parse_lp(to.trim(), font_size_px)?;
            match (f, v) {
                ((a, "px"), (b, "px")) => Some(super::computed_style::format_num(lerp(a, b), "px")),
                // <length> 语法 %-端点非法——组合形落 LengthPercentage 臂（interpolate_syntax
                // 分量按注册文法分派，此臂只承接纯 <length>）。
                _ => None,
            }
        }
        TypeKind::LengthPercentage => {
            let f = parse_lp(from.trim(), font_size_px)?;
            let v = parse_lp(to.trim(), font_size_px)?;
            match (f, v) {
                ((a, "px"), (b, "px")) => Some(super::computed_style::format_num(lerp(a, b), "px")),
                ((a, "%"), (b, "%")) => Some(super::computed_style::format_num(lerp(a, b), "%")),
                // px↔% 混合 → calc(a% + bpx)（css-values-4 §8.5.6 interpolation-calc；
                // 语料 0%→100px at .5 期望 `calc(50% + 50px)` 形）。
                ((a, au), (b, bu)) => {
                    let pct = lerp(if au == "%" { a } else { 0.0 }, if bu == "%" { b } else { 0.0 });
                    let px = lerp(if au == "px" { a } else { 0.0 }, if bu == "px" { b } else { 0.0 });
                    if px.abs() < 1e-9 {
                        Some(super::computed_style::format_num(pct, "%"))
                    } else if px < 0.0 {
                        Some(format!(
                            "calc({} - {})",
                            super::computed_style::format_num(pct, "%"),
                            super::computed_style::format_num(-px, "px")
                        ))
                    } else {
                        Some(format!(
                            "calc({} + {})",
                            super::computed_style::format_num(pct, "%"),
                            super::computed_style::format_num(px, "px")
                        ))
                    }
                }
            }
        }
        TypeKind::Color => {
            // css-parser 全色面（命名色/函数形）→ render Color u8 通道 lerp →
            // rgb/rgba canonical（语料期望 `rgb(128, 0, 0)` 整型面；alpha<1 走
            // 最短十进制 serialize_alpha 口径）。
            let f = color_channels(from)?;
            let v = color_channels(to)?;
            let (r, g, b, a) = (lerp(f.0, v.0), lerp(f.1, v.1), lerp(f.2, v.2), lerp(f.3, v.3));
            if a >= 255.0 {
                Some(format!(
                    "rgb({}, {}, {})",
                    r.round() as u8,
                    g.round() as u8,
                    b.round() as u8
                ))
            } else {
                Some(format!(
                    "rgba({}, {}, {}, {})",
                    r.round() as u8,
                    g.round() as u8,
                    b.round() as u8,
                    super::canvas::serialize_alpha(a.round() as u8)
                ))
            }
        }
        TypeKind::TransformList | TypeKind::TransformFunction => {
            crate::animation::transform_interp::interpolate_transform_value(from, to, t)
        }
        TypeKind::Plain => None,
    }
}

/// 色值端点解析 → (r, g, b, a 0-255)（css-parser 全色面 + render 解析）。
fn color_channels(s: &str) -> Option<(f64, f64, f64, f64)> {
    let cv = zero_css_parser::values::parse_color(s.trim())?;
    let c = crate::paint::color_value_to_render(&cv);
    Some((c.r as f64, c.g as f64, c.b as f64, c.a as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_grammar() {
        assert!(matches!(
            parse_syntax_descriptor("*"),
            Some(SyntaxDescriptor::Universal)
        ));
        assert!(matches!(
            parse_syntax_descriptor(" * "),
            Some(SyntaxDescriptor::Universal)
        ));
        assert!(matches!(
            parse_syntax_descriptor("normal"),
            Some(SyntaxDescriptor::Normal)
        ));
        assert!(parse_syntax_descriptor("length").is_none());
        assert!(parse_syntax_descriptor("").is_none());
        assert!(parse_syntax_descriptor("<length> | <percentage>").is_some());
        assert!(parse_syntax_descriptor("<length>+").is_some());
        assert!(parse_syntax_descriptor("<angle>#").is_some());
        assert!(parse_syntax_descriptor("thick | thin").is_some());
        assert!(parse_syntax_descriptor("default").is_none());
    }

    #[test]
    fn test_register_validation() {
        clear_all();
        // name 形。
        assert_eq!(
            register("no-dash", None, true, false, false, None),
            Err((RegErrorKind::Syntax, "Invalid property name"))
        );
        assert_eq!(
            register("", None, true, false, false, None),
            Err((RegErrorKind::Syntax, "Invalid property name"))
        );
        // '*' 可省 initial。
        assert!(register("--s1", Some("*"), true, false, false, None).is_ok());
        // typed 缺 initial → SyntaxError。
        assert!(register("--s2", Some("<length>"), true, false, false, None).is_err());
        // typed 缺 inherits 成员 → TypeError。
        assert!(register("--s3", Some("<length>"), false, false, true, Some("0px")).is_err());
        // typed 全参 → ok。
        assert!(register("--s4", Some("<length>"), true, false, true, Some("0px")).is_ok());
        // 重名 → InvalidModificationError。
        assert_eq!(
            register("--s4", Some("<percentage>"), true, false, true, Some("0%")),
            Err((RegErrorKind::InvalidModification, "Property is already registered"))
        );
        // 裸 component-name 字面量 → SyntaxError。
        assert!(register("--s5", Some("length"), true, false, true, Some("0px")).is_err());
    }

    #[test]
    fn test_canonicalize_and_interpolate() {
        clear_all();
        assert!(register("--a", Some("<angle>"), true, false, true, Some("100deg")).is_ok());
        assert_eq!(computed_value("--a", None, 16.0), Some("100deg".into()));
        assert_eq!(computed_value("--a", Some("0.5turn"), 16.0), Some("180deg".into()));
        assert_eq!(interpolate("--a", "100deg", "200deg", 0.5, 16.0), Some("150deg".into()));
        clear_all();
        assert!(register("--c", Some("<color>"), true, false, true, Some("rgb(0, 0, 0)")).is_ok());
        // 命名色串化（register-property.html 锚定 'green'→'rgb(0, 128, 0)'）。
        assert_eq!(
            computed_value("--c", Some("green"), 16.0),
            Some("rgb(0, 128, 0)".into())
        );
        assert_eq!(
            interpolate("--c", "rgb(0, 0, 0)", "rgb(255, 0, 0)", 0.5, 16.0),
            Some("rgb(128, 0, 0)".into())
        );
        clear_all();
        assert!(register("--lp", Some("<length-percentage>"), true, false, true, Some("0px")).is_ok());
        // 0%→100px at .5 = calc(0% + 50px)（css-values-4 interpolation-calc：pct 分量
        // (1-t)·from + t·to 双侧独立）。
        assert_eq!(
            interpolate("--lp", "0%", "100px", 0.5, 16.0),
            Some("calc(0% + 50px)".into())
        );
        assert_eq!(interpolate("--lp", "50px", "100px", 0.5, 16.0), Some("75px".into()));
        clear_all();
        assert!(register("--l", Some("<angle>+"), true, false, true, Some("0deg")).is_ok());
        assert_eq!(
            interpolate("--l", "100deg 200deg", "300deg 400deg", 0.5, 16.0),
            Some("200deg 300deg".into())
        );
        clear_all();
        assert!(register("--cl", Some("<angle>#"), true, false, true, Some("0deg")).is_ok());
        assert_eq!(
            interpolate("--cl", "100deg, 200deg", "300deg, 400deg", 0.5, 16.0),
            Some("200deg, 300deg".into())
        );
    }
}
