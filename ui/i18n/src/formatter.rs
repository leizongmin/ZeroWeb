//! 文案格式化：`{param}` 占位替换 + plural 变体选择（spec IF-007）。

use crate::diagnostics::DiagnosticKind;
use crate::locale::LocaleId;
use crate::message::{MessageEntry, MessageParamValue, MessageParams};
use crate::plural::plural_category_for;

/// 选择文案模板（locale 感知 plural，DC-10）。
///
/// 若 entry 含 plural_forms 且 params 提供 count，按 [`plural_category_for`] 选变体；
/// 否则用默认 value。仅返回模板（不带诊断），诊断版本见 [`select_template_diag`]。
pub fn select_template<'a>(entry: &'a MessageEntry, params: &MessageParams, locale: &LocaleId) -> &'a str {
    select_template_diag(entry, params, locale).0
}

/// 同 [`select_template`]，但额外返回 plural 诊断：entry 含 plural_forms、提供了 count，
/// 但缺失该 locale plural 类别对应变体（回落默认 value）时返回 `PluralFallback`。
pub fn select_template_diag<'a>(
    entry: &'a MessageEntry,
    params: &MessageParams,
    locale: &LocaleId,
) -> (&'a str, Option<DiagnosticKind>) {
    let count = params.entries.iter().find_map(|(_, v)| match v {
        MessageParamValue::Count(n) => Some(*n),
        _ => None,
    });
    if let Some(n) = count
        && !entry.plural_forms.is_empty()
    {
        let cat = plural_category_for(n, locale);
        if let Some(form) = entry.plural_forms.get(&cat) {
            return (form.as_str(), None);
        }
        // plural 变体缺失 → 默认 value + 诊断。
        return (entry.value.as_str(), Some(DiagnosticKind::PluralFallback));
    }
    (entry.value.as_str(), None)
}

/// 把 `{name}` 占位替换为参数值。缺失参数保留原占位（调用方决定是否报错）。
pub fn format_message(template: &str, params: &MessageParams) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            let name = &after[..close];
            if let Some(val) = params.entries.get(name) {
                match val {
                    MessageParamValue::Text(s) => out.push_str(s),
                    MessageParamValue::Count(n) => out.push_str(&n.to_string()),
                }
            } else {
                // 缺失参数：保留原 `{name}` 以便诊断。
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            rest = &after[close + 1..];
        } else {
            // 无闭合括号：原样输出剩余。
            out.push_str(rest);
            return out;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::LocaleId;
    use crate::plural::PluralCategory;

    fn en() -> LocaleId {
        LocaleId::new("en")
    }

    #[test]
    fn substitutes_text_param() {
        let mut p = MessageParams::new();
        p.set_text("name", "Zero");
        assert_eq!(format_message("Hello, {name}!", &p), "Hello, Zero!");
    }

    #[test]
    fn substitutes_count_and_plural() {
        let mut entry = MessageEntry::simple("You have {count} items.");
        entry
            .plural_forms
            .insert(PluralCategory::One, "You have {count} item.".to_string());
        let mut p = MessageParams::new();
        p.set_count("count", 1);
        assert_eq!(
            format_message(select_template(&entry, &p, &en()), &p),
            "You have 1 item."
        );

        let mut p2 = MessageParams::new();
        p2.set_count("count", 5);
        assert_eq!(
            format_message(select_template(&entry, &p2, &en()), &p2),
            "You have 5 items."
        );
    }

    #[test]
    fn plural_fallback_emits_diagnostic() {
        // entry 含 plural_forms 但缺 Other 变体；count=5（Other）→ 回落 value + PluralFallback。
        let mut entry = MessageEntry::simple("You have {count} items.");
        entry
            .plural_forms
            .insert(PluralCategory::One, "You have {count} item.".to_string());
        let mut p = MessageParams::new();
        p.set_count("count", 5);
        let (tmpl, diag) = select_template_diag(&entry, &p, &en());
        assert_eq!(tmpl, "You have {count} items.");
        assert_eq!(diag, Some(DiagnosticKind::PluralFallback));
    }

    #[test]
    fn missing_param_preserved() {
        let p = MessageParams::new();
        assert_eq!(format_message("{missing}", &p), "{missing}");
    }
}
