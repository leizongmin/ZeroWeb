//! Plural 规则（spec IF-007 plural forms）。
//!
//! M1 提供最小英语 CLDR plural 规则（one / other）；完整 CLDR plural 留 M2 评估 ICU4X/Fluent（TBD-7）。

use serde::{Deserialize, Serialize};

/// CLDR plural 类别（M1 子集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// 计数 → plural 类别。
///
/// M1 仅实现英语规则：`n == 1` → One，否则 Other。
pub fn plural_category(count: i64) -> PluralCategory {
    if count == 1 {
        PluralCategory::One
    } else {
        PluralCategory::Other
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
}
