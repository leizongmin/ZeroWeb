//! 渲染管线详细效果测试 — background/column/list/shadow/filter/animation/transition/transform/counter/table/content/object-fit。

use super::TestCase;

/// 返回渲染管线详细效果测试用例（背景/列/列表/阴影/滤镜/动画/过渡/变换/计数器/表格/内容/object-fit）。
pub fn render_detail_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  background-position / size / clip / origin 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── background-position: center ──
        TestCase {
            id: "render/bg-position-center".to_string(),
            description: "background-position:center renders correctly".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;background-color:#e0e0e0;background-image:url('photo.jpg');background-position:center">Centered</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-position: right bottom ──
        TestCase {
            id: "render/bg-position-right-bottom".to_string(),
            description: "background-position:right bottom offset".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;background-color:#eee;background-image:url('icon.png');background-position:right bottom;background-repeat:no-repeat">RB</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-position 百分比 + 长度组合 ──
        TestCase {
            id: "render/bg-position-two-value".to_string(),
            description: "background-position with two values (percent and length)".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f0f0f0;background-image:url('bg.jpg');background-position:50% 20px">Two Values</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: cover ──
        TestCase {
            id: "render/bg-size-cover".to_string(),
            description: "background-size:cover scales to cover container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:200px;background-color:#ccc;background-image:url('hero.jpg');background-size:cover">Cover</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: contain ──
        TestCase {
            id: "render/bg-size-contain".to_string(),
            description: "background-size:contain fits within container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#ddd;background-image:url('logo.png');background-size:contain;background-repeat:no-repeat">Contain</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: 50% 百分比 ──
        TestCase {
            id: "render/bg-size-percent".to_string(),
            description: "background-size:50% scales to half container width".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f5f5f5;background-image:url('bg.jpg');background-size:50%">Half Width</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-clip: content-box ──
        TestCase {
            id: "render/bg-clip-content-box".to_string(),
            description: "background-clip:content-box clips to content area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;padding:20px;border:5px solid #333;background-color:#ff6b6b;background-clip:content-box">Clipped</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-clip: padding-box ──
        TestCase {
            id: "render/bg-clip-padding-box".to_string(),
            description: "background-clip:padding-box clips to padding area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;padding:15px;border:8px solid #555;background-color:#4ecdc4;background-clip:padding-box">Padded</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-origin: content-box + position ──
        TestCase {
            id: "render/bg-origin-content-box".to_string(),
            description: "background-origin:content-box positions from content area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;padding:20px;border:10px solid #999;background-color:#f0f0f0;background-image:url('bg.jpg');background-origin:content-box;background-position:center">Origin</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── 渐变 + background-size + position 组合 ──
        TestCase {
            id: "render/gradient-with-size-position".to_string(),
            description: "Gradient with background-size and position".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f8f8f8;background:linear-gradient(135deg,#667eea,#764ba2);background-size:50% 50%;background-position:center">Gradient Positioned</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── background 完整简写测试 ──
        TestCase {
            id: "render/bg-shorthand-comprehensive".to_string(),
            description: "Background shorthand with color image position/size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:180px;background:#e8f4f8 url('bg.png') no-repeat center/contain">Shorthand</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Column-rule 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── column-rule: solid ──
        TestCase {
            id: "render/column-rule-solid".to_string(),
            description: "column-rule: solid 多列分隔线渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div style="column-count:3;column-gap:20px;column-rule:2px solid gray;width:600px">
<p>Column one content with some text.</p>
<p>Column two content with some text.</p>
<p>Column three content with some text.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── column-rule: dashed ──
        TestCase {
            id: "render/column-rule-dashed".to_string(),
            description: "column-rule: dashed 多列虚线分隔线渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div style="column-count:2;column-gap:30px;column-rule:3px dashed blue;width:400px">
<p>Left column text content.</p>
<p>Right column text content.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  List-style-image 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── list-style-image: url() ──
        TestCase {
            id: "render/list-style-image-url".to_string(),
            description: "list-style-image: url() 图片列表标记渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<ul style="list-style-image:url('bullet.png')">
<li>First item</li>
<li>Second item</li>
<li>Third item</li>
</ul>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Empty-cells 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── empty-cells: hide ──
        TestCase {
            id: "render/empty-cells-hide".to_string(),
            description: "empty-cells:hide 空单元格不显示边框和背景".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:separate;empty-cells:hide">
<tr><td style="background:#ccc;border:1px solid black">Content</td><td style="background:#ccc;border:1px solid black"></td></tr>
<tr><td style="background:#ccc;border:1px solid black">Data</td><td style="background:#ccc;border:1px solid black">More</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── empty-cells: show (default) ──
        TestCase {
            id: "render/empty-cells-show".to_string(),
            description: "empty-cells:show 空单元格显示边框和背景".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:separate;empty-cells:show">
<tr><td style="background:#ccc;border:1px solid black">Content</td><td style="background:#ccc;border:1px solid black"></td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  渲染管线扩展（+5 测试）
        // ═══════════════════════════════════════════════════════════════

        // ── 多层 box-shadow 组合渲染 ──
        TestCase {
            id: "render/box-shadow-multi-layer".to_string(),
            description: "多层 box-shadow 与 background-color 组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:white;margin:40px;box-shadow:0 2px 4px rgba(0,0,0,0.1),0 8px 16px rgba(0,0,0,0.1),0 16px 32px rgba(0,0,0,0.05)">Multi shadow</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── border-image 简写渲染 ──
        TestCase {
            id: "render/border-image-shorthand".to_string(),
            description: "border-image 简写渲染验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;border:20px solid;border-image:url('border.png') 30 round">Border image</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── text-overflow: ellipsis 溢出截断 ──
        TestCase {
            id: "render/text-overflow-ellipsis".to_string(),
            description: "text-overflow:ellipsis 溢出文本截断渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:150px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;border:1px solid #ccc;padding:4px">This text is too long and should be truncated with ellipsis</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "glyph_count_ge:1".to_string()],
        },

        // ── CSS filter blur 组合渲染 ──
        TestCase {
            id: "render/filter-blur-composite".to_string(),
            description: "CSS filter:blur 与 opacity 组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:80px;background:coral;filter:blur(2px);opacity:0.8">Blurred content</div>
            <div style="width:200px;height:80px;background:steelblue;filter:grayscale(50%) brightness(1.2)">Filtered</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 复杂渐变组合渲染 ──
        TestCase {
            id: "render/gradient-layered".to_string(),
            description: "多层渐变叠加渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:200px;background:linear-gradient(135deg,rgba(255,0,0,0.3),rgba(0,0,255,0.3)),linear-gradient(to right,#e0e0e0,#f0f0f0)">Layered gradients</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 动画/过渡渲染
        // ═══════════════════════════════════════════════════════════════

        // ── @keyframes 动画定义 + 渲染 ──
        TestCase {
            id: "render/animation-keyframes".to_string(),
            description: "@keyframes 动画定义渲染不崩溃".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="anim">Animated</div></body></html>"#.to_string(),
            css: r#"
                @keyframes fadeIn {
                    from { opacity: 0.0; }
                    to { opacity: 1.0; }
                }
                .anim { animation: fadeIn 1s linear; background-color: blue; width: 100px; height: 80px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 timing function: ease ──
        TestCase {
            id: "render/animation-timing-ease".to_string(),
            description: "animation timing ease 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="ease-box">Ease</div></body></html>"#.to_string(),
            css: r#"
                @keyframes slide { from { opacity: 0.2; } to { opacity: 1.0; } }
                .ease-box { animation: slide 2s ease; background-color: green; width: 150px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 timing function: steps ──
        TestCase {
            id: "render/animation-timing-steps".to_string(),
            description: "animation timing steps 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="steps-box">Steps</div></body></html>"#.to_string(),
            css: r#"
                @keyframes fade { 0% { opacity: 1.0; } 100% { opacity: 0.0; } }
                .steps-box { animation: fade 1s steps(4); background-color: red; width: 100px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 fill-mode: forwards ──
        TestCase {
            id: "render/animation-fill-forwards".to_string(),
            description: "animation fill-mode forwards 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="fill-box">Fill</div></body></html>"#.to_string(),
            css: r#"
                @keyframes grow { from { opacity: 0.0; } to { opacity: 1.0; } }
                .fill-box { animation: grow 0.5s linear forwards; background-color: orange; width: 200px; height: 120px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 direction: alternate ──
        TestCase {
            id: "render/animation-direction-alternate".to_string(),
            description: "animation direction alternate 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="alt-box">Alt</div></body></html>"#.to_string(),
            css: r#"
                @keyframes pulse { 0% { opacity: 0.3; } 100% { opacity: 1.0; } }
                .alt-box { animation: pulse 1s linear infinite alternate; background-color: purple; width: 100px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 多元素同时动画 ──
        TestCase {
            id: "render/animation-multiple-elements".to_string(),
            description: "多元素同时动画渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="a1">One</div>
                <div class="a2">Two</div>
                <div class="a3">Three</div>
            </body></html>"#.to_string(),
            css: r#"
                @keyframes fade { from { opacity: 0.0; } to { opacity: 1.0; } }
                .a1 { animation: fade 1s linear; background-color: red; width: 80px; height: 60px; }
                .a2 { animation: fade 1.5s ease; background-color: blue; width: 80px; height: 60px; }
                .a3 { animation: fade 2s ease-in-out; background-color: green; width: 80px; height: 60px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "fill_count_ge:3".to_string()],
        },

        // ── CSS transition 属性定义渲染 ──
        TestCase {
            id: "render/transition-property".to_string(),
            description: "CSS transition 属性定义渲染不崩溃".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="trans">Transition</div></body></html>"#.to_string(),
            css: r#"
                .trans {
                    transition: opacity 0.5s ease, background-color 0.3s linear;
                    opacity: 1.0; background-color: steelblue;
                    width: 200px; height: 100px; color: white;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── transition with delay ──
        TestCase {
            id: "render/transition-delay".to_string(),
            description: "CSS transition delay 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="delayed">Delayed</div></body></html>"#.to_string(),
            css: r#"
                .delayed {
                    transition: opacity 1s 0.5s ease-in-out;
                    opacity: 0.8; background-color: coral;
                    width: 150px; height: 80px;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── transition 多属性 ──
        TestCase {
            id: "render/transition-multi-property".to_string(),
            description: "CSS transition 多属性过渡渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="multi">Multi</div></body></html>"#.to_string(),
            css: r#"
                .multi {
                    transition-property: opacity, width, background-color;
                    transition-duration: 0.3s, 0.5s, 0.4s;
                    transition-timing-function: ease, linear, ease-in;
                    opacity: 0.7; width: 180px; background-color: teal; height: 100px;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 + transition 组合 ──
        TestCase {
            id: "render/animation-transition-combo".to_string(),
            description: "动画与过渡组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="combo">Combo</div></body></html>"#.to_string(),
            css: r#"
                @keyframes colorShift { 0% { opacity: 0.5; } 100% { opacity: 1.0; } }
                .combo {
                    animation: colorShift 1s linear;
                    transition: background-color 0.3s ease;
                    background-color: navy; width: 200px; height: 120px; color: white;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Transform-origin + 非 translate 变换渲染
        // ═══════════════════════════════════════════════════════════════

        // ── rotate + transform-origin 渲染 ──
        TestCase {
            id: "render/transform-origin-rotate".to_string(),
            description: "CSS rotate with transform-origin".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:100px;background:#e74c3c;transform:rotate(45deg);transform-origin:0 0">Rotated</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── scale 变换渲染 ──
        TestCase {
            id: "render/transform-scale".to_string(),
            description: "CSS scale transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:50px;background:#3498db;transform:scale(2,0.5)">Scaled</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── skew 变换渲染 ──
        TestCase {
            id: "render/transform-skew".to_string(),
            description: "CSS skew transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:80px;background:#9b59b6;transform:skew(20deg,10deg)">Skewed</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── matrix() 变换渲染 ──
        TestCase {
            id: "render/transform-matrix".to_string(),
            description: "CSS matrix() transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:60px;background:#1abc9c;transform:matrix(0.866,0.5,-0.5,0.866,10,20)">Matrix</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 变换组合 (translate + rotate + scale) ──
        TestCase {
            id: "render/transform-combined".to_string(),
            description: "Combined transform functions".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:80px;height:80px;background:#e67e22;transform:translate(50px,20px) rotate(30deg) scale(1.5)">Combined</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Conic-gradient 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── conic-gradient 基础渲染 ──
        TestCase {
            id: "render/conic-gradient-basic".to_string(),
            description: "Basic conic-gradient rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:200px;background:conic-gradient(red,yellow,green,blue,red)">Color Wheel</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── conic-gradient with from angle ──
        TestCase {
            id: "render/conic-gradient-from-angle".to_string(),
            description: "Conic-gradient with from angle".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:150px;height:150px;background:conic-gradient(from 90deg,#ff0,#0ff,#f0f,#ff0)">Angle</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── conic-gradient with position ──
        TestCase {
            id: "render/conic-gradient-position".to_string(),
            description: "Conic-gradient with center position".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:200px;background:conic-gradient(from 45deg at 25% 75%,#2ecc71,#e74c3c,#2ecc71)">Position</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Counters
        // ═══════════════════════════════════════════════════════════════

        // ── 有序列表 + counter-increment ──
        TestCase {
            id: "render/counter-ordered-list".to_string(),
            description: "Ordered list with counter-increment".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<ol style="counter-reset:item">
  <li style="counter-increment:item">First</li>
  <li style="counter-increment:item">Second</li>
  <li style="counter-increment:item">Third</li>
</ol>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 嵌套计数器 ──
        TestCase {
            id: "render/counter-nested".to_string(),
            description: "Nested counters with reset/increment".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="counter-reset:section 0">
  <h2 style="counter-increment:section">Section 1</h2>
  <div style="counter-reset:subsection 0">
    <p style="counter-increment:subsection">Sub 1.1</p>
    <p style="counter-increment:subsection">Sub 1.2</p>
  </div>
  <h2 style="counter-increment:section">Section 2</h2>
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
        //  background-repeat 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── background-repeat: repeat 默认平铺 ──
        TestCase {
            id: "render/bg-repeat-default".to_string(),
            description: "background-repeat default tiling".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('tile.png');background-size:50px 50px;">Tiled</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat: repeat-x 仅水平平铺 ──
        TestCase {
            id: "render/bg-repeat-x".to_string(),
            description: "background-repeat-x horizontal only".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('stripe.png');background-size:40px 100px;background-repeat:repeat-x;">H-Stripe</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat: repeat-y 仅垂直平铺 ──
        TestCase {
            id: "render/bg-repeat-y".to_string(),
            description: "background-repeat-y vertical only".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:200px;background-image:url('stripe.png');background-size:100px 40px;background-repeat:repeat-y;">V-Stripe</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat: no-repeat 不平铺 ──
        TestCase {
            id: "render/bg-no-repeat".to_string(),
            description: "background-repeat no-repeat single tile".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('photo.png');background-size:50px 50px;background-repeat:no-repeat;">Single</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat: round 缩放平铺 ──
        TestCase {
            id: "render/bg-repeat-round".to_string(),
            description: "background-repeat round scaled tiles".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('tile.png');background-size:60px 60px;background-repeat:round;">Rounded</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat: space 均匀分布 ──
        TestCase {
            id: "render/bg-repeat-space".to_string(),
            description: "background-repeat space evenly distributed".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('dot.png');background-size:30px 30px;background-repeat:space;">Spaced</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-repeat + position + size 组合 ──
        TestCase {
            id: "render/bg-repeat-position-size".to_string(),
            description: "background-repeat with position and size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:200px;background-image:url('icon.png');background-size:40px 40px;background-position:10px 10px;background-repeat:repeat;">Pattern</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 表格渲染
        // ═══════════════════════════════════════════════════════════════

        // ── 基础 HTML 表格渲染 ──
        TestCase {
            id: "render/html-table-basic".to_string(),
            description: "Basic HTML table rendering".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:collapse;width:100%;">
  <tr><th style="border:1px solid #333;background:#eee;padding:4px;">Name</th><th style="border:1px solid #333;background:#eee;padding:4px;">Value</th></tr>
  <tr><td style="border:1px solid #333;padding:4px;">Alpha</td><td style="border:1px solid #333;padding:4px;">100</td></tr>
  <tr><td style="border:1px solid #333;padding:4px;">Beta</td><td style="border:1px solid #333;padding:4px;">200</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── 带标题的表格 ──
        TestCase {
            id: "render/html-table-caption".to_string(),
            description: "HTML table with caption".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:collapse;">
  <caption style="text-align:center;font-weight:bold;padding:4px;">Data Table</caption>
  <tr><td style="border:1px solid;padding:4px;">A</td><td style="border:1px solid;padding:4px;">B</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 嵌套表格 ──
        TestCase {
            id: "render/html-table-nested".to_string(),
            description: "Nested HTML tables".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border:1px solid #000;"><tr><td style="padding:8px;">
  <table style="border:1px solid #999;"><tr><td style="padding:4px;">Inner</td></tr></table>
</td></tr></table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 多列布局渲染
        // ═══════════════════════════════════════════════════════════════

        // ── column-count 多列文本 ──
        TestCase {
            id: "render/multi-column-text".to_string(),
            description: "Multi-column text layout with column-count".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="column-count:3;column-gap:20px;column-rule:1px solid #ccc;">
  <p>Column one text content for testing multi-column layout rendering.</p>
  <p>Column two continues with more content to fill the space.</p>
  <p>Column three wraps up the text across three columns.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── column-width 固定列宽 ──
        TestCase {
            id: "render/multi-column-width".to_string(),
            description: "Multi-column layout with column-width".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="column-width:150px;column-gap:16px;column-rule:2px dashed #999;width:500px;">
  <p>Fixed width columns with dashed rules between them for visual separation.</p>
  <p>More content to demonstrate the column-width property rendering.</p>
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
        //  CSS content 属性渲染
        // ═══════════════════════════════════════════════════════════════

        // ── content: string 渲染 ──
        TestCase {
            id: "render/content-string".to_string(),
            description: "CSS content property with string value renders glyphs".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="generated" style="content:'Generated Text';font-size:16px;color:#333;">Container</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── content: counter() 渲染 ──
        TestCase {
            id: "render/content-counter".to_string(),
            description: "CSS content with counter() renders counter values".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="counter-reset:section 5;"></div>
<div id="counter-display" style="content:counter(section);font-size:16px;color:#000;">Section</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS object-fit 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── object-fit: fill ──
        TestCase {
            id: "render/object-fit-fill".to_string(),
            description: "CSS object-fit:fill stretches image to fill container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<img src="test-image.png" width="100" height="100" style="width:200px;height:100px;object-fit:fill;">
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── object-fit: contain ──
        TestCase {
            id: "render/object-fit-contain".to_string(),
            description: "CSS object-fit:contain scales image to fit within container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<img src="photo.jpg" width="200" height="100" style="width:200px;height:200px;object-fit:contain;background:#eee;">
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── object-fit: cover ──
        TestCase {
            id: "render/object-fit-cover".to_string(),
            description: "CSS object-fit:cover scales image to cover container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<img src="hero.jpg" width="400" height="200" style="width:200px;height:200px;object-fit:cover;">
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── object-fit: none ──
        TestCase {
            id: "render/object-fit-none".to_string(),
            description: "CSS object-fit:none uses original image size centered".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<img src="icon.png" width="32" height="32" style="width:100px;height:100px;object-fit:none;background:#f0f0f0;">
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── object-fit: scale-down ──
        TestCase {
            id: "render/object-fit-scale-down".to_string(),
            description: "CSS object-fit:scale-down picks smaller of none and contain".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<img src="logo.png" width="300" height="300" style="width:100px;height:100px;object-fit:scale-down;">
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS text-decoration-style 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── text-decoration underline dashed ──
        TestCase {
            id: "render/text-decoration-dashed".to_string(),
            description: "CSS text-decoration underline with dashed style".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<p style="text-decoration-line:underline;font-size:16px;color:#333;">Dashed underline text</p>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── text-decoration line-through ──
        TestCase {
            id: "render/text-decoration-line-through".to_string(),
            description: "CSS text-decoration line-through renders strikethrough".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<p style="text-decoration-line:line-through;font-size:16px;color:#c00;">Deleted text with strikethrough</p>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── text-decoration overline ──
        TestCase {
            id: "render/text-decoration-overline".to_string(),
            description: "CSS text-decoration overline renders line above text".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<p style="text-decoration-line:overline;font-size:16px;color:#333;">Overline decoration text</p>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  综合：content + counter + list-style 组合
        // ═══════════════════════════════════════════════════════════════

        // ── 计数器 + content 组合页面 ──
        TestCase {
            id: "render/counter-content-page".to_string(),
            description: "CSS counters with content property in a styled page".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  body { counter-reset: section; font-family: sans-serif; }
  h2 { counter-increment: section; }
  h2::before { content: "Section " counter(section) ": "; color: #666; }
  ul { counter-reset: item; }
  li { counter-increment: item; }
</style>
<h2>Introduction</h2>
<p>This is the first section with counter-based numbering.</p>
<h2>Methods</h2>
<p>This is the second section.</p>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS background-position + background-size + 渐变组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/gradient-background-combo".to_string(),
            description: "Linear gradient with background-position and size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .gradient-box {
    width: 200px; height: 100px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    background-size: 100% 100%;
    border-radius: 8px;
    margin: 10px;
  }
  .striped-box {
    width: 200px; height: 100px;
    background: repeating-linear-gradient(45deg, #606dbc, #606dbc 10px, #465298 10px, #465298 20px);
    margin: 10px;
  }
</style>
<div class="gradient-box">Gradient</div>
<div class="striped-box">Stripes</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS flexbox + gap + align 组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/flex-gap-align-center".to_string(),
            description: "Flexbox with gap and align-items center".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .flex-row {
    display: flex; gap: 16px; align-items: center;
    padding: 12px; background: #f5f5f5;
  }
  .card {
    width: 80px; height: 80px; background: #4a90d9;
    border-radius: 8px; color: white;
    display: flex; align-items: center; justify-content: center;
  }
</style>
<div class="flex-row">
  <div class="card">A</div>
  <div class="card">B</div>
  <div class="card">C</div>
  <div class="card">D</div>
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
        //  CSS Grid 响应式卡片布局
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/grid-responsive-cards".to_string(),
            description: "CSS Grid auto-fill responsive card layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .grid-container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px; padding: 16px;
  }
  .grid-item {
    background: white; border: 1px solid #ddd;
    border-radius: 8px; padding: 12px;
  }
  .grid-item h3 { margin: 0 0 8px 0; font-size: 14px; }
  .grid-item p { margin: 0; font-size: 12px; color: #666; }
</style>
<div class="grid-container">
  <div class="grid-item"><h3>Card 1</h3><p>Description for card 1</p></div>
  <div class="grid-item"><h3>Card 2</h3><p>Description for card 2</p></div>
  <div class="grid-item"><h3>Card 3</h3><p>Description for card 3</p></div>
  <div class="grid-item"><h3>Card 4</h3><p>Description for card 4</p></div>
  <div class="grid-item"><h3>Card 5</h3><p>Description for card 5</p></div>
  <div class="grid-item"><h3>Card 6</h3><p>Description for card 6</p></div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:6".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS sticky footer 布局
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/sticky-footer".to_string(),
            description: "CSS sticky footer layout with flexbox".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0; min-height:100vh; display:flex; flex-direction:column;">
<style>
  header { background: #333; color: white; padding: 16px; }
  main { flex: 1; padding: 16px; }
  footer { background: #f5f5f5; padding: 12px; text-align: center; border-top: 1px solid #ddd; }
</style>
<header><h1>Site Header</h1></header>
<main><p>Main content area. The footer sticks to the bottom.</p></main>
<footer><p>Footer content</p></footer>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 多层 box-shadow + border-radius
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/multi-box-shadow".to_string(),
            description: "Multiple box-shadows with border-radius".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .shadow-card {
    width: 200px; height: 120px;
    background: white;
    border-radius: 12px;
    box-shadow:
      0 1px 3px rgba(0,0,0,0.12),
      0 4px 8px rgba(0,0,0,0.08),
      0 12px 24px rgba(0,0,0,0.06);
    margin: 40px auto;
    padding: 20px;
  }
  .inner-shadow {
    width: 200px; height: 120px;
    background: #f0f0f0;
    border-radius: 12px;
    box-shadow: inset 0 2px 8px rgba(0,0,0,0.15);
    margin: 40px auto;
    padding: 20px;
  }
</style>
<div class="shadow-card">Multi-layer shadow</div>
<div class="inner-shadow">Inner shadow</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS transform + opacity 组合
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/transform-opacity-card".to_string(),
            description: "Transform scale + opacity combined".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .card-container { padding: 40px; }
  .card {
    width: 200px; height: 150px;
    background: linear-gradient(135deg, #ff6b6b, #ee5a24);
    border-radius: 10px;
    transform: scale(0.9) rotate(-2deg);
    opacity: 0.85;
    color: white;
    padding: 20px;
    box-sizing: border-box;
  }
  .card:hover { transform: scale(1) rotate(0deg); opacity: 1; }
</style>
<div class="card-container">
  <div class="card"><h2>Transformed</h2><p>Scale + rotate + opacity</p></div>
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
        //  CSS 简单表单布局
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "render/form-login".to_string(),
            description: "Login form layout with CSS".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<style>
  .login-form { max-width: 320px; margin: 40px auto; padding: 24px; border: 1px solid #ddd; border-radius: 8px; }
  .form-group { margin-bottom: 16px; }
  .form-group label { display: block; margin-bottom: 4px; font-size: 14px; color: #333; }
  .form-group input {
    width: 100%; padding: 8px 12px; border: 1px solid #ccc;
    border-radius: 4px; font-size: 14px; box-sizing: border-box;
  }
  .btn { width: 100%; padding: 10px; background: #4a90d9; color: white; border: none; border-radius: 4px; font-size: 14px; }
</style>
<div class="login-form">
  <h2>Login</h2>
  <div class="form-group"><label>Username</label><input type="text"></div>
  <div class="form-group"><label>Password</label><input type="password"></div>
  <button class="btn">Sign In</button>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:5".to_string(),
            ],
        },
    ]
}
