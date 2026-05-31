//! HTTP 请求与响应类型定义。
//!
//! 提供请求方法、请求体、响应体等类型。

use crate::NetError;

/// HTTP 请求方法。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET 请求。
    Get,
    /// POST 请求。
    Post,
    /// PUT 请求。
    Put,
    /// DELETE 请求。
    Delete,
    /// PATCH 请求。
    Patch,
    /// HEAD 请求。
    Head,
    /// OPTIONS 请求。
    Options,
}

impl HttpMethod {
    /// 转换为 reqwest Method。
    pub(crate) fn to_reqwest(&self) -> reqwest::Method {
        match self {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }
}

/// HTTP 请求。
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// 请求方法。
    pub method: HttpMethod,
    /// 请求 URL。
    pub url: String,
    /// 请求头。
    pub headers: Vec<(String, String)>,
    /// 请求体。
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// 创建 GET 请求。
    pub fn get(url: &str) -> Self {
        Self {
            method: HttpMethod::Get,
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// 创建 POST 请求。
    pub fn post(url: &str, body: Vec<u8>) -> Self {
        Self {
            method: HttpMethod::Post,
            url: url.to_string(),
            headers: Vec::new(),
            body: Some(body),
        }
    }

    /// 添加请求头。
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    /// 修改请求方法（builder 风格）。
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }
}

/// HTTP 响应。
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP 状态码。
    pub status_code: u16,
    /// 响应头。
    pub headers: Vec<(String, String)>,
    /// 响应体。
    pub body: Vec<u8>,
    /// 最终 URL（重定向后的 URL）。
    pub url: String,
    /// 经历的重定向次数。
    pub redirect_count: usize,
}

impl HttpResponse {
    /// 是否为成功状态码 (2xx)。
    pub fn is_success(&self) -> bool {
        (200..=299).contains(&self.status_code)
    }

    /// 是否为重定向 (3xx)。
    pub fn is_redirect(&self) -> bool {
        (300..=399).contains(&self.status_code)
    }

    /// 是否为客户端错误 (4xx)。
    pub fn is_client_error(&self) -> bool {
        (400..=499).contains(&self.status_code)
    }

    /// 是否为服务端错误 (5xx)。
    pub fn is_server_error(&self) -> bool {
        (500..=599).contains(&self.status_code)
    }

    /// 获取 Content-Type header。
    pub fn content_type(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| value.as_str())
    }

    /// 获取 Content-Type 中的 MIME 类型（不含参数）。
    ///
    /// 例如 "text/html; charset=utf-8" 返回 "text/html"。
    pub fn content_type_mime(&self) -> Option<&str> {
        self.content_type().map(|ct| {
            match ct.find(';') {
                Some(idx) => &ct[..idx],
                None => ct,
            }
            .trim()
        })
    }

    /// 获取 body 为 UTF-8 字符串。
    pub fn text(&self) -> Result<String, NetError> {
        String::from_utf8(self.body.clone()).map_err(|e| NetError::Http(e.to_string()))
    }

    /// 查找指定名称的响应头（大小写不敏感）。
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用 helper：创建一个最小 HttpResponse。
    fn test_response(status_code: u16, body: Vec<u8>, url: &str) -> HttpResponse {
        HttpResponse {
            status_code,
            headers: vec![],
            body,
            url: url.to_string(),
            redirect_count: 0,
        }
    }

    #[test]
    fn test_http_response_is_success() {
        let resp = test_response(200, vec![], "http://example.com");
        assert!(resp.is_success());

        let resp_404 = test_response(404, vec![], "http://example.com");
        assert!(!resp_404.is_success());
    }

    #[test]
    fn test_http_response_is_redirect() {
        let resp = test_response(301, vec![], "http://example.com");
        assert!(resp.is_redirect());

        let resp_200 = test_response(200, vec![], "http://example.com");
        assert!(!resp_200.is_redirect());
    }

    #[test]
    fn test_http_response_text() {
        let resp = test_response(200, b"Hello, World!".to_vec(), "http://example.com");
        assert_eq!(resp.text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_http_request_get_constructor() {
        let req = HttpRequest::get("https://example.com/api");
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.url, "https://example.com/api");
        assert!(req.body.is_none());
        assert!(req.headers.is_empty());
    }

    #[test]
    fn test_http_request_post_constructor() {
        let req = HttpRequest::post("https://example.com/api", b"data".to_vec());
        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.body, Some(b"data".to_vec()));
    }

    #[test]
    fn test_http_request_header_builder() {
        let req = HttpRequest::get("https://example.com")
            .header("Accept", "text/html")
            .header("X-Custom", "value");
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0], ("Accept".into(), "text/html".into()));
    }

    #[test]
    fn test_http_response_is_success_boundaries() {
        assert!(!test_response(199, vec![], "").is_success());
        assert!(test_response(200, vec![], "").is_success());
        assert!(test_response(299, vec![], "").is_success());
        assert!(!test_response(300, vec![], "").is_success());
    }

    #[test]
    fn test_http_response_is_redirect_boundaries() {
        assert!(!test_response(299, vec![], "").is_redirect());
        assert!(test_response(300, vec![], "").is_redirect());
        assert!(test_response(399, vec![], "").is_redirect());
        assert!(!test_response(400, vec![], "").is_redirect());
    }

    #[test]
    fn test_http_response_content_type_found() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.content_type(), Some("text/html; charset=utf-8"));
    }

    #[test]
    fn test_http_response_content_type_missing() {
        assert!(test_response(200, vec![], "").content_type().is_none());
    }

    #[test]
    fn test_http_response_text_invalid_utf8() {
        let resp = test_response(200, vec![0xFF, 0xFE], "");
        assert!(resp.text().is_err());
    }

    // ── Content-Type MIME 解析测试 ──

    #[test]
    fn test_content_type_mime_with_params() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.content_type_mime(), Some("text/html"));
    }

    #[test]
    fn test_content_type_mime_without_params() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.content_type_mime(), Some("application/json"));
    }

    #[test]
    fn test_content_type_mime_missing() {
        let resp = test_response(200, vec![], "");
        assert!(resp.content_type_mime().is_none());
    }

    // ── header() 查找方法测试 ──

    #[test]
    fn test_header_lookup_found() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Content-Type".into(), "text/plain".into()),
                ("X-Custom".into(), "value".into()),
            ],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.header("x-custom"), Some("value"));
        assert_eq!(resp.header("Content-Type"), Some("text/plain"));
    }

    #[test]
    fn test_header_lookup_missing() {
        let resp = test_response(200, vec![], "");
        assert!(resp.header("X-Not-Exist").is_none());
    }

    // ── redirect_count 字段测试 ──

    #[test]
    fn test_redirect_count_default() {
        let resp = test_response(200, vec![], "http://example.com");
        assert_eq!(resp.redirect_count, 0);
    }

    #[test]
    fn test_redirect_count_with_redirects() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: "http://example.com/final".to_string(),
            redirect_count: 3,
        };
        assert_eq!(resp.redirect_count, 3);
    }

    // ── HTTP header handling tests ──

    /// 验证多个同名 header 只返回第一个（当前行为）。
    #[test]
    fn test_header_multiple_same_name() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Set-Cookie".into(), "a=1".into()), ("Set-Cookie".into(), "b=2".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        // header() 使用 find，返回第一个匹配
        let val = resp.header("set-cookie");
        assert_eq!(val, Some("a=1"));
    }

    /// 验证 header 查找是大小写不敏感的。
    #[test]
    fn test_header_case_insensitive() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.header("content-type"), Some("text/html"));
        assert_eq!(resp.header("CONTENT-TYPE"), Some("text/html"));
        assert_eq!(resp.header("Content-Type"), Some("text/html"));
    }

    /// 验证自定义 header 通过 header() 可查询。
    #[test]
    fn test_custom_header_accessors() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![
                ("X-Request-Id".into(), "abc-123".into()),
                ("X-Rate-Limit".into(), "100".into()),
            ],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.header("x-request-id"), Some("abc-123"));
        assert_eq!(resp.header("x-rate-limit"), Some("100"));
    }

    /// 验证 Content-Type 不含参数时 content_type_mime 等于 content_type。
    #[test]
    fn test_content_type_mime_equals_content_type_when_no_params() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.content_type(), resp.content_type_mime());
    }

    /// 验证 content_type_mime 对多种 MIME 类型正确提取。
    #[test]
    fn test_content_type_mime_various_types() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html; charset=iso-8859-1".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp.content_type_mime(), Some("text/html"));

        let resp2 = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "multipart/form-data; boundary=----abc".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert_eq!(resp2.content_type_mime(), Some("multipart/form-data"));
    }

    // ── Response status code helper tests ──

    /// 验证常见 HTTP 错误状态码不是 success。
    #[test]
    fn test_status_code_helpers_various_codes() {
        // 2xx = success
        assert!(test_response(200, vec![], "").is_success());
        assert!(test_response(201, vec![], "").is_success());
        assert!(test_response(204, vec![], "").is_success());
        assert!(!test_response(204, vec![], "").is_redirect());

        // 3xx = redirect
        assert!(test_response(301, vec![], "").is_redirect());
        assert!(test_response(302, vec![], "").is_redirect());
        assert!(test_response(304, vec![], "").is_redirect());
        assert!(!test_response(301, vec![], "").is_success());

        // 4xx/5xx = neither success nor redirect
        assert!(!test_response(400, vec![], "").is_success());
        assert!(!test_response(400, vec![], "").is_redirect());
        assert!(!test_response(500, vec![], "").is_success());
        assert!(!test_response(503, vec![], "").is_redirect());
    }

    /// 验证 HttpRequest 方法类型正确映射。
    #[test]
    fn test_http_method_variants() {
        let methods = [
            (HttpMethod::Get, "GET"),
            (HttpMethod::Post, "POST"),
            (HttpMethod::Put, "PUT"),
            (HttpMethod::Delete, "DELETE"),
            (HttpMethod::Patch, "PATCH"),
            (HttpMethod::Head, "HEAD"),
            (HttpMethod::Options, "OPTIONS"),
        ];
        for (method, expected) in methods {
            assert_eq!(method.to_reqwest().as_str(), expected);
        }
    }

    /// 验证多个 header 可通过 builder 链式添加。
    #[test]
    fn test_request_multiple_headers_builder() {
        let req = HttpRequest::get("http://example.com")
            .header("Accept", "text/html")
            .header("Accept-Language", "en-US")
            .header("Authorization", "Bearer token123");
        assert_eq!(req.headers.len(), 3);

        let auth = req.headers.iter().find(|(k, _)| k == "Authorization");
        assert!(auth.is_some());
        assert_eq!(auth.unwrap().1, "Bearer token123");
    }

    // ── 新增边界条件测试 ──

    /// 测试 HttpResponse::text() 对无效 UTF-8 字节序列返回错误。
    #[test]
    fn test_http_response_text_non_utf8() {
        // 0xFF 0xFE 不是合法 UTF-8 序列
        let resp = test_response(200, vec![0xFF, 0xFE, 0xFD], "http://example.com");
        let result = resp.text();
        assert!(result.is_err(), "无效 UTF-8 body 应返回错误");
    }

    /// 测试 HttpRequest 通过 builder 链式调用添加多个 header，保持顺序和内容正确。
    #[test]
    fn test_http_request_header_chaining() {
        let req = HttpRequest::get("http://example.com/api")
            .header("Content-Type", "application/json")
            .header("X-Request-Id", "abc-123")
            .header("Cache-Control", "no-cache")
            .header("Authorization", "Bearer tok");
        assert_eq!(req.headers.len(), 4, "应通过链式调用添加 4 个 header");
        // 验证顺序与内容
        assert_eq!(req.headers[0], ("Content-Type".into(), "application/json".into()));
        assert_eq!(req.headers[1], ("X-Request-Id".into(), "abc-123".into()));
        assert_eq!(req.headers[2], ("Cache-Control".into(), "no-cache".into()));
        assert_eq!(req.headers[3], ("Authorization".into(), "Bearer tok".into()));
    }

    /// 测试 HttpRequest 通过 builder 方法链从 GET 切换为 POST，验证方法变更和 body 设定。
    ///
    /// 验证 with_method 可以改变请求方法，且 builder 链式调用保持 header 和 body 正确。
    #[test]
    fn test_http_request_builder_method_chaining() {
        // 从 GET 请求开始
        let req = HttpRequest::get("http://example.com/api")
            .header("Accept", "application/json")
            .with_method(HttpMethod::Post)
            .header("Content-Type", "application/json");

        // 方法已从 GET 变为 POST
        assert_eq!(req.method, HttpMethod::Post);
        // URL 不变
        assert_eq!(req.url, "http://example.com/api");
        // header 按添加顺序保留
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0], ("Accept".into(), "application/json".into()));
        assert_eq!(req.headers[1], ("Content-Type".into(), "application/json".into()));
        // GET 构造器不设 body
        assert!(req.body.is_none());

        // 也可以从 POST 开始切换为 PUT
        let req_put = HttpRequest::post("http://example.com/api", b"data".to_vec()).with_method(HttpMethod::Put);
        assert_eq!(req_put.method, HttpMethod::Put);
        assert_eq!(req_put.body, Some(b"data".to_vec()));
    }

    /// 测试 HttpResponse 状态码分类：2xx/3xx/4xx/5xx 各类别的判断方法。
    ///
    /// 验证 is_success、is_redirect、is_client_error、is_server_error
    /// 在各类状态码上的正确性。
    #[test]
    fn test_http_response_status_code_categories() {
        // 2xx — 成功
        assert!(test_response(200, vec![], "").is_success());
        assert!(test_response(201, vec![], "").is_success());
        assert!(test_response(204, vec![], "").is_success());
        assert!(!test_response(200, vec![], "").is_redirect());
        assert!(!test_response(200, vec![], "").is_client_error());
        assert!(!test_response(200, vec![], "").is_server_error());

        // 3xx — 重定向
        assert!(test_response(301, vec![], "").is_redirect());
        assert!(test_response(302, vec![], "").is_redirect());
        assert!(test_response(304, vec![], "").is_redirect());
        assert!(!test_response(301, vec![], "").is_success());
        assert!(!test_response(301, vec![], "").is_client_error());
        assert!(!test_response(301, vec![], "").is_server_error());

        // 4xx — 客户端错误
        assert!(test_response(400, vec![], "").is_client_error());
        assert!(test_response(401, vec![], "").is_client_error());
        assert!(test_response(403, vec![], "").is_client_error());
        assert!(test_response(404, vec![], "").is_client_error());
        assert!(!test_response(404, vec![], "").is_success());
        assert!(!test_response(404, vec![], "").is_redirect());
        assert!(!test_response(404, vec![], "").is_server_error());

        // 5xx — 服务端错误
        assert!(test_response(500, vec![], "").is_server_error());
        assert!(test_response(502, vec![], "").is_server_error());
        assert!(test_response(503, vec![], "").is_server_error());
        assert!(!test_response(500, vec![], "").is_success());
        assert!(!test_response(500, vec![], "").is_redirect());
        assert!(!test_response(500, vec![], "").is_client_error());

        // 边界值验证
        assert!(!test_response(199, vec![], "").is_success());
        assert!(!test_response(300, vec![], "").is_success());
        assert!(test_response(299, vec![], "").is_success());
        assert!(test_response(399, vec![], "").is_redirect());
        assert!(!test_response(400, vec![], "").is_redirect());
        assert!(!test_response(399, vec![], "").is_client_error());
        assert!(test_response(400, vec![], "").is_client_error());
        assert!(!test_response(499, vec![], "").is_server_error());
        assert!(test_response(500, vec![], "").is_server_error());
        assert!(test_response(599, vec![], "").is_server_error());
        assert!(!test_response(600, vec![], "").is_server_error());
    }
}
