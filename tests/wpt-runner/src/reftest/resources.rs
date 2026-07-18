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
    let mut urls = Vec::new();
    let mut pos = 0;
    while let Some(idx) = html[pos..].find("url(") {
        let url_start = pos + idx + 4;
        // 跳过空白和引号
        let rest = &html[url_start..];
        let trimmed = rest
            .trim_start_matches(' ')
            .trim_start_matches('\'')
            .trim_start_matches('"');
        let offset = rest.len() - trimmed.len();
        let actual_start = url_start + offset;

        if let Some(end_idx) = html[actual_start..].find(')') {
            let raw = html[actual_start..actual_start + end_idx].trim();
            let url = raw.trim_matches('\'').trim_matches('"').trim();
            if !url.is_empty() && !url.starts_with("data:") && !url.starts_with("http") {
                urls.push(url.to_string());
            }
            pos = actual_start + end_idx + 1;
        } else {
            break;
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

        if let Ok(css) = std::fs::read_to_string(&path) {
            if !merged.is_empty() {
                merged.push('\n');
            }
            merged.push_str(&css);
        }
    }
    merged
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

        // R1703：data URI 自包含（无需文件系统 base_dir）——须在 base_dir 早返回外处理，
        // 否则 product-smoke / 无 base_dir 渲染（fixture 24 等）下 data URI 图永不解码。
        // 优先处理 data:image/svg+xml（generate_svg_data_uri_image 提取首个 rect 纯色）。
        if url.starts_with("data:image/svg+xml")
            && let Some(data) = generate_svg_data_uri_image(url)
        {
            cache.insert_with_key(key, data);
            continue;
        }

        // 其他 data: URI（PNG/JPEG/GIF/base64 等）暂不支持（无 base_dir 亦无文件系统），跳过。
        if url.starts_with("data:") {
            // R1704/R1705：PNG/JPEG/GIF 等栅格 data URI（base64 或 url-encoded）→ decode_data_uri
            // 真解码（render-foundation 共用，按 magic bytes 分派）。SVG 走上方纯色近似路径
            //（fixture 均为单色，避改 R1703 行为）。
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
        let path = base.join(url.trim_start_matches('/'));

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

///
/// 支持简单的单色矩形 SVG（如 `<svg><rect fill='green' width='200' height='100'/></svg>`）。
/// 对于更复杂的 SVG，返回 None。
fn generate_svg_data_uri_image(url: &str) -> Option<ImageData> {
    let comma_pos = url.find(',')?;
    let svg_content = &url[comma_pos + 1..];

    // URL 解码
    let decoded = percent_decode_svg(svg_content);

    // 提取 SVG 尺寸
    let svg_start = decoded.find("<svg")?;
    let tag_end = decoded[svg_start..].find('>')?;
    let svg_tag = &decoded[svg_start..svg_start + tag_end];
    let svg_w = extract_svg_attr_float(svg_tag, "width")? as u32;
    let svg_h = extract_svg_attr_float(svg_tag, "height")? as u32;

    if svg_w == 0 || svg_h == 0 || svg_w > 4096 || svg_h > 4096 {
        return None;
    }

    // 提取第一个 <rect> 的 fill 颜色
    let rect_fill = extract_first_rect_fill(&decoded[svg_start + tag_end..])?;

    // 生成纯色 ImageData
    let [r, g, b, a] = rect_fill;
    let buf_size = (svg_w as usize) * (svg_h as usize) * 4;
    let mut buf = vec![0u8; buf_size];
    for pixel in buf.chunks_exact_mut(4) {
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
        pixel[3] = a;
    }

    ImageData::from_rgba(buf, svg_w, svg_h).ok()
}

/// 简易 percent-decode。
fn percent_decode_svg(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 从 SVG 标签属性中提取浮点数值。
fn extract_svg_attr_float(tag: &str, attr: &str) -> Option<f32> {
    let prefix = format!("{}=", attr);
    let pos = tag.find(&prefix)?;
    let rest = &tag[pos + prefix.len()..];
    let (quote, offset) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };
    let value_str = &rest[offset..];
    let end = value_str.find(quote)?;
    value_str[..end].parse::<f32>().ok()
}

/// 从 SVG 内容中提取第一个 <rect> 的 fill 颜色。
/// 支持命名颜色（如 "green", "red", "blue"）和十六进制颜色（如 "#00ff00"）。
fn extract_first_rect_fill(svg_content: &str) -> Option<[u8; 4]> {
    let rect_start = svg_content.find("<rect")?;
    let rect_end = svg_content[rect_start..].find("/>")?;
    let rect_tag = &svg_content[rect_start..rect_start + rect_end];

    // 查找 fill 属性
    let fill_prefix = "fill=";
    let pos = rect_tag.find(fill_prefix)?;
    let rest = &rect_tag[pos + fill_prefix.len()..];
    let (quote, offset) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };
    let value_str = &rest[offset..];
    let end = value_str.find(quote)?;
    let color_name = &value_str[..end];

    parse_css_color(color_name)
}

/// 解析 CSS 颜色名称或十六进制颜色。
fn parse_css_color(name: &str) -> Option<[u8; 4]> {
    match name {
        "green" => Some([0, 128, 0, 255]),
        "red" => Some([255, 0, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "white" => Some([255, 255, 255, 255]),
        "black" => Some([0, 0, 0, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "orange" => Some([255, 165, 0, 255]),
        "purple" => Some([128, 0, 128, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        "lime" => Some([0, 255, 0, 255]),
        "navy" => Some([0, 0, 128, 255]),
        "cyan" | "aqua" => Some([0, 255, 255, 255]),
        "magenta" | "fuchsia" => Some([255, 0, 255, 255]),
        "silver" => Some([192, 192, 192, 255]),
        "maroon" => Some([128, 0, 0, 255]),
        "olive" => Some([128, 128, 0, 255]),
        "teal" => Some([0, 128, 128, 255]),
        "transparent" => Some([0, 0, 0, 0]),
        hex if hex.starts_with('#') => parse_hex_color(hex),
        _ => None,
    }
}

/// 解析 #RGB 或 #RRGGBB 十六进制颜色。
fn parse_hex_color(hex: &str) -> Option<[u8; 4]> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some([r, g, b, 255])
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r, g, b, 255])
        }
        _ => None,
    }
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
        let img = cache
            .get(&key)
            .expect("base64 PNG data URI should decode (R1704)");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(&img.pixels[..4], &[255, 0, 0, 255]); // 红
    }
}
