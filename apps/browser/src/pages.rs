//! 欢迎页、错误页面和设置页面 HTML 模板

/// 欢迎页 HTML（编译期嵌入，样式内联于 `<style>`）
pub const WELCOME_HTML: &str = include_str!("../assets/welcome.html");

/// 错误页标签标题（与错误页主标题同源；Chrome 用错误标题作 tab 标题）。
pub fn error_page_tab_title() -> &'static str {
    match zero_browser_shell::UiLanguage::detect_from_env() {
        zero_browser_shell::UiLanguage::ZhCn => "无法访问此网站",
        zero_browser_shell::UiLanguage::EnUs => "This site can’t be reached",
    }
}

/// 生成错误页面 HTML。
///
/// Chrome 风格：标题 + 建议 + 可点击 URL + 重试按钮（重试 = 再次导航到原 URL）。
/// URL 与错误文字均经 HTML 转义——错误文字可能内嵌远端派生内容，属不可信输入。
pub fn generate_error_page(url: &str, error: &str) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let heading = error_page_tab_title();
    let (url_label, suggestions, reload_label) = match lang {
        UiLanguage::ZhCn => ("目标地址", ["检查网络连接。", "检查网址是否正确。"], "重新加载"),
        UiLanguage::EnUs => (
            "Address",
            ["Check your network connection.", "Check the web address."],
            "Reload",
        ),
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

/// HTML 文本转义（用于查看源代码页）。
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 内部页 query 参数值的 percent-encode（仅保留 RFC 3986 unreserved 字符）。
///
/// 历史页删除链接把原 URL 作为 `?url=` 参数携带：`&`/`?`/`=` 等必须转义，
/// 否则会被当作 query 分隔符截断；解码侧先按 `&`/`=` 切分再 percent-decode。
pub fn percent_encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// 表单 `application/x-www-form-urlencoded` 参数值解码（`+`→空格，`%XX` 还原）。
///
/// 书签编辑表单经渲染端表单序列化提交（GET query，WHATWG urlencoded serializer）：
/// 原文空格编码为 `+`、`+` 本身编码为 `%2B`，故先替换 `+` 再做 %XX 还原，顺序不可反
///（反序会把 `%2B` 还原出的 `+` 二次吞成空格）。
///
/// https://url.spec.whatwg.org/#concept-urlencoded-serializer
pub fn percent_decode_form_value(value: &str) -> String {
    let bytes = value.replace('+', " ").into_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                decoded.push(byte);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
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

/// 生成设置页面 HTML（Chrome 风格：CSS toggle switch 开关 + 分组卡片 + pill 选择器）。
///
/// 开关与选择器仍是 `<a>` 链接（`zero://settings/toggle/<key>` 等既有动作 URL 不变），
/// 仅替换视觉控件；无 JS 依赖，纯内联样式。
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

    let (page_title, section_search, section_home, section_privacy, section_appearance) = match lang {
        UiLanguage::ZhCn => ("设置 — ZeroBrowser", "搜索引擎", "主页", "隐私与安全", "外观"),
        UiLanguage::EnUs => (
            "Settings — ZeroBrowser",
            "Search Engine",
            "Home Page",
            "Privacy & Security",
            "Appearance",
        ),
    };

    // 开关状态文字与 aria 属性随语言/状态取值（对齐 Chrome：开关 + 状态文本）。
    let on_off = |value: bool| match lang {
        UiLanguage::ZhCn => {
            if value {
                "开"
            } else {
                "关"
            }
        }
        UiLanguage::EnUs => {
            if value {
                "On"
            } else {
                "Off"
            }
        }
    };

    // CSS toggle switch（Chrome/M3 风格 36×14 圆角轨道 + 10px 圆形滑块）。
    // 开=蓝色轨道、滑块居右；关=灰色轨道、滑块居左。对齐引擎已验证的
    // inline-flex + justify-content + align-items 能力（welcome 页同款）。
    let switch_link = |key: &str, label: &str, value: bool| {
        let (track_bg, knob_bg, justify) = if value {
            ("#1a73e8", "#ffffff", "flex-end")
        } else {
            ("#bdc1c6", "#5f6368", "flex-start")
        };
        format!(
            r#"<a href="zero://settings/toggle/{key}" role="switch" aria-checked="{value}" aria-label="{label}" style="display:inline-flex;width:36px;height:14px;border-radius:7px;background:{track_bg};align-items:center;justify-content:{justify};padding:2px;text-decoration:none;"><span style="display:inline-block;width:10px;height:10px;border-radius:50%;background:{knob_bg};"></span></a>"#,
        )
    };

    // 设置行：标签居左，状态文字 + 开关居右。
    let toggle_link = |key: &str, label: &str, value: bool| {
        format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:center;min-height:32px;font-size:14px;color:#3c4043;">
  <span>{label}</span>
  <span style="display:flex;align-items:center;gap:10px;">
    <span style="font-size:12px;color:#80868b;">{state}</span>
    {switch}
  </span>
</div>"#,
            state = on_off(value),
            switch = switch_link(key, label, value),
        )
    };

    // 可选项 pill（Chrome 分段选择风格）：选中=蓝底白字，未选=灰底深字。
    let pill_link = |href: &str, label: &str, selected: bool| {
        let (bg, fg) = if selected {
            ("#1a73e8", "#ffffff")
        } else {
            ("#f1f3f4", "#3c4043")
        };
        format!(
            r#"<a href="{href}" style="display:inline-block;padding:6px 14px;margin:0 8px 8px 0;border-radius:16px;background:{bg};color:{fg};font-size:13px;text-decoration:none;">{label}</a>"#,
        )
    };

    // 动作 pill（缩放增减、自定义等非选中态按钮，与选项 pill 同一视觉语言）。
    let action_link = |href: &str, label: &str| pill_link(href, label, false);

    let (cycle_search, home_presets_title, home_example, home_newtab, edit_home, home_edit_hint) = match lang {
        UiLanguage::ZhCn => (
            "轮换",
            "常用主页",
            "example.com",
            "欢迎页 (zero://newtab)",
            "自定义…",
            "点击后在地址栏输入主页 URL，按 Enter 保存",
        ),
        UiLanguage::EnUs => (
            "Cycle",
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
        r#"<div style="display:flex;align-items:center;flex-wrap:wrap;">{google}{bing}{ddg}{baidu}{cycle}</div>"#,
        google = pill_link(
            "zero://settings/set/search_engine/Google",
            "Google",
            settings.search_engine == SearchEngine::Google
        ),
        bing = pill_link(
            "zero://settings/set/search_engine/Bing",
            "Bing",
            settings.search_engine == SearchEngine::Bing
        ),
        ddg = pill_link(
            "zero://settings/set/search_engine/DuckDuckGo",
            "DDG",
            settings.search_engine == SearchEngine::DuckDuckGo
        ),
        baidu = pill_link(
            "zero://settings/set/search_engine/Baidu",
            if matches!(lang, UiLanguage::ZhCn) {
                "百度"
            } else {
                "Baidu"
            },
            settings.search_engine == SearchEngine::Baidu,
        ),
        cycle = pill_link("zero://settings/cycle/search_engine", cycle_search, false),
    );
    let home_actions = format!(
        r#"<p style="font-size:13px;color:#80868b;margin:12px 0 8px;">{home_presets_title}</p>
<div style="display:flex;align-items:center;flex-wrap:wrap;">
  {example_link}{newtab_link}{custom_link}
</div>
<p style="font-size:12px;color:#80868b;margin:8px 0 0;">{home_edit_hint}</p>"#,
        example_link = pill_link(
            "zero://settings/set/home_url/https%3A%2F%2Fexample.com",
            home_example,
            settings.home_url == "https://example.com",
        ),
        newtab_link = pill_link(
            "zero://settings/set/home_url/zero%3A%2F%2Fnewtab",
            home_newtab,
            settings.home_url == "zero://newtab",
        ),
        custom_link = action_link("zero://settings/edit/home_url", edit_home),
    );

    let zoom = (settings.default_zoom * 100.0) as u32;

    let zoom_actions = format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:center;min-height:32px;font-size:14px;color:#3c4043;">
  <span style="white-space:nowrap;">Zoom: <strong>{zoom}%</strong></span>
  <span style="display:flex;align-items:center;">{decrease}{increase}{reset}</span>
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
        r#"<div style="display:flex;justify-content:space-between;align-items:center;min-height:32px;font-size:14px;color:#3c4043;">
  <span>{download_dir_label}: <code>{dir}</code></span>
  <span>{edit}</span>
</div>"#,
        edit = action_link("zero://settings/edit/download_directory", edit_download_dir),
        dir = download_dir_display,
    );

    let (theme_section_label, theme_auto, theme_light, theme_dark, theme_cycle) = match lang {
        UiLanguage::ZhCn => ("主题", "自动", "亮色", "暗色", "轮换"),
        UiLanguage::EnUs => ("Theme", "Auto", "Light", "Dark", "Cycle"),
    };
    let theme_selected = |want: zero_browser_shell::ColorThemePreference| settings.color_theme == want;
    let theme_row = format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:center;min-height:32px;font-size:14px;color:#3c4043;">
  <span>{theme_section_label}</span>
  <span style="display:flex;align-items:center;">{auto_link}{light_link}{dark_link}{cycle_link}</span>
</div>"#,
        auto_link = pill_link(
            "zero://settings/set/color_theme/auto",
            theme_auto,
            theme_selected(zero_browser_shell::ColorThemePreference::Auto),
        ),
        light_link = pill_link(
            "zero://settings/set/color_theme/light",
            theme_light,
            theme_selected(zero_browser_shell::ColorThemePreference::Light),
        ),
        dark_link = pill_link(
            "zero://settings/set/color_theme/dark",
            theme_dark,
            theme_selected(zero_browser_shell::ColorThemePreference::Dark),
        ),
        cycle_link = action_link("zero://settings/cycle/color_theme", theme_cycle),
    );

    let js_row = toggle_link(
        "javascript_enabled",
        if matches!(lang, UiLanguage::ZhCn) {
            "允许 JavaScript"
        } else {
            "Allow JavaScript"
        },
        settings.javascript_enabled,
    );
    let cookies_row = toggle_link(
        "cookies_enabled",
        if matches!(lang, UiLanguage::ZhCn) {
            "允许 Cookie"
        } else {
            "Allow Cookies"
        },
        settings.cookies_enabled,
    );
    let block_3p_row = toggle_link(
        "block_third_party_cookies",
        if matches!(lang, UiLanguage::ZhCn) {
            "阻止第三方 Cookie"
        } else {
            "Block Third-Party Cookies"
        },
        settings.block_third_party_cookies,
    );
    let dnt_row = toggle_link(
        "do_not_track",
        if matches!(lang, UiLanguage::ZhCn) {
            "发送 Do Not Track"
        } else {
            "Send Do Not Track"
        },
        settings.do_not_track,
    );
    let (clear_data_label, clear_history_label, clear_cookies_label) = match lang {
        UiLanguage::ZhCn => ("清除浏览数据", "清除历史记录", "清除 Cookie"),
        UiLanguage::EnUs => ("Clear Browsing Data", "Clear History", "Clear Cookies"),
    };
    let clear_data_row = format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:center;min-height:32px;font-size:14px;color:#3c4043;">
  <span>{clear_data_label}</span>
  <span style="display:flex;align-items:center;">{history_link}{cookies_link}</span>
</div>"#,
        history_link = action_link("zero://settings/clear/history", clear_history_label),
        cookies_link = action_link("zero://settings/clear/cookies", clear_cookies_label),
    );
    let bookmarks_row = toggle_link(
        "show_bookmarks_bar",
        if matches!(lang, UiLanguage::ZhCn) {
            "显示书签栏"
        } else {
            "Show Bookmarks Bar"
        },
        settings.show_bookmarks_bar,
    );

    // 分组卡片：Chrome 设置风格——大圆角白卡 + 小号大写分组标题（蓝色强调）。
    let section_card = |title: &str, body: &str| {
        format!(
            r#"<div style="background: white; border-radius: 12px; padding: 20px 24px; margin-bottom: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);">
  <h2 style="font-size: 12px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: #1a73e8; margin: 0 0 12px;">{title}</h2>
  {body}
</div>"#,
        )
    };

    let current_label = if matches!(lang, UiLanguage::ZhCn) {
        "当前"
    } else {
        "Current"
    };
    let search_card = section_card(
        section_search,
        &format!(
            r#"<p style="font-size:14px;color:#3c4043;margin:0 0 12px;">{current_label}: <strong>{se_name}</strong></p>{search_actions}"#,
        ),
    );
    let home_card = section_card(
        section_home,
        &format!(
            r#"<p style="font-size:14px;color:#3c4043;margin:0 0 8px;"><code>{}</code></p>{home_actions}"#,
            settings.home_url,
        ),
    );
    let privacy_card = section_card(
        section_privacy,
        &format!("{js_row}{cookies_row}{block_3p_row}{dnt_row}{clear_data_row}"),
    );
    let appearance_card = section_card(
        section_appearance,
        &format!("{bookmarks_row}{theme_row}{zoom_actions}{download_dir_row}"),
    );

    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>{page_title}</title></head>
<body style="font-family: Segoe UI, sans-serif; margin: 0; padding: 40px; background: #f8f9fa; color: #333;">
  <div style="max-width: 600px; margin: 0 auto;">
    <h1 style="font-size: 28px; color: #1a73e8; margin-bottom: 24px;">{page_title}</h1>
    {search_card}
    {home_card}
    {privacy_card}
    {appearance_card}
    <p style="text-align: center; color: #999; font-size: 12px; margin-top: 32px;">
      ZeroBrowser v{version}
    </p>
  </div>
</body>
</html>"#,
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
///
/// `query` 为地址栏/搜索框提交的过滤词（`zero://history?q=…`）；非空时按 URL
/// 或标题子串过滤（`History::search`，大小写不敏感）。每条记录附带单条删除链接
///（`zero://history/remove?url=…`），删除后回跳时保留当前过滤词。
pub fn generate_history_html(history: &zero_browser_shell::History, query: Option<&str>) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, empty, clear_label, search_placeholder, search_label, delete_label, no_match) = match lang {
        UiLanguage::ZhCn => (
            "历史记录 — ZeroBrowser",
            "暂无历史记录。",
            "清除历史",
            "搜索历史记录",
            "搜索",
            "删除",
            "没有匹配的历史记录。",
        ),
        UiLanguage::EnUs => (
            "History — ZeroBrowser",
            "No history yet.",
            "Clear History",
            "Search history",
            "Search",
            "Delete",
            "No matching history entries.",
        ),
    };
    let active_query = query.unwrap_or("").trim();
    let entries: Vec<&zero_browser_shell::HistoryEntry> = if active_query.is_empty() {
        history.iter().take(100).collect()
    } else {
        history.search(active_query).take(100).collect()
    };
    // 删除链接回跳时携带当前过滤词，删除一条后仍停留在同一过滤视图。
    let remove_suffix = if active_query.is_empty() {
        String::new()
    } else {
        format!("&q={}", percent_encode_query_value(active_query))
    };
    // 时间分组（Chrome 行为：按日期分组展示）。条目时间倒序输入，逐条映射到
    // 本地日期组；同组合并、异组按首次出现顺序排列（新的组在前）。
    let today = local_today_days();
    // 条目级相对时间统一取同一 `now`，同屏时间轴一致。
    let now = unix_seconds_now();
    let mut groups: Vec<(HistoryGroup, Vec<&zero_browser_shell::HistoryEntry>)> = Vec::new();
    for entry in entries {
        let group = HistoryGroup::of(entry.visited_at());
        match groups.iter_mut().find(|(known, _)| *known == group) {
            Some((_, items)) => items.push(entry),
            None => groups.push((group, vec![entry])),
        }
    }
    let search_box = format!(
        r#"<form action="zero://history" method="get" style="margin:0 0 16px;">
  <input type="text" name="q" value="{value}" placeholder="{search_placeholder}" style="font-size:14px;padding:6px 10px;border:1px solid #dadce0;border-radius:4px;width:320px;">
  <button type="submit" style="font-size:14px;padding:6px 16px;margin-left:8px;background:#1a73e8;color:#fff;border:none;border-radius:4px;">{search_label}</button>
  <a href="zero://history/clear" style="margin-left:12px;color:#1a73e8;text-decoration:none;font-size:13px;">{clear_all}</a>
</form>"#,
        value = html_escape(active_query),
        clear_all = clear_label,
    );
    let body = if groups.is_empty() {
        let empty_text = if active_query.is_empty() { empty } else { no_match };
        format!(r#"{search_box}<p style="color:#666;">{empty_text}</p>"#)
    } else {
        let mut sections = String::new();
        for (group, items) in groups {
            let mut rows = String::new();
            for entry in items {
                // 条目级相对时间（Chrome 行为）：灰色小字缀在 URL 右侧；
                // 旧版档案条目（无时间戳）不显示。
                let visited = history_relative_time(entry.visited_at(), now, lang).unwrap_or_default();
                rows.push_str(&format!(
                    r#"<li style="margin:8px 0;"><a href="{href}" style="color:#1a73e8;text-decoration:none;">{label}</a><br><span style="font-size:12px;color:#80868b;">{url}</span><span style="font-size:12px;color:#9aa0a6;margin-left:8px;">{visited}</span> <a href="zero://history/remove?url={encoded}{remove_suffix}" style="font-size:12px;color:#d93025;text-decoration:none;margin-left:8px;">{delete_label}</a></li>"#,
                    href = html_escape(entry.url()),
                    label = html_escape(entry.title()),
                    url = html_escape(entry.url()),
                    encoded = percent_encode_query_value(entry.url()),
                ));
            }
            sections.push_str(&format!(
                r#"<h2 style="font-size:14px;font-weight:600;color:#5f6368;margin:20px 0 8px;">{header}</h2><ul style="padding-left:20px;margin:0;">{rows}</ul>"#,
                header = history_group_header(group, today, lang),
            ));
        }
        format!(r#"{search_box}{sections}"#)
    };
    internal_page_shell(title, &body)
}

/// 历史页时间分组键：本地日期（epoch 天序）或时间未知（旧版档案缺省 `visited_at == 0`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryGroup {
    /// 时间未知（旧版档案条目），单独成组。
    Unknown,
    /// 本地日期，值为 epoch 天序号（`days_from_civil` 口径）。
    Day(i64),
}

impl HistoryGroup {
    /// 条目访问时间 → 分组键。
    fn of(visited_at: u64) -> Self {
        if visited_at == 0 {
            Self::Unknown
        } else {
            Self::Day(unix_secs_to_local_days(visited_at.min(i64::MAX as u64) as i64))
        }
    }
}

/// 分组标题（Chrome 行为）：今天 / 昨天 / 7 天内星期几 / 更早显示日期；
/// 时间未知（旧版档案）显示「更早」。
fn history_group_header(group: HistoryGroup, today: i64, lang: zero_browser_shell::UiLanguage) -> String {
    use zero_browser_shell::UiLanguage;
    let (zh, en) = match group {
        HistoryGroup::Unknown => ("更早", "Earlier"),
        HistoryGroup::Day(days) => match today - days {
            0 => ("今天", "Today"),
            1 => ("昨天", "Yesterday"),
            2..=6 => {
                let name_zh = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
                let name_en = [
                    "Sunday",
                    "Monday",
                    "Tuesday",
                    "Wednesday",
                    "Thursday",
                    "Friday",
                    "Saturday",
                ];
                let index = weekday_index(days);
                return match lang {
                    UiLanguage::ZhCn => name_zh[index].to_string(),
                    UiLanguage::EnUs => name_en[index].to_string(),
                };
            }
            _ => {
                let (year, month, day) = civil_from_days(days);
                let months_en = [
                    "January",
                    "February",
                    "March",
                    "April",
                    "May",
                    "June",
                    "July",
                    "August",
                    "September",
                    "October",
                    "November",
                    "December",
                ];
                return match lang {
                    UiLanguage::ZhCn => format!("{year}年{month}月{day}日"),
                    UiLanguage::EnUs => format!("{} {day}, {year}", months_en[(month - 1) as usize]),
                };
            }
        },
    };
    match lang {
        UiLanguage::ZhCn => zh.to_string(),
        UiLanguage::EnUs => en.to_string(),
    }
}

/// 历史条目相对时间文本（Chrome 行为：每条记录旁显示相对时间）。
///
/// `now` 与 `visited_at` 均为 Unix 秒。`visited_at == 0`（旧版档案条目）或
/// 系统时钟早于 epoch 时不显示时间，返回 `None`；未来时间戳（时钟偏差）按
/// 刚刚处理。
fn history_relative_time(visited_at: u64, now: u64, lang: zero_browser_shell::UiLanguage) -> Option<String> {
    use zero_browser_shell::UiLanguage;
    if visited_at == 0 {
        return None;
    }
    let elapsed = now.saturating_sub(visited_at);
    let (zh, en) = if elapsed < 60 {
        ("刚刚".to_string(), "just now".to_string())
    } else if elapsed < 3_600 {
        let mins = elapsed / 60;
        (format!("{mins} 分钟前"), plural_ago(mins, "minute"))
    } else if elapsed < 86_400 {
        let hours = elapsed / 3_600;
        (format!("{hours} 小时前"), plural_ago(hours, "hour"))
    } else {
        let days = elapsed / 86_400;
        (format!("{days} 天前"), plural_ago(days, "day"))
    };
    Some(match lang {
        UiLanguage::ZhCn => zh,
        UiLanguage::EnUs => en,
    })
}

/// 英文相对时间 "N <unit>(s) ago"，1 不用复数。
fn plural_ago(count: u64, unit: &str) -> String {
    if count == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{count} {unit}s ago")
    }
}

/// 当前 Unix 时间秒（系统时钟早于 epoch 时按 0 记）。
fn unix_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// 本地时区「今天」的 epoch 天序。
fn local_today_days() -> i64 {
    unix_secs_to_local_days(unix_seconds_now().min(i64::MAX as u64) as i64)
}

/// Unix 秒 → 本地日期的 epoch 天序。Unix 平台经 `localtime_r` 取本地时区；
/// 其他平台无时区库依赖，回退 UTC（分组标签可能整体偏移时区差，可接受降级）。
#[cfg(unix)]
fn unix_secs_to_local_days(secs: i64) -> i64 {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let time = secs as libc::time_t;
    // SAFETY：`tm` 指向本函数栈上有效内存；`localtime_r` 线程安全、不保留入参指针。
    unsafe {
        libc::localtime_r(&time, &mut tm);
    }
    days_from_civil(tm.tm_year as i64 + 1900, tm.tm_mon as i64 + 1, tm.tm_mday as i64)
}

#[cfg(not(unix))]
fn unix_secs_to_local_days(secs: i64) -> i64 {
    secs.div_euclid(86_400)
}

/// epoch 天序 → 星期几索引（0 = 周日；1970-01-01 为周四）。
fn weekday_index(days: i64) -> usize {
    (days + 4).rem_euclid(7) as usize
}

/// 公历 (年, 月, 日) → epoch 天序（1970-01-01 = 0）。
/// Howard Hinnant `days_from_civil` 算法（闰年/400 年周期精确）。
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_index = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// epoch 天序 → 公历 (年, 月, 日)。Howard Hinnant `civil_from_days` 算法
///（[`days_from_civil`] 的逆变换）。
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// 生成新标签页 HTML。
///
/// 在静态欢迎页模板基础上注入两块内容：
/// - 中央搜索框（Chrome 新标签页行为）：表单直接以当前搜索引擎的结果页 URL 为
///   action（多进程下表单提交由渲染端自导航，`zero://` 动作不可达，故不走路由）；
///   百度查询参数为 `wd`，其余为 `q`。
/// - 历史记录中最新的站点（按 host 去重、时间倒序，最多 5 个，仅 http/https）
///   注入 Explore 区域上方；无历史时该区域为空，默认快捷方式始终保留在下方。
pub fn generate_welcome_html(
    history: &zero_browser_shell::History,
    search_engine: zero_browser_shell::SearchEngine,
) -> String {
    use zero_browser_shell::SearchEngine;
    let (action, query_key) = match search_engine {
        SearchEngine::Baidu => ("https://www.baidu.com/s", "wd"),
        SearchEngine::Google => ("https://www.google.com/search", "q"),
        SearchEngine::Bing => ("https://www.bing.com/search", "q"),
        SearchEngine::DuckDuckGo => ("https://duckduckgo.com/", "q"),
    };
    let search_box = format!(
        r#"<form class="search-form" action="{action}" method="get"><input class="search-input" type="text" name="{query_key}" value="" placeholder="Search the web · 搜索或输入网址" aria-label="Search the web · 搜索或输入网址"></form>"#,
    );
    let mut seen_hosts = std::collections::HashSet::new();
    let mut tiles = String::new();
    for entry in history.iter() {
        let url = entry.url();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            continue;
        }
        let Some(host) = url_host(url) else {
            continue;
        };
        if !seen_hosts.insert(host) {
            continue;
        }
        let host_display = host.to_string();
        let title = entry.title().trim();
        let title = if title.is_empty() { &host_display } else { title };
        tiles.push_str(&format!(
            r#"<a class="link-tile" href="{href}"><span class="link-title">{label}</span><span class="link-sub">{sub}</span></a>"#,
            href = html_escape(url),
            label = html_escape(title),
            sub = html_escape(&host_display),
        ));
        if seen_hosts.len() >= 5 {
            break;
        }
    }
    let recent = if tiles.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="links" style="margin-bottom:14px;">{tiles}</div>"#)
    };
    WELCOME_HTML
        .replace("<!-- search-box -->", &search_box)
        .replace("<!-- recent-sites -->", &recent)
}

/// 提取 http/https URL 的 host 段（含端口；无 `://` 前缀或空 host 返回 None）。
fn url_host(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() { None } else { Some(authority) }
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
///
/// `query` 为搜索框提交的过滤词（`zero://bookmarks?q=…`）；非空时按标题或 URL
/// 子串过滤（`Bookmarks::search`，大小写不敏感，同历史页模式）。每条书签附带编辑
///（`zero://bookmarks/edit?id=…`）与删除（`zero://bookmarks/delete?id=…`）链接；
/// 删除/编辑链接回跳时携带当前过滤词，操作一条后仍停留在同一过滤视图。
pub fn generate_bookmarks_html(bookmarks: &zero_browser_shell::Bookmarks, query: Option<&str>) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, empty, search_placeholder, search_label, edit_label, delete_label, no_match) = match lang {
        UiLanguage::ZhCn => (
            "书签 — ZeroBrowser",
            "暂无书签。",
            "搜索书签",
            "搜索",
            "编辑",
            "删除",
            "没有匹配的书签。",
        ),
        UiLanguage::EnUs => (
            "Bookmarks — ZeroBrowser",
            "No bookmarks yet.",
            "Search bookmarks",
            "Search",
            "Edit",
            "Delete",
            "No matching bookmarks.",
        ),
    };
    let active_query = query.unwrap_or("").trim();
    let rows: Vec<&zero_browser_shell::Bookmark> = if active_query.is_empty() {
        bookmarks.list_root().into_iter().collect()
    } else {
        bookmarks.search(active_query).collect()
    };
    // 编辑/删除链接回跳时携带当前过滤词，操作一条后仍停留在同一过滤视图。
    let query_suffix = if active_query.is_empty() {
        String::new()
    } else {
        format!("&q={}", percent_encode_query_value(active_query))
    };
    let search_box = format!(
        r#"<form action="zero://bookmarks" method="get" style="margin:0 0 16px;">
  <input type="text" name="q" value="{value}" placeholder="{search_placeholder}" aria-label="{search_placeholder}" style="font-size:14px;padding:6px 10px;border:1px solid #dadce0;border-radius:4px;width:320px;">
  <button type="submit" style="font-size:14px;padding:6px 16px;margin-left:8px;background:#1a73e8;color:#fff;border:none;border-radius:4px;">{search_label}</button>
</form>"#,
        value = html_escape(active_query),
    );
    let body = if rows.is_empty() {
        let empty_text = if active_query.is_empty() { empty } else { no_match };
        format!(r#"{search_box}<p style="color:#666;">{empty_text}</p>"#)
    } else {
        let mut items = String::new();
        for bookmark in rows {
            items.push_str(&format!(
                r#"<li style="margin:8px 0;"><a href="{href}" style="color:#1a73e8;text-decoration:none;">{label}</a><br><span style="font-size:12px;color:#80868b;">{href}</span> <a href="zero://bookmarks/edit?id={id}{query_suffix}" style="font-size:12px;color:#1a73e8;text-decoration:none;margin-left:8px;">{edit_label}</a> <a href="zero://bookmarks/delete?id={id}{query_suffix}" style="font-size:12px;color:#d93025;text-decoration:none;margin-left:8px;">{delete_label}</a></li>"#,
                href = html_escape(bookmark.url()),
                label = html_escape(bookmark.title()),
                id = bookmark.id().0,
            ));
        }
        format!(r#"{search_box}<ul style="padding-left:20px;">{items}</ul>"#)
    };
    internal_page_shell(title, &body)
}

/// 生成书签编辑表单页 HTML（`zero://bookmarks/edit?id=…`）。
///
/// 表单 GET 提交到 `zero://bookmarks/save`（id/title/url + 隐藏 `q` 保留过滤词）；
/// 取消链接回书签列表（同样保留过滤词）。
pub fn generate_bookmarks_edit_html(bookmark: &zero_browser_shell::Bookmark, query: Option<&str>) -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let (title, name_label, url_label, save_label, cancel_label) = match lang {
        UiLanguage::ZhCn => ("编辑书签 — ZeroBrowser", "名称", "网址", "保存", "取消"),
        UiLanguage::EnUs => ("Edit Bookmark — ZeroBrowser", "Name", "URL", "Save", "Cancel"),
    };
    let active_query = query.unwrap_or("").trim();
    let query_suffix = if active_query.is_empty() {
        String::new()
    } else {
        format!("?q={}", percent_encode_query_value(active_query))
    };
    let q_hidden = if active_query.is_empty() {
        String::new()
    } else {
        format!(
            r#"  <input type="hidden" name="q" value="{}">"#,
            html_escape(active_query)
        )
    };
    let body = format!(
        r#"<form action="zero://bookmarks/save" method="get">
  <input type="hidden" name="id" value="{id}">
{q_hidden}
  <p><label>{name_label}: <input type="text" name="title" value="{title_value}" aria-label="{name_label}" style="font-size:14px;padding:6px 10px;border:1px solid #dadce0;border-radius:4px;width:420px;"></label></p>
  <p><label>{url_label}: <input type="text" name="url" value="{url_value}" aria-label="{url_label}" style="font-size:14px;padding:6px 10px;border:1px solid #dadce0;border-radius:4px;width:420px;"></label></p>
  <button type="submit" style="font-size:14px;padding:6px 16px;background:#1a73e8;color:#fff;border:none;border-radius:4px;">{save_label}</button>
  <a href="zero://bookmarks{query_suffix}" style="margin-left:12px;color:#1a73e8;text-decoration:none;font-size:13px;">{cancel_label}</a>
</form>"#,
        id = bookmark.id().0,
        title_value = html_escape(bookmark.title()),
        url_value = html_escape(bookmark.url()),
    );
    internal_page_shell(title, &body)
}

/// 生成键盘快捷键帮助页 HTML（`zero://shortcuts`，Ctrl+/ 打开）。
///
/// 键位清单与 `app_input_keys.rs` 的实际绑定同源维护；分组表格呈现
///（导航 / 标签页 / 查找 / 缩放 / 其他），键位用 kbd 风格 code 元素。
pub fn generate_shortcuts_html() -> String {
    use zero_browser_shell::UiLanguage;
    let lang = UiLanguage::detect_from_env();
    let groups: Vec<(&str, Vec<(&str, &str)>)> = if matches!(lang, UiLanguage::ZhCn) {
        vec![
            (
                "导航",
                vec![
                    ("Alt + ←", "后退"),
                    ("Alt + →", "前进"),
                    ("Alt + Home", "打开主页"),
                    ("Ctrl + R 或 F5", "重新加载"),
                    ("Ctrl + Shift + R 或 Ctrl + F5", "强制重新加载（绕过缓存）"),
                    ("Ctrl + L", "聚焦地址栏"),
                    ("Esc", "停止加载"),
                ],
            ),
            (
                "标签页",
                vec![
                    ("Ctrl + T", "新建标签页"),
                    ("Ctrl + W", "关闭标签页"),
                    ("Ctrl + Shift + T", "重新打开已关闭的标签页"),
                    ("Ctrl + Shift + N", "新建无痕窗口"),
                    ("Ctrl + Shift + W", "关闭窗口"),
                    ("Ctrl + 1 ~ 8", "切换到对应标签页"),
                    ("Ctrl + 9", "切换到最后一个标签页"),
                    ("Ctrl + Tab 或 Ctrl + PageDown", "下一个标签页"),
                    ("Ctrl + Shift + Tab 或 Ctrl + PageUp", "上一个标签页"),
                ],
            ),
            (
                "查找",
                vec![
                    ("Ctrl + F", "页面内查找"),
                    ("Ctrl + G 或 F3", "查找下一个"),
                    ("Ctrl + Shift + G 或 Shift + F3", "查找上一个"),
                    ("Enter", "下一个（查找栏聚焦时）"),
                    ("Shift + Enter", "上一个（查找栏聚焦时）"),
                    ("Esc", "关闭查找栏"),
                ],
            ),
            (
                "缩放",
                vec![("Ctrl + +", "放大"), ("Ctrl + -", "缩小"), ("Ctrl + 0", "重置为 100%")],
            ),
            (
                "其他",
                vec![
                    ("Ctrl + D", "收藏 / 取消收藏当前页面"),
                    ("Ctrl + Shift + B", "显示 / 隐藏书签栏"),
                    ("Ctrl + H", "历史记录"),
                    ("Ctrl + J", "下载内容"),
                    ("Ctrl + ,", "设置"),
                    ("Ctrl + /", "键盘快捷键（本页）"),
                    ("Ctrl + P", "打印预览"),
                    ("Ctrl + Home / Ctrl + End", "滚动到页面顶部 / 底部"),
                    ("F11", "切换全屏"),
                ],
            ),
        ]
    } else {
        vec![
            (
                "Navigation",
                vec![
                    ("Alt + ←", "Go back"),
                    ("Alt + →", "Go forward"),
                    ("Alt + Home", "Open home page"),
                    ("Ctrl + R or F5", "Reload"),
                    ("Ctrl + Shift + R or Ctrl + F5", "Hard reload (bypass cache)"),
                    ("Ctrl + L", "Focus address bar"),
                    ("Esc", "Stop loading"),
                ],
            ),
            (
                "Tabs",
                vec![
                    ("Ctrl + T", "New tab"),
                    ("Ctrl + W", "Close tab"),
                    ("Ctrl + Shift + T", "Reopen closed tab"),
                    ("Ctrl + Shift + N", "New incognito window"),
                    ("Ctrl + Shift + W", "Close window"),
                    ("Ctrl + 1 ~ 8", "Switch to tab by index"),
                    ("Ctrl + 9", "Switch to last tab"),
                    ("Ctrl + Tab or Ctrl + PageDown", "Next tab"),
                    ("Ctrl + Shift + Tab or Ctrl + PageUp", "Previous tab"),
                ],
            ),
            (
                "Find",
                vec![
                    ("Ctrl + F", "Find in page"),
                    ("Ctrl + G or F3", "Find next"),
                    ("Ctrl + Shift + G or Shift + F3", "Find previous"),
                    ("Enter", "Next match (find bar focused)"),
                    ("Shift + Enter", "Previous match (find bar focused)"),
                    ("Esc", "Close find bar"),
                ],
            ),
            (
                "Zoom",
                vec![
                    ("Ctrl + +", "Zoom in"),
                    ("Ctrl + -", "Zoom out"),
                    ("Ctrl + 0", "Reset to 100%"),
                ],
            ),
            (
                "Other",
                vec![
                    ("Ctrl + D", "Bookmark / unbookmark current page"),
                    ("Ctrl + Shift + B", "Show / hide bookmarks bar"),
                    ("Ctrl + H", "History"),
                    ("Ctrl + J", "Downloads"),
                    ("Ctrl + ,", "Settings"),
                    ("Ctrl + /", "Keyboard shortcuts (this page)"),
                    ("Ctrl + P", "Print preview"),
                    ("Ctrl + Home / Ctrl + End", "Scroll to top / bottom"),
                    ("F11", "Toggle fullscreen"),
                ],
            ),
        ]
    };
    let mut body = String::new();
    for (group, entries) in groups {
        let mut rows = String::new();
        for (keys, action) in entries {
            rows.push_str(&format!(
                r#"<tr><td style="padding:6px 16px 6px 0;white-space:nowrap;"><code style="display:inline-block;background:#f1f3f4;border:1px solid #dadce0;border-radius:4px;padding:2px 8px;font-family:monospace;font-size:12px;color:#202124;">{keys}</code></td><td style="padding:6px 0;color:#3c4043;white-space:nowrap;">{action}</td></tr>"#,
            ));
        }
        body.push_str(&format!(
            r#"<h2 style="font-size:14px;font-weight:600;color:#202124;margin:20px 0 8px;">{group}</h2><table style="border-collapse:collapse;font-size:13px;">{rows}</table>"#,
        ));
    }
    let title = match lang {
        UiLanguage::ZhCn => "键盘快捷键 — ZeroBrowser",
        UiLanguage::EnUs => "Keyboard shortcuts — ZeroBrowser",
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
        let html = generate_history_html(&history, None);
        assert!(html.contains("zero://history/clear"));
        // 标题随 UiLanguage::detect_from_env() 变化（EnUs "History" / ZhCn "历史"），
        // 须同时接受两种 locale（与 generate_downloads_html_renders_empty_state 同模式）。
        assert!(html.contains("History") || html.contains("历史"));
    }

    #[test]
    fn generate_history_html_lists_entries_with_delete_links() {
        let mut history = zero_browser_shell::History::default();
        history.record("https://example.com/page?a=1&b=2", "Example");
        let html = generate_history_html(&history, None);
        // 单条删除链接：原 URL 作为 query 值时 `&`/`?`/`=` 必须已转义。
        assert!(html.contains("zero://history/remove?url="));
        assert!(html.contains("a%3D1%26b%3D2"));
        // 搜索框提交回 zero://history。
        assert!(html.contains(r#"action="zero://history""#));
        assert!(html.contains(r#"name="q""#));
    }

    #[test]
    fn generate_history_html_filters_by_query() {
        let mut history = zero_browser_shell::History::default();
        history.record("https://example.com/a", "Example");
        history.record("https://www.rust-lang.org/", "Rust Blog");
        let html = generate_history_html(&history, Some("rust"));
        assert!(html.contains("rust-lang.org"));
        assert!(
            !html.contains("example.com/a"),
            "non-matching entry must be filtered out"
        );
        // 过滤视图下的删除链接保留过滤词。
        assert!(html.contains("&q=rust"));
    }

    #[test]
    fn calendar_days_roundtrip_and_known_dates() {
        for &(year, month, day) in &[
            (1970, 1, 1),
            (2000, 2, 29),
            (2026, 10, 15),
            (1969, 12, 31),
            (2100, 3, 1),
        ] {
            let days = days_from_civil(year, month, day);
            assert_eq!(civil_from_days(days), (year, month, day), "{year}-{month}-{day}");
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn weekday_index_matches_known_weekdays() {
        // 1970-01-01 周四；2026-10-15 周四；2026-10-18 周日。
        assert_eq!(weekday_index(days_from_civil(1970, 1, 1)), 4);
        assert_eq!(weekday_index(days_from_civil(2026, 10, 15)), 4);
        assert_eq!(weekday_index(days_from_civil(2026, 10, 18)), 0);
    }

    #[test]
    fn history_group_header_labels_by_recency() {
        let today = days_from_civil(2026, 10, 15);
        let cases = [
            (HistoryGroup::Day(today), ("今天", "Today")),
            (HistoryGroup::Day(today - 1), ("昨天", "Yesterday")),
            (HistoryGroup::Day(today - 2), ("周二", "Tuesday")),
            (HistoryGroup::Day(today - 6), ("周五", "Friday")),
            (HistoryGroup::Day(today - 7), ("2026年10月8日", "October 8, 2026")),
            (HistoryGroup::Unknown, ("更早", "Earlier")),
        ];
        for (group, (zh, en)) in cases {
            assert_eq!(
                history_group_header(group, today, zero_browser_shell::UiLanguage::ZhCn),
                zh
            );
            assert_eq!(
                history_group_header(group, today, zero_browser_shell::UiLanguage::EnUs),
                en
            );
        }
    }

    #[test]
    fn generate_history_html_groups_today_entries() {
        let mut history = zero_browser_shell::History::default();
        history.record("https://example.com/a", "A");
        history.record("https://example.com/b", "B");
        let html = generate_history_html(&history, None);
        // 当天条目归入「今天 / Today」组（locale 随环境，二选一）。
        assert!(html.contains(">今天<") || html.contains(">Today<"));
        // 两条当天条目落在同一组：分组标题只出现一次。
        let today_headers = html.matches(">今天<").count() + html.matches(">Today<").count();
        assert_eq!(today_headers, 1);
    }

    #[test]
    fn generate_history_html_files_legacy_entries_under_unknown_group() {
        // 旧版档案条目无时间戳：反序列化后 visited_at == 0，归入「更早 / Earlier」组。
        let history: zero_browser_shell::History =
            serde_json::from_str(r#"{"entries":[{"url":"https://old.example/","title":"Old"}]}"#)
                .expect("legacy history JSON must deserialize");
        let html = generate_history_html(&history, None);
        assert!(html.contains("old.example"));
        assert!(html.contains(">更早<") || html.contains(">Earlier<"));
    }

    #[test]
    fn history_relative_time_buckets_by_elapsed() {
        use zero_browser_shell::UiLanguage;
        let now = 1_800_000_000u64;
        let cases = [
            (now, ("刚刚", "just now")),
            (now - 59, ("刚刚", "just now")),
            (now - 60, ("1 分钟前", "1 minute ago")),
            (now - 600, ("10 分钟前", "10 minutes ago")),
            (now - 3_599, ("59 分钟前", "59 minutes ago")),
            (now - 3_600, ("1 小时前", "1 hour ago")),
            (now - 7_200, ("2 小时前", "2 hours ago")),
            (now - 86_399, ("23 小时前", "23 hours ago")),
            (now - 86_400, ("1 天前", "1 day ago")),
            (now - 3 * 86_400, ("3 天前", "3 days ago")),
        ];
        for (visited_at, (zh, en)) in cases {
            assert_eq!(
                history_relative_time(visited_at, now, UiLanguage::ZhCn).as_deref(),
                Some(zh)
            );
            assert_eq!(
                history_relative_time(visited_at, now, UiLanguage::EnUs).as_deref(),
                Some(en)
            );
        }
    }

    #[test]
    fn history_relative_time_hides_unknown_timestamp_and_future() {
        use zero_browser_shell::UiLanguage;
        // 旧版档案条目（visited_at == 0）不显示时间。
        assert_eq!(history_relative_time(0, 1_000, UiLanguage::EnUs), None);
        // 未来时间戳（时钟偏差）按刚刚处理，不出现负数。
        assert_eq!(
            history_relative_time(2_000, 1_000, UiLanguage::EnUs).as_deref(),
            Some("just now")
        );
    }

    #[test]
    fn generate_history_html_shows_relative_time_per_entry() {
        use zero_browser_shell::UiLanguage;
        let mut history = zero_browser_shell::History::default();
        history.record("https://example.com/recent", "Recent");
        let lang = UiLanguage::detect_from_env();
        let now = unix_seconds_now();
        let html = generate_history_html(&history, None);
        // 刚记录的条目 elapsed < 60s，两种 locale 下分别显示刚刚/just now。
        let expected = history_relative_time(history.iter().next().expect("entry recorded").visited_at(), now, lang)
            .expect("fresh entry has relative time");
        assert!(html.contains(&expected), "html must contain relative time {expected}");
        // 旧版档案条目（无时间戳）不显示相对时间。
        let legacy: zero_browser_shell::History =
            serde_json::from_str(r#"{"entries":[{"url":"https://old.example/","title":"Old"}]}"#)
                .expect("legacy history JSON must deserialize");
        let legacy_html = generate_history_html(&legacy, None);
        assert!(!legacy_html.contains("just now"));
        assert!(!legacy_html.contains("刚刚<"));
    }

    #[test]
    fn generate_shortcuts_html_lists_grouped_shortcuts() {
        let html = generate_shortcuts_html();
        // 分组标题随环境 locale（zh 或 en），每组应至少出现其一。
        let zh_groups = ["导航", "标签页", "查找", "缩放", "其他"];
        let en_groups = ["Navigation", "Tabs", "Find", "Zoom", "Other"];
        assert!(
            zh_groups.iter().all(|g| html.contains(g)) || en_groups.iter().all(|g| html.contains(g)),
            "group headers missing in either locale"
        );
        // 代表性键位（导航 / 标签页 / 查找 / 缩放 / 其他各一）。
        for keys in ["Ctrl + R", "Ctrl + T", "Ctrl + F", "Ctrl + 0", "Ctrl + /"] {
            assert!(html.contains(keys), "missing shortcut {keys}");
        }
    }

    #[test]
    fn generate_error_page_has_retry_and_escaped_error() {
        let html = generate_error_page("http://127.0.0.1:9/x", "<script>alert(1)</script>");
        // 重试按钮与可点击 URL 均指向原 URL。
        assert_eq!(html.matches(r#"href="http://127.0.0.1:9/x""#).count(), 2);
        // 错误文字（远端派生）必须转义，不得注入标签。
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
        // 标题与 tab 标题同源。
        assert!(html.contains(error_page_tab_title()));
    }

    #[test]
    fn generate_welcome_html_injects_recent_sites_deduped_by_host() {
        let mut history = zero_browser_shell::History::default();
        history.record("https://example.com/second", "Example Second");
        history.record("https://example.com/first", "Example First");
        history.record("https://www.rust-lang.org/", "Rust");
        history.record("zero://settings", "Settings");
        let html = generate_welcome_html(&history, zero_browser_shell::SearchEngine::Google);
        // 同 host 去重：保留时间最新的一条（record 前插，最新在前）。
        assert!(html.contains("https://example.com/first"));
        assert!(!html.contains("https://example.com/second"));
        assert!(html.contains("https://www.rust-lang.org/"));
        // 默认快捷方式仍保留在下方。
        assert!(html.contains("https://github.com/leizongmin/ZeroWeb"));
    }

    #[test]
    fn generate_welcome_html_search_form_targets_current_engine() {
        let history = zero_browser_shell::History::default();
        let google = generate_welcome_html(&history, zero_browser_shell::SearchEngine::Google);
        assert!(google.contains(r#"action="https://www.google.com/search""#));
        assert!(google.contains(r#"name="q""#));
        let baidu = generate_welcome_html(&history, zero_browser_shell::SearchEngine::Baidu);
        assert!(baidu.contains(r#"action="https://www.baidu.com/s""#));
        assert!(baidu.contains(r#"name="wd""#));
    }

    #[test]
    fn generate_welcome_html_without_history_keeps_template() {
        let history = zero_browser_shell::History::default();
        let html = generate_welcome_html(&history, zero_browser_shell::SearchEngine::Google);
        // 无历史时 recent-sites 占位符替换为空串（不出现占位注释残留），
        // 默认快捷方式与中央搜索框保留。
        assert!(!html.contains("recent-sites"), "占位注释必须被替换");
        assert!(html.contains(r#"class="search-form""#));
        assert!(html.contains("https://github.com/leizongmin/ZeroWeb"));
    }

    #[test]
    fn percent_encode_query_value_escapes_delimiters() {
        assert_eq!(
            percent_encode_query_value("http://127.0.0.1:9/page?a=1&b=2"),
            "http%3A%2F%2F127.0.0.1%3A9%2Fpage%3Fa%3D1%26b%3D2"
        );
        assert_eq!(percent_encode_query_value("safe-._~"), "safe-._~");
    }

    #[test]
    fn url_host_extracts_authority() {
        assert_eq!(url_host("https://example.com/a?b=c"), Some("example.com"));
        assert_eq!(url_host("http://127.0.0.1:18080/x"), Some("127.0.0.1:18080"));
        assert_eq!(url_host("zero://newtab"), Some("newtab"));
        assert_eq!(url_host("https:///path-only"), None);
    }

    #[test]
    fn generate_settings_html_renders_switch_controls_with_aria() {
        let mut settings = zero_browser_shell::BrowserSettings::new();
        settings.javascript_enabled = true;
        settings.do_not_track = false;
        let html = generate_settings_html(&settings);
        // 开=蓝色轨道 + aria-checked=true；关=灰色轨道 + aria-checked=false。
        assert!(html.contains(r#"role="switch""#));
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("aria-checked=\"false\""));
        assert!(html.contains("background:#1a73e8;align-items:center;justify-content:flex-end"));
        assert!(html.contains("background:#bdc1c6;align-items:center;justify-content:flex-start"));
        // 状态文字与既有动作 URL 仍在。
        assert!(html.contains("zero://settings/toggle/javascript_enabled"));
        assert!(html.contains("zero://settings/cycle/color_theme"));
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
        let html = generate_bookmarks_html(&bookmarks, None);
        assert!(html.contains("Bookmarks") || html.contains("书签"));
    }

    #[test]
    fn generate_bookmarks_html_lists_entries_with_edit_delete_links() {
        let mut bookmarks = zero_browser_shell::Bookmarks::default();
        let id = bookmarks.add("Example", "https://example.com/page?a=1&b=2", None);
        let html = generate_bookmarks_html(&bookmarks, None);
        // 编辑/删除链接按数字 ID 携带，回跳保留过滤词的挂点在链接尾。
        assert!(html.contains(&format!("zero://bookmarks/edit?id={}", id.0)));
        assert!(html.contains(&format!("zero://bookmarks/delete?id={}", id.0)));
        // 搜索框提交回 zero://bookmarks。
        assert!(html.contains(r#"action="zero://bookmarks""#));
        assert!(html.contains(r#"name="q""#));
    }

    #[test]
    fn generate_bookmarks_html_filters_by_query() {
        let mut bookmarks = zero_browser_shell::Bookmarks::default();
        bookmarks.add("Example", "https://example.com/a", None);
        bookmarks.add("Rust Blog", "https://www.rust-lang.org/", None);
        let html = generate_bookmarks_html(&bookmarks, Some("rust"));
        assert!(html.contains("rust-lang.org"));
        assert!(
            !html.contains("example.com/a"),
            "non-matching entry must be filtered out"
        );
        // 过滤视图下的编辑/删除链接保留过滤词（`&q=` 由 percent_encode 编码）。
        assert!(html.contains("&q=rust"));
    }

    #[test]
    fn generate_bookmarks_edit_html_prefills_form() {
        let mut bookmarks = zero_browser_shell::Bookmarks::default();
        let id = bookmarks.add("My Site", "https://example.com/x?y=1", None);
        let bookmark = bookmarks.get(id).unwrap();
        let html = generate_bookmarks_edit_html(bookmark, Some("rust"));
        assert!(html.contains(r#"action="zero://bookmarks/save""#));
        assert!(html.contains(&format!(r#"name="id" value="{}""#, id.0)));
        // 属性值经 HTML 转义（URL 含 `?`/`&` 类字符不破坏属性边界）。
        assert!(html.contains(r#"name="url" value="https://example.com/x?y=1""#));
        assert!(html.contains(r#"name="title" value="My Site""#));
        // 过滤词经隐藏域随表单保留。
        assert!(html.contains(r#"name="q" value="rust""#));
        // 取消链接回书签列表（保留过滤词）。
        assert!(html.contains(r#"href="zero://bookmarks?q=rust""#));
    }

    #[test]
    fn percent_decode_form_value_roundtrip() {
        // 表单 urlencoded 语义：空格→`+`、`+`→`%2B`；解码顺序 `+` 先替换。
        assert_eq!(percent_decode_form_value("a+b"), "a b");
        assert_eq!(percent_decode_form_value("C%2B%2B+%E6%95%99%E7%A8%8B"), "C++ 教程");
        assert_eq!(
            percent_decode_form_value("https%3A%2F%2Fexample.com%2Fx"),
            "https://example.com/x"
        );
        // 截断的 %XX 按字面保留。
        assert_eq!(percent_decode_form_value("100%"), "100%");
    }
}
