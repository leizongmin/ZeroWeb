//! HTTP 客户端 — 封装 reqwest blocking 客户端。
//!
//! 提供同步 HTTP 请求发送能力。

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;

use crate::{HttpRequest, HttpResponse, NetError};

/// HTTP 客户端 — 封装 reqwest。
pub struct HttpClient {
    client: Client,
    /// 最大重定向次数。
    pub max_redirects: usize,
    /// 超时时间（秒）。
    pub timeout_secs: u64,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// 创建新的 HTTP 客户端，使用默认配置。
    pub fn new() -> Self {
        Self::with_config(30, 10)
    }

    /// 创建指定超时时间的 HTTP 客户端。
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self::with_config(timeout_secs, 10)
    }

    /// 创建指定最大重定向次数的 HTTP 客户端。
    pub fn with_max_redirects(max: usize) -> Self {
        Self::with_config(30, max)
    }

    /// 使用完整配置创建 HTTP 客户端。
    fn with_config(timeout_secs: u64, max_redirects: usize) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(max_redirects))
            .build()
            .unwrap_or_default();

        Self {
            client,
            max_redirects,
            timeout_secs,
        }
    }

    /// 发送 HTTP 请求。
    pub fn send(&self, request: HttpRequest) -> Result<HttpResponse, NetError> {
        let method = request.method.to_reqwest();
        let mut builder = self.client.request(method, &request.url);

        // 添加请求头
        let mut header_map = HeaderMap::new();
        for (name, value) in &request.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| NetError::Http(format!("invalid header name: {e}")))?;
            let header_value = reqwest::header::HeaderValue::from_bytes(value.as_bytes())
                .map_err(|e| NetError::Http(format!("invalid header value: {e}")))?;
            header_map.append(header_name, header_value);
        }
        builder = builder.headers(header_map);

        // 添加请求体
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder.send().map_err(|e| {
            if e.is_timeout() {
                NetError::Timeout
            } else if e.is_redirect() {
                NetError::TooManyRedirects
            } else {
                NetError::Network(e.to_string())
            }
        })?;

        // 转换响应
        let status_code = response.status().as_u16();
        let url = response.url().to_string();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().map_err(|e| NetError::Network(e.to_string()))?;
        let body = body.to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
            url,
        })
    }

    /// GET 请求。
    pub fn get(&self, url: &str) -> Result<HttpResponse, NetError> {
        self.send(HttpRequest::get(url))
    }

    /// POST 请求。
    pub fn post(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse, NetError> {
        self.send(HttpRequest::post(url, body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert_eq!(client.timeout_secs, 30);
        assert_eq!(client.max_redirects, 10);
    }

    #[test]
    fn test_http_client_with_timeout() {
        let client = HttpClient::with_timeout(60);
        assert_eq!(client.timeout_secs, 60);
        assert_eq!(client.max_redirects, 10);
    }

    #[test]
    fn test_http_client_config() {
        let client = HttpClient::with_max_redirects(5);
        assert_eq!(client.max_redirects, 5);
        assert_eq!(client.timeout_secs, 30);

        let client2 = HttpClient::with_timeout(120);
        assert_eq!(client2.timeout_secs, 120);
        assert_eq!(client2.max_redirects, 10);
    }
}
