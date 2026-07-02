//! 文案格式化：`{param}` 占位替换 + plural 变体选择（spec IF-007）。

use crate::message::{MessageEntry, MessageParamValue, MessageParams};
use crate::plural::plural_category;

/// 选择文案模板：若存在 plural_forms 且提供 `{count}`，按 plural 规则选变体，否则用默认 value。
pub fn select_template<'a>(entry: &'a MessageEntry, params: &MessageParams) -> &'a str {
    let count = params.entries.iter().find_map(|(_, v)| match v {
        MessageParamValue::Count(n) => Some(*n),
        _ => None,
    });
    if let Some(n) = count
        && let Some(form) = entry.plural_forms.get(&plural_category(n))
    {
        return form.as_str();
    }
    entry.value.as_str()
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
    use crate::plural::PluralCategory;

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
        assert_eq!(format_message(select_template(&entry, &p), &p), "You have 1 item.");

        let mut p2 = MessageParams::new();
        p2.set_count("count", 5);
        assert_eq!(format_message(select_template(&entry, &p2), &p2), "You have 5 items.");
    }

    #[test]
    fn missing_param_preserved() {
        let p = MessageParams::new();
        assert_eq!(format_message("{missing}", &p), "{missing}");
    }
}
