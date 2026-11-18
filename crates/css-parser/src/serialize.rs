//! CSSOM 规则文本序列化（cssom-1 §serializing CSS values）。
//!
//! 当前仅覆盖 `@font-feature-values` 规则 canonical cssText（wptfyi-css
//! knife-fv1：test_font_feature_values_parsing 序列化/round-trip 靶面——
//! 引擎 CSSOM insertRule 写面单源裁决）。
//! https://drafts.csswg.org/cssom-1/#serializing
//! https://drafts.csswg.org/css-fonts-4/#font-feature-values

use crate::ast::{FontFeatureValueDefinition, FontFeatureValueKind, FontFeatureValuesRule};

/// cssom-1 §serialize-an-identifier：ident 字符集（ASCII 字母数字/`-`/`_`）
/// 与 U+0080 以上字面直写，其余字符 `\` 逐字转义（空格 → `\ `、`:` → `\:`、
/// `;` → `\;`）。名称来自声明属性位（tokenizer 已解码 `\6e` 转义形并拒绝非
/// ident 起始），前导数字/空串形不可达。
/// https://drafts.csswg.org/cssom-1/#serialize-an-identifier
pub fn serialize_css_ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch as u32 >= 0x80 {
            out.push(ch);
        } else {
            out.push('\\');
            out.push(ch);
        }
    }
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
