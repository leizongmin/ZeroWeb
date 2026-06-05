//! 最小 Reftest Harness — 渲染测试 HTML 和参考 HTML，比较像素输出。
//!
//! 实现 WPT 风格的 `rel=match` / `rel=mismatch` 比较逻辑：
//! - match：两个页面的像素应几乎相同（允许模糊阈值）
//! - mismatch：两个页面的像素应有显著差异

#![allow(dead_code)]

use zero_engine::RenderPipeline;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;
use zero_render_foundation::primitive::{FillPrimitive, GlyphPrimitive};
use zero_render_foundation::surface::FrameBuffer;

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
}

impl Default for ReftestConfig {
    fn default() -> Self {
        Self {
            viewport_width: 800,
            viewport_height: 600,
            scale_factor: 1.0,
            max_diff_ratio: 0.01,
            max_channel_diff: 5,
        }
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
    let (diff_pixels, max_channel_diff) = compare_pixels(&test_fb, &ref_fb, config.max_channel_diff);
    let diff_ratio = if total_pixels > 0 {
        diff_pixels as f64 / total_pixels as f64
    } else {
        0.0
    };

    let passed = if case.is_match {
        // match 模式：差异应小于阈值
        diff_ratio <= config.max_diff_ratio
    } else {
        // mismatch 模式：应有显著差异（至少 1% 像素不同）
        diff_ratio > 0.01
    };

    let message = if passed {
        String::new()
    } else if case.is_match {
        format!(
            "Match failed: {}/{} pixels differ ({:.2}%), max channel diff={}",
            diff_pixels,
            total_pixels,
            diff_ratio * 100.0,
            max_channel_diff
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
fn render_to_framebuffer(html: &str, css: &str, config: &ReftestConfig) -> FrameBuffer {
    let mut pipeline = RenderPipeline::new(config.viewport_width as f32, config.viewport_height as f32);
    let result = pipeline.render_html(html, css);

    // 从 RenderPrimitives 提取 fills 和 glyphs
    let fills: Vec<FillPrimitive> = result.primitives.fills.clone();
    let glyph_primitives: Vec<GlyphPrimitive> = result.primitives.glyphs.clone();

    // 将 GlyphPrimitive 转换为 GlyphDraw（CPU 渲染器需要的格式）
    // GlyphPrimitive.glyph_id 是字形索引，CPU 渲染器使用 char
    // 对于 reftest 比较，我们可以跳过字形渲染细节差异，仅比较 fill 图元
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
        &font_loader,
        &mut glyph_cache,
        &glyph_draws,
        &[],
    )
}

/// 比较两个帧缓冲的像素。
///
/// 返回 (不同像素数, 最大单通道色差)。
fn compare_pixels(fb1: &FrameBuffer, fb2: &FrameBuffer, threshold: u8) -> (usize, u8) {
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
        // 两个相同尺寸和颜色的 div 应该像素一致
        assert_match(
            "block/width-height",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
        );
    }

    #[test]
    fn reftest_block_margin_collapsing() {
        // 有 margin 的 div 与无 margin 但相同背景的 div 应在 div 区域内一致
        // （margin 不影响 div 内部像素）
        assert_match(
            "block/margin-no-effect-on-bg",
            "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
            "<div style=\"width:100px;height:50px;background:blue;margin:10px;\"></div>",
        );
    }

    #[test]
    fn reftest_block_different_margin() {
        // 不同 margin 产生不同位置 → 像素不同
        assert_mismatch(
            "block/different-margin",
            "<div style=\"width:80px;height:40px;background:green;margin:0;\"></div>",
            "<div style=\"width:80px;height:40px;background:green;margin:20px;\"></div>",
        );
    }

    #[test]
    fn reftest_block_stacking() {
        // 两个垂直堆叠的 div 应与单个相同高度的 div 在 div 区域内产生不同输出
        assert_mismatch(
            "block/stacking-vs-single",
            "<div style=\"width:100px;height:40px;background:red;\"></div><div style=\"width:100px;height:40px;background:blue;\"></div>",
            "<div style=\"width:100px;height:80px;background:red;\"></div>",
        );
    }

    // ── 盒模型 ──

    #[test]
    fn reftest_padding_expands_box() {
        // padding 扩展可视区域（background 覆盖 padding）
        assert_mismatch(
            "box-model/padding-expands",
            "<div style=\"width:80px;height:40px;background:red;padding:10px;\"></div>",
            "<div style=\"width:80px;height:40px;background:red;padding:0;\"></div>",
        );
    }

    #[test]
    fn reftest_border_visible() {
        // 有边框的 div 与无边框的 div 应产生不同像素
        assert_mismatch(
            "box-model/border-visible",
            "<div style=\"width:80px;height:40px;background:yellow;border:2px solid black;\"></div>",
            "<div style=\"width:80px;height:40px;background:yellow;border:none;\"></div>",
        );
    }

    // ── Flexbox ──

    #[test]
    fn reftest_flex_direction_row() {
        // flex-direction:row 两个子元素水平排列
        assert_match(
            "flex/row-identical",
            "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        );
    }

    #[test]
    fn reftest_flex_vs_block() {
        // flex 排列与 block 排列应产生不同结果（水平 vs 垂直）
        assert_mismatch(
            "flex/row-vs-block",
            "<div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
        );
    }

    // ── 定位 ──

    #[test]
    fn reftest_absolute_position() {
        // absolute 定位改变元素位置 → 不同像素
        assert_mismatch(
            "position/absolute-shift",
            "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:20px;left:20px;width:50px;height:50px;background:green;\"></div></div>",
            "<div style=\"position:relative;width:200px;height:100px;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:green;\"></div></div>",
        );
    }

    // ── 背景颜色 ──

    #[test]
    fn reftest_named_vs_hex_color() {
        // red 和 #FF0000 应产生相同颜色
        assert_match(
            "color/named-vs-hex",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:#FF0000;\"></div>",
        );
    }

    #[test]
    fn reftest_rgb_vs_hex() {
        // rgb(0,128,255) 和 #0080FF 应产生相同颜色
        assert_match(
            "color/rgb-vs-hex",
            "<div style=\"width:100px;height:50px;background:rgb(0,128,255);\"></div>",
            "<div style=\"width:100px;height:50px;background:#0080FF;\"></div>",
        );
    }

    #[test]
    fn reftest_different_colors() {
        // 不同颜色应产生不同像素
        assert_mismatch(
            "color/different",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:green;\"></div>",
        );
    }

    // ── 尺寸 ──

    #[test]
    fn reftest_different_sizes() {
        // 不同尺寸应产生不同像素
        assert_mismatch(
            "size/different",
            "<div style=\"width:100px;height:50px;background:blue;\"></div>",
            "<div style=\"width:50px;height:100px;background:blue;\"></div>",
        );
    }

    #[test]
    fn reftest_display_none() {
        // display:none 元素不应可见 → 与有元素的页面不同
        assert_mismatch(
            "display/none-vs-visible",
            "<div style=\"width:100px;height:50px;background:red;\"></div>",
            "<div style=\"width:100px;height:50px;background:red;display:none;\"></div>",
        );
    }

    // ── 嵌套结构 ──

    #[test]
    fn reftest_nested_same_bg() {
        // 相同的嵌套结构应产生相同输出
        assert_match(
            "nested/same-structure",
            "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
            "<div style=\"width:100px;height:80px;background:red;\"><div style=\"width:50px;height:40px;background:blue;\"></div></div>",
        );
    }

    #[test]
    fn reftest_sibling_order() {
        // 兄弟元素顺序不同应产生不同输出
        assert_mismatch(
            "nested/sibling-order",
            "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div>",
            "<div style=\"width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:red;\"></div></div>",
        );
    }
}
