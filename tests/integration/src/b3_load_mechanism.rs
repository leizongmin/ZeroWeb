//! B3 加载机制验证（in-process）。
//!
//! 验证 renderer 重写后将走的 load 路径：`WebView` 经 `AsyncPageLoad`（per-tick host）
//! 与 `BlockingFetchHost` 同步 drain 到完成并产出渲染——即 §11 B3-2 的核心机制。
//! 无需 spawn 子进程（避免 GPU/Display 与二进制定位的 flaky），直接覆盖 B3 load 路径。

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
use zero_page_runtime::BlockingFetchHost;
#[cfg(test)]
use zero_webview::{AsyncPageLoad, WebView, WebViewConfig};

/// 自包含 HTML（无外链）：`AsyncPageLoad` + `BlockingFetchHost` drain 须完成并渲染。
#[test]
fn b3_load_self_contained_drains_and_renders() {
    let html = r#"<html><head><title>T</title></head><body>
        <div style="width:200px;height:100px;background:red">Box</div>
    </body></html>"#;
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    // 自包含页无外链 → fetch 不会被调用；mock 返回 Err 也无碍。
    let mut host = BlockingFetchHost::new(|_url: &str| Err("no external resources".to_string()));
    let mut load = AsyncPageLoad::from_html("zero://test".to_string(), html.to_string());
    for _ in 0..1000 {
        if !load.is_active() {
            break;
        }
        load.tick(&mut wv, &mut host, 8.0);
    }
    assert!(!load.is_active(), "load 须在 1000 tick 内完成");
    let render = wv.last_render().expect("drain 后 WebView 须产出渲染结果");
    assert!(
        !(render.primitives().fills.is_empty()
            && render.primitives().rounded_rects.is_empty()
            && render.primitives().images.is_empty()),
        "须产出至少一个可见图元"
    );
}

/// 外链 CSS：`BlockingFetchHost` 须被调用、应用样式后完成渲染。
#[test]
fn b3_load_external_css_via_blocking_host() {
    let html = r#"<html><head><link rel="stylesheet" href="https://x/style.css"></head><body><div class="box">hi</div></body></html>"#;
    let css = ".box { width: 200px; height: 100px; background: blue; }";
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    let calls = AtomicUsize::new(0);
    let mut host = BlockingFetchHost::new(|url: &str| {
        calls.fetch_add(1, Ordering::SeqCst);
        if url.contains("style.css") {
            Ok(css.as_bytes().to_vec())
        } else {
            Err("not found".to_string())
        }
    });
    let mut load = AsyncPageLoad::from_html("https://x/".to_string(), html.to_string());
    for _ in 0..1000 {
        if !load.is_active() {
            break;
        }
        load.tick(&mut wv, &mut host, 8.0);
    }
    assert!(!load.is_active(), "load 须完成");
    assert!(calls.load(Ordering::SeqCst) > 0, "BlockingFetchHost 须为 <link> 被调用");
    assert!(wv.last_render().is_some(), "应用 CSS 后须渲染");
}
