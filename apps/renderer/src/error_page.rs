//! 多进程渲染进程内联错误页 HTML。
//!
//! 与主进程侧 `apps/browser/src/pages.rs::generate_error_page` 保持同一模板：
//! Chrome 风格标题 + 建议 + 可点击 URL + 重试按钮（重试 = 再次导航到原 URL）。
//! 渲染进程不依赖 browser-shell，语言检测在此按同规则内联
//!（ZERO_BROWSER_UI_LANG → LC_ALL → LC_MESSAGES → LANG，zh 前缀中文，其余英文）。

/// 生成错误页面 HTML。
///
/// URL 与错误文字均经 HTML 转义——错误文字可能内嵌远端派生内容，属不可信输入。
pub fn generate_error_page(url: &str, error: &str) -> String {
    let zh = preferred_lang_is_zh();
    let (heading, url_label, suggestions, reload_label) = if zh {
        (
            "无法访问此网站",
            "目标地址",
            ["检查网络连接。", "检查网址是否正确。"],
            "重新加载",
        )
    } else {
        (
            "This site can’t be reached",
            "Address",
            ["Check your network connection.", "Check the web address."],
            "Reload",
        )
    };
    let escaped_url = html_escape(url);
    let suggestions_html = suggestions
        .iter()
        .map(|text| format!("<li style=\"margin:4px 0;\">{text}</li>"))
        .collect::<String>();
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{heading}</title></head>
<body style="font-family: Segoe UI, sans-serif; margin: 0; background: #fff; color: #202124; display: flex; justify-content: center;">
  <div style="max-width: 600px; margin-top: 12vh;">
    <h1 style="font-size: 22px; font-weight: 400; color: #202124; margin: 0 0 16px;">{heading}</h1>
    <p style="font-size: 14px; color: #5f6368; margin: 0 0 16px;">{url_label}: <a href="{href}" style="color: #1a73e8; text-decoration: none;"><code>{escaped_url}</code></a></p>
    <ul style="font-size: 14px; color: #5f6368; padding-left: 20px; margin: 0 0 24px;">{suggestions_html}</ul>
    <p style="font-size: 12px; color: #9aa0a6; margin: 0 0 24px;">{error_detail}</p>
    <a href="{href}" style="display: inline-block; background: #1a73e8; color: #fff; font-size: 14px; padding: 9px 22px; border-radius: 4px; text-decoration: none;">{reload_label}</a>
  </div>
</body>
</html>"#,
        href = escaped_url,
        error_detail = html_escape(error),
        suggestions_html = suggestions_html,
    )
}

/// 与 browser-shell `UiLanguage::detect_from_env` 同规则的中文环境判定。
fn preferred_lang_is_zh() -> bool {
    let configured = zero_runtime_config::optional_string("ZERO_BROWSER_UI_LANG");
    for value in configured.into_iter().chain(
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .filter_map(|key| std::env::var(key).ok()),
    ) {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        if normalized.starts_with("zh") {
            return true;
        }
        if normalized.starts_with("en") {
            return false;
        }
    }
    false
}

/// HTML 文本转义（与主进程侧 pages.rs::html_escape 同规则）。
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_page_has_retry_and_escaped_error() {
        let html = generate_error_page("http://127.0.0.1:9/x", "<script>alert(1)</script>");
        // 重试按钮与可点击 URL 均指向原 URL。
        assert_eq!(html.matches(r#"href="http://127.0.0.1:9/x""#).count(), 2);
        // 错误文字（远端派生）必须转义，不得注入标签。
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
