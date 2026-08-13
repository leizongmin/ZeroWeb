//! 失败 URL 负缓存 — 短期内不重试失败的请求（性能门禁优化 S6，2026-08-08）。
//!
//! 背景：renderer 每次 publish 都会重请求「未缓存/解码失败」的图片（paint_export.rs
//! `fetch_image_payloads_with_cache`），失败 URL 永不落缓存 → 每 publish 重试风暴。
//! 负缓存让失败 URL 在 TTL 内直接跳过请求（成功即清除）。
//!
//! 容量治理：失败 URL 由页面可控（任意 `<img>`/`fetch()` 失败均 `mark_failed`），
//! 故 [`NegativeCache`] 设上限并 LRU 淘汰（对齐同 crate `HttpCache::new` 的
//! `max_entries = 1000`），防止攻击者用大量不同失败 URL 无限撑大内存映射（R3375）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// 失败 URL 负缓存。
#[derive(Default)]
pub struct NegativeCache {
    entries: HashMap<String, Instant>,
    /// 失败冷却期（默认 30s）。
    ttl: Duration,
    /// 条目上限；超出后淘汰最旧（最接近自然过期）的失败记录。
    /// `0` 视为无限制（仅用于测试，生产路径恒 > 0）。
    max_entries: usize,
}

impl NegativeCache {
    /// 创建负缓存，指定失败冷却期（容量上限默认 1000）。
    pub fn new(ttl: Duration) -> Self {
        Self::with_max_entries(ttl, DEFAULT_MAX_ENTRIES)
    }

    /// 创建负缓存，显式指定失败冷却期与容量上限。
    pub fn with_max_entries(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_entries,
        }
    }

    /// 该 URL 是否处于失败冷却期内（不应重试）。
    ///
    /// 顺带淘汰该 URL 自身的过期记录（惰性清理，避免过期条目常驻）。
    pub fn is_recently_failed(&mut self, url: &str) -> bool {
        let active = self.entries.get(url).is_some_and(|t| t.elapsed() < self.ttl);
        if !active {
            self.entries.remove(url);
        }
        active
    }

    /// 记录一次失败。
    ///
    /// 超过容量上限时先清理所有已过 TTL 的条目；仍超额则淘汰最旧（`Instant` 最小、
    /// 最接近自然过期）的失败记录。失败 URL 由页面可控，无上限会导致内存映射单调增长。
    pub fn mark_failed(&mut self, url: &str) {
        self.entries.insert(url.to_string(), Instant::now());
        if self.max_entries > 0 && self.entries.len() > self.max_entries {
            self.evict_expired();
        }
        if self.max_entries > 0 && self.entries.len() > self.max_entries {
            self.evict_oldest();
        }
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

    /// 淘汰所有已过 TTL 的条目（`elapsed >= ttl`，已被 `is_recently_failed` 视为未失败）。
    fn evict_expired(&mut self) {
        self.entries.retain(|_, t| t.elapsed() < self.ttl);
    }

    /// 淘汰最旧（`Instant` 最小）的失败记录，直至不超容量。
    fn evict_oldest(&mut self) {
        while self.max_entries > 0 && self.entries.len() > self.max_entries {
            // Instant 为单调时钟，最小值 = 最早记录 = 最接近自然过期，淘汰代价最低。
            let victim = match self.entries.iter().min_by_key(|(_, t)| *t) {
                Some((k, _)) => k.clone(),
                None => break,
            };
            self.entries.remove(&victim);
        }
    }
}

/// 负缓存默认容量上限，对齐同 crate `HttpCache::new` 的 `max_entries`。
pub const DEFAULT_MAX_ENTRIES: usize = 1000;

/// 全局共享负缓存（进程内所有 fetch 路径共用同一份失败记忆）。
pub fn shared_negative_cache() -> Arc<Mutex<NegativeCache>> {
    static CACHE: OnceLock<Arc<Mutex<NegativeCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            Arc::new(Mutex::new(NegativeCache::with_max_entries(
                Duration::from_secs(30),
                DEFAULT_MAX_ENTRIES,
            )))
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_and_query_within_ttl() {
        let mut c = NegativeCache::new(Duration::from_secs(30));
        c.mark_failed("https://x/a");
        assert!(c.is_recently_failed("https://x/a"));
        c.mark_ok("https://x/a");
        assert!(!c.is_recently_failed("https://x/a"));
        assert!(c.is_empty());
    }

    /// R3375：失败 URL 由页面可控（任意失败 `<img>`/`fetch()`），负缓存无上限会
    /// 随不同失败 URL 数单调增长 → 内存放大 DoS。容量上限 + LRU 淘汰收敛。
    #[test]
    fn mark_failed_evicts_when_over_capacity_r3375() {
        let mut c = NegativeCache::with_max_entries(Duration::from_secs(30), 3);
        c.mark_failed("u1");
        c.mark_failed("u2");
        c.mark_failed("u3");
        assert_eq!(c.len(), 3);
        // 第 4 个不同失败 URL → 超容量淘汰最旧（u1，最早 Instant）。
        c.mark_failed("u4");
        assert_eq!(c.len(), 3);
        assert!(!c.is_recently_failed("u1"));
        assert!(c.is_recently_failed("u2"));
        assert!(c.is_recently_failed("u3"));
        assert!(c.is_recently_failed("u4"));
    }

    /// R3375：过期条目（`elapsed >= ttl`）虽被 `is_recently_failed` 视为未失败，
    /// 但若不清除会常驻映射；`is_recently_failed` 惰性清理 + `mark_failed` 主动
    /// 清理过期，确保过期记录不占容量。
    #[test]
    fn expired_entries_evicted_not_retained_r3375() {
        let mut c = NegativeCache::with_max_entries(Duration::from_millis(1), 1000);
        c.mark_failed("expired");
        // 等待 TTL 过期。
        std::thread::sleep(Duration::from_millis(5));
        // is_recently_failed 惰性清理：过期 URL 查询后即移除。
        assert!(!c.is_recently_failed("expired"));
        assert!(c.is_empty());
    }

    /// R3375：超容量时 `mark_failed` 先清过期条目再 LRU 淘汰——若过期清理已腾出
    /// 空间则无需淘汰仍有效的记录（避免过早驱逐活跃失败记忆）。
    #[test]
    fn mark_failed_clears_expired_before_lru_evict_r3375() {
        let mut c = NegativeCache::with_max_entries(Duration::from_millis(1), 2);
        c.mark_failed("old1");
        c.mark_failed("old2");
        // 等待前两个过期。
        std::thread::sleep(Duration::from_millis(5));
        // 第 3 个新失败：超容量，但清理过期（old1/old2）后腾出空间 → 不淘汰。
        c.mark_failed("new");
        assert_eq!(c.len(), 1);
        assert!(c.is_recently_failed("new"));
        assert!(!c.is_recently_failed("old1"));
        assert!(!c.is_recently_failed("old2"));
    }

    /// R3375：`max_entries = 0`（测试用，生产路径恒 > 0）视为无限制，不淘汰。
    #[test]
    fn zero_max_entries_means_unbounded_r3375() {
        let mut c = NegativeCache::with_max_entries(Duration::from_secs(30), 0);
        for i in 0..10 {
            c.mark_failed(&format!("u{i}"));
        }
        assert_eq!(c.len(), 10);
    }
}
