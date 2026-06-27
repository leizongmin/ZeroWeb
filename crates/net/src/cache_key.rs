//! HTTP 缓存键 — URL 规范化与 Vary 维度。

use crate::request::HttpResponse;

/// 去掉 URL fragment。
pub fn strip_url_fragment(url: &str) -> String {
    url::Url::parse(url)
        .map(|mut u| {
            u.set_fragment(None);
            u.to_string()
        })
        .unwrap_or_else(|_| url.to_string())
}

/// 构造缓存查找键（去 fragment；`vary_header` 来自已存储条目的响应 `Vary`）。
pub fn cache_lookup_key(url: &str, request_headers: &[(String, String)], vary_header: Option<&str>) -> String {
    let base = strip_url_fragment(url);
    match vary_header.filter(|v| !v.trim().is_empty()) {
        Some(vary) => match vary_suffix_from_fields(vary, request_headers) {
            Some(v) if !v.is_empty() => format!("{base}\0vary={v}"),
            _ => base,
        },
        None => base,
    }
}

/// 构造缓存存储键（根据响应 `Vary` 与本次请求头）。
pub fn cache_store_key(url: &str, request_headers: &[(String, String)], response: &HttpResponse) -> String {
    cache_lookup_key(url, request_headers, response.header("vary"))
}

/// 根据响应 `Vary` 头与本次请求头生成 vary 后缀（存储时写入条目）。
pub fn vary_suffix_for_store(response: &HttpResponse, request_headers: &[(String, String)]) -> Option<String> {
    let vary = response.header("vary")?;
    vary_suffix_from_fields(vary, request_headers)
}

fn vary_suffix_from_fields(vary: &str, request_headers: &[(String, String)]) -> Option<String> {
    let mut parts = Vec::new();
    for field in vary.split(',') {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        let value = header_value(request_headers, field).unwrap_or_default();
        parts.push(format!("{field}={value}"));
    }
    if parts.is_empty() { None } else { Some(parts.join(";")) }
}

fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fragment_without_vary() {
        assert_eq!(
            cache_lookup_key("https://example.com/a#frag", &[], None),
            "https://example.com/a"
        );
    }

    #[test]
    fn vary_accept_encoding_key() {
        let req = vec![("Accept-Encoding".into(), "gzip".into())];
        assert_eq!(
            cache_lookup_key("https://example.com/", &req, Some("Accept-Encoding")),
            "https://example.com/\0vary=Accept-Encoding=gzip"
        );
    }

    #[test]
    fn vary_multi_field_key() {
        let req = vec![
            ("Accept-Encoding".into(), "gzip".into()),
            ("Accept-Language".into(), "zh-CN".into()),
        ];
        assert_eq!(
            cache_lookup_key("https://example.com/", &req, Some("Accept-Encoding, Accept-Language")),
            "https://example.com/\0vary=Accept-Encoding=gzip;Accept-Language=zh-CN"
        );
    }

    #[test]
    fn vary_suffix_from_response() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Vary".into(), "Accept-Encoding".into())],
            body: vec![],
            url: "https://example.com/".into(),
            redirect_count: 0,
        };
        let req = vec![("Accept-Encoding".into(), "gzip".into())];
        assert_eq!(
            vary_suffix_for_store(&resp, &req),
            Some("Accept-Encoding=gzip".to_string())
        );
        assert_eq!(
            cache_store_key("https://example.com/", &req, &resp),
            "https://example.com/\0vary=Accept-Encoding=gzip"
        );
    }
}
