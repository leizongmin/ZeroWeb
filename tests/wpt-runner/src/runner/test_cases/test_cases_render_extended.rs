//! 渲染管线扩展合规性测试（text-decoration-style/color、3D 变换、组合渲染）。
//!
//! 从 test_cases_render.rs 拆分，保持单文件不超过 2000 行。

use super::TestCase;

/// 返回渲染管线扩展合规性测试用例。
pub fn render_extended_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  text-decoration-style 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── text-decoration: underline solid ──
        TestCase {
            id: "render/text-decoration-solid".to_string(),
            description: "CSS text-decoration-style: solid underline rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: underline solid; font-size: 20px; }</style>
<p class="t">Solid underline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── text-decoration: underline dotted ──
        TestCase {
            id: "render/text-decoration-dotted".to_string(),
            description: "CSS text-decoration-style: dotted underline rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: underline dotted; font-size: 20px; }</style>
<p class="t">Dotted underline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "stroke_count_ge:1".to_string(),
            ],
        },
        // ── text-decoration: underline dashed ──
        TestCase {
            id: "render/text-decoration-dashed".to_string(),
            description: "CSS text-decoration-style: dashed underline rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: underline dashed; font-size: 20px; }</style>
<p class="t">Dashed underline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "stroke_count_ge:1".to_string(),
            ],
        },
        // ── text-decoration: line-through double ──
        TestCase {
            id: "render/text-decoration-double".to_string(),
            description: "CSS text-decoration-style: double line-through rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: line-through double; font-size: 20px; }</style>
<p class="t">Double line-through text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },
        // ── text-decoration: underline wavy ──
        TestCase {
            id: "render/text-decoration-wavy".to_string(),
            description: "CSS text-decoration-style: wavy underline rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: underline wavy; font-size: 20px; }</style>
<p class="t">Wavy underline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:4".to_string(),
            ],
        },
        // ── text-decoration: overline dotted red ──
        TestCase {
            id: "render/text-decoration-overline-color".to_string(),
            description: "CSS text-decoration with overline, dotted style and red color".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration: overline dotted red; font-size: 20px; }</style>
<p class="t">Colored dotted overline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── text-decoration-color 长属性 ──
        TestCase {
            id: "render/text-decoration-color-blue".to_string(),
            description: "CSS text-decoration-color: blue with underline".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.t { text-decoration-line: underline; text-decoration-color: blue; font-size: 20px; }</style>
<p class="t">Blue underline text</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 3D 变换渲染扩展
        // ═══════════════════════════════════════════════════════════════

        // ── transform: rotateX ──
        TestCase {
            id: "render/transform-rotateX".to_string(),
            description: "CSS 3D transform: rotateX rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { width: 100px; height: 80px; background: #4488cc; transform: rotateX(30deg); }</style>
<div class="box">Rotated X</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transform: rotateY ──
        TestCase {
            id: "render/transform-rotateY".to_string(),
            description: "CSS 3D transform: rotateY rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { width: 100px; height: 80px; background: #cc44aa; transform: rotateY(45deg); }</style>
<div class="box">Rotated Y</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transform: perspective ──
        TestCase {
            id: "render/transform-perspective".to_string(),
            description: "CSS 3D transform: perspective function rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { width: 100px; height: 80px; background: #44cc88; transform: perspective(500px) rotateY(30deg); }</style>
<div class="box">Perspective</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transform: scale3d ──
        TestCase {
            id: "render/transform-scale3d".to_string(),
            description: "CSS 3D transform: scale3d rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { width: 60px; height: 40px; background: #ff8844; transform: scale3d(1.5, 1.5, 1); }</style>
<div class="box">Scaled 3D</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transform: translate3d ──
        TestCase {
            id: "render/transform-translate3d".to_string(),
            description: "CSS 3D transform: translate3d rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { width: 80px; height: 60px; background: #8844ff; transform: translate3d(20px, 10px, 0); }</style>
<div class="box">Translated 3D</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 组合渲染
        // ═══════════════════════════════════════════════════════════════

        // ── transform + box-shadow 组合 ──
        TestCase {
            id: "render/transform-box-shadow".to_string(),
            description: "CSS transform combined with box-shadow rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.card {
  width: 200px; height: 150px; background: #fff;
  transform: rotate(5deg);
  box-shadow: 5px 5px 15px rgba(0,0,0,0.3);
}
</style>
<div class="card">Transformed with shadow</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "shadow_count_ge:1".to_string(),
            ],
        },
        // ── 多层渐变 + transform ──
        TestCase {
            id: "render/gradient-transform-combo".to_string(),
            description: "CSS gradient background with transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.box {
  width: 150px; height: 100px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  transform: rotate(-3deg) scale(1.1);
  border-radius: 8px;
}
</style>
<div class="box">Gradient + Transform</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },
        // ── filter + opacity + transform ──
        TestCase {
            id: "render/filter-opacity-transform".to_string(),
            description: "CSS filter + opacity + transform combined rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.box {
  width: 120px; height: 80px; background: #e74c3c;
  filter: blur(2px); opacity: 0.7; transform: skewX(10deg);
}
</style>
<div class="box">Filtered + Transparent + Skewed</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── text-decoration + text-shadow 组合 ──
        TestCase {
            id: "render/text-decoration-shadow-combo".to_string(),
            description: "CSS text-decoration with text-shadow combined rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.fancy {
  font-size: 28px;
  text-decoration: underline wavy red;
  text-shadow: 2px 2px 4px rgba(0,0,0,0.5);
  color: #2c3e50;
}
</style>
<p class="fancy">Fancy text decoration</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── column-count + column-rule 综合页面 ──
        TestCase {
            id: "render/multi-column-page".to_string(),
            description: "CSS multi-column layout with column-rule rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.cols {
  column-count: 3; column-rule: 1px solid #ccc; column-gap: 20px;
  font-family: sans-serif; font-size: 14px;
}
</style>
<div class="cols">
  <p>First column content with some text to fill the space.</p>
  <p>Second paragraph that flows across columns automatically.</p>
  <p>Third paragraph continuing the multi-column layout flow.</p>
</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },
        // ── white-space + overflow 组合 ──
        TestCase {
            id: "render/white-space-overflow".to_string(),
            description: "CSS white-space pre with overflow hidden rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.pre-box {
  width: 200px; height: 80px; overflow: hidden;
  white-space: pre; font-family: monospace; font-size: 14px;
  background: #f5f5f5; border: 1px solid #ddd;
}
</style>
<div class="pre-box">This text preserves     whitespace
and newlines within the constrained box.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS quotes / scrollbar-gutter / background-attachment / hyphens 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── quotes: 自定义引号对 ──
        TestCase {
            id: "render/quotes-pairs".to_string(),
            description: "CSS quotes: Pairs custom quote marks rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>q { quotes: "«" "»" "‹" "›"; }</style>
<q>First level <q>nested</q> quote</q>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── quotes: none ──
        TestCase {
            id: "render/quotes-none".to_string(),
            description: "CSS quotes: none suppresses quote marks".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>q { quotes: none; }</style>
<q>No quotes here</q>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── scrollbar-gutter: stable ──
        TestCase {
            id: "render/scrollbar-gutter-stable".to_string(),
            description: "CSS scrollbar-gutter: stable reserves gutter space".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { scrollbar-gutter: stable; width: 200px; height: 100px; overflow: auto; }</style>
<div class="box">Scrollable content with stable gutter.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── scrollbar-gutter: stable both-edges ──
        TestCase {
            id: "render/scrollbar-gutter-both-edges".to_string(),
            description: "CSS scrollbar-gutter: stable both-edges rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { scrollbar-gutter: stable both-edges; width: 200px; height: 100px; overflow: auto; }</style>
<div class="box">Content with gutters on both edges.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ── background-attachment: fixed ──
        TestCase {
            id: "render/background-attachment-fixed".to_string(),
            description: "CSS background-attachment: fixed indicator rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.bg { background-attachment: fixed; background-image: url(/img/bg.png); width: 200px; height: 100px; }</style>
<div class="bg">Fixed background content.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── hyphens: auto ──
        TestCase {
            id: "render/hyphens-auto".to_string(),
            description: "CSS hyphens: auto indicator rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>p { hyphens: auto; width: 100px; }</style>
<p>Longwordthatneedshyphenation support.</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── text-wrap: nowrap ──
        TestCase {
            id: "render/text-wrap-nowrap".to_string(),
            description: "CSS text-wrap: nowrap prevents line wrapping".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.nowrap { text-wrap: nowrap; width: 100px; }</style>
<div class="nowrap">This is a long text that should not wrap.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── line-clamp: 3 ──
        TestCase {
            id: "render/line-clamp-3".to_string(),
            description: "CSS line-clamp: 3 limits text to 3 lines with ellipsis".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.clamped { line-clamp: 3; width: 200px; }</style>
<div class="clamped">This is the first line of text. This is the second line. This is the third line. This is the fourth line that should be clamped.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── 组合: scrollbar-gutter + scrollbar-width ──
        TestCase {
            id: "render/scrollbar-gutter-thin".to_string(),
            description: "CSS scrollbar-gutter: stable with scrollbar-width: thin".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.box { scrollbar-gutter: stable; scrollbar-width: thin; width: 200px; height: 100px; overflow: auto; }</style>
<div class="box">Thin scrollbar gutter content.</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS cursor 指示器渲染 ──

        TestCase {
            id: "render/cursor-pointer".to_string(),
            description: "CSS cursor: pointer indicator rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.link { cursor: pointer; width: 200px; height: 50px; background: #eee; }</style>
<div class="link">Click me</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        TestCase {
            id: "render/cursor-crosshair".to_string(),
            description: "CSS cursor: crosshair indicator rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.target { cursor: crosshair; width: 100px; height: 100px; background: #ddd; }</style>
<div class="target">Target area</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS image-rendering 指示器渲染 ──

        TestCase {
            id: "render/image-rendering-pixelated".to_string(),
            description: "CSS image-rendering: pixelated indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.pixel { image-rendering: pixelated; width: 100px; height: 100px; background: #ccc; }</style>
<div class="pixel">Pixel art</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        TestCase {
            id: "render/image-rendering-crisp-edges".to_string(),
            description: "CSS image-rendering: crisp-edges indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.sharp { image-rendering: crisp-edges; width: 100px; height: 100px; background: #bbb; }</style>
<div class="sharp">Sharp image</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS isolation 指示器渲染 ──

        TestCase {
            id: "render/isolation-isolate".to_string(),
            description: "CSS isolation: isolate stacking context indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.stack { isolation: isolate; width: 200px; height: 100px; background: #eef; }</style>
<div class="stack">Isolated stacking context</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS will-change 指示器渲染 ──

        TestCase {
            id: "render/will-change-transform".to_string(),
            description: "CSS will-change: transform performance hint indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.animated { will-change: transform; width: 200px; height: 100px; background: #ffe; }</style>
<div class="animated">Will animate</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS pointer-events 指示器渲染 ──

        TestCase {
            id: "render/pointer-events-none".to_string(),
            description: "CSS pointer-events: none click-through indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.overlay { pointer-events: none; width: 200px; height: 100px; background: rgba(0,0,0,0.1); }</style>
<div class="overlay">Invisible to clicks</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS user-select 指示器渲染 ──

        TestCase {
            id: "render/user-select-none".to_string(),
            description: "CSS user-select: none no-selection indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.noselect { user-select: none; width: 200px; height: 50px; background: #f0f0f0; }</style>
<div class="noselect">Cannot select this text</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS overscroll-behavior 指示器渲染 ──

        TestCase {
            id: "render/overscroll-behavior-contain".to_string(),
            description: "CSS overscroll-behavior: contain scroll boundary indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.scroll-box { overscroll-behavior: contain; width: 200px; height: 100px; overflow: auto; background: #f5f5f5; }</style>
<div class="scroll-box">Scrollable with boundary</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS touch-action 指示器渲染 ──

        TestCase {
            id: "render/touch-action-none".to_string(),
            description: "CSS touch-action: none touch behavior indicator".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>.notouch { touch-action: none; width: 200px; height: 100px; background: #fafafa; }</style>
<div class="notouch">No touch gestures</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS 交互属性组合渲染 ──

        TestCase {
            id: "render/interaction-combo".to_string(),
            description: "CSS cursor + isolation + will-change + pointer-events + user-select combined".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
.combo {
    cursor: pointer;
    isolation: isolate;
    will-change: transform;
    pointer-events: none;
    user-select: none;
    width: 300px;
    height: 150px;
    background: #e8e8ff;
}
</style>
<div class="combo">All interaction hints combined</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
    ]
}
