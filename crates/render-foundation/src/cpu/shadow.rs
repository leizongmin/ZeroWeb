//! 渲染阴影图元 — box-shadow 高斯模糊近似。

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::geometry::Rect;
use crate::primitive::ShadowPrimitive;
use crate::surface::FrameBuffer;

/// region 重放入口：足迹与 region 不相交的阴影整体跳过，相交者原样全量渲染。
///
/// 条带/包围盒 scratch 重放只把 region 内像素拷回宿主帧——阴影（整页 rect +
/// 3σ blur 外扩）此前在区域重放中仍全幅渲染，三遍 box-blur 在软光栅下成为
/// 重放主成本（T28-P37 实测 ~140ms/次，渐入期 16ms Poll 心跳被渲染阻塞饿死，
/// 菜单渐入只出 2 帧）。
///
/// 输出等价性只允许「整影跳过 / 原样渲染」二值选择：alpha 蒙版域 = 墨迹足迹
/// ∩ clip 窗口，三遍 box_blur 以该域为卷积域（定长核窗口截断、零填充），把
/// region 交集写进 clip 会缩小卷积域并改变域内像素值（T28-P37 R2 双审查实测
/// region 切过渐变时拷回行通道差达 45–108，TV1/D1），故不用 clip 窗口法。
/// 保守足迹 = rect ± (3σ + |spread| + |offset| + slack)，与 R4059 clip 窗口
/// （窗口外硬停）及 fb 边界求交；足迹与 region 不相交 ⇒ [`render_shadow`]
/// 必不写 region 行，跳过输出等价；相交则原参数渲染，卷积域与全帧一致，
/// 拷回范围逐位一致。inset 阴影墨迹限于盒内，同足迹只会多画不会错画。
pub fn render_shadow_for_region(fb: &mut FrameBuffer, shadow: &ShadowPrimitive, scale: f32, region: Option<Rect>) {
    let Some(r) = region else {
        render_shadow(fb, shadow, scale);
        return;
    };
    let grow = shadow.blur_radius * scale * 3.0
        + shadow.spread_radius.abs() * scale
        + shadow.offset_x.abs().max(shadow.offset_y.abs()) * scale
        + 2.0;
    let mut bound = Rect::new(
        shadow.rect.origin.x * scale - grow,
        shadow.rect.origin.y * scale - grow,
        shadow.rect.size.width * scale + grow * 2.0,
        shadow.rect.size.height * scale + grow * 2.0,
    );
    if let Some(c) = shadow.clip {
        bound = match bound.intersection(&c) {
            Some(intersected) => intersected,
            None => return,
        };
    }
    let (bx0, by0) = (bound.origin.x.max(0.0), bound.origin.y.max(0.0));
    let (bx1, by1) = (
        (bound.origin.x + bound.size.width).min(fb.width as f32),
        (bound.origin.y + bound.size.height).min(fb.height as f32),
    );
    if bx1 <= bx0 || by1 <= by0 {
        return;
    }
    if !Rect::new(bx0, by0, bx1 - bx0, by1 - by0).intersects(&r) {
        return;
    }
    render_shadow(fb, shadow, scale);
}

/// T28-P37 R2 返修：outset 阴影 alpha 蒙版内容寻址缓存。
///
/// 蒙版（初始 alpha → 三遍 box-blur → R4139 punch-out）是图元内容 + scale + fb 尺寸
/// 的纯函数，与 dst 像素、重放 region 无关；三遍 box_blur 是软光栅阴影主成本
/// （T28-P37 实测整页框阴影 ~140ms/次），渐入期菜单 bbox 逐帧重放（条带跳过、bbox
/// 每帧全量）会对恒定的页框阴影反复重算——r1 的 clip 窗口法以改写卷积域换速度
/// （TV1/D1 违规）被回退后，此处以记忆化在不改任何算术的前提下消除重复：首次构建
/// 走与旧实现逐行相同的代码，命中返回同一 `Arc<[f32]>`，合成逐像素算术不变 ⇒
/// 输出与不缓存逐位一致（冻结 oracle 与 region 重放字节一致契约双重满足）。键用
/// 位型表示（f32 参与 Hash/Eq 须 `to_bits`；NaN/-0.0 各自成键只影响命中率）。
/// 预算内全留、超限整体清空——淘汰只损命中不损输出；thread_local：软光栅渲染单
/// 线程执行，免同步原语。
const SHADOW_MASK_CACHE_BYTE_BUDGET: usize = 32 * 1024 * 1024;

#[derive(Clone, PartialEq, Eq, Hash)]
struct ShadowMaskKey {
    rect: [u32; 4],
    color: [u8; 4],
    offset: [u32; 2],
    spread: u32,
    blur: u32,
    clip: Option<[u32; 4]>,
    clip_out: Option<[u32; 4]>,
    scale: u32,
    fb: [u32; 2],
}

thread_local! {
    static SHADOW_MASK_CACHE: RefCell<HashMap<ShadowMaskKey, Arc<[f32]>>> =
        RefCell::new(HashMap::new());
}

/// outset 阴影几何（device 坐标）：σ blur 半径与 spread 扩展 + offset 偏移后的矩形。
/// [`render_shadow`] 的 area 计算与 [`build_outset_shadow_mask`] 的初始 alpha 共用
/// 同一表达式，保证缓存键与蒙版内容对同一图元恒一致。
fn outset_shadow_geometry(shadow: &ShadowPrimitive, scale: f32) -> (f32, f32, f32, f32, f32) {
    let blur_r = shadow.blur_radius * scale;
    let spread = shadow.spread_radius * scale;
    let sx = shadow.rect.left() * scale - spread + shadow.offset_x * scale;
    let sy = shadow.rect.top() * scale - spread + shadow.offset_y * scale;
    let sw = shadow.rect.size.width * scale + spread * 2.0;
    let sh = shadow.rect.size.height * scale + spread * 2.0;
    (sx, sy, sw, sh, blur_r)
}

/// 取（或构建并缓存）outset 阴影 alpha 蒙版；`area_*` 为调用方按图元与 fb 算出的
/// 蒙版域（键相同时域必相同，故不入键）。
fn outset_shadow_mask(
    shadow: &ShadowPrimitive,
    scale: f32,
    fb: &FrameBuffer,
    area_left: u32,
    area_top: u32,
    area_w: usize,
    area_h: usize,
) -> Arc<[f32]> {
    let rect_bits = |r: Rect| {
        [
            r.origin.x.to_bits(),
            r.origin.y.to_bits(),
            r.size.width.to_bits(),
            r.size.height.to_bits(),
        ]
    };
    let key = ShadowMaskKey {
        rect: rect_bits(shadow.rect),
        color: [shadow.color.r, shadow.color.g, shadow.color.b, shadow.color.a],
        offset: [shadow.offset_x.to_bits(), shadow.offset_y.to_bits()],
        spread: shadow.spread_radius.to_bits(),
        blur: shadow.blur_radius.to_bits(),
        clip: shadow.clip.map(rect_bits),
        clip_out: shadow.clip_out.map(rect_bits),
        scale: scale.to_bits(),
        fb: [fb.width, fb.height],
    };
    if let Some(mask) = SHADOW_MASK_CACHE.with(|cache| cache.borrow().get(&key).cloned()) {
        return mask;
    }
    let mask: Arc<[f32]> = build_outset_shadow_mask(shadow, scale, area_left, area_top, area_w, area_h).into();
    SHADOW_MASK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let new_bytes = mask.len() * std::mem::size_of::<f32>();
        let total: usize = cache.values().map(|m| m.len() * std::mem::size_of::<f32>()).sum();
        if total + new_bytes > SHADOW_MASK_CACHE_BYTE_BUDGET {
            cache.clear();
        }
        cache.insert(key, Arc::clone(&mask));
    });
    mask
}

/// 构建 outset 蒙版：初始 alpha → 三遍 box-blur → R4139 punch-out（自
/// [`render_shadow`] 原样移入，算术逐行不变）。
fn build_outset_shadow_mask(
    shadow: &ShadowPrimitive,
    scale: f32,
    area_left: u32,
    area_top: u32,
    area_w: usize,
    area_h: usize,
) -> Vec<f32> {
    let (sx, sy, sw, sh, blur_r) = outset_shadow_geometry(shadow, scale);
    let area_right = area_left + area_w as u32;
    let area_bottom = area_top + area_h as u32;

    let mut alpha_mask = vec![0.0f32; area_w * area_h];
    let shadow_alpha = shadow.color.a as f32 / 255.0;

    // 对阴影矩形区域内的像素设置初始 alpha
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            if fx >= sx && fx <= sx + sw && fy >= sy && fy <= sy + sh {
                let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
                alpha_mask[idx] = shadow_alpha;
            }
        }
    }

    // 三遍 box-blur（近似高斯模糊）
    // CSS 规范：box-shadow blur_radius 对应高斯标准差 σ = blur_radius / 2（而非
    // blur_radius 本身）。三遍等半径 box-blur（每遍半宽 r）合成 σ 满足
    // σ² ≈ ((2r+1)² - 1) / 4。由 σ 反解连续半宽 d = (sqrt(4σ²+1) - 1)/2，再按 d
    // 的小数部分把 3 遍在 floor/ceil 半宽间分配（m 遍用 ceil、3-m 遍用 floor），
    // 精确逼近目标 σ。旧实现直接用 radius=blur_r.ceil() 致 σ 偏大约 2.3 倍，
    // 阴影扩散过远（welcome.html 卡片 box-shadow:0 1px 3px 实测扩散 12px）。
    if blur_r > 0.5 {
        let sigma = blur_r * 0.5;
        let d = ((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5;
        let r_lo = d.floor() as usize;
        let r_hi = r_lo + 1;
        let m = ((d - r_lo as f32) * 3.0).round().clamp(0.0, 3.0) as usize;
        for _ in 0..(3 - m) {
            if r_lo > 0 {
                box_blur(&mut alpha_mask, area_w, area_h, r_lo);
            }
        }
        for _ in 0..m {
            box_blur(&mut alpha_mask, area_w, area_h, r_hi);
        }
    }

    // R4139（CSS Backgrounds §7.1）：outer shadow「is drawn outside the border
    // edge only: it is clipped inside the border-box of the element」——`clip_out`
    // （paint 侧写入的元素 border-box）非 None 时，模糊后把该区域 alpha 清零。
    // 旧实现无 punch-out：透明 bg/border 的元素下方阴影透出（outset-without-
    // border-radius-001 的 L 形阴影带应为盒外部分，盒下透黑）。图元级 `rect` 仍是
    // 阴影自身形状（低层契约不变），punch-out 只作用于显式携带的区域。
    if let Some(clip_out) = shadow.clip_out {
        let bx0 = clip_out.left() * scale;
        let by0 = clip_out.top() * scale;
        let bx1 = bx0 + clip_out.size.width * scale;
        let by1 = by0 + clip_out.size.height * scale;
        if bx1 > bx0 && by1 > by0 {
            for y in area_top..area_bottom {
                let fy = y as f32 + 0.5;
                if fy < by0 || fy > by1 {
                    continue;
                }
                for x in area_left..area_right {
                    let fx = x as f32 + 0.5;
                    if fx < bx0 || fx > bx1 {
                        continue;
                    }
                    let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
                    alpha_mask[idx] = 0.0;
                }
            }
        }
    }

    alpha_mask
}

/// 渲染 box-shadow。
///
/// 使用三遍 box-blur 近似高斯模糊：
/// 1. 计算阴影矩形（原始矩形 + spread + offset）
/// 2. 对阴影区域进行模糊（alpha 蒙版内容寻址缓存，见 [`outset_shadow_mask`]）
/// 3. 将模糊结果合成到帧缓冲
pub fn render_shadow(fb: &mut FrameBuffer, shadow: &ShadowPrimitive, scale: f32) {
    // R2476：inset 内阴影走单独路径（frame 形状 = box 减 offset+spread 收缩的洞，向内模糊，
    // 裁切到盒）；outset 走既有路径。
    if shadow.inset {
        render_inset_shadow(fb, shadow, scale);
        return;
    }
    let (sx, sy, sw, sh, blur_r) = outset_shadow_geometry(shadow, scale);

    if sw <= 0.0 || sh <= 0.0 {
        return;
    }

    // 计算需要渲染的区域（包含模糊扩展）
    let blur_extent = blur_r * 3.0; // 3σ 覆盖 99.7% 的高斯分布
    let area_left = (sx - blur_extent).floor().max(0.0) as u32;
    let area_top = (sy - blur_extent).floor().max(0.0) as u32;
    let area_right = (sx + sw + blur_extent).ceil().min(fb.width as f32) as u32;
    let area_bottom = (sy + sh + blur_extent).ceil().min(fb.height as f32) as u32;

    // R4059：裁剪窗口（paint 侧 overflow / contain:paint 裁剪写入）——blur 溢出 rect
    // 的部分须在窗口边界硬停（css-contain-1 §paint containment：后代绘制裁到 padding
    // 盒；overflow:hidden 同语义）。模糊在窗口外不渲染，合成阶段同样跳过窗口外像素。
    let clip_window = shadow
        .clip
        .map(|c| {
            (
                (c.left() * scale).floor().max(0.0),
                (c.top() * scale).floor().max(0.0),
                (c.right() * scale).ceil(),
                (c.bottom() * scale).ceil(),
            )
        })
        .unwrap_or((0.0, 0.0, fb.width as f32, fb.height as f32));
    let (cl, ct, cr, cb) = clip_window;
    let area_left = (area_left as f32).max(cl).floor() as u32;
    let area_top = (area_top as f32).max(ct).floor() as u32;
    let area_right = (area_right as f32).min(cr).ceil().min(fb.width as f32) as u32;
    let area_bottom = (area_bottom as f32).min(cb).ceil().min(fb.height as f32) as u32;

    if area_left >= area_right || area_top >= area_bottom {
        return;
    }

    let area_w = (area_right - area_left) as usize;
    let area_h = (area_bottom - area_top) as usize;

    if area_w == 0 || area_h == 0 {
        return;
    }

    // 步骤 1+2：生成（或命中缓存）阴影 alpha 蒙版——初始 alpha、三遍 box-blur 与
    // R4139 punch-out 只依赖图元、scale 与 fb 尺寸，与 dst、重放 region 无关。
    let alpha_mask = outset_shadow_mask(shadow, scale, fb, area_left, area_top, area_w, area_h);

    // 步骤 3：合成到帧缓冲
    let shadow_color = [shadow.color.r as f32, shadow.color.g as f32, shadow.color.b as f32];
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
            let alpha = alpha_mask[idx];

            if alpha < 0.001 {
                continue;
            }

            let dst = fb.get_pixel(x, y);
            let inv_a = 1.0 - alpha;
            let r = shadow_color[0] * alpha + dst[0] as f32 * inv_a;
            let g = shadow_color[1] * alpha + dst[1] as f32 * inv_a;
            let b = shadow_color[2] * alpha + dst[2] as f32 * inv_a;
            fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
        }
    }
}

/// R2476：渲染 inset（内）box-shadow（CSS Backgrounds §7.1）。
///
/// 内阴影在盒内：alpha 蒙版 = 盒（OUTER）内、洞（OUTER 经 offset 偏移 + spread 收缩）
/// 外的区域（frame 形状）。三遍 box-blur 软化洞边界（阴影向内淡出）；OUTER 边界因 area
/// 恰为盒、向外模糊被裁切而保持硬边。合成（source-over）裁切到盒。
fn render_inset_shadow(fb: &mut FrameBuffer, shadow: &ShadowPrimitive, scale: f32) {
    let blur_r = shadow.blur_radius * scale;
    let spread = shadow.spread_radius * scale;

    // OUTER = 盒（缩放后绝对坐标）
    let ox = shadow.rect.left() * scale;
    let oy = shadow.rect.top() * scale;
    let ow = shadow.rect.size.width * scale;
    let oh = shadow.rect.size.height * scale;
    if ow <= 0.0 || oh <= 0.0 {
        return;
    }

    // 洞：OUTER 经 offset 偏移 + spread 收缩（inset 正 spread 向内增长阴影 → 洞缩小）。
    let hx = ox + shadow.offset_x * scale + spread;
    let hy = oy + shadow.offset_y * scale + spread;
    let hw = ow - 2.0 * spread;
    let hh = oh - 2.0 * spread;

    // area = OUTER（内阴影限制在盒内）裁切到帧缓冲；R4059 裁剪窗口再收窄
    let area_left = ox.floor().max(0.0) as u32;
    let area_top = oy.floor().max(0.0) as u32;
    let area_right = (ox + ow).ceil().min(fb.width as f32) as u32;
    let area_bottom = (oy + oh).ceil().min(fb.height as f32) as u32;
    let (cl, ct, cr, cb) = shadow
        .clip
        .map(|c| {
            (
                (c.left() * scale).floor().max(0.0),
                (c.top() * scale).floor().max(0.0),
                (c.right() * scale).ceil(),
                (c.bottom() * scale).ceil(),
            )
        })
        .unwrap_or((0.0, 0.0, fb.width as f32, fb.height as f32));
    let area_left = (area_left as f32).max(cl).floor() as u32;
    let area_top = (area_top as f32).max(ct).floor() as u32;
    let area_right = (area_right as f32).min(cr).ceil().min(fb.width as f32) as u32;
    let area_bottom = (area_bottom as f32).min(cb).ceil().min(fb.height as f32) as u32;
    if area_left >= area_right || area_top >= area_bottom {
        return;
    }
    let area_w = (area_right - area_left) as usize;
    let area_h = (area_bottom - area_top) as usize;
    if area_w == 0 || area_h == 0 {
        return;
    }

    let shadow_alpha = shadow.color.a as f32 / 255.0;
    let mut alpha_mask = vec![0.0f32; area_w * area_h];

    // alpha = shadow_alpha 在 frame 区（OUTER 内且洞外）；洞退化（hw/hh<=0）→ 全盒阴影。
    let hole_degenerate = hw <= 0.0 || hh <= 0.0;
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let outside_hole = hole_degenerate || fx < hx || fx > hx + hw || fy < hy || fy > hy + hh;
            if outside_hole {
                let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
                alpha_mask[idx] = shadow_alpha;
            }
        }
    }

    // 三遍 box-blur（软化洞边界向内淡出）。R2476：用 box_blur_inset（正确 2D-clamp 实现），
    // 非 outset 的 box_blur——后者滑动窗口 init 对非均匀 mask（frame 形状）有 double-count
    // 残差泄漏（对 outset 均匀矩形 mask 良性，对 inset frame 致洞内残余 alpha）。
    if blur_r > 0.5 {
        let sigma = blur_r * 0.5;
        let d = ((4.0 * sigma * sigma + 1.0).sqrt() - 1.0) * 0.5;
        let r_lo = d.floor() as usize;
        let r_hi = r_lo + 1;
        let m = ((d - r_lo as f32) * 3.0).round().clamp(0.0, 3.0) as usize;
        for _ in 0..(3 - m) {
            if r_lo > 0 {
                box_blur_inset(&mut alpha_mask, area_w, area_h, r_lo);
            }
        }
        for _ in 0..m {
            box_blur_inset(&mut alpha_mask, area_w, area_h, r_hi);
        }
    }

    // 合成（source-over），裁切到盒（area=OUTER 自动满足）。
    let sc = [shadow.color.r as f32, shadow.color.g as f32, shadow.color.b as f32];
    for y in area_top..area_bottom {
        for x in area_left..area_right {
            let idx = (y - area_top) as usize * area_w + (x - area_left) as usize;
            let alpha = alpha_mask[idx];
            if alpha < 0.001 {
                continue;
            }
            let dst = fb.get_pixel(x, y);
            let inv_a = 1.0 - alpha;
            let r = sc[0] * alpha + dst[0] as f32 * inv_a;
            let g = sc[1] * alpha + dst[1] as f32 * inv_a;
            let b = sc[2] * alpha + dst[2] as f32 * inv_a;
            fb.set_pixel(x, y, [r.round() as u8, g.round() as u8, b.round() as u8, 255]);
        }
    }
}

/// R2476：单遍 box-blur（2D，边缘 clamp）—— inset 内阴影专用。
///
/// 与 outset 的 `box_blur`（可分离滑动窗口）不同：后者对均匀矩形 mask 良性，但滑动窗口
/// init 对非均匀 frame mask 有 double-count 残差泄漏。本函数直接逐像素求窗口均值（窗口
/// 越界 clamp 到边缘 → OUTER 边保持硬边，洞边界向内淡出），小 mask（盒内）+ 小 radius 成本可接受。
fn box_blur_inset(data: &mut [f32], width: usize, height: usize, radius: usize) {
    if width == 0 || height == 0 || radius == 0 {
        return;
    }
    let src = data.to_vec();
    for y in 0..height {
        let y0 = y.saturating_sub(radius);
        let y1 = (y + radius).min(height - 1);
        for x in 0..width {
            let x0 = x.saturating_sub(radius);
            let x1 = (x + radius).min(width - 1);
            let mut sum = 0.0f32;
            let mut cnt = 0usize;
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    sum += src[yy * width + xx];
                    cnt += 1;
                }
            }
            data[y * width + x] = if cnt > 0 { sum / cnt as f32 } else { 0.0 };
        }
    }
}

/// 单遍 box-blur — 对 alpha 蒙版进行水平+垂直模糊。
fn box_blur(data: &mut [f32], width: usize, height: usize, radius: usize) {
    if width == 0 || height == 0 || radius == 0 {
        return;
    }

    let kernel_size = radius * 2 + 1;
    let inv_size = 1.0 / kernel_size as f32;

    // 水平模糊
    let mut temp = vec![0.0f32; data.len()];
    for y in 0..height {
        let mut sum = 0.0;
        // 初始化窗口
        for dx in 0..=radius.min(width - 1) {
            sum += data[y * width + dx];
        }
        // 如果左边界外有值，用边界值填充
        for _ in 0..radius.saturating_sub(0) {
            // 已在初始化中处理
        }

        for x in 0..width {
            let right = x + radius;
            let left = if x > radius { x - radius - 1 } else { 0 };

            if right < width {
                sum += data[y * width + right];
            }
            if x > radius {
                sum -= data[y * width + left];
            }

            temp[y * width + x] = sum * inv_size;
        }
    }

    // 垂直模糊
    for x in 0..width {
        let mut sum = 0.0;
        for dy in 0..=radius.min(height - 1) {
            sum += temp[dy * width + x];
        }

        for y in 0..height {
            let bottom = y + radius;
            let top = if y > radius { y - radius - 1 } else { 0 };

            if bottom < height {
                sum += temp[bottom * width + x];
            }
            if y > radius {
                sum -= temp[top * width + x];
            }

            data[y * width + x] = sum * inv_size;
        }
    }
}
