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
        let body = response
            .bytes()
            .map_err(|e| NetError::Network(e.to_string()))?;
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

    #[test]
    fn test_http_client_default() {
        let client = HttpClient::default();
        assert_eq!(client.timeout_secs, 30);
        assert_eq!(client.max_redirects, 10);
    }

    // ── Integration tests using wiremock ──

    /// Helper: build a tokio runtime and run a wiremock server for one test.
    /// Since HttpClient uses reqwest::blocking, we can't share the same tokio
    /// runtime easily. Instead we use a standalone integration test file.
    /// These tests verify error path mapping without network access.

    #[test]
    fn test_send_invalid_url_returns_network_error() {
        let client = HttpClient::new();
        let req = HttpRequest::get("http://0.0.0.0:1");
        let result = client.send(req);
        assert!(result.is_err());
        // Should be a Network error (connection refused)
        match result.unwrap_err() {
            NetError::Network(msg) => {
                assert!(msg.contains("connection refused") || msg.contains("error"));
            }
            NetError::Timeout => {
                // Also acceptable: could timeout trying to connect
            }
            other => panic!("expected Network or Timeout error, got: {other:?}"),
        }
    }

    #[test]
    fn test_send_invalid_header_name() {
        let client = HttpClient::new();
        let req = HttpRequest::get("http://example.com/")
            .header("Bad Header", "value");
        // This will fail because "Bad Header" is not a valid header name
        let result = client.send(req);
        assert!(result.is_err());
        match result.unwrap_err() {
            NetError::Http(msg) => {
                assert!(msg.contains("invalid header name"));
            }
            other => panic!("expected Http error for bad header name, got: {other:?}"),
        }
    }
}

/// Integration tests for HttpClient::send() using raw TCP mock servers.
///
/// These tests use minimal HTTP/1.1 TCP servers to avoid tokio runtime conflicts
/// with reqwest::blocking.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::{Read, Write};

    /// A minimal HTTP/1.1 mock server for testing reqwest::blocking.
    struct MockHttpServer {
        listener: std::net::TcpListener,
    }

    impl MockHttpServer {
        /// Bind to a random available port on localhost.
        fn new() -> Self {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")
                .expect("failed to bind mock server");
            Self { listener }
        }

        /// Get the base URL (http://127.0.0.1:PORT).
        fn url(&self) -> String {
            let port = self.listener.local_addr().unwrap().port();
            format!("http://127.0.0.1:{port}")
        }

        /// Accept one connection, drain the request, and send a canned response.
        fn respond_once(&self, status: u16, extra_headers: &str, body: &str) {
            let mut stream = self.listener.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 8192];
            let _ = stream.read(&mut buf);

            let response = format!(
                "HTTP/1.1 {status} OK\r\n{extra_headers}Content-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    }

    /// GET 请求成功返回 200，验证状态码和响应体。
    #[test]
    fn test_send_get_200() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(200, "Content-Type: text/plain\r\n", "hello world");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 200);
        assert!(resp.is_success());
        assert_eq!(resp.text().unwrap(), "hello world");
        assert!(resp.content_type().is_some());
    }

    /// POST 请求发送 body 并验证响应。
    #[test]
    fn test_send_post_with_body() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(201, "", "created");
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&url, b"request body data".to_vec());
        let resp = client.send(req).unwrap();

        assert_eq!(resp.status_code, 201);
        assert!(resp.is_success());
    }

    /// 验证 404 响应正确解析（非成功状态码）。
    #[test]
    fn test_send_404() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(404, "", "not found");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 404);
        assert!(!resp.is_success());
        assert_eq!(resp.text().unwrap(), "not found");
    }

    /// 验证 500 响应正确解析。
    #[test]
    fn test_send_500() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(500, "", "internal error");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        assert_eq!(resp.status_code, 500);
        assert!(!resp.is_success());
    }

    /// 验证响应头正确解析。
    #[test]
    fn test_send_response_headers() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(200, "X-Response-Header: header-value\r\n", "ok");
        });

        let client = HttpClient::new();
        let resp = client.send(HttpRequest::get(&url)).unwrap();

        let header_val = resp
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("x-response-header"));
        assert!(header_val.is_some());
        assert_eq!(header_val.unwrap().1, "header-value");
    }

    /// 验证 POST 请求不带 body 也能成功发送。
    #[test]
    fn test_send_post_no_body() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(200, "", "ok");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Post,
            url,
            headers: Vec::new(),
            body: None,
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 PUT 请求方法正确映射。
    #[test]
    fn test_send_put_method() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(200, "", "ok");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Put,
            url,
            headers: Vec::new(),
            body: Some(b"put data".to_vec()),
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 DELETE 请求方法正确映射。
    #[test]
    fn test_send_delete_method() {
        let server = MockHttpServer::new();
        let url = server.url();

        std::thread::spawn(move || {
            server.respond_once(204, "", "");
        });

        let client = HttpClient::new();
        let req = HttpRequest {
            method: crate::HttpMethod::Delete,
            url,
            headers: Vec::new(),
            body: None,
        };
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 204);
    }

    /// 验证自定义请求头正确发送到服务端。
    #[test]
    fn test_send_custom_headers_received() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let request_url = format!("http://127.0.0.1:{port}/");

        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            // reqwest lowercases header names
            assert!(
                request_str.contains("x-custom: test-value"),
                "request should contain x-custom header, got: {request_str}"
            );

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::get(&request_url).header("X-Custom", "test-value");
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证 POST body 正确发送到服务端。
    #[test]
    fn test_send_post_body_received() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let request_url = format!("http://127.0.0.1:{port}/");

        std::thread::spawn(move || {
            let mut stream = listener.incoming().next().unwrap().unwrap();
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).unwrap();
            let request_str = String::from_utf8_lossy(&buf[..n]);

            assert!(
                request_str.contains("hello from test"),
                "request should contain body, got: {request_str}"
            );
            assert!(
                request_str.starts_with("POST"),
                "request should be POST, got: {request_str}"
            );

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::new();
        let req = HttpRequest::post(&request_url, b"hello from test".to_vec());
        let resp = client.send(req).unwrap();
        assert_eq!(resp.status_code, 200);
    }

    /// 验证重定向后 URL 更新。
    #[test]
    fn test_send_redirect_updates_url() {
        let listener1 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port1 = listener1.local_addr().unwrap().port();

        let listener2 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port2 = listener2.local_addr().unwrap().port();
        let target_url = format!("http://127.0.0.1:{port2}/final");

        let target_clone = target_url.clone();
        let h1 = std::thread::spawn(move || {
            let mut stream = listener1.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: {target_clone}\r\nContent-Length: 0\r\n\r\n"
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let h2 = std::thread::spawn(move || {
            let mut stream = listener2.incoming().next().unwrap().unwrap();
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = "final page";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        let client = HttpClient::with_max_redirects(5);
        let req = HttpRequest::get(&format!("http://127.0.0.1:{port1}/redirect"));
        let resp = client.send(req).unwrap();

        assert_eq!(resp.status_code, 200);
        assert!(resp.url.contains("/final"));
        assert_eq!(resp.text().unwrap(), "final page");

        let _ = h1.join();
        let _ = h2.join();
    }
}
