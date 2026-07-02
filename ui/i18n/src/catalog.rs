//! Message catalog、解析上下文与 `I18nProvider`（spec IF-007）。

use crate::diagnostics::{DiagnosticKind, I18nDiagnostic, I18nError};
use crate::direction::TextDirection;
use crate::formatter::{format_message, select_template_diag};
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
    /// 非致命诊断（缺失 key / fallback 生效 / plural 变体缺失，spec DC-10）。
    ///
    /// `None` 表示首选 locale 直接命中、plural 变体齐全。运行时/工具据此上报而不阻塞渲染。
    pub diagnostic: Option<I18nDiagnostic>,
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
                diagnostic: None,
            }),
            LocalizedText::Message(mref) => {
                let id = &mref.id;
                match self.lookup(&ctx.fallback_chain, id) {
                    Some((loc, cat, entry)) => {
                        let (template, plural_diag) = select_template_diag(entry, &mref.params, loc);
                        validate_params(template, &mref.params)?;
                        let text = format_message(template, &mref.params);
                        // 诊断优先级：plural 变体缺失 > fallback 生效（命中非首选 locale）。
                        let diagnostic = if let Some(kind) = plural_diag {
                            Some(I18nDiagnostic {
                                kind,
                                message: format!("plural form missing for {}", id.0),
                            })
                        } else if loc != ctx.fallback_chain.first().unwrap_or(loc) {
                            Some(I18nDiagnostic {
                                kind: DiagnosticKind::FallbackUsed,
                                message: format!("resolved {} via fallback to {}", id.0, loc.0),
                            })
                        } else {
                            None
                        };
                        Ok(ResolvedText {
                            text,
                            direction: cat.direction,
                            resolved_locale: loc.clone(),
                            diagnostic,
                        })
                    }
                    None => {
                        // 全部 locale 缺失：返回 key 占位（不报错）+ MissingKey 诊断（spec DC-10）。
                        Ok(ResolvedText {
                            text: id.0.to_string(),
                            direction: ctx.direction,
                            resolved_locale: ctx.locale.clone(),
                            diagnostic: Some(I18nDiagnostic {
                                kind: DiagnosticKind::MissingKey,
                                message: format!(
                                    "key {} missing across {} locale(s) in fallback chain",
                                    id.0,
                                    ctx.fallback_chain.len()
                                ),
                            }),
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
    use crate::plural::PluralCategory;

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

    // ── DC-10 flesh-out：诊断 + locale 感知 plural + RTL 集成快照 ─────────────

    /// 阿拉伯语 catalog（RTL）：含 one/two/few/many plural 变体，故意缺 zero/other。
    fn ar_catalog() -> MessageCatalog {
        let mut messages = HashMap::new();
        messages.insert(MessageId::new("app.title"), MessageEntry::simple("متصفح زيرو"));
        let mut files = MessageEntry::simple("{count} ملفات");
        files.plural_forms.insert(PluralCategory::One, "ملف واحد".to_string());
        files.plural_forms.insert(PluralCategory::Two, "ملفان".to_string());
        files
            .plural_forms
            .insert(PluralCategory::Few, "{count} ملفات".to_string());
        files
            .plural_forms
            .insert(PluralCategory::Many, "{count} ملفًا".to_string());
        messages.insert(MessageId::new("files.count"), files);
        MessageCatalog {
            locale: LocaleId::new("ar"),
            direction: TextDirection::Rtl,
            messages,
        }
    }

    fn ctx_ar() -> I18nContext {
        I18nContext {
            locale: LocaleId::new("ar"),
            fallback_chain: vec![LocaleId::new("ar")],
            direction: TextDirection::Rtl,
        }
    }

    #[test]
    fn missing_key_emits_diagnostic() {
        // DC-10：缺失 key → 占位文案 + MissingKey 诊断（不报错）。
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        let r = store
            .resolve(&LocalizedText::Message(MessageRef::new("does.not.exist")), &ctx_en())
            .unwrap();
        assert_eq!(r.text, "does.not.exist");
        assert_eq!(r.diagnostic.map(|d| d.kind), Some(DiagnosticKind::MissingKey));
    }

    #[test]
    fn fallback_hit_emits_diagnostic() {
        // 命中非首选 locale（en-US 请求 → en 命中）→ FallbackUsed 诊断。
        let mut store = CatalogStore::new();
        store.register(en_catalog());
        let ctx = I18nContext {
            locale: LocaleId::new("en-US"),
            fallback_chain: vec![LocaleId::new("en-US"), LocaleId::new("en")],
            direction: TextDirection::Ltr,
        };
        let r = store
            .resolve(&LocalizedText::Message(MessageRef::new("app.title")), &ctx)
            .unwrap();
        assert_eq!(r.text, "Zero Browser");
        assert_eq!(r.resolved_locale, LocaleId::new("en"));
        assert_eq!(r.diagnostic.map(|d| d.kind), Some(DiagnosticKind::FallbackUsed));
    }

    #[test]
    fn rtl_locale_resolves_with_rtl_direction() {
        // DC-10 RTL 快照：ar locale → 方向 Rtl + 命中 ar 文案，无诊断（首选直接命中）。
        let mut store = CatalogStore::new();
        store.register(ar_catalog());
        let r = store
            .resolve(&LocalizedText::Message(MessageRef::new("app.title")), &ctx_ar())
            .unwrap();
        assert_eq!(r.text, "متصفح زيرو");
        assert_eq!(r.direction, TextDirection::Rtl);
        assert_eq!(r.resolved_locale, LocaleId::new("ar"));
        assert!(r.diagnostic.is_none(), "direct hit → no diagnostic");
    }

    #[test]
    fn arabic_plural_selects_locale_correct_form() {
        // DC-10 plural：count=2 → Two 变体；count=5 → Few 变体（CLDR 阿拉伯规则）。
        let mut store = CatalogStore::new();
        store.register(ar_catalog());
        let mut m = MessageRef::new("files.count");
        m.params.set_count("count", 2);
        let r = store.resolve(&LocalizedText::Message(m), &ctx_ar()).unwrap();
        assert_eq!(r.text, "ملفان", "count=2 → Two form (literal)");
        assert!(r.diagnostic.is_none());

        let mut m2 = MessageRef::new("files.count");
        m2.params.set_count("count", 5);
        let r2 = store.resolve(&LocalizedText::Message(m2), &ctx_ar()).unwrap();
        assert_eq!(r2.text, "5 ملفات", "count=5 → Few form with placeholder");
        assert!(r2.diagnostic.is_none());
    }

    #[test]
    fn arabic_plural_missing_form_emits_diagnostic() {
        // count=0 → Zero 类别，ar catalog 缺 Zero 变体 → 回落默认 value + PluralFallback 诊断。
        let mut store = CatalogStore::new();
        store.register(ar_catalog());
        let mut m = MessageRef::new("files.count");
        m.params.set_count("count", 0);
        let r = store.resolve(&LocalizedText::Message(m), &ctx_ar()).unwrap();
        assert_eq!(r.diagnostic.map(|d| d.kind), Some(DiagnosticKind::PluralFallback));
    }
}
