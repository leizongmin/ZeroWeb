//! HTTP 缓存键 — URL 规范化与 Vary 维度。

use crate::request::HttpResponse;

/// 构造缓存查找/存储键（去 fragment，含 Vary 相关请求维度）。
pub fn cache_lookup_key(url: &str, request_headers: &[(String, String)]) -> String {
    let base = strip_fragment(url);
    match vary_suffix(request_headers) {
        Some(v) => format!("{base}\0vary={v}"),
        None => base,
    }
}

/// 根据响应 `Vary` 头与本次请求头生成 vary 后缀（存储时写入条目）。
pub fn vary_suffix_for_store(response: &HttpResponse, request_headers: &[(String, String)]) -> Option<String> {
    let vary = response.header("vary")?;
    vary_suffix_from_fields(vary, request_headers)
}

fn strip_fragment(url: &str) -> String {
    url::Url::parse(url)
        .map(|mut u| {
            u.set_fragment(None);
            u.to_string()
        })
        .unwrap_or_else(|_| url.to_string())
}

fn vary_suffix(request_headers: &[(String, String)]) -> Option<String> {
    // 与 reqwest 默认行为对齐：无显式头时也按 gzip 族区分（常见 Vary: Accept-Encoding）。
    Some(format!(
        "Accept-Encoding={}",
        header_value(request_headers, "accept-encoding").unwrap_or_else(|| "gzip, deflate, br".to_string())
    ))
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
    fn strips_fragment() {
        assert_eq!(
            cache_lookup_key("https://example.com/a#frag", &[]),
            "https://example.com/a\u{0}vary=Accept-Encoding=gzip, deflate, br"
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
    }
}
