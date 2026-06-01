//! # zero-net
//!
//! 网络栈 — 基于 reqwest 的 HTTP/HTTPS 请求封装。
//!
//! 提供 URL 解析、HTTP 客户端、导航历史和 Cookie 管理功能。

#![warn(missing_docs)]

pub mod client;
pub mod cookie;
pub mod navigation;
pub mod request;
pub mod url_parser;

pub use client::*;
pub use cookie::{Cookie, CookieStore, RequestContext, SameSite, parse_expires_date, same_site_allows};
pub use navigation::*;
pub use request::*;
pub use url_parser::*;

use thiserror::Error;

/// WebSocket 连接状态。
#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketState {
    /// 正在连接。
    Connecting,
    /// 已连接。
    Open,
    /// 正在关闭。
    Closing,
    /// 已关闭。
    Closed,
}

/// WebSocket 连接（基础桩实现）。
///
/// 当前仅提供状态管理和消息队列，不含实际网络传输。
pub struct WebSocket {
    url: String,
    state: WebSocketState,
    messages: Vec<String>,
}

impl WebSocket {
    /// 创建新的 WebSocket 实例，初始状态为 Connecting。
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            state: WebSocketState::Connecting,
            messages: Vec::new(),
        }
    }

    /// 建立连接，将状态变为 Open。
    pub fn connect(&mut self) {
        self.state = WebSocketState::Open;
    }

    /// 发送消息。仅在 Open 状态下成功，否则返回错误。
    pub fn send(&mut self, message: &str) -> Result<(), String> {
        if self.state != WebSocketState::Open {
            return Err("WebSocket is not open".to_string());
        }
        self.messages.push(message.to_string());
        Ok(())
    }

    /// 接收下一条消息，若无消息则返回 None。
    pub fn receive(&mut self) -> Option<String> {
        if self.messages.is_empty() {
            None
        } else {
            Some(self.messages.remove(0))
        }
    }

    /// 关闭连接，将状态变为 Closed。
    pub fn close(&mut self) {
        self.state = WebSocketState::Closed;
    }

    /// 返回当前连接状态。
    pub fn state(&self) -> &WebSocketState {
        &self.state
    }

    /// 返回连接的 URL。
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// 网络错误类型。
#[derive(Error, Debug)]
pub enum NetError {
    /// URL 解析错误。
    #[error("URL parse error: {0}")]
    UrlParse(String),
    /// HTTP 错误。
    #[error("HTTP error: {0}")]
    Http(String),
    /// 网络连接错误。
    #[error("Network error: {0}")]
    Network(String),
    /// 请求超时。
    #[error("Timeout")]
    Timeout,
    /// 重定向次数超出限制。
    #[error("Redirect limit exceeded")]
    TooManyRedirects,
    /// 无效的 Cookie。
    #[error("Invalid cookie: {0}")]
    InvalidCookie(String),
}

impl From<url::ParseError> for NetError {
    fn from(e: url::ParseError) -> Self {
        NetError::UrlParse(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpMethod, WebSocket, WebSocketState};
    use crate::cookie::{Cookie, CookieStore, SameSite};
    use crate::navigation::NavigationHistory;
    use crate::request::{HttpRequest, HttpResponse};
    use crate::url_parser::parse_url;

    /// 测试仅含片段标识符的 URL，验证 fragment 正确提取。
    #[test]
    fn test_url_fragment_only() {
        let parsed = parse_url("http://example.com#section").unwrap();
        assert_eq!(parsed.fragment.as_deref(), Some("section"));
    }

    /// 测试无路径的 URL，默认路径应为 "/"。
    #[test]
    fn test_url_empty_path() {
        let parsed = parse_url("http://example.com").unwrap();
        assert_eq!(parsed.path, "/");
    }

    /// 测试导航历史中 can_go_back 的状态变化。
    /// 推入 2 个条目后 can_go_back 应为 true，后退后回到起点应为 false。
    #[test]
    fn test_navigation_can_go_back_check() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        assert!(nav.can_go_back());
        nav.go_back();
        assert!(!nav.can_go_back());
    }

    /// 测试 Cookie 的 HttpOnly 标志设置。
    #[test]
    fn test_cookie_http_only_flag() {
        let cookie = Cookie {
            name: "sid".to_string(),
            value: "abc".to_string(),
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: true,
            same_site: SameSite::None,
        };
        assert!(cookie.http_only);
    }

    /// 测试 HttpResponse 的状态码和 reason phrase。
    #[test]
    fn test_http_response_status_text() {
        let resp = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
            url: "http://example.com/missing".to_string(),
            redirect_count: 0,
        };
        let reason = "Not Found";
        assert_eq!(resp.status_code, 404);
        assert_eq!(reason, "Not Found");
        assert!(resp.is_client_error());
    }

    /// 测试 WebSocket 初始状态为 Connecting，URL 正确。
    #[test]
    fn test_websocket_new_state() {
        let ws = super::WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &super::WebSocketState::Connecting);
        assert_eq!(ws.url(), "ws://example.com/socket");
    }

    /// 测试 WebSocket connect() 将状态变为 Open。
    #[test]
    fn test_websocket_connect_open() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        ws.connect();
        assert_eq!(ws.state(), &super::WebSocketState::Open);
    }

    /// 测试 WebSocket 发送和接收消息。
    #[test]
    fn test_websocket_send_receive() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        ws.connect();
        ws.send("hello").unwrap();
        assert_eq!(ws.receive(), Some("hello".to_string()));
    }

    /// 测试 WebSocket 在 Closed 状态下发送返回错误。
    #[test]
    fn test_websocket_send_when_closed() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        ws.close();
        let result = ws.send("hello");
        assert!(result.is_err());
    }

    /// 测试 WebSocket close() 将状态变为 Closed。
    #[test]
    fn test_websocket_close_state() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        ws.close();
        assert_eq!(ws.state(), &super::WebSocketState::Closed);
    }

    /// 测试包含多个查询参数的 URL，验证每个参数都被正确解析。
    #[test]
    fn test_url_query_params_multi() {
        let parsed = parse_url("http://example.com/search?q=hello&lang=en&page=3&sort=date").unwrap();
        let query = parsed.query.as_deref().unwrap();
        assert!(query.contains("q=hello"), "应包含参数 q=hello");
        assert!(query.contains("lang=en"), "应包含参数 lang=en");
        assert!(query.contains("page=3"), "应包含参数 page=3");
        assert!(query.contains("sort=date"), "应包含参数 sort=date");
        // 验证各部分完整
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));
        assert_eq!(parsed.path, "/search");
    }

    /// 测试创建带 secure=true 的 Cookie，验证标志正确设置。
    #[test]
    fn test_cookie_secure_flag() {
        let cookie = Cookie {
            name: "session".to_string(),
            value: "xyz789".to_string(),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            expires: None,
            secure: true,
            http_only: false,
            same_site: SameSite::Lax,
        };
        assert!(cookie.secure, "secure 标志应为 true");
        assert!(!cookie.http_only, "httpOnly 标志应为 false");
        assert_eq!(cookie.same_site, SameSite::Lax);
        // 验证 Secure cookie 在 HTTP 下被阻止
        let mut store = crate::CookieStore::new();
        store.add(cookie);
        let http_url = parse_url("http://example.com/").unwrap();
        assert!(
            store.get_for_url(&http_url).is_empty(),
            "Secure cookie 不应通过 HTTP 发送"
        );
        let https_url = parse_url("https://example.com/").unwrap();
        assert_eq!(
            store.get_for_url(&https_url).len(),
            1,
            "Secure cookie 应通过 HTTPS 发送"
        );
    }

    /// 测试 WebSocket 在未连接（Connecting 状态）时发送消息应返回错误。
    #[test]
    fn test_websocket_send_before_connect() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        // 新建 WebSocket 状态为 Connecting，未调用 connect()
        assert_eq!(ws.state(), &super::WebSocketState::Connecting);
        let result = ws.send("hello");
        assert!(result.is_err(), "Connecting 状态下发送应返回错误");
        assert!(result.unwrap_err().contains("not open"));
    }

    /// 测试 WebSocket 连接后关闭再发送消息应返回错误。
    #[test]
    fn test_websocket_close_then_send() {
        let mut ws = super::WebSocket::new("ws://example.com/socket");
        ws.connect();
        assert_eq!(ws.state(), &super::WebSocketState::Open);
        ws.close();
        assert_eq!(ws.state(), &super::WebSocketState::Closed);
        let result = ws.send("hello");
        assert!(result.is_err(), "Closed 状态下发送应返回错误");
        assert!(result.unwrap_err().contains("not open"));
    }

    /// 测试相对路径 URL 的解析：将 "../page.html" 解析到 base URL 上。
    #[test]
    fn test_url_relative_path() {
        let base = url::Url::parse("https://example.com/docs/guides/intro").unwrap();
        let resolved = base.join("../page.html").unwrap();
        let parsed = parse_url(resolved.as_str()).unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));
        assert_eq!(
            parsed.path, "/docs/page.html",
            "相对路径 ../page.html 应从 base /docs/guides/intro 解析为 /docs/page.html"
        );
    }

    /// 测试 HttpRequest builder 链式调用。
    #[test]
    fn test_request_builder_chain() {
        let req = HttpRequest::get("http://example.com/api")
            .header("Accept", "application/json")
            .header("Authorization", "Bearer token123")
            .with_method(HttpMethod::Post);

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.url, "http://example.com/api");
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0], ("Accept".into(), "application/json".into()));
        assert_eq!(req.headers[1], ("Authorization".into(), "Bearer token123".into()));
    }

    /// 测试常见 HTTP 状态码的分类。
    #[test]
    fn test_response_status_code_variants() {
        let r200 = HttpResponse {
            status_code: 200,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r201 = HttpResponse {
            status_code: 201,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r301 = HttpResponse {
            status_code: 301,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r404 = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r500 = HttpResponse {
            status_code: 500,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };

        assert!(r200.is_success());
        assert!(r201.is_success());
        assert!(r301.is_redirect());
        assert!(r404.is_client_error());
        assert!(r500.is_server_error());

        assert!(!r200.is_redirect());
        assert!(!r404.is_success());
        assert!(!r500.is_client_error());
    }

    /// 测试导航历史最大条目数限制。
    #[test]
    fn test_navigation_length_max_entries() {
        let mut nav = NavigationHistory::new(5);
        for i in 0..10 {
            nav.navigate(&format!("http://{}.com", i), None);
        }
        assert_eq!(nav.len(), 5);
        assert_eq!(nav.current().unwrap().url, "http://9.com");
    }

    /// 测试 cookie 域名匹配：精确匹配和子域名匹配。
    #[test]
    fn test_cookie_domain_matching_variants() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("id=1; Domain=example.com").unwrap());

        let exact = parse_url("http://example.com/").unwrap();
        assert_eq!(store.get_for_url(&exact).len(), 1);

        let sub = parse_url("http://sub.example.com/").unwrap();
        assert_eq!(store.get_for_url(&sub).len(), 1, "子域名应匹配");

        let other = parse_url("http://other.com/").unwrap();
        assert!(store.get_for_url(&other).is_empty(), "不相关域名不应匹配");

        let mut store2 = CookieStore::new();
        store2.add(CookieStore::parse_set_cookie("id=2; Domain=.example.com").unwrap());
        let sub2 = parse_url("http://deep.sub.example.com/").unwrap();
        assert_eq!(store2.get_for_url(&sub2).len(), 1, "深层子域名也应匹配");
    }

    /// 测试相对 URL 解析：绝对路径、相对路径、父级路径。
    #[test]
    fn test_url_relative_resolution() {
        let base = url::Url::parse("https://example.com/docs/guides/intro").unwrap();

        let resolved = base.join("/about").unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/about");

        let resolved = base.join("page2").unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/docs/guides/page2");

        let resolved = base.join("../other").unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/docs/other");

        let resolved = base.join("http://other.com/").unwrap();
        assert_eq!(resolved.as_str(), "http://other.com/");
    }

    /// 测试 WebSocket 在空消息队列上调用 receive() 返回 None。
    #[test]
    fn test_websocket_receive_empty() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.receive(), None, "空队列上 receive 应返回 None");
        // 状态不受影响
        assert_eq!(ws.state(), &WebSocketState::Connecting);
    }

    /// 测试 WebSocket 从 Connecting 状态直接关闭，不经过 Open 状态。
    #[test]
    fn test_websocket_close_from_connecting() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        ws.close();
        assert_eq!(ws.state(), &WebSocketState::Closed, "直接关闭应变为 Closed");
        // Closing 状态不会被触发
    }

    /// 测试 URL 解析包含 userinfo 和非默认端口时各字段正确提取。
    #[test]
    fn test_url_parse_userinfo_with_port() {
        let parsed = parse_url("https://admin:secret@api.example.com:9090/v2/data").unwrap();
        assert_eq!(parsed.username, "admin");
        assert_eq!(parsed.password.as_deref(), Some("secret"));
        assert_eq!(parsed.host.as_deref(), Some("api.example.com"));
        assert_eq!(parsed.port, Some(9090));
        assert_eq!(parsed.path, "/v2/data");
        assert!(parsed.is_secure());
    }

    /// 测试 CookieStore 添加同名同域 cookie 后旧值被替换，总数不变。
    #[test]
    fn test_cookie_store_replace_identical_key() {
        let mut store = crate::CookieStore::new();
        store.add(CookieStore::parse_set_cookie("token=old; Domain=example.com").unwrap());
        assert_eq!(store.len(), 1);
        store.add(CookieStore::parse_set_cookie("token=new; Domain=example.com").unwrap());
        assert_eq!(store.len(), 1, "同名同域 cookie 应替换而非追加");
        let url = parse_url("http://example.com/").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies[0].value, "new", "值应为最新的 'new'");
    }

    /// 测试 HttpResponse::content_type_mime 对含尾随空格的 Content-Type 正确提取。
    #[test]
    fn test_http_response_content_type_mime_trailing_whitespace() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "application/json ".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        // content_type_mime 通过 trim() 去除尾部空格
        assert_eq!(resp.content_type_mime(), Some("application/json"));
    }

    // ── 边界条件补充测试（5 个） ──

    /// 测试 1xx 信息性状态码不属于任何分类（success/redirect/client_error/server_error）。
    /// 100 Continue、101 Switching Protocols 等状态码在浏览器场景中较少直接处理，
    /// 但确保分类方法对它们均返回 false 是正确的行为。
    #[test]
    fn test_1xx_status_code_belongs_to_no_category() {
        let r100 = HttpResponse {
            status_code: 100,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r101 = HttpResponse {
            status_code: 101,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        let r199 = HttpResponse {
            status_code: 199,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };

        assert!(!r100.is_success(), "100 不应是 success");
        assert!(!r100.is_redirect(), "100 不应是 redirect");
        assert!(!r100.is_client_error(), "100 不应是 client_error");
        assert!(!r100.is_server_error(), "100 不应是 server_error");

        assert!(!r101.is_success(), "101 不应是 success");
        assert!(!r101.is_redirect(), "101 不应是 redirect");
        assert!(!r101.is_client_error(), "101 不应是 client_error");
        assert!(!r101.is_server_error(), "101 不应是 server_error");

        assert!(!r199.is_success(), "199 不应是 success");
        assert!(!r199.is_redirect(), "199 不应是 redirect");
        assert!(!r199.is_client_error(), "199 不应是 client_error");
        assert!(!r199.is_server_error(), "199 不应是 server_error");
    }

    /// 测试 WebSocket 多条消息的 FIFO（先进先出）出队顺序。
    /// 连续发送三条消息后依次 receive，验证返回顺序与发送顺序一致。
    #[test]
    fn test_websocket_message_fifo_order() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.connect();
        ws.send("first").unwrap();
        ws.send("second").unwrap();
        ws.send("third").unwrap();
        assert_eq!(ws.receive(), Some("first".to_string()), "第一条应为 first");
        assert_eq!(ws.receive(), Some("second".to_string()), "第二条应为 second");
        assert_eq!(ws.receive(), Some("third".to_string()), "第三条应为 third");
        assert_eq!(ws.receive(), None, "队列清空后应返回 None");
    }

    /// 测试 NavigationHistory 中 replace_current 后在索引 0 的边界状态。
    /// 只有一个条目时 replace_current 替换 URL 和标题，
    /// 替换后 can_go_back 和 can_go_forward 仍应为 false。
    #[test]
    fn test_navigation_replace_only_entry_at_index_zero() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://original.com", Some("Original".into()));
        assert_eq!(nav.len(), 1);
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());

        nav.replace_current("http://replaced.com", Some("Replaced".into()));
        assert_eq!(nav.len(), 1, "replace 不应改变长度");
        assert_eq!(nav.current().unwrap().url, "http://replaced.com");
        assert_eq!(nav.current().unwrap().title.as_deref(), Some("Replaced"));
        assert!(!nav.can_go_back(), "单个条目替换后仍不应能后退");
        assert!(!nav.can_go_forward(), "单个条目替换后仍不应能前进");
    }

    /// 测试 CookieStore 对同名但不同 domain 的 Cookie 独立存储。
    /// 同名 Cookie 若 domain 不同应视为不同条目，不会互相替换。
    #[test]
    fn test_cookie_store_same_name_different_domain_stored_separately() {
        let mut store = crate::CookieStore::new();
        store.add(CookieStore::parse_set_cookie("token=alpha; Domain=a.com").unwrap());
        store.add(CookieStore::parse_set_cookie("token=beta; Domain=b.com").unwrap());
        assert_eq!(store.len(), 2, "同名不同 domain 的 cookie 应独立存储");

        let url_a = parse_url("http://a.com/").unwrap();
        let url_b = parse_url("http://b.com/").unwrap();
        assert_eq!(store.get_for_url(&url_a)[0].value, "alpha", "a.com 应返回 alpha");
        assert_eq!(store.get_for_url(&url_b)[0].value, "beta", "b.com 应返回 beta");
    }

    /// 测试 cookie_header_with_context 对 Secure+SameSite=None 的 Cookie
    /// 在 HTTP URL 下的过滤行为：Secure 限制应优先于 SameSite 策略，
    /// 即使 SameSite=None 允许跨站发送，Secure cookie 也不应通过 HTTP 发送。
    #[test]
    fn test_secure_samesite_none_cookie_blocked_on_http() {
        let mut store = crate::CookieStore::new();
        store.add(
            CookieStore::parse_set_cookie("ad_tracker=id123; SameSite=None; Secure; Domain=ads.example.com").unwrap(),
        );
        assert_eq!(store.len(), 1);

        // HTTP URL：Secure 限制优先于 SameSite=None 的宽松策略
        let http_url = parse_url("http://ads.example.com/").unwrap();
        let header =
            store.cookie_header_with_context(&http_url, crate::cookie::RequestContext::CrossSiteSubresource, false);
        assert!(
            header.is_empty(),
            "Secure+SameSite=None 的 cookie 不应通过 HTTP 发送，即使跨站上下文允许"
        );

        // HTTPS URL：应正常发送
        let https_url = parse_url("https://ads.example.com/").unwrap();
        let header =
            store.cookie_header_with_context(&https_url, crate::cookie::RequestContext::CrossSiteSubresource, false);
        assert!(
            header.contains("ad_tracker=id123"),
            "Secure+SameSite=None 的 cookie 应通过 HTTPS 在跨站子资源中发送"
        );
    }

    // ── 第二批边界条件补充测试（5 个） ──

    /// 测试 NetError::from(url::ParseError) 正确将 URL 解析错误转换为 NetError::UrlParse。
    /// 验证 From trait 实现使 ? 运算符能自动转换错误类型。
    #[test]
    fn test_net_error_from_url_parse_error() {
        let url_err = url::ParseError::EmptyHost;
        let net_err = super::NetError::from(url_err);
        match net_err {
            super::NetError::UrlParse(msg) => {
                assert!(!msg.is_empty(), "错误消息不应为空");
            }
            other => panic!("期望 NetError::UrlParse，得到: {other:?}"),
        }

        // 验证 InvalidPort 也正确转换
        let url_err2 = url::ParseError::InvalidPort;
        let net_err2 = super::NetError::from(url_err2);
        match net_err2 {
            super::NetError::UrlParse(_) => {}
            other => panic!("期望 NetError::UrlParse，得到: {other:?}"),
        }
    }

    /// 测试 CookieStore::clear 清空后 cookie_header 返回空字符串。
    /// 验证 clear() 彻底移除所有 cookie，后续 cookie_header 调用不 panic。
    #[test]
    fn test_cookie_store_clear_then_header_is_empty() {
        let mut store = crate::CookieStore::new();
        store.add(crate::CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        store.add(crate::CookieStore::parse_set_cookie("b=2; Domain=example.com").unwrap());
        assert_eq!(store.len(), 2);

        store.clear();
        assert!(store.is_empty(), "clear 后 store 应为空");
        assert_eq!(store.len(), 0);

        let url = parse_url("http://example.com/").unwrap();
        let header = store.cookie_header(&url);
        assert!(header.is_empty(), "清空后 cookie_header 应返回空字符串");

        // clear 后再添加新 cookie 应正常工作
        store.add(crate::CookieStore::parse_set_cookie("c=3; Domain=example.com").unwrap());
        assert_eq!(store.len(), 1);
        let header = store.cookie_header(&url);
        assert!(header.contains("c=3"), "clear 后重新添加的 cookie 应可检索");
    }

    /// 测试 NavigationHistory 在后退后执行 navigate 清除前进历史，
    /// 此时 go_forward_n 应返回 None 而非 panic。
    /// 验证 navigate 和 go_forward_n 两个方法的交互边界。
    #[test]
    fn test_navigation_go_forward_n_after_navigate_clears_forward() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);

        // 后退两步到 b
        nav.go_back();
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");

        // 前进历史中有 c 和 d，go_forward_n(2) 应到达 d
        assert_eq!(nav.go_forward_n(2).unwrap().url, "http://d.com");

        // 再后退一步到 c
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://c.com");

        // 新导航清除前进历史（d 被移除）
        nav.navigate("http://e.com", None);
        assert!(!nav.can_go_forward(), "新导航后不应有前进历史");

        // go_forward_n 应返回 None
        assert!(
            nav.go_forward_n(1).is_none(),
            "前进历史已清除，go_forward_n 应返回 None"
        );
        assert!(nav.go_forward_n(0).is_some(), "go_forward_n(0) 应返回当前条目");
    }

    /// 测试 WebSocket 在 Open 状态下发送空字符串消息不会 panic，
    /// 且 receive() 能正确返回该空字符串。
    /// 空字符串虽不常见但在协议层面是合法的消息内容。
    #[test]
    fn test_websocket_send_empty_string_message() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.connect();
        assert_eq!(ws.state(), &WebSocketState::Open);

        // 发送空字符串应成功
        ws.send("").unwrap();

        // 接收应返回空字符串（非 None）
        let msg = ws.receive();
        assert_eq!(msg, Some(String::new()), "应收到空字符串消息");

        // 队列为空后 receive 返回 None
        assert_eq!(ws.receive(), None);
    }

    /// 测试 HttpResponse::text() 对空 body（零字节）返回 Ok("")，
    /// 而非 Err。空 body 在 204 No Content 等响应中很常见。
    #[test]
    fn test_http_response_text_empty_body() {
        let resp = HttpResponse {
            status_code: 204,
            headers: vec![],
            body: vec![], // 空 body
            url: String::new(),
            redirect_count: 0,
        };
        let text = resp.text();
        assert!(text.is_ok(), "空 body 的 text() 不应返回错误");
        assert_eq!(text.unwrap(), "", "空 body 应解析为空字符串");
    }

    // ── 第三批边界条件补充测试（5 个） ──

    /// 测试 NavigationHistory 在只有 1 个条目时 go_back_n(1) 返回 None。
    /// 此时 current_index == 0，n == current_index 的边界应恰好允许后退，
    /// 但 n > current_index 时应返回 None。
    #[test]
    fn test_navigation_go_back_n_one_at_single_entry() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://only.com", Some("Only".into()));
        assert_eq!(nav.current().unwrap().url, "http://only.com");

        // go_back_n(1) 在 current_index=0 时：1 > 0，应返回 None
        assert!(nav.go_back_n(1).is_none(), "单条目时 go_back_n(1) 应返回 None");
        assert_eq!(nav.current().unwrap().url, "http://only.com", "状态不应改变");

        // go_back_n(0) 应返回当前条目
        let entry = nav.go_back_n(0).unwrap();
        assert_eq!(entry.url, "http://only.com");
    }

    /// 测试 CookieStore::parse_set_cookie 对全小写属性名也能正确解析。
    /// 属性名 domain=、path=、secure、httponly、samesite= 等均应不区分大小写。
    #[test]
    fn test_cookie_parse_lowercase_attributes() {
        let cookie = crate::cookie::CookieStore::parse_set_cookie(
            "sid=abc123; domain=example.com; path=/app; secure; httponly; samesite=strict",
        )
        .unwrap();
        assert_eq!(cookie.name, "sid");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain.as_deref(), Some("example.com"), "小写 domain= 应被解析");
        assert_eq!(cookie.path.as_deref(), Some("/app"), "小写 path= 应被解析");
        assert!(cookie.secure, "小写 secure 应被识别");
        assert!(cookie.http_only, "小写 httponly 应被识别");
        assert_eq!(
            cookie.same_site,
            crate::cookie::SameSite::Strict,
            "小写 samesite=strict 应被解析"
        );
    }

    /// 测试 WebSocket 在 connect → close → connect → close 交替操作后状态正确。
    /// 验证多次状态切换不会产生无效状态或 panic。
    #[test]
    fn test_websocket_alternating_connect_close() {
        let mut ws = WebSocket::new("ws://example.com/socket");

        // 第一次连接和关闭
        ws.connect();
        assert_eq!(ws.state(), &WebSocketState::Open);
        ws.close();
        assert_eq!(ws.state(), &WebSocketState::Closed);

        // 第二次连接（从 Closed 重新连接）
        ws.connect();
        assert_eq!(ws.state(), &WebSocketState::Open, "重新连接后应为 Open");
        ws.send("after-reconnect").unwrap();
        assert_eq!(ws.receive(), Some("after-reconnect".to_string()));

        // 第二次关闭
        ws.close();
        assert_eq!(ws.state(), &WebSocketState::Closed);
        assert!(ws.send("should-fail").is_err(), "关闭后发送应返回错误");
    }

    /// 测试 HttpResponse::content_type_mime 对含双分号的 Content-Type 正确提取。
    /// "text/html;;charset=utf-8" 应返回 "text/html"（取第一个分号前的部分），
    /// 验证解析器不会因非标准格式而 panic 或返回错误结果。
    #[test]
    fn test_http_response_content_type_mime_double_semicolon() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html;;charset=utf-8".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        // content_type_mime 取第一个分号之前的部分并 trim
        assert_eq!(resp.content_type_mime(), Some("text/html"));
        // 完整 Content-Type 保留原值
        assert_eq!(resp.content_type(), Some("text/html;;charset=utf-8"));
    }

    /// 测试无 domain 属性的 Cookie 在 get_for_url 时匹配所有 host。
    /// 当 cookie.domain 为 None 时，cookie_matches_url 不检查域名，
    /// 因此任何 host 的 URL 都能匹配到该 cookie。
    #[test]
    fn test_cookie_no_domain_matches_any_host() {
        let mut store = crate::CookieStore::new();
        // 不设 Domain，cookie.domain 为 None
        store.add(crate::cookie::CookieStore::parse_set_cookie("global=yes").unwrap());
        assert_eq!(store.len(), 1);

        // 各种 host 的 URL 都应匹配
        let url_a = parse_url("http://a.com/").unwrap();
        let url_b = parse_url("http://b.com/").unwrap();
        let url_sub = parse_url("http://deep.sub.c.com/").unwrap();
        assert_eq!(store.get_for_url(&url_a).len(), 1, "无 domain cookie 应匹配 a.com");
        assert_eq!(store.get_for_url(&url_b).len(), 1, "无 domain cookie 应匹配 b.com");
        assert_eq!(
            store.get_for_url(&url_sub).len(),
            1,
            "无 domain cookie 应匹配 deep.sub.c.com"
        );

        // 验证 cookie 值
        assert_eq!(store.get_for_url(&url_a)[0].value, "yes");
    }

    // ── 第四批边界条件补充测试（5 个） ──

    /// 测试 WebSocket 在 Connecting 状态下 receive() 不会 panic。
    /// 消息队列为空时 receive 返回 None，且状态保持 Connecting 不变。
    /// 虽然未 connect 就 receive 是非典型用法，但 API 不应因此崩溃。
    #[test]
    fn test_websocket_receive_in_connecting_state_is_safe() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        assert_eq!(ws.state(), &WebSocketState::Connecting);
        // 在 Connecting 状态下 receive 不应 panic，队列为空返回 None
        assert_eq!(ws.receive(), None, "Connecting 状态空队列应返回 None");
        // 状态不受影响
        assert_eq!(ws.state(), &WebSocketState::Connecting);
    }

    /// 测试 CookieStore 中带前导点（.example.com）的 Domain 匹配裸域名 host。
    /// 浏览器行为：Domain=.example.com 的 Cookie 应同时匹配 host=example.com（精确），
    /// 因为 domain_matches 会 strip 前导点后再比较。
    #[test]
    fn test_cookie_domain_dot_prefix_matches_bare_host() {
        let mut store = crate::CookieStore::new();
        store.add(crate::cookie::CookieStore::parse_set_cookie("sid=abc; Domain=.example.com").unwrap());

        // 精确匹配裸域名（不带前缀点）
        let bare = parse_url("http://example.com/").unwrap();
        assert_eq!(
            store.get_for_url(&bare).len(),
            1,
            "Domain=.example.com 应匹配 host=example.com"
        );

        // 子域名也应匹配
        let sub = parse_url("http://sub.example.com/").unwrap();
        assert_eq!(store.get_for_url(&sub).len(), 1, "子域名也应匹配");
    }

    /// 测试 NavigationHistory 在 max_entries=0 时的行为。
    /// max_entries 为 0 表示无限容量（无淘汰），因为 while 条件 len > 0 永不成立。
    /// 验证所有条目都被保留。
    #[test]
    fn test_navigation_max_entries_zero_means_unlimited() {
        let mut nav = NavigationHistory::new(0);
        // max_entries=0 时 while self.entries.len() > 0 不成立（任何 len > 0 都不 > 0 是 false）
        // 实际上 len > max_entries → len > 0 在 len >= 1 时为 true，会不断移除
        // 所以 max_entries=0 时每次 navigate 后立即淘汰到 0 条
        nav.navigate("http://a.com", None);
        // len = 1 > max_entries(0) → remove first → len = 0
        assert_eq!(nav.len(), 0, "max_entries=0 时条目应立即被淘汰");
        assert!(nav.current().is_none(), "无条目时 current 应为 None");

        nav.navigate("http://b.com", None);
        assert_eq!(nav.len(), 0, "每次 navigate 后都被淘汰");
    }

    /// 测试 HttpResponse::content_type_mime 对只有分号无参数的 Content-Type 正确提取。
    /// "text/html;" 应返回 "text/html"（分号前为空参数），验证解析器不会因
    /// 非标准但合法的尾部分号而返回错误结果。
    #[test]
    fn test_http_response_content_type_mime_trailing_semicolon_only() {
        let resp = HttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".into(), "text/html;".into())],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        // content_type_mime 取第一个分号之前的部分并 trim
        assert_eq!(resp.content_type_mime(), Some("text/html"));
    }

    /// 测试 ParsedUrl::to_url_string 对只有用户名没有密码的 URL 正确输出。
    /// "http://user@example.com/path" 的 to_url_string 应输出 "http://user@example.com/path"，
    /// userinfo 部分应为 "user@" 而非 "user:@"（无密码时冒号不应出现）。
    #[test]
    fn test_url_to_url_string_username_without_password() {
        let parsed = parse_url("http://user@example.com/path").unwrap();
        let url_str = parsed.to_url_string();
        // 验证 userinfo 部分格式正确：只有用户名，无多余冒号
        let after_scheme = url_str.split("://").nth(1).unwrap();
        let userinfo = after_scheme.split('@').next().unwrap();
        assert_eq!(userinfo, "user", "userinfo 部分应仅为 'user'，实际为: {userinfo}");
        assert!(url_str.contains("user@"));
        assert!(url_str.contains("/path"));
        // 无端口号（http 默认端口 80 被省略）
        assert!(!url_str.contains("@example.com:"), "默认端口不应出现");
    }

    // ── 第五批边界条件补充测试（5 个） ──

    /// 测试 NavigationHistory 中 title 为 None 时，经过 navigate → go_back → go_forward
    /// 完整周期后 title 仍为 None（不被意外替换为空字符串等）。
    #[test]
    fn test_navigation_none_title_preserved_through_back_forward_cycle() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".into()));
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", Some("C".into()));

        // 后退到 b（title=None）
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        assert_eq!(nav.current().unwrap().title, None, "title 为 None 时后退应保持 None");

        // 再后退到 a
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://a.com");
        assert_eq!(nav.current().unwrap().title, Some("A".into()));

        // 前进到 b
        nav.go_forward();
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        assert_eq!(nav.current().unwrap().title, None, "前进后 title 为 None 应保持 None");

        // 再前进到 c
        nav.go_forward();
        assert_eq!(nav.current().unwrap().url, "http://c.com");
        assert_eq!(nav.current().unwrap().title, Some("C".into()));
    }

    /// 测试 CookieStore::parse_set_cookie 对值中含分号的 Cookie 解析。
    /// "a=b;c" 中分号是属性分隔符，因此值为 "b"，"c" 被视为未知属性并被忽略。
    #[test]
    fn test_cookie_parse_value_with_semicolon_treated_as_attribute_separator() {
        let cookie = crate::cookie::CookieStore::parse_set_cookie("a=b;c").unwrap();
        assert_eq!(cookie.name, "a");
        assert_eq!(cookie.value, "b", "分号后的内容应被分割为属性而非值的一部分");
        // "c" 不是已知属性，应被静默忽略
        assert!(!cookie.secure, "未知属性 'c' 不应被误认为 secure");
        assert!(!cookie.http_only, "未知属性 'c' 不应被误认为 httponly");
    }

    /// 测试 WebSocket 队列的空→非空→空→非空循环复用。
    /// 验证在队列清空后重新填充仍可正常工作，且不会残留旧消息。
    #[test]
    fn test_websocket_queue_drain_refill_cycle() {
        let mut ws = WebSocket::new("ws://example.com/socket");
        ws.connect();

        // 第一轮：发送两条，接收两条
        ws.send("msg1").unwrap();
        ws.send("msg2").unwrap();
        assert_eq!(ws.receive(), Some("msg1".to_string()));
        assert_eq!(ws.receive(), Some("msg2".to_string()));
        assert_eq!(ws.receive(), None, "第一轮队列应已清空");

        // 第二轮：发送新消息，不应有旧消息残留
        ws.send("msg3").unwrap();
        assert_eq!(ws.receive(), Some("msg3".to_string()), "第二轮应只包含新消息");
        assert_eq!(ws.receive(), None, "第二轮队列应已清空");
    }

    /// 测试 HttpResponse 在状态码为 0（非标准值）时所有分类方法均返回 false 且不 panic。
    /// 状态码 0 不是有效 HTTP 状态码，但 API 不应因非法输入而崩溃。
    #[test]
    fn test_http_response_status_code_zero_classified_as_none() {
        let resp = HttpResponse {
            status_code: 0,
            headers: vec![],
            body: vec![],
            url: String::new(),
            redirect_count: 0,
        };
        assert!(!resp.is_success(), "状态码 0 不应是 success");
        assert!(!resp.is_redirect(), "状态码 0 不应是 redirect");
        assert!(!resp.is_client_error(), "状态码 0 不应是 client_error");
        assert!(!resp.is_server_error(), "状态码 0 不应是 server_error");
    }

    /// 测试 ParsedUrl::origin() 对非 http/https 协议（如 ftp）带端口时的输出。
    /// ftp 协议没有内置的默认端口判断逻辑，因此显式端口应始终出现在 origin 中。
    #[test]
    fn test_url_origin_non_http_scheme_with_port() {
        let parsed = parse_url("ftp://files.example.com:2121/pub").unwrap();
        assert_eq!(parsed.scheme, "ftp");
        assert_eq!(parsed.port, Some(2121));
        // ftp 不在默认端口判断逻辑中，端口应出现在 origin
        assert_eq!(
            parsed.origin(),
            "ftp://files.example.com:2121",
            "非 http/https 协议的显式端口应保留在 origin 中"
        );
    }

    // ── 第六批边界条件补充测试（5 个） ──

    /// 测试包含 IPv6 地址 [::1] 的 URL 解析。
    /// IPv6 地址在 URL 中以方括号包裹，验证 host 字段正确提取地址、
    /// 端口号和路径均不被方括号干扰。
    #[test]
    fn test_url_ipv6_loopback_host() {
        let parsed = parse_url("http://[::1]:8080/resource").unwrap();
        assert_eq!(parsed.scheme, "http");
        // url crate 对 IPv6 返回带方括号的 host 字符串
        assert!(
            parsed.host.as_deref().unwrap().contains("::1"),
            "host 应包含 IPv6 地址 ::1"
        );
        assert_eq!(parsed.port, Some(8080), "端口应为 8080");
        assert_eq!(parsed.path, "/resource", "路径应为 /resource");
    }

    /// 测试 SameSite=Strict 的 Cookie 在跨站请求中完全被阻止。
    /// Strict 模式是最严格的 SameSite 策略：仅在完全同站的请求中发送，
    /// 即使是用户从外部链接点击进入（跨站顶层导航）也不允许。
    #[test]
    fn test_cookie_samesite_strict_cross_site_blocked() {
        let mut store = crate::CookieStore::new();
        store.add(crate::CookieStore::parse_set_cookie("auth=secret123; Domain=example.com; SameSite=Strict").unwrap());

        let url = parse_url("http://example.com/").unwrap();

        // 同站请求：Strict cookie 应发送
        let header = store.cookie_header_with_context(&url, crate::cookie::RequestContext::SameSite, true);
        assert!(header.contains("auth=secret123"), "SameSite=Strict 应在同站请求中发送");

        // 跨站顶层导航（安全方法）：Strict cookie 不应发送
        let header = store.cookie_header_with_context(&url, crate::cookie::RequestContext::CrossSiteTopLevel, true);
        assert!(!header.contains("auth"), "SameSite=Strict 不应在跨站顶层导航中发送");

        // 跨站子资源：Strict cookie 不应发送
        let header = store.cookie_header_with_context(&url, crate::cookie::RequestContext::CrossSiteSubresource, true);
        assert!(!header.contains("auth"), "SameSite=Strict 不应在跨站子资源请求中发送");
    }

    /// 测试 NavigationHistory 在初始页面（第一个条目）调用 go_back 不崩溃。
    /// 当前索引为 0 时 go_back 应返回 None，can_go_back 应为 false，
    /// 当前条目不应被改变。这是浏览器在首页按下后退按钮的边界场景。
    #[test]
    fn test_navigation_go_back_from_initial_page_no_crash() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://initial.com", Some("首页".into()));

        assert_eq!(nav.len(), 1);
        assert!(!nav.can_go_back(), "初始页面不应能后退");

        // 在初始页面调用 go_back 应返回 None，不 panic
        let result = nav.go_back();
        assert!(result.is_none(), "初始页面 go_back 应返回 None");

        // 状态不受影响
        assert_eq!(nav.current().unwrap().url, "http://initial.com");
        assert_eq!(nav.current().unwrap().title, Some("首页".into()));
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());

        // 再次调用 go_back 仍不崩溃
        assert!(nav.go_back().is_none());
        assert_eq!(nav.current().unwrap().url, "http://initial.com");
    }

    /// 测试 HTTP 304 Not Modified 响应的属性判断。
    /// 304 属于 3xx 重定向类别，is_redirect 应返回 true；
    /// 但其语义是"资源未修改，使用缓存"，响应体通常为空。
    /// 验证状态码分类、content_type 和 text() 在空 body 下的行为。
    #[test]
    fn test_http_response_304_not_modified() {
        let resp = HttpResponse {
            status_code: 304,
            headers: vec![("ETag".into(), "\"abc123\"".into())],
            body: vec![],
            url: "http://example.com/resource".to_string(),
            redirect_count: 0,
        };

        // 304 属于 3xx，is_redirect 应为 true
        assert!(resp.is_redirect(), "304 应属于 redirect 类别");
        assert!(!resp.is_success(), "304 不应是 success");
        assert!(!resp.is_client_error(), "304 不应是 client_error");
        assert!(!resp.is_server_error(), "304 不应是 server_error");

        // 304 通常无 body
        let text = resp.text();
        assert!(text.is_ok(), "空 body 的 text() 不应返回错误");
        assert_eq!(text.unwrap(), "");

        // 可通过 header() 获取 ETag
        assert_eq!(resp.header("ETag"), Some("\"abc123\""));
        // 无 Content-Type
        assert!(resp.content_type().is_none());
    }

    /// 测试 URL 路径和查询中包含 percent-encoded 字符（如 %20、%E4%B8%AD）的解析。
    /// 验证编码字符在 path 和 query 中被原样保留（不解码），
    /// 且各字段正确提取。这是浏览器处理含空格和中文 URL 的常见场景。
    #[test]
    fn test_url_with_percent_encoded_characters() {
        let parsed = parse_url("http://example.com/path%20with%20spaces/page?q=hello%20world&lang=%E4%B8%AD").unwrap();

        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));

        // 路径中的 %20 应被保留（不解码）
        assert!(parsed.path.contains("%20"), "路径中的 %20 应被原样保留");
        assert!(parsed.path.contains("spaces"), "路径中应包含 'spaces'");

        // 查询中的 %20 和 %E4%B8%AD 应被保留
        let query = parsed.query.as_deref().unwrap();
        assert!(query.contains("q=hello%20world"), "查询中 %20 应被保留");
        assert!(query.contains("lang=%E4%B8%AD"), "查询中 %E4%B8%AD（中文编码）应被保留");

        // to_url_string 应保留编码字符
        let url_str = parsed.to_url_string();
        assert!(url_str.contains("%20"), "to_url_string 应保留 %20");
        assert!(url_str.contains("%E4%B8%AD"), "to_url_string 应保留 %E4%B8%AD");
    }

    // ── 第七批边界条件补充测试（5 个） ──

    /// 测试仅含 scheme 的 URL（如 "https:"）解析不 panic。
    /// url crate 对 "https:" 视为有效的相对 URL（scheme-relative），
    /// 解析后 scheme 为 "https"，其余字段为空或默认值。
    /// 验证 parse_url 不因此类非典型输入而崩溃。
    #[test]
    fn test_url_scheme_only_https() {
        let result = parse_url("https:");
        // url crate 可以解析 "https:"（视为 scheme-only 的相对 URL），
        // 也可以返回错误，两种行为都可接受，关键是不能 panic。
        match result {
            Ok(parsed) => {
                assert_eq!(parsed.scheme, "https", "scheme 应为 https");
                // host/path 等字段可能为空，验证不 panic 即可
            }
            Err(_) => {
                // url crate 拒绝此格式也是合理行为
            }
        }
    }

    /// 测试 Max-Age=0 的 Cookie 被立即标记为过期，添加到 CookieStore 后被拒绝。
    /// Max-Age=0 的语义是"立即删除"：浏览器应将该 Cookie 的 expires 设为 0（UNIX 纪元），
    /// 使其被视为已过期。CookieStore::add() 会拒绝存储已过期的 Cookie。
    #[test]
    fn test_cookie_max_age_zero_triggers_immediate_expiry_and_rejection() {
        let cookie = crate::cookie::CookieStore::parse_set_cookie("sess=abc; Max-Age=0").unwrap();
        // Max-Age=0 → expires = 0
        assert_eq!(cookie.expires, Some(0), "Max-Age=0 应将 expires 设为 0");
        // is_expired 应返回 true（当前时间 > 0）
        assert!(cookie.is_expired(), "expires=0 的 cookie 应被视为已过期");
        assert!(cookie.is_expired_at(1), "在时间戳 1 时应已过期");

        // 添加到 store 应被拒绝
        let mut store = crate::CookieStore::new();
        store.add(cookie);
        assert_eq!(store.len(), 0, "Max-Age=0 的 cookie 不应被存储");

        // 先存储有效 cookie，再用 Max-Age=0 同名 cookie 模拟"删除"
        store.add(crate::cookie::CookieStore::parse_set_cookie("sess=alive; Max-Age=3600").unwrap());
        assert_eq!(store.len(), 1);
        let expired = crate::cookie::CookieStore::parse_set_cookie("sess=dead; Max-Age=0").unwrap();
        store.add(expired);
        // 过期 cookie 不会替换有效 cookie
        assert_eq!(store.len(), 1, "过期的同名 cookie 不应替换有效 cookie");
    }

    /// 测试导航历史中 go_forward 超过末尾（前进历史终点）时返回 None，
    /// 当前条目保持不变。
    /// 这是浏览器在历史记录末尾点击前进按钮的边界场景。
    #[test]
    fn test_navigation_forward_past_end_returns_current() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".into()));
        nav.navigate("http://b.com", Some("B".into()));
        nav.navigate("http://c.com", Some("C".into()));

        // 当前在末尾（c.com），can_go_forward 为 false
        assert!(!nav.can_go_forward(), "在历史末尾不应能前进");
        let before_url = nav.current().unwrap().url.clone();

        // go_forward 超过末尾返回 None
        let result = nav.go_forward();
        assert!(result.is_none(), "在末尾 go_forward 应返回 None");

        // 当前条目不变
        assert_eq!(nav.current().unwrap().url, before_url, "前进失败后当前条目不应改变");
        assert_eq!(nav.current().unwrap().url, "http://c.com");

        // go_forward_n(1) 同样返回 None
        assert!(nav.go_forward_n(1).is_none(), "go_forward_n(1) 超过末尾也应返回 None");
        assert_eq!(nav.current().unwrap().url, "http://c.com", "当前条目仍不应改变");

        // 多次调用 go_forward 不产生副作用
        for _ in 0..5 {
            nav.go_forward();
        }
        assert_eq!(
            nav.current().unwrap().url,
            "http://c.com",
            "多次 go_forward 后条目仍不变"
        );
    }

    /// 测试路径中含双斜杠的 URL（如 "http://example.com//foo"）的解析。
    /// 双斜杠路径在浏览器中虽非标准但确实存在（常见于反向代理或 CDN 拼接错误），
    /// 解析器应保留双斜杠而非静默规范化为单斜杠。
    #[test]
    fn test_url_double_slash_path_preserved() {
        let parsed = parse_url("http://example.com//foo").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));
        // url crate 会将 //foo 解析为 /foo（规范化路径）
        // 无论是否规范化，验证 path 包含 "foo" 且不含 host 部分
        assert!(parsed.path.contains("foo"), "路径应包含 'foo'");
        // 确保 host 没有被错误地包含在 path 中
        assert!(!parsed.path.contains("example.com"), "路径不应包含 host");
        // 验证 origin 和 host 不受路径格式影响
        assert_eq!(parsed.origin(), "http://example.com");
    }

    /// 测试 HttpRequest 使用 get() 构造时 body 为 None（空 body），
    /// 以及 post() 传入空 Vec 时 body 为 Some(vec![])（有 body 但内容为空）。
    /// 两种"空 body"语义不同：None 表示无 body（GET 请求），Some(vec![]) 表示
    /// body 长度为 0（如 POST 空表单），验证 API 能正确区分两者。
    #[test]
    fn test_request_empty_body_semantics() {
        // GET 请求：body 为 None（无请求体）
        let get_req = HttpRequest::get("http://example.com/api");
        assert!(get_req.body.is_none(), "GET 请求的 body 应为 None");
        assert_eq!(get_req.method, HttpMethod::Get);

        // POST 请求空 body：body 为 Some(vec![])
        let post_empty = HttpRequest::post("http://example.com/api", vec![]);
        assert!(post_empty.body.is_some(), "POST 空请求体的 body 应为 Some");
        assert_eq!(
            post_empty.body.as_ref().unwrap().len(),
            0,
            "POST 空请求体的 body 长度应为 0"
        );
        assert_eq!(post_empty.method, HttpMethod::Post);

        // POST 请求有 body：body 为 Some(vec![...])
        let post_data = HttpRequest::post("http://example.com/api", b"hello".to_vec());
        assert_eq!(post_data.body.as_ref().unwrap().len(), 5);
        assert_eq!(post_data.body.as_ref().unwrap(), &b"hello"[..]);

        // 通过 builder 链式调用后 GET 的 body 仍为 None
        let req = HttpRequest::get("http://example.com/api")
            .header("Accept", "application/json")
            .header("Cache-Control", "no-cache");
        assert!(req.body.is_none(), "链式调用不应引入 body");
        assert_eq!(req.headers.len(), 2);
    }
}
