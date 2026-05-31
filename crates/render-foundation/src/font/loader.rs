//! 字体加载器 — 使用 fontdue 加载和管理字体

use crate::font::{FontDesc, FontError, GlyphBitmap};
use hashbrown::HashMap;

/// 字体加载器 — 管理字体集合
pub struct FontLoader {
    /// 已加载的字体（fontdue 实例）
    fonts: HashMap<u32, fontdue::Font>,
    /// 下一个字体 ID
    next_id: u32,
    /// 字体族到 ID 的映射
    family_map: HashMap<String, Vec<u32>>,
}

impl FontLoader {
    /// 创建空的字体加载器
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            next_id: 0,
            family_map: HashMap::new(),
        }
    }

    /// 从字节数据加载字体
    pub fn load_font(&mut self, data: &[u8]) -> Result<u32, FontError> {
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|e| FontError::ParseFailed(e.to_string()))?;

        let id = self.next_id;
        self.next_id += 1;

        // 获取字体族名称
        if let Some(name) = get_font_family_name(&font) {
            self.family_map.entry(name).or_default().push(id);
        }

        self.fonts.insert(id, font);
        Ok(id)
    }

    /// 根据 ID 获取字体
    pub fn get(&self, id: u32) -> Option<&fontdue::Font> {
        self.fonts.get(&id)
    }

    /// 根据字体描述查找最佳匹配字体 ID
    pub fn find(&self, desc: &FontDesc) -> Option<u32> {
        self.family_map.get(&desc.family).and_then(|ids| ids.first().copied())
    }

    /// 渲染指定字符的 glyph
    pub fn rasterize_glyph(&self, font_id: u32, code_point: char, size: f32) -> Result<GlyphBitmap, FontError> {
        let font = self
            .fonts
            .get(&font_id)
            .ok_or_else(|| FontError::NotFound(format!("font_id={font_id}")))?;

        let (metrics, bitmap) = font.rasterize(code_point, size);

        Ok(GlyphBitmap {
            data: bitmap,
            width: metrics.width as u16,
            height: metrics.height as u16,
            x_offset: metrics.xmin as i16,
            y_offset: metrics.ymin as i16,
            advance: metrics.advance_width,
        })
    }

    /// 已加载字体数量
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 尝试获取字体的族名称
fn get_font_family_name(font: &fontdue::Font) -> Option<String> {
    // fontdue 0.9 不暴露 names 字段，返回 None
    // 后续可通过 swash 或其他方式获取字体元数据
    let _ = font;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_font_loader_empty() {
        let loader = FontLoader::new();
        assert!(loader.is_empty());
        assert_eq!(loader.len(), 0);
    }

    #[test]
    fn test_font_desc_normal() {
        let desc = FontDesc::normal("Arial");
        assert_eq!(desc.family, "Arial");
        assert_eq!(desc.weight, 400);
        assert!(!desc.italic);
    }

    #[test]
    fn test_font_desc_bold() {
        let desc = FontDesc::bold("Arial");
        assert_eq!(desc.weight, 700);
    }

    #[test]
    fn test_font_loader_get_nonexistent() {
        let loader = FontLoader::new();
        assert!(loader.get(999).is_none());
    }

    #[test]
    fn test_font_loader_rasterize_nonexistent() {
        let loader = FontLoader::new();
        let result = loader.rasterize_glyph(999, 'A', 16.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_font_loader_find_nonexistent() {
        let loader = FontLoader::new();
        let desc = FontDesc::normal("NonExistent");
        assert!(loader.find(&desc).is_none());
    }

    /// 加载真实字体文件并验证光栅化输出
    ///
    /// 使用系统 DejaVu 字体验证 fontdue 集成能正确解码字体、
    /// 光栅化 glyph 并生成有效的位图数据。
    #[test]
    fn test_font_loader_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        assert_eq!(loader.len(), 1);

        // Verify the font can be retrieved
        assert!(loader.get(font_id).is_some());

        // Rasterize 'A' at 16px
        let result = loader.rasterize_glyph(font_id, 'A', 16.0);
        assert!(result.is_ok(), "should rasterize 'A' glyph");

        let bitmap = result.unwrap();
        // Verify bitmap dimensions are reasonable
        assert!(bitmap.width > 0, "width should be > 0, got {}", bitmap.width);
        assert!(bitmap.height > 0, "height should be > 0, got {}", bitmap.height);
        // Verify bitmap data size matches dimensions
        assert_eq!(
            bitmap.data.len(),
            bitmap.width as usize * bitmap.height as usize,
            "bitmap data size should match width * height"
        );
        // Verify advance width is positive
        assert!(
            bitmap.advance > 0.0,
            "advance width should be > 0, got {}",
            bitmap.advance
        );

        // Verify bitmap contains non-zero pixels (the glyph is actually rendered)
        let non_zero_count = bitmap.data.iter().filter(|&&b| b > 0).count();
        assert!(non_zero_count > 0, "bitmap should contain non-zero pixels for 'A'");
    }

    /// 测试不同大小的光栅化产生不同尺寸的 glyph
    #[test]
    fn test_font_loader_rasterize_different_sizes() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load");

        let small = loader.rasterize_glyph(font_id, 'M', 12.0).unwrap();
        let large = loader.rasterize_glyph(font_id, 'M', 32.0).unwrap();

        // Larger font size should generally produce larger or equal bitmaps
        let small_area = small.width as u32 * small.height as u32;
        let large_area = large.width as u32 * large.height as u32;
        assert!(
            large_area >= small_area,
            "larger font size should produce >= bitmap area: {large_area} vs {small_area}"
        );

        // Advance width should scale proportionally
        assert!(
            large.advance > small.advance,
            "large advance ({}) should > small advance ({})",
            large.advance,
            small.advance
        );
    }

    /// 测试加载无效字节会返回解析错误
    #[test]
    fn test_font_loader_invalid_bytes() {
        let mut loader = FontLoader::new();
        let result = loader.load_font(&[0xFF, 0xFE, 0xFD, 0xFC]);
        assert!(result.is_err());
    }

    /// 测试重复加载同一字体会分配不同 ID
    #[test]
    fn test_font_loader_duplicate_loads() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let id1 = loader.load_font(&font_data).unwrap();
        let id2 = loader.load_font(&font_data).unwrap();
        assert_ne!(id1, id2, "each load should get a unique ID");
        assert_eq!(loader.len(), 2);
    }

    /// 测试多个不同字符的光栅化
    #[test]
    fn test_font_loader_rasterize_multiple_chars() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load");

        // Rasterize several characters and verify they all produce valid bitmaps
        for ch in ['A', 'z', '0', ' ', '!'] {
            let result = loader.rasterize_glyph(font_id, ch, 20.0);
            assert!(result.is_ok(), "should rasterize '{}'", ch);
            let bitmap = result.unwrap();
            assert_eq!(
                bitmap.data.len(),
                bitmap.width as usize * bitmap.height as usize,
                "bitmap data size mismatch for '{}'",
                ch
            );
            assert!(
                bitmap.advance >= 0.0,
                "advance should be >= 0 for '{}', got {}",
                ch,
                bitmap.advance
            );
        }
    }

    /// 测试 glyph 偏移量属性
    #[test]
    fn test_font_loader_glyph_offsets() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load");

        let bitmap = loader.rasterize_glyph(font_id, 'g', 16.0).unwrap();
        // 'g' typically has a negative y_offset (descender)
        // Just verify the offset values are within reasonable bounds
        assert!(
            bitmap.x_offset.abs() < 100,
            "x_offset should be reasonable, got {}",
            bitmap.x_offset
        );
        assert!(
            bitmap.y_offset.abs() < 100,
            "y_offset should be reasonable, got {}",
            bitmap.y_offset
        );
    }

    #[test]
    fn test_font_loader_default() {
        let loader = FontLoader::default();
        assert!(loader.is_empty());
    }

    #[test]
    fn test_font_desc_normal_default_weight() {
        let desc = FontDesc::normal("TestFont");
        assert_eq!(desc.weight, 400);
        assert!(!desc.italic);
        assert_eq!(desc.family, "TestFont");
    }

    #[test]
    fn test_font_desc_bold_weight() {
        let desc = FontDesc::bold("TestFont");
        assert_eq!(desc.weight, 700);
        assert!(!desc.italic);
    }

    #[test]
    fn test_font_desc_custom() {
        let desc = FontDesc {
            family: "Serif".to_string(),
            weight: 300,
            italic: true,
        };
        assert_eq!(desc.weight, 300);
        assert!(desc.italic);
    }
}
