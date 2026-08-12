//! Cookie 管理模块。
//!
//! 提供 HTTP Cookie 解析、存储和匹配功能。

use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::NetError;
use crate::url_parser::ParsedUrl;

/// SameSite 属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// 无限制。
    None,
    /// Lax 模式。
    Lax,
    /// Strict 模式。
    Strict,
}

/// 请求上下文，用于 SameSite 策略判断。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestContext {
    /// 同站请求（包括用户从同一站点发起的导航和子资源请求）。
    SameSite,
    /// 跨站顶层导航（用户从外部站点点击链接进入）。
    CrossSiteTopLevel,
    /// 跨站非导航请求（iframe 嵌入、fetch 等）。
    CrossSiteSubresource,
}

/// HTTP Cookie。
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie 名称。
    pub name: String,
    /// Cookie 值。
    pub value: String,
    /// 域名。
    pub domain: Option<String>,
    /// 是否为 host-only cookie（无显式 Domain 属性）。
    /// host-only cookie 仅精确匹配 host，不匹配子域名（RFC 6265 §5.3）。
    pub host_only: bool,
    /// 路径。
    pub path: Option<String>,
    /// 过期时间戳（从 UNIX epoch 起算的秒数）。
    pub expires: Option<u64>,
    /// 是否仅 HTTPS。
    pub secure: bool,
    /// 是否仅 HTTP（禁止 JS 访问）。
    pub http_only: bool,
    /// SameSite 策略。
    pub same_site: SameSite,
}

impl Cookie {
    /// 判断 Cookie 是否已过期。
    ///
    /// 如果 `expires` 为 `None`，视为会话 Cookie，不会过期。
    /// 如果 `expires` 对应的时间早于当前系统时间，视为已过期。
    pub fn is_expired(&self) -> bool {
        match self.expires {
            None => false,
            Some(secs) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();
                now > secs
            }
        }
    }

    /// 判断 Cookie 在给定时间点是否已过期。
    ///
    /// 主要用于测试：传入一个固定的 `now_secs`（从 UNIX epoch 起算的秒数）。
    pub fn is_expired_at(&self, now_secs: u64) -> bool {
        match self.expires {
            None => false,
            Some(secs) => now_secs > secs,
        }
    }
}

/// 尝试将 Expires 日期字符串解析为 UNIX 时间戳（秒）。
///
/// 支持的日期格式：
/// - RFC 1123: `Wed, 09 Jun 2021 10:18:14 GMT`
/// - RFC 850: `Wednesday, 09-Jun-21 10:18:14 GMT`
/// - ANSI C asctime: `Wed Jun 09 10:18:14 2021`
///
/// 解析失败时返回 `None`。
pub fn parse_expires_date(raw: &str) -> Option<u64> {
    // Month name to number (1-12)
    fn month_to_num(month: &str) -> Option<u32> {
        match month.to_ascii_lowercase().as_str() {
            "jan" => Some(1),
            "feb" => Some(2),
            "mar" => Some(3),
            "apr" => Some(4),
            "may" => Some(5),
            "jun" => Some(6),
            "jul" => Some(7),
            "aug" => Some(8),
            "sep" => Some(9),
            "oct" => Some(10),
            "nov" => Some(11),
            "dec" => Some(12),
            _ => None,
        }
    }

    // 将两位年份转换为四位数（0-68 → 20xx，69-99 → 19xx）
    fn fix_two_digit_year(y: u32) -> u32 {
        if y <= 68 { 2000 + y } else { 1900 + y }
    }

    // 简易的 days_in_year 函数
    fn is_leap_year(y: u32) -> bool {
        (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
    }

    fn days_in_month(y: u32, m: u32) -> u32 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(y) => 29,
            2 => 28,
            _ => 0,
        }
    }

    // 将日期时间转换为 UNIX 时间戳
    fn to_unix_secs(y: u32, m: u32, d: u32, h: u32, min: u32, s: u32) -> u64 {
        let mut days: u64 = 0;
        // 累加完整年份
        for yr in 1970..y {
            days += if is_leap_year(yr) { 366 } else { 365 };
        }
        // 累加完整月份
        for mo in 1..m {
            days += days_in_month(y, mo) as u64;
        }
        days += (d - 1) as u64;
        (days * 86400) + (h as u64 * 3600) + (min as u64 * 60) + s as u64
    }

    /// 校验日期时间字段范围（RFC 7231 §7.1.1.1）。
    ///
    /// R3346 deep-review：旧实现把 day/hour/minute/second 直接喂入 `to_unix_secs` 无范围
    /// 校验——day=0 致 `(d-1)` u32 下溢 **panic**（debug 构建崩溃）；day=32/hour=99/分秒=99
    /// 静默返回错误时间戳。此处统一拦截非法字段返回 None（spec：无效日期丢弃）。
    /// day 上界按所在月实际天数（days_in_month）精确判（30 天月的 31 日、2 月的 30/31 日须拒）。
    fn validate_date_fields(y: u32, m: u32, d: u32, h: u32, min: u32, s: u32) -> Option<()> {
        if d == 0 || d > days_in_month(y, m) {
            return None;
        }
        if h > 23 || min > 59 || s > 59 {
            return None;
        }
        Some(())
    }

    let raw = raw.trim();

    // ANSI C asctime 格式: "Wed Jun 09 10:18:14 2021"
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() == 4 {
        // "Jun 09 10:18:14 2021" → 但 asctime 是 5 部分（含星期名）
        // 这里先跳过，因为上面已排除含逗号的格式
    }

    // 尝试 RFC 1123: "Wed, 09 Jun 2021 10:18:14 GMT"
    if parts.len() == 6 && parts[0].ends_with(',') {
        let day: u32 = u32::from_str(parts[1]).ok()?;
        let month = month_to_num(parts[2])?;
        let year_raw: u32 = u32::from_str(parts[3]).ok()?;
        let year = if year_raw < 100 {
            fix_two_digit_year(year_raw)
        } else {
            year_raw
        };
        let time_parts: Vec<&str> = parts[4].split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }
        let hour: u32 = u32::from_str(time_parts[0]).ok()?;
        let minute: u32 = u32::from_str(time_parts[1]).ok()?;
        let second: u32 = u32::from_str(time_parts[2]).ok()?;
        validate_date_fields(year, month, day, hour, minute, second)?;
        return Some(to_unix_secs(year, month, day, hour, minute, second));
    }

    // 尝试 RFC 850: "Wednesday, 09-Jun-21 10:18:14 GMT"
    if parts.len() == 4 && parts[0].ends_with(',') {
        let day_year: Vec<&str> = parts[1].split('-').collect();
        if day_year.len() != 3 {
            return None;
        }
        let day: u32 = u32::from_str(day_year[0]).ok()?;
        let month = month_to_num(day_year[1])?;
        let year_short: u32 = u32::from_str(day_year[2]).ok()?;
        let year = fix_two_digit_year(year_short);
        let time_parts: Vec<&str> = parts[2].split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }
        let hour: u32 = u32::from_str(time_parts[0]).ok()?;
        let minute: u32 = u32::from_str(time_parts[1]).ok()?;
        let second: u32 = u32::from_str(time_parts[2]).ok()?;
        validate_date_fields(year, month, day, hour, minute, second)?;
        return Some(to_unix_secs(year, month, day, hour, minute, second));
    }

    // 尝试 ANSI C asctime: "Wed Jun  9 10:18:14 2021" (5 部分)
    if parts.len() == 5 {
        // parts: ["Wed", "Jun", "9", "10:18:14", "2021"]
        let month = month_to_num(parts[1])?;
        let day: u32 = u32::from_str(parts[2]).ok()?;
        let year: u32 = u32::from_str(parts[4]).ok()?;
        let time_parts: Vec<&str> = parts[3].split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }
        let hour: u32 = u32::from_str(time_parts[0]).ok()?;
        let minute: u32 = u32::from_str(time_parts[1]).ok()?;
        let second: u32 = u32::from_str(time_parts[2]).ok()?;
        validate_date_fields(year, month, day, hour, minute, second)?;
        return Some(to_unix_secs(year, month, day, hour, minute, second));
    }

    None
}

/// Cookie 存储的最大条目数。
const MAX_COOKIE_COUNT: usize = 4096;

/// Cookie 存储。
pub struct CookieStore {
    cookies: Vec<Cookie>,
}

impl CookieStore {
    /// 创建空的 Cookie 存储。
    pub fn new() -> Self {
        Self { cookies: Vec::new() }
    }

    /// 解析 Set-Cookie header 值。
    ///
    /// 格式：`name=value; Path=/; Secure; HttpOnly; SameSite=Lax`
    pub fn parse_set_cookie(header_value: &str) -> Result<Cookie, NetError> {
        let parts: Vec<&str> = header_value.split(';').collect();
        if parts.is_empty() {
            return Err(NetError::InvalidCookie("empty cookie".to_string()));
        }

        // 解析 name=value
        let first = parts[0].trim();
        let eq_pos = first
            .find('=')
            .ok_or_else(|| NetError::InvalidCookie("no '=' in cookie".to_string()))?;
        let name = first[..eq_pos].trim().to_string();
        let value = first[eq_pos + 1..].trim().to_string();

        // SEC-02: 拒绝含 CRLF/NULL 的 cookie（防止 HTTP 头部注入）
        if name.contains('\r')
            || name.contains('\n')
            || name.contains('\0')
            || value.contains('\r')
            || value.contains('\n')
            || value.contains('\0')
        {
            return Err(NetError::InvalidCookie(
                "cookie contains invalid characters (CRLF/NUL)".to_string(),
            ));
        }

        if name.is_empty() {
            return Err(NetError::InvalidCookie("empty cookie name".to_string()));
        }

        let mut cookie = Cookie {
            name,
            value,
            domain: None,
            host_only: true,
            path: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
        };

        let mut max_age_seen: Option<i64> = None;
        let mut expires_raw: Option<String> = None;

        // 解析属性——RFC 6265 §5.2：属性名 ASCII-大小写不敏感；未知属性忽略。
        // https://www.rfc-editor.org/rfc/rfc6265#section-5.2
        for part in parts.iter().skip(1) {
            let part = part.trim();
            // 无 '=' 的裸布尔属性（Secure / HttpOnly）。
            if part.eq_ignore_ascii_case("secure") {
                cookie.secure = true;
                continue;
            }
            if part.eq_ignore_ascii_case("httponly") {
                cookie.http_only = true;
                continue;
            }
            // key=value 属性：按首个 '=' 切分，属性名小写归一（ASCII-大小写不敏感匹配）。
            let Some(eq) = part.find('=') else {
                continue; // 未知裸属性 → 忽略
            };
            let attr_name = part[..eq].trim().to_ascii_lowercase();
            let attr_val = part[eq + 1..].trim();
            match attr_name.as_str() {
                "path" => cookie.path = Some(attr_val.to_string()),
                "domain" => {
                    cookie.domain = Some(attr_val.to_string());
                    cookie.host_only = false;
                }
                "max-age" => max_age_seen = i64::from_str(attr_val).ok(),
                "expires" => expires_raw = Some(attr_val.to_string()),
                // RFC 6265bis §5.2 + §5.4：SameSite 值 ASCII-大小写不敏感（Strict/Lax/None）；
                // 未识别值 → Default（同 Lax）。
                // https://httpwg.org/http-extensions/draft-ietf-httpbis-rfc6265bis.html
                "samesite" => {
                    cookie.same_site = if attr_val.eq_ignore_ascii_case("strict") {
                        SameSite::Strict
                    } else if attr_val.eq_ignore_ascii_case("lax") {
                        SameSite::Lax
                    } else if attr_val.eq_ignore_ascii_case("none") {
                        SameSite::None
                    } else {
                        SameSite::Lax // 未识别 → Default = Lax
                    };
                }
                _ => {} // 未知属性 → 忽略（RFC 6265 §5.2 step 6）
            }
        }

        // SEC-08: SameSite=None 必须设置 Secure 属性
        if cookie.same_site == SameSite::None && !cookie.secure {
            return Err(NetError::InvalidCookie(
                "SameSite=None cookie must have Secure attribute".to_string(),
            ));
        }

        // Max-Age 优先于 Expires
        if let Some(max_age) = max_age_seen {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            if max_age <= 0 {
                // 立即过期（设置到 epoch 起点）
                cookie.expires = Some(0);
            } else {
                cookie.expires = Some(now_secs.saturating_add(max_age as u64));
            }
        } else if let Some(ref raw) = expires_raw {
            cookie.expires = parse_expires_date(raw);
        }

        Ok(cookie)
    }

    /// 添加 cookie。
    ///
    /// 如果同名同 domain 同 path 的 cookie 已存在，替换旧值。
    /// 如果 cookie 已过期，不会添加。
    ///
    /// **注意**：如果 cookie 的 domain 为 None，则该 cookie 不会匹配任何 URL。
    /// 推荐使用 [`CookieStore::add_from_url`] 来正确设置 domain。
    pub fn add(&mut self, cookie: Cookie) {
        // 不存储已过期的 cookie
        if cookie.is_expired() {
            return;
        }
        // 如果同名同 domain 同 path，替换旧值
        self.cookies
            .retain(|c| !(c.name == cookie.name && c.domain == cookie.domain && c.path == cookie.path));
        self.cookies.push(cookie);

        // SEC-09: 超过最大条目时驱逐过期和最旧 cookie
        if self.cookies.len() > MAX_COOKIE_COUNT {
            self.evict_expired();
        }
        while self.cookies.len() > MAX_COOKIE_COUNT {
            self.cookies.remove(0);
        }
    }

    /// 从指定 URL 接收 cookie 并添加到存储。
    ///
    /// 如果 cookie 无显式 Domain 属性，使用 URL 的 host 作为 domain（host-only cookie），
    /// 遵循 RFC 6265 §5.3；无显式 Path 属性时按 RFC 6265 §5.1.4 计算 default-path。
    /// 推荐使用此方法替代 [`add`](Self::add)。
    pub fn add_from_url(&mut self, mut cookie: Cookie, url: &ParsedUrl) {
        let host = url.host.as_deref().unwrap_or("");
        if cookie.host_only {
            // 无显式 Domain 属性：host-only cookie，域 = 来源 host（RFC 6265 §5.3 step 4-5）。
            cookie.domain = url.host.clone();
        } else {
            // R3225：显式 Domain 须通过 RFC 6265 §5.3 step 5/6 domain-match 校验——来源 host 必须
            // domain-match cookie 的 Domain，否则拒绝（防 evil.com 跨域设 example.com cookie 注入）。
            // IP 字面量 host 不参与子域匹配（§5.1.3）——Domain 须精确等于该 IP。
            let domain_attr = cookie.domain.as_deref().unwrap_or("");
            let domain_ok = if is_ip_literal(host) {
                domain_attr.trim_start_matches('.').eq_ignore_ascii_case(host)
            } else {
                domain_matches(domain_attr, host)
            };
            if !domain_ok {
                return;
            }
        }
        // RFC 6265 §5.1.4 + §5.3 step 6：无显式 Path 属性 → 用 request-uri 的 default-path。
        if cookie.path.is_none() {
            cookie.path = Some(default_path(&url.path));
        }
        self.add(cookie);
    }

    /// 获取匹配 URL 且未过期的所有 cookies。
    pub fn get_for_url(&self, url: &ParsedUrl) -> Vec<&Cookie> {
        self.cookies
            .iter()
            .filter(|c| !c.is_expired() && cookie_matches_url(c, url))
            .collect()
    }

    /// 生成 Cookie header 值，考虑 SameSite 策略。
    ///
    /// `context` 用于判断 SameSite 限制：
    /// - `SameSite::Strict` 仅在 `RequestContext::SameSite` 时发送。
    /// - `SameSite::Lax` 在 `SameSite` 和 `CrossSiteTopLevel`（安全方法）时发送。
    /// - `SameSite::None` 始终发送。
    pub fn cookie_header_with_context(&self, url: &ParsedUrl, context: RequestContext, is_safe_method: bool) -> String {
        self.cookies
            .iter()
            .filter(|c| !c.is_expired() && cookie_matches_url(c, url))
            .filter(|c| same_site_allows(c.same_site, context, is_safe_method))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// 生成 Cookie header 值（不检查 SameSite，兼容旧调用方）。
    pub fn cookie_header(&self, url: &ParsedUrl) -> String {
        self.get_for_url(url)
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// 清除所有已过期的 cookies。
    pub fn evict_expired(&mut self) {
        self.cookies.retain(|c| !c.is_expired());
    }

    /// 清除所有 cookies。
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Cookie 总数。
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Cookie 存储是否为空。
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 判断 SameSite 策略是否允许在给定请求上下文中发送 cookie。
///
/// - `Strict`：仅同站请求允许。
/// - `Lax`：同站请求允许，跨站顶层导航 + 安全方法（GET/HEAD）也允许。
/// - `None`：始终允许。
pub fn same_site_allows(same_site: SameSite, context: RequestContext, is_safe_method: bool) -> bool {
    match same_site {
        SameSite::Strict => context == RequestContext::SameSite,
        SameSite::Lax => {
            context == RequestContext::SameSite || (context == RequestContext::CrossSiteTopLevel && is_safe_method)
        }
        SameSite::None => true,
    }
}

/// 检查 cookie 是否匹配给定 URL。
fn cookie_matches_url(cookie: &Cookie, url: &ParsedUrl) -> bool {
    // Secure cookie 只能用于 HTTPS
    if cookie.secure && url.scheme != "https" {
        return false;
    }

    // 域名匹配
    let host = url.host.as_deref().unwrap_or("");
    match &cookie.domain {
        Some(domain) => {
            if cookie.host_only {
                // SEC-01: host-only cookie 仅精确匹配 host，不匹配子域名（RFC 6265 §5.3）
                if !host.eq_ignore_ascii_case(domain) {
                    return false;
                }
            } else if !domain_matches(domain, host) {
                return false;
            }
        }
        None => {
            // domain=None 且 host_only=true：无法验证，不匹配任何 URL
            return false;
        }
    }

    // 路径匹配（RFC 6265 §5.1.4）
    if let Some(ref cookie_path) = cookie.path {
        if !url.path.starts_with(cookie_path) {
            return false;
        }
        // 路径前缀匹配需要 cookie_path 以 "/" 结尾，
        // 或者请求路径的下一字符是 "/"
        if !cookie_path.ends_with('/') && url.path.len() > cookie_path.len() {
            let next_char = url.path.as_bytes()[cookie_path.len()];
            if next_char != b'/' {
                return false;
            }
        }
    }

    true
}

/// RFC 6265 §5.1.4 default-path——从 request-uri 的 path 计算默认 cookie 路径。
///
/// 算法（等价于规范）：
/// 1. uri-path 为空或不以 `/` 开头 → `/`
/// 2. uri-path 不超过一个 `/` → `/`
/// 3. 否则 → 首字符到最右 `/`（不含）的子串
///
/// 例：`/a/b/c` → `/a/b`，`/a/b/` → `/a/b`，`/foo` → `/`，`""` → `/`。
/// https://www.rfc-editor.org/rfc/rfc6265#section-5.1.4
fn default_path(uri_path: &str) -> String {
    // step 1：空或不以 '/' 开头 → "/"
    if uri_path.is_empty() || !uri_path.starts_with('/') {
        return "/".to_string();
    }
    // step 2/3：找最右 '/'——若为下标 0（仅首字符为 '/'）→ "/"，否则取其前缀。
    match uri_path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => uri_path[..idx].to_string(),
        None => "/".to_string(), // 不可达（上面已保证 starts_with '/'），保守返 "/"
    }
}

/// 判断 host 是否为 IP 字面量（IPv4/IPv6）。IP 不参与 cookie 子域匹配（RFC 6265 §5.1.3）。
/// IPv6 URL host 经 `url` crate 返 `[::1]`（带括号），去括号后判。
fn is_ip_literal(host: &str) -> bool {
    let h = host.strip_prefix('[').and_then(|s| s.strip_suffix(']')).unwrap_or(host);
    h.parse::<std::net::IpAddr>().is_ok()
}

/// 检查域名是否匹配（支持子域名匹配）。
fn domain_matches(cookie_domain: &str, host: &str) -> bool {
    // R3224：空 cookie 域无效——不匹配任何 host（旧 return true 致空域 cookie 匹配全域，安全 smell）。
    if cookie_domain.is_empty() {
        return false;
    }

    let cookie_domain = cookie_domain.trim_start_matches('.');
    let host = host.trim_start_matches('.');

    // 精确匹配
    if host.eq_ignore_ascii_case(cookie_domain) {
        return true;
    }

    // 子域名匹配：host 是 cookie_domain 的子域名
    if host
        .to_ascii_lowercase()
        .ends_with(&format!(".{}", cookie_domain.to_ascii_lowercase()))
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url_parser::parse_url;

    #[test]
    fn test_parse_simple_cookie() {
        let cookie = CookieStore::parse_set_cookie("session=abc123").unwrap();
        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
    }

    #[test]
    fn test_parse_cookie_with_attributes() {
        let cookie = CookieStore::parse_set_cookie("id=42; Path=/app; Domain=example.com").unwrap();
        assert_eq!(cookie.name, "id");
        assert_eq!(cookie.value, "42");
        assert_eq!(cookie.path.as_deref(), Some("/app"));
        assert_eq!(cookie.domain.as_deref(), Some("example.com"));
    }

    #[test]
    fn test_parse_cookie_secure() {
        let cookie = CookieStore::parse_set_cookie("token=xyz; Secure").unwrap();
        assert!(cookie.secure);
    }

    #[test]
    fn test_parse_cookie_httponly() {
        let cookie = CookieStore::parse_set_cookie("jsession=abc; HttpOnly").unwrap();
        assert!(cookie.http_only);
    }

    #[test]
    fn test_parse_cookie_samesite() {
        let cookie = CookieStore::parse_set_cookie("test=1; SameSite=Strict").unwrap();
        assert_eq!(cookie.same_site, SameSite::Strict);

        let cookie2 = CookieStore::parse_set_cookie("test=1; SameSite=Lax").unwrap();
        assert_eq!(cookie2.same_site, SameSite::Lax);

        // SameSite=None without Secure should be rejected
        let result = CookieStore::parse_set_cookie("test=1; SameSite=None");
        assert!(result.is_err(), "SameSite=None without Secure should fail");

        // SameSite=None with Secure should succeed
        let cookie3 = CookieStore::parse_set_cookie("test=1; SameSite=None; Secure").unwrap();
        assert_eq!(cookie3.same_site, SameSite::None);
        assert!(cookie3.secure);
    }

    /// R3218：属性名 ASCII-大小写不敏感（RFC 6265 §5.2）——任意大小写属性名 + SameSite 值。
    #[test]
    fn test_parse_cookie_attribute_case_insensitive() {
        // 属性名任意大小写——`PATH`/`DOMAIN`/`MAX-AGE` 大写不再被静默忽略。
        let c = CookieStore::parse_set_cookie("id=1; PATH=/app; DOMAIN=example.com").unwrap();
        assert_eq!(c.path.as_deref(), Some("/app"), "PATH= 大写属性须解析");
        assert_eq!(c.domain.as_deref(), Some("example.com"), "DOMAIN= 大写属性须解析");
        assert!(!c.host_only, "DOMAIN 属性（任意大小写）须清 host_only");

        // MAX-AGE 大写属性须生效
        let c2 = CookieStore::parse_set_cookie("id=1; MAX-AGE=0").unwrap();
        assert!(c2.is_expired(), "MAX-AGE=0 大写属性须立即过期");

        // SECURE / HTTPONLY 大小写混合
        let c3 = CookieStore::parse_set_cookie("id=1; SeCuRe; HTTPONLY").unwrap();
        assert!(c3.secure, "大小写混合 Secure 须识别");
        assert!(c3.http_only, "HttpOnly 须识别");

        // SameSite 值任意大小写（RFC 6265bis §5.4 step 10）
        let strict = CookieStore::parse_set_cookie("id=1; SameSite=STRICT").unwrap();
        assert_eq!(strict.same_site, SameSite::Strict, "SameSite=STRICT 须 Strict");
        let lax = CookieStore::parse_set_cookie("id=1; SAMESITE=LaX").unwrap();
        assert_eq!(lax.same_site, SameSite::Lax, "SAMESITE=LaX 须 Lax");
        let none = CookieStore::parse_set_cookie("id=1; samesite=NoNe; secure").unwrap();
        assert_eq!(none.same_site, SameSite::None, "samesite=NoNe 须 None");
        assert!(none.secure);

        // 未识别 SameSite 值 → Default = Lax（不要求 Secure）
        let unrecognized = CookieStore::parse_set_cookie("id=1; SameSite=Foo").unwrap();
        assert_eq!(unrecognized.same_site, SameSite::Lax, "未识别 SameSite 须回落 Lax");
    }

    /// R3219：default_path 按 RFC 6265 §5.1.4 计算（直接测 helper）。
    #[test]
    fn test_default_path_rfc_6265_5_1_4() {
        // 空或不以 '/' 开头 → "/"
        assert_eq!(default_path(""), "/");
        assert_eq!(default_path("foo"), "/");
        // 仅一个 '/'（根或单段）→ "/"
        assert_eq!(default_path("/"), "/");
        assert_eq!(default_path("/foo"), "/");
        // 多段路径 → 首字符到最右 '/'（不含）
        assert_eq!(default_path("/a/b/c"), "/a/b");
        assert_eq!(default_path("/a/b/"), "/a/b");
        assert_eq!(default_path("/a/b"), "/a");
    }

    /// R3219：无显式 Path 的 cookie 经 add_from_url 按 default-path 存储，路径匹配遵循之。
    #[test]
    fn test_add_from_url_default_path() {
        let mut store = CookieStore::new();
        // 来自 /app/page.html 的 host-only cookie，无 Path 属性 → default-path=/app
        let cookie = CookieStore::parse_set_cookie("sess=1").unwrap();
        let request_url = parse_url("https://example.com/app/page.html").unwrap();
        store.add_from_url(cookie, &request_url);

        // /app/page（default-path 子路径）匹配
        let m = parse_url("https://example.com/app/page").unwrap();
        assert_eq!(store.get_for_url(&m).len(), 1, "/app/* 应匹配 default-path=/app");

        // /application 不匹配（/app 不是边界前缀）——避免 default-path 缺失导致全路径放行
        let sibling = parse_url("https://example.com/application").unwrap();
        assert!(
            store.get_for_url(&sibling).is_empty(),
            "/application 不应匹配 default-path=/app"
        );

        // /other 不匹配
        let other = parse_url("https://example.com/other").unwrap();
        assert!(
            store.get_for_url(&other).is_empty(),
            "/other 不应匹配 default-path=/app"
        );
    }

    /// R3224：domain_matches 空 cookie 域不匹配任何 host（旧 return true 致空域 cookie 匹配全域）。
    #[test]
    fn test_domain_matches_empty_rejects() {
        assert!(!domain_matches("", "example.com"), "空 cookie 域须不匹配任何 host");
        assert!(!domain_matches("", ""), "空 host + 空 cookie 域须不匹配");
        // 正常路径不受影响。
        assert!(domain_matches("example.com", "example.com"));
        assert!(domain_matches(".example.com", "sub.example.com"));
        assert!(!domain_matches("example.com", "other.com"));
    }

    /// R3225：add_from_url 校验显式 Domain 必须 domain-match 来源 host（RFC 6265 §5.3 step 5/6），
    /// 防跨域 cookie 注入；IP 字面量 host 要求 Domain 精确等于该 IP。
    #[test]
    fn test_add_from_url_domain_validation_r3225() {
        // ① 来源 host 与 Domain 同域 → 接受。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap();
        let url = parse_url("http://example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 1, "Domain=example.com 来自 example.com 须接受");

        // ② 来源 host 是 Domain 的子域 → 接受（sub.example.com domain-match example.com）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("b=2; Domain=example.com").unwrap();
        let url = parse_url("http://sub.example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 1, "Domain=example.com 来自 sub.example.com 须接受（子域）");

        // ③ 来源 host 与 Domain 不 domain-match → 拒绝（跨域注入防御）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("c=3; Domain=evil.com").unwrap();
        let url = parse_url("http://example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 0, "Domain=evil.com 来自 example.com 须拒绝（跨域）");

        // ④ 来源 host 是 Domain 的父域 → 拒绝（example.com 不能为 .com... 实测 example.com 不 domain-match le.com）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("d=4; Domain=sub.example.com").unwrap();
        let url = parse_url("http://example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(
            store.len(),
            0,
            "Domain=sub.example.com 来自 example.com 须拒绝（父域不能设子域 cookie）"
        );

        // ⑤ 空 Domain（Domain=）→ 拒绝（domain_matches 空域 false）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("e=5; Domain=").unwrap();
        let url = parse_url("http://example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 0, "空 Domain= 须拒绝");

        // ⑥ IP 字面量 host：Domain 精确等于 IP → 接受。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("f=6; Domain=192.168.1.1").unwrap();
        let url = parse_url("http://192.168.1.1/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(
            store.len(),
            1,
            "Domain=192.168.1.1 来自 192.168.1.1 须接受（IP 精确匹配）"
        );

        // ⑦ IP 字面量 host：Domain 不等于 IP → 拒绝（IP 不做子域后缀匹配）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("g=7; Domain=168.1.1").unwrap();
        let url = parse_url("http://192.168.1.1/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 0, "IP host 不做后缀匹配，Domain=168.1.1 须拒绝");

        // ⑧ IP 字面量 host：跨域 Domain → 拒绝。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("h=8; Domain=evil.com").unwrap();
        let url = parse_url("http://192.168.1.1/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 0, "IP host + 跨域 Domain 须拒绝");

        // ⑨ host-only cookie（无 Domain）来自任意 host → 接受（domain = 来源 host）。
        let mut store = CookieStore::new();
        let cookie = CookieStore::parse_set_cookie("i=9").unwrap();
        let url = parse_url("http://example.com/").unwrap();
        store.add_from_url(cookie, &url);
        assert_eq!(store.len(), 1, "host-only cookie 须接受（domain=来源 host）");
    }

    /// R3225：is_ip_literal 助函数——IPv4/IPv6（含 URL 括号形式）识别。
    #[test]
    fn test_is_ip_literal() {
        assert!(is_ip_literal("192.168.1.1"));
        assert!(is_ip_literal("10.0.0.1"));
        assert!(is_ip_literal("::1"), "IPv6 裸形式");
        assert!(is_ip_literal("[::1]"), "IPv6 URL 括号形式（url crate 返此）");
        assert!(is_ip_literal("[2001:db8::1]"));
        assert!(!is_ip_literal("example.com"));
        assert!(!is_ip_literal("sub.example.com"));
        assert!(!is_ip_literal(""));
    }

    #[test]
    fn test_cookie_store_add_get() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("b=2; Domain=other.com").unwrap());
        assert_eq!(store.len(), 2);

        let url = parse_url("http://example.com/page").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "a");
    }

    #[test]
    fn test_cookie_store_for_url() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("sess=abc; Domain=example.com; Path=/app").unwrap());

        let matching = parse_url("http://example.com/app/page").unwrap();
        let not_matching = parse_url("http://example.com/other").unwrap();
        let secure_only = parse_url("https://example.com/app").unwrap();

        // Path /app should match /app/page
        assert_eq!(store.get_for_url(&matching).len(), 1);
        // Path /app should NOT match /other
        assert!(store.get_for_url(&not_matching).is_empty());
        // Non-secure cookie should match both http and https
        assert_eq!(store.get_for_url(&secure_only).len(), 1);
    }

    #[test]
    fn test_cookie_header() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("b=2; Domain=example.com").unwrap());

        let url = parse_url("http://example.com/").unwrap();
        let header = store.cookie_header(&url);
        assert!(header.contains("a=1"));
        assert!(header.contains("b=2"));
    }

    #[test]
    fn test_parse_cookie_empty_string() {
        assert!(CookieStore::parse_set_cookie("").is_err());
    }

    #[test]
    fn test_parse_cookie_no_equals() {
        assert!(CookieStore::parse_set_cookie("justacookie").is_err());
    }

    #[test]
    fn test_parse_cookie_empty_name() {
        assert!(CookieStore::parse_set_cookie("=value").is_err());
    }

    #[test]
    fn test_parse_cookie_value_with_equals() {
        let cookie = CookieStore::parse_set_cookie("a=b=c").unwrap();
        assert_eq!(cookie.value, "b=c");
    }

    #[test]
    fn test_parse_cookie_max_age() {
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=3600").unwrap();
        // expires should be a computed future timestamp, not the raw string
        assert!(cookie.expires.is_some());
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Should be approximately now + 3600
        assert!(cookie.expires.unwrap() > now_secs);
        assert!(cookie.expires.unwrap() <= now_secs + 3600 + 1);
    }

    #[test]
    fn test_parse_cookie_max_age_zero_expires_immediately() {
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=0").unwrap();
        // Max-Age=0 means immediately expired → expires = 0
        assert_eq!(cookie.expires, Some(0));
    }

    #[test]
    fn test_parse_cookie_max_age_negative_expires_immediately() {
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=-1").unwrap();
        // Negative Max-Age means immediately expired
        assert_eq!(cookie.expires, Some(0));
    }

    #[test]
    fn test_parse_cookie_expires() {
        let cookie = CookieStore::parse_set_cookie("a=1; Expires=Wed, 09 Jun 2021 10:18:14 GMT").unwrap();
        // The parsed expiry should be a concrete UNIX timestamp
        assert!(cookie.expires.is_some());
        // Wed, 09 Jun 2021 10:18:14 GMT = 1623233894
        assert_eq!(cookie.expires.unwrap(), 1623233894);
    }

    #[test]
    fn test_cookie_store_replace_same_name() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=x.com").unwrap());
        store.add(CookieStore::parse_set_cookie("a=2; Domain=x.com").unwrap());
        assert_eq!(store.len(), 1);
        let url = parse_url("http://x.com/").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies[0].value, "2");
    }

    #[test]
    fn test_cookie_store_clear() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1").unwrap());
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_cookie_store_default() {
        let store = CookieStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn test_cookie_secure_over_http_blocked() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("secret=abc; Secure; Domain=example.com").unwrap());
        let http_url = parse_url("http://example.com/").unwrap();
        assert!(store.get_for_url(&http_url).is_empty());
    }

    #[test]
    fn test_cookie_header_no_match() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=x.com").unwrap());
        let other_url = parse_url("http://other.com/").unwrap();
        let header = store.cookie_header(&other_url);
        assert!(header.is_empty());
    }

    #[test]
    fn test_cookie_domain_subdomain_match() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=.example.com").unwrap());
        let sub_url = parse_url("http://sub.example.com/").unwrap();
        let cookies = store.get_for_url(&sub_url);
        assert_eq!(cookies.len(), 1);
    }

    // ── Cookie expiry tests ──

    #[test]
    fn test_cookie_is_expired_session_cookie() {
        // Session cookie (no expires) is never expired
        let cookie = CookieStore::parse_set_cookie("a=1").unwrap();
        assert!(!cookie.is_expired());
    }

    #[test]
    fn test_cookie_is_expired_future() {
        // Cookie with Max-Age far in the future should not be expired
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=86400").unwrap();
        assert!(!cookie.is_expired());
    }

    #[test]
    fn test_cookie_is_expired_at() {
        // Use is_expired_at to test with a fixed timestamp
        let mut cookie = CookieStore::parse_set_cookie("a=1").unwrap();
        // Manually set expires to timestamp 1000
        cookie.expires = Some(1000);

        // At time 500, not expired
        assert!(!cookie.is_expired_at(500));
        // At time 999, not expired
        assert!(!cookie.is_expired_at(999));
        // At time 1000, not expired (expires AT this time is still valid)
        assert!(!cookie.is_expired_at(1000));
        // At time 1001, expired
        assert!(cookie.is_expired_at(1001));
    }

    #[test]
    fn test_cookie_store_add_expired_cookie_rejected() {
        let mut store = CookieStore::new();
        // Max-Age=0 means immediately expired
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=0").unwrap();
        assert!(cookie.is_expired());
        store.add(cookie);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_cookie_store_evict_expired() {
        let mut store = CookieStore::new();
        // Add a valid cookie
        store.add(CookieStore::parse_set_cookie("valid=1; Domain=example.com").unwrap());
        // Add a cookie and manually expire it
        store.add(CookieStore::parse_set_cookie("expired=2; Domain=example.com").unwrap());
        // Manually set the second cookie to expired
        store.cookies[1].expires = Some(1);
        assert_eq!(store.len(), 2);
        store.evict_expired();
        assert_eq!(store.len(), 1);
        assert_eq!(store.cookies[0].name, "valid");
    }

    #[test]
    fn test_cookie_store_get_for_url_filters_expired() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("good=1; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("bad=2; Domain=example.com").unwrap());
        // Manually expire the second cookie
        store.cookies[1].expires = Some(1);

        let url = parse_url("http://example.com/").unwrap();
        let cookies = store.get_for_url(&url);
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "good");
    }

    #[test]
    fn test_parse_cookie_max_age_priority_over_expires() {
        // When both Max-Age and Expires are present, Max-Age takes priority
        let cookie = CookieStore::parse_set_cookie("a=1; Max-Age=3600; Expires=Wed, 09 Jun 2021 10:18:14 GMT").unwrap();
        // Should use Max-Age (now + 3600), not the past Expires
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(cookie.expires.unwrap() > now_secs);
    }

    // ── Expires date parsing tests ──

    #[test]
    fn test_parse_expires_date_rfc1123() {
        // Wed, 09 Jun 2021 10:18:14 GMT
        let ts = parse_expires_date("Wed, 09 Jun 2021 10:18:14 GMT").unwrap();
        assert_eq!(ts, 1623233894);
    }

    #[test]
    fn test_parse_expires_date_rfc850() {
        // Wednesday, 09-Jun-21 10:18:14 GMT (two-digit year: 21 → 2021)
        let ts = parse_expires_date("Wednesday, 09-Jun-21 10:18:14 GMT").unwrap();
        assert_eq!(ts, 1623233894);
    }

    #[test]
    fn test_parse_expires_date_asctime() {
        // Wed Jun 09 10:18:14 2021
        let ts = parse_expires_date("Wed Jun 09 10:18:14 2021").unwrap();
        assert_eq!(ts, 1623233894);
    }

    #[test]
    fn test_parse_expires_date_invalid() {
        assert!(parse_expires_date("not a date").is_none());
        assert!(parse_expires_date("").is_none());
    }

    #[test]
    fn test_parse_expires_date_two_digit_year_68_is_2068() {
        // 68 → 2068
        let ts = parse_expires_date("Wed, 01 Jan 68 00:00:00 GMT").unwrap();
        let jan_2068 = parse_expires_date("Mon, 01 Jan 2068 00:00:00 GMT").unwrap();
        assert_eq!(ts, jan_2068);
    }

    #[test]
    fn test_parse_expires_date_two_digit_year_69_is_1969() {
        // 69 → 1969 (before epoch, should still produce a small number)
        let ts = parse_expires_date("Tue, 01 Jan 69 00:00:00 GMT").unwrap();
        // 1969 is before epoch, so the timestamp wraps or is small
        // This just verifies it parses without panic
        assert!(ts <= 365 * 86400); // Should be a small number
    }

    // ── SameSite enforcement tests ──

    #[test]
    fn test_same_site_strict_allowed_same_site() {
        assert!(same_site_allows(SameSite::Strict, RequestContext::SameSite, true));
    }

    #[test]
    fn test_same_site_strict_blocked_cross_site_top_level() {
        assert!(!same_site_allows(
            SameSite::Strict,
            RequestContext::CrossSiteTopLevel,
            true
        ));
    }

    #[test]
    fn test_same_site_strict_blocked_cross_site_subresource() {
        assert!(!same_site_allows(
            SameSite::Strict,
            RequestContext::CrossSiteSubresource,
            true
        ));
    }

    #[test]
    fn test_same_site_lax_allowed_same_site() {
        assert!(same_site_allows(SameSite::Lax, RequestContext::SameSite, true));
    }

    #[test]
    fn test_same_site_lax_allowed_cross_site_top_level_safe_method() {
        assert!(same_site_allows(SameSite::Lax, RequestContext::CrossSiteTopLevel, true));
    }

    #[test]
    fn test_same_site_lax_blocked_cross_site_top_level_unsafe_method() {
        assert!(!same_site_allows(
            SameSite::Lax,
            RequestContext::CrossSiteTopLevel,
            false
        ));
    }

    #[test]
    fn test_same_site_lax_blocked_cross_site_subresource() {
        assert!(!same_site_allows(
            SameSite::Lax,
            RequestContext::CrossSiteSubresource,
            true
        ));
    }

    #[test]
    fn test_same_site_none_always_allowed() {
        assert!(same_site_allows(SameSite::None, RequestContext::SameSite, true));
        assert!(same_site_allows(
            SameSite::None,
            RequestContext::CrossSiteTopLevel,
            false
        ));
        assert!(same_site_allows(
            SameSite::None,
            RequestContext::CrossSiteSubresource,
            false
        ));
    }

    #[test]
    fn test_cookie_header_with_context_strict_same_site() {
        let mut store = CookieStore::new();
        store.add(
            CookieStore::parse_set_cookie("strict_cookie=v1; Domain=example.com; SameSite=Strict; Secure").unwrap(),
        );
        store.add(CookieStore::parse_set_cookie("none_cookie=v2; Domain=example.com; SameSite=None; Secure").unwrap());

        let url = parse_url("https://example.com/").unwrap();

        // Same-site: both sent
        let header = store.cookie_header_with_context(&url, RequestContext::SameSite, true);
        assert!(header.contains("strict_cookie=v1"));
        assert!(header.contains("none_cookie=v2"));

        // Cross-site top-level: only None
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, true);
        assert!(!header.contains("strict_cookie"));
        assert!(header.contains("none_cookie=v2"));
    }

    #[test]
    fn test_cookie_header_with_context_lax_top_level_get() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("lax_cookie=v1; Domain=example.com; SameSite=Lax").unwrap());

        let url = parse_url("http://example.com/").unwrap();

        // Cross-site top-level GET: Lax is allowed
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, true);
        assert!(header.contains("lax_cookie=v1"));

        // Cross-site top-level POST: Lax is blocked
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, false);
        assert!(header.is_empty());
    }

    #[test]
    fn test_cookie_header_with_context_cross_site_subresource() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("lax_cookie=v1; Domain=example.com; SameSite=Lax; Secure").unwrap());
        store.add(
            CookieStore::parse_set_cookie("strict_cookie=v2; Domain=example.com; SameSite=Strict; Secure").unwrap(),
        );
        store.add(CookieStore::parse_set_cookie("none_cookie=v3; Domain=example.com; SameSite=None; Secure").unwrap());

        let url = parse_url("https://example.com/").unwrap();

        // Cross-site subresource: only None allowed
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteSubresource, true);
        assert!(!header.contains("lax_cookie"));
        assert!(!header.contains("strict_cookie"));
        assert!(header.contains("none_cookie=v3"));
    }

    // ── Cookie security tests ──

    /// Secure cookie 只在 HTTPS 下发送。
    #[test]
    fn test_secure_cookie_only_over_https() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("secret=abc; Secure; Domain=example.com").unwrap());

        let http_url = parse_url("http://example.com/").unwrap();
        let https_url = parse_url("https://example.com/").unwrap();

        assert!(
            store.get_for_url(&http_url).is_empty(),
            "Secure cookie 不应通过 HTTP 发送"
        );
        assert_eq!(
            store.get_for_url(&https_url).len(),
            1,
            "Secure cookie 应通过 HTTPS 发送"
        );
    }

    /// HttpOnly 属性正确解析（不可通过脚本访问）。
    #[test]
    fn test_httponly_flag_prevents_script_access() {
        let cookie = CookieStore::parse_set_cookie("sid=secret; HttpOnly; Path=/").unwrap();
        assert!(cookie.http_only, "HttpOnly cookie 的 http_only 应为 true");
        assert_eq!(cookie.path.as_deref(), Some("/"));
    }

    /// SameSite=Strict 阻止跨站请求。
    #[test]
    fn test_samesite_strict_blocks_cross_site_request() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("auth=token; Domain=example.com; SameSite=Strict").unwrap());

        let url = parse_url("http://example.com/").unwrap();

        // 同站请求：发送
        let header = store.cookie_header_with_context(&url, RequestContext::SameSite, true);
        assert!(header.contains("auth=token"));

        // 跨站顶层导航：阻止
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, true);
        assert!(!header.contains("auth"));

        // 跨站子资源：阻止
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteSubresource, true);
        assert!(!header.contains("auth"));
    }

    /// SameSite=Lax 允许顶层安全方法导航。
    #[test]
    fn test_samesite_lax_allows_top_level_navigation() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("theme=dark; Domain=example.com; SameSite=Lax").unwrap());

        let url = parse_url("http://example.com/").unwrap();

        // 同站请求：发送
        let header = store.cookie_header_with_context(&url, RequestContext::SameSite, true);
        assert!(header.contains("theme=dark"));

        // 跨站顶层 GET 导航：发送
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, true);
        assert!(header.contains("theme=dark"), "Lax 应允许跨站顶层安全方法导航");

        // 跨站顶层 POST：阻止
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, false);
        assert!(!header.contains("theme"), "Lax 应阻止跨站不安全方法");
    }

    /// Cookie path 匹配：子路径匹配、不匹配兄弟路径。
    #[test]
    fn test_cookie_path_matching() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("app_sess=1; Path=/app; Domain=example.com; SameSite=Lax").unwrap());

        let matching = parse_url("http://example.com/app/page").unwrap();
        let matching_exact = parse_url("http://example.com/app").unwrap();
        // /application should NOT match Path=/app (RFC 6265 §5.1.4)
        let matching_prefix = parse_url("http://example.com/application").unwrap();
        let not_matching_parent = parse_url("http://example.com/other").unwrap();
        let not_matching_root = parse_url("http://example.com/").unwrap();

        assert_eq!(store.get_for_url(&matching).len(), 1, "/app/page 应匹配 Path=/app");
        assert_eq!(store.get_for_url(&matching_exact).len(), 1, "/app 应匹配 Path=/app");
        assert_eq!(
            store.get_for_url(&matching_prefix).len(),
            0,
            "/application 不应匹配 Path=/app（RFC 6265）"
        );
        assert!(
            store.get_for_url(&not_matching_parent).is_empty(),
            "/other 不应匹配 Path=/app"
        );
        assert!(store.get_for_url(&not_matching_root).is_empty(), "/ 不应匹配 Path=/app");
    }

    /// Cookie domain 匹配：精确匹配和子域名匹配。
    #[test]
    fn test_cookie_domain_matching_exact_and_subdomain() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("id=42; Domain=example.com").unwrap());

        let exact = parse_url("http://example.com/").unwrap();
        let sub = parse_url("http://sub.example.com/").unwrap();
        let other = parse_url("http://notexample.com/").unwrap();

        assert_eq!(store.get_for_url(&exact).len(), 1, "精确域名应匹配");
        assert_eq!(store.get_for_url(&sub).len(), 1, "子域名应匹配");
        assert!(store.get_for_url(&other).is_empty(), "不相关域名不应匹配");
    }

    /// 不同 Path 的同名 Cookie 是独立存储的。
    #[test]
    fn test_cookie_different_path_same_name_stored_separately() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("lang=en; Path=/en; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("lang=zh; Path=/zh; Domain=example.com").unwrap());

        assert_eq!(store.len(), 2, "不同 path 同名 cookie 应独立存储");

        let en_url = parse_url("http://example.com/en/page").unwrap();
        let zh_url = parse_url("http://example.com/zh/page").unwrap();

        let en_cookies = store.get_for_url(&en_url);
        assert_eq!(en_cookies.len(), 1);
        assert_eq!(en_cookies[0].value, "en");

        let zh_cookies = store.get_for_url(&zh_url);
        assert_eq!(zh_cookies.len(), 1);
        assert_eq!(zh_cookies[0].value, "zh");
    }

    /// 第三方 Cookie：SameSite=None 且 Secure=true，可在所有请求上下文中发送。
    #[test]
    fn test_cookie_third_party_attribute() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("tracking=xyz; SameSite=None; Secure; Domain=example.com").unwrap());

        assert_eq!(store.len(), 1);
        let cookie = &store.cookies[0];
        assert_eq!(cookie.same_site, SameSite::None);
        assert!(cookie.secure, "第三方 Cookie 必须标记 Secure");

        // HTTPS 下所有请求上下文均发送
        let url = parse_url("https://example.com/").unwrap();
        for ctx in [
            RequestContext::SameSite,
            RequestContext::CrossSiteTopLevel,
            RequestContext::CrossSiteSubresource,
        ] {
            let header = store.cookie_header_with_context(&url, ctx, false);
            assert!(
                header.contains("tracking=xyz"),
                "SameSite=None 应在 {ctx:?} 上下文中发送"
            );
        }

        // HTTP 下不发送（Secure 限制）
        let http_url = parse_url("http://example.com/").unwrap();
        assert!(
            store.get_for_url(&http_url).is_empty(),
            "Secure cookie 不应通过 HTTP 发送"
        );
    }

    /// Cookie with Secure attribute → only sent over HTTPS.
    /// 验证 Secure 属性的 Cookie 仅通过 HTTPS 发送，通过 HTTP 时不发送。
    #[test]
    fn test_cookie_secure_attribute() {
        let cookie = CookieStore::parse_set_cookie("token=secret; Secure; Domain=example.com").unwrap();
        assert!(cookie.secure, "Secure 属性应被正确解析");

        let mut store = CookieStore::new();
        store.add(cookie);

        // HTTP 请求：Secure cookie 不应发送
        let http_url = parse_url("http://example.com/page").unwrap();
        assert!(
            store.get_for_url(&http_url).is_empty(),
            "Secure cookie 不应通过 HTTP 发送"
        );

        // HTTPS 请求：Secure cookie 应发送
        let https_url = parse_url("https://example.com/page").unwrap();
        assert_eq!(
            store.get_for_url(&https_url).len(),
            1,
            "Secure cookie 应通过 HTTPS 发送"
        );
    }

    /// 会话 Cookie：无 Max-Age 和 Expires 的 Cookie 永不过期，且 expires 为 None。
    #[test]
    fn test_cookie_session_cookie_no_expiry() {
        let cookie = CookieStore::parse_set_cookie("sess=abc123; Path=/").unwrap();

        // 无 Max-Age 和 Expires → expires 应为 None（会话 Cookie）
        assert!(cookie.expires.is_none(), "会话 Cookie 的 expires 应为 None");
        assert!(!cookie.is_expired(), "会话 Cookie 不应过期");
        assert!(!cookie.is_expired_at(u64::MAX), "会话 Cookie 在任意时间点都不应过期");

        // 能正常存储和检索
        let mut store = CookieStore::new();
        store.add(cookie);
        assert_eq!(store.len(), 1, "会话 Cookie 应被存储");

        let url = parse_url("http://example.com/").unwrap();
        // cookie 没设 domain，不会匹配 example.com，所以这里再测一个有 domain 的
        let mut store2 = CookieStore::new();
        store2.add(CookieStore::parse_set_cookie("sess=abc; Domain=example.com").unwrap());
        assert_eq!(store2.get_for_url(&url).len(), 1, "会话 Cookie 应能被检索");
    }

    // ── 高优先级 SameSite 完整组合测试 ──

    /// 验证三种 SameSite 模式 × 三种请求上下文的完整组合行为。
    /// 使用 cookie_header_with_context 进行端到端验证。
    #[test]
    fn test_samesite_full_matrix() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("strict_ck=s; Domain=example.com; SameSite=Strict; Secure").unwrap());
        store.add(CookieStore::parse_set_cookie("lax_ck=l; Domain=example.com; SameSite=Lax; Secure").unwrap());
        store.add(CookieStore::parse_set_cookie("none_ck=n; Domain=example.com; SameSite=None; Secure").unwrap());

        let url = parse_url("https://example.com/").unwrap();

        // 同站请求（安全方法）：三种 cookie 都发送
        let header = store.cookie_header_with_context(&url, RequestContext::SameSite, true);
        assert!(header.contains("strict_ck=s"), "SameSite: Strict 应在同站发送");
        assert!(header.contains("lax_ck=l"), "SameSite: Lax 应在同站发送");
        assert!(header.contains("none_ck=n"), "SameSite: None 应在同站发送");

        // 跨站顶层导航（安全方法）：Lax 和 None 发送，Strict 不发送
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, true);
        assert!(!header.contains("strict_ck"), "SameSite: Strict 不应在跨站顶层发送");
        assert!(header.contains("lax_ck=l"), "SameSite: Lax 应在跨站顶层安全方法发送");
        assert!(header.contains("none_ck=n"), "SameSite: None 应在跨站顶层发送");

        // 跨站顶层导航（不安全方法）：仅 None 发送
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteTopLevel, false);
        assert!(
            !header.contains("strict_ck"),
            "SameSite: Strict 不应在跨站不安全方法发送"
        );
        assert!(!header.contains("lax_ck"), "SameSite: Lax 不应在跨站不安全方法发送");
        assert!(header.contains("none_ck=n"), "SameSite: None 应在跨站不安全方法发送");

        // 跨站子资源：仅 None 发送
        let header = store.cookie_header_with_context(&url, RequestContext::CrossSiteSubresource, true);
        assert!(!header.contains("strict_ck"), "SameSite: Strict 不应在跨站子资源发送");
        assert!(!header.contains("lax_ck"), "SameSite: Lax 不应在跨站子资源发送");
        assert!(header.contains("none_ck=n"), "SameSite: None 应在跨站子资源发送");
    }

    // ── 边界测试 ──

    #[test]
    /// 测试同名 Cookie 不同域名分别存储。
    fn test_cookie_store_same_name_different_domain() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("a=1; Domain=x.com").unwrap());
        store.add(CookieStore::parse_set_cookie("a=2; Domain=y.com").unwrap());
        assert_eq!(store.len(), 2, "same name + different domain → 2 entries");
    }

    #[test]
    /// 测试 cookie_header 多 Cookie 排序（保持插入顺序）。
    fn test_cookie_header_multiple_ordering() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("b=2; Domain=example.com").unwrap());
        store.add(CookieStore::parse_set_cookie("a=1; Domain=example.com").unwrap());
        let header = store.cookie_header(&parse_url("https://example.com/path").unwrap());
        assert!(header.contains("b=2"), "b=2 should be present");
        assert!(header.contains("a=1"), "a=1 should be present");
        // 插入顺序：b 先于 a
        let b_pos = header.find("b=2").unwrap();
        let a_pos = header.find("a=1").unwrap();
        assert!(b_pos < a_pos, "b=2 should appear before a=1");
    }

    #[test]
    /// 测试空 host URL 不匹配 Cookie domain。
    fn test_cookie_domain_empty_host() {
        let mut store = CookieStore::new();
        store.add(CookieStore::parse_set_cookie("test=1; Domain=example.com").unwrap());
        // data: URL 无 host
        let header = store.cookie_header(&parse_url("data:text/html,hello").unwrap());
        assert!(!header.contains("test=1"), "cookie should not match empty-host URL");
    }
}
