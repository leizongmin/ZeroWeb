//! Reftest Harness — 渲染测试 HTML 和参考 HTML，比较像素输出。
//!
//! 实现 WPT 风格的 `rel=match` / `rel=mismatch` 比较逻辑：
//! - match：两个页面的像素应几乎相同（允许模糊阈值）
//! - mismatch：两个页面的像素应有显著差异
//!
//! 支持分类容差（布局类 vs 文字类）和 per-test WPT fuzzy 注解覆盖。

#![allow(dead_code)]

use zero_engine::RenderPipeline;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;
use zero_render_foundation::primitive::{FillPrimitive, GlyphPrimitive, RoundedRectPrimitive};
use zero_render_foundation::surface::FrameBuffer;

use crate::manifest::FuzzyMeta;

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
    // 渲染测试页面
    let test_fb = render_to_framebuffer(&case.test_html, &case.css, config);
    // 渲染参考页面
    let ref_fb = render_to_framebuffer(&case.ref_html, &case.css, config);

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
    let (diff_pixels, max_channel_diff) = compare_pixels(&test_fb, &ref_fb, eff_channel_diff);
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

/// 将 HTML 渲染到 CPU 帧缓冲。
///
/// 如果 HTML 中包含 `<script>` 标签，会先通过 V8 runtime 执行其中的 JS 代码，
/// 然后再进行渲染。当前实现中 JS 执行不修改 DOM（适用于大多数 WPT reftest
/// 中 JS 仅用于设置/断言的场景）。
pub fn render_to_framebuffer(html: &str, css: &str, config: &ReftestConfig) -> FrameBuffer {
    // 提取并执行 <script> 标签中的 JS 代码
    execute_scripts(html);

    let mut pipeline = RenderPipeline::new(config.viewport_width as f32, config.viewport_height as f32);
    let result = pipeline.render_html(html, css);

    // 从 RenderPrimitives 提取 fills、rounded_rects 和 glyphs
    let fills: Vec<FillPrimitive> = result.primitives.fills.clone();
    let rounded_rects: Vec<RoundedRectPrimitive> = result.primitives.rounded_rects.clone();
    let glyph_primitives: Vec<GlyphPrimitive> = result.primitives.glyphs.clone();

    // 将 GlyphPrimitive 转换为 GlyphDraw（CPU 渲染器需要的格式）
    let glyph_draws: Vec<GlyphDraw> = glyph_primitives
        .iter()
        .map(|g| GlyphDraw {
            ch: char::from_u32(g.glyph_id).unwrap_or('?'),
            x: g.x,
            baseline_y: g.y,
            font_size: g.font_size,
            color: g.color,
            font_id: g.font_id.0,
        })
        .collect();

    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(1024);

    render_scene_to_framebuffer(
        config.viewport_width,
        config.viewport_height,
        config.scale_factor,
        &fills,
        &rounded_rects,
        &font_loader,
        &mut glyph_cache,
        &glyph_draws,
        &[],
        &[],
    )
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

/// 比较两个帧缓冲的像素。
///
/// 返回 (不同像素数, 最大单通道色差)。
pub fn compare_pixels(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8) -> (usize, u8) {
    let mut diff_pixels = 0usize;
    let mut max_diff = 0u8;

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
        }
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
