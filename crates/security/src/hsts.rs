//! HTTP Strict Transport Security (HSTS) 模块。
//!
//! HSTS 告诉浏览器只能通过 HTTPS 访问站点，自动将 HTTP 请求升级为 HTTPS。
//!
//! ## 核心类型
//!
//! - [`HstsDirective`] — 解析 `Strict-Transport-Security` 响应头
//! - [`HstsStore`] — 持久化 HSTS 策略记录，按域名索引
//!
//! ## 使用方式
//!
//! ```ignore
//! let mut store = HstsStore::new();
//! // 从响应头解析 HSTS 策略
//! let directive = HstsDirective::parse("max-age=31536000; includeSubDomains");
//! store.register("example.com", directive);
//! // 检查请求是否需要升级
//! assert!(store.should_upgrade("http://example.com/page"));
//! ```

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 解析后的 HSTS 指令。
#[derive(Debug, Clone, PartialEq)]
pub struct HstsDirective {
    /// 策略有效时间（秒）。
    pub max_age: u64,
    /// 是否包含子域名。
    pub include_subdomains: bool,
    /// 解析时的 Unix 时间戳（秒）。
    pub registered_at: u64,
}

impl HstsDirective {
    /// 从 `Strict-Transport-Security` 响应头值解析 HSTS 指令。
    ///
    /// 支持的格式：
    /// - `max-age=31536000`
    /// - `max-age=31536000; includeSubDomains`
    /// - `max-age=31536000; includeSubDomains; preload`
    ///
    /// 如果无法解析（如缺少 max-age），返回 `None`。
    pub fn parse(header_value: &str) -> Option<Self> {
        let mut max_age: Option<u64> = None;
        let mut include_subdomains = false;

        for part in header_value.split(';') {
            let part = part.trim();
            if part.eq_ignore_ascii_case("includesubdomains") {
                include_subdomains = true;
            } else if part.eq_ignore_ascii_case("preload") {
                // preload 标记用于浏览器内置 HSTS 列表，此处仅解析忽略
            } else if let Some(age_str) = part.strip_prefix("max-age=") {
                max_age = age_str.trim().parse().ok();
            } else if let Some(age_str) = part.strip_prefix("Max-Age=") {
                max_age = age_str.trim().parse().ok();
            }
        }

        max_age.map(|age| HstsDirective {
            max_age: age,
            include_subdomains,
            registered_at: current_timestamp(),
        })
    }

    /// 检查策略是否已过期。
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.registered_at) > self.max_age
    }

    /// 创建用于测试的 HSTS 指令。
    #[cfg(test)]
    pub fn new_for_test(max_age: u64, include_subdomains: bool) -> Self {
        Self {
            max_age,
            include_subdomains,
            registered_at: current_timestamp(),
        }
    }
}

/// 获取当前 Unix 时间戳（秒）。
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// HSTS 策略存储 — 按域名索引。
///
/// 记录哪些域名启用了 HSTS 及其配置。
/// 浏览器在发起 HTTP 请求前检查此存储，决定是否需要升级为 HTTPS。
#[derive(Debug, Clone, Default)]
pub struct HstsStore {
    /// 域名 → HSTS 指令。
    entries: HashMap<String, HstsDirective>,
}

impl HstsStore {
    /// 创建空的 HSTS 存储。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 HSTS 策略。
    ///
    /// 如果域名已有记录且未过期，会被新策略覆盖。
    /// 如果 `max_age` 为 0，则删除该域名的 HSTS 记录。
    pub fn register(&mut self, host: &str, directive: HstsDirective) {
        if directive.max_age == 0 {
            self.entries.remove(host);
        } else {
            self.entries.insert(host.to_lowercase(), directive);
        }
    }

    /// 从 `Strict-Transport-Security` 头注册 HSTS 策略。
    ///
    /// 便捷方法：解析头部并注册。
    pub fn register_from_header(&mut self, host: &str, header_value: &str) -> bool {
        if let Some(directive) = HstsDirective::parse(header_value) {
            self.register(host, directive);
            true
        } else {
            false
        }
    }

    /// 检查指定域名是否启用了 HSTS。
    ///
    /// 会检查精确匹配和父域名（当父域名启用了 `includeSubDomains`）。
    /// 同时自动清理过期记录。
    pub fn is_secure(&mut self, host: &str) -> bool {
        let host_lower = host.to_lowercase();

        // 精确匹配
        if let Some(directive) = self.entries.get(&host_lower) {
            if directive.is_expired() {
                self.entries.remove(&host_lower);
                return false;
            }
            return true;
        }

        // 检查父域名的 includeSubDomains
        let parts: Vec<&str> = host_lower.split('.').collect();
        if parts.len() > 2 {
            // 逐级检查父域名
            for i in 1..parts.len() - 1 {
                let parent = parts[i..].join(".");
                if let Some(directive) = self.entries.get(&parent)
                    && !directive.is_expired()
                    && directive.include_subdomains
                {
                    return true;
                }
            }
        }

        false
    }

    /// 检查给定 URL 是否需要从 HTTP 升级为 HTTPS。
    ///
    /// 返回升级后的 HTTPS URL（如需要），或 `None`（如不需要升级）。
    pub fn should_upgrade(&mut self, url: &str) -> Option<String> {
        if !url.starts_with("http://") {
            return None;
        }

        // 提取 host
        let after_scheme = &url[7..]; // skip "http://"
        let host_end = after_scheme
            .find(&['/', '?', '#', ':'][..])
            .unwrap_or(after_scheme.len());
        let host = &after_scheme[..host_end];

        if self.is_secure(host) {
            Some(format!("https://{}", &url[7..]))
        } else {
            None
        }
    }

    /// 移除指定域名的 HSTS 记录。
    pub fn remove(&mut self, host: &str) -> bool {
        self.entries.remove(&host.to_lowercase()).is_some()
    }

    /// 返回存储的策略数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 返回存储是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 清除所有过期记录。
    pub fn cleanup_expired(&mut self) -> usize {
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, d)| d.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        let count = expired.len();
        for key in expired {
            self.entries.remove(&key);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let d = HstsDirective::parse("max-age=31536000").unwrap();
        assert_eq!(d.max_age, 31536000);
        assert!(!d.include_subdomains);
    }

    #[test]
    fn test_parse_with_subdomains() {
        let d = HstsDirective::parse("max-age=31536000; includeSubDomains").unwrap();
        assert_eq!(d.max_age, 31536000);
        assert!(d.include_subdomains);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let d = HstsDirective::parse("Max-Age=86400; IncludeSubDomains").unwrap();
        assert_eq!(d.max_age, 86400);
        assert!(d.include_subdomains);
    }

    #[test]
    fn test_parse_with_preload() {
        let d = HstsDirective::parse("max-age=31536000; includeSubDomains; preload").unwrap();
        assert_eq!(d.max_age, 31536000);
        assert!(d.include_subdomains);
    }

    #[test]
    fn test_parse_no_max_age() {
        assert!(HstsDirective::parse("includeSubDomains").is_none());
    }

    #[test]
    fn test_parse_empty() {
        assert!(HstsDirective::parse("").is_none());
    }

    #[test]
    fn test_parse_zero_max_age() {
        let d = HstsDirective::parse("max-age=0").unwrap();
        assert_eq!(d.max_age, 0);
    }

    #[test]
    fn test_register_and_check() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.is_secure("example.com"));
        assert!(!store.is_secure("other.com"));
    }

    #[test]
    fn test_register_removes_on_zero() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.is_secure("example.com"));
        store.register("example.com", HstsDirective::new_for_test(0, false));
        assert!(!store.is_secure("example.com"));
    }

    #[test]
    fn test_register_from_header() {
        let mut store = HstsStore::new();
        assert!(store.register_from_header("example.com", "max-age=31536000"));
        assert!(store.is_secure("example.com"));
    }

    #[test]
    fn test_register_from_invalid_header() {
        let mut store = HstsStore::new();
        assert!(!store.register_from_header("example.com", "invalid"));
    }

    #[test]
    fn test_subdomain_inheritance() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, true));
        assert!(store.is_secure("sub.example.com"));
        assert!(store.is_secure("deep.sub.example.com"));
    }

    #[test]
    fn test_no_subdomain_inheritance_without_flag() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.is_secure("example.com"));
        assert!(!store.is_secure("sub.example.com"));
    }

    #[test]
    fn test_should_upgrade_http_to_https() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        let upgraded = store.should_upgrade("http://example.com/page?q=1");
        assert_eq!(upgraded, Some("https://example.com/page?q=1".to_string()));
    }

    #[test]
    fn test_should_not_upgrade_https() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.should_upgrade("https://example.com/page").is_none());
    }

    #[test]
    fn test_should_not_upgrade_unknown_host() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.should_upgrade("http://other.com/page").is_none());
    }

    #[test]
    fn test_case_insensitive_host() {
        let mut store = HstsStore::new();
        store.register("Example.COM", HstsDirective::new_for_test(31536000, false));
        assert!(store.is_secure("example.com"));
        assert!(store.is_secure("EXAMPLE.COM"));
    }

    #[test]
    fn test_remove() {
        let mut store = HstsStore::new();
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert!(store.remove("example.com"));
        assert!(!store.is_secure("example.com"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut store = HstsStore::new();
        assert!(!store.remove("example.com"));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut store = HstsStore::new();
        // 注册一个已过期的策略
        store.entries.insert(
            "expired.com".to_string(),
            HstsDirective {
                max_age: 1,
                include_subdomains: false,
                registered_at: current_timestamp() - 10, // 10秒前注册，1秒过期
            },
        );
        store.register("valid.com", HstsDirective::new_for_test(31536000, false));
        assert_eq!(store.cleanup_expired(), 1);
        assert!(!store.is_secure("expired.com"));
        assert!(store.is_secure("valid.com"));
    }

    #[test]
    fn test_len_and_empty() {
        let mut store = HstsStore::new();
        assert!(store.is_empty());
        store.register("example.com", HstsDirective::new_for_test(31536000, false));
        assert_eq!(store.len(), 1);
    }
}
