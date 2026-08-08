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

/// HTML 文本转义（用于查看源代码页）。
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 生成「查看源代码」页面。
pub fn generate_view_source_page(source_url: &str, html: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>查看源代码 — {title}</title></head>
<body style="margin:0;background:#fff;color:#202124;font-family:Consolas,Monaco,monospace;">
  <pre style="margin:0;padding:16px;white-space:pre-wrap;word-break:break-word;font-size:12px;line-height:1.5;">{body}</pre>
</body>
</html>"#,
        title = html_escape(source_url),
        body = html_escape(html),
    )
}

/// 生成「检查元素」信息页。
pub fn generate_inspect_element_page(
    source_url: &str,
    click_x: f32,
    click_y: f32,
    hit: Option<&zero_engine::ElementHit>,
) -> String {
    let (tag, id, class_name, box_x, box_y, box_w, box_h, outer) = match hit {
        Some(el) => (
            el.tag_name.clone(),
            el.id.clone().unwrap_or_else(|| "—".to_string()),
            el.class_name.clone().unwrap_or_else(|| "—".to_string()),
            format!("{:.1}", el.x),
            format!("{:.1}", el.y),
            format!("{:.1}", el.width),
            format!("{:.1}", el.height),
            format!(
                "<{} id=\"{}\" class=\"{}\"></{}>",
                el.tag_name,
                html_escape(&el.id.clone().unwrap_or_default()),
                html_escape(&el.class_name.clone().unwrap_or_default()),
                el.tag_name
            ),
        ),
        None => (
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "<!-- 未命中元素 -->".to_string(),
        ),
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>检查元素 — ZeroBrowser</title></head>
<body style="font-family:Segoe UI,sans-serif;margin:0;padding:24px;background:#f8f9fa;color:#202124;">
  <h1 style="font-size:20px;margin:0 0 8px;">检查元素</h1>
  <p style="margin:0 0 16px;color:#5f6368;font-size:13px;">页面: <code>{url}</code></p>
  <div style="background:#fff;border-radius:8px;padding:20px;box-shadow:0 1px 3px rgba(0,0,0,.08);max-width:720px;">
    <h2 style="font-size:16px;margin:0 0 12px;">命中信息</h2>
    <table style="border-collapse:collapse;font-size:14px;line-height:1.8;">
      <tr><td style="color:#5f6368;padding-right:16px;">点击坐标</td><td>({click_x:.1}, {click_y:.1})</td></tr>
      <tr><td style="color:#5f6368;padding-right:16px;">标签</td><td><code>{tag}</code></td></tr>
      <tr><td style="color:#5f6368;padding-right:16px;">id</td><td><code>{id}</code></td></tr>
      <tr><td style="color:#5f6368;padding-right:16px;">class</td><td><code>{class_name}</code></td></tr>
      <tr><td style="color:#5f6368;padding-right:16px;">布局盒</td><td>x={box_x}, y={box_y}, w={box_w}, h={box_h}</td></tr>
    </table>
    <h2 style="font-size:16px;margin:20px 0 8px;">外层 HTML</h2>
    <pre style="margin:0;padding:12px;background:#f1f3f4;border-radius:6px;font-size:12px;overflow:auto;">{outer}</pre>
  </div>
</body>
</html>"#,
        url = html_escape(source_url),
        click_x = click_x,
        click_y = click_y,
        tag = html_escape(&tag),
        id = html_escape(&id),
        class_name = html_escape(&class_name),
        box_x = box_x,
        box_y = box_y,
        box_w = box_w,
        box_h = box_h,
        outer = html_escape(&outer),
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

/// 生成设置页面 HTML（含可点击开关，通过 `zero://settings/toggle/<key>` 切换）。
pub fn generate_settings_html(settings: &zero_browser_shell::BrowserSettings) -> String {
    use zero_browser_shell::SearchEngine;
    use zero_browser_shell::UiLanguage;

    let lang = UiLanguage::detect_from_env();
    let se_name = match settings.search_engine {
        SearchEngine::Google => "Google",
        SearchEngine::Bing => "Bing",
        SearchEngine::DuckDuckGo => "DuckDuckGo",
        SearchEngine::Baidu => "百度",
    };

    let (page_title, section_search, section_home, section_privacy, section_appearance, toggle, current_label) =
        match lang {
            UiLanguage::ZhCn => (
                "设置 — ZeroBrowser",
                "搜索引擎",
                "主页",
                "隐私与安全",
                "外观",
                "切换",
                "当前",
            ),
            UiLanguage::EnUs => (
                "Settings — ZeroBrowser",
                "Search Engine",
                "Home Page",
                "Privacy & Security",
                "Appearance",
                "Toggle",
                "Current",
            ),
        };

    let yes_no = |value: bool| match lang {
        UiLanguage::ZhCn => {
            if value {
                "是"
            } else {
                "否"
            }
        }
        UiLanguage::EnUs => {
            if value {
                "Yes"
            } else {
                "No"
            }
        }
    };

    let toggle_link = |key: &str, label: &str, value: bool| {
        format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:center;line-height:2;">
  <span>{label}</span>
  <span>{current_label}: <strong>{value}</strong>
    <a href="zero://settings/toggle/{key}" style="margin-left:12px;color:#1a73e8;text-decoration:none;">{toggle}</a>
  </span>
</div>"#,
            value = yes_no(value),
        )
    };

    let action_link = |href: &str, label: &str| {
        format!(r#"<a href="{href}" style="margin-left:12px;color:#1a73e8;text-decoration:none;">{label}</a>"#)
    };

    let (cycle_search, home_presets_title, home_example, home_newtab, edit_home, home_edit_hint) = match lang {
        UiLanguage::ZhCn => (
            "切换搜索引擎",
            "常用主页",
            "example.com",
            "欢迎页 (zero://newtab)",
            "自定义…",
            "点击后在地址栏输入主页 URL，按 Enter 保存",
        ),
        UiLanguage::EnUs => (
            "Cycle Search Engine",
            "Common Home Pages",
            "example.com",
            "Welcome (zero://newtab)",
            "Custom…",
            "Click, type the home URL in the address bar, then press Enter",
        ),
    };

    let (zoom_decrease, zoom_increase, zoom_reset_label, download_dir_label, edit_download_dir) = match lang {
        UiLanguage::ZhCn => ("缩小", "放大", "重置 100%", "下载目录", "自定义…"),
        UiLanguage::EnUs => ("Decrease", "Increase", "Reset 100%", "Download folder", "Custom…"),
    };

    let search_actions = format!(
        r#"<span style="margin-left:12px;">{google}{bing}{ddg}{baidu}{cycle}</span>"#,
        google = action_link("zero://settings/set/search_engine/Google", "Google"),
        bing = action_link("zero://settings/set/search_engine/Bing", "Bing"),
        ddg = action_link("zero://settings/set/search_engine/DuckDuckGo", "DDG"),
        baidu = action_link(
            "zero://settings/set/search_engine/Baidu",
            if matches!(lang, UiLanguage::ZhCn) {
                "百度"
            } else {
                "Baidu"
            }
        ),
        cycle = action_link("zero://settings/cycle/search_engine", cycle_search),
    );
    let home_actions = format!(
        r#"<p style="font-size:13px;color:#80868b;margin:12px 0 4px;">{home_presets_title}</p>
<p style="font-size:14px;color:#666;line-height:2;">
  {example_link}{newtab_link}{custom_link}
</p>
<p style="font-size:12px;color:#80868b;margin:8px 0 0;">{home_edit_hint}</p>"#,
        example_link = action_link("zero://settings/set/home_url/https%3A%2F%2Fexample.com", home_example,),
        newtab_link = action_link("zero://settings/set/home_url/zero%3A%2F%2Fnewtab", home_newtab),
        custom_link = action_link("zero://settings/edit/home_url", edit_home),
    );

    let zoom = (settings.default_zoom * 100.0) as u32;

    let zoom_actions = format!(
        r#"<div style="line-height:2;">{current_label} zoom: <strong>{zoom}%</strong>
  {decrease}{increase}{reset}
</div>"#,
        decrease = action_link("zero://settings/adjust/default_zoom/down", zoom_decrease),
        increase = action_link("zero://settings/adjust/default_zoom/up", zoom_increase),
        reset = action_link("zero://settings/set/default_zoom/1.0", zoom_reset_label),
    );

    let download_dir_display = if settings.download_directory.is_empty() {
        match lang {
            UiLanguage::ZhCn => "系统默认",
            UiLanguage::EnUs => "System default",
        }
        .to_string()
    } else {
        settings.download_directory.clone()
    };
    let download_dir_row = format!(
        r#"<div style="line-height:2;">{download_dir_label}: <code>{dir}</code>{edit}</div>"#,
        edit = action_link("zero://settings/edit/download_directory", edit_download_dir),
        dir = download_dir_display,
    );

    let (theme_section_label, theme_auto, theme_light, theme_dark, theme_cycle) = match lang {
        UiLanguage::ZhCn => ("主题", "自动", "亮色", "暗色", "轮换"),
        UiLanguage::EnUs => ("Theme", "Auto", "Light", "Dark", "Cycle"),
    };
    let theme_name = match settings.color_theme {
        zero_browser_shell::ColorThemePreference::Auto => theme_auto,
        zero_browser_shell::ColorThemePreference::Light => theme_light,
        zero_browser_shell::ColorThemePreference::Dark => theme_dark,
    };
    let theme_row = format!(
        r#"<div style="line-height:2;">{theme_section_label}: <strong>{theme_name}</strong>
  {auto_link}{light_link}{dark_link}{cycle_link}
</div>"#,
        auto_link = action_link("zero://settings/set/color_theme/auto", theme_auto),
        light_link = action_link("zero://settings/set/color_theme/light", theme_light),
        dark_link = action_link("zero://settings/set/color_theme/dark", theme_dark),
        cycle_link = action_link("zero://settings/cycle/color_theme", theme_cycle),
    );

    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{page_title}</title></head>
<body style="font-family: Segoe UI, sans-serif; margin: 0; padding: 40px; background: #f8f9fa; color: #333;">
  <div style="max-width: 600px; margin: 0 auto;">
    <h1 style="font-size: 28px; color: #1a73e8; margin-bottom: 24px;">{page_title}</h1>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">{section_search}</h2>
      <p style="font-size: 14px; color: #666;">{current_label}: <strong>{se_name}</strong>{search_actions}</p>
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">{section_home}</h2>
      <p style="font-size: 14px; color: #666;"><code>{home}</code></p>
      {home_actions}
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">{section_privacy}</h2>
      <div style="font-size: 14px; color: #666;">
        {js_row}
        {cookies_row}
        {block_3p_row}
        {dnt_row}
      </div>
    </div>

    <div style="background: white; border-radius: 8px; padding: 24px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
      <h2 style="font-size: 18px; margin-top: 0; color: #555;">{section_appearance}</h2>
      <div style="font-size: 14px; color: #666;">
        {bookmarks_row}
        {theme_row}
        {zoom_actions}
        {download_dir_row}
      </div>
    </div>

    <p style="text-align: center; color: #999; font-size: 12px; margin-top: 32px;">
      ZeroBrowser v{version}
    </p>
  </div>
</body>
</html>"#,
        js_row = toggle_link(
            "javascript_enabled",
            if matches!(lang, UiLanguage::ZhCn) {
                "允许 JavaScript"
            } else {
                "Allow JavaScript"
            },
            settings.javascript_enabled,
        ),
        cookies_row = toggle_link(
            "cookies_enabled",
            if matches!(lang, UiLanguage::ZhCn) {
                "允许 Cookie"
            } else {
                "Allow Cookies"
            },
            settings.cookies_enabled,
        ),
        block_3p_row = toggle_link(
            "block_third_party_cookies",
            if matches!(lang, UiLanguage::ZhCn) {
                "阻止第三方 Cookie"
            } else {
                "Block Third-Party Cookies"
            },
            settings.block_third_party_cookies,
        ),
        dnt_row = toggle_link(
            "do_not_track",
            if matches!(lang, UiLanguage::ZhCn) {
                "发送 Do Not Track"
            } else {
                "Send Do Not Track"
            },
            settings.do_not_track,
        ),
        bookmarks_row = toggle_link(
            "show_bookmarks_bar",
            if matches!(lang, UiLanguage::ZhCn) {
                "显示书签栏"
            } else {
                "Show Bookmarks Bar"
            },
            settings.show_bookmarks_bar,
        ),
        home = settings.home_url,
        search_actions = search_actions,
        home_actions = home_actions,
        zoom_actions = zoom_actions,
        download_dir_row = download_dir_row,
        version = zero_product_version::VERSION,
    )
}

fn internal_page_shell(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{title}</title></head>
<body style="font-family: Segoe UI, sans-serif; margin: 0; padding: 40px; background: #f8f9fa; color: #333;">
  <div style="max-width: 720px; margin: 0 auto;">
    <h1 style="font-size: 28px; color: #1a73e8; margin-bottom: 24px;">{title}</h1>
    {body}
  </div>
</body>
</html>"#
    )
}

/// 生成历史记录页面 HTML。
pub fn generate_history_html(history: &zero_browser_shell::History) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, empty, clear_label) = match lang {
        UiLanguage::ZhCn => ("历史记录 — ZeroBrowser", "暂无历史记录。", "清除历史"),
        UiLanguage::EnUs => ("History — ZeroBrowser", "No history yet.", "Clear History"),
    };
    let mut rows = String::new();
    for entry in history.iter().take(100) {
        rows.push_str(&format!(
            r#"<li style="margin:8px 0;"><a href="{url}" style="color:#1a73e8;text-decoration:none;">{label}</a><br><span style="font-size:12px;color:#80868b;">{url}</span></li>"#,
            label = html_escape(entry.title()),
            url = html_escape(entry.url()),
        ));
    }
    let body = if rows.is_empty() {
        format!(
            r#"<p style="color:#666;">{empty}</p><p><a href="zero://history/clear" style="color:#1a73e8;text-decoration:none;">{clear_label}</a></p>"#
        )
    } else {
        format!(
            r#"<p><a href="zero://history/clear" style="color:#1a73e8;text-decoration:none;">{clear_label}</a></p><ul style="padding-left:20px;">{rows}</ul>"#
        )
    };
    internal_page_shell(title, &body)
}

/// 生成下载管理页面 HTML。
pub fn generate_downloads_html(downloads: &zero_browser_shell::DownloadManager) -> String {
    use zero_browser_shell::DownloadState;
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, empty) = match lang {
        UiLanguage::ZhCn => ("下载内容 — ZeroBrowser", "暂无下载任务。"),
        UiLanguage::EnUs => ("Downloads — ZeroBrowser", "No downloads yet."),
    };
    let mut rows = String::new();
    for entry in downloads.iter() {
        let state = match entry.state() {
            DownloadState::Pending => "Pending",
            DownloadState::Downloading => "Downloading",
            DownloadState::Paused => "Paused",
            DownloadState::Completed => "Completed",
            DownloadState::Cancelled => "Cancelled",
            DownloadState::Failed => "Failed",
        };
        rows.push_str(&format!(
            r#"<li style="margin:12px 0;padding:12px;background:white;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,0.08);"><strong>{name}</strong><br><span style="font-size:12px;color:#80868b;">{url}</span><br><span style="font-size:12px;color:#555;">{state} · {progress:.0}%</span></li>"#,
            name = html_escape(entry.filename()),
            url = html_escape(entry.url()),
            progress = entry.progress() * 100.0,
        ));
    }
    let body = if rows.is_empty() {
        format!("<p style=\"color:#666;\">{empty}</p>")
    } else {
        format!("<ul style=\"list-style:none;padding:0;\">{rows}</ul>")
    };
    internal_page_shell(title, &body)
}

/// 生成书签管理页面 HTML。
pub fn generate_bookmarks_html(bookmarks: &zero_browser_shell::Bookmarks) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, empty) = match lang {
        UiLanguage::ZhCn => ("书签 — ZeroBrowser", "暂无书签。"),
        UiLanguage::EnUs => ("Bookmarks — ZeroBrowser", "No bookmarks yet."),
    };
    let mut rows = String::new();
    for bookmark in bookmarks.list_root() {
        rows.push_str(&format!(
            r#"<li style="margin:8px 0;"><a href="{url}" style="color:#1a73e8;text-decoration:none;">{title}</a><br><span style="font-size:12px;color:#80868b;">{url}</span></li>"#,
            title = html_escape(bookmark.title()),
            url = html_escape(bookmark.url()),
        ));
    }
    let body = if rows.is_empty() {
        format!("<p style=\"color:#666;\">{empty}</p>")
    } else {
        format!("<ul style=\"padding-left:20px;\">{rows}</ul>")
    };
    internal_page_shell(title, &body)
}

/// 生成「关于 ZeroBrowser」页面 HTML。
pub fn generate_about_browser_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>About ZeroBrowser</title></head>
<body style="font-family: Segoe UI, sans-serif; margin: 0; padding: 40px; background: #f8f9fa; color: #202124;">
  <div style="max-width: 720px; margin: 0 auto;">
    <div style="background: white; border-radius: 14px; padding: 28px 32px; box-shadow: 0 8px 24px rgba(0,0,0,0.06);">
      <p style="margin: 0; color: #1a73e8; font-size: 13px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase;">About</p>
      <h1 style="font-size: 32px; margin: 10px 0 12px;">ZeroBrowser</h1>
      <p style="margin: 0 0 20px; color: #5f6368; font-size: 15px; line-height: 1.7;">
        ZeroBrowser is the desktop browser app built on top of the ZeroWeb stack. It combines
        the reusable <strong>ZeroWebView</strong> runtime, the <strong>BrowserShell</strong> state model,
        and a custom-drawn browser chrome written in Rust.
      </p>

      <div style="display: grid; gap: 12px; margin: 0 0 24px;">
        <div style="padding: 14px 16px; border-radius: 10px; background: #f1f3f4;">
          <strong>Engine:</strong> ZeroWeb custom engine
        </div>
        <div style="padding: 14px 16px; border-radius: 10px; background: #f1f3f4;">
          <strong>UI:</strong> custom Rust-rendered browser chrome
        </div>
        <div style="padding: 14px 16px; border-radius: 10px; background: #f1f3f4;">
          <strong>Status:</strong> experimental browser product shell
        </div>
      </div>

      <h2 style="font-size: 18px; margin: 0 0 10px;">What it includes</h2>
      <ul style="margin: 0 0 20px 20px; padding: 0; color: #3c4043; line-height: 1.8;">
        <li>tabs, navigation, bookmarks, autocomplete, downloads, and settings</li>
        <li>a reusable embedded webview boundary for the wider ZeroWeb project</li>
        <li>cross-platform browser chrome built without adopting a separate UI framework</li>
      </ul>

      <p style="margin: 0; color: #80868b; font-size: 13px;">
        Version: ZeroBrowser v{version}
      </p>
    </div>
  </div>
</body>
</html>"#,
        version = zero_product_version::VERSION,
    )
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

    #[test]
    fn generate_history_html_includes_clear_link() {
        let history = zero_browser_shell::History::default();
        let html = generate_history_html(&history);
        assert!(html.contains("zero://history/clear"));
        // 标题随 UiLanguage::detect_from_env() 变化（EnUs "History" / ZhCn "历史"），
        // 须同时接受两种 locale（与 generate_downloads_html_renders_empty_state 同模式）。
        assert!(html.contains("History") || html.contains("历史"));
    }

    #[test]
    fn generate_downloads_html_renders_empty_state() {
        let downloads = zero_browser_shell::DownloadManager::default();
        let html = generate_downloads_html(&downloads);
        assert!(html.contains("Downloads") || html.contains("下载"));
    }

    #[test]
    fn generate_bookmarks_html_renders_empty_state() {
        let bookmarks = zero_browser_shell::Bookmarks::default();
        let html = generate_bookmarks_html(&bookmarks);
        assert!(html.contains("Bookmarks") || html.contains("书签"));
    }
}
