//! # zero-ui-widgets
//!
//! 通用 UI SDK 基础控件（spec §8.4.1 `zero-ui-widgets` / FR-009 / DC-7）。
//!
//! - [`button`]：Button（完整 Widget 实现，event → action）。
//! - [`icon_button`]：IconButton（图标按钮 props）。
//! - [`toggle`]：Toggle（开关 props）。
//! - [`text_input`]：TextInput（retained 编辑状态 + IME caret，DC-8）。
//! - [`scrollbar`]：ScrollBar 几何 + drag → ScrollCommand（DC-4）。
//! - [`menu`]：Menu + ContextMenu；[`popup`]、[`popover`]、[`list_view`]、[`badge`]、[`tooltip`]、[`tabs`]、[`toolbar`]、[`progress`]：skeleton。
//!
//! 覆盖 spec FR-009 首批基础控件（Button/IconButton/TextInput/Toolbar/Menu/ContextMenu/Popup/Popover/ListView/Badge/Tooltip/ScrollBar/ProgressIndicator + Toggle）。
//! 不含浏览器专属语义（URL/书签/下载等属 `browser-ui/chrome`，spec FR-009）。

pub mod badge;
pub mod button;
pub mod colored_box;
pub mod icon;
pub mod icon_button;
pub mod list_view;
pub mod menu;
pub mod popover;
pub mod popup;
pub mod progress;
pub mod scrollbar;
pub mod tabs;
pub mod text_input;
pub mod toggle;
pub mod toolbar;
pub mod tooltip;

pub use button::{Button, ButtonSpec, ButtonVariant};
pub use colored_box::ColoredBox;
pub use icon::{Icon, IconKind};
pub use icon_button::IconButton;
pub use menu::{ContextMenu, Menu, MenuItem};
pub use scrollbar::{ScrollBarGeometry, ScrollOrientation, layout_scrollbar};
pub use text_input::{ACTION_TEXT_CHANGED, TextInputState, TextInputWidget};
pub use toggle::{Toggle, ToggleSpec, ToggleWidget};
