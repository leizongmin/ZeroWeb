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
}
