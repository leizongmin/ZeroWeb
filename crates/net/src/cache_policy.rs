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

/// 请求侧 Cache-Control 指令；仅包含影响私有缓存复用的 P0 字段。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RequestCacheControl {
    pub no_cache: bool,
    pub no_store: bool,
    pub only_if_cached: bool,
    pub max_age: Option<u64>,
    pub min_fresh: Option<u64>,
    /// `Some(None)` 表示未限定秒数的 `max-stale`。
    pub max_stale: Option<Option<u64>>,
}

/// Combine repeated Vary field lines in their received order, as required by HTTP field semantics.
/// https://www.rfc-editor.org/rfc/rfc9110#section-5.2
pub(crate) fn vary_header_value(response: &HttpResponse) -> Option<String> {
    let mut values = response
        .headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("vary"))
        .map(|(_, value)| value.as_str());
    let mut combined = values.next()?.to_string();
    for value in values {
        combined.push_str(", ");
        combined.push_str(value);
    }
    Some(combined)
}

/// A changed Vary field changes which request headers identify a stored representation.
/// https://www.rfc-editor.org/rfc/rfc9111#section-4.1
pub(crate) fn vary_field_changed(stored: Option<&str>, updated: Option<&str>) -> bool {
    let Some(updated) = updated else {
        return false;
    };

    fn normalized_fields(value: Option<&str>) -> Vec<String> {
        let mut fields = value
            .into_iter()
            .flat_map(|value| value.split(','))
            .map(str::trim)
            .filter(|field| !field.is_empty())
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();
        fields.sort_unstable();
        fields.dedup();
        fields
    }

    normalized_fields(stored) != normalized_fields(Some(updated))
}

/// 解析请求头中的 Cache-Control。
pub(crate) fn parse_request_cache_control(headers: &[(String, String)]) -> RequestCacheControl {
    let mut result = RequestCacheControl::default();
    for (_, value) in headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("cache-control"))
    {
        for directive in value.split(',').map(str::trim) {
            if directive.eq_ignore_ascii_case("no-cache") {
                result.no_cache = true;
            } else if directive.eq_ignore_ascii_case("no-store") {
                result.no_store = true;
            } else if directive.eq_ignore_ascii_case("only-if-cached") {
                result.only_if_cached = true;
            } else if directive.eq_ignore_ascii_case("max-stale") {
                result.max_stale = Some(None);
            } else {
                let lower = directive.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("max-age=") {
                    result.max_age = value.trim().parse().ok();
                } else if let Some(value) = lower.strip_prefix("min-fresh=") {
                    result.min_fresh = value.trim().parse().ok();
                } else if let Some(value) = lower.strip_prefix("max-stale=") {
                    result.max_stale = value.trim().parse().ok().map(Some);
                }
            }
        }
    }
    result
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

/// RFC 9110 §15.1——默认启发式可缓存的状态码（无显式新鲜度信息时的可存依据之一）。
/// https://www.rfc-editor.org/rfc/rfc9110#section-15.1
const HEURISTICALLY_CACHEABLE_STATUSES: [u16; 8] = [200, 203, 204, 404, 405, 410, 414, 501];

/// 计算缓存 TTL（秒）；`None` 或 `Some(0)` 表示不应作为新鲜资源提供。
pub(crate) fn compute_ttl_secs(cc: &CacheControl, response: &HttpResponse) -> Option<u64> {
    if cc.no_cache {
        return Some(0);
    }
    // R3392：`s-maxage` 仅对**共享缓存**（CDN/代理）生效（RFC 9111 §4.2.1），私有缓存
    // （浏览器）**必须忽略** 它并回退 max-age。旧实现无条件优先返回 s_maxage →
    // `Cache-Control: s-maxage=3600, max-age=60` 在浏览器侧误按 3600s 视为新鲜（过度缓存，
    // 可能在服务器期望的 60s 后仍发过期内容）。本仓 HttpCache 为浏览器私有缓存
    // （磁盘层亦为私有磁盘缓存，is_shared 标志恒 false），故一律忽略 s-maxage。
    // s_maxage 仍由 parse_cache_control 解析保留（is_shared 标志据此置位），但不参与 TTL。
    // 规范引用：https://www.rfc-editor.org/rfc/rfc9111#section-4.2.1
    if let Some(max_age) = cc.max_age {
        return Some(max_age);
    }
    if let Some(expires) = response.header("expires") {
        if let Ok(expires_time) = parse_http_date(expires) {
            // R3235：RFC 9111 §4.2.1——Expires 给出的新鲜期相对**响应生成时刻**（Date 头）而非客户端 now，
            // 以消除 server/client 时钟偏差：server 钟快于 client 时，`expires - now` 会多给若干秒。
            // 与 R3233 apparent_age（`max(0, response_time - date_value)`）配套：date_value 同时作为
            // 新鲜期起点 + 年龄起点，二者对齐消除偏差。Date 缺失（§6.6.1 谓 origin 应发，缺失时无基准）回落 now。
            let reference = response
                .header("date")
                .and_then(|d| parse_http_date(d).ok())
                .unwrap_or_else(|| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                });
            if expires_time > reference {
                return Some(expires_time - reference);
            }
            return Some(0);
        }
        // RFC 9111 §4.2.1/§4.2.2——Expires 字段**在场**但值不可解析：显式过期信息在场
        // 禁用启发式（§4.2.2 MUST NOT），无效新鲜度信息鼓励按 stale 处理 → TTL 0
        // （验证器在场时可存为仅再验证条目）。http-cache goal M3 freshness 语料
        // invalid-Expires 臂（`Expires: 0`）。
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
///
/// RFC 9111 §3——私有缓存存储条件：请求方法可理解（本实现全部 final 状态可理解）、
/// 状态码为 final（本实现仅见 final 状态）、无 `no-store`，且响应至少具备**一项可存
/// 依据**：`public` / `private` / `Expires` 字段在场（哪怕值不可解析——§3 仅要求字段
/// 存在）/ `max-age`，或状态码属于启发式可缓存集（RFC 9110 §15.1）。`s-maxage` 是
/// 共享缓存指令，不构成私有缓存依据（§3 该项以「cache is shared」为前提）。
/// http-cache goal M3：status 语料 299/4xx/5xx/未知码 fresh 腿（Expires 在场）与
/// heuristic 语料 true/false 集均依赖本条件。
/// https://www.rfc-editor.org/rfc/rfc9111#section-3
pub(crate) fn storable_mode(response: &HttpResponse) -> Option<CacheStoreMode> {
    let cc = parse_cache_control(response);
    if cc.no_store {
        return None;
    }
    let has_expires = response.header("expires").is_some();
    let has_explicit_basis = cc.public || cc.private || has_expires || cc.max_age.is_some();
    let heuristic_eligible = HEURISTICALLY_CACHEABLE_STATUSES.contains(&response.status_code) || cc.public;
    if !has_explicit_basis && !heuristic_eligible {
        return None;
    }
    // https://www.rfc-editor.org/rfc/rfc9111#section-4.1
    // Vary: * 永不匹配后续请求——复用被禁止，但 RFC 只禁**使用**不禁存储（http-cache
    // goal M2：WPT request-cache-force-cache vary 臂判据——Chromium 对不可复用的
    // Vary:* 条目仍以其验证器发条件请求，故须存为仅再验证条目；复用禁止由 lookup
    // 层保证——revalidate_only 条目恒 Revalidate 不 Hit，且 only-if-cached/force-cache
    // 的 stale 复用臂跳过 Vary:* 条目，见 resource_loader）。
    if vary_header_value(response).is_some_and(|vary| vary.split(',').any(|field| field.trim() == "*")) {
        return has_validators(response).then_some(CacheStoreMode::RevalidateOnly);
    }
    if cc.no_cache {
        return has_validators(response).then_some(CacheStoreMode::RevalidateOnly);
    }
    let ttl = if cc.max_age.is_some() || has_expires {
        // RFC 9111 §4.2.1——显式过期信息在场：max-age 优先于 Expires（首个匹配）；
        // Expires 不可解析已由 compute_ttl_secs 归 0。
        compute_ttl_secs(&cc, response).unwrap_or(0)
    } else {
        // RFC 9111 §4.2.2——启发式新鲜度：**仅**当无显式过期时间（MUST NOT 例外已由
        // 上方分支排除），且（状态码启发式可缓存 或 `public` 显式可缓存标记）。
        // LM 启发式取「生成以来时长的 10%」；无 Last-Modified → 0（立即过期，验证器
        // 在场时可存为仅再验证条目）。`private` 不在此列（§4.2.2 的显式可缓存标记
        // 示例仅 public；保守不对 private 启发式）。
        // http-cache goal M3：heuristic 语料 true 集（200/203/204/404/405/410/414/501
        // +LM，299+public+LM）/ false 集（201/202/299 裸/403/502/503/504+LM）。
        // https://www.rfc-editor.org/rfc/rfc9111#section-4.2.2
        heuristic_freshness_secs(response)
    };
    match ttl {
        ttl if ttl > 0 => Some(CacheStoreMode::Fresh(ttl)),
        // RFC 9111 §3——生成即过期（ttl=0：`max-age=0` / 已过期 `Expires` / 无 LM 的
        // 启发式）但带验证器的响应**可存储**为仅再验证条目：后续请求得以发条件请求
        // 拿 304，而非每次全量回源（WPT request-cache 语料 stale 臂依赖此语义）；
        // 无验证器时存了也无法再验证，不存。
        // https://www.rfc-editor.org/rfc/rfc9111#section-3
        _ => has_validators(response).then_some(CacheStoreMode::RevalidateOnly),
    }
}

/// RFC 9111 §4.2.2——LM 启发式：`10% × (Date 或接收时刻 − Last-Modified)`。
/// 无 Last-Modified（或不可解析）→ 0。heuristic 语料 299+public+LM(−300) → ~30s。
/// https://www.rfc-editor.org/rfc/rfc9111#section-4.2.2
fn heuristic_freshness_secs(response: &HttpResponse) -> u64 {
    let Some(modified) = response.header("last-modified").and_then(|m| parse_http_date(m).ok()) else {
        return 0;
    };
    let reference = response
        .header("date")
        .and_then(|d| parse_http_date(d).ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
    reference.saturating_sub(modified) / 10
}

/// 请求是否携带**显式条件头**。
///
/// `default` 模式请求若由脚本显式设置 `If-Modified-Since` / `If-None-Match` /
/// `If-Unmodified-Since` / `If-Match` / `If-Range` 之一，fetch 规范将缓存模式置为
/// `no-store`：不读缓存、不写缓存、用户条件头原样透传（调用方自行管理再验证，
/// 缓存层不得代合并验证器——否则日期变体腿会命中旧 200 而非服务器 304）。
/// Chromium 对齐行为，WPT request-cache-default-conditional 断言面。
/// https://fetch.spec.whatwg.org/#http-network-or-cache-fetch（step 16）
pub fn has_explicit_conditional_request_headers(headers: &[(String, String)]) -> bool {
    const CONDITIONAL_HEADERS: [&str; 5] = [
        "if-modified-since",
        "if-none-match",
        "if-unmodified-since",
        "if-match",
        "if-range",
    ];
    headers.iter().any(|(name, _)| {
        CONDITIONAL_HEADERS
            .iter()
            .any(|header| name.eq_ignore_ascii_case(header))
    })
}

/// `no-store` / `reload` 绕行请求的网络可观察注入头。
///
/// 返回需追加的 `(头名, 值)`：缺 `Pragma` 补 `Pragma: no-cache`，缺 `Cache-Control`
/// 补 `Cache-Control: no-cache`（任一已存在则不重复注入）。WPT request-cache-reload
/// / -no_store 语料的 `expected_no_cache_headers` 断言两头**同时**在场——这是
/// 服务器可观测的缓存绕行形态。
/// https://fetch.spec.whatwg.org/#http-network-or-cache-fetch（step 18）
pub fn bypass_cache_request_injection(headers: &[(String, String)]) -> Vec<(String, String)> {
    let mut injection = Vec::with_capacity(2);
    if !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("pragma")) {
        injection.push(("Pragma".into(), "no-cache".into()));
    }
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("cache-control"))
    {
        injection.push(("Cache-Control".into(), "no-cache".into()));
    }
    injection
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

    // ── RFC 9111 §3 存储条件 + §4.2.2 启发式（http-cache goal M3）──

    /// Date: Tue, 02 Jan 2024 00:00:00 GMT。
    const ANCHOR_DATE: &str = "Tue, 02 Jan 2024 00:00:00 GMT";
    /// ANCHOR_DATE + 100000s → Fresh(100000)。
    const FUTURE_EXPIRES: &str = "Wed, 03 Jan 2024 03:46:40 GMT";
    /// ANCHOR_DATE − 300s → LM 启发式 = 30s。
    const RECENT_LAST_MODIFIED: &str = "Mon, 01 Jan 2024 23:55:00 GMT";

    #[test]
    fn m3_status_with_explicit_freshness_storable_regardless_of_code() {
        // §3 可存依据 = Expires/max-age 在场，与状态码无关——status 语料 299/4xx/
        // 5xx/未知码 fresh 腿（Expires +100000 模板）全部可存。
        let r299 = resp(vec![("cache-control", "max-age=100")]);
        let mut r299 = r299;
        r299.status_code = 299;
        assert_eq!(storable_mode(&r299), Some(CacheStoreMode::Fresh(100)));

        let r599 = resp(vec![("expires", FUTURE_EXPIRES), ("date", ANCHOR_DATE)]);
        let mut r599 = r599;
        r599.status_code = 599;
        assert_eq!(storable_mode(&r599), Some(CacheStoreMode::Fresh(100000)));

        let r500 = resp(vec![("expires", FUTURE_EXPIRES), ("date", ANCHOR_DATE)]);
        let mut r500 = r500;
        r500.status_code = 500;
        assert_eq!(storable_mode(&r500), Some(CacheStoreMode::Fresh(100000)));
    }

    #[test]
    fn m3_status_without_any_basis_not_storable() {
        // §3——无 public/private/Expires/max-age 且状态码非启发式可缓存集 → 不存。
        // status/heuristic 语料 false 腿（299 裸、201/202/403/502/503/504 + LM）。
        for status in [299, 201, 202, 403, 502, 503, 504] {
            let mut r = resp(vec![("last-modified", RECENT_LAST_MODIFIED), ("date", ANCHOR_DATE)]);
            r.status_code = status;
            assert_eq!(storable_mode(&r), None, "status {status} 无可存依据须不存");
        }
    }

    #[test]
    fn m3_heuristic_freshness_for_cacheable_status_with_lm() {
        // §4.2.2——启发式可缓存状态码 + 无显式过期 + LM → 10% 启发式。
        let mut r = resp(vec![("last-modified", RECENT_LAST_MODIFIED), ("date", ANCHOR_DATE)]);
        r.status_code = 404;
        assert_eq!(storable_mode(&r), Some(CacheStoreMode::Fresh(30)));
        r.status_code = 200;
        assert_eq!(storable_mode(&r), Some(CacheStoreMode::Fresh(30)));
    }

    #[test]
    fn m3_heuristic_freshness_via_public_marker_for_uncached_status() {
        // §4.2.2——显式可缓存标记（public）为 299 打开启发式（heuristic 语料 true 腿）。
        let r = resp(vec![
            ("cache-control", "public"),
            ("last-modified", RECENT_LAST_MODIFIED),
            ("date", ANCHOR_DATE),
        ]);
        let mut r = r;
        r.status_code = 299;
        assert_eq!(storable_mode(&r), Some(CacheStoreMode::Fresh(30)));
    }

    #[test]
    fn m3_heuristic_suppressed_by_explicit_expiration_info() {
        // §4.2.2 MUST NOT——显式过期信息在场禁用启发式：max-age=0 + LM → 仅再验证，
        // 而非启发式 30s；invalid Expires + LM → 同（§4.2.1 无效新鲜度按 stale）。
        let max_age_zero = resp(vec![
            ("cache-control", "max-age=0"),
            ("last-modified", RECENT_LAST_MODIFIED),
            ("date", ANCHOR_DATE),
        ]);
        let mut max_age_zero = max_age_zero;
        max_age_zero.status_code = 404;
        assert_eq!(storable_mode(&max_age_zero), Some(CacheStoreMode::RevalidateOnly));

        let invalid_expires = resp(vec![
            ("expires", "0"),
            ("last-modified", RECENT_LAST_MODIFIED),
            ("date", ANCHOR_DATE),
        ]);
        let mut invalid_expires = invalid_expires;
        invalid_expires.status_code = 200;
        assert_eq!(storable_mode(&invalid_expires), Some(CacheStoreMode::RevalidateOnly));

        // invalid Expires 无验证器 → 存了也无法再验证，不存（freshness 语料 leg 4）。
        let invalid_expires_no_validator = resp(vec![("expires", "0"), ("date", ANCHOR_DATE)]);
        assert_eq!(storable_mode(&invalid_expires_no_validator), None);
    }

    #[test]
    fn m3_heuristic_without_lm_is_immediately_stale() {
        // §4.2.2——启发式可缓存状态码 + 无 LM → ttl 0；无验证器不存。
        let r = resp(vec![("date", ANCHOR_DATE)]);
        assert_eq!(storable_mode(&r), None);
        // 有验证器 → 仅再验证条目。
        let with_validator = resp(vec![("date", ANCHOR_DATE), ("etag", "\"v1\"")]);
        assert_eq!(storable_mode(&with_validator), Some(CacheStoreMode::RevalidateOnly));
    }

    #[test]
    fn request_cache_control_parses_reuse_constraints() {
        let policy = parse_request_cache_control(&[(
            "Cache-Control".into(),
            "max-age=10, min-fresh=3, max-stale=20, only-if-cached".into(),
        )]);
        assert_eq!(policy.max_age, Some(10));
        assert_eq!(policy.min_fresh, Some(3));
        assert_eq!(policy.max_stale, Some(Some(20)));
        assert!(policy.only_if_cached);

        let unlimited = parse_request_cache_control(&[("cache-control".into(), "max-stale".into())]);
        assert_eq!(unlimited.max_stale, Some(None));
    }

    #[test]
    fn vary_star_is_storable_only_for_revalidation() {
        // http-cache goal M2：Vary:* 仅禁复用不禁存储（RFC 9111 §4.1）——带验证器
        // 存为仅再验证条目（lookup 恒 Revalidate 不 Hit；loader stale 复用臂跳过），
        // 无验证器不存。
        let no_validator = resp(vec![("cache-control", "max-age=60"), ("vary", "*")]);
        assert_eq!(storable_mode(&no_validator), None);
        let with_validator = resp(vec![("cache-control", "max-age=60"), ("etag", "\"v1\""), ("vary", "*")]);
        assert_eq!(storable_mode(&with_validator), Some(CacheStoreMode::RevalidateOnly));
    }

    #[test]
    fn repeated_vary_field_lines_are_combined_and_vary_star_unstorable_without_validator() {
        let r = resp(vec![("vary", "Accept-Language"), ("Vary", "Accept-Encoding")]);
        assert_eq!(
            vary_header_value(&r).as_deref(),
            Some("Accept-Language, Accept-Encoding")
        );

        let star = resp(vec![
            ("cache-control", "max-age=60"),
            ("vary", "Accept-Language"),
            ("vary", "*"),
        ]);
        assert_eq!(storable_mode(&star), None);
    }

    #[test]
    fn vary_field_change_ignores_case_order_and_whitespace() {
        assert!(!vary_field_changed(
            Some("Accept-Encoding, Accept-Language"),
            Some(" accept-language,accept-encoding "),
        ));
        assert!(vary_field_changed(None, Some("Accept-Language")));
        assert!(vary_field_changed(Some("Accept-Language"), Some("Accept-Encoding")));
        assert!(!vary_field_changed(Some("Accept-Language"), None));
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

    /// R3235：RFC 9111 §4.2.1——Expires 新鲜期相对 Date（响应生成时刻）而非客户端 now。
    /// Date/Expires 同在未来且相距 100s → 新鲜期 = 100（非 expires - wallclock now）。
    #[test]
    fn compute_ttl_expires_uses_date_not_now_r3235() {
        // Date 07:28:00 + Expires 07:29:40 = 相距 100s（均在未来，排除 wallclock 干扰）。
        let r = resp(vec![
            ("date", "Wed, 21 Oct 2099 07:28:00 GMT"),
            ("expires", "Wed, 21 Oct 2099 07:29:40 GMT"),
        ]);
        // storable_mode → Fresh(100)；旧实现 expires - now(2026) 会得巨大值（数十年）。
        assert_eq!(storable_mode(&r), Some(CacheStoreMode::Fresh(100)));
        // 直接验 compute_ttl_secs（经 cc 无 max-age/s_maxage → 落 Expires 分支）。
        let cc = parse_cache_control(&r);
        assert_eq!(compute_ttl_secs(&cc, &r), Some(100));

        // Expires 早于 Date（生成即过期）→ ttl=0（不可作新鲜资源）。
        let already_expired = resp(vec![
            ("date", "Wed, 21 Oct 2099 07:28:00 GMT"),
            ("expires", "Wed, 21 Oct 2099 07:27:00 GMT"),
        ]);
        assert_eq!(storable_mode(&already_expired), None);

        // 无 Date 时回落 now（行为同旧）：Expires 在远过去 → ttl=0 → 不可新鲜存储。
        let no_date_past = resp(vec![("expires", "Wed, 21 Oct 2015 07:28:00 GMT")]);
        assert_eq!(storable_mode(&no_date_past), None);
    }

    /// RFC 9111 §3——生成即过期但带验证器的响应存为仅再验证条目（stale 臂前置）。
    #[test]
    fn stale_at_birth_with_validators_is_storable_for_revalidation() {
        // max-age=0 + ETag → 可存（每次使用前再验证）。
        let with_etag = resp(vec![("cache-control", "max-age=0"), ("etag", "\"v1\"")]);
        assert_eq!(storable_mode(&with_etag), Some(CacheStoreMode::RevalidateOnly));
        // 过期 Expires + Last-Modified → 可存。
        let with_lm = resp(vec![
            ("date", "Wed, 21 Oct 2099 07:28:00 GMT"),
            ("expires", "Wed, 21 Oct 2099 07:27:00 GMT"),
            ("last-modified", "Wed, 21 Oct 2015 07:28:00 GMT"),
        ]);
        assert_eq!(storable_mode(&with_lm), Some(CacheStoreMode::RevalidateOnly));
        // 生成即过期但无验证器 → 存了无法再验证，不存。
        let bare = resp(vec![("cache-control", "max-age=0")]);
        assert_eq!(storable_mode(&bare), None);
    }

    /// fetch spec §4.6 step 16——五显式条件头识别（大小写不敏感、逐头独立）。
    #[test]
    fn explicit_conditional_request_headers_are_detected() {
        assert!(has_explicit_conditional_request_headers(&[(
            "If-None-Match".into(),
            "\"v1\"".into()
        )]));
        assert!(has_explicit_conditional_request_headers(&[(
            "if-modified-since".into(),
            "Wed, 21 Oct 2015 07:28:00 GMT".into()
        )]));
        for name in ["If-Unmodified-Since", "If-Match", "If-Range"] {
            assert!(
                has_explicit_conditional_request_headers(&[(name.into(), "x".into())]),
                "{name} 属显式条件头"
            );
        }
        assert!(!has_explicit_conditional_request_headers(&[
            ("Accept".into(), "*/*".into()),
            ("Cache-Control".into(), "no-cache".into()),
        ]));
    }

    /// fetch spec §4.6 step 18——绕行注入两头齐发，用户已给的任一头不重复注入。
    #[test]
    fn bypass_injection_adds_pragma_and_cache_control_only_when_absent() {
        let injection = bypass_cache_request_injection(&[("Accept".into(), "*/*".into())]);
        assert_eq!(
            injection,
            vec![
                ("Pragma".to_string(), "no-cache".to_string()),
                ("Cache-Control".to_string(), "no-cache".to_string()),
            ]
        );
        // 用户已带 Cache-Control → 只补 Pragma。
        assert_eq!(
            bypass_cache_request_injection(&[("Cache-Control".into(), "no-cache".into())]),
            vec![("Pragma".to_string(), "no-cache".to_string())]
        );
        // 两头均在 → 零注入。
        assert!(
            bypass_cache_request_injection(&[
                ("Pragma".into(), "no-cache".into()),
                ("Cache-Control".into(), "no-cache".into()),
            ])
            .is_empty()
        );
    }
}
