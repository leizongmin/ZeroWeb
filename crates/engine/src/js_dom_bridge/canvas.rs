//! Canvas 2D host 操作派发。从 js_dom_bridge.rs 拆出（R2974，文件大小治理 slice 2）。
//! `canvas_context_op(reg, handle, op, args)` 经 `__zw_canvas_op` 回调派发全部 Canvas 2D 操作
//!（路径/矩形/文本/图像/变换/合成/像素/状态），辅以值解析 helper（颜色/line-join/line-cap/
//! composite/image-data wire）。纯 zero_canvas/zero_render_foundation 类型，无 DOM/选择器依赖。
//! pub canvas_context_op 经 `pub use canvas::*` 重导出，register_dom_callbacks 调用点零改动。

use std::collections::HashMap;

/// Canvas host 注册表：上下文表 + 渐变表。
///
/// 上下文由 `getContext('2d')` 创建（`getContext2d` op），按 id 索引。
/// 渐变由 `createLinearGradient`/`createRadialGradient`/`createConicGradient` 创建，独立 id 命名空间
///（spec：CanvasGradient 为独立对象，可被任意 context 的 fillStyle/strokeStyle 引用）。`addColorStop`
/// 经渐变 id 变更停止点；`setFillStyleGradient`/`setStrokeStyleGradient` 经渐变 id 查表克隆到 context 样式。
pub struct CanvasRegistry {
    /// 下一个 context id（从 1 起）。
    pub next_ctx_id: u64,
    /// context id → CanvasContext。
    pub contexts: HashMap<u64, zero_canvas::CanvasContext>,
    /// 下一个 gradient id（从 1 起，与 context id 独立命名空间）。
    pub next_grad_id: u64,
    /// gradient id → CanvasStyle（仅渐变变体）。
    pub gradients: HashMap<u64, zero_canvas::CanvasStyle>,
}

impl CanvasRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self {
            next_ctx_id: 1,
            contexts: HashMap::new(),
            next_grad_id: 1,
            gradients: HashMap::new(),
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
    use zero_render_foundation::color::Color;
    zero_css_parser::values::parse_color(s.trim())
        .map(|cv| crate::color_value_to_render(&cv))
        .unwrap_or_else(|| Color::rgba(0, 0, 0, 255))
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
        "source-atop" => C::SourceAtop,
        "lighter" => C::Lighter,
        "copy" => C::Copy,
        "xor" => C::Xor,
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
    match op {
        "getContext2d" => {
            let id = reg.next_ctx_id;
            reg.next_ctx_id += 1;
            let w = arg(0).trim().parse::<u32>().unwrap_or(300);
            let h = arg(1).trim().parse::<u32>().unwrap_or(150);
            reg.contexts.insert(id, zero_canvas::CanvasContext::new(w, h));
            id.to_string()
        }
        "setFillStyle" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_fill_color(parse_canvas_color(arg(0)));
            }
            "ok".into()
        }
        "setStrokeStyle" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.set_stroke_color(parse_canvas_color(arg(0)));
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
                ctx.arc(f(0), f(1), f(2), f(3), f(4));
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
                ctx.fill_text(arg(0), f(1), f(2));
            }
            "ok".into()
        }
        "strokeText" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.fill_text(arg(0), f(1), f(2)); // canvas crate 无独立 stroke_text；近似 fill_text（headless 简化）
            }
            "ok".into()
        }
        "measureText" => {
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let m = ctx.measure_text(arg(0));
                // R3303：spec TextMetrics 全 10 字段 csv（width, actualBoxAscent/Descent/Left/Right,
                // fontBoxAscent/Descent, alphabetic/hanging/ideographicBaseline）。JS 构完整 TextMetrics。
                format!(
                    "{},{},{},{},{},{},{},{},{},{}",
                    m.width,
                    m.actual_bounding_box_ascent,
                    m.actual_bounding_box_descent,
                    m.actual_bounding_box_left,
                    m.actual_bounding_box_right,
                    m.font_bounding_box_ascent,
                    m.font_bounding_box_descent,
                    m.alphabetic_baseline,
                    m.hanging_baseline,
                    m.ideographic_baseline,
                )
            } else {
                // 无 ctx → 全 0（与既有 0 width 同语义）。
                "0,0,0,0,0,0,0,0,0,0".into()
            }
        }
        // fillRect：经 path（rasterize 到 pixel_buffer，绕过 fill_rect 便捷法不写 pixel_buffer 之限制）。
        "fillRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let (x, y, w, h) = (f(0), f(1), f(2), f(3));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + w, y);
                ctx.line_to(x + w, y + h);
                ctx.line_to(x, y + h);
                ctx.close_path();
                ctx.fill();
            }
            "ok".into()
        }
        "strokeRect" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let (x, y, w, h) = (f(0), f(1), f(2), f(3));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + w, y);
                ctx.line_to(x + w, y + h);
                ctx.line_to(x, y + h);
                ctx.close_path();
                ctx.stroke();
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
                let radii: Vec<f32> = arg(4)
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
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
                let (x, y, w, h) = (f(0), f(1), f(2), f(3));
                ctx.move_to(x, y);
                ctx.line_to(x + w, y);
                ctx.line_to(x + w, y + h);
                ctx.line_to(x, y + h);
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
                ctx.set_shadow_color(parse_canvas_color(arg(0)));
            }
            "ok".into()
        }
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
            let dx = arg(0).trim().parse::<u32>().unwrap_or(0);
            let dy = arg(1).trim().parse::<u32>().unwrap_or(0);
            let w = arg(2).trim().parse::<u32>().unwrap_or(0);
            let h = arg(3).trim().parse::<u32>().unwrap_or(0);
            let data: Vec<u8> = arg(4)
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                let img = zero_canvas::ImageData {
                    width: w,
                    height: h,
                    data,
                };
                ctx.put_image_data(&img, dx, dy);
            }
            "ok".into()
        }
        // ── drawImage 系列（R2799，canvas slice 5）：源 canvas → 本 ctx。host draw_image* 已存在且
        // 真写 pixel_buffer（draw_image_sized：最近邻采样 + transform + source-over alpha 混合 + global_alpha）。
        // args[0] = 源 ImageData wire（shim 经源 canvas getImageData 取），后续为目标几何。
        // **已知限制**：固定 source-over（不消费 globalCompositeOperation）；源限 canvas（img decode defer）。
        "drawImage" => {
            let img = parse_image_data_wire(arg(0));
            let (dx, dy) = (f(1), f(2));
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
                ctx.draw_image(&img, dx, dy);
            }
            "ok".into()
        }
        "drawImageScaled" => {
            let img = parse_image_data_wire(arg(0));
            let (dx, dy, dw, dh) = (f(1), f(2), f(3), f(4));
            if let Some(ctx) = reg.contexts.get_mut(&hid()) {
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
                let img = ctx.get_image_data(0, 0, w, h);
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
            let (x, y, w, h) = (
                arg(0).trim().parse::<u32>().unwrap_or(0),
                arg(1).trim().parse::<u32>().unwrap_or(0),
                arg(2).trim().parse::<u32>().unwrap_or(0),
                arg(3).trim().parse::<u32>().unwrap_or(0),
            );
            if let Some(ctx) = reg.contexts.get(&hid()) {
                let img = ctx.get_image_data(x, y, w, h);
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
            if let Some(style) = reg.gradients.get_mut(&gid) {
                style.add_color_stop(offset, parse_canvas_color(arg(2)));
            }
            "ok".into()
        }
        "setFillStyleGradient" => {
            let gid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&gid)) {
                ctx.set_fill_style(style.clone());
            }
            "ok".into()
        }
        "setStrokeStyleGradient" => {
            let gid = arg(0).trim().parse::<u64>().unwrap_or(0);
            if let (Some(ctx), Some(style)) = (reg.contexts.get_mut(&hid()), reg.gradients.get(&gid)) {
                ctx.set_stroke_style(style.clone());
            }
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
        "setFont" => {
            if let Some(ctx) = reg.contexts.get_mut(&hid())
                && let Some(fd) = zero_canvas::FontDescriptor::parse_css(arg(0))
            {
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
                let weight_str = match fd.weight {
                    zero_canvas::FontWeight::Bold => "bold ",
                    zero_canvas::FontWeight::Normal => "",
                };
                return format!("{}{}{}px {}", style_str, weight_str, fd.size, fd.family);
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
        _ => B::Alphabetic, // alphabetic + 非法值 → Alphabetic
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
