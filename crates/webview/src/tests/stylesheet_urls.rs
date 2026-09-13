//! 外链字体请求必须以声明所在样式表为基准，不能落到文档目录。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};

use crate::{AsyncPageLoad, WebView, WebViewConfig};

struct StylesheetHost {
    stylesheets: HashMap<&'static str, &'static str>,
    requests: Vec<String>,
}

impl AsyncFetchHost for StylesheetHost {
    fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        self.requests.push(url.to_string());
        let (tx, rx) = channel();
        tx.send(
            self.stylesheets
                .get(url)
                .map(|text| text.to_string())
                .ok_or_else(|| format!("unexpected stylesheet: {url}")),
        )
        .unwrap();
        rx
    }

    fn fetch_bytes_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        self.requests.push(url.to_string());
        let (tx, rx) = channel();
        tx.send(Ok(Vec::new())).unwrap();
        rx
    }
}

#[test]
fn external_and_imported_fonts_use_their_stylesheet_base() {
    // https://drafts.csswg.org/css-values-4/#relative-urls
    let html = r#"<link rel="stylesheet" href="/assets/theme/main.css">
        <style>@font-face { font-family: Inline; src: url(fonts/inline.ttf) }</style>"#;
    let mut load = AsyncPageLoad::from_html("https://example.test/articles/post.html", html.to_string());
    let mut webview = WebView::new(WebViewConfig::default());
    let mut host = StylesheetHost {
        stylesheets: HashMap::from([
            (
                "https://example.test/assets/theme/main.css",
                r#"@import url("nested/extra.css");
                   @font-face { font-family: Main; src: url("../fonts/main.ttf") }"#,
            ),
            (
                "https://example.test/assets/theme/nested/extra.css",
                r#"@font-face { font-family: Extra; src: url(../fonts/extra.ttf) }"#,
            ),
        ]),
        requests: Vec::new(),
    };
    for _ in 0..50 {
        if !load.is_active() {
            break;
        }
        let _ = load.tick(&mut webview, &mut host, 500.0);
    }
    assert!(!load.is_active(), "load must settle");
    for expected in [
        "https://example.test/assets/fonts/main.ttf",
        "https://example.test/assets/theme/fonts/extra.ttf",
        "https://example.test/articles/fonts/inline.ttf",
    ] {
        assert!(
            host.requests.iter().any(|url| url == expected),
            "missing {expected}: {:?}",
            host.requests
        );
    }
    assert_eq!(
        host.requests.len(),
        5,
        "no duplicate or document-relative external requests"
    );
}
