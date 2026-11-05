//! R5421（html-semantics M2 s292）：SVG 固有尺寸两处布局缺口——
//!
//! 1. **ratio-only inline both-auto 的约束方程 frame 缺口**（CSS2 §10.3.4 块级替换
//!    元素：margin-box = CB 宽 → used content 宽 = CB − 自身水平 frame）。R2054 C2
//!    把 all-auto viewBox-only img 撑满父 content 宽，但未减自身 margin/border/
//!    padding——naturalWidth-naturalHeight-width-height 18F 面（740 CB + 5px
//!    border/margin img 应 content 720×240）旧渲染 740×246.67。chromium oracle
//!    实证 content 720×240（border box 730×250）。
//! 2. **no-ratio img 的 attr 臂 bogus 比注入**（CSS Sizing 4 §4.1：无 viewBox SVG
//!    无自然比）：img_intrinsic_no_ratio 与 bogus 解码 pixmap 并存时，
//!    (Some(w),None)/(None,Some(h)) attr 臂 R2172 注入 pixmap 比——density-vs-zoom
//!    "intrinsic width only: baseline"（width=60 SVG 应 60×150，bogus 比 60/100
//!    → 60×100）。修复：no_ratio 键命中时跳比注入，缺轴（CSS auto）落 default
//!    object size 对应轴（300/150，SVG_DEFAULT_W/H 同源）。
//! 3. **s292 L1：no-ratio img 的 s283 固有维注入误读 bogus pixmap**：零维 viewBox
//!    （R3760 早退 1×1）形态 sizes 表存 1×1、no_ratio 存真维 (Some(60),None)——
//!    s283 auto 态注入旧读 sizes → 内容宽 1（probe 11×160）。修复：noratio img
//!    注入先取 no_ratio 真维（密度折算同通道口径），无真维保持 auto。
//!
//! 绿色 background fill rect = border-box 几何（background painting area）。
//! kill-switch：`ZW_RATIO_CONSTRAINT_FRAME=0`（缺口 1）、`ZW_NORATIO_ATTR_GUARD=0`
//! （缺口 2）；OFF 轮由 corpus 单案精确回退覆盖（R3858 口径）。

use std::collections::HashMap;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;

/// viewBox-only SVG（RatioOnly：仅 intrinsic_ratio 进 ratios map，无 sizes 条目，
/// decode_svg_bytes 同构）。viewBox 600×200 → ratio 3。
const RATIO_SRC: &str = "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 600 200\"></svg>";

/// width-only 无 viewBox SVG（NoRatio{Some(60),None}：no_ratio + bogus usvg pixmap
/// (60,100) 并存——生产 image_snapshot 侧 pixmap 仍留 image_sizes 供背景图读）。
const WIDTH_ONLY_SRC: &str = "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\"></svg>";

/// height-only 无 viewBox SVG（NoRatio{None,Some(60)}：no_ratio + bogus pixmap
/// (100,60) 并存）。
const HEIGHT_ONLY_SRC: &str = "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" height=\"60\"></svg>";

/// width-only + viewBox SVG（ComputedIntrinsic(60,20)：usvg 按 viewBox 比补轴，
/// 真实固有尺寸进 image_sizes，无 no_ratio）。
const COMPUTED_SRC: &str =
    "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\" viewBox=\"0 0 600 200\"></svg>";

/// width=60 + 零维 viewBox SVG（R3760 早退：decode 1×1 pixmap 进 image_sizes +
/// NoRatio{Some(60),None} 进 no_ratio——naturalWidth 语料 vb0w 驱动形）。
const ZERO_VB_SRC: &str =
    "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\" viewBox=\"0 0 0 200\"></svg>";

/// 渲染并返回绿色 background fill（img）的 border-box rect (x, y, w, h)。
/// `key` = `simple_hash(SRC.trim())`（image 资源键，r5411/r5418 同构）。
fn render_green(src: &str, body_html: String, setup: impl FnOnce(&mut RenderPipeline, u64)) -> (f32, f32, f32, f32) {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    setup(&mut pipeline, simple_hash(src.trim()));
    let result = pipeline.render_html(&body_html, "");
    let green = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.color.a == 255)
        .expect("img background fill");
    (
        green.rect.origin.x,
        green.rect.origin.y,
        green.rect.size.width,
        green.rect.size.height,
    )
}

/// ★load-bearing（naturalWidth 族 18F 驱动形）：viewBox-only img（ratio 3）+
/// div width:740 + img border 5px/margin 5px/display:block——约束方程解
/// content 720×240 → border box 730×250（chromium oracle 实证）。旧径 740×246.67
/// 不减 frame，border box 750×256.67，本针红。
#[test]
fn r5421_ratio_only_frame_subtracted_in_constraint() {
    let body = format!(
        r#"<body><div style="width:740px"><img src='{SRC}' style="border:5px solid black;margin:5px;display:block;background:#008000"></div></body>"#,
        SRC = RATIO_SRC
    );
    let (x, y, w, h) = render_green(RATIO_SRC, body, |p, key| {
        let mut ratios = HashMap::new();
        ratios.insert(key, 3.0f32);
        p.set_image_ratios(ratios);
    });
    assert!(
        (x - 13.0).abs() < 0.5,
        "左缘 = body 8 + img margin 5（水平不折叠），实测 {x}"
    );
    assert!(
        (y - 8.0).abs() < 0.5,
        "顶缘 8：img margin-top 5 与 div 塌穿（父无边/内距），并入 body 缘 max(8,5)=8（CSS2 §8.3.1），实测 {y}"
    );
    assert!(
        (w - 730.0).abs() < 0.5,
        "content 720（740 CB − margin 10 − border 10）+ border 10 → bb 730，实测 {w}"
    );
    assert!(
        (h - 250.0).abs() < 0.5,
        "content 240（720/ratio 3）+ border 10 → bb 250，实测 {h}"
    );
}

/// 守卫（R2054 C2 原形不受 frame 侵入）：ratio-only img（ratio 2）+ div width:200
/// 无 margin/border——content 200×100（frame 0 减法恒等），旧值逐字节不变。
#[test]
fn r5421_ratio_only_no_frame_unchanged() {
    let body = format!(
        r#"<body><div style="width:200px"><img src='{SRC}' style="margin:0;display:block;background:#008000"></div></body>"#,
        SRC = RATIO_SRC
    );
    let (x, y, w, h) = render_green(RATIO_SRC, body, |p, key| {
        let mut ratios = HashMap::new();
        ratios.insert(key, 2.0f32);
        p.set_image_ratios(ratios);
    });
    assert!(
        (x - 8.0).abs() < 0.5 && (y - 8.0).abs() < 0.5,
        "body 缘 8，实测 ({x},{y})"
    );
    assert!((w - 200.0).abs() < 0.5, "R2054 C2：父宽 200 减零 frame，实测 {w}");
    assert!((h - 100.0).abs() < 0.5, "200/ratio 2 → 100，实测 {h}");
}

/// ★load-bearing（density-vs-zoom "intrinsic width only: baseline" 驱动形）：
/// width=60 无 viewBox SVG 走 src attr 臂 (Some(60),None) + no_ratio(60,None) +
/// bogus pixmap sizes(60,100) 并存——旧 R2172 注入 pixmap 比 60/100 → 60×100；
/// 修复后不注入比、缺轴（height auto）落 default object size 高 150 → 60×150
///（chromium oracle：width-only SVG 高 = default 150）。
#[test]
fn r5421_noratio_attr_branch_default_axis() {
    let body = format!(
        r#"<body><img src='{SRC}' style="margin:0;display:block;background:#008000"></body>"#,
        SRC = WIDTH_ONLY_SRC
    );
    let (x, y, w, h) = render_green(WIDTH_ONLY_SRC, body, |p, key| {
        let mut sizes = HashMap::new();
        sizes.insert(key, (60.0f32, 100.0f32)); // bogus usvg pixmap（宽 60 巧同 attr）
        p.set_image_sizes(sizes);
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = HashMap::new();
        no_ratio.insert(key, (Some(60.0), None));
        p.set_image_no_ratio(no_ratio);
    });
    assert!(
        (x - 8.0).abs() < 0.5 && (y - 8.0).abs() < 0.5,
        "body 缘 8，实测 ({x},{y})"
    );
    assert!((w - 60.0).abs() < 0.5, "固有宽 60（attr/intrinsic 同值），实测 {w}");
    assert!(
        (h - 150.0).abs() < 0.5,
        "no-ratio 缺轴落 default 高 150（bogus 比 60/100 旧径 100），实测 {h}"
    );
}

/// 对称臂：height=60 无 viewBox SVG 走 (None,Some(60)) + bogus pixmap(100,60)——
/// 修复后缺轴（width auto）落 default object size 宽 300 → 300×60（旧径 bogus 比
/// 100/60 → 100×60）。
#[test]
fn r5421_noratio_attr_branch_default_axis_symmetric() {
    let body = format!(
        r#"<body><img src='{SRC}' style="margin:0;display:block;background:#008000"></body>"#,
        SRC = HEIGHT_ONLY_SRC
    );
    let (_, y, w, h) = render_green(HEIGHT_ONLY_SRC, body, |p, key| {
        let mut sizes = HashMap::new();
        sizes.insert(key, (100.0f32, 60.0f32)); // bogus usvg pixmap（高 60 巧同 attr）
        p.set_image_sizes(sizes);
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = HashMap::new();
        no_ratio.insert(key, (None, Some(60.0)));
        p.set_image_no_ratio(no_ratio);
    });
    assert!(
        (w - 300.0).abs() < 0.5,
        "no-ratio 缺轴落 default 宽 300（bogus 比 100/60 旧径 100），实测 {w}"
    );
    assert!((h - 60.0).abs() < 0.5, "固有高 60，实测 {h}");
    assert!((y - 8.0).abs() < 0.5, "body 缘 8，实测 {y}");
}

/// 守卫（ComputedIntrinsic 不受 no-ratio 守卫波及）：width=60 + viewBox SVG 走
/// (Some(60),None) attr 臂但**无** no_ratio 键——R2172 比注入保留（60/20=3），
/// 渲染 60×20（probe s1 形，s291 前既绿）。
#[test]
fn r5421_computed_intrinsic_ratio_injection_kept() {
    let body = format!(
        r#"<body><img src='{SRC}' style="margin:0;display:block;background:#008000"></body>"#,
        SRC = COMPUTED_SRC
    );
    let (x, y, w, h) = render_green(COMPUTED_SRC, body, |p, key| {
        let mut sizes = HashMap::new();
        sizes.insert(key, (60.0f32, 20.0f32)); // ComputedIntrinsic 真实固有（usvg 比补轴）
        p.set_image_sizes(sizes);
    });
    assert!(
        (x - 8.0).abs() < 0.5 && (y - 8.0).abs() < 0.5,
        "body 缘 8，实测 ({x},{y})"
    );
    assert!((w - 60.0).abs() < 0.5, "固有宽 60，实测 {w}");
    assert!(
        (h - 20.0).abs() < 0.5,
        "R2172 注入比 3 → 高 20（ComputedIntrinsic 不入守卫），实测 {h}"
    );
}

/// ★load-bearing（s292 L1：naturalWidth 语料 vb0w 面）：零维 viewBox + SVG
/// width=60——sizes 表 bogus 1×1 与 no_ratio 真维 (Some(60),None) 并存，s283
/// auto 态固有宽注入旧读 sizes → 内容宽 1×150（bb 11×160）；修复后 noratio img
/// 注入先取 no_ratio 真维 60 → 60×150 + border 5×2 → bb 70×160。
#[test]
fn r5421_noratio_l1_no_ratio_dims_win_over_bogus_pixmap() {
    let body = format!(
        r#"<body><img src='{SRC}' style="border:5px solid black;margin:0;display:block;background:#008000"></body>"#,
        SRC = ZERO_VB_SRC
    );
    let (x, y, w, h) = render_green(ZERO_VB_SRC, body, |p, key| {
        let mut sizes = HashMap::new();
        sizes.insert(key, (1.0f32, 1.0f32)); // R3760 零维 viewBox 早退 pixmap
        p.set_image_sizes(sizes);
        let mut no_ratio: HashMap<u64, (Option<f32>, Option<f32>)> = HashMap::new();
        no_ratio.insert(key, (Some(60.0), None));
        p.set_image_no_ratio(no_ratio);
    });
    assert!(
        (x - 8.0).abs() < 0.5 && (y - 8.0).abs() < 0.5,
        "body 缘 8（margin 0），实测 ({x},{y})"
    );
    assert!(
        (w - 70.0).abs() < 0.5,
        "L1：固有宽取 no_ratio 真维 60 + border 10 → bb 70（旧径 bogus pixmap → 11），实测 {w}"
    );
    assert!(
        (h - 160.0).abs() < 0.5,
        "no-ratio 缺轴落 default 高 150 + border 10 → bb 160，实测 {h}"
    );
}
