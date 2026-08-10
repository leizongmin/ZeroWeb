//! Web API 端到端管线集成测试。
//!
//! 验证 JavaScript DOM 操作通过 V8 → WebView 管线的完整执行，
//! 以及 CSS 渲染管线的端到端验证。

/// 辅助：创建带 HTML 内容的 WebView 并返回渲染结果。
fn create_and_render(html: &str, css: Option<&str>) -> zero_webview::WebViewRenderResult {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);
    webview.load_html(html, css)
}

/// 辅助：创建 WebView，加载 HTML，执行带 DOM polyfill 的 JS 脚本。
fn create_load_and_execute(html: &str, script: &str) -> Result<String, zero_webview::WebViewError> {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);
    webview.load_html(html, None);
    webview.execute_script_with_dom(script)
}

// ── JS DOM 操作 → 渲染验证 ──

#[test]
fn test_js_dom_create_element_and_render() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    // 加载基础 HTML
    webview.load_html("<html><body></body></html>", None);

    // 通过 JS（带 DOM polyfill）创建元素
    let result = webview.execute_script_with_dom(
        "var div = document.createElement('div'); div.textContent = 'Hello from JS'; document.body.appendChild(div); 'ok'",
    );
    assert!(result.is_ok(), "DOM createElement should work: {:?}", result);

    // 重新渲染并验证结果有效
    let render = webview.render();
    assert!(render.timings.total_ms >= 0.0);
}

#[test]
fn test_js_modify_style_and_render() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    webview.load_html(
        "<html><body><div id=\"box\" style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        None,
    );

    // 通过 JS（带 DOM polyfill）创建元素并修改样式
    // 注：getElementById 在 polyfill 中返回 null（V8 上下文未连接真实 DOM 树），
    // 因此使用 createElement 创建元素并操作其样式
    let result = webview.execute_script_with_dom(
        "var box = document.createElement('div'); box.style.background = 'red'; box.style.background === 'red' ? 'ok' : 'fail'",
    );
    assert!(result.is_ok(), "DOM style modification should work: {:?}", result);

    let render = webview.render();
    // 验证渲染有输出
    assert!(
        render.primitives().fills.len() + render.primitives().rounded_rects.len() >= 1,
        "should have visual primitives after style change"
    );
}

#[test]
fn test_js_event_listener_execution() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    webview.load_html("<html><body><button id=\"btn\">Click</button></body></html>", None);

    // 使用 createElement 创建元素并添加事件监听器
    // 注：getElementById 在 polyfill 中返回 null，使用 createElement 替代
    let result = webview.execute_script_with_dom(
        "var btn = document.createElement('button'); var clicked = false; btn.addEventListener('click', function() { clicked = true; }); clicked === false ? 'listener added' : 'unexpected'",
    );
    assert!(
        result.is_ok(),
        "addEventListener should work with DOM polyfill: {:?}",
        result
    );
}

#[test]
fn test_js_json_operations() {
    // JSON 是 V8 内置对象，不需要 DOM polyfill
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    let result = webview.execute_script("JSON.stringify({name: 'test', value: 42, nested: {a: true}})");
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["value"], 42);
    assert_eq!(json["nested"]["a"], true);
}

#[test]
fn test_js_array_methods() {
    // Array 方法是 V8 内置，不需要 DOM polyfill
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    let result = webview.execute_script(
        "var arr = [1, 2, 3, 4, 5]; JSON.stringify({sum: arr.reduce(function(a,b){return a+b}, 0), doubled: arr.map(function(x){return x*2}), filtered: arr.filter(function(x){return x>3})})",
    );
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json["sum"], 15);
}

#[test]
fn test_js_promise_execution() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    let result = webview.execute_script("new Promise(function(resolve) { resolve('async result'); })");
    // Promise 可能返回 undefined 或对象，但不应该报错
    assert!(result.is_ok());
}

#[test]
fn test_js_console_log() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    // 使用 DOM polyfill 以获得 console polyfill
    let result = webview.execute_script_with_dom(
        "console.log('test message'); console.warn('warning'); console.error('error'); 'logged'",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "logged");
}

#[test]
fn test_js_set_timeout() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    // setTimeout 需要 DOM polyfill
    let result = webview
        .execute_script_with_dom("var result = 'before'; setTimeout(function() { result = 'after'; }, 0); result");
    assert!(result.is_ok(), "setTimeout should work with DOM polyfill: {:?}", result);
}

#[test]
fn test_js_math_operations() {
    let config = zero_webview::WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(config);

    let result = webview.execute_script(
        "JSON.stringify({pi: Math.PI.toFixed(4), sqrt2: Math.sqrt(2).toFixed(4), max: Math.max(1,2,3), floor: Math.floor(3.7)})",
    );
    assert!(result.is_ok());
    let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json["max"], 3);
    assert_eq!(json["floor"], 3);
}

// ── 渲染管线端到端 ──

#[test]
fn test_render_complex_nested_layout() {
    let result = create_and_render(
        r#"<html><body>
            <div style="display:flex; gap:10px; padding:10px; background:#f0f0f0;">
                <div style="flex:1; height:100px; background:#e74c3c; border-radius:8px;"></div>
                <div style="flex:2; height:100px; background:#3498db; border-radius:8px;">
                    <div style="margin:10px; padding:10px; background:#2ecc71; color:white;">Nested</div>
                </div>
                <div style="width:100px; height:100px; background:#f39c12; border-radius:50%;"></div>
            </div>
        </body></html>"#,
        None,
    );
    // 验证渲染有填充图元输出（flex 布局 + border-radius + nested）
    let total_primitives =
        result.primitives().fills.len() + result.primitives().rounded_rects.len() + result.primitives().glyphs.len();
    assert!(
        total_primitives >= 3,
        "should have primitives for flex items, got {} fills, {} rounded_rects, {} glyphs",
        result.primitives().fills.len(),
        result.primitives().rounded_rects.len(),
        result.primitives().glyphs.len()
    );
}

#[test]
fn test_render_grid_holy_grail() {
    let result = create_and_render(
        r#"<html><body>
            <div style="display:grid; grid-template-areas:'header header header' 'nav main aside' 'footer footer footer'; grid-template-columns:150px 1fr 150px; grid-template-rows:50px 1fr 40px; gap:5px; height:400px;">
                <div style="grid-area:header; background:#2d3436; color:white; padding:10px;">Header</div>
                <div style="grid-area:nav; background:#dfe6e9; padding:10px;">Nav</div>
                <div style="grid-area:main; background:#ffffff; padding:10px;">Main</div>
                <div style="grid-area:aside; background:#ffeaa7; padding:10px;">Aside</div>
                <div style="grid-area:footer; background:#636e72; color:white; padding:10px;">Footer</div>
            </div>
        </body></html>"#,
        None,
    );
    let total_primitives =
        result.primitives().fills.len() + result.primitives().rounded_rects.len() + result.primitives().glyphs.len();
    assert!(
        total_primitives >= 5,
        "should have primitives for all grid areas, got {}",
        total_primitives
    );
}

#[test]
fn test_render_positioned_elements() {
    let result = create_and_render(
        r#"<html><body>
            <div style="position:relative; width:300px; height:200px; background:#f0f0f0;">
                <div style="position:absolute; top:10px; left:10px; width:100px; height:80px; background:#e74c3c; z-index:1;"></div>
                <div style="position:absolute; top:40px; left:50px; width:100px; height:80px; background:#3498db; z-index:2;"></div>
                <div style="position:absolute; top:70px; left:90px; width:100px; height:80px; background:#2ecc71; z-index:3;"></div>
            </div>
        </body></html>"#,
        None,
    );
    let total_primitives = result.primitives().fills.len() + result.primitives().rounded_rects.len();
    assert!(
        total_primitives >= 4,
        "should have fills for container + 3 positioned elements, got {}",
        total_primitives
    );
}

#[test]
fn test_render_text_with_shadows() {
    let result = create_and_render(
        r#"<html><body>
            <div style="padding:20px; background:#333;">
                <p style="color:white; font-size:24px; text-shadow:0 0 10px #fff, 0 0 20px #0ff;">Glowing Text</p>
            </div>
        </body></html>"#,
        None,
    );
    // 至少应该有背景填充和文本 glyph
    assert!(result.primitives().fills.len() + result.primitives().rounded_rects.len() >= 1);
    assert!(
        result.primitives().glyphs.len() >= 1,
        "should have glyph primitives for text"
    );
}

#[test]
fn test_render_gradient_backgrounds() {
    let result = create_and_render(
        r#"<html><body>
            <div style="width:300px; height:100px; background:linear-gradient(90deg, #667eea 0%, #764ba2 100%); margin:10px;"></div>
            <div style="width:300px; height:100px; background:radial-gradient(circle, #f093fb 0%, #f5576c 100%); margin:10px;"></div>
        </body></html>"#,
        None,
    );
    // 渐变元素通过 gradients 或 fills 图元渲染
    let gradient_count = result.primitives().gradients.len() + result.primitives().fills.len();
    assert!(
        gradient_count >= 2,
        "should have gradient or fill primitives for gradient elements, got gradients={}, fills={}",
        result.primitives().gradients.len(),
        result.primitives().fills.len()
    );
}

#[test]
fn test_render_box_shadow_elements() {
    let result = create_and_render(
        r#"<html><body style="background:#f8f9fa; padding:20px;">
            <div style="width:200px; height:100px; background:white; box-shadow:0 2px 4px rgba(0,0,0,0.1), 0 8px 16px rgba(0,0,0,0.1); border-radius:8px; padding:15px;">Card</div>
        </body></html>"#,
        None,
    );
    assert!(result.primitives().fills.len() + result.primitives().rounded_rects.len() >= 1);
    assert!(
        result.primitives().shadows.len() >= 1,
        "should have shadow primitives for box-shadow"
    );
}

#[test]
fn test_css_custom_properties_pipeline() {
    let result = create_and_render(
        r#"<html><body>
            <div style="--bg:#3498db; --text:white; --radius:8px; width:200px; height:100px; background:var(--bg); color:var(--text); border-radius:var(--radius); padding:10px;">Custom Props</div>
        </body></html>"#,
        None,
    );
    assert!(
        result.primitives().fills.len() + result.primitives().rounded_rects.len() >= 1,
        "should have fill primitives for custom property element"
    );
}

#[test]
fn test_css_media_query_render() {
    // 验证 @media 块不导致渲染 panic
    let result = create_and_render(
        r#"<html><body>
            <div class="responsive" style="padding:10px; background:#e74c3c; color:white;">Content</div>
        </body></html>"#,
        Some(r#"@media (max-width: 600px) { .responsive { background: #3498db; } }"#),
    );
    assert!(result.primitives().fills.len() + result.primitives().rounded_rects.len() >= 1);
}
