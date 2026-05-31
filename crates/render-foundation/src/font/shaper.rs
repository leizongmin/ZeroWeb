//! 文本整形 — 将 Unicode 文本转换为 Glyph 序列，支持简单换行。
//!
//! TextShaper 基于 fontdue 将每个字符映射为 glyph ID 和前进宽度，
//! 并根据行宽进行简单的逐字符换行。

use crate::font::loader::FontLoader;
use crate::primitive::FontId;

/// 单个整形后的 Glyph 信息
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// 字体内部的 glyph 索引
    pub glyph_id: u32,
    /// 字体 ID
    pub font_id: FontId,
    /// 相对于行首的水平前进宽度（像素）
    pub advance_x: f32,
    /// 该字符的 Unicode 码点（用于回退标识）
    pub code_point: char,
}

/// 一行整形结果
#[derive(Debug, Clone)]
pub struct ShapedLine {
    /// 该行的 glyph 序列
    pub glyphs: Vec<ShapedGlyph>,
    /// 行的总前进宽度
    pub width: f32,
}

/// 文本整形器 — 将文本字符串转换为带位置的 glyph 序列
pub struct TextShaper<'a> {
    /// 字体加载器引用
    font_loader: &'a FontLoader,
    /// 默认字体 ID（整形时使用）
    default_font_id: Option<FontId>,
}

impl<'a> TextShaper<'a> {
    /// 使用指定字体加载器创建整形器。
    ///
    /// `default_font_id` 为整形使用的默认字体 ID，可以为 None（此时整形仅产生占位 glyph）。
    pub fn new(font_loader: &'a FontLoader, default_font_id: Option<FontId>) -> Self {
        Self {
            font_loader,
            default_font_id,
        }
    }

    /// 将文本整形为 glyph 序列，不进行换行（单行模式）。
    ///
    /// 返回每个字符对应的 glyph ID 和累积前进宽度。
    pub fn shape_single_line(&self, text: &str, font_size: f32) -> Vec<ShapedGlyph> {
        let mut glyphs = Vec::with_capacity(text.len());
        let font_id = self.default_font_id.unwrap_or(FontId(0));

        for ch in text.chars() {
            let (glyph_id, advance_x) = if let Some(fid) = self.default_font_id {
                match self.query_glyph_metrics(fid.0, ch, font_size) {
                    Some((gid, adv)) => (gid, adv),
                    None => (0u32, font_size * 0.6),
                }
            } else {
                // 无字体可用，使用码点作为占位 glyph_id
                (ch as u32, font_size * 0.6)
            };

            glyphs.push(ShapedGlyph {
                glyph_id,
                font_id,
                advance_x,
                code_point: ch,
            });
        }

        glyphs
    }

    /// 将文本整形为多行 glyph 序列，在指定行宽处换行。
    ///
    /// 换行规则：逐字符累积前进宽度，当累积宽度超过 `max_line_width` 时
    /// 在最后一个空格处折行；如果没有空格则在超限处折行。
    /// 显式换行符 `'\n'` 强制折行。
    pub fn shape_with_line_wrap(&self, text: &str, font_size: f32, max_line_width: f32) -> Vec<ShapedLine> {
        if max_line_width <= 0.0 || text.is_empty() {
            return vec![ShapedLine {
                glyphs: vec![],
                width: 0.0,
            }];
        }

        let glyphs = self.shape_single_line(text, font_size);

        let mut lines: Vec<ShapedLine> = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();
        let mut current_width: f32 = 0.0;
        let mut last_space_idx: Option<usize> = None;
        let mut width_at_last_space: f32 = 0.0;

        for glyph in glyphs.iter() {
            // 显式换行符
            if glyph.code_point == '\n' {
                lines.push(ShapedLine {
                    glyphs: std::mem::take(&mut current_line),
                    width: current_width,
                });
                current_width = 0.0;
                last_space_idx = None;
                width_at_last_space = 0.0;
                continue;
            }

            // 记录空格位置
            if glyph.code_point == ' ' {
                last_space_idx = Some(current_line.len());
                width_at_last_space = current_width;
            }

            let new_width = current_width + glyph.advance_x;

            if new_width > max_line_width && !current_line.is_empty() {
                // 需要换行
                if let Some(space_idx) = last_space_idx {
                    // 在最后一个空格处折行
                    let remaining: Vec<ShapedGlyph> = current_line.split_off(space_idx);
                    lines.push(ShapedLine {
                        glyphs: current_line,
                        width: width_at_last_space,
                    });
                    // 跳过折行处的空格
                    current_line = remaining.into_iter().skip(1).collect();
                    current_width = current_line.iter().map(|g| g.advance_x).sum();
                } else {
                    // 没有空格，在超限处折行
                    lines.push(ShapedLine {
                        glyphs: std::mem::take(&mut current_line),
                        width: current_width,
                    });
                    current_line.push(glyph.clone());
                    current_width = glyph.advance_x;
                }
                last_space_idx = None;
                width_at_last_space = 0.0;
            } else {
                current_line.push(glyph.clone());
                current_width = new_width;
            }
        }

        // 最后一行
        if !current_line.is_empty() {
            lines.push(ShapedLine {
                glyphs: current_line,
                width: current_width,
            });
        }

        if lines.is_empty() {
            lines.push(ShapedLine {
                glyphs: vec![],
                width: 0.0,
            });
        }

        lines
    }

    /// 查询指定字符在字体中的 glyph 索引和前进宽度。
    fn query_glyph_metrics(&self, font_id: u32, code_point: char, font_size: f32) -> Option<(u32, f32)> {
        let font = self.font_loader.get(font_id)?;
        let metrics = font.metrics(code_point, font_size);
        let glyph_index = font.lookup_glyph_index(code_point) as u32;
        Some((glyph_index, metrics.advance_width))
    }
}

/// 计算文本在指定字体大小下单行渲染所需的总宽度（像素）。
///
/// 如果没有可用字体则返回每个字符约 0.6 * font_size 的估算宽度。
pub fn measure_text_width(shaper: &TextShaper, text: &str, font_size: f32) -> f32 {
    shaper
        .shape_single_line(text, font_size)
        .iter()
        .map(|g| g.advance_x)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::loader::FontLoader;

    /// 创建空的 TextShaper（无字体）。
    fn make_empty_shaper() -> TextShaper<'static> {
        static LOADER: std::sync::OnceLock<FontLoader> = std::sync::OnceLock::new();
        let loader = LOADER.get_or_init(FontLoader::new);
        // Safety: FontLoader::new() 返回的对象在 OnceLock 中，生命周期为 'static
        // 我们通过 unsafe 指向 static 引用，但由于 TextShaper 使用在测试中
        // 且测试是同步的，这里使用 transmute 模拟 'static 生命周期
        // 这是一个已知的测试限制
        unsafe {
            let loader_ref: &'static FontLoader = loader;
            TextShaper::new(loader_ref, None)
        }
    }

    /// 查找一个可用的系统字体文件
    fn find_system_font() -> Option<std::path::PathBuf> {
        let candidates = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return Some(std::path::PathBuf::from(path));
            }
        }
        None
    }

    /// 加载系统字体数据（如果可用）
    fn load_system_font_data() -> Option<Vec<u8>> {
        let path = find_system_font()?;
        std::fs::read(path).ok()
    }

    /// 测试空文本整形。
    #[test]
    fn test_shape_empty_text() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("", 16.0);
        assert!(glyphs.is_empty());
    }

    /// 测试无字体时整形产生占位 glyph。
    #[test]
    fn test_shape_no_font_placeholder() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("AB", 16.0);
        assert_eq!(glyphs.len(), 2);
        // 无字体时 glyph_id 等于 code_point
        assert_eq!(glyphs[0].glyph_id, 'A' as u32);
        assert_eq!(glyphs[1].glyph_id, 'B' as u32);
        // 占位 advance 约 0.6 * font_size
        assert!((glyphs[0].advance_x - 16.0 * 0.6).abs() < 0.01);
    }

    /// 测试 measure_text_width 对空文本返回 0。
    #[test]
    fn test_measure_empty_text() {
        let shaper = make_empty_shaper();
        let width = measure_text_width(&shaper, "", 16.0);
        assert_eq!(width, 0.0);
    }

    /// 测试 measure_text_width 对非空文本返回正值。
    #[test]
    fn test_measure_nonempty_text() {
        let shaper = make_empty_shaper();
        let width = measure_text_width(&shaper, "Hello", 16.0);
        assert!(width > 0.0);
    }

    /// 测试换行：短文本不换行。
    #[test]
    fn test_line_wrap_short_text() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("Hi", 16.0, 1000.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 2);
    }

    /// 测试换行：显式换行符折行。
    #[test]
    fn test_line_wrap_newline() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("A\nB", 16.0, 1000.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].glyphs.len(), 1);
        assert_eq!(lines[0].glyphs[0].code_point, 'A');
        assert_eq!(lines[1].glyphs.len(), 1);
        assert_eq!(lines[1].glyphs[0].code_point, 'B');
    }

    /// 测试换行：宽度不足时在空格处折行。
    #[test]
    fn test_line_wrap_at_space() {
        let shaper = make_empty_shaper();
        // 每个字符 advance ≈ 0.6 * 16 = 9.6
        // "A B C D" = 7 chars, 总宽度 ≈ 67.2
        // 设 max_width = 25，第 3 个字符 "B" 时超出（28.8 > 25）
        // 在空格处折行 → 应产生多行
        let lines = shaper.shape_with_line_wrap("A B C D", 16.0, 25.0);
        assert!(lines.len() >= 2, "应在空格处换行，实际 {} 行", lines.len());
    }

    /// 测试换行：空文本返回单空行。
    #[test]
    fn test_line_wrap_empty() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("", 16.0, 100.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty());
    }

    /// 测试换行：零宽度返回空行。
    #[test]
    fn test_line_wrap_zero_width() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("Hello", 16.0, 0.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty());
    }

    /// 测试整形：跳过换行符本身不产生 glyph。
    #[test]
    fn test_shape_newline_not_in_glyphs() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("X\nY", 16.0, 1000.0);
        // 换行符不应出现在任何行的 glyph 中
        for line in &lines {
            assert!(!line.glyphs.iter().any(|g| g.code_point == '\n'));
        }
    }

    /// 测试 ShapedLine width 字段。
    #[test]
    fn test_shaped_line_width() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("ABC", 16.0, 1000.0);
        assert_eq!(lines.len(), 1);
        let expected_width: f32 = lines[0].glyphs.iter().map(|g| g.advance_x).sum();
        assert!((lines[0].width - expected_width).abs() < 0.01);
    }

    /// 测试连续换行符产生空行。
    #[test]
    fn test_consecutive_newlines() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("A\n\nB", 16.0, 1000.0);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].glyphs.len(), 1);
        assert_eq!(lines[1].glyphs.len(), 0);
        assert_eq!(lines[2].glyphs.len(), 1);
    }

    // ── 使用真实字体的测试（有系统字体时执行）──────────

    /// 测试使用真实字体整形。
    #[test]
    fn test_shape_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let glyphs = shaper.shape_single_line("Hello", 16.0);
        assert_eq!(glyphs.len(), 5, "应为 5 个字符生成 5 个 glyph");

        // 每个 glyph 的 advance 应该是正数
        for glyph in &glyphs {
            assert!(glyph.advance_x > 0.0, "advance_x 应为正数");
        }

        // 总宽度应该合理（5 个字符大约 40-80px）
        let total: f32 = glyphs.iter().map(|g| g.advance_x).sum();
        assert!(total > 20.0 && total < 200.0, "总宽度应合理，实际 {}", total);
    }

    /// 测试使用真实字体的换行。
    #[test]
    fn test_line_wrap_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        // "Hello World" 在 50px 宽度内应该换行
        let lines = shaper.shape_with_line_wrap("Hello World", 16.0, 50.0);
        assert!(lines.len() >= 2, "应在 50px 内换行，实际 {} 行", lines.len());

        // 每行的实际宽度不应超过 max_width 太多
        for line in &lines {
            // 允许单字符行超过 max_width（无法再分割）
            if line.glyphs.len() > 1 {
                assert!(line.width <= 60.0, "行宽度应接近 max_width，实际 {}", line.width);
            }
        }
    }

    /// 测试使用真实字体测量文本宽度。
    #[test]
    fn test_measure_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let width_16 = measure_text_width(&shaper, "Hello", 16.0);
        let width_32 = measure_text_width(&shaper, "Hello", 32.0);

        assert!(width_16 > 0.0, "16px 文本宽度应为正");
        assert!(width_32 > width_16, "32px 应比 16px 宽: {} vs {}", width_32, width_16);
    }
}
