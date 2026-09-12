//! Android chrome 对 Rust browser shell 的单一状态入口。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};
use zero_browser_shell::{BrowserShell, ProfilePaths, TabId};

const MAX_URL_BYTES: usize = 16 * 1024;
const MAX_LIST_ITEMS: usize = 200;

/// RFC §6.3：8 个预声明 renderer isolated Service slots。
pub(crate) const RENDERER_SLOT_COUNT: usize = 8;

struct AndroidBrowser {
    shell: BrowserShell,
    paths: ProfilePaths,
    /// profile 根目录（下载字节落 `<root>/downloads/`；宿主仅测试触达，定向豁免）。
    #[allow(dead_code)]
    root: PathBuf,
    revision: u64,
    /// tab → renderer slot 亲和（RFC §6.3 换槽模型）。槽是稀缺资源：标签首次
    /// 需要渲染时分配，超量按 LRU 逐出（挂起语义：切回该标签时重新导航）。
    tab_slots: HashMap<TabId, usize>,
    /// 每个槽最近服务的 tab（LRU 逐出依据）；None = 空闲槽。
    slot_tenants: [Option<TabId>; RENDERER_SLOT_COUNT],
    // LRU 时钟仅 android 运行时消费（宿主只有测试触达），定向豁免 dead_code。
    #[allow(dead_code)]
    slot_use_clock: u64,
    #[allow(dead_code)]
    slot_last_use: [u64; RENDERER_SLOT_COUNT],
}

static BROWSER: OnceLock<Mutex<Option<AndroidBrowser>>> = OnceLock::new();

fn browser() -> &'static Mutex<Option<AndroidBrowser>> {
    BROWSER.get_or_init(|| Mutex::new(None))
}

pub(crate) fn load_profile(root: &str) -> Result<String, String> {
    if root.is_empty() {
        return Err("Android profile path is empty".to_string());
    }
    let root = PathBuf::from(root);
    let paths = ProfilePaths::new(root.clone());
    let shell = BrowserShell::load_profile(&paths);
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    *state = Some(AndroidBrowser {
        shell,
        paths,
        root,
        revision: 1,
        tab_slots: HashMap::new(),
        slot_tenants: [None; RENDERER_SLOT_COUNT],
        slot_use_clock: 0,
        slot_last_use: [0; RENDERER_SLOT_COUNT],
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
    })?;
    release_tab_slot(id)?;
    Ok(())
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

// 以下槽管理入口仅被 android cfg 的 JNI 路径调用（宿主仅测试触达）。
#[allow(dead_code)]
/// 活动标签的渲染槽（只读；snapshot 驱动 Kotlin 绑槽用）。
pub(crate) fn active_tab_slot() -> Result<Option<usize>, String> {
    let state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_ref()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    Ok(browser
        .shell
        .active_tab_id()
        .and_then(|id| browser.tab_slots.get(&id).copied()))
}

#[allow(dead_code)]
/// 触碰活动标签的 LRU 时钟（切标签时保持其槽不被逐出）；已分配则返回槽号。
pub(crate) fn touch_active_tab_slot() -> Result<Option<usize>, String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let Some(tab) = browser.shell.active_tab_id() else {
        return Ok(None);
    };
    let Some(&slot) = browser.tab_slots.get(&tab) else {
        return Ok(None);
    };
    browser.slot_use_clock = browser.slot_use_clock.wrapping_add(1);
    browser.slot_last_use[slot] = browser.slot_use_clock;
    Ok(Some(slot))
}

#[allow(dead_code)]
/// 为活动标签分配渲染槽：优先空闲槽；8 槽全满时按 LRU 逐出最旧租户
/// （RFC §6.3「超额标签 LRU 挂起」——被逐标签切回时需重新导航）。
/// 返回 (槽号, 被逐出的旧 tab id)；JNI 层负责清除被逐槽的 transport。
pub(crate) fn assign_active_tab_slot() -> Result<(usize, Option<u64>), String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let Some(tab) = browser.shell.active_tab_id() else {
        return Err("Android browser has no active tab".to_string());
    };
    browser.slot_use_clock = browser.slot_use_clock.wrapping_add(1);
    let clock = browser.slot_use_clock;
    let mut evicted = None;
    let slot = match browser.tab_slots.get(&tab).copied() {
        Some(slot) => slot,
        None => {
            let idle = (0..RENDERER_SLOT_COUNT).find(|&i| browser.slot_tenants[i].is_none());
            let slot = match idle {
                Some(slot) => slot,
                None => (0..RENDERER_SLOT_COUNT)
                    .min_by_key(|&i| browser.slot_last_use[i])
                    .expect("RENDERER_SLOT_COUNT > 0"),
            };
            if let Some(old_tenant) = browser.slot_tenants[slot].replace(tab) {
                browser.tab_slots.remove(&old_tenant);
                evicted = Some(old_tenant.0);
            }
            browser.tab_slots.insert(tab, slot);
            slot
        }
    };
    browser.slot_tenants[slot] = Some(tab);
    browser.slot_last_use[slot] = clock;
    Ok((slot, evicted))
}

/// 标签关闭时回收其渲染槽。返回释放的槽号（无槽则 None）。
pub(crate) fn release_tab_slot(tab_id: u64) -> Result<Option<usize>, String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    if let Some(slot) = browser.tab_slots.remove(&TabId(tab_id)) {
        if browser.slot_tenants[slot] == Some(TabId(tab_id)) {
            browser.slot_tenants[slot] = None;
        }
        return Ok(Some(slot));
    }
    Ok(None)
}

/// 只读查询某标签当前占用的槽（关闭标签前供 JNI 层定位待清理的 transport）。
pub(crate) fn tab_slot(tab_id: u64) -> Result<Option<usize>, String> {
    let state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_ref()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    Ok(browser.tab_slots.get(&TabId(tab_id)).copied())
}

#[cfg(target_os = "android")]
pub(crate) fn page_loaded(title: &str) -> Result<(), String> {
    mutate(|browser| browser.shell.on_page_loaded(title))
}

#[allow(dead_code)]
/// 记录并落盘一次下载（FR-006 browser 进程接管）：字节写入 `<profile>/downloads/`
/// （文件名前缀 DownloadId 防碰撞，展示名保持干净），DownloadManager 记完成态并随
/// profile 持久化；写盘失败标 Failed，不伪造完成记录。文件路径不回传 renderer。
pub(crate) fn record_download(url: &str, filename: &str, body: &[u8]) -> Result<(), String> {
    let mut state = browser()
        .lock()
        .map_err(|_| "Android browser state lock poisoned".to_string())?;
    let browser = state
        .as_mut()
        .ok_or_else(|| "Android browser profile is not initialized".to_string())?;
    let id = browser.shell.downloads_mut().start_download(url, filename);
    let dir = browser.root.join("downloads");
    let outcome = std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(dir.join(format!("{}-{filename}", id.0)), body))
        .map_err(|error| format!("write download failed: {error}"));
    match outcome {
        Ok(()) => {
            let total = body.len() as u64;
            browser.shell.downloads_mut().update_progress(id, total, Some(total));
            browser.shell.downloads_mut().mark_completed(id);
        }
        Err(error) => {
            browser.shell.downloads_mut().mark_failed(id);
            browser.shell.save_profile(&browser.paths)?;
            return Err(error);
        }
    }
    browser.shell.save_profile(&browser.paths)?;
    browser.revision = browser.revision.saturating_add(1);
    Ok(())
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
    // 活动槽在持锁状态下内联读取（active_tab_slot() 会重入 browser 锁）。
    let active_renderer_slot = browser
        .shell
        .active_tab_id()
        .and_then(|id| browser.tab_slots.get(&id).copied());
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
                "rendererSlot": browser.tab_slots.get(&tab.id()).copied(),
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
        "activeRendererSlot": active_renderer_slot,
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

    /// 槽分配/LRU/回收全链：单函数串行执行——BROWSER 是进程级单例，避免并行互踩。
    #[test]
    fn renderer_slots_assign_evict_lru_and_release() {
        use super::{
            RENDERER_SLOT_COUNT, active_tab_slot, assign_active_tab_slot, load_profile, new_tab_with_url,
            record_download, release_tab_slot, select_tab, snapshot, touch_active_tab_slot,
        };
        use serde_json::Value;
        use std::sync::atomic::{AtomicU64, Ordering};

        static SEQ: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "zeroweb-facade-slot-test-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        load_profile(root.to_str().unwrap()).unwrap();

        let active_tab = || {
            let snap: Value = serde_json::from_str(&snapshot().unwrap()).unwrap();
            snap["activeTabId"].as_u64().unwrap()
        };

        // 无标签渲染需求时查询为空
        assert_eq!(active_tab_slot().unwrap(), None);

        // 建满 8 个带 URL 的标签：依次占用槽 0-7
        let mut tabs = Vec::new();
        for i in 0..RENDERER_SLOT_COUNT {
            new_tab_with_url(&format!("https://example.com/{i}")).unwrap();
            let (slot, evicted) = assign_active_tab_slot().unwrap();
            assert_eq!(slot, i);
            assert_eq!(evicted, None);
            tabs.push(active_tab());
        }

        // 快照按标签暴露渲染槽（多标签缩略图数据源，M4 切片 8）
        let snap: Value = serde_json::from_str(&snapshot().unwrap()).unwrap();
        for (i, tab_id) in tabs.iter().enumerate() {
            let entry = snap["tabs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|tab| tab["id"].as_u64() == Some(*tab_id))
                .unwrap();
            assert_eq!(entry["rendererSlot"].as_u64(), Some(i as u64));
        }

        // 触碰槽 0 的租户（模拟用户切回第一个标签），再建第 9 个标签：
        // LRU 逐出应命中槽 1（最久未用），而非刚触碰的槽 0
        select_tab(tabs[0]).unwrap();
        assert_eq!(touch_active_tab_slot().unwrap(), Some(0));
        new_tab_with_url("https://example.com/9").unwrap();
        let (slot, evicted) = assign_active_tab_slot().unwrap();
        assert_eq!(slot, 1, "LRU 应逐出最久未用的槽 1");
        assert_eq!(evicted, Some(tabs[1]), "被逐租户应是槽 1 原标签");
        assert_eq!(active_tab_slot().unwrap(), Some(1));

        // 关闭当前标签：回收其槽
        let current = active_tab();
        release_tab_slot(current).unwrap();
        assert_eq!(active_tab_slot().unwrap(), None, "新活动标签尚未分配槽");

        // 下载记录：落盘 + 完成态进快照（FR-006 browser 进程接管）
        let body = b"zeroweb download payload";
        record_download("https://example.com/report.pdf", "report.pdf", body).unwrap();
        let snap: Value = serde_json::from_str(&snapshot().unwrap()).unwrap();
        assert_eq!(snap["downloadCount"].as_u64().unwrap(), 1);
        let entry = &snap["downloads"][0];
        assert_eq!(entry["filename"].as_str().unwrap(), "report.pdf");
        assert_eq!(entry["state"].as_str().unwrap(), "Completed");
        assert_eq!(entry["totalBytes"].as_u64().unwrap(), body.len() as u64);
        let stored =
            std::fs::read(root.join(format!("downloads/{}-report.pdf", entry["id"].as_u64().unwrap()))).unwrap();
        assert_eq!(stored, body);
    }
}
