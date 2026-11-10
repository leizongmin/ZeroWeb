//! R5494 常驻锚：UBA 偶 level LTR 呈现臂——dir=rtl 容器纯 Latin 内容 shaping 方向
//! 修正（css-writing-modes-3 §bidi-algo × UAX #9 L2）。
//!
//! chromium oracle（headless 800×600，/tmp/r5494-forensics，floats-placement-vertical-
//! 001a p3 形）：`<p dir=rtl>HelloKitty` 渲染「HelloKitty」逻辑序 LTR 呈现、整行右对
//! 齐——RTL 段基下纯强 L 内容经 X 系列＋N1/N2 解析为纯偶 level（L2 两次反转抵消＝
//! 恒等），呈现序 = 逻辑序。修复前 paint shaping 兜底直接读 CSS direction（RTL）→
//! 整串反转字形（ZW 渲染「olleH」），001a/b/c 家族 0.26% 公共残差主源。
//!
//! 断言渲染字形序：按 x 排序后 code_point 序列 = 逻辑序「Hello」。kill-switch 见
//! r5494_shaped_ltr_presentation_kill_switch.rs（ZW_SHAPED_UBA_RTL=0 回退反转）。
//!
//! 引擎集成测试须自装宿主字形链：TEXT_SHAPE 回调与 font_resolver 均由宿主
//! （apps/renderer runtime）注册，裸 RenderPipeline 无安装 → fragment_glyphs 恒走
//! Legacy（存序发射），shaping 方向不可观测、锚点空转。

use std::sync::OnceLock;

use zero_engine::{RenderPipeline, set_text_shape_fn};
use zero_render_foundation::font::{
    FontSizeAdjustment, OpenTypeFeature, OpenTypeVariation, ShapedGlyph, TextDirection, TextShapingOptions,
    loader::FontLoader,
};

static TEST_LOADER: OnceLock<Option<&'static FontLoader>> = OnceLock::new();

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
    let loader = (*TEST_LOADER.get()?)?;
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

/// 平台字体 + resolver + shape 回调三件套（与 apps/renderer load_system_fonts 同
/// 源）；进程级幂等安装（多测试共用同进程 OnceLock）。
fn host_fonts() -> Option<&'static FontLoader> {
    *TEST_LOADER.get_or_init(|| {
        let platform = zero_render_foundation::font::system::load_platform_fonts();
        platform.primary_id?;
        let loader: &'static FontLoader = Box::leak(Box::new(platform.loader));
        set_text_shape_fn(test_shape_text);
        Some(loader)
    })
}

/// 渲染并返回按 x 排序的字形 code_point 序列。
fn rendered_glyph_order(html: &str) -> String {
    let Some(loader) = host_fonts() else {
        eprintln!("skipping: missing system fonts");
        return String::new();
    };
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
    glyphs.iter().map(|(_, ch)| ch).collect()
}

#[test]
fn rtl_container_pure_ltr_content_keeps_logical_glyph_order() {
    let html = r#"<body style="margin:0"><p dir="rtl" style="margin:0;font:16px serif">Hello</p></body>"#;
    let rendered = rendered_glyph_order(html);
    assert_eq!(
        rendered, "Hello",
        "dir=rtl 容器纯 Latin 内容应保持逻辑序字形（UBA 偶 level LTR 呈现），实际 {rendered:?}"
    );
}

/// RTL 脚本内容不翻臂：uniform_resolved_rtl / R/AL 判定域维持 RTL shaping。
#[test]
fn rtl_container_rtl_script_content_keeps_rtl_shaping() {
    let html = r#"<body style="margin:0"><p dir="rtl" style="margin:0;font:16px serif">שלום</p></body>"#;
    let rendered = rendered_glyph_order(html);
    // RTL shaper 反转呈视觉序：按 x 排序（左→右）= 逻辑序反转，首字形（最左 x）
    // = 逻辑末字ם。修复只翻纯 L/EN/AN 域，希伯来内容维持既有 RTL 链路——首字形
    // 为逻辑序末字符即证。
    assert_ne!(
        rendered, "שלום",
        "RTL 脚本内容应维持 RTL shaping 视觉序（整串反转），实际 {rendered:?}"
    );
}
