//! 欢迎页、错误页面和设置页面 HTML 模板

/// 欢迎页 HTML（编译期嵌入，样式内联于 `<style>`）
pub const WELCOME_HTML: &str = include_str!("../assets/welcome.html");

/// 生成错误页面 HTML
pub fn generate_error_page(url: &str, error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>加载失败</title></head>
<body style="font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #fff3f3;">
  <div style="text-align: center;">
    <h1 style="color: #c62828;">页面加载失败</h1>
    <p style="color: #555;">无法加载: <code>{url}</code></p>
    <p style="color: #888; font-size: 14px;">错误: {error}</p>
  </div>
</body>
</html>"#
    )
}

/// 从 HTML 文档提取 `<title>` 文本。
pub fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let tag_start = lower.find("<title")?;
    let content_start = html[tag_start..].find('>')? + tag_start + 1;
    let rest = &html[content_start..];
    let lower_rest = rest.to_ascii_lowercase();
    let end = lower_rest.find("</title>")?;
    let title = rest[..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_html_title_reads_head_title() {
        let html = r#"<!doctype html><html><head><title>Example Domain</title></head><body></body></html>"#;
        assert_eq!(extract_html_title(html).as_deref(), Some("Example Domain"));
    }

    #[test]
    fn extract_html_title_returns_none_when_missing() {
        assert!(extract_html_title("<html><body>Hi</body></html>").is_none());
    }
}

/// 生成设置页面 HTML
pub fn generate_settings_html(settings: &zero_browser_shell::BrowserSettings) -> String {
    use zero_browser_shell::SearchEngine;

    let se_name = match settings.search_engine {
        SearchEngine::Google => "Google",
        SearchEngine::Bing => "Bing",
        SearchEngine::DuckDuckGo => "DuckDuckGo",
        SearchEngine::Baidu => "百度",
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>设置 — ZeroBrowser</title></head>
<body style="font-family: sans-serif; margin: 0; padding: 40px; background: #f8f9fa; color: #333;">
  <div style="max-width: 600px; margin: 0 auto;">
    <h1 style="font-size: 28px; color: #1a73e8; margin-bottom: 24px;">设置</h1>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">搜索引擎</h2>
      <p style="font-size: 14px; color: #666;">当前: <strong>{se_name}</strong></p>
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">主页</h2>
      <p style="font-size: 14px; color: #666;"><code>{home}</code></p>
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">隐私与安全</h2>
      <div style="font-size: 14px; color: #666; line-height: 2;">
        <div>✅ 允许 JavaScript: <strong>{js}</strong></div>
        <div>✅ 允许 Cookie: <strong>{cookies}</strong></div>
        <div>✅ 阻止第三方 Cookie: <strong>{block_3p}</strong></div>
        <div>✅ 发送 Do Not Track: <strong>{dnt}</strong></div>
      </div>
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">外观</h2>
      <div style="font-size: 14px; color: #666; line-height: 2;">
        <div>显示书签栏: <strong>{bookmarks}</strong></div>
        <div>默认缩放: <strong>{zoom}%</strong></div>
      </div>
    </div>

    <p style="text-align: center; color: #999; font-size: 12px; margin-top: 32px;">
      ZeroBrowser v0.1.0 — 基于 Rust 构建
    </p>
  </div>
</body>
</html>"#,
        home = settings.home_url,
        js = if settings.javascript_enabled { "是" } else { "否" },
        cookies = if settings.cookies_enabled { "是" } else { "否" },
        block_3p = if settings.block_third_party_cookies {
            "是"
        } else {
            "否"
        },
        dnt = if settings.do_not_track { "是" } else { "否" },
        bookmarks = if settings.show_bookmarks_bar { "是" } else { "否" },
        zoom = (settings.default_zoom * 100.0) as u32,
    )
}
