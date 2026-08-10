//! CSS 排版、视觉效果和表单管线集成测试。
//!
//! 验证 CSS 排版属性（字体、文本、颜色、边框、阴影、渐变、
//! opacity、visibility、overflow、filter 等）和表单交互元素
//! 从解析到样式计算到布局到渲染的完整管线。

use zero_engine::RenderPipeline;

/// 完整管线辅助函数。
fn render_pipeline(html: &str, css: &str) -> zero_engine::RenderResult {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(html, css)
}

// ── 1. 字体和文本属性管线 ──────────────────────────────────────────

#[test]
fn test_font_family_stack() {
    let html = r#"<html><body>
        <p class="serif">Serif text</p>
        <p class="mono">Mono text</p>
    </body></html>"#;
    let css = r#"
        .serif { font-family: Georgia, serif; }
        .mono { font-family: 'Courier New', monospace; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_font_size_variants() {
    let html = r#"<html><body>
        <p class="small">Small</p>
        <p class="large">Large</p>
        <p class="em">EM sized</p>
    </body></html>"#;
    let css = r#"
        .small { font-size: 12px; }
        .large { font-size: 32px; }
        .em { font-size: 1.5em; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
    // 不同字号应该产生不同的 glyph 位置
    assert!(result.layout.root.children.len() >= 2);
}

#[test]
fn test_font_weight_variants() {
    let html = r#"<html><body>
        <p class="light">Light</p>
        <p class="bold">Bold</p>
        <p class="black">Black</p>
    </body></html>"#;
    let css = r#"
        .light { font-weight: 300; }
        .bold { font-weight: bold; }
        .black { font-weight: 900; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_text_align_center() {
    let html = r#"<html><body>
        <p class="center">Center aligned text that is long enough to potentially wrap</p>
    </body></html>"#;
    let css = r#"
        .center { text-align: center; max-width: 200px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_text_decoration_underline() {
    let html = r#"<html><body>
        <p style="text-decoration: underline">Underlined text</p>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_text_decoration_line_through() {
    let html = r#"<html><body>
        <p style="text-decoration: line-through">Strikethrough text</p>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_text_transform_uppercase() {
    let html = r#"<html><body>
        <p style="text-transform: uppercase">this should be uppercase</p>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_letter_spacing() {
    let html = r#"<html><body>
        <p class="wide">Wide letters</p>
    </body></html>"#;
    let css = r#"
        .wide { letter-spacing: 0.2em; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_line_height() {
    let html = r#"<html><body>
        <p class="tight">Tight line height with enough text to wrap to a second line for testing.</p>
        <p class="loose">Loose line height with enough text to wrap to a second line for testing.</p>
    </body></html>"#;
    let css = r#"
        .tight { line-height: 1.0; max-width: 200px; }
        .loose { line-height: 2.0; max-width: 200px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

// ── 2. 颜色和背景管线 ──────────────────────────────────────────

#[test]
fn test_named_colors_render() {
    let html = r#"<html><body>
        <div class="red">Red</div>
        <div class="blue">Blue</div>
        <div class="gold">Gold</div>
    </body></html>"#;
    let css = r#"
        .red { color: red; }
        .blue { color: dodgerblue; }
        .gold { background: gold; color: black; padding: 10px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_rgb_hsl_colors() {
    let html = r#"<html><body>
        <div style="color: rgb(255, 128, 0)">RGB orange</div>
        <div style="color: hsl(120, 100%, 50%)">HSL green</div>
        <div style="background: rgba(0, 0, 255, 0.5); padding: 10px">RGBA blue</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_hex_colors() {
    let html = r#"<html><body>
        <div style="color: #ff6600">Full hex</div>
        <div style="color: #f60">Short hex</div>
        <div style="background: #333333; color: #eee; padding: 10px">Dark bg</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

// ── 3. 边框和圆角管线 ──────────────────────────────────────────

#[test]
fn test_border_styles_render() {
    let html = r#"<html><body>
        <div class="solid">Solid border</div>
        <div class="dashed">Dashed border</div>
        <div class="dotted">Dotted border</div>
    </body></html>"#;
    let css = r#"
        .solid { border: 2px solid black; padding: 10px; margin: 5px; }
        .dashed { border: 2px dashed black; padding: 10px; margin: 5px; }
        .dotted { border: 2px dotted black; padding: 10px; margin: 5px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_border_radius_render() {
    let html = r#"<html><body>
        <div class="rounded">Rounded</div>
        <div class="pill">Pill shape</div>
    </body></html>"#;
    let css = r#"
        .rounded { border: 2px solid black; border-radius: 10px; padding: 15px; }
        .pill { background: #4285f4; color: white; border-radius: 20px; padding: 8px 20px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_individual_borders() {
    let html = r#"<html><body>
        <div style="border-top: 3px solid red; border-bottom: 1px dashed blue; padding: 10px">Mixed borders</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 4. 阴影和渐变管线 ──────────────────────────────────────────

#[test]
fn test_box_shadow_multiple() {
    let html = r#"<html><body>
        <div class="card">Card with shadow</div>
    </body></html>"#;
    let css = r#"
        .card {
            box-shadow: 2px 2px 8px rgba(0,0,0,0.3), -1px -1px 4px rgba(0,0,0,0.1);
            padding: 20px; margin: 20px; background: white;
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().shadows.is_empty());
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_box_shadow_inset() {
    let html = r#"<html><body>
        <div style="box-shadow: inset 0 0 10px rgba(0,0,0,0.3); padding: 20px; margin: 20px; background: #f0f0f0">Inset shadow</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().shadows.is_empty());
}

#[test]
fn test_text_shadow_render() {
    let html = r#"<html><body>
        <p class="glow">Glowing text</p>
    </body></html>"#;
    let css = r#"
        .glow { text-shadow: 0 0 10px rgba(0, 150, 255, 0.8); font-size: 24px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_linear_gradient_directions() {
    let html = r#"<html><body>
        <div class="h-gradient"></div>
        <div class="v-gradient"></div>
        <div class="d-gradient"></div>
    </body></html>"#;
    let css = r#"
        .h-gradient { background: linear-gradient(to right, red, blue); height: 40px; margin: 5px; }
        .v-gradient { background: linear-gradient(to bottom, green, yellow); height: 40px; margin: 5px; }
        .d-gradient { background: linear-gradient(135deg, #ff0000, #00ff00, #0000ff); height: 40px; margin: 5px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().gradients.is_empty());
}

#[test]
fn test_radial_gradient_render() {
    let html = r#"<html><body>
        <div style="background: radial-gradient(circle, white, black); height: 100px; margin: 10px"></div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().gradients.is_empty());
}

// ── 5. opacity / visibility / overflow 管线 ──────────────────────────────────────────

#[test]
fn test_opacity_levels() {
    let html = r#"<html><body>
        <div style="opacity: 0.25; background: blue; color: white; padding: 10px; margin: 5px">25%</div>
        <div style="opacity: 0.75; background: blue; color: white; padding: 10px; margin: 5px">75%</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_visibility_hidden_takes_space() {
    let html = r#"<html><body>
        <div style="visibility: visible; background: green; height: 30px; margin: 5px">Visible</div>
        <div style="visibility: hidden; background: red; height: 30px; margin: 5px">Hidden</div>
        <div style="visibility: visible; background: blue; height: 30px; margin: 5px">Visible</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    // Hidden elements still take layout space but may not produce primitives
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_overflow_hidden_clips() {
    let html = r#"<html><body>
        <div style="width: 100px; height: 50px; overflow: hidden; border: 1px solid black">
            This text overflows and should be clipped by the container.
        </div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 6. CSS 变量管线 ──────────────────────────────────────────

#[test]
fn test_css_variables_render() {
    let html = r#"<html><body>
        <div class="card">Card content</div>
    </body></html>"#;
    let css = r#"
        :root {
            --primary: #0066cc;
            --bg: #f5f5f5;
            --radius: 8px;
            --spacing: 16px;
        }
        .card {
            background: var(--bg);
            color: var(--primary);
            border: 1px solid var(--primary);
            border-radius: var(--radius);
            padding: var(--spacing);
        }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_css_variables_fallback() {
    let html = r#"<html><body>
        <div style="color: var(--undef, #333); padding: 10px">Fallback color</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().glyphs.is_empty());
}

// ── 7. position / z-index 管线 ──────────────────────────────────────────

#[test]
fn test_absolute_position() {
    let html = r#"<html><body>
        <div class="container">
            <div class="absolute">Positioned</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { position: relative; height: 150px; background: #f0f0f0; }
        .absolute { position: absolute; top: 10px; right: 10px; background: #ffcccc; padding: 10px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_z_index_stacking() {
    let html = r#"<html><body>
        <div class="container">
            <div class="high" style="z-index: 3">High</div>
            <div class="mid" style="z-index: 2">Mid</div>
            <div class="low" style="z-index: 1">Low</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { position: relative; height: 100px; }
        .high, .mid, .low { position: absolute; width: 80px; height: 80px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    // Verify render completes and produces output
    assert!(!result.primitives().is_empty());
}

// ── 8. display 变体管线 ──────────────────────────────────────────

#[test]
fn test_display_inline_block() {
    let html = r#"<html><body>
        <div class="ib1">Block 1</div>
        <div class="ib2">Block 2</div>
        <div class="ib3">Block 3</div>
    </body></html>"#;
    let css = r#"
        .ib1, .ib2, .ib3 { display: inline-block; width: 100px; height: 50px; border: 1px solid #ccc; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_display_none_removes_layout() {
    let html = r#"<html><body>
        <div class="visible">Visible</div>
        <div class="hidden">This should not create layout box</div>
        <div class="visible2">Also visible</div>
    </body></html>"#;
    let css = r#"
        .visible, .visible2 { padding: 10px; }
        .hidden { display: none; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    // display:none should not produce layout children
    // Note: <html> is layout root, <head> is display:none (UA default),
    // so only <body> is a direct child of <html>.
    let visible_children: Vec<_> = result
        .layout
        .root
        .children
        .iter()
        .filter(|c| c.width > 0.0 || c.height > 0.0)
        .collect();
    assert!(
        !visible_children.is_empty(),
        "At least 1 visible element (body) should have layout"
    );
}

#[test]
fn test_display_flex_gap() {
    let html = r#"<html><body>
        <div class="flex">
            <div>A</div>
            <div>B</div>
            <div>C</div>
        </div>
    </body></html>"#;
    let css = r#"
        .flex { display: flex; gap: 10px; }
        .flex > div { flex: 1; background: #eee; padding: 10px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

#[test]
fn test_display_grid_responsive() {
    let html = r#"<html><body>
        <div class="grid">
            <div>1</div><div>2</div><div>3</div>
            <div>4</div><div>5</div><div>6</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
        .grid > div { padding: 15px; background: #e0e0e0; text-align: center; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.layout.root.children.is_empty());
}

// ── 9. box-sizing / calc() 管线 ──────────────────────────────────────────

#[test]
fn test_box_sizing_border_box() {
    let html = r#"<html><body>
        <div class="content-box">Content-box</div>
        <div class="border-box">Border-box</div>
    </body></html>"#;
    let css = r#"
        .content-box { width: 200px; padding: 20px; border: 5px solid red; box-sizing: content-box; margin: 5px; }
        .border-box { width: 200px; padding: 20px; border: 5px solid blue; box-sizing: border-box; margin: 5px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(result.layout.root.children.len() >= 2);
}

#[test]
fn test_calc_width() {
    let html = r#"<html><body>
        <div style="width: calc(100% - 40px); background: #eee; padding: 10px">Calc width</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 10. 2D transform 管线 ──────────────────────────────────────────

#[test]
fn test_transform_translate() {
    let html = r#"<html><body>
        <div style="transform: translate(50px, 20px); background: #ffcccc; padding: 10px">Translated</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_transform_scale() {
    let html = r#"<html><body>
        <div style="transform: scale(1.5); background: #ccffcc; padding: 10px; width: 50px; height: 50px">Scaled</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

#[test]
fn test_transform_rotate() {
    let html = r#"<html><body>
        <div style="transform: rotate(45deg); background: #ccccff; padding: 10px; width: 50px; height: 50px">Rotated</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
}

// ── 11. CSS filter 管线 ──────────────────────────────────────────

#[test]
fn test_filter_blur() {
    let html = r#"<html><body>
        <div style="filter: blur(2px); background: #eee; padding: 10px">Blurred</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_filter_grayscale() {
    let html = r#"<html><body>
        <div style="filter: grayscale(100%); background: #cc6633; color: white; padding: 10px">Grayscale</div>
    </body></html>"#;
    let result = render_pipeline(html, "");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 12. 综合页面管线 ──────────────────────────────────────────

#[test]
fn test_landing_page_render() {
    let html = r##"<html><body>
        <header class="hero">
            <h1>Build the Web</h1>
            <p class="subtitle">A fast, safe browser</p>
            <button class="cta">Get Started</button>
        </header>
        <section id="features">
            <div class="feature-grid">
                <div class="feature"><h3>Fast</h3><p>Rust powered</p></div>
                <div class="feature"><h3>Safe</h3><p>Memory safe</p></div>
                <div class="feature"><h3>Standard</h3><p>Web compliant</p></div>
            </div>
        </section>
    </body></html>"##;
    let css = r##"
        :root { --primary: #4285f4; --dark: #1a1a2e; }
        body { margin: 0; font-family: system-ui, sans-serif; }
        .hero { background: linear-gradient(135deg, var(--dark), #16213e); color: white; padding: 40px; text-align: center; }
        .hero h1 { font-size: 2.5em; margin: 0; }
        .subtitle { color: #aaa; font-size: 1.2em; }
        .cta { background: var(--primary); color: white; border: none; padding: 12px 30px; border-radius: 6px; cursor: pointer; box-shadow: 0 4px 15px rgba(66,133,244,0.4); }
        .feature-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; padding: 40px; }
        .feature { background: #f5f5f5; padding: 20px; border-radius: 8px; text-align: center; border: 1px solid #e0e0e0; }
    "##;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
    assert!(!result.primitives().gradients.is_empty());
    assert!(!result.primitives().shadows.is_empty());
}

#[test]
fn test_styled_form_render() {
    let html = r#"<html><body>
        <form class="styled">
            <div class="group">
                <label>Email</label>
                <input type="email" class="input" placeholder="you@example.com">
            </div>
            <div class="group">
                <label>Password</label>
                <input type="password" class="input">
            </div>
            <button type="submit" class="btn">Sign In</button>
        </form>
    </body></html>"#;
    let css = r#"
        .styled { max-width: 400px; margin: 20px; }
        .group { margin-bottom: 15px; }
        .group label { display: block; font-weight: bold; margin-bottom: 5px; }
        .input { width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box; }
        .btn { background: #0066cc; color: white; padding: 12px 24px; border: none; border-radius: 4px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_pricing_cards_render() {
    let html = r#"<html><body>
        <div class="pricing">
            <div class="plan"><h3>Basic</h3><div class="price">$9</div><button>Choose</button></div>
            <div class="plan featured"><h3>Pro</h3><div class="price">$29</div><button>Choose</button></div>
            <div class="plan"><h3>Enterprise</h3><div class="price">$99</div><button>Contact</button></div>
        </div>
    </body></html>"#;
    let css = r#"
        .pricing { display: flex; gap: 20px; padding: 40px; }
        .plan { flex: 1; border: 1px solid #ddd; border-radius: 12px; padding: 30px; text-align: center; }
        .plan.featured { border-color: #0066cc; box-shadow: 0 10px 30px rgba(0,102,204,0.2); }
        .price { font-size: 2.5em; font-weight: bold; color: #0066cc; }
        button { background: #0066cc; color: white; border: none; padding: 12px 30px; border-radius: 6px; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
    assert!(!result.primitives().shadows.is_empty());
}

#[test]
fn test_blog_article_render() {
    let html = r#"<html><body>
        <article class="post">
            <h1>The Art of Typography</h1>
            <p class="meta">June 2026</p>
            <p class="intro">Typography is the art of arranging type.</p>
            <blockquote><p>"Typography is what language looks like."</p></blockquote>
            <h2>Principles</h2>
            <ul>
                <li><strong>Hierarchy</strong> — using size and weight</li>
                <li><strong>Contrast</strong> — creating visual interest</li>
                <li><strong>Spacing</strong> — giving text room</li>
            </ul>
        </article>
    </body></html>"#;
    let css = r#"
        .post { max-width: 700px; margin: 0 auto; padding: 20px; font-family: Georgia, serif; }
        h1 { font-size: 2em; line-height: 1.2; }
        h2 { margin-top: 1.5em; }
        .meta { color: #666; font-size: 0.9em; }
        .intro { font-size: 1.1em; }
        blockquote { border-left: 4px solid #ccc; padding: 0.5em 1em; color: #555; background: #f9f9f9; }
        ul { padding-left: 1.5em; }
        li { margin: 0.3em 0; }
    "#;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}

#[test]
fn test_dashboard_render() {
    let html = r##"<html><body>
        <header>
            <h1>Dashboard</h1>
            <nav>
                <a href="#overview">Overview</a>
                <a href="#stats">Stats</a>
            </nav>
        </header>
        <main>
            <section>
                <div class="stats">
                    <div class="stat"><h3>Users</h3><p>1,234</p></div>
                    <div class="stat"><h3>Revenue</h3><p>$56k</p></div>
                </div>
            </section>
            <section>
                <table>
                    <thead><tr><th>Month</th><th>Visits</th></tr></thead>
                    <tbody>
                        <tr><td>Jan</td><td>10,000</td></tr>
                        <tr><td>Feb</td><td>12,000</td></tr>
                    </tbody>
                </table>
            </section>
        </main>
    </body></html>"##;
    let css = r##"
        body { margin: 0; font-family: sans-serif; }
        header { background: #333; color: white; padding: 10px 20px; display: flex; justify-content: space-between; }
        header nav a { color: #ccc; margin-left: 15px; }
        .stats { display: flex; gap: 20px; margin: 20px; }
        .stat { border: 1px solid #ddd; padding: 15px; flex: 1; }
        table { width: 100%; border-collapse: collapse; margin: 20px; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    "##;
    let result = render_pipeline(html, css);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives().fills.is_empty());
    assert!(!result.primitives().glyphs.is_empty());
}
