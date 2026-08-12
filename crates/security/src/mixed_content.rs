//! 混合内容检测模块。
//!
//! 当 HTTPS 页面加载 HTTP 子资源时，检测并阻止混合内容。

use crate::origin::Origin;

/// 混合内容类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedContentType {
    /// 升级混合内容（image、audio、video 等可自动升级）。
    OptionallyBlockable,
    /// 阻塞型混合内容（script、iframe、XHR 等必须阻止）。
    Blockable,
}

/// 混合内容检查结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MixedContentStatus {
    /// 非混合内容（同协议或非安全上下文）。
    NotMixedContent,
    /// 可升级的混合内容（应警告但可尝试升级为 HTTPS）。
    OptionallyBlockable,
    /// 必须阻止的混合内容。
    Blockable,
}

/// 判断是否为混合内容场景。
///
/// 当页面源为 HTTPS 而资源为 HTTP 时，属于混合内容。
///
/// `page_origin` 为页面源。
/// `resource_url` 为资源 URL。
///
/// R3343：scheme 大小写不敏感（RFC 3986 §3.1）。生产链路（fetch_proxy）传入的 URL 未经
/// 规范化，`HTTP://`/`HtTp://` 须等价于 `http://`，否则阻塞型混合内容绕过检测。
pub fn is_mixed_content(page_origin: &Origin, resource_url: &str) -> bool {
    if !page_origin.is_secure() {
        return false;
    }
    // 大小写不敏感匹配 `http://`：取前 7 字节比对（避免对长 URL 整串 to_lowercase 分配）。
    resource_url.len() >= 7 && resource_url.as_bytes()[..7].eq_ignore_ascii_case(b"http://")
}

/// 检查混合内容状态。
///
/// `page_origin` 为页面源。
/// `resource_url` 为资源 URL。
/// `resource_type` 为资源类型（如 "script", "img", "style", "connect", "font", "media", "object"）。
pub fn check_mixed_content(page_origin: &Origin, resource_url: &str, resource_type: &str) -> MixedContentStatus {
    if !is_mixed_content(page_origin, resource_url) {
        return MixedContentStatus::NotMixedContent;
    }

    match classify_resource_type(resource_type) {
        MixedContentType::OptionallyBlockable => MixedContentStatus::OptionallyBlockable,
        MixedContentType::Blockable => MixedContentStatus::Blockable,
    }
}

/// 将 HTTP URL 升级为 HTTPS（用于可升级混合内容）。
///
/// `url` 必须以 `http://` 开头（scheme 大小写不敏感，R3343）。
pub fn upgrade_to_https(url: &str) -> Option<String> {
    // 大小写不敏感剥离 `http://` 前缀，输出规范化为小写 `https://`。
    if url.len() >= 7 && url.as_bytes()[..7].eq_ignore_ascii_case(b"http://") {
        Some(format!("https://{}", &url[7..]))
    } else {
        None
    }
}

/// 根据资源类型分类混合内容。
fn classify_resource_type(resource_type: &str) -> MixedContentType {
    // 可选阻塞的混合内容类型
    let optionally_blockable = ["img", "audio", "video", "media"];
    if optionally_blockable.contains(&resource_type) {
        return MixedContentType::OptionallyBlockable;
    }

    // 所有其他（script, style, connect, font, object, iframe, frame, xhr, fetch 等）为阻塞型
    MixedContentType::Blockable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_mixed_content_https_page_https_resource() {
        let page = Origin::parse("https://example.com").unwrap();
        assert!(!is_mixed_content(&page, "https://cdn.example.com/script.js"));
    }

    #[test]
    fn test_mixed_content_https_page_http_resource() {
        let page = Origin::parse("https://example.com").unwrap();
        assert!(is_mixed_content(&page, "http://cdn.example.com/script.js"));
    }

    #[test]
    fn test_not_mixed_content_http_page() {
        let page = Origin::parse("http://example.com").unwrap();
        assert!(!is_mixed_content(&page, "http://cdn.example.com/script.js"));
    }

    #[test]
    fn test_not_mixed_content_relative_url() {
        let page = Origin::parse("https://example.com").unwrap();
        assert!(!is_mixed_content(&page, "script.js"));
    }

    #[test]
    fn test_mixed_content_check_blockable_script() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://evil.com/script.js", "script");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    #[test]
    fn test_mixed_content_check_blockable_style() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://evil.com/style.css", "style");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    #[test]
    fn test_mixed_content_check_blockable_connect() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://api.example.com/data", "connect");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    #[test]
    fn test_mixed_content_check_optionally_blockable_img() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/photo.jpg", "img");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);
    }

    #[test]
    fn test_mixed_content_check_optionally_blockable_audio() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/audio.mp3", "audio");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);
    }

    #[test]
    fn test_mixed_content_check_optionally_blockable_video() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/video.mp4", "video");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);
    }

    #[test]
    fn test_mixed_content_not_mixed_content_https_resource() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "https://cdn.example.com/script.js", "script");
        assert_eq!(status, MixedContentStatus::NotMixedContent);
    }

    #[test]
    fn test_mixed_content_not_mixed_http_page() {
        let page = Origin::parse("http://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/script.js", "script");
        assert_eq!(status, MixedContentStatus::NotMixedContent);
    }

    #[test]
    fn test_upgrade_to_https() {
        assert_eq!(
            upgrade_to_https("http://example.com/script.js"),
            Some("https://example.com/script.js".to_string())
        );
    }

    #[test]
    fn test_upgrade_to_https_non_http() {
        assert_eq!(upgrade_to_https("https://example.com/script.js"), None);
        assert_eq!(upgrade_to_https("script.js"), None);
    }

    #[test]
    fn test_mixed_content_check_media_type() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/media.mp4", "media");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);
    }

    #[test]
    fn test_mixed_content_check_font_blockable() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/font.woff2", "font");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    #[test]
    fn test_mixed_content_check_object_blockable() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/flash.swf", "object");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    // ---- 混合内容：不同资源类型的差异化处理 ----

    #[test]
    fn test_mixed_content_blocks_http_script_on_https() {
        // HTTPS 页面加载 HTTP script → 必须阻止
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/app.js", "script");
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    #[test]
    fn test_mixed_content_img_upgradeable_on_https() {
        // HTTPS 页面加载 HTTP img → 可升级（OptionallyBlockable）
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/photo.jpg", "img");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);
        // 可升级为 HTTPS
        assert_eq!(
            upgrade_to_https("http://cdn.example.com/photo.jpg"),
            Some("https://cdn.example.com/photo.jpg".to_string())
        );
    }

    // ---- 混合内容：特殊 URI 方案和大小写 ----

    #[test]
    fn test_mixed_content_data_uri_not_flagged() {
        // data: URI 不是混合内容（即使页面是 HTTPS）
        let page = Origin::parse("https://example.com").unwrap();
        assert!(!is_mixed_content(&page, "data:text/html,<h1>Hello</h1>"));
    }

    #[test]
    fn test_mixed_content_blob_uri_not_flagged() {
        // blob: URI 不是混合内容
        let page = Origin::parse("https://example.com").unwrap();
        assert!(!is_mixed_content(&page, "blob:https://example.com/abc-123"));
    }

    #[test]
    fn test_mixed_content_javascript_uri_not_flagged() {
        // javascript: URI 不是混合内容（但有其他安全隐患）
        let page = Origin::parse("https://example.com").unwrap();
        assert!(!is_mixed_content(&page, "javascript:alert(1)"));
    }

    #[test]
    fn test_mixed_content_case_insensitive_http_scheme() {
        // HTTP URL 的大写变体也应被识别为混合内容
        let page = Origin::parse("https://example.com").unwrap();
        // 当前实现对 http:// 前缀检查是大小写敏感的
        // HTTP:// 和 Http:// 等变体也应该被检测到
        // 注意：如果 URL 在调用前已被规范化为小写，则此测试通过
        // 如果未规范化，则这是一个需要修复的 bug
        assert!(
            is_mixed_content(&page, "http://cdn.example.com/script.js"),
            "小写 http:// 应被检测为混合内容"
        );
    }

    /// 测试已经是 HTTPS 的 URL 不需要升级（upgrade_to_https 返回 None）。
    #[test]
    fn test_upgrade_to_https_already_https() {
        // HTTPS URL → 无需升级，返回 None
        assert_eq!(upgrade_to_https("https://example.com/script.js"), None);
        // 其他非 http:// 协议 → 也返回 None
        assert_eq!(upgrade_to_https("data:text/html,<h1>Hi</h1>"), None);
    }

    /// 测试 "worker" 资源类型被归类为 Blockable（阻塞型混合内容）。
    #[test]
    fn test_mixed_content_worker_type() {
        let page = Origin::parse("https://example.com").unwrap();
        let status = check_mixed_content(&page, "http://cdn.example.com/worker.js", "worker");
        // worker 不在可选阻塞列表中，应为 Blockable
        assert_eq!(status, MixedContentStatus::Blockable);
    }

    /// 测试 upgrade-insecure-requests 将 HTTP 图片资源升级为 HTTPS。
    ///
    /// 当页面使用 CSP upgrade-insecure-requests 指令时，
    /// OptionallyBlockable 类型的混合内容（如 img）应通过
    /// upgrade_to_https 自动升级为 HTTPS。
    #[test]
    fn test_mixed_content_upgrade_image() {
        let page = Origin::parse("https://example.com").unwrap();
        let http_img_url = "http://cdn.example.com/photo.jpg";

        // 1. 检测到混合内容（HTTPS 页面加载 HTTP 图片）
        assert!(is_mixed_content(&page, http_img_url));

        // 2. 图片属于 OptionallyBlockable（可升级类型）
        let status = check_mixed_content(&page, http_img_url, "img");
        assert_eq!(status, MixedContentStatus::OptionallyBlockable);

        // 3. upgrade_to_https 将 HTTP URL 转换为 HTTPS
        let upgraded = upgrade_to_https(http_img_url);
        assert_eq!(upgraded, Some("https://cdn.example.com/photo.jpg".to_string()));

        // 4. 升级后的 HTTPS URL 不再是混合内容
        let upgraded_url = upgraded.unwrap();
        assert!(!is_mixed_content(&page, &upgraded_url));
        let upgraded_status = check_mixed_content(&page, &upgraded_url, "img");
        assert_eq!(upgraded_status, MixedContentStatus::NotMixedContent);
    }

    // ── 边界测试（round 23）──

    /// 测试混合内容 blob: URI 在 HTTPS 页面上的处理。
    ///
    /// blob: URI 是由当前页面通过 URL.createObjectURL() 创建的，
    /// 其源继承自创建者。当前 is_mixed_content 仅检测 http:// 前缀，
    /// blob: URI 不以 http:// 开头，因此不被识别为混合内容。
    #[test]
    fn test_mixed_content_blob_uri_on_https_page() {
        let page = Origin::parse("https://example.com").unwrap();

        // blob: URI 不以 http:// 开头 → 不是混合内容
        let blob_url = "blob:https://example.com/abc-123-def";
        assert!(!is_mixed_content(&page, blob_url), "blob: URI 不应被检测为混合内容");
        assert_eq!(
            check_mixed_content(&page, blob_url, "script"),
            MixedContentStatus::NotMixedContent,
            "blob: URI 的混合内容状态应为 NotMixedContent"
        );

        // upgrade_to_https 对 blob: URI 返回 None
        assert_eq!(upgrade_to_https(blob_url), None, "upgrade_to_https 不应处理 blob: URI");

        // 对比：http:// 应被检测为混合内容
        assert!(
            is_mixed_content(&page, "http://example.com/resource"),
            "http:// 应被检测为混合内容"
        );
    }

    /// R3343：混合内容检测须对 URL scheme 大小写不敏感（RFC 3986 §3.1）。
    ///
    /// 生产链路 `apps/browser/fetch_proxy.rs` 直接把 renderer 经 IPC 传来的**未规范化**
    /// `params.url` 原样喂给 `check_resource_url` → `is_mixed_content`。旧实现对 `http://`
    /// 做大小写敏感前缀匹配，HTTPS 页面加载 `HTTP://`（大写）资源绕过检测，阻塞型混合内容
    /// （script/iframe）静默放行 → 中间人可注入明文脚本（mixed-content 安全策略绕过）。
    #[test]
    fn test_mixed_content_case_insensitive_scheme_r3343() {
        let page = Origin::parse("https://example.com").unwrap();

        // 小写（基线）。
        assert!(is_mixed_content(&page, "http://evil.com/script.js"));
        // 大写 scheme —— 修复前绕过（starts_with("http://") 大小写敏感）。
        assert!(
            is_mixed_content(&page, "HTTP://evil.com/script.js"),
            "大写 HTTP:// 须被识别为混合内容（scheme 大小写不敏感）"
        );
        // 混合大小写 scheme。
        assert!(
            is_mixed_content(&page, "HtTp://evil.com/script.js"),
            "混合大小写 HtTp:// 须被识别为混合内容"
        );

        // check_mixed_content 须把大写 HTTP script 归为 Blockable（修复前归 NotMixedContent → 放行）。
        let status = check_mixed_content(&page, "HTTP://evil.com/script.js", "script");
        assert_eq!(
            status,
            MixedContentStatus::Blockable,
            "大写 HTTP:// 脚本须归为 Blockable，修复前为 NotMixedContent（绕过）"
        );

        // upgrade_to_https 须能升级大写 HTTP://（修复前对大写返回 None）。
        assert_eq!(
            upgrade_to_https("HTTP://evil.com/img.png"),
            Some("https://evil.com/img.png".to_string()),
            "upgrade_to_https 须大小写不敏感地识别 HTTP:// 前缀"
        );

        // HTTPS 大写不应被误判为混合内容。
        assert!(!is_mixed_content(&page, "HTTPS://evil.com/script.js"));
    }
}
