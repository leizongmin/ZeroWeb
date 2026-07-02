//! 字体 fallback 策略（spec IF-008 `fallback_chain` 的纯逻辑辅助）。
//!
//! `DefaultFallbackPolicy` 给 FontProvider 实现提供一个一致的通用族兜底顺序，
//! 保证 UI 与 WebView 都得到一致的 fallback chain（DC-11）。

use crate::font_request::{FontFamily, FontRequest};

/// 默认 fallback 策略：请求族 → 泛型无衬线 → 衬线 → 等宽 → emoji 兜底。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultFallbackPolicy;

impl DefaultFallbackPolicy {
    /// 生成有序 fallback 族列表（去重，保留请求族在前）。
    pub fn fallback_families(request: &FontRequest) -> Vec<FontFamily> {
        let mut out: Vec<FontFamily> = request.families.clone();
        let push_if_absent = |out: &mut Vec<FontFamily>, fam: &str| {
            let f = FontFamily::new(fam);
            if !out.iter().any(|x| x.0.eq_ignore_ascii_case(&f.0)) {
                out.push(f);
            }
        };
        push_if_absent(&mut out, "sans-serif");
        push_if_absent(&mut out, "serif");
        push_if_absent(&mut out, "monospace");
        push_if_absent(&mut out, "Apple Color Emoji");
        push_if_absent(&mut out, "Segoe UI Emoji");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_appends_generic_in_order() {
        let req = FontRequest::new("Inter");
        let chain = DefaultFallbackPolicy::fallback_families(&req);
        // 请求族在前，泛型族在后。
        assert_eq!(chain[0], FontFamily::new("Inter"));
        assert!(chain.iter().any(|f| f.0 == "sans-serif"));
        assert!(chain.iter().any(|f| f.0 == "monospace"));
        // emoji 兜底存在。
        assert!(chain.iter().any(|f| f.0.contains("Emoji")));
    }

    #[test]
    fn fallback_dedupes_case_insensitive() {
        let mut req = FontRequest::new("Sans-Serif");
        req.families.clear();
        req.families.push(FontFamily::new("sans-serif"));
        let chain = DefaultFallbackPolicy::fallback_families(&req);
        let sans_count = chain.iter().filter(|f| f.0.eq_ignore_ascii_case("sans-serif")).count();
        assert_eq!(sans_count, 1, "duplicate sans-serif must be deduped");
    }
}
