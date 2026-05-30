//! CORS（跨源资源共享）模块。
//!
//! 提供 CORS 策略检查和简单请求判断功能。

use crate::origin::Origin;

/// CORS 请求类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorsRequestType {
    /// 简单请求（GET/HEAD/POST with simple content types）。
    Simple,
    /// 预检请求（OPTIONS preflight）。
    Preflight,
}

/// CORS 策略配置。
#[derive(Debug, Clone)]
pub struct CorsPolicy {
    /// 允许的源列表，`*` 表示允许所有。
    pub allow_origins: Vec<String>,
    /// 允许的 HTTP 方法。
    pub allow_methods: Vec<String>,
    /// 允许的请求头。
    pub allow_headers: Vec<String>,
    /// 是否允许携带凭证。
    pub allow_credentials: bool,
    /// 预检缓存时间（秒）。
    pub max_age: Option<u32>,
}

impl Default for CorsPolicy {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        }
    }
}

/// CORS 检查结果。
#[derive(Debug, Clone)]
pub struct CorsResult {
    /// 是否允许。
    pub allowed: bool,
    /// 原因说明。
    pub reason: String,
}

/// 检查 CORS 请求是否允许。
pub fn check_cors(
    policy: &CorsPolicy,
    request_origin: &Origin,
    request_method: &str,
    request_headers: &[(String, String)],
) -> CorsResult {
    // 检查 origin
    let origin_allowed = if policy.allow_origins.iter().any(|o| o == "*") {
        // 通配符允许所有源，但如果 allow_credentials 为 true，不能使用通配符
        if policy.allow_credentials {
            return CorsResult {
                allowed: false,
                reason: "credentials with wildcard origin not allowed".to_string(),
            };
        }
        true
    } else {
        let origin_str = format!(
            "{}://{}",
            request_origin.scheme,
            if request_origin.port == 80 && request_origin.scheme == "http"
                || request_origin.port == 443 && request_origin.scheme == "https"
            {
                request_origin.host.clone()
            } else {
                format!("{}:{}", request_origin.host, request_origin.port)
            }
        );
        policy
            .allow_origins
            .iter()
            .any(|o| o.eq_ignore_ascii_case(&origin_str))
    };

    if !origin_allowed {
        return CorsResult {
            allowed: false,
            reason: "origin not in allow list".to_string(),
        };
    }

    // 检查方法
    let method_allowed = policy
        .allow_methods
        .iter()
        .any(|m| m.eq_ignore_ascii_case(request_method));
    if !method_allowed {
        return CorsResult {
            allowed: false,
            reason: format!("method {request_method} not allowed"),
        };
    }

    // 检查请求头
    let simple_headers = [
        "accept",
        "accept-language",
        "content-language",
        "content-type",
    ];
    let simple_content_types = [
        "application/x-www-form-urlencoded",
        "multipart/form-data",
        "text/plain",
    ];

    for (name, value) in request_headers {
        let name_lower = name.to_ascii_lowercase();
        if simple_headers.contains(&name_lower.as_str()) {
            // Content-Type 需要额外检查是否为简单类型
            if name_lower == "content-type" {
                let ct_lower = value.to_ascii_lowercase();
                let ct_main = ct_lower.split(';').next().unwrap_or("").trim();
                if !simple_content_types.contains(&ct_main) {
                    // 非简单 Content-Type 需要在 allow_headers 中
                    if !policy
                        .allow_headers
                        .iter()
                        .any(|h| h.eq_ignore_ascii_case(name))
                    {
                        return CorsResult {
                            allowed: false,
                            reason: format!("header {name} not allowed"),
                        };
                    }
                }
            }
            continue;
        }

        // 非简单头必须在 allow_headers 中
        if !policy
            .allow_headers
            .iter()
            .any(|h| h.eq_ignore_ascii_case(name))
        {
            return CorsResult {
                allowed: false,
                reason: format!("header {name} not allowed"),
            };
        }
    }

    CorsResult {
        allowed: true,
        reason: "allowed".to_string(),
    }
}

/// 判断是否为简单请求（不需要预检）。
pub fn is_simple_request(
    method: &str,
    content_type: Option<&str>,
    headers: &[(String, String)],
) -> bool {
    // 简单方法
    let simple_methods = ["GET", "HEAD", "POST"];
    if !simple_methods
        .iter()
        .any(|m| m.eq_ignore_ascii_case(method))
    {
        return false;
    }

    // 检查 Content-Type
    if let Some(ct) = content_type {
        let ct_lower = ct.to_ascii_lowercase();
        let ct_main = ct_lower.split(';').next().unwrap_or("").trim();
        let simple_content_types = [
            "application/x-www-form-urlencoded",
            "multipart/form-data",
            "text/plain",
        ];
        if !simple_content_types.contains(&ct_main) {
            return false;
        }
    }

    // 检查是否有非简单头
    let simple_headers = [
        "accept",
        "accept-language",
        "content-language",
        "content-type",
    ];
    for (name, _) in headers {
        let name_lower = name.to_ascii_lowercase();
        if !simple_headers.contains(&name_lower.as_str()) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_wildcard_allows_all() {
        let policy = CorsPolicy::default();
        let origin = Origin::parse("http://evil.com").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_specific_origin_allowed() {
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_specific_origin_blocked() {
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://evil.com").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(!result.allowed);
    }

    #[test]
    fn test_cors_allow_methods() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();

        let get_result = check_cors(&policy, &origin, "GET", &[]);
        assert!(get_result.allowed);

        let put_result = check_cors(&policy, &origin, "PUT", &[]);
        assert!(!put_result.allowed);
    }

    #[test]
    fn test_cors_allow_credentials() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: true,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        // Wildcard with credentials should be rejected
        assert!(!result.allowed);
    }

    #[test]
    fn test_is_simple_request_get() {
        assert!(is_simple_request("GET", None, &[]));
    }

    #[test]
    fn test_is_simple_request_post() {
        assert!(is_simple_request(
            "POST",
            Some("application/x-www-form-urlencoded"),
            &[]
        ));
        assert!(is_simple_request("POST", Some("text/plain"), &[]));
    }

    #[test]
    fn test_is_not_simple_request_put() {
        assert!(!is_simple_request("PUT", None, &[]));
    }

    #[test]
    fn test_cors_policy_default_values() {
        let policy = CorsPolicy::default();
        assert_eq!(policy.allow_origins, vec!["*"]);
        assert!(policy.allow_methods.contains(&"GET".to_string()));
        assert!(policy.allow_headers.is_empty());
        assert!(!policy.allow_credentials);
        assert!(policy.max_age.is_none());
    }

    #[test]
    fn test_cors_custom_port_origin_matching() {
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com:3000".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com:3000").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_port_80_origin_formatting() {
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        assert_eq!(origin.port, 80);
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_port_443_origin_formatting() {
        let policy = CorsPolicy {
            allow_origins: vec!["https://example.com".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(origin.port, 443);
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_non_simple_header_blocked() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(
            &policy,
            &origin,
            "GET",
            &[("X-Custom".to_string(), "value".to_string())],
        );
        assert!(!result.allowed);
        assert!(result.reason.contains("X-Custom"));
    }

    #[test]
    fn test_cors_non_simple_header_allowed() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec!["X-Custom".to_string()],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(
            &policy,
            &origin,
            "GET",
            &[("X-Custom".to_string(), "value".to_string())],
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_non_simple_content_type_blocked() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(
            &policy,
            &origin,
            "POST",
            &[("Content-Type".to_string(), "application/json".to_string())],
        );
        assert!(!result.allowed);
    }

    #[test]
    fn test_cors_content_type_with_charset_param() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(
            &policy,
            &origin,
            "POST",
            &[(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )],
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_case_insensitive_method_matching() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(&policy, &origin, "get", &[]);
        assert!(result.allowed);
    }

    #[test]
    fn test_cors_empty_allow_origins_blocks_all() {
        let policy = CorsPolicy {
            allow_origins: vec![],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(!result.allowed);
    }

    #[test]
    fn test_is_simple_request_head() {
        assert!(is_simple_request("HEAD", None, &[]));
    }

    #[test]
    fn test_is_simple_request_with_custom_header() {
        assert!(!is_simple_request(
            "GET",
            None,
            &[("X-Custom".to_string(), "val".to_string())]
        ));
    }
}
