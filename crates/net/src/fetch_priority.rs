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
    let path = url.split('?').next().unwrap_or(url).rsplit('/').next().unwrap_or("");
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
}
