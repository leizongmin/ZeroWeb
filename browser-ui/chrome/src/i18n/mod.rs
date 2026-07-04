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

use std::collections::HashMap as StdHashMap;
use zero_ui_i18n::{
    CatalogStore, I18nContext, I18nError, I18nProvider, LocaleId, LocalizedText, MessageCatalog, MessageEntry, MessageId,
    MessageParams, MessageRef, ResolvedText, TextDirection, direction_for, fallback_chain,
};

// ── YAML catalog 反序列化（零-ui-i18n 类型自带 serde，但 MessageCatalog 无 serde derive，
//    故定义本地中间结构，再转换为 MessageCatalog）────────────────────────────

/// YAML catalog 的中间反序列化格式。
#[derive(serde::Deserialize)]
struct YamlCatalog {
    locale: String,
    direction: String,
    messages: StdHashMap<String, YamlEntry>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum YamlEntry {
    /// 简写格式：`key: "value"` → 等价于 `{ value: "value" }`
    Simple(String),
    /// 完整格式：`key: { value: "...", description: "...", plural: { one: "..." } }`
    Full {
        value: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        plural: Option<StdHashMap<String, String>>,
    },
}

/// 把 YAML 字符串解析为 [`MessageCatalog`]。
///
/// YAML 格式见 `browser-ui/chrome/i18n/en-US.yaml`。
pub fn catalog_from_yaml(yaml: &str) -> Result<MessageCatalog, String> {
    let yc: YamlCatalog =
        serde_yaml::from_str(yaml).map_err(|e| format!("i18n YAML 解析失败: {e}"))?;
    let direction = match yc.direction.to_lowercase().as_str() {
        "rtl" => TextDirection::Rtl,
        _ => TextDirection::Ltr,
    };
    let mut messages = hashbrown::HashMap::with_capacity(yc.messages.len());
    for (id, entry) in yc.messages {
        let msg_entry = match entry {
            YamlEntry::Simple(v) => MessageEntry::simple(&v),
            YamlEntry::Full {
                value,
                description,
                plural,
            } => {
                let mut e = MessageEntry::simple(&value);
                e.description = description;
                let pf = plural.unwrap_or_default();
                for (cat, tmpl) in pf {
                    let cat = match cat.as_str() {
                        "zero" => zero_ui_i18n::PluralCategory::Zero,
                        "one" => zero_ui_i18n::PluralCategory::One,
                        "two" => zero_ui_i18n::PluralCategory::Two,
                        "few" => zero_ui_i18n::PluralCategory::Few,
                        "many" => zero_ui_i18n::PluralCategory::Many,
                        "other" => zero_ui_i18n::PluralCategory::Other,
                        _ => continue,
                    };
                    e.plural_forms.insert(cat, tmpl);
                }
                e
            }
        };
        messages.insert(MessageId::new(&id), msg_entry);
    }
    Ok(MessageCatalog {
        locale: LocaleId::new(&yc.locale),
        direction,
        messages,
    })
}

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
    // 状态 / 计数（带参数，shell 组装用）。
    /// 导航状态模板：`{back}` / `{fwd}` 为 Text 参数（"◀" / "▶" 或本地化标识）。
    pub const NAV_STATUS: &str = "browser.nav_status";
    /// 书签计数模板：`{count}` 为 Count 参数（支持 plural）。
    pub const N_BOOKMARKS: &str = "browser.n_bookmarks";
    /// 安全状态模板：`{status}` 为安全色名（如 "secure"/"insecure"）。
    pub const SECURITY_STATUS: &str = "browser.security_status";
    // 安全标识 tooltip（SecurityBadge）。
    pub const SECURITY_SECURE: &str = "browser.security.secure";
    pub const SECURITY_INSECURE: &str = "browser.security.insecure";
    pub const SECURITY_MIXED: &str = "browser.security.mixed";
    pub const SECURITY_DANGEROUS: &str = "browser.security.dangerous";
}

/// 默认 locale（`en`）。
pub const DEFAULT_LOCALE: &str = "en";

/// 默认英语文案 catalog（spec FR-013 / DC-10）。
///
/// YAML 规范源在 `browser-ui/chrome/i18n/en-US.yaml`，编译时嵌入。
/// 返回的 catalog 覆盖 [`ids`] 中全部 message id；后续 locale 翻译以同结构新增 YAML 文件
/// 注册到 [`catalog_store`]。
pub fn default_catalog() -> MessageCatalog {
    catalog_from_yaml(include_str!("../../i18n/en-US.yaml"))
        .expect("default catalog YAML 编译时嵌入，必须合法")
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

/// 解析 message id 为可见文案（默认 locale，**infallible**）。
///
/// 组件构造期便利入口：命中 → 文案；缺失/异常 → 回退为 id 本身（不 panic、不阻断）。
/// 缺失 key 仍会在 [`ResolvedText::diagnostic`] 体现（调用方可选查询）；本函数只取 `.text`。
///
/// 用于 chrome 组件把硬编码文案改为 message id 引用（DC-10 浏览器文案接入）：
/// 组件持有 id，构造期经本函数解析为字符串进 WidgetSpec props。
pub fn localized_label(id: &str) -> String {
    resolve_default(&catalog_store(), id)
        .map(|r| r.text)
        .unwrap_or_else(|_| id.to_string())
}

/// 解析 message id 为可见文案（默认 locale，**infallible**，**带参数**）。
///
/// 先经 catalog 解析模板，再经 [`zero_ui_i18n::formatter::format_message`] 替换 `{param}`。
/// 缺失 key 或缺失参数均回退为 id 本身（不 panic、不阻断渲染）。
pub fn localized_label_with_params(id: &str, params: &MessageParams) -> String {
    let store = catalog_store();
    let locale = LocaleId::new(DEFAULT_LOCALE);
    let ctx = context_for(&locale);
    let mref = MessageRef {
        id: MessageId::new(id),
        params: params.clone(),
    };
    match store.resolve(&LocalizedText::Message(mref), &ctx) {
        Ok(resolved) => resolved.text,
        Err(_) => id.to_string(),
    }
}

/// 导航状态文案：`can_go_back` / `can_go_forward` → 本地化字符串（如 "Back·on Fwd·off"）。
///
/// 模板 `browser.nav_status`（`"Back·{back} Fwd·{fwd}"`）由 catalog 提供，{back}/{fwd}
/// 替换为 `"on"` / `"off"` 文本（后续可替换为图标或本地化等价词）。
pub fn nav_status_label(can_go_back: bool, can_go_forward: bool) -> String {
    let mut params = MessageParams::new();
    params.set_text("back", if can_go_back { "on" } else { "off" });
    params.set_text("fwd", if can_go_forward { "on" } else { "off" });
    localized_label_with_params(ids::NAV_STATUS, &params)
}

/// 书签计数文案：`count` → 本地化字符串（如 `"3 bookmarks"` / `"1 bookmark"`）。
pub fn bookmarks_label(count: usize) -> String {
    let mut params = MessageParams::new();
    params.set_count("count", count as i64);
    localized_label_with_params(ids::N_BOOKMARKS, &params)
}

/// 安全状态文案：`status_name`（如 "secure"/"insecure"）→ 本地化字符串（如 "Security: secure"）。
pub fn security_status_label(status_name: &str) -> String {
    let mut params = MessageParams::new();
    params.set_text("status", status_name);
    localized_label_with_params(ids::SECURITY_STATUS, &params)
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
            (ids::SECURITY_SECURE, "Connection is secure"),
            (ids::SECURITY_INSECURE, "Connection is not secure"),
            (ids::SECURITY_MIXED, "This page has mixed content"),
            (ids::SECURITY_DANGEROUS, "Deceptive site ahead"),
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
        // 带 `{param}` 占位的消息需提供虚拟参数。
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
            ids::SECURITY_SECURE,
            ids::SECURITY_INSECURE,
            ids::SECURITY_MIXED,
            ids::SECURITY_DANGEROUS,
        ];
        for id in all_ids {
            assert!(
                resolve_default(&store, id).is_ok(),
                "id {id} missing from default catalog"
            );
        }
        // 参数化 ids：resolve 需提供 params（validate_params 阶段校验 {param}）。
        let param_ids = [ids::NAV_STATUS, ids::N_BOOKMARKS, ids::SECURITY_STATUS];
        for id in param_ids {
            let mref = MessageRef {
                id: MessageId::new(id),
                params: dummy_params_for(id),
            };
            let ctx = context_for(&LocaleId::new(DEFAULT_LOCALE));
            assert!(
                store.resolve(&LocalizedText::Message(mref), &ctx).is_ok(),
                "parametrized id {id} missing from default catalog"
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

    #[test]
    fn localized_label_resolves_known_and_falls_back_for_unknown() {
        // DC-10 组件便利入口：已知 id → 文案；未知 id → id 本身（infallible，不 panic）。
        assert_eq!(localized_label(ids::NEW_TAB), "New Tab");
        assert_eq!(localized_label(ids::RELOAD), "Reload");
        assert_eq!(localized_label("browser.does_not_exist"), "browser.does_not_exist");
    }

    #[test]
    fn localized_label_with_params_substitutes_and_falls_back() {
        // 参数化标签：正常替换 {param}；缺失 key 回落 id。
        let mut p = MessageParams::new();
        p.set_text("back", "on");
        p.set_text("fwd", "off");
        let result = localized_label_with_params(ids::NAV_STATUS, &p);
        assert_eq!(result, "Back·on Fwd·off");

        // 缺失 key → 回落 id。
        let fallback = localized_label_with_params("browser.nope", &p);
        assert_eq!(fallback, "browser.nope");
    }

    #[test]
    fn nav_status_label_produces_on_off() {
        assert_eq!(nav_status_label(true, false), "Back·on Fwd·off");
        assert_eq!(nav_status_label(false, true), "Back·off Fwd·on");
    }

    #[test]
    fn security_status_label_formats_status() {
        assert_eq!(security_status_label("secure"), "Security: secure");
        assert_eq!(security_status_label("insecure"), "Security: insecure");
    }

    #[test]
    fn bookmarks_label_plural() {
        assert_eq!(bookmarks_label(3), "3 bookmarks");
        assert_eq!(bookmarks_label(1), "1 bookmark");
        assert_eq!(bookmarks_label(0), "0 bookmarks");
    }

    /// 为参数化 id 构造包含所有必需占位参数的虚拟 MessageParams。
    fn dummy_params_for(id: &str) -> MessageParams {
        let mut p = MessageParams::new();
        if id == ids::NAV_STATUS {
            p.set_text("back", "x");
            p.set_text("fwd", "x");
        } else if id == ids::N_BOOKMARKS {
            p.set_count("count", 1);
        } else if id == ids::SECURITY_STATUS {
            p.set_text("status", "x");
        }
        p
    }

    #[test]
    fn default_catalog_resolves_parameterized_ids() {
        // DC-10：NAV_STATUS / N_BOOKMARKS 在默认 catalog 中存在且接受参数。
        let store = catalog_store();
        let ctx = context_for(&LocaleId::new(DEFAULT_LOCALE));

        // NAV_STATUS — 填充 {back}/{fwd} 后应正常解析（不含占位残留）。
        let mut nav_p = MessageParams::new();
        nav_p.set_text("back", "on");
        nav_p.set_text("fwd", "off");
        let nav = store
            .resolve(
                &LocalizedText::Message(MessageRef {
                    id: MessageId::new(ids::NAV_STATUS),
                    params: nav_p,
                }),
                &ctx,
            )
            .unwrap_or_else(|e| panic!("NAV_STATUS: {e:?}"));
        assert_eq!(
            nav.text, "Back·on Fwd·off",
            "NAV_STATUS with params should be formatted"
        );

        // N_BOOKMARKS — 填充 {count} 后应正常解析。
        let mut bm_p = MessageParams::new();
        bm_p.set_count("count", 3);
        let bm = store
            .resolve(
                &LocalizedText::Message(MessageRef {
                    id: MessageId::new(ids::N_BOOKMARKS),
                    params: bm_p,
                }),
                &ctx,
            )
            .unwrap_or_else(|e| panic!("N_BOOKMARKS: {e:?}"));
        assert_eq!(bm.text, "3 bookmarks", "N_BOOKMARKS with params should be formatted");
    }
}
