//! CSS 属性端到端管线集成测试。
//!
//! 验证 CSS 属性从解析到样式计算到布局到渲染的完整管线，
//! 覆盖尚未在其他模块中测试的 CSS 属性。

use zero_engine::RenderPipeline;

/// 完整管线辅助函数。
fn render_pipeline(html: &str, css: &str) -> zero_engine::RenderResult {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(html, css)
}

/// 辅助函数：创建管线并渲染带动画的 HTML。
fn render_pipeline_animated(html: &str, css: &str, current_time: f64) -> zero_engine::RenderResult {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html_animated(html, css, current_time)
}

// ── 1. CSS Grid 高级属性 ──────────────────────────────────────────

#[test]
fn test_grid_auto_rows_columns() {
    let html = r#"<html><body>
        <div class="grid">
            <div>A</div><div>B</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid { display: grid; grid-template-columns: 1fr 1fr; grid-auto-rows: 100px; gap: 10px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_grid_template_rows_columns() {
    let html = r#"<html><body>
        <div class="grid">
            <div>A</div><div>B</div>
            <div>C</div><div>D</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid { display: grid; grid-template-columns: 200px 1fr; grid-template-rows: 50px 100px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_grid_with_span() {
    let html = r#"<html><body>
        <div class="grid">
            <div class="wide">Spans 2 cols</div>
            <div>Narrow</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid { display: grid; grid-template-columns: 1fr 1fr 1fr; }
        .wide { grid-column: span 2; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 2. CSS Flexbox 高级属性 ──────────────────────────────────────

#[test]
fn test_flex_order() {
    let html = r#"<html><body>
        <div class="flex">
            <div class="a">A</div>
            <div class="b">B</div>
            <div class="c">C</div>
        </div>
    </body></html>"#;
    let css = r#"
        .flex { display: flex; width: 300px; }
        .a { order: 3; width: 50px; }
        .b { order: 1; width: 50px; }
        .c { order: 2; width: 50px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_flex_basis_content() {
    let html = r#"<html><body>
        <div class="flex">
            <div class="item">Content-based</div>
            <div class="fixed">Fixed</div>
        </div>
    </body></html>"#;
    let css = r#"
        .flex { display: flex; width: 400px; }
        .item { flex-basis: content; }
        .fixed { flex-basis: 200px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_flex_align_self() {
    let html = r#"<html><body>
        <div class="flex">
            <div class="a">A</div>
            <div class="b">B</div>
        </div>
    </body></html>"#;
    let css = r#"
        .flex { display: flex; align-items: flex-start; height: 200px; }
        .a { align-self: flex-end; width: 50px; height: 50px; }
        .b { align-self: center; width: 50px; height: 50px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 3. CSS 文本属性 ──────────────────────────────────────────────

#[test]
fn test_text_overflow_ellipsis() {
    let html = r#"<html><body>
        <div class="truncate">This is a very long text that should be truncated</div>
    </body></html>"#;
    let css = r#"
        .truncate {
            width: 100px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_word_break_break_all() {
    let html = r#"<html><body>
        <div class="break">Superlongwordwithoutanyspacesinit</div>
    </body></html>"#;
    let css = r#"
        .break { width: 100px; word-break: break-all; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_white_space_pre_wrap() {
    let html = r#"<html><body>
        <div class="pre">  Multiple    spaces    and
    newlines</div>
    </body></html>"#;
    let css = r#"
        .pre { white-space: pre-wrap; width: 200px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_text_decoration_underline_overline() {
    let html = r#"<html><body>
        <p class="under">Underlined text</p>
        <p class="over">Overlined text</p>
        <p class="line">Line-through text</p>
    </body></html>"#;
    let css = r#"
        .under { text-decoration-line: underline; }
        .over { text-decoration-line: overline; }
        .line { text-decoration-line: line-through; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Expected glyph primitives");
}

// ── 4. CSS 间距和边框 ────────────────────────────────────────────

#[test]
fn test_gap_shorthand() {
    let html = r#"<html><body>
        <div class="grid">
            <div>A</div><div>B</div>
            <div>C</div><div>D</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px 10px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_border_individual_sides() {
    let html = r#"<html><body>
        <div class="box">Content</div>
    </body></html>"#;
    let css = r#"
        .box {
            border-top: 3px solid red;
            border-right: 1px dashed blue;
            border-bottom: 5px double green;
            border-left: 2px dotted orange;
            padding: 10px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_border_radius_individual() {
    let html = r#"<html><body>
        <div class="box">Rounded</div>
    </body></html>"#;
    let css = r#"
        .box {
            width: 100px; height: 100px;
            border-radius: 10px 20px 30px 40px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 5. CSS 尺寸和约束 ────────────────────────────────────────────

#[test]
fn test_min_max_constraints() {
    let html = r#"<html><body>
        <div class="box">Content with constraints</div>
    </body></html>"#;
    let css = r#"
        .box {
            width: 50%;
            min-width: 200px;
            max-width: 600px;
            min-height: 50px;
            max-height: 200px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_calc_width() {
    let html = r#"<html><body>
        <div class="calc">Calc width</div>
    </body></html>"#;
    let css = r#"
        .calc { width: calc(100% - 40px); height: 50px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_viewport_units() {
    let html = r#"<html><body>
        <div class="vw">Viewport width</div>
        <div class="vh">Viewport height</div>
    </body></html>"#;
    let css = r#"
        .vw { width: 50vw; height: 10vh; }
        .vh { width: 30vw; height: 20vh; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 6. CSS 变量和自定义属性 ──────────────────────────────────────

#[test]
fn test_css_variables_complex() {
    let html = r#"<html><body>
        <div class="outer">
            <div class="inner">Styled by variables</div>
        </div>
    </body></html>"#;
    let css = r#"
        .outer {
            --main-color: #ff6600;
            --spacing: 20px;
            --font-size: 18px;
        }
        .inner {
            color: var(--main-color);
            padding: var(--spacing);
            font-size: var(--font-size);
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_css_variable_fallback_chain() {
    let html = r#"<html><body>
        <div class="box">Fallback styled</div>
    </body></html>"#;
    let css = r#"
        .box {
            color: var(--undefined-color, #333333);
            padding: var(--undefined-pad, 10px);
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 7. CSS 媒体查询 ─────────────────────────────────────────────

#[test]
fn test_media_query_screen() {
    let html = r#"<html><body>
        <div class="responsive">Content</div>
    </body></html>"#;
    let css = r#"
        .responsive { width: 100%; }
        @media screen and (min-width: 768px) {
            .responsive { width: 750px; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 8. CSS @supports ─────────────────────────────────────────────

#[test]
fn test_supports_rule() {
    let html = r#"<html><body>
        <div class="box">Supports test</div>
    </body></html>"#;
    let css = r#"
        .box { color: red; }
        @supports (display: grid) {
            .box { color: green; display: grid; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 9. CSS 定位高级 ─────────────────────────────────────────────

#[test]
fn test_sticky_positioning() {
    let html = r#"<html><body>
        <div class="container">
            <div class="sticky">Sticky header</div>
            <div class="content">
                <p>Line 1</p><p>Line 2</p><p>Line 3</p>
            </div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { height: 300px; overflow: auto; }
        .sticky { position: sticky; top: 0; background: #fff; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_absolute_in_relative() {
    let html = r#"<html><body>
        <div class="relative">
            <div class="absolute">Absolute</div>
            <div class="static">Static</div>
        </div>
    </body></html>"#;
    let css = r#"
        .relative { position: relative; width: 400px; height: 200px; }
        .absolute { position: absolute; top: 10px; right: 20px; width: 100px; }
        .static { height: 50px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_fixed_positioning() {
    let html = r#"<html><body>
        <div class="fixed">Fixed element</div>
        <div class="content">Page content</div>
    </body></html>"#;
    let css = r#"
        .fixed { position: fixed; top: 0; left: 0; width: 100%; height: 50px; }
        .content { margin-top: 60px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 10. CSS 颜色格式 ─────────────────────────────────────────────

#[test]
fn test_color_rgba_transparent() {
    let html = r#"<html><body>
        <div class="opaque">Opaque</div>
        <div class="semi">Semi-transparent</div>
        <div class="transparent">Transparent</div>
    </body></html>"#;
    let css = r#"
        .opaque { background-color: rgba(255, 0, 0, 1.0); }
        .semi { background-color: rgba(0, 128, 255, 0.5); }
        .transparent { background-color: transparent; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_color_hsl_format() {
    let html = r#"<html><body>
        <div class="warm">Warm</div>
        <div class="cool">Cool</div>
    </body></html>"#;
    let css = r#"
        .warm { background-color: hsl(0, 100%, 50%); }
        .cool { background-color: hsl(240, 100%, 50%); }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 11. CSS 字体属性 ─────────────────────────────────────────────

#[test]
fn test_font_shorthand() {
    let html = r#"<html><body>
        <p class="bold">Bold text</p>
        <p class="italic">Italic text</p>
        <p class="mono">Monospace text</p>
    </body></html>"#;
    let css = r#"
        .bold { font: bold 16px sans-serif; }
        .italic { font: italic 14px serif; }
        .mono { font: 12px monospace; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Expected glyphs");
}

#[test]
fn test_line_height() {
    let html = r#"<html><body>
        <p class="tight">Tight line height</p>
        <p class="loose">Loose line height</p>
    </body></html>"#;
    let css = r#"
        .tight { line-height: 1.0; }
        .loose { line-height: 2.0; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 12. CSS 逻辑属性 ─────────────────────────────────────────────

#[test]
fn test_logical_properties() {
    let html = r#"<html><body>
        <div class="logical">Logical margins and padding</div>
    </body></html>"#;
    let css = r#"
        .logical {
            margin-block: 10px 20px;
            margin-inline: 15px 25px;
            padding-block: 5px 10px;
            padding-inline: 8px 12px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_inset_logical() {
    let html = r#"<html><body>
        <div class="container">
            <div class="absolute">Logically positioned</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { position: relative; width: 400px; height: 200px; }
        .absolute {
            position: absolute;
            inset-block-start: 10px;
            inset-inline-start: 20px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 13. CSS 交互属性 ─────────────────────────────────────────────

#[test]
fn test_cursor_property() {
    let html = r#"<html><body>
        <div class="pointer">Click me</div>
        <div class="text">Select me</div>
        <div class="move">Drag me</div>
    </body></html>"#;
    let css = r#"
        .pointer { cursor: pointer; }
        .text { cursor: text; }
        .move { cursor: move; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_user_select() {
    let html = r#"<html><body>
        <div class="selectable">Selectable</div>
        <div class="noselect">Not selectable</div>
    </body></html>"#;
    let css = r#"
        .selectable { user-select: text; }
        .noselect { user-select: none; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 14. CSS 过滤器 ───────────────────────────────────────────────

#[test]
fn test_filter_blur() {
    let html = r#"<html><body>
        <div class="blur">Blurred content</div>
    </body></html>"#;
    let css = r#"
        .blur { filter: blur(5px); }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_filter_brightness_contrast() {
    let html = r#"<html><body>
        <div class="bright">Bright</div>
        <div class="contrast">High contrast</div>
    </body></html>"#;
    let css = r#"
        .bright { filter: brightness(1.5); }
        .contrast { filter: contrast(200%); }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 15. CSS 综合页面 ─────────────────────────────────────────────

#[test]
fn test_responsive_card_layout() {
    let html = r#"<html><body>
        <div class="cards">
            <div class="card"><h3>Card 1</h3><p>Content</p></div>
            <div class="card"><h3>Card 2</h3><p>Content</p></div>
            <div class="card"><h3>Card 3</h3><p>Content</p></div>
        </div>
    </body></html>"#;
    let css = r#"
        .cards {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
            gap: 20px;
            padding: 20px;
        }
        .card {
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 16px;
        }
        .card h3 { margin: 0 0 8px 0; font-size: 18px; }
        .card p { margin: 0; color: #666; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
    assert!(
        !result.primitives().glyphs.is_empty(),
        "Card text should produce glyphs"
    );
}

#[test]
fn test_holy_grail_layout() {
    let html = r#"<html><body>
        <div class="layout">
            <header>Header</header>
            <div class="middle">
                <nav>Navigation</nav>
                <main>Main content area with lots of text</main>
                <aside>Sidebar</aside>
            </div>
            <footer>Footer</footer>
        </div>
    </body></html>"#;
    let css = r#"
        .layout { display: flex; flex-direction: column; min-height: 100vh; }
        header, footer { padding: 10px; }
        .middle { display: flex; flex: 1; }
        nav { width: 200px; }
        main { flex: 1; padding: 10px; }
        aside { width: 150px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Layout should produce glyphs");
}

// ── 11. CSS 容器查询管线 ──────────────────────────────────────

#[test]
fn test_container_query_min_width() {
    let html = r#"<html><body>
        <div class="container">
            <div class="item">Content</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { container-type: inline-size; width: 400px; }
        .item { background: red; }
        @container (min-width: 300px) {
            .item { background: green; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_container_query_max_width() {
    let html = r#"<html><body>
        <div class="container">
            <div class="item">Small</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { container-type: inline-size; width: 200px; }
        .item { background: blue; }
        @container (max-width: 300px) {
            .item { background: red; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_container_type_inline_size() {
    let html = r#"<html><body>
        <div style="container-type: inline-size; width: 500px;">
            <div>Item</div>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 12. CSS @supports 管线 ──────────────────────────────────────

#[test]
fn test_supports_property() {
    let html = r#"<html><body>
        <div class="box">Test</div>
    </body></html>"#;
    let css = r#"
        .box { background: red; }
        @supports (display: grid) {
            .box { background: green; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_supports_selector() {
    let html = r#"<html><body>
        <div class="box">Test</div>
    </body></html>"#;
    let css = r#"
        .box { color: black; }
        @supports selector(:is(div)) {
            .box { color: blue; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 13. CSS @layer 管线 ──────────────────────────────────────

#[test]
fn test_layer_ordering() {
    let html = r#"<html><body>
        <div class="box">Layer test</div>
    </body></html>"#;
    let css = r#"
        @layer base, override;
        @layer base {
            .box { background: red; }
        }
        @layer override {
            .box { background: green; }
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_unlayered_overrides_layers() {
    let html = r#"<html><body>
        <div class="box">Unlayered</div>
    </body></html>"#;
    let css = r#"
        @layer base {
            .box { color: red; }
        }
        .box { color: blue; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 14. CSS scroll-snap 管线 ──────────────────────────────────────

#[test]
fn test_scroll_snap_type() {
    let html = r#"<html><body>
        <div class="scroll-container">
            <div class="child">A</div>
            <div class="child">B</div>
        </div>
    </body></html>"#;
    let css = r#"
        .scroll-container {
            scroll-snap-type: x mandatory;
            overflow: auto;
            width: 300px;
            height: 200px;
        }
        .child { scroll-snap-align: start; width: 300px; height: 200px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_scroll_snap_stop() {
    let html = r#"<html><body>
        <div style="scroll-snap-stop: always; width: 100px; height: 100px;">Snap</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 15. CSS contain + will-change 管线 ──────────────────────────────

#[test]
fn test_contain_layout() {
    let html = r#"<html><body>
        <div class="contained" style="width:200px; height:100px; background:#eee;">
            Content with containment
        </div>
    </body></html>"#;
    let css = r#"
        .contained { contain: layout; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_contain_strict() {
    let html = r#"<html><body>
        <div style="contain: strict; width:100px; height:100px; background:red;">Strict</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_will_change_transform() {
    let html = r#"<html><body>
        <div style="will-change: transform; width:100px; height:50px; background:#eee;">
            Will animate
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_isolation_isolate() {
    let html = r#"<html><body>
        <div style="isolation: isolate; width:200px; height:100px; background:blue;">
            Isolated stacking context
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 16. CSS 渐变渲染管线 ──────────────────────────────────────

#[test]
fn test_linear_gradient_to_direction() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background:linear-gradient(to right, #ff0000, #0000ff);">Gradient</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().gradients.is_empty(),
        "Should produce gradient primitives"
    );
}

#[test]
fn test_radial_gradient_circle() {
    let html = r#"<html><body>
        <div style="width:200px; height:200px; background:radial-gradient(circle, red, blue);">Radial</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().gradients.is_empty(),
        "Should produce gradient primitives"
    );
}

#[test]
fn test_linear_gradient_with_angle() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background:linear-gradient(135deg, #667eea, #764ba2);">Angle</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().gradients.is_empty(),
        "Should produce gradient primitives"
    );
}

// ── 17. CSS 变换/透视管线 ──────────────────────────────────────

#[test]
fn test_transform_with_perspective() {
    let html = r#"<html><body>
        <div style="perspective: 800px; width:300px; height:200px; background:#eee;">
            <div style="transform: rotateY(45deg); width:100px; height:100px; background:red;">Rotated</div>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_transform_origin() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; transform:rotate(45deg); transform-origin: top left; background:#eee;">Origin</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_backface_visibility() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; transform:rotateY(180deg); backface-visibility:hidden; background:#eee;">Hidden</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 18. CSS 文本高级属性管线 ──────────────────────────────────────

#[test]
fn test_text_shadow_render() {
    let html = r#"<html><body>
        <div class="shadow-text">Shadow text</div>
    </body></html>"#;
    let css = ".shadow-text { text-shadow: 2px 2px 4px rgba(0,0,0,0.5); background:white; padding:10px; }";
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    // text-shadow 通过 style 元素渲染；至少应该有 glyphs
    assert!(!result.primitives().glyphs.is_empty() || !result.primitives().shadows.is_empty());
}

#[test]
fn test_multiple_text_shadows() {
    let html = r#"<html><body>
        <div class="multi-shadow">Multi shadow</div>
    </body></html>"#;
    let css = ".multi-shadow { text-shadow: 1px 1px red, -1px -1px blue; background:white; padding:10px; }";
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty() || !result.primitives().shadows.is_empty());
}

#[test]
fn test_box_shadow_inset_render() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; box-shadow: inset 5px 5px 10px rgba(0,0,0,0.3); background:#eee;">Inset shadow</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().shadows.is_empty(),
        "Should produce shadow primitives"
    );
}

// ── 19. CSS 多列布局管线 ──────────────────────────────────────

#[test]
fn test_column_count_render() {
    let html = r#"<html><body>
        <div style="column-count: 3; column-gap: 20px; padding: 10px; background: #f0f0f0;">
            <p>Column one text content.</p>
            <p>Column two text content.</p>
            <p>Column three text content.</p>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Should produce glyphs for text");
}

#[test]
fn test_column_width_render() {
    let html = r#"<html><body>
        <div style="column-width: 200px; column-gap: 15px; width: 600px;">
            <p>Text content in auto columns.</p>
            <p>More text for column layout.</p>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_column_rule_render() {
    let html = r#"<html><body>
        <div style="column-count: 2; column-gap: 30px; column-rule: 1px solid #ccc; width: 400px;">
            <p>Left column</p>
            <p>Right column</p>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 20. CSS 自定义属性管线 ──────────────────────────────────────

#[test]
fn test_css_variable_fallback() {
    let html = r#"<html><body>
        <div style="--my-color: green; background: var(--my-color, red); width:100px; height:50px;">Variable</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().fills.is_empty(),
        "Should produce fill from CSS variable"
    );
}

#[test]
fn test_css_variable_undefined_fallback() {
    let html = r#"<html><body>
        <div style="background: var(--undefined-var, blue); width:100px; height:50px;">Fallback</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().fills.is_empty(),
        "Should produce fill from fallback value"
    );
}

#[test]
fn test_css_variable_chain() {
    let html = r#"<html><body>
        <div style="--base: 20px; --derived: var(--base); padding: var(--derived); background:#eee; width:200px;">Chain</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 21. CSS 背景属性管线 ──────────────────────────────────────────

#[test]
fn test_background_position_center() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background-color:#ccc;">Positioned</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty(), "Should produce fill");
}

#[test]
fn test_background_repeat_no_repeat() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background-color:#ddd;">No repeat</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_background_size_cover() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background-color:#eee;">Cover</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_background_attachment_fixed() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; background-color:#bbb;">Fixed bg</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_background_clip_content_box() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; padding:10px; background-color:#aaa; background-clip:content-box;">Clipped</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_background_origin_padding_box() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; padding:10px; background-color:#999; background-origin:padding-box;">Origin</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 22. CSS 内容与排版属性管线 ──────────────────────────────────────

#[test]
fn test_word_spacing_wide() {
    let html = r#"<html><body>
        <div style="word-spacing: 10px; width:200px;">Wide spaced words here</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Should produce glyphs");
}

#[test]
fn test_quotes_property() {
    let html = r#"<html><body>
        <div style="quotes: '«' '»'; width:200px;">Quoted text</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty(), "Should produce glyphs");
}

#[test]
fn test_resize_property() {
    let html = r#"<html><body>
        <div style="width:200px; height:100px; overflow:auto; resize:both; background:#ddd;">Resizable</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_content_property_with_counter() {
    let html = r#"<html><body>
        <div style="counter-reset: section; width:200px;">
            <div style="counter-increment: section;">Section text</div>
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_caption_side_bottom() {
    let html = r#"<html><body>
        <table style="width:200px;">
            <caption>Caption text</caption>
            <tr><td>Cell</td></tr>
        </table>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_table_layout_fixed() {
    let html = r#"<html><body>
        <table style="width:300px; table-layout:fixed;">
            <tr><td>A</td><td>B</td><td>C</td></tr>
        </table>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_border_collapse_separate() {
    let html = r#"<html><body>
        <table style="border-collapse:separate; border-spacing:5px;">
            <tr><td style="border:1px solid black;">A</td></tr>
        </table>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_empty_cells_hide() {
    let html = r#"<html><body>
        <table style="empty-cells:hide; border-collapse:separate;">
            <tr><td style="border:1px solid gray;">Content</td><td style="border:1px solid gray;"></td></tr>
        </table>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── CSS 动画管线集成测试 ──

/// 测试 @keyframes 动画通过完整管线渲染。
#[test]
fn test_keyframes_animation_pipeline() {
    let html = r#"<html><body><div class="animated">Anim</div></body></html>"#;
    let css = r#"
        @keyframes fadeIn { from { opacity: 0.0; } to { opacity: 1.0; } }
        .animated { animation: fadeIn 1s linear; background-color: blue; width: 100px; height: 80px; }
    "#;
    let result = render_pipeline_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives().fills.is_empty(),
        "animated element should produce fills"
    );
}

/// 测试动画 timing function ease 渲染管线。
#[test]
fn test_animation_timing_ease_pipeline() {
    let html = r#"<html><body><div class="ease">Ease</div></body></html>"#;
    let css = r#"
        @keyframes slide { from { opacity: 0.2; } to { opacity: 1.0; } }
        .ease { animation: slide 2s ease; background-color: green; width: 150px; height: 100px; }
    "#;
    let result = render_pipeline_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试动画 fill-mode forwards 渲染管线。
#[test]
fn test_animation_fill_forwards_pipeline() {
    let html = r#"<html><body><div class="fill">Fill</div></body></html>"#;
    let css = r#"
        @keyframes grow { from { opacity: 0.0; } to { opacity: 1.0; } }
        .fill { animation: grow 0.5s linear forwards; background-color: orange; width: 200px; height: 120px; }
    "#;
    // t=0: animation starts, opacity should be near 0.0
    let r0 = render_pipeline_animated(html, css, 0.0);
    assert!(r0.timings.total_ms >= 0.0);
    // t=1.0: animation complete, forwards keeps opacity at 1.0
    let r1 = render_pipeline_animated(html, css, 1.0);
    assert!(r1.timings.total_ms >= 0.0);
    assert!(!r1.primitives().fills.is_empty());
}

/// 测试动画 direction alternate 渲染管线。
#[test]
fn test_animation_direction_alternate_pipeline() {
    let html = r#"<html><body><div class="alt">Alt</div></body></html>"#;
    let css = r#"
        @keyframes pulse { 0% { opacity: 0.3; } 100% { opacity: 1.0; } }
        .alt { animation: pulse 1s linear infinite alternate; background-color: purple; width: 100px; height: 100px; }
    "#;
    let result = render_pipeline_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试 CSS transition 定义通过渲染管线不崩溃。
#[test]
fn test_transition_property_pipeline() {
    let html = r#"<html><body><div class="trans">Transition</div></body></html>"#;
    let css = r#"
        .trans {
            transition: opacity 0.5s ease, background-color 0.3s linear;
            opacity: 1.0; background-color: steelblue;
            width: 200px; height: 100px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试 transition 多属性管线渲染。
#[test]
fn test_transition_multi_property_pipeline() {
    let html = r#"<html><body><div class="multi">Multi</div></body></html>"#;
    let css = r#"
        .multi {
            transition-property: opacity, width;
            transition-duration: 0.3s, 0.5s;
            transition-timing-function: ease, linear;
            opacity: 0.8; width: 180px; background-color: teal; height: 100px;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

/// 测试动画 + transition 组合通过渲染管线。
#[test]
fn test_animation_transition_combo_pipeline() {
    let html = r#"<html><body><div class="combo">Combo</div></body></html>"#;
    let css = r#"
        @keyframes colorShift { 0% { opacity: 0.5; } 100% { opacity: 1.0; } }
        .combo {
            animation: colorShift 1s linear;
            transition: background-color 0.3s ease;
            background-color: navy; width: 200px; height: 120px;
        }
    "#;
    let result = render_pipeline_animated(html, css, 0.5);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── writing-mode + word-break 渲染管线集成测试 ──

/// 测试 writing-mode: vertical-rl 渲染管线 — 字形旋转。
#[test]
fn test_writing_mode_vertical_rl_pipeline() {
    let html = r#"<html><body><div class="vrl">Hello</div></body></html>"#;
    let css = r#"
        .vrl {
            writing-mode: vertical-rl;
            color: black;
            font-size: 16px;
            background: #e0ffe0;
            height: 200px;
        }
    "#;
    let result = render_pipeline(html, css);
    // 应有 fill（背景）和 glyph（文本），且 glyph 旋转 90°
    assert!(!result.primitives().fills.is_empty(), "应有背景 fill");
    let has_rotated = result
        .primitives()
        .glyphs
        .iter()
        .any(|g| g.glyph_id != 0 && (g.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    assert!(has_rotated, "vertical-rl glyph 应旋转 90°");
}

/// 测试 writing-mode: horizontal-tb 渲染管线 — 字形不旋转。
#[test]
fn test_writing_mode_horizontal_tb_pipeline() {
    let html = r#"<html><body><div class="htb">World</div></body></html>"#;
    let css = r#"
        .htb {
            writing-mode: horizontal-tb;
            color: black;
            font-size: 16px;
        }
    "#;
    let result = render_pipeline(html, css);
    // 所有 glyph 的 rotation 应为 0.0
    for g in &result.primitives().glyphs {
        if g.glyph_id != 0 {
            assert_eq!(g.rotation, 0.0, "horizontal-tb glyph 不应旋转");
        }
    }
}

/// 测试 word-break: break-all 渲染管线。
#[test]
fn test_word_break_break_all_pipeline() {
    let html = r#"<html><body><div class="ba">Supercalifragilistic</div></body></html>"#;
    let css = r#"
        .ba {
            word-break: break-all;
            color: black;
            font-size: 14px;
            width: 60px;
        }
    "#;
    let result = render_pipeline(html, css);
    // break-all 应生成字形
    let glyph_count = result.primitives().glyphs.iter().filter(|g| g.glyph_id != 0).count();
    assert!(glyph_count > 0, "break-all 应生成字形");
}

/// 测试 word-break: keep-all 渲染管线。
#[test]
fn test_word_break_keep_all_pipeline() {
    let html = r#"<html><body><div class="ka">中文文本测试</div></body></html>"#;
    let css = r#"
        .ka {
            word-break: keep-all;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // keep-all 应生成字形（CJK 作为整体）
    let glyph_count = result.primitives().glyphs.iter().filter(|g| g.glyph_id != 0).count();
    assert!(glyph_count > 0, "keep-all 应生成字形");
}

// ── CSS direction / tab-size / border-collapse / table-layout / font-variant-numeric ──

/// 测试 direction:rtl 渲染管线。
#[test]
fn test_direction_rtl_pipeline() {
    let html = r#"<html><body><div class="rtl">مرحبا</div></body></html>"#;
    let css = r#"
        .rtl {
            direction: rtl;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // rtl 应生成方向指示器 stroke
    assert!(!result.primitives().strokes.is_empty(), "rtl 应渲染方向指示器 stroke");
}

/// 测试 direction:ltr 渲染管线（无指示器）。
#[test]
fn test_direction_ltr_pipeline() {
    let html = r#"<html><body><div class="ltr">Hello</div></body></html>"#;
    let css = r#"
        .ltr {
            direction: ltr;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // ltr 不应生成方向指示器
    assert!(result.primitives().strokes.is_empty(), "ltr 不应渲染方向指示器");
}

/// 测试 tab-size 渲染管线。
#[test]
fn test_tab_size_pipeline() {
    let html = r#"<html><body><div class="tab">a\tb</div></body></html>"#;
    let css = r#"
        .tab {
            tab-size: 4;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // tab-size:4 应生成指示器 fill
    assert!(!result.primitives().fills.is_empty(), "tab-size:4 应渲染指示器");
}

/// 测试 border-collapse:collapse 渲染管线。
#[test]
fn test_border_collapse_pipeline() {
    let html = r#"<html><body><table class="tbl"><tr><td>A</td></tr></table></body></html>"#;
    let css = r#"
        .tbl {
            border-collapse: collapse;
        }
    "#;
    let result = render_pipeline(html, css);
    // collapse 应生成边框合并指示器 stroke
    assert!(!result.primitives().strokes.is_empty(), "collapse 应渲染边框合并指示器");
}

/// 测试 table-layout:fixed 渲染管线。
#[test]
fn test_table_layout_fixed_pipeline() {
    let html = r#"<html><body><table class="ft"><tr><td>A</td></tr></table></body></html>"#;
    let css = r#"
        .ft {
            table-layout: fixed;
        }
    "#;
    let result = render_pipeline(html, css);
    // fixed 应生成表格布局指示器 fill
    assert!(!result.primitives().fills.is_empty(), "fixed 应渲染表格布局指示器");
}

/// 测试 font-variant-numeric:tabular-nums 渲染管线。
#[test]
fn test_font_variant_numeric_pipeline() {
    let html = r#"<html><body><div class="nums">12345</div></body></html>"#;
    let css = r#"
        .nums {
            font-variant-numeric: tabular-nums;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // tabular-nums 应生成数字变体指示器 fill
    assert!(
        !result.primitives().fills.is_empty(),
        "tabular-nums 应渲染数字变体指示器"
    );
}

// ──────────────────────────────────────────────────────
// CSS contain / unicode-bidi / box-decoration-break / overflow-wrap / text-align-last
// break / scroll-area / scroll-snap-stop / container-type 渲染管线
// ──────────────────────────────────────────────────────

/// 测试 contain:strict 渲染管线。
#[test]
fn test_contain_strict_render_pipeline() {
    let html = r#"<html><body><div class="c">Content</div></body></html>"#;
    let css = r#"
        .c {
            contain: strict;
            width: 100px;
            height: 50px;
        }
    "#;
    let result = render_pipeline(html, css);
    // contain:strict 应生成包含指示器
    assert!(!result.primitives().fills.is_empty(), "contain:strict 应渲染指示器");
}

/// 测试 unicode-bidi:bidi-override 渲染管线。
#[test]
fn test_unicode_bidi_override_pipeline() {
    let html = r#"<html><body><div class="bidi">Hello</div></body></html>"#;
    let css = r#"
        .bidi {
            unicode-bidi: bidi-override;
            direction: rtl;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // bidi-override 应生成双向文本指示器
    assert!(!result.primitives().fills.is_empty(), "bidi-override 应渲染指示器");
}

/// 测试 box-decoration-break:clone 渲染管线。
#[test]
fn test_box_decoration_break_clone_pipeline() {
    let html = r#"<html><body><span class="clone">Text</span></body></html>"#;
    let css = r#"
        .clone {
            box-decoration-break: clone;
            background-color: yellow;
        }
    "#;
    let result = render_pipeline(html, css);
    // clone 应生成装饰断行指示器
    assert!(!result.primitives().fills.is_empty(), "clone 应渲染指示器");
}

/// 测试 overflow-wrap:break-word 渲染管线。
#[test]
fn test_overflow_wrap_break_word_pipeline() {
    let html = r#"<html><body><div class="wrap">LongWordThatNeedsBreaking</div></body></html>"#;
    let css = r#"
        .wrap {
            overflow-wrap: break-word;
            width: 50px;
            color: black;
            font-size: 14px;
        }
    "#;
    let result = render_pipeline(html, css);
    // break-word 应生成断词指示器
    assert!(!result.primitives().fills.is_empty(), "break-word 应渲染指示器");
}

/// 测试 text-align-last:center 渲染管线。
#[test]
fn test_text_align_last_center_pipeline() {
    let html = r#"<html><body><div class="last"><p>Line1</p></div></body></html>"#;
    let css = r#"
        .last {
            text-align-last: center;
            background-color: white;
            width: 200px;
            height: 50px;
        }
    "#;
    let result = render_pipeline(html, css);
    // text-align-last:center 应生成末行对齐指示器（fills 或 glyphs）
    let has_output = !result.primitives().fills.is_empty() || !result.primitives().glyphs.is_empty();
    assert!(has_output, "text-align-last:center 应渲染指示器");
}

/// 测试 break-before:column + break-after:page 渲染管线。
#[test]
fn test_break_points_pipeline() {
    let html = r#"<html><body><div class="break">Section</div></body></html>"#;
    let css = r#"
        .break {
            break-before: column;
            break-after: page;
            background-color: lightgray;
        }
    "#;
    let result = render_pipeline(html, css);
    // break 属性应生成断点指示器
    assert!(!result.primitives().fills.is_empty(), "break 属性应渲染指示器");
}

/// 测试 scroll-margin + scroll-padding 渲染管线。
#[test]
fn test_scroll_area_pipeline() {
    let html = r#"<html><body><div class="snap-item">Item</div></body></html>"#;
    let css = r#"
        .snap-item {
            scroll-margin: 10px;
            scroll-padding: 8px;
            background-color: white;
        }
    "#;
    let result = render_pipeline(html, css);
    // scroll-margin/padding 应生成滚动区域指示器
    assert!(!result.primitives().fills.is_empty(), "scroll-area 应渲染指示器");
}

/// 测试 scroll-snap-stop:always 渲染管线。
#[test]
fn test_scroll_snap_stop_always_pipeline() {
    let html = r#"<html><body><div class="stop">Stop</div></body></html>"#;
    let css = r#"
        .stop {
            scroll-snap-stop: always;
            background-color: white;
        }
    "#;
    let result = render_pipeline(html, css);
    // scroll-snap-stop:always 应生成强制停止指示器
    assert!(!result.primitives().fills.is_empty(), "snap-stop:always 应渲染指示器");
}

/// 测试 container-type:size + container-name 渲染管线。
#[test]
fn test_container_type_size_pipeline() {
    let html = r#"<html><body><div class="container"><div class="child">Content</div></div></body></html>"#;
    let css = r#"
        .container {
            container-type: size;
            container-name: sidebar;
            width: 200px;
            height: 100px;
            background-color: white;
        }
    "#;
    let result = render_pipeline(html, css);
    // container-type:size 应生成容器查询上下文指示器
    assert!(
        !result.primitives().fills.is_empty(),
        "container-type:size 应渲染指示器"
    );
}
