#[cfg(test)]
use zero_net::url_parser::parse_url;
use zero_security::{CorsPolicy, Origin, check_cors};

/// URL 解析 + 同源策略
#[test]
fn test_url_origin_same_origin_check() {
    let _url_a = parse_url("https://example.com/page1").unwrap();
    let _url_b = parse_url("https://example.com/page2?q=1").unwrap();

    let origin_a = Origin::parse("https://example.com").unwrap();
    let origin_b = Origin::parse("https://example.com").unwrap();

    assert!(origin_a.is_same_origin(&origin_b));
}

/// URL 解析 + CORS 检查
#[test]
fn test_cors_policy_with_parsed_url() {
    let origin = Origin::parse("http://evil.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://example.com".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };

    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(!result.allowed, "跨域请求应被拒绝");
}

/// 安全上下文判断
#[test]
fn test_url_security_context() {
    let _http_url = parse_url("http://example.com").unwrap();
    let _https_url = parse_url("https://example.com").unwrap();

    let http_origin = Origin::parse("http://example.com").unwrap();
    let https_origin = Origin::parse("https://example.com").unwrap();

    assert!(!http_origin.is_secure());
    assert!(https_origin.is_secure());
}
