//! WebView API 性能基准测试。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zero_webview_api::{WebView, WebViewBuilder, WebViewConfig};

/// 基准：通过 Builder 创建 WebView
fn bench_webview_builder_create(c: &mut Criterion) {
    c.bench_function("webview_builder_create", |b| {
        b.iter(|| {
            let _wv = WebViewBuilder::new()
                .width(1024)
                .height(768)
                .user_agent("BenchBot/1.0")
                .transparent(false)
                .build();
        })
    });
}

/// 基准：加载简单 HTML 并渲染
fn bench_webview_load_html_simple(c: &mut Criterion) {
    let html = r#"<html><body><div>Hello</div></body></html>"#;
    c.bench_function("webview_load_html_simple", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig::default());
            wv.load_html(black_box(html), None);
        })
    });
}

/// 基准：加载带 CSS 的 HTML
fn bench_webview_load_html_with_css(c: &mut Criterion) {
    let html = r#"<html><body><div class="box">Content</div></body></html>"#;
    let css = r#".box { width: 200px; height: 100px; background-color: red; margin: 10px; }"#;
    c.bench_function("webview_load_html_with_css", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig::default());
            wv.load_html(black_box(html), Some(black_box(css)));
        })
    });
}

/// 基准：重新渲染
fn bench_webview_render(c: &mut Criterion) {
    let html = r#"<html><body><div>Render benchmark</div></body></html>"#;
    c.bench_function("webview_render", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig::default());
            wv.load_html(html, None);
            wv.render();
        })
    });
}

/// 基准：注入 CSS 重新渲染
fn bench_webview_inject_css(c: &mut Criterion) {
    let html = r#"<html><body><div id="app">Test</div></body></html>"#;
    c.bench_function("webview_inject_css", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig::default());
            wv.load_html(html, None);
            wv.inject_css(black_box("div { color: blue; font-size: 16px; }"));
        })
    });
}

/// 基准：复杂页面加载渲染
fn bench_webview_complex_page(c: &mut Criterion) {
    let html = r#"<html><head><title>Bench</title></head><body>
        <header><h1>Title</h1><nav><a>Link 1</a><a>Link 2</a></nav></header>
        <main><section><p>Paragraph 1</p><p>Paragraph 2</p></section>
        <aside><div>Sidebar</div></aside></main>
        <footer><p>Footer</p></footer>
    </body></html>"#;
    let css = r#"
        body { margin: 0; font-family: sans-serif; }
        header { background: #333; color: white; padding: 10px; }
        main { display: flex; padding: 20px; }
        section { flex: 1; }
        aside { width: 200px; }
        footer { background: #eee; padding: 10px; }
    "#;
    c.bench_function("webview_complex_page", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig {
                width: 1440,
                height: 900,
                ..Default::default()
            });
            wv.load_html(black_box(html), Some(black_box(css)));
        })
    });
}

/// 基准：resize 后重新渲染
fn bench_webview_resize_and_render(c: &mut Criterion) {
    let html = r#"<html><body><div>Resize test</div></body></html>"#;
    c.bench_function("webview_resize_and_render", |b| {
        b.iter(|| {
            let mut wv = WebView::new(WebViewConfig::default());
            wv.load_html(html, None);
            wv.resize(1024, 768);
            wv.render();
        })
    });
}

criterion_group!(
    benches,
    bench_webview_builder_create,
    bench_webview_load_html_simple,
    bench_webview_load_html_with_css,
    bench_webview_render,
    bench_webview_inject_css,
    bench_webview_complex_page,
    bench_webview_resize_and_render,
);
criterion_main!(benches);
