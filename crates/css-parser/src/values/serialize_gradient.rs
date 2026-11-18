//! 渐变声明值 canonical 串化（css-images-4
//! [§serializing-gradients](https://drafts.csswg.org/css-images-4/#serializing-gradients)）。
//!
//! canonical 序：`<func>(<配置原文> [in <method>][, <色标段>])`——方向/形状配置
//! 前置（as-written 原序保留，css-images-4 `to <side-or-corner>` `||` 序）、
//! `in <method>` 后置；声明法等于站型隐含缺省法时整子句冗余省略
//!（[#gradient-color-interpolation](https://drafts.csswg.org/css-images-4/#gradient-color-interpolation)：
//! 站含 predefined 色 `color()` → 缺省 oklab，否则 srgb）；xyz → xyz-d65 别名
//! 展开（css-color-4 §9.1）；shorter hue 缺省省略（css-color-4 §12.2）。
//!
//! 切片边界：色标段 as-written 透传（FIXME: 色标级 canonical——颜色/位置逐
//! token 归一——归 computed 串化刀）；其余非渐变面（none/url/filter/
//! light-dark/多层混合）返 None passthrough 零回归（g24 口径：文法 invalid
//! 才拒，未支持面不接管）。image-set 面 g45 起接管（同文件
//! `image_set_specified_to_css`；options AST 在 parse_extended_visual）。

use super::parse_extended_visual::{
    BackgroundImageValue, ImageSetOptionValue, ImageSetResolution, ImageSetValue, parse_background_image,
};
use super::parse_transform::{GradientSpecifiedParts, GradientValue};
use super::serialize_math::format_dimension_inner;

/// 渐变值 canonical 串化：`GradientValue` + 解析期捕获的
/// [`GradientSpecifiedParts`] 原文 → canonical 声明串。
///
/// `specified` 为 None（程序化构造）或方法未识别 → None（调用方 passthrough）。
pub fn serialize_gradient_specified_value(g: &GradientValue) -> Option<String> {
    let (func, parts): (&str, &GradientSpecifiedParts) = match g {
        GradientValue::Linear(l) => (
            if l.repeating {
                "repeating-linear-gradient"
            } else {
                "linear-gradient"
            },
            l.specified.as_ref()?,
        ),
        GradientValue::Radial(r) => (
            if r.repeating {
                "repeating-radial-gradient"
            } else {
                "radial-gradient"
            },
            r.specified.as_ref()?,
        ),
        GradientValue::Conic(c) => (
            if c.repeating {
                "repeating-conic-gradient"
            } else {
                "conic-gradient"
            },
            c.specified.as_ref()?,
        ),
    };
    let mut out = String::from(func);
    out.push('(');
    let mut emitted = false;
    if let Some(h) = &parts.head {
        out.push_str(h);
        emitted = true;
    }
    if let Some(m) = &parts.method {
        if !method_is_redundant_default(m, &parts.body) {
            if emitted {
                out.push(' ');
            }
            out.push_str(m);
            emitted = true;
        }
    }
    if emitted && !parts.body.is_empty() {
        out.push_str(", ");
    }
    out.push_str(&parts.body);
    out.push(')');
    Some(out)
}

/// 声明法是否为站型隐含缺省（css-images-4 #gradient-color-interpolation）：
/// canonical 文本形如 `in <space>[ <hue> hue]`——带显式色相法词（解析期已省略
/// shorter 缺省）即非缺省；否则空间 ==（站含 `color(` predefined → oklab，
/// 否则 srgb）时整子句冗余省略。
fn method_is_redundant_default(method: &str, body: &str) -> bool {
    let mut words = method.split_whitespace();
    if words.next() != Some("in") {
        return false;
    }
    let space = match words.next() {
        Some(s) => s,
        None => return false,
    };
    if words.next().is_some() {
        return false; // 显式 hue 法恒非缺省
    }
    let default = if body_has_predefined_stop(body) {
        "oklab"
    } else {
        "srgb"
    };
    space == default
}

/// 色标段是否含 predefined 色站（顶层段前缀 `color(`，大小写不敏感；
/// css-color-4 §9 predefined RGB/XYZ/D55 家族——legacy rgb()/hex/named 均不命中）。
fn body_has_predefined_stop(body: &str) -> bool {
    body.split(',')
        .any(|seg| seg.trim().to_ascii_lowercase().starts_with("color("))
}

/// background-image 族声明值 canonical 入口（css-images-4 §serializing-gradients）：
/// 逐顶层图层 parse→渐变层 canonical 串化 + image-set 层 canonical 串化
///（g45：`-webkit-` 前缀落标准形、url 引号化、缺省分辨率补 `1x`、type()
/// 引号序归一、calc 分辨率折叠）；任一层为其余非渐变面（none/url/filter()/
/// light-dark()/文法 invalid）→ 整值 None passthrough。
pub fn canonicalize_background_image_specified(value: &str) -> Option<String> {
    let layers = split_top_layers(value.trim());
    let mut out = Vec::with_capacity(layers.len());
    for layer in layers {
        let v = layer.trim();
        if v.is_empty() {
            return None;
        }
        let parsed = parse_background_image(v)?;
        match &parsed {
            BackgroundImageValue::Gradient(g) => out.push(serialize_gradient_specified_value(g)?),
            // g45：image-set 层投影串化恒 Some(canonical)（已 canonical 形恒
            // 等回放——dispatcher 钉「no-calc form projects canonically」锚定）；
            // None passthrough 仅余非图像集面。
            BackgroundImageValue::ImageSet(is) => out.push(image_set_specified_to_css(is)?),
            // 其余非渐变面 passthrough（见模块头注切片边界）。
            _ => return None,
        }
    }
    Some(out.join(", "))
}

/// image-set() 指定值 canonical 串化（css-images-4 §image-set-nota、页面
/// serialization-canonical oracle）：
/// - 函数名恒 `image-set(`（`-webkit-` 别名 parse 期已剥除）；
/// - 选项图像 url 引号化 `url("…")`（`"`/`\` 反斜杠转义），渐变层走
///   [`serialize_gradient_specified_value`] canonical；
/// - 分辨率：缺省补 `1x`；明文字面原文保形（`1x`/`192dpi`/`0dppx` 恒等——
///   指定面不做单位归一）；calc 字面树折叠 `calc(<n>dppx)`（NaN/±∞ 走
///   [`format_dimension_inner`] §10.6 canonical——calc-infinity-nan 语料形态），
///   非纯树（sibling-index() 等）calc 原文保形；
/// - type()：MIME 重引号化双引号 + 置分辨率之后（`<resolution> type("…")`
///   canonical 序，输入序 `||` 不限）。
pub(crate) fn image_set_specified_to_css(is: &ImageSetValue) -> Option<String> {
    let mut out = String::from("image-set(");
    for (idx, opt) in is.options.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&image_set_option_specified_to_css(opt)?);
    }
    out.push(')');
    Some(out)
}

/// image-set() 单选项指定值 canonical 串化。
fn image_set_option_specified_to_css(opt: &ImageSetOptionValue) -> Option<String> {
    let mut out = match &*opt.image {
        BackgroundImageValue::Url(u) => format!("url(\"{}\")", escape_css_string(u)),
        BackgroundImageValue::Gradient(g) => serialize_gradient_specified_value(g)?,
        // 选项图像文法限 url/字符串/渐变（parse 期收口），防御臂保穷尽。
        _ => return None,
    };
    match &opt.resolution {
        None => out.push_str(" 1x"),
        Some(ImageSetResolution::Plain { text, .. }) => {
            out.push(' ');
            out.push_str(text);
        }
        Some(ImageSetResolution::Calc { text, folded_dppx }) => match folded_dppx {
            Some(n) => {
                out.push_str(" calc(");
                out.push_str(&format_dimension_inner(*n, Some("dppx")));
                out.push(')');
            }
            None => {
                out.push(' ');
                out.push_str(text);
            }
        },
    }
    if let Some(mime) = &opt.mime {
        // MIME 引号内原文保形发射（parse 期转义序列不还原——`type("a\"")`
        // round-trip 恒等；裸 `"` 已被 parse 闭引号扫描拒收，恒良构）。
        out.push_str(" type(\"");
        out.push_str(mime);
        out.push_str("\")");
    }
    Some(out)
}

/// css 字符串发射转义：`\` 与 `"` 反斜杠转义（css-syntax §3.3.3 字符串发射
/// 最小面；语料 `type("a\"")` 钉）。
fn escape_css_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch == '\\' || ch == '"' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// g43 写面拒收判定（css-images-4 渐变文法）：任一顶层图层为渐变前缀
///（linear/radial/conic-gradient 及 repeating 变体）或 image-set 前缀
///（g45：image-set/-webkit-image-set，拒收形含嵌套 image-set、负字面分辨率、
/// 双 resolution/type、bogus 单位等语料钉面）但
/// [`parse_background_image`] 解析失败 → 整声明 invalid（CSSOM setProperty
/// no-op，调用方经回调 "null" 哨兵拒写；此前 parse 失败由 shim CSSOM store
/// 回显原文伪装成功，g30 同型通道）。非渐变/图像集面（none/url/未识别函数）
/// 恒 false passthrough。驱动：gradient-interpolation-method-invalid 292F、
/// image-set-parsing 48F。
/// https://drafts.csswg.org/css-images-4/#gradients
pub fn background_image_decl_invalid(value: &str) -> bool {
    let layers = split_top_layers(value.trim());
    !layers.is_empty()
        && layers.iter().any(|layer| {
            // image-set 判定用子串含（非仅前缀）：`image(image-set(…) 2x)`
            // 间接嵌套形层头是 image()，子串含 + parse 失败同拒；含子串但
            // parse 成功的合法形（url 文件名撞串等）不受影响。
            (is_gradient_prefixed(layer.trim()) || layer.to_ascii_lowercase().contains("image-set("))
                && parse_background_image(layer.trim()).is_none()
        })
}

/// 渐变函数前缀识别（大小写不敏感；含 repeating 变体）。
pub(crate) fn is_gradient_prefixed(v: &str) -> bool {
    let lower = v.to_ascii_lowercase();
    [
        "linear-gradient(",
        "repeating-linear-gradient(",
        "radial-gradient(",
        "repeating-radial-gradient(",
        "conic-gradient(",
        "repeating-conic-gradient(",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
}

/// 括号感知顶层逗号分割（图层分割；url()/函数内逗号受深度保护）。
fn split_top_layers(s: &str) -> Vec<&str> {
    let mut segs = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                segs.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    segs.push(&s[start..]);
    segs
}
