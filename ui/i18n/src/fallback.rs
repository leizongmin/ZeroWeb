//! Fallback chain（spec IF-007 fallback）。
//!
//! 给定 locale，生成 `locale → parent → ... → root` 的查找链。

use crate::locale::LocaleId;

/// 生成 fallback chain（含自身）。例：`en-US` → `[en-US, en]`；`zh-Hans-CN` → `[zh-Hans-CN, zh-Hans, zh]`。
pub fn fallback_chain(locale: &LocaleId) -> Vec<LocaleId> {
    let mut chain = Vec::new();
    let mut cur = Some(locale.clone());
    while let Some(loc) = cur {
        chain.push(loc.clone());
        cur = loc.parent();
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_descends_to_root() {
        let chain = fallback_chain(&LocaleId::new("zh-Hans-CN"));
        assert_eq!(
            chain,
            vec![
                LocaleId::new("zh-Hans-CN"),
                LocaleId::new("zh-Hans"),
                LocaleId::new("zh"),
            ]
        );
    }

    #[test]
    fn single_segment_locale() {
        let chain = fallback_chain(&LocaleId::new("en"));
        assert_eq!(chain, vec![LocaleId::new("en")]);
    }
}
