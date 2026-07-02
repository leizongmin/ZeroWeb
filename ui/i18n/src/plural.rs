//! Plural 规则（spec IF-007 plural forms）。
//!
//! 提供 CLDR cardinal plural 规则的**手写实现**（英语 / 阿拉伯语 / 俄语 / 波兰语，
//! 覆盖 one/two/few/many/other/zero 全类别），不引入 ICU4X/Fluent（TBD-7 决策：
//! 接口先行 + 仓内规则可控 + 零新依赖；更多语种按需补充）。

use crate::locale::LocaleId;
use serde::{Deserialize, Serialize};

/// CLDR plural 类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// 计数 → plural 类别（英语默认规则；`n == 1` → One，否则 Other）。
///
/// locale 感知版本见 [`plural_category_for`]。
pub fn plural_category(count: i64) -> PluralCategory {
    plural_category_for(count, &LocaleId::new("en"))
}

/// 计数 + locale → plural 类别（CLDR cardinal 规则，DC-10）。
///
/// 覆盖：英语/根（one/other）、阿拉伯语（zero/one/two/few/many/other）、
/// 俄语/乌克兰语/白俄罗斯语（one/few/many/other）、波兰语（one/few/many）。
/// 未覆盖语种回落英语规则（one/other）。负数按绝对值处理（计数语义）。
pub fn plural_category_for(count: i64, locale: &LocaleId) -> PluralCategory {
    let lang = locale.0.as_str().split('-').next().unwrap_or("").to_ascii_lowercase();
    let n = count.unsigned_abs();
    let mod100 = n % 100;
    let mod10 = n % 10;
    match lang.as_str() {
        // 阿拉伯语：zero/one/two/few(3..10)/many(11..99)/other。
        "ar" => match n {
            0 => PluralCategory::Zero,
            1 => PluralCategory::One,
            2 => PluralCategory::Two,
            _ if (3..=10).contains(&mod100) => PluralCategory::Few,
            _ if (11..=99).contains(&mod100) => PluralCategory::Many,
            _ => PluralCategory::Other,
        },
        // 俄语/乌克兰语/白俄罗斯语：one/few/many/other。
        "ru" | "uk" | "be" => {
            if mod10 == 1 && mod100 != 11 {
                PluralCategory::One
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                PluralCategory::Few
            } else if mod10 == 0 || (5..=9).contains(&mod10) || (11..=14).contains(&mod100) {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }
        // 波兰语：one(1)/few(2..4, 非 12..14)/many(其余)。
        "pl" => {
            if n == 1 {
                PluralCategory::One
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                PluralCategory::Few
            } else {
                PluralCategory::Many
            }
        }
        // 默认（英语/根）：one(n==1) / other。
        _ => {
            if n == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_plural() {
        assert_eq!(plural_category(1), PluralCategory::One);
        assert_eq!(plural_category(0), PluralCategory::Other);
        assert_eq!(plural_category(5), PluralCategory::Other);
    }

    #[test]
    fn arabic_plural_full_categories() {
        let ar = LocaleId::new("ar");
        assert_eq!(plural_category_for(0, &ar), PluralCategory::Zero);
        assert_eq!(plural_category_for(1, &ar), PluralCategory::One);
        assert_eq!(plural_category_for(2, &ar), PluralCategory::Two);
        assert_eq!(plural_category_for(5, &ar), PluralCategory::Few); // 3..10
        assert_eq!(plural_category_for(10, &ar), PluralCategory::Few);
        assert_eq!(plural_category_for(15, &ar), PluralCategory::Many); // 11..99
        assert_eq!(plural_category_for(99, &ar), PluralCategory::Many);
        assert_eq!(plural_category_for(100, &ar), PluralCategory::Other); // mod100=0
        assert_eq!(plural_category_for(103, &ar), PluralCategory::Few); // mod100=3
        assert_eq!(plural_category_for(115, &ar), PluralCategory::Many); // mod100=15
    }

    #[test]
    fn russian_plural_categories() {
        let ru = LocaleId::new("ru");
        assert_eq!(plural_category_for(1, &ru), PluralCategory::One);
        assert_eq!(plural_category_for(2, &ru), PluralCategory::Few);
        assert_eq!(plural_category_for(3, &ru), PluralCategory::Few);
        assert_eq!(plural_category_for(5, &ru), PluralCategory::Many);
        assert_eq!(plural_category_for(11, &ru), PluralCategory::Many);
        assert_eq!(plural_category_for(21, &ru), PluralCategory::One); // mod10=1, mod100=21≠11
        assert_eq!(plural_category_for(22, &ru), PluralCategory::Few);
        assert_eq!(plural_category_for(25, &ru), PluralCategory::Many);
    }

    #[test]
    fn polish_plural_categories() {
        let pl = LocaleId::new("pl");
        assert_eq!(plural_category_for(1, &pl), PluralCategory::One);
        assert_eq!(plural_category_for(2, &pl), PluralCategory::Few);
        assert_eq!(plural_category_for(22, &pl), PluralCategory::Few);
        assert_eq!(plural_category_for(5, &pl), PluralCategory::Many);
        assert_eq!(plural_category_for(12, &pl), PluralCategory::Many);
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        let xx = LocaleId::new("xx-YY");
        assert_eq!(plural_category_for(1, &xx), PluralCategory::One);
        assert_eq!(plural_category_for(2, &xx), PluralCategory::Other);
    }
}
