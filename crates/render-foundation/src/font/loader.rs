//! 字体加载器 — 使用 fontdue 加载和管理字体

use crate::font::{FontDesc, FontError, GlyphBitmap};
use hashbrown::{HashMap, HashSet};
use shaping::ShapeCache;
use std::sync::Arc;

mod metrics;
mod shaping;
mod unicode_range;

/// FreeType 光栅化路径（`freetype-raster` feature，默认关）。
///
/// Phase 2（fontdue→chromium-matching 光栅化替换）的实验通道：用 FreeType
/// `FT_Render_Glyph`（chromium Linux 同栈）替换 fontdue tight-ink 光栅化。
/// feature 关时整个模块不编译，CI / 默认构建保持纯 Rust。
///
/// GlyphBitmap 坐标约定：paint 经 `glyph_top_left(x, baseline_y, x_offset, y_offset, height)`
/// = `(x + x_offset, baseline_y − y_offset − height)` 定位位图左上角。FreeType
/// `bitmap_left`（pen→位图左缘 px）`bitmap_top`（baseline→位图顶 px，向上正）映射：
/// 位图顶 = baseline − bitmap_top = baseline − y_offset − height ⇒ **y_offset = bitmap_top − height**。
#[cfg(feature = "freetype-raster")]
mod freetype_raster {
    use crate::font::{FontError, GlyphBitmap};
    use std::cell::{OnceCell, RefCell};
    use std::collections::HashMap;

    enum GlyphSelector {
        CodePoint(char),
        GlyphIndex(u16),
    }

    thread_local! {
        static FT_LIB: OnceCell<freetype::Library> = const { OnceCell::new() };
        // face 缓存（2026-08-07 CJK 优化）：Face<Rc<Vec<u8>>> 自含字体字节，
        // 一次解析（19MB TTC ~6.6ms）后复用——此前每次 new_memory_face2
        // 重新解析整字体是 CJK 栅格化重尾根因（探针实测 6.6ms/字 → 目标 <0.1ms）。
        // 容量上限 8（按字体字节 hash 键），超限清空重建（低频字体轮换场景）。
        static FT_FACE_CACHE: RefCell<HashMap<u64, freetype::Face<Vec<u8>>>> = RefCell::new(HashMap::new());
        // 栅格化统计（env CJK_RASTER_STAT=1 时打印；诊断用）
        static RASTER_STAT: RefCell<(u64, f64)> = const { RefCell::new((0, 0.0)) };
    }

    const FACE_CACHE_MAX: usize = 8;

    /// 采样 FNV-1a 哈希（缓存键；O(1)）。
    ///
    /// 2026-08-07 探针实测：全量遍历 19MB CJK TTC 在 debug 下 ~50ms/次
    ///（CJK 栅格化重尾主因之一）——改为**前 4KB + 总长**采样（不同字体的
    /// 头部表数据几乎必然不同，冲突概率可忽略；每次调用成本 ~微秒）。
    fn bytes_hash(bytes: &[u8], face_index: u32) -> u64 {
        let window = &bytes[..bytes.len().min(4096)];
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in window {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= bytes.len() as u64;
        h ^= u64::from(face_index) << 32;
        h
    }

    /// 在线程局部 FreeType Library 上跑一次闭包（懒初始化）。
    fn with_lib<R>(f: impl FnOnce(&freetype::Library) -> R) -> R {
        FT_LIB.with(|cell| {
            let lib = cell.get_or_init(|| freetype::Library::init().expect("FreeType Library::init failed"));
            f(lib)
        })
    }

    /// 用 FreeType 光栅化单字形 → GlyphBitmap（与 fontdue 路径同坐标约定）。
    ///
    /// `font_bytes`：字体 sfnt 字节（来自 FontLoader.font_data）。`size`：字号 px。
    /// 失败（字形缺失 / FreeType 错误）由调用方回退 fontdue。
    ///
    /// 性能（2026-08-07 探针，NotoSansCJK 19MB）：face 缓存（RefCell borrow
    /// 贯穿，免 clone——Face<Vec<u8>> clone 会复制 19MB）后每次栅格化不再
    /// 重新解析字体（6.6ms/字 → 目标 <0.1ms/字）。
    pub(crate) fn rasterize(
        font_bytes: &[u8],
        face_index: u32,
        code_point: char,
        size: f32,
    ) -> Result<GlyphBitmap, FontError> {
        if size <= 0.0 {
            return Err(FontError::NotFound(format!("non-positive size {size}")));
        }
        let _raster_t = std::time::Instant::now();
        let result = rasterize_inner(font_bytes, face_index, GlyphSelector::CodePoint(code_point), size);
        // OnceLock 缓存 env 状态（避免每字一次 std::env::var——热路径）
        static RASTER_STAT_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let raster_stat_enabled = *RASTER_STAT_ENABLED.get_or_init(|| std::env::var("CJK_RASTER_STAT").is_ok());
        if raster_stat_enabled {
            RASTER_STAT.with(|s| {
                let mut s = s.borrow_mut();
                s.0 += 1;
                s.1 += _raster_t.elapsed().as_secs_f64() * 1000.0;
                if s.0 % 50 == 0 {
                    eprintln!(
                        "[raster-stat] count={} total={:.0}ms avg={:.3}ms",
                        s.0,
                        s.1,
                        s.1 / s.0 as f64
                    );
                }
            });
        }
        result
    }

    /// 用字体内部 glyph index 光栅化整形后的字形。
    pub(crate) fn rasterize_indexed(
        font_bytes: &[u8],
        face_index: u32,
        glyph_index: u16,
        size: f32,
    ) -> Result<GlyphBitmap, FontError> {
        if size <= 0.0 {
            return Err(FontError::NotFound(format!("non-positive size {size}")));
        }
        rasterize_inner(font_bytes, face_index, GlyphSelector::GlyphIndex(glyph_index), size)
    }

    fn rasterize_inner(
        font_bytes: &[u8],
        face_index: u32,
        selector: GlyphSelector,
        size: f32,
    ) -> Result<GlyphBitmap, FontError> {
        // thread_local with 不允许借用逃逸：cache borrow 与 face 使用都在闭包内
        FT_FACE_CACHE.with(|cache_cell| {
            let mut cache = cache_cell.borrow_mut();
            with_lib(|lib| {
                let key = bytes_hash(font_bytes, face_index);
                if !cache.contains_key(&key) {
                    if cache.len() >= FACE_CACHE_MAX {
                        cache.clear(); // 低频字体轮换：清空重建
                    }
                    let face = lib
                        .new_memory_face2(font_bytes.to_vec(), isize::try_from(face_index).unwrap_or(0))
                        .map_err(|e| FontError::ParseFailed(format!("FreeType new_memory_face: {e:?}")))?;
                    cache.insert(key, face);
                }
                // face 方法均为 &self：RefCell borrow 贯穿，免 clone
                let face = cache.get(&key).expect("face 已插入");
                face.set_char_size((size * 64.0) as isize, (size * 64.0) as isize, 0, 0)
                    .map_err(|e| FontError::ParseFailed(format!("FreeType set_char_size: {e:?}")))?;
                let idx = match selector {
                    GlyphSelector::CodePoint(code_point) => face
                        .get_char_index(code_point as usize)
                        .ok_or_else(|| FontError::NotFound(format!("no glyph index for {code_point:?}")))?,
                    GlyphSelector::GlyphIndex(glyph_index) => glyph_index as u32,
                };
                // LoadFlag::DEFAULT（含 TARGET_NORMAL = full hinting）。R1069 A/B 实测（css-text
                // Oracle 1650 案）：DEFAULT 381 pass > LIGHT(TARGET_LIGHT) 371 > NO_HINTING 357 ≈
                // fontdue 基线。NOHINT==fontdue 证 fontdue tight-ink 即 unhinted，FreeType full
                // hinting 向 chromium（hinted）收敛——故 DEFAULT 为最优，勿改 LIGHT/NOHINT。
                face.load_glyph(idx, freetype::face::LoadFlag::DEFAULT)
                    .map_err(|e| FontError::ParseFailed(format!("FreeType load_glyph: {e:?}")))?;
                let glyph = face.glyph();
                glyph
                    .render_glyph(freetype::RenderMode::Normal)
                    .map_err(|e| FontError::ParseFailed(format!("FreeType render_glyph: {e:?}")))?;
                let bitmap = glyph.bitmap();
                let width = bitmap.width().max(0) as u16;
                let height = bitmap.rows().max(0) as u16;
                let pitch = bitmap.pitch().unsigned_abs() as usize;
                // 灰度位图按行拷贝到紧凑 width×height 缓冲（pitch 可 ≥ width）。
                let mut data = vec![0u8; width as usize * height as usize];
                if width > 0 && height > 0 && pitch > 0 {
                    let src = bitmap.buffer();
                    let copy_w = (width as usize).min(pitch).min(src.len());
                    for y in 0..height as usize {
                        let src_off = y * pitch;
                        if src_off + copy_w <= src.len() {
                            let dst_off = y * width as usize;
                            data[dst_off..dst_off + copy_w].copy_from_slice(&src[src_off..src_off + copy_w]);
                        }
                    }
                }
                let x_offset = glyph.bitmap_left() as i16;
                let top = glyph.bitmap_top();
                // y_offset = bitmap_top − height（见模块注释坐标推导）。
                let y_offset = (top - height as i32) as i16;
                let advance = glyph.advance().x as f64 / 64.0;
                Ok(GlyphBitmap {
                    data,
                    width,
                    height,
                    x_offset,
                    y_offset,
                    advance: advance as f32,
                })
            })
        })
    }

    /// 轻量测量 advance（不产位图）——与 [`rasterize`] 完全同源：同一 face
    /// 缓存，`load_glyph`（FT_Load_Glyph 即计算 advance 字段，render 只产
    /// 位图）后直接读 advance，返回 FreeType 26.6 定点量化值。
    ///
    /// 布局测量与绘制光栅化必须用同源值，否则宽度漂移（fontdue metrics 是
    /// 浮点，与 FreeType 定点有约 0.5% 差异，会破坏 reftest 像素对比）。
    pub(crate) fn measure_advance(
        font_bytes: &[u8],
        face_index: u32,
        code_point: char,
        size: f32,
    ) -> Result<f32, FontError> {
        if size <= 0.0 {
            return Err(FontError::NotFound(format!("non-positive size {size}")));
        }
        FT_FACE_CACHE.with(|cache_cell| {
            let mut cache = cache_cell.borrow_mut();
            with_lib(|lib| {
                let key = bytes_hash(font_bytes, face_index);
                if !cache.contains_key(&key) {
                    if cache.len() >= FACE_CACHE_MAX {
                        cache.clear(); // 低频字体轮换：清空重建
                    }
                    let face = lib
                        .new_memory_face2(font_bytes.to_vec(), isize::try_from(face_index).unwrap_or(0))
                        .map_err(|e| FontError::ParseFailed(format!("FreeType new_memory_face: {e:?}")))?;
                    cache.insert(key, face);
                }
                // face 方法均为 &self：RefCell borrow 贯穿，免 clone
                let face = cache.get(&key).expect("face 已插入");
                face.set_char_size((size * 64.0) as isize, (size * 64.0) as isize, 0, 0)
                    .map_err(|e| FontError::ParseFailed(format!("FreeType set_char_size: {e:?}")))?;
                let idx = face
                    .get_char_index(code_point as usize)
                    .ok_or_else(|| FontError::NotFound(format!("no glyph index for {code_point:?}")))?;
                face.load_glyph(idx, freetype::face::LoadFlag::DEFAULT)
                    .map_err(|e| FontError::ParseFailed(format!("FreeType load_glyph: {e:?}")))?;
                Ok((face.glyph().advance().x as f64 / 64.0) as f32)
            })
        })
    }
}

/// 字体加载器 — 管理字体集合
pub struct FontLoader {
    /// 已加载的字体（fontdue 实例；Arc 共享使 `duplicate` 免深拷贝 19MB CJK 解析结果）
    fonts: HashMap<u32, Arc<fontdue::Font>>,
    /// 字体原始字节数据（供 rustybuzz / freetype-raster 使用；Arc 共享同上）
    font_data: HashMap<u32, Arc<Vec<u8>>>,
    /// TTC/OTC collection face index；普通单字体文件为 0。
    face_indices: HashMap<u32, u32>,
    /// 下一个字体 ID
    next_id: u32,
    /// 字体 ID 对应的进程唯一实例 ID；duplicate 保留，后续新字体重新分配。
    font_instance_ids: HashMap<u32, u64>,
    /// `@font-face font-feature-settings` 提供的 face 级默认 feature。
    font_features: HashMap<u32, Vec<crate::font::OpenTypeFeature>>,
    /// `@font-face font-variation-settings` 提供的 face 级默认 axis 坐标。
    font_variations: HashMap<u32, Vec<crate::font::OpenTypeVariation>>,
    /// `@font-face unicode-range` 提供的 face 级闭区间；缺项表示 unrestricted。
    font_unicode_ranges: HashMap<u32, Vec<(u32, u32)>>,
    /// `@font-face size-adjust` 提供的 face 级字号缩放因子。
    font_size_adjustments: HashMap<u32, f32>,
    /// 字体族到 ID 的映射
    family_map: HashMap<String, Vec<u32>>,
    /// 由 `@font-face` 声明注册的 alias 名称。
    family_aliases: HashSet<String>,
    /// 回退字体链（CJK、Emoji 等），在主字体缺字时使用
    fallback_chain: Vec<u32>,
    /// 预注册位图 glyph（font_id, glyph_id, size_bits）→ 光栅结果
    bitmap_glyphs: HashMap<(u32, u32, u32), GlyphBitmap>,
    /// Ahem 测试字体 ID（WPT 标准测试字体，每个字符渲染为完美填充方块）
    ahem_font_id: Option<u32>,
    /// 有界 shaping 缓存（font_instance_id, size_bits, direction, features, text）。
    ///
    /// duplicate 共享 base 字体缓存；后续 `@font-face` 即使复用相同 font_id，也因
    /// instance ID 不同而隔离。
    shape_cache: ShapeCache,
}

impl FontLoader {
    /// 创建空的字体加载器
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            font_data: HashMap::new(),
            face_indices: HashMap::new(),
            next_id: 0,
            font_instance_ids: HashMap::new(),
            font_features: HashMap::new(),
            font_variations: HashMap::new(),
            font_unicode_ranges: HashMap::new(),
            font_size_adjustments: HashMap::new(),
            family_map: HashMap::new(),
            family_aliases: HashSet::new(),
            fallback_chain: Vec::new(),
            bitmap_glyphs: HashMap::new(),
            ahem_font_id: None,
            shape_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// 深拷贝全部已加载字体（含已解析的 fontdue 轮廓数据）。
    ///
    /// 用途：复用已解析的 base 字体集（系统 + CJK + Ahem）构造 fresh loader 追加
    /// `@font-face` 自定义字体——避免对 19MB CJK 字体重复 `from_bytes` 解析
    ///（~0.5s/次，reftest harness 每 distinct 键一次）。字体数据 Arc 共享
    ///（引用计数 + 元数据映射拷贝），比重新解析快两个量级。
    pub fn duplicate(&self) -> Self {
        Self {
            fonts: self.fonts.clone(),
            font_data: self.font_data.clone(),
            face_indices: self.face_indices.clone(),
            next_id: self.next_id,
            font_instance_ids: self.font_instance_ids.clone(),
            font_features: self.font_features.clone(),
            font_variations: self.font_variations.clone(),
            font_unicode_ranges: self.font_unicode_ranges.clone(),
            font_size_adjustments: self.font_size_adjustments.clone(),
            family_map: self.family_map.clone(),
            family_aliases: self.family_aliases.clone(),
            fallback_chain: self.fallback_chain.clone(),
            bitmap_glyphs: self.bitmap_glyphs.clone(),
            ahem_font_id: self.ahem_font_id,
            shape_cache: self.shape_cache.clone(),
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
        self.shape_cache.lock().expect("shape cache poisoned").clear();
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
    /// 自动识别 WOFF / WOFF2 容器并先解码为 sfnt，再交给 fontdue。
    /// `.ttf` / `.otf` 裸 sfnt 直接加载。
    pub fn load_font(&mut self, data: &[u8]) -> Result<u32, FontError> {
        self.load_font_at_index(data, 0)
    }

    /// 从 TTC/OTC collection 的指定 face 加载字体。
    pub fn load_font_at_index(&mut self, data: &[u8], face_index: u32) -> Result<u32, FontError> {
        // Decode supported webfont containers. Decode failure falls through to fontdue,
        // which returns the existing ParseFailed error at the trust boundary.
        let decoded = if crate::font::woff::is_woff(data) {
            crate::font::woff::decode_woff(data)
        } else if crate::font::woff::is_woff2(data) && std::env::var("ZW_WOFF2").as_deref() != Ok("0") {
            crate::font::woff::decode_woff2(data)
        } else {
            None
        };
        let bytes: &[u8] = decoded.as_deref().unwrap_or(data);

        let font = fontdue::Font::from_bytes(
            bytes,
            fontdue::FontSettings {
                collection_index: face_index,
                ..fontdue::FontSettings::default()
            },
        )
        .map_err(|e| FontError::ParseFailed(e.to_string()))?;

        let id = self.next_id;
        self.next_id += 1;
        static NEXT_FONT_INSTANCE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let instance_id = NEXT_FONT_INSTANCE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.font_instance_ids.insert(id, instance_id);

        // 从字体字节中提取字体族名称（fontdue 不暴露 name 表）。
        // WOFF 解码后的 sfnt 含 name 表，解析路径与裸 sfnt 一致。
        if let Some(name) = parse_font_family_name(bytes) {
            // 检测 Ahem 测试字体
            if name.eq_ignore_ascii_case("Ahem") {
                self.ahem_font_id = Some(id);
            }
            self.family_map.entry(name).or_default().push(id);
        }

        self.fonts.insert(id, Arc::new(font));
        self.font_data.insert(id, Arc::new(bytes.to_vec()));
        self.face_indices.insert(id, face_index);
        Ok(id)
    }

    /// 注册字体族别名（`@font-face` 的 `font-family` 描述符 → 已加载字体 ID）。
    ///
    /// `load_font` 按字体内部 name 表注册族名；但 CSS `@font-face` 创建的族名是
    /// 声明值（可能与字体内部名不同）。此方法把声明族名映射到已加载的 font_id，
    /// 使 `build_font_resolver` 能按 `@font-face` 声明族名匹配。
    pub fn register_family_alias(&mut self, alias: &str, font_id: u32) {
        self.family_map.entry(alias.to_string()).or_default().push(font_id);
        self.family_aliases.insert(alias.to_string());
    }

    /// 根据 ID 获取字体
    pub fn get(&self, id: u32) -> Option<&fontdue::Font> {
        self.fonts.get(&id).map(|f| f.as_ref())
    }

    /// 获取字体原始字节数据（供 rustybuzz 等 shaping 引擎使用）。
    pub fn get_font_data(&self, id: u32) -> Option<&[u8]> {
        self.font_data.get(&id).map(|v| v.as_ref().as_slice())
    }

    /// 返回字体实例在 TTC/OTC collection 中的 face index。
    pub fn face_index(&self, id: u32) -> u32 {
        self.face_indices.get(&id).copied().unwrap_or(0)
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
        let expose_alias_faces = std::env::var("ZW_FONT_UNICODE_RANGE_FACES").as_deref() != Ok("0");

        // 已知字体族名 → ID（Regular face）
        for (name, ids) in &self.family_map {
            if let Some(&id) = ids.first() {
                resolver.insert(name.clone(), id);
            }
            if expose_alias_faces && self.family_aliases.contains(name) {
                for (index, &id) in ids.iter().enumerate() {
                    resolver.insert(format!("{name}:face={index}"), id);
                }
            }
        }

        // 同一族名的第二个 face 视为 Bold（如 Arial + Arial Bold）
        for (name, ids) in &self.family_map {
            if !self.family_aliases.contains(name)
                && let Some(&bold_id) = ids.get(1)
            {
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
        // R1263：Liberation Sans 优先——chromium 的 sans-serif 默认 "Arial" → fontconfig
        // → Liberation Sans（同 initial/serif → "Times New Roman" → LiberationSerif 谱系，R1259）。
        // 旧序 DejaVu Sans 首位致 sans-serif→DejaVuSans（≠ CHR Liberation Sans）font-wall
        //（welcome.html 用 sans-serif 故受影响）。R631 测 NotoSansCJK（fontconfig "sans-serif" 错）
        // zero change 被 fallback 失效；LiberationSans（"Arial" 真实匹配）是正确字体。
        let sans_names = ["Liberation Sans", "DejaVu Sans", "Arial", "Helvetica", "Noto Sans"];
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

        // Phase 2（freetype-raster feature）：非-Ahem 字形优先 FreeType 光栅化
        //（chromium Linux 同栈），失败回退 fontdue。feature 关时不编译，走纯 fontdue。
        #[cfg(feature = "freetype-raster")]
        if let Some(bytes) = self.font_data.get(&font_id)
            && let Ok(bm) = freetype_raster::rasterize(bytes, self.face_index(font_id), code_point, size)
        {
            return Ok(bm);
        }
        // FreeType 失败（字形缺失等）→ 回退 fontdue 路径（下方 fontdue 代码）。

        let font = self
            .fonts
            .get(&font_id)
            .map(|f| f.as_ref())
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

    /// 按字体内部 OpenType glyph index 光栅化整形后的字形。
    ///
    /// 本路径不做字体回退：glyph index 只在产生它的字体 face 内有意义，调用方
    /// 必须同时携带对应的 `font_id`。
    pub fn rasterize_glyph_index(&self, font_id: u32, glyph_index: u16, size: f32) -> Result<GlyphBitmap, FontError> {
        if let Some(bytes) = self.font_data.get(&font_id)
            && let Ok(face) = rustybuzz::ttf_parser::Face::parse(bytes, self.face_index(font_id))
            && glyph_index >= face.number_of_glyphs()
        {
            return Err(FontError::GlyphNotFound {
                font_id,
                glyph_id: u32::from(glyph_index),
            });
        }

        #[cfg(feature = "freetype-raster")]
        if let Some(bytes) = self.font_data.get(&font_id)
            && let Ok(bitmap) = freetype_raster::rasterize_indexed(bytes, self.face_index(font_id), glyph_index, size)
        {
            return Ok(bitmap);
        }

        let font = self
            .fonts
            .get(&font_id)
            .map(|font| font.as_ref())
            .ok_or_else(|| FontError::NotFound(format!("font_id={font_id}")))?;
        let (metrics, bitmap) = font.rasterize_indexed(glyph_index, size);
        Ok(GlyphBitmap {
            data: bitmap,
            width: metrics.width as u16,
            height: metrics.height as u16,
            x_offset: metrics.xmin as i16,
            y_offset: metrics.ymin as i16,
            advance: metrics.advance_width,
        })
    }

    /// Ahem 字体特殊光栅化：生成完美填充方块。
    ///
    /// Ahem 是 WPT 标准测试字体，每个字符应渲染为边长 = font_size 的实心方块。
    /// fontdue 的光栅化结果与 Skia（Chrome）存在差异，直接生成方块可确保像素级对齐。
    fn rasterize_ahem_glyph(&self, font_id: u32, code_point: char, size: f32) -> Result<GlyphBitmap, FontError> {
        let font = self
            .fonts
            .get(&font_id)
            .map(|f| f.as_ref())
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

    /// 构建字形查找链：primary + 去重后的回退链（primary 缺失字形时按序尝试）。
    fn lookup_chain(&self, primary_id: u32) -> Vec<u32> {
        let mut chain = Vec::with_capacity(1 + self.fallback_chain.len());
        chain.push(primary_id);
        for &id in &self.fallback_chain {
            if id != primary_id && !chain.contains(&id) {
                chain.push(id);
            }
        }
        chain
    }

    /// 按有序 CSS face 列表与系统回退链解析字符字体，不触发光栅化。
    /// 在预构建的 fallback 链上解析字符字体（shaper 批量扫描复用同一 chain，
    /// 避免逐 grapheme 重建去重——中文长文逐字扫描下 O(graphemes × chain) 分配）。
    pub(crate) fn resolve_font_for_code_point_in_chain(&self, chain: &[u32], code_point: char) -> Option<u32> {
        chain.iter().copied().find(|font_id| {
            self.font_allows_code_point(*font_id, code_point)
                && self
                    .fonts
                    .get(font_id)
                    .is_some_and(|font| code_point.is_whitespace() || font.has_glyph(code_point))
        })
    }

    /// 在主字体及回退链中渲染 glyph，返回实际使用的字体 ID
    pub fn rasterize_glyph_with_fallback(
        &self,
        primary_id: u32,
        code_point: char,
        size: f32,
    ) -> Result<(u32, GlyphBitmap), FontError> {
        if self.font_allows_code_point(primary_id, code_point)
            && let Some(bitmap) = self.bitmap_glyphs.get(&(primary_id, code_point as u32, size.to_bits()))
        {
            return Ok((primary_id, bitmap.clone()));
        }

        let chain = self.lookup_chain(primary_id);

        for font_id in chain {
            if !self.font_allows_code_point(font_id, code_point) {
                continue;
            }
            let Some(font) = self.fonts.get(&font_id).map(|f| f.as_ref()) else {
                continue;
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

    /// 获取水平排版行 metrics：`(ascent, descent)`，其中 `descent` 通常为负值。
    pub fn line_metrics(&self, font_id: u32, size: f32) -> Option<(f32, f32)> {
        let font = self.fonts.get(&font_id).map(|f| f.as_ref())?;
        let metrics = font.horizontal_line_metrics(size)?;
        Some((metrics.ascent, metrics.descent))
    }

    /// 获取水平排版行 metrics 含 `line_gap`：`(ascent, descent, line_gap)`。
    ///
    /// `line_gap` 是字体的行间距（OS/2 sTypoLineGap / hhea lineGap）。chromium 与
    /// fontdue 的 `line-height:normal` = `ascent − descent + line_gap`（按字号缩放）。
    ///
    /// **接通状态（R2202 核对 2026-07-29；R2393 复核 2026-08-01）**：`line-height:normal`
    /// 经 `resolve_normal_line_height` 在 provider=Some 时已走真实 `ascent − descent + line_gap`
    /// ——**reftest runner 已接通**（`reftest.rs:568` 调 `set_font_metric_map`），
    /// **生产 webview/renderer dormant 接通**（env `ZW_PERFONT_LINEHEIGHT=1` 激活，默认关）。
    /// **R2393 实证生产激活 = net 负，保持 dormant**（welcome 英文 +0.44pp 恶化；morning 中文
    /// 零变化——全显式 line-height 无 normal 行，「CJK lever」假设证伪）→ **勿再以 font-metric
    /// 生产激活为 lever**（证据 `evidence/font-metric-activation-ab-2026-08-01.md`）。IFC
    /// `strut_ascent` / `half-leading` 仍用 `0.8·fs` / `1.2` 近似（R759 仅修 Ahem 为 1.0）——
    /// R834 实证单点改 strut_ascent 反退 welcome，须与 IFC strut/half-leading 真实化打包（深结构
    /// R834 谱系）。本方法供上述 line-height:normal 真实度量路径消费此前被 [`line_metrics`]
    /// 丢弃的 `line_gap`。
    pub fn line_metrics_full(&self, font_id: u32, size: f32) -> Option<(f32, f32, f32)> {
        let font = self.fonts.get(&font_id).map(|f| f.as_ref())?;
        let metrics = font.horizontal_line_metrics(size)?;
        Some((metrics.ascent, metrics.descent, metrics.line_gap))
    }

    /// 测量字符 advance 宽度（含回退）
    pub fn measure_advance(&self, primary_id: u32, code_point: char, size: f32) -> f32 {
        if self.font_allows_code_point(primary_id, code_point)
            && let Some(bitmap) = self.bitmap_glyphs.get(&(primary_id, code_point as u32, size.to_bits()))
        {
            return bitmap.advance;
        }
        // 快路径：轻量测量（不产位图）——CJK 页每帧 paint 对每个 (字符, 字号)
        // 测量一次，旧实现走 rasterize_glyph_with_fallback 连位图一起光栅化
        // （每字形数十微秒）。advance 必须与光栅化路径同源，否则布局宽度漂移：
        // - freetype-raster（默认 feature）：FreeType load_glyph 读 advance
        //   （26.6 定点，与绘制光栅化完全一致）
        // - 纯 fontdue：metrics（纯几何，不产位图）
        // 仅当 advance > 0 时提前返回；零宽字形等边缘 case 回退旧路径，覆盖
        // 语义（glyph_has_coverage 位图判定）与旧版完全一致。
        let chain = self.lookup_chain(primary_id);
        for font_id in chain {
            if !self.font_allows_code_point(font_id, code_point) {
                continue;
            }
            let Some(font) = self.fonts.get(&font_id).map(|f| f.as_ref()) else {
                continue;
            };
            // 与 rasterize_glyph_with_fallback 相同的缺字检查（空白字符跳过——
            // 空白在任意字体都视为可用，由 advance 决定实际宽度）
            if !code_point.is_whitespace() && !font.has_glyph(code_point) {
                continue;
            }
            #[cfg(feature = "freetype-raster")]
            if let Some(bytes) = self.font_data.get(&font_id)
                && let Ok(advance) = freetype_raster::measure_advance(bytes, self.face_index(font_id), code_point, size)
                && advance > 0.0
            {
                return advance;
            }
            #[cfg(not(feature = "freetype-raster"))]
            {
                let metrics = font.metrics(code_point, size);
                if metrics.advance_width > 0.0 {
                    return metrics.advance_width;
                }
            }
        }
        // 回退：零宽字形等边缘 case 走原光栅化路径（coverage 语义一致）
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

    /// Phase 2（`freetype-raster` feature）：FreeType 光栅化路径端到端 + 坐标约定守卫。
    ///
    /// 用 bundled Ahem.ttf（WPT 标准方块字体）调 `freetype_raster::rasterize`，
    /// 断言位图非空 + 尺寸 ≈ font_size（Ahem 方块）+ y_offset = bitmap_top − height
    /// 约定成立（y_offset 在 [−height, 0] 区间，位图顶在 baseline 上方）。仅 feature 开时编译。
    #[cfg(feature = "freetype-raster")]
    #[test]
    fn freetype_rasterize_ahem_glyph_end_to_end() {
        // loader.rs 在 crates/render-foundation/src/font/，4 级 .. 回 workspace 根。
        const AHEM_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");
        let bm = freetype_raster::rasterize(AHEM_TTF, 0, 'X', 20.0).expect("FreeType should rasterize Ahem 'X' @20px");
        // Ahem 方块：位图非空，宽高 ≈ 20px（FreeType @20px 实测 20×20，A4）。
        assert!(
            bm.width > 0 && bm.height > 0,
            "non-empty bitmap, got {}x{}",
            bm.width,
            bm.height
        );
        assert!(
            (bm.width as i32 - 20).abs() <= 1 && (bm.height as i32 - 20).abs() <= 1,
            "Ahem @20px ≈ 20x20, got {}x{}",
            bm.width,
            bm.height
        );
        // 坐标约定：y_offset = bitmap_top − height。Ahem 方块顶 ≈ ascent（~16px @20px），
        // 故 y_offset ≈ 16 − 20 = −4 ± 容差。负值表示位图顶在 baseline 上方。
        assert!(
            (-(bm.height as i16)..=0).contains(&bm.y_offset),
            "y_offset in [-height, 0], got {}",
            bm.y_offset
        );
        // advance ≈ font_size（Ahem 等宽 = em）。
        assert!((bm.advance - 20.0).abs() < 2.0, "Ahem advance ≈ 20, got {}", bm.advance);
    }

    /// R1950 诊断：grid-fitted slot advance（当前 ZW，glyph.advance().x/64）vs
    /// linearHoriAdvance（chromium/Skia 用于 text layout，unhinted scaled）。
    /// 验假设：linearHoriAdvance ≈ chromium 0.797×fs，slot advance（hinted）≈0.833×fs。
    /// 跳过若系统无 LiberationSerif。
    #[test]
    #[cfg(feature = "freetype-raster")]
    fn diag_linear_vs_gridfitted_advance() {
        let path = "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf";
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("diag: LiberationSerif not found at {path}, skip");
                return;
            }
        };
        let lib = freetype::Library::init().expect("FT init");
        let face = lib.new_memory_face2(data, 0).expect("FT face");
        let size = 16.0_f64;
        face.set_char_size((size * 64.0) as isize, (size * 64.0) as isize, 0, 0)
            .expect("FT set_char_size");
        eprintln!("diag: char gridfitted(×fs) linear(×fs)  [chromium m≈0.797×fs]");
        for ch in ['m', 'i', 'W', 'n', 'o', 'a', 'l'] {
            let idx = match face.get_char_index(ch as usize) {
                Some(i) => i,
                None => continue,
            };
            face.load_glyph(idx, freetype::face::LoadFlag::DEFAULT)
                .expect("FT load_glyph");
            let slot = face.glyph();
            let gridfitted = slot.advance().x as f64 / 64.0;
            let linear = slot.linear_hori_advance() as f64 / 65536.0;
            eprintln!(
                "diag:  {ch:?}  {gf:.4} ({gfr:.4}×fs)   {lin:.4} ({linr:.4}×fs)",
                gf = gridfitted,
                gfr = gridfitted / size,
                lin = linear,
                linr = linear / size,
            );
        }
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
    fn test_rasterize_glyph_index_matches_code_point() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let glyph_index = loader
            .get(font_id)
            .expect("font should remain loaded")
            .lookup_glyph_index('A');

        let by_code_point = loader.rasterize_glyph(font_id, 'A', 16.0).expect("code point raster");
        let by_index = loader
            .rasterize_glyph_index(font_id, glyph_index, 16.0)
            .expect("indexed raster");

        assert_eq!(by_index.data, by_code_point.data);
        assert_eq!(by_index.width, by_code_point.width);
        assert_eq!(by_index.height, by_code_point.height);
        assert_eq!(by_index.x_offset, by_code_point.x_offset);
        assert_eq!(by_index.y_offset, by_code_point.y_offset);
        assert_eq!(by_index.advance, by_code_point.advance);
    }

    #[test]
    fn rasterize_glyph_index_rejects_out_of_range_index() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("load bundled Lato");

        assert!(matches!(
            loader.rasterize_glyph_index(font_id, u16::MAX, 16.0),
            Err(FontError::GlyphNotFound {
                font_id: rejected_font,
                glyph_id
            }) if rejected_font == font_id && glyph_id == u32::from(u16::MAX)
        ));
    }

    #[test]
    fn shape_cache_reuses_base_font_results_across_duplicates() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        const AHEM_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");

        let first = loader.shape_text_cached(font_id, "AV", 16.0).expect("first shape");
        let second = loader.shape_text_cached(font_id, "AV", 16.0).expect("cached shape");
        assert_eq!(first.len(), second.len());
        assert_eq!(
            first.iter().map(|glyph| glyph.glyph_id).collect::<Vec<_>>(),
            second.iter().map(|glyph| glyph.glyph_id).collect::<Vec<_>>()
        );
        assert_eq!(loader.shape_cache.lock().expect("shape cache").len(), 1);

        let mut duplicate = loader.duplicate();
        assert_eq!(
            duplicate.shape_cache.lock().expect("duplicate shape cache").len(),
            1,
            "duplicate loaders should reuse stable base-font shaping entries"
        );
        assert_eq!(
            duplicate.font_instance_ids.get(&font_id),
            loader.font_instance_ids.get(&font_id)
        );

        let left_id = loader.load_font(AHEM_TTF).expect("left appended font");
        let right_id = duplicate.load_font(AHEM_TTF).expect("right appended font");
        assert_eq!(left_id, right_id, "duplicates reuse the same numeric font ID");
        assert_ne!(
            loader.font_instance_ids.get(&left_id),
            duplicate.font_instance_ids.get(&right_id),
            "new fonts in separate duplicates require distinct cache identities"
        );
    }

    #[test]
    fn shape_cache_separates_text_direction() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");

        let ltr = loader
            .shape_text_cached_with_direction(font_id, "ABC", 16.0, crate::font::TextDirection::LeftToRight)
            .expect("LTR shape");
        let rtl = loader
            .shape_text_cached_with_direction(font_id, "ABC", 16.0, crate::font::TextDirection::RightToLeft)
            .expect("RTL shape");

        assert_eq!(ltr.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(rtl.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(), vec![2, 1, 0]);
        assert_eq!(loader.shape_cache.lock().expect("shape cache").len(), 2);
    }

    #[test]
    fn shape_cache_separates_open_type_features() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");

        let enabled = loader
            .shape_text_cached_with_features(
                font_id,
                "fi",
                16.0,
                crate::font::TextDirection::LeftToRight,
                &[crate::font::OpenTypeFeature::new(*b"liga", 1)],
            )
            .expect("liga enabled shape");
        let disabled = loader
            .shape_text_cached_with_features(
                font_id,
                "fi",
                16.0,
                crate::font::TextDirection::LeftToRight,
                &[crate::font::OpenTypeFeature::new(*b"liga", 0)],
            )
            .expect("liga disabled shape");

        assert_eq!(enabled.len(), 1);
        assert_eq!(disabled.len(), 2);
        assert_eq!(loader.shape_cache.lock().expect("shape cache").len(), 2);
    }

    #[test]
    fn caller_features_override_font_face_defaults() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        loader.register_font_features(font_id, vec![crate::font::OpenTypeFeature::new(*b"liga", 0)]);

        let face_default = loader
            .shape_text_cached_with_features(font_id, "fi", 16.0, crate::font::TextDirection::LeftToRight, &[])
            .expect("face default shape");
        let caller_override = loader
            .shape_text_cached_with_features(
                font_id,
                "fi",
                16.0,
                crate::font::TextDirection::LeftToRight,
                &[crate::font::OpenTypeFeature::new(*b"liga", 1)],
            )
            .expect("caller override shape");

        assert_eq!(face_default.len(), 2);
        assert_eq!(caller_override.len(), 1);
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
}

// ── CJK 栅格化性能探针（临时诊断，2026-08-07）──
// 修复（2026-08-08）：模块内无条件引用 freetype_raster——quickjs 矩阵
//（--no-default-features，render CPU 测试步骤）编译失败。gate 整个探针模块。
#[cfg(test)]
#[cfg(feature = "freetype-raster")]
mod cjk_raster_probe {
    use super::*;
    use std::time::Instant;

    fn load_system_cjk_font() -> Option<Vec<u8>> {
        for path in [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc",
        ] {
            if let Ok(bytes) = std::fs::read(path) {
                return Some(bytes);
            }
        }
        None
    }

    #[test]
    fn cjk_raster_cost_probe() {
        let Some(bytes) = load_system_cjk_font() else {
            eprintln!("无系统 CJK 字体，跳过");
            return;
        };
        eprintln!("CJK 字体: {} KB", bytes.len() / 1024);
        let chars = ['测', '中', '文', '字', '体', '栅', '格', '化', '优', '化'];
        let size = 16.0f32;

        // fontdue
        let font = fontdue::Font::from_bytes(bytes.clone(), fontdue::FontSettings::default()).expect("fontdue");
        let mut total = 0.0f64;
        for _ in 0..50 {
            let t = Instant::now();
            for &c in &chars {
                let _ = font.rasterize(c, size);
            }
            total += t.elapsed().as_secs_f64();
        }
        eprintln!("fontdue: {:.3} ms/字", total / 50.0 / chars.len() as f64 * 1000.0);

        // freetype_raster（face 缓存生效验证：前 3 次含首次解析 vs 后 47 次命中缓存）
        let mut first_total = 0.0f64;
        let mut rest_total = 0.0f64;
        let mut ok = 0;
        for i in 0..50 {
            let t = Instant::now();
            for &c in &chars {
                if freetype_raster::rasterize(&bytes, 0, c, size).is_ok() {
                    ok += 1;
                }
            }
            let d = t.elapsed().as_secs_f64();
            if i < 3 {
                first_total += d;
            } else {
                rest_total += d;
            }
        }
        eprintln!(
            "freetype_raster: 前 3 次 {:.3} ms/字，后 47 次 {:.3} ms/字 (成功 {}/{})",
            first_total / 3.0 / chars.len() as f64 * 1000.0,
            rest_total / 47.0 / chars.len() as f64 * 1000.0,
            ok,
            50 * chars.len()
        );
    }
}
