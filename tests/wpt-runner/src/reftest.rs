//! Reftest Harness — 渲染测试 HTML 和参考 HTML，比较像素输出。
//!
//! 实现 WPT 风格的 `rel=match` / `rel=mismatch` 比较逻辑：
//! - match：两个页面的像素应几乎相同（允许模糊阈值）
//! - mismatch：两个页面的像素应有显著差异
//!
//! 支持分类容差（布局类 vs 文字类）和 per-test WPT fuzzy 注解覆盖。
//! 支持 CPU 软件渲染和 GPU 无头渲染两种模式。

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey, decode_data_uri};
use zero_render_foundation::rendering_thread::render_threading_enabled_for_tests;
use zero_render_foundation::surface::FrameBuffer;

use crate::manifest::FuzzyMeta;
use crate::runner_text_metrics;

mod reftest_compare;
mod reftest_fonts;
mod reftest_scripts;
mod struct_check;

pub use struct_check::*;

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
use zero_render_foundation::font::loader::FontLoader;

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
    /// 亚像素级差异像素数（通道差恰好为 1，通常来自 f32 坐标漂移/AA 抖动）。
    /// 诊断维度，不参与通过判定（D2，见 f32-layout-precision-audit）。
    pub subpixel_diff_pixels: usize,
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
    /// 渲染媒体类型（DC-12 @media print/screen 级联过滤；R1991）。默认 `Screen` = 零行为变更。
    pub media_type: zero_css_parser::media_query::MediaType,
    /// wpt-data 根目录：用于解析以 `/` 开头的 WPT 绝对路径资源
    /// （如 `/common/reftest-wait.js`——WPT URL 语义，非文件系统绝对路径；
    /// R546 谱系已修 ref 路径，2026-08-07 补 external script 路径）。
    pub wpt_root: Option<PathBuf>,
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
            wpt_root: None,
            fuzzy_override: None,
            min_mismatch_ratio: 0.005,
            media_type: zero_css_parser::media_query::MediaType::Screen,
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
    let _case_t0 = std::time::Instant::now();
    // 渲染测试页面
    let test_fb = render_to_framebuffer_with_base(&case.test_html, &case.css, config, base_dir);
    let _case_t1 = std::time::Instant::now();
    // 渲染参考页面（图片相对参考文件目录解析，缺失时回落到测试目录）
    let ref_base = case.ref_base_dir.as_deref().or(base_dir);
    let ref_fb = render_to_framebuffer_with_base(&case.ref_html, &case.css, config, ref_base);
    let _case_t2 = std::time::Instant::now();
    if std::env::var("ZW_CASE_STAGES").is_ok() {
        eprintln!(
            "[case-stages] {} test={:.1}ms ref={:.1}ms",
            case.id,
            _case_t1.duration_since(_case_t0).as_secs_f64() * 1000.0,
            _case_t2.duration_since(_case_t1).as_secs_f64() * 1000.0
        );
    }

    // 尺寸必须一致
    if test_fb.width != ref_fb.width || test_fb.height != ref_fb.height {
        return ReftestResult {
            id: case.id.clone(),
            passed: false,
            diff_pixels: 0,
            total_pixels: 0,
            diff_ratio: 0.0,
            max_channel_diff: 0,
            subpixel_diff_pixels: 0,
            test_near_solid: false,
            message: format!(
                "Size mismatch: test={}x{} ref={}x{}",
                test_fb.width, test_fb.height, ref_fb.width, ref_fb.height
            ),
        };
    }

    let total_pixels = (test_fb.width as usize) * (test_fb.height as usize);
    let eff_channel_diff = config.effective_max_channel_diff();
    let (diff_pixels, max_channel_diff, subpixel_diff_pixels) =
        compare_pixels_labeled(&test_fb, &ref_fb, eff_channel_diff, &case.id);
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
        subpixel_diff_pixels,
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
            subpixel_diff_pixels: 0,
            test_near_solid: false,
            message: format!(
                "Size mismatch: test={}x{} ref={}x{}",
                test_fb.width, test_fb.height, ref_fb.width, ref_fb.height
            ),
        };
    }

    let total_pixels = (test_fb.width as usize) * (test_fb.height as usize);
    let eff_channel_diff = config.effective_max_channel_diff();
    let (diff_pixels, max_channel_diff, subpixel_diff_pixels) =
        compare_pixels_labeled(&test_fb, &ref_fb, eff_channel_diff, &case.id);
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
        subpixel_diff_pixels,
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

/// 递归算 LayoutBox 树的最大 abs bottom（镜像 pipeline.rs `layout_extent_y`）。
/// 供 R2005 Print tall-framebuffer（`ZW_PRINT_TALL_FB`）算 page-aligned 渲染高度。
fn layout_extent_max_y(b: &zero_layout_engine::types::LayoutBox, offset_y: f32) -> f32 {
    let mut max_y = offset_y + b.y + b.height;
    for child in &b.children {
        max_y = max_y.max(layout_extent_max_y(child, offset_y + b.y));
    }
    max_y
}

/// 同 [`render_to_framebuffer_with_base`]，但额外返回布局树根（DC-13 结构检查用）。
///
/// `RenderResult.layout.root` 在 `render_full_scene`（仅借 `result.primitives()`）后移出，
/// 供 product-smoke 结构自动检查（如兄弟盒重叠检测）遍历。Framebuffer 渲染不受影响。
///
/// `render_to_framebuffer_with_layout_with_base` = 3-tuple（root + html，旧调用方）；
/// `render_to_framebuffer_with_layout_and_paint_skip_with_base` = 4-tuple（额外 paint_skip
/// 集，供 struct-check 排除 orphan 假阳性，R2198）。两者皆薄 wrapper 委托 `render_with_layout_inner`。
pub fn render_to_framebuffer_with_layout_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> (FrameBuffer, zero_layout_engine::types::LayoutBox, String) {
    let (fb, root, _paint_skip, html, _timings) = render_with_layout_inner(html, css, config, base_dir, false);
    (fb, root, html)
}

/// 同上，额外返回 `LayoutResult.paint_skip_node_ids`（R2197 Phase A slice 3 orphan 元素集），
/// 供 product-smoke struct-check 的 `check_sibling_overlaps` 排除 orphan 假阳性（orphan 是
/// hit-test proxy，paint-skip = 非视觉盒，不计视觉重叠检测）。
pub fn render_to_framebuffer_with_layout_and_paint_skip_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> (
    FrameBuffer,
    zero_layout_engine::types::LayoutBox,
    std::collections::HashSet<zero_dom::NodeId>,
    String,
) {
    let (fb, root, paint_skip, html, _timings) = render_with_layout_inner(html, css, config, base_dir, false);
    (fb, root, paint_skip, html)
}

/// 渲染 HTML 到 FrameBuffer 并返回管线阶段耗时（perf-gate 页面级基准测量用）。
///
/// 与 [`render_to_framebuffer_with_base`] 同一 engine-direct 路径（`render_with_layout_inner`），
/// 额外返回 `RenderResult.timings`（parse/style/layout/paint/total_ms，均为 `render_html`
/// 阶段耗时，见 `crates/engine/src/pipeline/mod.rs` `PipelineTimings`）。供 zero-wpt-runner
/// `perf` 子命令做页面级首屏基准（性能门禁体系，见 docs/specs/performance-and-resource-budget.md）。
pub fn render_to_framebuffer_with_timings(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> (FrameBuffer, zero_engine::PipelineTimings) {
    let (fb, _root, _paint_skip, _html, timings) = render_with_layout_inner(html, css, config, base_dir, false);
    (fb, timings)
}

/// 进程级缓存的「默认字体」FontLoader（系统 + CJK + Ahem + 回退链）。
///
/// 默认字体集合跨所有 reftest case 完全一致，fontdue `from_bytes` 解析（尤以 ~16MB
/// NotoSansCJK）是单案串行成本的大头（见 `render_with_layout_inner` 注释）。
/// `build_font_resolver` / `build_line_metric_map` 取 `&self`（只读），故无 `@font-face`
/// 的 case 可经此单例零成本复用；全进程只 build 一次，rayon 多线程并发只读安全。
static BASE_FONT_LOADER: std::sync::OnceLock<zero_render_foundation::font::loader::FontLoader> =
    std::sync::OnceLock::new();

/// @font-face 组合缓存（2026-08-07 优化）：声明需加载的 @font-face 的 case 此前每次
/// create_font_loader 全量重载系统 + 19MB CJK 字体（~480ms/case × 2 render，Ahem 系多数
/// 慢 case 的根因）。按「按 base_dir 解析后的 @font-face src 路径列表」缓存
/// Arc<FontLoader>——键取 load_font_faces_into 的真实输入，相同组合只创建一次，
/// 命中后 Arc clone（引用计数）近乎零成本。绝对 src（WPT 通用 `/fonts/*`）解析结果
/// 与 base_dir 无关 → test 与 reference/ 子目录的 ref 渲染共享同一键；相对 src 解析为
/// 各自目录路径 → 正确区分。全字符串键，无哈希碰撞风险。容量上限 32，超限清空重建。
static FRESH_LOADER_CACHE: std::sync::OnceLock<
    std::sync::Mutex<
        std::collections::HashMap<String, std::sync::Arc<zero_render_foundation::font::loader::FontLoader>>,
    >,
> = std::sync::OnceLock::new();

const FRESH_LOADER_CACHE_MAX: usize = 32;

fn render_with_layout_inner(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
    use_gpu: bool,
) -> (
    FrameBuffer,
    zero_layout_engine::types::LayoutBox,
    std::collections::HashSet<zero_dom::NodeId>,
    String,
    zero_engine::PipelineTimings,
) {
    let _zw_t0 = std::time::Instant::now();
    // R3268 canvas 显示链路：registry 在 script 执行与渲染间共享（getContext 写入的
    // canvas 像素经 painter 桥接为图元 + canvas_images 注入 image_cache）。
    let canvas_registry: std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(zero_engine::js_dom_bridge::CanvasRegistry::new()));
    // 执行页面 <script>（含 DOM 变更），把 JS 后的最终 HTML 用于后续渲染。
    let mutated_html = apply_scripted_dom_mutations(html, base_dir, config.wpt_root.as_deref(), &canvas_registry);
    let _zw_t1 = std::time::Instant::now();
    let media_ctx = zero_css_parser::media_query::MediaContext::with_type(
        config.viewport_width as f64,
        config.viewport_height as f64,
        config.media_type,
    );
    // R2426：展开 inline `<style>` 内的 @import（harness sync 路径补全——collect_stylesheets
    // 不抓 @import；passed/linked CSS 的 @import 在 merge_page_css 内展开）。
    let styled_html = match base_dir {
        Some(base) => crate::reftest::resources::expand_style_imports(&mutated_html, base, &media_ctx),
        None => mutated_html,
    };
    let html: &str = &styled_html;

    let _zw_t2 = std::time::Instant::now();
    // 先构建图像缓存，提取固有尺寸供 paint 阶段使用
    let mut image_cache = build_image_cache(html, base_dir);
    let (image_sizes, image_ratios, image_no_ratio) = extract_image_metrics(&mut image_cache, html);

    let combined_css = merge_page_css(html, css, base_dir, Some(&media_ctx));
    let _zw_t3 = std::time::Instant::now();

    let _zw_t4 = std::time::Instant::now();
    let mut pipeline = RenderPipeline::new(config.viewport_width as f32, config.viewport_height as f32);
    pipeline.set_canvas_registry(Some(canvas_registry.clone()));
    let _zw_t4a = std::time::Instant::now();
    pipeline.set_skip_indicators(true);
    // R1991：@media print/screen 级联按渲染媒体类型过滤（DC-12）。默认 Screen = 零变更；
    // `--media print` 经 config.media_type 传入使 @media print 生效（量真实 WPT yield）。
    pipeline.set_media_type(config.media_type);
    pipeline.set_image_sizes(image_sizes);
    pipeline.set_image_ratios(image_ratios);
    pipeline.set_image_no_ratio(image_no_ratio);

    // 字体查找表（在 render_html 之前，供 Painter 解析 CSS font-family）。
    // 扫描外链/传入 CSS + 内联 <style> 的 @font-face（常声明在内联 <style>）。
    let _zw_t4b = std::time::Instant::now();
    let font_scan_css = format!("{combined_css}\n{}", extract_inline_style_css(html));
    let faces = extract_font_faces(&font_scan_css);
    let has_font_face = !faces.is_empty();
    // 仅 Ahem 的 @font-face（harness 按 family 名合成方块，load_font_faces_into 跳过，
    // 且 create_font_loader 已加载 Ahem.ttf）→ fresh loader 与 BASE 内容等价 →
    // 直接复用 BASE_FONT_LOADER，免 ~480ms 创建成本。
    let needs_custom_faces = faces
        .iter()
        .any(|(family, _, _, _, _, _, _)| !family.eq_ignore_ascii_case("Ahem"));
    let _zw_t4c = std::time::Instant::now();
    // 杠杆4：默认字体（系统 + CJK + Ahem + 回退链）跨 case 完全一致，而 fontdue
    // from_bytes 解析（尤以 ~16MB NotoSansCJK）占单案 ~85% 串行成本（实测每 render
    // ~0.4s，每 case 2 render ≈ 0.8s，而单案总成本仅 ~0.9s）。build_font_resolver /
    // build_line_metric_map 均取 &self，故无 @font-face 的多数 case 复用进程级缓存的
    // base loader（BASE_FONT_LOADER），零 re-parse；仅声明需加载自定义字体的 case
    //（字体测试）走 fresh owned loader（含 re-parse + 自定义字体）。
    // 杠杆4 + @font-face 缓存：无 @font-face / 仅 Ahem 复用 BASE_FONT_LOADER（进程级）；
    // 声明需加载 @font-face 的 case 按「src 解析后的路径列表」缓存 Arc<FontLoader>——
    // 键取 load_font_faces_into 的真实输入，页面间仅内联样式不同的 case 共享同一键
    //（WPT 通用 ahem.css 系、absolute-src 的 test/ref 对），只 create 一次。
    let fresh_arc: Option<std::sync::Arc<zero_render_foundation::font::loader::FontLoader>> =
        if has_font_face && needs_custom_faces {
            let cache = FRESH_LOADER_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
            // 全字符串键：@font-face src 按 base_dir 解析后的路径列表（Debug 序列化）。
            // 无哈希碰撞风险；不同页面即使其余 CSS 不同，只要 @font-face 声明解析结果
            // 相同即共享同一 loader（绝对 src 跨目录共享——reference/ 子目录的 ref
            // 渲染与 test 渲染同键；相对 src 解析为各自目录路径，正确区分）。
            let key = format!(
                "{:?}",
                faces
                    .iter()
                    .map(
                        |(family, sources, weight, is_italic, stretch, feature_settings, unicode_ranges)| (
                            family,
                            sources
                                .iter()
                                .map(|src| resolve_font_src(src, base_dir))
                                .collect::<Vec<_>>(),
                            weight,
                            is_italic,
                            stretch.map(f32::to_bits),
                            feature_settings,
                            unicode_ranges,
                        )
                    )
                    .collect::<Vec<_>>()
            );
            let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(fl) = guard.get(&key) {
                Some(fl.clone())
            } else {
                if guard.len() >= FRESH_LOADER_CACHE_MAX {
                    guard.clear();
                }
                // 从 BASE 深拷贝而非 create_font_loader 全量重解析：19MB CJK fontdue
                // 解析（~0.5s）只做一次，duplicate 复用已解析结果，每键成本降一个量级。
                let mut fl = BASE_FONT_LOADER.get_or_init(create_font_loader).duplicate();
                load_font_faces_into(&mut fl, base_dir, &font_scan_css);
                let arc = std::sync::Arc::new(fl);
                guard.insert(key, arc.clone());
                Some(arc)
            }
        } else {
            None
        };
    let font_loader: &FontLoader = fresh_arc
        .as_deref()
        .unwrap_or_else(|| BASE_FONT_LOADER.get_or_init(create_font_loader));
    let _zw_t4d = std::time::Instant::now();
    pipeline.set_font_resolver(font_loader.build_font_resolver());
    pipeline.set_font_metric_map(font_loader.build_line_metric_map());

    let _zw_t5 = std::time::Instant::now();
    let result = runner_text_metrics::with_measure_ctx(font_loader, 0u32, || pipeline.render_html(html, &combined_css));
    // R3268：canvas 像素注入 ImageCache（图元 image_key = ctx_id）
    for (ctx_id, cw, ch, rgba) in &result.canvas_images {
        if let Ok(data) = ImageData::from_rgba(rgba.clone(), *cw, *ch) {
            image_cache.insert_with_key(ImageKey::new(*ctx_id), data);
        }
    }
    let _zw_t6 = std::time::Instant::now();

    // PERF 诊断（env ZW_RENDER_STAGES=1）：打印各阶段耗时（parse/style/layout/paint/total）
    if std::env::var("ZW_RENDER_STAGES").is_ok() {
        eprintln!(
            "[stages] parse={:.1}ms style={:.1}ms layout={:.1}ms paint={:.1}ms total={:.1}ms",
            result.timings.parse_ms,
            result.timings.style_ms,
            result.timings.layout_ms,
            result.timings.paint_ms,
            result.timings.total_ms
        );
        eprintln!(
            "[stages] script={:.1}ms expand+image_cache={:.1}ms merge_css={:.1}ms pipeline_new={:.1}ms set_*={:.1}ms font_scan={:.1}ms font_loader={:.1}ms resolver={:.1}ms render={:.1}ms",
            _zw_t1.duration_since(_zw_t0).as_secs_f64() * 1000.0,
            _zw_t2.duration_since(_zw_t1).as_secs_f64() * 1000.0,
            _zw_t4.duration_since(_zw_t3).as_secs_f64() * 1000.0,
            _zw_t4a.duration_since(_zw_t4).as_secs_f64() * 1000.0,
            _zw_t4b.duration_since(_zw_t4a).as_secs_f64() * 1000.0,
            _zw_t4c.duration_since(_zw_t4b).as_secs_f64() * 1000.0,
            _zw_t4d.duration_since(_zw_t4c).as_secs_f64() * 1000.0,
            _zw_t5.duration_since(_zw_t4d).as_secs_f64() * 1000.0,
            _zw_t6.duration_since(_zw_t5).as_secs_f64() * 1000.0
        );
    }

    // DEBUG: dump layout box tree geometry (absolute y / margin-top / padding-top)
    // 用途：诊断产品 smoke 垂直偏移（如 welcome 36px 顶部偏移）。
    if std::env::var("LAYOUT_DUMP").is_ok() {
        dump_layout_tree(&result.layout.root, html);
    }

    // DEBUG: dump primitives for diagnostic
    if std::env::var("REFTEST_DEBUG").is_ok() {
        eprintln!("=== Primitives for {} ===", html.lines().take(1).next().unwrap_or(""));
        eprintln!("  fills: {}", result.primitives().fills.len());
        eprintln!("  images: {}", result.primitives().images.len());
        eprintln!("  rounded_rects: {}", result.primitives().rounded_rects.len());
        eprintln!("  glyphs: {}", result.primitives().glyphs.len());
        eprintln!("  gradients: {}", result.primitives().gradients.len());
        eprintln!("  strokes: {}", result.primitives().strokes.len());
        for (i, fill) in result.primitives().fills.iter().enumerate().take(20) {
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
        for (i, img) in result.primitives().images.iter().enumerate().take(10) {
            eprintln!(
                "  image[{}]: ({:.1},{:.1},{:.1},{:.1}) key={:?}",
                i, img.rect.origin.x, img.rect.origin.y, img.rect.size.width, img.rect.size.height, img.image_key
            );
        }
    }

    let mut glyph_cache = GlyphCache::new(1024);

    // R2005 P5 bounded：Print 模式 + env `ZW_PRINT_TALL_FB=1` 时按 page-aligned doc extent
    // 渲染 tall framebuffer（非 viewport 600），让分页可视化 + 产 evidence PNG（验证 R1999-R2001
    // 分页的渲染像素，此前仅 e2e layout-extent 程序验证）。default-off → 现有 print 测试
    //（如 test_media_type_print_applies_print_rules）用 viewport 高度，零影响。
    let fb_height = if matches!(config.media_type, zero_css_parser::media_query::MediaType::Print)
        && std::env::var("ZW_PRINT_TALL_FB").as_deref() == Ok("1")
    {
        let extent = layout_extent_max_y(&result.layout.root, 0.0);
        let page_h = zero_layout_engine::print_pagination::PRINT_PAGE_HEIGHT_A4;
        (((extent / page_h).ceil() * page_h).max(config.viewport_height as f32)) as u32
    } else {
        config.viewport_height
    };

    // 使用已构建的图像缓存（包含固有尺寸信息）
    // R3270（#5）：--gpu 走真 GpuRenderer::new_headless 渲染（取代 CPU stub）。
    // GPU 场景含未实现特性时返回 false → 回退 CPU（P0-1 语义，慢但对）。
    // S2 可选线程化（#3 渲染线程化 RFC）：ZW_RENDER_THREAD=1 时光栅化在
    // 独立线程执行（thread::scope），结果与单线程逐像素一致——默认关，
    // 测试确定性优先；env 用于验证线程路径正确性。
    let fb = if use_gpu {
        render_full_scene_gpu_reftest(
            config.viewport_width,
            fb_height,
            config.scale_factor,
            &result.display_list.primitives,
            font_loader,
            &mut glyph_cache,
            &mut image_cache,
        )
    } else if render_threading_enabled_for_tests() {
        std::thread::scope(|s| {
            s.spawn(|| {
                render_full_scene(
                    config.viewport_width,
                    fb_height,
                    config.scale_factor,
                    &result.display_list.primitives,
                    font_loader,
                    &mut glyph_cache,
                    Some(&mut image_cache),
                    &[],
                    &[],
                    &[],
                    &[],
                )
            })
            .join()
            .expect("渲染线程 panic")
        })
    } else {
        render_full_scene(
            config.viewport_width,
            fb_height,
            config.scale_factor,
            &result.display_list.primitives,
            font_loader,
            &mut glyph_cache,
            Some(&mut image_cache),
            &[],
            &[],
            &[],
            &[],
        )
    };
    // render_full_scene 仅借 result.display_list.primitives（借用已结束）；移出 layout 根供结构检查。
    // 一并返回 mutated_html（render_html 实际解析的 HTML，经 script DOM 变更后可能与调用方传入
    // 的原 html 不同）——DC-13 结构检查须用它建 labels，否则 layout 树 node_id 与 collect_dom_labels
    //（解析原 html）不匹配 → 真元素误标 "(anon)"（R1499：morning disqus loadDisqus() appendChild
    // 致 mutated_html 与原 html 不同，p/table 误标 anon）。
    (
        fb,
        Arc::try_unwrap(result.layout.root).unwrap_or_else(|arc| (*arc).clone()),
        result.layout.paint_skip_node_ids,
        styled_html,
        result.timings,
    )
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
    let canvas_registry: std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(zero_engine::js_dom_bridge::CanvasRegistry::new()));
    let mutated_html = apply_scripted_dom_mutations(html, base_dir, config.wpt_root.as_deref(), &canvas_registry);
    let media_ctx = zero_css_parser::media_query::MediaContext::with_type(
        config.viewport_width as f64,
        config.viewport_height as f64,
        config.media_type,
    );
    let styled_html = match base_dir {
        Some(base) => crate::reftest::resources::expand_style_imports(&mutated_html, base, &media_ctx),
        None => mutated_html,
    };
    let html: &str = &styled_html;

    let mut image_cache = build_image_cache(html, base_dir);
    let (image_sizes, image_ratios, image_no_ratio) = extract_image_metrics(&mut image_cache, html);
    let combined_css = merge_page_css(html, css, base_dir, Some(&media_ctx));

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
fn merge_page_css(
    html: &str,
    css: &str,
    base_dir: Option<&Path>,
    media_ctx: Option<&zero_css_parser::media_query::MediaContext>,
) -> String {
    let linked_css = load_linked_stylesheets(html, base_dir);
    let combined = if css.is_empty() {
        linked_css
    } else if linked_css.is_empty() {
        css.to_string()
    } else {
        format!("{linked_css}\n{css}")
    };
    // R2426：展开 passed/linked CSS 内的 @import（harness sync 路径补全——collect_stylesheets
    // 不抓 @import；inline `<style>` 的 @import 由 render 函数的 expand_style_imports 处理）。
    match (base_dir, media_ctx) {
        (Some(base), Some(ctx)) => {
            let mut chain = std::collections::HashSet::new();
            crate::reftest::resources::expand_at_imports(&combined, base, ctx, &mut chain)
        }
        _ => combined,
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

pub fn dump_layout_tree(root: &zero_layout_engine::types::LayoutBox, html: &str) {
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
///
/// R3270（#5）：真 `GpuRenderer::new_headless` 渲染（取代 CPU stub）。全图元 +
/// ImageCache + glyph atlas 已由 `render_full_scene_gpu` 支持；GPU 场景含未实现
/// 特性（clip/blend/模糊阴影/滤镜）时返回 false → 回退 CPU（P0-1 语义，慢但对）。
/// device 创建受 `GPU_CREATE_MUTEX` 序列化（并发 job 安全，创建后各 device 独立渲染）。
pub fn render_to_framebuffer_gpu_with_base(
    html: &str,
    css: &str,
    config: &ReftestConfig,
    base_dir: Option<&Path>,
) -> FrameBuffer {
    render_with_layout_inner(html, css, config, base_dir, true).0
}

/// GPU 无头渲染（#5）：render_full_scene_gpu → read_pixels；不支持特性回退 CPU。
#[allow(clippy::too_many_arguments)]
fn render_full_scene_gpu_reftest(
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: &zero_render_foundation::primitive::RenderPrimitives,
    font_loader: &FontLoader,
    glyph_cache: &mut GlyphCache,
    image_cache: &mut ImageCache,
) -> FrameBuffer {
    let mut renderer = match zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(width, height) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[reftest] GPU 渲染器初始化失败，回退 CPU: {e}");
            return render_full_scene(
                width,
                height,
                scale_factor,
                primitives,
                font_loader,
                glyph_cache,
                Some(image_cache),
                &[],
                &[],
                &[],
                &[],
            );
        }
    };
    let rendered = renderer.render_full_scene_gpu(
        primitives,
        font_loader,
        glyph_cache,
        Some(image_cache),
        &[],
        &[],
        &[],
        &[],
        scale_factor,
    );
    if !rendered {
        // GPU 未实现特性 → CPU 回退（P0-1）
        return render_full_scene(
            width,
            height,
            scale_factor,
            primitives,
            font_loader,
            glyph_cache,
            Some(image_cache),
            &[],
            &[],
            &[],
            &[],
        );
    }
    let pixels = renderer.read_pixels().expect("GPU read_pixels");
    let mut fb = zero_render_foundation::surface::FrameBuffer::new(width, height);
    fb.data.copy_from_slice(&pixels);
    fb
}

#[cfg(all(test, feature = "v8"))]
mod tests;
