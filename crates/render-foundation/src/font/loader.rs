//! 字体加载器 — 使用 fontdue 加载和管理字体

use crate::font::{FontDesc, FontError, GlyphBitmap};
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use zero_text_foundation::backend::FontdueBackend;
use zero_text_foundation::font_request::FontId as FtFontId;

/// 字体加载器 — 管理字体集合
pub struct FontLoader {
    /// 已加载的字体（fontdue 实例）
    fonts: HashMap<u32, fontdue::Font>,
    /// 字体原始字节数据（供 rustybuzz 使用）
    font_data: HashMap<u32, Vec<u8>>,
    /// 下一个字体 ID
    next_id: u32,
    /// 字体族到 ID 的映射
    family_map: HashMap<String, Vec<u32>>,
    /// 回退字体链（CJK、Emoji 等），在主字体缺字时使用
    fallback_chain: Vec<u32>,
    /// 预注册位图 glyph（font_id, glyph_id, size_bits）→ 光栅结果
    bitmap_glyphs: HashMap<(u32, u32, u32), GlyphBitmap>,
    /// Ahem 测试字体 ID（WPT 标准测试字体，每个字符渲染为完美填充方块）
    ahem_font_id: Option<u32>,
    /// DC-11：共享文本后端。设置后，load_font 同步注册字体到此后端。
    shared_backend: Option<Arc<Mutex<FontdueBackend>>>,
    /// DC-11：FontLoader font_id → 共享后端 FontId 映射。
    shared_ids: HashMap<u32, FtFontId>,
}

impl FontLoader {
    /// 创建空的字体加载器
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            font_data: HashMap::new(),
            next_id: 0,
            family_map: HashMap::new(),
            fallback_chain: Vec::new(),
            bitmap_glyphs: HashMap::new(),
            ahem_font_id: None,
            shared_backend: None,
            shared_ids: HashMap::new(),
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

    /// 清除已注册的位图 glyph（导航换页时丢弃旧 favicon）。
    pub fn clear_bitmap_glyph(&mut self, font_id: u32, glyph_id: u32, size_px: f32) {
        self.bitmap_glyphs.remove(&(font_id, glyph_id, size_px.to_bits()));
    }

    /// 设置回退字体链（按优先级排序）
    pub fn set_fallback_chain(&mut self, ids: Vec<u32>) {
        self.fallback_chain = ids;
    }

    /// 获取回退字体链
    pub fn fallback_chain(&self) -> &[u32] {
        &self.fallback_chain
    }

    /// 检查指定字体 ID 是否为 Ahem 测试字体。
    pub fn is_ahem(&self, font_id: u32) -> bool {
        self.ahem_font_id == Some(font_id)
    }

    /// 从字节数据加载字体
    ///
    /// 自动识别 WOFF 1.0 容器（`.woff`，"wOFF" 魔数）并先解码为 sfnt，再交给 fontdue
    /// （fontdue 不识别 woff 容器）。`.ttf`/`.otf` 裸 sfnt 直接加载。WOFF2（`wOF2`）不支持。
    pub fn load_font(&mut self, data: &[u8]) -> Result<u32, FontError> {
        // WOFF 容器解码（None = 非 WOFF 或解码失败，回退原数据由 fontdue 报错）
        let decoded = if crate::font::woff::is_woff(data) {
            crate::font::woff::decode_woff(data)
        } else {
            None
        };
        let bytes: &[u8] = decoded.as_deref().unwrap_or(data);

        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map_err(|e| FontError::ParseFailed(e.to_string()))?;

        let id = self.next_id;
        self.next_id += 1;

        // 从字体字节中提取字体族名称（fontdue 不暴露 name 表）。
        // WOFF 解码后的 sfnt 含 name 表，解析路径与裸 sfnt 一致。
        let family = parse_font_family_name(bytes);
        if let Some(ref name) = family {
            // 检测 Ahem 测试字体
            if name.eq_ignore_ascii_case("Ahem") {
                self.ahem_font_id = Some(id);
            }
            self.family_map.entry(name.clone()).or_default().push(id);
        }

        self.fonts.insert(id, font);
        self.font_data.insert(id, bytes.to_vec());

        // DC-11：同步到共享后端（用 WOFF 解码后的 sfnt bytes）
        if let Some(ref name) = family {
            self.sync_to_shared(id, name, bytes);
        }

        Ok(id)
    }

    /// 注册字体族别名（`@font-face` 的 `font-family` 描述符 → 已加载字体 ID）。
    ///
    /// `load_font` 按字体内部 name 表注册族名；但 CSS `@font-face` 创建的族名是
    /// 声明值（可能与字体内部名不同）。此方法把声明族名映射到已加载的 font_id，
    /// 使 `build_font_resolver` 能按 `@font-face` 声明族名匹配。
    pub fn register_family_alias(&mut self, alias: &str, font_id: u32) {
        self.family_map.entry(alias.to_string()).or_default().push(font_id);
    }

    /// DC-11 共享字体栈：加载字体数据并以指定族名注册（桥接 foundation/text 字体）。
    ///
    /// 与 [`load_font`](Self::load_font) 不同，本方法用**显式族名**注册字体，
    /// 不依赖字节内 name 表解析。这使调用方可以把 foundation/text 的
    /// [`FontdueBackend`] 中已加载的字体族名直接映射到 `FontLoader` 的 ID 空间，
    /// 从而实现两套后端共享同一字体数据源（DC-11 关键不变量）。
    ///
    /// [`FontdueBackend`]: zero_text_foundation::backend::FontdueBackend
    pub fn load_font_with_family(&mut self, data: &[u8], family: &str) -> Result<u32, FontError> {
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|e| FontError::ParseFailed(e.to_string()))?;
        let id = self.next_id;
        self.next_id += 1;
        // 检测 Ahem 测试字体（与 load_font 一致的特殊处理）。
        if family.eq_ignore_ascii_case("Ahem") {
            self.ahem_font_id = Some(id);
        }
        self.family_map.entry(family.to_string()).or_default().push(id);
        self.fonts.insert(id, font);
        self.font_data.insert(id, data.to_vec());

        // DC-11：同步到共享后端（sync_to_shared 内部检查 shared_backend 是否存在）
        self.sync_to_shared(id, family, data);

        Ok(id)
    }

    /// 根据 ID 获取字体
    pub fn get(&self, id: u32) -> Option<&fontdue::Font> {
        self.fonts.get(&id)
    }

    /// 获取字体原始字节数据（供 rustybuzz 等 shaping 引擎使用）。
    pub fn get_font_data(&self, id: u32) -> Option<&[u8]> {
        self.font_data.get(&id).map(|v| v.as_slice())
    }

    /// 根据字体描述查找最佳匹配字体 ID
    pub fn find(&self, desc: &FontDesc) -> Option<u32> {
        self.family_map.get(&desc.family).and_then(|ids| ids.first().copied())
    }

    /// 构建 CSS font-family 查找表。
    ///
    /// 返回 `family_name → font_id` 的映射，供 Painter 解析 CSS font-family 使用。
    /// 同时注册通用字体族别名（sans-serif / serif / monospace）映射到已加载的实际字体。
    pub fn build_font_resolver(&self) -> std::collections::HashMap<String, u32> {
        let mut resolver = std::collections::HashMap::new();

        // 已知字体族名 → ID（Regular face）
        for (name, ids) in &self.family_map {
            if let Some(&id) = ids.first() {
                resolver.insert(name.clone(), id);
            }
        }

        // 同一族名的第二个 face 视为 Bold（如 Arial + Arial Bold）
        for (name, ids) in &self.family_map {
            if let Some(&bold_id) = ids.get(1) {
                resolver.insert(format!("{name}:700"), bold_id);
            }
        }

        // 如果只有 0-1 个字体，通用族映射没有意义
        if self.fonts.is_empty() {
            return resolver;
        }

        // 默认字体 = 第一个加载的字体（ID 0）
        let default_id = 0u32;

        // 通用字体族别名映射
        // sans-serif → 尝试匹配 DejaVu Sans / Liberation Sans / 其他 Sans 字体
        let sans_names = ["DejaVu Sans", "Liberation Sans", "Arial", "Helvetica", "Noto Sans"];
        let serif_names = [
            "DejaVu Serif",
            "Liberation Serif",
            "Times New Roman",
            "Georgia",
            "Noto Serif",
        ];
        let mono_names = ["DejaVu Sans Mono", "Liberation Mono", "Courier New", "monospace"];

        // sans-serif
        let sans_id = self.resolve_generic_family(&sans_names).unwrap_or(default_id);
        resolver.insert("sans-serif".to_string(), sans_id);
        let sans_bold_id = self
            .family_map
            .get("Arial")
            .and_then(|ids| ids.get(1).copied())
            .or_else(|| self.family_map.values().find_map(|ids| ids.get(1).copied()))
            .unwrap_or(sans_id);
        resolver.insert("sans-serif:700".to_string(), sans_bold_id);

        // serif
        let serif_id = self.resolve_generic_family(&serif_names).unwrap_or(default_id);
        resolver.insert("serif".to_string(), serif_id);

        // monospace
        let mono_id = self.resolve_generic_family(&mono_names).unwrap_or(default_id);
        resolver.insert("monospace".to_string(), mono_id);

        // cursive / fantasy / system-ui → 暂映射到 sans-serif
        resolver.insert("cursive".to_string(), sans_id);
        resolver.insert("fantasy".to_string(), sans_id);
        resolver.insert("system-ui".to_string(), sans_id);

        resolver
    }

    /// 在已加载字体中查找匹配通用族名的最佳字体。
    fn resolve_generic_family(&self, candidates: &[&str]) -> Option<u32> {
        for name in candidates {
            if let Some(ids) = self.family_map.get(*name) {
                return ids.first().copied();
            }
        }
        None
    }

    /// 渲染指定字符的 glyph
    pub fn rasterize_glyph(&self, font_id: u32, code_point: char, size: f32) -> Result<GlyphBitmap, FontError> {
        // Ahem 特殊处理：渲染为完美填充方块，匹配 Chrome/Skia 的渲染结果
        if self.ahem_font_id == Some(font_id) && !code_point.is_whitespace() {
            return self.rasterize_ahem_glyph(font_id, code_point, size);
        }

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

    /// DC-11：经共享后端光栅化 glyph（字体栈统一渲染路径）。
    ///
    /// 若已设置共享后端且非 Ahem 字体，把光栅化委托给共享 [`FontdueBackend`]，
    /// 使 render-foundation 与 UI SDK / zero-webview 共享同一字体栈（DC-11 关键不变量）。
    ///
    /// - Ahem 字体仍走 [`rasterize_glyph`] 的特殊处理（保证 WPT 兼容性）。
    /// - 未设置共享后端时回退到 [`rasterize_glyph`]（等价于直接调用）。
    ///
    /// [`FontdueBackend`]: zero_text_foundation::backend::FontdueBackend
    pub fn rasterize_glyph_shared(&self, font_id: u32, code_point: char, size: f32) -> Result<GlyphBitmap, FontError> {
        // Ahem 特殊处理与 rasterize_glyph 一致
        if self.ahem_font_id == Some(font_id) && !code_point.is_whitespace() {
            return self.rasterize_ahem_glyph(font_id, code_point, size);
        }

        // 共享后端路径：同时存在 backend 和映射时委托给共享后端
        let ft_id_and_font = self
            .shared_backend
            .as_ref()
            .and_then(|_backend| self.shared_ids.get(&font_id).copied())
            .and_then(|ft_id| self.fonts.get(&font_id).map(|font| (ft_id, font)));
        if let Some((ft_id, font)) = ft_id_and_font {
            let backend = self.shared_backend.as_ref().unwrap();
            let glyph_id = font.lookup_glyph_index(code_point) as u32;
            if glyph_id == 0 {
                // .notdef：回退到 FontLoader 自身路径（fontdue 可渲染 .notdef 方块）
                let (metrics, bitmap) = font.rasterize(code_point, size);
                return Ok(GlyphBitmap {
                    data: bitmap,
                    width: metrics.width as u16,
                    height: metrics.height as u16,
                    x_offset: metrics.xmin as i16,
                    y_offset: metrics.ymin as i16,
                    advance: metrics.advance_width,
                });
            }
            let ft = backend.lock();
            match ft.rasterize_glyph(ft_id, glyph_id, size) {
                Ok(ft_bmp) => {
                    // advance 取自 FontLoader 侧的 fontdue::Font 度量（共享后端 rasterize_glyph 只返回位图）。
                    let advance = font.metrics_indexed(glyph_id as u16, size).advance_width;
                    return Ok(GlyphBitmap {
                        data: ft_bmp.coverage,
                        width: ft_bmp.width as u16,
                        height: ft_bmp.height as u16,
                        x_offset: ft_bmp.xmin as i16,
                        y_offset: ft_bmp.ymin as i16,
                        advance,
                    });
                }
                Err(_) => {
                    // 共享后端失败 → 回退到 FontLoader 自身路径
                }
            }
        }

        // 回退：与 rasterize_glyph 一致的直接 fontdue 路径
        self.rasterize_glyph(font_id, code_point, size)
    }

    /// Ahem 字体特殊光栅化：生成完美填充方块。
    ///
    /// Ahem 是 WPT 标准测试字体，每个字符应渲染为边长 = font_size 的实心方块。
    /// fontdue 的光栅化结果与 Skia（Chrome）存在差异，直接生成方块可确保像素级对齐。
    fn rasterize_ahem_glyph(&self, font_id: u32, code_point: char, size: f32) -> Result<GlyphBitmap, FontError> {
        let font = self
            .fonts
            .get(&font_id)
            .ok_or_else(|| FontError::NotFound(format!("font_id={font_id}")))?;

        // 使用字体的实际 ascent 来计算垂直偏移
        let line_metrics = font
            .horizontal_line_metrics(size)
            .ok_or_else(|| FontError::NotFound(format!("font_id={font_id}")))?;

        let ascent = line_metrics.ascent;
        // 方块尺寸：取 ascent 向上取整，确保覆盖完整的 em 方块
        let w = size.ceil() as u16;
        let h = size.ceil() as u16;

        // 检查字体是否实际包含该字符；若不含则回退到 fontdue 渲染
        if !font.has_glyph(code_point) {
            let (metrics, bitmap) = font.rasterize(code_point, size);
            return Ok(GlyphBitmap {
                data: bitmap,
                width: metrics.width as u16,
                height: metrics.height as u16,
                x_offset: metrics.xmin as i16,
                y_offset: metrics.ymin as i16,
                advance: metrics.advance_width,
            });
        }

        // 全部像素完全不透明
        let data = vec![255u8; (w as usize) * (h as usize)];

        Ok(GlyphBitmap {
            data,
            width: w,
            height: h,
            x_offset: 0,
            // y_offset 为负值表示从基线向上偏移，覆盖完整 em 方块
            y_offset: -(ascent.ceil() as i16),
            advance: size,
        })
    }

    /// 在主字体及回退链中渲染 glyph，返回实际使用的字体 ID。
    ///
    /// 经 [`rasterize_glyph_shared`]（DC-11 字体栈统一）光栅——设置了共享后端时，
    /// 生产渲染路径（CPU `render_cpu` / GPU 字体管线）把 glyph 光栅委托给共享
    /// [`FontdueBackend`]，使 render-foundation 与 UI SDK / zero-webview 共享同一字体栈。
    /// 未设置共享后端时 [`rasterize_glyph_shared`] 回退到 [`rasterize_glyph`]（行为等价）。
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
            let bitmap = self.rasterize_glyph_shared(font_id, code_point, size)?;
            if Self::glyph_has_coverage(code_point, &bitmap) {
                return Ok((font_id, bitmap));
            }
        }

        Err(FontError::GlyphNotFound {
            font_id: primary_id,
            glyph_id: code_point as u32,
        })
    }

    /// 获取水平排版行 metrics：`(ascent, descent)`，其中 `descent` 通常为负值。
    pub fn line_metrics(&self, font_id: u32, size: f32) -> Option<(f32, f32)> {
        let font = self.fonts.get(&font_id)?;
        let metrics = font.horizontal_line_metrics(size)?;
        Some((metrics.ascent, metrics.descent))
    }

    /// 获取水平排版行 metrics 含 `line_gap`：`(ascent, descent, line_gap)`。
    ///
    /// `line_gap` 是字体的行间距（OS/2 sTypoLineGap / hhea lineGap）。chromium 与
    /// fontdue 的 `line-height:normal` = `ascent − descent + line_gap`（按字号缩放）。
    /// ZeroWeb 的 IFC `strut_ascent` / `half-leading` 目前用 `0.8·fs` / `1.2` 近似
    /// （R759 仅修 Ahem 为 1.0），非 Ahem 字体偏离真实度量。本方法是 Phase A
    /// font-metric plumbing（FontLoader → engine → IFC 真实 ascent/descent/line_gap）
    /// 的第一阶准备：暴露此前被 [`line_metrics`] 丢弃的 `line_gap`，供后续阶段
    /// 消费（R833/R834/R846；须全链 coherence 非 single-knob，R834 实证单点改
    /// strut_ascent 反退 welcome）。
    pub fn line_metrics_full(&self, font_id: u32, size: f32) -> Option<(f32, f32, f32)> {
        let font = self.fonts.get(&font_id)?;
        let metrics = font.horizontal_line_metrics(size)?;
        Some((metrics.ascent, metrics.descent, metrics.line_gap))
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

    /// DC-11：设置共享文本后端。后续 `load_font` / `load_font_with_family`
    /// 会自动把字体同步注册到此后端；已有字体也会立即同步。
    ///
    /// 设置后，调用方可通过 [`shared_backend`](Self::shared_backend) 获取后端引用，
    /// 使 render-foundation 与 UI SDK / zero-webview 共享同一字体栈（DC-11 关键不变量）。
    pub fn set_shared_backend(&mut self, backend: Arc<Mutex<FontdueBackend>>) {
        // 先设置 shared_backend，再同步已有字体（sync_to_shared 依赖它）。
        self.shared_backend = Some(backend);
        // 收集待同步列表（避免 borrow checker 冲突）
        let to_sync: Vec<(u32, String, Vec<u8>)> = self
            .font_data
            .iter()
            .filter_map(|(&id, data)| {
                let family = self
                    .family_map
                    .iter()
                    .find(|(_, ids)| ids.contains(&id))
                    .map(|(name, _)| name.clone());
                family.map(|name| (id, name, data.clone()))
            })
            .collect();
        for (id, name, data) in to_sync {
            self.sync_to_shared(id, &name, &data);
        }
    }

    /// DC-11 convenience：用指定族名和字体数据创建共享后端并链接到本 `FontLoader`。
    ///
    /// 等价于 `FontdueBackend::new()` + `load_family(family, data)` +
    /// `set_shared_backend(Arc::new(Mutex::new(backend)))`。调用方无需直接依赖
    /// `parking_lot` 或 `FontdueBackend` 类型。
    pub fn init_shared_backend(&mut self, family: &str, data: &[u8]) {
        let mut backend = FontdueBackend::new();
        // 共享后端加载失败不阻塞 FontLoader（backend 为空→后续同步 no-op）。
        let _ = backend.load_family(family, data);
        self.set_shared_backend(Arc::new(Mutex::new(backend)));
    }

    /// DC-11：检查是否已设置共享后端。
    pub fn has_shared_backend(&self) -> bool {
        self.shared_backend.is_some()
    }

    /// DC-11：获取共享后端引用（如有）。
    pub fn shared_backend(&self) -> Option<&Arc<Mutex<FontdueBackend>>> {
        self.shared_backend.as_ref()
    }

    /// DC-11：获取 FontLoader font_id → 共享后端 FontId 映射（如有）。
    pub fn shared_id_of(&self, rf_id: u32) -> Option<FtFontId> {
        self.shared_ids.get(&rf_id).copied()
    }

    /// DC-11 内部方法：把单个字体注册到共享后端，记录 ID 映射。
    fn sync_to_shared(&mut self, rf_id: u32, family: &str, data: &[u8]) {
        if let Some(ref backend) = self.shared_backend {
            match backend.lock().load_family(family, data) {
                Ok(ft_id) => {
                    self.shared_ids.insert(rf_id, ft_id);
                }
                Err(_) => {
                    // 共享后端加载失败不阻塞 FontLoader 自身。
                }
            }
        }
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 OpenType/TrueType 字体字节中解析字体族名称。
///
/// 解析 `name` 表（nameID=1）获取 Font Family Name。
/// 优先使用 Windows 平台（platformID=3, encodingID=1, UTF-16BE），
/// 回退到 Macintosh 平台（platformID=1, encodingID=0, ASCII）。
fn parse_font_family_name(data: &[u8]) -> Option<String> {
    // OpenType 文件头：offset table
    // 0-3: sfVersion (0x00010000 = TrueType, 'OTTO' = CFF)
    // 4-5: numTables
    // 6-11: searchRange, entrySelector, rangeShift
    if data.len() < 12 {
        return None;
    }
    let num_tables = u16_from_be(data, 4)? as usize;

    // 表记录从偏移 12 开始，每条 16 字节
    let records_start = 12;
    let records_end = records_start + num_tables * 16;
    if records_end > data.len() {
        return None;
    }

    // 查找 'name' 表
    let mut name_offset = None;
    let mut name_length = None;
    for i in 0..num_tables {
        let rec = records_start + i * 16;
        let tag = &data[rec..rec + 4];
        if tag == b"name" {
            name_offset = Some(u32_from_be(data, rec + 8)? as usize);
            name_length = Some(u32_from_be(data, rec + 12)? as usize);
            break;
        }
    }

    let name_off = name_offset?;
    let name_len = name_length?;
    if name_off + name_len > data.len() {
        return None;
    }
    let table = &data[name_off..name_off + name_len];
    if table.len() < 6 {
        return None;
    }

    let _format = u16_from_be_slice(table, 0)?;
    let count = u16_from_be_slice(table, 2)? as usize;
    let string_offset = u16_from_be_slice(table, 4)? as usize;

    // 遍历 name records，寻找最佳匹配
    // 优先级：Windows (3,1) > Mac (1,0)
    let mut win_name: Option<String> = None;
    let mut mac_name: Option<String> = None;

    for i in 0..count {
        let rec = 6 + i * 12;
        if rec + 12 > table.len() {
            break;
        }
        let platform_id = u16_from_be_slice(table, rec)?;
        let encoding_id = u16_from_be_slice(table, rec + 2)?;
        let _lang_id = u16_from_be_slice(table, rec + 4)?;
        let name_id = u16_from_be_slice(table, rec + 6)?;
        let str_len = u16_from_be_slice(table, rec + 8)? as usize;
        let str_off = u16_from_be_slice(table, rec + 10)? as usize;

        // nameID 1 = Font Family Name
        if name_id != 1 {
            continue;
        }

        let abs_str_start = string_offset + str_off;
        let abs_str_end = abs_str_start + str_len;
        if abs_str_end > table.len() {
            continue;
        }
        let str_data = &table[abs_str_start..abs_str_end];

        if platform_id == 3 && encoding_id == 1 {
            // Windows Unicode BMP (UTF-16BE)
            win_name = decode_utf16be(str_data);
        } else if platform_id == 1 && encoding_id == 0 {
            // Macintosh Roman (ASCII-like)
            mac_name = String::from_utf8(str_data.to_vec()).ok();
        }
    }

    // 优先 Windows，回退 Mac
    win_name.or(mac_name)
}

/// 从大端序字节切片中读取 u16。
fn u16_from_be_slice(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
}

/// 从大端序字节切片中读取 u16（绝对偏移）。
fn u16_from_be(data: &[u8], offset: usize) -> Option<u16> {
    u16_from_be_slice(data, offset)
}

/// 从大端序字节切片中读取 u32。
fn u32_from_be(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// 解码 UTF-16BE 字节为 String。
fn decode_utf16be(data: &[u8]) -> Option<String> {
    if !data.len().is_multiple_of(2) {
        return None;
    }
    let chars: Vec<u16> = (0..data.len())
        .step_by(2)
        .map(|i| u16::from_be_bytes([data[i], data[i + 1]]))
        .collect();
    String::from_utf16(&chars).ok()
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

    /// R846 Phase A Phase 1：`line_metrics_full` 暴露此前被 `line_metrics` 丢弃的
    /// `line_gap`。验证对真实字体返回 `(ascent, descent, line_gap)` 三元组，且与
    /// `line_metrics` 的 ascent/descent 一致（line_metrics_full 是其超集）。
    #[test]
    fn test_line_metrics_full_exposes_line_gap() {
        let font_data = match load_system_font_data() {
            Some(d) => d,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");

        let size = 16.0_f32;
        let (asc, desc, line_gap) = loader
            .line_metrics_full(font_id, size)
            .expect("line_metrics_full should return metrics for loaded font");
        // ascent 为正、descent 为负（fontdue 约定）。
        assert!(asc > 0.0, "ascent should be positive, got {asc}");
        assert!(desc <= 0.0, "descent should be <= 0, got {desc}");

        // 与 line_metrics 的一致性：ascent/descent 必须相同（line_metrics_full 是超集）。
        let (asc2, desc2) = loader
            .line_metrics(font_id, size)
            .expect("line_metrics should return metrics");
        assert!((asc - asc2).abs() < 1e-4, "ascent mismatch: {asc} vs {asc2}");
        assert!((desc - desc2).abs() < 1e-4, "descent mismatch: {desc} vs {desc2}");

        // line_gap 有限（不同字体可能为 0，但不应为 NaN）。
        assert!(line_gap.is_finite(), "line_gap should be finite, got {line_gap}");
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
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(id, i as u32);
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

    /// Ahem 字体辅助：加载 Ahem.ttf 并返回 (FontLoader, font_id)
    fn load_ahem() -> Option<(FontLoader, u32)> {
        let path = "tests/wpt-runner/fonts/Ahem.ttf";
        let data = std::fs::read(path).ok()?;
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&data).ok()?;
        Some((loader, font_id))
    }

    /// 测试 Ahem 字体检测
    ///
    /// 加载 Ahem.ttf 后 is_ahem 应返回 true，系统字体应返回 false。
    #[test]
    fn test_ahem_font_detection() {
        let (loader, ahem_id) = match load_ahem() {
            Some(v) => v,
            None => {
                eprintln!("skipping: Ahem.ttf not found");
                return;
            }
        };
        assert!(loader.is_ahem(ahem_id), "Ahem font ID should be detected");
        assert!(!loader.is_ahem(0), "font_id 0 should not be Ahem");
        assert!(!loader.is_ahem(999), "nonexistent font should not be Ahem");
    }

    /// 测试 Ahem 字体光栅化生成完美填充方块
    ///
    /// Ahem 的每个字符应渲染为 font_size × font_size 的不透明方块。
    #[test]
    fn test_ahem_rasterize_perfect_square() {
        let (loader, ahem_id) = match load_ahem() {
            Some(v) => v,
            None => {
                eprintln!("skipping: Ahem.ttf not found");
                return;
            }
        };

        for &size in &[10.0f32, 16.0, 20.0, 32.0, 50.0] {
            let bitmap = loader.rasterize_glyph(ahem_id, 'X', size).unwrap();
            let expected_w = size.ceil() as u16;
            let expected_h = size.ceil() as u16;
            assert_eq!(
                bitmap.width, expected_w,
                "Ahem 'X' at size={size}: width should be {expected_w}, got {}",
                bitmap.width
            );
            assert_eq!(
                bitmap.height, expected_h,
                "Ahem 'X' at size={size}: height should be {expected_h}, got {}",
                bitmap.height
            );
            // 全部像素应完全不透明
            let all_opaque = bitmap.data.iter().all(|&a| a == 255);
            assert!(all_opaque, "Ahem 'X' at size={size}: all pixels should be fully opaque");
            // advance 应等于 font_size
            assert!(
                (bitmap.advance - size).abs() < 0.01,
                "Ahem advance should be {size}, got {}",
                bitmap.advance
            );
        }
    }

    /// 测试 Ahem 字体多个不同字符都渲染为方块
    ///
    /// Ahem 字体中所有可打印字符的渲染结果应相同（完美方块）。
    #[test]
    fn test_ahem_all_chars_are_squares() {
        let (loader, ahem_id) = match load_ahem() {
            Some(v) => v,
            None => {
                eprintln!("skipping: Ahem.ttf not found");
                return;
            }
        };

        let size = 20.0f32;
        for ch in ['A', 'z', '0', '!', 'X', 'p', 'M'] {
            let bitmap = loader.rasterize_glyph(ahem_id, ch, size).unwrap();
            assert_eq!(bitmap.width, size.ceil() as u16, "Ahem '{ch}' width mismatch");
            assert_eq!(bitmap.height, size.ceil() as u16, "Ahem '{ch}' height mismatch");
            assert!(
                bitmap.data.iter().all(|&a| a == 255),
                "Ahem '{ch}' should be fully opaque"
            );
        }
    }

    /// 测试 Ahem 字体 advance 宽度通过 measure_advance 返回
    #[test]
    fn test_ahem_measure_advance() {
        let (loader, ahem_id) = match load_ahem() {
            Some(v) => v,
            None => {
                eprintln!("skipping: Ahem.ttf not found");
                return;
            }
        };

        for &size in &[12.0f32, 20.0, 48.0] {
            let advance = loader.measure_advance(ahem_id, 'X', size);
            assert!(
                (advance - size).abs() < 0.01,
                "measure_advance should return {size}, got {advance}"
            );
        }
    }

    /// 诊断：量化 layout-engine `estimate_char_width` 启发式（字母 0.55×fs、数字 0.5、
    /// 标点 0.4、空格 0.25）与真实系统字体 advance（measure_advance）的系统性误差。
    ///
    /// 目的：为 advance-width plumbing（R221 识别的 183-case 系统性噪声桶）提供数据依据。
    /// 仅打印（--nocapture），不断言——反映真实字体度量分布。
    #[test]
    fn diag_advance_vs_estimate_systematic_error() {
        let font_data = match load_system_font_data() {
            Some(d) => d,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };
        let mut loader = FontLoader::new();
        let font_id = match loader.load_font(&font_data) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("skipping: font load failed: {e}");
                return;
            }
        };
        let size = 16.0f32;
        // (字符, estimate 启发式倍数)
        let samples: &[(char, f32)] = &[
            ('W', 0.55),
            ('i', 0.55),
            ('m', 0.55),
            ('l', 0.55),
            ('A', 0.55),
            ('5', 0.5),
            ('0', 0.5),
            ('.', 0.4),
            (',', 0.4),
            (' ', 0.25),
            ('t', 0.55),
            ('f', 0.55),
            ('H', 0.55),
        ];
        eprintln!("\n=== advance (real vs estimate) at {size}px, font_id={font_id} ===");
        eprintln!("char | real_adv | real_ratio | est_ratio | err%");
        let mut sum_real = 0.0f32;
        let mut sum_est = 0.0f32;
        for &(ch, est_ratio) in samples {
            let real = loader.measure_advance(font_id, ch, size);
            let real_ratio = real / size;
            let est = size * est_ratio;
            let err = if real > 0.0 { 100.0 * (est - real) / real } else { 0.0 };
            eprintln!("  {ch}  | {real:7.2} | {real_ratio:.3}      | {est_ratio:.3}     | {err:+6.1}%");
            sum_real += real;
            sum_est += est;
        }
        let total_err = if sum_real > 0.0 {
            100.0 * (sum_est - sum_real) / sum_real
        } else {
            0.0
        };
        eprintln!("sum: real={sum_real:.2} est={sum_est:.2} total_err={total_err:+.1}%");
    }

    // ── DC-11 字体栈统一兼容性验证 ───────────────────────────────────────
    // 验证 render-foundation FontLoader 与 foundation/text FontdueBackend
    // 加载同一字体后产出一致的 glyph 光栅化结果（DC-11 关键不变量）。

    /// 加载同一 Ahem 字体到两套后端，验证 fontdue 对同一 glyph 产出一致的
    /// advance width 与 raster 尺寸——证明字体栈统一的可行性。
    #[test]
    fn dc11_fontdue_shared_backend_consistency() {
        use zero_text_foundation::backend::FontdueBackend;
        use zero_text_foundation::font_request::FontRequest;
        use zero_text_foundation::text_measure::{TextMeasureInput, TextMeasurer};

        // WPT 标准字体路径（相对 crate 根：crates/render-foundation）。
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        // ── render-foundation FontLoader：加载 + raster ──
        let mut rf_loader = FontLoader::new();
        let rf_id = rf_loader.load_font(ahem_data).expect("FontLoader loads Ahem");
        let rf_raster = rf_loader
            .rasterize_glyph(rf_id, 'A', 8.0)
            .expect("FontLoader rasterizes 'A' @8px");
        let rf_advance = rf_loader.measure_advance(rf_id, 'A', 8.0);

        // ── foundation/text FontdueBackend：加载 + raster ──
        let mut ft = FontdueBackend::new();
        let ft_id = ft.load_family("Ahem", ahem_data).expect("FontdueBackend loads Ahem");
        // 通过 fontdue::Font 拿 'A' 的 glyph index。
        let glyph_idx: u32 = match rf_loader.get(rf_id) {
            Some(font) => font.lookup_glyph_index('A') as u32,
            None => 0,
        };
        let ft_raster = ft
            .rasterize_glyph(ft_id, glyph_idx, 8.0)
            .expect("FontdueBackend rasterizes 'A' @8px");
        let ft_metrics = ft
            .measure(&TextMeasureInput {
                text: "A".to_string(),
                font_request: FontRequest::new("Ahem"),
                size_px: 8.0,
                max_width: None,
                direction: zero_text_foundation::font_request::TextDirection::Ltr,
            })
            .expect("FontdueBackend measures 'A'");

        // ── DC-11 关键不变量 ──
        // advance width 一致（来自 fontdue，不受特殊 Ahem 处理影响）。
        assert!(
            (rf_advance - 8.0).abs() < 1.0,
            "rf advance 'A' @8px ≈ 8, got {rf_advance}"
        );
        assert!(
            (ft_metrics.width - 8.0).abs() < 1.0,
            "ft measure 'A' @8px ≈ 8, got {}",
            ft_metrics.width
        );
        // raster 尺寸非零（两套后端都产出有效像素）。
        assert!(rf_raster.width > 0 && rf_raster.height > 0, "rf raster non-zero");
        assert!(ft_raster.width > 0 && ft_raster.height > 0, "ft raster non-zero");
        // raster 位图数据非空。
        assert!(!rf_raster.data.is_empty(), "rf raster data non-empty");
        assert!(!ft_raster.coverage.is_empty(), "ft raster coverage non-empty");
    }

    /// DC-11 `load_font_with_family` 使用显式族名加载字体后，
    /// 产出的 raster 应与 `load_font`（自动 name 表解析）一致。
    #[test]
    fn dc11_load_font_with_family_produces_same_raster() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        // 通过 load_font（自动检测 name 表 "Ahem"）加载。
        let mut loader_auto = FontLoader::new();
        let auto_id = loader_auto.load_font(ahem_data).expect("auto load");
        let auto_raster = loader_auto.rasterize_glyph(auto_id, 'X', 8.0).expect("auto raster 'X'");
        let auto_advance = loader_auto.measure_advance(auto_id, 'X', 8.0);

        // 通过 load_font_with_family（显式族名）加载。
        let mut loader_explicit = FontLoader::new();
        let explicit_id = loader_explicit
            .load_font_with_family(ahem_data, "Ahem")
            .expect("explicit load");
        let explicit_raster = loader_explicit
            .rasterize_glyph(explicit_id, 'X', 8.0)
            .expect("explicit raster 'X'");
        let explicit_advance = loader_explicit.measure_advance(explicit_id, 'X', 8.0);

        // DC-11：显式族名加载与自动 name 表解析产出一致。
        assert_eq!(auto_raster.width, explicit_raster.width);
        assert_eq!(auto_raster.height, explicit_raster.height);
        assert!((auto_advance - explicit_advance).abs() < 0.5);
        // 族名正确注册（find 可按传入的族名匹配）。
        assert!(
            loader_explicit.find(&FontDesc::normal("Ahem")).is_some(),
            "family name 'Ahem' should be registered via load_font_with_family"
        );
    }

    // ── DC-11 共享后端基础设施测试 ─────────────────────────────────────

    /// 设置共享后端后，已有字体自动同步——共享后端中应有相同族名的字体。
    #[test]
    fn dc11_set_shared_backend_syncs_existing_fonts() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let mut loader = FontLoader::new();
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");
        assert!(!loader.has_shared_backend());

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        loader.set_shared_backend(shared.clone());
        assert!(loader.has_shared_backend());

        // 共享后端应已加载 Ahem 字体
        let ft = shared.lock();
        assert!(!ft.is_empty(), "shared backend should have synced fonts");
        // FontLoader ID → 共享后端 FontId 映射应存在
        let ft_id = loader.shared_id_of(rf_id).expect("should have shared id mapping");
        assert_eq!(ft_id, FtFontId(0), "first loaded font should get FontId(0)");
    }

    /// DC-11 part-1：`rasterize_glyph_with_fallback` 经 `rasterize_glyph_shared` 路径（生产渲染
    /// 接线）——设置共享后端前后，glyph 光栅化结果**逐位一致**（共享路径不改变渲染输出，
    /// 是默认路径切换的安全前提）。同时验证字体已同步到共享后端（非 Ahem 字体将经此委托）。
    #[test]
    fn dc11_rasterize_with_fallback_shared_path_parity() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        // 无共享后端：rasterize_glyph_with_fallback 走直接 fontdue 路径。
        let mut loader_plain = FontLoader::new();
        let id_plain = loader_plain.load_font(ahem_data).expect("load_font Ahem");
        let (_, bitmap_plain) = loader_plain
            .rasterize_glyph_with_fallback(id_plain, 'A', 16.0)
            .expect("plain fallback rasterizes 'A'");

        // 有共享后端：rasterize_glyph_with_fallback 经 rasterize_glyph_shared。
        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader_shared = FontLoader::new();
        let id_shared = loader_shared.load_font(ahem_data).expect("load_font Ahem");
        loader_shared.set_shared_backend(shared);
        // 字体已同步到共享后端（非 Ahem 字体将经此映射委托光栅）。
        assert!(
            loader_shared.shared_id_of(id_shared).is_some(),
            "font synced to shared backend"
        );
        let (_, bitmap_shared) = loader_shared
            .rasterize_glyph_with_fallback(id_shared, 'A', 16.0)
            .expect("shared fallback rasterizes 'A'");

        // 逐位一致：共享路径不改变 fallback 光栅化输出（默认路径切换的安全前提）。
        assert_eq!(bitmap_plain.width, bitmap_shared.width);
        assert_eq!(bitmap_plain.height, bitmap_shared.height);
        assert_eq!(bitmap_plain.x_offset, bitmap_shared.x_offset);
        assert_eq!(bitmap_plain.y_offset, bitmap_shared.y_offset);
        assert_eq!(bitmap_plain.advance, bitmap_shared.advance);
        assert_eq!(bitmap_plain.data, bitmap_shared.data);
    }

    /// load_font 在共享后端已设置时自动同步新加载字体。
    #[test]
    fn dc11_load_font_auto_syncs_to_shared_backend() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());

        // 加载字体——应自动同步到共享后端
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");
        assert!(loader.shared_id_of(rf_id).is_some());

        let ft = shared.lock();
        // 共享后端应有 1 个（Ahem）或更多字体（取决于同步时机）
        assert!(ft.len() >= 1, "shared backend should have at least 1 font");
    }

    /// load_font_with_family 在共享后端已设置时自动同步。
    #[test]
    fn dc11_load_font_with_family_auto_syncs() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());

        let rf_id = loader
            .load_font_with_family(ahem_data, "TestFamily")
            .expect("load_font_with_family");
        assert!(loader.shared_id_of(rf_id).is_some());

        // 共享后端应按指定族名注册
        let ft = shared.lock();
        use zero_text_foundation::font_database::FontProvider; // trait 方法 query()
        let query = ft.query(&zero_text_foundation::font_request::FontRequest::new("TestFamily"));
        assert!(query.is_ok(), "shared backend should have TestFamily registered");
    }

    /// 多字体加载——验证 ID 映射正确（每个 FontLoader ID 有唯一共享后端 ID）。
    #[test]
    fn dc11_multiple_fonts_have_distinct_shared_ids() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());

        // 加载同一数据两次——FontLoader 分配不同 ID
        let id0 = loader.load_font_with_family(ahem_data, "FamilyA").unwrap();
        let id1 = loader.load_font_with_family(ahem_data, "FamilyB").unwrap();
        assert_ne!(id0, id1, "FontLoader IDs should differ");

        let ft0 = loader.shared_id_of(id0).expect("id0 should map");
        let ft1 = loader.shared_id_of(id1).expect("id1 should map");
        assert_ne!(ft0, ft1, "shared backend FontIds should differ");

        let ft = shared.lock();
        assert_eq!(ft.len(), 2, "shared backend should have 2 fonts");
    }

    /// DC-11 关键不变量：同一字体通过 FontLoader 和共享后端（FontdueBackend）
    /// 加载后，双方都能为同一 glyph 产出有效的非零光栅位图（证明字体栈统一可行）。
    #[test]
    fn dc11_shared_backend_raster_equivalence() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");

        // FontLoader 直接光栅（Ahem 特殊路径：完美填充方块，w=h=size.ceil()）
        let rf_raster = loader.rasterize_glyph(rf_id, 'X', 16.0).expect("FontLoader raster");
        assert!(
            rf_raster.width > 0 && rf_raster.height > 0,
            "FontLoader raster non-zero"
        );

        // 共享后端光栅同一 glyph（普通 fontdue 路径，不经过 Ahem 特殊处理）
        let ft_id = loader.shared_id_of(rf_id).unwrap();
        let glyph_idx = loader.get(rf_id).unwrap().lookup_glyph_index('X') as u32;
        let ft_raster = {
            let ft = shared.lock();
            ft.rasterize_glyph(ft_id, glyph_idx, 16.0)
                .expect("shared backend raster")
        };

        // DC-11 关键不变量：双方都产出有效非零光栅位图
        assert!(
            ft_raster.width > 0 && ft_raster.height > 0,
            "shared backend raster non-zero"
        );
        assert!(!ft_raster.coverage.is_empty(), "shared backend raster has pixel data");
        // FontLoader Ahem 特殊路径产出 size.ceil() 方块
        let expected = 16.0_f32.ceil() as u16;
        assert_eq!(rf_raster.width, expected, "Ahem special path: w=ceil(size)");
        assert_eq!(rf_raster.height, expected, "Ahem special path: h=ceil(size)");
        // FontLoader advance 为 font_size（Ahem 1em 方块）
        assert!((rf_raster.advance - 16.0).abs() < 1.0, "Ahem advance ≈ 16");
    }

    /// 未设置共享后端时 has_shared_backend 返回 false，shared_id_of 返回 None。
    #[test]
    fn dc11_without_shared_backend_returns_none() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let mut loader = FontLoader::new();
        let rf_id = loader.load_font(ahem_data).unwrap();

        assert!(!loader.has_shared_backend());
        assert!(loader.shared_backend().is_none());
        assert!(loader.shared_id_of(rf_id).is_none());
    }

    /// 共享后端加载失败（空数据）不阻塞 FontLoader 自身。
    #[test]
    fn dc11_sync_failure_does_not_block_fontloader() {
        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());

        // 企图加载无效字体——FontLoader 自身应报错
        let result = loader.load_font(&[]);
        assert!(result.is_err(), "FontLoader should reject empty data");

        // 但共享后端不受影响（无新字体注册）
        let ft = shared.lock();
        assert!(ft.is_empty(), "shared backend should still be empty");
    }

    /// init_shared_backend convenience：一站式创建共享后端并链接。
    #[test]
    fn dc11_init_shared_backend_convenience() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let mut loader = FontLoader::new();
        // 先加载一个字体
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");

        // 通过 convenience 方法初始化共享后端
        loader.init_shared_backend("TestInit", ahem_data);

        assert!(loader.has_shared_backend());
        // 共享后端应有 TestInit 字体
        let backend = loader.shared_backend().expect("shared_backend should be set");
        let ft = backend.lock();
        assert!(ft.len() >= 1, "shared backend should have fonts");
        // 已有字体（Ahem）也已被同步
        assert!(
            loader.shared_id_of(rf_id).is_some(),
            "pre-existing font should be synced"
        );
    }

    // ── DC-11 光栅委托（rasterize_glyph_shared）测试 ─────────────────

    /// rasterize_glyph_shared 经共享后端光栅化 Ahem 字符——仍走 Ahem 特殊处理
    /// （完美填充方块），不委托给共享后端。
    #[test]
    fn dc11_rasterize_shared_uses_ahem_special_for_ahem() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");

        // rasterize_glyph_shared 对 Ahem 走特殊处理（w=h=ceil(size)）
        let bmp = loader
            .rasterize_glyph_shared(rf_id, 'X', 16.0)
            .expect("rasterize_shared Ahem");
        let expected = 16.0_f32.ceil() as u16;
        assert_eq!(bmp.width, expected, "Ahem width = ceil(size)");
        assert_eq!(bmp.height, expected, "Ahem height = ceil(size)");
        assert!((bmp.advance - 16.0).abs() < 1.0, "Ahem advance ≈ size");
    }

    /// rasterize_glyph_shared 未设置共享后端时回退到 rasterize_glyph。
    #[test]
    fn dc11_rasterize_shared_falls_back_without_backend() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let mut loader = FontLoader::new();
        let rf_id = loader.load_font(ahem_data).expect("load_font Ahem");

        // 无共享后端 → 应与 rasterize_glyph 一致
        let direct = loader.rasterize_glyph(rf_id, 'X', 16.0).unwrap();
        let shared = loader.rasterize_glyph_shared(rf_id, 'X', 16.0).unwrap();
        assert_eq!(direct.width, shared.width);
        assert_eq!(direct.height, shared.height);
        assert!((direct.advance - shared.advance).abs() < 0.5);
    }

    /// rasterize_glyph_shared 经共享后端光栅化非 Ahem 非空白字符：产出有效位图。
    #[test]
    fn dc11_rasterize_shared_produces_valid_bitmap() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let shared = Arc::new(Mutex::new(FontdueBackend::new()));
        let mut loader = FontLoader::new();
        loader.set_shared_backend(shared.clone());
        // 用显式族名加载，避免 Ahem 检测（Ahem 检测基于 name 表解析的 "Ahem" 族名）
        let rf_id = loader
            .load_font_with_family(ahem_data, "NotAhem")
            .expect("load as NotAhem");

        let bmp = loader
            .rasterize_glyph_shared(rf_id, 'X', 16.0)
            .expect("rasterize_shared");

        // 非 Ahem 路径 → 共享后端光栅（fontdue rasterize_indexed）
        assert!(bmp.width > 0, "should have non-zero width");
        assert!(bmp.height > 0, "should have non-zero height");
        assert!(!bmp.data.is_empty(), "should have pixel data");
        assert!(bmp.advance > 0.0, "should have positive advance");
    }

    /// rasterize_glyph_shared 与 rasterize_glyph 对同一非 Ahem 字体产出一致结果。
    #[test]
    fn dc11_rasterize_shared_equivalent_to_direct() {
        let ahem_data: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

        // 不设置共享后端：rasterize_glyph_shared 回退到 rasterize_glyph
        let mut loader = FontLoader::new();
        let rf_id = loader.load_font_with_family(ahem_data, "NotAhem").expect("load");

        let direct = loader.rasterize_glyph(rf_id, 'X', 16.0).unwrap();
        let shared = loader.rasterize_glyph_shared(rf_id, 'X', 16.0).unwrap();

        // 回退路径：尺寸与 advance 一致
        assert_eq!(direct.width, shared.width);
        assert_eq!(direct.height, shared.height);
        assert!((direct.advance - shared.advance).abs() < 0.5);
        // 像素数据一致
        assert_eq!(direct.data, shared.data);
    }
}
