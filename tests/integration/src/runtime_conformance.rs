//! T7 运行时一致性 gate。
//!
//! 同一份**自包含** HTML+CSS 经两条路径渲染，产出图元须一致：
//! - **engine-direct**（WPT 路径）：`RenderPipeline::render_html`
//! - **WebView**（TabWorker 路径）：`WebView::load_html` → `last_render`
//!
//! 这是「三路径同一套处理逻辑」的硬指标——验证渲染核心确实共享。
//! 自包含页（无外链/脚本）两路径都归约到同一 engine 调用，图元计数必须相等；
//! 不一致即暴露 WebView 与 engine 的渲染分叉。带外链/脚本的一致性需 B3（renderer 经 WebView）后覆盖。

#[cfg(test)]
use zero_engine::RenderPipeline;
#[cfg(test)]
use zero_render_foundation::primitive::RenderPrimitives;
#[cfg(test)]
use zero_webview::{WebView, WebViewConfig};

/// 图元总数：各类型 vec 长度之和。
#[cfg(test)]
fn primitive_count(p: &RenderPrimitives) -> usize {
    p.fills.len()
        + p.rounded_rects.len()
        + p.gradients.len()
        + p.shadows.len()
        + p.images.len()
        + p.strokes.len()
        + p.path_fills.len()
        + p.path_strokes.len()
        + p.clips.len()
}

/// 比较同一 HTML+CSS 经 engine-direct 与 WebView 的图元总数（须相等）。
#[cfg(test)]
fn assert_paths_match(label: &str, html: &str, css: &str, vw: u32, vh: u32) {
    // engine-direct（WPT 路径）
    let mut pipeline = RenderPipeline::new(vw as f32, vh as f32);
    let engine_result = pipeline.render_html(html, css);
    let engine_count = primitive_count(engine_result.primitives());

    // WebView（TabWorker 路径）
    let mut wv = WebView::new(WebViewConfig {
        width: vw,
        height: vh,
        ..Default::default()
    });
    wv.load_html(html, if css.is_empty() { None } else { Some(css) });
    let wv_render = wv.last_render().expect("WebView 须产出渲染结果");
    let wv_count = primitive_count(&wv_render.primitives);

    assert_eq!(
        engine_count, wv_count,
        "{label}: engine-direct({engine_count}) != WebView({wv_count}) —— 渲染核心分叉"
    );
}

/// 单个带样式 div：最基本一致性。
#[test]
fn t7_simple_styled_div() {
    let html = r#"<html><body><div style="width:200px;height:100px;background:red">Box</div></body></html>"#;
    assert_paths_match("simple styled div", html, "", 800, 600);
}

/// 多元素 + 外链 CSS（自包含内联）：验证复合页面一致性。
#[test]
fn t7_complex_page_with_css() {
    let html = r#"<html><body>
        <div class="card"><h1>Title</h1><p>Body text</p></div>
        <div class="card"><p>More</p></div>
    </body></html>"#;
    let css = r#".card { background:#eee; width:300px; height:150px; margin:10px; }"#;
    assert_paths_match("complex page with css", html, css, 1024, 768);
}

/// 圆角 + 阴影：验证非 fill 图元也一致。
#[test]
fn t7_rounded_rect_and_shadow() {
    let html = r#"<html><body>
        <div style="width:200px;height:120px;background:#3366cc;border-radius:12px;box-shadow:0 4px 8px rgba(0,0,0,0.3)">Card</div>
    </body></html>"#;
    assert_paths_match("rounded rect + shadow", html, "", 800, 600);
}

/// 不同视口尺寸：验证视口不影响两路径一致性。
#[test]
fn t7_viewport_independence() {
    let html = r#"<html><body><div style="width:50%;height:50%;background:green">Half</div></body></html>"#;
    assert_paths_match("viewport independence", html, "", 400, 500);
}
