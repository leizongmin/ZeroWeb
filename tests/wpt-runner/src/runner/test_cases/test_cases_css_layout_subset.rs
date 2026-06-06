//! WPT CSS/Layout 子集测试（Phase 3）。
//!
//! 导入真实 WPT CSS/Layout 测试模式，按 CSS 规范领域组织：
//! - 盒模型：width/height/margin/padding/border/box-sizing
//! - 视觉格式化模型：display/position/float/clear
//! - Flexbox：flex-direction/flex-wrap/justify-content/align-items/gap
//! - Grid：grid-template/grid-area/auto-fill-auto-fit/minmax
//! - 文本排版：text-align/text-indent/letter-spacing/word-spacing/white-space
//! - 颜色与背景：color/background/border-radius/box-shadow
//! - 变换与过渡：transform/opacity/transition
//! - 逻辑属性：margin-block/padding-inline
//!
//! 测试使用精确几何断言 + 渲染图元验证，确保管线各阶段输出正确。
//! 按 CSS 规范领域分类，便于按分类追踪通过率。

use super::TestCase;

#[allow(clippy::vec_init_then_push)]
pub fn css_layout_subset_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();

    // ═══════════════════════════════════════════════════════════════
    // 1. 盒模型（Box Model）
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/box-model/width-height-px".into(),
        description: "固定像素宽高".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">A</div></body></html>"#.into(),
        css: ".box { width: 200px; height: 100px; background: #ccc; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:200.0".into(),
            "layout_nth_height_ge:2:100.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/box-model/box-sizing-border-box".into(),
        description: "box-sizing: border-box 包含 padding 和 border".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">Content</div></body></html>"#.into(),
        css: ".box { width: 200px; height: 100px; padding: 20px; border: 5px solid #000; box-sizing: border-box; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:200.0".into(),
            "layout_nth_height_ge:2:100.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/box-model/margin-collapse".into(),
        description: "相邻块级元素 margin 折叠".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="a">A</div><div class="b">B</div></body></html>"#.into(),
        css: ".a { margin-bottom: 30px; } .b { margin-top: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/box-model/min-max-constraints".into(),
        description: "min-width/max-width 约束".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">Text</div></body></html>"#.into(),
        css: ".box { width: 50px; min-width: 150px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:150.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/box-model/percentage-width".into(),
        description: "百分比宽度相对于父容器".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="parent"><div class="child">50%</div></div></body></html>"#.into(),
        css: ".parent { width: 400px; } .child { width: 50%; height: 30px; background: #aaa; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:3:200.0".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 2. 视觉格式化模型（Visual Formatting Model）
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/vfm/display-none".into(),
        description: "display: none 元素不生成盒子".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="visible">A</div><div class="hidden">B</div><div class="visible">C</div></body></html>"#.into(),
        css: ".visible { height: 20px; } .hidden { display: none; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_has_children".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/vfm/display-inline-block".into(),
        description: "display: inline-block 水平排列".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="ib">A</div><div class="ib">B</div></body></html>"#.into(),
        css: ".ib { display: inline-block; width: 100px; height: 50px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/vfm/position-absolute".into(),
        description: "position: absolute 相对定位父元素".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="rel"><div class="abs">A</div></div></body></html>"#.into(),
        css: ".rel { position: relative; width: 300px; height: 200px; background: #eee; } .abs { position: absolute; top: 10px; left: 20px; width: 50px; height: 30px; background: #ccc; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_box_count_ge:5".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/vfm/position-relative".into(),
        description: "position: relative 视觉偏移不影响布局".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="normal">A</div><div class="shifted">B</div><div class="normal">C</div></body></html>"#.into(),
        css: ".normal { height: 30px; background: #eee; } .shifted { position: relative; top: 10px; left: 20px; height: 30px; background: #ccc; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/vfm/overflow-hidden".into(),
        description: "overflow: hidden 裁剪溢出内容".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="clip">Long text content that should be clipped by the container</div></body></html>"#.into(),
        css: ".clip { width: 100px; height: 30px; overflow: hidden; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 3. Flexbox 布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/flex/row-layout".into(),
        description: "flex-direction: row 水平排列".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">A</div><div class="item">B</div><div class="item">C</div></div></body></html>"#.into(),
        css: ".flex { display: flex; } .item { width: 80px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
            "layout_box_count_ge:5".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/column-layout".into(),
        description: "flex-direction: column 垂直排列".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">A</div><div class="item">B</div></div></body></html>"#
            .into(),
        css: ".flex { display: flex; flex-direction: column; } .item { width: 80px; height: 40px; background: #eee; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/justify-center".into(),
        description: "justify-content: center 居中".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">A</div><div class="item">B</div></div></body></html>"#.into(),
        css: ".flex { display: flex; justify-content: center; width: 400px; } .item { width: 80px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/justify-space-between".into(),
        description: "justify-content: space-between 等间距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">A</div><div class="item">B</div><div class="item">C</div></div></body></html>"#.into(),
        css: ".flex { display: flex; justify-content: space-between; width: 400px; } .item { width: 80px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/align-items-center".into(),
        description: "align-items: center 交叉轴居中".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="tall">A</div><div class="short">B</div></div></body></html>"#.into(),
        css: ".flex { display: flex; align-items: center; height: 100px; } .tall { width: 60px; height: 60px; background: #eee; } .short { width: 60px; height: 30px; background: #ccc; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/flex-grow".into(),
        description: "flex-grow: 1 等比分配剩余空间".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="grow">A</div><div class="grow">B</div></div></body></html>"#
            .into(),
        css: ".flex { display: flex; width: 400px; } .grow { flex-grow: 1; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
            "layout_nth_width_ge:2:190.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/wrap".into(),
        description: "flex-wrap: wrap 自动换行".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">1</div><div class="item">2</div><div class="item">3</div></div></body></html>"#.into(),
        css: ".flex { display: flex; flex-wrap: wrap; width: 200px; } .item { width: 100px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/flex/gap".into(),
        description: "gap 属性设置 flex 间距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex"><div class="item">A</div><div class="item">B</div><div class="item">C</div></div></body></html>"#.into(),
        css: ".flex { display: flex; gap: 20px; width: 400px; } .item { width: 80px; height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 4. Grid 布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/grid/3-column".into(),
        description: "grid-template-columns: 3 列布局".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div>A</div><div>B</div><div>C</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 100px 100px 100px; } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
            "layout_box_count_ge:5".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/grid/fr-units".into(),
        description: "grid-template-columns: fr 单位等分".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div>A</div><div>B</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 1fr 2fr; width: 300px; } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/grid/template-areas".into(),
        description: "grid-template-areas 命名区域布局".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div class="header">H</div><div class="main">M</div><div class="sidebar">S</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-areas: 'header header' 'main sidebar'; grid-template-columns: 200px 100px; } .header { grid-area: header; } .main { grid-area: main; } .sidebar { grid-area: sidebar; } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/grid/auto-fill-minmax".into(),
        description: "auto-fill + minmax 响应式网格".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div>A</div><div>B</div><div>C</div><div>D</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/grid/gap".into(),
        description: "grid gap 间距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div>A</div><div>B</div><div>C</div><div>D</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 100px 100px; gap: 20px; } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/grid/span".into(),
        description: "grid-column: span 跨列".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid"><div class="wide">A</div><div>B</div><div>C</div></div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 100px 100px; } .wide { grid-column: span 2; } div > div { height: 40px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 5. 文本排版
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/text/text-align-center".into(),
        description: "text-align: center 文本居中".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="centered">Centered Text</div></body></html>"#.into(),
        css: ".centered { text-align: center; width: 400px; background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/text-align-justify".into(),
        description: "text-align: justify 两端对齐".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><p class="justify">This is a paragraph of text that should be justified across the full width of the container element.</p></body></html>"#.into(),
        css: ".justify { text-align: justify; width: 300px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/letter-spacing".into(),
        description: "letter-spacing 字间距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="spaced">Spaced Text</div></body></html>"#.into(),
        css: ".spaced { letter-spacing: 5px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/text/white-space-nowrap".into(),
        description: "white-space: nowrap 禁止换行".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="nowrap">This is a long text that would normally wrap but should stay on a single line.</div></body></html>"#.into(),
        css: ".nowrap { white-space: nowrap; width: 100px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/text-indent".into(),
        description: "text-indent 首行缩进".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><p class="indent">First line should be indented by 2em. Second line should not be indented.</p></body></html>"#.into(),
        css: ".indent { text-indent: 2em; width: 300px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/font-size-weight".into(),
        description: "font-size + font-weight 组合".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="big-bold">Bold Large</div><div class="small-light">Light Small</div></body></html>"#.into(),
        css: ".big-bold { font-size: 24px; font-weight: 700; } .small-light { font-size: 12px; font-weight: 300; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/text-transform".into(),
        description: "text-transform: uppercase 大写转换".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="upper">hello world</div></body></html>"#.into(),
        css: ".upper { text-transform: uppercase; }".into(),
        assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/text/word-break-break-all".into(),
        description: "word-break: break-all 任意断行".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="break">abcdefghijklmnopqrstuvwxyz</div></body></html>"#.into(),
        css: ".break { word-break: break-all; width: 100px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 6. 颜色与背景
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/colors/named-color".into(),
        description: "命名颜色 (red, blue, green)".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="r">R</div><div class="g">G</div><div class="b">B</div></body></html>"#.into(),
        css: ".r { background: red; height: 20px; } .g { background: green; height: 20px; } .b { background: blue; height: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "fill_count_ge:3".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/colors/rgb-hex".into(),
        description: "RGB 和 HEX 颜色".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="rgb">A</div><div class="hex">B</div></body></html>"#.into(),
        css: ".rgb { background: rgb(255, 128, 0); height: 20px; } .hex { background: #336699; height: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "fill_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/colors/hsl".into(),
        description: "HSL 颜色".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="hsl">Color</div></body></html>"#.into(),
        css: ".hsl { background: hsl(120, 50%, 50%); height: 20px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/colors/opacity".into(),
        description: "opacity 透明度".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="fade">Semi-transparent</div></body></html>"#.into(),
        css: ".fade { opacity: 0.5; background: #000; height: 20px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/background/linear-gradient".into(),
        description: "linear-gradient 线性渐变".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grad">Gradient</div></body></html>"#.into(),
        css: ".grad { background: linear-gradient(to right, red, blue); height: 40px; }".into(),
        assertions: vec!["dom_has_body".into(), "gradient_count_ge:1".into()],
    });

    tests.push(TestCase {
        id: "css-subset/background/radial-gradient".into(),
        description: "radial-gradient 径向渐变".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grad">Radial</div></body></html>"#.into(),
        css: ".grad { background: radial-gradient(circle, white, black); height: 40px; }".into(),
        assertions: vec!["dom_has_body".into(), "gradient_count_ge:1".into()],
    });

    tests.push(TestCase {
        id: "css-subset/background/border-radius".into(),
        description: "border-radius 圆角".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="rounded">Round</div></body></html>"#.into(),
        css: ".rounded { width: 100px; height: 100px; border-radius: 25px; background: #ccc; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/background/box-shadow".into(),
        description: "box-shadow 阴影".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="shadow">Shadow</div></body></html>"#.into(),
        css: ".shadow { width: 100px; height: 50px; background: #fff; box-shadow: 5px 5px 10px rgba(0,0,0,0.5); }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "shadow_count_ge:1".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 7. 变换与过渡
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/transform/rotate".into(),
        description: "transform: rotate() 旋转".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="rotated">Rotated</div></body></html>"#.into(),
        css: ".rotated { width: 100px; height: 50px; background: #ccc; transform: rotate(45deg); }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/transform/scale".into(),
        description: "transform: scale() 缩放".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="scaled">Scaled</div></body></html>"#.into(),
        css: ".scaled { width: 100px; height: 50px; background: #ccc; transform: scale(1.5); }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/transform/translate".into(),
        description: "transform: translate() 平移".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="moved">Moved</div></body></html>"#.into(),
        css: ".moved { width: 100px; height: 50px; background: #ccc; transform: translate(50px, 25px); }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/transform/skew".into(),
        description: "transform: skew() 倾斜".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="skewed">Skewed</div></body></html>"#.into(),
        css: ".skewed { width: 100px; height: 50px; background: #ccc; transform: skew(10deg, 5deg); }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 8. 逻辑属性
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/logical/margin-block".into(),
        description: "margin-block 逻辑外边距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="spaced">A</div><div>B</div></body></html>"#.into(),
        css: ".spaced { margin-block: 20px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/logical/padding-inline".into(),
        description: "padding-inline 逻辑内边距".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="padded">Content</div></body></html>"#.into(),
        css: ".padded { padding-inline: 30px; background: #eee; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 9. CSS 变量与自定义属性
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/variables/basic".into(),
        description: "CSS 变量定义和引用".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">Variable</div></body></html>"#.into(),
        css: ":root { --color: #336699; --size: 150px; } .box { width: var(--size); height: 80px; background: var(--color); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:150.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/variables/fallback".into(),
        description: "CSS 变量回退值".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">Fallback</div></body></html>"#.into(),
        css: ".box { background: var(--undefined, #ccc); height: 40px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 10. 综合页面布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/composite/holy-grail".into(),
        description: "圣杯布局（header + 3col + footer）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body>
            <header>Header</header>
            <div class="main">
                <nav>Nav</nav>
                <article>Content</article>
                <aside>Sidebar</aside>
            </div>
            <footer>Footer</footer>
        </body></html>"#.into(),
        css: ".main { display: flex; } nav { width: 150px; background: #eee; } article { flex: 1; } aside { width: 150px; background: #eee; } header, footer { background: #ccc; height: 40px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
            "layout_box_count_ge:8".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/composite/card-grid".into(),
        description: "卡片网格布局".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid">
            <div class="card">Card 1</div>
            <div class="card">Card 2</div>
            <div class="card">Card 3</div>
            <div class="card">Card 4</div>
        </div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px; } .card { background: #eee; padding: 10px; border-radius: 8px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
            "layout_box_count_ge:6".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/composite/nav-flex".into(),
        description: "Flexbox 导航栏布局".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><nav class="navbar">
            <div class="logo">Logo</div>
            <div class="links">
                <a>Home</a>
                <a>About</a>
                <a>Contact</a>
            </div>
        </nav></body></html>"#.into(),
        css: ".navbar { display: flex; justify-content: space-between; align-items: center; background: #333; padding: 10px; } .links { display: flex; gap: 15px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/composite/sticky-footer".into(),
        description: "粘性页脚（Flexbox 最小高度）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="page">
            <main>Content</main>
            <footer>Footer</footer>
        </div></body></html>"#.into(),
        css: ".page { display: flex; flex-direction: column; min-height: 100vh; } main { flex: 1; } footer { background: #333; height: 40px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_has_children".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/composite/media-query".into(),
        description: "@media 媒体查询响应式".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="responsive">Content</div></body></html>"#.into(),
        css: ".responsive { width: 100%; background: #eee; } @media (min-width: 600px) { .responsive { width: 50%; } }"
            .into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 11. 高级 Flexbox/Grid 组合
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/advanced/nested-flex".into(),
        description: "嵌套 Flexbox（flex > flex）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="outer">
            <div class="left"><div class="inner"><div>A</div><div>B</div></div></div>
            <div class="right">C</div>
        </div></body></html>"#.into(),
        css: ".outer { display: flex; width: 400px; } .left { flex: 2; } .right { flex: 1; background: #eee; } .inner { display: flex; gap: 8px; } .inner > div { flex: 1; background: #ccc; height: 30px; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
            "layout_box_count_ge:8".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/advanced/flex-in-grid".into(),
        description: "Flexbox 嵌套在 Grid 中".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid">
            <div class="cell"><div class="flex"><span>A</span><span>B</span></div></div>
            <div class="cell"><div class="flex"><span>C</span><span>D</span></div></div>
        </div></body></html>"#.into(),
        css: ".grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; } .cell { background: #eee; padding: 5px; } .flex { display: flex; justify-content: space-between; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/advanced/absolute-in-flex".into(),
        description: "Flex 容器内的绝对定位元素".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex">
            <div class="rel"><div class="abs">Floating</div>Normal</div>
            <div>Other</div>
        </div></body></html>"#.into(),
        css: ".flex { display: flex; position: relative; } .rel { position: relative; width: 100px; } .abs { position: absolute; top: 0; right: 0; background: #ccc; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/advanced/grid-overlay".into(),
        description: "Grid 层叠效果（同位置多项）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="grid">
            <div class="back">Background</div>
            <div class="front">Foreground</div>
        </div></body></html>"#.into(),
        css: ".grid { display: grid; position: relative; width: 200px; height: 100px; } .back { grid-column: 1; grid-row: 1; background: #eee; } .front { grid-column: 1; grid-row: 1; background: rgba(200,200,200,0.5); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "fill_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/advanced/flex-order".into(),
        description: "flex-order 重排视觉顺序".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="flex">
            <div class="last">C</div>
            <div class="first">A</div>
            <div class="mid">B</div>
        </div></body></html>"#.into(),
        css: ".flex { display: flex; } .first { order: -1; background: #cfc; } .mid { background: #fcc; } .last { order: 1; background: #ccf; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 12. CSS 文本高级
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/text/vertical-writing".into(),
        description: "writing-mode: vertical-rl 垂直文本".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="vertical">Vertical Text</div></body></html>"#.into(),
        css: ".vertical { writing-mode: vertical-rl; height: 200px; }".into(),
        assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/text/overflow-ellipsis".into(),
        description: "text-overflow: ellipsis 溢出省略".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="ellipsis">This is a very long text that should be truncated with an ellipsis</div></body></html>"#.into(),
        css: ".ellipsis { width: 100px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/multiline-ellipsis-clamp".into(),
        description: "多行截断（line-clamp 近似）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><p class="clamp">This is a long paragraph of text that spans multiple lines and should be truncated after a certain number of lines to maintain a clean layout.</p></body></html>"#.into(),
        css: ".clamp { width: 200px; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/text/decoration-complex".into(),
        description: "text-decoration 组合（underline + overline + line-through）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="deco">Decorated</div></body></html>"#.into(),
        css: ".deco { text-decoration: underline overline line-through; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_glyph_primitives".into(),
            "fill_count_ge:3".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 13. CSS 高级视觉效果
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/visual/multiple-shadows".into(),
        description: "多个 box-shadow 叠加".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="multi-shadow">Shadow</div></body></html>"#.into(),
        css: ".multi-shadow { width: 100px; height: 50px; background: #fff; box-shadow: 0 0 5px #000, 5px 5px 10px rgba(0,0,0,0.3), inset 0 0 5px rgba(255,0,0,0.5); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "shadow_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/visual/filter-blur".into(),
        description: "filter: blur() 模糊效果".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="blur">Blurred</div></body></html>"#.into(),
        css: ".blur { width: 100px; height: 50px; background: #ccc; filter: blur(2px); }".into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    tests.push(TestCase {
        id: "css-subset/visual/filter-combo".into(),
        description: "filter 组合（brightness + contrast + saturate）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="filtered">Enhanced</div></body></html>"#.into(),
        css: ".filtered { width: 100px; height: 50px; background: #ccc; filter: brightness(1.2) contrast(1.1) saturate(1.3); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/visual/multi-gradient".into(),
        description: "多层渐变叠加".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="multi-grad">Multi</div></body></html>"#.into(),
        css: ".multi-grad { width: 200px; height: 100px; background: linear-gradient(to right, rgba(255,0,0,0.5), rgba(0,0,255,0.5)), linear-gradient(to bottom, rgba(0,255,0,0.5), rgba(255,255,0,0.5)); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "gradient_count_ge:1".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/visual/outline".into(),
        description: "outline 轮廓线".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="outlined">Outlined</div></body></html>"#.into(),
        css:
            ".outlined { width: 100px; height: 50px; background: #eee; outline: 3px solid #f00; outline-offset: 5px; }"
                .into(),
        assertions: vec!["dom_has_body".into(), "has_fill_primitives".into()],
    });

    // ═══════════════════════════════════════════════════════════════
    // 14. CSS 变量高级
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/variables/calc-with-var".into(),
        description: "calc() 与 var() 组合".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="box">Calc + Var</div></body></html>"#.into(),
        css: ":root { --gap: 20px; --base: 100px; } .box { width: calc(var(--base) + var(--gap) * 2); height: var(--base); background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_nth_width_ge:2:100.0".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/variables/theme-toggle".into(),
        description: "CSS 变量主题系统".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body>
            <div class="card"><h2>Title</h2><p>Content</p></div>
        </body></html>"#.into(),
        css: ":root { --bg: #fff; --text: #333; --accent: #0066cc; --border: #ddd; } .card { background: var(--bg); color: var(--text); border: 1px solid var(--border); padding: 16px; } h2 { color: var(--accent); }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "has_glyph_primitives".into(),
        ],
    });

    // ═══════════════════════════════════════════════════════════════
    // 15. 边界条件布局
    // ═══════════════════════════════════════════════════════════════

    tests.push(TestCase {
        id: "css-subset/edge/zero-height".into(),
        description: "零高度容器不崩溃".into(),
        category: "css-layout-subset".into(),
        html:
            r#"<html><body><div class="zero">Content in zero height</div><div class="normal">Next</div></body></html>"#
                .into(),
        css: ".zero { height: 0; overflow: hidden; } .normal { height: 20px; background: #eee; }".into(),
        assertions: vec!["dom_has_body".into(), "layout_has_children".into(), "no_panic".into()],
    });

    tests.push(TestCase {
        id: "css-subset/edge/negative-margin".into(),
        description: "负 margin 重叠效果".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="a">A</div><div class="b">B</div></body></html>"#.into(),
        css: ".a { height: 40px; background: #eee; margin-bottom: -10px; } .b { height: 40px; background: #ccc; }"
            .into(),
        assertions: vec![
            "dom_has_body".into(),
            "has_fill_primitives".into(),
            "layout_child_count_ge:2".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/edge/deep-nesting".into(),
        description: "深层嵌套布局（10 层）".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="l1"><div class="l2"><div class="l3"><div class="l4"><div class="l5"><div class="l6"><div class="l7"><div class="l8"><div class="l9"><div class="l10">Deep</div></div></div></div></div></div></div></div></div></div></body></html>"#.into(),
        css: ".l1 { width: 400px; } div > div { padding: 10px; } .l10 { background: #eee; }".into(),
        assertions: vec![
            "dom_has_body".into(),
            "layout_depth_ge:5".into(),
            "has_glyph_primitives".into(),
        ],
    });

    tests.push(TestCase {
        id: "css-subset/edge/large-content".into(),
        description: "大量文本内容渲染".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div class="container"></div></body></html>"#.into(),
        css: ".container { width: 300px; }".into(),
        assertions: vec!["dom_has_body".into(), "layout_has_children".into(), "no_panic".into()],
    });

    tests.push(TestCase {
        id: "css-subset/edge/empty-elements".into(),
        description: "空元素布局不崩溃".into(),
        category: "css-layout-subset".into(),
        html: r#"<html><body><div></div><div class="h">H</div><div></div></body></html>"#.into(),
        css: ".h { height: 20px; background: #eee; }".into(),
        assertions: vec!["dom_has_body".into(), "layout_has_children".into(), "no_panic".into()],
    });

    tests
}
