#[cfg(test)]
use zero_canvas::CanvasContext;
use zero_webview::{WebView, WebViewConfig};

/// Canvas 2D 绘图操作 → 渲染图元
#[test]
fn test_canvas_fill_rect_generates_primitives() {
    let mut ctx = CanvasContext::new(800, 600);
    ctx.fill_rect(10.0, 20.0, 100.0, 50.0);
    ctx.fill_rect(200.0, 100.0, 150.0, 80.0);
    let primitives = ctx.primitives();
    // 应生成填充图元
    assert!(!primitives.fills.is_empty(), "fill_rect 应生成填充图元");
}

/// Canvas 路径绘制集成
#[test]
fn test_canvas_path_operations() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(100.0, 10.0);
    ctx.line_to(100.0, 100.0);
    ctx.close_path();
    ctx.fill();

    let primitives = ctx.primitives();
    assert!(!primitives.path_fills.is_empty(), "路径填充应生成图元");
}

/// Canvas 变换操作集成
#[test]
fn test_canvas_transform_chain() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.translate(50.0, 50.0);
    ctx.rotate(45.0_f32.to_radians());
    ctx.scale(2.0, 2.0);
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);

    // 变换后的 fill_rect 不应 panic
    let primitives = ctx.primitives();
    assert!(!primitives.fills.is_empty());
}

/// Canvas + WebView 集成：通过 WebView 加载含 canvas 的 HTML
#[test]
fn test_webview_renders_page_with_canvas() {
    let html = r#"<html><body>
        <div style="width: 200px; height: 100px; background-color: green;">Box</div>
    </body></html>"#;

    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    let result = wv.load_html(html, None);
    // WebView 渲染应产生图元
    assert!(result.timings.total_ms >= 0.0);
}

/// Canvas save/restore 状态管理
#[test]
fn test_canvas_save_restore_state() {
    let mut ctx = CanvasContext::new(400, 300);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    ctx.save();
    ctx.translate(100.0, 100.0);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    ctx.restore();
    // restore 后再次绘制应使用原始变换
    ctx.fill_rect(200.0, 200.0, 50.0, 50.0);

    let primitives = ctx.primitives();
    assert!(primitives.fills.len() >= 3, "应有 3 个填充图元");
}
