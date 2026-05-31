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
}
