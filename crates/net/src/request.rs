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
}
