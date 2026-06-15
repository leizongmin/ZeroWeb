//! Reftest Harness — 渲染测试 HTML 和参考 HTML，比较像素输出。
//!
//! 实现 WPT 风格的 `rel=match` / `rel=mismatch` 比较逻辑：
//! - match：两个页面的像素应几乎相同（允许模糊阈值）
//! - mismatch：两个页面的像素应有显著差异
//!
//! 支持分类容差（布局类 vs 文字类）和 per-test WPT fuzzy 注解覆盖。
//! 支持 CPU 软件渲染和 GPU 无头渲染两种模式。

#![allow(dead_code)]

use std::char;
use std::path::Path;

use zero_engine::RenderPipeline;
use zero_engine::paint::simple_hash;
use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};
use zero_render_foundation::surface::FrameBuffer;

use crate::manifest::FuzzyMeta;

/// 将 FrameBuffer 保存为 PNG 文件（用于失败诊断）。
fn save_fb_as_png(fb: &FrameBuffer, path: &Path) {
    use std::io::BufWriter;
    let Ok(file) = std::fs::File::create(path) else {
        return;
    };
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, fb.width, fb.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let Ok(mut writer) = encoder.write_header() else {
        return;
    };
    // FrameBuffer data is RGBA
    let _ = writer.write_image_data(&fb.data);
    let _ = writer.finish();
}

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
    /// 根据分类创建配置（使用分类默认容差）。
    pub fn for_category(category: ReftestCategory) -> Self {
        Self {
            max_diff_ratio: category.default_max_diff_ratio(),
            max_channel_diff: category.default_max_channel_diff(),
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
}

/// 运行单个 reftest 用例。
pub fn run_reftest(case: &ReftestCase, config: &ReftestConfig) -> ReftestResult {
    run_reftest_with_base(case, config, None)
}

/// 运行单个 reftest 用例（支持基于 base_dir 的图片加载）。
pub fn run_reftest_with_base(case: &ReftestCase, config: &ReftestConfig, base_dir: Option<&Path>) -> ReftestResult {
    // 渲染测试页面
    let test_fb = render_to_framebuffer_with_base(&case.test_html, &case.css, config, base_dir);
    // 渲染参考页面
    let ref_fb = render_to_framebuffer_with_base(&case.ref_html, &case.css, config, base_dir);

    // 尺寸必须一致
    if test_fb.width != ref_fb.width || test_fb.height != ref_fb.height {
        return ReftestResult {
            id: case.id.clone(),
            passed: false,
            diff_pixels: 0,
            total_pixels: 0,
            diff_ratio: 0.0,
            max_channel_diff: 0,
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
    }
}

/// 使用 GPU 无头渲染运行 reftest（回退到 CPU 如果 GPU 不可用）。
pub fn run_reftest_gpu(case: &ReftestCase, config: &ReftestConfig) -> ReftestResult {
    run_reftest_gpu_with_base(case, config, None)
}

/// 使用 GPU 无头渲染运行 reftest（支持基于 base_dir 的图片加载）。
pub fn run_reftest_gpu_with_base(case: &ReftestCase, config: &ReftestConfig, base_dir: Option<&Path>) -> ReftestResult {
    // 渲染测试页面和参考页面
    let test_fb = render_to_framebuffer_gpu_with_base(&case.test_html, &case.css, config, base_dir);
    let ref_fb = render_to_framebuffer_gpu_with_base(&case.ref_html, &case.css, config, base_dir);

    // 尺寸必须一致
    if test_fb.width != ref_fb.width || test_fb.height != ref_fb.height {
        return ReftestResult {
            id: case.id.clone(),
            passed: false,
            diff_pixels: 0,
            total_pixels: 0,
            diff_ratio: 0.0,
            max_channel_diff: 0,
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
    }
}

/// 从 HTML 中提取所有 `<img src="...">` 的 URL。
fn extract_img_srcs(html: &str) -> Vec<String> {
    let mut srcs = Vec::new();
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("<img") {
        let tag_start = pos + idx;
        // 找到标签真正的结束位置（跳过引号内的 >）
        let Some(tag_end) = find_tag_end(&html[tag_start..]) else {
            break;
        };
        let tag = &html[tag_start..tag_start + tag_end];
        // 在标签内查找 src 属性
        if let Some(src_start) = tag.find("src=\"").or_else(|| tag.find("src='")) {
            let quote = &tag[src_start + 4..src_start + 5];
            let value_start = src_start + 5;
            if let Some(value_end) = tag[value_start..].find(quote) {
                let src_value = &tag[value_start..value_start + value_end];
                if !src_value.is_empty() {
                    srcs.push(src_value.to_string());
                }
            }
        }
        pos = tag_start + tag_end + 1;
    }
    srcs
}

/// 找到 HTML 标签的真正结束位置（> 在引号外）。
/// 返回 > 字符的偏移量（相对于起始位置）。
fn find_tag_end(html: &str) -> Option<usize> {
    let mut in_quote: Option<char> = None;
    for (i, c) in html.char_indices() {
        match in_quote {
            Some(q) if q == c => in_quote = None, // 关闭引号
            Some(_) => {}                         // 引号内，忽略
            None => match c {
                '"' | '\'' => in_quote = Some(c), // 进入引号
                '>' => return Some(i),            // 引号外的 > 是标签结束
                _ => {}
            },
        }
    }
    None
}

/// 已知 WPT 支持图片 URL → 颜色映射。
///
/// WPT 参考文件大量使用小纯色图片（如 1x1-green.png, swatch-blue.png）来
/// 创建参考视觉输出。这里将常见 URL 的 hash 值映射到对应颜色，
/// 直接渲染为 fill rectangle，避免图片解码/定位问题。
fn get_support_image_color(key: &ImageKey) -> Option<[u8; 4]> {
    // 常见 WPT 支持图片 URL 列表及其颜色
    const KNOWN_IMAGES: &[(&str, [u8; 4])] = &[
        // 1x1 images
        ("support/1x1-green.png", [0, 128, 0, 255]),
        ("support/1x1-white.png", [255, 255, 255, 255]),
        ("support/1x1-navy.png", [0, 0, 128, 255]),
        ("support/1x1-red.png", [255, 0, 0, 255]),
        // swatch images (solid color)
        ("support/swatch-blue.png", [0, 0, 255, 255]),
        ("support/swatch-green.png", [0, 128, 0, 255]),
        ("support/swatch-orange.png", [255, 165, 0, 255]),
        ("support/swatch-red.png", [255, 0, 0, 255]),
        ("support/swatch-lime.png", [0, 255, 0, 255]),
        ("support/swatch-yellow.png", [255, 255, 0, 255]),
        // 15x15 solid color images
        ("support/black15x15.png", [0, 0, 0, 255]),
        ("support/blue15x15.png", [0, 0, 255, 255]),
        ("support/green15x15.png", [0, 128, 0, 255]),
        ("support/red15x15.png", [255, 0, 0, 255]),
        ("support/orange15x15.png", [255, 165, 0, 255]),
        // 96x96 solid color images
        ("support/blue96x96.png", [0, 0, 255, 255]),
        ("support/black96x96.png", [0, 0, 0, 255]),
        // 60x60 images
        ("support/60x60-green.png", [0, 128, 0, 255]),
        ("support/60x60-red.png", [255, 0, 0, 255]),
        ("support/60x60-blue.png", [0, 0, 255, 255]),
        // 100x100 images
        ("support/100x100-red.png", [255, 0, 0, 255]),
        ("support/100x100-green.png", [0, 128, 0, 255]),
        ("support/100x100-blue.png", [0, 0, 255, 255]),
        // Other common support images
        ("support/aqua_color.png", [0, 255, 255, 255]),
    ];

    for (url, color) in KNOWN_IMAGES {
        let url_hash = ImageKey::new(simple_hash(url));
        if key == &url_hash {
            return Some(*color);
        }
    }
    None
}
fn extract_css_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("url(") {
        let url_start = pos + idx + 4;
        // 跳过空白和引号
        let rest = &html[url_start..];
        let trimmed = rest
            .trim_start_matches(' ')
            .trim_start_matches('\'')
            .trim_start_matches('"');
        let offset = rest.len() - trimmed.len();
        let actual_start = url_start + offset;

        if let Some(end_idx) = html[actual_start..].find(')') {
            let raw = html[actual_start..actual_start + end_idx].trim();
            let url = raw.trim_matches('\'').trim_matches('"').trim();
            if !url.is_empty() && !url.starts_with("data:") && !url.starts_with("http") {
                urls.push(url.to_string());
            }
            pos = actual_start + end_idx + 1;
        } else {
            break;
        }
    }
    urls
}

fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("<link") {
        let tag_start = pos + idx;
        let Some(tag_end) = find_tag_end(&html[tag_start..]) else {
            break;
        };
        let tag = &html[tag_start..tag_start + tag_end];
        let tag_lower = tag.to_ascii_lowercase();
        if !tag_lower.contains("rel=\"stylesheet\"")
            && !tag_lower.contains("rel='stylesheet'")
            && !tag_lower.contains("rel=\"alternate stylesheet\"")
            && !tag_lower.contains("rel='alternate stylesheet'")
        {
            pos = tag_start + tag_end + 1;
            continue;
        }

        if let Some(href_start) = tag.find("href=\"").or_else(|| tag.find("href='")) {
            let quote = &tag[href_start + 5..href_start + 6];
            let value_start = href_start + 6;
            if let Some(value_end) = tag[value_start..].find(quote) {
                let href_value = tag[value_start..value_start + value_end].trim();
                if !href_value.is_empty() {
                    hrefs.push(href_value.to_string());
                }
            }
        }

        pos = tag_start + tag_end + 1;
    }
    hrefs
}

fn load_linked_stylesheets(html: &str, base_dir: Option<&Path>) -> String {
    let Some(base) = base_dir else {
        return String::new();
    };

    let mut merged = String::new();
    for href in extract_stylesheet_hrefs(html) {
        if href.starts_with("data:") || href.starts_with("http://") || href.starts_with("https://") {
            continue;
        }

        let path = if href.starts_with('/') {
            Path::new("tests/wpt-runner/wpt-data").join(href.trim_start_matches('/'))
        } else {
            base.join(&href)
        };

        if let Ok(css) = std::fs::read_to_string(&path) {
            if !merged.is_empty() {
                merged.push('\n');
            }
            merged.push_str(&css);
        }
    }
    merged
}

/// 从基础目录加载图片文件并解码为 RGBA 数据，放入 ImageCache。
///
/// 对于每个 URL，用 `simple_hash(url)` 生成 ImageKey（与 paint 系统一致），
/// 然后尝试从 `base_dir` 解析相对路径并加载 PNG 文件。
fn build_image_cache(html: &str, base_dir: Option<&Path>) -> ImageCache {
    let mut cache = ImageCache::new(256, 64 * 1024 * 1024);

    let Some(base) = base_dir else {
        return cache;
    };

    // 收集所有需要加载的 URL
    let mut all_urls = extract_img_srcs(html);
    all_urls.extend(extract_css_urls(html));
    all_urls.sort_unstable();
    all_urls.dedup();

    for url in &all_urls {
        let key = ImageKey::new(simple_hash(url));

        // 优先处理 data URI（SVG 等）
        if url.starts_with("data:image/svg+xml")
            && let Some(data) = generate_svg_data_uri_image(url)
        {
            cache.insert_with_key(key, data);
            continue;
        }

        // 跳过非文件 URL（如 data: URI 无法从文件系统加载）
        if url.starts_with("data:") {
            continue;
        }

        let path = base.join(url);

        // 尝试加载 PNG 文件
        if let Ok(data) = load_png_file(&path) {
            cache.insert_with_key(key, data);
        }
    }

    cache
}

/// 把 png crate 解码（已 EXPAND|STRIP_16）的输出缓冲按其输出色型转换为 RGBA8。
///
/// EXPAND 不保证 RGBA：palette 无 tRNS / RGB 输入 → 输出 RGB（3 字节/像素），
/// grayscale → 1 字节/像素。本函数按 OutputInfo.color_type 统一补齐为 RGBA。
fn convert_png_buffer_to_rgba(raw: &[u8], color_type: png::ColorType, bit_depth: png::BitDepth) -> Vec<u8> {
    use png::ColorType::*;
    // STRIP_16 保证 ≤8bit；EXPAND 保证非 palette/indexed。剩余可能的 16-bit 输入
    //（如 Rgb16）经 STRIP_16 后变 8-bit。
    if bit_depth != png::BitDepth::Eight {
        // 理论上 STRIP_16 已处理；保留兜底以防异常输入。
        return raw.to_vec();
    }
    match color_type {
        Rgba => raw.to_vec(),
        Rgb => {
            let n = raw.len() / 3;
            let mut out = Vec::with_capacity(n * 4);
            for px in raw.chunks_exact(3) {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        Grayscale => {
            let mut out = Vec::with_capacity(raw.len() * 4);
            for &g in raw {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        GrayscaleAlpha => {
            let n = raw.len() / 2;
            let mut out = Vec::with_capacity(n * 4);
            for px in raw.chunks_exact(2) {
                out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
            out
        }
        _ => raw.to_vec(),
    }
}

/// 加载并解码 PNG 文件为 RGBA ImageData。
fn load_png_file(path: &Path) -> Result<ImageData, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("无法打开 {}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    // DC-14 anti-false-pass：正确加载任意 color type 的 PNG（palette/grayscale/RGB/RGBA）。
    // EXPAND|STRIP_16 把 palette→RGB(A)、grayscale→RGB(A)、低位深→8bit；但输出色型不一定是
    // RGBA（palette 无 tRNS / RGB 输入 → 输出 RGB=3 字节/像素）。须用 output_buffer_size 分配
    // 并按 next_frame 返回的 OutputInfo.color_type 转换为 RGBA，否则按 4 字节解释会错位
    // → alpha=0 退化透明（swatch-green.png 实测 [0,128,0,0]，图像类 reftest 假通过根因）。
    // 历史上此路径曾 env-gated（ZERO_PNG_EXPAND）默认关闭以保 436 baseline，但 DC-14 要求
    // 真实测量——正确的图像渲染是 anti-false-pass 的前提。启用后暴露 vrl-004/008 等真实
    // 布局差异（net -2），这些是须修复的真实失败而非应隐藏的假通过。
    // ZERO_PNG_EXPAND=0 逃生舱回退到旧的「按 RGBA 直读」路径（诊断/回归对比用）。
    let expand = !matches!(std::env::var("ZERO_PNG_EXPAND").as_deref(), Ok("0"));
    if expand {
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    }
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("PNG 解码失败 {}: {e}", path.display()))?;

    let info = reader.info().clone();
    let width = info.width;
    let height = info.height;

    if expand {
        let mut raw = vec![0u8; reader.output_buffer_size()];
        let output_info = reader
            .next_frame(&mut raw)
            .map_err(|e| format!("PNG 读取失败 {}: {e}", path.display()))?;
        let rgba = convert_png_buffer_to_rgba(&raw, output_info.color_type, output_info.bit_depth);
        return ImageData::from_rgba(rgba, width, height);
    }

    // 逃生舱路径（ZERO_PNG_EXPAND=0）：假设 RGBA8 直读（旧 baseline 行为）。
    let buf_size = (width as usize) * (height as usize) * 4;
    let mut buf = vec![0u8; buf_size];
    reader
        .next_frame(&mut buf)
        .map_err(|e| format!("PNG 读取失败 {}: {e}", path.display()))?;

    ImageData::from_rgba(buf, width, height)
}

/// 从 SVG data URI 生成 ImageData。
///
/// 支持简单的单色矩形 SVG（如 `<svg><rect fill='green' width='200' height='100'/></svg>`）。
/// 对于更复杂的 SVG，返回 None。
fn generate_svg_data_uri_image(url: &str) -> Option<ImageData> {
    let comma_pos = url.find(',')?;
    let svg_content = &url[comma_pos + 1..];

    // URL 解码
    let decoded = percent_decode_svg(svg_content);

    // 提取 SVG 尺寸
    let svg_start = decoded.find("<svg")?;
    let tag_end = decoded[svg_start..].find('>')?;
    let svg_tag = &decoded[svg_start..svg_start + tag_end];
    let svg_w = extract_svg_attr_float(svg_tag, "width")? as u32;
    let svg_h = extract_svg_attr_float(svg_tag, "height")? as u32;

    if svg_w == 0 || svg_h == 0 || svg_w > 4096 || svg_h > 4096 {
        return None;
    }

    // 提取第一个 <rect> 的 fill 颜色
    let rect_fill = extract_first_rect_fill(&decoded[svg_start + tag_end..])?;

    // 生成纯色 ImageData
    let [r, g, b, a] = rect_fill;
    let buf_size = (svg_w as usize) * (svg_h as usize) * 4;
    let mut buf = vec![0u8; buf_size];
    for pixel in buf.chunks_exact_mut(4) {
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
        pixel[3] = a;
    }

    ImageData::from_rgba(buf, svg_w, svg_h).ok()
}

/// 简易 percent-decode。
fn percent_decode_svg(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 从 SVG 标签属性中提取浮点数值。
fn extract_svg_attr_float(tag: &str, attr: &str) -> Option<f32> {
    let prefix = format!("{}=", attr);
    let pos = tag.find(&prefix)?;
    let rest = &tag[pos + prefix.len()..];
    let (quote, offset) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };
    let value_str = &rest[offset..];
    let end = value_str.find(quote)?;
    value_str[..end].parse::<f32>().ok()
}

/// 从 SVG 内容中提取第一个 <rect> 的 fill 颜色。
/// 支持命名颜色（如 "green", "red", "blue"）和十六进制颜色（如 "#00ff00"）。
fn extract_first_rect_fill(svg_content: &str) -> Option<[u8; 4]> {
    let rect_start = svg_content.find("<rect")?;
    let rect_end = svg_content[rect_start..].find("/>")?;
    let rect_tag = &svg_content[rect_start..rect_start + rect_end];

    // 查找 fill 属性
    let fill_prefix = "fill=";
    let pos = rect_tag.find(fill_prefix)?;
    let rest = &rect_tag[pos + fill_prefix.len()..];
    let (quote, offset) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };
    let value_str = &rest[offset..];
    let end = value_str.find(quote)?;
    let color_name = &value_str[..end];

    parse_css_color(color_name)
}

/// 解析 CSS 颜色名称或十六进制颜色。
fn parse_css_color(name: &str) -> Option<[u8; 4]> {
    match name {
        "green" => Some([0, 128, 0, 255]),
        "red" => Some([255, 0, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "white" => Some([255, 255, 255, 255]),
        "black" => Some([0, 0, 0, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "orange" => Some([255, 165, 0, 255]),
        "purple" => Some([128, 0, 128, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        "lime" => Some([0, 255, 0, 255]),
        "navy" => Some([0, 0, 128, 255]),
        "cyan" | "aqua" => Some([0, 255, 255, 255]),
        "magenta" | "fuchsia" => Some([255, 0, 255, 255]),
        "silver" => Some([192, 192, 192, 255]),
        "maroon" => Some([128, 0, 0, 255]),
        "olive" => Some([128, 128, 0, 255]),
        "teal" => Some([0, 128, 128, 255]),
        "transparent" => Some([0, 0, 0, 0]),
        hex if hex.starts_with('#') => parse_hex_color(hex),
        _ => None,
    }
}

/// 解析 #RGB 或 #RRGGBB 十六进制颜色。
fn parse_hex_color(hex: &str) -> Option<[u8; 4]> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some([r, g, b, 255])
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r, g, b, 255])
        }
        _ => None,
    }
}

/// 从 ImageCache 中提取所有图像的固有尺寸。
///
/// 遍历 HTML 中的所有图片 URL，查找缓存中对应的 ImageData，
/// 返回 (url_hash → (width, height)) 映射，供 Painter 用于
/// background-size: auto 计算。
fn extract_image_sizes(image_cache: &mut ImageCache, html: &str) -> std::collections::HashMap<u64, (f32, f32)> {
    let mut sizes = std::collections::HashMap::new();

    let mut all_urls = extract_img_srcs(html);
    all_urls.extend(extract_css_urls(html));
    all_urls.sort_unstable();
    all_urls.dedup();

    for url in &all_urls {
        let key = ImageKey::new(simple_hash(url));
        if let Some(data) = image_cache.get(&key) {
            let s = data.size();
            sizes.insert(key.0, (s.width, s.height));
        }
    }

    sizes
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
    // 提取并执行 <script> 标签中的 JS 代码
    execute_scripts(html);

    // 先构建图像缓存，提取固有尺寸供 paint 阶段使用
    let mut image_cache = build_image_cache(html, base_dir);
    let image_sizes = extract_image_sizes(&mut image_cache, html);

    let linked_css = load_linked_stylesheets(html, base_dir);
    let combined_css = if css.is_empty() {
        linked_css
    } else if linked_css.is_empty() {
        css.to_string()
    } else {
        format!("{linked_css}\n{css}")
    };

    let mut pipeline = RenderPipeline::new(config.viewport_width as f32, config.viewport_height as f32);
    pipeline.set_skip_indicators(true);
    pipeline.set_image_sizes(image_sizes);

    // 构建字体查找表（在 render_html 之前，以便 Painter 解析 CSS font-family）
    let font_loader = create_font_loader();
    let font_resolver = font_loader.build_font_resolver();
    pipeline.set_font_resolver(font_resolver);

    let result = pipeline.render_html(html, &combined_css);

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
    )
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

/// 从 HTML 中提取 `<script>` 标签内容并通过 V8 runtime 执行。
///
/// 当前实现为"执行但不修改 DOM"模式：
/// - JS 代码在独立的 V8 sandbox 中执行
/// - 不提供 DOM API（document, window 等）
/// - JS 执行结果不影响后续渲染
///
/// 这适用于大多数 WPT CSS reftest 场景，其中 JS 仅用于：
/// - 设置 CSS 变量或类名（已通过 HTML 内联处理）
/// - 断言测试条件（不影响渲染输出）
/// - 动态生成内容（少数场景，后续版本支持）
fn execute_scripts(html: &str) {
    let scripts = extract_script_content(html);
    if scripts.is_empty() {
        return;
    }

    // 合并所有 <script> 内容
    let combined_js: String = scripts.join(";\n");
    if combined_js.trim().is_empty() {
        return;
    }

    // 使用 V8 sandbox 执行 JS
    use zero_script_sandbox::{SandboxConfig, V8Sandbox};

    let config = SandboxConfig {
        timeout_ms: 5000, // 5 秒超时
        ..Default::default()
    };

    if let Ok(mut sandbox) = V8Sandbox::with_config(config)
        && let Err(e) = sandbox.execute(&combined_js)
    {
        // JS 执行失败不阻塞渲染（reftest 仍可运行）
        eprintln!("  [reftest JS] Script execution warning: {e}");
    }
}

/// 从 HTML 字符串中提取所有 `<script>` 标签的内容。
fn extract_script_content(html: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        // 查找 <script 标签
        let Some(script_start) = html[pos..].find("<script") else {
            break;
        };
        let abs_start = pos + script_start;

        // 跳过 <script> 或 <script type="...">
        let Some(tag_end) = html[abs_start..].find('>') else {
            break;
        };
        let content_start = abs_start + tag_end + 1;

        // 检查是否是外部脚本（src=），跳过外部脚本
        let tag_content = &html[abs_start..abs_start + tag_end];
        if tag_content.contains("src=") {
            pos = content_start;
            continue;
        }

        // 查找 </script>
        let Some(close_tag) = html[content_start..].find("</script>") else {
            break;
        };
        let script_content = html[content_start..content_start + close_tag].to_string();

        if !script_content.trim().is_empty() {
            scripts.push(script_content);
        }

        pos = content_start + close_tag + "</script>".len();
    }

    scripts
}

/// 创建加载了系统字体和 Ahem 测试字体的 FontLoader。
///
/// 加载顺序：
/// 1. 系统字体（DejaVu/Liberation 系列）
/// 2. Ahem 测试字体（WPT 标准测试字体，每个字符渲染为实心方块）
fn create_font_loader() -> FontLoader {
    let mut loader = FontLoader::new();
    let mut fallback_ids: Vec<u32> = Vec::new();

    // 系统字体路径（Linux 常见路径）
    let system_font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];

    for path in &system_font_paths {
        if let Ok(data) = std::fs::read(path) {
            let _ = loader.load_font(&data);
        }
    }

    // 加载 CJK 字体（Noto Sans CJK）并加入回退链——主字体缺 CJK 字形时回退到此，
    // 使中文/日文/韩文字符可渲染（DC-13 welcome.html 等含 CJK 文本的真实页面）。
    let cjk_font_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ];
    for path in &cjk_font_paths {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(id) = loader.load_font(&data) {
                fallback_ids.push(id);
            }
            break;
        }
    }

    // 加载 Ahem 测试字体（WPT reftest 标准字体）
    let ahem_path = "tests/wpt-runner/fonts/Ahem.ttf";
    if let Ok(data) = std::fs::read(ahem_path) {
        let _ = loader.load_font(&data);
    }

    if !fallback_ids.is_empty() {
        loader.set_fallback_chain(fallback_ids);
    }

    loader
}

/// 比较两个帧缓冲的像素。
///
/// 返回 (不同像素数, 最大单通道色差)。
pub fn compare_pixels(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8) -> (usize, u8) {
    compare_pixels_labeled(fb1, fb2, threshold, "")
}

/// 带标签的像素对比 —— 标签会附加到 REFTEST_BBOX 诊断行，便于定位差异归属。
pub fn compare_pixels_labeled(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8, label: &str) -> (usize, u8) {
    let mut diff_pixels = 0usize;
    let mut max_diff = 0u8;
    // 调试工具：设置 REFTEST_BBOX 环境变量时，打印差异像素的包围盒，
    // 帮助定位失败用例的差异区域（图像分析工具不可靠时的精确替代）。
    let track_bbox = std::env::var("REFTEST_BBOX").is_ok();
    let fw = fb1.width as usize;
    let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
    let (mut max_x, mut max_y) = (0usize, 0usize);

    for i in (0..fb1.data.len()).step_by(4) {
        let r1 = fb1.data[i];
        let g1 = fb1.data[i + 1];
        let b1 = fb1.data[i + 2];
        let a1 = fb1.data[i + 3];

        let r2 = fb2.data.get(i).copied().unwrap_or(0);
        let g2 = fb2.data.get(i + 1).copied().unwrap_or(0);
        let b2 = fb2.data.get(i + 2).copied().unwrap_or(0);
        let a2 = fb2.data.get(i + 3).copied().unwrap_or(0);

        let dr = (r1 as i16 - r2 as i16).unsigned_abs() as u8;
        let dg = (g1 as i16 - g2 as i16).unsigned_abs() as u8;
        let db = (b1 as i16 - b2 as i16).unsigned_abs() as u8;
        let da = (a1 as i16 - a2 as i16).unsigned_abs() as u8;

        let channel_max = dr.max(dg).max(db).max(da);
        max_diff = max_diff.max(channel_max);

        if channel_max > threshold {
            diff_pixels += 1;
            if track_bbox {
                let px = (i / 4) % fw;
                let py = (i / 4) / fw;
                if px < min_x {
                    min_x = px;
                }
                if py < min_y {
                    min_y = py;
                }
                if px > max_x {
                    max_x = px;
                }
                if py > max_y {
                    max_y = py;
                }
            }
        }
    }

    if track_bbox && diff_pixels > 0 {
        eprintln!("[REFTEST_BBOX] {label} x=[{min_x},{max_x}] y=[{min_y},{max_y}] fb_w={fw}");
    }

    (diff_pixels, max_diff)
}

/// 将帧缓冲保存为 PNG 文件。
pub fn save_framebuffer_png(fb: &FrameBuffer, path: &std::path::Path) -> Result<(), String> {
    // 简单的 BMP 保存（避免引入 PNG 编码依赖）
    // 使用 PPM 格式（最简单的无损图像格式）
    let ppm_path = path.with_extension("ppm");
    let mut content = format!("P6\n{} {}\n255\n", fb.width, fb.height);
    for i in (0..fb.data.len()).step_by(4) {
        content.push(fb.data[i] as char);
        content.push(fb.data[i + 1] as char);
        content.push(fb.data[i + 2] as char);
    }
    std::fs::write(&ppm_path, content.as_bytes()).map_err(|e| format!("Failed to save framebuffer: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
