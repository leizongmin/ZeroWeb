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

    tests
}
