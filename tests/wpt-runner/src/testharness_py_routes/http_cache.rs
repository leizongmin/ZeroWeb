//! `fetch/http-cache` 端点族（47 号 http-cache goal M3 语料导入）——上游
//! wptserve handler 1:1 移植：`resources/http-cache.py`（uuid stash 状态机，
//! web-platform-tests pinned rev 315976933870b34d6ea30e3f6643403edae678ba）与
//! `cached_pragma_rand.py` / `cached_pragma_immutable_rand.py`（Pragma ×
//! Cache-Control 对照端点）。服务器侧语义在本文件；缓存语义由外层
//! `wpt_cache_py_wrapped_handler` 承载（loader 镜像，DC-3 单实现）。

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use base64::Engine as _;

use super::PyEndpointCtx;
use crate::testharness::wpt_query_value;

/// http-cache.py 单请求服务器态记录（上游 state dict 1:1——serde 字段名与
/// JSON 产物对齐，语料 `getServerState`/`checkRequests` 按 `request_method`/
/// `request_headers`/`response_headers` 键消费；`now` 仅随响应序列化）。
#[derive(Clone, serde::Serialize)]
pub(crate) struct HttpCachePyState {
    /// 响应生成时刻的 `time.time()` 秒（上游 float 直序列化）。
    now: f64,
    request_method: String,
    /// 全部请求头（键小写化、同名后者胜——上游 dict 推导同语义）。
    request_headers: BTreeMap<String, String>,
    /// NOTEHDRS 记名响应头（键小写化——etag/last-modified 条件验证比对源）。
    response_headers: BTreeMap<String, String>,
}

/// http-cache.py 端点 per-case stash（uuid → 请求态序列）。take/put-back =
/// HashMap remove/insert：`dispatch=state` 与 test 分支首步取走（querystate
/// 同款——取走即清空）；per-case 构建，案内键自清、跨案零泄漏。
pub(crate) type HttpCachePyStash = Arc<Mutex<HashMap<String, Vec<HttpCachePyState>>>>;

/// `fetch/http-cache/resources/http-cache.py` 路由臂——委托 1:1 移植体。
pub(crate) fn route_http_cache_py(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    Some(http_cache_py_fixture(ctx, path_part, req))
}

/// `cached_pragma_rand.py` / `cached_pragma_immutable_rand.py` 路由臂——
/// Pragma: no-cache × Cache-Control max-age 对照端点（pragma-no-cache-with-
/// cache-control.html 消费：public 臂期望不被缓存即两次体不同，immutable 臂
/// 期望缓存复用即两次体相同；响应体随机 token 即可判别）。
pub(crate) fn route_cached_pragma_py(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    _req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let cache_control = if clean.ends_with("cached_pragma_immutable_rand.py") {
        "max-age=2592000, immutable"
    } else {
        "max-age=2592000, public"
    };
    let headers: Vec<(String, String)> = vec![
        ("X-XSS-Protection".into(), "0".into()),
        ("Content-Type".into(), "text/html".into()),
        ("Cache-Control".into(), cache_control.into()),
        ("Pragma".into(), "no-cache".into()),
    ];
    let body = format!("Token: {}", next_pragma_token());
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// token 唯一性来源（SystemTime 纳秒 + 进程内计数器）——语料只比对两次响应
/// 体是否不同，无需 uuid4 形态本身。
fn next_pragma_token() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("zw-{}-{}", nanos, COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// 上游 NOTEHDRS——记入 state.response_headers 的响应头（304 条件验证与
/// etag/lm 语料比对源；键小写化比对）。
const NOTE_HDRS: [&str; 4] = ["content-type", "access-control-allow-origin", "last-modified", "etag"];
/// 上游 NOBODYSTATUS——204/304 响应体置空。
const NOBODY_STATUS: [u16; 2] = [204, 304];
/// 上游 LOCATIONHDRS——magic locations（值拼接 `request.url&target=`）。
const LOCATION_HDRS: [&str; 2] = ["location", "content-location"];
/// 上游 DATEHDRS——magic dates（整数值 → `now + delta` 的 IMF-fixdate）。
const DATE_HDRS: [&str; 3] = ["date", "expires", "last-modified"];

/// `resources/http-cache.py` 端点仿真——上游 wptserve handler 1:1 移植：
/// - `Access-Control-Allow-Credentials: true` 分发前统一在场；
/// - OPTIONS 先于 uuid 判定 → preflight（ACO=请求 origin 或 `*`、ACAM GET、
///   ACAH=ACRM 或 `*`、ACMA 86400，体 `Preflight request`）；
/// - 缺/空 uuid → 404 `UUID not found`（上游 falsy 判定）；
/// - `dispatch=state` → stash take 的 JSON（取走即清空，缺记录回 `null`）；
/// - `dispatch=test` → 状态机：take stash（缺省空序列）→ 解析 Test-Requests
///   头（base64 JSON；缺失/坏值 400）→ `requests[len(server_state)]` 取当前
///   config（falsy 404 `Config not found`）→ config.response_headers 逐项
///   magic 展开（LOCATIONHDRS 值拼 `request.url&target=`；DATEHDRS 整数值
///   转 `now+delta` IMF-fixdate）并记名 NOTEHDRS → 追加 state（now/
///   request_method/全部请求头小写化/记名响应头）→ ACO/CT 缺省补 `*`/
///   text/plain → `Server-Request-Count` → response_status 取状态（缺省
///   200 OK）→ expected_type 以 `validated` 结尾时按首条 state 的
///   last-modified/etag 与请求 If-Modified-Since/If-None-Match 全等生成
///   304，未命中改判 999 `304 Not Generated` → 体 = response_body 或
///   uuid（204/304 置空）；
/// - 其余 dispatch → 404 `Fallthrough`。
fn http_cache_py_fixture(
    ctx: &PyEndpointCtx,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Result<zero_engine::fetch_bridge::FetchResponse, String> {
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let get_param = |name: &str| wpt_query_value(query, name);
    let req_header = |name: &str| {
        req.headers
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };
    let mut headers: Vec<(String, String)> = vec![("Access-Control-Allow-Credentials".into(), "true".into())];

    if req.method.eq_ignore_ascii_case("OPTIONS") {
        headers.push((
            "Access-Control-Allow-Origin".into(),
            req_header("origin").unwrap_or("*").to_string(),
        ));
        headers.push(("Access-Control-Allow-Methods".into(), "GET".into()));
        headers.push((
            "Access-Control-Allow-Headers".into(),
            req_header("access-control-request-headers").unwrap_or("*").to_string(),
        ));
        headers.push(("Access-Control-Max-Age".into(), "86400".into()));
        return Ok(response(200, "OK", headers, "Preflight request"));
    }
    // 上游 falsy 判定——缺参/空值同路径。
    let Some(uuid) = get_param("uuid").filter(|uuid| !uuid.is_empty()) else {
        headers.push(("Content-Type".into(), "text/plain".into()));
        return Ok(response(404, "Not Found", headers, "UUID not found"));
    };
    match get_param("dispatch").as_deref() {
        Some("state") => {
            let taken = ctx
                .http_cache_py_stash
                .lock()
                .ok()
                .and_then(|mut all| all.remove(&uuid));
            let json = taken
                .map(|states| serde_json::to_string(&states).unwrap_or_else(|_| "[]".to_string()))
                .unwrap_or_else(|| "null".to_string());
            headers.push(("Content-Type".into(), "text/plain".into()));
            Ok(response(200, "OK", headers, &json))
        }
        Some("test") => http_cache_py_test_arm(ctx, req, &uuid, headers),
        _ => {
            headers.push(("Content-Type".into(), "text/plain".into()));
            Ok(response(404, "Not Found", headers, "Fallthrough"))
        }
    }
}

/// `dispatch=test` 状态机（上游 handle_test 1:1——流程注记见
/// [`http_cache_py_fixture`]）。
fn http_cache_py_test_arm(
    ctx: &PyEndpointCtx,
    req: &zero_engine::fetch_bridge::FetchRequest,
    uuid: &str,
    mut headers: Vec<(String, String)>,
) -> Result<zero_engine::fetch_bridge::FetchResponse, String> {
    // 上游 stash take（缺记录空序列）——查询键自清由 put-back 恢复。
    let mut server_state = ctx
        .http_cache_py_stash
        .lock()
        .ok()
        .and_then(|mut all| all.remove(uuid))
        .unwrap_or_default();
    let requests = match req
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("test-requests"))
        .map(|(_, value)| value.as_str())
    {
        Some(encoded) => match base64::engine::general_purpose::STANDARD.decode(encoded.as_bytes()) {
            Ok(bytes) => match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(value) => value,
                Err(_) => return Ok(bad_test_requests(headers)),
            },
            Err(_) => return Ok(bad_test_requests(headers)),
        },
        None => return Ok(bad_test_requests(headers)),
    };
    // 上游 `requests[len(server_state)]` + falsy 判定（缺位/null → 404）。
    let Some(config) = requests.get(server_state.len()).filter(|config| !config.is_null()) else {
        headers.push(("Content-Type".into(), "text/plain".into()));
        return Ok(response(404, "Not Found", headers, "Config not found"));
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default();
    let mut noted_headers: BTreeMap<String, String> = BTreeMap::new();
    if let Some(response_headers) = config.get("response_headers").and_then(|value| value.as_array()) {
        for header in response_headers {
            let Some(pair) = header.as_array() else {
                continue;
            };
            let Some((name, value)) = pair.first().and_then(|name| name.as_str()).zip(pair.get(1)) else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            let mut value_string = match value {
                serde_json::Value::String(value) => value.clone(),
                // 语料整数值只出现在 DATEHDRS（magic dates）；其余数字头无消费方。
                serde_json::Value::Number(number) => number.to_string(),
                _ => String::new(),
            };
            if LOCATION_HDRS.contains(&lower.as_str()) {
                value_string = if value_string.is_empty() {
                    req.url.clone()
                } else {
                    format!("{}&target={}", req.url, value_string)
                };
            } else if DATE_HDRS.contains(&lower.as_str())
                && let Some(delta) = value.as_i64()
            {
                value_string = http_date_from_unix(now as i64 + delta);
            }
            set_response_header(&mut headers, name, &value_string);
            if NOTE_HDRS.contains(&lower.as_str()) {
                noted_headers.insert(lower, value_string);
            }
        }
    }
    let request_headers: BTreeMap<String, String> = req
        .headers
        .iter()
        .map(|(name, value)| (name.to_ascii_lowercase(), value.clone()))
        .collect();
    // 上游先记 state 后补缺省头——记名判定先于 move 取出。
    let aco_noted = noted_headers.contains_key("access-control-allow-origin");
    let content_type_noted = noted_headers.contains_key("content-type");
    server_state.push(HttpCachePyState {
        now,
        request_method: req.method.clone(),
        request_headers,
        response_headers: noted_headers,
    });
    if let Ok(mut all) = ctx.http_cache_py_stash.lock() {
        all.insert(uuid.to_string(), server_state.clone());
    }

    if !aco_noted {
        headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
    }
    if !content_type_noted {
        headers.push(("Content-Type".into(), "text/plain".into()));
    }
    headers.push(("Server-Request-Count".into(), server_state.len().to_string()));

    let (mut code, mut phrase) = config
        .get("response_status")
        .and_then(|value| value.as_array())
        .and_then(|pair| match pair.as_slice() {
            [code, phrase] => Some((code.as_u64()? as u16, phrase.as_str()?.to_string())),
            _ => None,
        })
        .unwrap_or((200, "OK".to_string()));
    // expected_type 以 `validated` 结尾 → 按首条 state 记名头生成条件 304；
    // 对应验证器缺省或值不等 → 改判 999 `304 Not Generated`（上游逐字）。
    if config
        .get("expected_type")
        .and_then(|value| value.as_str())
        .is_some_and(|expected_type| expected_type.ends_with("validated"))
    {
        let reference = &server_state[0].response_headers;
        let req_header = |name: &str| {
            req.headers
                .iter()
                .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.as_str())
        };
        if let Some(previous_lm) = reference.get("last-modified")
            && req_header("if-modified-since") == Some(previous_lm.as_str())
        {
            code = 304;
        }
        if let Some(previous_etag) = reference.get("etag")
            && req_header("if-none-match") == Some(previous_etag.as_str())
        {
            code = 304;
        }
        if code != 304 {
            code = 999;
            phrase = "304 Not Generated".to_string();
        }
    }
    let body = if NOBODY_STATUS.contains(&code) {
        String::new()
    } else {
        config
            .get("response_body")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| uuid.to_string())
    };
    Ok(response(code, &phrase, headers, &body))
}

/// Test-Requests 头缺失/坏 base64/坏 JSON → 400（上游 bare except 同路径）。
fn bad_test_requests(mut headers: Vec<(String, String)>) -> zero_engine::fetch_bridge::FetchResponse {
    headers.push(("Content-Type".into(), "text/plain".into()));
    response(400, "Bad Request", headers, "No or bad Test-Requests request header")
}

/// 同名响应头覆盖（上游 `response.headers.set` 单头 last-wins 语义）。
fn set_response_header(headers: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(existing) = headers
        .iter_mut()
        .find(|(existing_name, _)| existing_name.eq_ignore_ascii_case(name))
    {
        existing.1 = value.to_string();
    } else {
        headers.push((name.to_string(), value.to_string()));
    }
}

fn response(
    status: u16,
    status_text: &str,
    headers: Vec<(String, String)>,
    body: &str,
) -> zero_engine::fetch_bridge::FetchResponse {
    zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: status_text.to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }
}

/// RFC 9110 §1.2 IMF-fixdate 生成（上游 http_date——`Wk, DD Mon YYYY
/// HH:MM:SS GMT`）；magic dates 的 `now + delta` 折算由调用方完成。
fn http_date_from_unix(unix_secs: i64) -> String {
    const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = unix_secs.div_euclid(86_400);
    let secs_of_day = unix_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    // 1970-01-01 = Thursday（索引 3）。
    let weekday = (days + 3).rem_euclid(7) as usize;
    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        WEEKDAYS[weekday],
        day,
        MONTHS[(month - 1) as usize],
        year,
        secs_of_day / 3_600,
        secs_of_day % 3_600 / 60,
        secs_of_day % 60,
    )
}

/// 天序号（1970-01-01 = 0）→ 格里历 `(year, month, day)`——Howard Hinnant
/// civil_from_days 算法（纪元整除展开，跨负值安全）。
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_date_epoch_and_known_dates() {
        assert_eq!(http_date_from_unix(0), "Thu, 01 Jan 1970 00:00:00 GMT");
        // disk_cache 测试同款锚点（Wed, 21 Oct 2015 07:28:00 GMT = 1445412480）。
        assert_eq!(http_date_from_unix(1_445_412_480), "Wed, 21 Oct 2015 07:28:00 GMT");
        // cache_policy 测试同款锚点（Tue, 02 Jan 2024 00:00:00 GMT = 1704153600）。
        assert_eq!(http_date_from_unix(1_704_153_600), "Tue, 02 Jan 2024 00:00:00 GMT");
        // 闰日（Sat, 29 Feb 2020 12:00:00 GMT = 1582977600）。
        assert_eq!(http_date_from_unix(1_582_977_600), "Sat, 29 Feb 2020 12:00:00 GMT");
    }

    #[test]
    fn http_date_negative_secs_pre_epoch() {
        // 1969-12-31 23:59:59 GMT = -1。
        assert_eq!(http_date_from_unix(-1), "Wed, 31 Dec 1969 23:59:59 GMT");
    }

    #[test]
    fn http_date_matches_zero_net_parser_roundtrip() {
        // 与 zero-net 解析侧（cookie::parse_expires_date）互验——生成格式必须
        // 被本仓缓存解析路径接受。
        let stamp = http_date_from_unix(1_764_602_400);
        let parsed = zero_net::cookie::parse_expires_date(&stamp).expect("generated date must parse");
        assert_eq!(parsed, 1_764_602_400);
    }
}
