//! 失败 URL 负缓存 — 短期内不重试失败的请求（性能门禁优化 S6，2026-08-08）。
//!
//! 背景：renderer 每次 publish 都会重请求「未缓存/解码失败」的图片（paint_export.rs
//! `fetch_image_payloads_with_cache`），失败 URL 永不落缓存 → 每 publish 重试风暴。
//! 负缓存让失败 URL 在 TTL 内直接跳过请求（成功即清除）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// 失败 URL 负缓存。
#[derive(Default)]
pub struct NegativeCache {
    entries: HashMap<String, Instant>,
    /// 失败冷却期（默认 30s）。
    ttl: Duration,
}

impl NegativeCache {
    /// 创建负缓存，指定失败冷却期。
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    /// 该 URL 是否处于失败冷却期内（不应重试）。
    pub fn is_recently_failed(&self, url: &str) -> bool {
        self.entries.get(url).is_some_and(|t| t.elapsed() < self.ttl)
    }

    /// 记录一次失败。
    pub fn mark_failed(&mut self, url: &str) {
        self.entries.insert(url.to_string(), Instant::now());
    }

    /// 记录一次成功（清除失败标记）。
    pub fn mark_ok(&mut self, url: &str) {
        self.entries.remove(url);
    }

    /// 冷却期内条目数（诊断用）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否无冷却期条目。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 全局共享负缓存（进程内所有 fetch 路径共用同一份失败记忆）。
pub fn shared_negative_cache() -> Arc<Mutex<NegativeCache>> {
    static CACHE: OnceLock<Arc<Mutex<NegativeCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| Arc::new(Mutex::new(NegativeCache::new(Duration::from_secs(30)))))
        .clone()
}
