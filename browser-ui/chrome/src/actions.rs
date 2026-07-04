//! Chrome widget 发出的稳定 ActionId（spec §8.4.1A / IF-002）。
//!
//! chrome widget 在 `Widget::event` 识别指针/键盘交互后，emit 这些点分 id；
//! apps/browser 的 action reducer 监听并映射到 [`crate::BrowserAction`] 或 `BrowserCommandId`
//!（spec §8.4.1B 同 command 多入口）。
//!
//! 命名约定：`browser.<target>.<verb>`，与 [`crate::i18n::ids`] 同前缀避免歧义。

use zero_ui_core::action::ActionId;

/// nav 段 4 按钮（back / forward / reload-stop / home）。
pub const NAV_BACK: &str = "browser.nav.back";
pub const NAV_FORWARD: &str = "browser.nav.forward";
/// reload / stop（loading 时 = stop，否则 = reload）—— 由 reducer 据 loading 状态分支。
pub const NAV_RELOAD_OR_STOP: &str = "browser.nav.reload_or_stop";
pub const NAV_HOME: &str = "browser.nav.home";

/// toolbar menu（更多）按钮：展开 / 关闭 menu。
pub const MENU_TOGGLE: &str = "browser.menu.toggle";

/// toolbar trailing 图标：download / theme。
pub const TRAILING_DOWNLOAD: &str = "browser.trailing.download";
pub const TRAILING_THEME: &str = "browser.trailing.theme";

/// 地址栏：点击聚焦（host 内置焦点逻辑）+ 提交（Enter）。
pub const ADDRESS_SUBMIT: &str = "browser.address.submit";

/// 标签页：activate（点击 tab）/ close（点击 ×）/ new（点击 +）。
pub const TAB_ACTIVATE: &str = "browser.tab.activate";
pub const TAB_CLOSE: &str = "browser.tab.close";
pub const TAB_NEW: &str = "browser.tab.new";

/// 书签栏：点击书签导航（payload = bookmark id）。
pub const BOOKMARK_OPEN: &str = "browser.bookmark.open";

/// 查找栏：next / prev / close。
pub const FIND_NEXT: &str = "browser.find.next";
pub const FIND_PREV: &str = "browser.find.prev";
pub const FIND_CLOSE: &str = "browser.find.close";

/// 便捷构造。
pub fn id(name: &str) -> ActionId {
    ActionId::new(name)
}
