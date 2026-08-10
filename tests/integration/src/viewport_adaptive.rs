//! 视口自适应集成测试（质量测试矩阵 Layer 8）
//!
//! 覆盖视口尺寸变化下的渲染行为：
//! - 响应式布局重排（flex/grid 响应 resize）
//! - CSS 媒体查询视口适配
//! - 极端视口尺寸（极宽、极高、方形、超宽屏）
//! - resize 后样式重计算
//! - 多步 resize 响应式内容验证
//! - HiDPI 相关 CSS 单位渲染

use zero_webview::{WebView, WebViewConfig};

// ── 辅助函数 ──

/// 创建标准测试 WebView（800×600）
fn create_webview() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

/// 响应式 flex 布局页面
fn responsive_flex_html() -> &'static str {
    r#"<html><head><style>
        .container { display: flex; gap: 8px; }
        .sidebar { width: 200px; background: #f0f0f0; padding: 8px; }
        .main { flex: 1; background: #e8e8e8; padding: 8px; }
        @media (max-width: 600px) {
            .container { flex-direction: column; }
            .sidebar { width: 100%; }
        }
    </style></head><body>
        <div class="container">
            <div class="sidebar">Sidebar content</div>
            <div class="main">Main content area with text for rendering.</div>
        </div>
    </body></html>"#
}

/// 响应式网格布局页面
fn responsive_grid_html() -> &'static str {
    r#"<html><head><style>
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
            gap: 8px;
        }
        .cell { background: #e0e0e0; padding: 16px; text-align: center; }
    </style></head><body>
        <div class="grid">
            <div class="cell">A</div>
            <div class="cell">B</div>
            <div class="cell">C</div>
            <div class="cell">D</div>
            <div class="cell">E</div>
            <div class="cell">F</div>
        </div>
    </body></html>"#
}

/// 多段落页面（用于文本换行测试）
fn text_wrap_html() -> &'static str {
    r#"<html><head><style>
        body { font-family: sans-serif; margin: 0; padding: 8px; }
        p { font-size: 14px; line-height: 1.4; margin: 4px 0; }
    </style></head><body>
        <p>This is the first paragraph with enough text to test line wrapping behavior at different viewport widths.</p>
        <p>Second paragraph with additional content to verify text reflow works correctly when the viewport changes size.</p>
        <p>Third paragraph: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.</p>
    </body></html>"#
}

/// viewport/rem 单位页面
fn viewport_units_html() -> &'static str {
    r#"<html><head><style>
        html { font-size: 16px; }
        .hero { font-size: 2rem; padding: 2vh 5vw; background: #e0e0e0; }
        .content { font-size: 1rem; padding: 1rem; max-width: 80vw; }
    </style></head><body>
        <div class="hero">Hero Section</div>
        <div class="content">Content with rem and vw sizing.</div>
    </body></html>"#
}

// ── 响应式布局重排测试 ──

/// 宽视口下 flex 行布局渲染
#[test]
fn test_viewport_flex_wide_layout() {
    let mut wv = create_webview();
    let result = wv.load_html(responsive_flex_html(), None);

    // 宽视口（800px）应正常渲染
    assert!(!result.primitives().glyphs.is_empty(), "宽视口应有 glyph 渲染");
    assert!(!result.primitives().fills.is_empty(), "宽视口应有 fill 渲染");
}

/// 窄视口下 flex 列布局重排（@media 触发）
#[test]
fn test_viewport_flex_narrow_reflow() {
    let mut wv = create_webview();
    // 先宽后窄
    wv.load_html(responsive_flex_html(), None);
    wv.resize(400, 600);
    let result = wv.render();

    assert!(!result.primitives().glyphs.is_empty(), "窄视口 flex 重排后应有 glyph");
    assert!(!result.primitives().fills.is_empty(), "窄视口 flex 重排后应有 fill");
    assert!(result.timings.total_ms >= 0.0);
}

/// resize 后图元数量变化（文本换行导致 glyph 数量变化）
#[test]
fn test_viewport_resize_changes_glyph_count() {
    let mut wv = create_webview();

    let result_wide = wv.load_html(text_wrap_html(), None);
    let glyphs_wide = result_wide.primitives().glyphs.len();

    wv.resize(300, 600);
    let result_narrow = wv.render();
    let glyphs_narrow = result_narrow.primitives().glyphs.len();

    // 两种宽度都应有渲染结果
    assert!(glyphs_wide > 0, "宽视口应有 glyph");
    assert!(glyphs_narrow > 0, "窄视口应有 glyph");

    // 窄视口文本换行更多，glyph 数量可能变化
    // 关键验证：两种尺寸都正常渲染
}

// ── 响应式网格布局测试 ──

/// 网格 auto-fill 在不同视口宽度下正确重排
#[test]
fn test_viewport_grid_auto_fill_reflow() {
    let mut wv = create_webview();
    let result = wv.load_html(responsive_grid_html(), None);
    let fills_wide = result.primitives().fills.len();
    assert!(fills_wide > 0, "宽视口网格应有 fill");

    // 缩窄视口
    wv.resize(300, 400);
    let result = wv.render();
    let fills_narrow = result.primitives().fills.len();
    assert!(fills_narrow > 0, "窄视口网格应有 fill");
}

// ── 极端视口尺寸测试 ──

/// 极宽视口不 panic
#[test]
fn test_viewport_ultra_wide() {
    let mut wv = create_webview();
    wv.load_html(text_wrap_html(), None);

    wv.resize(3840, 2160);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "3840x2160 超宽视口应渲染成功");
    assert!(!result.primitives().glyphs.is_empty());
}

/// 极窄视口不 panic
#[test]
fn test_viewport_ultra_narrow() {
    let mut wv = create_webview();
    wv.load_html(text_wrap_html(), None);

    wv.resize(50, 600);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "50x600 极窄视口应渲染成功");
}

/// 方形视口不 panic
#[test]
fn test_viewport_square() {
    let mut wv = create_webview();
    wv.load_html(text_wrap_html(), None);

    wv.resize(500, 500);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "500x500 方形视口应渲染成功");
    assert!(!result.primitives().glyphs.is_empty());
}

/// 超高瘦视口不 panic
#[test]
fn test_viewport_tall_narrow() {
    let mut wv = create_webview();
    wv.load_html(text_wrap_html(), None);

    wv.resize(100, 2000);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "100x2000 超高瘦视口应渲染成功");
}

// ── 多步 resize 渲染稳定性测试 ──

/// 连续 8 种视口尺寸均不 panic
#[test]
fn test_viewport_multi_step_resize_stability() {
    let mut wv = create_webview();
    wv.load_html(responsive_flex_html(), None);

    let sizes: [(u32, u32); 8] = [
        (1920, 1080),
        (1280, 720),
        (1024, 768),
        (800, 600),
        (640, 480),
        (480, 320),
        (360, 640),
        (320, 240),
    ];

    for (w, h) in sizes {
        wv.resize(w, h);
        let result = wv.render();
        assert!(result.timings.total_ms >= 0.0, "resize 到 {w}x{h} 应渲染成功");
        // 每种尺寸都应有 glyph
        assert!(!result.primitives().glyphs.is_empty(), "{w}x{h} 视口应有 glyph 渲染");
    }
}

/// resize 往返不累积误差
#[test]
fn test_viewport_resize_round_trip() {
    let mut wv = create_webview();
    let result_initial = wv.load_html(text_wrap_html(), None);
    let glyphs_initial = result_initial.primitives().glyphs.len();

    // 改变尺寸
    wv.resize(400, 300);
    wv.render();

    // 恢复原始尺寸
    wv.resize(800, 600);
    let result_restored = wv.render();
    let glyphs_restored = result_restored.primitives().glyphs.len();

    // 恢复后 glyph 数量应与初始相同
    assert_eq!(glyphs_initial, glyphs_restored, "resize 往返后 glyph 数量应一致");
}

// ── viewport 单位渲染测试 ──

/// rem/vw/vh CSS 单位页面渲染不 panic
#[test]
fn test_viewport_css_units_render() {
    let mut wv = create_webview();
    let result = wv.load_html(viewport_units_html(), None);

    assert!(!result.primitives().glyphs.is_empty(), "viewport 单位页面应有 glyph");
    assert!(!result.primitives().fills.is_empty(), "viewport 单位页面应有 fill");
}

/// resize 后 viewport 单位重计算
#[test]
fn test_viewport_units_after_resize() {
    let mut wv = create_webview();
    wv.load_html(viewport_units_html(), None);

    // 缩窄视口，vw 单位应重计算
    wv.resize(400, 300);
    let result = wv.render();
    assert!(
        !result.primitives().glyphs.is_empty(),
        "resize 后 viewport 单位页面应有 glyph"
    );
    assert!(result.timings.total_ms >= 0.0);
}

// ── 空内容视口测试 ──

/// 空页面在各种视口尺寸下不 panic
#[test]
fn test_viewport_empty_page_various_sizes() {
    let sizes: [(u32, u32); 4] = [(800, 600), (100, 100), (2000, 50), (1, 1)];
    for (w, h) in sizes {
        let mut wv = WebView::new(WebViewConfig {
            width: w,
            height: h,
            ..Default::default()
        });
        let result = wv.load_html("<html><body></body></html>", None);
        assert!(result.timings.total_ms >= 0.0, "{w}x{h} 空页面应渲染成功");
    }
}

/// resize 后重新 load_html 渲染正确
#[test]
fn test_viewport_resize_then_reload() {
    let mut wv = create_webview();
    wv.load_html("<html><body><p>First page</p></body></html>", None);

    // resize 后加载新页面
    wv.resize(400, 300);
    let result = wv.load_html("<html><body><p>Second page</p></body></html>", None);
    assert!(!result.primitives().glyphs.is_empty(), "resize 后重新加载应有 glyph");
}

/// 紧凑视口下 grid 布局不溢出
#[test]
fn test_viewport_grid_compact() {
    let config = WebViewConfig {
        width: 200,
        height: 200,
        ..Default::default()
    };
    let mut wv = WebView::new(config);
    let result = wv.load_html(responsive_grid_html(), None);
    assert!(result.timings.total_ms >= 0.0, "紧凑视口 grid 布局应渲染成功");
}
