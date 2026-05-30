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
        self.family_map
            .get(&desc.family)
            .and_then(|ids| ids.first().copied())
    }

    /// 渲染指定字符的 glyph
    pub fn rasterize_glyph(
        &self,
        font_id: u32,
        code_point: char,
        size: f32,
    ) -> Result<GlyphBitmap, FontError> {
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
}
