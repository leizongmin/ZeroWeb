//! 浏览器 UI 文案 message catalog（spec FR-013 / DC-10）。
//!
//! 通用 i18n 机制（locale/catalog/fallback/plural/RTL）在 `zero-ui-i18n`；**浏览器专属文案**
//! （菜单项、按钮、状态提示等）不进通用 crate，集中在本模块（spec FR-013 / DC-10
//! 「浏览器文案在 browser-ui/chrome/i18n」）。
//!
//! 本模块提供：
//! - [`ids`]：浏览器文案 message id 常量（点分命名，如 `browser.new_tab`）。
//! - [`default_catalog`]：默认英语（`en`）文案 catalog。
//! - [`catalog_store`]：注册了默认 catalog 的 [`CatalogStore`]（测试/运行时便利）。
//! - [`resolve`]：按 id 解析文案（返回 [`ResolvedText`]，含 fallback 诊断）。
//!
//! 组件消费：chrome 组件持有 message id（或 [`LocalizedText`]），由渲染层（`render.rs` 的
//! `ChromePanel` 等）在 paint 前经本 catalog 解析为可见字符串。当前 `shell_demo` / `browser_menu`
//! 的字面文案逐步迁移到本 catalog 的 id（DC-10 浏览器文案接入，本模块为入口）。

use zero_ui_i18n::{
    CatalogStore, I18nContext, I18nError, I18nProvider, LocaleId, LocalizedText, MessageCatalog, MessageEntry,
    MessageId, MessageRef, ResolvedText, TextDirection, direction_for, fallback_chain,
};

/// 浏览器文案 message id 常量（spec FR-013 点分命名）。
///
/// 集中定义避免散落字面量；组件/DSL/测试一律引用这些常量，确保 id 拼写一致、便于统计与本地化。
pub mod ids {
    // 菜单项（BrowserMenu）。
    pub const NEW_TAB: &str = "browser.new_tab";
    pub const NEW_WINDOW: &str = "browser.new_window";
    pub const RELOAD: &str = "browser.reload";
    pub const CLOSE_TAB: &str = "browser.close_tab";
    pub const CLOSE_MENU: &str = "browser.close_menu";
    pub const OPEN_MENU: &str = "browser.open_menu";
    // 工具栏 / bars。
    pub const FIND: &str = "browser.find";
    pub const BOOKMARKS: &str = "browser.bookmarks";
    pub const DOWNLOADS: &str = "browser.downloads";
    pub const SETTINGS: &str = "browser.settings";
    // 导航按钮（NavigationButtons）。
    pub const BACK: &str = "browser.back";
    pub const FORWARD: &str = "browser.forward";
}

/// 默认 locale（`en`）。
pub const DEFAULT_LOCALE: &str = "en";

/// 默认英语文案 catalog（spec FR-013 / DC-10）。
///
/// 返回的 catalog 覆盖 [`ids`] 中全部 message id；后续 locale 翻译以同结构新增 catalog
/// 注册到 [`catalog_store`]。
pub fn default_catalog() -> MessageCatalog {
    let mut messages = hashbrown::HashMap::new();
    let entry = |v: &str| MessageEntry::simple(v);
    messages.insert(MessageId::new(ids::NEW_TAB), entry("New Tab"));
    messages.insert(MessageId::new(ids::NEW_WINDOW), entry("New Window"));
    messages.insert(MessageId::new(ids::RELOAD), entry("Reload"));
    messages.insert(MessageId::new(ids::CLOSE_TAB), entry("Close Tab"));
    messages.insert(MessageId::new(ids::CLOSE_MENU), entry("Close Menu"));
    messages.insert(MessageId::new(ids::OPEN_MENU), entry("Menu"));
    messages.insert(MessageId::new(ids::FIND), entry("Find"));
    messages.insert(MessageId::new(ids::BOOKMARKS), entry("Bookmarks"));
    messages.insert(MessageId::new(ids::DOWNLOADS), entry("Downloads"));
    messages.insert(MessageId::new(ids::SETTINGS), entry("Settings"));
    messages.insert(MessageId::new(ids::BACK), entry("Back"));
    messages.insert(MessageId::new(ids::FORWARD), entry("Forward"));
    MessageCatalog {
        locale: LocaleId::new(DEFAULT_LOCALE),
        direction: TextDirection::Ltr,
        messages,
    }
}

/// 注册了默认 catalog 的 [`CatalogStore`]（测试 / 运行时便利入口）。
pub fn catalog_store() -> CatalogStore {
    let mut store = CatalogStore::new();
    store.register(default_catalog());
    store
}

/// 为 `locale` 构造解析上下文（fallback chain + 文本方向）。
fn context_for(locale: &LocaleId) -> I18nContext {
    I18nContext {
        locale: locale.clone(),
        fallback_chain: fallback_chain(locale),
        direction: direction_for(locale),
    }
}

/// 按 message id（点分字符串）在 `store` 中解析文案。
///
/// - 命中 → [`ResolvedText`]（首选 locale 直接命中时 `diagnostic = None`）。
/// - 全 fallback chain 未命中 → [`I18nError`]（DC-10 缺失 key 诊断，调用方据此 fallback/上报）。
///
/// 默认 locale 调用便利：[`resolve_default`]。
pub fn resolve(store: &CatalogStore, id: &str, locale: &LocaleId) -> Result<ResolvedText, I18nError> {
    let ctx = context_for(locale);
    store.resolve(&LocalizedText::Message(MessageRef::new(id)), &ctx)
}

/// 在默认 locale（`en`）解析 message id（最常见的测试 / 单语种入口）。
pub fn resolve_default(store: &CatalogStore, id: &str) -> Result<ResolvedText, I18nError> {
    resolve(store, id, &LocaleId::new(DEFAULT_LOCALE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_resolves_all_known_ids() {
        // DC-10：每个 ids 常量都应在默认 catalog 中有文案。
        let store = catalog_store();
        for (id, expected) in [
            (ids::NEW_TAB, "New Tab"),
            (ids::NEW_WINDOW, "New Window"),
            (ids::RELOAD, "Reload"),
            (ids::CLOSE_TAB, "Close Tab"),
            (ids::CLOSE_MENU, "Close Menu"),
            (ids::OPEN_MENU, "Menu"),
            (ids::FIND, "Find"),
            (ids::BOOKMARKS, "Bookmarks"),
            (ids::DOWNLOADS, "Downloads"),
            (ids::SETTINGS, "Settings"),
            (ids::BACK, "Back"),
            (ids::FORWARD, "Forward"),
        ] {
            let resolved = resolve_default(&store, id).unwrap_or_else(|e| panic!("resolve {id} failed: {e:?}"));
            assert_eq!(resolved.text, expected, "message {id}");
            assert_eq!(resolved.direction, TextDirection::Ltr);
            assert_eq!(resolved.resolved_locale, LocaleId::new(DEFAULT_LOCALE));
            // 首选 locale 直接命中 → 无 fallback 诊断。
            assert!(resolved.diagnostic.is_none(), "message {id} should have no diagnostic");
        }
    }

    #[test]
    fn unknown_id_produces_missing_key_diagnostic() {
        // DC-10：缺失 key 不报错（不阻断渲染），回退为 key 占位文案 + MissingKey 诊断
        // （运行时/工具据此上报，spec DC-10「缺失 key 走 fallback 并产生 diagnostic」）。
        let store = catalog_store();
        let resolved = resolve_default(&store, "browser.does_not_exist").expect("missing key resolves (non-fatal)");
        assert_eq!(resolved.text, "browser.does_not_exist", "placeholder = key id");
        let diag = resolved
            .diagnostic
            .as_ref()
            .expect("missing key must carry a diagnostic");
        assert_eq!(diag.kind, zero_ui_i18n::diagnostics::DiagnosticKind::MissingKey);
    }

    #[test]
    fn default_catalog_covers_all_id_constants() {
        // 完整性：ids 模块导出的每个常量都必须在默认 catalog 中可解析（防止新增 id 漏译）。
        let store = catalog_store();
        let all_ids = [
            ids::NEW_TAB,
            ids::NEW_WINDOW,
            ids::RELOAD,
            ids::CLOSE_TAB,
            ids::CLOSE_MENU,
            ids::OPEN_MENU,
            ids::FIND,
            ids::BOOKMARKS,
            ids::DOWNLOADS,
            ids::SETTINGS,
            ids::BACK,
            ids::FORWARD,
        ];
        for id in all_ids {
            assert!(
                resolve_default(&store, id).is_ok(),
                "id {id} missing from default catalog"
            );
        }
    }

    #[test]
    fn literal_text_passes_through() {
        // LocalizedText::Literal 直接回显（catalog 不干预）——组件可混合 message id 与动态字面量（如 URL）。
        let store = catalog_store();
        let ctx = context_for(&LocaleId::new(DEFAULT_LOCALE));
        let resolved = store
            .resolve(&LocalizedText::Literal("https://example.com".into()), &ctx)
            .expect("literal resolves");
        assert_eq!(resolved.text, "https://example.com");
    }
}
