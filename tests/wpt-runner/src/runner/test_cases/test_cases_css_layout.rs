//! CSS 布局高级特性标准合规性测试。
//!
//! 覆盖：
//! - CSS Grid 高级特性（grid-template-rows/columns with repeat/auto-fill/auto-fit, grid-area 命名定位, 嵌套网格, grid 间距）
//! - CSS Flexbox 高级特性（flex-wrap wrap-reverse, order 负值, align-content space-between/space-around, flex-basis auto vs 0, 嵌套 flex 容器）
//! - CSS 定位（position:sticky 模拟, fixed 定位配合 transform, relative 容器链中的 absolute, z-index 多上下文堆叠, position:absolute margin:auto 居中）
//! - CSS 盒模型边缘情况（box-sizing:border-box 配合 padding, 负边距折叠, 百分比宽度配合 padding, max-width 约束, min-height 配合 overflow）
//! - CSS 文本和溢出（text-overflow:ellipsis, overflow:hidden 剪切, white-space:nowrap, word-break:break-all, text-align:justify 多行文本）

use super::TestCase;

/// 返回 CSS 布局高级特性相关的测试用例。
pub fn css_layout_compliance_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  CSS Grid 高级特性
        // ═══════════════════════════════════════════════════════════════

        // ── grid-template-columns with repeat ──
        TestCase {
            id: "css-layout/grid-repeat-auto-fill".to_string(),
            description: "grid-template-columns 使用 repeat 和 auto-fill".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid"><div class="item">1</div><div class="item">2</div><div class="item">3</div><div class="item">4</div></div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); gap: 10px; width: 500px; }
                     .item { height: 100px; background: #ccc; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── grid-template-rows with auto-fit ──
        TestCase {
            id: "css-layout/grid-auto-fit-rows".to_string(),
            description: "grid-template-rows 使用 repeat 和 auto-fit".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid"><div class="item">Item 1</div><div class="item">Item 2<br>Line 2</div><div class="item">Item 3</div></div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-rows: repeat(auto-fit, minmax(50px, 1fr)); gap: 5px; }
                     .item { padding: 10px; background: #e0e0e0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── grid-area named placement ──
        TestCase {
            id: "css-layout/grid-area-named".to_string(),
            description: "grid-area 命名定位".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid"><div class="header">Header</div><div class="sidebar">Sidebar</div><div class="main">Main Content</div><div class="footer">Footer</div></div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-areas: "header header" "sidebar main" "footer footer"; grid-template-rows: 60px 1fr 50px; grid-template-columns: 200px 1fr; gap: 5px; height: 500px; }
                     .header { grid-area: header; background: #4a90e2; }
                     .sidebar { grid-area: sidebar; background: #f0f0f0; }
                     .main { grid-area: main; background: #ffffff; }
                     .footer { grid-area: footer; background: #333; color: white; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── nested grids ──
        TestCase {
            id: "css-layout/nested-grids".to_string(),
            description: "嵌套网格布局".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="outer-grid"><div class="outer-item">Outer 1</div><div class="inner-container"><div class="inner-grid"><div class="inner-item">Inner 1</div><div class="inner-item">Inner 2</div></div></div><div class="outer-item">Outer 2</div></div></body></html>"#.to_string(),
            css: r#".outer-grid { display: grid; grid-template-columns: 1fr 2fr 1fr; gap: 10px; }
                     .outer-item { background: #4a90e2; color: white; padding: 20px; }
                     .inner-container { padding: 10px; }
                     .inner-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 5px; }
                     .inner-item { background: #e74c3c; color: white; padding: 15px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── grid with gap ──
        TestCase {
            id: "css-layout/grid-with-gap".to_string(),
            description: "grid 布局配合 gap 间距".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid"><div class="item">1</div><div class="item">2</div><div class="item">3</div><div class="item">4</div><div class="item">5</div><div class="item">6</div></div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-columns: repeat(3, 1fr); grid-template-rows: repeat(2, 100px); gap: 20px; }
                     .item { background: linear-gradient(45deg, #ff6b6b, #4ecdc4); color: white; display: flex; align-items: center; justify-content: center; font-size: 24px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Flexbox 高级特性
        // ═══════════════════════════════════════════════════════════════

        // ── flex-wrap wrap-reverse ──
        TestCase {
            id: "css-layout/flex-wrap-reverse".to_string(),
            description: "flex-wrap: wrap-reverse".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex-container"><div class="item">Item 1</div><div class="item">Item 2</div><div class="item">Item 3</div><div class="item">Item 4</div><div class="item">Item 5</div><div class="item">Item 6</div></div></body></html>"#.to_string(),
            css: r#".flex-container { display: flex; flex-wrap: wrap-reverse; width: 300px; height: 400px; border: 2px solid #333; }
                     .item { width: 100px; height: 50px; background: #4ecdc4; color: white; display: flex; align-items: center; justify-content: center; margin: 5px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── order with negative values ──
        TestCase {
            id: "css-layout/order-negative".to_string(),
            description: "order 属性使用负值".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex-container"><div class="item" style="order: -1">First (order -1)</div><div class="item" style="order: 1">Last (order 1)</div><div class="item" style="order: 0">Middle (order 0)</div></div></body></html>"#.to_string(),
            css: r#".flex-container { display: flex; gap: 10px; }
                     .item { padding: 20px; background: #ff6b6b; color: white; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── align-content space-between ──
        TestCase {
            id: "css-layout/align-content-between".to_string(),
            description: "align-content: space-between".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex-container"><div class="item">Item 1</div><div class="item">Item 2</div><div class="item">Item 3</div></div></body></html>"#.to_string(),
            css: r#".flex-container { display: flex; flex-wrap: wrap; height: 300px; align-content: space-between; border: 2px solid #333; }
                     .item { width: 100px; height: 80px; background: #4ecdc4; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── flex-basis auto vs 0 ──
        TestCase {
            id: "css-layout/flex-basis-comparison".to_string(),
            description: "flex-basis: auto vs 0 对比".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="auto-case">
                    <h3>flex-basis: auto</h3>
                    <div class="flex-auto"><div class="item">Content</div></div>
                </div>
                <div class="zero-case">
                    <h3>flex-basis: 0</h3>
                    <div class="flex-zero"><div class="item">Content</div></div>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".container { display: flex; gap: 20px; }
                     .flex-auto, .flex-zero { display: flex; width: 150px; }
                     .flex-auto div { flex: 1 1 auto; }
                     .flex-zero div { flex: 1 1 0; }
                     .item { background: #ff6b6b; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── nested flex containers ──
        TestCase {
            id: "css-layout/nested-flex".to_string(),
            description: "嵌套 flex 容器".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="outer-flex">
                <div class="outer-item">Top</div>
                <div class="inner-flex">
                    <div class="inner-item">Inner 1</div>
                    <div class="inner-item">Inner 2</div>
                </div>
                <div class="outer-item">Bottom</div>
            </div></body></html>"#.to_string(),
            css: r#".outer-flex { display: flex; flex-direction: column; height: 300px; }
                     .outer-item { background: #4a90e2; color: white; padding: 10px; text-align: center; }
                     .inner-flex { display: flex; flex: 1; }
                     .inner-item { flex: 1; background: #e74c3c; color: white; margin: 5px; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 定位
        // ═══════════════════════════════════════════════════════════════

        // ── position:sticky simulation ──
        TestCase {
            id: "css-layout/position-sticky".to_string(),
            description: "position:sticky 模拟".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div style="height: 2000px; background: #f0f0f0;">
                <div style="top: 0; position: sticky; background: #ff6b6b; padding: 10px;">Sticky Header</div>
                <p>Scroll down to see the sticky behavior</p><br><br><br><br><br><br><br><br><br>
                <p>More content...</p><br><br><br><br><br><br><br><br><br>
                <p>Even more content...</p><br><br><br><br><br><br><br><br><br>
                <p>Bottom content</p>
            </div></body></html>"#.to_string(),
            css: r#"p { margin: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── fixed positioning with transforms ──
        TestCase {
            id: "css-layout/fixed-with-transform".to_string(),
            description: "fixed 定位配合 transform".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div style="height: 2000px;">
                <div style="position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: #4ecdc4; padding: 20px; color: white;">Fixed Center</div>
                <p>Scroll to see the fixed element</p><br><br><br><br><br><br><br><br><br>
                <p>More content...</p><br><br><br><br><br><br><br><br><br>
            </div></body></html>"#.to_string(),
            css: r#"p { margin: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── absolute in relative container chains ──
        TestCase {
            id: "css-layout/absolute-relative-chains".to_string(),
            description: "relative 容器链中的 absolute 定位".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <div class="grandparent">
                    <div class="parent">
                        <div class="child">Child</div>
                    </div>
                    <div class="sibling">Sibling</div>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".grandparent { position: relative; width: 400px; height: 400px; background: #f0f0f0; }
                     .parent { position: relative; width: 200px; height: 200px; background: #e0e0e0; }
                     .child { position: absolute; top: 20px; left: 20px; width: 50px; height: 50px; background: #ff6b6b; }
                     .sibling { width: 100px; height: 100px; background: #4ecdc4; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── z-index stacking with multiple contexts ──
        TestCase {
            id: "css-layout/z-index-multiple-contexts".to_string(),
            description: "z-index 多上下文堆叠".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <div class="context-1">
                    <div class="box" style="z-index: 1;">Box 1 (z-index: 1)</div>
                    <div class="box" style="z-index: 3;">Box 3 (z-index: 3)</div>
                    <div class="box" style="z-index: 2;">Box 2 (z-index: 2)</div>
                </div>
                <div class="context-2">
                    <div class="box" style="z-index: 2;">Box 4 (z-index: 2)</div>
                    <div class="box" style="z-index: 1;">Box 5 (z-index: 1)</div>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".context-1, .context-2 { position: relative; width: 300px; height: 200px; margin: 20px; }
                     .context-1 { background: #e0e0e0; }
                     .context-2 { background: #f0f0f0; }
                     .box { position: absolute; width: 80px; height: 80px; color: white; display: flex; align-items: center; justify-content: center; }
                     .context-1 .box:nth-child(1) { background: #ff6b6b; top: 10px; left: 10px; }
                     .context-1 .box:nth-child(2) { background: #4ecdc4; top: 60px; left: 60px; }
                     .context-1 .box:nth-child(3) { background: #45b7d1; top: 30px; left: 120px; }
                     .context-2 .box:nth-child(1) { background: #f39c12; top: 50px; left: 20px; }
                     .context-2 .box:nth-child(2) { background: #9b59b6; top: 20px; left: 150px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── position:absolute with margin:auto centering ──
        TestCase {
            id: "css-layout/absolute-margin-auto".to_string(),
            description: "position:absolute with margin:auto 居中".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="parent">
                <div class="centered">Centered Box</div>
                <div class="other">Other Content</div>
            </div></body></html>"#.to_string(),
            css: r#".parent { position: relative; width: 400px; height: 300px; background: #f0f0f0; }
                     .centered { position: absolute; top: 0; bottom: 0; left: 0; right: 0; margin: auto; width: 200px; height: 100px; background: #ff6b6b; color: white; display: flex; align-items: center; justify-content: center; }
                     .other { position: absolute; top: 20px; left: 20px; width: 50px; height: 50px; background: #4ecdc4; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 盒模型边缘情况
        // ═══════════════════════════════════════════════════════════════

        // ── box-sizing:border-box with padding ──
        TestCase {
            id: "css-layout/box-sizing-border-box".to_string(),
            description: "box-sizing:border-box 配合 padding".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="box border-box">Box with border-box</div>
                <div class="box content-box">Box with content-box</div>
            </div></body></html>"#.to_string(),
            css: r#".container { display: flex; gap: 20px; }
                     .box { width: 200px; padding: 20px; background: #4ecdc4; color: white; }
                     .border-box { box-sizing: border-box; height: 100px; }
                     .content-box { box-sizing: content-box; height: 100px; background: #ff6b6b; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── negative margins collapsing ──
        TestCase {
            id: "css-layout/negative-margins".to_string(),
            description: "负边距折叠效果".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="positive-margin">Positive Margin</div>
                <div class="negative-margin">Negative Margin (-20px)</div>
                <div class="negative-margin-2">Negative Margin (-40px)</div>
                <div class="positive-margin-2">Positive Margin (+20px)</div>
            </div></body></html>"#.to_string(),
            css: r#".container { border: 1px solid #333; width: 300px; }
                     .positive-margin { margin: 20px 0; background: #4ecdc4; color: white; }
                     .negative-margin { margin: -20px 0; background: #ff6b6b; color: white; }
                     .negative-margin-2 { margin: -40px 0; background: #f39c12; color: white; }
                     .positive-margin-2 { margin: 20px 0; background: #9b59b6; color: white; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── percentage widths with padding ──
        TestCase {
            id: "css-layout/percentage-widths-padding".to_string(),
            description: "百分比宽度配合 padding".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="parent">
                <div class="child">Child with 100% width and padding</div>
                <div class="child-no-padding">Child with 100% width, no padding</div>
            </div></body></html>"#.to_string(),
            css: r#".parent { width: 400px; border: 2px solid #333; }
                     .child { width: 100%; padding: 20px; background: #4ecdc4; color: white; }
                     .child-no-padding { width: 100%; background: #ff6b6b; color: white; height: 50px; margin-top: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── max-width constraints ──
        TestCase {
            id: "css-layout/max-width-constraints".to_string(),
            description: "max-width 约束效果".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="no-constraint">No max-width constraint (width: 100%)</div>
                <div class="with-constraint">With max-width: 300px</div>
                <div class="narrow-constraint">With max-width: 150px</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 500px; border: 1px solid #333; }
                     .no-constraint { width: 100%; background: #4ecdc4; color: white; padding: 10px; }
                     .with-constraint { width: 100%; max-width: 300px; background: #ff6b6b; color: white; padding: 10px; margin-top: 10px; }
                     .narrow-constraint { width: 100%; max-width: 150px; background: #f39c12; color: white; padding: 10px; margin-top: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── min-height with overflow ──
        TestCase {
            id: "css-layout/min-height-overflow".to_string(),
            description: "min-height 配合 overflow".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="min-height-box">
                    <p>This box has min-height: 150px</p>
                    <p>Content that may cause overflow...</p>
                    <p>More content here...</p>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 300px; }
                     .min-height-box { min-height: 150px; overflow: auto; border: 2px solid #333; background: #4ecdc4; color: white; padding: 10px; }
                     p { margin: 10px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 文本和溢出
        // ═══════════════════════════════════════════════════════════════

        // ── text-overflow:ellipsis ──
        TestCase {
            id: "css-layout/text-ellipsis".to_string(),
            description: "text-overflow:ellipsis 省略号效果".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="ellipsis">This is a long text that should be truncated with ellipsis</div>
                <div class="no-ellipsis">This is a long text without truncation</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 200px; }
                     .ellipsis { width: 150px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; background: #4ecdc4; color: white; padding: 5px; }
                     .no-ellipsis { width: 150px; background: #ff6b6b; color: white; padding: 5px; margin-top: 5px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── overflow:hidden clipping ──
        TestCase {
            id: "css-layout/overflow-hidden".to_string(),
            description: "overflow:hidden 剪切效果".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="overflow-hidden">
                    <p>Line 1</p>
                    <p>Line 2</p>
                    <p>Line 3 - This should be clipped</p>
                    <p>Line 4 - Also clipped</p>
                    <p>Line 5 - Still clipped</p>
                </div>
                <div class="visible">
                    <p>Line 1</p>
                    <p>Line 2</p>
                    <p>Line 3</p>
                    <p>Line 4</p>
                    <p>Line 5</p>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 200px; }
                     .overflow-hidden { height: 100px; overflow: hidden; border: 2px solid #333; background: #4ecdc4; color: white; }
                     .visible { height: 100px; border: 2px solid #333; background: #ff6b6b; color: white; margin-top: 10px; }
                     p { margin: 5px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── white-space:nowrap ──
        TestCase {
            id: "css-layout/white-space-nowrap".to_string(),
            description: "white-space:nowrap 不换行".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="nowrap">This text should not wrap and extend beyond the container</div>
                <div class="normal-wrap">This text should normally wrap within the container</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 300px; border: 1px solid #333; }
                     .nowrap { white-space: nowrap; background: #4ecdc4; color: white; padding: 10px; }
                     .normal-wrap { background: #ff6b6b; color: white; padding: 10px; margin-top: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── word-break:break-all ──
        TestCase {
            id: "css-layout/word-break-break-all".to_string(),
            description: "word-break:break-all 单词强制断行".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="break-all">Thisisareallylongwordthatshouldbreakatanycharacterwithinthecontainerwidth</div>
                <div class="normal-break">This is a normal break text that should wrap normally</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 200px; border: 1px solid #333; }
                     .break-all { word-break: break-all; background: #4ecdc4; color: white; padding: 10px; }
                     .normal-break { background: #ff6b6b; color: white; padding: 10px; margin-top: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── text-align:justify with multiple lines ──
        TestCase {
            id: "css-layout/text-align-justify".to_string(),
            description: "text-align:justify 多行文本对齐".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="justify">This is the first line of justified text. It should stretch to fill the container width. This is the second line that should also be fully justified. And here is the final line to complete the justification.</div>
                <div class="left">This is left-aligned text. It should not be justified.</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 300px; border: 1px solid #333; }
                     .justify { text-align: justify; background: #4ecdc4; color: white; padding: 10px; margin-bottom: 10px; }
                     .left { text-align: left; background: #ff6b6b; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Grid 高级特性 - 额外测试
        // ═══════════════════════════════════════════════════════════════

        // ── grid-template-columns with minmax ──
        TestCase {
            id: "css-layout/grid-minmax".to_string(),
            description: "grid-template-columns 使用 minmax 函数".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid">
                <div class="item">Item 1</div>
                <div class="item">Item 2</div>
                <div class="item">Item 3</div>
                <div class="item">Item 4</div>
            </div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-columns: minmax(100px, 1fr) minmax(150px, 2fr) repeat(2, minmax(80px, 1fr)); gap: 10px; }
                     .item { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── grid with auto-flow row ──
        TestCase {
            id: "css-layout/grid-auto-flow-row".to_string(),
            description: "grid 使用 auto-flow row 自动排列".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid">
                <div class="item">1</div>
                <div class="item">2</div>
                <div class="item">3</div>
                <div class="item">4</div>
                <div class="item">5</div>
                <div class="item">6</div>
                <div class="item">7</div>
                <div class="item">8</div>
            </div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-auto-flow: row; grid-template-columns: repeat(3, 1fr); grid-auto-rows: 80px; gap: 15px; }
                     .item { background: #4ecdc4; color: white; display: flex; align-items: center; justify-content: center; font-size: 24px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Flexbox 高级特性 - 额外测试
        // ═══════════════════════════════════════════════════════════════

        // ── align-content space-around ──
        TestCase {
            id: "css-layout/align-content-around".to_string(),
            description: "align-content: space-around".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex-container">
                <div class="item">Item 1</div>
                <div class="item">Item 2</div>
                <div class="item">Item 3</div>
            </div></body></html>"#.to_string(),
            css: r#".flex-container { display: flex; flex-wrap: wrap; height: 300px; align-content: space-around; border: 2px solid #333; }
                     .item { width: 100px; height: 80px; background: #ff6b6b; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── flex-grow with flex-basis ──
        TestCase {
            id: "css-layout/flex-grow-basis".to_string(),
            description: "flex-grow 配合 flex-basis".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex-container">
                <div class="item-grow">flex-grow: 2</div>
                <div class="item-grow">flex-grow: 2</div>
                <div class="item-grow-0">flex-grow: 1 (basis: 100px)</div>
                <div class="item-grow">flex-grow: 2</div>
            </div></body></html>"#.to_string(),
            css: r#".flex-container { display: flex; gap: 10px; }
                     .item-grow { flex: 1 1 0; background: #4ecdc4; color: white; padding: 20px; }
                     .item-grow-0 { flex: 1 1 100px; background: #ff6b6b; color: white; padding: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 定位 - 额外测试
        // ═══════════════════════════════════════════════════════════════

        // ── sticky with offset ──
        TestCase {
            id: "css-layout/sticky-with-offset".to_string(),
            description: "position:sticky 配合 offset".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div style="height: 2000px; background: #f0f0f0;">
                <div style="top: 10px; position: sticky; background: #4ecdc4; padding: 10px; color: white;">Sticky with top: 10px</div>
                <p>Scroll down to see sticky behavior with offset</p><br><br><br><br><br><br><br><br><br>
                <p>More content...</p><br><br><br><br><br><br><br><br><br>
                <p>Even more content...</p><br><br><br><br><br><br><br><br><br>
                <p>Bottom content</p>
            </div></body></html>"#.to_string(),
            css: r#"p { margin: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── fixed with inset properties ──
        TestCase {
            id: "css-layout/fixed-inset".to_string(),
            description: "position:fixed 配合 inset 属性".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div style="height: 2000px;">
                <div style="position: fixed; inset: 20px; background: #ff6b6b; padding: 20px; color: white;">Fixed with inset</div>
                <p>Scroll to see the fixed element with inset</p><br><br><br><br><br><br><br><br><br>
                <p>More content...</p><br><br><br><br><br><br><br><br><br>
            </div></body></html>"#.to_string(),
            css: r#"p { margin: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 盒模型边缘情况 - 额外测试
        // ═══════════════════════════════════════════════════════════════

        // ─── vertical padding with percentage ──
        TestCase {
            id: "css-layout/vertical-percentage-padding".to_string(),
            description: "垂直方向的百分比 padding".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="parent">
                <div class="child">Child with 20% padding</div>
            </div></body></html>"#.to_string(),
            css: r#".parent { width: 300px; }
                     .child { height: 200px; padding: 20%; background: #4ecdc4; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── box model calculations ──
        TestCase {
            id: "css-layout/box-model-calculations".to_string(),
            description: "盒模型尺寸计算".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="box">Box with width: 200px; height: 100px; padding: 10px; margin: 10px; border: 5px solid #333;</div>
                <div class="measurements">Total width: 230px (200 + 10 + 10 + 5 + 5)</div>
            </div></body></html>"#.to_string(),
            css: r#".container { border: 1px dashed #999; width: 400px; }
                     .box { width: 200px; height: 100px; padding: 10px; margin: 10px; border: 5px solid #333; background: #4ecdc4; color: white; }
                     .measurements { margin-top: 10px; color: #666; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 文本和溢出 - 额外测试
        // ═══════════════════════════════════════════════════════════════

        // ─── text-overflow with multiple lines ──
        TestCase {
            id: "css-layout/text-overflow-multiple".to_string(),
            description: "多行文本省略".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="multi-line-ellipsis">This is a very long text that should be truncated with ellipsis when it overflows its container. It should work across multiple lines as well.</div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 200px; }
                     .multi-line-ellipsis { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; text-overflow: ellipsis; background: #4ecdc4; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ─── overflow with scroll ──
        TestCase {
            id: "css-layout/overflow-scroll".to_string(),
            description: "overflow:scroll 滚动效果".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="scroll-box">
                    <p>Line 1</p>
                    <p>Line 2</p>
                    <p>Line 3</p>
                    <p>Line 4</p>
                    <p>Line 5</p>
                    <p>Line 6</p>
                    <p>Line 7</p>
                    <p>Line 8</p>
                    <p>Line 9</p>
                    <p>Line 10</p>
                </div>
            </div></body></html>"#.to_string(),
            css: r#".container { width: 200px; }
                     .scroll-box { height: 150px; overflow: scroll; border: 2px solid #333; background: #4ecdc4; color: white; padding: 10px; }
                     p { margin: 5px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
    ]
}
