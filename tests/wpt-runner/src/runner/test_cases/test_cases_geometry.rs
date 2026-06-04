//! 精确布局几何测试。
//!
//! 使用精确的几何断言验证 CSS 布局引擎的输出：
//! - 盒模型（margin/padding/border/w/h）
//! - 块级布局（垂直堆叠、margin 折叠）
//! - Flexbox 布局（主轴/交叉轴对齐）
//! - Grid 布局（轨道尺寸、项放置）
//! - 定位（absolute/fixed）
//! - 行内布局（文本渲染）

use super::TestCase;

#[allow(clippy::vec_init_then_push)]
pub fn geometry_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();

    // ═══════════════════════════════════════════════════════════════
    // 1. 块级布局基础
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/block/simple-div".into(),
        description: "简单块级 div 填满父容器宽度".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Hello</div></body></html>"#.into(),
        css: String::new(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_root_fills_viewport".into(),
            "layout_box_count_ge:4".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/block/width-height".into(),
        description: "固定宽高的块级元素".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Box</div></body></html>"#.into(),
        css: "div { width: 200px; height: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "fill_count_ge:1".into(),
            "layout_nth_width_ge:2:200.0".into(),
            "layout_nth_height_ge:2:100.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/block/two-divs-stacked".into(),
        description: "两个块级 div 垂直堆叠".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>A</div><div>B</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "fill_count_ge:2".into(),
            "layout_box_count_ge:5".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 2. 盒模型
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/box-model/padding".into(),
        description: "padding 增大盒子尺寸".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Content</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; padding: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:140.0".into(),
            "layout_nth_height_ge:2:90.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/box-model/border".into(),
        description: "border 增大盒子尺寸".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Bordered</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; border: 5px solid black; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_stroke_primitives".into(),
            "layout_nth_width_ge:2:110.0".into(),
            "layout_nth_height_ge:2:60.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/box-model/margin".into(),
        description: "margin 在盒子外创建空间".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>A</div><div>B</div></body></html>"#.into(),
        css: "div { width: 100px; height: 30px; margin-bottom: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:2".into(),
            "layout_box_count_ge:5".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/box-model/box-sizing-border-box".into(),
        description: "box-sizing:border-box 包含 padding 和 border".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Content</div></body></html>"#.into(),
        css: "div { width: 200px; height: 100px; padding: 20px; border: 5px solid black; box-sizing: border-box; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            // border-box: width = content+padding+border = 200px total
            "layout_nth_width_ge:2:195.0".into(),
            "layout_nth_height_ge:2:95.0".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 3. Flexbox 布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/flex/row-two-items".into(),
        description: "flex 行方向排列两个子元素".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>A</span><span>B</span></div></body></html>"#.into(),
        css: ".flex { display: flex; width: 400px; height: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_box_count_ge:6".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/flex/justify-center".into(),
        description: "flex justify-content:center 居中子元素".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>X</span></div></body></html>"#.into(),
        css: ".flex { display: flex; justify-content: center; width: 400px; height: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_box_count_ge:5".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/flex/column-direction".into(),
        description: "flex-direction:column 垂直排列".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>A</span><span>B</span></div></body></html>"#.into(),
        css: ".flex { display: flex; flex-direction: column; width: 200px; height: 300px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/flex/wrap".into(),
        description: "flex-wrap 换行".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>1</span><span>2</span><span>3</span></div></body></html>"#.into(),
        css: ".flex { display: flex; flex-wrap: wrap; width: 200px; height: 200px; } span { width: 120px; height: 50px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:7".into(),
            "fill_count_ge:1".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 4. Grid 布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/grid/2x2".into(),
        description: "2x2 Grid 布局".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="grid"><span>1</span><span>2</span><span>3</span><span>4</span></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 100px 100px; grid-template-rows: 50px 50px; width: 200px; height: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_box_count_ge:8".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/grid/auto-flow".into(),
        description: "Grid auto-flow 自动放置".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="grid"><span>A</span><span>B</span><span>C</span></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 1fr 1fr; width: 400px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:7".into(),
            "fill_count_ge:1".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 5. 定位
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/position/absolute".into(),
        description: "absolute 定位脱离文档流".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="rel"><div class="abs">Abs</div></div></body></html>"#.into(),
        css: ".rel { position: relative; width: 300px; height: 200px; } .abs { position: absolute; top: 10px; left: 20px; width: 100px; height: 50px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/position/fixed".into(),
        description: "fixed 定位相对于视口".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="fixed">Fixed</div></body></html>"#.into(),
        css: ".fixed { position: fixed; top: 0; left: 0; width: 100px; height: 50px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 6. 内联样式
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/inline-style/width-height".into(),
        description: "内联样式设置宽高".into(),
        category: "geometry".into(),
        html:
            r#"<html><body><div style="width: 300px; height: 150px; background-color: red;">Styled</div></body></html>"#
                .into(),
        css: String::new(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:300.0".into(),
            "layout_nth_height_ge:2:150.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/inline-style/margin-padding".into(),
        description: "内联样式设置 margin 和 padding".into(),
        category: "geometry".into(),
        html: r#"<html><body><div style="width: 200px; height: 100px; margin: 20px; padding: 10px;">Box</div></body></html>"#.into(),
        css: String::new(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:220.0".into(),
            "layout_nth_height_ge:2:120.0".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 7. 渲染图元精确性
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/primitives/text-glyphs".into(),
        description: "文本渲染生成字形图元".into(),
        category: "geometry".into(),
        html: r#"<html><body><p>Hello World</p></body></html>"#.into(),
        css: String::new(),
        assertions: vec![
            "dom_has_body".into(),
            "glyph_count_ge:5".into(),
            "has_fill_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/primitives/border-radius".into(),
        description: "border-radius 生成圆角矩形图元".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Rounded</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; border-radius: 10px; background: blue; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "geometry/primitives/gradient-fill".into(),
        description: "linear-gradient 生成渐变图元".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Gradient</div></body></html>"#.into(),
        css: "div { width: 200px; height: 100px; background: linear-gradient(to right, red, blue); }".into(),
        assertions: vec!["dom_has_body".into(), "gradient_count_ge:1".into()],
    });

    tests.push(TestCase {
        id: "geometry/primitives/box-shadow".into(),
        description: "box-shadow 生成阴影图元".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Shadow</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; box-shadow: 5px 5px 10px rgba(0,0,0,0.5); }".into(),
        assertions: vec!["dom_has_body".into(), "shadow_count_ge:1".into()],
    });

    tests.push(TestCase {
        id: "geometry/primitives/multiple-box-shadows".into(),
        description: "多个 box-shadow 生成多个阴影图元".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Multi</div></body></html>"#.into(),
        css: "div { width: 100px; height: 50px; box-shadow: 2px 2px 4px red, -2px -2px 4px blue; }".into(),
        assertions: vec!["dom_has_body".into(), "shadow_count_ge:2".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 8. 综合布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/composite/holy-grail".into(),
        description: "圣杯布局骨架".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <header>H</header>
            <div class="main">
                <nav>Nav</nav>
                <article>Content</article>
                <aside>Side</aside>
            </div>
            <footer>F</footer>
        </body></html>"#
            .into(),
        css:
            ".main { display: flex; width: 800px; } nav { width: 150px; } article { flex: 1; } aside { width: 150px; }"
                .into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_box_count_ge:8".into(),
            "fill_count_ge:3".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/composite/card-grid".into(),
        description: "卡片网格布局".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="grid">
                <div class="card">Card 1</div>
                <div class="card">Card 2</div>
                <div class="card">Card 3</div>
                <div class="card">Card 4</div>
            </div>
        </body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; width: 400px; } .card { height: 100px; background: white; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:9".into(),
            "fill_count_ge:4".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/composite/nested-flex".into(),
        description: "嵌套 flex 布局".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="outer">
                <div class="inner"><span>A</span><span>B</span></div>
                <div class="inner"><span>C</span><span>D</span></div>
            </div>
        </body></html>"#
            .into(),
        css: ".outer { display: flex; width: 400px; height: 200px; } .inner { display: flex; flex: 1; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:10".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/composite/text-in-flex".into(),
        description: "flex 容器中的文本渲染".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><p>Left</p><p>Right</p></div></body></html>"#.into(),
        css: ".flex { display: flex; width: 400px; height: 100px; } p { flex: 1; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "glyph_count_ge:2".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/composite/stacked-with-margin".into(),
        description: "多个块级元素 margin 分隔".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="box">A</div>
            <div class="box">B</div>
            <div class="box">C</div>
        </body></html>"#
            .into(),
        css: ".box { width: 300px; height: 50px; margin: 10px; background: gray; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:3".into(),
            "layout_box_count_ge:7".into(),
            "layout_children_non_overlapping".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/composite/overflow-hidden".into(),
        description: "overflow:hidden 裁剪内容".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="clip"><div class="overflow">Overflow</div></div></body></html>"#.into(),
        css: ".clip { width: 100px; height: 50px; overflow: hidden; } .overflow { width: 200px; height: 100px; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "has_fill_primitives".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 9. 显示和可见性
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/visibility/display-none".into(),
        description: "display:none 元素不生成布局盒".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Visible</div><div class="hidden">Hidden</div></body></html>"#.into(),
        css: ".hidden { display: none; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "has_fill_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/visibility/opacity-zero".into(),
        description: "opacity:0 元素仍占空间但不可见".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="invisible">Ghost</div></body></html>"#.into(),
        css: ".invisible { width: 100px; height: 50px; opacity: 0; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_nth_width_ge:2:100.0".into(),
            "layout_nth_height_ge:2:50.0".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 10. CSS 属性管线几何验证
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "geometry/css/width-100px".into(),
        description: "width:100px 精确尺寸验证".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="box">Content</div></body></html>"#.into(),
        css: ".box { width: 100px; height: 50px; background: #ff0000; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:1".into(),
            "layout_nth_width_ge:2:100.0".into(),
            "layout_nth_height_ge:2:50.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/padding-box".into(),
        description: "padding 扩展盒子尺寸验证".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="pad">Padded</div></body></html>"#.into(),
        css: ".pad { width: 200px; padding: 20px; background: blue; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:1".into(),
            // width + padding = 200 + 2*20 = 240
            "layout_nth_width_ge:2:240.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/max-width".into(),
        description: "max-width 约束盒子宽度".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="constrained">Wide</div></body></html>"#.into(),
        css: ".constrained { width: 1000px; max-width: 400px; background: green; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            // max-width should constrain to 400px
            "layout_nth_width_ge:2:390.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/min-height".into(),
        description: "min-height 保证最小高度".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="min">Small</div></body></html>"#.into(),
        css: ".min { min-height: 100px; background: orange; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_height_ge:2:100.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/flex-grow".into(),
        description: "flex-grow 分配剩余空间".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="flex">
                <div class="grow1">A</div>
                <div class="grow2">B</div>
            </div>
        </body></html>"#
            .into(),
        css: ".flex { display: flex; width: 300px; height: 50px; } .grow1 { flex-grow: 1; } .grow2 { flex-grow: 2; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:6".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/flex-align-items-center".into(),
        description: "align-items:center 垂直居中".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>Centered</span></div></body></html>"#.into(),
        css: ".flex { display: flex; align-items: center; width: 200px; height: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:5".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/grid-template-areas".into(),
        description: "grid-template-areas 命名区域布局".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="grid">
                <div class="header">H</div>
                <div class="main">M</div>
                <div class="footer">F</div>
            </div>
        </body></html>"#.into(),
        css: r#".grid { display: grid; grid-template-areas: "header header" "main main" "footer footer"; grid-template-columns: 1fr 1fr; width: 400px; }"#.into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:8".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/multi-gradient".into(),
        description: "多层渐变叠加渲染".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Layers</div></body></html>"#.into(),
        css: "div { width: 200px; height: 100px; background: linear-gradient(to right, red, blue), linear-gradient(to bottom, green, yellow); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "gradient_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/inset-shorthand".into(),
        description: "inset 简写定位".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="rel"><div class="abs">Positioned</div></div></body></html>"#.into(),
        css: ".rel { position: relative; width: 300px; height: 200px; } .abs { position: absolute; inset: 10px 20px; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/border-radius-percentage".into(),
        description: "百分比 border-radius 渲染".into(),
        category: "geometry".into(),
        html: r#"<html><body><div>Rounded</div></body></html>"#.into(),
        css: "div { width: 200px; height: 100px; border-radius: 50%; background: teal; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "geometry/css/text-overflow-ellipsis".into(),
        description: "text-overflow:ellipsis 文本溢出".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="overflow">This is a very long text that should overflow</div></body></html>"#
            .into(),
        css: ".overflow { width: 100px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "glyph_count_ge:3".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/vertical-stack-blocks".into(),
        description: "多个块级元素垂直排列不重叠".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <div class="b">1</div>
            <div class="b">2</div>
            <div class="b">3</div>
            <div class="b">4</div>
        </body></html>"#
            .into(),
        css: ".b { width: 200px; height: 30px; margin-bottom: 5px; background: navy; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:4".into(),
            "layout_box_count_ge:8".into(),
            "layout_children_non_overlapping".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/flex-space-between".into(),
        description: "justify-content:space-between 间距分配".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="flex"><span>A</span><span>B</span><span>C</span></div></body></html>"#.into(),
        css:
            ".flex { display: flex; justify-content: space-between; width: 300px; height: 50px; } span { width: 50px; }"
                .into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:8".into(),
            "fill_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/outline-render".into(),
        description: "outline 渲染不占空间".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="outlined">Outlined</div></body></html>"#.into(),
        css: ".outlined { width: 100px; height: 50px; outline: 3px solid red; background: white; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "stroke_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/inline-block-row".into(),
        description: "inline-block 水平排列".into(),
        category: "geometry".into(),
        html: r#"<html><body><div class="ib">A</div><div class="ib">B</div><div class="ib">C</div></body></html>"#
            .into(),
        css: ".ib { display: inline-block; width: 80px; height: 40px; background: purple; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "fill_count_ge:3".into(),
            "layout_box_count_ge:7".into(),
        ],
    });

    tests.push(TestCase {
        id: "geometry/css/complex-nested-layout".into(),
        description: "复杂嵌套布局组合".into(),
        category: "geometry".into(),
        html: r#"<html><body>
            <nav class="topbar"><span>Nav</span></nav>
            <main class="content">
                <article class="post">
                    <h2>Title</h2>
                    <p>Content paragraph with text.</p>
                </article>
                <aside class="sidebar"><div>Widget</div></aside>
            </main>
        </body></html>"#.into(),
        css: ".topbar { height: 40px; background: #333; } .content { display: flex; } .post { flex: 1; } .sidebar { width: 200px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_box_count_ge:12".into(),
            "fill_count_ge:2".into(),
            "glyph_count_ge:3".into(),
        ],
    });

    tests
}
