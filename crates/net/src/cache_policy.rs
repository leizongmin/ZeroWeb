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

/// 缓存写入模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheStoreMode {
    /// 可在 TTL 内作为新鲜资源提供。
    Fresh(u64),
    /// 可存储但每次使用前必须再验证（`Cache-Control: no-cache`）。
    RevalidateOnly,
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

fn has_validators(response: &HttpResponse) -> bool {
    response.header("etag").is_some() || response.header("last-modified").is_some()
}

/// RFC 9111 §4.2.3——计算响应被缓存接收时的「初始年龄」（秒），即 `corrected_initial_age`。
///
/// `corrected_initial_age = max(apparent_age, age_value)`：
/// - `age_value`：`Age` 头（delta-seconds），缺失/非法 → 0。CDN/共享缓存用它表明响应在其处已存活的秒数。
/// - `apparent_age`：`Date` 头隐含的年龄 = `max(0, response_time - date_value)`。**`Date` 缺失 → 0**
///   （非 spec 字面 `date_value=0` 外推——那会令 `apparent_age=response_time` 致瞬时过期，病态；
///   此处取与主流浏览器一致的「无 `Date` 不据此判龄」）。
///
/// 注：`request_time`/`response_delay` 未跟踪（`HttpResponse` API 不携带 `request_time`），
/// 故 `corrected_age_value` 退化为 `age_value`（`response_delay` 通常亚秒级，主导项为 `Age`）。
/// 返回值用于新鲜度检查：`fresh ⇔ resident_time + initial_age <= freshness_lifetime`。
pub(crate) fn compute_initial_age(response: &HttpResponse) -> u64 {
    // https://www.rfc-editor.org/rfc/rfc9111#section-4.2.3
    let age_value = response
        .header("age")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let apparent_age = response
        .header("date")
        .and_then(|d| parse_http_date(d).ok())
        .map(|date_value| {
            let response_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            response_time.saturating_sub(date_value)
        })
        .unwrap_or(0);
    apparent_age.max(age_value)
}

/// 响应是否应写入缓存及其模式。
pub(crate) fn storable_mode(response: &HttpResponse) -> Option<CacheStoreMode> {
    let cc = parse_cache_control(response);
    if cc.no_store || !is_cacheable_status(response.status_code) {
        return None;
    }
    if cc.no_cache {
        return has_validators(response).then_some(CacheStoreMode::RevalidateOnly);
    }
    match compute_ttl_secs(&cc, response) {
        Some(ttl) if ttl > 0 => Some(CacheStoreMode::Fresh(ttl)),
        _ => None,
    }
}

fn parse_http_date(date_str: &str) -> Result<u64, ()> {
    crate::cookie::parse_expires_date(date_str).ok_or(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resp(headers: Vec<(&str, &str)>) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: headers.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            body: vec![],
            url: "https://example.com/".into(),
            redirect_count: 0,
        }
    }

    #[test]
    fn no_cache_storable_with_validators() {
        let r = resp(vec![("cache-control", "no-cache"), ("etag", "\"x\"")]);
        assert_eq!(storable_mode(&r), Some(CacheStoreMode::RevalidateOnly));
    }

    #[test]
    fn no_cache_not_storable_without_validators() {
        let r = resp(vec![("cache-control", "no-cache")]);
        assert_eq!(storable_mode(&r), None);
    }

    /// R3233：compute_initial_age 对照 RFC 9111 §4.2.3——`corrected_initial_age = max(apparent_age, age_value)`。
    #[test]
    fn compute_initial_age_r3233() {
        // 无 Age / Date → 0（不据此判龄，避免病态瞬时过期）。
        assert_eq!(compute_initial_age(&resp(vec![("cache-control", "max-age=60")])), 0);
        // Age 头 → age_value。
        assert_eq!(compute_initial_age(&resp(vec![("age", "90")])), 90);
        // 非法 Age → 0（不解析为数字）。
        assert_eq!(compute_initial_age(&resp(vec![("age", "not-a-number")])), 0);
        // 带空白的 Age 头 → trim 后解析。
        assert_eq!(compute_initial_age(&resp(vec![("age", "  42  ")])), 42);
        // Date 头在远过去 → apparent_age 巨大（响应早已过期生成）；无 Age 时取 apparent_age。
        let far_past = resp(vec![("date", "Wed, 21 Oct 2015 07:28:00 GMT")]);
        assert!(
            compute_initial_age(&far_past) > 1_000_000,
            "远过去 Date 须算出大 apparent_age"
        );
        // Age + 远过去 Date → 取 max（此处 apparent_age 主导）。
        let both = resp(vec![("age", "10"), ("date", "Wed, 21 Oct 2015 07:28:00 GMT")]);
        assert!(compute_initial_age(&both) > 1_000_000);
        // Date 在未来（时钟偏差/伪造）→ apparent_age saturating 为 0；回落 age_value。
        let future = resp(vec![("age", "30"), ("date", "Wed, 21 Oct 2099 07:28:00 GMT")]);
        assert_eq!(compute_initial_age(&future), 30);
    }
}
