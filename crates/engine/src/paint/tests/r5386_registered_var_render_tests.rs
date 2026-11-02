//! R5386（css-properties-values-api-1 §2 + css-variables-1 §2）：`@property` 注册
//! 初值在**管线渲染路径**的 var() 解析。render_html 开头的文本级 var() 替换
//!（bing-t5，css_vars）此前对未注册到文本表的名字直接替换 fallback
//!（`var(--bg, red)` → red），抢在级联层之前——注册初值（initial-value）永无
//! 兜底机会（highlight-cascade-004：选区绿被替红）。修复 = 注册名 var() 留
//! 级联层（skip，R5106 冲突门同构），initial-value / 显式声明 / 继承按级联
//! 语义解析。driving: highlight-cascade-004。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 未显式声明时注册初值兜底（有 fallback 形态——修复目标形态）：div 背景
/// `var(--bg, red)` + `@property --bg { initial-value: green }` → 绿非红。
#[test]
fn registered_initial_beats_fallback_normal_bg() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div style="font: 40px Ahem; width: 200px; height: 60px">plain</div></body>"#,
        r#"@property --bg { syntax: "<color>"; inherits: false; initial-value: green; }
div { background-color: var(--bg, red); }"#,
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "注册初值 green 应兜底（fallback red 不应被文本层抢跑），实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(255, 0, 0, 255)),
        "fallback red 不应出现在任何 fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 注册初值在 ::selection 高亮路径（highlight-cascade-004 语义，span owner 形——
/// WPT 案面的 div 直含文本形态由 corpus reftest 直接钉）：`span::selection
/// { background-color: var(--bg, red) }` → 选区带绿非红。
#[test]
fn registered_initial_beats_fallback_selection_bg() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#t".to_string(), 0, 8)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div style="font: 40px Ahem"><span id="t" style="font: 40px Ahem">selected</span> rest</div></body>"#,
        r#"@property --bg { syntax: "<color>"; inherits: false; initial-value: green; }
span::selection { background-color: var(--bg, red); }"#,
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "选区带应铺注册初值 green，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(255, 0, 0, 255)),
        "fallback red 不应出现，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 显式声明压过注册初值（skip 后级联按元素解析的语义锚）：`--bg: blue` 在场
/// → 蓝非绿非红（文本级替换不走，级联显式声明胜）。
#[test]
fn explicit_declaration_beats_registered_initial() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div style="font: 40px Ahem; width: 200px; height: 60px">plain</div></body>"#,
        r#"@property --bg { syntax: "<color>"; inherits: false; initial-value: green; }
div { --bg: blue; background-color: var(--bg, red); }"#,
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 0, 255, 255)),
        "显式 --bg: blue 应压过注册初值，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "注册初值 green 不应出现在任何 fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 未注册名的 fallback 文本级替换行为不变（修复只针对注册名）：`var(--undef, red)`
/// 无注册 → 红（文本层替换 fallback，既有 bing-t5 语义保持）。
#[test]
fn unregistered_fallback_still_text_substituted() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><div style="font: 40px Ahem; width: 200px; height: 60px">plain</div></body>"#,
        "div { background-color: var(--undef, red); }",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(255, 0, 0, 255)),
        "未注册名 fallback 替换行为应不变（红），实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
