//! Context menu data model and lightweight browser UI localization.

/// Browser UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLanguage {
    /// Simplified Chinese.
    ZhCn,
    /// English.
    EnUs,
}

impl UiLanguage {
    /// Detect the preferred UI language from common environment variables.
    pub fn detect_from_env() -> Self {
        let configured = zero_runtime_config::optional_string("ZERO_BROWSER_UI_LANG");
        for value in configured.into_iter().chain(
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .into_iter()
                .filter_map(|key| std::env::var(key).ok()),
        ) {
            let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
            if normalized.starts_with("zh") {
                return Self::ZhCn;
            }
            if normalized.starts_with("en") {
                return Self::EnUs;
            }
        }
        Self::EnUs
    }
}

/// Browser main menu label keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserMenuLabel {
    /// New tab.
    NewTab,
    /// New private tab.
    NewPrivateTab,
    /// Bookmark this tab.
    BookmarkThisTab,
    /// Show bookmarks bar.
    ShowBookmarksBar,
    /// Hide bookmarks bar.
    HideBookmarksBar,
    /// About browser.
    AboutBrowser,
    /// Settings.
    Settings,
    /// Browsing history page.
    History,
    /// Downloads page.
    Downloads,
    /// Bookmarks manager page.
    BookmarksManager,
}

/// Tab context menu label keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMenuLabel {
    /// Reload tab.
    Reload,
    /// Pin tab.
    Pin,
    /// Unpin tab.
    Unpin,
    /// Mute tab.
    Mute,
    /// Unmute tab.
    Unmute,
    /// Duplicate tab.
    Duplicate,
    /// Close other tabs.
    CloseOthers,
    /// Close tabs to the right.
    CloseToRight,
    /// Close tab.
    Close,
}

/// Resolve a tab context menu label for the given UI language.
pub fn tab_menu_label(label: TabMenuLabel, language: UiLanguage) -> &'static str {
    match language {
        UiLanguage::ZhCn => match label {
            TabMenuLabel::Reload => "重新加载",
            TabMenuLabel::Pin => "固定标签页",
            TabMenuLabel::Unpin => "取消固定标签页",
            TabMenuLabel::Mute => "静音标签页",
            TabMenuLabel::Unmute => "取消静音标签页",
            TabMenuLabel::Duplicate => "复制标签页",
            TabMenuLabel::CloseOthers => "关闭其他标签页",
            TabMenuLabel::CloseToRight => "关闭右侧标签页",
            TabMenuLabel::Close => "关闭标签页",
        },
        UiLanguage::EnUs => match label {
            TabMenuLabel::Reload => "Reload",
            TabMenuLabel::Pin => "Pin Tab",
            TabMenuLabel::Unpin => "Unpin Tab",
            TabMenuLabel::Mute => "Mute Tab",
            TabMenuLabel::Unmute => "Unmute Tab",
            TabMenuLabel::Duplicate => "Duplicate Tab",
            TabMenuLabel::CloseOthers => "Close Other Tabs",
            TabMenuLabel::CloseToRight => "Close Tabs to the Right",
            TabMenuLabel::Close => "Close Tab",
        },
    }
}

/// Resolve a browser main menu label for the given UI language.
pub fn browser_menu_label(label: BrowserMenuLabel, language: UiLanguage) -> &'static str {
    match language {
        UiLanguage::ZhCn => match label {
            BrowserMenuLabel::NewTab => "新建标签页",
            BrowserMenuLabel::NewPrivateTab => "新建无痕标签页",
            BrowserMenuLabel::BookmarkThisTab => "收藏当前标签页",
            BrowserMenuLabel::ShowBookmarksBar => "显示书签栏",
            BrowserMenuLabel::HideBookmarksBar => "隐藏书签栏",
            BrowserMenuLabel::AboutBrowser => "关于 ZeroBrowser",
            BrowserMenuLabel::Settings => "设置",
            BrowserMenuLabel::History => "历史记录",
            BrowserMenuLabel::Downloads => "下载内容",
            BrowserMenuLabel::BookmarksManager => "书签管理",
        },
        UiLanguage::EnUs => match label {
            BrowserMenuLabel::NewTab => "New Tab",
            BrowserMenuLabel::NewPrivateTab => "New Private Tab",
            BrowserMenuLabel::BookmarkThisTab => "Bookmark This Tab",
            BrowserMenuLabel::ShowBookmarksBar => "Show Bookmarks Bar",
            BrowserMenuLabel::HideBookmarksBar => "Hide Bookmarks Bar",
            BrowserMenuLabel::AboutBrowser => "About ZeroBrowser",
            BrowserMenuLabel::Settings => "Settings",
            BrowserMenuLabel::History => "History",
            BrowserMenuLabel::Downloads => "Downloads",
            BrowserMenuLabel::BookmarksManager => "Bookmarks",
        },
    }
}

/// Context menu item.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    id: String,
    label: String,
    item_type: MenuItemType,
    /// 是否启用。`false` 时渲染为灰显且不可点击。
    enabled: bool,
    /// 可选的快捷键提示文本（如 "Ctrl+T"），由 GUI 右对齐显示。
    /// 仅为展示用字符串，UI-agnostic。
    shortcut: Option<String>,
}

/// Context menu item type.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemType {
    /// Clickable action item.
    Action,
    /// Separator line.
    Separator,
    /// Submenu with children.
    SubMenu(Vec<MenuItem>),
}

impl MenuItem {
    /// Create an action item.
    pub fn action(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type: MenuItemType::Action,
            enabled: true,
            shortcut: None,
        }
    }

    /// Create a disabled action item.
    pub fn action_disabled(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type: MenuItemType::Action,
            enabled: false,
            shortcut: None,
        }
    }

    /// Create a separator item.
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            item_type: MenuItemType::Separator,
            enabled: false,
            shortcut: None,
        }
    }

    /// Create a submenu item.
    pub fn sub_menu(id: &str, label: &str, children: Vec<MenuItem>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            item_type: MenuItemType::SubMenu(children),
            enabled: true,
            shortcut: None,
        }
    }

    /// Builder：为该项附加快捷键提示文本（如 "Ctrl+T"）。
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// 就地设置 enabled 状态（无需重建 MenuItem）。
    /// 用于 GUI 在运行时根据页面能力禁用菜单项，
    /// 例如 about:blank / 内部页禁用"查看源代码"。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Item id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Visible label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether this item is enabled (clickable).
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Shortcut hint text (e.g. "Ctrl+T") shown right-aligned by the GUI.
    pub fn shortcut(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }

    /// Whether this item is a separator.
    pub fn is_separator(&self) -> bool {
        self.item_type == MenuItemType::Separator
    }

    /// Whether this item is a submenu.
    pub fn is_sub_menu(&self) -> bool {
        matches!(self.item_type, MenuItemType::SubMenu(_))
    }

    /// Children if this item is a submenu.
    pub fn children(&self) -> Option<&[MenuItem]> {
        match &self.item_type {
            MenuItemType::SubMenu(children) => Some(children),
            _ => None,
        }
    }
}

/// Context type that determines the default menu items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// Blank page area.
    Page,
    /// Link.
    Link,
    /// Image.
    Image,
    /// Selected text.
    Selection,
    /// Editable region.
    Editable,
}

/// Browser context menu.
#[derive(Debug, Clone)]
pub struct ContextMenu {
    context_type: ContextType,
    items: Vec<MenuItem>,
    source_url: Option<String>,
}

impl ContextMenu {
    /// Create a default menu for the given context.
    pub fn new(context_type: ContextType) -> Self {
        let items = default_items_for_context(context_type);
        Self {
            context_type,
            items,
            source_url: None,
        }
    }

    /// Create a default menu for the given context with an associated URL.
    pub fn with_url(context_type: ContextType, url: &str) -> Self {
        let items = default_items_for_context(context_type);
        Self {
            context_type,
            items,
            source_url: Some(url.to_string()),
        }
    }

    /// Create a menu with custom items.
    pub fn with_items(context_type: ContextType, items: Vec<MenuItem>) -> Self {
        Self {
            context_type,
            items,
            source_url: None,
        }
    }

    /// Context type.
    pub fn context_type(&self) -> ContextType {
        self.context_type
    }

    /// Menu items.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Optional source URL.
    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    /// Number of menu items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the menu has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Find a menu item by id.
    pub fn find_item(&self, id: &str) -> Option<&MenuItem> {
        find_item_recursive(&self.items, id)
    }
}

fn find_item_recursive<'a>(items: &'a [MenuItem], id: &str) -> Option<&'a MenuItem> {
    for item in items {
        if item.id == id {
            return Some(item);
        }
        if let Some(children) = item.children()
            && let Some(found) = find_item_recursive(children, id)
        {
            return Some(found);
        }
    }
    None
}

fn default_items_for_context(context_type: ContextType) -> Vec<MenuItem> {
    let lang = UiLanguage::detect_from_env();
    match context_type {
        ContextType::Page => match lang {
            UiLanguage::ZhCn => vec![
                MenuItem::action("back", "后退"),
                MenuItem::action("forward", "前进"),
                MenuItem::action("reload", "重新加载"),
                MenuItem::separator(),
                MenuItem::action("save_as", "另存为..."),
                MenuItem::action("print", "打印..."),
                MenuItem::separator(),
                MenuItem::action("view_source", "查看源代码"),
                MenuItem::action("inspect", "检查元素"),
            ],
            UiLanguage::EnUs => vec![
                MenuItem::action("back", "Back"),
                MenuItem::action("forward", "Forward"),
                MenuItem::action("reload", "Reload"),
                MenuItem::separator(),
                MenuItem::action("save_as", "Save As..."),
                MenuItem::action("print", "Print..."),
                MenuItem::separator(),
                MenuItem::action("view_source", "View Source"),
                MenuItem::action("inspect", "Inspect"),
            ],
        },
        ContextType::Link => match lang {
            UiLanguage::ZhCn => vec![
                MenuItem::action("open_link", "在新标签页中打开链接"),
                MenuItem::action("copy_link", "复制链接地址"),
                MenuItem::separator(),
                MenuItem::action("save_link", "将链接另存为..."),
                MenuItem::action("bookmark_link", "将链接添加为书签"),
            ],
            UiLanguage::EnUs => vec![
                MenuItem::action("open_link", "Open Link in New Tab"),
                MenuItem::action("copy_link", "Copy Link Address"),
                MenuItem::separator(),
                MenuItem::action("save_link", "Save Link As..."),
                MenuItem::action("bookmark_link", "Bookmark Link"),
            ],
        },
        ContextType::Image => match lang {
            UiLanguage::ZhCn => vec![
                MenuItem::action("open_image", "在新标签页中打开图片"),
                MenuItem::action("copy_image_url", "复制图片地址"),
                MenuItem::action("save_image", "将图片另存为..."),
                MenuItem::separator(),
                MenuItem::action("copy_image", "复制图片"),
            ],
            UiLanguage::EnUs => vec![
                MenuItem::action("open_image", "Open Image in New Tab"),
                MenuItem::action("copy_image_url", "Copy Image Address"),
                MenuItem::action("save_image", "Save Image As..."),
                MenuItem::separator(),
                MenuItem::action("copy_image", "Copy Image"),
            ],
        },
        ContextType::Selection => match lang {
            UiLanguage::ZhCn => vec![
                MenuItem::action("copy", "复制"),
                MenuItem::action("search_selection", "使用搜索引擎搜索"),
                MenuItem::separator(),
                MenuItem::action("print", "打印..."),
            ],
            UiLanguage::EnUs => vec![
                MenuItem::action("copy", "Copy"),
                MenuItem::action("search_selection", "Search with Default Engine"),
                MenuItem::separator(),
                MenuItem::action("print", "Print..."),
            ],
        },
        ContextType::Editable => match lang {
            UiLanguage::ZhCn => vec![
                MenuItem::action("cut", "剪切"),
                MenuItem::action("copy", "复制"),
                MenuItem::action("paste", "粘贴"),
                MenuItem::action("select_all", "全选"),
                MenuItem::separator(),
                MenuItem::action("undo", "撤销"),
                MenuItem::action("redo", "重做"),
            ],
            UiLanguage::EnUs => vec![
                MenuItem::action("cut", "Cut"),
                MenuItem::action("copy", "Copy"),
                MenuItem::action("paste", "Paste"),
                MenuItem::action("select_all", "Select All"),
                MenuItem::separator(),
                MenuItem::action("undo", "Undo"),
                MenuItem::action("redo", "Redo"),
            ],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_menu_label_is_localized_for_chinese() {
        assert_eq!(
            browser_menu_label(BrowserMenuLabel::AboutBrowser, UiLanguage::ZhCn),
            "关于 ZeroBrowser"
        );
        assert_eq!(browser_menu_label(BrowserMenuLabel::Settings, UiLanguage::ZhCn), "设置");
    }

    #[test]
    fn browser_menu_label_is_localized_for_english() {
        assert_eq!(
            browser_menu_label(BrowserMenuLabel::AboutBrowser, UiLanguage::EnUs),
            "About ZeroBrowser"
        );
        assert_eq!(
            browser_menu_label(BrowserMenuLabel::Settings, UiLanguage::EnUs),
            "Settings"
        );
    }

    #[test]
    fn tab_menu_label_is_localized() {
        assert_eq!(tab_menu_label(TabMenuLabel::Pin, UiLanguage::ZhCn), "固定标签页");
        assert_eq!(tab_menu_label(TabMenuLabel::Close, UiLanguage::EnUs), "Close Tab");
    }
}
