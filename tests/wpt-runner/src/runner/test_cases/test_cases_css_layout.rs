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

        // ═══════════════════════════════════════════════════════════════
        //  CSS 多列布局
        // ═══════════════════════════════════════════════════════════════

        // ── column-count 基础 ──
        TestCase {
            id: "css-layout/column-count-basic".to_string(),
            description: "column-count 多列布局".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="multi-col">
                <p>第一段文字内容，用于测试多列布局效果。</p>
                <p>第二段文字内容，继续填充多列。</p>
                <p>第三段文字内容，验证列数正确。</p>
                <p>第四段文字内容，更多填充。</p>
            </div></body></html>"#.to_string(),
            css: r#".multi-col { column-count: 3; column-gap: 20px; column-rule: 1px solid #ccc; padding: 10px; background: #f8f8f8; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── column-width 配合 column-count ──
        TestCase {
            id: "css-layout/column-width-constraint".to_string(),
            description: "column-width 约束列宽".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="cols">
                <p>Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu.</p>
                <p>Nu xi omicron pi rho sigma tau upsilon phi chi psi omega.</p>
            </div></body></html>"#.to_string(),
            css: r#".cols { column-width: 150px; column-count: 4; padding: 10px; background: #e8f5e9; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 变量在布局中的使用
        // ═══════════════════════════════════════════════════════════════

        // ── 自定义属性控制间距 ──
        TestCase {
            id: "css-layout/custom-properties-spacing".to_string(),
            description: "CSS 自定义属性控制间距".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="item">A</div>
                <div class="item">B</div>
                <div class="item">C</div>
                <div class="item">D</div>
            </div></body></html>"#.to_string(),
            css: r#":root { --gap: 16px; --item-bg: #6c5ce7; --item-color: white; }
                     .container { display: flex; flex-wrap: wrap; gap: var(--gap); padding: var(--gap); background: #dfe6e9; }
                     .item { width: 100px; height: 80px; background: var(--item-bg); color: var(--item-color); display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 自定义属性回退值 ──
        TestCase {
            id: "css-layout/custom-properties-fallback".to_string(),
            description: "CSS 自定义属性回退值".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="box" style="background: var(--undefined-color, #e74c3c);">Fallback</div></body></html>"#.to_string(),
            css: r#".box { width: 200px; height: 100px; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Flexbox 对齐/间距边缘情况
        // ═══════════════════════════════════════════════════════════════

        // ── flex 嵌套对齐 ──
        TestCase {
            id: "css-layout/flex-nested-alignment".to_string(),
            description: "Flex 嵌套对齐测试".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="outer">
                <div class="inner"><div class="item">1</div><div class="item">2</div></div>
                <div class="inner"><div class="item">3</div></div>
            </div></body></html>"#.to_string(),
            css: r#".outer { display: flex; flex-direction: column; gap: 10px; padding: 10px; background: #f0f0f0; }
                     .inner { display: flex; gap: 5px; background: #ddd; padding: 5px; }
                     .item { width: 50px; height: 50px; background: #00b894; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── flex-grow 比例分配 ──
        TestCase {
            id: "css-layout/flex-grow-ratio".to_string(),
            description: "flex-grow 按比例分配剩余空间".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex">
                <div class="small">1x</div>
                <div class="medium">2x</div>
                <div class="large">3x</div>
            </div></body></html>"#.to_string(),
            css: r#".flex { display: flex; width: 600px; gap: 10px; background: #f8f9fa; padding: 10px; }
                     .small { flex: 1; height: 50px; background: #74b9ff; }
                     .medium { flex: 2; height: 50px; background: #a29bfe; }
                     .large { flex: 3; height: 50px; background: #fd79a8; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── flex 基线对齐 ──
        TestCase {
            id: "css-layout/flex-align-baseline".to_string(),
            description: "flex align-items:baseline 对齐".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="flex">
                <div class="box" style="font-size:14px;">Small</div>
                <div class="box" style="font-size:24px;">Big</div>
                <div class="box" style="font-size:18px;">Medium</div>
            </div></body></html>"#.to_string(),
            css: r#".flex { display: flex; align-items: baseline; gap: 10px; padding: 10px; background: #ffeaa7; }
                     .box { padding: 8px; background: #fdcb6e; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Grid 嵌套布局
        // ═══════════════════════════════════════════════════════════════

        // ── 嵌套网格布局 ──
        TestCase {
            id: "css-layout/grid-nested".to_string(),
            description: "嵌套网格布局".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="outer-grid">
                <div class="cell"><div class="inner-grid"><div class="item">A1</div><div class="item">A2</div></div></div>
                <div class="cell"><div class="inner-grid"><div class="item">B1</div><div class="item">B2</div><div class="item">B3</div></div></div>
            </div></body></html>"#.to_string(),
            css: r#".outer-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; padding: 10px; background: #dfe6e9; }
                     .cell { background: #b2bec3; padding: 10px; }
                     .inner-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(60px, 1fr)); gap: 5px; }
                     .item { height: 40px; background: #636e72; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── grid 隐式轨道 ──
        TestCase {
            id: "css-layout/grid-implicit-tracks".to_string(),
            description: "grid 隐式轨道 auto-rows/auto-columns".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="grid">
                <div class="item">1</div><div class="item">2</div><div class="item">3</div>
                <div class="item">4</div><div class="item">5</div><div class="item">6</div>
                <div class="item">7</div><div class="item">8</div><div class="item">9</div>
            </div></body></html>"#.to_string(),
            css: r#".grid { display: grid; grid-template-columns: repeat(3, 1fr); grid-auto-rows: 80px; gap: 8px; padding: 10px; background: #e8e8e8; }
                     .item { background: #e17055; color: white; display: flex; align-items: center; justify-content: center; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  响应式布局模式
        // ═══════════════════════════════════════════════════════════════

        // ── 卡片网格响应式 ──
        TestCase {
            id: "css-layout/responsive-card-grid".to_string(),
            description: "响应式卡片网格".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="cards">
                <div class="card"><h3>Card 1</h3><p>Content for card one.</p></div>
                <div class="card"><h3>Card 2</h3><p>Content for card two.</p></div>
                <div class="card"><h3>Card 3</h3><p>Content for card three.</p></div>
                <div class="card"><h3>Card 4</h3><p>Content for card four.</p></div>
            </div></body></html>"#.to_string(),
            css: r#".cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 16px; padding: 16px; background: #f5f6fa; }
                     .card { background: white; border: 1px solid #ddd; border-radius: 8px; padding: 16px; }
                     .card h3 { margin: 0 0 8px 0; color: #2d3436; }
                     .card p { margin: 0; color: #636e72; font-size: 14px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 圣杯布局 ──
        TestCase {
            id: "css-layout/holy-grail".to_string(),
            description: "CSS Grid 圣杯布局".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="layout">
                <header>Header</header>
                <nav>Navigation</nav>
                <main>Main Content Area</main>
                <aside>Sidebar</aside>
                <footer>Footer</footer>
            </div></body></html>"#.to_string(),
            css: r#".layout { display: grid; grid-template-areas: "header header header" "nav main aside" "footer footer footer"; grid-template-columns: 150px 1fr 150px; grid-template-rows: 50px 1fr 40px; gap: 5px; height: 400px; }
                     header { grid-area: header; background: #2d3436; color: white; padding: 10px; }
                     nav { grid-area: nav; background: #dfe6e9; padding: 10px; }
                     main { grid-area: main; background: #ffffff; padding: 10px; }
                     aside { grid-area: aside; background: #ffeaa7; padding: 10px; }
                     footer { grid-area: footer; background: #636e72; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── Flexbox 粘性页脚 ──
        TestCase {
            id: "css-layout/flex-sticky-footer".to_string(),
            description: "Flexbox 粘性页脚模式".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="page">
                <header>Page Header</header>
                <main>Main content that pushes footer down.</main>
                <footer>Sticky Footer</footer>
            </div></body></html>"#.to_string(),
            css: r#".page { display: flex; flex-direction: column; height: 400px; }
                     header { background: #0984e3; color: white; padding: 15px; }
                     main { flex: 1; background: #f5f6fa; padding: 15px; }
                     footer { background: #2d3436; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 定位边缘情况
        // ═══════════════════════════════════════════════════════════════

        // ── 绝对定位叠放 ──
        TestCase {
            id: "css-layout/absolute-stacking".to_string(),
            description: "绝对定位元素 z-index 叠放".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="box bottom">Bottom (z:1)</div>
                <div class="box middle">Middle (z:2)</div>
                <div class="box top">Top (z:3)</div>
            </div></body></html>"#.to_string(),
            css: r#".container { position: relative; width: 300px; height: 200px; background: #f0f0f0; }
                     .box { position: absolute; width: 150px; height: 100px; color: white; padding: 10px; }
                     .bottom { top: 0; left: 0; background: rgba(231, 76, 60, 0.8); z-index: 1; }
                     .middle { top: 30px; left: 50px; background: rgba(46, 204, 113, 0.8); z-index: 2; }
                     .top { top: 60px; left: 100px; background: rgba(52, 152, 219, 0.8); z-index: 3; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── fixed 定位导航栏 ──
        TestCase {
            id: "css-layout/fixed-navbar".to_string(),
            description: "fixed 定位导航栏".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <nav class="navbar">Fixed Navigation Bar</nav>
                <div class="content">
                    <p>Content line 1</p><p>Content line 2</p>
                    <p>Content line 3</p><p>Content line 4</p>
                </div>
            </body></html>"#.to_string(),
            css: r#".navbar { position: fixed; top: 0; left: 0; width: 100%; height: 50px; background: #2c3e50; color: white; padding: 10px; z-index: 100; }
                     .content { padding-top: 70px; background: #ecf0f1; min-height: 300px; }
                     p { margin: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 变换与渐变组合
        // ═══════════════════════════════════════════════════════════════

        // ── transform 旋转 + 平移组合 ──
        TestCase {
            id: "css-layout/transform-combine".to_string(),
            description: "transform 旋转和平移组合".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><div class="container">
                <div class="box rotate">Rotated 45deg</div>
                <div class="box translate">Translated</div>
                <div class="box both">Both</div>
            </div></body></html>"#.to_string(),
            css: r#".container { display: flex; gap: 60px; padding: 40px; justify-content: center; align-items: center; height: 300px; background: #f8f9fa; }
                     .box { width: 100px; height: 100px; background: #e74c3c; color: white; display: flex; align-items: center; justify-content: center; }
                     .rotate { transform: rotate(45deg); }
                     .translate { transform: translateX(20px) translateY(10px); background: #3498db; }
                     .both { transform: rotate(-15deg) translateX(30px); background: #2ecc71; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 渐变背景组合 ──
        TestCase {
            id: "css-layout/gradient-combinations".to_string(),
            description: "多种渐变背景组合".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <div class="linear">Linear Gradient</div>
                <div class="radial">Radial Gradient</div>
            </body></html>"#.to_string(),
            css: r#".linear { width: 300px; height: 100px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 10px; margin: 10px; }
                     .radial { width: 300px; height: 100px; background: radial-gradient(circle, #f093fb 0%, #f5576c 100%); color: white; padding: 10px; margin: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  文本排版边缘情况
        // ═══════════════════════════════════════════════════════════════

        // ── 混合字号行内元素 ──
        TestCase {
            id: "css-layout/mixed-font-sizes-inline".to_string(),
            description: "混合字号行内元素排版".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body><p class="text">Normal text <span class="big">BIG text</span> normal <span class="small">small text</span> normal.</p></body></html>"#.to_string(),
            css: r#".text { font-size: 16px; line-height: 1.5; padding: 10px; background: #fff3e0; }
                     .big { font-size: 32px; font-weight: bold; color: #e74c3c; }
                     .small { font-size: 10px; color: #7f8c8d; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 长单词换行测试 ──
        TestCase {
            id: "css-layout/long-word-wrapping".to_string(),
            description: "长单词换行行为测试".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <div class="box break-all">Superlongwordthatdoesnotfitinonelinebreakall</div>
                <div class="box break-word">Superlongwordthatdoesnotfitinonelinebreakword</div>
            </body></html>"#.to_string(),
            css: r#".box { width: 120px; padding: 8px; margin: 10px; background: #e8f5e9; border: 1px solid #81c784; }
                     .break-all { word-break: break-all; }
                     .break-word { overflow-wrap: break-word; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── text-align 多种对齐 ──
        TestCase {
            id: "css-layout/text-align-modes".to_string(),
            description: "text-align 多种对齐模式".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
                <p class="left">Left aligned text.</p>
                <p class="center">Center aligned text.</p>
                <p class="right">Right aligned text.</p>
                <p class="justify">Justified text that spans across the full width of the container.</p>
            </body></html>"#.to_string(),
            css: r#"p { width: 300px; padding: 8px; margin: 5px; background: #e3f2fd; }
                     .left { text-align: left; }
                     .center { text-align: center; }
                     .right { text-align: right; }
                     .justify { text-align: justify; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Grid 高级测试
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/grid-named-areas".to_string(),
            description: "Grid with grid-template-areas named layout".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-named">
    <header class="gh">Header</header>
    <nav class="gn">Nav</nav>
    <main class="gm">Main</main>
    <aside class="ga">Aside</aside>
    <footer class="gf">Footer</footer>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-named {
    display: grid;
    grid-template-areas:
        "header header"
        "nav main"
        "nav aside"
        "footer footer";
    grid-template-columns: 200px 1fr;
    grid-template-rows: auto 1fr 1fr auto;
    width: 600px;
    gap: 8px;
}
.gh { grid-area: header; background: #2196F3; color: white; padding: 10px; }
.gn { grid-area: nav; background: #4CAF50; padding: 10px; }
.gm { grid-area: main; background: #FF9800; padding: 10px; }
.ga { grid-area: aside; background: #9C27B0; color: white; padding: 10px; }
.gf { grid-area: footer; background: #607D8B; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-auto-fill-minmax".to_string(),
            description: "Grid auto-fill with minmax responsive layout".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-auto">
    <div class="card">Card 1</div>
    <div class="card">Card 2</div>
    <div class="card">Card 3</div>
    <div class="card">Card 4</div>
    <div class="card">Card 5</div>
    <div class="card">Card 6</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-auto {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 10px;
    width: 500px;
}
.card {
    background: #E3F2FD;
    padding: 15px;
    border: 1px solid #90CAF9;
    text-align: center;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-span".to_string(),
            description: "Grid column and row spanning".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-span">
    <div class="wide">Wide (span 2 cols)</div>
    <div class="tall">Tall (span 2 rows)</div>
    <div class="normal">Normal</div>
    <div class="normal">Normal</div>
    <div class="both">Span 2x2</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-span {
    display: grid;
    grid-template-columns: repeat(3, 100px);
    grid-template-rows: repeat(3, 80px);
    gap: 5px;
    width: 320px;
}
.wide { grid-column: span 2; background: #FFCDD2; padding: 10px; }
.tall { grid-row: span 2; background: #C8E6C9; padding: 10px; }
.normal { background: #E0E0E0; padding: 10px; }
.both { grid-column: span 2; grid-row: span 2; background: #B39DDB; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-auto-rows-cols".to_string(),
            description: "Grid auto-rows and auto-columns sizing".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-auto-rc">
    <div>Item 1</div>
    <div>Item 2</div>
    <div>Item 3</div>
    <div>Item 4</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-auto-rc {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-auto-rows: 60px;
    grid-auto-columns: 80px;
    gap: 10px;
    width: 400px;
}
.grid-auto-rc > div {
    background: #FFF3E0;
    padding: 8px;
    border: 1px solid #FFE0B2;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-implicit".to_string(),
            description: "Grid implicit tracks from auto-placement".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-implicit">
    <div>A</div><div>B</div><div>C</div>
    <div>D</div><div>E</div><div>F</div>
    <div>G</div><div>H</div><div>I</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-implicit {
    display: grid;
    grid-template-columns: repeat(3, 80px);
    gap: 5px;
}
.grid-implicit > div {
    background: #E8F5E9;
    padding: 10px;
    border: 1px solid #A5D6A7;
    text-align: center;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-place-items".to_string(),
            description: "Grid place-items alignment".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="grid-place">
    <div class="center">Centered</div>
    <div class="start">Start</div>
    <div class="end">End</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.grid-place {
    display: grid;
    grid-template-columns: repeat(3, 120px);
    grid-template-rows: 100px;
    gap: 10px;
    width: 400px;
}
.center { place-self: center; background: #F3E5F5; padding: 8px; }
.start { place-self: start; background: #E0F7FA; padding: 8px; }
.end { place-self: end; background: #FFF8E1; padding: 8px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-nested".to_string(),
            description: "Nested grid containers".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="outer">
    <div class="inner-a">
        <div class="cell">A1</div>
        <div class="cell">A2</div>
    </div>
    <div class="inner-b">
        <div class="cell">B1</div>
        <div class="cell">B2</div>
        <div class="cell">B3</div>
    </div>
</div>
</body></html>"#.to_string(),
            css: r#"
.outer {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 15px;
    width: 400px;
}
.inner-a, .inner-b {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 5px;
    background: #ECEFF1;
    padding: 10px;
}
.cell {
    background: #B0BEC5;
    padding: 8px;
    text-align: center;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/grid-responsive-cards".to_string(),
            description: "Responsive card grid with auto-fill".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="responsive-grid">
    <div class="rcard"><h3>Title 1</h3><p>Description 1</p></div>
    <div class="rcard"><h3>Title 2</h3><p>Description 2</p></div>
    <div class="rcard"><h3>Title 3</h3><p>Description 3</p></div>
    <div class="rcard"><h3>Title 4</h3><p>Description 4</p></div>
</div>
</body></html>"#.to_string(),
            css: r#"
.responsive-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
    width: 500px;
}
.rcard {
    background: #FAFAFA;
    border: 1px solid #E0E0E0;
    border-radius: 8px;
    padding: 12px;
}
.rcard h3 { margin: 0 0 8px 0; font-size: 14px; color: #333; }
.rcard p { margin: 0; font-size: 12px; color: #666; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Flexbox 高级测试
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/flex-wrap-reverse".to_string(),
            description: "Flexbox wrap-reverse layout".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="flex-wrap-rev">
    <div>A</div><div>B</div><div>C</div>
    <div>D</div><div>E</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.flex-wrap-rev {
    display: flex;
    flex-wrap: wrap-reverse;
    width: 300px;
    gap: 5px;
}
.flex-wrap-rev > div {
    width: 100px;
    height: 50px;
    background: #FFCCBC;
    padding: 5px;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/flex-align-self".to_string(),
            description: "Flexbox align-self overrides".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="flex-as">
    <div class="stretch">Stretch</div>
    <div class="start">Start</div>
    <div class="center">Center</div>
    <div class="end">End</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.flex-as {
    display: flex;
    height: 120px;
    gap: 5px;
    width: 400px;
}
.flex-as > div { width: 80px; background: #D1C4E9; padding: 5px; }
.flex-as .stretch { align-self: stretch; }
.flex-as .start { align-self: flex-start; background: #C5CAE9; }
.flex-as .center { align-self: center; background: #B2DFDB; }
.flex-as .end { align-self: flex-end; background: #FFE0B2; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 自定义属性高级用法
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/css-var-fallback".to_string(),
            description: "CSS custom properties with var() fallback chains".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="theme">
    <p class="primary">Primary text</p>
    <p class="secondary">Secondary text</p>
    <div class="box">Box with fallback</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.theme {
    --color-primary: #1976D2;
    --color-secondary: #388E3C;
    --spacing: 16px;
}
.primary { color: var(--color-primary); padding: var(--spacing); background: #E3F2FD; }
.secondary { color: var(--color-secondary); padding: var(--spacing); background: #E8F5E9; }
.box {
    color: var(--undefined-color, #757575);
    background: var(--undefined-bg, #F5F5F5);
    padding: var(--undefined-pad, 8px);
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/css-var-calc".to_string(),
            description: "CSS variables combined with calc()".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="calc-var">
    <div class="item">Item 1</div>
    <div class="item">Item 2</div>
    <div class="item">Item 3</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.calc-var {
    --base-size: 80px;
    --gap: 10px;
    --columns: 3;
    display: flex;
    gap: var(--gap);
    width: 300px;
}
.item {
    width: calc((100% - var(--gap) * 2) / var(--columns));
    background: #CE93D8;
    padding: 10px;
    text-align: center;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/css-var-scope".to_string(),
            description: "CSS variables scope and inheritance".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="scope-root">
    <div class="child-a">
        <span>Child A</span>
    </div>
    <div class="child-b">
        <span>Child B</span>
    </div>
</div>
</body></html>"#.to_string(),
            css: r#"
.scope-root { --size: 16px; --bg: #BBDEFB; padding: 10px; background: #E3F2FD; }
.child-a { --bg: #C8E6C9; background: var(--bg); padding: var(--size); }
.child-b { --bg: #FFE0B2; background: var(--bg); padding: var(--size); }
.child-a span, .child-b span { font-size: var(--size); }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 多列布局
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/multi-column-count".to_string(),
            description: "CSS multi-column layout with column-count".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="cols-3">
    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt.</p>
    <p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat.</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.cols-3 {
    column-count: 3;
    column-gap: 20px;
    column-rule: 1px solid #BDBDBD;
    width: 600px;
    padding: 10px;
    background: #FAFAFA;
}
.cols-3 p { margin: 0 0 10px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/multi-column-width".to_string(),
            description: "CSS multi-column with column-width".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="cols-width">
    <h2>Multi-Column Article</h2>
    <p>Text content that flows across multiple columns based on column-width.</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.cols-width {
    column-width: 150px;
    column-gap: 15px;
    column-rule: 2px dashed #90CAF9;
    width: 500px;
    padding: 15px;
    background: #FFF;
}
.cols-width h2 { column-span: all; margin: 0 0 10px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  @supports 高级
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/supports-basic".to_string(),
            description: "CSS @supports feature query".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="supports-test">
    <p class="grid-check">Grid support check</p>
    <p class="flex-check">Flexbox support check</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.supports-test p { padding: 8px; margin: 5px; }
@supports (display: grid) {
    .grid-check { background: #C8E6C9; color: #2E7D32; }
}
@supports (display: flex) {
    .flex-check { background: #BBDEFB; color: #1565C0; }
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/supports-not".to_string(),
            description: "CSS @supports with NOT operator".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="supports-not">
    <p>NOT fallback test</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.supports-not p { background: #FFCDD2; padding: 10px; margin: 5px; }
@supports not (display: grid) {
    .supports-not p { background: #FFF9C4; }
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/supports-and-or".to_string(),
            description: "CSS @supports with AND/OR operators".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="supports-logic">
    <p class="and-test">AND test</p>
    <p class="or-test">OR test</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.supports-logic p { padding: 10px; margin: 5px; }
@supports (display: flex) and (gap: 10px) {
    .and-test { background: #E8F5E9; border: 2px solid #4CAF50; }
}
@supports (display: grid) or (display: flex) {
    .or-test { background: #E3F2FD; border: 2px solid #2196F3; }
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS @media range + 逻辑属性 + scroll-snap
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "css-layout/media-range-syntax".to_string(),
            description: "CSS @media range syntax queries".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="responsive">
    <p class="wide">Wide content</p>
    <p class="narrow">Narrow content</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.responsive p { padding: 10px; margin: 5px; background: #F5F5F5; }
@media (width >= 600px) {
    .wide { background: #E8F5E9; }
}
@media (width < 600px) {
    .narrow { background: #FFF3E0; }
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/logical-properties".to_string(),
            description: "CSS logical properties margin-block/padding-inline".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="logical">
    <p>Logical properties test</p>
</div>
</body></html>"#.to_string(),
            css: r#"
.logical {
    margin-block: 10px 20px;
    padding-inline: 15px;
    background: #E8EAF6;
}
.logical p {
    margin-block-start: 5px;
    margin-block-end: 5px;
    padding-inline-start: 10px;
    padding-inline-end: 10px;
    background: #C5CAE9;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/scroll-snap".to_string(),
            description: "CSS scroll-snap container and items".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div class="snap-container">
    <div class="snap-item" style="background:#EF5350">Slide 1</div>
    <div class="snap-item" style="background:#42A5F5">Slide 2</div>
    <div class="snap-item" style="background:#66BB6A">Slide 3</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.snap-container {
    display: flex;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    width: 300px;
    height: 150px;
}
.snap-item {
    scroll-snap-align: start;
    min-width: 300px;
    height: 150px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 24px;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        // ── CSS filter 布局集成测试 ──
        TestCase {
            id: "css-layout/filter-blur-layout".to_string(),
            description: "CSS filter: blur() 不影响布局但生成 FilterPrimitive".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="width: 200px; height: 100px; background: #e0e0e0; filter: blur(3px);">
                <p style="color: black; font-size: 14px;">Blurred content</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/filter-grayscale-card".to_string(),
            description: "CSS filter: grayscale() 应用于卡片布局".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="display: flex; gap: 10px;">
                <div style="filter: grayscale(1); background: #f0f0f0; padding: 10px; width: 150px;">
                    <h3 style="color: #333; font-size: 16px;">Gray Card</h3>
                    <p style="color: #666; font-size: 12px;">Desaturated</p>
                </div>
                <div style="background: #f0f0f0; padding: 10px; width: 150px;">
                    <h3 style="color: #333; font-size: 16px;">Normal Card</h3>
                    <p style="color: #666; font-size: 12px;">Full color</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── text-overflow 布局测试 ──
        TestCase {
            id: "css-layout/text-overflow-ellipsis-flex".to_string(),
            description: "text-overflow: ellipsis 在 flex 容器中".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="display: flex; width: 300px; border: 1px solid #ccc;">
                <div style="flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: black; font-size: 14px;">
                    This is a very long text that should be truncated with ellipsis
                </div>
                <div style="flex-shrink: 0; color: blue; font-size: 14px;">More</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 多属性组合布局测试 ──
        TestCase {
            id: "css-layout/multi-column-with-spacing".to_string(),
            description: "column-count + letter-spacing 组合渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="column-count: 2; column-gap: 20px; letter-spacing: 1px; color: #333; font-size: 14px; width: 400px;">
                <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.</p>
                <p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip.</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        TestCase {
            id: "css-layout/positioned-with-filter".to_string(),
            description: "absolute 定位 + CSS filter 组合".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="position: relative; width: 300px; height: 200px; background: #f5f5f5;">
                <div style="position: absolute; top: 20px; left: 20px; width: 100px; height: 80px; background: #42A5F5; filter: brightness(1.2);"></div>
                <div style="position: absolute; bottom: 20px; right: 20px; width: 100px; height: 80px; background: #66BB6A; filter: sepia(0.5);"></div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── 响应式布局测试 ──
        TestCase {
            id: "css-layout/responsive-pricing-table".to_string(),
            description: "响应式定价卡片 — flexbox + filter + text-overflow".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body style="margin: 0; padding: 20px;">
            <div style="display: flex; flex-wrap: wrap; gap: 16px; justify-content: center;">
                <div style="width: 200px; padding: 20px; background: white; border: 1px solid #e0e0e0;">
                    <h3 style="color: #333; font-size: 18px; margin: 0 0 8px;">Basic</h3>
                    <div style="color: #2196F3; font-size: 32px; font-weight: bold;">$9</div>
                    <p style="color: #666; font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">Essential features for getting started with the platform</p>
                </div>
                <div style="width: 200px; padding: 20px; background: white; border: 2px solid #2196F3; filter: drop-shadow(0 4 12 rgba(33,150,243,0.3));">
                    <h3 style="color: #2196F3; font-size: 18px; margin: 0 0 8px;">Pro</h3>
                    <div style="color: #2196F3; font-size: 32px; font-weight: bold;">$29</div>
                    <p style="color: #666; font-size: 12px;">Advanced features for growing teams</p>
                </div>
                <div style="width: 200px; padding: 20px; background: white; border: 1px solid #e0e0e0; filter: grayscale(0.3);">
                    <h3 style="color: #333; font-size: 18px; margin: 0 0 8px;">Enterprise</h3>
                    <div style="color: #333; font-size: 32px; font-weight: bold;">$99</div>
                    <p style="color: #666; font-size: 12px;">Complete solution for large organizations</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 间距属性综合测试 ──
        TestCase {
            id: "css-layout/spacing-comprehensive".to_string(),
            description: "letter-spacing + word-spacing + line-height 综合排版".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="width: 400px; color: #222;">
                <h1 style="font-size: 24px; letter-spacing: 2px; word-spacing: 5px; line-height: 1.4; margin: 0 0 12px;">Spaced Heading Title</h1>
                <p style="font-size: 14px; letter-spacing: 0.5px; line-height: 1.8; margin: 0 0 8px;">Body text with slight letter spacing and generous line height for readability.</p>
                <p style="font-size: 12px; letter-spacing: -0.5px; word-spacing: -2px; color: #888;">Condensed text with negative spacing for captions.</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
                "glyph_count_ge:10".to_string(),
            ],
        },
        // ── Grid + 间距组合测试 ──
        TestCase {
            id: "css-layout/grid-with-typography".to_string(),
            description: "Grid 布局 + 排版间距组合".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; width: 400px;">
                <div style="background: #e3f2fd; padding: 12px;">
                    <h3 style="color: #1565C0; font-size: 16px; letter-spacing: 1px; margin: 0 0 4px;">Column A</h3>
                    <p style="color: #333; font-size: 13px; word-spacing: 3px; margin: 0;">Text with word spacing in grid cell</p>
                </div>
                <div style="background: #e8f5e9; padding: 12px;">
                    <h3 style="color: #2E7D32; font-size: 16px; letter-spacing: 1px; margin: 0 0 4px;">Column B</h3>
                    <p style="color: #333; font-size: 13px; word-spacing: 3px; margin: 0;">Another grid cell text</p>
                </div>
                <div style="background: #fff3e0; padding: 12px; grid-column: span 2;">
                    <p style="color: #E65100; font-size: 14px; letter-spacing: 2px; text-align: center;">Spanning full width with centered spaced text</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
    ]
}
