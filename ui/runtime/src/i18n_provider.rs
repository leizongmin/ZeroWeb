//! I18n 运行时（spec FR-013 / DC-10）。
//!
//! 持有 catalog 集合与当前 locale；locale 切换触发 layout+paint+semantics 失效
//! （RTL 影响布局方向与可镜像图标，spec DC-10）。

use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_i18n::{
    CatalogStore, I18nContext, I18nError, I18nProvider, LocaleId, LocalizedText, ResolvedText, TextDirection,
    direction_for, fallback_chain,
};

/// 运行时 i18n 状态。
#[derive(Debug)]
pub struct I18nRuntime {
    store: CatalogStore,
    context: I18nContext,
}

impl I18nRuntime {
    pub fn new(default_locale: LocaleId) -> I18nRuntime {
        let chain = fallback_chain(&default_locale);
        let direction = direction_for(&default_locale);
        I18nRuntime {
            store: CatalogStore::new(),
            context: I18nContext {
                locale: default_locale,
                fallback_chain: chain,
                direction,
            },
        }
    }

    pub fn register_catalog(&mut self, catalog: zero_ui_i18n::MessageCatalog) -> &mut I18nRuntime {
        self.store.register(catalog);
        self
    }

    pub fn context(&self) -> &I18nContext {
        &self.context
    }

    /// 切换 locale；返回需要触发的失效（layout+paint+semantics，因为 RTL/方向/文案长度都可能变）。
    pub fn set_locale(&mut self, locale: LocaleId) -> InvalidationFlags {
        let chain = fallback_chain(&locale);
        let direction = direction_for(&locale);
        self.context = I18nContext {
            locale,
            fallback_chain: chain,
            direction,
        };
        InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT | InvalidationFlags::NEEDS_SEMANTICS
    }

    pub fn current_direction(&self) -> TextDirection {
        self.context.direction
    }
}

impl I18nProvider for I18nRuntime {
    fn resolve(&self, text: &LocalizedText, ctx: &I18nContext) -> Result<ResolvedText, I18nError> {
        self.store.resolve(text, ctx)
    }
    fn direction(&self, locale: &LocaleId) -> TextDirection {
        direction_for(locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_switch_invalidates_layout_paint_semantics() {
        // DC-10：locale 切换影响布局方向与文案 → layout+paint+semantics 全失效。
        let mut rt = I18nRuntime::new(LocaleId::new("en"));
        let inv = rt.set_locale(LocaleId::new("ar"));
        assert!(inv.contains(InvalidationFlags::NEEDS_LAYOUT));
        assert!(inv.contains(InvalidationFlags::NEEDS_PAINT));
        assert!(inv.contains(InvalidationFlags::NEEDS_SEMANTICS));
        // RTL 检测同步生效。
        assert_eq!(rt.current_direction(), TextDirection::Rtl);
    }

    #[test]
    fn fallback_chain_updates_with_locale() {
        let mut rt = I18nRuntime::new(LocaleId::new("en"));
        assert_eq!(rt.context().fallback_chain, vec![LocaleId::new("en")]);
        rt.set_locale(LocaleId::new("en-US"));
        assert_eq!(
            rt.context().fallback_chain,
            vec![LocaleId::new("en-US"), LocaleId::new("en")]
        );
    }
}
