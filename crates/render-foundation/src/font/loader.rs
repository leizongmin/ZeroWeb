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
    /// 回退字体链（CJK、Emoji 等），在主字体缺字时使用
    fallback_chain: Vec<u32>,
    /// 预注册位图 glyph（font_id, glyph_id, size_bits）→ 光栅结果
    bitmap_glyphs: HashMap<(u32, u32, u32), GlyphBitmap>,
}

impl FontLoader {
    /// 创建空的字体加载器
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            next_id: 0,
            family_map: HashMap::new(),
            fallback_chain: Vec::new(),
            bitmap_glyphs: HashMap::new(),
        }
    }

    /// 注册预光栅化的位图 glyph（如图标 atlas），按 `(font_id, glyph_id, size_px)` 查找。
    pub fn register_bitmap_glyph(&mut self, font_id: u32, glyph_id: u32, size_px: f32, bitmap: GlyphBitmap) {
        self.bitmap_glyphs
            .insert((font_id, glyph_id, size_px.to_bits()), bitmap);
    }

    /// 是否已注册指定位图 glyph。
    pub fn has_bitmap_glyph(&self, font_id: u32, glyph_id: u32, size_px: f32) -> bool {
        self.bitmap_glyphs.contains_key(&(font_id, glyph_id, size_px.to_bits()))
    }

    /// 设置回退字体链（按优先级排序）
    pub fn set_fallback_chain(&mut self, ids: Vec<u32>) {
        self.fallback_chain = ids;
    }

    /// 获取回退字体链
    pub fn fallback_chain(&self) -> &[u32] {
        &self.fallback_chain
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

    /// 在主字体及回退链中渲染 glyph，返回实际使用的字体 ID
    pub fn rasterize_glyph_with_fallback(
        &self,
        primary_id: u32,
        code_point: char,
        size: f32,
    ) -> Result<(u32, GlyphBitmap), FontError> {
        if let Some(bitmap) = self.bitmap_glyphs.get(&(primary_id, code_point as u32, size.to_bits())) {
            return Ok((primary_id, bitmap.clone()));
        }

        let mut chain = Vec::with_capacity(1 + self.fallback_chain.len());
        chain.push(primary_id);
        for &id in &self.fallback_chain {
            if id != primary_id && !chain.contains(&id) {
                chain.push(id);
            }
        }

        for font_id in chain {
            let font = match self.fonts.get(&font_id) {
                Some(font) => font,
                None => continue,
            };
            // 主字体缺字时会 rasterize .notdef 方块；须先检查字体是否包含该字符
            if !code_point.is_whitespace() && !font.has_glyph(code_point) {
                continue;
            }
            let bitmap = self.rasterize_glyph(font_id, code_point, size)?;
            if Self::glyph_has_coverage(code_point, &bitmap) {
                return Ok((font_id, bitmap));
            }
        }

        Err(FontError::GlyphNotFound {
            font_id: primary_id,
            glyph_id: code_point as u32,
        })
    }

    /// 测量字符 advance 宽度（含回退）
    pub fn measure_advance(&self, primary_id: u32, code_point: char, size: f32) -> f32 {
        if let Some(bitmap) = self.bitmap_glyphs.get(&(primary_id, code_point as u32, size.to_bits())) {
            return bitmap.advance;
        }
        self.rasterize_glyph_with_fallback(primary_id, code_point, size)
            .map(|(_, bitmap)| bitmap.advance)
            .unwrap_or(size * 0.5)
    }

    fn glyph_has_coverage(code_point: char, bitmap: &GlyphBitmap) -> bool {
        if code_point.is_whitespace() {
            return bitmap.advance > 0.0;
        }
        bitmap.width > 0 && bitmap.height > 0
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

    /// 测试加载空字节数据返回错误
    ///
    /// 空的 &[u8] 不是有效字体，load_font 应返回 ParseFailed 错误。
    #[test]
    fn test_font_loader_empty_data() {
        let mut loader = FontLoader::new();
        let result = loader.load_font(&[]);
        assert!(result.is_err(), "空数据应返回错误");
    }

    /// 测试字体 ID 单调递增
    ///
    /// 连续加载多个字体会分配 0, 1, 2... 的递增 ID。
    #[test]
    fn test_font_loader_id_monotonically_increases() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let id0 = loader.load_font(&font_data).unwrap();
        let id1 = loader.load_font(&font_data).unwrap();
        let id2 = loader.load_font(&font_data).unwrap();
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert!(id0 < id1 && id1 < id2, "字体 ID 应严格递增");
    }

    /// 测试加载非常短（但非空）的无效数据
    ///
    /// 仅 1 字节的数据不是有效字体格式。
    #[test]
    fn test_font_loader_single_byte_data() {
        let mut loader = FontLoader::new();
        let result = loader.load_font(&[0x00]);
        assert!(result.is_err(), "单字节数据应返回解析错误");
    }

    /// 测试光栅化控制字符不 panic
    ///
    /// 光栅化 NULL 字符（U+0000）等控制字符应返回有效结果或至少不崩溃。
    #[test]
    fn test_font_loader_rasterize_control_char() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load");

        // NULL 字符
        let result = loader.rasterize_glyph(font_id, '\0', 16.0);
        // fontdue 应能处理，即使结果可能是空的 glyph
        assert!(result.is_ok(), "光栅化 NULL 字符不应失败");
    }

    /// 测试字体加载器的内存管理
    #[test]
    fn test_font_loader_memory_usage() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();

        // 加载多个字体
        for i in 0..5 {
            let font_id = loader.load_font(&font_data).expect("should load");
            assert_eq!(font_id, i);
        }

        // 验证数量
        assert_eq!(loader.len(), 5);
        assert!(!loader.is_empty());

        // 清理所有字体
        // 注意：没有直接的卸载方法，这是测试真实场景
    }

    /// 测试不同字符的 advance 宽度
    #[test]
    fn test_font_loader_rasterize_advance_width() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load");

        // 空格字符
        let space = loader.rasterize_glyph(font_id, ' ', 16.0).unwrap();
        assert!(space.advance >= 0.0);
        assert!(space.width == 0 || space.advance > 0.0);

        // 宽字符 'W' vs 窄字符 'i'
        let w = loader.rasterize_glyph(font_id, 'W', 16.0).unwrap();
        let i = loader.rasterize_glyph(font_id, 'i', 16.0).unwrap();

        // 'W' 通常比 'i' 宽
        assert!(w.advance >= i.advance);
    }

    /// 测试字体查找功能
    #[test]
    fn test_font_loader_find_by_family() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let _font_id = loader.load_font(&font_data).expect("should load");

        // 测试查找
        let desc = FontDesc::normal("Arial"); // 可能不匹配，但测试 API
        let found_id = loader.find(&desc);

        // 由于 get_font_family_name 总是返回 None，find 总是返回 None
        assert!(found_id.is_none());
    }

    /// 测试字体 ID 重用（通过重复加载模拟）
    #[test]
    fn test_font_loader_id_sequence() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();

        // 加载多个字体验证 ID 分配
        let ids: Vec<u32> = (0..10)
            .map(|_| loader.load_font(&font_data).expect("should load"))
            .collect();

        // 验证 ID 是连续的
        for i in 0..10 {
            assert_eq!(ids[i], i as u32);
        }

        // 验证不重复
        let unique_ids: std::collections::HashSet<u32> = ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), 10);
    }

    /// 测试 rasterize_glyph 对无效 font_id 的处理
    #[test]
    fn test_font_loader_invalid_font_id() {
        let loader = FontLoader::new();

        // 尝试获取不存在的字体
        let result = loader.get(999999);
        assert!(result.is_none());

        // 尝试渲染不存在的字体
        let result = loader.rasterize_glyph(999999, 'A', 16.0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::font::FontError::NotFound(_)));
    }

    /// 测试字体加载器的空状态
    #[test]
    fn test_font_loader_state_operations() {
        let loader = FontLoader::new();

        // 初始状态
        assert!(loader.is_empty());
        assert_eq!(loader.len(), 0);

        // 验证无 font_id 的行为
        assert!(loader.get(0).is_none());
        assert!(loader.find(&FontDesc::normal("Test")).is_none());
    }

    /// 测试字体描述符的等价性
    #[test]
    fn test_font_desc_equality() {
        let desc1 = FontDesc::normal("Arial");
        let desc2 = FontDesc {
            family: "Arial".to_string(),
            weight: 400,
            italic: false,
        };
        assert_eq!(desc1, desc2);

        let desc3 = FontDesc::bold("Arial");
        assert_ne!(desc1, desc3);

        let desc4 = FontDesc::italic("Arial");
        assert_ne!(desc1, desc4);
    }

    /// 测试字体描述符的字符串表示
    #[test]
    fn test_font_desc_string_display() {
        let desc = FontDesc::bold("Arial");
        assert_eq!(desc.family, "Arial");
        assert_eq!(desc.weight, 700);
        assert!(!desc.italic);

        let desc = FontDesc::italic("Times New Roman");
        assert_eq!(desc.family, "Times New Roman");
        assert_eq!(desc.weight, 400);
        assert!(desc.italic);
    }

    /// 测试字体加载器的容量处理
    #[test]
    fn test_font_loader_large_dataset() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();

        // 加载多个副本来测试大量数据处理
        for i in 0..20 {
            let font_id = loader.load_font(&font_data).expect("should load");
            assert_eq!(font_id, i);

            // 验证每个字体都能正常渲染
            let result = loader.rasterize_glyph(font_id, 'A', 16.0);
            assert!(result.is_ok());
        }

        assert_eq!(loader.len(), 20);
        assert!(!loader.is_empty());
    }

    /// 测试字体加载器的边界条件
    #[test]
    fn test_font_loader_edge_cases() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();

        // 测试极大字体尺寸
        let font_id = loader.load_font(&font_data).expect("should load");
        let result = loader.rasterize_glyph(font_id, 'A', 1000.0);
        assert!(result.is_ok());

        // 测试极小字体尺寸
        let result = loader.rasterize_glyph(font_id, 'A', 1.0);
        assert!(result.is_ok());
    }

    /// 测试 fallback 跳过主字体的 .notdef 方块
    #[test]
    fn test_fallback_skips_primary_missing_glyph() {
        let primary_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };
        let cjk_path = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc";
        let cjk_data = match std::fs::read(cjk_path) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("skipping: no NotoSansCJK at {cjk_path}");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let primary = loader.load_font(&primary_data).unwrap();
        let cjk = loader.load_font(&cjk_data).unwrap();
        loader.set_fallback_chain(vec![cjk]);

        let primary_font = loader.get(primary).unwrap();
        assert!(!primary_font.has_glyph('中'));

        let (resolved, _) = loader.rasterize_glyph_with_fallback(primary, '中', 20.0).unwrap();
        assert_eq!(resolved, cjk);
    }

    /// 测试字体描述符的权重转换
    #[test]
    fn test_font_desc_weight_conversions() {
        // 测试标准权重
        let normal = FontDesc::normal("Test");
        assert_eq!(normal.weight, 400);

        let bold = FontDesc::bold("Test");
        assert_eq!(bold.weight, 700);

        // 测试自定义权重
        let custom = FontDesc::new("Test", 300, false);
        assert_eq!(custom.weight, 300);

        let custom_bold = FontDesc::new("Test", 800, true);
        assert_eq!(custom_bold.weight, 800);
        assert!(custom_bold.italic);
    }
}
