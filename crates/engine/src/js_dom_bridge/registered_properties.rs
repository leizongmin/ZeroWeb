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
    /// `<string>`——引号字符串字面量（注册门引号文法，§5.2 support-syntax）。
    StringKind,
    /// `<custom-ident>`——ident 字面量（注册门禁 CSS 宽关键字六枚）。
    CustomIdent,
    /// `<url>`——url() 函数形（裸 ident 非法）。
    UrlKind,
    /// `<image>`——url()/gradient 族/image()/image-set()/paint()/cross-fade()/light-dark()。
    ImageKind,
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
            "string" => Self::StringKind,
            "custom-ident" => Self::CustomIdent,
            "url" => Self::UrlKind,
            "image" => Self::ImageKind,
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
    // 步骤 3：syntax 文法（WebIDL 字典缺省 `*`——css-properties-values-api-1
    // PropertyDescriptor syntax = "*"；register-property-syntax-parsing 语料
    // `assert_valid(undefined, "undefined")` 锚定缺省 universal 形可带 initial）。
    let syntax_raw = syntax_raw.unwrap_or("*");
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
    // SyntaxError」）；'*' 可省但带值须过 <declaration-value> 门；normal 形禁带
    //（语料无锚、Chrome SyntaxError 口径）。校验走注册门 strict face（刀 RP1：
    // font-relative 长度拒/% 门/calc 分型/坏 token/var() 禁——见
    // [`validate_initial_strict`]），与 lenient 计算面分离。
    let initial_computed = if typed {
        let Some(iv) = initial.filter(|v| !v.trim().is_empty()) else {
            return Err((RegErrorKind::Syntax, "initial-value is required for this syntax"));
        };
        match validate_initial_strict(&syntax, iv) {
            Some(c) => Some(c),
            None => return Err((RegErrorKind::Syntax, "initial-value does not parse for this syntax")),
        }
    } else if universal {
        match initial {
            Some(iv) if !iv.trim().is_empty() => match validate_initial_strict(&syntax, iv) {
                Some(c) => Some(c),
                None => return Err((RegErrorKind::Syntax, "initial-value does not parse for this syntax")),
            },
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

/// ident 字面量 CSS 转义解码（css-syntax-3 §4.3.7 consume an escaped character）：
/// `\` + 1–6 位 hex（后随单个空白一并消费）→ 码点（越界/代理/空 → U+FFFD）；
/// `\` + 非 hex 字面直写；`\` EOF → U+FFFD。解码恒有定义（合法转义形全覆盖），
/// 返回 String 而非 Option——非法性由调用方 [`valid_ident_literal`] 按解码形裁决
///（`banan\61`≡`banana`、`\1F914`→🤔 语料锚定）。
fn decode_css_ident(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let mut hex = String::new();
        while let Some(&h) = chars.peek() {
            if hex.len() < 6 && h.is_ascii_hexdigit() {
                hex.push(h);
                chars.next();
            } else {
                break;
            }
        }
        if !hex.is_empty() {
            // hex 后随单个空白是转义定界符，一并消费（`<\6c ength>` 类型名臂不
            // 走本函数——ident 字面量通道专属）。
            if matches!(chars.peek(), Some(' ' | '\t' | '\n' | '\r' | '\u{000C}')) {
                chars.next();
            }
            let v = u32::from_str_radix(&hex, 16).unwrap_or(0);
            out.push(char::from_u32(v).filter(|&c| c != '\0').unwrap_or('\u{FFFD}'));
        } else {
            match chars.next() {
                Some(n) => out.push(n),
                None => out.push('\u{FFFD}'),
            }
        }
    }
    out
}

/// ident 字面量校验（§support-syntax custom-ident 字面量：CSS ident 字符集子面——
/// 字母/数字/连字符/下划线 + U+0080 以上直写形（css-syntax-3 §4.3.3 `<ident-start-char>`/
/// `<ident-char>` 非 ASCII 段——`\1F914`→🤔 解码形语料锚定合法），首字符非数字；
/// 禁 CSS 宽关键字与 component-name 同名）。
fn valid_ident_literal(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '-' || c as u32 >= 0x80 => {}
        _ => return false,
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c as u32 >= 0x80) {
        return false;
    }
    // 禁名面（§support-syntax）：CSS 宽关键字 + `default`（ASCII ci——语料
    // INHERIT/unsEt/REVert/deFAUlT 四形锚定）+ component-name 同名。
    // `none` **合法**（语料 `<url> | none` / `<transform-list>|none` 两页 Chrome-green
    // 锚定——tentative 页先行面；首版误禁曾致两页仍死）。
    let lower = s.to_ascii_lowercase();
    !matches!(
        lower.as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
    ) && !TypeKind::is_component_name(s)
}

/// `<syntax-string>` 文法解析（§support-syntax：`*` / `normal` / `<type>`[+|#]
/// [ `|` `<type>`[+|#] ]*，ident 字面量可作分量；`*` 不可组合；非法返 None）。
/// 文法硬化（register-property-syntax-parsing 语料锚定）：`<…>` 括号内禁空白
/// （`< length>`/`<length >`/`<\6c ength>` 非法——类型名不做转义解码）；
/// 乘子须紧邻分量（`<length> +` 非法）；`<transform-list>` 禁乘子（已自带
/// 列表语义）；ident 字面量做 CSS 转义解码（`banan\61`≡`banana`、`\1F914`
/// 非 ASCII）并按解码形大小写不敏感禁 CSS 宽关键字/component-name 同名
///（`<length>|INHERIT` 非法）。
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
        // 乘子须紧邻分量（`<length> +` 语料锚定非法——剥乘子后核心段不得有
        // 尾随空白；`|` 两侧空白由 alt.trim 消化，`<length>+ ` 整体 trim 后合法）。
        let (list, core) = match alt.as_bytes().last() {
            Some(b'+') => {
                let core = &alt[..alt.len() - 1];
                if core.ends_with(char::is_whitespace) {
                    return None;
                }
                (ListMarker::Space, core)
            }
            Some(b'#') => {
                let core = &alt[..alt.len() - 1];
                if core.ends_with(char::is_whitespace) {
                    return None;
                }
                (ListMarker::Comma, core)
            }
            _ => (ListMarker::Single, alt),
        };
        if core.is_empty() {
            return None;
        }
        let comp = if let Some(inner) = core.strip_prefix('<').and_then(|c| c.strip_suffix('>')) {
            // 类型名：括号内禁空白（含转义——类型名不解码，`<\6c ength>` 锚定非法）。
            if inner.is_empty() || inner.chars().any(char::is_whitespace) {
                return None;
            }
            let kind = TypeKind::from_name(inner)?;
            // <transform-list> 已自带列表语义，禁 +/# 乘子。
            if kind == TypeKind::TransformList && list != ListMarker::Single {
                return None;
            }
            SyntaxComponent {
                kind: Some(kind),
                ident: String::new(),
                list,
            }
        } else {
            // ident 字面量：CSS 转义解码后校验（含 ',' 与空白拒——`banana,nya`
            // 锚定）；禁名按解码形大小写不敏感（INHERIT/unsEt/deFAUlt 锚定）。
            let decoded = decode_css_ident(core);
            if decoded.is_empty() || decoded.chars().any(|c| c.is_whitespace() || c == ',') {
                return None;
            }
            if !valid_ident_literal(&decoded) {
                return None;
            }
            SyntaxComponent {
                kind: None,
                ident: decoded,
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
        // 引号串/ident/url()/image 函数形：计算串化取原文（注册门裁决文法——
        // 语料 `url(a)`/`linear-gradient(...)`/`light-dark(...)` 原样回显）。
        TypeKind::StringKind | TypeKind::CustomIdent | TypeKind::UrlKind | TypeKind::ImageKind => Some(v.to_string()),
    }
}

/// `123deg` → 123.0（deg canonical；grad/rad/turn 换算；单位 ci——css 单位
/// ascii 大小写不敏感，`3dPpX` 同口径）。
fn parse_angle(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    angle_unit_to_deg(num, &unit.to_ascii_lowercase())
}

/// time 端点解析（s canonical；单位 ci）。
fn parse_time(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    time_unit_to_sec(num, &unit.to_ascii_lowercase())
}

/// resolution 端点解析（dppx canonical；1dppx=96dpi、1dpcm=2.54dpi；单位 ci；
/// 负值恒拒——css-values-4 §6.4「allowed range of <resolution> values always
/// excludes negative values」，register-property-syntax-parsing `-5.3dpcm` 锚定）。
fn parse_resolution(s: &str) -> Option<f64> {
    let (num, unit) = split_num_unit(s)?;
    resolution_unit_to_dppx(num, &unit.to_ascii_lowercase())
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

// ===================== 注册门 strict face（刀 RP1，2026-11-18） =====================
// 与 lenient 计算面（canonicalize_*/parse_lp：em→px 求值消费）分离——注册门按
// register-property-syntax-parsing 语料硬裁：font-relative 长度拒、纯 <length>
// 拒 %、calc 分型校验、<declaration-value> 坏 token 面（bad-string/bad-url/
// 顶层 `;`/顶层 `!`/var()）、CSS 宽关键字值禁、<string>/<url>/<image>/
// <transform-function> 值形门、整数 token 文法（`1.0`/`1e0` 拒）。

/// 注册门名义视口（视口/容器族原子折 px 用——注册时无视口上下文）。
/// FIXME(regprop-vp)：initial 含视口族时 gCS 兜底按名义视口显示，计算链
/// （style-system 视口上下文）接管后替换。
const REGISTER_NOMINAL_VIEWPORT: (f64, f64) = (1280.0, 800.0);

/// 注册门 calc 原子族（css-values-4 §10.5 型别子面）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum CalcAtom {
    /// 无单位数。
    Num(f64),
    /// 绝对/视口/容器长度（已折 px）。
    Len(f64),
    /// 百分比（% 前数值）。
    Pct(f64),
    /// len+% 混合形（px 部, % 部）。
    MixedLenPct(f64, f64),
    /// 角度（折 deg）。
    Angle(f64),
    /// 时间（折 s）。
    Time(f64),
    /// 分辨率（折 dppx）。
    Res(f64),
}

/// 角度原子单位折 deg（ci——css 单位 ascii 大小写不敏感）。
fn angle_unit_to_deg(n: f64, unit: &str) -> Option<f64> {
    match unit {
        "deg" => Some(n),
        "grad" => Some(n * 360.0 / 400.0),
        "rad" => Some(n * 180.0 / std::f64::consts::PI),
        "turn" => Some(n * 360.0),
        _ => None,
    }
}

/// time 原子单位折 s。
fn time_unit_to_sec(n: f64, unit: &str) -> Option<f64> {
    match unit {
        "s" => Some(n),
        "ms" => Some(n / 1000.0),
        _ => None,
    }
}

/// 分辨率原子单位折 dppx（1dppx=96dpi、1dpcm=2.54dpi；负值恒拒）。
fn resolution_unit_to_dppx(n: f64, unit: &str) -> Option<f64> {
    let v = match unit {
        "dppx" | "x" => n,
        "dpi" => n / 96.0,
        "dpcm" => n * 2.54 / 96.0,
        _ => return None,
    };
    (v >= 0.0).then_some(v)
}

/// calc 原子长度单位折 px（绝对族 + 视口/容器族名义视口；font-relative 族与
/// 未知单位拒——`calc(4px + 3em)` 语料锚定非法）。
fn length_unit_to_px(n: f64, unit: &str) -> Option<f64> {
    let (w, h) = REGISTER_NOMINAL_VIEWPORT;
    Some(match unit {
        "px" => n,
        "cm" => n * 96.0 / 2.54,
        "mm" => n * 96.0 / 25.4,
        "q" => n * 96.0 / 254.0,
        "in" => n * 96.0,
        "pt" => n * 96.0 / 72.0,
        "pc" => n * 16.0,
        "vw" | "svw" | "lvw" | "dvw" | "cqw" | "cqi" | "vi" | "svi" | "lvi" | "dvi" => n * w / 100.0,
        "vh" | "svh" | "lvh" | "dvh" | "cqh" | "cqb" | "vb" | "svb" | "lvb" | "dvb" => n * h / 100.0,
        "vmin" | "svmin" | "lvmin" | "dvmin" | "cqmin" => n * w.min(h) / 100.0,
        "vmax" | "svmax" | "lvmax" | "dvmax" | "cqmax" => n * w.max(h) / 100.0,
        _ => return None,
    })
}

/// ± 分型算术（css-values-4 §10.1：同族相加减；len↔% 混合归 [`CalcAtom::MixedLenPct`]）。
fn calc_add_sub(a: CalcAtom, b: CalcAtom, add: bool) -> Option<CalcAtom> {
    let s = |x: f64, y: f64| if add { x + y } else { x - y };
    use CalcAtom::*;
    Some(match (a, b) {
        (Num(x), Num(y)) => Num(s(x, y)),
        (Len(x), Len(y)) => Len(s(x, y)),
        (Pct(x), Pct(y)) => Pct(s(x, y)),
        (Angle(x), Angle(y)) => Angle(s(x, y)),
        (Time(x), Time(y)) => Time(s(x, y)),
        (Res(x), Res(y)) => Res(s(x, y)),
        (Len(x), Pct(y)) => MixedLenPct(x, if add { y } else { -y }),
        (Pct(x), Len(y)) => MixedLenPct(if add { y } else { -y }, x),
        (MixedLenPct(px, pc), Len(y)) => MixedLenPct(if add { px + y } else { px - y }, pc),
        (Len(y), MixedLenPct(px, pc)) => MixedLenPct(if add { px + y } else { y - px }, pc),
        (MixedLenPct(px, pc), Pct(y)) => MixedLenPct(px, if add { pc + y } else { pc - y }),
        (Pct(y), MixedLenPct(px, pc)) => MixedLenPct(if add { px } else { -px }, if add { pc + y } else { y - pc }),
        (MixedLenPct(px, pc), MixedLenPct(qx, qc)) => {
            MixedLenPct(if add { px + qx } else { px - qx }, if add { pc + qc } else { pc - qc })
        }
        _ => return None,
    })
}

/// 原子标量缩放（乘除共用；`<t> * <number>` / `<t> / <number>`）。
fn calc_scale(a: CalcAtom, k: f64) -> CalcAtom {
    match a {
        CalcAtom::Num(x) => CalcAtom::Num(x * k),
        CalcAtom::Len(x) => CalcAtom::Len(x * k),
        CalcAtom::Pct(x) => CalcAtom::Pct(x * k),
        CalcAtom::MixedLenPct(x, y) => CalcAtom::MixedLenPct(x * k, y * k),
        CalcAtom::Angle(x) => CalcAtom::Angle(x * k),
        CalcAtom::Time(x) => CalcAtom::Time(x * k),
        CalcAtom::Res(x) => CalcAtom::Res(x * k),
    }
}

/// ×÷ 分型算术（css-values-4 §10.1：乘除单侧标量；除数须标量）。
fn calc_mul_div(a: CalcAtom, b: CalcAtom, mul: bool) -> Option<CalcAtom> {
    if let (CalcAtom::Num(x), CalcAtom::Num(y)) = (a, b) {
        return Some(CalcAtom::Num(if mul { x * y } else { x / y }));
    }
    if mul {
        match (a, b) {
            (CalcAtom::Num(x), other) => Some(calc_scale(other, x)),
            (other, CalcAtom::Num(y)) => Some(calc_scale(other, y)),
            _ => None,
        }
    } else {
        match (a, b) {
            (other, CalcAtom::Num(y)) => Some(calc_scale(other, 1.0 / y)),
            _ => None,
        }
    }
}

/// 注册门 calc 解析求值器（css-values-4 §10 最小子面）：数字+单位原子
///（font-relative 原子即拒）、%/角度/时间/分辨率、`+ - * /` 分型算术、
/// 括号与嵌套 calc()、`var(` 即拒。min()/max()/clamp() 未接
///（FIXME(regprop-calc-fns)：注册门罕见形，语料面未覆盖）。
struct RegisterCalcParser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> RegisterCalcParser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_whitespace() {
            self.i += 1;
        }
    }

    /// expr := term (('+'|'-') term)*。css-values-4 §10.2 要求 `+`/`-` 两侧空白，
    /// 此处放宽为紧邻也可（`2px*4 + 10px` 语料锚定合法面；无空白的 `1px -2px`
    /// 形语料无锚，FIXME(regprop-calc-ws)：如需严格可在 term 返回时保留空白游标）。
    fn parse_expr(&mut self) -> Option<CalcAtom> {
        let mut acc = self.parse_term()?;
        loop {
            self.skip_ws();
            let op = match self.b.get(self.i) {
                Some(b'+') | Some(b'-') => self.b[self.i],
                _ => return Some(acc),
            };
            self.i += 1;
            let rhs = self.parse_term()?;
            acc = calc_add_sub(acc, rhs, op == b'+')?;
        }
    }

    /// term := factor (('*'|'/') factor)*
    fn parse_term(&mut self) -> Option<CalcAtom> {
        let mut acc = self.parse_factor()?;
        loop {
            self.skip_ws();
            let op = match self.b.get(self.i) {
                Some(b'*') => b'*',
                Some(b'/') => b'/',
                _ => return Some(acc),
            };
            self.i += 1;
            let rhs = self.parse_factor()?;
            acc = calc_mul_div(acc, rhs, op == b'*')?;
        }
    }

    /// factor := ('-'|'+')? primary
    fn parse_factor(&mut self) -> Option<CalcAtom> {
        self.skip_ws();
        match self.b.get(self.i) {
            Some(b'-') => {
                self.i += 1;
                Some(calc_scale(self.parse_factor()?, -1.0))
            }
            Some(b'+') => {
                self.i += 1;
                self.parse_factor()
            }
            _ => self.parse_primary(),
        }
    }

    /// primary := '(' expr ')' | calc( expr ) | 数字原子
    fn parse_primary(&mut self) -> Option<CalcAtom> {
        self.skip_ws();
        match self.b.get(self.i) {
            Some(b'(') => {
                self.i += 1;
                let a = self.parse_expr()?;
                self.skip_ws();
                if self.b.get(self.i) != Some(&b')') {
                    return None;
                }
                self.i += 1;
                Some(a)
            }
            Some(&c) if c == b'c' || c == b'C' => {
                // calc( 嵌套（ci）；var( 注册门即拒；其余函数未接。
                let rest = &self.b[self.i..];
                if rest.len() >= 5 && rest[..4].to_ascii_lowercase().starts_with(b"calc") && rest[4] == b'(' {
                    self.i += 5;
                    let a = self.parse_expr()?;
                    self.skip_ws();
                    if self.b.get(self.i) != Some(&b')') {
                        return None;
                    }
                    self.i += 1;
                    Some(a)
                } else {
                    None
                }
            }
            Some(&c) if c.is_ascii_digit() || c == b'.' => self.parse_number_atom(),
            _ => None,
        }
    }

    /// 数字原子：css 数字文法（含指数）+ 可选单位段（ci alpha run / `%`）。
    fn parse_number_atom(&mut self) -> Option<CalcAtom> {
        let start = self.i;
        let mut digits = 0usize;
        while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
            self.i += 1;
            digits += 1;
        }
        if self.b.get(self.i) == Some(&b'.') {
            self.i += 1;
            while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return None;
        }
        // 指数段（`e` 后须有数字，否则回退交单位段——`2e` 非法 css 数字）。
        if matches!(self.b.get(self.i), Some(b'e' | b'E')) {
            let save = self.i;
            self.i += 1;
            if matches!(self.b.get(self.i), Some(b'+' | b'-')) {
                self.i += 1;
            }
            let mut ed = 0usize;
            while matches!(self.b.get(self.i), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
                ed += 1;
            }
            if ed == 0 {
                self.i = save;
            }
        }
        let num: f64 = std::str::from_utf8(&self.b[start..self.i]).ok()?.parse().ok()?;
        let ustart = self.i;
        while matches!(self.b.get(self.i), Some(c) if c.is_ascii_alphabetic()) {
            self.i += 1;
        }
        let unit = std::str::from_utf8(&self.b[ustart..self.i]).ok()?.to_ascii_lowercase();
        // `%` 单字单位（非 alpha——alpha 单位段后独立判定）。
        if unit.is_empty() && self.b.get(self.i) == Some(&b'%') {
            self.i += 1;
            return Some(CalcAtom::Pct(num));
        }
        Some(if unit.is_empty() {
            CalcAtom::Num(num)
        } else if unit == "%" {
            CalcAtom::Pct(num)
        } else if let Some(a) = angle_unit_to_deg(num, &unit) {
            CalcAtom::Angle(a)
        } else if let Some(t) = time_unit_to_sec(num, &unit) {
            CalcAtom::Time(t)
        } else if let Some(r) = resolution_unit_to_dppx(num, &unit) {
            CalcAtom::Res(r)
        } else {
            CalcAtom::Len(length_unit_to_px(num, &unit)?)
        })
    }
}

/// 注册门 calc() 分型求值入口：整值须 `calc( … )` 单函数形（ci 前缀、尾 `)`），
/// 表达式须整段消费。`calc(1px) junk` 残余 → None。
fn calc_eval_strict(value: &str) -> Option<CalcAtom> {
    let t = value.trim();
    let b = t.as_bytes();
    if !(b.len() >= 6 && b[..4].to_ascii_lowercase().as_slice() == b"calc" && b[4] == b'(' && b[b.len() - 1] == b')') {
        return None;
    }
    let mut p = RegisterCalcParser {
        b: &b[5..b.len() - 1],
        i: 0,
    };
    let a = p.parse_expr()?;
    p.skip_ws();
    if p.i != p.b.len() {
        return None;
    }
    Some(a)
}

/// 注册门 length 原子（raw 非 calc 形）：parse_length 全文法 + font-relative
/// 族拒 + % 仅 length-percentage 放行（`10%` for `<length>` 拒、`0`/`10vmin` 收）
/// + 视口/容器族名义视口折 px。
fn strict_length_atom(s: &str, allow_percent: bool) -> Option<CalcAtom> {
    use zero_css_parser::values::LengthValue as L;
    let v = zero_css_parser::values::parse_length(s.trim())?;
    Some(match v {
        L::Px(n) => CalcAtom::Len(n),
        L::Percentage(n) if allow_percent => CalcAtom::Pct(n),
        L::Percentage(_) => return None,
        // font-relative 族（css-values-4 §6.2）：注册门拒——初始值须与元素
        // 字体上下文无关（`10em`/`calc(2ex + 16px)`/`calc(4px + 3em)` 语料锚定）。
        L::Em(_)
        | L::Ex(_)
        | L::Rex(_)
        | L::Cap(_)
        | L::Rcap(_)
        | L::Rem(_)
        | L::Ch(_)
        | L::Rch(_)
        | L::Ic(_)
        | L::Ric(_)
        | L::Lh(_)
        | L::Rlh(_) => return None,
        other => CalcAtom::Len(viewport_length_to_px(other)?),
    })
}

/// raw LengthValue 视口/容器族折 px（名义视口；非视口族 None）。
fn viewport_length_to_px(v: zero_css_parser::values::LengthValue) -> Option<f64> {
    use zero_css_parser::values::LengthValue as L;
    let (w, h) = REGISTER_NOMINAL_VIEWPORT;
    Some(match v {
        L::Vw(n)
        | L::Svw(n)
        | L::Lvw(n)
        | L::Dvw(n)
        | L::Cqw(n)
        | L::Cqi(n)
        | L::Vi(n)
        | L::Svi(n)
        | L::Lvi(n)
        | L::Dvi(n) => n * w / 100.0,
        L::Vh(n)
        | L::Svh(n)
        | L::Lvh(n)
        | L::Dvh(n)
        | L::Cqh(n)
        | L::Cqb(n)
        | L::Vb(n)
        | L::Svb(n)
        | L::Lvb(n)
        | L::Dvb(n) => n * h / 100.0,
        L::Vmin(n) | L::Svmin(n) | L::Lvmin(n) | L::Dvmin(n) | L::Cqmin(n) => n * w.min(h) / 100.0,
        L::Vmax(n) | L::Svmax(n) | L::Lvmax(n) | L::Dvmax(n) | L::Cqmax(n) => n * w.max(h) / 100.0,
        _ => return None,
    })
}

/// css-syntax-3 §4.3.10 数字 token 文法（`1.0`/`1e0` 非整数、`2.3e4`/`.5` 合法）。
fn valid_number_token(s: &str) -> bool {
    let t = s.trim();
    let mut chars = t.chars().peekable();
    // 数 token 号符（`-109` 语料锚定；与 valid_integer_token 同形）。
    if matches!(chars.peek(), Some('+' | '-')) {
        chars.next();
    }
    let mut digits = 0usize;
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
        chars.next();
        digits += 1;
    }
    if chars.peek() == Some(&'.') {
        chars.next();
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            chars.next();
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if matches!(chars.peek(), Some('e' | 'E')) {
        chars.next();
        if matches!(chars.peek(), Some('+' | '-')) {
            chars.next();
        }
        let mut ed = 0usize;
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            chars.next();
            ed += 1;
        }
        if ed == 0 {
            return false;
        }
    }
    chars.peek().is_none()
}

/// css-syntax-3 §4.3.11 整数 token 文法（仅符号+数字——`1.0`/`1e0` 拒锚定）。
fn valid_integer_token(s: &str) -> bool {
    let t = s.trim();
    let mut chars = t.chars().peekable();
    if matches!(chars.peek(), Some('+' | '-')) {
        chars.next();
    }
    let mut digits = 0usize;
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
        chars.next();
        digits += 1;
    }
    digits > 0 && chars.peek().is_none()
}

/// CSS 宽关键字五枚（值面 ci 禁用；`default` 仅 <custom-ident> 值面追加——
/// `*`/`default` 合法 + `<custom-ident>`/`default` 非法语料双锚）。
fn css_wide_keyword_value(s: &str) -> bool {
    matches!(
        s.to_ascii_lowercase().as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}

/// <custom-ident> 值面（css-values-4 §4.2.1）：转义解码后 ident 字符集 + CSS
/// 宽关键字与 `default` ci 禁（`foo initial bar` 逐项锚定）。
fn valid_custom_ident_value(s: &str) -> bool {
    let decoded = decode_css_ident(s.trim());
    if decoded.is_empty() {
        return false;
    }
    let mut chars = decoded.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '-' || c as u32 >= 0x80 => {}
        _ => return false,
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c as u32 >= 0x80) {
        return false;
    }
    !(css_wide_keyword_value(&decoded) || decoded.eq_ignore_ascii_case("default"))
}

/// <string> 值面：整值恰为一个字符串 token（css-syntax-3 §4.3.2/§4.3.4）——
/// 首非引号拒；串后残余 token 拒（`"foo" bar` 锚定）；EOF 未闭合合法
///（`'foo bar` 锚定）；串内裸换行 bad（`"\n` 锚定）。
fn valid_string_token(s: &str) -> bool {
    let t = s.trim();
    let b = t.as_bytes();
    if b.is_empty() || (b[0] != b'"' && b[0] != b'\'') {
        return false;
    }
    let mut i = 0usize;
    if !scan_string_token(b, &mut i) {
        return false;
    }
    b[i..].iter().all(|&c| (c as char).is_whitespace())
}

/// <url> 值面（css-values-4 §3.3 子面）：`url( … )` 未引号 token 形/引号函数
/// 形 ∪ 裸字符串形；裸 ident 拒（`banana.png` 锚定）。
fn valid_url_value(s: &str) -> bool {
    let t = s.trim();
    let b = t.as_bytes();
    if b.len() >= 5 && b[..3].to_ascii_lowercase().as_slice() == b"url" {
        let mut j = 3;
        while j < b.len() && (b[j] as char).is_whitespace() {
            j += 1;
        }
        if j < b.len() && b[j] == b'(' {
            let mut i = j + 1;
            let mut stack = Vec::new();
            if !scan_url_after_open(b, &mut i, &mut stack) {
                return false;
            }
            if i == b.len() {
                return true;
            }
            // 引号函数形：`url(` 已入栈，余段（串 token + 闭括号/后续）交通用
            // 尾扫描收尾——custom-property-animation-image/url `url("https://…")`
            // 语料锚定（单发 `i == b.len()` 判定会漏扫余段）。
            return scan_declaration_tail(b, &mut i, &mut stack);
        }
    }
    valid_string_token(t)
}

/// <image> 值面（css-images-3 §3 <image> 生产式子面）：url() 形 ∪ 渐变/图像
/// 函数族 ident( 非空参数 )；裸 ident（`banana.png`）与 `none` 关键字拒。
fn valid_image_value(s: &str) -> bool {
    let t = s.trim();
    if valid_url_value(t) {
        return true;
    }
    let b = t.as_bytes();
    if b.is_empty() || !(b[0].is_ascii_alphabetic() || b[0] == b'-' || b[0] == b'_' || b[0] >= 0x80) {
        return false;
    }
    let mut i = 0usize;
    while i < b.len() && is_ident_char_byte(b[i]) {
        i += 1;
    }
    let name = t[..i].to_ascii_lowercase();
    if !matches!(
        name.as_str(),
        "linear-gradient"
            | "radial-gradient"
            | "conic-gradient"
            | "repeating-linear-gradient"
            | "repeating-radial-gradient"
            | "repeating-conic-gradient"
            | "image"
            | "image-set"
            | "paint"
            | "cross-fade"
            | "element"
            | "light-dark"
            | "-webkit-gradient"
            | "-webkit-linear-gradient"
            | "-webkit-radial-gradient"
            | "-webkit-cross-fade"
            | "-webkit-image-set"
    ) || i >= b.len()
        || b[i] != b'('
    {
        return false;
    }
    valid_bracketed_args(t, i)
}

/// <transform-function> 值面：ident( 非空参数 balanced )（`scale()` 空参拒）。
fn valid_transform_function(s: &str) -> bool {
    let t = s.trim();
    let b = t.as_bytes();
    if b.is_empty() || !(b[0].is_ascii_alphabetic() || b[0] == b'-' || b[0] == b'_' || b[0] >= 0x80) {
        return false;
    }
    let mut i = 0usize;
    while i < b.len() && is_ident_char_byte(b[i]) {
        i += 1;
    }
    if i >= b.len() || b[i] != b'(' {
        return false;
    }
    valid_bracketed_args(t, i)
}

/// `ident(` 后参数段门：括号平衡（字符串感知）+ 参数非空白空 + 尾 `)` 后无残余。
/// i 入参指向 `(` 位置。
fn valid_bracketed_args(t: &str, open: usize) -> bool {
    let b = t.as_bytes();
    let mut depth = 0i32;
    let mut saw_value = false;
    let mut i = open;
    while i < b.len() {
        let c = b[i];
        if c == b'"' || c == b'\'' {
            if !scan_string_token(b, &mut i) {
                return false;
            }
            saw_value = true;
            continue;
        }
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return saw_value && t[i + 1..].trim().is_empty();
                }
                if depth < 0 {
                    return false;
                }
            }
            _ if !(c as char).is_whitespace() => saw_value = true,
            _ => {}
        }
        i += 1;
    }
    false
}

/// <transform-list> 值面：≥1 个空白分隔 transform-function（token 感知切分——
/// 函数参数内空白不分隔；`translateX(2px) rotate(20deg)` 双函数合法）。
fn valid_transform_list_value(s: &str) -> bool {
    let items = split_value_items(s, false);
    !items.is_empty() && items.iter().all(|it| valid_transform_function(it))
}

/// 顶层分隔切分（字符串/括号感知）：comma=true 按 `,`（depth0、串外），false
/// 按空白 run。逗号形保留空段（空段下游严格门必拒）；空白形滤空段。
fn split_value_items(s: &str, comma: bool) -> Vec<&str> {
    let b = s.as_bytes();
    let mut items: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c == b'"' || c == b'\'' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 1;
                    if i < b.len() {
                        i += utf8_char_len(b[i]);
                    }
                    continue;
                }
                let q = b[i];
                i += 1;
                if q == c || q == b'\n' || q == b'\r' || q == 0x0C {
                    break;
                }
            }
            continue;
        }
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        let is_sep = depth == 0 && (if comma { c == b',' } else { (c as char).is_whitespace() });
        if is_sep {
            let piece = s[start..i].trim();
            if comma || !piece.is_empty() {
                items.push(piece);
            }
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }
    let last = s[start..].trim();
    if comma || !last.is_empty() {
        items.push(last);
    }
    items
}

/// CSS 注释剥离（css-syntax-3 §4.2：`/* … */` → 单空白）。字符串 token 内
/// `/*` 非注释（字符串感知）；未闭合注释剥至 EOF。
fn strip_css_comments(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c == b'"' || c == b'\'' {
            let start = i;
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 1;
                    if i < b.len() {
                        i += utf8_char_len(b[i]);
                    }
                    continue;
                }
                let q = b[i];
                i += 1;
                if q == c || q == b'\n' || q == b'\r' || q == 0x0C {
                    break;
                }
            }
            out.push_str(s.get(start..i).unwrap_or(&s[start..]));
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                i += 1;
            }
            if i < b.len() {
                i += 2;
            }
            out.push(' ');
        } else {
            let end = (i + utf8_char_len(c)).min(b.len());
            // end 恒字符边界（合法 UTF-8 首字节 + char_len）；unwrap_or 的兜底
            // 参数急切求值会先切 s[i..i+1]，多字节字符（🤔）即越界 panic——直接切。
            out.push_str(&s[i..end]);
            i = end;
        }
    }
    out
}

/// 值内 var() 引用探测（initial-value 禁 var()——`var(--foo)`/`var(invalid var
/// ref)`/`foo var(--foo, bla)` 语料锚定；ci 函数名；字符串内不触发）。
fn contains_var_function(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let b = lower.as_bytes();
    let mut i = 0usize;
    let mut in_str: u8 = 0;
    while i < b.len() {
        let c = b[i];
        if in_str != 0 {
            if c == b'\\' {
                i += 1;
                if i < b.len() {
                    i += utf8_char_len(b[i]);
                }
                continue;
            }
            if c == in_str {
                in_str = 0;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' => {
                in_str = c;
                i += 1;
            }
            b'v' if b[i..].starts_with(b"var(") => return true,
            _ => i += 1,
        }
    }
    false
}

/// 字符串 token 扫描（css-syntax-3 §4.3.2 consume-string-token）：`\` 转义消费
/// 一完整字符（`\`+换行 = 行续接，不判 bad）；裸换行 = bad-string false；
/// EOF 未闭合 = 合法 true（`"` 单引号字符语料锚定）。出参 i = token 后字节。
fn scan_string_token(b: &[u8], i: &mut usize) -> bool {
    let quote = b[*i];
    *i += 1;
    while *i < b.len() {
        let c = b[*i];
        if c == b'\\' {
            *i += 1;
            if *i < b.len() {
                *i += utf8_char_len(b[*i]);
            }
            continue;
        }
        if c == quote {
            *i += 1;
            return true;
        }
        if c == b'\n' || c == b'\r' || c == 0x0C {
            return false;
        }
        *i += 1;
    }
    true
}

/// `url(` 后内容门（css-syntax-3 §4.3.6 consume-url-token）：前导空白后引号 =
/// 函数形（url( 本体入括号栈，交回通用扫描）；未引号内容：引号/`(` 即 bad、
/// 空白后须直达 `)`（`url(moo '')` bad-url 锚定）、EOF = bad。
fn scan_url_after_open(b: &[u8], i: &mut usize, stack: &mut Vec<u8>) -> bool {
    let mut j = *i;
    while j < b.len() && (b[j] as char).is_whitespace() {
        j += 1;
    }
    if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
        stack.push(b'(');
        *i = j;
        return true;
    }
    let mut content = false;
    while j < b.len() {
        let c = b[j];
        if c == b')' {
            *i = j + 1;
            return true;
        }
        if c == b'"' || c == b'\'' || c == b'(' {
            return false;
        }
        if (c as char).is_whitespace() {
            if !content {
                return false;
            }
            while j < b.len() && (b[j] as char).is_whitespace() {
                j += 1;
            }
            if j < b.len() && b[j] == b')' {
                *i = j + 1;
                return true;
            }
            return false;
        }
        if c == b'\\' {
            j += 1;
            if j >= b.len() {
                return false;
            }
            j += utf8_char_len(b[j]);
            content = true;
            continue;
        }
        content = true;
        j += 1;
    }
    false
}

/// ident 字符（css-syntax-3 §4.3.3 `<ident-char>` 子面——ASCII 段 + U+0080 以上）。
fn is_ident_char_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c >= 0x80
}

/// universal initial-value 的 <declaration-value> 门（css-syntax-3 §5.2 生产式
/// 最小面 + declaration 上下文坏面）：bad-string（串内裸换行）、bad-url、不对应
/// 闭括号（`([)]` 锚定）、顶层 `;`（`semi;colon` 锚定）、顶层 `!`（`whee!` 锚定）
/// → false；未匹配开括号合法（`unmatched opening bracket is valid :(` 锚定）；
/// `url(` + 空白 + 引号 = 函数形合法。
fn valid_declaration_value(s: &str) -> bool {
    let b = s.as_bytes();
    let mut stack: Vec<u8> = Vec::new();
    let mut i = 0usize;
    scan_declaration_tail(b, &mut i, &mut stack)
}

/// 通用值尾扫描（`<declaration-value>` 循环体共享面）：串 token 跳读、url( ci
/// 特判交 [`scan_url_after_open`]、括号栈配对、顶层 `;`/`!` 拒。调用方可预置
/// 栈与 i（valid_url_value 引号函数形续扫）。
fn scan_declaration_tail(b: &[u8], i: &mut usize, stack: &mut Vec<u8>) -> bool {
    while *i < b.len() {
        let c = b[*i];
        match c {
            b'"' | b'\'' => {
                if !scan_string_token(b, i) {
                    return false;
                }
            }
            _ if c.is_ascii_alphabetic() || c == b'_' || c == b'-' || c >= 0x80 => {
                // ident 起始：url( 特判（ci，§4.3.6 url token）；其余 ident 跳过。
                let start = *i;
                while *i < b.len() && is_ident_char_byte(b[*i]) {
                    *i += 1;
                }
                if b[start..*i].to_ascii_lowercase().as_slice() == b"url" && *i < b.len() {
                    let mut j = *i;
                    while j < b.len() && (b[j] as char).is_whitespace() {
                        j += 1;
                    }
                    if j < b.len() && b[j] == b'(' {
                        *i = j + 1;
                        if !scan_url_after_open(b, i, stack) {
                            return false;
                        }
                    }
                }
            }
            b'(' | b'[' | b'{' => {
                stack.push(c);
                *i += 1;
            }
            b')' | b']' | b'}' => {
                let open = match c {
                    b')' => b'(',
                    b']' => b'[',
                    _ => b'{',
                };
                if stack.pop() != Some(open) {
                    return false;
                }
                *i += 1;
            }
            b';' if stack.is_empty() => return false,
            b'!' if stack.is_empty() => return false,
            _ => *i += 1,
        }
    }
    true
}

/// UTF-8 首字节 → 完整字符字节长（续字节/非法首字节按 1 容忍——扫描面不做
/// 严格编码校验，&str 输入已保证合法编码）。
fn utf8_char_len(c: u8) -> usize {
    match c {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// 注册门单值严格校验（css-properties-values-api-1 §supported-syntax-strings
/// 分型子面 + 语料硬化）。raw 须已注释剥离；返回 canonical 串（与 lenient
/// 计算面同 format 口径——注册门只加严接收性，不改串化形）。
fn strict_scalar(kind: TypeKind, raw: &str) -> Option<String> {
    let v = raw.trim();
    match kind {
        TypeKind::Integer => {
            // 整数 token 裸面（`1.0`/`1e0` 拒）；calc 数字族放行
            //（`calc(3.1415)` 语料锚定合法——used-time 取整）。
            if valid_integer_token(v) {
                return Some(format!("{}", v.parse::<f64>().ok()?.round() as i64));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Num(n) => Some(format!("{}", n.round() as i64)),
                _ => None,
            }
        }
        TypeKind::Number => {
            if valid_number_token(v) {
                return Some(super::computed_style::format_num(v.parse::<f64>().ok()?, ""));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Num(n) => Some(super::computed_style::format_num(n, "")),
                _ => None,
            }
        }
        TypeKind::Length | TypeKind::LengthPercentage => {
            let allow_pct = kind == TypeKind::LengthPercentage;
            let a = match calc_eval_strict(v) {
                Some(a) => a,
                None => strict_length_atom(v, allow_pct)?,
            };
            match a {
                CalcAtom::Len(n) => Some(super::computed_style::format_num(n, "px")),
                CalcAtom::Pct(n) if allow_pct => Some(super::computed_style::format_num(n, "%")),
                CalcAtom::MixedLenPct(..) if allow_pct => Some(v.to_string()),
                _ => None,
            }
        }
        TypeKind::Percentage => {
            if let Some(num) = v.strip_suffix('%') {
                let n = num.trim().parse::<f64>().ok()?;
                return Some(super::computed_style::format_num(n, "%"));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Pct(n) => Some(super::computed_style::format_num(n, "%")),
                _ => None,
            }
        }
        TypeKind::Angle => {
            if let Some(deg) = parse_angle(v) {
                return Some(super::computed_style::format_num(deg, "deg"));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Angle(deg) => Some(super::computed_style::format_num(deg, "deg")),
                _ => None,
            }
        }
        TypeKind::Time => {
            if let Some(sec) = parse_time(v) {
                return Some(super::computed_style::format_num(sec, "s"));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Time(sec) => Some(super::computed_style::format_num(sec, "s")),
                _ => None,
            }
        }
        TypeKind::Resolution => {
            if let Some(dppx) = parse_resolution(v) {
                return Some(super::computed_style::format_num(dppx, "dppx"));
            }
            match calc_eval_strict(v)? {
                CalcAtom::Res(dppx) => Some(super::computed_style::format_num(dppx, "dppx")),
                _ => None,
            }
        }
        // 色面复用 lenient 通道（parse_color 全色面 + rgb/rgba canonical 同形）。
        TypeKind::Color => canonicalize_scalar(TypeKind::Color, v, 16.0),
        TypeKind::TransformFunction => valid_transform_function(v).then(|| v.to_string()),
        TypeKind::TransformList => valid_transform_list_value(v).then(|| v.to_string()),
        TypeKind::StringKind => valid_string_token(v).then(|| v.to_string()),
        TypeKind::CustomIdent => valid_custom_ident_value(v).then(|| v.to_string()),
        TypeKind::UrlKind => valid_url_value(v).then(|| v.to_string()),
        TypeKind::ImageKind => valid_image_value(v).then(|| v.to_string()),
    }
}

/// 注册门分量校验（列表 token 感知切分逐项 [`strict_scalar`]；ident 字面量
/// 解码后大小写敏感全等——`Foo | bar`/`foo` 拒、`bAnAnA`/`bAnAnA` 收锚定）。
fn strict_component(comp: &SyntaxComponent, raw: &str) -> Option<String> {
    match comp.kind {
        None => {
            // ident 字面量：Single 整值解码全等；列表形 token 感知切分逐项
            // 解码全等（`foo+`/`foo foo foo`、`--foo+`/`--foo --foo` 锚定）。
            match comp.list {
                ListMarker::Single => {
                    let decoded = decode_css_ident(raw.trim());
                    (decoded == comp.ident).then(|| raw.trim().to_string())
                }
                list => {
                    let parts: Vec<&str> = split_value_items(raw, list == ListMarker::Comma);
                    if parts.is_empty() {
                        return None;
                    }
                    parts
                        .iter()
                        .all(|p| decode_css_ident(p) == comp.ident)
                        .then(|| raw.trim().to_string())
                }
            }
        }
        Some(kind) => match comp.list {
            ListMarker::Single => strict_scalar(kind, raw),
            ListMarker::Space => {
                let parts: Vec<&str> = split_value_items(raw, false);
                if parts.is_empty() {
                    return None;
                }
                parts
                    .iter()
                    .map(|p| strict_scalar(kind, p))
                    .collect::<Option<Vec<_>>>()
                    .map(|v| v.join(" "))
            }
            ListMarker::Comma => {
                let parts: Vec<&str> = split_value_items(raw, true);
                if parts.is_empty() {
                    return None;
                }
                parts
                    .iter()
                    .map(|p| strict_scalar(kind, p))
                    .collect::<Option<Vec<_>>>()
                    .map(|v| v.join(", "))
            }
        },
    }
}

/// 注册 initial-value 严格门（css-properties-values-api-1 §register 值可解析 +
/// register-property-syntax-parsing 语料硬化）：var() 引用禁；universal 形走
/// <declaration-value> 生产式 + CSS 宽关键字整值禁；combined 形逐备选严格分型。
/// 返回 canonical 串。与 lenient 计算面分离——声明值消费（[`computed_value`]）
/// 不经本门（var() 已由样式引擎替换、em→px 按元素上下文求值）。
fn validate_initial_strict(syntax: &SyntaxDescriptor, value: &str) -> Option<String> {
    let stripped = strip_css_comments(value);
    if contains_var_function(&stripped) {
        return None;
    }
    match syntax {
        SyntaxDescriptor::Universal => {
            // CSS 宽关键字整值判按 trim 形；坏 token 扫描用原值（`"\n` 尾随
            // 换行属串内证据——trim 会把 bad-string 吃成 EOF 合法串，语料锚定
            // 非法面）。
            if css_wide_keyword_value(stripped.trim()) {
                return None;
            }
            valid_declaration_value(&stripped).then(|| stripped.trim().to_string())
        }
        SyntaxDescriptor::Normal => None,
        SyntaxDescriptor::Combined(components) => components.iter().find_map(|comp| strict_component(comp, &stripped)),
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
        // 引号串/ident/url()/image：离散插值（css-properties-values-api-1 §animation
        // 「either the value is interpolated or discrete」——四子型无连续轴）。
        TypeKind::StringKind | TypeKind::CustomIdent | TypeKind::UrlKind | TypeKind::ImageKind => None,
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

    /// 刀 RP1 注册门残面钉（register-property-syntax-parsing 语料锚定）——
    /// valid 群（曾拒今收：calc 分型/视口族/注释/转义 ident/虚线语法）与
    /// invalid 群（曾收今拒：CSS 宽关键字值/declaration 坏 token/typed 失配）。
    #[test]
    fn test_register_gate_strict_faces() {
        clear_all();
        // —— valid 群
        let valid = [
            ("<length>", "10px /*:)*/"),
            ("<length>", " calc(-2px)"),
            ("<length>", "calc(2px*4 + 10px)"),
            ("<length>", "calc(7in - 12px)"),
            ("<length>+", "2px 7px calc(8px)"),
            ("<length>#", "2px, 7px, calc(8px)"),
            ("<length-percentage>", "calc(-11px + 10.4%)"),
            ("<length>", "10vmin"),
            ("<percentage> | <length>+", "calc(100vh - 10px) 30px"),
            ("<number>", "calc(1 / 2)"),
            ("<integer>", "calc(1)"),
            ("<integer>", "calc(3.1415 + 3.1415)"),
            ("<angle>", "calc(50grad + 3.14159rad)"),
            ("<time>", "calc(2s - 9ms)"),
            ("<resolution>", "3dPpX"),
            ("banana", "banan\\61"),
            ("foo+|bar", "foo foo foo"),
            ("--foo+", "--foo --foo"),
            ("<string>", "'foo bar"),
            ("<string># | <string>+", "\"foo\", 'bar"),
            ("<image>", "linear-gradient(yellow, blue)"),
            ("<image>", "light-dark(none, none)"),
            // 引号 url 函数形（custom-property-animation-image/url 电池 P2F 钉——
            // 单发 `i == b.len()` 判定漏扫引号余段；语料 url(a) 未引号形之外）。
            ("<image>", "url(\"https://example.com/from\")"),
            ("<url>", "url(\"https://example.com/to\")"),
            ("<image>+", "url(a) url(\"b\")"),
            ("<transform-list>", "translateX(2px) rotate(20deg)"),
            ("<custom-ident>", "banan\\61"),
            ("<length>", "0"),
            ("<number>", "-109"),
            // 转义 ident + 字面 emoji 值面（语料 \1F914 族；strip_css_comments
            // 多字节边界回归钉——unwrap_or 兜底急切求值曾在 emoji 上 panic）。
            ("\\1F914", "🤔"),
            ("hmm\\1F914", "hmm🤔"),
            ("\\1F914hmm", "🤔hmm"),
            ("\\1F914\\1F914", "🤔🤔"),
            ("\\1F914 hmm", "🤔hmm"),
            // 缺省 syntax = '*'（WebIDL 字典缺省 universal 面）。
            ("", ""),
        ];
        for (i, (syntax, value)) in valid.iter().enumerate() {
            let name = format!("--gv{}", i);
            let r = if syntax.is_empty() {
                register(&name, None, true, false, true, Some("undefined"))
            } else {
                register(&name, Some(syntax), true, false, true, Some(value))
            };
            assert!(r.is_ok(), "valid group #{}: {} / {}", i, syntax, value);
        }
        clear_all();
        // —— invalid 群
        let invalid = [
            (Some("*"), "initial"),
            (Some("*"), "inherit"),
            (Some("*"), "unset"),
            (Some("*"), "revert"),
            (Some("*"), "revert-layer"),
            (Some("*"), ")"),
            (Some("*"), "([)]"),
            (Some("*"), "whee!"),
            (Some("*"), "\"\n"),
            (Some("*"), "url(moo '')"),
            (Some("*"), "semi;colon"),
            (Some("*"), "var(invalid var ref)"),
            (Some("*"), "var(--foo)"),
            (Some("<custom-ident>"), "default"),
            (Some("<custom-ident>"), "INHERIT"),
            (Some("<custom-ident>+"), "foo initial bar"),
            (Some("<length>"), "var(--moo)"),
            (Some("<length>"), "10%"),
            (Some("<length>"), "10em"),
            (Some("<length>"), "calc(5px + 10%)"),
            (Some("<length>"), "calc(4px + 3em)"),
            (Some("<length>"), "10"),
            (Some("<length>+"), ""),
            (Some("<length-percentage>"), "calc(2px + 10% + 7ex)"),
            (Some("<percentage>"), "0"),
            (Some("<integer>"), "1.0"),
            (Some("<integer>"), "1e0"),
            (Some("<resolution>"), "-5.3dpcm"),
            (Some("<transform-function>"), "scale()"),
            (Some("<transform-list>"), "scale()"),
            (Some("<image>"), "banana.png"),
            (Some("<image>"), "none"),
            (Some("<url>"), "banana.png"),
            (Some("<string>"), "foo"),
            (Some("<string>"), "\"foo\" bar"),
            (Some("<string>"), "'foo' 2px"),
            (Some("<string>+"), "foo 'bar'"),
            (Some("<number>|foo"), "foo var(--foo, bla)"),
            (Some("Foo | bar"), "foo"),
            (Some("banana"), "bAnAnA"),
            // 乘子须紧邻分量（`<length> +` syntax 即非法）。
            (Some("<length> +"), "10px"),
            // 备选混合 run 逐项同族门（`1px 1%` 非 length+ 单族）。
            (Some("<integer>+ | <percentage>+ | <length>+ "), "1px 1%"),
            (Some("<integer>+ | <percentage>+ | <length>+ "), "1% 1px"),
            // 禁名 ci 面（语料 INHERIT/unsEt/REVert/deFAUlT 四形 + 裸 default
            // 语法分量禁；小写形由 valid_ident_literal 同门覆盖）。
            (Some("initial"), "initial"),
            (Some("<length>|INHERIT"), "10px"),
            (Some("<percentage>|unsEt"), "2%"),
            (Some("<color>|REVert"), "red"),
            (Some("<integer>|deFAUlT"), "1"),
            (Some("default"), "default"),
        ];
        for (i, (syntax, value)) in invalid.iter().enumerate() {
            let name = format!("--gi{}", i);
            let r = register(&name, *syntax, true, false, true, Some(value));
            assert!(r.is_err(), "invalid group #{}: {:?} / {}", i, syntax, value);
        }
        // 同 syntax 串合法形对照（`<integer>+ | <percentage>+ | <length>+ ` 带尾
        // 空白合法——整值单族 run 收）。
        assert!(
            register(
                "--gv-homogeneous",
                Some("<integer>+ | <percentage>+ | <length>+ "),
                true,
                false,
                true,
                Some("1")
            )
            .is_ok()
        );
        clear_all();
        // universal 坏形放行对照：`default` 非宽关键字值、未匹配开括号合法。
        assert!(register("--gu1", Some("*"), true, false, true, Some("default")).is_ok());
        assert!(
            register(
                "--gu2",
                Some("*"),
                true,
                false,
                true,
                Some("unmatched opening bracket is valid :(")
            )
            .is_ok()
        );
        clear_all();
    }
}
