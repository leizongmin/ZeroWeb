//! CSSOM 规则文本序列化（cssom-1 §serializing CSS values）。
//!
//! 当前仅覆盖 `@font-feature-values` 规则 canonical cssText（wptfyi-css
//! knife-fv1：test_font_feature_values_parsing 序列化/round-trip 靶面——
//! 引擎 CSSOM insertRule 写面单源裁决）。
//! https://drafts.csswg.org/cssom-1/#serializing
//! https://drafts.csswg.org/css-fonts-4/#font-feature-values

use crate::ast::{FontFeatureValueDefinition, FontFeatureValueKind, FontFeatureValuesRule};
use std::fmt::Write as _;

/// cssom-1 §serialize-an-identifier：ident 字符集（ASCII 字母数字/`-`/`_`）
/// 与 U+0080 以上字面直写；C0/C1 控制字符（U+0000-U+001F/U+007F）按规范
/// `\` + 十六进制 + 空格转义（corpus `--tab\ttab` → `--tab\9 tab` 钉）；
/// 其余字符 `\` 逐字转义（空格 → `\ `、`:` → `\:`、`;` → `\;`）。名称来自
/// 声明属性位（tokenizer 已解码 `\6e` 转义形并拒绝非 ident 起始），前导
/// 数字/空串形不可达。
/// https://drafts.csswg.org/cssom-1/#serialize-an-identifier
pub fn serialize_css_ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        let c = ch as u32;
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || c >= 0x80 {
            out.push(ch);
        } else if c < 0x20 || c == 0x7F {
            out.push('\\');
            let _ = write!(out, "{:x} ", c);
        } else {
            out.push('\\');
            out.push(ch);
        }
    }
    out
}

/// `@property` 规则 canonical cssText（css-properties-values-api-1 §at-property-rule +
/// cssom-1 serialize；corpus at-property-cssom.html cssText 断言钉形）：
/// `@property <ident> { syntax: "<content>"; inherits: b; initial-value: <text>; }`
/// ——描述符固定序 syntax→inherits→initial-value、缺 initial-value 段省略（universal
/// 无初值钉）。syntax 内容 = string 字面内文（tokenizer 解码后的原文 whitespace 保形，
/// corpus `' <color># '` 钉）；名按 serialize_css_ident 重转义（corpus `--tab\ttab` →
/// `--tab\9 tab` 钉）。刀 ATP（wptfyi-css 2026-11-18）。
/// https://drafts.css-houdini.org/css-properties-values-api-1/
pub fn serialize_property_rule_css_text(rule: &crate::ast::PropertyRule) -> String {
    let mut out = String::with_capacity(rule.name.len() + rule.syntax.len() + 48);
    out.push_str("@property ");
    out.push_str(&serialize_css_ident(&rule.name));
    out.push_str(" { syntax: \"");
    // string 字面内文（tokenizer 解码后、外引号剥除——存储面 decl.value 原文含引号，
    // 解码抽取归 property_validate::syntax_string_content 单源）。
    let content =
        crate::parser::property_validate::syntax_string_content(&rule.syntax).unwrap_or_else(|| rule.syntax.clone());
    for ch in content.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out.push_str("\"; inherits: ");
    out.push_str(if rule.inherits { "true" } else { "false" });
    out.push(';');
    if let Some(iv) = &rule.initial_value {
        out.push_str(" initial-value: ");
        out.push_str(iv);
        out.push(';');
    }
    out.push_str(" }");
    out
}

/// `@font-feature-values` 规则 canonical cssText：
/// `@font-feature-values <fam>, <fam> { @styleset { n: 1 2; ... } @swash { ... } }`。
/// 值定义按类型首现顺序聚块、块内保持源序（invalid 定义已在解析站丢弃——
/// 「invalid declarations should not affect valid ones」wpt 段钉）；空定义
/// 序列化为 `... { }`（「no value definitions」段钉）。族名序列化只须
/// round-trip 可再解析（wpt valuesText 比较不含族名段——上游注释
/// 「family names are serialized inconsistently」钉）。
pub fn serialize_font_feature_values_css_text(rule: &FontFeatureValuesRule) -> String {
    let mut out = String::from("@font-feature-values ");
    let families: Vec<String> = rule.families.iter().map(|f| serialize_font_family(f)).collect();
    out.push_str(&families.join(", "));
    out.push_str(" {");
    let mut order: Vec<FontFeatureValueKind> = Vec::new();
    for definition in &rule.definitions {
        if !order.contains(&definition.kind) {
            order.push(definition.kind);
        }
    }
    for kind in order {
        out.push_str(" @");
        out.push_str(kind_css_name(kind));
        out.push_str(" {");
        for definition in rule
            .definitions
            .iter()
            .filter(|d: &&FontFeatureValueDefinition| d.kind == kind)
        {
            out.push(' ');
            out.push_str(&serialize_css_ident(&definition.name));
            out.push(':');
            for value in &definition.values {
                out.push(' ');
                out.push_str(&value.to_string());
            }
            out.push(';');
        }
        out.push_str(" }");
    }
    out.push_str(" }");
    out
}

/// 族名序列化：ident 安全形（含多词 ident 族——空格连写可再解析为同族）
/// 裸排；含非 ident 字符的加引号并转义 `\`/`"`。
fn serialize_font_family(family: &str) -> String {
    if family.is_empty() {
        return "\"\"".to_string();
    }
    let ident_safe = family
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' || c as u32 >= 0x80);
    if ident_safe {
        family.to_string()
    } else {
        format!("\"{}\"", family.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn kind_css_name(kind: FontFeatureValueKind) -> &'static str {
    match kind {
        FontFeatureValueKind::Stylistic => "stylistic",
        FontFeatureValueKind::Styleset => "styleset",
        FontFeatureValueKind::CharacterVariant => "character-variant",
        FontFeatureValueKind::Swash => "swash",
        FontFeatureValueKind::Ornaments => "ornaments",
        FontFeatureValueKind::Annotation => "annotation",
    }
}

/// 关键帧选择器列表 canonical 串化：from/to 保词、百分比整数去小数尾
///（`50%`；非整数保 f64 最短形），列表 `, ` 连（css-animations-1
/// §CSSKeyframeRule keyText 串化形——`serialize-a-keyframe-selector`）。
/// https://drafts.csswg.org/css-animations-1/#csskeyframerule
pub fn serialize_keyframe_selectors(
    selectors: &[crate::ast::KeyframeSelector],
) -> String {
    selectors
        .iter()
        .map(|s| match s {
            crate::ast::KeyframeSelector::From => "from".to_string(),
            crate::ast::KeyframeSelector::To => "to".to_string(),
            crate::ast::KeyframeSelector::Percentage(p) => {
                if p.fract() == 0.0 {
                    format!("{}%", *p as i64)
                } else {
                    format!("{}%", p)
                }
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `@keyframes` 规则 canonical cssText（css-animations-1 §at-ruledef-keyframes +
/// cssom-1 serializing；wptfyi-css 刀 KF：cssRules 读面 wire cssText 与 insertRule
/// 写面经 `__zw_parse_keyframes_rule` 单源共用，property-rule 同型）。形：
/// `@keyframes <name> { <sel-list> { decls } }`——名按 [`serialize_css_ident`]
/// 重转义；声明 `p: v;`（`!important` 保形）空格连；空关键帧 `@keyframes n { }`
/// 单空格形（Chrome 同形，property-rule 空块钉同型）。
/// https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes
pub fn serialize_keyframes_css_text(rule: &crate::ast::KeyframesRule) -> String {
    let mut out = String::with_capacity(48);
    out.push_str("@keyframes ");
    if rule.name_is_string {
        // string 形名带引号重排（css-animations-1 §typedef-keyframes-name <string>
        // 支——`\`/`"` 转义保 round-trip 再解析归 String token，绕 custom-ident
        // CSS-wide 排除——`"none"` 名规则 cssRules 读面不断链）。
        let mut quoted = String::with_capacity(rule.name.len() + 2);
        quoted.push('"');
        for ch in rule.name.chars() {
            match ch {
                '"' | '\\' => {
                    quoted.push('\\');
                    quoted.push(ch);
                }
                _ => quoted.push(ch),
            }
        }
        quoted.push('"');
        out.push_str(&quoted);
    } else {
        out.push_str(&serialize_css_ident(&rule.name));
    }
    out.push_str(" {");
    for kf in &rule.keyframes {
        out.push(' ');
        out.push_str(&serialize_keyframe_selectors(&kf.selectors));
        out.push_str(" {");
        if !kf.declarations.is_empty() {
            let decls = kf
                .declarations
                .iter()
                .map(|d| {
                    if d.important {
                        format!("{}: {} !important;", d.property, d.value)
                    } else {
                        format!("{}: {};", d.property, d.value)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            out.push(' ');
            out.push_str(&decls);
            out.push(' ');
        }
        out.push('}');
    }
    out.push_str(" }");
    out
}
