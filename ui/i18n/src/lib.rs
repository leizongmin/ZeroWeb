//! # zero-ui-i18n
//!
//! 通用 UI SDK 国际化资源系统（spec FR-013 / IF-007 / DC-10）。
//!
//! 提供 locale/catalog/fallback/参数替换/plural/text direction 的最小可用实现：
//! - [`locale`]：`LocaleId` 与父级派生。
//! - [`message`]：`MessageId`/`LocalizedText`/`MessageRef`/`MessageEntry`。
//! - [`catalog`]：`MessageCatalog`/`I18nContext`/`I18nProvider` + `CatalogStore` 实现。
//! - [`fallback`]：fallback chain 生成。
//! - [`plural`]：CLDR plural category（M1 英语规则）。
//! - [`direction`]：locale → RTL/LTR 方向。
//! - [`formatter`]：`{param}` 占位替换 + plural 变体选择。
//! - [`diagnostics`]：`I18nError`。
//!
//! 浏览器文案不在此 crate；属 `browser-ui/chrome/i18n` 或 `apps/browser/i18n`（spec FR-013）。
//! 完整 CLDR plural / ICU formatting 留 M2 评估（TBD-7）。

pub mod catalog;
pub mod diagnostics;
pub mod direction;
pub mod fallback;
pub mod formatter;
pub mod locale;
pub mod message;
pub mod plural;

pub use catalog::{CatalogStore, I18nContext, I18nProvider, MessageCatalog, ResolvedText};
pub use diagnostics::I18nError;
pub use direction::{TextDirection, direction_for};
pub use fallback::fallback_chain;
pub use locale::LocaleId;
pub use message::{LocalizedText, MessageEntry, MessageId, MessageParams, MessageRef};
pub use plural::{PluralCategory, plural_category};
