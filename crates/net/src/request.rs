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

    /// 获取 body 为 UTF-8 字符串。
    pub fn text(&self) -> Result<String, NetError> {
        String::from_utf8(self.body.clone()).map_err(|e| NetError::Http(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_is_success() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: "http://example.com".to_string(),
        };
        assert!(resp.is_success());

        let resp_404 = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
            url: "http://example.com".to_string(),
        };
        assert!(!resp_404.is_success());
    }

    #[test]
    fn test_http_response_is_redirect() {
        let resp = HttpResponse {
            status_code: 301,
            headers: vec![],
            body: vec![],
            url: "http://example.com".to_string(),
        };
        assert!(resp.is_redirect());

        let resp_200 = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: "http://example.com".to_string(),
        };
        assert!(!resp_200.is_redirect());
    }

    #[test]
    fn test_http_response_text() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: b"Hello, World!".to_vec(),
            url: "http://example.com".to_string(),
        };
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
        let r199 = HttpResponse {
            status_code: 199,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r200 = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r299 = HttpResponse {
            status_code: 299,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r300 = HttpResponse {
            status_code: 300,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        assert!(!r199.is_success());
        assert!(r200.is_success());
        assert!(r299.is_success());
        assert!(!r300.is_success());
    }

    #[test]
    fn test_http_response_is_redirect_boundaries() {
        let r299 = HttpResponse {
            status_code: 299,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r300 = HttpResponse {
            status_code: 300,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r399 = HttpResponse {
            status_code: 399,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        let r400 = HttpResponse {
            status_code: 400,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        assert!(!r299.is_redirect());
        assert!(r300.is_redirect());
        assert!(r399.is_redirect());
        assert!(!r400.is_redirect());
    }

    #[test]
    fn test_http_response_content_type_found() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
            body: vec![],
            url: String::new(),
        };
        assert_eq!(resp.content_type(), Some("text/html; charset=utf-8"));
    }

    #[test]
    fn test_http_response_content_type_missing() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: String::new(),
        };
        assert!(resp.content_type().is_none());
    }

    #[test]
    fn test_http_response_text_invalid_utf8() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![0xFF, 0xFE],
            url: String::new(),
        };
        assert!(resp.text().is_err());
    }
}
