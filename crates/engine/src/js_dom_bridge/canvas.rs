//! Canvas 2D host 操作派发。从 js_dom_bridge.rs 拆出（R2974，文件大小治理 slice 2）。
//! `canvas_context_op(reg, handle, op, args)` 经 `__zw_canvas_op` 回调派发全部 Canvas 2D 操作
//!（路径/矩形/文本/图像/变换/合成/像素/状态），辅以值解析 helper（颜色/line-join/line-cap/
//! composite/image-data wire）。纯 zero_canvas/zero_render_foundation 类型，无 DOM/选择器依赖。
//! pub canvas_context_op 经 `pub use canvas::*` 重导出，register_dom_callbacks 调用点零改动。

use std::collections::HashMap;

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

/// canvas `lineCap` 串 → LineCap（spec: butt/round/square；未知回落 Butt = 默认）。
fn parse_line_cap(s: &str) -> zero_canvas::LineCap {
    match s.trim().to_ascii_lowercase().as_str() {
        "round" => zero_canvas::LineCap::Round,
        "square" => zero_canvas::LineCap::Square,
        _ => zero_canvas::LineCap::Butt,
    }
}

/// canvas `globalCompositeOperation` 串 → CompositeOperation（spec canonical 名；未知回落 SourceOver = 默认）。
/// **已知限制（记录）**：composite 仅经 `composite_pixel` 在 `blit_rect_to_pixels`（rect-fill 便捷法 + stroke 路径）
/// 生效；path-based `fill()`（`blit_path_to_pixels`）**不消费** composite——故 shim `fillRect`（path 实现）
/// 不受 composite 影响。getter/setter 状态往返真实（host 持 state，save/restore 含）。
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

/// `HTMLCanvasElement.getContext('2d')` 派发（R2795，canvas slice 1）。host 持 `CanvasContext` 注册表
///（`Arc<Mutex<(next_id, HashMap<id, CanvasContext>)>>`），JS 经 `__zw_canvas_op(handle, op, ...args)`
/// 串参派发（避免 JSON/serde 依赖）。**关键**：zero-canvas `fill_rect`/`stroke_rect` 便捷法**不写
/// pixel_buffer**（仅记 primitives），但 `fill()`/`stroke()`（path-based）经 `blit_path/stroke_to_pixels`
/// **写 pixel_buffer**——故 `fillRect` shim 经 beginPath+moveTo+lines+fill 实现（rasterize，getImageData
/// 可回读）。`getContext2d` 创建上下文返 id；`getImageData` 返 `"{w},{h};{r},{g},{b},{a},..."`。
/// 供 `__zw_canvas_op` 回调 → shim canvas element + CanvasRenderingContext2D proxy。
pub fn canvas_context_op(
    reg: &mut (u64, HashMap<u64, zero_canvas::CanvasContext>),
    handle: &str,
    op: &str,
    args: &[String],
) -> String {
    let arg = |i: usize| args.get(i).map(String::as_str).unwrap_or("0");
    let f = |i: usize| arg(i).trim().parse::<f32>().unwrap_or(0.0);
    let hid = || handle.trim().parse::<u64>().unwrap_or(0);
    match op {
        "getContext2d" => {
            let (next, ctxs) = reg;
            let id = *next;
            *next += 1;
            let w = arg(0).trim().parse::<u32>().unwrap_or(300);
            let h = arg(1).trim().parse::<u32>().unwrap_or(150);
            ctxs.insert(id, zero_canvas::CanvasContext::new(w, h));
            id.to_string()
        }
        "setFillStyle" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_fill_color(parse_canvas_color(arg(0)));
            }
            "ok".into()
        }
        "setStrokeStyle" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_stroke_color(parse_canvas_color(arg(0)));
            }
            "ok".into()
        }
        "setLineWidth" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_line_width(f(0));
            }
            "ok".into()
        }
        "beginPath" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.begin_path();
            }
            "ok".into()
        }
        "closePath" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.close_path();
            }
            "ok".into()
        }
        "moveTo" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.move_to(f(0), f(1));
            }
            "ok".into()
        }
        "lineTo" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.line_to(f(0), f(1));
            }
            "ok".into()
        }
        "arc" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.arc(f(0), f(1), f(2), f(3), f(4));
            }
            "ok".into()
        }
        "fill" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.fill();
            }
            "ok".into()
        }
        "stroke" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.stroke();
            }
            "ok".into()
        }
        // fillRect：经 path（rasterize 到 pixel_buffer，绕过 fill_rect 便捷法不写 pixel_buffer 之限制）。
        "fillRect" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
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
            if let Some(ctx) = reg.1.get_mut(&hid()) {
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
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.clear_rect(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        // ── slice 2：path 曲线 / 状态栈 / transforms / line 样式 / globalAlpha（R2796）──
        "quadraticCurveTo" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.quadratic_curve_to(f(0), f(1), f(2), f(3));
            }
            "ok".into()
        }
        "bezierCurveTo" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.bezier_curve_to(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        "ellipse" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.ellipse(f(0), f(1), f(2), f(3), f(4), f(5), f(6));
            }
            "ok".into()
        }
        "arcTo" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.arc_to(f(0), f(1), f(2), f(3), f(4));
            }
            "ok".into()
        }
        // rect 路径命令：CanvasContext 无 rect() 方法，用 MoveTo+3 LineTo（匹配 Path2D::rect，不 auto-close）。
        "rect" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                let (x, y, w, h) = (f(0), f(1), f(2), f(3));
                ctx.move_to(x, y);
                ctx.line_to(x + w, y);
                ctx.line_to(x + w, y + h);
                ctx.line_to(x, y + h);
            }
            "ok".into()
        }
        "clip" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.clip();
            }
            "ok".into()
        }
        "save" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.save();
            }
            "ok".into()
        }
        "restore" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.restore();
            }
            "ok".into()
        }
        "translate" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.translate(f(0), f(1));
            }
            "ok".into()
        }
        "rotate" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.rotate(f(0));
            }
            "ok".into()
        }
        "scale" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.scale(f(0), f(1));
            }
            "ok".into()
        }
        "setTransform" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_transform(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        "transform" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.transform(f(0), f(1), f(2), f(3), f(4), f(5));
            }
            "ok".into()
        }
        // R2985 getTransform：返当前 2D 变换矩阵 "a,b,c,d,e,f"（shim 包 DOMMatrix）。只读（get_transform
        // 取 &self），无 ctx → identity "1,0,0,1,0,0"。Canvas 2D spec getTransform() → DOMMatrix。
        "getTransform" => {
            if let Some(ctx) = reg.1.get(&hid()) {
                let t = ctx.get_transform();
                return format!("{},{},{},{},{},{}", t.a, t.b, t.c, t.d, t.e, t.f);
            }
            "1,0,0,1,0,0".into()
        }
        // R2985 resetTransform：重置为单位矩阵（spec setTransform(identity)）。
        "resetTransform" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.reset_transform();
            }
            "ok".into()
        }
        "setGlobalAlpha" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_global_alpha(f(0));
            }
            "ok".into()
        }
        "setLineDash" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
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
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_line_join(parse_line_join(arg(0)));
            }
            "ok".into()
        }
        "setLineCap" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_line_cap(parse_line_cap(arg(0)));
            }
            "ok".into()
        }
        // ── slice 4：globalCompositeOperation / shadow / putImageData（R2798）──
        // composite 状态真实（host 持 state，save/restore 含）；effect 仅经 composite_pixel 在 rect-blit/stroke
        // 生效，path-based fill 不消费（见 parse_composite_operation 注释）。
        "setCompositeOperation" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_composite_operation(parse_composite_operation(arg(0)));
            }
            "ok".into()
        }
        "setShadowColor" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_shadow_color(parse_canvas_color(arg(0)));
            }
            "ok".into()
        }
        "setShadowBlur" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_shadow_blur(f(0));
            }
            "ok".into()
        }
        "setShadowOffsetX" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.set_shadow_offset_x(f(0));
            }
            "ok".into()
        }
        "setShadowOffsetY" => {
            if let Some(ctx) = reg.1.get_mut(&hid()) {
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
            if let Some(ctx) = reg.1.get_mut(&hid()) {
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
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.draw_image(&img, dx, dy);
            }
            "ok".into()
        }
        "drawImageScaled" => {
            let img = parse_image_data_wire(arg(0));
            let (dx, dy, dw, dh) = (f(1), f(2), f(3), f(4));
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.draw_image_with_size(&img, dx, dy, dw, dh);
            }
            "ok".into()
        }
        "drawImageSliced" => {
            let img = parse_image_data_wire(arg(0));
            let (sx, sy, sw, sh, dx, dy, dw, dh) = (f(1), f(2), f(3), f(4), f(5), f(6), f(7), f(8));
            if let Some(ctx) = reg.1.get_mut(&hid()) {
                ctx.draw_image_sliced(&img, sx, sy, sw, sh, dx, dy, dw, dh);
            }
            "ok".into()
        }
        // toDataURL（R2797，canvas slice 3）：pixel_buffer（经 get_image_data 取全 RGBA）→ PNG 编码 →
        // 返**逗号分隔十进制串**（shim 转 Latin-1 → btoa → `data:image/png;base64,...`）。复用 png crate
        //（miniz_oxide 已 transitive）；编码失败返空串（shim 回落 `data:,`）。仅 'image/png'（jpeg/webp defer）。
        "toDataURL" => {
            if let Some(ctx) = reg.1.get(&hid()) {
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
            if let Some(ctx) = reg.1.get(&hid()) {
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
        _ => "ok".into(),
    }
}
