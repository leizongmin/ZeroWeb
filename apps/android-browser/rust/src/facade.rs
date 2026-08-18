//! Android chrome 对 Rust browser shell 的单一状态入口。

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};
use zero_browser_shell::{BrowserShell, ProfilePaths, TabId};

const MAX_URL_BYTES: usize = 16 * 1024;
const MAX_LIST_ITEMS: usize = 200;

struct AndroidBrowser {
    shell: BrowserShell,
    paths: ProfilePaths,
    revision: u64,
}

static BROWSER: OnceLock<Mutex<Option<AndroidBrowser>>> = OnceLock::new();

fn browser() -> &'static Mutex<Option<AndroidBrowser>> {
    BROWSER.get_or_init(|| Mutex::new(None))
}

pub(crate) fn load_profile(root: &str) -> Result<String, String> {
    if root.is_empty() {
        return Err("Android profile path is empty".to_string());
    }
    let paths = ProfilePaths::new(PathBuf::from(root));
    let shell = BrowserShell::load_profile(&paths);
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    *state = Some(AndroidBrowser {
        shell,
        paths,
        revision: 1,
    });
    snapshot_locked(&state)
}

pub(crate) fn snapshot() -> Result<String, String> {
    let state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    snapshot_locked(&state)
}

pub(crate) fn navigate(url: &str) -> Result<(), String> {
    let url = validated_url(url)?;
    mutate(|browser| browser.shell.navigate(url))
}

pub(crate) fn new_tab() -> Result<(), String> {
    mutate(|browser| {
        browser.shell.new_tab(None);
    })
}

pub(crate) fn new_tab_with_url(url: &str) -> Result<(), String> {
    let url = validated_url(url)?;
    mutate(|browser| {
        browser.shell.new_tab(Some(url));
    })
}

pub(crate) fn close_tab(id: u64) -> Result<(), String> {
    mutate(|browser| {
        browser.shell.close_tab(TabId(id));
        if browser.shell.is_empty() {
            browser.shell.new_tab(None);
        }
    })
}

pub(crate) fn select_tab(id: u64) -> Result<(), String> {
    mutate(|browser| browser.shell.switch_tab(TabId(id)))
}

pub(crate) fn go_back() -> Result<bool, String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let did_navigate = browser.shell.go_back();
    if did_navigate {
        browser.shell.save_profile(&browser.paths)?;
        browser.revision = browser.revision.saturating_add(1);
    }
    Ok(did_navigate)
}

pub(crate) fn go_forward() -> Result<bool, String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let did_navigate = browser.shell.go_forward();
    if did_navigate {
        browser.shell.save_profile(&browser.paths)?;
        browser.revision = browser.revision.saturating_add(1);
    }
    Ok(did_navigate)
}

pub(crate) fn toggle_bookmark() -> Result<(), String> {
    mutate(|browser| {
        browser.shell.toggle_current_bookmark();
    })
}

pub(crate) fn remove_bookmark(url: &str) -> Result<(), String> {
    let url = validated_url(url)?;
    mutate(|browser| {
        browser.shell.remove_bookmark_by_url(url);
    })
}

pub(crate) fn clear_history() -> Result<(), String> {
    mutate(|browser| browser.shell.history_mut().clear())
}

fn mutate(update: impl FnOnce(&mut AndroidBrowser)) -> Result<(), String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    update(browser);
    browser.shell.save_profile(&browser.paths)?;
    browser.revision = browser.revision.saturating_add(1);
    Ok(())
}

fn snapshot_locked(state: &Option<AndroidBrowser>) -> Result<String, String> {
    let browser = state
        .as_ref()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let active_tab_id = browser.shell.active_tab_id().map(|id| id.0);
    let tabs: Vec<Value> = browser
        .shell
        .tabs()
        .map(|tab| {
            json!({
                "id": tab.id().0,
                "url": tab.url(),
                "title": tab.title(),
                "loading": tab.is_loading(),
                "crashed": tab.is_crashed(),
            })
        })
        .collect();
    let bookmarks: Vec<Value> = browser
        .shell
        .bookmarks()
        .iter()
        .take(MAX_LIST_ITEMS)
        .map(|bookmark| json!({ "title": bookmark.title(), "url": bookmark.url() }))
        .collect();
    let history: Vec<Value> = browser
        .shell
        .history()
        .iter()
        .take(MAX_LIST_ITEMS)
        .map(|entry| json!({ "title": entry.title(), "url": entry.url() }))
        .collect();
    let downloads: Vec<Value> = browser
        .shell
        .downloads()
        .iter()
        .take(MAX_LIST_ITEMS)
        .map(|download| {
            json!({
                "id": download.id().0,
                "url": download.url(),
                "filename": download.filename(),
                "downloadedBytes": download.downloaded_bytes(),
                "totalBytes": download.total_bytes(),
                "state": format!("{:?}", download.state()),
            })
        })
        .collect();
    serde_json::to_string(&json!({
        "revision": browser.revision,
        "activeTabId": active_tab_id,
        "tabs": tabs,
        "bookmarked": browser.shell.is_current_page_bookmarked(),
        "bookmarkCount": browser.shell.bookmarks().len(),
        "historyCount": browser.shell.history().len(),
        "downloadCount": browser.shell.downloads().len(),
        "bookmarks": bookmarks,
        "history": history,
        "downloads": downloads,
    }))
    .map_err(|error| format!("serialize Android browser snapshot failed: {error}"))
}

fn validated_url(url: &str) -> Result<&str, String> {
    let url = url.trim();
    if url.is_empty() || url.len() > MAX_URL_BYTES {
        return Err("URL must be between 1 and 16384 bytes".to_string());
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only HTTP(S) URLs are accepted".to_string());
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::validated_url;

    #[test]
    fn facade_only_accepts_bounded_http_urls() {
        assert_eq!(validated_url(" https://example.com ").unwrap(), "https://example.com");
        assert!(validated_url("file:///tmp/page.html").is_err());
        assert!(validated_url("").is_err());
    }
}
