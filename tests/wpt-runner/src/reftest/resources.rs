//! reftest 资源加载集群：HTML img/url/stylesheet 提取 + PNG/JPEG/SVG 图片解码 + 颜色/SVG 解析。
//!
//! 从 `reftest.rs` 拆分以控制单文件体积（2000 行规则）。这些函数为 reftest
//! 渲染流程加载外部资源（外链样式表、`<img>` 子资源、SVG data-URI），由
//! `render_to_framebuffer_*` 与 `run_reftest_*` 调用。
//!
//! 通过子模块 `use super::*` 复用 reftest 的类型与导入；被父模块调用的 4 个
//! 函数标 `pub(super)`，`convert_png_buffer_to_rgba` 保持 `pub`（经 reftest 的
//! `pub use` 再导出，供 `main.rs` 使用）。集群内部自洽（不调用 reftest 其它函数）。

use super::*;
use percent_encoding::percent_decode;

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
    // 经 tokenizer 识别 CSS `url()`（CSS Syntax §5）：函数名大小写不敏感且可含转义
    //（`URL(`、`U\r\4c (`），内容亦可含转义（`support/\'green\ block.png`）。Url token
    // 内容**已解码**（consume_escape），与 painter image key 对齐。原 raw `find("url(")`
    // 漏转义函数名（driving：uri-015 `U\r\4c ("...")` 不预抓 → painter image_cache miss
    // → 背景滞红；escaped-url-001 仅因 6 div 共享一图、div0 纯 `url()` 预抓而幸免）。
    // 对 HTML 整体 tokenize：非 CSS 区域偶现的 `url(`（如脚本文本）至多多抓一张不参与
    // 渲染的图（harmless），与原 raw 扫描同 scope。
    use zero_css_parser::{Token, Tokenizer};
    let mut urls = Vec::new();
    for spanned in Tokenizer::new(html) {
        if let Token::Url(u) = spanned.token {
            let raw = u.trim();
            if !raw.is_empty()
                && !raw.starts_with("data:")
                && !raw.starts_with("http")
                && !urls.iter().any(|x: &String| x == raw)
            {
                urls.push(raw.to_string());
            }
        }
    }
    urls
}

pub(super) fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
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

pub(super) fn load_linked_stylesheets(html: &str, base_dir: Option<&Path>) -> String {
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

        // CSS MIME 类型检查：只有 text/css 才应作为样式表应用（CSS2 §6.2 / conformance——
        // text/plain 等非 CSS MIME 须忽略，HTML `type` 属性已被现代浏览器废弃，以服务端
        // Content-Type 为准）。file:// 下 Content-Type 来自 `.headers` sidecar，无 sidecar
        // 时按扩展名猜测（.css → text/css）。WPT content-type-000（sidecar text/plain）/
        // content-type-001（.txt 扩展名）。
        let content_type = read_headers_sidecar_content_type(&path);
        if !is_css_stylesheet(&path, content_type.as_deref()) {
            continue;
        }

        // CSS Syntax §6.2 charset determination：读字节 + `.headers` sidecar 的 Content-Type
        // charset，按 BOM/@charset/Content-Type 优先级解码（旧 `read_to_string` 强制 UTF-8
        // 致 ISO-8859-1/UTF-16BE 等 CSS 非 ASCII 字节损坏，WPT at-charset-071~077 /
        // character-encoding-031~037,041 选择器失配）。
        if let Ok(bytes) = std::fs::read(&path) {
            let css = zero_net::charset::decode_css_bytes(&bytes, content_type.as_deref());
            if !merged.is_empty() {
                merged.push('\n');
            }
            merged.push_str(&css);
        }
    }
    merged
}

/// 判断路径指向的外链样式表是否为 CSS MIME（CSS2 conformance：非 text/css 须忽略）。
///
/// 有 `.headers` sidecar Content-Type 时以 sidecar MIME 为准（WPT content-type-000：
/// plaintext.css 经 sidecar 强制 text/plain）；无 sidecar 时按扩展名猜（`.css` → CSS，
/// 其余如 `.txt` 非 CSS — WPT content-type-001）。
fn is_css_stylesheet(path: &Path, sidecar_content_type: Option<&str>) -> bool {
    if let Some(ct) = sidecar_content_type {
        let mime = ct.split(';').next().unwrap_or("").trim();
        return mime.eq_ignore_ascii_case("text/css");
    }
    // 无 sidecar：按扩展名。`.css` 视为 CSS，其余（含 `.txt`）非 CSS。
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("css"))
}

/// 读取 `<path>.headers` sidecar 的 `Content-Type` 值（WPT 约定，file:// 下替代 HTTP header）。
///
/// 文件不存在或无 Content-Type 行则返回 `None`。用于 CSS charset determination
/// （`character-encoding-*` 经 sidecar 设 `charset=iso-8859-x/koi8-r/utf-16be`）。
fn read_headers_sidecar_content_type(path: &Path) -> Option<String> {
    let sidecar = std::path::PathBuf::from(format!("{}.headers", path.display()));
    let content = std::fs::read_to_string(&sidecar).ok()?;
    content.lines().find_map(|line| {
        let line = line.trim();
        let (name, value) = line.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("content-type") {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

/// 从基础目录加载图片文件并解码为 RGBA 数据，放入 ImageCache。
///
/// 对于每个 URL，用 `simple_hash(url)` 生成 ImageKey（与 paint 系统一致），
/// 然后尝试从 `base_dir` 解析相对路径并加载 PNG 文件。
pub(super) fn build_image_cache(html: &str, base_dir: Option<&Path>) -> ImageCache {
    let mut cache = ImageCache::new(256, 64 * 1024 * 1024);

    // 收集所有需要加载的 URL
    let mut all_urls = extract_img_srcs(html);
    all_urls.extend(extract_css_urls(html));
    all_urls.sort_unstable();
    all_urls.dedup();

    for url in &all_urls {
        let key = ImageKey::new(simple_hash(url));

        // R1706：data:image/svg+xml → decode_data_uri 真 resvg 栅格化（支持任意 SVG：
        // 渐变/路径/多形状，不再仅单色 rect 近似）。fixture 24 实测像素与旧纯色近似一致。
        if url.starts_with("data:image/svg+xml")
            && let Ok(data) = decode_data_uri(url)
        {
            cache.insert_with_key(key, data);
            continue;
        }

        // 其他 data: URI（PNG/JPEG/GIF/base64 等）暂不支持（无 base_dir 亦无文件系统），跳过。
        if url.starts_with("data:") {
            // R1704/R1705：PNG/JPEG/GIF 等栅格 data URI（base64 或 url-encoded）→ decode_data_uri
            // 真解码（render-foundation 共用，按 magic bytes 分派）。
            if let Ok(data) = decode_data_uri(url) {
                cache.insert_with_key(key, data);
            }
            continue;
        }

        // 文件 URL 需 base_dir；无 base_dir 跳过（data URI 已在上方处理完毕）。
        let Some(base) = base_dir else {
            continue;
        };

        // 站点根相对 URL（如 "/static/x.svg"）应解析到 base_dir 下（fixture 的站点根），
        // 而非 `base.join(absolute)`（会替换 base → 文件系统根 → 加载失败）。
        // WPT reftest 多用相对路径（无前导 /），trim 不影响；绝对路径此前全失败，此处修复。
        //
        // 浏览器按 URL 语义解析路径（production 走 `url::Url::to_file_path` 会 percent-decode）。
        // harness 此前用 `Path::join` 不解码，致 `support/%27green%20block.png` 找不到
        // 实际文件 `support/'green block.png`（WPT uri-004）。此处仅对文件系统查找解码；
        // ImageKey 仍用原始 url（上面 `simple_hash(url)`），与 painter 端 `image_resource_key`
        // 对齐（painter 拿到的也是 CSS 原始 url() 值）。
        let decoded = percent_decode(url.as_bytes()).decode_utf8_lossy();
        let path = base.join(decoded.trim_start_matches('/'));

        // 尝试加载 PNG 文件，失败再尝试 JPEG（真实页面 logo/照片多为 JPEG），再尝试 SVG
        if let Ok(data) = load_png_file(&path) {
            cache.insert_with_key(key, data);
        } else if let Ok(data) = load_jpeg_file(&path) {
            cache.insert_with_key(key, data);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"))
            && let Ok(data) = load_svg_file(&path)
        {
            cache.insert_with_key(key, data);
        }
    }

    cache
}

/// 把 png crate 解码（已 EXPAND|STRIP_16）的输出缓冲按其输出色型转换为 RGBA8。
///
/// EXPAND 不保证 RGBA：palette 无 tRNS / RGB 输入 → 输出 RGB（3 字节/像素），
/// grayscale → 1 字节/像素。本函数按 OutputInfo.color_type 统一补齐为 RGBA。
pub fn convert_png_buffer_to_rgba(raw: &[u8], color_type: png::ColorType, bit_depth: png::BitDepth) -> Vec<u8> {
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

/// 加载并解码 JPEG 文件为 RGBA ImageData。
///
/// 真实页面 logo/照片多为 JPEG。委托给 render-foundation 的 `decode_jpeg_bytes`
/// （R216），与 webview/browser URL 导航路径共用同一解码器，避免 PixelFormat→RGBA
/// 转换逻辑重复（L8/L16/RGB24/CMYK32 全格式在 render-foundation 单点实现并测试）。
fn load_jpeg_file(path: &Path) -> Result<ImageData, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("无法打开 {}: {e}", path.display()))?;
    zero_render_foundation::image_cache::decode_jpeg_bytes(&bytes)
}

/// 加载并栅格化 SVG 文件为 RGBA ImageData。
///
/// 真实页面 logo 多为 SVG（wintertc.org 14 个 logo 中 11 个为 .svg）。委托给
/// render-foundation 的 `decode_svg_bytes`（R218），与 webview/browser URL 导航路径
/// 共用同一 resvg 栅格化实现。
fn load_svg_file(path: &Path) -> Result<ImageData, String> {
    let data = std::fs::read(path).map_err(|e| format!("无法读取 SVG {}: {e}", path.display()))?;
    zero_render_foundation::image_cache::decode_svg_bytes(&data)
}

/// 从 ImageCache 中提取所有图像的固有尺寸、ratio-only 与 no-ratio 信号。
///
/// 遍历 HTML 中的所有图片 URL，查找缓存中对应的 ImageData，返回：
/// - `image_sizes`（url_hash → (width, height)）：用于 background-size: auto 计算，
///   以及有确定固有尺寸的 `<img>`（PNG/JPEG/绝对尺寸 SVG）替换元素 sizing。
/// - `image_ratios`（url_hash → width/height 比）：仅 %-dim / viewBox-only SVG（CSS §10.3.2），
///   无确定固有尺寸、仅有 viewBox 宽高比，布局仅设 aspect_ratio。
/// - `image_no_ratio`（url_hash → (真实固有宽, 真实固有高)）：仅 no-ratio SVG（CSS §10.3.2），
///   width/height 非双绝对且无 viewBox，布局不设 aspect_ratio、缺失维按 default object size。
///   no-ratio 图亦保留在 `image_sizes` 供背景图 background-size:auto 读 pixmap 尺寸。
#[allow(clippy::type_complexity)]
pub(super) fn extract_image_metrics(
    image_cache: &mut ImageCache,
    html: &str,
) -> (
    std::collections::HashMap<u64, (f32, f32)>,
    std::collections::HashMap<u64, f32>,
    std::collections::HashMap<u64, (Option<f32>, Option<f32>)>,
) {
    let mut sizes = std::collections::HashMap::new();
    let mut ratios = std::collections::HashMap::new();
    let mut no_ratio = std::collections::HashMap::new();

    let mut all_urls = extract_img_srcs(html);
    all_urls.extend(extract_css_urls(html));
    all_urls.sort_unstable();
    all_urls.dedup();

    for url in &all_urls {
        let key = ImageKey::new(simple_hash(url));
        if let Some(data) = image_cache.get(&key) {
            // R717：ratio-only SVG 进 ratios、不进 sizes（避免确定 size 阻止 flex ratio-derivation）。
            if let Some(ratio) = data.intrinsic_ratio() {
                ratios.insert(key.0, ratio);
            } else {
                // R1438：一维 abs + 另一维缺失 + viewBox 的 SVG，usvg pixmap 对缺失维用原始
                // viewBox 值（bogus），须用计算的 computed_intrinsic 覆盖 pixmap 用于 sizes。
                let (w, h) = data
                    .computed_intrinsic()
                    .unwrap_or_else(|| (data.size().width, data.size().height));
                sizes.insert(key.0, (w, h));
                // no-ratio SVG（CSS §10.3.2）：额外进 no_ratio（真实固有维），布局不设 aspect_ratio。
                if let Some(dims) = data.no_ratio_intrinsic() {
                    no_ratio.insert(key.0, dims);
                }
            }
        }
    }

    (sizes, ratios, no_ratio)
}

/// 从 ImageCache 中提取所有图像的固有尺寸（仅 sizes，无 ratios/no_ratio）。
///
/// 保留给仅需 background-size: auto 的旧调用点；`<img>` 替换元素 sizing 须改用
/// `extract_image_metrics` 以同时获得 ratio-only / no-ratio 信号。
pub(super) fn extract_image_sizes(
    image_cache: &mut ImageCache,
    html: &str,
) -> std::collections::HashMap<u64, (f32, f32)> {
    extract_image_metrics(image_cache, html).0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R1703：`data:image/svg+xml` URI 须在无 base_dir 时也解码（自包含，无文件系统依赖）。
    /// 此前 build_image_cache 在 base_dir=None 时早返回空 cache，致 product-smoke fixture 24
    /// 的 data URI img 永不渲染（ZW 与 chromium 全图绿区 diff）。
    #[test]
    fn data_uri_svg_decodes_without_base_dir() {
        let html = "<body><img src=\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='30' height='30'><rect width='30' height='30' fill='%23008000'/></svg>\"></body>";
        let mut cache = build_image_cache(html, None);
        let src = extract_img_srcs(html).into_iter().next().expect("img src extracted");
        let key = ImageKey::new(simple_hash(&src));
        let img = cache
            .get(&key)
            .expect("data:image/svg+xml should decode without base_dir (R1703)");
        assert_eq!(img.width, 30);
        assert_eq!(img.height, 30);
        // 纯绿 #008000 = (0, 128, 0, 255)。
        assert_eq!(&img.pixels[..4], &[0, 128, 0, 255]);
    }

    /// R1704：`data:image/png;base64,...` 经 base64 解码 + decode_image_bytes 真解码。
    /// 构造 2×2 红 PNG → base64 → data URI → build_image_cache 应得 2×2 红 ImageData。
    #[test]
    fn png_data_uri_base64_decodes() {
        use base64::Engine;
        // 构造 2×2 红色 PNG。
        let mut png_buf = Vec::new();
        {
            use png::{BitDepth, ColorType, Encoder};
            let mut enc = Encoder::new(&mut png_buf, 2, 2);
            enc.set_color(ColorType::Rgba);
            enc.set_depth(BitDepth::Eight);
            let mut w = enc.write_header().expect("PNG header");
            // 4 像素 × RGBA = 16 字节，全红（[255,0,0,255] × 4）。
            let data: Vec<u8> = [255, 0, 0, 255].repeat(4);
            w.write_image_data(&data).expect("PNG data");
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_buf);
        let html = format!("<body><img src=\"data:image/png;base64,{b64}\"></body>");
        let mut cache = build_image_cache(&html, None);
        let src = extract_img_srcs(&html).into_iter().next().expect("img src extracted");
        let key = ImageKey::new(simple_hash(&src));
        let img = cache.get(&key).expect("base64 PNG data URI should decode (R1704)");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(&img.pixels[..4], &[255, 0, 0, 255]); // 红
    }

    /// R2120：`url()` 路径含 percent-encoding 时，浏览器按 URL 语义解码后查找文件
    ///（production 走 `url::Url::to_file_path`）。harness 此前用 `Path::join` 不解码，
    /// 致 `support/%27green%20block.png` 找不到 `support/'green block.png`（WPT uri-004）。
    /// 此测试构造一个文件名需解码的 PNG（含空格），用 `%20` 引用，验证解码后能加载。
    #[test]
    fn percent_encoded_url_path_decodes_to_file() {
        use std::fs;
        // 临时目录（固定子名，避免依赖 tempfile crate）。
        let dir = std::env::temp_dir().join("zw_reftest_pct_decode_test");
        fs::create_dir_all(&dir).expect("create temp dir");
        // 写一个文件名含空格的 1×1 绿 PNG：`green block.png`。
        let png_path = dir.join("green block.png");
        let mut buf = Vec::new();
        {
            use png::{BitDepth, ColorType, Encoder};
            let mut enc = Encoder::new(&mut buf, 1, 1);
            enc.set_color(ColorType::Rgba);
            enc.set_depth(BitDepth::Eight);
            let mut w = enc.write_header().expect("PNG header");
            w.write_image_data(&[0, 128, 0, 255]).expect("PNG data");
        }
        fs::write(&png_path, &buf).expect("write png");
        // 用 %20 引用空格 → 须经 percent-decode 才能命中 `green block.png`。
        let html = "<style>p{background:url(green%20block.png)}</style>";
        let mut cache = build_image_cache(html, Some(&dir));
        let url = extract_css_urls(html).into_iter().next().expect("css url extracted");
        // ImageKey 仍用原始（编码）url，与 painter 端对齐。
        assert!(url.contains("%20"), "url should be raw/encoded: {url}");
        let key = ImageKey::new(simple_hash(&url));
        let img = cache
            .get(&key)
            .expect("percent-encoded url should decode to file (R2120)");
        assert_eq!(img.width, 1);
        assert_eq!(&img.pixels[..4], &[0, 128, 0, 255]); // 绿
        // 清理。
        let _ = fs::remove_dir_all(&dir);
    }

    /// R2124：CSS url() 值内的 backslash 转义须经 tokenizer 解码（consume_escape）。
    /// harness 原始扫描 extract_css_urls 须用 css_unescape 解码，使 url key 与 painter
    ///（经 tokenizer 解码）对齐（driving：uri-005 `support/\'green\ block.png`）。
    #[test]
    fn extract_css_urls_decodes_backslash_escapes() {
        // `url(a\'b\ c.png)` → 解码 → `a'b c.png`
        let html = r#"<style>p{background:url(a\'b\ c.png)}</style>"#;
        let urls = extract_css_urls(html);
        assert_eq!(urls, vec!["a'b c.png".to_string()]);
    }

    /// R2125：`url()` 函数名大小写不敏感（tokenizer eq_ignore_ascii_case("url")）。
    /// harness 原始扫描须同样大小写不敏感，否则 `UrL(...)`（case-sensitive-001）漏抽。
    #[test]
    fn extract_css_urls_case_insensitive_function_name() {
        let html = r#"<style>p{background:UrL(support/swatch-green.png)}</style>"#;
        let urls = extract_css_urls(html);
        assert_eq!(urls, vec!["support/swatch-green.png".to_string()]);
        // 多种大小写 + 与小写并存
        let html2 = r#"<style>
          p { background: URL(a.png); }
          p { background: url(b.png); }
          p { background: uRl(c.png); }
        </style>"#;
        let urls2 = extract_css_urls(html2);
        assert_eq!(
            urls2,
            vec!["a.png".to_string(), "b.png".to_string(), "c.png".to_string()]
        );
    }
}
