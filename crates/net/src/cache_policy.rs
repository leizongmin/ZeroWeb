//! HTTP 缓存策略 — 解析 Cache-Control / Expires，判定是否可存储。

use std::time::{SystemTime, UNIX_EPOCH};

use crate::request::HttpResponse;

/// Cache-Control 指令解析结果。
#[derive(Debug, Clone, Default)]
pub(crate) struct CacheControl {
    pub max_age: Option<u64>,
    pub s_maxage: Option<u64>,
    pub no_cache: bool,
    pub no_store: bool,
    pub public: bool,
    pub private: bool,
    pub must_revalidate: bool,
}

/// 解析 Cache-Control 头。
pub(crate) fn parse_cache_control(response: &HttpResponse) -> CacheControl {
    let mut cc = CacheControl::default();
    if let Some(value) = response.header("cache-control") {
        for directive in value.split(',') {
            let directive = directive.trim();
            if directive.eq_ignore_ascii_case("no-cache") {
                cc.no_cache = true;
            } else if directive.eq_ignore_ascii_case("no-store") {
                cc.no_store = true;
            } else if directive.eq_ignore_ascii_case("public") {
                cc.public = true;
            } else if directive.eq_ignore_ascii_case("private") {
                cc.private = true;
            } else if directive.eq_ignore_ascii_case("must-revalidate") {
                cc.must_revalidate = true;
            } else {
                let lower = directive.to_ascii_lowercase();
                if let Some(age_str) = lower.strip_prefix("max-age=") {
                    cc.max_age = age_str.trim().parse().ok();
                } else if let Some(age_str) = lower.strip_prefix("s-maxage=") {
                    cc.s_maxage = age_str.trim().parse().ok();
                }
            }
        }
    }
    cc
}

/// 判断 HTTP 状态码是否可缓存。
pub(crate) fn is_cacheable_status(status: u16) -> bool {
    matches!(
        status,
        200 | 203 | 204 | 206 | 300 | 301 | 302 | 304 | 307 | 308 | 404 | 405 | 410 | 414 | 501
    )
}

/// 计算缓存 TTL（秒）；`None` 或 `Some(0)` 表示不应作为新鲜资源提供。
pub(crate) fn compute_ttl_secs(cc: &CacheControl, response: &HttpResponse) -> Option<u64> {
    if cc.no_cache {
        return Some(0);
    }
    if let Some(s_maxage) = cc.s_maxage {
        return Some(s_maxage);
    }
    if let Some(max_age) = cc.max_age {
        return Some(max_age);
    }
    if let Some(expires) = response.header("expires")
        && let Ok(expires_time) = parse_http_date(expires)
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if expires_time > now {
            return Some(expires_time - now);
        }
        return Some(0);
    }
    None
}

/// 响应是否应写入缓存；返回新鲜期秒数。
pub(crate) fn storable_ttl(response: &HttpResponse) -> Option<u64> {
    let cc = parse_cache_control(response);
    if cc.no_store || !is_cacheable_status(response.status_code) {
        return None;
    }
    match compute_ttl_secs(&cc, response) {
        Some(ttl) if ttl > 0 => Some(ttl),
        _ => None,
    }
}

fn parse_http_date(date_str: &str) -> Result<u64, ()> {
    crate::cookie::parse_expires_date(date_str).ok_or(())
}
