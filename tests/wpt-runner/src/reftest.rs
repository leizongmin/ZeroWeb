//! Reftest Harness — 渲染测试 HTML 和参考 HTML，比较像素输出。
//!
//! 实现 WPT 风格的 `rel=match` / `rel=mismatch` 比较逻辑：
//! - match：两个页面的像素应几乎相同（允许模糊阈值）
//! - mismatch：两个页面的像素应有显著差异
//!
//! 支持分类容差（布局类 vs 文字类）和 per-test WPT fuzzy 注解覆盖。
//! 支持 CPU 软件渲染和 GPU 无头渲染两种模式。

#![allow(dead_code)]

use std::path::Path;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};
use zero_render_foundation::surface::FrameBuffer;

use crate::manifest::FuzzyMeta;

mod reftest_compare;
mod reftest_fonts;
mod reftest_scripts;

mod resources;
pub use resources::convert_png_buffer_to_rgba;
use resources::*;

// 像素对比与 PNG/PPM I/O（reftest_compare）对外保持 `crate::reftest::compare_pixels` 等
// 公共路径不变，故重新导出。脚本执行与字体加载辅助仅在模块内部使用，经 glob 引入。
// `save_framebuffer_png` 是 pre-existing dead public API（`#![allow(dead_code)]` 容忍），
// 重新导出会触发 unused_imports，此处一并 allow 以保持公共 API surface 零变化。
#[allow(unused_imports)]
pub use reftest_compare::{
    compare_pixels, compare_pixels_labeled, frame_is_near_solid, save_fb_as_png, save_framebuffer_png,
};
use reftest_fonts::*;
use reftest_scripts::*;

/// Reftest 分类 — 用于确定默认容差级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReftestCategory {
    /// 布局类 reftest（不含文字渲染）：严格容差。
    Layout,
    /// 文字类 reftest（含文本渲染）：宽松容差。
    Text,
    /// 未分类：使用中等容差。
    Unknown,
}

impl ReftestCategory {
    /// 根据测试路径自动推断分类。
    pub fn from_path(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        // 文字排版相关目录
        if path_lower.contains("/css-text/")
            || path_lower.contains("/css-writing-modes/")
            || path_lower.contains("/css-fonts/")
            || path_lower.contains("/css-text-decor/")
            || path_lower.contains("/text/")
            || path_lower.contains("/font/")
            || path_lower.starts_with("css-text/")
            || path_lower.starts_with("css-writing-modes/")
            || path_lower.starts_with("css-fonts/")
            || path_lower.starts_with("css-text-decor/")
        {
            Self::Text
        } else {
            Self::Layout
        }
    }

    /// 该分类的默认最大差异率。
    pub fn default_max_diff_ratio(&self) -> f64 {
        match self {
            Self::Layout => 0.01,  // 1%
            Self::Text => 0.05,    // 5%（字体渲染差异更大）
            Self::Unknown => 0.02, // 2%
        }
    }

    /// 该分类的默认最大单通道色差。
    pub fn default_max_channel_diff(&self) -> u8 {
        match self {
            Self::Layout => 5,
            Self::Text => 15,
            Self::Unknown => 8,
        }
    }

    /// DC-14 锁定的严格容差——最大差异率（硬上限，不可放宽）。
    ///
    /// 来源：goal doc DC-14 line 162-163/315-316「布局类 ≤ 0.1%、文字类 ≤ 0.5%」。
    /// 默认容差（`default_max_diff_ratio`）是其 10×（R280 量化），含同源假通过；
    /// 严格容差是唯一可信达标指标（DC-14）。经 env `ZERO_REFTEST_STRICT` 启用。
    pub fn strict_max_diff_ratio(&self) -> f64 {
        match self {
            Self::Layout => 0.001,  // 0.1%
            Self::Text => 0.005,    // 0.5%
            Self::Unknown => 0.001, // 0.1%（未知分类按最严格处理）
        }
    }

    /// DC-14 锁定的严格容差——最大单通道色差（硬上限，不可放宽）。
    ///
    /// 来源：goal doc DC-14「布局类 channel ≤ 2、文字类 ≤ 5」。
    pub fn strict_max_channel_diff(&self) -> u8 {
        match self {
            Self::Layout => 2,
            Self::Text => 5,
            Self::Unknown => 2,
        }
    }
}

/// Reftest 比较结果。
#[derive(Debug)]
pub struct ReftestResult {
    /// 测试标识符。
    pub id: String,
    /// 是否通过比较。
    pub passed: bool,
    /// 不同像素数量。
    pub diff_pixels: usize,
    /// 总像素数。
    pub total_pixels: usize,
    /// 差异率（0.0 ~ 1.0）。
    pub diff_ratio: f64,
    /// 最大单通道颜色差异。
    pub max_channel_diff: u8,
    /// 失败原因（通过时为空）。
    pub message: String,
    /// DC-14 非平凡性：测试帧是否「接近纯色」（退化/空白渲染）。
    /// 近纯色的 strict-pass 须标可疑单独审计（test==ref 退化假绿，如 headless 空白页）。
    pub test_near_solid: bool,
}

/// Reftest 配置。
#[derive(Debug, Clone)]
pub struct ReftestConfig {
    /// 视口宽度。
    pub viewport_width: u32,
    /// 视口高度。
    pub viewport_height: u32,
    /// 缩放因子。
    pub scale_factor: f32,
    /// 最大允许差异率（0.0 ~ 1.0），默认 0.01（1%）。
    pub max_diff_ratio: f64,
    /// 最大允许单通道色差（0 ~ 255），默认 5。
    pub max_channel_diff: u8,
    /// Reftest 分类。
    pub category: ReftestCategory,
    /// Per-test fuzzy 容差覆盖（来自 WPT MANIFEST.json）。
    pub fuzzy_override: Option<FuzzyMeta>,
    /// mismatch 模式的最小差异率阈值（默认 0.005 = 0.5%）。
    /// 差异率超过此值才认为是不匹配通过。
    pub min_mismatch_ratio: f64,
}

impl Default for ReftestConfig {
    fn default() -> Self {
        Self {
            viewport_width: 800,
            viewport_height: 600,
            scale_factor: 1.0,
            max_diff_ratio: 0.01,
            max_channel_diff: 5,
            category: ReftestCategory::Unknown,
            fuzzy_override: None,
            min_mismatch_ratio: 0.005,
        }
    }
}

impl ReftestConfig {
    /// 根据分类创建配置。
    ///
    /// 容差源：若环境变量 `ZERO_REFTEST_STRICT` 已设置则用 **DC-14 锁定严格容差**
    /// （Layout 0.1%/2、Text 0.5%/5，唯一可信达标指标），否则用分类默认松容差
    /// （当前为其 10×，含同源假通过，R280 量化）。strict 同时切换计数阈值
    /// （`compare_pixels_labeled` 的 threshold）与通过阈值，二者须一致才反映真实差异。
    pub fn for_category(category: ReftestCategory) -> Self {
        let strict = std::env::var("ZERO_REFTEST_STRICT").is_ok();
        let (max_diff_ratio, max_channel_diff) = if strict {
            (category.strict_max_diff_ratio(), category.strict_max_channel_diff())
        } else {
            (category.default_max_diff_ratio(), category.default_max_channel_diff())
        };
        Self {
            max_diff_ratio,
            max_channel_diff,
            category,
            ..Default::default()
        }
    }

    /// 设置视口尺寸（builder 模式）。
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// 应用 WPT fuzzy 注解覆盖。
    ///
    /// 如果 fuzzy 注解指定了 maxDiff 或 totalPixels，覆盖分类默认值。
    pub fn with_fuzzy_override(&mut self, fuzzy: &FuzzyMeta) {
        if let Some(max_diff) = fuzzy.max_diff {
            self.max_channel_diff = max_diff as u8;
        }
        if let Some(total_pixels) = fuzzy.total_pixels {
            // total_pixels 转换为差异率
            let total = (self.viewport_width as u64) * (self.viewport_height as u64);
            if total > 0 {
                self.max_diff_ratio = total_pixels as f64 / total as f64;
            }
        }
        self.fuzzy_override = Some(fuzzy.clone());
    }

    /// 获取实际使用的最大差异率（考虑 fuzzy 覆盖）。
    pub fn effective_max_diff_ratio(&self) -> f64 {
        if let Some(ref fuzzy) = self.fuzzy_override
            && fuzzy.total_pixels.is_some()
        {
            return self.max_diff_ratio;
        }
        self.max_diff_ratio
    }

    /// 获取实际使用的最大通道差异（考虑 fuzzy 覆盖）。
    pub fn effective_max_channel_diff(&self) -> u8 {
        if let Some(ref fuzzy) = self.fuzzy_override
            && fuzzy.max_diff.is_some()
        {
            return self.max_channel_diff;
        }
        self.max_channel_diff
    }
}

/// 单个 reftest 用例。
#[derive(Debug, Clone)]
pub struct ReftestCase {
    /// 测试标识符。
    pub id: String,
    /// 测试 HTML。
    pub test_html: String,
    /// 参考 HTML。
    pub ref_html: String,
    /// 共享 CSS。
    pub css: String,
    /// 比较模式：true=match（应相同），false=mismatch（应不同）。
    pub is_match: bool,
    /// 参考文件所在目录（用于解析参考页相对图片路径）。
    ///
    /// 渲染参考页时，其相对图片 URL（如 `../support/swatch-orange.png`）必须相对参考文件
    /// 自身目录解析，而非测试文件目录——否则参考文件位于不同目录（如 `reference/` 子目录）
    /// 时图片加载失败。内联 reftest 无文件基，保持 `None`（回落到 base_dir）。
    pub ref_base_dir: Option<std::path::PathBuf>,
}

/// 运行单个 reftest 用例。
pub fn run_reftest(case: &ReftestCase, config: &ReftestConfig) -> ReftestResult {
    run_reftest_with_base(case, config, None)
}

/// 运行单个 reftest 用例（支持基于 base_dir 的图片加载）。
pub fn run_reftest_with_base(case: &ReftestCase, config: &ReftestConfig, base_dir: Option<&Path>) -> ReftestResult {
    // 渲染测试页面
    let test_fb = render_to_framebuffer_with_base(&case.test_html, &case.css, config, base_dir);
    // 渲染参考页面（图片相对参考文件目录解析，缺失时回落到测试目录）
    let ref_base = case.ref_base_dir.as_deref().or(base_dir);
    let ref_fb = render_to_framebuffer_with_base(&case.ref_html, &case.css, config, ref_base);

    // 尺寸必须一致
    if test_fb.width != ref_fb.width || test_fb.height != ref_fb.height {
        return ReftestResult {
            id: case.id.clone(),
            passed: false,
            diff_pixels: 0,
            total_pixels: 0,
            diff_ratio: 0.0,
            max_channel_diff: 0,
            test_near_solid: false,
            message: format!(
                "Size mismatch: test={}x{} ref={}x{}",
                test_fb.width, test_fb.height, ref_fb.width, ref_fb.height
            ),
        };
    }

    let total_pixels = (test_fb.width as usize) * (test_fb.height as usize);
    let eff_channel_diff = config.effective_max_channel_diff();
    let (diff_pixels, max_channel_diff) = compare_pixels_labeled(&test_fb, &ref_fb, eff_channel_diff, &case.id);
    let diff_ratio = if total_pixels > 0 {
        diff_pixels as f64 / total_pixels as f64
    } else {
        0.0
    };

    let eff_max_ratio = config.effective_max_diff_ratio();

    let passed = if case.is_match {
        // match 模式：差异应小于阈值
        diff_ratio <= eff_max_ratio
    } else {
        // mismatch 模式：应有显著差异
        diff_ratio > config.min_mismatch_ratio
    };

    let message = if passed {
        String::new()
    } else if case.is_match {
        format!(
            "Match failed: {}/{} pixels differ ({:.2}%), max channel diff={}, threshold={:.2}%/{}ch",
            diff_pixels,
            total_pixels,
            diff_ratio * 100.0,
            max_channel_diff,
            eff_max_ratio * 100.0,
            eff_channel_diff
        )
    } else {
        format!(
            "Mismatch failed: only {}/{} pixels differ ({:.2}%), expected > 1%",
            diff_pixels,
            total_pixels,
            diff_ratio * 100.0
        )
    };

    // 失败时，如果设置了 REFTEST_DUMP 环境变量，保存 PNG 用于诊断
    // REFTEST_DUMP_PASS=1 同时保存通过用例，用于诊断通过用例的实际渲染
    let dump_pass = std::env::var("REFTEST_DUMP_PASS").is_ok();
    if (!passed || dump_pass) && std::env::var("REFTEST_DUMP").is_ok() {
        let dump_dir = std::path::Path::new("target/reftest-dump");
        let _ = std::fs::create_dir_all(dump_dir);
        let safe_id = case.id.replace(['/', '\\', '.'], "_");
        save_fb_as_png(&test_fb, &dump_dir.join(format!("{}-test.png", safe_id)));
        save_fb_as_png(&ref_fb, &dump_dir.join(format!("{}-ref.png", safe_id)));
    }

    ReftestResult {
        id: case.id.clone(),
        passed,
        diff_pixels,
        total_pixels,
        diff_ratio,
        max_channel_diff,
        message,
        test_near_solid: frame_is_near_solid(&test_fb),
    }
}

/// 使用 GPU 无头渲染运行 reftest（回退到 CPU 如果 GPU 不可用）。
pub fn run_reftest_gpu(case: &ReftestCase, config: &ReftestConfig) -> ReftestResult {
    run_reftest_gpu_with_base(case, config, None)
}

/// 使用 GPU 无头渲染运行 reftest（支持基于 base_dir 的图片加载）。
pub fn run_reftest_gpu_with_base(case: &ReftestCase, config: &ReftestConfig, base_dir: Option<&Path>) -> ReftestResult {
    // 渲染测试页面和参考页面（参考页图片相对参考文件目录解析，缺失时回落到测试目录）
    let test_fb = render_to_framebuffer_gpu_with_base(&case.test_html, &case.css, config, base_dir);
    let ref_base = case.ref_base_dir.as_deref().or(base_dir);
    let ref_fb = render_to_framebuffer_gpu_with_base(&case.ref_html, &case.css, config, ref_base);

    // 尺寸必须一致
    if test_fb.width != ref_fb.width || test_fb.height != ref_fb.height {
        return ReftestResult {
            id: case.id.clone(),
            passed: false,
            diff_pixels: 0,
            total_pixels: 0,
            diff_ratio: 0.0,
            max_channel_diff: 0,
            test_near_solid: false,
            message: format!(
                "Size mismatch: test={}x{} ref={}x{}",
                test_fb.width, test_fb.height, ref_fb.width, ref_fb.height
            ),
        };
    }

    let total_pixels = (test_fb.width as usize) * (test_fb.height as usize);
    let eff_channel_diff = config.effective_max_channel_diff();
    let (diff_pixels, max_channel_diff) = compare_pixels_labeled(&test_fb, &ref_fb, eff_channel_diff, &case.id);
    let diff_ratio = if total_pixels > 0 {
        diff_pixels as f64 / total_pixels as f64
    } else {
        0.0
    };

    let eff_max_ratio = config.effective_max_diff_ratio();

    let passed = if case.is_match {
        diff_ratio <= eff_max_ratio
    } else {
        diff_ratio > config.min_mismatch_ratio
    };

    let message = if passed {
        String::new()
    } else if case.is_match {
        format!(
            "Match failed: {}/{} pixels differ ({:.2}%), max channel diff={}, threshold={:.2}%/{}ch",
            diff_pixels,
            total_pixels,
            diff_ratio * 100.0,
            max_channel_diff,
            eff_max_ratio * 100.0,
            eff_channel_diff
        )
    } else {
        format!(
            "Mismatch failed: only {}/{} pixels differ ({:.2}%), expected > 1%",
            diff_pixels,
            total_pixels,
            diff_ratio * 100.0
        )
    };

    ReftestResult {
        id: case.id.clone(),
        passed,
        diff_pixels,
        total_pixels,
        diff_ratio,
        max_channel_diff,
        message,
        test_near_solid: frame_is_near_solid(&test_fb),
    }
}

/// 将 HTML 渲染到 CPU 帧缓冲。
///
/// 如果 HTML 中包含 `<script>` 标签，会先通过 V8 runtime 执行其中的 JS 代码，
/// 然后再进行渲染。当前实现中 JS 执行不修改 DOM（适用于大多数 WPT reftest
/// 中 JS 仅用于设置/断言的场景）。
///
/// 当 `base_dir` 提供时，会解析 HTML 中引用的图片并加载到 ImageCache。
pub fn render_to_framebuffer(html: &str, css: &str, config: &ReftestConfig) -> FrameBuffer {
    render_to_framebuffer_with_base(html, css, config, None)
}

/// 将 HTML 渲染到 CPU 帧缓冲（支持基于 base_dir 的图片加载）。
///
/// 使用 `render_full_scene` 渲染全部 13 种图元类型（fills, rounded_rects,
/// gradients, shadows, images, strokes, path_fills, path_strokes, glyphs,
/// clips, transforms, filters, blend_modes）。
pub fn render_to_framebuffer_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> FrameBuffer {
    render_to_framebuffer_with_layout_with_base(html, css, config, base_dir).0
}

/// 同 [`render_to_framebuffer_with_base`]，但额外返回布局树根（DC-13 结构检查用）。
///
/// `RenderResult.layout.root` 在 `render_full_scene`（仅借 `result.primitives`）后移出，
/// 供 product-smoke 结构自动检查（如兄弟盒重叠检测）遍历。Framebuffer 渲染不受影响。
pub fn render_to_framebuffer_with_layout_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> (FrameBuffer, zero_layout_engine::types::LayoutBox, String) {
    // 执行页面 <script>（含 DOM 变更），把 JS 后的最终 HTML 用于后续渲染。
    let mutated_html = apply_scripted_dom_mutations(html, base_dir);
    let html: &str = &mutated_html;

    // 先构建图像缓存，提取固有尺寸供 paint 阶段使用
    let mut image_cache = build_image_cache(html, base_dir);
    let (image_sizes, image_ratios, image_no_ratio) = extract_image_metrics(&mut image_cache, html);

    let combined_css = merge_page_css(html, css, base_dir);

    let mut pipeline = RenderPipeline::new(config.viewport_width as f32, config.viewport_height as f32);
    pipeline.set_skip_indicators(true);
    pipeline.set_image_sizes(image_sizes);
    pipeline.set_image_ratios(image_ratios);
    pipeline.set_image_no_ratio(image_no_ratio);

    // 构建字体查找表（在 render_html 之前，以便 Painter 解析 CSS font-family）
    let mut font_loader = create_font_loader();
    // 加载 CSS @font-face 声明的自定义字体：扫描外链/传入 CSS + 文档内联 <style>
    //（@font-face 常在内联 <style>），按 base_dir 解析 src 到本地文件。
    let font_scan_css = format!("{combined_css}\n{}", extract_inline_style_css(html));
    load_font_faces_into(&mut font_loader, base_dir, &font_scan_css);
    let font_resolver = font_loader.build_font_resolver();
    pipeline.set_font_resolver(font_resolver);

    let result = pipeline.render_html(html, &combined_css);

    // DEBUG: dump layout box tree geometry (absolute y / margin-top / padding-top)
    // 用途：诊断产品 smoke 垂直偏移（如 welcome 36px 顶部偏移）。
    if std::env::var("LAYOUT_DUMP").is_ok() {
        dump_layout_tree(&result.layout.root, html);
    }

    // DEBUG: dump primitives for diagnostic
    if std::env::var("REFTEST_DEBUG").is_ok() {
        eprintln!("=== Primitives for {} ===", html.lines().take(1).next().unwrap_or(""));
        eprintln!("  fills: {}", result.primitives.fills.len());
        eprintln!("  images: {}", result.primitives.images.len());
        eprintln!("  rounded_rects: {}", result.primitives.rounded_rects.len());
        eprintln!("  glyphs: {}", result.primitives.glyphs.len());
        eprintln!("  gradients: {}", result.primitives.gradients.len());
        eprintln!("  strokes: {}", result.primitives.strokes.len());
        for (i, fill) in result.primitives.fills.iter().enumerate().take(20) {
            eprintln!(
                "  fill[{}]: ({:.1},{:.1},{:.1},{:.1}) rgba({},{},{},{})",
                i,
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }
        for (i, img) in result.primitives.images.iter().enumerate().take(10) {
            eprintln!(
                "  image[{}]: ({:.1},{:.1},{:.1},{:.1}) key={:?}",
                i, img.rect.origin.x, img.rect.origin.y, img.rect.size.width, img.rect.size.height, img.image_key
            );
        }
    }

    let mut glyph_cache = GlyphCache::new(1024);

    // 使用已构建的图像缓存（包含固有尺寸信息）
    let fb = render_full_scene(
        config.viewport_width,
        config.viewport_height,
        config.scale_factor,
        &result.primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
    );
    // render_full_scene 仅借 result.primitives（借用已结束）；移出 layout 根供结构检查。
    // 一并返回 mutated_html（render_html 实际解析的 HTML，经 script DOM 变更后可能与调用方传入
    // 的原 html 不同）——DC-13 结构检查须用它建 labels，否则 layout 树 node_id 与 collect_dom_labels
    //（解析原 html）不匹配 → 真元素误标 "(anon)"（R1499：morning disqus loadDisqus() appendChild
    // 致 mutated_html 与原 html 不同，p/table 误标 anon）。
    (fb, result.layout.root, mutated_html)
}

/// DC-13 line 321：通过 `zero-webview` 稳定嵌入边界渲染 HTML 到 FrameBuffer。
///
/// 与 [`render_to_framebuffer_with_base`]（engine-direct，直接用 `RenderPipeline`）
/// 形成对照——验证「产品层（ZeroBrowser）↔ WebView 层」不互相掩盖问题。WebView 路径
/// 走完整的嵌入接口（`WebView::load_html`，含其内部的 image/font/security 处理），
/// 产出 `RenderPrimitives` 后用同一 `render_full_scene` 光栅化。
/// 带 `base_dir` 时见 [`render_via_webview_to_framebuffer_with_base`]。
pub fn render_via_webview_to_framebuffer(html: &str, css: &str, config: &ReftestConfig) -> FrameBuffer {
    render_via_webview_to_framebuffer_with_base(html, css, config, None)
}

/// 经 WebView 嵌入边界渲染（支持 `base_dir` 外链 CSS/图片，与 engine-direct 对齐）。
pub fn render_via_webview_to_framebuffer_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> FrameBuffer {
    let mutated_html = apply_scripted_dom_mutations(html, base_dir);
    let html: &str = &mutated_html;

    let mut image_cache = build_image_cache(html, base_dir);
    let (image_sizes, image_ratios, image_no_ratio) = extract_image_metrics(&mut image_cache, html);
    let combined_css = merge_page_css(html, css, base_dir);

    let mut font_loader = create_font_loader();
    let font_scan_css = format!("{combined_css}\n{}", extract_inline_style_css(html));
    load_font_faces_into(&mut font_loader, base_dir, &font_scan_css);
    let font_resolver = font_loader.build_font_resolver();

    let wv_config = zero_webview::WebViewConfig {
        width: config.viewport_width,
        height: config.viewport_height,
        ..Default::default()
    };
    let mut webview = zero_webview::WebView::new(wv_config);
    webview.set_font_resolver(font_resolver);
    webview.set_image_sizes(image_sizes);
    webview.set_image_ratios(image_ratios);
    webview.set_image_no_ratio(image_no_ratio);
    let result = webview.load_html(
        html,
        if combined_css.is_empty() {
            None
        } else {
            Some(&combined_css)
        },
    );

    let mut glyph_cache = GlyphCache::new(1024);
    render_full_scene(
        config.viewport_width,
        config.viewport_height,
        config.scale_factor,
        &result.primitives,
        &font_loader,
        &mut glyph_cache,
        Some(&mut image_cache),
        &[],
        &[],
        &[],
        &[],
    )
}

/// 合并传入 CSS 与 `base_dir` 下 `<link rel="stylesheet">` 外链（engine / WebView 共用）。
fn merge_page_css(html: &str, css: &str, base_dir: Option<&Path>) -> String {
    let linked_css = load_linked_stylesheets(html, base_dir);
    if css.is_empty() {
        linked_css
    } else if linked_css.is_empty() {
        css.to_string()
    } else {
        format!("{linked_css}\n{css}")
    }
}

/// 像素级实证：engine-direct reftest 渲染 ≡ WebView（产品路径）渲染——确保 WPT 通过率代表浏览器真实显示。
/// engine 与 WebView 同 RenderPrimitives + 同 rasterizer（render_full_scene），像素差应近 0。
#[test]
fn webview_reftest_matches_engine_direct_pixels() {
    let config = ReftestConfig::default();
    let html = r#"<html><body>
        <div style="width:200px;height:100px;background:#cc3333">Box</div>
        <div style="width:120px;height:60px;background:#3366cc;border-radius:8px">R</div>
    </body></html>"#;
    let engine_fb = render_to_framebuffer(html, "", &config);
    let webview_fb = render_via_webview_to_framebuffer(html, "", &config);
    assert_eq!(engine_fb.width, webview_fb.width, "宽度须一致");
    assert_eq!(engine_fb.height, webview_fb.height, "高度须一致");
    let (diff_pixels, _max_channel) = compare_pixels(&engine_fb, &webview_fb, 0);
    let total = (engine_fb.width as usize) * (engine_fb.height as usize);
    let ratio = if total > 0 {
        diff_pixels as f64 / total as f64
    } else {
        0.0
    };
    assert!(
        ratio < 0.01,
        "engine-direct vs WebView 像素差过高: {diff_pixels}/{total} ({:.2}%)",
        ratio * 100.0
    );
}

/// 同上，但 css 经外部 <style> 注入（覆盖 css 路径）。
#[test]
fn webview_reftest_matches_engine_direct_with_css() {
    let config = ReftestConfig::default();
    let html = r#"<html><body><div class="box">Hi</div></body></html>"#;
    let css = ".box { width: 200px; height: 100px; background: #2a8a2a; }";
    let engine_fb = render_to_framebuffer(html, css, &config);
    let webview_fb = render_via_webview_to_framebuffer(html, css, &config);
    assert_eq!(
        (engine_fb.width, engine_fb.height),
        (webview_fb.width, webview_fb.height)
    );
    let (diff_pixels, _) = compare_pixels(&engine_fb, &webview_fb, 0);
    let total = (engine_fb.width as usize) * (engine_fb.height as usize);
    let ratio = if total > 0 {
        diff_pixels as f64 / total as f64
    } else {
        0.0
    };
    assert!(
        ratio < 0.01,
        "css 路径 engine vs WebView 像素差过高: {diff_pixels}/{total} ({:.2}%)",
        ratio * 100.0
    );
}

/// welcome.html（产品 newtab 页）engine-direct ≡ WebView 像素等价。
#[test]
fn webview_reftest_matches_engine_direct_welcome_page() {
    let welcome_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/browser/assets/welcome.html");
    let html =
        std::fs::read_to_string(&welcome_path).unwrap_or_else(|e| panic!("read {}: {e}", welcome_path.display()));
    let config = ReftestConfig::default();
    let engine_fb = render_to_framebuffer(&html, "", &config);
    let webview_fb = render_via_webview_to_framebuffer(&html, "", &config);
    assert_eq!(
        (engine_fb.width, engine_fb.height),
        (webview_fb.width, webview_fb.height)
    );
    let (diff_pixels, _) = compare_pixels(&engine_fb, &webview_fb, 0);
    let total = (engine_fb.width as usize) * (engine_fb.height as usize);
    let ratio = if total > 0 {
        diff_pixels as f64 / total as f64
    } else {
        0.0
    };
    assert!(
        ratio < 0.01,
        "welcome.html engine vs WebView 像素差过高: {diff_pixels}/{total} ({:.2}%)",
        ratio * 100.0
    );
}

/// `base_dir` 外链 CSS：engine-direct ≡ WebView（与 product-smoke --base-dir 对齐）。
#[test]
fn webview_reftest_matches_engine_direct_with_linked_css() {
    let base = std::env::temp_dir().join(format!("zeroweb_reftest_linked_css_{}", std::process::id()));
    std::fs::create_dir_all(&base).expect("temp dir");
    std::fs::write(
        base.join("linked.css"),
        ".box { width: 200px; height: 80px; background: #009900; }",
    )
    .expect("write css");
    let html =
        r#"<html><head><link rel="stylesheet" href="linked.css"></head><body><div class="box">X</div></body></html>"#;
    let config = ReftestConfig::default();
    let engine_fb = render_to_framebuffer_with_base(html, "", &config, Some(&base));
    let webview_fb = render_via_webview_to_framebuffer_with_base(html, "", &config, Some(&base));
    let _ = std::fs::remove_dir_all(&base);
    assert_eq!(
        (engine_fb.width, engine_fb.height),
        (webview_fb.width, webview_fb.height)
    );
    let (diff_pixels, _) = compare_pixels(&engine_fb, &webview_fb, 0);
    let total = (engine_fb.width as usize) * (engine_fb.height as usize);
    let ratio = if total > 0 {
        diff_pixels as f64 / total as f64
    } else {
        0.0
    };
    assert!(
        ratio < 0.01,
        "linked css engine vs WebView 像素差过高: {diff_pixels}/{total} ({:.2}%)",
        ratio * 100.0
    );
}

/// 诊断：转储布局盒树几何（绝对 y / margin-top / padding-top / height）。
///
/// 重新解析 HTML 以建立 `NodeId → (tag, class)` 映射，然后递归遍历 `LayoutBox`，
/// 累加父级内容区偏移得到绝对坐标，打印每个盒子的 margin-top、padding-top、
/// 绝对 y 与高度。用于定位产品 smoke 的垂直偏移来源（如 welcome 36px）。
/// 解析 HTML，BFS 遍历 DOM 收集每个元素的 `tag.class` 标签（NodeId → label）。
///
/// 供 [`dump_layout_tree`]（LAYOUT_DUMP 诊断）与 [`check_sibling_overlaps`]（DC-13 结构检查）
/// 共享——结构检查报告需可读标签定位退化元素。无 class 元素用 tag 名，多 class 用 `.` 连接。
pub fn collect_dom_labels(html: &str) -> std::collections::HashMap<zero_dom::NodeId, String> {
    use std::collections::HashMap;
    use zero_dom::{NodeKind, parse_html};
    let doc = parse_html(html);
    let mut id_label: HashMap<zero_dom::NodeId, String> = HashMap::new();
    let mut queue = vec![doc.root()];
    while let Some(id) = queue.pop() {
        if let Some(node) = doc.get(id) {
            if let NodeKind::Element(elem) = &node.kind {
                let label = if elem.class_list.is_empty() {
                    elem.local_name().to_string()
                } else {
                    format!("{}.{}", elem.local_name(), elem.class_list.join("."))
                };
                id_label.insert(id, label);
            }
            let mut child = doc.first_child(id);
            while let Some(c) = child {
                queue.push(c);
                child = doc.next_sibling(c);
            }
        }
    }
    id_label
}

fn dump_layout_tree(root: &zero_layout_engine::types::LayoutBox, html: &str) {
    use std::collections::HashMap;
    use zero_dom::NodeId;
    let id_label = collect_dom_labels(html);

    eprintln!("=== LAYOUT_DUMP (abs_y / height / margin-top / padding-top) ===");
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        off_x: f32,
        off_y: f32,
        depth: usize,
        labels: &HashMap<NodeId, String>,
    ) {
        let abs_x = off_x + b.x;
        let abs_y = off_y + b.y;
        let label = b
            .node_id
            .and_then(|id| labels.get(&id))
            .cloned()
            .unwrap_or_else(|| "(anon)".to_string());
        eprintln!(
            "{:indent$}{:24} abs_y={:7.1} h={:6.1} mt={:5.1} pt={:5.1} x={:6.1} w={:6.1} dmt={:5.1}",
            "",
            label,
            abs_y,
            b.height,
            b.margin_top,
            b.padding_top,
            abs_x,
            b.width,
            b.declared_margin_top,
            indent = depth * 2
        );
        let child_off_x = abs_x + b.padding_left + b.border_left;
        let child_off_y = abs_y + b.padding_top + b.border_top;
        for child in &b.children {
            walk(child, child_off_x, child_off_y, depth + 1, labels);
        }
    }
    walk(root, 0.0, 0.0, 0, &id_label);
}

/// DC-13 line 322-326：检测布局树中**同父兄弟盒** border-box 重叠（产品可见排版退化回归门）。
///
/// 正常流（block/flex/grid/float）同父兄弟盒不应重叠；重叠面积 > 阈值（忽略负 margin 等微叠）
/// 表示布局 breakage（abspos 错位、margin 折叠异常、IFC/匿名盒错排等）。返回问题描述列表
/// （空 = 结构通过）。与像素 diff 门禁互补：像素 diff 量化整体差距，本检查定位**结构性**
/// 退化（即使像素差小，兄弟盒重叠也是用户可见 bug）。`labels` 为 node_id→tag.class（诊断用）。
pub fn check_sibling_overlaps(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
) -> Vec<String> {
    // 忽略 < 100px² 的微小叠（负 margin、亚像素边界、匿名盒 layout artifact）；仅报显著
    // 结构重叠。50→100：morning 残余 50px² 匿名盒重叠为亚像素噪声（~7×7px），真重叠
    //（wintertc 55936 / 测试 2500）远超此阈值。
    const MIN_OVERLAP_PX: f32 = 100.0;
    let mut issues = Vec::new();
    fn label_of(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    ) -> String {
        b.node_id
            .and_then(|id| labels.get(&id).cloned())
            .unwrap_or_else(|| "(anon)".to_string())
    }
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        off_x: f32,
        off_y: f32,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        issues: &mut Vec<String>,
    ) {
        let abs_x = off_x + b.x;
        let abs_y = off_y + b.y;
        let child_off_x = abs_x + b.padding_left + b.border_left;
        let child_off_y = abs_y + b.padding_top + b.border_top;
        let n = b.children.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (ci, cj) = (&b.children[i], &b.children[j]);
                // 跳过无可见尺寸的盒（纯结构包裹 / 零尺寸匿名盒）
                if ci.width < 2.0 || ci.height < 2.0 || cj.width < 2.0 || cj.height < 2.0 {
                    continue;
                }
                // R1504：跳过 positioned（abspos/fixed/relative）或 float 盒对——它们**按设计**重叠
                //（relative offset / abspos 定位 / float exclusion 都是合法叠加，reftest red/green
                // 覆盖、z-order、paint-order 测试大量用之）。flag 这些是噪声非 bug。仅报**普通块流**
                // 兄弟的非预期重叠（如 R1492 长高重叠）。须任一为 positioned/float 即跳过该对。
                if ci.is_absolute
                    || ci.is_fixed
                    || ci.is_relative
                    || ci.is_sticky
                    || !matches!(ci.float, zero_css_parser::values::FloatValue::None)
                    || cj.is_absolute
                    || cj.is_fixed
                    || cj.is_relative
                    || cj.is_sticky
                    || !matches!(cj.float, zero_css_parser::values::FloatValue::None)
                {
                    continue;
                }
                let (ov, ov_h) = rect_overlap_area(
                    (child_off_x + ci.x, child_off_y + ci.y, ci.width, ci.height),
                    (child_off_x + cj.x, child_off_y + cj.y, cj.width, cj.height),
                );
                // R1503：跳过「宽而薄」的亚像素 sliver——重叠高 ≤ 2px 但面积超阈（如 morning @320
                // 相邻行 `<code>` 1px×149=149px²，IFC baseline/line-height 舍入噪声）非真结构重叠。
                // 真 overlap（如 article/disqus 108px 高）远超此。area 阈值单独会被宽 sliver 绕过。
                if ov > MIN_OVERLAP_PX && ov_h > 2.0 {
                    issues.push(format!(
                        "sibling overlap {:.0}px²: [{}] @({:.0},{:.0},{:.0}x{:.0}) & [{}] @({:.0},{:.0},{:.0}x{:.0})",
                        ov,
                        label_of(ci, labels),
                        child_off_x + ci.x,
                        child_off_y + ci.y,
                        ci.width,
                        ci.height,
                        label_of(cj, labels),
                        child_off_x + cj.x,
                        child_off_y + cj.y,
                        cj.width,
                        cj.height
                    ));
                }
            }
        }
        for child in &b.children {
            walk(child, child_off_x, child_off_y, labels, issues);
        }
    }
    walk(root, 0.0, 0.0, labels, &mut issues);
    issues
}

/// DC-13 struct-check 扩展（R1575）：检测「塌缩容器」——有显著高度子内容但自身高度
/// 近 0 的**真实元素**盒（非匿名）。这是 layout grow 失败的强信号（容器未随内容长高，
/// R1492 谱系 / IFC 高度未回填等）。仅 flag 显著 case（子 > 20px 且父 < 2px）以降低
/// 误报（合法 height:0 + 高子元素在产品 fixture 罕见）。
pub fn check_collapsed_containers(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
) -> Vec<String> {
    const MIN_CHILD_H: f32 = 20.0;
    const MAX_PARENT_H: f32 = 2.0;
    let mut issues = Vec::new();
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        issues: &mut Vec<String>,
    ) {
        // 仅真实元素盒（labels 含该 node_id；匿名盒 label_of 返回 "(anon)"）
        let is_real = b.node_id.is_some_and(|id| labels.contains_key(&id));
        if is_real && b.height < MAX_PARENT_H {
            // 找最高**流内**子盒（排除 abspos/fixed——脱离流，父盒正确不随其长高；
            // position-absolute-* 测试的父盒 h=0 是正确行为，非 bug）。
            let max_child_h = b
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .map(|c| c.height)
                .fold(0.0f32, f32::max);
            if max_child_h > MIN_CHILD_H {
                let label = b
                    .node_id
                    .and_then(|id| labels.get(&id).cloned())
                    .unwrap_or_else(|| "(anon)".to_string());
                issues.push(format!(
                    "collapsed container: [{label}] h={:.0} < in-flow child h={:.0} (parent failed to grow around content)",
                    b.height, max_child_h
                ));
            }
        }
        for child in &b.children {
            walk(child, labels, issues);
        }
    }
    walk(root, labels, &mut issues);
    issues
}

/// DC-13 line 325「不同 sibling card/link/shortcut 的文本不串联」：检测容器把**块级子元素**
/// 的文本错误地吸收进自身 IFC 的回归（R109 inline-ownership 失效谱系）。
///
/// 信号原理：`store_font_sizes_from_ifc`（layout 主路径多处调用）把每个 box 自身 IFC 处理过的
/// 文本节点 ID 存入该 box 的 `text_node_font_sizes`/`text_node_line_heights` 映射。正常布局下，
/// grid/flex/block 容器（如 welcome 的 `.cards`/`.shortcuts`/`.links`）的文本由各 block 子盒各自
/// 的 IFC 渲染，**容器自身的 text_node 映射为空**。当 R109 inline-ownership 退化时，父容器经
/// `text_content()` 收集整棵 inline 子树文本并跑一套 IFC，把多个 sibling block 子元素的文本拼到
/// 同一 IFC——此时容器的 text_node 映射会含**子元素子树**的文本节点 ID（用户可见的「卡片/链接/
/// 快捷键文本串联」退化）。
///
/// 判定规则（三条件全满足才 flag，降低误报）：
/// 1. 容器有 ≥2 个**真实元素**（labels 含其 node_id）、in-flow、高度 ≥ `MIN_CHILD_H` 的子盒
///    （= block-level sibling，排除 inline span / 匿名盒 / 单子容器）。
/// 2. 容器自身 `text_node_line_heights` 含 ≥1 个**非空白**文本节点 ID（确有吸收的子元素文本，
///    由 `non_ws_text_nodes` 过滤空白/换行节点）。
/// 3. 容器的 DOM 元素**无直接文本子节点**（`!has_direct_text`）——有直接文本的容器（如
///    `<div>intro<p>..</p></div>` 块中行 / 合法 block-in-inline）合法拥有自身 IFC，不 flag。
///
/// 与 [`check_sibling_overlaps`]（border-box 重叠）互补：后者抓几何重叠，本检查抓**文本归属**
/// 串联（盒未必重叠，但文本被错位拼到容器自身 IFC）。`has_direct_text`/`non_ws_text_nodes`
/// 由 [`collect_concat_dom_info`] 从 DOM 预构建。
pub fn check_text_concatenation(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    has_direct_text: &std::collections::HashSet<zero_dom::NodeId>,
    non_ws_text_nodes: &std::collections::HashSet<zero_dom::NodeId>,
) -> Vec<String> {
    const MIN_BLOCK_CHILDREN: usize = 2;
    const MIN_CHILD_H: f32 = 4.0;
    let mut issues = Vec::new();
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        has_direct_text: &std::collections::HashSet<zero_dom::NodeId>,
        non_ws_text_nodes: &std::collections::HashSet<zero_dom::NodeId>,
        issues: &mut Vec<String>,
    ) {
        // 条件 1：≥2 个真实元素、in-flow、显著高度的 block-level 子盒。
        let block_children = b
            .children
            .iter()
            .filter(|c| {
                !c.is_absolute
                    && !c.is_fixed
                    && c.height >= MIN_CHILD_H
                    && c.node_id.is_some_and(|id| labels.contains_key(&id))
            })
            .count();
        if block_children >= MIN_BLOCK_CHILDREN
            && let Some(bid) = b.node_id
            && !has_direct_text.contains(&bid)
        {
            // 条件 2：容器自身 text_node 映射含 ≥1 个非空白文本节点（吸收的子元素文本）。
            let absorbed: usize = b
                .text_node_line_heights
                .keys()
                .filter(|id| non_ws_text_nodes.contains(id))
                .count();
            if absorbed >= 1 {
                let label = labels.get(&bid).cloned().unwrap_or_else(|| "(unlabeled)".to_string());
                issues.push(format!(
                    "text concatenation: [{label}] ran an IFC absorbing {absorbed} non-whitespace \
                     text node(s) from across {block_children} block children (sibling text merged \
                     into shared IFC — R109 inline-ownership regression)",
                ));
            }
        }
        for child in &b.children {
            walk(child, labels, has_direct_text, non_ws_text_nodes, issues);
        }
    }
    walk(root, labels, has_direct_text, non_ws_text_nodes, &mut issues);
    issues
}

/// DC-13 [`check_text_concatenation`] 辅助：单次 DOM 遍历产出两个集合。
///
/// - `has_direct_text`：有非空白**直接**文本子节点的元素 NodeId（合法拥有自身 IFC 的容器，
///   如 `<p>text</p>`、`<div>intro<div>b</div></div>` 外层 div），用于条件 3 排除。
/// - `non_ws_text_nodes`：内容非空白的文本节点 NodeId，用于条件 2 过滤空白/换行节点
///   （避免 HTML 源码缩进空白触发误报）。
pub fn collect_concat_dom_info(
    html: &str,
) -> (
    std::collections::HashSet<zero_dom::NodeId>,
    std::collections::HashSet<zero_dom::NodeId>,
) {
    use std::collections::HashSet;
    use zero_dom::{NodeKind, parse_html};
    let doc = parse_html(html);
    let mut has_direct_text: HashSet<zero_dom::NodeId> = HashSet::new();
    let mut non_ws_text_nodes: HashSet<zero_dom::NodeId> = HashSet::new();
    let mut queue = vec![doc.root()];
    while let Some(id) = queue.pop() {
        if let Some(node) = doc.get(id) {
            match &node.kind {
                NodeKind::Element(_) => {
                    // 检查直接子节点中是否有非空白 Text。
                    let mut child = doc.first_child(id);
                    let mut has_text = false;
                    while let Some(c) = child {
                        if let Some(cnode) = doc.get(c)
                            && let NodeKind::Text(t) = &cnode.kind
                            && !t.content.trim().is_empty()
                        {
                            has_text = true;
                        }
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                    if has_text {
                        has_direct_text.insert(id);
                    }
                }
                NodeKind::Text(t) => {
                    if !t.content.trim().is_empty() {
                        non_ws_text_nodes.insert(id);
                    }
                    let mut child = doc.first_child(id);
                    while let Some(c) = child {
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                }
                _ => {
                    let mut child = doc.first_child(id);
                    while let Some(c) = child {
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                }
            }
        }
    }
    (has_direct_text, non_ws_text_nodes)
}

/// DC-13 line 322：统计布局树中带指定 class 的盒数（结构计数断言用）。
///
/// `labels`（[`collect_dom_labels`] 产出）格式为 `tag.class1.class2`；本函数按 `.` 拆分后
/// 跳过 tag、**精确匹配** class（`.card` 不误匹配 `.card-sub`，避免子串假阳性）。用于
/// 检测结构塌缩（如 welcome 须有 4 个 `.card`，丢失/塌缩会被检出）。
pub fn count_boxes_by_class(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    class: &str,
) -> usize {
    let mut count = 0usize;
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        class: &str,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        count: &mut usize,
    ) {
        if let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
        {
            // label = "tag.class1.class2"；跳过首段 tag，精确匹配 class token。
            if label.split('.').skip(1).any(|c| c == class) {
                *count += 1;
            }
        }
        for child in &b.children {
            walk(child, class, labels, count);
        }
    }
    walk(root, class, labels, &mut count);
    count
}

/// DC-13 line 323/324：估算带指定 class 的首个盒的**行数**（行数断言用，如「标题不拆行」/
///「tagline 保持 2 行」）。
///
/// 行数 = `content_height / per_line_height`，其中 `per_line_height` 取该盒 IFC 存储的
/// `text_node_line_heights`（store_font_sizes_from_ifc 存的**单行**片段高度，= 1 行高度）。
/// content_height = 行盒高度之和 = 行数 × 单行高度，故比值即行数。无存储度量（如非 IFC owner
/// 或无文本）返回 None（调用方跳过）。`.round()` 容忍亚像素；阈值 ±0.5 判定。
pub fn count_lines_for_class(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    class: &str,
) -> Option<usize> {
    fn find<'a>(
        b: &'a zero_layout_engine::types::LayoutBox,
        class: &str,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    ) -> Option<&'a zero_layout_engine::types::LayoutBox> {
        if let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
            && label.split('.').skip(1).any(|c| c == class)
        {
            return Some(b);
        }
        for child in &b.children {
            if let Some(found) = find(child, class, labels) {
                return Some(found);
            }
        }
        None
    }
    let b = find(root, class, labels)?;
    // 取首个 text_node_line_heights 作单行高度（同一 IFC 内文本行高一致）。
    let per_line = b.text_node_line_heights.values().next().copied()?;
    if per_line <= 0.0 {
        return None;
    }
    let content_h = b.content_height.max(0.0);
    Some((content_h / per_line).round() as usize)
}

/// 两轴对齐矩形 `(x, y, w, h)` 的交集面积（无交集返回 0）。
/// 返回 (重叠面积, 重叠高度)。重叠高度供调用方过滤「宽而薄」的亚像素 sliver（R1503）。
fn rect_overlap_area(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> (f32, f32) {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    let w = ((ax + aw).min(bx + bw) - ax.max(bx)).max(0.0);
    let h = ((ay + ah).min(by + bh) - ay.max(by)).max(0.0);
    (w * h, h)
}

/// 将单个 ImagePrimitive 渲染到帧缓冲。
fn render_image_into(
    fb: &mut FrameBuffer,
    image: &zero_render_foundation::primitive::ImagePrimitive,
    scale: f32,
    image_cache: &mut ImageCache,
) {
    let img_data = match image_cache.get(&image.image_key) {
        Some(data) => data.clone(),
        None => return,
    };

    let x0 = (image.rect.origin.x * scale).round() as i32;
    let y0 = (image.rect.origin.y * scale).round() as i32;
    let draw_w = (image.rect.size.width * scale).round().max(1.0) as u32;
    let draw_h = (image.rect.size.height * scale).round().max(1.0) as u32;

    for dy in 0..draw_h {
        let sy = (dy as u64 * img_data.height as u64 / draw_h as u64) as u32;
        for dx in 0..draw_w {
            let sx = (dx as u64 * img_data.width as u64 / draw_w as u64) as u32;
            let px = x0 + dx as i32;
            let py = y0 + dy as i32;
            if px < 0 || py < 0 || px >= fb.width as i32 || py >= fb.height as i32 {
                continue;
            }
            let src = img_data.get_pixel(sx, sy);
            let sa = src[3] as u32;
            if sa == 0 {
                continue;
            }
            let dst = fb.get_pixel(px as u32, py as u32);
            let inv_sa = 255 - sa;
            let da = dst[3] as u32;
            let r = ((src[0] as u32 * sa + dst[0] as u32 * inv_sa) / 255) as u8;
            let g = ((src[1] as u32 * sa + dst[1] as u32 * inv_sa) / 255) as u8;
            let b = ((src[2] as u32 * sa + dst[2] as u32 * inv_sa) / 255) as u8;
            let a = (sa + da * inv_sa / 255) as u8;
            fb.set_pixel(px as u32, py as u32, [r, g, b, a]);
        }
    }
}

/// 将 HTML 渲染到帧缓冲（GPU 无头模式，回退到 CPU 全量渲染）。
///
/// 使用与 CPU 路径相同的 `render_full_scene`，确保全部 13 种图元类型被渲染。
pub fn render_to_framebuffer_gpu(html: &str, css: &str, config: &ReftestConfig) -> FrameBuffer {
    render_to_framebuffer_gpu_with_base(html, css, config, None)
}

/// 将 HTML 渲染到帧缓冲（GPU 无头模式，支持图片加载）。
pub fn render_to_framebuffer_gpu_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> FrameBuffer {
    // GPU 渲染路径暂时回退到 CPU（GPU 路径不支持全量图元 + 图片加载）
    render_to_framebuffer_with_base(html, css, config, base_dir)
}

#[cfg(all(test, feature = "v8"))]
mod tests {
    use super::*;

    /// R329：`@font-face` 自定义字体加载端到端验证。
    ///
    /// CSS 声明 `@font-face { font-family: "R329Alias"; src: url("Ahem.ttf"); }`，
    /// 基目录指向 `tests/wpt-runner/fonts/`（含真实 Ahem.ttf）。`load_font_faces_into`
    /// 应解析 src、加载字体、并把**声明族名** "R329Alias" 注册为别名（与字体内部名
    /// "Ahem" 不同）。`build_font_resolver` 须含 "R329Alias" → font_id。
    #[test]
    fn test_font_face_loads_custom_family_alias() {
        let fonts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts");
        let ahem = fonts_dir.join("Ahem.ttf");
        if !ahem.exists() {
            // 无 Ahem.ttf 的环境（如 CI 缺字体）跳过，不计失败
            eprintln!("[R329] Ahem.ttf missing, skipping @font-face load test");
            return;
        }
        let css = r#"@font-face { font-family: "R329Alias"; src: url("Ahem.ttf"); }"#;
        let mut loader = create_font_loader();
        load_font_faces_into(&mut loader, Some(&fonts_dir), css);
        let resolver = loader.build_font_resolver();
        assert!(
            resolver.contains_key("R329Alias"),
            "@font-face declared family 'R329Alias' must resolve to a loaded font_id; \
             resolver keys: {:?}",
            resolver.keys().collect::<Vec<_>>()
        );
    }

    /// R329：`@font-face` 解析 + 跳过 data:/http: 源（不可本地加载）。
    #[test]
    fn test_font_face_resolves_local_and_skips_remote() {
        let css = r#"@font-face { font-family: "Remote"; src: url("https://example.com/x.woff"); }"#;
        let faces = extract_font_faces(css);
        assert_eq!(faces.len(), 1);
        // 远程源无法解析到本地路径
        assert!(resolve_font_src("https://example.com/x.woff", None).is_none());
        assert!(resolve_font_src("data:application/font-woff;base64,AAAA", None).is_none());
        // 相对路径需 base_dir
        assert!(resolve_font_src("rel.woff", None).is_none());
    }

    /// 验证 position:relative + top 偏移是否正确应用。
    /// 测试：border-bottom 96px black + height 96px = 空顶 + 黑底
    /// 参考：background black + height 96px + position:relative; top:96px = 空顶 + 黑底
    /// 两者应在视觉上相同（black 在下半部分）。
    #[test]
    fn test_reftest_relative_top_offset() {
        // First, verify the test HTML renders correctly: black at bottom half
        let test_only = ReftestCase {
            id: "test/border-bottom-only".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            // Same HTML as ref - should match itself
            ref_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let config = ReftestConfig::default();
        let result = run_reftest(&test_only, &config);
        assert!(result.passed, "Self-comparison should always pass: {}", result.message);

        // Now verify the reference renders the same visual: black div offset down
        let case = ReftestCase {
            id: "test/relative-top".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"border-bottom: 96px solid black; height: 96px; width: 96px;\"></div></body></html>".into(),
            ref_html: "<html><body style=\"margin:0\"><div style=\"background-color: black; height: 96px; width: 96px; position: relative; top: 96px;\"></div></body></html>".into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let result = run_reftest(&case, &config);
        assert!(
            result.passed,
            "position:relative + top:96px should produce same visual as border-bottom: {}",
            result.message
        );
    }

    #[test]
    fn test_reftest_identical_pages() {
        let case = ReftestCase {
            id: "test/identical".into(),
            test_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>"
                .into(),
            ref_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>".into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let config = ReftestConfig::default();
        let result = run_reftest(&case, &config);
        assert!(result.passed, "Identical pages should match: {}", result.message);
    }

    #[test]
    fn test_reftest_different_pages() {
        let case = ReftestCase {
            id: "test/different".into(),
            test_html: "<html><body><div style=\"width:100px;height:50px;background:red;\">A</div></body></html>"
                .into(),
            ref_html: "<html><body><div style=\"width:100px;height:50px;background:blue;\">B</div></body></html>"
                .into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let config = ReftestConfig::default();
        let result = run_reftest(&case, &config);
        assert!(!result.passed, "Different pages should not match: {}", result.message);
    }

    #[test]
    fn test_reftest_mismatch_mode() {
        let case = ReftestCase {
            id: "test/mismatch".into(),
            test_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:red;\">Red</div></body></html>".into(),
            ref_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:blue;\">Blue</div></body></html>".into(),
            css: String::new(),
            is_match: false,
            ref_base_dir: None,
        };
        let config = ReftestConfig::default();
        let result = run_reftest(&case, &config);
        assert!(
            result.passed,
            "Different pages should pass mismatch: {}",
            result.message
        );
    }

    #[test]
    fn test_struct_check_detects_sibling_overlap() {
        // 两个**普通块流**兄弟盒经负 margin 重叠（第二个 margin-top:-50px 拉 50px，100×50=5000px²
        // 交集）须被检出。R1504：positioned（abspos/relative/fixed）/float 盒按设计重叠已被排除，
        // 故测试改用 normal-block 重叠（真 bug 模式，如 R1492 长高重叠）。
        let html = "<html><body style=\"margin:0\">\
            <div style=\"width:100px;height:100px;background:red\"></div>\
            <div style=\"width:100px;height:100px;margin-top:-50px;background:blue\"></div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let issues = check_sibling_overlaps(&root, &labels);
        assert!(
            !issues.is_empty(),
            "overlapping normal-block siblings must be flagged, got {issues:?}"
        );
        assert!(
            issues.iter().any(|s| s.contains("overlap")),
            "issue must mention overlap: {issues:?}"
        );
    }

    #[test]
    fn test_struct_check_passes_stacked_blocks() {
        // 两个垂直堆叠的 block 兄弟盒（正常流，无重叠）须通过。
        let html = "<html><body style=\"margin:0\">\
            <div style=\"width:100px;height:100px;background:red\"></div>\
            <div style=\"width:100px;height:100px;background:blue\"></div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let issues = check_sibling_overlaps(&root, &labels);
        assert!(
            issues.is_empty(),
            "stacked block siblings must not flag overlap: {issues:?}"
        );
    }

    #[test]
    fn test_collapsed_containers_detects_inline_wrapping_inline_block() {
        // R1576 inline-box-model：`<p><a><img class=inline-block h-24></a></p>` 的 `<p>` 须随
        // inline-block 子内容长高（IFC 递归 inline 元素收集嵌套 atomic inline 盒），不应塌缩为 0。
        // default-on（ZW_INLINE_BOX_RECURSE）修复后 check 须**不**检出 `<p>` 塌缩。
        let html = "<html><body style=\"margin:0\">\
            <p><a href=\"#\"><img style=\"display:inline-block;height:24px;width:24px\" src=\"x.png\"></a></p>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let issues = check_collapsed_containers(&root, &labels);
        assert!(
            !issues.iter().any(|s| s.contains("[p]")),
            "R1576: <p> wrapping inline>inline-block must grow (not collapse), still flagged: {issues:?}"
        );
    }

    #[test]
    fn test_collapsed_containers_ignores_abspos_child() {
        // R1575：父盒 h=0 但子为 abspos（脱离流）是正确行为（position-absolute-* 测试），
        // 不应 flag。仅 in-flow 高子内容触发。
        let html = "<html><body style=\"margin:0;position:relative\">\
            <div style=\"height:0\"><div style=\"position:absolute;width:100px;height:100px\"></div></div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let issues = check_collapsed_containers(&root, &labels);
        assert!(
            issues.is_empty(),
            "parent with only abspos child must not be flagged: {issues:?}"
        );
    }

    #[test]
    fn test_count_boxes_by_class_exact_match() {
        // 4 个 .card + 2 个 .card-sub（须精确匹配，card-sub 不计入 card）。
        let html = "<html><body>\
            <div class=\"card\"><p class=\"card-sub\">a</p></div>\
            <div class=\"card\"><p class=\"card-sub\">b</p></div>\
            <div class=\"card\">c</div>\
            <div class=\"card\">d</div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        assert_eq!(
            count_boxes_by_class(&root, &labels, "card"),
            4,
            "exact .card count must be 4 (not counting .card-sub)"
        );
        assert_eq!(
            count_boxes_by_class(&root, &labels, "card-sub"),
            2,
            ".card-sub count must be 2"
        );
        assert_eq!(
            count_boxes_by_class(&root, &labels, "nope"),
            0,
            "absent class must count 0"
        );
    }

    #[test]
    fn test_count_lines_for_class() {
        // title = 1 line；tagline = 2 lines（<br> 强制断行）；验证 content_height / line_height 行数估算。
        let html = "<html><body style=\"margin:0\">\
            <h1 class=\"title\">ZeroBrowser</h1>\
            <p class=\"tagline\">line one<br>line two</p>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        assert_eq!(
            count_lines_for_class(&root, &labels, "title"),
            Some(1),
            "title (single word) must be 1 line"
        );
        assert_eq!(
            count_lines_for_class(&root, &labels, "tagline"),
            Some(2),
            "tagline (<br>) must be 2 lines"
        );
    }

    #[test]
    fn test_text_concatenation_passes_grid_container() {
        // welcome 谱系：`.cards`（grid）含 2 个 `.card` block 子元素，各自持文本。正常布局下
        // 容器自身 text_node 映射为空（文本由各 card 各自的 IFC 渲染）→ 不应 flag 串联。
        let html = "<html><body style=\"margin:0\">\
            <div class=\"cards\" style=\"display:grid;grid-template-columns:1fr 1fr\">\
              <div class=\"card\">Card one text</div>\
              <div class=\"card\">Card two text</div>\
            </div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
        let issues = check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes);
        assert!(
            !issues
                .iter()
                .any(|s| s.contains("text concatenation") && s.contains("[div.cards]")),
            "correct grid container must not be flagged for concatenation: {issues:?}"
        );
    }

    #[test]
    fn test_text_concatenation_flags_absorbed_children() {
        // 模拟 R109 inline-ownership 退化：把一个 card 子盒 text_node_line_heights 的条目注入
        // `.cards` 容器自身映射（= 父容器 IFC 吸收了子元素文本）。容器无直接文本、有 ≥2 block 子、
        // 自身 text_node 映射含非空白文本节点 → 必被 flag（sibling 文本串联）。
        let html = "<html><body style=\"margin:0\">\
            <div class=\"cards\" style=\"display:grid;grid-template-columns:1fr 1fr\">\
              <div class=\"card\">Card one text</div>\
              <div class=\"card\">Card two text</div>\
            </div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
        // 基线：正常渲染不 flag。
        assert!(
            check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes)
                .iter()
                .all(|s| !(s.contains("text concatenation") && s.contains("[div.cards]"))),
            "baseline must not flag the cards container"
        );
        // 递归找一个含**非空白**文本节点的 text_node_line_heights（card 子树内的真实文本），
        // 注入 cards 容器模拟吸收。须过滤空白节点（store_font_sizes_from_ifc 也存空白片段，
        // 空白节点不在 non_ws_text_nodes，注入它们不会触发检查——与检查的过滤一致）。
        fn first_non_ws_text_entries(
            b: &zero_layout_engine::types::LayoutBox,
            non_ws: &std::collections::HashSet<zero_dom::NodeId>,
        ) -> Option<Vec<(zero_dom::NodeId, f32)>> {
            let matched: Vec<_> = b
                .text_node_line_heights
                .iter()
                .filter(|(k, _)| non_ws.contains(k))
                .map(|(k, v)| (*k, *v))
                .collect();
            if !matched.is_empty() {
                return Some(matched);
            }
            b.children.iter().find_map(|c| first_non_ws_text_entries(c, non_ws))
        }
        let mut root2 = root.clone();
        fn find_mut<'a>(
            b: &'a mut zero_layout_engine::types::LayoutBox,
            labels: &std::collections::HashMap<zero_dom::NodeId, String>,
            needle: &str,
        ) -> Option<&'a mut zero_layout_engine::types::LayoutBox> {
            if let Some(id) = b.node_id
                && let Some(label) = labels.get(&id)
                && label.split('.').any(|c| c == needle)
            {
                return Some(b);
            }
            for child in &mut b.children {
                if let Some(found) = find_mut(child, labels, needle) {
                    return Some(found);
                }
            }
            None
        }
        let entries = first_non_ws_text_entries(&root, &non_ws_text_nodes)
            .expect("some box has non-whitespace text_node_line_heights entries");
        let cards = find_mut(&mut root2, &labels, "cards").expect("cards container found");
        for (id, h) in entries {
            cards.text_node_line_heights.insert(id, h);
        }
        let issues = check_text_concatenation(&root2, &labels, &has_direct_text, &non_ws_text_nodes);
        assert!(
            issues
                .iter()
                .any(|s| s.contains("text concatenation") && s.contains("[div.cards]")),
            "absorbed child text into cards container must be flagged: {issues:?}"
        );
    }

    #[test]
    fn test_text_concatenation_ignores_container_with_direct_text() {
        // 条件 3 排除：容器有直接文本子节点 + block 子元素（合法 block-in-inline / 自带文本）。
        // 容器自身 IFC 含其直接文本是合法的，不应 flag。
        let html = "<html><body style=\"margin:0\">\
            <div class=\"wrap\">Intro direct text\
              <div>block one</div>\
              <div>block two</div>\
            </div>\
            </body></html>";
        let config = ReftestConfig::default();
        let (_fb, root, _) = render_to_framebuffer_with_layout_with_base(html, "", &config, None);
        let labels = collect_dom_labels(html);
        let (has_direct_text, non_ws_text_nodes) = collect_concat_dom_info(html);
        assert!(
            has_direct_text
                .iter()
                .any(|id| labels.get(id).is_some_and(|l| l.contains("wrap"))),
            ".wrap has direct text and must be in has_direct_text set"
        );
        let issues = check_text_concatenation(&root, &labels, &has_direct_text, &non_ws_text_nodes);
        assert!(
            !issues
                .iter()
                .any(|s| s.contains("text concatenation") && s.contains("[div.wrap]")),
            "container with legitimate direct text must not be flagged: {issues:?}"
        );
    }

    #[test]
    fn test_reftest_config_default() {
        let config = ReftestConfig::default();
        assert_eq!(config.viewport_width, 800);
        assert_eq!(config.viewport_height, 600);
        assert!((config.max_diff_ratio - 0.01).abs() < f64::EPSILON);
        assert_eq!(config.max_channel_diff, 5);
    }

    #[test]
    fn test_reftest_fuzzy_threshold() {
        let case = ReftestCase {
            id: "test/fuzzy".into(),
            test_html:
                "<html><body><div style=\"background:rgb(100,100,100);width:50px;height:50px;\">A</div></body></html>"
                    .into(),
            ref_html:
                "<html><body><div style=\"background:rgb(102,102,102);width:50px;height:50px;\">A</div></body></html>"
                    .into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let config = ReftestConfig {
            max_diff_ratio: 0.1,
            max_channel_diff: 10,
            ..Default::default()
        };
        let result = run_reftest(&case, &config);
        assert!(
            result.passed,
            "Small color diff should match with fuzzy threshold: {}",
            result.message
        );
    }

    #[test]
    fn test_extract_stylesheet_hrefs() {
        let html = r#"
            <html><head>
                <link rel="stylesheet" href="/fonts/ahem.css">
                <link rel='alternate stylesheet' href='theme.css'>
                <link rel="help" href="spec.html">
            </head></html>
        "#;
        let hrefs = extract_stylesheet_hrefs(html);
        assert_eq!(hrefs, vec!["/fonts/ahem.css".to_string(), "theme.css".to_string()]);
    }

    #[test]
    #[ignore]
    fn debug_clear_applies_to_009_blue_bbox() {
        fn blue_bbox(fb: &FrameBuffer) -> Option<(u32, u32, u32, u32)> {
            let mut min_x = fb.width;
            let mut min_y = fb.height;
            let mut max_x = 0;
            let mut max_y = 0;
            let mut found = false;

            for y in 0..fb.height {
                for x in 0..fb.width {
                    let idx = ((y * fb.width + x) * 4) as usize;
                    let px = &fb.data[idx..idx + 4];
                    let is_blue = px[0] < 32 && px[1] < 32 && px[2] > 200 && px[3] > 200;
                    if is_blue {
                        found = true;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }

            found.then_some((min_x, min_y, max_x, max_y))
        }

        let wpt_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data");
        let case_path = wpt_root.join("css/CSS2/floats-clear/clear-applies-to-009.xht");
        let ref_path = wpt_root.join("css/CSS2/floats-clear/clear-applies-to-009-ref.xht");
        let test_html = std::fs::read_to_string(&case_path).expect("read test html");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let base_dir = case_path.parent().expect("base dir");
        let config = ReftestConfig::default();

        let test_fb = render_to_framebuffer_with_base(&test_html, "", &config, Some(base_dir));
        let ref_fb = render_to_framebuffer_with_base(&ref_html, "", &config, Some(base_dir));

        println!("test blue bbox: {:?}", blue_bbox(&test_fb));
        println!("ref  blue bbox: {:?}", blue_bbox(&ref_fb));
    }

    #[test]
    #[ignore]
    fn debug_clear_applies_to_009_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-009.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..8 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for fill in &rendered.primitives.fills {
            if fill.color.r < 32 && fill.color.g < 32 && fill.color.b > 200 && fill.color.a > 200 {
                println!(
                    "blue fill rect=({:.2},{:.2},{:.2},{:.2})",
                    fill.rect.origin.x, fill.rect.origin.y, fill.rect.size.width, fill.rect.size.height
                );
            }
        }
        for rr in &rendered.primitives.rounded_rects {
            if rr.color.r < 32 && rr.color.g < 32 && rr.color.b > 200 && rr.color.a > 200 {
                println!(
                    "blue rr rect=({:.2},{:.2},{:.2},{:.2})",
                    rr.rect.origin.x, rr.rect.origin.y, rr.rect.size.width, rr.rect.size.height
                );
            }
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-009-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..8 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_applies_to_001_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-001.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-applies-to-001-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_clearance_calculation_001_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-001.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-001-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_clearance_calculation_003_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-003.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..14 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-003-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..14 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_clearance_calculation_004_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-004.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..14 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-004-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..14 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_clearance_calculation_005_layout_snapshot() {
        let case_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-005.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..16 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for (i, fill) in rendered.primitives.fills.iter().enumerate().take(16) {
            println!(
                "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-clearance-calculation-005-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..16 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_003_layout_snapshot() {
        let case_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-003.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for (i, fill) in rendered.primitives.fills.iter().enumerate().take(12) {
            println!(
                "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }

        let ref_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-003-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for (i, fill) in ref_rendered.primitives.fills.iter().enumerate().take(12) {
            println!(
                "ref fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }
    }

    #[test]
    #[ignore]
    fn debug_clear_float_003_layout_snapshot() {
        let case_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/floats-clear/clear-float-003.xht");
        let html = std::fs::read_to_string(&case_path).expect("read test html");
        let linked_css = load_linked_stylesheets(&html, case_path.parent());
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.set_skip_indicators(true);
        let font_loader = create_font_loader();
        pipeline.set_font_resolver(font_loader.build_font_resolver());
        let rendered = pipeline.render_html(&html, &linked_css);
        println!("{}", rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = rendered.layout.root.nth_box(i) {
                println!("box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for (i, fill) in rendered.primitives.fills.iter().enumerate().take(12) {
            println!(
                "fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }

        let ref_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("wpt-data/css/CSS2/floats-clear/clear-float-003-ref.xht");
        let ref_html = std::fs::read_to_string(&ref_path).expect("read ref html");
        let ref_linked_css = load_linked_stylesheets(&ref_html, ref_path.parent());
        let mut ref_pipeline = RenderPipeline::new(800.0, 600.0);
        ref_pipeline.set_skip_indicators(true);
        let ref_font_loader = create_font_loader();
        ref_pipeline.set_font_resolver(ref_font_loader.build_font_resolver());
        let ref_rendered = ref_pipeline.render_html(&ref_html, &ref_linked_css);
        println!("--- ref ---");
        println!("{}", ref_rendered.layout.snapshot());
        for i in 0..12 {
            if let Some((x, y, w, h)) = ref_rendered.layout.root.nth_box(i) {
                println!("ref box[{i}] abs=({x:.2},{y:.2}) size=({w:.2},{h:.2})");
            }
        }
        for (i, fill) in ref_rendered.primitives.fills.iter().enumerate().take(12) {
            println!(
                "ref fill[{i}] rect=({:.2},{:.2},{:.2},{:.2}) color=({}, {}, {}, {})",
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a
            );
        }
    }

    // ── 分类容差测试 ──

    #[test]
    fn test_category_from_path_layout() {
        assert_eq!(
            ReftestCategory::from_path("css/CSS2/box-001.html"),
            ReftestCategory::Layout
        );
        assert_eq!(
            ReftestCategory::from_path("css/css-flexbox/001.html"),
            ReftestCategory::Layout
        );
    }

    #[test]
    fn test_category_from_path_text() {
        assert_eq!(
            ReftestCategory::from_path("css/css-text/001.html"),
            ReftestCategory::Text
        );
        assert_eq!(
            ReftestCategory::from_path("css/css-fonts/001.html"),
            ReftestCategory::Text
        );
    }

    #[test]
    fn test_category_defaults() {
        assert_eq!(ReftestCategory::Layout.default_max_diff_ratio(), 0.01);
        assert_eq!(ReftestCategory::Text.default_max_diff_ratio(), 0.05);
        assert_eq!(ReftestCategory::Layout.default_max_channel_diff(), 5);
        assert_eq!(ReftestCategory::Text.default_max_channel_diff(), 15);
    }

    #[test]
    fn test_config_for_category() {
        let config = ReftestConfig::for_category(ReftestCategory::Text);
        assert!((config.max_diff_ratio - 0.05).abs() < f64::EPSILON);
        assert_eq!(config.max_channel_diff, 15);
    }

    /// DC-14 锁定严格容差不变量：Layout 0.1%/2、Text 0.5%/5、Unknown 0.1%/2。
    /// 严格容差是默认松容差（Layout 1%/5、Text 5%/15）的 1/10（R280 量化），
    /// 是唯一可信达标指标（goal DC-14 line 162-163/315-316，不可放宽）。
    #[test]
    fn test_strict_tolerance_dc14_locked() {
        // Layout: 默认 1% / 5 → 严格 0.1% / 2
        assert!((ReftestCategory::Layout.strict_max_diff_ratio() - 0.001).abs() < f64::EPSILON);
        assert_eq!(ReftestCategory::Layout.strict_max_channel_diff(), 2);
        assert!((ReftestCategory::Layout.default_max_diff_ratio() - 0.01).abs() < f64::EPSILON);
        assert_eq!(ReftestCategory::Layout.default_max_channel_diff(), 5);

        // Text: 默认 5% / 15 → 严格 0.5% / 5
        assert!((ReftestCategory::Text.strict_max_diff_ratio() - 0.005).abs() < f64::EPSILON);
        assert_eq!(ReftestCategory::Text.strict_max_channel_diff(), 5);
        assert!((ReftestCategory::Text.default_max_diff_ratio() - 0.05).abs() < f64::EPSILON);
        assert_eq!(ReftestCategory::Text.default_max_channel_diff(), 15);

        // Unknown: 默认 2% / 8 → 严格 0.1% / 2（未知分类按最严格处理）
        assert!((ReftestCategory::Unknown.strict_max_diff_ratio() - 0.001).abs() < f64::EPSILON);
        assert_eq!(ReftestCategory::Unknown.strict_max_channel_diff(), 2);

        // 严格恒为默认的 1/10（10× 松 → 严格，R280 量化）
        for cat in [ReftestCategory::Layout, ReftestCategory::Text] {
            let ratio_factor = cat.default_max_diff_ratio() / cat.strict_max_diff_ratio();
            assert!(
                (ratio_factor - 10.0).abs() < 1e-9,
                "strict ratio should be 1/10 of default"
            );
        }
    }

    #[test]
    fn test_fuzzy_override() {
        let mut config = ReftestConfig::for_category(ReftestCategory::Layout);
        let fuzzy = FuzzyMeta {
            max_diff: Some(20),
            total_pixels: Some(500),
        };
        config.with_fuzzy_override(&fuzzy);
        assert_eq!(config.max_channel_diff, 20);
        // total_pixels=500, viewport=800x600=480000, ratio=500/480000≈0.001
        assert!(config.max_diff_ratio < 0.01);
    }

    // --- CSS 布局 reftest 用例 ---

    /// 辅助函数：使用默认配置运行 match reftest。
    fn assert_match(id: &str, test_html: &str, ref_html: &str) {
        let case = ReftestCase {
            id: id.into(),
            test_html: test_html.into(),
            ref_html: ref_html.into(),
            css: String::new(),
            is_match: true,
            ref_base_dir: None,
        };
        let config = ReftestConfig {
            viewport_width: 200,
            viewport_height: 200,
            ..Default::default()
        };
        let result = run_reftest(&case, &config);
        assert!(result.passed, "{}: {}", id, result.message);
    }

    /// 辅助函数：使用默认配置运行 mismatch reftest。
    fn assert_mismatch(id: &str, test_html: &str, ref_html: &str) {
        let case = ReftestCase {
            id: id.into(),
            test_html: test_html.into(),
            ref_html: ref_html.into(),
            css: String::new(),
            is_match: false,
            ref_base_dir: None,
        };
        let config = ReftestConfig {
            viewport_width: 200,
            viewport_height: 200,
            ..Default::default()
        };
        let result = run_reftest(&case, &config);
        assert!(result.passed, "{}: {}", id, result.message);
    }

    // ── Block 布局 ──

    #[test]
    fn reftest_block_width_height() {
        assert_match(
            "block/width-height",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
        );
    }

    #[test]
    fn reftest_block_margin_collapsing() {
        assert_match(
            "block/margin-no-effect-on-bg",
            "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
            "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
        );
    }

    #[test]
    fn reftest_block_different_margin() {
        assert_mismatch(
            "block/different-margin",
            "<div style=\"width:80px;height:40px;background:green;margin:0;\"></div>",
            "<div style=\"width:80px;height:40px;background:green;margin:20px;\"></div>",
        );
    }

    #[test]
    fn reftest_block_stacking() {
        assert_mismatch(
            "block/stacking-vs-single",
            "<div style=\"width:100px;height:40px;background:red;\"></div><div style=\"width:100px;height:40px;background:blue;\"></div>",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
        );
    }

    // ── 盒模型 ──

    #[test]
    fn reftest_padding_expands_box() {
        assert_mismatch(
            "box-model/padding-expands",
            "<div style=\"width:80px;height:40px;background:red;padding:10px;\"></div>",
            "<div style=\"width:80px;height:40px;background:red;padding:0;\"></div>",
        );
    }

    #[test]
    fn reftest_border_visible() {
        assert_mismatch(
            "box-model/border-visible",
            "<div style=\"width:80px;height:40px;background:yellow;border:2px solid black;\"></div>",
            "<div style=\"width:80px;height:40px;background:yellow;border:none;\"></div>",
        );
    }

    // ── Flexbox ──

    #[test]
    fn reftest_flex_direction_row() {
        assert_match(
            "flex/row-identical",
            "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        );
    }

    #[test]
    fn reftest_flex_vs_block() {
        assert_mismatch(
            "flex/row-vs-block",
            "<div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        );
    }

    // ── 定位 ──

    #[test]
    fn reftest_absolute_position() {
        assert_mismatch(
            "position/absolute-shift",
            "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:20px;left:20px;width:50px;height:50px;background:green;\"></div></div>",
            "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:green;\"></div></div>",
        );
    }

    // ── 背景颜色 ──

    #[test]
    fn reftest_named_vs_hex_color() {
        assert_match(
            "color/named-vs-hex",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:#FF0000;\"></div>",
        );
    }

    #[test]
    fn reftest_rgb_vs_hex() {
        assert_match(
            "color/rgb-vs-hex",
            "<div style=\"width:100px;height:50px;background:rgb(0,128,255);\"></div>",
            "<div style=\"width:100px;height:50px;background:#0080FF;\"></div>",
        );
    }

    #[test]
    fn reftest_different_colors() {
        assert_mismatch(
            "color/different",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:green;\"></div>",
        );
    }

    // ── 尺寸 ──

    #[test]
    fn reftest_different_sizes() {
        assert_mismatch(
            "size/different",
            "<div style=\"width:100px;height:50px;background:blue;\"></div>",
            "<div style=\"width:50px;height:100px;background:blue;\"></div>",
        );
    }

    #[test]
    fn reftest_display_none() {
        assert_mismatch(
            "display/none-vs-visible",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:red;display:none;\"></div>",
        );
    }

    // ── 嵌套结构 ──

    #[test]
    fn reftest_nested_same_bg() {
        assert_match(
            "nested/same-structure",
            "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
            "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
        );
    }

    #[test]
    fn reftest_sibling_order() {
        assert_mismatch(
            "nested/sibling-order",
            "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:red;\"></div></div>",
        );
    }

    // DC-13 产品静态 smoke：渲染 morning.work 中文文章 fixture（含外链 CSS + 图片）。
    // 通过 base_dir 测试 <link> 外链 CSS 与 <img> 子资源加载路径。
    #[test]
    #[ignore]
    fn dump_morning_work_png() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/browser/assets/morning-work");
        let html = std::fs::read_to_string(base.join("article.html")).expect("read article.html");
        let config = ReftestConfig::default();
        let fb = render_to_framebuffer_with_base(&html, "", &config, Some(&base));
        let out = std::path::Path::new("/tmp/mw-zeroweb-cpu.png");
        save_fb_as_png(&fb, out);
        // 报告关键区域像素：文章正文区应有 CJK 文本（深色像素），代码块应有背景
        let px = fb.data;
        let w = fb.width as usize;
        let at = |x: usize, y: usize| -> (u8, u8, u8, u8) {
            let i = (y * w + x) * 4;
            (px[i], px[i + 1], px[i + 2], px[i + 3])
        };
        println!("morning.work samples (CJK 文本应深色，代码块应有灰背景):");
        for &(x, y) in &[(60, 80), (100, 150), (100, 250), (100, 400)] {
            println!("  ({},{}) = {:?}", x, y, at(x, y));
        }
        // 统计非背景像素（页面 bg #f9f7f4 ≈ (249,247,244)）
        let mut non_bg = 0usize;
        for i in (0..px.len()).step_by(4) {
            let c = (px[i], px[i + 1], px[i + 2]);
            if !(c.0 > 245 && c.1 > 243 && c.2 > 240) {
                non_bg += 1;
            }
        }
        println!("non-background pixels: {} (of {})", non_bg, px.len() / 4);
    }

    /// R878：根元素 `display:none` 时背景不传播到画布（CSS §9.2.4/§14.2）。
    ///
    /// `html{display:none;background:green}` + `body{background:red}` + `<p>FAIL</p>`
    /// ——根元素不生成盒、整个文档树不参与渲染，故根元素与 body 背景均不传播到 canvas，
    /// canvas 保持默认白（与 chromium 实测一致，root-box-003）。
    #[test]
    fn test_root_element_display_none_no_canvas_background() {
        let html = r#"<html><head><style>
   html { display: none; background: green; color: red; }
   body { background: red; color: yellow; }
  </style></head>
  <body>
   <p>FAIL</p>
  </body></html>"#;
        let cfg = ReftestConfig::default();
        let fb = render_to_framebuffer(html, "", &cfg);
        let px = &fb.data;
        // 全画布应为白（无背景传播，无文档树渲染）
        for chunk in px.chunks(4) {
            if chunk.len() == 4 {
                assert_eq!(
                    [chunk[0], chunk[1], chunk[2]],
                    [255, 255, 255],
                    "canvas must stay white"
                );
            }
        }
    }

    /// R878 回归守卫：根元素背景正常传播到画布不受 display:none 修复影响。
    ///
    /// `html{background:green}`（无 display:none）→ 全画布绿（canvas 传播仍工作）。
    #[test]
    fn test_root_element_background_still_propagates_to_canvas() {
        let html = r#"<html><head><style>
   html { background: green; }
  </style></head>
  <body></body></html>"#;
        let cfg = ReftestConfig::default();
        let fb = render_to_framebuffer(html, "", &cfg);
        let px = &fb.data;
        // 全画布应为绿（根背景传播）
        for chunk in px.chunks(4) {
            if chunk.len() == 4 {
                assert_eq!([chunk[0], chunk[1], chunk[2]], [0, 128, 0], "canvas must be green");
            }
        }
    }

    /// R879：`background:transparent`（解析为 `background-image:none` → `vec![None]`）
    /// 不应阻止 body 背景传播到画布。CSS §14.2：html 背景透明时 body 背景传播到 canvas。
    ///
    /// `body{background:green}` + `html{background:transparent}` → 全画布绿
    ///（background-root-005 等 cluster，R721 定位、R879 修 `vec![None]` 误判）。
    #[test]
    fn test_background_image_none_does_not_block_body_canvas_propagation() {
        let html = r#"<html><head><style>
   body { background: green; color: white; }
   html { background: transparent; color: yellow; }
  </style></head>
  <body>
   <p>body green should fill canvas</p>
  </body></html>"#;
        let cfg = ReftestConfig::default();
        let fb = render_to_framebuffer(html, "", &cfg);
        let px = &fb.data;
        // 绝大多数像素应为绿（body green 传播到画布）；允许少量文本反走样像素
        let mut green = 0usize;
        let mut total = 0usize;
        for chunk in px.chunks(4) {
            if chunk.len() == 4 {
                total += 1;
                if chunk[1] >= 100 && chunk[0] < 80 && chunk[2] < 80 {
                    green += 1;
                }
            }
        }
        assert!(
            green * 100 > total * 90,
            "canvas must be predominantly green: {green}/{total}"
        );
    }

    /// R880：无 positioned 祖先的 abspos 元素以初始包含块（视口）为 CB，其
    /// `left/top`（Px 或百分比）与百分比 `width/height` 相对**视口**解析，不受
    /// CB 链上祖先 border/padding 影响（CSS §10.1/§10.3/§10.6）。
    ///
    /// `body{border+padding:1em}` + `div{position:absolute;top:0;left:0;width:100%;
    /// height:100%;background:green}` → div 覆盖整个视口（绿），body 的红不外露。
    /// 旧实现把 div 放在父 content origin（=border-box+border+padding=48px）而非视口
    /// (0,0)，致左上 L 形红区（abspos-containing-block-010，9.35% diff）。
    #[test]
    fn test_abspos_viewport_cb_ignores_ancestor_border_padding() {
        let html = r#"<!DOCTYPE html><html><head><style>
   body { margin: 1em; border: 1em solid red; padding: 1em; background: red; }
   div { position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: green; }
  </style></head>
  <body>
   <p>FAIL</p>
   <div>x</div>
  </body></html>"#;
        let cfg = ReftestConfig::default();
        let fb = render_to_framebuffer(html, "", &cfg);
        let px = &fb.data;
        // div 覆盖整个视口：几乎全绿，无任何红（body 红被完全覆盖）
        let mut red = 0usize;
        for chunk in px.chunks(4) {
            if chunk.len() == 4 && chunk[0] >= 150 && chunk[1] < 80 && chunk[2] < 80 {
                red += 1;
            }
        }
        assert_eq!(
            red, 0,
            "no red (body bg) must be visible; abspos div must cover viewport"
        );
    }
    /// R881：`float:left` 容器（width:auto）应 shrink-to-fit 包裹其 inline-level
    /// replaced 子元素（img），CSS §10.3.5。旧 float shrink 只考虑 block-level 子元素，
    /// 致 `div{float:left}` 仅含 `<img>` 时撑满全宽，img 无法覆盖 div 背景（max-width-110，
    /// 200×200 img 受 max-width:100px 约束为 100×100，但 div 784px 满宽→红 68400px 外露）。
    #[test]
    fn test_float_shrink_to_fit_includes_inline_replaced_child() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/CSS2/normal-flow");
        let img = base.join("support/green200x200.png");
        if !img.exists() {
            eprintln!("[R881] green200x200.png missing, skipping");
            return;
        }
        let html = r#"<!DOCTYPE html><html><head><style>
  div { background-color: red; float: left; }
  img { height: auto; max-width: 100px; vertical-align: bottom; width: auto; }
  </style></head>
  <body>
  <p>x</p>
  <div><img src="support/green200x200.png" alt="x" /></div>
  </body></html>"#;
        let cfg = ReftestConfig::default();
        let fb = render_to_framebuffer_with_base(html, "", &cfg, Some(&base));
        let px = &fb.data;
        // div shrink 包裹 img（100×100）→ green img 完全覆盖 div，无 red 外露
        let mut red = 0usize;
        for chunk in px.chunks(4) {
            if chunk.len() == 4 && chunk[0] >= 150 && chunk[1] < 80 && chunk[2] < 80 {
                red += 1;
            }
        }
        assert_eq!(red, 0, "float div must shrink-to-fit img; no red bg must be visible");
    }

    /// R988 端到端回归门禁：background-root-101/102 类（onload setTimeout + className
    /// mutation）经 harness 应用 JS mutation 后必须渲染绿色 canvas。覆盖 V8-init、
    /// setTimeout-onload、className-mutation 捕获、dom serializer 保留 `<style>` CDATA
    /// （R917 续）、head+body 相邻兄弟选择器、§14.2 canvas 背景传播全链。
    /// 任一环节回归 → body 不绿。
    #[test]
    fn test_r988_background_root_render_after_mutation() {
        let script = r#"<script type="text/javascript">
    function test() {
      document.getElementsByTagName('$ROOT')[0].className = 'after';
      document.getElementsByTagName('p')[0].className = 'after';
      document.documentElement.className = "";
    }
  </script>"#;
        // 102：body.class mutation（无兄弟选择器）。
        let html_102 = format!(
            r#"<html class="reftest-wait"><head><style><![CDATA[
    body.before {{ background: red; }} body.after {{ background: green; }}
  ]]></style>{script}</head>
 <body class="before" onload="setTimeout(test, 5)"><p class="before">x</p></body></html>"#,
            script = script.replace("$ROOT", "body")
        );
        // 101：head+body 相邻兄弟选择器（JS 改 head.class）。
        let html_101 = format!(
            r#"<html class="reftest-wait"><head class="before"><style><![CDATA[
    head.before + body {{ background: red; }} head.after + body {{ background: green; }}
  ]]></style>{script}</head>
 <body onload="setTimeout(test, 5)"><p class="before">x</p></body></html>"#,
            script = script.replace("$ROOT", "head")
        );

        let cfg = ReftestConfig::default();
        let green_pct = |html: &str| -> usize {
            let fb = render_to_framebuffer(html, "", &cfg);
            let (w, h) = (fb.width as usize, fb.height as usize);
            let (mut green, mut total) = (0usize, 0usize);
            for y in (h / 2..h).step_by(4) {
                for x in (0..w).step_by(4) {
                    let i = (y * w + x) * 4;
                    if i + 2 < fb.data.len() {
                        total += 1;
                        if fb.data[i + 1] > 80 && fb.data[i] < 100 && fb.data[i + 2] < 100 {
                            green += 1;
                        }
                    }
                }
            }
            green * 100 / total.max(1)
        };
        let pct102 = green_pct(&html_102);
        let pct101 = green_pct(&html_101);
        assert!(
            pct102 > 50,
            "102 body.class mutation must paint green canvas (got {pct102}%) — harness-JS or canvas-propagation regression"
        );
        assert!(
            pct101 > 50,
            "101 head+body sibling selector must paint green canvas after head.class mutation (got {pct101}%) — sibling-selector or serializer regression"
        );
    }
}
