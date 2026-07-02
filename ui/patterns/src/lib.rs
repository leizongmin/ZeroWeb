//! # zero-ui-patterns
//!
//! 通用 UI SDK 组合模式（spec §8.4.1 `zero-ui-patterns` / FR-009 / DC-7）。
//!
//! 由 `ui/widgets` 组合而成的跨应用模式：
//! [`search_field`]、[`suggestion_list`]、[`command_palette`]、[`data_list`]、
//! [`status_bubble`]、[`tab_bar`]、[`dialog_scaffold`]。
//!
//! 浏览器领域组件（AddressBar/BrowserTabStrip 等）在 `browser-ui/chrome`，不在此 crate。

pub mod command_palette;
pub mod data_list;
pub mod dialog_scaffold;
pub mod search_field;
pub mod status_bubble;
pub mod suggestion_list;
pub mod tab_bar;

pub use command_palette::{CommandEntry, CommandPalette};
pub use search_field::SearchField;
pub use suggestion_list::{Suggestion, SuggestionList};
pub use tab_bar::TabBar;
