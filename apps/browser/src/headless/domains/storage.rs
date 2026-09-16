//! CDP Storage 域 — cookie 会话 jar 面。

use serde_json::Value;

use crate::headless::HeadlessServer;

use crate::headless::protocol::ProtocolError;
use crate::headless::session::HeadlessSession;

impl HeadlessServer {
    /// Storage.getCookies — 浏览器级全量枚举（PW 按 URL 客户端侧过滤）。
    pub(super) fn cmd_storage_get_cookies(&self, session: &mut HeadlessSession) -> Value {
        let cookies: Vec<Value> = session
            .cookie_store
            .all()
            .iter()
            .map(|c| Self::cookie_to_cdp(c))
            .collect();
        serde_json::json!({ "cookies": cookies })
    }

    /// Storage.setCookies — CDP cookie 描述 → jar（url 或 domain+path 定位作用域）。
    pub(super) fn cmd_storage_set_cookies(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        let cookies = params
            .get("cookies")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ProtocolError {
                code: -32602,
                message: "Missing 'cookies' parameter".into(),
            })?;
        for descriptor in cookies {
            let name = descriptor.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = descriptor.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let mut builder = zero_net::cookie::Cookie {
                name: name.to_string(),
                value: value.to_string(),
                domain: None,
                host_only: true,
                path: None,
                expires: None,
                secure: false,
                http_only: false,
                same_site: zero_net::cookie::SameSite::Lax,
                creation_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            if let Some(url) = descriptor.get("url").and_then(|v| v.as_str()) {
                if let Ok(parsed) = zero_net::parse_url(url) {
                    builder.domain = parsed.host.clone();
                    builder.secure = parsed.scheme == "https";
                    if let Some(path) = descriptor.get("path").and_then(|v| v.as_str()) {
                        builder.path = Some(path.to_string());
                    } else {
                        builder.path = Some(parsed.path.trim_end_matches('/').to_string());
                    }
                    session.cookie_store.add(builder);
                    continue;
                }
            }
            if let Some(domain) = descriptor.get("domain").and_then(|v| v.as_str()) {
                builder.domain = Some(domain.trim_start_matches('.').to_string());
                builder.host_only = false;
            }
            if let Some(path) = descriptor.get("path").and_then(|v| v.as_str()) {
                builder.path = Some(path.to_string());
            }
            if let Some(secure) = descriptor.get("secure").and_then(|v| v.as_bool()) {
                builder.secure = secure;
            }
            if let Some(http_only) = descriptor.get("httpOnly").and_then(|v| v.as_bool()) {
                builder.http_only = http_only;
            }
            if let Some(expires) = descriptor.get("expires").and_then(|v| v.as_f64()) {
                builder.expires = if expires < 0.0 { None } else { Some(expires as u64) };
            }
            session.cookie_store.add(builder);
        }
        Ok(serde_json::json!({}))
    }

    /// Storage.clearCookies — 清空 jar。
    pub(super) fn cmd_storage_clear_cookies(&self, session: &mut HeadlessSession) -> Value {
        session.cookie_store.clear();
        serde_json::json!({})
    }

    /// Network.getCookies — DevTools frontend CookiesModel 数据源（Application 面板
    /// Cookies 视图按页面 URL 拉取；devtools goal M2-N3）。`urls` 非空时按 host 后缀
    /// 匹配过滤，缺省全量。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/Network/#method-getCookies
    pub(super) fn cmd_network_get_cookies(&self, session: &mut HeadlessSession, params: &Value) -> Value {
        let urls: Vec<String> = params
            .get("urls")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let cookies: Vec<Value> = session
            .cookie_store
            .all()
            .iter()
            .filter(|c| {
                urls.is_empty()
                    || urls.iter().any(|url| {
                        let host = zero_net::parse_url(url).ok().and_then(|p| p.host).unwrap_or_default();
                        let domain = c.domain.clone().unwrap_or_default();
                        !host.is_empty()
                            && !domain.is_empty()
                            && (host == domain || host.ends_with(&format!(".{domain}")))
                    })
            })
            .map(|c| Self::cookie_to_cdp(c))
            .collect();
        serde_json::json!({ "cookies": cookies })
    }

    /// Network.setCookie — 单 cookie 写入（frontend Cookies 视图编辑回写，M2-N3）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/Network/#method-setCookie
    pub(super) fn cmd_network_set_cookie(
        &self,
        session: &mut HeadlessSession,
        params: Value,
    ) -> Result<Value, ProtocolError> {
        self.cmd_storage_set_cookies(session, serde_json::json!({ "cookies": [params] }))?;
        Ok(serde_json::json!({ "success": true }))
    }

    /// Network.clearBrowserCookies — 清空 jar（frontend Cookies 视图删除入口）。
    ///
    /// https://chromedevtools.github.io/devtools-protocol/tot/Network/#method-clearBrowserCookies
    pub(super) fn cmd_network_clear_browser_cookies(&self, session: &mut HeadlessSession) -> Value {
        session.cookie_store.clear();
        serde_json::json!({})
    }

    /// Cookie struct → CDP cookie 描述形状。
    pub(super) fn cookie_to_cdp(cookie: &zero_net::cookie::Cookie) -> Value {
        let expires = cookie
            .expires
            .map(|e| serde_json::json!(e as f64))
            .unwrap_or_else(|| serde_json::json!(-1.0));
        serde_json::json!({
            "name": cookie.name,
            "value": cookie.value,
            "domain": cookie.domain.clone().unwrap_or_default(),
            "path": cookie.path.clone().unwrap_or_else(|| "/".to_string()),
            "expires": expires,
            "size": cookie.name.len() + cookie.value.len(),
            "httpOnly": cookie.http_only,
            "secure": cookie.secure,
            "session": cookie.expires.is_none(),
            "priority": "Medium",
        })
    }
}
