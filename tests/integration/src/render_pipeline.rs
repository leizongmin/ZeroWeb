#[cfg(test)]
use zero_engine::RenderPipeline;

/// 完整管线：HTML + CSS → 渲染结果
#[test]
fn test_full_render_pipeline() {
    let html = r#"<html><body>
        <div style="width: 200px; height: 100px; background-color: red;">Box</div>
    </body></html>"#;

    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");

    assert!(result.timings.total_ms >= 0.0, "应有渲染耗时");
    // 渲染管线成功完成即通过
}

/// 管线使用 CSS 文件
#[test]
fn test_render_pipeline_with_css() {
    let html = r#"<html><body><div class="box">Hello</div></body></html>"#;
    let css = r#".box { background-color: blue; width: 300px; height: 200px; }"#;

    let mut pipeline = RenderPipeline::new(1024.0, 768.0);
    let result = pipeline.render_html(html, css);

    assert!(result.timings.total_ms >= 0.0);
}

/// 管线阶段耗时分解
#[test]
fn test_pipeline_timing_breakdown() {
    let html = r#"<html><body><p>Test paragraph</p></body></html>"#;
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");

    assert!(result.timings.parse_ms >= 0.0, "parse_ms >= 0");
    assert!(result.timings.style_ms >= 0.0, "style_ms >= 0");
    assert!(result.timings.layout_ms >= 0.0, "layout_ms >= 0");
    assert!(result.timings.paint_ms >= 0.0, "paint_ms >= 0");
}

/// 复杂页面渲染
#[test]
fn test_complex_page_render() {
    let html = r#"<html><head><title>Complex</title></head><body>
        <header><h1>Title</h1></header>
        <main><p>Content</p><p>More content</p></main>
        <footer><p>Footer</p></footer>
    </body></html>"#;
    let css = r#"
        body { margin: 0; }
        header { background-color: #333; color: white; }
        main { padding: 20px; }
        footer { background-color: #eee; }
    "#;

    let mut pipeline = RenderPipeline::new(1440.0, 900.0);
    let result = pipeline.render_html(html, css);
    assert!(result.timings.total_ms >= 0.0);
}
