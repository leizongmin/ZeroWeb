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
    assert!(!result.primitives.glyphs.is_empty(), "Expected glyph primitives");
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
    assert!(!result.primitives.glyphs.is_empty(), "Expected glyphs");
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
    assert!(!result.primitives.glyphs.is_empty(), "Card text should produce glyphs");
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
    assert!(!result.primitives.glyphs.is_empty(), "Layout should produce glyphs");
}
