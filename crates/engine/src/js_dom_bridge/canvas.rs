//! Canvas 2D host 操作派发。从 js_dom_bridge.rs 拆出（R2974，文件大小治理 slice 2）。
//! `canvas_context_op(reg, handle, op, args)` 经 `__zw_canvas_op` 回调派发全部 Canvas 2D 操作
//!（路径/矩形/文本/图像/变换/合成/像素/状态），辅以值解析 helper（颜色/line-join/line-cap/
//! composite/image-data wire）。纯 zero_canvas/zero_render_foundation 类型，无 DOM/选择器依赖。
//! pub canvas_context_op 经 `pub use canvas::*` 重导出，register_dom_callbacks 调用点零改动。

use std::collections::HashMap;

/// Canvas host 注册表：上下文表 + 渐变表 + Path2D 表。
///
/// 上下文由 `getContext('2d')` 创建（`getContext2d` op），按 id 索引。
/// 渐变由 `createLinearGradient`/`createRadialGradient`/`createConicGradient` 创建，独立 id 命名空间
///（spec：CanvasGradient 为独立对象，可被任意 context 的 fillStyle/strokeStyle 引用）。`addColorStop`
/// 经渐变 id 变更停止点；`setFillStyleGradient`/`setStrokeStyleGradient` 经渐变 id 查表克隆到 context 样式。
/// Path2D（R3306）由 `createPath` 创建（`new Path2D()`），独立 id 命名空间；ctx.fill(path)/stroke(path)/
/// clip(path) 经 path id 查表取 Path2D 引用调 fill_path/stroke_path/clip_path（替代 ctx 当前路径）。
pub struct CanvasRegistry {
    /// 下一个 context id（从 1 起）。
    pub next_ctx_id: u64,
    /// context id → CanvasContext。
    pub contexts: HashMap<u64, zero_canvas::CanvasContext>,
    /// 下一个 gradient id（从 1 起，与 context id 独立命名空间）。
    pub next_grad_id: u64,
    /// gradient id → CanvasStyle（仅渐变变体）。
    pub gradients: HashMap<u64, zero_canvas::CanvasStyle>,
    /// R34xx：gradient id → 引用它的 (context id, 槽位) 列表（0=fill, 1=stroke）。
    /// addColorStop 后对引用 context live 重放（spec：渐变对象 live——改停止点影响后续
    /// 绘制，2d.gradient.object.update）。
    pub grad_refs: HashMap<u64, Vec<(u64, u8)>>,
    /// R34xx：共享字体加载器（headless/testharness 路径——webview load_html 把 @font-face
    /// 字体字节 load_font + register_family_alias 进来；canvas text 真文本光栅消费）。
    pub font_loader: std::sync::Arc<std::sync::Mutex<zero_render_foundation::font::loader::FontLoader>>,
    /// 下一个 Path2D id（从 1 起，独立命名空间，R3306）。
    pub next_path_id: u64,
    /// path id → Path2D（R3306）。
    pub paths: HashMap<u64, zero_canvas::Path2D>,
}

impl CanvasRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self {
            next_ctx_id: 1,
            contexts: HashMap::new(),
            next_grad_id: 1,
            gradients: HashMap::new(),
            grad_refs: HashMap::new(),
            next_path_id: 1,
            paths: HashMap::new(),
            font_loader: std::sync::Arc::new(std::sync::Mutex::new(
                zero_render_foundation::font::loader::FontLoader::new(),
            )),
        }
    }
}

impl Default for CanvasRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// canvas 颜色串 → render Color（复用 CSS 颜色解析：named/hex/rgb/hsl 等）。解析失败回落黑色。
fn parse_canvas_color(s: &str) -> zero_render_foundation::color::Color {
    try_parse_canvas_color(s).unwrap_or_else(|| zero_render_foundation::color::Color::rgba(0, 0, 0, 255))
}

/// R34xx：可失败颜色解析（无效返回 None，调用方保持旧值——spec 忽略无效 strokeStyle/fillStyle）。
fn try_parse_canvas_color(s: &str) -> Option<zero_render_foundation::color::Color> {
    zero_css_parser::values::parse_color(s.trim()).map(|cv| crate::color_value_to_render(&cv))
}

/// R34xx（wide-gamut 目录）：`color(display-p3 c1 c2 c3[/ alpha])` 直接取 p3 通道
///（0-1 或 %；alpha 缺省 1）——p3 画布的 fillStyle 已处画布空间，免 sRGB 二次转换
///（2d.color.space.p3.fillText 的 color(display-p3 100% 0 0) → (255,0,0)）。
fn parse_p3_channels(s: &str) -> Option<zero_render_foundation::color::Color> {
    let t = s.trim();
    let inner = t.strip_prefix("color(")?.strip_suffix(')')?;
    let mut parts = inner.split_whitespace();
    let space = parts.next()?;
    if !space.eq_ignore_ascii_case("display-p3") {
        return None;
    }
    let mut comps = [0.0f64; 3];
    let mut alpha = 255u8;
    let mut i = 0usize;
    for p in parts {
        if p.starts_with('/') {
            let a = p.trim_start_matches('/').trim();
            if !a.is_empty() {
                alpha = ((a.parse::<f64>().ok()? * 255.0).round().clamp(0.0, 255.0)) as u8;
            }
            continue;
        }
        if i >= 3 {
            return None;
        }
        let is_pct = p.ends_with('%');
        let num: f64 = p.trim_end_matches('%').parse().ok()?;
        comps[i] = if is_pct { num / 100.0 } else { num };
        i += 1;
    }
    if i < 3 {
        return None;
    }
    let q = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    Some(zero_render_foundation::color::Color {
        r: q(comps[0]),
        g: q(comps[1]),
        b: q(comps[2]),
        a: alpha,
    })
}

/// R34xx：Canvas 颜色属性 getter 的规范化序列化（spec：opaque → `#rrggbb`，alpha → `rgba(...)`——
/// 2d.shadow.attributes.shadowColor.valid 断言 'lime' → '#00ff00'、'RGBA(0,255,0,0)' → 'rgba(0, 255, 0, 0)'）。
fn color_to_canvas_css(c: &zero_render_foundation::color::Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, serialize_alpha(c.a))
    }
}

/// R34xx：alpha 序列化——最短十进制表示使 `round(d*255) == a`（u8 无法精确保存
/// 0.5*255=127.5，短表示使 'rgba(255,255,255,0.5)' 回读 '0.5' 而非 '0.502'；
/// driving: 2d.fillStyle.get.halftransparent/semitransparent）。0 特判为 '0'。
fn serialize_alpha(a: u8) -> String {
    if a == 0 {
        return "0".to_string();
    }
    // round(n/10^p * 255) == a ⟺ n/10^p ∈ [(a-0.5)/255, (a+0.5)/255)。
    for p in 1..=6 {
        let scale = 10_u64.pow(p);
        let lo = (((a as f64 - 0.5) / 255.0) * scale as f64).floor() as i64;
        let hi = (((a as f64 + 0.5) / 255.0) * scale as f64).ceil() as i64;
        for n in lo.max(0)..=hi.min(scale as i64) {
            if ((n as f64 / scale as f64 * 255.0).round() as i64) == a as i64 {
                // 首位 p 无尾零（更短的 p 已排除同值），直接按 p 位小数格式化。
                let int = (n / scale as i64) as u64;
                let frac = (n % scale as i64) as u64;
                return if frac == 0 {
                    int.to_string()
                } else {
                    format!("{int}.{frac:0p$}", p = p as usize)
                };
            }
        }
    }
    // 兜底：3 位四舍五入（旧行为；实际不可能到达——p=6 已覆盖全部 u8）。
    let alpha = ((a as f64 / 255.0) * 1000.0).round() / 1000.0;
    alpha.to_string()
}

/// R34xx：CSS Color 4 颜色序列化（color-mix/相对色输入 → `color(srgb r g b[/ a])`——
/// 2d.fillStyle.colormix 期望 'color(srgb 0.5 0 0.5)'）。分量 round 2 位。
fn color_to_css4(c: &zero_render_foundation::color::Color) -> String {
    let fmt = |v: u8| {
        let x = ((v as f64 / 255.0) * 100.0).round() / 100.0;
        format!("{x}")
    };
    if c.a == 255 {
        format!("color(srgb {} {} {})", fmt(c.r), fmt(c.g), fmt(c.b))
    } else {
        let a = ((c.a as f64 / 255.0) * 100.0).round() / 100.0;
        format!("color(srgb {} {} {} / {a})", fmt(c.r), fmt(c.g), fmt(c.b))
    }
}

/// canvas `lineJoin` 串 → LineJoin（spec: miter/round/bevel；未知回落 Miter = 默认）。
fn parse_line_join(s: &str) -> zero_canvas::LineJoin {
    match s.trim().to_ascii_lowercase().as_str() {
        "round" => zero_canvas::LineJoin::Round,
        "bevel" => zero_canvas::LineJoin::Bevel,
        _ => zero_canvas::LineJoin::Miter,
    }
}

/// canvas `pattern repetition` 串 → PatternRepetition（spec: repeat/repeat-x/repeat-y/no-repeat；空串/未知回落 Repeat = 默认）。
fn parse_pattern_repetition(s: &str) -> zero_canvas::PatternRepetition {
    use zero_canvas::PatternRepetition as P;
    match s.trim().to_ascii_lowercase().as_str() {
        "repeat-x" => P::RepeatX,
        "repeat-y" => P::RepeatY,
        "no-repeat" => P::NoRepeat,
        _ => P::Repeat, // "" / "repeat" / 未知 → Repeat（spec：空串 = repeat）
    }
}

/// canvas `lineCap` 串 → LineCap（spec: butt/round/square；未知回落 Butt = 默认）。
fn parse_line_cap(s: &str) -> zero_canvas::LineCap {
    match s.trim().to_ascii_lowercase().as_str() {
        "round" => zero_canvas::LineCap::Round,
        "square" => zero_canvas::LineCap::Square,
        _ => zero_canvas::LineCap::Butt,
    }
}

/// canvas `globalCompositeOperation` 串 → CompositeOperation（spec canonical 名；未知回落 SourceOver = 默认）。
/// composite 经 `composite_pixel` 在 rect-blit / rect-gradient / stroke / path-fill（`blit_path_to_pixels`
/// + `blit_path_gradient`，R3236 起）路径生效——故 shim 的 `fillRect`/`fill()`（path 实现）均受 composite 影响。
///
/// **剩余限制（记）**：
/// - `drawImage` 固定 source-over（不消费 composite）；
/// - blend 模式（multiply/screen/overlay/hue/color/luminosity 等）`composite_pixel` 用 source-over 因子——仅 Porter-Duff 模式真实合成。
///
/// getter/setter 状态往返真实（host 持 state，save/restore 含）。
fn parse_composite_operation(s: &str) -> zero_canvas::CompositeOperation {
    use zero_canvas::CompositeOperation as C;
    match s.trim().to_ascii_lowercase().as_str() {
        "source-over" => C::SourceOver,
        "destination-over" => C::DestinationOver,
        "destination-out" => C::DestinationOut,
        "destination-atop" => C::DestinationAtop,
        "destination-in" => C::DestinationIn,
        "source-in" => C::SourceIn,
        "source-out" => C::SourceOut,
        "source-atop" => C::SourceAtop,
        "lighter" => C::Lighter,
        "copy" => C::Copy,
        "xor" => C::Xor,
        "clear" => C::Clear,
        "multiply" => C::Multiply,
        "screen" => C::Screen,
        "overlay" => C::Overlay,
        "darken" => C::Darken,
        "lighten" => C::Lighten,
        "color-dodge" => C::ColorDodge,
        "color-burn" => C::ColorBurn,
        "hard-light" => C::HardLight,
        "soft-light" => C::SoftLight,
        "difference" => C::Difference,
        "exclusion" => C::Exclusion,
        "hue" => C::Hue,
        "saturation" => C::Saturation,
        "color" => C::Color,
        "luminosity" => C::Luminosity,
        _ => C::SourceOver,
    }
}

/// R34xx：host canvas bitmap 尺寸上限（16384²×4 ≈ 1GB）——巨尺寸（2^31-1 等）用例
/// 只断言 IDL 属性反射；钳制防 CanvasContext::new 分配 ~2^62 字节 abort。
const MAX_CANVAS_DIM: u32 = 16384;

/// canvas ImageData 线串 `"w:h;r,g,b,a,..."`（getImageData 对偶格式）→ `ImageData`。
/// 供 `drawImage` 系列桥接：shim 经源 canvas 的 getImageData 取全 RGBA wire 串，作为 drawImage 源传入。
/// 解析失败（无 `;`/无 `:`）返空 ImageData（draw_image_sized 对 0×0 早退，安全）。
fn parse_image_data_wire(s: &str) -> zero_canvas::ImageData {
    let s = s.trim();
    let Some((dim, csv)) = s.split_once(';') else {
        return zero_canvas::ImageData {
            width: 0,
            height: 0,
            data: vec![],
        };
    };
    let (w, h) = match dim.split_once(':') {
        Some((w, h)) => (
            w.trim().parse::<u32>().unwrap_or(0),
            h.trim().parse::<u32>().unwrap_or(0),
        ),
        None => (0, 0),
    };
    let data: Vec<u8> = csv
        .split(',')
        .filter(|t| !t.trim().is_empty())
        .filter_map(|t| t.trim().parse::<u8>().ok())
        .collect();
    zero_canvas::ImageData {
        width: w,
        height: h,
        data,
    }
}

/// RGBA 像素 → canvas ImageData 线串 `"w:h;r,g,b,a,..."`（getImageData 对偶格式，R3309 createImageBitmap）。
/// 供 drawImage 系列（经 `parse_image_data_wire` 反向解析）消费。与 getImageData 输出端编码逻辑一致。
fn encode_image_wire(pixels: &[u8], width: u32, height: u32) -> String {
    let mut out = format!("{}:{};", width, height);
    let mut first = true;
    for b in pixels {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&b.to_string());
    }
    out
}

/// `HTMLCanvasElement.getContext('2d')` 派发（R2795，canvas slice 1）。host 持 `CanvasRegistry`
///（`Arc<Mutex<CanvasRegistry>>`，上下文表 + 渐变表），JS 经 `__zw_canvas_op(handle, op, ...args)`
/// 串参派发（避免 JSON/serde 依赖）。**关键**：zero-canvas `fill_rect`/`stroke_rect` 便捷法**不写
/// pixel_buffer**（仅记 primitives），但 `fill()`/`stroke()`（path-based）经 `blit_path/stroke_to_pixels`
/// **写 pixel_buffer**——故 `fillRect` shim 经 beginPath+moveTo+lines+fill 实现（rasterize，getImageData
/// 可回读）。渐变样式经 `blit_rect_gradient`/`blit_path_gradient` 逐像素采样光栅化（R3079）。
/// `getContext2d` 创建上下文返 id；`getImageData` 返 `"{w},{h};{r},{g},{b},{a},..."`。
/// 供 `__zw_canvas_op` 回调 → shim canvas element + CanvasRenderingContext2D proxy。
pub fn canvas_context_op(reg: &mut CanvasRegistry, handle: &str, op: &str, args: &[String]) -> String {
    let arg = |i: usize| args.get(i).map(String::as_str).unwrap_or("0");
    let f = |i: usize| arg(i).trim().parse::<f32>().unwrap_or(0.0);
    let hid = || handle.trim().parse::<u64>().unwrap_or(0);
    // Path2D id（R3306）：Path2D 操作首参为 path id（args[0]）。
    let pid = || arg(0).trim().parse::<u64>().unwrap_or(0);
    match op {
        "getContext2d" => {
            let id = reg.next_ctx_id;
            reg.next_ctx_id += 1;
            // R34xx：host bitmap 尺寸钳制——WPT 巨尺寸用例（2d.canvas.host.size.large
            // 的 2^31-1）只断言 IDL 属性反射，不触碰 bitmap；未钳制则
            // CanvasContext::new 分配 ~2^62 字节 abort。属性值由 JS 侧独立持有。
            let w = arg(0).trim().parse::<u32>().unwrap_or(300).min(MAX_CANVAS_DIM);
            let h = arg(1).trim().parse::<u32>().unwrap_or(150).min(MAX_CANVAS_DIM);
            let mut ctx = zero_canvas::CanvasContext::new(w, h);
            // R34xx（color-type 目录）：getContext({colorSpace})——缓冲区解释空间。
            ctx.set_color_space(zero_canvas::CanvasColorSpace::parse_name(arg(2)));
            // R34xx：注入共享字体加载器（@font-face 字体真文本光栅）。
            ctx.set_font_loader(Some(reg.font_loader.clone()));
            reg.contexts.insert(id, ctx);
            id.to_string()
        }
        // R3308：canvas resize（spec 设 canvas.width/height 清空 bitmap + 重置绘图状态）。
        // handle = context id，args[0]/[1] = 新 width/height。调 CanvasContext::resize（重置全状态）。
        "resizeContext" => {
            // R34xx：同 getContext2d 的尺寸钳制（巨尺寸 abort 防护）。
            let w = arg(0).trim().parse::<u32>().unwrap_or(300).min(MAX_CANVAS_DIM);
            let h = arg(1).trim().parse::<u32>().unwrap_or(150).min(MAX_CANVAS_DIM);
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.resize(w, h);
                // R34xx：resize 重建 context 丢失字体加载器 → 重新注入（canvas
                // width/height 设置后 fillText 不再绘制——2d.color.space.p3.fillText）。
                ctx.set_font_loader(Some(reg.font_loader.clone()));
            }
            "ok".into()
        }
        // R3254-C8：transferToImageBitmap 的 bitmap 清空——只清像素（透明黑），保留绘图状态
        //（spec；此前复用 resizeContext 会重置 fillStyle/transform 等全部状态）。
        "clearBitmap" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.clear_bitmap();
            }
            "ok".into()
        }
        // R3309：createImageBitmap（HTML spec ImageBitmap）——解码图片字节为 wire 串供 drawImage 消费。
        // args[0] = data URI（`data:image/png;base64,...`）。复用 render-foundation::decode_data_uri
        //（PNG/JPEG/WebP/SVG 统一解码），输出 wire 串 `"w:h;r,g,b,a,..."`（getImageData 对偶格式，
        // drawImage 经 parse_image_data_wire 消费）。解码失败返 `"0:0;"`（JS 侧据尺寸 0 判失败 reject）。
        // 无 ctx 依赖（纯解码），handle 忽略。
        "decodeImageBitmap" => {
            let src = arg(0);
            match zero_render_foundation::image_cache::decode_data_uri(src) {
                Ok(img) => encode_image_wire(&img.pixels, img.width, img.height),
                Err(_) => "0:0;".into(),
            }
        }
        "setFillStyle" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：无效颜色忽略保持旧值（同 setStrokeStyle）。
                // R34xx（wide-gamut）：p3 画布 + color(display-p3 …) → 直取 p3 通道
                //（已处画布空间）；否则 sRGB 解析 → 画布空间转换。
                if ctx.color_space_of() == zero_canvas::CanvasColorSpace::DisplayP3
                    && let Some(c) = parse_p3_channels(arg(0))
                {
                    ctx.set_fill_color_raw(c);
                } else if let Some(color) = try_parse_canvas_color(arg(0)) {
                    ctx.set_fill_color(color);
                }
            }
            "ok".into()
        }
        "setStrokeStyle" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：无效颜色忽略保持旧值（spec：2d.line.invalid.strokestyle 设
                // 'nonsense' 后仍用原绿色）。旧实现 parse 失败回落黑色覆盖旧值。
                // R34xx（wide-gamut）：同 setFillStyle 的 color(display-p3) 直取。
                if ctx.color_space_of() == zero_canvas::CanvasColorSpace::DisplayP3
                    && let Some(c) = parse_p3_channels(arg(0))
                {
                    ctx.set_stroke_color_raw(c);
                } else if let Some(color) = try_parse_canvas_color(arg(0)) {
                    ctx.set_stroke_color(color);
                }
            }
            "ok".into()
        }
        "setLineWidth" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_line_width(f(0));
            }
            "ok".into()
        }
        "beginPath" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.begin_path();
            }
            "ok".into()
        }
        "closePath" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.close_path();
            }
            "ok".into()
        }
        "moveTo" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.move_to(f(0), f(1));
            }
            "ok".into()
        }
        "lineTo" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.line_to(f(0), f(1));
            }
            "ok".into()
        }
        "arc" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：anticlockwise 第 6 参（'true'/'false'/'1'/'0'）。
                let ccw = matches!(arg(5).trim(), "true" | "1");
                ctx.arc(f(0), f(1), f(2), f(3), f(4), ccw);
            }
            "ok".into()
        }
        "fill" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.fill();
            }
            "ok".into()
        }
        "stroke" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.stroke();
            }
            "ok".into()
        }
        // R3078：Canvas 2D 文本 API。fill_text 绘制（canvas crate fill_text 写 pixel_buffer）；measure_text 返
        // TextMetrics（R3303 spec 全 10 字段，csv 串参返 JS 构 TextMetrics）。spec CanvasRenderingContext2D。
        "fillText" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：第 4 参 maxWidth（spec fillText(text,x,y,maxWidth)；0/负/非有限 → None）。
                let mw = f(3);
                ctx.fill_text(arg(0), f(1), f(2), (mw.is_finite() && mw > 0.0).then_some(mw));
            }
            "ok".into()
        }
        "strokeText" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：真描边色路径（stroke_text——与 fill_text 同真字体光栅）。
                let mw = f(3);
                ctx.stroke_text(arg(0), f(1), f(2), (mw.is_finite() && mw > 0.0).then_some(mw));
            }
            "ok".into()
        }
        "measureText" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let m = ctx.measure_text(arg(0));
                // R3303：spec TextMetrics 全 10 字段 csv（width, actualBoxAscent/Descent/Left/Right,
                // fontBoxAscent/Descent, alphabetic/hanging/ideographicBaseline）。JS 构完整 TextMetrics。
                // R34xx：`|` 后接逐字形墨迹（l,t,r,b 逗号分隔、分号分隔字形）——
                // shim TextMetrics.getActualBoundingBox(start,end) 子串 bbox。
                let mut glyphs = String::new();
                for (i, (gpen, gl, gt, gr, gb)) in m.glyph_rects.iter().enumerate() {
                    if i > 0 {
                        glyphs.push(';');
                    }
                    glyphs.push_str(&format!("{gpen},{gl},{gt},{gr},{gb}"));
                }
                format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}|{}|{}",
                    m.width,
                    m.actual_bounding_box_ascent,
                    m.actual_bounding_box_descent,
                    m.actual_bounding_box_left,
                    m.actual_bounding_box_right,
                    m.font_bounding_box_ascent,
                    m.font_bounding_box_descent,
                    m.em_height_ascent,
                    m.em_height_descent,
                    m.alphabetic_baseline,
                    m.hanging_baseline,
                    m.ideographic_baseline,
                    glyphs,
                    // R34xx：对齐锚定（getActualBoundingBox 的 rect 钳制原点侧——
                    // full-bounds 与 API rect 同约定）。
                    alignment_anchor(ctx, m.width),
                )
            } else {
                // 无 ctx → 全 0（与既有 0 width 同语义）。
                "0,0,0,0,0,0,0,0,0,0,0,0|".into()
            }
        }
        // fillRect：经 path（rasterize 到 pixel_buffer，绕过 fill_rect 便捷法不写 pixel_buffer 之限制）。
        // fillRect/strokeRect 直接绘制，不得改动当前路径（HTML spec §4.12.5.1：fillRect 不影响
        // current default path）。旧实现 begin_path+fill 会清空并重写路径，使 save/restore 后的
        // fill() 把残留矩形并入路径（even-odd 镂空，上游 2d.state.saverestore.path WPT 失败）。
        "fillRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.fill_rect(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        "strokeRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.stroke_rect(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        "clearRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.clear_rect(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        // ── slice 2：path 曲线 / 状态栈 / transforms / line 样式 / globalAlpha（R2796）──
        "quadraticCurveTo" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.quadratic_curve_to(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        "bezierCurveTo" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.bezier_curve_to(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        "ellipse" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.ellipse(f(0), f(1), f(2), f(3), f(4), f(5), f(6));
            }
            "ok".into()
        }
        "arcTo" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.arc_to(f(0), f(1), f(2), f(3), f(4));
            }
            "ok".into()
        }
        // R3291：Canvas 2D roundRect（HTML Canvas `dom-context-2d-api` roundRect）。radii 串参格式：
        // 单值 "r" / 四角 "tl,tr,br,bl" / 两值 "h v"（spec 两值 = [tl&tr&br&bl 按 [a,b] 规则]——本层透传
        // canvas crate，flattener best-effort 退化矩形）。fillRule 参数（"nonzero"/"evenodd"）作为 args 末项
        // 透传但当前 is_point_in_path 用奇偶规则（canvas crate 限制）。
        "roundRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：radii 串支持角对——"p<x>,<y>"（DOMPoint x,y，JS join(',') 后
                // host split(',') 会拆成 'p<x>' 与 '<y>' **两个相邻项**）与裸标量 "10"。
                // R56：'p' 项与其后一项配对为 (x, y)——旧版 split_once(',') 在拆分后的
                // 'p40' 上永远失败 → unwrap_or((pair,pair)) 把 DOMPoint(40,20) 解成 (40,40)。
                let parts: Vec<&str> = arg(4).split(',').filter(|s| !s.trim().is_empty()).collect();
                let mut radii: Vec<(f32, f32)> = Vec::new();
                let mut i = 0;
                while i < parts.len() {
                    if let Some(x) = parts[i].trim().strip_prefix('p') {
                        let rx = x.trim().parse::<f32>().unwrap_or(0.0);
                        let ry = parts
                            .get(i + 1)
                            .and_then(|v| v.trim().parse::<f32>().ok())
                            .unwrap_or(rx);
                        radii.push((rx, ry));
                        i += 2;
                    } else if let Ok(v) = parts[i].trim().parse::<f32>() {
                        radii.push((v, v));
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                ctx.round_rect(f(0), f(1), f(2), f(3), radii);
            }
            "ok".into()
        }
        // R3291：Canvas 2D isPointInPath（hit-test 点是否在当前路径填充区内）。返 "1"/"0"（JS 转 bool）。
        // spec 三形式：isPointInPath(x,y) / isPointInPath(x,y,fillRule) / isPointInPath(path,x,y[,fillRule])。
        // 当前实现无 Path2D 参数形式（host 串参无 path 引用），仅 ctx 当前路径形式（最高频）。
        "isPointInPath" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return if ctx.is_point_in_path(f(0), f(1)) {
                    "1".into()
                } else {
                    "0".into()
                };
            }
            "0".into()
        }
        // R3291：Canvas 2D isPointInStroke（hit-test 点是否在当前路径描边区内，lineWidth 半宽内）。
        "isPointInStroke" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return if ctx.is_point_in_stroke(f(0), f(1)) {
                    "1".into()
                } else {
                    "0".into()
                };
            }
            "0".into()
        }
        // rect 路径命令：CanvasContext 无 rect() 方法，用 MoveTo+3 LineTo（匹配 Path2D::rect，不 auto-close）。
        "rect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R56：rect 子路径闭合（spec dom-context-2d-rect —— Path2D::rect 带
                // closePath；旧实现缺 close → fill 段式扫描线左边无边界段，单边交点
                // 奇数配不出对 → 整个矩形填不出）。
                let (x, y, w, h) = (f(0), f(1), f(2), f(3));
                ctx.move_to(x, y);
                ctx.line_to(x + w, y);
                ctx.line_to(x + w, y + h);
                ctx.line_to(x, y + h);
                ctx.close_path();
            }
            "ok".into()
        }
        "clip" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.clip();
            }
            "ok".into()
        }
        "save" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.save();
            }
            "ok".into()
        }
        "restore" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.restore();
            }
            "ok".into()
        }
        "translate" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.translate(f(0), f(1));
            }
            "ok".into()
        }
        "rotate" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.rotate(f(0));
            }
            "ok".into()
        }
        "scale" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.scale(f(0), f(1));
            }
            "ok".into()
        }
        "setTransform" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_transform(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        "transform" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.transform(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        // R2985 getTransform：返当前 2D 变换矩阵 "a,b,c,d,e,f"（shim 包 DOMMatrix）。只读（get_transform
        // 取 &self），无 ctx → identity "1,0,0,1,0,0"。Canvas 2D spec getTransform() → DOMMatrix。
        "getTransform" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let t = ctx.get_transform();
                return format!("{},{},{},{},{},{}", t.a, t.b, t.c, t.d, t.e, t.f);
            }
            "1,0,0,1,0,0".into()
        }
        // R2985 resetTransform：重置为单位矩阵（spec setTransform(identity)）。
        "resetTransform" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.reset_transform();
            }
            "ok".into()
        }
        // R34xx：reset()（spec：清空画布 + 上下文状态回默认——2d.fillStyle.CSSRGB 尾部调用）。
        // 经 `CanvasContext::new` 重建（全状态默认；尺寸不变）。
        "reset" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let (w, h) = (ctx.width(), ctx.height());
                *ctx = zero_canvas::CanvasContext::new(w, h);
            }
            "ok".into()
        }
        "setGlobalAlpha" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_global_alpha(f(0));
            }
            "ok".into()
        }
        "setLineDash" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let segs: Vec<f32> = arg(0)
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
                ctx.set_line_dash(segs);
            }
            "ok".into()
        }
        "setLineJoin" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_line_join(parse_line_join(arg(0)));
            }
            "ok".into()
        }
        "setLineCap" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_line_cap(parse_line_cap(arg(0)));
            }
            "ok".into()
        }
        // ── slice 4：globalCompositeOperation / shadow / putImageData（R2798）──
        // composite 状态真实（host 持 state，save/restore 含）；effect 仅经 composite_pixel 在 rect-blit/stroke
        // 生效，path-based fill 不消费（见 parse_composite_operation 注释）。
        "setCompositeOperation" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_composite_operation(parse_composite_operation(arg(0)));
            }
            "ok".into()
        }
        "setShadowColor" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：无效颜色忽略保持旧值（同 setStrokeStyle——2d.shadow.attributes
                // shadowColor.invalid 设 'bogus' 应保持原值）。
                if let Some(color) = try_parse_canvas_color(arg(0)) {
                    ctx.set_shadow_color(color);
                }
            }
            "ok".into()
        }
        "validateColor" => {
            if try_parse_canvas_color(arg(0)).is_some() {
                "1".to_string()
            } else {
                String::new()
            }
        }
        "parseColorCss4" => {
            // R34xx：CSS Color 4 输入（color-mix/相对色）→ 规范化 color() 串；普通颜色返空
            //（JS 回退 hex getter）。Mix/RelativeColor 变体才需要 color() 保留。
            let cv = zero_css_parser::values::parse_color(arg(0).trim());
            match cv {
                Some(
                    zero_css_parser::values::ColorValue::Mix(_) | zero_css_parser::values::ColorValue::RelativeColor(_),
                ) => color_to_css4(&crate::color_value_to_render(&cv.unwrap())),
                _ => String::new(),
            }
        }
        "getFillStyle" => reg
            .contexts
            .get(&hid())
            .map(|ctx| match ctx.fill_style() {
                zero_canvas::CanvasStyle::Color(c) => color_to_canvas_css(c),
                _ => String::new(), // 渐变/图案——JS 侧缓存返回对象
            })
            .unwrap_or_default(),
        "getStrokeStyle" => reg
            .contexts
            .get(&hid())
            .map(|ctx| match ctx.stroke_style() {
                zero_canvas::CanvasStyle::Color(c) => color_to_canvas_css(c),
                _ => String::new(),
            })
            .unwrap_or_default(),
        "getShadowColor" => reg
            .contexts
            .get(&hid())
            .map(|ctx| color_to_canvas_css(ctx.shadow_color()))
            .unwrap_or_default(),
        "setShadowBlur" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_shadow_blur(f(0));
            }
            "ok".into()
        }
        "setShadowOffsetX" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_shadow_offset_x(f(0));
            }
            "ok".into()
        }
        "setShadowOffsetY" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_shadow_offset_y(f(0));
            }
            "ok".into()
        }
        // putImageData（get_imageData 对偶）：args = [dx, dy, w, h, "r,g,b,a,..."]。
        // 直接写 pixel_buffer（put_image_data copy_from_slice，1:1 替换，无 composite/alpha 合成）。
        "putImageData" => {
            let dx = arg(0).trim().parse::<i32>().unwrap_or(0);
            let dy = arg(1).trim().parse::<i32>().unwrap_or(0);
            let w = arg(2).trim().parse::<u32>().unwrap_or(0);
            let h = arg(3).trim().parse::<u32>().unwrap_or(0);
            let data: Vec<u8> = arg(4)
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let mut img = zero_canvas::ImageData {
                    width: w,
                    height: h,
                    data,
                };
                // R34xx（color-type 目录）：源 ImageData 色彩空间 ≠ canvas → 转换
                //（putImageData 的 colorSpace 参数——2d.color.type.u8srgb.to.u8p3）。
                let src_cs = zero_canvas::CanvasColorSpace::parse_name(arg(5));
                src_cs.convert_buffer(zero_canvas::CanvasContext::color_space_of(ctx), &mut img.data);
                ctx.put_image_data(&img, dx, dy);
            }
            "ok".into()
        }
        // ── drawImage 系列（R2799，canvas slice 5）：源 canvas → 本 ctx。host draw_image* 已存在且
        // 真写 pixel_buffer（draw_image_sized：最近邻采样 + transform + source-over alpha 混合 + global_alpha）。
        // args[0] = 源 ImageData wire（shim 经源 canvas getImageData 取），后续为目标几何。
        // **已知限制**：固定 source-over（不消费 globalCompositeOperation）；源限 canvas（img decode defer）。
        "drawImage" => {
            let mut img = parse_image_data_wire(arg(0));
            // R34xx（color-type 目录）：源位图色彩空间 ≠ canvas → 转换（p3 位图在
            // srgb 画布——createImageBitmap.p3.rgba.unorm8 的 (255,0,0) → (234,51,35)）。
            let src_cs = zero_canvas::CanvasColorSpace::parse_name(arg(3));
            let (dx, dy) = (f(1), f(2));
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                src_cs.convert_buffer(ctx.color_space_of(), &mut img.data);
                ctx.draw_image(&img, dx, dy);
            }
            "ok".into()
        }
        "drawImageScaled" => {
            let mut img = parse_image_data_wire(arg(0));
            let src_cs = zero_canvas::CanvasColorSpace::parse_name(arg(5));
            let (dx, dy, dw, dh) = (f(1), f(2), f(3), f(4));
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                src_cs.convert_buffer(ctx.color_space_of(), &mut img.data);
                ctx.draw_image_with_size(&img, dx, dy, dw, dh);
            }
            "ok".into()
        }
        "drawImageSliced" => {
            let img = parse_image_data_wire(arg(0));
            let (sx, sy, sw, sh, dx, dy, dw, dh) = (f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8));
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.draw_image_sliced(&img, sx, sy, sw, sh, dx, dy, dw, dh);
            }
            "ok".into()
        }
        // toDataURL（R2797，canvas slice 3）：pixel_buffer（经 get_image_data 取全 RGBA）→ PNG 编码 →
        // 返**逗号分隔十进制串**（shim 转 Latin-1 → btoa → `data:image/png;base64,...`）。复用 png crate
        //（miniz_oxide 已 transitive）；编码失败返空串（shim 回落 `data:,`）。仅 'image/png'（jpeg/webp defer）。
        "toDataURL" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let w = ctx.width().max(1);
                let h = ctx.height().max(1);
                let img = ctx.get_image_data(0, 0, w as i32, h as i32);
                let mut out: Vec<u8> = Vec::new();
                {
                    let mut enc = png::Encoder::new(&mut out, w, h);
                    enc.set_color(png::ColorType::Rgba);
                    enc.set_depth(png::BitDepth::Eight);
                    if let Ok(mut writer) = enc.write_header() {
                        if writer.write_image_data(&img.data).is_err() {
                            return String::new();
                        }
                    } else {
                        return String::new();
                    }
                }
                return out.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
            }
            String::new()
        }
        "getImageData" => {
            // R34xx：有符号解析（负 dims 翻转/负坐标越界透明语义）。
            let (x, y, w, h) = (
                arg(0).trim().parse::<i32>().unwrap_or(0),
                arg(1).trim().parse::<i32>().unwrap_or(0),
                arg(2).trim().parse::<i32>().unwrap_or(0),
                arg(3).trim().parse::<i32>().unwrap_or(0),
            );
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let mut img = ctx.get_image_data(x, y, w, h);
                // R34xx（color-type 目录）：请求 colorSpace ≠ canvas → 转换
                //（getImageData settings.colorSpace——u8p3 画布 srgb 回读）。
                let req_cs = zero_canvas::CanvasColorSpace::parse_name(arg(4));
                zero_canvas::CanvasContext::color_space_of(ctx).convert_buffer(req_cs, &mut img.data);
                let mut out = format!("{}:{};", img.width, img.height);
                let mut first = true;
                for b in &img.data {
                    if !first {
                        out.push(',');
                    }
                    first = false;
                    out.push_str(&b.to_string());
                }
                return out;
            }
            "0:0;".into()
        }
        // ── 渐变（R3079）：CanvasGradient 独立对象，独立 id 命名空间。createLinear/Radial/Conic 返渐变 id；
        // addColorStop 经 id 变更停止点；setFill/StrokeStyleGradient 经 id 查表克隆到 context 样式（fill_rect/
        // fill/fill_with_path 在 canvas crate 经 sample_at 逐像素光栅化）。spec CanvasRenderingContext2D。
        "createLinearGradient" => {
            let id = reg.next_grad_id;
            reg.next_grad_id += 1;
            let grad = zero_canvas::LinearGradient::new(f(0), f(1), f(2), f(3));
            reg.gradients.insert(id, zero_canvas::CanvasStyle::LinearGradient(grad));
            id.to_string()
        }
        "createRadialGradient" => {
            let id = reg.next_grad_id;
            reg.next_grad_id += 1;
            let grad = zero_canvas::RadialGradient::new(f(0), f(1), f(2), f(3), f(4), f(5));
            reg.gradients.insert(id, zero_canvas::CanvasStyle::RadialGradient(grad));
            id.to_string()
        }
        "createConicGradient" => {
            let id = reg.next_grad_id;
            reg.next_grad_id += 1;
            let grad = zero_canvas::ConicGradient::new(f(0), f(1), f(2));
            reg.gradients.insert(id, zero_canvas::CanvasStyle::ConicGradient(grad));
            id.to_string()
        }
        // addColorStop(gradId, offset, color)：变更渐变停止点。offset 经 canvas crate clamp 到 [0,1]（spec）。
        "addColorStop" => {
            let gid = arg(0).trim().parse::<u64>().unwrap_or(0);
            let offset = f(1).clamp(0.0, 1.0);
            let color_str = arg(2);
            if let Some(style) = reg.gradients.get_mut(&gid) {
                // R34xx：stop 含 CSS Color 4 现代函数（color-mix/相对色）→ 渐变 OKLab 插值
                //（driving: 2d.gradient.colormix/relativecolor——Chromium 对现代颜色 stop
                // 按 OKLab 插值，legacy 颜色按 sRGB）。
                if matches!(
                    zero_css_parser::values::parse_color(color_str.trim()),
                    Some(zero_css_parser::values::ColorValue::Mix(_))
                        | Some(zero_css_parser::values::ColorValue::RelativeColor(_))
                ) {
                    style.set_oklab_interpolation(true);
                }
                style.add_color_stop(offset, parse_canvas_color(color_str));
            }
            // R34xx：live 重放——引用该渐变 id 的 context 样式同步更新（2d.gradient.object.update：
            // 第二次 fill 前 addColorStop 新停止点须生效）。
            if let Some(style) = reg.gradients.get(&gid)
                && let Some(refs) = reg.grad_refs.get(&gid).cloned()
            {
                for (ctx_id, slot) in refs {
                    if let Some(ctx) = reg.contexts.get_mut(&ctx_id) {
                        match slot {
                            0 => ctx.set_fill_style(style.clone()),
                            _ => ctx.set_stroke_style(style.clone()),
                        }
                    }
                }
            }
            "ok".into()
        }
        "setFillStyleGradient" => {
            let gid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&gid)) {
                ctx.set_fill_style(style.clone());
            }
            // R34xx：记录引用（addColorStop live 重放用）。
            reg.grad_refs.entry(gid).or_default().push((hid(), 0));
            "ok".into()
        }
        "setStrokeStyleGradient" => {
            let gid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&gid)) {
                ctx.set_stroke_style(style.clone());
            }
            reg.grad_refs.entry(gid).or_default().push((hid(), 1));
            "ok".into()
        }
        // ── 图案（R3085）：createPattern 经 ImageData wire（shim 从源 canvas getImageData 取）建 CanvasStyle::Pattern
        // 存渐变注册表（同 id 命名空间）。fill/stroke 经 is_per_pixel_style 路由逐像素平铺（canvas crate
        // sample_pattern_pixel）。setFillStylePattern/setStrokeStylePattern 与 gradient 版同（注册表查表克隆）。
        "createPattern" => {
            let img = parse_image_data_wire(arg(0));
            let rep = parse_pattern_repetition(arg(1));
            let id = reg.next_grad_id;
            reg.next_grad_id += 1;
            reg.gradients.insert(
                id,
                zero_canvas::CanvasStyle::Pattern(zero_canvas::CanvasPattern::new(img, rep)),
            );
            id.to_string()
        }
        "setFillStylePattern" => {
            let pid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&pid)) {
                ctx.set_fill_style(style.clone());
            }
            "ok".into()
        }
        "setStrokeStylePattern" => {
            let pid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&pid)) {
                ctx.set_stroke_style(style.clone());
            }
            "ok".into()
        }
        // R3304：Canvas 2D 文本/线连接状态属性（ctx.font / textAlign / textBaseline / direction / miterLimit）。
        // Rust 后端（CanvasContext::set_font/set_text_align/set_text_baseline/set_direction/set_miter_limit
        // + getter）早全，但缺 host op 派发 + JS shim 暴露 → 页面 `ctx.font='20px Arial'` no-op，measureText/
        // fillText 恒用默认 10px。setFont 解析 CSS font 简写串（FontDescriptor::parse_css），解析失败忽略（spec
        // 忽略非法 font 串保持原值）。getter 返归一化 spec 字符串。https://html.spec.whatwg.org/multipage/canvas.html
        "setLetterSpacing" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                // R34xx：存原始 CSS 长度串（相对单位随字号重解析——change.font 用例）。
                if zero_canvas::parse_length_px(arg(0), ctx.font().size).is_some() {
                    ctx.set_letter_spacing(arg(0));
                }
            }
            "ok".into()
        }
        "setWordSpacing" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid())
                && zero_canvas::parse_length_px(arg(0), ctx.font().size).is_some()
            {
                ctx.set_word_spacing(arg(0));
            }
            "ok".into()
        }
        // R34xx：fontKerning（'none' → shaping 关 kern——2d.text.drawing.style.fontKerning
        // 的 measure 宽度对比）。值集大小写在 shim 侧校验。
        "setFontKerning" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_font_kerning(arg(0));
            }
            "ok".into()
        }
        // R34xx：ctx.lang（BCP47——shaping 语言系统；'tr' → TRK 关 fi 连字。
        // 2d.text.measure.lang）。shim 侧已把 'inherit' 解析为 canvas 元素 lang。
        "setLang" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_lang(arg(0));
            }
            "ok".into()
        }
        "setFont" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid())
                && let Some(mut fd) = zero_canvas::FontDescriptor::parse_css_with_current(arg(0), ctx.font().size)
            {
                // R34xx：letterSpacing/wordSpacing/fontKerning 跨字体变更保持（spec——
                // change.font 用例；parse_css 新描述符默认 0/off，须继承现有值。
                // reset.fontKerning.none：二次设置 ctx.font 后 kerning 状态保持）。
                fd.letter_spacing = ctx.font().letter_spacing.clone();
                fd.word_spacing = ctx.font().word_spacing.clone();
                fd.kerning_none = ctx.font().kerning_none;
                ctx.set_font(fd);
                // 解析失败：spec 忽略非法 font 串，保持原值（返 ok 不报错）。
            }
            "ok".into()
        }
        // ctx.font getter：返 CSS font 简写串（"style weight sizepx family"）。real browser 返解析后规范化串。
        "getFont" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let fd = ctx.font();
                let style_str = match fd.style {
                    zero_canvas::FontStyle::Italic => "italic ",
                    zero_canvas::FontStyle::Normal => "",
                };
                // R34xx：small-caps variant 重建（font.parse.complex——'italic small-caps 12px ...'）。
                let variant_str = if fd.small_caps { "small-caps " } else { "" };
                // R34xx：数值 weight 优先（'italic 300 12px serif' 保留 300）。
                let weight_str = match fd.weight_value {
                    Some(v) if v != 400 => format!("{v} "),
                    _ => match fd.weight {
                        zero_canvas::FontWeight::Bold => "bold ".to_string(),
                        zero_canvas::FontWeight::Normal => String::new(),
                    },
                };
                // R34xx：规范化——仅通用族关键字小写 + 逗号后空格（WPT font.parse.* 期望
                // 'serif'/'cursive, fantasy, ...' 小写、自定义族名 'UnquotedFont' 保留）。
                // CSV 感知拆分（引号内逗号是族名一部分——'..., "QuotedFont\\\","'）。
                let mut family = String::new();
                let mut seg = String::new();
                let mut in_quote = false;
                let mut chars = fd.family.chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        '\\' if in_quote => {
                            // CSV 引号内反斜杠转义（\" → 字面引号）。
                            seg.push(ch);
                            if let Some(&next) = chars.peek() {
                                seg.push(next);
                                chars.next();
                            }
                        }
                        '"' => {
                            in_quote = !in_quote;
                            seg.push(ch);
                        }
                        ',' if !in_quote => {
                            let f = seg.trim();
                            let lower = f.to_ascii_lowercase();
                            let out = match lower.as_str() {
                                "serif" | "sans-serif" | "cursive" | "fantasy" | "monospace" | "system-ui"
                                | "ui-serif" | "ui-sans-serif" | "ui-monospace" | "ui-rounded" | "emoji" | "math"
                                | "fangsong" => lower,
                                _ => f.to_string(),
                            };
                            if !family.is_empty() {
                                family.push_str(", ");
                            }
                            family.push_str(&out);
                            seg.clear();
                        }
                        _ => seg.push(ch),
                    }
                }
                {
                    let f = seg.trim();
                    let lower = f.to_ascii_lowercase();
                    let out = match lower.as_str() {
                        "serif" | "sans-serif" | "cursive" | "fantasy" | "monospace" | "system-ui" | "ui-serif"
                        | "ui-sans-serif" | "ui-monospace" | "ui-rounded" | "emoji" | "math" | "fangsong" => lower,
                        _ => f.to_string(),
                    };
                    if !family.is_empty() && !out.is_empty() {
                        family.push_str(", ");
                    }
                    family.push_str(&out);
                }
                return format!("{}{}{}{}px {family}", style_str, variant_str, weight_str, fd.size);
            }
            "10px sans-serif".into()
        }
        "setTextAlign" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_text_align(parse_text_align(arg(0)));
            }
            "ok".into()
        }
        "getTextAlign" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return match ctx.text_align() {
                    zero_canvas::TextAlign::Start => "start",
                    zero_canvas::TextAlign::End => "end",
                    zero_canvas::TextAlign::Left => "left",
                    zero_canvas::TextAlign::Right => "right",
                    zero_canvas::TextAlign::Center => "center",
                }
                .into();
            }
            "start".into()
        }
        "setTextBaseline" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_text_baseline(parse_text_baseline(arg(0)));
            }
            "ok".into()
        }
        "getTextBaseline" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return match ctx.text_baseline() {
                    zero_canvas::TextBaseline::Top => "top",
                    zero_canvas::TextBaseline::Middle => "middle",
                    zero_canvas::TextBaseline::Alphabetic => "alphabetic",
                    zero_canvas::TextBaseline::Bottom => "bottom",
                    zero_canvas::TextBaseline::Hanging => "hanging",
                    zero_canvas::TextBaseline::Ideographic => "ideographic",
                }
                .into();
            }
            "alphabetic".into()
        }
        "setDirection" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_direction(parse_text_direction(arg(0)));
            }
            "ok".into()
        }
        "getDirection" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return match ctx.direction() {
                    zero_canvas::TextDirection::Ltr => "ltr",
                    zero_canvas::TextDirection::Rtl => "rtl",
                    zero_canvas::TextDirection::Inherit => "inherit",
                }
                .into();
            }
            "inherit".into()
        }
        "setMiterLimit" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_miter_limit(f(0));
            }
            "ok".into()
        }
        "getMiterLimit" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return ctx.miter_limit().to_string();
            }
            "10".into()
        }
        // R3305：lineDashOffset（虚线动画 marching-ants 基础）+ getLineDash（返展开后偶长数组，spec）+
        // imageSmoothingEnabled / imageSmoothingQuality（drawImage 缩放重采样控制）。Rust 后端早全，仅缺
        // host op 派发 + JS shim 暴露。getLineDash 从 host 读（权威——奇长输入被展开为偶长，客户端镜像
        // 无法推断）。https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d
        "setLineDashOffset" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_line_dash_offset(f(0));
            }
            "ok".into()
        }
        "getLineDashOffset" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return ctx.get_line_dash_offset().to_string();
            }
            "0".into()
        }
        "getLineDash" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let dash = ctx.get_line_dash();
                let mut s = String::with_capacity(dash.len() * 2);
                for (i, v) in dash.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    s.push_str(&v.to_string());
                }
                return s;
            }
            "".into()
        }
        "setImageSmoothingEnabled" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_image_smoothing_enabled(arg(0).trim() == "1" || arg(0).trim() == "true");
            }
            "ok".into()
        }
        "getImageSmoothingEnabled" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return if ctx.image_smoothing_enabled() { "1" } else { "0" }.into();
            }
            "1".into()
        }
        "setImageSmoothingQuality" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_image_smoothing_quality(parse_image_smoothing_quality(arg(0)));
            }
            "ok".into()
        }
        "getImageSmoothingQuality" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                return match ctx.image_smoothing_quality() {
                    zero_canvas::ImageSmoothingQuality::Low => "low",
                    zero_canvas::ImageSmoothingQuality::Medium => "medium",
                    zero_canvas::ImageSmoothingQuality::High => "high",
                }
                .into();
            }
            "high".into()
        }
        // R3306：Path2D（spec CanvasPath，`new Path2D()` / `new Path2D(other)` / `new Path2D(svgString)`）。
        // Path2D 为 context 无关的可复用路径对象，经独立 id 命名空间存注册表。ctx.fill(path)/stroke(path)/
        // clip(path) 取 path id 查表得 Path2D 调 fill_path/stroke_path/clip_path（替代 ctx 当前路径）。
        // path id 为 args[0]，坐标参从 args[1] 起。
        // R3307：svgString 构造形式（`new Path2D("M10 10 L90 90")`）补全——首参为非数字串时走
        // `Path2D::from_svg`（canvas crate SVG path 解析器，lenient 不抛，详见 path.rs R3307 注释）。
        "createPath" => {
            let id = reg.next_path_id;
            reg.next_path_id += 1;
            let raw0 = arg(0);
            // 三态分支：path id（数字）→ 复制既有 Path2D（new Path2D(other)）；
            //         非数字非空串 → SVG path data 解析（new Path2D(svgString)）；
            //         空串/缺省 → 建空（new Path2D()）。
            let path = if let Ok(other_id) = raw0.trim().parse::<u64>()
                && let Some(src) = reg.paths.get(&other_id)
            {
                let mut p = zero_canvas::Path2D::new();
                p.add_path(src);
                p
            } else if !raw0.trim().is_empty() {
                // SVG path data：lenient 解析（非法命令静默跳过，real browser spec 亦尽力解析非法 path data 不抛）。
                let mut p = zero_canvas::Path2D::new();
                p.from_svg(raw0);
                p
            } else {
                zero_canvas::Path2D::new()
            };
            reg.paths.insert(id, path);
            id.to_string()
        }
        "pathMoveTo" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.move_to(f(1), f(2));
            }
            "ok".into()
        }
        "pathLineTo" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.line_to(f(1), f(2));
            }
            "ok".into()
        }
        "pathClose" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.close_path();
            }
            "ok".into()
        }
        "pathArc" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                // R34xx：anticlockwise 第 6 参。
                let ccw = matches!(arg(6).trim(), "true" | "1");
                p.arc(f(1), f(2), f(3), f(4), f(5), ccw);
            }
            "ok".into()
        }
        "pathArcTo" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.arc_to(f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        "pathQuadratic" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.quadratic_curve_to(f(1), f(2), f(3), f(4));
            }
            "ok".into()
        }
        "pathBezier" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.bezier_curve_to(f(1), f(2), f(3), f(4), f(5), f(6));
            }
            "ok".into()
        }
        "pathEllipse" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.ellipse(f(1), f(2), f(3), f(4), f(5), f(6), f(7));
            }
            "ok".into()
        }
        "pathRect" => {
            if let Some(p) = reg.paths.get_mut(&pid()) {
                p.rect(f(1), f(2), f(3), f(4));
            }
            "ok".into()
        }
        // addPath(src)：dest.add_path(src)（spec Path2D.addPath）。先克隆 src 避免借用冲突。
        "addPath" => {
            let src_id = arg(1).trim().parse::<u64>().unwrap_or(0);
            if let Some(src) = reg.paths.get(&src_id).cloned()
                && let Some(dest) = reg.paths.get_mut(&pid())
            {
                dest.add_path(&src);
            }
            "ok".into()
        }
        // ctx.fill(path)/stroke(path)/clip(path)：path id 为 args[0]，ctx 来自 handle。
        "fillPath" => {
            if let (Some(ctx), Some(path)) = (reg.contexts.get_mut(&hid()), reg.paths.get(&pid())) {
                ctx.fill_path(path);
            }
            "ok".into()
        }
        "strokePath" => {
            if let (Some(ctx), Some(path)) = (reg.contexts.get_mut(&hid()), reg.paths.get(&pid())) {
                ctx.stroke_path(path);
            }
            "ok".into()
        }
        "clipPath" => {
            if let (Some(ctx), Some(path)) = (reg.contexts.get_mut(&hid()), reg.paths.get(&pid())) {
                ctx.clip_path(path);
            }
            "ok".into()
        }
        _ => "ok".into(),
    }
}

/// 解析 textAlign 字符串（spec CSS：start/end/left/right/center，大小写不敏感，非法 → Start 默认）。
fn parse_text_align(s: &str) -> zero_canvas::TextAlign {
    use zero_canvas::TextAlign as A;
    match s.trim().to_ascii_lowercase().as_str() {
        "end" => A::End,
        "left" => A::Left,
        "right" => A::Right,
        "center" => A::Center,
        _ => A::Start, // start + 非法值 → Start（spec 非法值忽略，保持原值；此处保守默认）
    }
}

/// 解析 textBaseline 字符串（top/middle/alphabetic/bottom，大小写不敏感，非法 → Alphabetic 默认）。
fn parse_text_baseline(s: &str) -> zero_canvas::TextBaseline {
    use zero_canvas::TextBaseline as B;
    match s.trim().to_ascii_lowercase().as_str() {
        "top" => B::Top,
        "middle" => B::Middle,
        "bottom" => B::Bottom,
        "hanging" => B::Hanging,
        "ideographic" => B::Ideographic,
        _ => B::Alphabetic, // alphabetic + 非法值 → Alphabetic
    }
}

/// R34xx：measureText 对齐锚定（与 fill_text 的 ox 同语义，按 textAlign/direction 与
/// 文本宽度推导——getActualBoundingBox 的 rect 原点钳制用）。
fn alignment_anchor(ctx: &zero_canvas::CanvasContext, width: f32) -> f32 {
    use zero_canvas::{TextAlign, TextDirection};
    let rtl = matches!(ctx.direction(), TextDirection::Rtl);
    match ctx.text_align() {
        TextAlign::Center => -width / 2.0,
        TextAlign::Right => -width,
        TextAlign::Left => 0.0,
        TextAlign::Start => {
            if rtl {
                -width
            } else {
                0.0
            }
        }
        TextAlign::End => {
            if rtl {
                0.0
            } else {
                -width
            }
        }
    }
}

/// 解析 direction 字符串（ltr/rtl/inherit，大小写不敏感，非法 → Inherit 默认）。
fn parse_text_direction(s: &str) -> zero_canvas::TextDirection {
    use zero_canvas::TextDirection as D;
    match s.trim().to_ascii_lowercase().as_str() {
        "ltr" => D::Ltr,
        "rtl" => D::Rtl,
        _ => D::Inherit, // inherit + 非法值 → Inherit
    }
}

/// 解析 imageSmoothingQuality 字符串（low/medium/high，大小写不敏感，非法 → High 默认，R3305）。
fn parse_image_smoothing_quality(s: &str) -> zero_canvas::ImageSmoothingQuality {
    use zero_canvas::ImageSmoothingQuality as Q;
    match s.trim().to_ascii_lowercase().as_str() {
        "low" => Q::Low,
        "medium" => Q::Medium,
        _ => Q::High, // high + 非法值 → High（headless 默认）
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // R34xx：alpha 最短可回滚序列化（driving: 2d.fillStyle.get.halftransparent）。
    #[test]
    fn test_serialize_alpha_roundtrip() {
        // 0.5 → u8 128（127.5 四舍五入）→ 序列化回 '0.5' 而非 '0.502'。
        assert_eq!(serialize_alpha(128), "0.5");
        // 0.45 → 115（114.75 四舍五入）→ '0.45'（semitransparent 正则 0\.4\d+）。
        assert_eq!(serialize_alpha(115), "0.45");
        assert_eq!(serialize_alpha(0), "0");
        assert_eq!(serialize_alpha(204), "0.8"); // 0.8*255=204 精确
        assert_eq!(serialize_alpha(191), "0.75"); // 0.75*255=191.25 → 191
        assert_eq!(serialize_alpha(3), "0.01"); // 0.01*255=2.55 → 3
        // 任意 u8 都能找到 ≤6 位表示（兜底分支不可达）。
        for a in 0u8..=255u8 {
            let s = serialize_alpha(a);
            assert!(!s.is_empty(), "a={a}");
        }
    }

    #[test]
    fn test_color_to_canvas_css_alpha() {
        let c = zero_render_foundation::color::Color::rgba(255, 255, 255, 128);
        assert_eq!(color_to_canvas_css(&c), "rgba(255, 255, 255, 0.5)");
        let c = zero_render_foundation::color::Color::rgba(0, 255, 0, 0);
        assert_eq!(color_to_canvas_css(&c), "rgba(0, 255, 0, 0)");
        let c = zero_render_foundation::color::Color::rgb(255, 0, 0);
        assert_eq!(color_to_canvas_css(&c), "#ff0000");
    }
}
