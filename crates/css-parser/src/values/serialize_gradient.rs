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
//! token 归一——归 computed 串化刀）；非渐变面（none/url/image-set/filter/
//! light-dark/多层混合）返 None passthrough 零回归（g24 口径：文法 invalid
//! 才拒，未支持面不接管）。

use super::parse_extended_visual::{BackgroundImageValue, parse_background_image};
use super::parse_transform::{GradientSpecifiedParts, GradientValue};

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
/// 逐顶层图层 parse→渐变层 canonical 串化；任一层为非渐变面（none/url/
/// image-set()/filter()/light-dark()/文法 invalid）→ 整值 None passthrough
///（url 引号化与 image-set canonical 归 image-set 刀）。
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
            // 非渐变面 passthrough（见模块头注切片边界）。
            _ => return None,
        }
    }
    Some(out.join(", "))
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
