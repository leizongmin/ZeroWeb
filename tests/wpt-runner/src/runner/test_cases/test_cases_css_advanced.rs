//! CSS 高级特性合规性测试（Container Queries、Containment、高级背景、高级视觉效果）。
//!
//! 覆盖 CSS Container Queries、CSS Containment、高级 background 属性组合、
//! 高级 filter/clip-path/mask/isolation 视觉效果。

use super::TestCase;

/// 返回 CSS 高级特性合规性测试用例。
pub fn css_advanced_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  CSS Container Queries
        // ═══════════════════════════════════════════════════════════════

        // ── container-type: inline-size 基础 ──
        TestCase {
            id: "css-advanced/container-type-inline-size".to_string(),
            description: "CSS container-type: inline-size basic rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .container { container-type: inline-size; width: 300px; background: #f0f0f0; padding: 10px; }
  .item { background: #4a90d9; color: white; padding: 8px; margin: 4px 0; }
  @container (min-width: 250px) { .item { background: #d94a4a; } }
</style>
<div class="container">
  <div class="item">Item 1</div>
  <div class="item">Item 2</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── container-type: size ──
        TestCase {
            id: "css-advanced/container-type-size".to_string(),
            description: "CSS container-type: size rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .container { container-type: size; width: 200px; height: 200px; background: #e8e8e8; }
  .child { padding: 10px; }
  @container (min-height: 150px) { .child { background: #4ad94a; } }
</style>
<div class="container"><div class="child">Sized Container</div></div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── container-name 命名容器 ──
        TestCase {
            id: "css-advanced/container-name".to_string(),
            description: "CSS container-name with named query".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .sidebar { container-type: inline-size; container-name: sidebar; width: 200px; background: #fafafa; padding: 10px; }
  .widget { background: #ddd; padding: 8px; margin: 4px 0; }
  @container sidebar (min-width: 180px) { .widget { background: #90d9; } }
</style>
<div class="sidebar"><div class="widget">Widget 1</div></div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── container 嵌套 ──
        TestCase {
            id: "css-advanced/container-nested".to_string(),
            description: "CSS nested container queries".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .outer { container-type: inline-size; width: 500px; background: #f5f5f5; padding: 10px; }
  .inner { container-type: inline-size; width: 200px; background: #eee; padding: 8px; margin: 10px 0; }
  .card { background: #ccc; padding: 5px; }
  @container (min-width: 400px) { .inner { background: #ddd; } }
  @container (min-width: 150px) { .card { background: #bbb; } }
</style>
<div class="outer">
  <div class="inner"><div class="card">Nested Card</div></div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── container 响应式卡片布局 ──
        TestCase {
            id: "css-advanced/container-responsive-cards".to_string(),
            description: "CSS container queries responsive card layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .card-container { container-type: inline-size; width: 100%; }
  .card { display: flex; gap: 8px; padding: 10px; background: #f0f0f0; margin: 4px 0; }
  @container (min-width: 400px) { .card { flex-direction: row; } }
  @container (max-width: 399px) { .card { flex-direction: column; } }
</style>
<div class="card-container"><div class="card">
  <div>Image</div><div>Content</div>
</div></div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Containment
        // ═══════════════════════════════════════════════════════════════

        // ── contain: layout ──
        TestCase {
            id: "css-advanced/contain-layout".to_string(),
            description: "CSS contain: layout rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .contained { contain: layout; background: #e0e0ff; padding: 10px; margin: 10px; }
  .child { background: #c0c0ff; padding: 5px; }
</style>
<div class="contained"><div class="child">Contained Layout</div></div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── contain: strict ──
        TestCase {
            id: "css-advanced/contain-strict".to_string(),
            description: "CSS contain: strict rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .strict { contain: strict; width: 200px; height: 100px; background: #ffe0e0; padding: 5px; overflow: hidden; }
</style>
<div class="strict">Strictly contained element with content</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── contain: content ──
        TestCase {
            id: "css-advanced/contain-content".to_string(),
            description: "CSS contain: content rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .content-contained { contain: content; background: #e0ffe0; padding: 15px; margin: 5px; }
</style>
<div class="content-contained">Content containment scope</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── contain: inline-size ──
        TestCase {
            id: "css-advanced/contain-inline-size".to_string(),
            description: "CSS contain: inline-size rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .is-contained { contain: inline-size; background: #fff0e0; padding: 10px; }
</style>
<div class="is-contained">Inline-size containment</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── content-visibility: auto ──
        TestCase {
            id: "css-advanced/content-visibility-auto".to_string(),
            description: "CSS content-visibility: auto rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .offscreen { content-visibility: auto; contain-intrinsic-size: 0 200px; background: #f0f0f0; margin: 5px; }
</style>
<div class="offscreen">Potentially offscreen content</div>
<div class="offscreen">More content</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  高级 Background 属性组合
        // ═══════════════════════════════════════════════════════════════

        // ── 多层 background-image 渐变叠加 ──
        TestCase {
            id: "css-advanced/multi-layer-gradient".to_string(),
            description: "CSS multiple layered gradient backgrounds".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .layers {
    width: 300px; height: 200px;
    background-image: linear-gradient(45deg, rgba(255,0,0,0.3), rgba(0,0,255,0.3)),
                      radial-gradient(circle, rgba(0,255,0,0.5), transparent);
    background-color: #333;
  }
</style>
<div class="layers">Multi-layer gradient</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-position 多值组合 ──
        TestCase {
            id: "css-advanced/bg-position-multi-value".to_string(),
            description: "CSS background-position with multiple values".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .pos-right-bottom { width: 300px; height: 200px; background: #4a90d9; background-position: right bottom; }
  .pos-center { width: 300px; height: 200px; background: #d94a4a; background-position: center; margin-top: 5px; }
  .pos-25pct-75pct { width: 300px; height: 200px; background: #4ad94a; background-position: 25% 75%; margin-top: 5px; }
</style>
<div class="pos-right-bottom">Right Bottom</div>
<div class="pos-center">Center</div>
<div class="pos-25pct-75pct">25% 75%</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:3".to_string(),
            ],
        },

        // ── background-size cover/contain ──
        TestCase {
            id: "css-advanced/bg-size-cover-contain".to_string(),
            description: "CSS background-size cover and contain".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .cover { width: 300px; height: 100px; background: linear-gradient(to right, #ff0000, #0000ff); background-size: cover; }
  .contain { width: 300px; height: 100px; background: linear-gradient(to right, #00ff00, #ffff00); background-size: contain; margin-top: 5px; }
  .explicit { width: 300px; height: 100px; background: linear-gradient(to right, #ff00ff, #00ffff); background-size: 50% 100%; margin-top: 5px; }
</style>
<div class="cover">Cover</div>
<div class="contain">Contain</div>
<div class="explicit">50% 100%</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── background-clip text 效果 ──
        TestCase {
            id: "css-advanced/bg-clip-text".to_string(),
            description: "CSS background-clip: text rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .text-clip {
    font-size: 48px; font-weight: bold;
    background: linear-gradient(to right, #ff0000, #0000ff);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }
</style>
<div class="text-clip">Gradient Text</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── background-attachment: fixed ──
        TestCase {
            id: "css-advanced/bg-attachment-fixed".to_string(),
            description: "CSS background-attachment: fixed rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .fixed-bg {
    width: 300px; height: 200px; overflow: auto;
    background: linear-gradient(#e66465, #9198e5);
    background-attachment: fixed;
    padding: 20px; color: white;
  }
</style>
<div class="fixed-bg">Fixed background attachment content</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-origin content-box ──
        TestCase {
            id: "css-advanced/bg-origin-content-box".to_string(),
            description: "CSS background-origin: content-box rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .border-box { width: 300px; height: 100px; background: #4a90d9; background-origin: border-box; border: 10px solid #333; padding: 20px; }
  .content-box { width: 300px; height: 100px; background: #d94a4a; background-origin: content-box; border: 10px solid #333; padding: 20px; margin-top: 5px; }
</style>
<div class="border-box">Border Box</div>
<div class="content-box">Content Box</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  高级视觉效果
        // ═══════════════════════════════════════════════════════════════

        // ── filter 多函数组合 ──
        TestCase {
            id: "css-advanced/filter-multi-function".to_string(),
            description: "CSS filter with multiple functions combined".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .multi { width: 200px; height: 150px; background: #4a90d9; filter: blur(2px) brightness(1.2) saturate(1.5); }
  .sepia { width: 200px; height: 150px; background: #d94a4a; filter: sepia(0.8) contrast(1.3); margin-top: 5px; }
  .hue { width: 200px; height: 150px; background: #4ad94a; filter: hue-rotate(90deg) grayscale(0.3); margin-top: 5px; }
</style>
<div class="multi">Blur+Brightness+Saturate</div>
<div class="sepia">Sepia+Contrast</div>
<div class="hue">Hue-rotate+Grayscale</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── filter drop-shadow ──
        TestCase {
            id: "css-advanced/filter-drop-shadow".to_string(),
            description: "CSS filter: drop-shadow() rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .shadow1 { width: 150px; height: 100px; background: #4a90d9; filter: drop-shadow(4px 4px 2px rgba(0,0,0,0.5)); margin: 20px; }
  .shadow2 { width: 150px; height: 100px; background: #d94a4a; filter: drop-shadow(0 0 10px rgba(255,0,0,0.8)); margin: 20px; }
</style>
<div class="shadow1">Drop Shadow 1</div>
<div class="shadow2">Glow Shadow</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── clip-path 组合 ──
        TestCase {
            id: "css-advanced/clip-path-combined".to_string(),
            description: "CSS clip-path with various shapes".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .inset { width: 200px; height: 100px; background: #4a90d9; clip-path: inset(10px 20px 10px 20px); margin: 5px; }
  .circle { width: 150px; height: 150px; background: #d94a4a; clip-path: circle(50%); margin: 5px; }
  .polygon { width: 200px; height: 150px; background: #4ad94a; clip-path: polygon(50% 0%, 100% 100%, 0% 100%); margin: 5px; }
</style>
<div class="inset">Inset Clip</div>
<div class="circle">Circle</div>
<div class="polygon">Triangle</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── isolation + mix-blend-mode 组合 ──
        TestCase {
            id: "css-advanced/isolation-blend-mode".to_string(),
            description: "CSS isolation + mix-blend-mode combined rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .isolated { isolation: isolate; background: #ffffff; padding: 20px; }
  .blend-1 { width: 150px; height: 80px; background: rgba(255,0,0,0.7); mix-blend-mode: multiply; }
  .blend-2 { width: 120px; height: 60px; background: rgba(0,0,255,0.7); mix-blend-mode: screen; margin-top: -30px; margin-left: 30px; }
</style>
<div class="isolated">
  <div class="blend-1">Multiply</div>
  <div class="blend-2">Screen</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── opacity + transform + filter 三合一 ──
        TestCase {
            id: "css-advanced/opacity-transform-filter".to_string(),
            description: "CSS opacity + transform + filter triple combination".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .combo {
    width: 200px; height: 150px; background: #4a90d9; color: white; padding: 20px;
    opacity: 0.8;
    transform: rotate(5deg) scale(0.95);
    filter: blur(1px) brightness(1.1);
  }
</style>
<div class="combo">Opacity + Transform + Filter</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Scroll Snap 扩展
        // ═══════════════════════════════════════════════════════════════

        // ── scroll-snap 完整容器 ──
        TestCase {
            id: "css-advanced/scroll-snap-full".to_string(),
            description: "CSS scroll-snap container with snap alignment".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .snap-container {
    width: 300px; height: 200px; overflow: auto;
    scroll-snap-type: x mandatory;
    display: flex; gap: 0;
  }
  .snap-item { min-width: 300px; height: 200px; scroll-snap-align: center; display: flex; align-items: center; justify-content: center; color: white; font-size: 24px; }
  .s1 { background: #4a90d9; }
  .s2 { background: #d94a4a; }
  .s3 { background: #4ad94a; }
</style>
<div class="snap-container">
  <div class="snap-item s1">Slide 1</div>
  <div class="snap-item s2">Slide 2</div>
  <div class="snap-item s3">Slide 3</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:3".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── scroll-snap-stop: always ──
        TestCase {
            id: "css-advanced/scroll-snap-stop".to_string(),
            description: "CSS scroll-snap-stop: always rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .snap-y { width: 200px; height: 300px; overflow-y: auto; scroll-snap-type: y proximity; }
  .snap-section { height: 300px; scroll-snap-align: start; scroll-snap-stop: always; padding: 20px; }
  .sec1 { background: #e8f0fe; }
  .sec2 { background: #fce8e8; }
  .sec3 { background: #e8fce8; }
</style>
<div class="snap-y">
  <div class="snap-section sec1">Section 1 (stop)</div>
  <div class="snap-section sec2">Section 2 (stop)</div>
  <div class="snap-section sec3">Section 3 (stop)</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  高级排版
        // ═══════════════════════════════════════════════════════════════

        // ── text-wrap: balance ──
        TestCase {
            id: "css-advanced/text-wrap-balance".to_string(),
            description: "CSS text-wrap: balance rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  h1 { text-wrap: balance; width: 300px; font-size: 24px; background: #f0f0f0; padding: 10px; }
</style>
<h1>This is a long heading that should be balanced across lines</h1>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── line-clamp 多行截断 ──
        TestCase {
            id: "css-advanced/line-clamp".to_string(),
            description: "CSS -webkit-line-clamp multi-line truncation".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .clamped {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    width: 300px; background: #f5f5f5; padding: 10px;
  }
</style>
<div class="clamped">This is a very long text that should be clamped to exactly three lines of content before being truncated with an ellipsis at the end of the third line.</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── text-shadow 多重阴影 ──
        TestCase {
            id: "css-advanced/text-shadow-multiple".to_string(),
            description: "CSS text-shadow with multiple shadows".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .multi-shadow {
    font-size: 36px; color: #4a90d9;
    text-shadow: 2px 2px 0 #d94a4a, -2px -2px 0 #4ad94a, 0 0 10px rgba(0,0,0,0.3);
  }
</style>
<div class="multi-shadow">Multi Shadow Text</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── scrollbar-width + scrollbar-color ──
        TestCase {
            id: "css-advanced/scrollbar-styling".to_string(),
            description: "CSS scrollbar-width and scrollbar-color rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .thin { scrollbar-width: thin; scrollbar-color: #4a90d9 transparent; overflow-y: auto; height: 100px; }
  .auto { scrollbar-width: auto; scrollbar-color: #d94a4a #f0f0f0; overflow-y: auto; height: 100px; margin-top: 10px; }
</style>
<div class="thin">
  <p>Line 1</p><p>Line 2</p><p>Line 3</p><p>Line 4</p><p>Line 5</p>
  <p>Line 6</p><p>Line 7</p><p>Line 8</p><p>Line 9</p><p>Line 10</p>
</div>
<div class="auto">
  <p>Line 1</p><p>Line 2</p><p>Line 3</p><p>Line 4</p><p>Line 5</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 响应式综合页面
        // ═══════════════════════════════════════════════════════════════

        // ── Container Queries + Grid 综合布局 ──
        TestCase {
            id: "css-advanced/container-grid-dashboard".to_string(),
            description: "CSS container queries + grid dashboard layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .dashboard { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 10px; padding: 10px; }
  .widget { container-type: inline-size; background: #f8f8f8; border: 1px solid #ddd; border-radius: 8px; padding: 15px; }
  .widget-content { display: flex; flex-direction: column; gap: 5px; }
  @container (min-width: 220px) { .widget-content { flex-direction: row; } }
  .stat { background: #4a90d9; color: white; padding: 8px; border-radius: 4px; text-align: center; }
</style>
<div class="dashboard">
  <div class="widget"><div class="widget-content">
    <div class="stat">100</div><div class="stat">200</div>
  </div></div>
  <div class="widget"><div class="widget-content">
    <div class="stat">300</div><div class="stat">400</div>
  </div></div>
  <div class="widget"><div class="widget-content">
    <div class="stat">500</div><div class="stat">600</div>
  </div></div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:3".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── Containment + overflow + filter 综合页面 ──
        TestCase {
            id: "css-advanced/contain-overflow-filter".to_string(),
            description: "CSS containment + overflow + filter combined page".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .card-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 15px; padding: 10px; }
  .card { contain: content; border: 1px solid #ccc; border-radius: 8px; overflow: hidden; }
  .card-header { background: #4a90d9; color: white; padding: 10px; }
  .card-body { padding: 10px; max-height: 100px; overflow: hidden; }
  .blurred { filter: blur(0.5px); opacity: 0.9; }
</style>
<div class="card-grid">
  <div class="card"><div class="card-header">Card 1</div><div class="card-body">Content with containment</div></div>
  <div class="card blurred"><div class="card-header">Card 2</div><div class="card-body">Blurred content</div></div>
  <div class="card"><div class="card-header">Card 3</div><div class="card-body">More content here with some extra text for overflow testing</div></div>
  <div class="card"><div class="card-header">Card 4</div><div class="card-body">Normal card</div></div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:4".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── will-change 性能提示 ──
        TestCase {
            id: "css-advanced/will-change".to_string(),
            description: "CSS will-change performance hints rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .animated { width: 200px; height: 100px; background: #4a90d9; will-change: transform, opacity; }
  .scroll-hint { width: 200px; height: 100px; background: #d94a4a; will-change: scroll-position; margin-top: 10px; }
  .content-hint { width: 200px; height: 100px; background: #4ad94a; will-change: contents; margin-top: 10px; }
</style>
<div class="animated">Transform + Opacity</div>
<div class="scroll-hint">Scroll Position</div>
<div class="content-hint">Contents</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:3".to_string(),
            ],
        },

        // ── appearance 表单控件外观 ──
        TestCase {
            id: "css-advanced/appearance-controls".to_string(),
            description: "CSS appearance property for form controls".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .native { appearance: auto; }
  .none-appearance { appearance: none; background: #f0f0f0; border: 1px solid #ccc; padding: 5px; }
  .button-appearance { appearance: button; padding: 8px 16px; }
  .textfield { appearance: textfield; padding: 5px; }
</style>
<input class="native" type="text" value="Native" /><br><br>
<input class="none-appearance" type="text" value="None" /><br><br>
<div class="button-appearance">Button Appearance</div>
<input class="textfield" type="text" value="Textfield" />
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── writing-mode + direction 双向布局 ──
        TestCase {
            id: "css-advanced/writing-mode-direction".to_string(),
            description: "CSS writing-mode + direction combined layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .horizontal { writing-mode: horizontal-tb; direction: ltr; background: #e8f0fe; padding: 10px; margin: 5px; }
  .vertical-rl { writing-mode: vertical-rl; direction: rtl; background: #fce8e8; padding: 10px; margin: 5px; height: 200px; }
  .vertical-lr { writing-mode: vertical-lr; direction: ltr; background: #e8fce8; padding: 10px; margin: 5px; height: 200px; }
</style>
<div class="horizontal">Horizontal LTR</div>
<div class="vertical-rl">垂直从右到左</div>
<div class="vertical-lr">垂直从左到右</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
    ]
}
