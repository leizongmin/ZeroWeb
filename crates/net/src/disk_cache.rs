//! 磁盘 HTTP 缓存 — 跨会话持久化，对齐浏览器 Disk Cache 层。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cache_policy::{CacheStoreMode, storable_mode};
use crate::request::HttpResponse;
use crate::resource_policy::origin_from_url;

/// 默认磁盘缓存上限（200 MiB）。
const DEFAULT_DISK_MAX_BYTES: u64 = 200 * 1024 * 1024;

/// 每个 origin 的磁盘配额（50 MiB）。
const DEFAULT_ORIGIN_MAX_BYTES: u64 = 50 * 1024 * 1024;

/// 磁盘缓存元数据（JSON 侧车文件）。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskEntryMeta {
    url: String,
    resource_url: Option<String>,
    origin: Option<String>,
    vary: Option<String>,
    revalidate_only: bool,
    status_code: u16,
    headers: Vec<(String, String)>,
    etag: Option<String>,
    last_modified: Option<String>,
    /// 绝对过期时间（Unix 秒）；`revalidate_only` 条目设为存储时刻。
    expires_at: u64,
    /// RFC 9111 §4.2.3 响应接收时的初始年龄（秒）。`#[serde(default)]` 兼容旧 meta 文件（缺失→0）。
    /// `expires_at` 已据此提前（`expires_at = put_now + ttl - initial_age`）；此字段供 304 refresh 重算。
    #[serde(default)]
    initial_age_secs: u64,
    /// 最近访问时间（Unix 秒），用于 LRU 淘汰。
    last_access: u64,
    body_len: u64,
}

/// 磁盘索引条目（用于 Vary 查找）。
#[derive(Debug, Clone)]
pub struct DiskIndexEntry {
    /// 缓存键。
    pub key: String,
    /// 原始资源 URL。
    pub resource_url: Option<String>,
    /// 响应 `Vary` 头。
    pub vary: Option<String>,
}

/// 磁盘 HTTP 缓存根目录下的持久化存储。
#[derive(Debug)]
pub struct DiskHttpCache {
    root: PathBuf,
    max_bytes: u64,
    origin_max_bytes: u64,
}

impl DiskHttpCache {
    /// 打开或创建指定目录下的磁盘缓存。
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            max_bytes: DEFAULT_DISK_MAX_BYTES,
            origin_max_bytes: DEFAULT_ORIGIN_MAX_BYTES,
        })
    }

    /// 使用默认用户缓存目录（`ZERO_CACHE_DIR` 或平台 cache dir）。
    pub fn open_default() -> io::Result<Self> {
        Self::open(default_cache_dir())
    }

    /// 枚举磁盘条目元数据（用于重建内存索引）。
    pub fn list_index_entries(&self) -> Vec<DiskIndexEntry> {
        walk_disk_entries(&self.root)
            .into_iter()
            .filter_map(|paths| {
                let text = fs::read_to_string(&paths.meta).ok()?;
                let meta: DiskEntryMeta = serde_json::from_str(&text).ok()?;
                Some(DiskIndexEntry {
                    key: meta.url,
                    resource_url: meta.resource_url,
                    vary: meta.vary,
                })
            })
            .collect()
    }

    /// 读取条目的 `Vary` 头（不更新 LRU）。
    pub fn entry_vary(&self, key: &str) -> Option<String> {
        let paths = self.entry_paths(key);
        let text = fs::read_to_string(&paths.meta).ok()?;
        let meta: DiskEntryMeta = serde_json::from_str(&text).ok()?;
        if meta.url != key {
            return None;
        }
        meta.vary
    }

    /// 读取条目（含 stale；不删除过期项）。
    pub fn read(&mut self, key: &str) -> Option<DiskCacheHit> {
        let paths = self.entry_paths(key);
        let meta_text = fs::read_to_string(&paths.meta).ok()?;
        let mut meta: DiskEntryMeta = serde_json::from_str(&meta_text).ok()?;
        if meta.url != key {
            return None;
        }
        let body = fs::read(&paths.body).ok()?;
        if body.len() as u64 != meta.body_len {
            let _ = self.remove_files(&paths);
            return None;
        }
        let now = unix_now();
        meta.last_access = now;
        let _ = fs::write(&paths.meta, serde_json::to_string(&meta).unwrap_or_default());
        Some(DiskCacheHit {
            body,
            headers: meta.headers,
            status_code: meta.status_code,
            url: meta.url,
            resource_url: meta.resource_url,
            vary: meta.vary,
            revalidate_only: meta.revalidate_only,
            etag: meta.etag,
            last_modified: meta.last_modified,
            fresh_for_secs: if meta.revalidate_only {
                0
            } else {
                meta.expires_at.saturating_sub(now)
            },
        })
    }

    /// 尝试读取新鲜缓存条目。
    pub fn get(&mut self, key: &str) -> Option<DiskCacheHit> {
        let hit = self.read(key)?;
        if hit.fresh_for_secs == 0 {
            return None;
        }
        Some(hit)
    }

    /// 304 Not Modified — 延长新鲜期并更新验证器。
    pub fn refresh_not_modified(&mut self, key: &str, response: &HttpResponse) -> bool {
        let paths = self.entry_paths(key);
        let Ok(meta_text) = fs::read_to_string(&paths.meta) else {
            return false;
        };
        let Ok(mut meta) = serde_json::from_str::<DiskEntryMeta>(&meta_text) else {
            return false;
        };
        if meta.url != key {
            return false;
        }
        let now = unix_now();
        // R3233：304 可携新 Age/Date（CDN 重报）→ 重算 initial_age，并在 expires_at 中抵扣。
        let initial_age = crate::cache_policy::compute_initial_age(response);
        meta.initial_age_secs = initial_age;
        match storable_mode(response) {
            Some(CacheStoreMode::Fresh(ttl)) => {
                meta.revalidate_only = false;
                meta.expires_at = now.saturating_add(ttl.saturating_sub(initial_age));
            }
            Some(CacheStoreMode::RevalidateOnly) => {
                meta.revalidate_only = true;
                meta.expires_at = now;
            }
            None => {
                meta.expires_at = now.saturating_add(60);
            }
        }
        if let Some(etag) = response.header("etag") {
            meta.etag = Some(etag.to_string());
        }
        if let Some(lm) = response.header("last-modified") {
            meta.last_modified = Some(lm.to_string());
        }
        // R3232：RFC 9111 §4.3.4——304 的元数据字段须并入持久化的 meta.headers（同名替换，缺则追加；
        // 与内存层 not_modified 一致）。旧实现仅更 etag/last_modified 便捷字段，meta.headers 保留旧
        // Cache-Control/Expires/Date/Vary——磁盘读回的响应头为旧值。
        for field in [
            "cache-control",
            "content-location",
            "date",
            "expires",
            "vary",
            "etag",
            "last-modified",
        ] {
            if let Some(val) = response.header(field) {
                merge_header(&mut meta.headers, field, val);
            }
        }
        meta.last_access = now;
        fs::write(&paths.meta, serde_json::to_string(&meta).unwrap_or_default()).is_ok()
    }

    /// 条件请求头（含 stale 条目，供再验证）。
    pub fn conditional_headers(&self, key: &str) -> Vec<(String, String)> {
        let paths = self.entry_paths(key);
        let Ok(meta_text) = fs::read_to_string(&paths.meta) else {
            return Vec::new();
        };
        let Ok(meta) = serde_json::from_str::<DiskEntryMeta>(&meta_text) else {
            return Vec::new();
        };
        if meta.url != key {
            return Vec::new();
        }
        let mut headers = Vec::new();
        if let Some(etag) = meta.etag {
            headers.push(("If-None-Match".to_string(), etag));
        }
        if let Some(lm) = meta.last_modified {
            headers.push(("If-Modified-Since".to_string(), lm));
        }
        headers
    }

    /// 存储可缓存响应；`key` 为 [`cache_store_key`] 结果。
    pub fn put(&mut self, key: &str, response: &HttpResponse) -> bool {
        let mode = match storable_mode(response) {
            Some(m) => m,
            None => return false,
        };
        let now = unix_now();
        // R3233：RFC 9111 §4.2.3——初始年龄（Age/Date 头）提前抵扣新鲜期：
        // `expires_at = put_now + (ttl - initial_age)`，使 read() 的 fresh_for_secs 自然反映剩余新鲜期。
        let initial_age = crate::cache_policy::compute_initial_age(response);
        let (expires_at, revalidate_only) = match mode {
            CacheStoreMode::Fresh(ttl) => (now.saturating_add(ttl.saturating_sub(initial_age)), false),
            CacheStoreMode::RevalidateOnly => (now, true),
        };
        let resource_url = response.url.clone();
        let origin = Some(origin_from_url(&resource_url));
        let meta = DiskEntryMeta {
            url: key.to_string(),
            resource_url: Some(resource_url),
            origin: origin.clone(),
            vary: response.header("vary").map(str::to_string),
            revalidate_only,
            status_code: response.status_code,
            headers: response.headers.clone(),
            etag: response.header("etag").map(str::to_string),
            last_modified: response.header("last-modified").map(str::to_string),
            expires_at,
            initial_age_secs: initial_age,
            last_access: now,
            body_len: response.body.len() as u64,
        };
        let paths = self.entry_paths(key);
        if let Some(parent) = paths.meta.parent() {
            let _ = fs::create_dir_all(parent);
        }
        self.evict_if_needed(meta.body_len, origin.as_deref().unwrap_or(""));
        if fs::write(&paths.body, &response.body).is_err() {
            return false;
        }
        if fs::write(&paths.meta, serde_json::to_string(&meta).unwrap_or_default()).is_err() {
            let _ = fs::remove_file(&paths.body);
            return false;
        }
        true
    }

    /// 移除单条缓存。
    pub fn remove(&mut self, url: &str) -> bool {
        self.remove_files(&self.entry_paths(url))
    }

    /// 清空磁盘缓存目录内容。
    pub fn clear(&mut self) -> io::Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root)?;
        }
        fs::create_dir_all(&self.root)
    }

    /// 磁盘缓存占用字节数（body 文件总和）。
    pub fn total_bytes(&self) -> u64 {
        walk_disk_entries(&self.root)
            .into_iter()
            .filter_map(|p| fs::metadata(p.body).ok().map(|m| m.len()))
            .sum()
    }

    fn entry_paths(&self, url: &str) -> EntryPaths {
        let key = cache_key(url);
        let shard_a = &key[..2];
        let shard_b = &key[2..4];
        let dir = self.root.join(shard_a).join(shard_b);
        EntryPaths {
            meta: dir.join(format!("{key}.json")),
            body: dir.join(format!("{key}.bin")),
        }
    }

    fn remove_files(&self, paths: &EntryPaths) -> bool {
        let a = fs::remove_file(&paths.meta).is_ok();
        let b = fs::remove_file(&paths.body).is_ok();
        a || b
    }

    fn evict_if_needed(&mut self, incoming: u64, origin: &str) {
        self.prune_expired();
        while !origin.is_empty() && self.origin_bytes(origin).saturating_add(incoming) > self.origin_max_bytes {
            if !self.evict_oldest_for_origin(origin) {
                break;
            }
        }
        while self.total_bytes().saturating_add(incoming) > self.max_bytes {
            if self.evict_oldest_global().is_none() {
                break;
            }
        }
    }

    fn origin_bytes(&self, origin: &str) -> u64 {
        walk_disk_entries(&self.root)
            .into_iter()
            .filter_map(|paths| {
                let text = fs::read_to_string(&paths.meta).ok()?;
                let meta: DiskEntryMeta = serde_json::from_str(&text).ok()?;
                if meta.origin.as_deref() == Some(origin) {
                    fs::metadata(&paths.body).ok().map(|m| m.len())
                } else {
                    None
                }
            })
            .sum()
    }

    fn evict_oldest_for_origin(&mut self, origin: &str) -> bool {
        let oldest = walk_disk_entries(&self.root)
            .into_iter()
            .filter_map(|paths| {
                let text = fs::read_to_string(&paths.meta).ok()?;
                let meta: DiskEntryMeta = serde_json::from_str(&text).ok()?;
                if meta.origin.as_deref() == Some(origin) {
                    Some((paths, meta.last_access))
                } else {
                    None
                }
            })
            .min_by_key(|(_, access)| *access)
            .map(|(p, _)| p);
        if let Some(paths) = oldest {
            let _ = fs::remove_file(&paths.meta);
            let _ = fs::remove_file(&paths.body);
            return true;
        }
        false
    }

    fn evict_oldest_global(&mut self) -> Option<EntryPaths> {
        let oldest = find_oldest_entry(&self.root)?;
        let _ = fs::remove_file(&oldest.meta);
        let _ = fs::remove_file(&oldest.body);
        Some(oldest)
    }

    fn prune_expired(&mut self) {
        let now = unix_now();
        for paths in walk_disk_entries(&self.root) {
            if let Ok(text) = fs::read_to_string(&paths.meta)
                && let Ok(meta) = serde_json::from_str::<DiskEntryMeta>(&text)
                && !meta.revalidate_only
                && now >= meta.expires_at
            {
                let _ = fs::remove_file(&paths.meta);
                let _ = fs::remove_file(&paths.body);
            }
        }
    }
}

/// 磁盘缓存命中结果。
#[derive(Debug, Clone)]
pub struct DiskCacheHit {
    /// 响应体。
    pub body: Vec<u8>,
    /// 响应头。
    pub headers: Vec<(String, String)>,
    /// HTTP 状态码。
    pub status_code: u16,
    /// 缓存键（Hash）。
    pub url: String,
    /// 原始资源 URL。
    pub resource_url: Option<String>,
    /// 响应 `Vary` 头。
    pub vary: Option<String>,
    /// 是否每次使用前必须再验证。
    pub revalidate_only: bool,
    /// ETag。
    pub etag: Option<String>,
    /// Last-Modified。
    pub last_modified: Option<String>,
    /// 剩余新鲜期（秒）。
    pub fresh_for_secs: u64,
}

impl DiskCacheHit {
    /// 转为 [`HttpResponse`].
    pub fn into_response(self) -> HttpResponse {
        HttpResponse {
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
            url: self.url,
            redirect_count: 0,
        }
    }
}

/// 默认 HTTP 磁盘缓存目录。
pub fn default_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ZERO_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    dirs::cache_dir()
        .map(|d| d.join("ZeroBrowser").join("HTTP Cache"))
        .unwrap_or_else(|| PathBuf::from(".zero-browser-cache"))
}

struct EntryPaths {
    meta: PathBuf,
    body: PathBuf,
}

fn walk_disk_entries(root: &Path) -> Vec<EntryPaths> {
    fn walk(dir: &Path, out: &mut Vec<EntryPaths>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                out.push(EntryPaths {
                    meta: path.clone(),
                    body: path.with_file_name(format!("{stem}.bin")),
                });
            }
        }
    }
    let mut out = Vec::new();
    if root.is_dir() {
        walk(root, &mut out);
    }
    out
}

fn find_oldest_entry(root: &Path) -> Option<EntryPaths> {
    walk_disk_entries(root)
        .into_iter()
        .filter_map(|paths| {
            let text = fs::read_to_string(&paths.meta).ok()?;
            let meta: DiskEntryMeta = serde_json::from_str(&text).ok()?;
            Some((paths, meta.last_access))
        })
        .min_by_key(|(_, access)| *access)
        .map(|(p, _)| p)
}

/// FNV-1a 64 — 跨平台稳定，用于文件名。
fn cache_key(url: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in url.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 用 304 响应的元数据字段更新 header 列表（RFC 9111 §4.3.4——同名替换，缺则追加；name 大小写不敏感）。
/// 与 `http_cache::merge_header` 同语义（磁盘层独立副本，避免跨模块 pub）。
fn merge_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    for (n, v) in headers.iter_mut() {
        if n.eq_ignore_ascii_case(name) {
            *v = value.to_string();
            return;
        }
    }
    headers.push((name.to_string(), value.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cache() -> (DiskHttpCache, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "zero-disk-cache-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        (DiskHttpCache::open(&dir).unwrap(), dir)
    }

    fn sample_response(body: &[u8]) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=3600".into()),
                ("ETag".into(), "\"v1\"".into()),
            ],
            body: body.to_vec(),
            url: "https://example.com/app.js".to_string(),
            redirect_count: 0,
        }
    }

    #[test]
    fn disk_put_get_roundtrip() {
        let (mut cache, dir) = temp_cache();
        let url = "https://example.com/app.js";
        let resp = sample_response(b"console.log(1);");
        assert!(cache.put(url, &resp));
        let hit = cache.get(url).expect("disk hit");
        assert_eq!(hit.body, b"console.log(1);");
        assert_eq!(hit.status_code, 200);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disk_no_store_not_persisted() {
        let (mut cache, dir) = temp_cache();
        let url = "https://example.com/private";
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Cache-Control".into(), "no-store".into())],
            body: b"x".to_vec(),
            url: url.to_string(),
            redirect_count: 0,
        };
        assert!(!cache.put(url, &resp));
        assert!(cache.get(url).is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disk_no_cache_stored_for_revalidation() {
        let (mut cache, dir) = temp_cache();
        let key = "https://example.com/api";
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "no-cache".into()),
                ("ETag".into(), "\"v1\"".into()),
            ],
            body: b"data".to_vec(),
            url: key.to_string(),
            redirect_count: 0,
        };
        assert!(cache.put(key, &resp));
        assert!(cache.get(key).is_none());
        let hit = cache.read(key).expect("stale readable");
        assert!(hit.revalidate_only);
        assert_eq!(hit.fresh_for_secs, 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disk_expired_entry_not_served_as_fresh() {
        let (mut cache, dir) = temp_cache();
        let key = "https://example.com/old.css";
        let paths = cache.entry_paths(key);
        if let Some(parent) = paths.meta.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let expired = DiskEntryMeta {
            url: key.to_string(),
            resource_url: Some("https://example.com/old.css".to_string()),
            origin: Some("https://example.com".into()),
            vary: None,
            revalidate_only: false,
            status_code: 200,
            headers: vec![],
            etag: Some("\"x\"".to_string()),
            last_modified: None,
            expires_at: 1,
            initial_age_secs: 0,
            last_access: 1,
            body_len: 3,
        };
        fs::write(&paths.meta, serde_json::to_string(&expired).unwrap()).unwrap();
        fs::write(&paths.body, b"old").unwrap();
        assert!(cache.get(key).is_none());
        let hit = cache.read(key).expect("stale readable");
        assert_eq!(hit.fresh_for_secs, 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disk_survives_reopen() {
        let dir = std::env::temp_dir().join(format!("zero-disk-cache-reopen-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let url = "https://example.com/logo.png";
        {
            let mut cache = DiskHttpCache::open(&dir).unwrap();
            assert!(cache.put(url, &sample_response(b"PNG")));
        }
        {
            let mut cache = DiskHttpCache::open(&dir).unwrap();
            let hit = cache.get(url).expect("persisted");
            assert_eq!(hit.body, b"PNG");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn conditional_headers_from_disk_meta() {
        let (mut cache, dir) = temp_cache();
        let url = "https://example.com/x";
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=60".into()),
                ("ETag".into(), "\"abc\"".into()),
                ("Last-Modified".into(), "Wed, 21 Oct 2015 07:28:00 GMT".into()),
            ],
            body: b"data".to_vec(),
            url: url.to_string(),
            redirect_count: 0,
        };
        cache.put(url, &resp);
        let headers = cache.conditional_headers(url);
        assert!(headers.iter().any(|(k, v)| k == "If-None-Match" && v == "\"abc\""));
        let _ = fs::remove_dir_all(dir);
    }

    /// R3232：磁盘层 304 Not Modified 须并入 304 的元数据 header（RFC 9111 §4.3.4）——
    /// 旧 refresh_not_modified 仅更 etag/last_modified 便捷字段，meta.headers 保留旧
    /// Cache-Control/Expires/Date/Vary，磁盘读回（DiskCacheHit.headers）+ 跨会话持久化的头为旧值。
    /// 镜像 R3231 内存层 test_cache_304_merges_metadata_headers_r3231。
    #[test]
    fn test_disk_refresh_not_modified_merges_headers_r3232() {
        let (mut cache, dir) = temp_cache();
        let key = "https://example.com/r3232";
        // 存 200：Cache-Control: max-age=60 + ETag "v1" + 旧 Expires。
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=60".into()),
                ("ETag".into(), "\"v1\"".into()),
                ("Expires".into(), "Wed, 21 Oct 2015 07:28:00 GMT".into()),
            ],
            body: b"hello".to_vec(),
            url: key.to_string(),
            redirect_count: 0,
        };
        assert!(cache.put(key, &resp));

        // 304 携新 Cache-Control: max-age=300 + ETag "v2" + 新 Vary（追加）。
        let not_mod = HttpResponse {
            status_code: 304,
            headers: vec![
                ("Cache-Control".into(), "max-age=300".into()),
                ("ETag".into(), "\"v2\"".into()),
                ("Vary".into(), "Accept-Encoding".into()),
            ],
            body: Vec::new(),
            url: key.to_string(),
            redirect_count: 0,
        };
        assert!(cache.refresh_not_modified(key, &not_mod), "304 须刷新磁盘缓存条目");

        // 读回：headers 须反映 304 的元数据（旧 max-age=60 / "v1" 被替换；Vary 追加）。
        let hit = cache.read(key).expect("disk 可读");
        assert_eq!(
            header_val(&hit, "cache-control"),
            Some("max-age=300"),
            "304 的 Cache-Control 须并入 meta.headers"
        );
        assert_eq!(
            header_val(&hit, "etag"),
            Some("\"v2\""),
            "304 的 ETag 须并入 meta.headers"
        );
        assert_eq!(hit.etag, Some("\"v2\"".to_string()), "etag 便捷字段亦更新");
        assert_eq!(
            header_val(&hit, "vary"),
            Some("Accept-Encoding"),
            "304 新增的 Vary 须追加到 meta.headers"
        );
        // body + status 仍为缓存的 200（304 仅 revalidate，不替换 body）。
        assert_eq!(hit.body, b"hello");
        assert_eq!(hit.status_code, 200);

        // 跨会话持久化：重新打开缓存，并入的头仍生效（磁盘层核心价值）。
        let root = cache.root.clone();
        drop(cache);
        let mut reopened = DiskHttpCache::open(&root).unwrap();
        let hit2 = reopened.read(key).expect("reopen 后仍可读");
        assert_eq!(
            header_val(&hit2, "cache-control"),
            Some("max-age=300"),
            "并入的头须持久化到 meta 文件"
        );
        assert_eq!(header_val(&hit2, "etag"), Some("\"v2\""));
        let _ = fs::remove_dir_all(dir);
    }

    /// R3233：磁盘层 Age 头（RFC 9111 §4.2.3）——`Age > max-age` 时 put 抵扣后 expires_at ≤ now → 不新鲜。
    /// 旧实现忽略 Age，把 CDN 已存活 N 秒的响应当全新鲜写入磁盘（expires_at = now + ttl）。
    #[test]
    fn test_disk_age_header_reduces_freshness_r3233() {
        let (mut cache, dir) = temp_cache();
        let key = "https://example.com/disk-aged";
        // Age(150) > max-age(100) → expires_at = now + (100 - 150) saturating → now → 不新鲜。
        let aged = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=100".into()),
                ("Age".into(), "150".into()),
                ("ETag".into(), "\"v1\"".into()),
            ],
            body: b"cdn".to_vec(),
            url: key.to_string(),
            redirect_count: 0,
        };
        assert!(cache.put(key, &aged));
        // get() 仅返新鲜条目；Age 抵扣后已过期 → None。
        assert!(cache.get(key).is_none(), "Age>max-age 须判过期，get() 返 None");
        // read() 仍返 stale 条目（含 ETag 供条件再验证）。
        let hit = cache.read(key).expect("stale 可读");
        assert_eq!(hit.fresh_for_secs, 0);
        assert_eq!(hit.body, b"cdn");

        // 对照：无 Age，max-age=100 → get() 返 Some（新鲜）。
        let fresh_key = "https://example.com/disk-fresh";
        let fresh = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Cache-Control".into(), "max-age=100".into()),
                ("ETag".into(), "\"v1\"".into()),
            ],
            body: b"fresh".to_vec(),
            url: fresh_key.to_string(),
            redirect_count: 0,
        };
        assert!(cache.put(fresh_key, &fresh));
        assert!(cache.get(fresh_key).is_some(), "无 Age 的 max-age=100 须新鲜");

        // 跨会话 reopen：Age 抵扣仍持久（initial_age 烘入 expires_at 的 meta 文件）。
        let root = cache.root.clone();
        drop(cache);
        let mut reopened = DiskHttpCache::open(&root).unwrap();
        assert!(reopened.get(key).is_none(), "reopen 后 Age>max-age 仍判过期");
        assert!(reopened.get(fresh_key).is_some(), "reopen 后无 Age 条目仍新鲜");
        let _ = fs::remove_dir_all(dir);
    }

    /// 大小写不敏感地从 `DiskCacheHit.headers` 取首个匹配值（R3232 测试用）。
    fn header_val<'a>(hit: &'a DiskCacheHit, name: &str) -> Option<&'a str> {
        hit.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn per_origin_quota_evicts_oldest_in_origin() {
        let (cache, dir) = temp_cache();
        let mut cache = DiskHttpCache {
            root: cache.root,
            max_bytes: DEFAULT_DISK_MAX_BYTES,
            origin_max_bytes: 100,
        };
        for i in 0..3 {
            let url = format!("https://example.com/file{i}.bin");
            let resp = HttpResponse {
                status_code: 200,
                headers: vec![("Cache-Control".into(), "max-age=3600".into())],
                body: vec![0u8; 40],
                url,
                redirect_count: 0,
            };
            assert!(cache.put(&resp.url, &resp));
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(cache.total_bytes() <= 100);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cache_key_is_stable() {
        assert_eq!(cache_key("https://example.com/a"), cache_key("https://example.com/a"));
        assert_ne!(cache_key("https://example.com/a"), cache_key("https://example.com/b"));
    }

    #[test]
    fn default_cache_dir_is_non_empty() {
        let dir = default_cache_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn unix_now_is_reasonable() {
        let now = unix_now();
        let floor = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(now <= floor + 1);
    }
}
