//! 标签页 favicon 异步拉取（后台线程，主线程 poll 注册位图）。

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use zero_browser_shell::TabId;
use zero_render_foundation::font::GlyphBitmap;
use zero_render_foundation::font::loader::FontLoader;

use crate::tab_favicon;

struct PendingFaviconFetch {
    tab_id: TabId,
    favicon_url: String,
    size_px: f32,
    rx: Receiver<Option<GlyphBitmap>>,
}

struct PendingBookmarkFaviconFetch {
    bookmark_url: String,
    favicon_url: String,
    size_px: f32,
    rx: Receiver<Option<GlyphBitmap>>,
}

/// 进行中的 favicon 网络请求队列。
pub struct FaviconFetchState {
    pending: Vec<PendingFaviconFetch>,
    pending_bookmarks: Vec<PendingBookmarkFaviconFetch>,
    /// 已完成（成功或失败）的书签 favicon URL，避免重复发起。
    completed_bookmark_urls: std::collections::HashSet<String>,
}

impl FaviconFetchState {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            pending_bookmarks: Vec::new(),
            completed_bookmark_urls: std::collections::HashSet::new(),
        }
    }

    /// 在后台拉取 favicon；若已有相同 Tab + URL 的请求则忽略。
    pub fn request(&mut self, tab_id: TabId, page_url: &str, html: Option<&str>, size_px: f32) {
        if tab_favicon::skip_favicon_fetch(page_url) {
            return;
        }
        let Some(favicon_url) = tab_favicon::pick_favicon_url(page_url, html) else {
            return;
        };
        if self
            .pending
            .iter()
            .any(|p| p.tab_id == tab_id && p.favicon_url == favicon_url)
        {
            return;
        }

        let (tx, rx) = mpsc::channel();
        let fetch_url = favicon_url.clone();
        thread::spawn(move || {
            let bitmap = tab_favicon::fetch_favicon_bitmap(&fetch_url, size_px);
            let _ = tx.send(bitmap);
        });

        self.pending.push(PendingFaviconFetch {
            tab_id,
            favicon_url,
            size_px,
            rx,
        });
    }

    /// 轮询已完成请求并注册位图；有更新时返回 `true`。
    pub fn poll(&mut self, font_loader: &mut FontLoader) -> bool {
        let mut changed = false;
        self.pending.retain_mut(|pending| match pending.rx.try_recv() {
            Ok(Some(bitmap)) => {
                tab_favicon::register_tab_favicon_bitmap(font_loader, pending.tab_id, pending.size_px, bitmap);
                changed = true;
                false
            }
            Ok(None) => false,
            Err(TryRecvError::Empty) => true,
            Err(TryRecvError::Disconnected) => false,
        });
        changed
    }

    /// Tab 导航或关闭时取消尚未完成的请求。
    pub fn cancel_tab(&mut self, tab_id: TabId) {
        self.pending.retain(|p| p.tab_id != tab_id);
    }

    /// 为书签请求 favicon（按书签 URL 去重；内部页跳过；已完成的不再重发）。
    pub fn request_bookmark(&mut self, bookmark_url: &str, size_px: f32) {
        if tab_favicon::skip_favicon_fetch(bookmark_url) {
            return;
        }
        if self.completed_bookmark_urls.contains(bookmark_url) {
            return;
        }
        let page_url = bookmark_url.to_string();
        let Some(favicon_url) = tab_favicon::pick_favicon_url(&page_url, None) else {
            return;
        };
        if self.pending_bookmarks.iter().any(|p| p.bookmark_url == page_url) {
            return;
        }

        let (tx, rx) = mpsc::channel();
        let fetch_url = favicon_url.clone();
        thread::spawn(move || {
            let bitmap = tab_favicon::fetch_favicon_bitmap(&fetch_url, size_px);
            let _ = tx.send(bitmap);
        });

        self.pending_bookmarks.push(PendingBookmarkFaviconFetch {
            bookmark_url: page_url,
            favicon_url,
            size_px,
            rx,
        });
    }

    /// 轮询书签 favicon 请求；有更新时返回 `true`。
    pub fn poll_bookmarks(&mut self, font_loader: &mut FontLoader) -> bool {
        let mut changed = false;
        let completed = &mut self.completed_bookmark_urls;
        self.pending_bookmarks
            .retain_mut(|pending| match pending.rx.try_recv() {
                Ok(Some(bitmap)) => {
                    tab_favicon::register_bookmark_favicon_bitmap(
                        font_loader,
                        &pending.bookmark_url,
                        pending.size_px,
                        bitmap,
                    );
                    completed.insert(pending.bookmark_url.clone());
                    changed = true;
                    false
                }
                Ok(None) => false,
                Err(TryRecvError::Empty) => true,
                Err(TryRecvError::Disconnected) => {
                    completed.insert(pending.bookmark_url.clone());
                    false
                }
            });
        changed
    }
}

impl Default for FaviconFetchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_browser_shell::TabId;

    #[test]
    fn request_skips_internal_page_urls() {
        let mut state = FaviconFetchState::new();
        state.request(TabId(1), "zero://newtab", None, 14.0);
        assert!(state.pending.is_empty());
    }

    #[test]
    fn poll_applies_completed_bitmap() {
        let mut state = FaviconFetchState::new();
        let (tx, rx) = mpsc::channel();
        state.pending.push(PendingFaviconFetch {
            tab_id: TabId(2),
            favicon_url: "https://example.com/favicon.ico".to_string(),
            size_px: 14.0,
            rx,
        });

        let bitmap = tab_favicon::default_favicon_bitmap(14.0);
        tx.send(Some(bitmap)).unwrap();

        let mut loader = FontLoader::new();
        assert!(state.poll(&mut loader));
        assert!(tab_favicon::has_tab_favicon(&loader, TabId(2), 14.0));
    }

    #[test]
    fn cancel_tab_drops_pending_requests() {
        let mut state = FaviconFetchState::new();
        let (_tx, rx) = mpsc::channel();
        state.pending.push(PendingFaviconFetch {
            tab_id: TabId(3),
            favicon_url: "https://example.com/favicon.ico".to_string(),
            size_px: 14.0,
            rx,
        });
        state.cancel_tab(TabId(3));
        assert!(state.pending.is_empty());
    }

    #[test]
    fn request_bookmark_skips_internal_urls() {
        let mut state = FaviconFetchState::new();
        state.request_bookmark("zero://newtab", 14.0);
        assert!(state.pending_bookmarks.is_empty());
    }

    #[test]
    fn request_bookmark_dedupes_same_url() {
        let mut state = FaviconFetchState::new();
        state.request_bookmark("https://example.com", 14.0);
        state.request_bookmark("https://example.com", 14.0);
        assert_eq!(state.pending_bookmarks.len(), 1);
    }

    #[test]
    fn poll_bookmarks_applies_and_marks_completed() {
        let mut state = FaviconFetchState::new();
        let (tx, rx) = mpsc::channel();
        state.pending_bookmarks.push(PendingBookmarkFaviconFetch {
            bookmark_url: "https://example.com".to_string(),
            favicon_url: "https://example.com/favicon.ico".to_string(),
            size_px: 14.0,
            rx,
        });
        tx.send(Some(tab_favicon::default_favicon_bitmap(14.0))).unwrap();

        let mut loader = FontLoader::new();
        assert!(state.poll_bookmarks(&mut loader));
        assert!(tab_favicon::has_bookmark_favicon(&loader, "https://example.com", 14.0));
        // 完成后不应再被重新请求
        state.request_bookmark("https://example.com", 14.0);
        assert!(state.pending_bookmarks.is_empty());
    }
}
