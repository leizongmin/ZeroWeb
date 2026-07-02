//! # zero-ui-widgets
//!
//! 通用 UI SDK 基础控件（spec §8.4.1 `zero-ui-widgets` / FR-009 / DC-7）。
//!
//! - [`button`]：Button（完整 Widget 实现，event → action）。
//! - [`text_input`]：TextInput（retained 编辑状态 + IME caret，DC-8）。
//! - [`scrollbar`]：ScrollBar 几何 + drag → ScrollCommand（DC-4）。
//! - [`menu`]、[`popup`]、[`popover`]、[`list_view`]、[`badge`]、[`tooltip`]、[`tabs`]、[`toolbar`]、[`progress`]：skeleton。
//!
//! 不含浏览器专属语义（URL/书签/下载等属 `browser-ui/chrome`，spec FR-009）。

pub mod badge;
pub mod button;
pub mod list_view;
pub mod menu;
pub mod popover;
pub mod popup;
pub mod progress;
pub mod scrollbar;
pub mod tabs;
pub mod text_input;
pub mod toolbar;
pub mod tooltip;

pub use button::{Button, ButtonSpec};
pub use scrollbar::{ScrollBarGeometry, ScrollOrientation, layout_scrollbar};
pub use text_input::TextInputState;
