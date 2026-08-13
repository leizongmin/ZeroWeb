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

    // ── R3372：缓存键安全/确定性属性锁定 ──

    #[test]
    /// R3372：URL 中的字面 `\0` 经 `url` crate percent-encode 为 `%00`，
    /// 故 `cache_lookup_key` 用 `\0` 作 vary 后缀分隔符不会与 base 冲突（无键注入）。
    fn url_nul_is_percent_encoded_key_separator_stays_unique_r3372() {
        let stripped = strip_url_fragment("https://x.com/a\0b");
        assert_eq!(stripped, "https://x.com/a%00b", "字面 NUL 必须 percent-encode");
        assert!(!stripped.contains('\0'), "规范化后 URL 不得含裸 \\0");
        // 键分隔符 \0 仅出现在 vary 后缀边界，不在 base 中
        let key = cache_lookup_key("https://x.com/a\0b", &[], None);
        assert_eq!(key, "https://x.com/a%00b");
    }

    #[test]
    /// R3372：store 与 lookup 用同一 `cache_lookup_key`，故 store 键 == lookup 键
    /// （防止存/查不对称导致永远 miss 或错误命中——确定性）。
    fn store_key_equals_lookup_key_r3372() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Vary".into(), "Accept-Language".into())],
            body: vec![],
            url: "https://example.com/".into(),
            redirect_count: 0,
        };
        let req = vec![("Accept-Language".into(), "en-US".into())];
        let store = cache_store_key("https://example.com/page#frag", &req, &resp);
        let lookup = cache_lookup_key("https://example.com/page", &req, Some("Accept-Language"));
        assert_eq!(store, lookup, "store 键须与 lookup 键一致（fragment 已剥离）");
    }

    #[test]
    /// R3372：Vary 头大小写不一致不改变「请求头值匹配」语义——header_value 用
    /// eq_ignore_ascii_case 查找请求头。Vary 字段名直接嵌入键（保留原始大小写），
    /// 但同语义请求头值仍被正确取到。
    fn vary_field_case_insensitive_request_header_lookup_r3372() {
        // Vary 写小写 accept-encoding，请求头写 Accept-Encoding → 值仍取到
        let req = vec![("Accept-Encoding".into(), "br".into())];
        let k_lower = cache_lookup_key("https://x.com/", &req, Some("accept-encoding"));
        let k_mixed = cache_lookup_key("https://x.com/", &req, Some("AcCePt-EnCoDiNg"));
        // 三者均取到请求头值 br（仅 Vary 字段名大小写在键里不同）
        assert!(k_lower.contains("=br"), "小写 Vary 字段应取到请求头值：{k_lower}");
        assert!(k_mixed.contains("=br"), "混合大小写 Vary 字段应取到请求头值：{k_mixed}");
    }

    #[test]
    /// R3372：Vary 请求头缺失 → 值为空串（unwrap_or_default），键仍含字段名标记；
    /// 纯空白/逗号 Vary → 退化为无 vary 后缀（等同 base）。
    fn vary_missing_header_and_empty_vary_r3372() {
        // 请求未带 Accept-Encoding → 值空
        let k = cache_lookup_key("https://x.com/", &[], Some("Accept-Encoding"));
        assert_eq!(k, "https://x.com/\0vary=Accept-Encoding=");
        // 纯空白 Vary → 无后缀
        assert_eq!(cache_lookup_key("https://x.com/", &[], Some("   ")), "https://x.com/");
        // 逗号 + 空白字段 → 全跳过 → 无后缀
        assert_eq!(cache_lookup_key("https://x.com/", &[], Some(" , , ")), "https://x.com/");
    }
}
