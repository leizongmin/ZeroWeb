//! Fetch 优先级 — 对齐 [`zero_engine::preload::LoadPriority`] 语义。

/// 资源 fetch 优先级（数值越大越优先）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FetchPriority(u8);

impl FetchPriority {
    /// 最低（prefetch / idle）。
    pub const IDLE: Self = Self(0);
    /// 低。
    pub const LOW: Self = Self(1);
    /// 默认子资源。
    pub const MEDIUM: Self = Self(2);
    /// preload / 关键子资源。
    pub const HIGH: Self = Self(3);
    /// 主文档、blocking CSS。
    pub const CRITICAL: Self = Self(4);

    /// 从 IPC / 内部 meta 数值解析（clamp 0–4）。
    pub fn from_u8(v: u8) -> Self {
        Self(v.min(4))
    }

    /// 按资源类型推断默认优先级。
    pub fn infer(resource_type: &str) -> Self {
        match resource_type {
            "document" => Self::CRITICAL,
            "style" | "stylesheet" => Self::CRITICAL,
            "script" => Self::HIGH,
            "font" => Self::HIGH,
            "image" | "img" => Self::MEDIUM,
            "prefetch" => Self::LOW,
            "preconnect" | "dns-prefetch" => Self::IDLE,
            _ => Self::MEDIUM,
        }
    }

    /// 从 [`FetchParams`](zero_protocol::message::FetchParams) 的自定义头解析。
    pub fn from_fetch_headers(headers: &[(String, String)], url: &str) -> (Self, &'static str) {
        let mut resource_type = "";
        let mut priority = None;
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("x-zero-resource-type") {
                resource_type = match v.as_str() {
                    "document" => "document",
                    "style" | "stylesheet" => "style",
                    "script" => "script",
                    "font" => "font",
                    "image" | "img" => "image",
                    "prefetch" => "prefetch",
                    _ => "other",
                };
            }
            if k.eq_ignore_ascii_case("x-zero-priority")
                && let Ok(n) = v.parse::<u8>()
            {
                priority = Some(Self::from_u8(n));
            }
        }
        let rt = if resource_type.is_empty() {
            infer_resource_type_from_url(url)
        } else {
            resource_type
        };
        let pri = priority.unwrap_or_else(|| Self::infer(rt));
        (pri, rt)
    }
}

/// 从 URL 路径推断资源类型（无显式 hint 时的兜底）。
pub fn infer_resource_type_from_url(url: &str) -> &'static str {
    // RFC 3986：fragment（`#...`）不属于 path，且查询串（`?...`）也不参与扩展名；
    // 二者都先剥离，避免 `app.js#frag` 因 `#frag` 后缀被误判为 document。
    let no_frag = url.split('#').next().unwrap_or(url);
    let path = no_frag
        .split('?')
        .next()
        .unwrap_or(no_frag)
        .rsplit('/')
        .next()
        .unwrap_or("");
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".css") {
        "style"
    } else if lower.ends_with(".js") || lower.ends_with(".mjs") {
        "script"
    } else if lower.ends_with(".woff2")
        || lower.ends_with(".woff")
        || lower.ends_with(".ttf")
        || lower.ends_with(".otf")
    {
        "font"
    } else if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".svg")
    {
        "image"
    } else {
        "document"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_css_is_critical() {
        assert_eq!(FetchPriority::infer("style"), FetchPriority::CRITICAL);
    }

    #[test]
    fn parse_custom_headers() {
        let headers = vec![
            ("X-Zero-Resource-Type".into(), "script".into()),
            ("X-Zero-Priority".into(), "3".into()),
        ];
        let (p, rt) = FetchPriority::from_fetch_headers(&headers, "https://x/a.js");
        assert_eq!(rt, "script");
        assert_eq!(p, FetchPriority::HIGH);
    }

    // ── R3384：infer_resource_type_from_url 测试加固 + fragment 修复回归 ──

    #[test]
    fn infer_resource_type_extensions_case_insensitive() {
        assert_eq!(infer_resource_type_from_url("https://x/A.CSS"), "style");
        assert_eq!(infer_resource_type_from_url("https://x/A.Js"), "script");
        assert_eq!(infer_resource_type_from_url("https://x/x.WOFF2"), "font");
        assert_eq!(infer_resource_type_from_url("https://x/IMG.PNG"), "image");
    }

    #[test]
    fn infer_resource_type_query_string_ignored() {
        assert_eq!(infer_resource_type_from_url("https://x/a.js?v=2"), "script");
        assert_eq!(infer_resource_type_from_url("https://x/a.css?cache=bust"), "style");
    }

    /// R3384 回归锁定：fragment（#frag）须被剥离，否则 `app.js#frag` 因
    /// 后缀 `.js#frag` 不匹配 `.js` 被误判为 document（修复前确证 bug）。
    #[test]
    fn infer_resource_type_fragment_stripped_r3384() {
        assert_eq!(infer_resource_type_from_url("https://x/app.js#frag"), "script");
        assert_eq!(infer_resource_type_from_url("https://x/a.css#section"), "style");
        assert_eq!(infer_resource_type_from_url("https://x/a.js?b=1#frag"), "script");
        assert_eq!(infer_resource_type_from_url("https://x/pic.png#"), "image");
    }

    #[test]
    fn infer_resource_type_trailing_slash_is_document() {
        // 目录路径（无扩展名 basename）→ document。
        assert_eq!(infer_resource_type_from_url("https://x/dir/"), "document");
        assert_eq!(infer_resource_type_from_url("https://x/"), "document");
    }
}
