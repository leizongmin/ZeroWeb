//! Browser UI layout constants.

/// Shared font size for browser chrome text in logical pixels.
pub const CHROME_FONT_SIZE: f32 = 13.0;
/// Shared icon size for top chrome controls in logical pixels.
pub const CHROME_ICON_SIZE: f32 = 16.0;
/// Height of the tab body area.
pub const TAB_BAR_HEIGHT: f32 = 34.0;
/// Space between the top window edge and the tab strip.
pub const TAB_BAR_TOP_INSET: f32 = 6.0;
/// Total height of the tab strip including the top inset.
pub const TAB_STRIP_HEIGHT: f32 = TAB_BAR_TOP_INSET + TAB_BAR_HEIGHT;
/// Height of the address row.
pub const ADDRESS_BAR_HEIGHT: f32 = 50.0;
/// Horizontal padding around the address row.
pub const ADDRESS_BAR_PADDING: f32 = 10.0;
/// Vertical inset used to turn the address row into a pill control.
pub const ADDRESS_BAR_INPUT_V_INSET: f32 = 6.0;
/// Extra text top/bottom padding inside the address bar pill.
pub const ADDRESS_BAR_TEXT_V_PAD: f32 = 3.0;
/// Horizontal inner padding inside the address bar.
pub const ADDRESS_BAR_INNER_PAD_H: f32 = 12.0;
/// Reserved leading slot for site/status affordances in the address bar.
pub const ADDRESS_BAR_LEADING_SLOT_WIDTH: f32 = 28.0;
/// Reserved trailing padding inside the address bar.
pub const ADDRESS_BAR_TRAILING_PAD: f32 = 12.0;
/// Width of one trailing action slot inside the address bar pill.
pub const ADDRESS_BAR_ACTION_SLOT_WIDTH: f32 = 28.0;
/// Number of trailing action slots (bookmark, permissions).
pub const ADDRESS_BAR_TRAILING_SLOT_COUNT: f32 = 2.0;
/// Total width reserved for trailing action slots inside the address bar pill.
pub const ADDRESS_BAR_TRAILING_SLOTS: f32 = ADDRESS_BAR_ACTION_SLOT_WIDTH * ADDRESS_BAR_TRAILING_SLOT_COUNT;
/// Gap between the address bar and the trailing toolbar actions.
pub const TOOLBAR_TRAILING_GAP: f32 = 8.0;
/// Width of the trailing download button slot.
pub const TOOLBAR_DOWNLOAD_BUTTON_WIDTH: f32 = 32.0;
/// Width of the trailing color-theme toggle button slot.
pub const TOOLBAR_THEME_BUTTON_WIDTH: f32 = 32.0;
/// Width of the trailing browser menu button slot.
pub const TOOLBAR_MENU_BUTTON_WIDTH: f32 = 32.0;
/// Total height of the toolbar: tab strip plus address row.
pub const TOOLBAR_HEIGHT: f32 = TAB_STRIP_HEIGHT + ADDRESS_BAR_HEIGHT;
/// Width of the new tab button slot.
pub const NEW_TAB_BTN_WIDTH: f32 = 34.0;
/// Width of one custom window control button.
pub const WINDOW_CONTROL_BTN_WIDTH: f32 = 46.0;
/// Total width reserved for custom window controls.
pub const WINDOW_CONTROLS_WIDTH: f32 = WINDOW_CONTROL_BTN_WIDTH * 3.0;
/// Size of the compact hover background behind a window control icon.
pub const WINDOW_CONTROL_HOVER_SIZE: f32 = 28.0;
/// Corner radius of the compact hover background behind a window control icon.
pub const WINDOW_CONTROL_HOVER_RADIUS: f32 = 7.0;
/// Width of one navigation button slot.
pub const NAV_BUTTON_WIDTH: f32 = 36.0;
/// Leading space before the first navigation button.
pub const NAV_SECTION_LEADING_PAD: f32 = 10.0;
/// Space between the navigation section and the address bar.
pub const NAV_SECTION_TRAILING_GAP: f32 = 10.0;
/// Hover circle diameter used by navigation and new-tab controls.
pub const NAV_BUTTON_HOVER_DIAMETER: f32 = 28.0;
/// Minimum width of a pinned tab.
pub const TAB_PINNED_WIDTH: f32 = 52.0;
/// Minimum width of a tab.
pub const TAB_MIN_WIDTH: f32 = 100.0;
/// Minimum width when the tab strip is crowded.
pub const TAB_MIN_WIDTH_COMPRESSED: f32 = 52.0;
/// 极限最小宽度：标签多到正常压缩仍溢出时，进一步压缩到此宽度，
/// 只保留 favicon（无 close 按钮、无文本），确保所有标签都可见。
pub const TAB_ABSOLUTE_MIN_WIDTH: f32 = 36.0;
/// Hide tab title text below this tab width (physical layout units before scale).
pub const TAB_TITLE_HIDE_WIDTH: f32 = 84.0;
/// Maximum width of a tab.
pub const TAB_MAX_WIDTH: f32 = 240.0;
/// Close button icon box size inside a tab.
pub const TAB_CLOSE_SIZE: f32 = 16.0;
/// Top radius of a tab.
pub const TAB_TOP_RADIUS: f32 = 12.0;
/// Bottom foot radius of the active tab shape.
pub const TAB_FOOT_RADIUS: f32 = 7.0;
/// Favicon size inside a tab.
pub const TAB_ICON_SIZE: f32 = 14.0;
/// Vertical inset of separators between adjacent inactive tabs.
pub const TAB_SEPARATOR_INSET: f32 = 8.0;
/// Maximum visible rows in the autocomplete dropdown.
pub const AUTOCOMPLETE_MAX_VISIBLE: usize = 6;
/// Height of one autocomplete row.
pub const AUTOCOMPLETE_ROW_HEIGHT: f32 = 44.0;
/// Corner radius of the autocomplete dropdown panel.
pub const AUTOCOMPLETE_DROPDOWN_RADIUS: f32 = 8.0;
/// Horizontal padding inside an autocomplete row.
pub const AUTOCOMPLETE_ROW_PAD_H: f32 = 12.0;
/// Vertical padding inside an autocomplete row.
pub const AUTOCOMPLETE_ROW_PAD_V: f32 = 6.0;
/// Height of the bookmarks bar.
pub const BOOKMARKS_BAR_HEIGHT: f32 = 28.0;
/// Horizontal padding at the start of the bookmarks bar.
pub const BOOKMARKS_BAR_PAD_H: f32 = 8.0;
/// Horizontal padding inside a bookmark item pill.
pub const BOOKMARKS_BAR_ITEM_PAD_H: f32 = 8.0;
/// Gap between bookmark icon and label text.
pub const BOOKMARKS_BAR_ICON_GAP: f32 = 6.0;
/// Gap between bookmark items.
pub const BOOKMARKS_BAR_ITEM_GAP: f32 = 4.0;
/// Corner radius of a bookmark item hover pill.
pub const BOOKMARKS_BAR_ITEM_RADIUS: f32 = 4.0;
/// Icon size in the bookmarks bar.
pub const BOOKMARKS_BAR_ICON_SIZE: f32 = 14.0;
/// Text size in the bookmarks bar.
pub const BOOKMARKS_BAR_FONT_SIZE: f32 = 12.0;
/// macOS inset reserved for traffic lights in unified titlebar mode.
pub const MACOS_TRAFFIC_LIGHT_INSET: f32 = 78.0;
/// Height of the find bar.
pub const FIND_BAR_HEIGHT: f32 = 36.0;
/// Width of the floating find bar panel.
pub const FIND_BAR_WIDTH: f32 = 380.0;
/// Margin from the content frame edge for the floating find bar.
pub const FIND_BAR_FLOAT_MARGIN: f32 = 12.0;
/// Corner radius of the floating find bar panel.
pub const FIND_BAR_FLOAT_RADIUS: f32 = 8.0;
/// Height of the floating status bar used for hovered links.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;
/// Margin between the floating status bar and the content frame edge.
pub const STATUS_BAR_FLOAT_MARGIN: f32 = 8.0;
/// Horizontal padding inside the floating status bar.
pub const STATUS_BAR_FLOAT_PAD_H: f32 = 8.0;
/// Radius of the floating status bar.
pub const STATUS_BAR_FLOAT_RADIUS: f32 = 8.0;
/// Height of the legacy download bar slot (kept for tests referencing height).
pub const DOWNLOAD_BAR_HEIGHT: f32 = 28.0;
/// Width of the floating download panel.
pub const DOWNLOAD_PANEL_WIDTH: f32 = 280.0;
/// Height of the floating download panel.
pub const DOWNLOAD_PANEL_HEIGHT: f32 = 72.0;
/// Margin from the content frame edge for the floating download panel.
pub const DOWNLOAD_PANEL_FLOAT_MARGIN: f32 = 12.0;
/// Corner radius of the floating download panel.
pub const DOWNLOAD_PANEL_RADIUS: f32 = 8.0;
/// Width of the context menu panel.
pub const CONTEXT_MENU_WIDTH: f32 = 220.0;
/// Row height inside the context menu.
pub const CONTEXT_MENU_ROW_HEIGHT: f32 = 32.0;
/// Compact row height for separator items (visual breathing room without bloating the menu).
pub const CONTEXT_MENU_SEPARATOR_HEIGHT: f32 = 10.0;
/// Corner radius of the context menu panel.
pub const CONTEXT_MENU_RADIUS: f32 = 8.0;
/// Horizontal padding inside a context menu row.
pub const CONTEXT_MENU_PAD_H: f32 = 12.0;
/// Horizontal inset of the page frame relative to chrome.
/// 保持 0：内容区不缩水；减重视觉靠圆角与边框，不靠 gutter（参考 Chrome 非最大化模式）。
pub const PAGE_FRAME_INSET_H: f32 = 0.0;
/// Top gap between chrome and the page frame.
pub const PAGE_FRAME_INSET_TOP: f32 = 0.0;
/// Bottom gap between the page frame and the outer window edge.
pub const PAGE_FRAME_INSET_BOTTOM: f32 = 0.0;
/// Extra bottom reserve used to prevent clipping in maximized windows.
pub const PAGE_FRAME_BOTTOM_CLIP_GUARD: f32 = 24.0;
/// Additional bottom reserve for floating UI in maximized windows.
pub const PAGE_FRAME_BOTTOM_UI_GUARD: f32 = STATUS_BAR_HEIGHT;
/// Border width of the page frame (非最大化时；最大化时运行时归零)。
pub const PAGE_FRAME_BORDER: f32 = 1.0;
/// Radius of the page frame (非最大化时；最大化时运行时归零)。
pub const PAGE_FRAME_RADIUS: f32 = 4.0;
/// 页面视口 drop shadow 模糊半径（非最大化时；最大化时运行时归零）。
pub const PAGE_FRAME_SHADOW_BLUR: f32 = 6.0;
/// 页面视口 drop shadow 垂直偏移（非最大化时；最大化时运行时归零）。
pub const PAGE_FRAME_SHADOW_OFFSET_Y: f32 = 1.0;
/// 页面视口 drop shadow 不透明度（叠加在黑色之上）。
pub const PAGE_FRAME_SHADOW_ALPHA: u8 = 22;
/// Thickness of classic reserved scrollbars in logical pixels.
pub const SCROLLBAR_THICKNESS: f32 = 10.0;
/// Minimum scrollbar thumb length in logical pixels.
pub const SCROLLBAR_MIN_THUMB: f32 = 32.0;
/// Width of the custom outer frame border used on undecorated Wayland windows.
pub const WINDOW_FRAME_BORDER: f32 = 1.0;
