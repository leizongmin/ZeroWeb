//! Message catalog、解析上下文与 `I18nProvider`（spec IF-007）。

use crate::diagnostics::I18nError;
use crate::direction::TextDirection;
use crate::formatter::{format_message, select_template};
use crate::locale::LocaleId;
use crate::message::{LocalizedText, MessageEntry, MessageId, MessageParams};
use hashbrown::HashMap;

/// 单个 locale 的 message 目录（spec IF-007 `MessageCatalog`）。
#[derive(Debug, Clone, PartialEq)]
pub struct MessageCatalog {
    pub locale: LocaleId,
    pub direction: TextDirection,
    pub messages: HashMap<MessageId, MessageEntry>,
}

/// 解析上下文（spec IF-007 `I18nContext`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nContext {
    pub locale: LocaleId,
    pub fallback_chain: Vec<LocaleId>,
    pub direction: TextDirection,
}

/// 解析结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedText {
    pub text: String,
    pub direction: TextDirection,
    /// 实际命中的 locale（用于诊断 fallback）。
    pub resolved_locale: LocaleId,
}

/// i18n 提供者（spec IF-007 `I18nProvider`）。
pub trait I18nProvider {
    fn resolve(&self, text: &LocalizedText, ctx: &I18nContext) -> Result<ResolvedText, I18nError>;
    fn direction(&self, locale: &LocaleId) -> TextDirection;
}

/// 基于 catalog 集合的最小 `I18nProvider` 实现（持有多个 locale 的 catalog，按 fallback chain 查找）。
#[derive(Debug, Default)]
pub struct CatalogStore {
    catalogs: HashMap<LocaleId, MessageCatalog>,
}

impl CatalogStore {
    pub fn new() -> CatalogStore {
        CatalogStore::default()
    }

    pub fn register(&mut self, catalog: MessageCatalog) -> &mut CatalogStore {
        self.catalogs.insert(catalog.locale.clone(), catalog);
        self
    }

    fn lookup<'a>(
        &'a self,
        chain: &'a [LocaleId],
        id: &MessageId,
    ) -> Option<(&'a LocaleId, &'a MessageCatalog, &'a crate::message::MessageEntry)> {
        for loc in chain {
            if let Some(cat) = self.catalogs.get(loc)
                && let Some(entry) = cat.messages.get(id)
            {
                return Some((loc, cat, entry));
            }
        }
        None
    }
}

impl I18nProvider for CatalogStore {
    fn resolve(&self, text: &LocalizedText, ctx: &I18nContext) -> Result<ResolvedText, I18nError> {
        match text {
            LocalizedText::Literal(s) => Ok(ResolvedText {
                text: s.clone(),
                direction: ctx.direction,
                resolved_locale: ctx.locale.clone(),
            }),
            LocalizedText::Message(mref) => {
                let id = &mref.id;
                match self.lookup(&ctx.fallback_chain, id) {
                    Some((loc, cat, entry)) => {
                        let template = select_template(entry, &mref.params);
                        validate_params(template, &mref.params)?;
                        let text = format_message(template, &mref.params);
                        Ok(ResolvedText {
                            text,
                            direction: cat.direction,
                            resolved_locale: loc.clone(),
                        })
                    }
                    None => {
                        // 全部 locale 缺失：返回 key 占位（不报错，spec IF-007）。
                        // 诊断（MissingKey）由运行时 mutable provider 层在 M2 补充。
                        Ok(ResolvedText {
                            text: id.0.to_string(),
                            direction: ctx.direction,
                            resolved_locale: ctx.locale.clone(),
                        })
                    }
                }
            }
        }
    }

    fn direction(&self, locale: &LocaleId) -> TextDirection {
        crate::direction::direction_for(locale)
    }
}

/// 校验模板中所有 `{name}` 占位都在 params 中提供。
fn validate_params(template: &str, params: &MessageParams) -> Result<(), I18nError> {
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            let name = &after[..close];
            if !params.entries.contains_key(name) {
                return Err(I18nError::MissingParam(name.to_string()));
            }
            rest = &after[close + 1..];
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessageEntry, MessageRef};

    fn en_catalog() -> MessageCatalog {
        let mut messages = HashMap::new();
        messages.insert(MessageId::new("app.title"), MessageEntry::simple("Zero Browser"));
        messages.insert(
            MessageId::new("tabs.count"),
            MessageEntry {
                value: "Tabs: {count}".to_string(),
                description: None,
                plural_forms: HashMap::new(),
            },
        );
        MessageCatalog {
            locale: LocaleId::new("en"),
            direction: TextDirection::Ltr,
            messages,
        }
    }

    fn ctx_en() -> I18nContext {
        I18nContext {
            locale: LocaleId::new("en"),
            fallback_chain: vec![LocaleId::new("en")],
            direction: TextDirection::Ltr,
        }
    }

    #[test]
    fn resolve_message_with_param() {
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        let mut mref = MessageRef::new("tabs.count");
        mref.params.set_count("count", 3);
        let resolved = store.resolve(&LocalizedText::Message(mref), &ctx_en()).unwrap();
        assert_eq!(resolved.text, "Tabs: 3");
        assert_eq!(resolved.resolved_locale, LocaleId::new("en"));
    }

    #[test]
    fn missing_param_is_error() {
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        // 模板需要 {count}，但不提供 → Err(MissingParam)。
        let mref = MessageRef::new("tabs.count");
        let err = store.resolve(&LocalizedText::Message(mref), &ctx_en()).unwrap_err();
        assert_eq!(err, I18nError::MissingParam("count".to_string()));
    }

    #[test]
    fn missing_key_returns_placeholder() {
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        let mref = MessageRef::new("does.not.exist");
        let resolved = store.resolve(&LocalizedText::Message(mref), &ctx_en()).unwrap();
        // 缺失 key → key 占位（不报错）。
        assert_eq!(resolved.text, "does.not.exist");
    }

    #[test]
    fn fallback_chain_resolves_parent() {
        // en-US 请求，catalog 只注册了 en → fallback 命中 en。
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        let ctx = I18nContext {
            locale: LocaleId::new("en-US"),
            fallback_chain: vec![LocaleId::new("en-US"), LocaleId::new("en")],
            direction: TextDirection::Ltr,
        };
        let resolved = store
            .resolve(&LocalizedText::Message(MessageRef::new("app.title")), &ctx)
            .unwrap();
        assert_eq!(resolved.text, "Zero Browser");
        assert_eq!(resolved.resolved_locale, LocaleId::new("en"));
    }
}
