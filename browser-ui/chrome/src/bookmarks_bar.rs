//! BookmarksBar — 书签栏（spec §8.4.1A）。
//!
//! 组合通用 [`Toolbar`]（+ 文件夹下拉 [`Menu`] 由 shell 在 overlay 层注入子项）；
//! 点击书签发出 navigate / open-bookmark action。bookmark tree 来自 browser-shell。

use crate::browser_action::BrowserAction;
use zero_ui_widgets::toolbar::{Toolbar, ToolbarItem};

/// 单个书签节点（URL 书签或文件夹）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkNode {
    /// 稳定书签 id（toolbar item id 用）。
    pub id: String,
    pub title: String,
    /// `None` = 文件夹（点击展开子项，不直接导航）。
    pub url: Option<String>,
}

/// 书签栏（props）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookmarksBar {
    pub roots: Vec<BookmarkNode>,
}

impl BookmarksBar {
    pub fn new(roots: Vec<BookmarkNode>) -> BookmarksBar {
        BookmarksBar { roots }
    }

    /// 组合通用 toolbar：每个根书签一个按钮（文件夹/URL 均占一位）。
    pub fn build_toolbar(&self) -> Toolbar {
        let mut tb = Toolbar::new();
        for b in &self.roots {
            tb.push(ToolbarItem::new(&b.id, &format!("bookmark.{}", b.id)));
        }
        tb
    }

    /// 是否为文件夹书签（文件夹下拉由 shell 用通用 Menu 注入子书签）。
    pub fn is_folder(&self, bookmark_id: &str) -> bool {
        self.roots.iter().any(|b| b.id == bookmark_id && b.url.is_none())
    }

    /// toolbar item id → BrowserAction：
    /// URL 书签 → Navigate(url)；文件夹 → OpenBookmark(id)；未知 → None。
    pub fn on_activate(&self, bookmark_id: &str) -> Option<BrowserAction> {
        let b = self.roots.iter().find(|b| b.id == bookmark_id)?;
        match &b.url {
            Some(url) => Some(BrowserAction::Navigate(url.clone())),
            None => Some(BrowserAction::OpenBookmark(b.id.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BookmarksBar {
        BookmarksBar::new(vec![
            BookmarkNode {
                id: "mdn".into(),
                title: "MDN".into(),
                url: Some("https://developer.mozilla.org".into()),
            },
            BookmarkNode {
                id: "dev".into(),
                title: "Dev".into(),
                url: None,
            },
        ])
    }

    #[test]
    fn toolbar_has_item_per_root() {
        let tb = sample().build_toolbar();
        assert_eq!(tb.items.len(), 2);
    }

    #[test]
    fn url_bookmark_navigates_folder_opens() {
        let bar = sample();
        assert_eq!(
            bar.on_activate("mdn"),
            Some(BrowserAction::Navigate("https://developer.mozilla.org".into()))
        );
        // 文件夹 → OpenBookmark。
        assert_eq!(bar.on_activate("dev"), Some(BrowserAction::OpenBookmark("dev".into())));
        assert!(bar.is_folder("dev"));
        assert!(!bar.is_folder("mdn"));
    }

    #[test]
    fn unknown_id_is_none() {
        assert!(sample().on_activate("nope").is_none());
    }
}
