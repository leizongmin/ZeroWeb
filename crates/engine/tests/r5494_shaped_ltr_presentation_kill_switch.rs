//! R5494 kill-switch 回退锚（独立测试进程——`ZW_SHAPED_UBA_RTL=0` 整体回退 UBA
//! 方向臂：偶 level LTR 呈现臂与 uniform_resolved_rtl 臂同开关，回退 CSS direction
//! 兜底旧径；进程内 set_var 会与并行单测互踩，R3858 教训）。
//!
//! 回退契约：dir=rtl 容器纯 Latin 内容重新走 RTL shaping → 字形整串反转
//! （「Hello」→「olleH」，floats-placement-vertical-001a/b/c 家族 0.26% 残差旧径，
//! chromium oracle 实证正确呈现为逻辑序「HelloKitty」）。
//!
//! 引擎集成测试须自装宿主字形链：TEXT_SHAPE 回调与 font_resolver 均由宿主
//! （apps/renderer runtime）注册，裸 RenderPipeline 无安装 → fragment_glyphs 恒走
//! Legacy（存序发射），shaping 方向不可观测。

use std::sync::OnceLock;

use zero_engine::{RenderPipeline, set_text_shape_fn};
use zero_render_foundation::font::{
    FontSizeAdjustment, OpenTypeFeature, OpenTypeVariation, ShapedGlyph, TextDirection, TextShapingOptions,
    loader::FontLoader,
};

static TEST_LOADER: OnceLock<&'static FontLoader> = OnceLock::new();

/// 宿主 shape_text 的测试直连替身（renderer 的 MEASURE_CTX thread-local 在引擎
/// 测试进程不可复用，直连静态 loader）。
fn test_shape_text(
    font_ids: &[u32],
    text: &str,
    font_size: f32,
    direction: TextDirection,
    features: &[OpenTypeFeature],
    variations: &[OpenTypeVariation],
    adjustment: FontSizeAdjustment,
) -> Option<Vec<ShapedGlyph>> {
    let loader = TEST_LOADER.get()?;
    loader.shape_text_cached_with_font_ids_and_options(
        font_ids,
        text,
        font_size,
        TextShapingOptions {
            direction,
            features,
            variations,
            adjustment,
            language: None,
        },
    )
}

#[test]
fn kill_switch_restores_rtl_shaping_for_pure_ltr_content() {
    let platform = zero_render_foundation::font::system::load_platform_fonts();
    if platform.primary_id.is_none() {
        eprintln!("skipping: missing system fonts");
        return;
    }
    let loader: &'static FontLoader = Box::leak(Box::new(platform.loader));
    assert!(TEST_LOADER.set(loader).is_ok(), "loader 单进程单实例");
    set_text_shape_fn(test_shape_text);
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读缓存于首次
    // render，须在渲染前置 0。
    unsafe { std::env::set_var("ZW_SHAPED_UBA_RTL", "0") };
    let html = r#"<body style="margin:0"><p dir="rtl" style="margin:0;font:16px serif">Hello</p></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_font_resolver(loader.build_font_resolver());
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    let mut glyphs: Vec<(f32, char)> = p
        .glyphs
        .iter()
        .filter_map(|g| g.code_point().map(|ch| (g.x, ch)))
        .collect();
    glyphs.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("有限 x 坐标"));
    let rendered: String = glyphs.iter().map(|(_, ch)| ch).collect();
    assert_eq!(
        rendered, "olleH",
        "kill-switch 关闭应回退 CSS direction 兜底（RTL shaping 整串反转，001a 家族 0.26% 残差旧径），实际 {rendered:?}"
    );
}
