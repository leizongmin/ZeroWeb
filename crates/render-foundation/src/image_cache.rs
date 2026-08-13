//! 图片对象缓存与 GC — 管理已解码图片的缓存和生命周期
//!
//! 提供：
//! - 基于引用计数的图片缓存
//! - LRU 风格的垃圾回收
//! - 图片数据存储（RGBA 像素数据）

use crate::geometry::Size;
use hashbrown::HashMap;

/// 图片缓存键 — 唯一标识一张图片
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImageKey(pub u64);

impl ImageKey {
    /// 创建新的图片键
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 已解码的图片数据
#[derive(Debug, Clone)]
pub struct ImageData {
    /// RGBA 像素数据（行优先）
    pub pixels: Vec<u8>,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
    /// 像素内容摘要（R3254-M2：插入时计算一次，GPU 纹理缓存键复用——像素在插入
    /// ImageCache 后不可变，避免每帧对每张图全量哈希）。
    pub content_hash: u64,
    /// 纯色检测结果 — 当所有像素相同时缓存该颜色，用于优化渲染
    solid_color: Option<[u8; 4]>,
    /// 仅含宽高比、无确定固有尺寸的信号（CSS §10.3.2）。
    ///
    /// 仅 SVG 会出现：当 `<svg>` 的 width/height 为百分比或缺失（仅有 viewBox）时，
    /// 替换元素**无确定固有尺寸**，仅有 viewBox 给出的宽高比。此时栅格化仍按 usvg
    /// 解析尺寸产出像素（供绘制），但布局须把此 `ratio` 当作唯一信号——不设确定 size，
    /// 让 taffy/flex 按上下文 ratio-derive（如 flex column width 拉伸 → height=width/ratio）。
    /// PNG/JPEG/绝对尺寸 SVG 为 `None`（走 image_sizes 正常固有尺寸路径）。
    intrinsic_ratio: Option<f32>,
    /// no-ratio 信号（CSS §10.3.2）：SVG 的 width/height 非双绝对且**无**可用 viewBox
    /// 宽高比时，替换元素既无确定固有尺寸、也无固有宽高比。usvg 对缺失维给出默认值
    ///（如缺 height 时 h=100），该默认值不是真实固有尺寸——故 pixmap 的该维不可用于
    /// 比例推导。`Some((w, h))` 仅 no-ratio SVG 出现，`w`/`h` 为真实固有宽高（仅 abs
    /// 属性存在的维，缺失维为 `None`）；布局须**不设 aspect_ratio**，按 CSS §10.3.2
    /// default object size（宽 300 / 高 150）回退。PNG/JPEG/both-abs/ratio-only SVG 为 `None`。
    no_ratio: Option<(Option<f32>, Option<f32>)>,
    /// 计算的真实固有尺寸（CSS §10.3.2）：一维 abs + 另一维缺失 + viewBox 宽高比时，
    /// usvg 对缺失维用原始 viewBox 值（pixmap bogus），须按 abs 维 × ratio 计算。`Some((w,h))`
    /// 覆盖 pixmap 用于 image_sizes（aspect_ratio 由 w/h 推导）。仅该类 SVG 出现；其余为 `None`。
    computed_intrinsic: Option<(f32, f32)>,
}

/// RGBA 像素内容摘要（R3254-M2 共享实现——ImageData 插入时预存一次，GPU 纹理缓存
/// 键复用同一算法，保证「同像素同 hash」）。
pub fn hash_pixels(pixels: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

impl ImageData {
    /// 从 RGBA 字节数据创建图片
    ///
    /// # Errors
    /// 如果数据长度不等于 `width * height * 4` 则返回错误
    pub fn from_rgba(pixels: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() != expected {
            return Err(format!(
                "pixel data size mismatch: expected {expected}, got {}",
                pixels.len()
            ));
        }
        // 检测纯色图片：所有像素相同时缓存颜色值
        let solid_color = if pixels.len() >= 4 {
            let first = [pixels[0], pixels[1], pixels[2], pixels[3]];
            let all_same = pixels.chunks_exact(4).all(|chunk| {
                chunk[0] == first[0] && chunk[1] == first[1] && chunk[2] == first[2] && chunk[3] == first[3]
            });
            if all_same { Some(first) } else { None }
        } else {
            None
        };
        let content_hash = hash_pixels(&pixels);
        Ok(Self {
            pixels,
            width,
            height,
            content_hash,
            solid_color,
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
        })
    }

    /// 创建指定尺寸的空（全透明）图片
    pub fn new_empty(width: u32, height: u32) -> Self {
        let pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        let content_hash = hash_pixels(&pixels);
        Self {
            pixels,
            width,
            height,
            content_hash,
            solid_color: Some([0, 0, 0, 0]),
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
        }
    }

    /// 获取指定位置的像素 (R, G, B, A)。
    ///
    /// 越界坐标钳制到最近有效像素；像素缓冲与宽高不一致时返回透明（避免渲染崩溃）。
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let expected = (self.width as usize) * (self.height as usize) * 4;
        if self.width == 0 || self.height == 0 || self.pixels.len() != expected {
            return [0, 0, 0, 0];
        }
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// 获取图片尺寸
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// 如果图片是纯色（所有像素相同），返回该颜色
    ///
    /// 用于渲染优化：纯色图片缩放时不需要双线性插值，直接填充目标矩形即可。
    /// 这消除了 WPT reftest 中 swatch 图片（如 1x1-green.png、15x15-green.png）
    /// 缩放到大尺寸时的边缘抗锯齿伪影。
    pub fn solid_color(&self) -> Option<[u8; 4]> {
        self.solid_color
    }

    /// 仅含宽高比、无确定固有尺寸的信号（CSS §10.3.2，仅 SVG 出现）。
    ///
    /// 返回 `Some(ratio)` 表示此图无确定固有尺寸、仅有 viewBox 宽高比——布局须以
    /// ratio-only 信号处理（不设确定 size，仅设 aspect_ratio）。`None` 表示有确定
    /// 固有尺寸（width/height 字段有效，走 image_sizes 路径）。
    pub fn intrinsic_ratio(&self) -> Option<f32> {
        self.intrinsic_ratio
    }

    /// no-ratio 信号（CSS §10.3.2）：返回 `Some((w, h))` 表示此 SVG 既无确定固有尺寸
    /// 也无固有宽高比（width/height 非双绝对且无 viewBox），`w`/`h` 为真实固有宽高
    ///（仅 abs 属性存在的维，缺失维 `None`）。布局须不设 aspect_ratio，缺失维按
    /// default object size（宽 300 / 高 150）回退。`None` 表示非 no-ratio（走 sizes/ratios 路径）。
    pub fn no_ratio_intrinsic(&self) -> Option<(Option<f32>, Option<f32>)> {
        self.no_ratio
    }

    /// 计算的真实固有尺寸（CSS §10.3.2）：`Some((w, h))` 表示此 SVG 为「一维 abs + 另一维
    /// 缺失 + viewBox」类，usvg pixmap 对缺失维用原始 viewBox 值（bogus），须用此计算值
    ///（abs × ratio）覆盖 pixmap 用于 image_sizes。`None` 表示 pixmap 尺寸有效。
    pub fn computed_intrinsic(&self) -> Option<(f32, f32)> {
        self.computed_intrinsic
    }

    /// 字节大小估算
    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }
}

/// 图片缓存条目
#[derive(Debug)]
struct CacheEntry {
    /// 图片数据
    data: ImageData,
    /// 引用计数
    ref_count: u32,
    /// 最近访问的代数（用于 GC）
    last_access_gen: u64,
}

/// 图片对象缓存 — 管理已解码图片的生命周期
///
/// 使用引用计数和代际标记实现 GC：
/// - 插入图片时 ref_count = 1
/// - 每次 `get` 时递增 ref_count 并更新 last_access_gen
/// - `gc()` 移除 ref_count == 0 或长时间未访问的条目
pub struct ImageCache {
    /// 缓存条目
    entries: HashMap<ImageKey, CacheEntry>,
    /// 下一个键 ID
    next_key: u64,
    /// 当前世代（每次 GC 递增）
    current_gen: u64,
    /// 最大缓存条目数
    max_entries: usize,
    /// 最大字节数
    max_bytes: usize,
}

impl ImageCache {
    /// 创建新的图片缓存
    ///
    /// - `max_entries`: 最大缓存条目数
    /// - `max_bytes`: 最大缓存字节数
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            next_key: 0,
            current_gen: 0,
            max_entries,
            max_bytes,
        }
    }

    /// R34xx：快照全部缓存条目（key hash → ImageData clone），供 host 回调查询
    ///（webview 的 __zw_get_image_wire——headless canvas drawImage img 源 G5）。
    pub fn snapshot_entries(&self) -> std::collections::HashMap<u64, ImageData> {
        self.entries.iter().map(|(k, e)| (k.0, e.data.clone())).collect()
    }

    /// 插入图片数据，返回缓存键
    pub fn insert(&mut self, data: ImageData) -> ImageKey {
        let key = ImageKey::new(self.next_key);
        self.next_key += 1;

        let entry = CacheEntry {
            data,
            ref_count: 1,
            last_access_gen: self.current_gen,
        };
        self.entries.insert(key.clone(), entry);
        key
    }

    /// 使用指定的缓存键插入图片数据。
    ///
    /// 用于 reftest 场景：paint 系统通过 `simple_hash(src)` 生成 ImageKey，
    /// 外部加载器需要用相同的 key 将解码后的图片数据注入缓存。
    pub fn insert_with_key(&mut self, key: ImageKey, data: ImageData) {
        let entry = CacheEntry {
            data,
            ref_count: 1,
            last_access_gen: self.current_gen,
        };
        self.entries.insert(key, entry);
    }

    /// 获取图片数据的引用，并递增引用计数
    pub fn get(&mut self, key: &ImageKey) -> Option<&ImageData> {
        let entry = self.entries.get_mut(key)?;
        entry.ref_count = entry.ref_count.saturating_add(1);
        entry.last_access_gen = self.current_gen;
        Some(&entry.data)
    }

    /// 释放一次引用（递减引用计数）
    pub fn release(&mut self, key: &ImageKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
        }
    }

    /// 执行垃圾回收
    ///
    /// 移除以下条目：
    /// - ref_count == 0 的条目
    /// - 如果总条目数或总字节数超过限制，按 LRU 淘汰最旧条目
    pub fn gc(&mut self) {
        self.current_gen += 1;

        // 先移除 ref_count == 0 的条目
        self.entries.retain(|_, entry| entry.ref_count > 0);

        // 如果仍然超限，按 last_access_gen 排序淘汰最旧条目
        while self.entries.len() > self.max_entries || self.total_bytes() > self.max_bytes {
            if self.entries.is_empty() {
                break;
            }
            // 找到最旧的条目并移除
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_access_gen)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }

    /// 当前缓存条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 当前缓存字节总数
    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|e| e.data.byte_size()).sum()
    }

    /// 当前世代
    pub fn generation(&self) -> u64 {
        self.current_gen
    }

    /// 获取指定键的引用计数（用于测试）
    pub fn ref_count(&self, key: &ImageKey) -> Option<u32> {
        self.entries.get(key).map(|e| e.ref_count)
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_gen += 1;
    }

    /// 复制当前缓存内容供 UI 线程快照使用（不共享可变状态）。
    pub fn duplicate_for_snapshot(&self) -> Self {
        let mut out = Self::new(self.max_entries, self.max_bytes);
        out.next_key = self.next_key;
        for (key, entry) in &self.entries {
            out.entries.insert(
                key.clone(),
                CacheEntry {
                    data: entry.data.clone(),
                    ref_count: 1,
                    last_access_gen: 0,
                },
            );
        }
        out
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(256, 64 * 1024 * 1024) // 256 entries, 64 MB
    }
}

// ─── 图片解码（PNG / JPEG / SVG）──────────────────────────────────────
//
// 供 URL 导航路径（webview fetch_url）抓取 `<img src>` 子资源后解码为 ImageData。
// render-foundation 拥有 ImageData 结构，故解码逻辑置此，供 webview / reftest 等复用。
// 支持格式：PNG（最常见）/ JPEG（goal doc DC-13「PNG/JPEG 基础解码」）/ WebP
// （R1793「PNG/JPEG/WebP 基础解码」，image-webp 纯 Rust）/ SVG 栅格化
// （resvg + tiny-skia，goal doc DC-13「SVG 栅格化」）。`decode_image_bytes` 按
// 魔数字节（PNG/JPEG/WebP）或文本内容嗅探（SVG）分发格式。

/// 将 PNG 字节流解码为 RGBA `ImageData`。
///
/// 正确处理任意 PNG color type（palette / grayscale / RGB / RGBA）与位深：
/// `EXPAND | STRIP_16` 变换把 palette→RGB(A)、grayscale→RGB(A)、16bit→8bit，
/// 再按 `OutputInfo.color_type` 转换为 RGBA。直接按 4 字节直读会导致非 RGBA 输入
/// 错位（alpha=0 退化透明）。
///
/// # Errors
/// 解码失败时返回描述性错误字符串（调用方决定降级策略）。
pub fn decode_png_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use png::Transformations;
    let mut decoder = png::Decoder::new(bytes);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("PNG 解码失败: {e}"))?;

    let info = reader.info().clone();
    let width = info.width;
    let height = info.height;

    let mut raw = vec![0u8; reader.output_buffer_size()];
    let output_info = reader.next_frame(&mut raw).map_err(|e| format!("PNG 读取失败: {e}"))?;
    let rgba = convert_png_buffer_to_rgba(&raw, output_info.color_type, output_info.bit_depth);
    ImageData::from_rgba(rgba, width, height)
}

/// 解码 JPEG 字节为 `ImageData`（RGBA）。
///
/// goal doc DC-13「图片子资源 / ImageCache」要求 PNG/JPEG 基础解码。
/// 使用 `jpeg-decoder`（纯 Rust，MIT/Apache-2.0），输出统一转 RGBA。
pub fn decode_jpeg_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use jpeg_decoder::Decoder;
    let mut decoder = Decoder::new(bytes);
    let pixels = decoder.decode().map_err(|e| format!("JPEG 解码失败: {e}"))?;
    let info = decoder.info().ok_or_else(|| "JPEG 无图像元数据".to_string())?;
    let width = info.width as u32;
    let height = info.height as u32;
    let rgba = convert_jpeg_pixels_to_rgba(&pixels, info.pixel_format);
    ImageData::from_rgba(rgba, width, height)
}

/// 解码 WebP 字节为 `ImageData`（RGBA）。
///
/// goal doc Support Envelope「图片子资源 / ImageCache」要求 PNG/JPEG/WebP 基础解码
///（rendering-compat.md line 76）。使用 `image-webp`（纯 Rust，MIT/Apache-2.0）；
/// 该 crate 已作为 `image` 的传递依赖存在于 Cargo.lock（0.2.4），此处提升为直接依赖。
/// `read_image` 在 `has_alpha` 时输出 RGBA，否则输出 RGB（补 alpha=255），统一转 RGBA。
pub fn decode_webp_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use std::io::Cursor;
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(bytes)).map_err(|e| format!("WebP 解码失败: {e}"))?;
    let (width, height) = decoder.dimensions();
    let has_alpha = decoder.has_alpha();
    let buf_size = decoder
        .output_buffer_size()
        .ok_or_else(|| "WebP 无输出缓冲区大小（可能为动画，暂不支持）".to_string())?;
    let mut raw = vec![0u8; buf_size];
    decoder
        .read_image(&mut raw)
        .map_err(|e| format!("WebP 读取失败: {e}"))?;
    let rgba = if has_alpha {
        // 已是 RGBA，直接使用。
        raw
    } else {
        // RGB → RGBA（补 alpha=255）。
        let mut out = Vec::with_capacity(raw.len() / 3 * 4);
        for px in raw.chunks_exact(3) {
            out.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        out
    };
    ImageData::from_rgba(rgba, width, height)
}

/// 把 `jpeg-decoder` 输出的像素（RGB24/L8/L16/CMYK32）转换为 RGBA。
///
/// JPEG 不含 alpha 通道，故统一补 alpha=255。CMYK 按 JPEG 惯例（Adobe 倒置 K）
/// 用「255 - value」近似转 RGB。
fn convert_jpeg_pixels_to_rgba(raw: &[u8], pixel_format: jpeg_decoder::PixelFormat) -> Vec<u8> {
    use jpeg_decoder::PixelFormat;
    match pixel_format {
        PixelFormat::RGB24 => {
            let mut out = Vec::with_capacity(raw.len() / 3 * 4);
            for px in raw.chunks_exact(3) {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        PixelFormat::L8 => {
            let mut out = Vec::with_capacity(raw.len() * 4);
            for &g in raw {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        PixelFormat::L16 => {
            // 16-bit grayscale（big-endian u16），降级为 8-bit RGBA（取高字节）。
            let mut out = Vec::with_capacity(raw.len() / 2 * 4);
            for px in raw.chunks_exact(2) {
                let hi = px[0];
                out.extend_from_slice(&[hi, hi, hi, 255]);
            }
            out
        }
        PixelFormat::CMYK32 => {
            // CMYK → RGB（Adobe JPEG 惯例：K 倒置，C/M/Y 取 255-value）
            let mut out = Vec::with_capacity(raw.len() / 4 * 4);
            for px in raw.chunks_exact(4) {
                let c = 255 - px[0];
                let m = 255 - px[1];
                let y = 255 - px[2];
                let k = px[3] as u32;
                let r = (c as u32 * k / 255).min(255) as u8;
                let g = (m as u32 * k / 255).min(255) as u8;
                let b = (y as u32 * k / 255).min(255) as u8;
                out.extend_from_slice(&[r, g, b, 255]);
            }
            out
        }
    }
}

/// 按魔数字节嗅探图片格式并解码（PNG / JPEG / SVG）。
///
/// 比 URL 扩展名更可靠（URL 可能无扩展名或扩展名错误）。PNG 文件以 `\x89PNG` 开头；
/// JPEG 文件以 `\xFF\xD8\xFF` 开头；SVG 为文本，嗅探 UTF-8 内容起始 `<svg` / `<?xml`。
/// 未知格式返回错误，调用方可记录日志并降级（不阻断页面加载）。
pub fn decode_image_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    if bytes.starts_with(b"\x89PNG") {
        decode_png_bytes(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg_bytes(bytes)
    } else if is_webp_magic(bytes) {
        decode_webp_bytes(bytes)
    } else if looks_like_svg(bytes) {
        decode_svg_bytes(bytes)
    } else {
        Err(format!(
            "unsupported image format (magic bytes: {:?}); only PNG/JPEG/WebP/SVG supported",
            bytes.get(..4).unwrap_or(&[])
        ))
    }
}

/// WebP magic：`RIFF` (offset 0..4) + 文件大小 4 字节 + `WEBP` (offset 8..12)。
/// 见 https://developers.google.com/speed/webp/docs/riff_container。
fn is_webp_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// 是否为可独立进程解码的栅格图像字节（PNG/JPEG/WebP magic 检测）。
///
/// D1：image-decoder 独立进程只处理栅格格式；SVG（依赖资源加载）与
/// data URI 保持在调用进程内解码。供 webview 侧解码分发使用。
pub fn is_raster_image_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG") || bytes.starts_with(&[0xFF, 0xD8, 0xFF]) || is_webp_magic(bytes)
}

/// 解析 `data:` URI 为原始字节。
///
/// `data:[<mediatype>][;base64],<payload>` —— header 含 `base64` 则 base64 解码 payload，
/// 否则按字节 percent-decode（%XX）。无逗号或解码失败返回 Err。
///
/// https://url.spec.whatwg.org/#data-urls
pub fn decode_data_uri_bytes(src: &str) -> Result<Vec<u8>, String> {
    if !src.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) {
        return Err("不是 data URI".to_string());
    }
    let Some(comma) = src.find(',') else {
        return Err("data URI 缺逗号分隔符".to_string());
    };
    let header = &src[..comma];
    let payload = &src[comma + 1..];
    let bytes = if header.split(';').any(|part| part.eq_ignore_ascii_case("base64")) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| format!("data URI base64 解码失败: {e}"))?
    } else {
        percent_decode_bytes(payload)
    };
    Ok(bytes)
}

/// R1705：解析 `data:` URI 并解码为 `ImageData`（renderer 多进程路径 + wpt-runner 共用）。
///
/// 所得字节交 `decode_image_bytes` 按 magic 分派（PNG/JPEG/WebP/SVG）。
/// 解码失败返回 Err（调用方降级，不阻断页面）。
pub fn decode_data_uri(src: &str) -> Result<ImageData, String> {
    decode_image_bytes(&decode_data_uri_bytes(src)?)
}

/// 字节级 percent-decode（%XX → byte）；非 % 字节原样保留（data URI 非 base64 payload）。
fn percent_decode_bytes(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

/// 嗅探字节是否为 SVG（文本，跳过 UTF-8 BOM 与前导空白后以 `<svg` 或 `<?xml` 开头）。
///
/// `<?xml` 声明不一定是 SVG，但在图片加载上下文中非 PNG/JPEG 的 XML 应尝试 SVG
/// 解码（resvg 对非 SVG XML 会解析失败并返回错误，降级安全）。
fn looks_like_svg(bytes: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(bytes) else {
        return false;
    };
    let trimmed = s.trim_start_matches('\u{feff}').trim_start();
    trimmed.starts_with("<svg") || trimmed.starts_with("<?xml")
}

/// 把 SVG 字节栅格化为 RGBA `ImageData`（goal doc DC-13「SVG 栅格化」）。
///
/// 用 resvg + tiny-skia 按 SVG 内在尺寸（width/height 属性或 viewBox）栅格化。
/// 字体走默认空 fontdb（logo 一般无文本）；过大尺寸由 pixmap 分配失败自然兜底。
///
/// CSS §10.3.2：当 `<svg>` 的 width/height 为百分比或缺失（仅有 viewBox）时，替换元素
/// **无确定固有尺寸**，仅有 viewBox 宽高比。此时栅格化仍按 usvg 解析尺寸产出像素，
/// 但会设置 `intrinsic_ratio` 信号，让布局以 ratio-only 处理（不设确定 size）。
/// 当 width/height 非双绝对且**无** viewBox 时，替换元素既无确定固有尺寸也无固有宽高比
///（no-ratio）——usvg 对缺失维给出默认值（如缺 height 时 h=100），该默认值非真实固有
/// 尺寸，布局须以 `no_ratio` 信号处理（不设 aspect_ratio，缺失维按 default object size 回退）。
pub fn decode_svg_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())
        .map_err(|e| format!("SVG 解析失败: {e}"))?;
    let size = tree.size();
    // usvg Size 的 width()/height() 返回 f32（SVG 内在尺寸）
    let w = size.width().ceil() as u32;
    let h = size.height().ceil() as u32;
    if w == 0 || h == 0 {
        return Err("SVG 零尺寸".to_string());
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| format!("SVG pixmap 分配失败 {w}x{h}"))?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let rgba = pixmap.take();
    let mut data = ImageData::from_rgba(rgba, w, h)?;
    // ★ chromium 实测（visudet replaced-elements 簇 4 变体 × 7 SVG + css-flexbox
    // aspect-ratio-intrinsic-size-007，2026-07-15）：
    // - **INLINE `<img>`**（CSS2 §10.3.2）：非 BothAbs SVG 一律按 **default object size 300×150**
    //   sizing，**不**对显式 CSS 维应用 viewBox 宽高比（height-25-ratio-2.svg 配 width:40 → 40×150
    //   非 40×20）。仅 width+height 双绝对属性提供真固有尺寸 + ratio。
    // - **FLEX item**（flexbox §9.2.4 transferred-size）：ratio-only / 单 abs 维+viewBox SVG **保留**
    //   viewBox 宽高比（aspect-ratio-intrinsic-007：large-green-rectangle.svg 配 flex column →
    //   784×392，width stretch + height=width/ratio）。
    // 故 viewBox 比（RatioOnly / ComputedIntrinsic）仍走 intrinsic_ratio（flex transferred-size 用），
    // 但布局层对 INLINE 不应用该比（tree.rs branch 3 inline 分支用 default object size）。
    // NoRatio（无 viewBox）→ no_ratio(None,None)（inline default object size，无 ratio）。
    // （纠正 R1438「单 abs 维+viewBox → computed intrinsic 进 image_sizes」——inline 方向远离 chromium。）
    match svg_intrinsic_kind(bytes) {
        // 双绝对 → 真固有尺寸（pixmap w/h 有效，走 image_sizes；两信号均 None）
        SvgIntrinsicKind::BothAbs => {}
        // viewBox 比（无 abs 维 / 单 abs 维）：保留 ratio 供 flex transferred-size；inline 由
        // 布局层 default object size 处理（不应用此 ratio）。
        SvgIntrinsicKind::RatioOnly(ratio) => data.intrinsic_ratio = Some(ratio),
        // R2054：一维 abs + 另一维缺失 + viewBox 比 → **计算的真实固有尺寸**（abs × ratio）
        // 走 computed_intrinsic（→ image_sizes），使 INLINE auto+auto 用该固有尺寸（chromium
        // visudet replaced-elements-all-auto：height-25-ratio-2 → 50×25，非 300×150）。
        // 旧实现设 intrinsic_ratio 致 layout ratio-only 分支 INLINE 走 default 300×150（错）。
        // flex transferred-size 仍经 image_sizes 的 aspect_ratio 推导（tree.rs:439-441）。
        SvgIntrinsicKind::ComputedIntrinsic(cw, ch) if ch > 0.0 && cw > 0.0 => {
            data.computed_intrinsic = Some((cw, ch));
        }
        // 无 viewBox 比：default object size，无 ratio。
        SvgIntrinsicKind::ComputedIntrinsic(_, _) => {}
        // R2054：NoRatio 保留真实 abs 固有维（仅 abs 属性存在的维，缺失维 None）——旧实现
        // 丢弃为 (None,None) 致 height-25-no-ratio / width-50-no-ratio 失去固有维走全 default。
        // 布局 no_ratio 分支（tree.rs:543）用 w_opt/h_opt + unwrap_or(300/150) 正确分派。
        SvgIntrinsicKind::NoRatio { width, height } => data.no_ratio = Some((width, height)),
    }
    Ok(data)
}

/// SVG `<svg>` 根元素的固有尺寸分类（CSS §10.3.2）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum SvgIntrinsicKind {
    /// width/height 双绝对 → 真固有尺寸（pixmap w/h 有效，走 image_sizes）。
    BothAbs,
    /// 非双绝对 + 有 viewBox → 仅宽高比（走 image_ratios，不设确定 size）。
    RatioOnly(f32),
    /// 非双绝对 + 无 viewBox → 无固有宽高比；`width`/`height` 为真实固有维（仅 abs 属性
    /// 存在的维，缺失维 `None`）。usvg 对缺失维的默认值非真实固有尺寸，不可用于比例推导。
    NoRatio { width: Option<f32>, height: Option<f32> },
    /// 一维 abs + 另一维**缺失** + viewBox 宽高比 → 有可计算的真实固有尺寸（abs 维 × ratio）。
    /// usvg 对缺失维用原始 viewBox 值（pixmap bogus，如 `height="25" viewBox 1000×500` →
    /// pixmap (1000,25) 应 (50,25)），故携带计算值 `(w, h)` 覆盖 pixmap（走 image_sizes）。
    /// 仅「另一维缺失」触发（百分比维存在时仍 RatioOnly，避 flex ratio-derivation 回归）。
    ComputedIntrinsic(f32, f32),
}

/// 解析 SVG `<svg>` 根元素属性，分类其固有尺寸类型（CSS §10.3.2）。
///
/// - `BothAbs`：width/height 双绝对（真固有尺寸，走 image_sizes）。
/// - `RatioOnly(ratio)`：非双绝对且 viewBox 提供有效宽高比（ratio = viewBox_w / viewBox_h）。
/// - `NoRatio { width, height }`：非双绝对且无可用 viewBox 比；`width`/`height` 为 abs 属性
///   存在维的真实值（缺失维 `None`）。
/// - `ComputedIntrinsic(w, h)`：一维 abs + 另一维缺失 + viewBox 比 → 计算的真实固有尺寸
///   （abs × ratio），覆盖 usvg bogus pixmap。
fn svg_intrinsic_kind(bytes: &[u8]) -> SvgIntrinsicKind {
    let Ok(s) = std::str::from_utf8(bytes) else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let trimmed = s.trim_start_matches('\u{feff}').trim_start();
    let Some(svg_start) = trimmed.find("<svg") else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let after = &trimmed[svg_start..];
    let Some(tag_end) = after.find('>') else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let tag = &after[4..tag_end]; // 去掉 "<svg"

    let width = extract_svg_attr(tag, "width");
    let height = extract_svg_attr(tag, "height");
    let viewbox = extract_svg_attr(tag, "viewBox").or_else(|| extract_svg_attr(tag, "viewbox"));

    let w_val = width.as_deref().and_then(parse_abs_length_value);
    let h_val = height.as_deref().and_then(parse_abs_length_value);
    // 两维都绝对 → 真固有尺寸（走 image_sizes 正常路径）
    if w_val.is_some() && h_val.is_some() {
        return SvgIntrinsicKind::BothAbs;
    }
    // 否则优先 ratio-only：从 viewBox 取比（min-x min-y width height）
    if let Some(vb) = viewbox {
        let nums: Vec<&str> = vb.split([' ', ',']).filter(|t| !t.is_empty()).collect();
        if nums.len() == 4
            && let (Ok(vw), Ok(vh)) = (nums[2].parse::<f32>(), nums[3].parse::<f32>())
            && vh > 0.0
        {
            let ratio = vw / vh;
            // 一维 abs + 另一维**缺失** + viewBox → 计算真实固有尺寸（usvg pixmap 对缺失维
            // 用原始 viewBox 值，bogus）。如 `height="25" viewBox 1000×500`（ratio 2）→ (50,25)。
            // 仅「另一维属性缺失」触发；若另一维是百分比（属性存在），仍 RatioOnly（避 flex
            // ratio-derivation 回归，mixed-percent 案保持 R717 行为）。
            if let Some(h) = h_val
                && width.is_none()
            {
                return SvgIntrinsicKind::ComputedIntrinsic(h * ratio, h);
            }
            if let Some(w) = w_val
                && height.is_none()
            {
                return SvgIntrinsicKind::ComputedIntrinsic(w, w / ratio);
            }
            return SvgIntrinsicKind::RatioOnly(ratio);
        }
    }
    // 非双绝对 + 无可用 viewBox → no-ratio（真实固有维 = abs 属性存在维）
    SvgIntrinsicKind::NoRatio {
        width: w_val,
        height: h_val,
    }
}

/// 从 SVG 起始标签内容中提取 `name="value"` / `name='value'` 属性值。
fn extract_svg_attr(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=");
    let idx = tag.find(&pat)?;
    let after = &tag[idx + pat.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &after[1..];
    let end = inner.find(quote)?;
    Some(inner[..end].to_string())
}

/// SVG width/height 属性值解析为绝对长度值（正数 + 可选单位，非百分比/auto/none）。
///
/// 返回 `Some(value)` 当且仅当该值是有效正数绝对长度（缺失/百分比/auto/none/非数 → `None`）。
fn parse_abs_length_value(v: &str) -> Option<f32> {
    let v = v.trim();
    if v.is_empty() || v.ends_with('%') {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if lower == "auto" || lower == "none" {
        return None;
    }
    let num_end = v
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(v.len());
    let n: f32 = v[..num_end].parse().ok()?;
    (n > 0.0).then_some(n)
}

/// 把 PNG 解码后的原始缓冲（经 EXPAND 后的 RGB/RGBA/Grayscale/GrayscaleAlpha 8-bit）
/// 转换为 RGBA。
fn convert_png_buffer_to_rgba(raw: &[u8], color_type: png::ColorType, bit_depth: png::BitDepth) -> Vec<u8> {
    use png::ColorType::*;
    if bit_depth != png::BitDepth::Eight {
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

#[cfg(test)]
mod decode_tests {
    use super::*;

    /// 2×2 纯红 RGBA PNG（手工构造的合法 PNG 字节）。
    fn red_2x2_png() -> Vec<u8> {
        use png::{BitDepth, ColorType, Encoder};
        let mut buf = Vec::new();
        {
            let mut encoder = Encoder::new(&mut buf, 2, 2);
            encoder.set_color(ColorType::Rgba);
            encoder.set_depth(BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            // write_image_data 接收原始像素（无行过滤字节，编码器自加）：
            // 2×2×4 = 16 字节，4 个纯红像素。
            let data: Vec<u8> = [255, 0, 0, 255].repeat(4);
            writer.write_image_data(&data).unwrap();
        }
        buf
    }

    #[test]
    fn decode_png_bytes_rgba_2x2() {
        let png = red_2x2_png();
        let img = decode_png_bytes(&png).expect("decode should succeed");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        // 左上像素应为纯红
        assert_eq!(img.get_pixel(0, 0), [255, 0, 0, 255]);
    }

    #[test]
    fn decode_png_bytes_invalid_returns_err() {
        let result = decode_png_bytes(b"not a png");
        assert!(result.is_err());
    }

    #[test]
    fn convert_jpeg_pixels_to_rgba_rgb() {
        use jpeg_decoder::PixelFormat;
        // 2 个 RGB 像素：红、绿
        let raw = [255, 0, 0, 0, 255, 0];
        let rgba = convert_jpeg_pixels_to_rgba(&raw, PixelFormat::RGB24);
        assert_eq!(rgba, [255, 0, 0, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn convert_jpeg_pixels_to_rgba_grayscale() {
        use jpeg_decoder::PixelFormat;
        // 2 个灰度像素：128、64
        let raw = [128, 64];
        let rgba = convert_jpeg_pixels_to_rgba(&raw, PixelFormat::L8);
        assert_eq!(rgba, [128, 128, 128, 255, 64, 64, 64, 255]);
    }

    /// 解码真实的 4×3 纯绿 JPEG fixture（quality 95，DCT 在纯色块上近无损）。
    #[test]
    fn decode_jpeg_bytes_green_4x3() {
        let bytes = include_bytes!("testdata/green_4x3.jpg");
        let img = decode_jpeg_bytes(bytes).expect("JPEG decode should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        // JPEG 有损：断言绿色主导（G 高、R/B 低）而非精确 (0,255,0)。
        let px = img.get_pixel(1, 1);
        assert!(px[1] > 200, "green channel should be high, got {}", px[1]);
        assert!(px[0] < 50, "red channel should be low, got {}", px[0]);
        assert!(px[2] < 50, "blue channel should be low, got {}", px[2]);
        assert_eq!(px[3], 255, "alpha should be fully opaque");
    }

    #[test]
    fn decode_jpeg_bytes_invalid_returns_err() {
        // JPEG 魔数 + 无效正文 → 库解码失败
        let result = decode_jpeg_bytes(&[0xFF, 0xD8, 0xFF, 0x00, 0x00]);
        assert!(result.is_err());
    }

    /// R1793：解码真实 4×3 纯绿 WebP fixture（lossless，RGB → 补 alpha=255）。
    #[test]
    fn decode_webp_bytes_green_4x3() {
        let bytes = include_bytes!("testdata/green_4x3.webp");
        let img = decode_webp_bytes(bytes).expect("WebP decode should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        // 纯绿，补 alpha=255
        assert_eq!(&img.pixels[..4], &[0, 255, 0, 255]);
    }

    /// R1793：WebP 魔数（RIFF....WEBP）+ 无效正文 → 库解码失败。
    #[test]
    fn decode_webp_bytes_invalid_returns_err() {
        // RIFF + WEBP 魔数但无有效 VP8/VP8L chunk
        let bad = b"RIFF\x00\x00\x00\x00WEBPrest is garbage";
        let result = decode_webp_bytes(bad);
        assert!(result.is_err());
    }

    /// R1793：`is_webp_magic` 边界（≥12 字节、RIFF+WEBP 双魔数）。
    #[test]
    fn is_webp_magic_detects_riff_webp() {
        assert!(is_webp_magic(b"RIFF\x00\x00\x00\x00WEBPVP8 "));
        assert!(!is_webp_magic(b"RIFF\x00\x00\x00\x00")); // < 12 字节，无 WEBP
        assert!(!is_webp_magic(b"RIFF\x00\x00\x00\x00WAVI")); // RIFF 但非 WEBP
        assert!(!is_webp_magic(b"\x89PNG\r\n\x1a\n")); // PNG 不匹配
    }

    /// 分发器：按魔数字节/内容正确路由 PNG / JPEG / WebP / SVG / 未知格式。
    #[test]
    fn decode_image_bytes_dispatches_by_magic() {
        // PNG → 成功
        let png = red_2x2_png();
        let img = decode_image_bytes(&png).expect("PNG should dispatch and decode");
        assert_eq!(img.width, 2);

        // JPEG → 成功
        let jpeg = include_bytes!("testdata/green_4x3.jpg");
        let img = decode_image_bytes(jpeg).expect("JPEG should dispatch and decode");
        assert_eq!(img.width, 4);

        // WebP → 成功（RIFF....WEBP 魔数路由）
        let webp = include_bytes!("testdata/green_4x3.webp");
        let img = decode_image_bytes(webp).expect("WebP should dispatch and decode");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);

        // SVG → 成功（文本内容嗅探路由）
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\">\
                   <rect width=\"4\" height=\"3\" fill=\"rgb(0,255,0)\"/></svg>";
        let img = decode_image_bytes(svg).expect("SVG should dispatch and rasterize");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);

        // 未知魔数 → 错误（unsupported）
        let result = decode_image_bytes(b"GIF89a rest of gif");
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("unsupported"),
            "should report unsupported format"
        );
    }

    /// R1705：`decode_data_uri` base64 PNG → ImageData（renderer + wpt-runner 共用入口）。
    #[test]
    fn decode_data_uri_base64_png() {
        use base64::Engine;
        let png = red_2x2_png();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        let src = format!("data:image/png;base64,{b64}");
        let img = decode_data_uri(&src).expect("base64 PNG data URI should decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(&img.pixels[..4], &[255, 0, 0, 255]); // 红
    }

    #[test]
    fn decode_data_uri_bytes_supports_font_payloads() {
        assert_eq!(
            decode_data_uri_bytes("data:font/woff2;BASE64,d09GMg==").unwrap(),
            b"wOF2"
        );
        assert_eq!(
            decode_data_uri_bytes("data:application/font-sfnt,%00%01%00%00").unwrap(),
            [0, 1, 0, 0]
        );
        assert!(decode_data_uri_bytes("https://example.com/font.woff2").is_err());
    }

    /// R1705：无逗号的非法 data URI → Err（调用方降级，不阻断）。
    #[test]
    fn decode_data_uri_missing_comma_is_err() {
        assert!(decode_data_uri("data:image/png;base64").is_err());
    }

    /// 4×3 纯绿 SVG（含 `<?xml` 声明）栅格化往返：断言绿色主导 + alpha=255。
    #[test]
    fn decode_svg_bytes_green_4x3() {
        let svg = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                   <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\">\
                   <rect width=\"4\" height=\"3\" fill=\"rgb(0,255,0)\"/></svg>";
        let img = decode_svg_bytes(svg.as_bytes()).expect("SVG rasterize should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        let px = img.get_pixel(1, 1);
        assert!(px[1] > 200, "green channel should be high, got {}", px[1]);
        assert!(px[3] == 255, "alpha should be fully opaque, got {}", px[3]);
    }

    #[test]
    fn decode_svg_bytes_invalid_returns_err() {
        // 非 SVG XML → resvg 解析失败
        let result = decode_svg_bytes(b"<not-a-svg></not-a-svg>");
        assert!(result.is_err());
    }

    /// 绝对 width/height 的 SVG → 真固有尺寸 → BothAbs（走 image_sizes 路径）。
    #[test]
    fn svg_kind_absolute_dims_is_both_abs() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"100\" height=\"50\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::BothAbs);
    }

    /// 百分比 width/height + viewBox → ratio-only，ratio = viewBox w/h。
    #[test]
    fn svg_kind_percent_dims_uses_viewbox() {
        // aspect-ratio-intrinsic-size-007 驱动案：100%×100% + viewBox 7500×3750 → ratio 2.0
        let svg =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"100%\" viewBox=\"0 0 7500 3750\">\
                   <rect width=\"7500\" height=\"3750\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 仅 viewBox 无 width/height → ratio-only。
    #[test]
    fn svg_kind_viewbox_only() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\">\
                   <rect width=\"1000\" height=\"500\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 一维百分比、一维绝对 → 仍 ratio-only（非双绝对），ratio 来自 viewBox。
    #[test]
    fn svg_kind_mixed_percent_and_absolute() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"200\" height=\"100\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 无 width/height 也无 viewBox → no-ratio，无任何固有维。
    #[test]
    fn svg_kind_no_dims_no_viewbox_is_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect fill=\"green\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: None,
                height: None
            }
        );
    }

    /// 仅 width 绝对、无 height、无 viewBox → no-ratio，真实固有宽 = width 值，高 None。
    /// 驱动案：visudet width-50-no-ratio.svg（CSS §10.3.2 no-ratio replaced sizing）。
    #[test]
    fn svg_kind_width_only_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"orange\" width=\"100%\" height=\"100%\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: Some(50.0),
                height: None
            }
        );
    }

    /// 仅 height 绝对、无 width、无 viewBox → no-ratio，真实固有高 = height 值，宽 None。
    /// 驱动案：visudet height-25-no-ratio.svg。
    #[test]
    fn svg_kind_height_only_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"blue\" width=\"100%\" height=\"100%\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: None,
                height: Some(25.0)
            }
        );
    }

    /// R1438：height abs + width 缺失 + viewBox → ComputedIntrinsic（abs×ratio）。
    /// 驱动案：visudet height-25-ratio-2.svg（height="25" viewBox 1000×500，ratio 2 → (50,25)）。
    #[test]
    fn svg_kind_height_abs_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"fuchsia\" width=\"1000\" height=\"500\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::ComputedIntrinsic(50.0, 25.0));
    }

    /// R1438：width abs + height 缺失 + viewBox → ComputedIntrinsic（abs, abs/ratio）。
    /// 驱动案：visudet width-50-ratio-2.svg（width="50" viewBox 1000×500，ratio 2 → (50,25)）。
    #[test]
    fn svg_kind_width_abs_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"silver\" width=\"1000\" height=\"500\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::ComputedIntrinsic(50.0, 25.0));
    }

    /// R1438 gate：一维 abs + 另一维**百分比**（属性存在）+ viewBox → 仍 RatioOnly
    ///（不触发 ComputedIntrinsic，避 flex ratio-derivation 回归；mixed-percent 保持 R717）。
    #[test]
    fn svg_kind_mixed_percent_abs_viewbox_still_ratio_only() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"200\" height=\"100\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 端到端：百分比维 + viewBox SVG（RatioOnly）经 decode 保留 intrinsic_ratio（供 flex
    /// transferred-size；INLINE 由布局层 default object size 处理，不应用此 ratio）。
    #[test]
    fn decode_svg_percent_dims_keeps_intrinsic_ratio() {
        let svg =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"100%\" viewBox=\"0 0 7500 3750\">\
                   <rect width=\"7500\" height=\"3750\" fill=\"green\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), Some(2.0));
        assert_eq!(img.no_ratio_intrinsic(), None);
    }

    /// R2054：width-only no-ratio SVG 经 decode 后 no_ratio **保留真实 abs 固有维**
    ///（Some(50), None）——旧实现丢弃为 (None,None) 致 visudet width-50-no-ratio.svg
    /// 失去固有宽走全 default（应 50×150）。布局 no_ratio 分支用 w_opt/h_opt + unwrap_or
    /// 正确分派（显式 width 时 width=50，auto height 时 default 150）。
    #[test]
    fn decode_svg_width_only_no_ratio_sets_field() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"orange\" width=\"100%\" height=\"100%\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), Some((Some(50.0), None)));
    }

    /// R2054：height abs + viewBox ComputedIntrinsic SVG 经 decode → **computed_intrinsic**
    ///（→ image_sizes），使 INLINE auto+auto 用 (50,25)（chromium visudet all-auto：
    /// height-25-ratio-2 → 50×25）。旧实现设 intrinsic_ratio 致 layout ratio-only 分支
    /// INLINE 走 default 300×150（错）。computed_intrinsic 不再设 intrinsic_ratio（flex
    /// transferred-size 经 image_sizes aspect_ratio 推导）。
    #[test]
    fn decode_svg_height_abs_viewbox_sets_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"fuchsia\" width=\"1000\" height=\"500\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.computed_intrinsic(), Some((50.0, 25.0)));
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), None);
    }

    /// 端到端：双绝对 SVG（width+height 均 abs）经 decode 后**不**进 no_ratio（走 image_sizes
    /// 固有尺寸路径），intrinsic_ratio/no_ratio 均 None。
    #[test]
    fn decode_svg_both_abs_not_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"black\" width=\"100%\" height=\"100%\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, fill: u8) -> ImageData {
        let pixels = vec![fill; (w as usize) * (h as usize) * 4];
        ImageData::from_rgba(pixels, w, h).unwrap()
    }

    #[test]
    fn test_image_data_from_rgba() {
        let pixels = vec![255u8; 2 * 2 * 4];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.get_pixel(0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn test_image_data_from_rgba_wrong_size() {
        let pixels = vec![255u8; 10];
        let result = ImageData::from_rgba(pixels, 2, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_data_new_empty() {
        let img = ImageData::new_empty(4, 4);
        assert_eq!(img.pixels.len(), 4 * 4 * 4);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_image_data_size() {
        let img = ImageData::new_empty(10, 20);
        let size = img.size();
        assert_eq!(size.width, 10.0);
        assert_eq!(size.height, 20.0);
    }

    #[test]
    fn test_image_data_byte_size() {
        let img = ImageData::new_empty(3, 4);
        assert_eq!(img.byte_size(), 3 * 4 * 4);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 255));
        assert_eq!(cache.len(), 1);

        let img = cache.get(&key).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(cache.ref_count(&key), Some(2)); // insert gives 1, get adds 1
    }

    #[test]
    fn test_cache_release() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        assert_eq!(cache.ref_count(&key), Some(1));

        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_gc_removes_zero_ref() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        cache.gc();
        assert!(cache.ref_count(&key).is_none());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_gc_keeps_referenced() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 200));
        // ref_count is 1, should be kept
        cache.gc();
        assert!(cache.ref_count(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_entries() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let _key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let _key2 = cache.insert(make_image(1, 1, 20));
        let _key3 = cache.insert(make_image(1, 1, 30)); // triggers over max_entries

        cache.gc();
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_bytes() {
        let mut cache = ImageCache::new(100, 32); // 32 bytes max
        let key1 = cache.insert(make_image(2, 2, 10)); // 16 bytes
        let _key2 = cache.insert(make_image(2, 2, 20)); // 16 bytes = total 32
        assert_eq!(cache.total_bytes(), 32);

        // Access key1 so it's newer
        let _ = cache.get(&key1);

        // Insert another, total > 32
        let _key3 = cache.insert(make_image(2, 2, 30)); // total = 48
        cache.gc();
        assert!(cache.total_bytes() <= 32);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(1, 1, 0));
        cache.insert(make_image(1, 1, 1));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default() {
        let cache = ImageCache::default();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_release_nonexistent_is_noop() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        // Should not panic
        cache.release(&key);
    }

    #[test]
    fn test_cache_generation_increases_on_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert_eq!(cache.generation(), 1);
        cache.gc();
        assert_eq!(cache.generation(), 2);
    }

    #[test]
    fn test_cache_total_bytes() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(2, 2, 0)); // 2*2*4 = 16 bytes
        cache.insert(make_image(3, 3, 0)); // 3*3*4 = 36 bytes
        assert_eq!(cache.total_bytes(), 52);
    }

    #[test]
    fn test_image_key_new() {
        let key = ImageKey::new(42);
        assert_eq!(key.0, 42);
    }

    #[test]
    fn test_image_data_get_pixel_clamps_out_of_bounds() {
        let pixels = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        // 双线性采样在右/下边界可能传入 x=width 或 y=height，应钳制而非 panic。
        assert_eq!(img.get_pixel(2, 0), [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 2), [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(99, 99), [255, 255, 0, 255]);
    }

    #[test]
    fn test_image_data_get_pixel_mismatched_buffer_returns_transparent() {
        let img = ImageData {
            pixels: vec![255; 8],
            width: 2,
            height: 2,
            content_hash: 0,
            solid_color: None,
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
        };
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_image_data_get_pixel_various_positions() {
        // 2x2 image with distinct pixel values
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0), [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(1, 1), [255, 255, 0, 255]);
    }

    #[test]
    fn test_image_key_equality_and_hash() {
        let k1 = ImageKey::new(10);
        let k2 = ImageKey::new(10);
        let k3 = ImageKey::new(20);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(k1.clone());
        set.insert(k2.clone());
        assert_eq!(set.len(), 1);
        set.insert(k3.clone());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_cache_multiple_get_increments_ref_count() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        assert_eq!(cache.ref_count(&key), Some(1));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(2));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(3));
    }

    #[test]
    fn test_cache_release_saturating_sub() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(1, 1, 0));
        // ref_count starts at 1
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
        // Releasing below 0 should saturate at 0
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_clear_increases_generation() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.insert(make_image(1, 1, 0));
        cache.clear();
        assert_eq!(cache.generation(), 1);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_sequential_insert_unique_keys() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(1, 1, 20));
        let k3 = cache.insert(make_image(1, 1, 30));
        assert_ne!(k1, k2);
        assert_ne!(k2, k3);
        assert_eq!(cache.len(), 3);
        // Each has ref_count 1
        assert_eq!(cache.ref_count(&k1), Some(1));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.ref_count(&k3), Some(1));
    }

    #[test]
    fn test_image_data_from_rgba_large() {
        let pixels = vec![128u8; 100 * 100 * 4];
        let img = ImageData::from_rgba(pixels, 100, 100).unwrap();
        assert_eq!(img.byte_size(), 100 * 100 * 4);
        assert_eq!(img.get_pixel(50, 50), [128, 128, 128, 128]);
    }

    /// 测试 LRU 淘汰顺序：先插入（最旧世代）的条目应被优先淘汰
    #[test]
    fn test_cache_gc_lru_eviction_order() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let key2 = cache.insert(make_image(1, 1, 20)); // gen 0
        // 推进世代后插入 key3，使 key1/key2 有更旧的 last_access_gen
        cache.gc(); // gen -> 1，key1 和 key2 的 last_access_gen 仍为 0
        let key3 = cache.insert(make_image(1, 1, 30)); // last_access_gen = 1

        cache.gc(); // gen -> 2，应淘汰 gen=0 的条目（key1 和 key2 之一）
        assert_eq!(cache.len(), 2);
        // key3 最新（gen=1），一定保留
        assert!(cache.ref_count(&key3).is_some(), "最新的 key3 应保留");
        // key1 和 key2 同为 gen=0，淘汰其中一个即可
        let remaining = cache.ref_count(&key1).is_some() as usize + cache.ref_count(&key2).is_some() as usize;
        assert_eq!(remaining, 1, "key1 和 key2 中应恰好保留一个");
    }

    /// 测试多次 GC 后访问时间更新使条目免于被淘汰
    #[test]
    fn test_cache_gc_get_updates_lru_and_protects_entry() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));

        // 先执行一次 GC 使世代推进到 1
        cache.gc();
        // 此时 key1 和 key2 的 last_access_gen 仍为 0

        // 访问 key1 使其 last_access_gen 更新为 1
        let _ = cache.get(&key1);
        // key2 的 last_access_gen 仍为 0（更旧）

        // 插入第三个，触发超限
        let key3 = cache.insert(make_image(1, 1, 30)); // gen=1

        cache.gc(); // 世代推进到 2
        assert_eq!(cache.len(), 2);
        // key1 被访问过（gen=1），应保留；key2 最旧（gen=0），应被淘汰
        assert!(cache.ref_count(&key1).is_some(), "被访问过的 key1 应保留");
        assert!(cache.ref_count(&key2).is_none(), "未被访问的 key2 应被淘汰");
        assert!(cache.ref_count(&key3).is_some(), "key3 应保留");
    }

    /// 测试混合 release + get 模式下 GC 的正确性
    #[test]
    fn test_cache_gc_mixed_release_and_get_pattern() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));
        let key3 = cache.insert(make_image(1, 1, 30));

        // key1: release → ref_count=0（应被 GC 移除）
        cache.release(&key1);
        // key2: get → ref_count=2（应保留）
        let _ = cache.get(&key2);
        // key3: 保持 ref_count=1（应保留）

        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "ref_count=0 的 key1 应被移除");
        assert_eq!(cache.ref_count(&key2), Some(2), "key2 ref_count 应为 2");
        assert_eq!(cache.ref_count(&key3), Some(1), "key3 ref_count 应为 1");
        assert_eq!(cache.len(), 2);
    }

    /// 测试超大图片超过 max_bytes 时被淘汰
    #[test]
    fn test_cache_gc_single_image_exceeds_max_bytes() {
        let mut cache = ImageCache::new(10, 16); // 仅允许 16 字节
        let _key1 = cache.insert(make_image(1, 1, 10)); // 4 字节
        let _key2 = cache.insert(make_image(1, 1, 20)); // 4 字节
        assert_eq!(cache.total_bytes(), 8);

        // 插入一个 4x4（64 字节）的图片，远超 max_bytes
        let _key_big = cache.insert(make_image(4, 4, 255)); // 64 字节
        assert!(cache.total_bytes() > 16);

        cache.gc();
        // GC 应不断淘汰最旧条目直到总字节数 ≤ max_bytes
        // 即使淘汰所有旧条目后只剩 key_big (64 > 16)，也会继续淘汰
        // 最终 key_big 也会被淘汰（因为 64 > 16）
        assert!(cache.total_bytes() <= 16, "GC 后总字节数应不超过 max_bytes");
    }

    // -- 边界条件测试 --
    /// 测试 ImageKey 边界值
    #[test]
    fn test_image_key_boundary_values() {
        let k_min = ImageKey::new(0);
        assert_eq!(k_min.0, 0);
        let k_max = ImageKey::new(u64::MAX);
        assert_eq!(k_max.0, u64::MAX);
        assert_ne!(k_min, k_max);
    }

    /// 测试 ImageData::new_empty 零尺寸
    #[test]
    fn test_image_data_new_empty_zero_dims() {
        let img = ImageData::new_empty(0, 0);
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData::from_rgba 零宽度
    #[test]
    fn test_image_data_from_rgba_zero_width() {
        let result = ImageData::from_rgba(vec![], 0, 5);
        assert!(result.is_ok());
        let img = result.unwrap();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 5);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData get_pixel 1x1 图片
    #[test]
    fn test_image_data_get_pixel_1x1() {
        let pixels = vec![42, 84, 126, 255];
        let img = ImageData::from_rgba(pixels, 1, 1).unwrap();
        assert_eq!(img.get_pixel(0, 0), [42, 84, 126, 255]);
    }

    /// 测试 ImageCache gc 后再 insert
    #[test]
    fn test_cache_insert_after_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 100));
        cache.release(&k1);
        cache.gc();
        assert!(cache.is_empty());

        let k2 = cache.insert(make_image(1, 1, 200));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.len(), 1);
    }

    /// 测试 ImageCache release 然后 get（ref_count 从 0 回到 1）
    #[test]
    fn test_cache_release_then_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        // get 仍然能取回数据（条目还在，只是 ref_count=0）
        let img = cache.get(&key);
        assert!(img.is_some());
        assert_eq!(cache.ref_count(&key), Some(1));
    }

    /// 测试 ImageCache gc 在空缓存上
    #[test]
    fn test_cache_gc_empty() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 1);
    }

    /// 测试 ImageData::new_empty 所有像素透明
    #[test]
    fn test_image_data_new_empty_all_transparent() {
        let img = ImageData::new_empty(2, 2);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 1), [0, 0, 0, 0]);
    }

    /// 测试 ImageCache max_entries=1
    #[test]
    fn test_cache_max_entries_one() {
        let mut cache = ImageCache::new(1, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        // gc to advance generation so k1 has older gen
        cache.gc();
        let k2 = cache.insert(make_image(1, 1, 20));
        // Now len = 2 > max_entries = 1
        cache.gc();
        // Should evict oldest entry, keeping only 1
        assert!(cache.len() <= 1);
        assert!(cache.ref_count(&k2).is_some(), "较新的 k2 应保留");
        assert!(cache.ref_count(&k1).is_none(), "较旧的 k1 应被淘汰");
    }

    /// 测试连续插入相同内容的数据产生不同的缓存键，缓存中两个条目并存。
    ///
    /// 对 ImageCache 调用两次 insert 传入相同像素数据，应返回不同的 key，
    /// 缓存中两个条目独立存在，可通过各自的 key 分别访问。
    #[test]
    fn test_image_cache_double_insert_same_key() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        let data1 = make_image(2, 2, 128);
        let data2 = make_image(2, 2, 128); // 相同尺寸和填充值

        let key1 = cache.insert(data1);
        let key2 = cache.insert(data2);

        // 两次插入应返回不同的 key
        assert_ne!(key1, key2, "两次插入应返回不同的 key");
        assert_eq!(cache.len(), 2, "缓存中应有 2 个条目");

        // 两个 key 都能独立获取
        let img1 = cache.get(&key1);
        assert!(img1.is_some(), "key1 应能获取到图片");
        assert_eq!(img1.unwrap().width, 2);

        let img2 = cache.get(&key2);
        assert!(img2.is_some(), "key2 应能获取到图片");
        assert_eq!(img2.unwrap().width, 2);

        // 引用计数各被增加（insert=1 + get=1 = 2）
        assert_eq!(cache.ref_count(&key1), Some(2));
        assert_eq!(cache.ref_count(&key2), Some(2));

        // 释放 key1 后 GC，key1 被移除，key2 保留
        cache.release(&key1);
        cache.release(&key1); // release the extra ref from get
        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "key1 应被 GC 移除");
        assert!(cache.ref_count(&key2).is_some(), "key2 应保留");
    }

    /// 测试 max_entries=0 时 GC 会清除所有条目
    ///
    /// 当 max_entries 设置为 0 时，每次 GC 都会淘汰所有条目，
    /// 因为任何条目数（>0）都超过限制。insert 本身不会拒绝插入，
    /// 但 GC 后缓存一定为空。
    #[test]
    fn test_image_cache_zero_max_entries() {
        let mut cache = ImageCache::new(0, 1024 * 1024);

        // 插入多个条目
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(2, 2, 20));
        let k3 = cache.insert(make_image(3, 3, 30));
        assert_eq!(cache.len(), 3, "插入后应有 3 个条目");

        // GC 后所有条目因 max_entries=0 被淘汰
        cache.gc();
        assert!(cache.is_empty(), "GC 后缓存应为空");
        assert_eq!(cache.len(), 0);
        assert!(cache.ref_count(&k1).is_none(), "k1 应被淘汰");
        assert!(cache.ref_count(&k2).is_none(), "k2 应被淘汰");
        assert!(cache.ref_count(&k3).is_none(), "k3 应被淘汰");

        // 再次插入后立即 GC，同样被淘汰
        let k4 = cache.insert(make_image(1, 1, 40));
        cache.gc();
        assert!(cache.is_empty(), "再次 GC 后缓存仍应为空");
        assert!(cache.ref_count(&k4).is_none());
    }

    /// 测试 ImageData::from_rgba 使用 0x0 尺寸的空数据创建成功
    ///
    /// 当 width=0、height=0 时，expected 字节数为 0，
    /// 传入空 Vec 应成功创建一个零尺寸图片。
    /// 验证零尺寸图片的 size() 返回 (0,0)，byte_size() 返回 0。
    #[test]
    fn test_image_data_from_rgba_zero_dimensions() {
        let img = ImageData::from_rgba(vec![], 0, 0).expect("0x0 应创建成功");
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.pixels.is_empty());
        assert_eq!(img.byte_size(), 0);

        let size = img.size();
        assert_eq!(size.width, 0.0);
        assert_eq!(size.height, 0.0);
    }

    /// 测试 GC 在 max_bytes=0 时清除所有条目
    ///
    /// 当最大字节限制为 0 时，任何非零大小的图片都会被 GC 淘汰。
    #[test]
    fn test_cache_gc_zero_max_bytes() {
        let mut cache = ImageCache::new(100, 0);
        let k1 = cache.insert(make_image(1, 1, 10));
        assert_eq!(cache.len(), 1);

        cache.gc();
        assert!(cache.is_empty(), "max_bytes=0 时 GC 应淘汰所有条目");
        assert!(cache.ref_count(&k1).is_none());
    }

    /// 测试释放所有条目后 GC 清空缓存
    ///
    /// 所有条目 ref_count 降为 0 后，GC 应移除全部。
    #[test]
    fn test_cache_gc_after_release_all() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(2, 2, 20));
        let k3 = cache.insert(make_image(3, 3, 30));

        cache.release(&k1);
        cache.release(&k2);
        cache.release(&k3);

        cache.gc();
        assert!(cache.is_empty(), "释放全部引用后 GC 应清空缓存");
    }

    /// 测试插入 1x1 图片后 total_bytes 正确
    ///
    /// 1x1 RGBA 图片占用 4 字节。
    #[test]
    fn test_cache_total_bytes_single_pixel() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(1, 1, 255));
        assert_eq!(cache.total_bytes(), 4, "1x1 RGBA 图片应为 4 字节");
    }

    /// 测试多次 clear 后 generation 持续递增
    #[test]
    fn test_cache_clear_generation_multiple() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.clear();
        assert_eq!(cache.generation(), 1);
        cache.clear();
        assert_eq!(cache.generation(), 2);
        cache.clear();
        assert_eq!(cache.generation(), 3);
    }
}
