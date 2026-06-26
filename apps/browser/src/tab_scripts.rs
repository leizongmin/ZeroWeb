//! 页面 `<script>` 执行 — Tab worker 内在加载完成后调用。

use tracing::warn;
use zero_engine::{PageScript, extract_page_scripts, resolve_document_url};
use zero_webview::WebView;

/// 按文档顺序执行页面脚本（内联 + 外链 `src`）。
pub fn run_page_scripts(wv: &mut WebView, javascript_enabled: bool) {
    if !javascript_enabled {
        return;
    }
    let html = wv.html_content().to_string();
    if html.is_empty() {
        return;
    }
    if !should_run_scripts_for_url(wv.url()) {
        return;
    }
    let base = wv.url().unwrap_or("about:blank").to_string();
    for script in extract_page_scripts(&html) {
        match script {
            PageScript::Inline(code) => {
                if let Err(e) = wv.execute_script_with_dom(&code) {
                    warn!("inline script error: {e}");
                }
            }
            PageScript::External(src) => {
                let abs = resolve_document_url(&base, &src);
                let code = match wv.fetch_text_at(&abs) {
                    Ok(code) => code,
                    Err(e) => {
                        warn!("external script fetch {abs}: {e}");
                        continue;
                    }
                };
                if let Err(e) = wv.execute_script_with_dom(&code) {
                    warn!("external script {abs} error: {e}");
                }
            }
        }
    }
}

fn should_run_scripts_for_url(url: Option<&str>) -> bool {
    match url {
        None | Some("") | Some("about:blank") => false,
        Some(u) if u.starts_with("zero://") => false,
        Some(u) if u.starts_with("view-source:") => false,
        _ => true,
    }
}
